use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ops::stock::security_decision_committee::{
    SecurityDecisionCommitteeError, SecurityDecisionCommitteeRequest,
    SecurityDecisionCommitteeResult, security_decision_committee,
};
use crate::ops::stock::security_scorecard::{
    SecurityScorecardBuildInput, SecurityScorecardDocument, SecurityScorecardError,
    build_security_scorecard,
};

// 2026-04-09 CST: 这里新增主席裁决请求合同，原因是 Task 1 要把“最终正式决议”从投委会线中拆出来，
// 目的：让主席线拥有独立 Tool 入口，后续 package / verify / audit 都可以围绕这条线接入，而不是继续把 committee 当最终出口。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityChairResolutionRequest {
    pub symbol: String,
    #[serde(default)]
    pub market_symbol: Option<String>,
    #[serde(default)]
    pub sector_symbol: Option<String>,
    #[serde(default)]
    pub market_profile: Option<String>,
    #[serde(default)]
    pub sector_profile: Option<String>,
    #[serde(default)]
    pub as_of_date: Option<String>,
    #[serde(default = "default_lookback_days")]
    pub lookback_days: usize,
    #[serde(default = "default_disclosure_limit")]
    pub disclosure_limit: usize,
    #[serde(default = "default_stop_loss_pct")]
    pub stop_loss_pct: f64,
    #[serde(default = "default_target_return_pct")]
    pub target_return_pct: f64,
    #[serde(default = "default_min_risk_reward_ratio")]
    pub min_risk_reward_ratio: f64,
    #[serde(default = "default_created_at")]
    pub created_at: String,
    #[serde(default)]
    pub scorecard_model_path: Option<String>,
}

// 2026-04-09 CST: 这里新增主席正式裁决对象，原因是设计已经明确主席才是唯一正式决议出口，
// 目的：把最终动作、理由、量化参考、投委会参考和执行约束沉淀成独立正式对象，避免继续沿用 committee / scorecard 代替最终结论。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityChairResolutionDocument {
    pub chair_resolution_id: String,
    pub contract_version: String,
    pub document_type: String,
    pub generated_at: String,
    pub symbol: String,
    pub analysis_date: String,
    pub decision_id: String,
    pub committee_session_ref: String,
    pub master_scorecard_ref: String,
    pub selected_action: String,
    pub selected_exposure_side: String,
    pub chair_reasoning: String,
    pub why_followed_quant: String,
    pub why_followed_committee: String,
    pub override_reason: Option<String>,
    pub execution_constraints: Vec<String>,
    pub final_confidence: f64,
    pub signed_off_at: String,
}

// 2026-04-09 CST: 这里新增主席裁决 Tool 的聚合返回对象，原因是当前阶段主席线仍需要显式暴露其读取的两条输入线，
// 目的：让测试、CLI 和后续治理层可以同屏看到 committee / scorecard / chair_resolution 三条线，验证它们是独立对象而非同一个输出换名。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SecurityChairResolutionResult {
    pub committee_result: SecurityDecisionCommitteeResult,
    pub scorecard: SecurityScorecardDocument,
    pub chair_resolution: SecurityChairResolutionDocument,
}

