use thiserror::Error;

use crate::runtime::security_execution_store::SecurityExecutionStore;
use crate::runtime::security_execution_store::SecurityExecutionStoreError;
use crate::tools::contracts::PostTradeReviewDimension;
use crate::tools::contracts::PostTradeReviewOutcome;
use crate::tools::contracts::SecurityPostTradeReviewRequest;
use crate::tools::contracts::SecurityPostTradeReviewResult;
use crate::tools::contracts::SecurityRecordPositionAdjustmentResult;

const POSITION_CONTINUITY_TOLERANCE: f64 = 1e-9;

// 2026-04-08 CST: 这里新增投后复盘正式 Tool，原因是证券主链已经具备计划对象与调仓事件对象，下一步必须能形成正式复盘闭环；
// 目的：让系统基于 position_plan_ref 与 adjustment_event_ref 回读执行事实，产出结构化复盘结论，而不是继续依赖对话层人工总结。
#[derive(Debug, Error)]
pub enum SecurityPostTradeReviewError {
    #[error("security_post_trade_review 缺少 decision_ref")]
    MissingDecisionRef,
    #[error("security_post_trade_review 缺少 approval_ref")]
    MissingApprovalRef,
    #[error("security_post_trade_review 缺少 evidence_version")]
    MissingEvidenceVersion,
    #[error("security_post_trade_review 缺少 position_plan_ref")]
    MissingPositionPlanRef,
    #[error("security_post_trade_review 缺少 symbol")]
    MissingSymbol,
    #[error("security_post_trade_review 缺少 analysis_date")]
    MissingAnalysisDate,
    #[error("security_post_trade_review 至少需要一条 adjustment_event_ref")]
    EmptyAdjustmentEventRefs,
    #[error("未找到仓位计划记录 `{position_plan_ref}`")]
    PositionPlanNotFound { position_plan_ref: String },
    #[error("未找到调仓事件记录 `{adjustment_event_ref}`")]
    AdjustmentEventNotFound { adjustment_event_ref: String },
    #[error("调仓事件 `{adjustment_event_ref}` 不属于同一 position_plan_ref")]
    EventPlanMismatch { adjustment_event_ref: String },
    #[error("调仓事件 `{adjustment_event_ref}` 的 symbol 与复盘请求不一致")]
    EventSymbolMismatch { adjustment_event_ref: String },
    #[error("调仓事件 `{adjustment_event_ref}` 的 decision_ref 与复盘请求不一致")]
    EventDecisionMismatch { adjustment_event_ref: String },
    #[error("调仓事件 `{adjustment_event_ref}` 的 approval_ref 与复盘请求不一致")]
    EventApprovalMismatch { adjustment_event_ref: String },
    #[error("调仓事件 `{adjustment_event_ref}` 的 evidence_version 与复盘请求不一致")]
    EventEvidenceMismatch { adjustment_event_ref: String },
    #[error("调仓事件顺序必须按 event_date 递增")]
    EventDateOutOfOrder,
    #[error("调仓事件链存在仓位衔接断裂")]
    BrokenPositionContinuity,
    #[error("{0}")]
    Store(#[from] SecurityExecutionStoreError),
}

pub fn security_post_trade_review(
    request: &SecurityPostTradeReviewRequest,
) -> Result<SecurityPostTradeReviewResult, SecurityPostTradeReviewError> {
    validate_post_trade_review_request(request)?;

    let store = SecurityExecutionStore::workspace_default()?;
    let position_plan = store
        .load_position_plan(&request.position_plan_ref)?
        .ok_or_else(|| SecurityPostTradeReviewError::PositionPlanNotFound {
            position_plan_ref: request.position_plan_ref.clone(),
        })?;

    // 2026-04-08 CST: 这里先把复盘请求与正式计划锚点做强一致性校验，原因是方案 C 不能容忍 review 对着另一份计划事实做“错配复盘”；
    // 目的：确保后续聚合时 symbol / decision / approval / evidence 都来自同一条正式执行链。
    if position_plan.symbol != request.symbol
        || position_plan.decision_ref != request.decision_ref
        || position_plan.approval_ref != request.approval_ref
        || position_plan.evidence_version != request.evidence_version
    {
        return Err(SecurityPostTradeReviewError::EventPlanMismatch {
            adjustment_event_ref: request.position_plan_ref.clone(),
        });
    }

    let mut adjustment_events = Vec::with_capacity(request.adjustment_event_refs.len());
    for adjustment_event_ref in &request.adjustment_event_refs {
        let event = store
            .load_adjustment_event(adjustment_event_ref)?
            .ok_or_else(|| SecurityPostTradeReviewError::AdjustmentEventNotFound {
                adjustment_event_ref: adjustment_event_ref.clone(),
            })?;
        validate_adjustment_event(request, &event)?;
        adjustment_events.push(event);
    }

    validate_event_sequence(&adjustment_events)?;

    let review_outcome = classify_review_outcome(&adjustment_events);
    let decision_accuracy = classify_decision_accuracy(&adjustment_events);
    let execution_quality = classify_execution_quality(&adjustment_events);
    let risk_control_quality = classify_risk_control_quality(&adjustment_events);
    let correction_actions =
        build_correction_actions(&adjustment_events, &review_outcome, &execution_quality);
    let next_cycle_guidance = build_next_cycle_guidance(
        &adjustment_events,
        &review_outcome,
        &decision_accuracy,
        &risk_control_quality,
    );
    // 2026-04-08 CST: 这里统一用复盘日期生成 post_trade_review_ref，原因是同一标的可能围绕同一计划在不同阶段重复复盘；
    // 目的：让复盘对象的引用既稳定可读，又能区分不同复盘时点。
    let post_trade_review_ref = format!(
        "post-trade-review:{}:{}:v1",
        request.symbol.trim(),
        request.analysis_date.trim()
    );

    Ok(SecurityPostTradeReviewResult::assemble(
        post_trade_review_ref,
        request,
        review_outcome,
        decision_accuracy,
        execution_quality,
        risk_control_quality,
        correction_actions,
        next_cycle_guidance,
    ))
}

fn validate_post_trade_review_request(
    request: &SecurityPostTradeReviewRequest,
) -> Result<(), SecurityPostTradeReviewError> {
    if request.decision_ref.trim().is_empty() {
        return Err(SecurityPostTradeReviewError::MissingDecisionRef);
    }
    if request.approval_ref.trim().is_empty() {
        return Err(SecurityPostTradeReviewError::MissingApprovalRef);
    }
    if request.evidence_version.trim().is_empty() {
        return Err(SecurityPostTradeReviewError::MissingEvidenceVersion);
    }
    if request.position_plan_ref.trim().is_empty() {
        return Err(SecurityPostTradeReviewError::MissingPositionPlanRef);
    }
    if request.symbol.trim().is_empty() {
        return Err(SecurityPostTradeReviewError::MissingSymbol);
    }
    if request.analysis_date.trim().is_empty() {
        return Err(SecurityPostTradeReviewError::MissingAnalysisDate);
    }
    if request.adjustment_event_refs.is_empty() {
        return Err(SecurityPostTradeReviewError::EmptyAdjustmentEventRefs);
    }

    Ok(())
}

fn validate_adjustment_event(
    request: &SecurityPostTradeReviewRequest,
    event: &SecurityRecordPositionAdjustmentResult,
) -> Result<(), SecurityPostTradeReviewError> {
    if event.position_plan_ref != request.position_plan_ref {
        return Err(SecurityPostTradeReviewError::EventPlanMismatch {
            adjustment_event_ref: event.adjustment_event_ref.clone(),
        });
    }
    if event.symbol != request.symbol {
        return Err(SecurityPostTradeReviewError::EventSymbolMismatch {
            adjustment_event_ref: event.adjustment_event_ref.clone(),
        });
    }
    if event.decision_ref != request.decision_ref {
        return Err(SecurityPostTradeReviewError::EventDecisionMismatch {
            adjustment_event_ref: event.adjustment_event_ref.clone(),
        });
    }
    if event.approval_ref != request.approval_ref {
        return Err(SecurityPostTradeReviewError::EventApprovalMismatch {
            adjustment_event_ref: event.adjustment_event_ref.clone(),
        });
    }
    if event.evidence_version != request.evidence_version {
        return Err(SecurityPostTradeReviewError::EventEvidenceMismatch {
            adjustment_event_ref: event.adjustment_event_ref.clone(),
        });
    }

    Ok(())
}

fn validate_event_sequence(
    events: &[SecurityRecordPositionAdjustmentResult],
) -> Result<(), SecurityPostTradeReviewError> {
    for pair in events.windows(2) {
        let previous = &pair[0];
        let next = &pair[1];
        if previous.event_date > next.event_date {
            return Err(SecurityPostTradeReviewError::EventDateOutOfOrder);
        }
        if (previous.after_position_pct - next.before_position_pct).abs()
            > POSITION_CONTINUITY_TOLERANCE
        {
            return Err(SecurityPostTradeReviewError::BrokenPositionContinuity);
        }
    }

    Ok(())
}

fn classify_review_outcome(
    events: &[SecurityRecordPositionAdjustmentResult],
) -> PostTradeReviewOutcome {
    let has_off_plan = events
        .iter()
        .any(|event| matches!(event.plan_alignment, crate::tools::contracts::PositionPlanAlignment::OffPlan));
    let has_justified = events.iter().any(|event| {
        matches!(
            event.plan_alignment,
            crate::tools::contracts::PositionPlanAlignment::JustifiedDeviation
        )
    });

    if has_off_plan {
        PostTradeReviewOutcome::Invalidated
    } else if has_justified {
        PostTradeReviewOutcome::Mixed
    } else {
        PostTradeReviewOutcome::Validated
    }
}

fn classify_decision_accuracy(
    events: &[SecurityRecordPositionAdjustmentResult],
) -> PostTradeReviewDimension {
    let has_off_plan = events
        .iter()
        .any(|event| matches!(event.plan_alignment, crate::tools::contracts::PositionPlanAlignment::OffPlan));
    let has_justified = events.iter().any(|event| {
        matches!(
            event.plan_alignment,
            crate::tools::contracts::PositionPlanAlignment::JustifiedDeviation
        )
    });

    if has_off_plan {
        PostTradeReviewDimension::Weak
    } else if has_justified {
        PostTradeReviewDimension::Acceptable
    } else {
        PostTradeReviewDimension::Strong
    }
}

fn classify_execution_quality(
    events: &[SecurityRecordPositionAdjustmentResult],
) -> PostTradeReviewDimension {
    classify_decision_accuracy(events)
}

fn classify_risk_control_quality(
    events: &[SecurityRecordPositionAdjustmentResult],
) -> PostTradeReviewDimension {
    let has_risk_response = events.iter().any(|event| {
        matches!(
            event.event_type,
            crate::tools::contracts::PositionAdjustmentEventType::Reduce
                | crate::tools::contracts::PositionAdjustmentEventType::Exit
                | crate::tools::contracts::PositionAdjustmentEventType::RiskUpdate
        )
    });
    let has_off_plan = events
        .iter()
        .any(|event| matches!(event.plan_alignment, crate::tools::contracts::PositionPlanAlignment::OffPlan));

    if has_risk_response {
        PostTradeReviewDimension::Strong
    } else if has_off_plan {
        PostTradeReviewDimension::Weak
    } else {
        PostTradeReviewDimension::Acceptable
    }
}

fn build_correction_actions(
    events: &[SecurityRecordPositionAdjustmentResult],
    review_outcome: &PostTradeReviewOutcome,
    execution_quality: &PostTradeReviewDimension,
) -> Vec<String> {
    let mut actions = Vec::new();

    if events.iter().any(|event| {
        matches!(
            event.plan_alignment,
            crate::tools::contracts::PositionPlanAlignment::JustifiedDeviation
        )
    }) {
        actions.push("把 justified_deviation 的触发条件前移到下一版仓位计划，减少临盘被动偏离。".to_string());
    }
    if matches!(review_outcome, PostTradeReviewOutcome::Invalidated) {
        actions.push("重新检查原始决策假设，必要时重做投研会和仓位上限设定。".to_string());
    }
    if matches!(execution_quality, PostTradeReviewDimension::Acceptable | PostTradeReviewDimension::Weak) {
        actions.push("继续按统一 decision_ref / approval_ref / position_plan_ref 记录每次调仓，压缩执行漂移。".to_string());
    }
    if actions.is_empty() {
        actions.push("当前执行与计划基本一致，保留现有调仓纪律并持续复核。".to_string());
    }

    actions
}

fn build_next_cycle_guidance(
    events: &[SecurityRecordPositionAdjustmentResult],
    review_outcome: &PostTradeReviewOutcome,
    decision_accuracy: &PostTradeReviewDimension,
    risk_control_quality: &PostTradeReviewDimension,
) -> Vec<String> {
    let mut guidance = Vec::new();

    if events.iter().any(|event| {
        matches!(
            event.event_type,
            crate::tools::contracts::PositionAdjustmentEventType::Reduce
        )
    }) {
        guidance.push("下一轮只在量价重新确认后恢复加仓，避免仓位过早回补。".to_string());
    } else {
        guidance.push("下一轮继续按计划节奏分批执行，不要在未确认前一次性拉满仓位。".to_string());
    }
    if matches!(review_outcome, PostTradeReviewOutcome::Mixed | PostTradeReviewOutcome::Invalidated)
    {
        guidance.push("把本轮偏离原因写回下一版仓位计划，确保下次复盘能直接对照纠偏。".to_string());
    }
    if matches!(decision_accuracy, PostTradeReviewDimension::Weak)
        || matches!(risk_control_quality, PostTradeReviewDimension::Weak)
    {
        guidance.push("优先收紧仓位与止损纪律，再考虑恢复进攻性配置。".to_string());
    }

    guidance
}
