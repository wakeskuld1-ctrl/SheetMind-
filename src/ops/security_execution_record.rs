use serde::{Deserialize, Serialize};
use thiserror::Error;

// 2026-04-12 CST: Add the formal execution-record request contract, because P8
// needs execution lifecycle events to become replayable stock artifacts.
// Purpose: preserve action, status, sizing, and lifecycle refs in one stable input shape.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityExecutionRecordRequest {
    pub symbol: String,
    pub analysis_date: String,
    pub decision_ref: String,
    pub approval_ref: String,
    pub position_plan_ref: String,
    #[serde(default)]
    pub condition_review_ref: Option<String>,
    pub execution_action: String,
    pub execution_status: String,
    pub executed_gross_pct: f64,
    pub execution_summary: String,
    pub created_at: String,
}

// 2026-04-12 CST: Add the formal execution binding block, because later
// post-trade reviews must be able to inherit the same upstream refs directly.
// Purpose: keep execution linkage machine-readable instead of prose-driven.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityExecutionRecordBinding {
    pub decision_ref: String,
    pub approval_ref: String,
    pub position_plan_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition_review_ref: Option<String>,
}

// 2026-04-12 CST: Add the formal execution record document, because P8 needs
// a stable object for build/add/reduce/exit/freeze/unfreeze style lifecycle events.
// Purpose: preserve execution facts in a reusable contract for later package and review wiring.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityExecutionRecordDocument {
    pub contract_version: String,
    pub document_type: String,
    pub execution_record_id: String,
    pub symbol: String,
    pub analysis_date: String,
    pub execution_action: String,
    pub execution_status: String,
    pub executed_gross_pct: f64,
    pub execution_summary: String,
    pub binding: SecurityExecutionRecordBinding,
    pub created_at: String,
}

// 2026-04-12 CST: Add a stable result envelope, because the stock CLI keeps
// returning named payload objects for formal domain artifacts.
// Purpose: leave room for later path expansion without breaking outer response shape.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityExecutionRecordResult {
    pub execution_record: SecurityExecutionRecordDocument,
}

#[derive(Debug, Error)]
pub enum SecurityExecutionRecordError {
    #[error("execution record analysis_date cannot be empty")]
    EmptyAnalysisDate,
    #[error("execution record action cannot be empty")]
    EmptyExecutionAction,
    #[error("execution record status cannot be empty")]
    EmptyExecutionStatus,
    #[error("execution record summary cannot be empty")]
    EmptyExecutionSummary,
    #[error("execution record gross pct cannot be negative")]
    NegativeExecutedGrossPct,
}

// 2026-04-12 CST: Add the first minimal execution-record builder, because P8
// needs a formal lifecycle event object before post-trade review can attach to it.
// Purpose: produce deterministic execution facts with stable IDs and bindings.
pub fn security_execution_record(
    request: &SecurityExecutionRecordRequest,
) -> Result<SecurityExecutionRecordResult, SecurityExecutionRecordError> {
    if request.analysis_date.trim().is_empty() {
        return Err(SecurityExecutionRecordError::EmptyAnalysisDate);
    }
    if request.execution_action.trim().is_empty() {
        return Err(SecurityExecutionRecordError::EmptyExecutionAction);
    }
    if request.execution_status.trim().is_empty() {
        return Err(SecurityExecutionRecordError::EmptyExecutionStatus);
    }
    if request.execution_summary.trim().is_empty() {
        return Err(SecurityExecutionRecordError::EmptyExecutionSummary);
    }
    if request.executed_gross_pct < 0.0 {
        return Err(SecurityExecutionRecordError::NegativeExecutedGrossPct);
    }

    let normalized_action = request.execution_action.trim().to_ascii_lowercase();
    let normalized_status = request.execution_status.trim().to_ascii_lowercase();

    Ok(SecurityExecutionRecordResult {
        execution_record: SecurityExecutionRecordDocument {
            contract_version: "security_execution_record.v1".to_string(),
            document_type: "security_execution_record".to_string(),
            execution_record_id: format!(
                "execution-record:{}:{}:{}:v1",
                request.symbol, request.analysis_date, normalized_action
            ),
            symbol: request.symbol.clone(),
            analysis_date: request.analysis_date.clone(),
            execution_action: normalized_action,
            execution_status: normalized_status,
            executed_gross_pct: request.executed_gross_pct,
            execution_summary: request.execution_summary.clone(),
            binding: SecurityExecutionRecordBinding {
                decision_ref: request.decision_ref.clone(),
                approval_ref: request.approval_ref.clone(),
                position_plan_ref: request.position_plan_ref.clone(),
                condition_review_ref: request.condition_review_ref.clone(),
            },
            created_at: request.created_at.clone(),
        },
    })
}
