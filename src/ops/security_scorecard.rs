use std::collections::BTreeMap;
use std::fs;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use thiserror::Error;

use crate::ops::stock::security_decision_committee::SecurityDecisionCommitteeResult;
use crate::ops::stock::security_decision_evidence_bundle::build_evidence_bundle_feature_seed;

// 2026-04-09 CST: 这里新增正式评分卡对象合同，原因是用户明确要求证券评分卡不能再用主观分析分冒充正式治理对象；
// 目的：把评分结果升级为可落盘、可版本化、可复盘解释的正式 artifact。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityScorecardDocument {
    pub scorecard_id: String,
    pub contract_version: String,
    pub document_type: String,
    pub generated_at: String,
    pub symbol: String,
    pub analysis_date: String,
    pub decision_id: String,
    pub decision_ref: String,
    pub approval_ref: String,
    pub score_status: String,
    pub label_definition: String,
    pub model_binding: SecurityScorecardModelBinding,
    pub raw_feature_snapshot: BTreeMap<String, Value>,
    pub feature_contributions: Vec<SecurityScoreFeatureContribution>,
    pub group_breakdown: Vec<SecurityScoreGroupBreakdown>,
    pub base_score: Option<f64>,
    pub total_score: Option<f64>,
    pub success_probability: Option<f64>,
    pub quant_signal: String,
    pub quant_stance: String,
    pub recommendation_action: String,
    pub exposure_side: String,
    pub score_summary: String,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityScorecardModelBinding {
    pub model_id: Option<String>,
    pub model_version: Option<String>,
    pub training_window: Option<String>,
    pub oot_window: Option<String>,
    pub positive_label_definition: Option<String>,
    pub binning_version: Option<String>,
    pub coefficient_version: Option<String>,
    pub model_sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityScoreFeatureContribution {
    pub feature_name: String,
    pub group_name: String,
    pub raw_value: Value,
    pub bin_label: Option<String>,
    pub matched: bool,
    pub woe: Option<f64>,
    pub logit_contribution: Option<f64>,
    pub points: f64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityScoreGroupBreakdown {
    pub group_name: String,
    pub feature_count: usize,
    pub point_total: f64,
}

// 2026-04-10 CST: 这里新增对客解释层合同，原因是用户明确指出客户不关心 scorecard 内部模型字段，
// 目的：把内部评分结果统一翻译成“看多/震荡/看空 + 依据 + 风险 + 关键位 + 动作建议”的业务输出，避免把模型实现细节直接暴露给客户。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityScorecardClientView {
    pub symbol: String,
    pub analysis_date: String,
    pub overall_score: u8,
    pub risk_level: String,
    pub tomorrow: SecurityScorecardClientHorizonView,
    pub short_term: SecurityScorecardClientHorizonView,
    pub swing_term: SecurityScorecardClientHorizonView,
    pub mid_long_term: SecurityScorecardClientHorizonView,
    pub detailed_horizons: Vec<SecurityScorecardClientHorizonView>,
    pub core_reasons: Vec<String>,
    pub risk_alerts: Vec<String>,
    pub key_levels: SecurityScorecardClientKeyLevels,
    pub action_advice: String,
}

// 2026-04-10 CST: 这里补对客关键价位合同，原因是用户要求客户看到“看哪些位”，
// 目的：让外层 Skill / 产品可以直接展示支撑位、阻力位和确认价，不必再回拆技术快照。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityScorecardClientKeyLevels {
    pub support_level_20: f64,
    pub resistance_level_20: f64,
    pub breakout_watch_level: f64,
}