// 2026-04-09 CST: 这里单独定义主席裁决错误边界，原因是主席线同时依赖投委会线和量化线，
// 目的：给 dispatcher 一个稳定错误口径，不把内部依赖细节直接泄漏到外层。
#[derive(Debug, Error)]
pub enum SecurityChairResolutionError {
    #[error("security chair resolution committee preparation failed: {0}")]
    Committee(#[from] SecurityDecisionCommitteeError),
    #[error("security chair resolution scorecard preparation failed: {0}")]
    Scorecard(#[from] SecurityScorecardError),
}

// 2026-04-09 CST: 这里实现主席正式裁决总入口，原因是 Task 1 要先提供最小可用的主席线产品入口，
// 目的：在不破坏现有 submit_approval 主链的前提下，先让系统具备“独立主席对象输出最终动作”的正式能力。
pub fn security_chair_resolution(
    request: &SecurityChairResolutionRequest,
) -> Result<SecurityChairResolutionResult, SecurityChairResolutionError> {
    let committee_request = SecurityDecisionCommitteeRequest {
        symbol: request.symbol.clone(),
        market_symbol: request.market_symbol.clone(),
        sector_symbol: request.sector_symbol.clone(),
        market_profile: request.market_profile.clone(),
        sector_profile: request.sector_profile.clone(),
        as_of_date: request.as_of_date.clone(),
        lookback_days: request.lookback_days,
        disclosure_limit: request.disclosure_limit,
        stop_loss_pct: request.stop_loss_pct,
        target_return_pct: request.target_return_pct,
        min_risk_reward_ratio: request.min_risk_reward_ratio,
    };
    let committee_result = security_decision_committee(&committee_request)?;
    let scorecard = build_security_scorecard(
        &committee_result,
        &SecurityScorecardBuildInput {
            generated_at: request.created_at.clone(),
            decision_id: committee_result.decision_card.decision_id.clone(),
            decision_ref: committee_result.decision_card.decision_id.clone(),
            approval_ref: format!("chair-only-{}", committee_result.decision_card.decision_id),
            scorecard_model_path: request.scorecard_model_path.clone(),
        },
    )?;
    let chair_resolution =
        build_security_chair_resolution(&committee_result, &scorecard, &request.created_at);

    Ok(SecurityChairResolutionResult {
        committee_result,
        scorecard,
        chair_resolution,
    })
}

// 2026-04-09 CST: 这里集中构建主席正式裁决对象，原因是主席线虽然当前先做最小实现，
// 但“只读输入线、单独形成正式最终动作”的语义必须在一个稳定 builder 里固化；目的：为后续 package 挂接保留统一入口。
pub fn build_security_chair_resolution(
    committee_result: &SecurityDecisionCommitteeResult,
    scorecard: &SecurityScorecardDocument,
    generated_at: &str,
) -> SecurityChairResolutionDocument {
    let selected_action = committee_result.decision_card.recommendation_action.clone();
    let selected_exposure_side = committee_result.decision_card.exposure_side.clone();
    let signed_off_at = normalize_created_at(generated_at);
    let committee_session_ref = committee_result.committee_session_ref.clone();
    let master_scorecard_ref = scorecard.scorecard_id.clone();
    let execution_constraints =
        build_execution_constraints(committee_result, scorecard, &selected_action);

    SecurityChairResolutionDocument {
        chair_resolution_id: format!("chair-{}", committee_result.decision_card.decision_id),
        contract_version: "security_chair_resolution.v1".to_string(),
        document_type: "security_chair_resolution".to_string(),
        generated_at: signed_off_at.clone(),
        symbol: committee_result.symbol.clone(),
        analysis_date: committee_result.analysis_date.clone(),
        decision_id: committee_result.decision_card.decision_id.clone(),
        committee_session_ref,
        master_scorecard_ref,
        selected_action: selected_action.clone(),
        selected_exposure_side,
        chair_reasoning: build_chair_reasoning(committee_result, scorecard, &selected_action),
        why_followed_quant: build_quant_reason(scorecard),
        why_followed_committee: build_committee_reason(committee_result),
        override_reason: None,
        execution_constraints,
        final_confidence: committee_result.decision_card.confidence_score,
        signed_off_at,
    }
}

// 2026-04-09 CST: 这里把主席对量化线的采纳说明单独成句，原因是用户明确要求量化计分卡线与主席裁决线分离，
// 目的：后续复盘时可以清楚地区分“主席是否参考量化线”与“主席最终怎么裁决”这两件事。
fn build_quant_reason(scorecard: &SecurityScorecardDocument) -> String {
    if scorecard.score_status == "ready" {
        return format!(
            "主席已参考量化线，量化立场 `{}` / 量化信号 `{}` 已完成正式打分。",
            scorecard.quant_stance, scorecard.quant_signal
        );
    }

    format!(
        "主席未将量化线作为唯一依据，原因是 scorecard 当前状态为 `{}`，并保留了 {} 条限制说明。",
        scorecard.score_status,
        scorecard.limitations.len()
    )
}

// 2026-04-09 CST: 这里把主席对投委会线的采纳说明单独成句，原因是用户要求委员会建议与主席裁决同时保留且不可混线，
// 目的：让最终正式决议能够明确回溯到“投委会给了什么多数意见、风控席是什么状态”。
fn build_committee_reason(committee_result: &SecurityDecisionCommitteeResult) -> String {
    format!(
        "主席参考投委会多数票 `{}`（{} 票），并读取风控席状态 `{}`。",
        committee_result.vote_tally.majority_vote,
        committee_result.vote_tally.majority_count,
        committee_result.risk_veto.status
    )
}

// 2026-04-09 CST: 这里生成主席整体裁决理由，原因是正式最终决议不能只给动作、不解释依据，
// 目的：把“为什么采纳当前动作”固定成正式字段，便于审计与后续复盘纠偏。
fn build_chair_reasoning(
    committee_result: &SecurityDecisionCommitteeResult,
    scorecard: &SecurityScorecardDocument,
    selected_action: &str,
) -> String {
    format!(
        "主席在同读投委会线与量化线后，正式签发 `{}` 动作；投委会多数票为 `{}`，量化线状态为 `{}`。",
        selected_action, committee_result.vote_tally.majority_vote, scorecard.score_status
    )
}

// 2026-04-09 CST: 这里统一提取执行约束，原因是最终正式决议除了动作，还必须带可执行的限制条件，
// 目的：让后续 package / 执行层不需要重新回扫多份对象，直接消费主席线给出的约束摘要。
fn build_execution_constraints(
    committee_result: &SecurityDecisionCommitteeResult,
    scorecard: &SecurityScorecardDocument,
    selected_action: &str,
) -> Vec<String> {
    let mut constraints = Vec::new();
    constraints.push(format!(
        "主席正式动作 `{selected_action}` 需遵守风险否决状态 `{}`。",
        committee_result.risk_veto.status
    ));
    constraints.extend(
        committee_result
            .decision_card
            .required_next_actions
            .iter()
            .take(3)
            .cloned(),
    );
    constraints.extend(scorecard.limitations.iter().take(2).cloned());
    dedupe_strings(&mut constraints);
    constraints
}

fn dedupe_strings(values: &mut Vec<String>) {
    let mut deduped = Vec::new();
    for value in values.drain(..) {
        if !deduped.contains(&value) {
            deduped.push(value);
        }
    }
    *values = deduped;
}

fn normalize_created_at(value: &str) -> String {
    if value.trim().is_empty() {
        Utc::now().to_rfc3339()
    } else {
        value.trim().to_string()
    }
}

fn default_created_at() -> String {
    Utc::now().to_rfc3339()
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

fn default_min_risk_reward_ratio() -> f64 {
    2.0
}
