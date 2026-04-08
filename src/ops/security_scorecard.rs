use std::collections::BTreeMap;
use std::fs;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use thiserror::Error;

use crate::ops::stock::security_decision_committee::SecurityDecisionCommitteeResult;

// 2026-04-09 CST: 这里新增正式评分卡对象合同，原因是用户明确要求证券评分卡不能再用主观分析分冒充正式治理对象；
// 目的：把评分结果升级为可落盘、可版本化、可进入 package、可做后续复盘与验真的正式 artifact。
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
    // 2026-04-09 CST: 这里新增量化信号字段，原因是 Task 1 要把 scorecard 正式语义收敛为量化线输出，
    // 目的：让主席线与后续总卡明确消费 quant_signal，而不是把 recommendation_action 误当成最终正式决议。
    pub quant_signal: String,
    // 2026-04-09 CST: 这里新增量化立场字段，原因是用户要求量化线和主席线彻底分开，
    // 目的：沉淀 scorecard 自身的量化方向语义，避免后续继续复用旧字段造成混线。
    pub quant_stance: String,
    pub recommendation_action: String,
    pub exposure_side: String,
    pub score_summary: String,
    pub limitations: Vec<String>,
}

// 2026-04-09 CST: 这里显式保留模型绑定元数据，原因是评分卡后续必须能追溯到“哪一版分箱/系数/训练窗口”；
// 目的：让 package、verify 和复盘都能基于稳定字段追踪模型来源，而不是退回口头说明。
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

// 2026-04-09 CST: 这里记录单特征贡献明细，原因是用户要求以后必须能解释“这个分数是怎么算出来的”；
// 目的：把原始值、命中的分箱、WOE、points 和归因分组一起落盘，避免评分卡再次退化成黑盒总分。
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

// 2026-04-09 CST: 这里新增分组归因摘要，原因是正式评分卡后续仍需要保留 T/F/E/V 这类用户可读归因视角；
// 目的：在不手工指定主观权重的前提下，把模型分数按组做聚合展示，方便复盘和后续调参。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityScoreGroupBreakdown {
    pub group_name: String,
    pub feature_count: usize,
    pub point_total: f64,
}

// 2026-04-09 CST: 这里定义评分卡构建输入，原因是评分卡既依赖投决结果，也依赖运行时锚点与可选模型路径；
// 目的：把落盘所需元数据集中收口，避免 submit_approval 继续膨胀成手写 JSON 拼装器。
#[derive(Debug, Clone, PartialEq)]
pub struct SecurityScorecardBuildInput {
    pub generated_at: String,
    pub decision_id: String,
    pub decision_ref: String,
    pub approval_ref: String,
    pub scorecard_model_path: Option<String>,
}

// 2026-04-09 CST: 这里新增模型 artifact 合同，原因是本轮正式边界是“消费离线训练产物”，不是运行时手工拍权重；
// 目的：为后续真实分箱/WOE/贡献回归结果预留稳定输入格式，同时本轮先支持无模型时的正式退化语义。
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
    let raw_feature_snapshot = build_raw_feature_snapshot(committee);
    let scorecard_id = format!("scorecard-{}", input.decision_id);
    let recommendation_action = committee.decision_card.recommendation_action.clone();
    let exposure_side = committee.decision_card.exposure_side.clone();
    // 2026-04-09 CST: 这里先生成量化线自身字段，原因是 Task 1 要明确 scorecard 是量化线，不是主席线；
    // 目的：即便当前仍保留 recommendation_action / exposure_side 兼容字段，也要同时落正式 quant_signal / quant_stance。
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
            score_summary:
                "未提供评分卡模型 artifact，系统已落正式 scorecard 对象，但不会伪造主观分数。"
                    .to_string(),
            limitations: vec![
                "当前未提供评分卡模型 artifact，无法执行分箱、WOE 与点数累加。".to_string(),
                "本轮只保留正式对象、原始特征快照与治理链挂接，不输出伪造分数。".to_string(),
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
        vec!["部分特征未命中模型分箱，当前评分结果仅可用于治理留痕与复核。".to_string()]
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
        score_summary: format!(
            "评分卡已基于模型 {}:{} 完成打分。",
            model.model_id, model.model_version
        ),
        limitations,
    })
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
    let warn_count = committee
        .risk_gates
        .iter()
        .filter(|gate| gate.result == "warn")
        .count();
    let mut snapshot = BTreeMap::new();
    snapshot.insert(
        "integrated_stance".to_string(),
        Value::String(
            committee
                .evidence_bundle
                .integrated_conclusion
                .stance
                .clone(),
        ),
    );
    snapshot.insert(
        "technical_alignment".to_string(),
        Value::String(
            committee
                .evidence_bundle
                .technical_context
                .contextual_conclusion
                .alignment
                .clone(),
        ),
    );
    snapshot.insert(
        "overall_evidence_status".to_string(),
        Value::String(
            committee
                .evidence_bundle
                .evidence_quality
                .overall_status
                .clone(),
        ),
    );
    snapshot.insert(
        "fundamental_status".to_string(),
        Value::String(committee.evidence_bundle.fundamental_context.status.clone()),
    );
    snapshot.insert(
        "disclosure_status".to_string(),
        Value::String(committee.evidence_bundle.disclosure_context.status.clone()),
    );
    snapshot.insert(
        "data_gap_count".to_string(),
        json!(committee.evidence_bundle.data_gaps.len()),
    );
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
        let Some(raw) = value.as_str() else {
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

// 2026-04-09 CST: 这里集中维护量化信号映射，原因是 scorecard 已被明确为量化线正式输出，
// 目的：让 chair_resolution / master_scorecard / replay 都消费同一套 quant_signal 口径，避免各处手写导致语义漂移。
fn derive_quant_signal(score_status: Option<&str>, fallback_action: &str) -> String {
    match score_status {
        Some("ready") => format!("quant_{fallback_action}"),
        Some("feature_incomplete") => "quant_incomplete".to_string(),
        Some(_) => "quant_unavailable".to_string(),
        None => "quant_unavailable".to_string(),
    }
}

// 2026-04-09 CST: 这里集中维护量化立场映射，原因是 scorecard 与主席线都会读取这一层方向语义，
// 目的：把“量化怎么想”和“主席最终怎么裁决”明确拆开，同时保持回放和复盘口径稳定。
fn derive_quant_stance(score_status: Option<&str>, fallback_exposure_side: &str) -> String {
    match score_status {
        Some("ready") => fallback_exposure_side.to_string(),
        Some("feature_incomplete") => "guarded".to_string(),
        Some(_) => "unavailable".to_string(),
        None => "unavailable".to_string(),
    }
}
