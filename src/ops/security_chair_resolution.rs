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

// 2026-04-09 CST: 这里新增主席裁决请求合同，原因是 Task 1 要把“最终正式动作”从投委会线中拆出来，
// 目的：让主席线拥有独立 Tool 入口，后续 package / verify / audit 都可以围绕这条线接入。
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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SecurityChairResolutionResult {
    pub committee_result: SecurityDecisionCommitteeResult,
    pub scorecard: SecurityScorecardDocument,
    pub chair_resolution: SecurityChairResolutionDocument,
}

#[derive(Debug, Error)]
pub enum SecurityChairResolutionError {
    #[error("security chair resolution committee preparation failed: {0}")]
    Committee(#[from] SecurityDecisionCommitteeError),
    #[error("security chair resolution scorecard preparation failed: {0}")]
    Scorecard(#[from] SecurityScorecardError),
}

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

fn build_committee_reason(committee_result: &SecurityDecisionCommitteeResult) -> String {
    format!(
        "主席参考投委会多数票 `{}`（{} 票），并读取风控席状态 `{}`。",
        committee_result.vote_tally.majority_vote,
        committee_result.vote_tally.majority_count,
        committee_result.risk_veto.status
    )
}

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
