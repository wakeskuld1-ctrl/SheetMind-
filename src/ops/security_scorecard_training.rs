use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use thiserror::Error;

use crate::ops::stock::security_decision_evidence_bundle::{
    CROSS_BORDER_ETF_PROXY_FIELDS, CROSS_BORDER_ETF_PROXY_NUMERIC_FIELDS, EQUITY_ETF_PROXY_FIELDS,
    EQUITY_ETF_PROXY_NUMERIC_FIELDS, GOLD_ETF_PROXY_FIELDS, GOLD_ETF_PROXY_NUMERIC_FIELDS,
    SecurityExternalProxyInputs, TREASURY_ETF_PROXY_FIELDS, TREASURY_ETF_PROXY_NUMERIC_FIELDS,
    required_etf_feature_family, resolve_etf_subscope,
};
use crate::ops::stock::security_forward_outcome::{
    SecurityForwardOutcomeError, SecurityForwardOutcomeRequest, security_forward_outcome,
};
use crate::ops::stock::security_scorecard::{
    SecurityScorecardModelArtifact, SecurityScorecardModelBin, SecurityScorecardModelFeatureSpec,
    normalize_integrated_stance_for_modeling,
};
use crate::ops::stock::security_scorecard_model_registry::{
    SecurityScorecardCandidateArtifactInput, SecurityScorecardModelRegistry, sanitize_identifier,
};
use crate::ops::stock::security_scorecard_refit_run::{
    SecurityScorecardRefitError, SecurityScorecardRefitRequest, SecurityScorecardRefitRun,
    security_scorecard_refit,
};
use crate::runtime::stock_history_store::{StockHistoryStore, StockHistoryStoreError};

// 2026-04-09 CST: 这里新增正式训练入口请求合同，原因是 Task 5 需要把离线训练从临时脚本提升为可治理的一等 Tool；
// 目的：集中冻结市场范围、样本范围、目标头与运行时路径，避免训练参数散落在 Skill 或 CLI 外层。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityScorecardTrainingRequest {
    #[serde(default = "default_created_at")]
    pub created_at: String,
    #[serde(default)]
    pub training_runtime_root: Option<String>,
    pub market_scope: String,
    pub instrument_scope: String,
    pub symbol_list: Vec<String>,
    #[serde(default)]
    pub market_symbol: Option<String>,
    #[serde(default)]
    pub sector_symbol: Option<String>,
    #[serde(default)]
    pub market_profile: Option<String>,
    #[serde(default)]
    pub sector_profile: Option<String>,
    #[serde(default)]
    pub instrument_subscope: Option<String>,
    pub horizon_days: usize,
    pub target_head: String,
    pub train_range: String,
    pub valid_range: String,
    pub test_range: String,
    // 2026-04-11 CST: Add governed per-symbol sample targets, because Scheme B
    // needs the training pool size to be explicit and adjustable instead of being
    // hard-coded to a toy-sized split inside the collector.
    // Purpose: let training runs scale the sample pool while keeping the request
    // contract auditable and reproducible.
    #[serde(default = "default_train_samples_per_symbol")]
    pub train_samples_per_symbol: usize,
    #[serde(default = "default_valid_samples_per_symbol")]
    pub valid_samples_per_symbol: usize,
    #[serde(default = "default_test_samples_per_symbol")]
    pub test_samples_per_symbol: usize,
    pub feature_set_version: String,
    pub label_definition_version: String,
    #[serde(default = "default_lookback_days")]
    pub lookback_days: usize,
    #[serde(default = "default_disclosure_limit")]
    pub disclosure_limit: usize,
    #[serde(default = "default_stop_loss_pct")]
    pub stop_loss_pct: f64,
    #[serde(default = "default_target_return_pct")]
    pub target_return_pct: f64,
    #[serde(default)]
    pub external_proxy_inputs: Option<SecurityExternalProxyInputs>,
}

// 2026-04-09 CST: 这里定义训练入口聚合返回对象，原因是调用方不仅要拿到 artifact，还要拿到 refit_run 和 registry；
// 目的：让后续 package、回算和审计链可以在一次调用后继续消费正式治理输出，而不是重新拼接路径。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SecurityScorecardTrainingResult {
    pub artifact: SecurityScorecardModelArtifact,
    pub artifact_path: String,
    pub refit_run: SecurityScorecardRefitRun,
    pub model_registry: SecurityScorecardModelRegistry,
    pub refit_run_path: String,
    pub model_registry_path: String,
    pub metrics_summary_json: Value,
}

