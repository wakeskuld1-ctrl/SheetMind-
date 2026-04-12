use serde::{Deserialize, Serialize};
use thiserror::Error;

// 2026-04-12 CST: Add the formal post-trade review request contract, because P8
// needs review conclusions to become replayable lifecycle artifacts.
// Purpose: keep review status, summary, attribution, and governance follow-up stable for later replay.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityPostTradeReviewRequest {
    pub symbol: String,
    pub analysis_date: String,
    pub decision_ref: String,
    pub approval_ref: String,
    pub position_plan_ref: String,
    pub execution_record_ref: String,
    pub review_status: String,
    pub review_summary: String,
    pub attribution: SecurityPostTradeReviewAttribution,
    pub recommended_governance_action: String,
    pub created_at: String,
}

// 2026-04-12 CST: Add the formal layered attribution block, because P8 needs
// review outputs to classify issues by data/model/governance/execution instead of prose only.
// Purpose: make post-trade review suitable for future governance feedback loops.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityPostTradeReviewAttribution {
    pub data_issue: bool,
    pub model_issue: bool,
    pub governance_issue: bool,
    pub execution_issue: bool,
}

// 2026-04-12 CST: Add the formal post-trade binding block, because review
// documents must inherit the same upstream refs as the rest of the lifecycle.
// Purpose: keep execution-to-review linkage explicit and machine-readable.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityPostTradeReviewBinding {
    pub decision_ref: String,
    pub approval_ref: String,
    pub position_plan_ref: String,
    pub execution_record_ref: String,
}

// 2026-04-12 CST: Add the formal post-trade review document, because P8 needs
// a stable close-the-loop artifact after execution events.
// Purpose: preserve review state, layered attribution, and governance follow-up in one contract.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityPostTradeReviewDocument {
    pub contract_version: String,
    pub document_type: String,
    pub post_trade_review_id: String,
    pub symbol: String,
    pub analysis_date: String,
    pub review_status: String,
    pub review_summary: String,
    pub attribution: SecurityPostTradeReviewAttribution,
    pub recommended_governance_action: String,
    pub review_notes: Vec<String>,
    pub binding: SecurityPostTradeReviewBinding,
    pub created_at: String,
}

// 2026-04-12 CST: Wrap the post-trade review document in a named result envelope,
// because the stock CLI keeps exposing lifecycle objects as named payloads.
// Purpose: leave room for later expansion without changing the outer tool response.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityPostTradeReviewResult {
    pub post_trade_review: SecurityPostTradeReviewDocument,
}

#[derive(Debug, Error)]
pub enum SecurityPostTradeReviewError {
    #[error("post trade review analysis_date cannot be empty")]
    EmptyAnalysisDate,
    #[error("post trade review execution_record_ref cannot be empty")]
    EmptyExecutionRecordRef,
    #[error("post trade review status cannot be empty")]
    EmptyReviewStatus,
    #[error("post trade review summary cannot be empty")]
    EmptyReviewSummary,
    #[error("post trade review governance action cannot be empty")]
    EmptyGovernanceAction,
}

// 2026-04-12 CST: Add the first minimal post-trade review builder, because P8
// needs a formal replayable review object before governance feedback is wired deeper.
// Purpose: produce deterministic layered review output that can later feed promotion and retraining logic.
pub fn security_post_trade_review(
    request: &SecurityPostTradeReviewRequest,
) -> Result<SecurityPostTradeReviewResult, SecurityPostTradeReviewError> {
    if request.analysis_date.trim().is_empty() {
        return Err(SecurityPostTradeReviewError::EmptyAnalysisDate);
    }
    if request.execution_record_ref.trim().is_empty() {
        return Err(SecurityPostTradeReviewError::EmptyExecutionRecordRef);
    }
    if request.review_status.trim().is_empty() {
        return Err(SecurityPostTradeReviewError::EmptyReviewStatus);
    }
    if request.review_summary.trim().is_empty() {
        return Err(SecurityPostTradeReviewError::EmptyReviewSummary);
    }
    if request.recommended_governance_action.trim().is_empty() {
        return Err(SecurityPostTradeReviewError::EmptyGovernanceAction);
    }

    let normalized_status = request.review_status.trim().to_ascii_lowercase();
    let normalized_action = request
        .recommended_governance_action
        .trim()
        .to_ascii_lowercase();

    Ok(SecurityPostTradeReviewResult {
        post_trade_review: SecurityPostTradeReviewDocument {
            contract_version: "security_post_trade_review.v1".to_string(),
            document_type: "security_post_trade_review".to_string(),
            post_trade_review_id: format!(
                "post-trade-review:{}:{}:{}:v1",
                request.symbol, request.analysis_date, normalized_status
            ),
            symbol: request.symbol.clone(),
            analysis_date: request.analysis_date.clone(),
            review_status: normalized_status,
            review_summary: request.review_summary.clone(),
            attribution: request.attribution.clone(),
            recommended_governance_action: normalized_action.clone(),
            review_notes: build_review_notes(&request.attribution, &normalized_action),
            binding: SecurityPostTradeReviewBinding {
                decision_ref: request.decision_ref.clone(),
                approval_ref: request.approval_ref.clone(),
                position_plan_ref: request.position_plan_ref.clone(),
                execution_record_ref: request.execution_record_ref.clone(),
            },
            created_at: request.created_at.clone(),
        },
    })
}

// 2026-04-12 CST: Keep review notes derived from structured attribution, because
// P8 wants post-trade review semantics to remain machine-readable and auditable.
// Purpose: provide a stable minimal reasoning trail without regressing to free-form summaries.
fn build_review_notes(
    attribution: &SecurityPostTradeReviewAttribution,
    governance_action: &str,
) -> Vec<String> {
    let mut notes = Vec::new();

    if attribution.data_issue {
        notes.push("review identified a data-quality issue".to_string());
    }
    if attribution.model_issue {
        notes.push("review identified a model-quality issue".to_string());
    }
    if attribution.governance_issue {
        notes.push("review identified a governance-process issue".to_string());
    }
    if attribution.execution_issue {
        notes.push("review identified an execution-process issue".to_string());
    }
    if notes.is_empty() {
        notes.push("review did not mark a specific failure layer".to_string());
    }
    notes.push(format!(
        "governance follow-up should proceed as `{governance_action}`"
    ));

    notes
}
