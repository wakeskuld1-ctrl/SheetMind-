use thiserror::Error;

use crate::runtime::security_execution_store::SecurityExecutionStore;
use crate::runtime::security_execution_store::SecurityExecutionStoreError;
use crate::tools::contracts::{
    SecurityPositionPlanRecordRequest, SecurityPositionPlanRecordResult,
};

#[derive(Debug, Error)]
pub enum SecurityPositionPlanRecordError {
    #[error("security_position_plan_record 缺少 decision_ref")]
    MissingDecisionRef,
    #[error("security_position_plan_record 缺少 approval_ref")]
    MissingApprovalRef,
    #[error("security_position_plan_record 缺少 evidence_version")]
    MissingEvidenceVersion,
    #[error("security_position_plan_record 缺少 symbol")]
    MissingSymbol,
    #[error("security_position_plan_record 缺少 analysis_date")]
    MissingAnalysisDate,
    #[error("{0}")]
    Store(#[from] SecurityExecutionStoreError),
}

// 2026-04-08 CST: 这里新增仓位计划记录正式 Tool，原因是证券主链后续的调仓事件和投后复盘都需要一个稳定的计划锚点；
// 目的：先把 briefing 顶层 `position_plan` 升级为可引用对象，并沿正式 dispatcher/catalog 主链对外暴露，而不是继续只停留在报告 JSON 子层。
pub fn security_position_plan_record(
    request: &SecurityPositionPlanRecordRequest,
) -> Result<SecurityPositionPlanRecordResult, SecurityPositionPlanRecordError> {
    validate_position_plan_record_request(request)?;

    // 2026-04-08 CST: 这里先用确定性 ref 生成规则收口 position_plan_ref，原因是 Task 2 目标是先接通正式 Tool 主链，
    // 目的：让后续调仓事件能有稳定引用目标，同时不在本轮提前引入新的持久化复杂度。
    let position_plan_ref = format!(
        "position-plan:{}:{}:v1",
        request.symbol.trim(),
        request.analysis_date.trim()
    );

    let result =
        SecurityPositionPlanRecordResult::from_position_plan(position_plan_ref, request.clone());
    // 2026-04-08 CST: 这里把正式计划对象落到执行层 runtime，原因是后续调仓记录与投后复盘都要沿同一条 ref 链回读；
    // 目的：避免 position_plan_record 只返回一个引用却没有实际事实源，导致 post_trade_review 无法真正聚合。
    let store = SecurityExecutionStore::workspace_default()?;
    store.upsert_position_plan(&result)?;
    Ok(result)
}

// 2026-04-08 CST: 这里集中校验仓位计划记录请求边界，原因是后续事件记录和复盘都会依赖这些引用字段；
// 目的：先把最基本的 decision/approval/evidence/symbol/date 五个锚点收口到单点验证，避免 dispatcher 和调用方各自脑补。
fn validate_position_plan_record_request(
    request: &SecurityPositionPlanRecordRequest,
) -> Result<(), SecurityPositionPlanRecordError> {
    if request.decision_ref.trim().is_empty() {
        return Err(SecurityPositionPlanRecordError::MissingDecisionRef);
    }
    if request.approval_ref.trim().is_empty() {
        return Err(SecurityPositionPlanRecordError::MissingApprovalRef);
    }
    if request.evidence_version.trim().is_empty() {
        return Err(SecurityPositionPlanRecordError::MissingEvidenceVersion);
    }
    if request.symbol.trim().is_empty() {
        return Err(SecurityPositionPlanRecordError::MissingSymbol);
    }
    if request.analysis_date.trim().is_empty() {
        return Err(SecurityPositionPlanRecordError::MissingAnalysisDate);
    }

    Ok(())
}