// 2026-04-10 CST: 这里新增多周期投决卡视图，原因是用户确认不仅要看明天，还要一起看 5/10/20/30/60/180 日，
// 目的：把“赚钱概率 / 亏钱概率 / 盈亏比 / 交易性价比”沉成统一的周期指标对象，并支持上层按四层主视图或七档细节展示。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityScorecardClientHorizonView {
    pub horizon_label: String,
    pub trading_days: usize,
    pub profit_probability_pct: f64,
    pub loss_probability_pct: f64,
    pub profit_loss_ratio: f64,
    pub trade_value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SecurityScorecardBuildInput {
    pub generated_at: String,
    pub decision_id: String,
    pub decision_ref: String,
    pub approval_ref: String,
    pub scorecard_model_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityScorecardModelArtifact {
    pub model_id: String,
    pub model_version: String,
    pub label_definition: String,
    #[serde(default)]
    pub training_window: Option<String>,
    #[serde(default)]
    pub oot_window: Option<String>,
    #[serde(default)]
    pub positive_label_definition: Option<String>,
    #[serde(default)]
    pub binning_version: Option<String>,
    #[serde(default)]
    pub coefficient_version: Option<String>,
    #[serde(default)]
    pub model_sha256: Option<String>,
    #[serde(default)]
    pub intercept: Option<f64>,
    pub base_score: f64,
    #[serde(default)]
    pub features: Vec<SecurityScorecardModelFeatureSpec>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityScorecardModelFeatureSpec {
    pub feature_name: String,
    pub group_name: String,
    #[serde(default)]
    pub bins: Vec<SecurityScorecardModelBin>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityScorecardModelBin {
    pub bin_label: String,
    #[serde(default)]
    pub match_values: Vec<String>,
    #[serde(default)]
    pub min_inclusive: Option<f64>,
    #[serde(default)]
    pub max_exclusive: Option<f64>,
    #[serde(default)]
    pub woe: Option<f64>,
    #[serde(default)]
    pub logit_contribution: Option<f64>,
    #[serde(default)]
    pub points: f64,
}

#[derive(Debug, Error)]
pub enum SecurityScorecardError {
    #[error("证券评分卡构建失败: {0}")]
    Build(String),
}

pub fn build_security_scorecard(
    committee: &SecurityDecisionCommitteeResult,
    input: &SecurityScorecardBuildInput,
) -> Result<SecurityScorecardDocument, SecurityScorecardError> {
    // 2026-04-10 CST: ?? scorecard ?????
    // ????????????????????? artifact ??????????
    let raw_feature_snapshot = build_raw_feature_snapshot(committee);
    let scorecard_id = format!("scorecard-{}", input.decision_id);
    let recommendation_action = committee.decision_card.recommendation_action.clone();
    let exposure_side = committee.decision_card.exposure_side.clone();
    let fallback_quant_signal = derive_quant_signal(None, &recommendation_action);
    let fallback_quant_stance = derive_quant_stance(None, &exposure_side);

    let Some(model_path) = input
        .scorecard_model_path
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    else {
        return Ok(SecurityScorecardDocument {
            scorecard_id,
            contract_version: "security_scorecard.v1".to_string(),
            document_type: "security_scorecard".to_string(),
            generated_at: input.generated_at.clone(),
            symbol: committee.symbol.clone(),
            analysis_date: committee.analysis_date.clone(),
            decision_id: input.decision_id.clone(),
            decision_ref: input.decision_ref.clone(),
            approval_ref: input.approval_ref.clone(),
            score_status: "model_unavailable".to_string(),
            label_definition: "horizon_10d_stop_5pct_target_10pct".to_string(),
            model_binding: SecurityScorecardModelBinding {
                model_id: None,
                model_version: None,
                training_window: None,
                oot_window: None,
                positive_label_definition: None,
                binning_version: None,
                coefficient_version: None,
                model_sha256: None,
            },
            raw_feature_snapshot,
            feature_contributions: Vec::new(),
            group_breakdown: Vec::new(),
            base_score: None,
            total_score: None,
            success_probability: None,
            quant_signal: fallback_quant_signal,
            quant_stance: fallback_quant_stance,
            recommendation_action,
            exposure_side,
            score_summary: "??? scorecard ?? artifact??????????????????".to_string(),
            limitations: vec![
                "????? scorecard ?? artifact????????WOE ??????".to_string(),
                "???????????????????????????????".to_string(),
            ],
        });
    };

    let model = load_scorecard_model(model_path)?;
    let contributions = score_features(&model, &raw_feature_snapshot);
    let total_points = contributions.iter().map(|item| item.points).sum::<f64>();
    let total_score = model.base_score + total_points;
    let group_breakdown = build_group_breakdown(&contributions);
    let score_status = if contributions.iter().all(|item| item.matched) {
        "ready"
    } else {
        "feature_incomplete"
    };
    let limitations = if score_status == "ready" {
        Vec::new()
    } else {
        vec!["??????????????????????????????".to_string()]
    };
    let quant_signal = derive_quant_signal(Some(score_status), &recommendation_action);
    let quant_stance = derive_quant_stance(Some(score_status), &exposure_side);
    let success_probability = model.intercept.map(|intercept| {
        logistic(
            intercept
                + contributions
                    .iter()
                    .filter_map(|item| item.logit_contribution)
                    .sum::<f64>(),
        )
    });

    Ok(SecurityScorecardDocument {
        scorecard_id,
        contract_version: "security_scorecard.v1".to_string(),
        document_type: "security_scorecard".to_string(),
        generated_at: input.generated_at.clone(),
        symbol: committee.symbol.clone(),
        analysis_date: committee.analysis_date.clone(),
        decision_id: input.decision_id.clone(),
        decision_ref: input.decision_ref.clone(),
        approval_ref: input.approval_ref.clone(),
        score_status: score_status.to_string(),
        label_definition: model.label_definition.clone(),
        model_binding: SecurityScorecardModelBinding {
            model_id: Some(model.model_id.clone()),
            model_version: Some(model.model_version.clone()),
            training_window: model.training_window.clone(),
            oot_window: model.oot_window.clone(),
            positive_label_definition: model.positive_label_definition.clone(),
            binning_version: model.binning_version.clone(),
            coefficient_version: model.coefficient_version.clone(),
            model_sha256: model.model_sha256.clone(),
        },
        raw_feature_snapshot,
        feature_contributions: contributions,
        group_breakdown,
        base_score: Some(model.base_score),
        total_score: Some(total_score),
        success_probability,
        quant_signal,
        quant_stance,
        recommendation_action,
        exposure_side,
        score_summary: format!("???????? {}:{} ?????", model.model_id, model.model_version),
        limitations,
    })
}

// 2026-04-10 CST: 这里新增 scorecard -> client view 的转换入口，原因是对客输出不能继续直接暴露模型字段，
// 目的：在不破坏内部落盘 scorecard 文档的前提下，给 submit_approval / Skill 返回稳定的人话解释层。
pub fn build_security_scorecard_client_view(
    committee: &SecurityDecisionCommitteeResult,
    scorecard: &SecurityScorecardDocument,
) -> SecurityScorecardClientView {
    let stock_analysis = &committee.evidence_bundle.technical_context.stock_analysis;
    let horizon_inputs = [1_usize, 5, 10, 20, 30, 60, 180];
    let detailed_horizons = horizon_inputs
        .into_iter()
        .map(|trading_days| build_horizon_view(committee, scorecard, trading_days))
        .collect::<Vec<_>>();
    let tomorrow = select_horizon_view(&detailed_horizons, 1);
    let short_term = select_horizon_view(&detailed_horizons, 5);
    let swing_term = average_horizon_views("波段", &[20, 30], &detailed_horizons);
    let mid_long_term = average_horizon_views("中长线", &[60, 180], &detailed_horizons);
    let overall_score = derive_overall_score(committee, scorecard);
    let risk_level = derive_risk_level(committee);
    let mut core_reasons = build_core_reasons(committee, scorecard);
    let mut risk_alerts = build_risk_alerts(committee, scorecard);
    let action_advice = build_action_advice(committee, &tomorrow, &swing_term, &risk_level);

    dedupe_string_list(&mut core_reasons);
    dedupe_string_list(&mut risk_alerts);

    SecurityScorecardClientView {
        symbol: committee.symbol.clone(),
        analysis_date: committee.analysis_date.clone(),
        overall_score,
        risk_level,
        tomorrow,
        short_term,
        swing_term,
        mid_long_term,
        detailed_horizons,
        core_reasons,
        risk_alerts,
        key_levels: SecurityScorecardClientKeyLevels {
            support_level_20: stock_analysis.indicator_snapshot.support_level_20,
            resistance_level_20: stock_analysis.indicator_snapshot.resistance_level_20,
            breakout_watch_level: stock_analysis
                .indicator_snapshot
                .resistance_level_20
                .max(stock_analysis.indicator_snapshot.close),
        },
        action_advice,
    }
}

fn load_scorecard_model(
    path: &str,
) -> Result<SecurityScorecardModelArtifact, SecurityScorecardError> {
    let payload = fs::read(path).map_err(|error| {
        SecurityScorecardError::Build(format!("failed to read scorecard model: {error}"))
    })?;
    serde_json::from_slice(&payload).map_err(|error| {
        SecurityScorecardError::Build(format!("failed to parse scorecard model: {error}"))
    })
}

fn build_raw_feature_snapshot(
    committee: &SecurityDecisionCommitteeResult,
) -> BTreeMap<String, Value> {
    // 2026-04-10 CST: 这里改为先复用 evidence bundle 的统一特征种子，原因是训练链已经明确以 evidence seed 作为正式原子特征合同；
    // 目的：让线上 scorecard 与 training / snapshot 共享同一批底层特征，避免再从 committee 摘要字段里拼一套缩水口径。
    let warn_count = committee
        .risk_gates
        .iter()
        .filter(|gate| gate.result == "warn")
        .count();
    let mut snapshot = build_evidence_bundle_feature_seed(&committee.evidence_bundle);

    // 2026-04-10 CST: 这里继续补 committee 专属治理字段，原因是线上 scorecard 仍需保留投票与风控摘要给 chair / package / 审批链引用；
    // 目的：在复用统一 evidence seed 的同时，不丢掉评分卡现有治理场景依赖的决策态字段。
    snapshot.insert("warn_count".to_string(), json!(warn_count));
    snapshot.insert(
        "majority_vote".to_string(),
        Value::String(committee.vote_tally.majority_vote.clone()),
    );
    snapshot.insert(
        "majority_count".to_string(),
        json!(committee.vote_tally.majority_count),
    );
    snapshot.insert(
        "buy_count".to_string(),
        json!(committee.vote_tally.buy_count),
    );
    snapshot.insert(
        "hold_count".to_string(),
        json!(committee.vote_tally.hold_count),
    );
    snapshot.insert(
        "reduce_count".to_string(),
        json!(committee.vote_tally.reduce_count),
    );
    snapshot.insert(
        "avoid_count".to_string(),
        json!(committee.vote_tally.avoid_count),
    );
    snapshot.insert(
        "abstain_count".to_string(),
        json!(committee.vote_tally.abstain_count),
    );
    snapshot.insert(
        "risk_veto_status".to_string(),
        Value::String(committee.risk_veto.status.clone()),
    );
    snapshot.insert(
        "committee_confidence_score".to_string(),
        json!(committee.decision_card.confidence_score),
    );
    snapshot.insert(
        "recommendation_action".to_string(),
        Value::String(committee.decision_card.recommendation_action.clone()),
    );
    snapshot.insert(
        "exposure_side".to_string(),
        Value::String(committee.decision_card.exposure_side.clone()),
    );
    snapshot
}

fn score_features(
    model: &SecurityScorecardModelArtifact,
    raw_feature_snapshot: &BTreeMap<String, Value>,
) -> Vec<SecurityScoreFeatureContribution> {
    model
        .features
        .iter()
        .map(|feature| {
            let raw_value = raw_feature_snapshot
                .get(&feature.feature_name)
                .cloned()
                .unwrap_or(Value::Null);
            let matched_bin = feature
                .bins
                .iter()
                .find(|bin| value_matches_bin(&raw_value, bin));
            match matched_bin {
                Some(bin) => SecurityScoreFeatureContribution {
                    feature_name: feature.feature_name.clone(),
                    group_name: feature.group_name.clone(),
                    raw_value,
                    bin_label: Some(bin.bin_label.clone()),
                    matched: true,
                    woe: bin.woe,
                    logit_contribution: bin.logit_contribution,
                    points: bin.points,
                },
                None => SecurityScoreFeatureContribution {
                    feature_name: feature.feature_name.clone(),
                    group_name: feature.group_name.clone(),
                    raw_value,
                    bin_label: None,
                    matched: false,
                    woe: None,
                    logit_contribution: None,
                    points: 0.0,
                },
            }
        })
        .collect()
}

fn value_matches_bin(value: &Value, bin: &SecurityScorecardModelBin) -> bool {
    if !bin.match_values.is_empty() {
        // 2026-04-10 CST: 这里补齐 bool/string 分箱匹配，原因是训练链会把 bool 特征按字符串类别 `true/false` 建箱，
        // 目的：让线上 scorecard 对布尔类别特征的命中规则与训练保持一致，避免 evidence seed 已补齐后仍然卡在 bool 特征未命中。
        let Some(raw) = categorical_value_as_string(value) else {
            return false;
        };
        return bin.match_values.iter().any(|item| item == raw);
    }

    let Some(number) = value.as_f64() else {
        return false;
    };
    let lower_ok = bin
        .min_inclusive
        .map(|lower| number >= lower)
        .unwrap_or(true);
    let upper_ok = bin
        .max_exclusive
        .map(|upper| number < upper)
        .unwrap_or(true);
    lower_ok && upper_ok
}

fn categorical_value_as_string(value: &Value) -> Option<&str> {
    match value {
        Value::String(raw) => Some(raw.as_str()),
        Value::Bool(true) => Some("true"),
        Value::Bool(false) => Some("false"),
        _ => None,
    }
}

fn build_group_breakdown(
    contributions: &[SecurityScoreFeatureContribution],
) -> Vec<SecurityScoreGroupBreakdown> {
    let mut grouped = BTreeMap::<String, (usize, f64)>::new();
    for contribution in contributions {
        let entry = grouped
            .entry(contribution.group_name.clone())
            .or_insert((0_usize, 0.0_f64));
        entry.0 += 1;
        entry.1 += contribution.points;
    }
    grouped
        .into_iter()
        .map(
            |(group_name, (feature_count, point_total))| SecurityScoreGroupBreakdown {
                group_name,
                feature_count,
                point_total,
            },
        )
        .collect()
}

fn logistic(value: f64) -> f64 {
    1.0 / (1.0 + (-value).exp())
}

// 2026-04-10 CST: 这里统一生成多周期赚钱/亏钱指标，原因是用户明确要求投决卡要收敛为概率与交易性价比，而不是方向词，
// 目的：复用现有 scorecard / committee / 风险闸门，给 1/5/10/20/30/60/180 日输出轻量但稳定的客户指标。
fn build_horizon_view(
    committee: &SecurityDecisionCommitteeResult,
    scorecard: &SecurityScorecardDocument,
    trading_days: usize,
) -> SecurityScorecardClientHorizonView {
    let profit_probability = derive_profit_probability(committee, scorecard, trading_days);
    let loss_probability = derive_loss_probability(committee, trading_days, profit_probability);
    let profit_loss_ratio = derive_profit_loss_ratio(committee, trading_days);
    let trade_value = classify_trade_value(profit_probability, loss_probability, profit_loss_ratio);

    SecurityScorecardClientHorizonView {
        horizon_label: horizon_label(trading_days).to_string(),
        trading_days,
        profit_probability_pct: round_pct(profit_probability * 100.0),
        loss_probability_pct: round_pct(loss_probability * 100.0),
        profit_loss_ratio: round_ratio(profit_loss_ratio),
        trade_value,
    }
}

// 2026-04-10 CST: 这里把多项周期结果聚合成主视图项，原因是用户前台不适合直接同时看七档结果，
// 目的：保留内部七档细节的同时，对客主展示只暴露“明日 / 短线 / 波段 / 中长线”四层。
fn average_horizon_views(
    label: &str,
    trading_days: &[usize],
    detailed_horizons: &[SecurityScorecardClientHorizonView],
) -> SecurityScorecardClientHorizonView {
    let selected = detailed_horizons
        .iter()
        .filter(|item| trading_days.contains(&item.trading_days))
        .collect::<Vec<_>>();
    let count = selected.len().max(1) as f64;
    let profit_probability_pct = selected
        .iter()
        .map(|item| item.profit_probability_pct)
        .sum::<f64>()
        / count;
    let loss_probability_pct = selected
        .iter()
        .map(|item| item.loss_probability_pct)
        .sum::<f64>()
        / count;
    let profit_loss_ratio = selected
        .iter()
        .map(|item| item.profit_loss_ratio)
        .sum::<f64>()
        / count;
    let trade_value = classify_trade_value(
        profit_probability_pct / 100.0,
        loss_probability_pct / 100.0,
        profit_loss_ratio,
    );

    SecurityScorecardClientHorizonView {
        horizon_label: label.to_string(),
        trading_days: *trading_days.last().unwrap_or(&0),
        profit_probability_pct: round_pct(profit_probability_pct),
        loss_probability_pct: round_pct(loss_probability_pct),
        profit_loss_ratio: round_ratio(profit_loss_ratio),
        trade_value,
    }
}

fn select_horizon_view(
    detailed_horizons: &[SecurityScorecardClientHorizonView],
    trading_days: usize,
) -> SecurityScorecardClientHorizonView {
    detailed_horizons
        .iter()
        .find(|item| item.trading_days == trading_days)
        .cloned()
        .unwrap_or(SecurityScorecardClientHorizonView {
            horizon_label: horizon_label(trading_days).to_string(),
            trading_days,
            profit_probability_pct: 50.0,
            loss_probability_pct: 50.0,
            profit_loss_ratio: 1.0,
            trade_value: "中".to_string(),
        })
}

// 2026-04-10 CST: 这里计算总体分数，原因是用户明确指出投决卡仍然需要一个总分来承接平衡计分卡主轴，
// 目的：把内部原始 total_score / success_probability / confidence 等信息压缩成 0~100 的客户综合分。
fn derive_overall_score(
    committee: &SecurityDecisionCommitteeResult,
    scorecard: &SecurityScorecardDocument,
) -> u8 {
    let readiness = if scorecard.score_status == "ready" {
        1.0
    } else {
        0.65
    };
    let confidence_component = committee.decision_card.confidence_score.clamp(0.0, 1.0);
    let model_component = calibrated_model_probability(scorecard);
    let stance_bonus = stance_edge(committee) * 10.0;
    let fail_penalty = committee
        .risk_gates
        .iter()
        .filter(|gate| gate.result == "fail")
        .count() as f64
        * 10.0;
    let warn_penalty = committee
        .risk_gates
        .iter()
        .filter(|gate| gate.result == "warn")
        .count() as f64
        * 3.0;
    let score = 35.0
        + confidence_component * 28.0
        + model_component * 22.0
        + readiness * 10.0
        + stance_bonus
        - fail_penalty
        - warn_penalty;
    score.round().clamp(0.0, 100.0) as u8
}

// 2026-04-10 CST: 这里统一风控强弱为风险等级，原因是客户更容易理解“低 / 中 / 高风险”，
// 目的：把 risk_gates / volatility / 数据完整性压成一个简洁标签，减少客户端自行解释成本。
fn derive_risk_level(committee: &SecurityDecisionCommitteeResult) -> String {
    let fail_count = committee
        .risk_gates
        .iter()
        .filter(|gate| gate.result == "fail")
        .count();
    let warn_count = committee
        .risk_gates
        .iter()
        .filter(|gate| gate.result == "warn")
        .count();
    let stock_analysis = &committee.evidence_bundle.technical_context.stock_analysis;
    if fail_count >= 1
        || stock_analysis.volatility_state == "expanding"
        || committee.decision_card.status == "blocked"
    {
        return "高".to_string();
    }
    if warn_count >= 1 || committee.decision_card.status == "needs_more_evidence" {
        return "中".to_string();
    }
    "低".to_string()
}

fn derive_profit_probability(
    committee: &SecurityDecisionCommitteeResult,
    scorecard: &SecurityScorecardDocument,
    trading_days: usize,
) -> f64 {
    let confidence_edge = committee.decision_card.confidence_score.clamp(0.0, 1.0) - 0.5;
    let model_edge = calibrated_model_probability(scorecard) - 0.5;
    let horizon_edge = horizon_edge_multiplier(trading_days);
    let gate_penalty = gate_penalty(committee);
    let technical_edge = technical_edge(committee);
    let raw_probability = 0.50
        + confidence_edge * 0.22
        + model_edge * 0.30
        + stance_edge(committee) * horizon_edge
        + technical_edge * 0.18
        - gate_penalty;
    raw_probability.clamp(0.08, 0.88)
}

fn derive_loss_probability(
    committee: &SecurityDecisionCommitteeResult,
    trading_days: usize,
    profit_probability: f64,
) -> f64 {
    let flat_probability = flat_probability(committee, trading_days);
    (1.0 - flat_probability - profit_probability).clamp(0.05, 0.88)
}

fn derive_profit_loss_ratio(
    committee: &SecurityDecisionCommitteeResult,
    trading_days: usize,
) -> f64 {
    let (target_return_pct, stop_loss_pct, _) = extract_risk_profile(committee);
    let stock_analysis = &committee.evidence_bundle.technical_context.stock_analysis;
    let horizon_gain = match trading_days {
        1 => 0.20,
        5 => 0.55,
        10 => 0.75,
        20 => 1.00,
        30 => 1.10,
        60 => 1.18,
        180 => 1.05,
        _ => 1.0,
    };
    let trend_bonus = match stock_analysis.trend_bias.as_str() {
        "bullish" => 0.12,
        "bearish" => -0.12,
        _ => 0.0,
    };
    let breakout_bonus = match stock_analysis.breakout_signal.as_str() {
        "confirmed_resistance_breakout" => 0.18,
        "confirmed_support_breakdown" => -0.18,
        _ => 0.0,
    };
    let upside = target_return_pct.max(0.01) * (1.0 + horizon_gain + trend_bonus + breakout_bonus);
    let downside = stop_loss_pct.max(0.01)
        * (1.0
            + if stock_analysis.volatility_state == "expanding" {
                0.25
            } else {
                0.0
            }
            + if committee.decision_card.status == "blocked" {
                0.18
            } else {
                0.0
            });
    (upside / downside).clamp(0.2, 3.5)
}

fn classify_trade_value(
    profit_probability: f64,
    loss_probability: f64,
    profit_loss_ratio: f64,
) -> String {
    let score = profit_probability * profit_loss_ratio - loss_probability;
    if score >= 0.35 {
        "高".to_string()
    } else if score >= 0.15 {
        "中".to_string()
    } else if score >= 0.0 {
        "偏低".to_string()
    } else {
        "低".to_string()
    }
}

fn horizon_label(trading_days: usize) -> &'static str {
    match trading_days {
        1 => "明日",
        5 => "5日",
        10 => "10日",
        20 => "20日",
        30 => "30日",
        60 => "60日",
        180 => "180日",
        _ => "多周期",
    }
}

fn calibrated_model_probability(scorecard: &SecurityScorecardDocument) -> f64 {
    let raw = scorecard.success_probability.unwrap_or(0.5).clamp(0.0, 1.0);
    (0.5 + (raw - 0.5) * 0.2).clamp(0.05, 0.95)
}

fn stance_edge(committee: &SecurityDecisionCommitteeResult) -> f64 {
    match committee
        .evidence_bundle
        .integrated_conclusion
        .stance
        .as_str()
    {
        "constructive" => 0.14,
        "watchful_positive" => 0.07,
        "cautious" => -0.10,
        "technical_only" => -0.04,
        _ => 0.0,
    }
}

fn technical_edge(committee: &SecurityDecisionCommitteeResult) -> f64 {
    let stock_analysis = &committee.evidence_bundle.technical_context.stock_analysis;
    let trend_edge = match stock_analysis.trend_bias.as_str() {
        "bullish" => 0.12,
        "bearish" => -0.12,
        _ => 0.0,
    };
    let strength_edge = match stock_analysis.trend_strength.as_str() {
        "strong" => 0.08,
        "moderate" => 0.04,
        "weak" => -0.02,
        _ => 0.0,
    };
    let breakout_edge = match stock_analysis.breakout_signal.as_str() {
        "confirmed_resistance_breakout" => 0.12,
        "confirmed_support_breakdown" => -0.12,
        "resistance_breakout_watch" => 0.04,
        "support_breakdown_watch" => -0.04,
        _ => 0.0,
    };
    trend_edge + strength_edge + breakout_edge
}

fn gate_penalty(committee: &SecurityDecisionCommitteeResult) -> f64 {
    let fail_penalty = committee
        .risk_gates
        .iter()
        .filter(|gate| gate.result == "fail")
        .count() as f64
        * 0.07;
    let warn_penalty = committee
        .risk_gates
        .iter()
        .filter(|gate| gate.result == "warn")
        .count() as f64
        * 0.025;
    fail_penalty + warn_penalty
}

fn horizon_edge_multiplier(trading_days: usize) -> f64 {
    match trading_days {
        1 => 0.55,
        5 => 0.80,
        10 => 0.92,
        20 => 1.05,
        30 => 1.08,
        60 => 0.96,
        180 => 0.72,
        _ => 1.0,
    }
}

fn flat_probability(committee: &SecurityDecisionCommitteeResult, trading_days: usize) -> f64 {
    let base: f64 = match trading_days {
        1 => 0.18,
        5 => 0.14,
        10 => 0.11,
        20 => 0.08,
        30 => 0.08,
        60 => 0.12,
        180 => 0.18,
        _ => 0.12,
    };
    let evidence_penalty: f64 = if committee.decision_card.status == "needs_more_evidence" {
        0.06
    } else {
        0.0
    };
    let blocked_bonus: f64 = if committee.decision_card.status == "blocked" {
        0.08
    } else {
        0.0
    };
    (base + evidence_penalty + blocked_bonus).clamp(0.06_f64, 0.30_f64)
}

fn extract_risk_profile(committee: &SecurityDecisionCommitteeResult) -> (f64, f64, f64) {
    let downside = parse_pct_range_floor(&committee.decision_card.downside_risk).unwrap_or(0.05);
    let upside =
        parse_pct_range_floor(&committee.decision_card.expected_return_range).unwrap_or(0.12);
    let ratio = if downside <= f64::EPSILON {
        1.0
    } else {
        upside / downside
    };
    (upside, downside, ratio)
}

fn parse_pct_range_floor(value: &str) -> Option<f64> {
    let token = value
        .split(|character: char| !character.is_ascii_digit() && character != '.')
        .find(|part| !part.trim().is_empty())?;
    token.parse::<f64>().ok().map(|number| number / 100.0)
}

fn round_pct(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

fn round_ratio(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

// 2026-04-10 CST: 这里集中生成对客“核心依据”，原因是客户需要知道为什么是这个判断，
// 目的：把技术面、信息面、门禁状态压缩成 2~4 条可读理由，而不是直接展示模型点数。
fn build_core_reasons(
    committee: &SecurityDecisionCommitteeResult,
    scorecard: &SecurityScorecardDocument,
) -> Vec<String> {
    let bundle = &committee.evidence_bundle;
    let stock_analysis = &bundle.technical_context.stock_analysis;
    let mut reasons = vec![
        bundle.integrated_conclusion.headline.clone(),
        stock_analysis.consultation_conclusion.headline.clone(),
    ];

    match stock_analysis.breakout_signal.as_str() {
        "confirmed_resistance_breakout" => reasons.push(format!(
            "价格已站上近 20 日阻力 {:.2}，短线属于突破确认结构。",
            stock_analysis.indicator_snapshot.resistance_level_20
        )),
        "confirmed_support_breakdown" => reasons.push(format!(
            "价格已跌破近 20 日支撑 {:.2}，短线属于破位结构。",
            stock_analysis.indicator_snapshot.support_level_20
        )),
        "resistance_breakout_watch" => reasons.push(format!(
            "价格接近近 20 日阻力 {:.2}，仍需放量确认后才能转强。",
            stock_analysis.indicator_snapshot.resistance_level_20
        )),
        _ => reasons.push(format!(
            "当前仍在近 20 日支撑 {:.2} 与阻力 {:.2} 区间内运行，方向还需要市场给出确认。",
            stock_analysis.indicator_snapshot.support_level_20,
            stock_analysis.indicator_snapshot.resistance_level_20
        )),
    }

    if bundle.fundamental_context.status == "available" {
        reasons.push(bundle.fundamental_context.headline.clone());
    }
    if bundle.disclosure_context.status == "available" {
        reasons.push(bundle.disclosure_context.headline.clone());
    }
    if scorecard.score_status != "ready" {
        reasons.push("当前对客结论已生成，但底层评分特征仍有未完全命中的项目。".to_string());
    }

    reasons.into_iter().take(4).collect()
}

// 2026-04-10 CST: 这里集中生成对客“风险提示”，原因是用户要求客户能直接看到不该做什么、为什么先别动，
// 目的：把 risk_gate / data_gap / 技术风险统一翻译成风险语言，而不是让客户自行解读内部阻断字段。
fn build_risk_alerts(
    committee: &SecurityDecisionCommitteeResult,
    scorecard: &SecurityScorecardDocument,
) -> Vec<String> {
    let bundle = &committee.evidence_bundle;
    let stock_analysis = &bundle.technical_context.stock_analysis;
    let mut alerts = Vec::new();

    for gate in committee
        .risk_gates
        .iter()
        .filter(|gate| gate.result == "fail" || gate.result == "warn")
    {
        alerts.push(format!("{}：{}", gate.gate_name, gate.reason));
    }

    alerts.extend(
        bundle
            .integrated_conclusion
            .risk_flags
            .iter()
            .take(2)
            .cloned(),
    );
    alerts.extend(
        stock_analysis
            .consultation_conclusion
            .risk_flags
            .iter()
            .take(2)
            .cloned(),
    );
    alerts.extend(scorecard.limitations.iter().take(1).cloned());

    if alerts.is_empty() {
        alerts.push("当前未出现新的硬性风险闸门，但仍需围绕关键位观察确认。".to_string());
    }

    alerts
}

// 2026-04-10 CST: 这里统一收集阻断原因，原因是客户需要知道“卡在哪一关”，
// 目的：把分散在 risk_gates / decision_card.status 中的门禁原因收成一个可直接展示的列表。
// 2026-04-10 CST: 这里生成客户动作建议，原因是最终输出要回答“下一步怎么做”，
// 目的：基于判断和门禁状态，直接给出观察/持有/等待突破/回避这类动作，而不是停留在模型状态说明。
fn build_action_advice(
    committee: &SecurityDecisionCommitteeResult,
    tomorrow: &SecurityScorecardClientHorizonView,
    swing_term: &SecurityScorecardClientHorizonView,
    risk_level: &str,
) -> String {
    let stock_analysis = &committee.evidence_bundle.technical_context.stock_analysis;
    let has_risk_reward_fail = committee
        .risk_gates
        .iter()
        .any(|gate| gate.gate_name == "risk_reward_gate" && gate.result == "fail");
    if has_risk_reward_fail {
        return format!(
            "当前先观察，不追价；只有放量站稳 {:.2} 上方后，再评估是否转入试探仓位。当前明日赚钱概率 {:.1}%，但交易性价比仍未过关。",
            stock_analysis.indicator_snapshot.resistance_level_20, tomorrow.profit_probability_pct
        );
    }
    if committee.decision_card.status == "needs_more_evidence" {
        return "当前先等待信息补齐或下一轮确认，不建议直接执行。".to_string();
    }

    if tomorrow.trade_value == "高" && swing_term.trade_value != "低" && risk_level != "高" {
        return format!(
            "可进入重点跟踪；若回踩 {:.2} 不破且量能维持，可按试探仓位处理。",
            stock_analysis.indicator_snapshot.resistance_level_20
        );
    }
    if tomorrow.loss_probability_pct >= 55.0 || risk_level == "高" {
        return format!(
            "当前优先控制风险；若无法收回 {:.2} 上方，应继续按偏防守思路处理。",
            stock_analysis.indicator_snapshot.support_level_20,
        );
    }
    format!(
        "当前按区间震荡与等待确认处理，重点观察 {:.2} 支撑与 {:.2} 阻力谁先被有效突破。",
        stock_analysis.indicator_snapshot.support_level_20,
        stock_analysis.indicator_snapshot.resistance_level_20
    )
}

fn dedupe_string_list(values: &mut Vec<String>) {
    let mut deduped = Vec::new();
    for value in values.drain(..) {
        if !deduped.contains(&value) {
            deduped.push(value);
        }
    }
    *values = deduped;
}

fn derive_quant_signal(score_status: Option<&str>, fallback_action: &str) -> String {
    match score_status {
        Some("ready") => format!("quant_{fallback_action}"),
        Some("feature_incomplete") => "quant_incomplete".to_string(),
        Some(_) => "quant_unavailable".to_string(),
        None => "quant_unavailable".to_string(),
    }
}

fn derive_quant_stance(score_status: Option<&str>, fallback_exposure_side: &str) -> String {
    match score_status {
        Some("ready") => fallback_exposure_side.to_string(),
        Some("feature_incomplete") => "guarded".to_string(),
        Some(_) => "unavailable".to_string(),
        None => "unavailable".to_string(),
    }
}