// 2026-04-09 CST: 这里集中定义训练入口错误边界，原因是 Task 5 同时覆盖样本采集、分箱建模、落盘与 refit 接线；
// 目的：向 dispatcher 暴露稳定、可定位的错误语义，避免把底层失败原样泄漏到外层。
#[derive(Debug, Error)]
pub enum SecurityScorecardTrainingError {
    #[error("security scorecard training build failed: {0}")]
    Build(String),
    #[error("security scorecard training history loading failed: {0}")]
    History(#[from] StockHistoryStoreError),
    #[error("security scorecard training outcome loading failed: {0}")]
    Outcome(#[from] SecurityForwardOutcomeError),
    #[error("security scorecard training persist failed: {0}")]
    Persist(String),
    #[error("security scorecard training refit failed: {0}")]
    Refit(#[from] SecurityScorecardRefitError),
}

#[derive(Debug, Clone, PartialEq)]
struct TrainingDateRange {
    start: NaiveDate,
    end: NaiveDate,
}

#[derive(Debug, Clone, PartialEq)]
struct TrainingSample {
    symbol: String,
    as_of_date: String,
    split_name: String,
    label: f64,
    forward_return: f64,
    max_drawdown: f64,
    max_runup: f64,
    hit_upside_first: bool,
    hit_stop_first: bool,
    feature_values: BTreeMap<String, TrainingFeatureValue>,
}

#[derive(Debug, Clone, PartialEq)]
enum TrainingFeatureKind {
    Numeric,
    Categorical,
}

#[derive(Debug, Clone, PartialEq)]
enum TrainingFeatureValue {
    Numeric(f64),
    Category(String),
}

#[derive(Debug, Clone, PartialEq)]
struct TrainingFeatureConfig {
    feature_name: &'static str,
    group_name: &'static str,
    kind: TrainingFeatureKind,
}

#[derive(Debug, Clone, PartialEq)]
struct FeatureModel {
    feature_name: String,
    group_name: String,
    kind: TrainingFeatureKind,
    bins: Vec<FeatureBinModel>,
}

#[derive(Debug, Clone, PartialEq)]
struct FeatureBinModel {
    bin_label: String,
    match_values: Vec<String>,
    min_inclusive: Option<f64>,
    max_exclusive: Option<f64>,
    woe: f64,
    predicted_value: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
struct TrainedLogisticModel {
    intercept: f64,
    coefficients: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq)]
struct TrainedRegressionModel {
    baseline: f64,
}

#[derive(Debug, Clone, PartialEq)]
enum TrainedHeadModel {
    Classification(TrainedLogisticModel),
    Regression(TrainedRegressionModel),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrainingTargetMode {
    Classification,
    Regression,
}

// 2026-04-09 CST: 这里实现 Task 5 的最小正式训练入口，原因是我们需要先把训练主链跑通，再继续做回算重估和晋级治理；
// 目的：以最小的“样本采集 -> 分箱 -> WOE -> logistic -> artifact -> refit”闭环承接现有 scorecard 体系。
pub fn security_scorecard_training(
    request: &SecurityScorecardTrainingRequest,
) -> Result<SecurityScorecardTrainingResult, SecurityScorecardTrainingError> {
    validate_request(request)?;

    let train_range = parse_date_range(&request.train_range)?;
    let valid_range = parse_date_range(&request.valid_range)?;
    let test_range = parse_date_range(&request.test_range)?;
    let instrument_subscope = resolve_request_instrument_subscope(request);
    let feature_configs = training_feature_configs_for_instrument(
        &request.instrument_scope,
        instrument_subscope.as_deref(),
    );
    let samples = collect_samples(
        request,
        &train_range,
        &valid_range,
        &test_range,
        &feature_configs,
    )?;
    let train_samples = samples_for_split(&samples, "train");
    if train_samples.len() < 2 {
        return Err(SecurityScorecardTrainingError::Build(
            "train split does not contain enough samples".to_string(),
        ));
    }
    let target_mode = target_mode_for_head(&request.target_head)?;
    validate_train_split_for_target(&train_samples, target_mode)?;

    let feature_models = build_feature_models(&train_samples, &feature_configs, target_mode)?;
    let trained_model = match target_mode {
        TrainingTargetMode::Classification => {
            let train_matrix = encode_samples(&train_samples, &feature_models)?;
            TrainedHeadModel::Classification(train_logistic_model(&train_matrix))
        }
        TrainingTargetMode::Regression => {
            TrainedHeadModel::Regression(train_regression_head_model(&train_samples))
        }
    };
    let artifact = build_artifact(
        request,
        &feature_models,
        &trained_model,
        instrument_subscope.as_deref(),
    );

    let runtime_root = resolve_runtime_root(request);
    let artifact_path = runtime_root.join("scorecard_artifacts").join(format!(
        "{}__{}.json",
        sanitize_identifier(&artifact.model_id),
        sanitize_identifier(&artifact.model_version)
    ));
    persist_json(&artifact_path, &artifact)?;

    let metrics_summary_json = build_metrics_summary(
        &samples,
        &feature_models,
        &trained_model,
        target_mode,
        &request.target_head,
    );
    let refit_result = security_scorecard_refit(&SecurityScorecardRefitRequest {
        created_at: request.created_at.clone(),
        refit_runtime_root: Some(runtime_root.to_string_lossy().to_string()),
        market_scope: request.market_scope.clone(),
        instrument_scope: request.instrument_scope.clone(),
        feature_set_version: request.feature_set_version.clone(),
        label_definition_version: request.label_definition_version.clone(),
        train_range: request.train_range.clone(),
        valid_range: request.valid_range.clone(),
        test_range: request.test_range.clone(),
        candidate_artifact: SecurityScorecardCandidateArtifactInput {
            model_id: artifact.model_id.clone(),
            model_version: artifact.model_version.clone(),
            horizon_days: request.horizon_days,
            target_head: request.target_head.clone(),
            status: "candidate".to_string(),
            artifact_path: artifact_path.to_string_lossy().to_string(),
            metrics_summary_json: metrics_summary_json.clone(),
            published_at: Some(request.created_at.clone()),
            instrument_subscope: instrument_subscope.clone(),
            model_grade: "candidate".to_string(),
            grade_reason: "retained_as_candidate".to_string(),
        },
        comparison_to_champion_json: None,
        promotion_decision: Some("candidate_only".to_string()),
    })?;

    Ok(SecurityScorecardTrainingResult {
        artifact,
        artifact_path: artifact_path.to_string_lossy().to_string(),
        refit_run: refit_result.refit_run,
        model_registry: refit_result.model_registry,
        refit_run_path: refit_result.refit_run_path,
        model_registry_path: refit_result.model_registry_path,
        metrics_summary_json,
    })
}

fn validate_request(
    request: &SecurityScorecardTrainingRequest,
) -> Result<(), SecurityScorecardTrainingError> {
    for (field_name, field_value) in [
        ("market_scope", request.market_scope.trim()),
        ("instrument_scope", request.instrument_scope.trim()),
        ("target_head", request.target_head.trim()),
        ("train_range", request.train_range.trim()),
        ("valid_range", request.valid_range.trim()),
        ("test_range", request.test_range.trim()),
        ("feature_set_version", request.feature_set_version.trim()),
        (
            "label_definition_version",
            request.label_definition_version.trim(),
        ),
    ] {
        if field_value.is_empty() {
            return Err(SecurityScorecardTrainingError::Build(format!(
                "{field_name} cannot be empty"
            )));
        }
    }
    if request.horizon_days == 0 {
        return Err(SecurityScorecardTrainingError::Build(
            "horizon_days must be greater than 0".to_string(),
        ));
    }
    for (field_name, sample_target) in [
        ("train_samples_per_symbol", request.train_samples_per_symbol),
        ("valid_samples_per_symbol", request.valid_samples_per_symbol),
        ("test_samples_per_symbol", request.test_samples_per_symbol),
    ] {
        if sample_target == 0 {
            return Err(SecurityScorecardTrainingError::Build(format!(
                "{field_name} must be greater than 0"
            )));
        }
    }
    if target_mode_for_head(&request.target_head).is_err() {
        return Err(SecurityScorecardTrainingError::Build(format!(
            "unsupported target_head `{}`",
            request.target_head
        )));
    }
    if request.symbol_list.is_empty() {
        return Err(SecurityScorecardTrainingError::Build(
            "symbol_list cannot be empty".to_string(),
        ));
    }
    if request.instrument_scope.eq_ignore_ascii_case("ETF")
        && resolve_request_instrument_subscope(request).is_none()
    {
        return Err(SecurityScorecardTrainingError::Build(
            "etf instrument_subscope could not be resolved".to_string(),
        ));
    }
    Ok(())
}

fn target_mode_for_head(
    target_head: &str,
) -> Result<TrainingTargetMode, SecurityScorecardTrainingError> {
    match target_head {
        "direction_head" | "upside_first_head" | "stop_first_head" => {
            Ok(TrainingTargetMode::Classification)
        }
        "return_head" | "drawdown_head" | "path_quality_head" => Ok(TrainingTargetMode::Regression),
        _ => Err(SecurityScorecardTrainingError::Build(format!(
            "unsupported target_head `{target_head}`"
        ))),
    }
}

fn validate_train_split_for_target(
    train_samples: &[&TrainingSample],
    target_mode: TrainingTargetMode,
) -> Result<(), SecurityScorecardTrainingError> {
    if target_mode != TrainingTargetMode::Classification {
        return Ok(());
    }

    let positive_count = train_samples
        .iter()
        .filter(|sample| sample.label >= 0.5)
        .count();
    let negative_count = train_samples.len().saturating_sub(positive_count);
    if positive_count == 0 || negative_count == 0 {
        return Err(SecurityScorecardTrainingError::Build(
            "train split must contain both positive and negative labels".to_string(),
        ));
    }

    Ok(())
}

// 2026-04-11 CST: Resolve ETF training sub-pool identity from the request, because
// Scheme C now splits ETF artifacts into equity, treasury, gold, and cross-border
// families rather than keeping one generic ETF namespace.
// Purpose: guarantee that one governed training run produces one ETF sub-pool model
// id and never silently mixes incompatible ETF symbols.
fn resolve_request_instrument_subscope(
    request: &SecurityScorecardTrainingRequest,
) -> Option<String> {
    if !request.instrument_scope.eq_ignore_ascii_case("ETF") {
        return None;
    }
    if let Some(explicit) = request
        .instrument_subscope
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        return Some(explicit.to_string());
    }

    let mut resolved = None::<String>;
    for symbol in &request.symbol_list {
        let current = resolve_etf_subscope(
            symbol,
            request.market_profile.as_deref(),
            request.sector_profile.as_deref(),
        )?
        .to_string();
        match resolved.as_ref() {
            Some(existing) if existing != &current => return None,
            Some(_) => {}
            None => resolved = Some(current),
        }
    }

    resolved
}

// 2026-04-11 CST: Branch training feature configs by instrument scope, because ETF
// and equity now need separate model families even though they still share the
// surrounding governance chain.
// Purpose: keep equity behavior stable while letting ETF training consume the extra
// numeric differentiators required for cross-sectional separation.
fn training_feature_configs_for_instrument(
    instrument_scope: &str,
    instrument_subscope: Option<&str>,
) -> Vec<TrainingFeatureConfig> {
    let mut configs = vec![
        TrainingFeatureConfig {
            feature_name: "integrated_stance",
            group_name: "M",
            kind: TrainingFeatureKind::Categorical,
        },
        TrainingFeatureConfig {
            feature_name: "technical_alignment",
            group_name: "T",
            kind: TrainingFeatureKind::Categorical,
        },
        TrainingFeatureConfig {
            feature_name: "data_gap_count",
            group_name: "R",
            kind: TrainingFeatureKind::Numeric,
        },
        TrainingFeatureConfig {
            feature_name: "risk_note_count",
            group_name: "R",
            kind: TrainingFeatureKind::Numeric,
        },
    ];

    if instrument_scope.eq_ignore_ascii_case("ETF") {
        // 2026-04-11 CST: Narrow ETF training configs to the governed minimum factor
        // family of each ETF sub-pool, because Scheme B now treats treasury, gold,
        // cross-border, and equity ETF as different modeling entrances.
        // Purpose: make ETF artifacts differ not only by model id but also by the
        // minimum factor family they are structurally allowed to consume.
        configs.extend(
            required_etf_feature_family(instrument_subscope)
                .iter()
                .copied()
                .map(etf_training_feature_config),
        );
    }

    configs
}

// 2026-04-11 CST: Route ETF proxy-contract fields into categorical training bins,
// because Scheme C introduces placeholder external-status fields that should not be
// treated like numeric oscillators or distance ratios.
// Purpose: keep ETF training compatible with stable proxy field contracts while
// preserving numeric handling for the existing technical differentiators.
fn etf_training_feature_config(feature_name: &'static str) -> TrainingFeatureConfig {
    // 2026-04-12 UTC+08: Route every governed ETF proxy field into group X, because
    // external validation showed the old split left proxy statuses in X but pushed
    // proxy values into T, which flattened ETF-specific explanatory power and kept
    // `group_X_points` near zero.
    // Purpose: make ETF proxy history contribute through one dedicated ETF-external
    // factor lane instead of leaking into the generic technical bucket.
    if is_etf_external_proxy_feature(feature_name) {
        TrainingFeatureConfig {
            feature_name,
            group_name: "X",
            kind: if is_etf_external_proxy_categorical_feature(feature_name) {
                TrainingFeatureKind::Categorical
            } else {
                TrainingFeatureKind::Numeric
            },
        }
    } else {
        TrainingFeatureConfig {
            feature_name,
            group_name: "T",
            kind: TrainingFeatureKind::Numeric,
        }
    }
}

// 2026-04-12 UTC+08: Keep ETF proxy-field recognition centralized, because the
// training, scorecard, and external-validation diagnostics now all need one stable
// answer to “what counts as ETF external information”.
// Purpose: prevent the ETF proxy family from drifting across sub-pools or silently
// dropping new proxy fields back into the technical bucket.
fn is_etf_external_proxy_feature(feature_name: &str) -> bool {
    TREASURY_ETF_PROXY_FIELDS
        .iter()
        .chain(TREASURY_ETF_PROXY_NUMERIC_FIELDS.iter())
        .chain(GOLD_ETF_PROXY_FIELDS.iter())
        .chain(GOLD_ETF_PROXY_NUMERIC_FIELDS.iter())
        .chain(CROSS_BORDER_ETF_PROXY_FIELDS.iter())
        .chain(CROSS_BORDER_ETF_PROXY_NUMERIC_FIELDS.iter())
        .chain(EQUITY_ETF_PROXY_FIELDS.iter())
        .chain(EQUITY_ETF_PROXY_NUMERIC_FIELDS.iter())
        .any(|candidate| *candidate == feature_name)
}

// 2026-04-12 UTC+08: Split ETF proxy fields by storage kind, because proxy states
// remain categorical while proxy values must stay numeric for regression/binning.
// Purpose: keep the X-group migration auditable without corrupting numeric bins.
fn is_etf_external_proxy_categorical_feature(feature_name: &str) -> bool {
    TREASURY_ETF_PROXY_FIELDS
        .iter()
        .chain(GOLD_ETF_PROXY_FIELDS.iter())
        .chain(CROSS_BORDER_ETF_PROXY_FIELDS.iter())
        .chain(EQUITY_ETF_PROXY_FIELDS.iter())
        .any(|candidate| *candidate == feature_name)
}

fn parse_date_range(raw: &str) -> Result<TrainingDateRange, SecurityScorecardTrainingError> {
    let Some((start_raw, end_raw)) = raw.split_once("..") else {
        return Err(SecurityScorecardTrainingError::Build(format!(
            "invalid date range `{raw}`"
        )));
    };
    let start = NaiveDate::parse_from_str(start_raw.trim(), "%Y-%m-%d").map_err(|error| {
        SecurityScorecardTrainingError::Build(format!(
            "invalid range start `{}`: {error}",
            start_raw.trim()
        ))
    })?;
    let end = NaiveDate::parse_from_str(end_raw.trim(), "%Y-%m-%d").map_err(|error| {
        SecurityScorecardTrainingError::Build(format!(
            "invalid range end `{}`: {error}",
            end_raw.trim()
        ))
    })?;
    if end < start {
        return Err(SecurityScorecardTrainingError::Build(format!(
            "invalid date range `{raw}`: end is earlier than start"
        )));
    }
    Ok(TrainingDateRange { start, end })
}

fn collect_samples(
    request: &SecurityScorecardTrainingRequest,
    train_range: &TrainingDateRange,
    valid_range: &TrainingDateRange,
    test_range: &TrainingDateRange,
    feature_configs: &[TrainingFeatureConfig],
) -> Result<Vec<TrainingSample>, SecurityScorecardTrainingError> {
    let store = StockHistoryStore::workspace_default()?;
    let mut samples = Vec::new();

    // 2026-04-11 CST: Route sampling counts from the request instead of static
    // literals, because training governance now requires the sample pool size to
    // be explicit and reviewable for each run.
    // Purpose: make larger and repeatable training datasets possible without
    // editing code every time the sample plan changes.
    let split_targets = [
        ("train", train_range, request.train_samples_per_symbol),
        ("valid", valid_range, request.valid_samples_per_symbol),
        ("test", test_range, request.test_samples_per_symbol),
    ];

    for symbol in &request.symbol_list {
        for (split_name, range, target_count) in split_targets {
            let candidate_dates = load_dates_in_range(&store, symbol, range, 200)?;
            let selected_dates = select_evenly_spaced_dates(&candidate_dates, target_count);
            for as_of_date in selected_dates {
                let outcome_result = security_forward_outcome(&SecurityForwardOutcomeRequest {
                    symbol: symbol.clone(),
                    market_symbol: request.market_symbol.clone(),
                    sector_symbol: request.sector_symbol.clone(),
                    market_profile: request.market_profile.clone(),
                    sector_profile: request.sector_profile.clone(),
                    as_of_date: Some(as_of_date.clone()),
                    lookback_days: request.lookback_days,
                    disclosure_limit: request.disclosure_limit,
                    horizons: vec![request.horizon_days],
                    stop_loss_pct: request.stop_loss_pct,
                    target_return_pct: request.target_return_pct,
                    label_definition_version: request.label_definition_version.clone(),
                    external_proxy_inputs: request.external_proxy_inputs.clone(),
                })?;
                let outcome = outcome_result
                    .forward_outcomes
                    .first()
                    .cloned()
                    .ok_or_else(|| {
                        SecurityScorecardTrainingError::Build(format!(
                            "missing forward outcome for {symbol} at {as_of_date}"
                        ))
                    })?;
                let feature_values = extract_feature_values(
                    &outcome_result.snapshot.raw_features_json,
                    feature_configs,
                )?;
                let label = derive_training_label(
                    &request.target_head,
                    &outcome,
                    request.stop_loss_pct,
                    request.target_return_pct,
                )?;
                samples.push(TrainingSample {
                    symbol: symbol.clone(),
                    as_of_date: as_of_date.clone(),
                    split_name: split_name.to_string(),
                    label,
                    forward_return: outcome.forward_return,
                    max_drawdown: outcome.max_drawdown,
                    max_runup: outcome.max_runup,
                    hit_upside_first: outcome.hit_upside_first,
                    hit_stop_first: outcome.hit_stop_first,
                    feature_values,
                });
            }
        }
    }

    if samples.is_empty() {
        return Err(SecurityScorecardTrainingError::Build(
            "no samples were collected for the requested ranges".to_string(),
        ));
    }

    Ok(samples)
}

fn derive_training_label(
    target_head: &str,
    outcome: &crate::ops::stock::security_forward_outcome::SecurityForwardOutcomeDocument,
    stop_loss_pct: f64,
    target_return_pct: f64,
) -> Result<f64, SecurityScorecardTrainingError> {
    match target_head {
        "direction_head" => Ok(if outcome.positive_return { 1.0 } else { 0.0 }),
        "return_head" => Ok(outcome.forward_return),
        "drawdown_head" => Ok(outcome.max_drawdown),
        "upside_first_head" => Ok(if outcome.hit_upside_first { 1.0 } else { 0.0 }),
        "stop_first_head" => Ok(if outcome.hit_stop_first { 1.0 } else { 0.0 }),
        "path_quality_head" => Ok(derive_path_quality_label(
            outcome.max_runup,
            outcome.max_drawdown,
            outcome.hit_upside_first,
            outcome.hit_stop_first,
            stop_loss_pct,
            target_return_pct,
        )),
        _ => Err(SecurityScorecardTrainingError::Build(format!(
            "unsupported target_head `{target_head}`"
        ))),
    }
}

fn derive_path_quality_label(
    max_runup: f64,
    max_drawdown: f64,
    hit_upside_first: bool,
    hit_stop_first: bool,
    stop_loss_pct: f64,
    target_return_pct: f64,
) -> f64 {
    let safe_target = target_return_pct.max(0.01);
    let safe_stop = stop_loss_pct.max(0.01);
    let runup_component = clamp_value(40.0 + 35.0 * (max_runup / safe_target), 0.0, 100.0);
    let drawdown_penalty = clamp_value(25.0 * (max_drawdown / safe_stop), 0.0, 30.0);
    let event_bonus = match (hit_upside_first, hit_stop_first) {
        (true, false) => 20.0,
        (false, true) => -25.0,
        (true, true) => -10.0,
        (false, false) => 0.0,
    };
    clamp_value(runup_component - drawdown_penalty + event_bonus, 0.0, 100.0)
}

fn load_dates_in_range(
    store: &StockHistoryStore,
    symbol: &str,
    range: &TrainingDateRange,
    min_history_rows: usize,
) -> Result<Vec<String>, SecurityScorecardTrainingError> {
    let end_text = range.end.format("%Y-%m-%d").to_string();
    let lookback_days = (range.end - range.start).num_days().unsigned_abs() as usize + 32;
    let rows = store.load_recent_rows(symbol, Some(&end_text), lookback_days.max(32))?;

    let mut qualified_dates = Vec::new();
    for row in rows {
        let is_in_range = NaiveDate::parse_from_str(&row.trade_date, "%Y-%m-%d")
            .map(|trade_date| trade_date >= range.start && trade_date <= range.end)
            .unwrap_or(false);
        if !is_in_range {
            continue;
        }
        let history_rows =
            store.load_recent_rows(symbol, Some(&row.trade_date), min_history_rows)?;
        if history_rows.len() >= min_history_rows {
            qualified_dates.push(row.trade_date);
        }
    }

    Ok(qualified_dates)
}

fn select_evenly_spaced_dates(dates: &[String], target_count: usize) -> Vec<String> {
    if target_count == 0 || dates.is_empty() {
        return Vec::new();
    }
    if dates.len() <= target_count {
        return dates.to_vec();
    }

    let mut selected = Vec::new();
    for index in 0..target_count {
        let position = if target_count == 1 {
            dates.len() - 1
        } else {
            index * (dates.len() - 1) / (target_count - 1)
        };
        let candidate = dates[position].clone();
        if !selected.contains(&candidate) {
            selected.push(candidate);
        }
    }
    selected
}

fn extract_feature_values(
    raw_features_json: &BTreeMap<String, Value>,
    feature_configs: &[TrainingFeatureConfig],
) -> Result<BTreeMap<String, TrainingFeatureValue>, SecurityScorecardTrainingError> {
    let mut feature_values = BTreeMap::new();
    let is_etf_context = raw_features_json
        .get("sector_profile")
        .and_then(Value::as_str)
        .map(|value| value.contains("etf"))
        .unwrap_or(false);
    for config in feature_configs {
        let value = raw_features_json.get(config.feature_name).ok_or_else(|| {
            SecurityScorecardTrainingError::Build(format!(
                "snapshot missing feature `{}`",
                config.feature_name
            ))
        })?;
        let feature_value = match config.kind {
            TrainingFeatureKind::Numeric => value
                .as_f64()
                .or_else(|| value.as_i64().map(|number| number as f64))
                .map(TrainingFeatureValue::Numeric)
                .ok_or_else(|| {
                    SecurityScorecardTrainingError::Build(format!(
                        "feature `{}` is not numeric",
                        config.feature_name
                    ))
                })?,
            TrainingFeatureKind::Categorical => match value {
                // 2026-04-12 UTC+08: Normalize ETF integrated stance before training,
                // because ETF information synthesis now emits richer stance labels than
                // the first ETF scorecard generation expected.
                // Purpose: keep ETF retraining and ETF runtime scoring on the same
                // governed bucket semantics so new live stance words do not create
                // artificial feature-incomplete scorecards.
                Value::String(text)
                    if is_etf_context && config.feature_name == "integrated_stance" =>
                {
                    TrainingFeatureValue::Category(normalize_integrated_stance_for_modeling(text))
                }
                Value::String(text) => TrainingFeatureValue::Category(text.clone()),
                Value::Bool(flag) => TrainingFeatureValue::Category(flag.to_string()),
                Value::Null => TrainingFeatureValue::Category("__missing__".to_string()),
                _ => TrainingFeatureValue::Category(value.to_string()),
            },
        };
        feature_values.insert(config.feature_name.to_string(), feature_value);
    }
    Ok(feature_values)
}

fn samples_for_split<'a>(
    samples: &'a [TrainingSample],
    split_name: &str,
) -> Vec<&'a TrainingSample> {
    samples
        .iter()
        .filter(|sample| sample.split_name == split_name)
        .collect()
}

fn build_feature_models(
    train_samples: &[&TrainingSample],
    feature_configs: &[TrainingFeatureConfig],
    target_mode: TrainingTargetMode,
) -> Result<Vec<FeatureModel>, SecurityScorecardTrainingError> {
    let total_positive = train_samples
        .iter()
        .filter(|sample| sample.label >= 0.5)
        .count() as f64;
    let total_negative = train_samples.len() as f64 - total_positive;

    feature_configs
        .iter()
        .map(|config| {
            let bins = match (config.kind.clone(), target_mode) {
                (TrainingFeatureKind::Categorical, TrainingTargetMode::Classification) => {
                    build_categorical_bins(
                        train_samples,
                        config.feature_name,
                        total_positive,
                        total_negative,
                    )?
                }
                (TrainingFeatureKind::Numeric, TrainingTargetMode::Classification) => {
                    build_numeric_bins(
                        train_samples,
                        config.feature_name,
                        total_positive,
                        total_negative,
                    )?
                }
                (TrainingFeatureKind::Categorical, TrainingTargetMode::Regression) => {
                    build_categorical_prediction_bins(train_samples, config.feature_name)?
                }
                (TrainingFeatureKind::Numeric, TrainingTargetMode::Regression) => {
                    build_numeric_prediction_bins(train_samples, config.feature_name)?
                }
            };
            Ok(FeatureModel {
                feature_name: config.feature_name.to_string(),
                group_name: config.group_name.to_string(),
                kind: config.kind.clone(),
                bins,
            })
        })
        .collect()
}

fn build_categorical_prediction_bins(
    train_samples: &[&TrainingSample],
    feature_name: &str,
) -> Result<Vec<FeatureBinModel>, SecurityScorecardTrainingError> {
    let mut bucket_targets: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for sample in train_samples {
        let TrainingFeatureValue::Category(value) = sample
            .feature_values
            .get(feature_name)
            .cloned()
            .ok_or_else(|| {
                SecurityScorecardTrainingError::Build(format!(
                    "sample missing categorical feature `{feature_name}`"
                ))
            })?
        else {
            return Err(SecurityScorecardTrainingError::Build(format!(
                "feature `{feature_name}` expected categorical value"
            )));
        };
        bucket_targets.entry(value).or_default().push(sample.label);
    }

    let fallback_prediction =
        train_samples.iter().map(|sample| sample.label).sum::<f64>() / train_samples.len() as f64;

    let mut bins = bucket_targets
        .into_iter()
        .map(|(category, values)| FeatureBinModel {
            bin_label: category.clone(),
            match_values: vec![category],
            min_inclusive: None,
            max_exclusive: None,
            woe: 0.0,
            predicted_value: Some(values.iter().sum::<f64>() / values.len() as f64),
        })
        .collect::<Vec<_>>();
    // 2026-04-12 UTC+08: Append a categorical fallback bin for regression heads,
    // because pooled ETF validation now evaluates unseen holdout categories and the
    // old category-only bins collapse into missing predictions.
    // Purpose: keep regression artifacts numerically usable on same-pool holdout
    // symbols without pretending the unseen category is equivalent to a seen label.
    bins.push(FeatureBinModel {
        bin_label: "__other__".to_string(),
        match_values: vec!["__other__".to_string()],
        min_inclusive: None,
        max_exclusive: None,
        woe: 0.0,
        predicted_value: Some(fallback_prediction),
    });

    Ok(bins)
}

fn build_numeric_prediction_bins(
    train_samples: &[&TrainingSample],
    feature_name: &str,
) -> Result<Vec<FeatureBinModel>, SecurityScorecardTrainingError> {
    let numeric_values = train_samples
        .iter()
        .map(|sample| {
            let TrainingFeatureValue::Numeric(value) = sample
                .feature_values
                .get(feature_name)
                .cloned()
                .ok_or_else(|| {
                    SecurityScorecardTrainingError::Build(format!(
                        "sample missing numeric feature `{feature_name}`"
                    ))
                })?
            else {
                return Err(SecurityScorecardTrainingError::Build(format!(
                    "feature `{feature_name}` expected numeric value"
                )));
            };
            Ok(value)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let thresholds = build_numeric_thresholds(&numeric_values);
    let mut template_bins = build_numeric_intervals(&thresholds);
    let mut bucket_targets = vec![Vec::<f64>::new(); template_bins.len()];

    for sample in train_samples {
        let TrainingFeatureValue::Numeric(value) = sample
            .feature_values
            .get(feature_name)
            .cloned()
            .ok_or_else(|| {
                SecurityScorecardTrainingError::Build(format!(
                    "sample missing numeric feature `{feature_name}`"
                ))
            })?
        else {
            return Err(SecurityScorecardTrainingError::Build(format!(
                "feature `{feature_name}` expected numeric value"
            )));
        };

        let Some((index, _)) = template_bins
            .iter()
            .enumerate()
            .find(|(_, bin)| numeric_bin_matches(bin, value))
        else {
            return Err(SecurityScorecardTrainingError::Build(format!(
                "no numeric bin matched feature `{feature_name}` value {value}"
            )));
        };
        bucket_targets[index].push(sample.label);
    }

    for (index, bin) in template_bins.iter_mut().enumerate() {
        let values = &bucket_targets[index];
        bin.predicted_value = if values.is_empty() {
            None
        } else {
            Some(values.iter().sum::<f64>() / values.len() as f64)
        };
    }

    Ok(template_bins)
}

fn build_categorical_bins(
    train_samples: &[&TrainingSample],
    feature_name: &str,
    total_positive: f64,
    total_negative: f64,
) -> Result<Vec<FeatureBinModel>, SecurityScorecardTrainingError> {
    let mut bucket_counts: BTreeMap<String, (f64, f64)> = BTreeMap::new();
    for sample in train_samples {
        let TrainingFeatureValue::Category(value) = sample
            .feature_values
            .get(feature_name)
            .cloned()
            .ok_or_else(|| {
                SecurityScorecardTrainingError::Build(format!(
                    "sample missing categorical feature `{feature_name}`"
                ))
            })?
        else {
            return Err(SecurityScorecardTrainingError::Build(format!(
                "feature `{feature_name}` expected categorical value"
            )));
        };
        let entry = bucket_counts.entry(value).or_insert((0.0, 0.0));
        if sample.label >= 0.5 {
            entry.0 += 1.0;
        } else {
            entry.1 += 1.0;
        }
    }

    let mut bins = bucket_counts
        .into_iter()
        .map(
            |(value, (positive_count, negative_count))| FeatureBinModel {
                bin_label: value.clone(),
                match_values: vec![value],
                min_inclusive: None,
                max_exclusive: None,
                woe: compute_woe(
                    positive_count,
                    negative_count,
                    total_positive,
                    total_negative,
                ),
                predicted_value: None,
            },
        )
        .collect::<Vec<_>>();
    // 2026-04-12 UTC+08: Append a neutral categorical fallback bin for
    // classification heads, because pooled ETF holdout scoring must not degrade to
    // `feature_incomplete` whenever a new same-pool category appears after training.
    // Purpose: preserve runtime continuity while keeping unseen categories neutral
    // instead of overclaiming direction information.
    bins.push(FeatureBinModel {
        bin_label: "__other__".to_string(),
        match_values: vec!["__other__".to_string()],
        min_inclusive: None,
        max_exclusive: None,
        woe: 0.0,
        predicted_value: None,
    });

    Ok(bins)
}

fn build_numeric_bins(
    train_samples: &[&TrainingSample],
    feature_name: &str,
    total_positive: f64,
    total_negative: f64,
) -> Result<Vec<FeatureBinModel>, SecurityScorecardTrainingError> {
    let numeric_values = train_samples
        .iter()
        .map(|sample| {
            let TrainingFeatureValue::Numeric(value) = sample
                .feature_values
                .get(feature_name)
                .cloned()
                .ok_or_else(|| {
                    SecurityScorecardTrainingError::Build(format!(
                        "sample missing numeric feature `{feature_name}`"
                    ))
                })?
            else {
                return Err(SecurityScorecardTrainingError::Build(format!(
                    "feature `{feature_name}` expected numeric value"
                )));
            };
            Ok(value)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let thresholds = build_numeric_thresholds(&numeric_values);
    let template_bins = build_numeric_intervals(&thresholds);
    let mut bucket_counts = vec![(0.0_f64, 0.0_f64); template_bins.len()];

    for sample in train_samples {
        let TrainingFeatureValue::Numeric(value) = sample
            .feature_values
            .get(feature_name)
            .cloned()
            .ok_or_else(|| {
                SecurityScorecardTrainingError::Build(format!(
                    "sample missing numeric feature `{feature_name}`"
                ))
            })?
        else {
            return Err(SecurityScorecardTrainingError::Build(format!(
                "feature `{feature_name}` expected numeric value"
            )));
        };
        let Some((index, _)) = template_bins
            .iter()
            .enumerate()
            .find(|(_, bin)| numeric_bin_matches(bin, value))
        else {
            return Err(SecurityScorecardTrainingError::Build(format!(
                "no numeric bin matched feature `{feature_name}` value {value}"
            )));
        };
        if sample.label >= 0.5 {
            bucket_counts[index].0 += 1.0;
        } else {
            bucket_counts[index].1 += 1.0;
        }
    }

    Ok(template_bins
        .into_iter()
        .enumerate()
        .map(|(index, bin)| FeatureBinModel {
            bin_label: bin.bin_label,
            match_values: Vec::new(),
            min_inclusive: bin.min_inclusive,
            max_exclusive: bin.max_exclusive,
            woe: compute_woe(
                bucket_counts[index].0,
                bucket_counts[index].1,
                total_positive,
                total_negative,
            ),
            predicted_value: None,
        })
        .collect())
}

fn build_numeric_thresholds(values: &[f64]) -> Vec<f64> {
    let mut sorted = values.to_vec();
    sorted.sort_by(|left, right| left.total_cmp(right));
    sorted.dedup_by(|left, right| (*left - *right).abs() <= 1e-9);
    if sorted.len() <= 1 {
        return Vec::new();
    }

    let mut thresholds = vec![sorted[sorted.len() / 3], sorted[(sorted.len() * 2) / 3]];
    thresholds.sort_by(|left, right| left.total_cmp(right));
    thresholds.dedup_by(|left, right| (*left - *right).abs() <= 1e-9);
    thresholds
}

fn build_numeric_intervals(thresholds: &[f64]) -> Vec<FeatureBinModel> {
    if thresholds.is_empty() {
        return vec![FeatureBinModel {
            bin_label: "all".to_string(),
            match_values: Vec::new(),
            min_inclusive: None,
            max_exclusive: None,
            woe: 0.0,
            predicted_value: None,
        }];
    }

    let mut bins = Vec::new();
    let mut lower = None;
    for (index, threshold) in thresholds.iter().enumerate() {
        bins.push(FeatureBinModel {
            bin_label: format!("bin_{}", index + 1),
            match_values: Vec::new(),
            min_inclusive: lower,
            max_exclusive: Some(*threshold),
            woe: 0.0,
            predicted_value: None,
        });
        lower = Some(*threshold);
    }
    bins.push(FeatureBinModel {
        bin_label: format!("bin_{}", thresholds.len() + 1),
        match_values: Vec::new(),
        min_inclusive: lower,
        max_exclusive: None,
        woe: 0.0,
        predicted_value: None,
    });
    bins
}

fn numeric_bin_matches(bin: &FeatureBinModel, value: f64) -> bool {
    let lower_match = bin.min_inclusive.map(|min| value >= min).unwrap_or(true);
    let upper_match = bin.max_exclusive.map(|max| value < max).unwrap_or(true);
    lower_match && upper_match
}

fn compute_woe(
    positive_count: f64,
    negative_count: f64,
    total_positive: f64,
    total_negative: f64,
) -> f64 {
    let smooth = 0.5;
    let positive_rate = (positive_count + smooth) / (total_positive + smooth * 2.0);
    let negative_rate = (negative_count + smooth) / (total_negative + smooth * 2.0);
    (positive_rate / negative_rate).ln()
}

fn encode_samples(
    samples: &[&TrainingSample],
    feature_models: &[FeatureModel],
) -> Result<Vec<(Vec<f64>, f64)>, SecurityScorecardTrainingError> {
    samples
        .iter()
        .map(|sample| {
            let mut row = vec![1.0_f64];
            for feature_model in feature_models {
                row.push(resolve_feature_woe(feature_model, sample)?);
            }
            Ok((row, sample.label))
        })
        .collect()
}

fn resolve_feature_woe(
    feature_model: &FeatureModel,
    sample: &TrainingSample,
) -> Result<f64, SecurityScorecardTrainingError> {
    let value = sample
        .feature_values
        .get(&feature_model.feature_name)
        .ok_or_else(|| {
            SecurityScorecardTrainingError::Build(format!(
                "sample missing feature `{}`",
                feature_model.feature_name
            ))
        })?;

    match (&feature_model.kind, value) {
        (TrainingFeatureKind::Categorical, TrainingFeatureValue::Category(category)) => {
            feature_model
                .bins
                .iter()
                .find(|bin| {
                    bin.match_values
                        .iter()
                        .any(|candidate| candidate == category)
                })
                .map(|bin| bin.woe)
                .ok_or_else(|| {
                    SecurityScorecardTrainingError::Build(format!(
                        "no categorical bin matched feature `{}` value `{category}`",
                        feature_model.feature_name
                    ))
                })
        }
        (TrainingFeatureKind::Numeric, TrainingFeatureValue::Numeric(number)) => feature_model
            .bins
            .iter()
            .find(|bin| numeric_bin_matches(bin, *number))
            .map(|bin| bin.woe)
            .ok_or_else(|| {
                SecurityScorecardTrainingError::Build(format!(
                    "no numeric bin matched feature `{}` value {}",
                    feature_model.feature_name, number
                ))
            }),
        _ => Err(SecurityScorecardTrainingError::Build(format!(
            "feature `{}` kind mismatch",
            feature_model.feature_name
        ))),
    }
}

// 2026-04-09 CST: 这里使用最小批量梯度下降拟合 logistic，原因是 Task 5 首版只要求纯 Rust 的轻量闭环，不提前引入额外训练框架；
// 目的：先稳定产出可回放的 coefficient artifact，为后续更复杂的 walk-forward 和晋级治理打底。
fn train_logistic_model(encoded_train_rows: &[(Vec<f64>, f64)]) -> TrainedLogisticModel {
    let parameter_count = encoded_train_rows
        .first()
        .map(|(row, _)| row.len())
        .unwrap_or(1);
    let mut beta = vec![0.0_f64; parameter_count];
    let average_norm = encoded_train_rows
        .iter()
        .map(|(row, _)| row.iter().map(|value| value * value).sum::<f64>())
        .sum::<f64>()
        / encoded_train_rows.len() as f64;
    let learning_rate = 1.0 / average_norm.max(1.0);

    for _ in 0..10_000 {
        let mut gradient = vec![0.0_f64; parameter_count];
        for (row, label) in encoded_train_rows {
            let prediction = logistic(dot(row, &beta));
            let error = prediction - *label;
            for (index, value) in row.iter().enumerate() {
                gradient[index] += error * value;
            }
        }
        let mut max_change = 0.0_f64;
        for index in 0..parameter_count {
            let step = learning_rate * gradient[index] / encoded_train_rows.len() as f64;
            beta[index] -= step;
            max_change = max_change.max(step.abs());
        }
        if max_change <= 1e-8 {
            break;
        }
    }

    TrainedLogisticModel {
        intercept: beta[0],
        coefficients: beta.into_iter().skip(1).collect(),
    }
}

fn train_regression_head_model(train_samples: &[&TrainingSample]) -> TrainedRegressionModel {
    let baseline =
        train_samples.iter().map(|sample| sample.label).sum::<f64>() / train_samples.len() as f64;
    TrainedRegressionModel { baseline }
}

fn build_artifact(
    request: &SecurityScorecardTrainingRequest,
    feature_models: &[FeatureModel],
    trained_model: &TrainedHeadModel,
    instrument_subscope: Option<&str>,
) -> SecurityScorecardModelArtifact {
    let model_id = if let Some(subscope) = instrument_subscope {
        format!(
            "{}_{}_{}_{}d_{}",
            request.market_scope.to_lowercase(),
            request.instrument_scope.to_lowercase(),
            subscope,
            request.horizon_days,
            request.target_head
        )
    } else {
        format!(
            "{}_{}_{}d_{}",
            request.market_scope.to_lowercase(),
            request.instrument_scope.to_lowercase(),
            request.horizon_days,
            request.target_head
        )
    };
    let model_version = format!("candidate_{}", sanitize_identifier(&request.created_at));

    let features = feature_models
        .iter()
        .enumerate()
        .map(|(index, feature_model)| SecurityScorecardModelFeatureSpec {
            feature_name: feature_model.feature_name.clone(),
            group_name: feature_model.group_name.clone(),
            bins: feature_model
                .bins
                .iter()
                .map(|bin| SecurityScorecardModelBin {
                    bin_label: bin.bin_label.clone(),
                    match_values: bin.match_values.clone(),
                    min_inclusive: bin.min_inclusive,
                    max_exclusive: bin.max_exclusive,
                    woe: match trained_model {
                        TrainedHeadModel::Classification(_) => Some(bin.woe),
                        TrainedHeadModel::Regression(_) => None,
                    },
                    logit_contribution: match trained_model {
                        TrainedHeadModel::Classification(model) => {
                            Some(model.coefficients.get(index).copied().unwrap_or(0.0) * bin.woe)
                        }
                        TrainedHeadModel::Regression(_) => None,
                    },
                    points: match trained_model {
                        TrainedHeadModel::Classification(model) => {
                            model.coefficients.get(index).copied().unwrap_or(0.0) * bin.woe * 100.0
                        }
                        TrainedHeadModel::Regression(_) => 0.0,
                    },
                    predicted_value: bin.predicted_value,
                })
                .collect(),
        })
        .collect();

    SecurityScorecardModelArtifact {
        model_id,
        model_version,
        label_definition: request.label_definition_version.clone(),
        target_head: Some(request.target_head.clone()),
        prediction_mode: Some(match trained_model {
            TrainedHeadModel::Classification(_) => "classification".to_string(),
            TrainedHeadModel::Regression(_) => "regression".to_string(),
        }),
        prediction_baseline: match trained_model {
            TrainedHeadModel::Classification(_) => None,
            TrainedHeadModel::Regression(model) => Some(model.baseline),
        },
        training_window: Some(request.train_range.clone()),
        oot_window: Some(request.test_range.clone()),
        positive_label_definition: match trained_model {
            TrainedHeadModel::Classification(_) => {
                Some(format!("positive_return_{}d", request.horizon_days))
            }
            TrainedHeadModel::Regression(_) => None,
        },
        instrument_subscope: instrument_subscope.map(|value| value.to_string()),
        binning_version: Some("woe_binning.v1".to_string()),
        coefficient_version: Some(match trained_model {
            TrainedHeadModel::Classification(_) => "woe_logistic.v1".to_string(),
            TrainedHeadModel::Regression(_) => "bin_mean_regression.v1".to_string(),
        }),
        model_sha256: None,
        intercept: match trained_model {
            TrainedHeadModel::Classification(model) => Some(model.intercept),
            TrainedHeadModel::Regression(_) => None,
        },
        base_score: match trained_model {
            TrainedHeadModel::Classification(_) => 600.0,
            TrainedHeadModel::Regression(model) => model.baseline,
        },
        features,
    }
}

fn build_metrics_summary(
    samples: &[TrainingSample],
    feature_models: &[FeatureModel],
    trained_model: &TrainedHeadModel,
    target_mode: TrainingTargetMode,
    target_head: &str,
) -> Value {
    let train_metrics =
        evaluate_split(samples, "train", feature_models, trained_model, target_mode);
    let valid_metrics =
        evaluate_split(samples, "valid", feature_models, trained_model, target_mode);
    let test_metrics = evaluate_split(samples, "test", feature_models, trained_model, target_mode);
    let readiness_assessment = build_readiness_assessment(
        samples,
        &train_metrics,
        &valid_metrics,
        &test_metrics,
        target_mode,
        target_head,
    );

    json!({
        "target_mode": match target_mode {
            TrainingTargetMode::Classification => "classification",
            TrainingTargetMode::Regression => "regression",
        },
        "train": train_metrics,
        "valid": valid_metrics,
        "test": test_metrics,
        "feature_count": feature_models.len(),
        "sample_count": samples.len(),
        "sample_breakdown": build_sample_breakdown(samples),
        "readiness_assessment": readiness_assessment,
    })
}

// 2026-04-11 CST: Add a governed sample composition panel, because Scheme B
// requires training runs to explain not just fit quality but also how many
// symbols and snapshots supported each split.
// Purpose: make it obvious when a result comes from a too-thin sample pool.
fn build_sample_breakdown(samples: &[TrainingSample]) -> Value {
    json!({
        "train": summarize_split_samples(samples, "train"),
        "valid": summarize_split_samples(samples, "valid"),
        "test": summarize_split_samples(samples, "test"),
    })
}

fn summarize_split_samples(samples: &[TrainingSample], split_name: &str) -> Value {
    let split_samples = samples_for_split(samples, split_name);
    let symbols = split_samples
        .iter()
        .map(|sample| sample.symbol.clone())
        .collect::<BTreeSet<_>>();
    let first_as_of_date = split_samples
        .iter()
        .map(|sample| sample.as_of_date.as_str())
        .min()
        .map(str::to_string);
    let last_as_of_date = split_samples
        .iter()
        .map(|sample| sample.as_of_date.as_str())
        .max()
        .map(str::to_string);

    json!({
        "sample_count": split_samples.len(),
        "unique_symbol_count": symbols.len(),
        "symbols": symbols.into_iter().collect::<Vec<_>>(),
        "first_as_of_date": first_as_of_date,
        "last_as_of_date": last_as_of_date,
    })
}

// 2026-04-11 CST: Add a formal readiness assessment panel, because Scheme B
// needs training runs to declare whether the current sample pool is only
// research-grade or materially closer to production usage.
// Purpose: let downstream governance consume explicit sample, balance, and path
// coverage statuses instead of relying on human interpretation of raw metrics.
fn build_readiness_assessment(
    samples: &[TrainingSample],
    train_metrics: &Value,
    valid_metrics: &Value,
    test_metrics: &Value,
    target_mode: TrainingTargetMode,
    target_head: &str,
) -> Value {
    let train_sample_count = samples_for_split(samples, "train").len();
    let valid_sample_count = samples_for_split(samples, "valid").len();
    let test_sample_count = samples_for_split(samples, "test").len();
    let minimum_sample_status =
        if train_sample_count >= 12 && valid_sample_count >= 6 && test_sample_count >= 6 {
            "sample_ready"
        } else {
            "sample_thin"
        };

    let class_balance_status = if target_mode == TrainingTargetMode::Classification {
        let train_positive_rate = metric_f64(train_metrics, &["positive_rate"]).unwrap_or(0.0);
        let train_negative_rate = metric_f64(train_metrics, &["negative_rate"]).unwrap_or(0.0);
        if train_positive_rate >= 0.25 && train_negative_rate >= 0.25 {
            "class_balance_ready"
        } else {
            "class_imbalance_warning"
        }
    } else {
        "not_applicable"
    };

    let train_upside_rate = metric_f64(
        train_metrics,
        &["horizon_event_summary", "hit_upside_first_rate"],
    )
    .unwrap_or(0.0);
    let train_stop_rate = metric_f64(
        train_metrics,
        &["horizon_event_summary", "hit_stop_first_rate"],
    )
    .unwrap_or(0.0);
    let path_event_coverage_status = if train_upside_rate >= 0.05 && train_stop_rate >= 0.05 {
        "path_event_ready"
    } else {
        "path_event_sparse"
    };
    // 2026-04-11 CST: Split out head-level path-event readiness, because P4 path
    // heads need a direct governance field that says whether the selected event head
    // itself has enough positive coverage instead of only seeing the combined horizon
    // event summary.
    // Purpose: let upside-first and stop-first heads explain their own event coverage.
    let head_path_event_coverage_status = if target_mode == TrainingTargetMode::Classification {
        match target_head {
            "upside_first_head" | "stop_first_head" => {
                let train_event_positive_rate =
                    metric_f64(train_metrics, &["event_positive_rate"]).unwrap_or(0.0);
                let train_negative_rate =
                    metric_f64(train_metrics, &["negative_rate"]).unwrap_or(0.0);
                if train_event_positive_rate >= 0.05 && train_negative_rate >= 0.05 {
                    "path_event_ready"
                } else {
                    "path_event_sparse"
                }
            }
            _ => "not_path_event_head",
        }
    } else {
        "not_applicable"
    };

    let production_readiness = match target_mode {
        TrainingTargetMode::Classification => {
            let valid_auc = metric_f64(valid_metrics, &["auc"]);
            let test_auc = metric_f64(test_metrics, &["auc"]);
            if minimum_sample_status == "sample_ready"
                && class_balance_status == "class_balance_ready"
                && path_event_coverage_status == "path_event_ready"
                && valid_auc.unwrap_or(0.0) >= 0.65
                && test_auc.unwrap_or(0.0) >= 0.65
            {
                "shadow_candidate_ready"
            } else {
                "research_candidate_only"
            }
        }
        TrainingTargetMode::Regression => {
            let valid_rmse = metric_f64(valid_metrics, &["rmse"]);
            let test_rmse = metric_f64(test_metrics, &["rmse"]);
            if minimum_sample_status == "sample_ready"
                && valid_rmse.is_some()
                && test_rmse.is_some()
            {
                "shadow_candidate_ready"
            } else {
                "research_candidate_only"
            }
        }
    };

    let mut notes = Vec::new();
    if minimum_sample_status != "sample_ready" {
        notes.push(
            "sample pool is still too thin across train/valid/test splits for production promotion"
                .to_string(),
        );
    }
    if target_mode == TrainingTargetMode::Classification
        && class_balance_status != "class_balance_ready"
    {
        notes.push(
            "class balance is still skewed in the governed train split and can distort probability calibration"
                .to_string(),
        );
    }
    if path_event_coverage_status != "path_event_ready" {
        notes.push(
            "path events are still too sparse, so upside-first and stop-first behavior should stay research-only"
                .to_string(),
        );
    }
    if production_readiness != "shadow_candidate_ready" {
        notes.push(
            "the current run should remain a research candidate until path coverage and out-of-sample quality both improve"
                .to_string(),
        );
    }

    json!({
        "minimum_sample_status": minimum_sample_status,
        "class_balance_status": class_balance_status,
        "path_event_coverage_status": path_event_coverage_status,
        "head_path_event_coverage_status": head_path_event_coverage_status,
        "production_readiness": production_readiness,
        "notes": notes,
    })
}

fn metric_f64(metric_root: &Value, path: &[&str]) -> Option<f64> {
    let mut current = metric_root;
    for key in path {
        current = current.get(*key)?;
    }
    current
        .as_f64()
        .or_else(|| current.as_i64().map(|number| number as f64))
}

fn evaluate_split(
    samples: &[TrainingSample],
    split_name: &str,
    feature_models: &[FeatureModel],
    trained_model: &TrainedHeadModel,
    target_mode: TrainingTargetMode,
) -> Value {
    let split_samples = samples_for_split(samples, split_name);
    if split_samples.is_empty() {
        return empty_split_metrics(target_mode);
    }

    match target_mode {
        TrainingTargetMode::Classification => {
            evaluate_classification_split(&split_samples, feature_models, trained_model)
        }
        TrainingTargetMode::Regression => {
            evaluate_regression_split(&split_samples, feature_models, trained_model)
        }
    }
}

fn empty_split_metrics(target_mode: TrainingTargetMode) -> Value {
    let base = json!({
        "sample_count": 0,
        "horizon_event_summary": {
            "avg_forward_return": Value::Null,
            "avg_max_drawdown": Value::Null,
            "avg_max_runup": Value::Null,
            "hit_upside_first_rate": Value::Null,
            "hit_stop_first_rate": Value::Null,
        },
    });
    match target_mode {
        TrainingTargetMode::Classification => json!({
            "sample_count": 0,
            "accuracy": Value::Null,
            "positive_rate": Value::Null,
            "negative_rate": Value::Null,
            "average_probability": Value::Null,
            "auc": Value::Null,
            "ks": Value::Null,
            "confusion_matrix": {
                "tp": 0,
                "tn": 0,
                "fp": 0,
                "fn": 0,
            },
            "horizon_event_summary": base["horizon_event_summary"].clone(),
        }),
        TrainingTargetMode::Regression => json!({
            "sample_count": 0,
            "mae": Value::Null,
            "rmse": Value::Null,
            "average_prediction": Value::Null,
            "average_actual": Value::Null,
            "directional_hit_rate": Value::Null,
            "horizon_event_summary": base["horizon_event_summary"].clone(),
        }),
    }
}

fn evaluate_classification_split(
    split_samples: &[&TrainingSample],
    feature_models: &[FeatureModel],
    trained_model: &TrainedHeadModel,
) -> Value {
    let TrainedHeadModel::Classification(trained_model) = trained_model else {
        return empty_split_metrics(TrainingTargetMode::Classification);
    };

    let mut correct_count = 0_usize;
    let mut positive_count = 0_usize;
    let mut tp = 0_usize;
    let mut tn = 0_usize;
    let mut fp = 0_usize;
    let mut fn_count = 0_usize;
    let mut probability_pairs = Vec::new();
    let mut probability_sum = 0.0_f64;
    let mut forward_return_sum = 0.0_f64;
    let mut max_drawdown_sum = 0.0_f64;
    let mut max_runup_sum = 0.0_f64;
    let mut hit_upside_first_count = 0_usize;
    let mut hit_stop_first_count = 0_usize;

    for sample in split_samples {
        let probability = predict_probability(sample, feature_models, trained_model).unwrap_or(0.5);
        probability_sum += probability;
        probability_pairs.push((probability, sample.label));
        let predicted = if probability >= 0.5 { 1.0 } else { 0.0 };
        if (predicted - sample.label).abs() <= 1e-9 {
            correct_count += 1;
        }
        match (predicted >= 0.5, sample.label >= 0.5) {
            (true, true) => tp += 1,
            (false, false) => tn += 1,
            (true, false) => fp += 1,
            (false, true) => fn_count += 1,
        }
        if sample.label >= 0.5 {
            positive_count += 1;
        }
        forward_return_sum += sample.forward_return;
        max_drawdown_sum += sample.max_drawdown;
        max_runup_sum += sample.max_runup;
        if sample.hit_upside_first {
            hit_upside_first_count += 1;
        }
        if sample.hit_stop_first {
            hit_stop_first_count += 1;
        }
    }
    let sample_count = split_samples.len();
    let sample_count_f64 = sample_count as f64;
    let negative_count = sample_count.saturating_sub(positive_count);

    json!({
        "sample_count": sample_count,
        "accuracy": correct_count as f64 / sample_count_f64,
        "positive_rate": positive_count as f64 / sample_count_f64,
        "negative_rate": negative_count as f64 / sample_count_f64,
        // 2026-04-11 CST: Add an explicit event-positive rate metric, because P4
        // path-event heads need the training panel to say how often the selected
        // event actually occurs instead of making readers infer it from positive_rate.
        // Purpose: expose direct event coverage for upside-first and stop-first heads.
        "event_positive_rate": positive_count as f64 / sample_count_f64,
        "average_probability": probability_sum / sample_count_f64,
        "auc": compute_auc(&probability_pairs),
        "ks": compute_ks(&probability_pairs),
        "confusion_matrix": {
            "tp": tp,
            "tn": tn,
            "fp": fp,
            "fn": fn_count,
        },
        "horizon_event_summary": {
            "avg_forward_return": forward_return_sum / sample_count_f64,
            "avg_max_drawdown": max_drawdown_sum / sample_count_f64,
            "avg_max_runup": max_runup_sum / sample_count_f64,
            "hit_upside_first_rate": hit_upside_first_count as f64 / sample_count_f64,
            "hit_stop_first_rate": hit_stop_first_count as f64 / sample_count_f64,
        },
    })
}

fn evaluate_regression_split(
    split_samples: &[&TrainingSample],
    feature_models: &[FeatureModel],
    trained_model: &TrainedHeadModel,
) -> Value {
    let TrainedHeadModel::Regression(trained_model) = trained_model else {
        return empty_split_metrics(TrainingTargetMode::Regression);
    };

    let mut absolute_error_sum = 0.0_f64;
    let mut squared_error_sum = 0.0_f64;
    let mut prediction_sum = 0.0_f64;
    let mut actual_sum = 0.0_f64;
    let mut directional_hit_count = 0_usize;
    let mut forward_return_sum = 0.0_f64;
    let mut max_drawdown_sum = 0.0_f64;
    let mut max_runup_sum = 0.0_f64;
    let mut hit_upside_first_count = 0_usize;
    let mut hit_stop_first_count = 0_usize;

    for sample in split_samples {
        let prediction = predict_regression_value(sample, feature_models, trained_model)
            .unwrap_or(trained_model.baseline);
        let error = prediction - sample.label;
        absolute_error_sum += error.abs();
        squared_error_sum += error * error;
        prediction_sum += prediction;
        actual_sum += sample.label;
        if prediction.signum() == sample.label.signum() {
            directional_hit_count += 1;
        }
        forward_return_sum += sample.forward_return;
        max_drawdown_sum += sample.max_drawdown;
        max_runup_sum += sample.max_runup;
        if sample.hit_upside_first {
            hit_upside_first_count += 1;
        }
        if sample.hit_stop_first {
            hit_stop_first_count += 1;
        }
    }

    let sample_count = split_samples.len();
    let sample_count_f64 = sample_count as f64;

    json!({
        "sample_count": sample_count,
        "mae": absolute_error_sum / sample_count_f64,
        "rmse": (squared_error_sum / sample_count_f64).sqrt(),
        "average_prediction": prediction_sum / sample_count_f64,
        "average_actual": actual_sum / sample_count_f64,
        "directional_hit_rate": directional_hit_count as f64 / sample_count_f64,
        "horizon_event_summary": {
            "avg_forward_return": forward_return_sum / sample_count_f64,
            "avg_max_drawdown": max_drawdown_sum / sample_count_f64,
            "avg_max_runup": max_runup_sum / sample_count_f64,
            "hit_upside_first_rate": hit_upside_first_count as f64 / sample_count_f64,
            "hit_stop_first_rate": hit_stop_first_count as f64 / sample_count_f64,
        },
    })
}

// 2026-04-11 CST: Add a lightweight AUC implementation to the formal metrics
// panel, because governance now requires fit quality to disclose more than raw
// accuracy when training-backed conclusions are claimed.
// Purpose: give the training summary a ranking-quality metric without pulling in
// a heavier external statistics dependency.
fn compute_auc(probability_pairs: &[(f64, f64)]) -> Value {
    let positive_scores = probability_pairs
        .iter()
        .filter(|(_, label)| *label >= 0.5)
        .map(|(probability, _)| *probability)
        .collect::<Vec<_>>();
    let negative_scores = probability_pairs
        .iter()
        .filter(|(_, label)| *label < 0.5)
        .map(|(probability, _)| *probability)
        .collect::<Vec<_>>();

    if positive_scores.is_empty() || negative_scores.is_empty() {
        return Value::Null;
    }

    let mut favorable_pairs = 0.0_f64;
    for positive_score in &positive_scores {
        for negative_score in &negative_scores {
            if positive_score > negative_score {
                favorable_pairs += 1.0;
            } else if (*positive_score - *negative_score).abs() <= 1e-12 {
                favorable_pairs += 0.5;
            }
        }
    }

    json!(favorable_pairs / (positive_scores.len() * negative_scores.len()) as f64)
}

fn compute_ks(probability_pairs: &[(f64, f64)]) -> Value {
    let total_positive = probability_pairs
        .iter()
        .filter(|(_, label)| *label >= 0.5)
        .count();
    let total_negative = probability_pairs
        .iter()
        .filter(|(_, label)| *label < 0.5)
        .count();

    if total_positive == 0 || total_negative == 0 {
        return Value::Null;
    }

    let mut ranked_pairs = probability_pairs.to_vec();
    ranked_pairs.sort_by(|left, right| right.0.total_cmp(&left.0));

    let mut cumulative_positive = 0.0_f64;
    let mut cumulative_negative = 0.0_f64;
    let mut max_ks = 0.0_f64;
    for (_, label) in ranked_pairs {
        if label >= 0.5 {
            cumulative_positive += 1.0 / total_positive as f64;
        } else {
            cumulative_negative += 1.0 / total_negative as f64;
        }
        max_ks = max_ks.max((cumulative_positive - cumulative_negative).abs());
    }

    json!(max_ks)
}

fn predict_probability(
    sample: &TrainingSample,
    feature_models: &[FeatureModel],
    trained_model: &TrainedLogisticModel,
) -> Result<f64, SecurityScorecardTrainingError> {
    let mut logit = trained_model.intercept;
    for (index, feature_model) in feature_models.iter().enumerate() {
        let coefficient = trained_model
            .coefficients
            .get(index)
            .copied()
            .unwrap_or(0.0);
        logit += coefficient * resolve_feature_woe(feature_model, sample)?;
    }
    Ok(logistic(logit))
}

fn predict_regression_value(
    sample: &TrainingSample,
    feature_models: &[FeatureModel],
    trained_model: &TrainedRegressionModel,
) -> Result<f64, SecurityScorecardTrainingError> {
    let mut matched_values = Vec::new();
    for feature_model in feature_models {
        let value = sample
            .feature_values
            .get(&feature_model.feature_name)
            .ok_or_else(|| {
                SecurityScorecardTrainingError::Build(format!(
                    "sample missing feature `{}`",
                    feature_model.feature_name
                ))
            })?;

        let matched_bin = match (&feature_model.kind, value) {
            (TrainingFeatureKind::Categorical, TrainingFeatureValue::Category(category)) => {
                feature_model.bins.iter().find(|bin| {
                    bin.match_values
                        .iter()
                        .any(|candidate| candidate == category)
                })
            }
            (TrainingFeatureKind::Numeric, TrainingFeatureValue::Numeric(number)) => feature_model
                .bins
                .iter()
                .find(|bin| numeric_bin_matches(bin, *number)),
            _ => {
                return Err(SecurityScorecardTrainingError::Build(format!(
                    "feature `{}` kind mismatch",
                    feature_model.feature_name
                )));
            }
        };

        if let Some(predicted_value) = matched_bin.and_then(|bin| bin.predicted_value) {
            matched_values.push(predicted_value);
        }
    }

    if matched_values.is_empty() {
        Ok(trained_model.baseline)
    } else {
        Ok(matched_values.iter().sum::<f64>() / matched_values.len() as f64)
    }
}

fn clamp_value(value: f64, min_value: f64, max_value: f64) -> f64 {
    value.max(min_value).min(max_value)
}

fn resolve_runtime_root(request: &SecurityScorecardTrainingRequest) -> PathBuf {
    request
        .training_runtime_root
        .as_ref()
        .map(|path| PathBuf::from(path.trim()))
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| {
            PathBuf::from(".worktrees")
                .join("SheetMind-Scenes-inspect")
                .join(".sheetmind_scenes_runtime")
        })
}

fn persist_json<T: Serialize>(
    path: &Path,
    value: &T,
) -> Result<(), SecurityScorecardTrainingError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| SecurityScorecardTrainingError::Persist(error.to_string()))?;
    }
    let payload = serde_json::to_vec_pretty(value)
        .map_err(|error| SecurityScorecardTrainingError::Persist(error.to_string()))?;
    fs::write(path, &payload)
        .map_err(|error| SecurityScorecardTrainingError::Persist(error.to_string()))?;
    Ok(())
}

fn dot(left: &[f64], right: &[f64]) -> f64 {
    left.iter().zip(right.iter()).map(|(x, y)| x * y).sum()
}

fn logistic(value: f64) -> f64 {
    1.0 / (1.0 + (-value).exp())
}

fn default_created_at() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn default_lookback_days() -> usize {
    260
}

fn default_disclosure_limit() -> usize {
    8
}

fn default_stop_loss_pct() -> f64 {
    0.05
}

fn default_target_return_pct() -> f64 {
    0.12
}

fn default_train_samples_per_symbol() -> usize {
    6
}

fn default_valid_samples_per_symbol() -> usize {
    3
}

fn default_test_samples_per_symbol() -> usize {
    3
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::{Value, json};

    use super::{
        FeatureModel, SecurityScorecardTrainingRequest, TrainedHeadModel, TrainedLogisticModel,
        TrainingFeatureKind, TrainingFeatureValue, TrainingSample, build_artifact,
        build_categorical_bins, build_categorical_prediction_bins, extract_feature_values,
        resolve_request_instrument_subscope, training_feature_configs_for_instrument,
    };

    #[test]
    fn etf_training_feature_config_exposes_etf_specific_numeric_signals() {
        // 2026-04-11 CST: Add an ETF-specific training-config red test, reason:
        // the user required ETF and equity to become separate model families instead
        // of sharing the same coarse four-feature setup.
        // Purpose: lock that ETF training must carry its own differentiating numeric
        // signals before we touch implementation.
        let configs = training_feature_configs_for_instrument("ETF", Some("equity_etf"));
        let feature_names = configs
            .iter()
            .map(|config| (config.feature_name, &config.kind))
            .collect::<Vec<_>>();

        assert!(
            feature_names
                .iter()
                .any(|(name, kind)| *name == "close_vs_sma50"
                    && matches!(kind, TrainingFeatureKind::Numeric))
        );
        assert!(
            feature_names
                .iter()
                .any(|(name, kind)| *name == "volume_ratio_20"
                    && matches!(kind, TrainingFeatureKind::Numeric))
        );
        assert!(
            feature_names
                .iter()
                .any(|(name, kind)| *name == "rsrs_zscore_18_60"
                    && matches!(kind, TrainingFeatureKind::Numeric))
        );
    }

    #[test]
    fn extract_feature_values_normalizes_etf_integrated_stance_bucket() {
        // 2026-04-12 UTC+08: Add a red training-side normalization test, because the
        // ETF live rerun now proves `mixed_watch` and `watchful_positive` reach the
        // governed scorecard path while the old ETF artifacts only learned the older
        // stance vocabulary.
        // Purpose: force the training collector to collapse ETF stance wording into
        // the same modeling bucket used by runtime scoring.
        let raw_features_json = BTreeMap::from([
            (
                "sector_profile".to_string(),
                Value::String("treasury_etf".to_string()),
            ),
            (
                "integrated_stance".to_string(),
                Value::String("watchful_positive".to_string()),
            ),
        ]);
        let feature_configs = vec![super::TrainingFeatureConfig {
            feature_name: "integrated_stance",
            group_name: "M",
            kind: TrainingFeatureKind::Categorical,
        }];

        let feature_values =
            extract_feature_values(&raw_features_json, &feature_configs).expect("feature values");

        assert_eq!(
            feature_values.get("integrated_stance"),
            Some(&super::TrainingFeatureValue::Category(
                "watchful_context".to_string()
            ))
        );
        assert_ne!(json!(feature_values.len()), json!(0));
    }

    #[test]
    fn etf_training_feature_config_separates_treasury_and_gold_subscopes() {
        // 2026-04-11 CST: Add a red test for ETF subscope-specific feature families, reason:
        // Scheme B now requires treasury ETF and gold ETF to stop sharing one generic ETF
        // feature entrance even before external factors are connected.
        // Purpose: force training config selection to expose different minimum factor families
        // for different ETF pools instead of only tagging them with different model ids.
        let treasury_configs = training_feature_configs_for_instrument("ETF", Some("treasury_etf"));
        let gold_configs = training_feature_configs_for_instrument("ETF", Some("gold_etf"));

        let treasury_feature_names = treasury_configs
            .iter()
            .map(|config| config.feature_name)
            .collect::<Vec<_>>();
        let gold_feature_names = gold_configs
            .iter()
            .map(|config| config.feature_name)
            .collect::<Vec<_>>();

        assert!(
            treasury_feature_names.contains(&"boll_width_ratio_20"),
            "treasury ETF training should require volatility-compression structure"
        );
        assert!(
            treasury_configs.iter().any(|config| {
                config.feature_name == "yield_curve_proxy_status"
                    && matches!(config.kind, TrainingFeatureKind::Categorical)
            }),
            "treasury ETF training should expose the yield-curve proxy contract as a categorical feature"
        );
        assert!(
            treasury_configs.iter().any(|config| {
                config.feature_name == "yield_curve_slope_delta_bp_5d"
                    && matches!(config.kind, TrainingFeatureKind::Numeric)
            }),
            "treasury ETF training should expose the yield-curve slope delta as a numeric feature"
        );
        assert!(
            treasury_configs.iter().any(|config| {
                config.feature_name == "funding_liquidity_proxy_status"
                    && matches!(config.kind, TrainingFeatureKind::Categorical)
            }),
            "treasury ETF training should expose the funding-liquidity proxy contract as a categorical feature"
        );
        assert!(
            treasury_configs.iter().any(|config| {
                config.feature_name == "funding_liquidity_spread_delta_bp_5d"
                    && matches!(config.kind, TrainingFeatureKind::Numeric)
            }),
            "treasury ETF training should expose the funding-liquidity spread delta as a numeric feature"
        );
        assert!(
            treasury_feature_names.contains(&"rsrs_zscore_18_60"),
            "treasury ETF training should require trend-normalized structure"
        );
        assert!(
            !treasury_feature_names.contains(&"williams_r_14"),
            "treasury ETF training should not reuse the gold ETF oscillator entrance as-is"
        );
        assert!(
            gold_feature_names.contains(&"williams_r_14"),
            "gold ETF training should require oscillator structure"
        );
        assert!(
            gold_configs.iter().any(|config| {
                config.feature_name == "gold_spot_proxy_status"
                    && matches!(config.kind, TrainingFeatureKind::Categorical)
            }),
            "gold ETF training should expose the gold-spot proxy contract as a categorical feature"
        );
        assert!(
            gold_configs.iter().any(|config| {
                config.feature_name == "gold_spot_proxy_return_5d"
                    && matches!(config.kind, TrainingFeatureKind::Numeric)
            }),
            "gold ETF training should expose the gold-spot proxy return as a numeric feature"
        );
        assert!(
            gold_configs.iter().any(|config| {
                config.feature_name == "usd_index_proxy_return_5d"
                    && matches!(config.kind, TrainingFeatureKind::Numeric)
            }),
            "gold ETF training should expose the USD proxy return as a numeric feature"
        );
        assert!(
            gold_configs.iter().any(|config| {
                config.feature_name == "real_rate_proxy_status"
                    && matches!(config.kind, TrainingFeatureKind::Categorical)
            }),
            "gold ETF training should expose the real-rate proxy contract as a categorical feature"
        );
        assert!(
            gold_configs.iter().any(|config| {
                config.feature_name == "real_rate_proxy_delta_bp_5d"
                    && matches!(config.kind, TrainingFeatureKind::Numeric)
            }),
            "gold ETF training should expose the real-rate proxy delta as a numeric feature"
        );
        assert!(
            gold_feature_names.contains(&"mfi_14"),
            "gold ETF training should require money-flow structure"
        );
        assert!(
            !gold_feature_names.contains(&"rsrs_zscore_18_60"),
            "gold ETF training should not inherit the treasury ETF structure gate by default"
        );
        let cross_border_configs =
            training_feature_configs_for_instrument("ETF", Some("cross_border_etf"));
        let cross_border_feature_names = cross_border_configs
            .iter()
            .map(|config| config.feature_name)
            .collect::<Vec<_>>();
        assert!(
            cross_border_configs.iter().any(|config| {
                config.feature_name == "fx_proxy_status"
                    && matches!(config.kind, TrainingFeatureKind::Categorical)
            }),
            "cross-border ETF training should expose the FX proxy contract as a categorical feature"
        );
        assert!(
            cross_border_configs.iter().any(|config| {
                config.feature_name == "fx_return_5d"
                    && matches!(config.kind, TrainingFeatureKind::Numeric)
            }),
            "cross-border ETF training should expose the FX return as a numeric feature"
        );
        assert!(
            cross_border_configs.iter().any(|config| {
                config.feature_name == "overseas_market_proxy_status"
                    && matches!(config.kind, TrainingFeatureKind::Categorical)
            }),
            "cross-border ETF training should expose the overseas-market proxy contract as a categorical feature"
        );
        assert!(
            cross_border_configs.iter().any(|config| {
                config.feature_name == "overseas_market_return_5d"
                    && matches!(config.kind, TrainingFeatureKind::Numeric)
            }),
            "cross-border ETF training should expose the overseas-market return as a numeric feature"
        );
        assert!(
            cross_border_configs.iter().any(|config| {
                config.feature_name == "market_session_gap_status"
                    && matches!(config.kind, TrainingFeatureKind::Categorical)
            }),
            "cross-border ETF training should expose the market-session-gap contract as a categorical feature"
        );
        assert!(
            cross_border_configs.iter().any(|config| {
                config.feature_name == "market_session_gap_days"
                    && matches!(config.kind, TrainingFeatureKind::Numeric)
            }),
            "cross-border ETF training should expose the market-session-gap days as a numeric feature"
        );
        assert!(
            cross_border_feature_names.contains(&"close_vs_sma50"),
            "cross-border ETF training should keep core relative-strength structure"
        );
        assert!(
            !cross_border_feature_names.contains(&"mfi_14"),
            "cross-border ETF training should not inherit the gold ETF oscillator entrance by default"
        );
        let equity_configs = training_feature_configs_for_instrument("ETF", Some("equity_etf"));
        let equity_feature_names = equity_configs
            .iter()
            .map(|config| config.feature_name)
            .collect::<Vec<_>>();
        assert!(
            equity_configs.iter().any(|config| {
                config.feature_name == "etf_fund_flow_proxy_status"
                    && matches!(config.kind, TrainingFeatureKind::Categorical)
            }),
            "equity ETF training should expose the fund-flow proxy contract as a categorical feature"
        );
        assert!(
            equity_configs.iter().any(|config| {
                config.feature_name == "etf_fund_flow_5d"
                    && matches!(config.kind, TrainingFeatureKind::Numeric)
            }),
            "equity ETF training should expose the fund-flow delta as a numeric feature"
        );
        assert!(
            equity_configs.iter().any(|config| {
                config.feature_name == "premium_discount_proxy_status"
                    && matches!(config.kind, TrainingFeatureKind::Categorical)
            }),
            "equity ETF training should expose the premium-discount proxy contract as a categorical feature"
        );
        assert!(
            equity_configs.iter().any(|config| {
                config.feature_name == "premium_discount_pct"
                    && matches!(config.kind, TrainingFeatureKind::Numeric)
            }),
            "equity ETF training should expose the premium-discount ratio as a numeric feature"
        );
        assert!(
            equity_configs.iter().any(|config| {
                config.feature_name == "benchmark_relative_strength_status"
                    && matches!(config.kind, TrainingFeatureKind::Categorical)
            }),
            "equity ETF training should expose the benchmark-relative-strength contract as a categorical feature"
        );
        assert!(
            equity_configs.iter().any(|config| {
                config.feature_name == "benchmark_relative_return_5d"
                    && matches!(config.kind, TrainingFeatureKind::Numeric)
            }),
            "equity ETF training should expose the benchmark-relative return as a numeric feature"
        );
        assert!(
            equity_feature_names.contains(&"rsrs_zscore_18_60"),
            "equity ETF training should keep the ETF relative-strength structure gate"
        );
        assert!(
            !equity_feature_names.contains(&"market_session_gap_days"),
            "equity ETF training should not inherit the cross-border session-gap entrance by default"
        );
    }

    #[test]
    fn etf_training_feature_config_routes_numeric_proxy_features_into_x_group() {
        // 2026-04-12 UTC+08: Add a red test for ETF numeric proxy grouping, because
        // external validation now shows `group_X_points` staying at zero even after
        // governed ETF proxy history is available.
        // Purpose: require numeric ETF proxy features to stay in the ETF-specific
        // `X` group instead of being flattened into generic technical `T`.
        let cross_border_configs =
            training_feature_configs_for_instrument("ETF", Some("cross_border_etf"));
        let equity_configs = training_feature_configs_for_instrument("ETF", Some("equity_etf"));

        assert!(
            cross_border_configs.iter().any(|config| {
                config.feature_name == "fx_return_5d"
                    && config.group_name == "X"
                    && matches!(config.kind, TrainingFeatureKind::Numeric)
            }),
            "cross-border ETF FX return should contribute through group X"
        );
        assert!(
            cross_border_configs.iter().any(|config| {
                config.feature_name == "overseas_market_return_5d"
                    && config.group_name == "X"
                    && matches!(config.kind, TrainingFeatureKind::Numeric)
            }),
            "cross-border ETF overseas-market return should contribute through group X"
        );
        assert!(
            cross_border_configs.iter().any(|config| {
                config.feature_name == "market_session_gap_days"
                    && config.group_name == "X"
                    && matches!(config.kind, TrainingFeatureKind::Numeric)
            }),
            "cross-border ETF session-gap days should contribute through group X"
        );
        assert!(
            equity_configs.iter().any(|config| {
                config.feature_name == "etf_fund_flow_5d"
                    && config.group_name == "X"
                    && matches!(config.kind, TrainingFeatureKind::Numeric)
            }),
            "equity ETF fund-flow delta should contribute through group X"
        );
        assert!(
            equity_configs.iter().any(|config| {
                config.feature_name == "premium_discount_pct"
                    && config.group_name == "X"
                    && matches!(config.kind, TrainingFeatureKind::Numeric)
            }),
            "equity ETF premium-discount ratio should contribute through group X"
        );
        assert!(
            equity_configs.iter().any(|config| {
                config.feature_name == "benchmark_relative_return_5d"
                    && config.group_name == "X"
                    && matches!(config.kind, TrainingFeatureKind::Numeric)
            }),
            "equity ETF benchmark-relative return should contribute through group X"
        );
    }

    #[test]
    fn etf_training_artifact_model_id_carries_treasury_subscope() {
        // 2026-04-11 CST: Add a red test for ETF sub-pool artifact identity, reason:
        // the user required bond, gold, cross-border, and equity ETF tracks to stop
        // sharing one generic ETF model namespace.
        // Purpose: force training artifacts to publish their ETF sub-pool so runtime
        // governance can reject wrong-bucket bindings later.
        let request = SecurityScorecardTrainingRequest {
            created_at: "2026-04-11T19:00:00+08:00".to_string(),
            training_runtime_root: None,
            market_scope: "A_SHARE".to_string(),
            instrument_scope: "ETF".to_string(),
            symbol_list: vec!["511010.SH".to_string()],
            market_symbol: Some("510300.SH".to_string()),
            sector_symbol: Some("511060.SH".to_string()),
            market_profile: Some("a_share_core".to_string()),
            sector_profile: Some("bond_etf_peer".to_string()),
            instrument_subscope: None,
            horizon_days: 10,
            target_head: "direction_head".to_string(),
            train_range: "2025-01-01..2025-06-30".to_string(),
            valid_range: "2025-07-01..2025-09-30".to_string(),
            test_range: "2025-10-01..2025-12-31".to_string(),
            train_samples_per_symbol: 6,
            valid_samples_per_symbol: 3,
            test_samples_per_symbol: 3,
            feature_set_version: "security_feature_snapshot.v1".to_string(),
            label_definition_version: "security_forward_outcome.v1".to_string(),
            lookback_days: 260,
            disclosure_limit: 8,
            stop_loss_pct: 0.01,
            target_return_pct: 0.015,
            external_proxy_inputs: None,
        };
        let trained_model = TrainedHeadModel::Classification(TrainedLogisticModel {
            intercept: 0.0,
            coefficients: Vec::new(),
        });

        let artifact = build_artifact(
            &request,
            &Vec::<FeatureModel>::new(),
            &trained_model,
            resolve_request_instrument_subscope(&request).as_deref(),
        );

        assert_eq!(
            artifact.instrument_subscope.as_deref(),
            Some("treasury_etf")
        );
        assert_eq!(
            artifact.model_id,
            "a_share_etf_treasury_etf_10d_direction_head"
        );
    }

    #[test]
    fn categorical_bins_include_other_bucket_for_holdout_generalization() {
        // 2026-04-12 UTC+08: Add a red test for categorical classification fallback,
        // because Scheme B now relies on pooled ETF training to generalize onto
        // unseen same-pool symbols without dropping the scorecard to incomplete.
        // Purpose: require categorical WOE bins to publish a neutral `__other__`
        // bucket so holdout categories can stay scorable.
        let samples = vec![
            TrainingSample {
                symbol: "513500.SH".to_string(),
                as_of_date: "2025-01-10".to_string(),
                split_name: "train".to_string(),
                label: 1.0,
                forward_return: 0.02,
                max_drawdown: 0.01,
                max_runup: 0.03,
                hit_upside_first: true,
                hit_stop_first: false,
                feature_values: BTreeMap::from([(
                    "integrated_stance".to_string(),
                    TrainingFeatureValue::Category("watchful_context".to_string()),
                )]),
            },
            TrainingSample {
                symbol: "513100.SH".to_string(),
                as_of_date: "2025-01-17".to_string(),
                split_name: "train".to_string(),
                label: 0.0,
                forward_return: -0.01,
                max_drawdown: 0.02,
                max_runup: 0.01,
                hit_upside_first: false,
                hit_stop_first: true,
                feature_values: BTreeMap::from([(
                    "integrated_stance".to_string(),
                    TrainingFeatureValue::Category("cautious".to_string()),
                )]),
            },
        ];
        let train_refs = samples.iter().collect::<Vec<_>>();

        let bins = build_categorical_bins(&train_refs, "integrated_stance", 1.0, 1.0)
            .expect("categorical bins should build");

        assert!(
            bins.iter().any(|bin| {
                bin.bin_label == "__other__"
                    && bin.match_values == vec!["__other__".to_string()]
                    && (bin.woe - 0.0).abs() < 1e-9
            }),
            "categorical classification bins should expose a neutral other bucket"
        );
    }

    #[test]
    fn categorical_prediction_bins_include_other_bucket_for_holdout_generalization() {
        // 2026-04-12 UTC+08: Add a red test for categorical regression fallback,
        // because pooled ETF regression heads now need a governed fallback value
        // when a holdout symbol emits a new categorical state.
        // Purpose: require regression binning to keep a stable `__other__`
        // prediction bucket instead of returning no matched prediction.
        let samples = vec![
            TrainingSample {
                symbol: "512800.SH".to_string(),
                as_of_date: "2025-01-10".to_string(),
                split_name: "train".to_string(),
                label: 0.04,
                forward_return: 0.04,
                max_drawdown: 0.02,
                max_runup: 0.05,
                hit_upside_first: true,
                hit_stop_first: false,
                feature_values: BTreeMap::from([(
                    "integrated_stance".to_string(),
                    TrainingFeatureValue::Category("watchful_context".to_string()),
                )]),
            },
            TrainingSample {
                symbol: "512000.SH".to_string(),
                as_of_date: "2025-01-17".to_string(),
                split_name: "train".to_string(),
                label: -0.02,
                forward_return: -0.02,
                max_drawdown: 0.03,
                max_runup: 0.01,
                hit_upside_first: false,
                hit_stop_first: true,
                feature_values: BTreeMap::from([(
                    "integrated_stance".to_string(),
                    TrainingFeatureValue::Category("cautious".to_string()),
                )]),
            },
        ];
        let train_refs = samples.iter().collect::<Vec<_>>();

        let bins = build_categorical_prediction_bins(&train_refs, "integrated_stance")
            .expect("categorical prediction bins should build");

        assert!(
            bins.iter().any(|bin| {
                bin.bin_label == "__other__"
                    && bin.match_values == vec!["__other__".to_string()]
                    && bin.predicted_value.is_some()
            }),
            "categorical regression bins should expose an other bucket with fallback prediction"
        );
    }
}
