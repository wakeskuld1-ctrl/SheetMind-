use serde_json::{Value, json};

use crate::ops::stock::import_stock_price_history::{
    ImportStockPriceHistoryRequest, import_stock_price_history,
};
use crate::ops::stock::security_analysis_contextual::{
    SecurityAnalysisContextualRequest, security_analysis_contextual,
};
use crate::ops::stock::security_analysis_fullstack::{
    SecurityAnalysisFullstackRequest, security_analysis_fullstack,
};
use crate::ops::stock::security_analysis_resonance::{
    AppendResonanceEventTagsRequest, AppendResonanceFactorSeriesRequest,
    BootstrapResonanceTemplateFactorsRequest, EvaluateSecurityResonanceRequest,
    RegisterResonanceFactorRequest, SecurityAnalysisResonanceRequest, append_resonance_event_tags,
    append_resonance_factor_series, bootstrap_resonance_template_factors,
    evaluate_security_resonance, register_resonance_factor, security_analysis_resonance,
};
use crate::ops::stock::security_chair_resolution::{
    SecurityChairResolutionRequest, security_chair_resolution,
};
use crate::ops::stock::security_condition_review::{
    SecurityConditionReviewRequest, security_condition_review,
};
use crate::ops::stock::security_committee_vote::{
    SecurityCommitteeMemberAgentRequest as SecurityCommitteeVoteMemberAgentRequest,
    SecurityCommitteeVoteRequest, security_committee_member_agent as security_committee_vote_member_agent,
    security_committee_vote,
};
use crate::ops::stock::security_decision_briefing::{
    SecurityDecisionBriefingRequest, security_decision_briefing,
};
use crate::ops::stock::security_decision_committee::{
    SecurityCommitteeMemberAgentRequest as SecurityDecisionCommitteeMemberAgentRequest,
    SecurityDecisionCommitteeRequest, security_committee_member_agent, security_decision_committee,
};
use crate::ops::stock::security_decision_evidence_bundle::{
    SecurityDecisionEvidenceBundleRequest, security_decision_evidence_bundle,
};
use crate::ops::stock::security_decision_package_revision::{
    SecurityDecisionPackageRevisionRequest, security_decision_package_revision,
};
use crate::ops::stock::security_decision_submit_approval::{
    SecurityDecisionSubmitApprovalRequest, security_decision_submit_approval,
};
use crate::ops::stock::security_decision_verify_package::{
    SecurityDecisionVerifyPackageRequest, security_decision_verify_package,
};
use crate::ops::stock::security_execution_record::{
    SecurityExecutionRecordRequest, security_execution_record,
};
use crate::ops::stock::security_feature_snapshot::{
    SecurityFeatureSnapshotRequest, security_feature_snapshot,
};
use crate::ops::stock::security_forward_outcome::{
    SecurityForwardOutcomeRequest, security_forward_outcome,
};
use crate::ops::stock::security_post_trade_review::{
    SecurityPostTradeReviewRequest, security_post_trade_review,
};
use crate::ops::stock::security_position_plan_record::security_position_plan_record;
use crate::ops::stock::security_record_position_adjustment::security_record_position_adjustment;
use crate::ops::stock::security_record_post_meeting_conclusion::{
    SecurityRecordPostMeetingConclusionRequest, security_record_post_meeting_conclusion,
};
use crate::ops::stock::security_scorecard_refit_run::{
    SecurityScorecardRefitRequest, security_scorecard_refit,
};
use crate::ops::stock::security_scorecard_training::{
    SecurityScorecardTrainingRequest, security_scorecard_training,
};
use crate::ops::stock::signal_outcome_research::{
    BackfillSecuritySignalOutcomesRequest, RecordSecuritySignalSnapshotRequest,
    SignalOutcomeResearchSummaryRequest, StudySecuritySignalAnalogsRequest,
    backfill_security_signal_outcomes, record_security_signal_snapshot,
    signal_outcome_research_summary, study_security_signal_analogs,
};
use crate::ops::stock::sync_stock_price_history::{
    SyncStockPriceHistoryRequest, sync_stock_price_history,
};
use crate::ops::stock::sync_template_resonance_factors::{
    SyncTemplateResonanceFactorsRequest, sync_template_resonance_factors,
};
use crate::ops::stock::technical_consultation_basic::{
    TechnicalConsultationBasicRequest, technical_consultation_basic,
};
use crate::tools::contracts::{
    SecurityPositionPlanRecordRequest, SecurityRecordPositionAdjustmentRequest, ToolResponse,
};

pub(super) fn dispatch_import_stock_price_history(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<ImportStockPriceHistoryRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match import_stock_price_history(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_sync_stock_price_history(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<SyncStockPriceHistoryRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match sync_stock_price_history(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_sync_template_resonance_factors(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<SyncTemplateResonanceFactorsRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match sync_template_resonance_factors(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_technical_consultation_basic(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<TechnicalConsultationBasicRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match technical_consultation_basic(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_analysis_contextual(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<SecurityAnalysisContextualRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_analysis_contextual(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_analysis_fullstack(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<SecurityAnalysisFullstackRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_analysis_fullstack(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_decision_briefing(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<SecurityDecisionBriefingRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_decision_briefing(&request) {
        Ok(result) => ToolResponse::ok_serialized(&result),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_position_plan_record(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<SecurityPositionPlanRecordRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_position_plan_record(&request) {
        Ok(result) => ToolResponse::ok_serialized(&result),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_record_position_adjustment(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<SecurityRecordPositionAdjustmentRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_record_position_adjustment(&request) {
        Ok(result) => ToolResponse::ok_serialized(&result),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_committee_vote(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<SecurityCommitteeVoteRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_committee_vote(&request) {
        Ok(result) => ToolResponse::ok_serialized(&result),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_condition_review(args: Value) -> ToolResponse {
    // 2026-04-10 CST: 这里新增条件复核 dispatcher 入口，原因是 Task 3 需要把最小复核合同从内部函数升级为正式 Tool；
    // 目的：把 JSON 请求解析、规则执行与结构化响应收口在 stock dispatcher，保持证券主链接线方式一致。
    let request = match serde_json::from_value::<SecurityConditionReviewRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_condition_review(&request) {
        Ok(result) => ToolResponse::ok_serialized(&result),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_execution_record(args: Value) -> ToolResponse {
    // 2026-04-10 CST: 这里新增 execution_record dispatcher 入口，原因是 Task 5 要把条件复核挂接继续沉淀到执行层正式 Tool；
    // 目的：统一承接 CLI JSON -> request 解析 -> execution_record 结构化结果，避免上层继续依赖隐式内部调用。
    let request = match serde_json::from_value::<SecurityExecutionRecordRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_execution_record(&request) {
        Ok(result) => ToolResponse::ok_serialized(&result),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_post_trade_review(args: Value) -> ToolResponse {
    // 2026-04-10 CST: 这里新增 post_trade_review dispatcher 入口，原因是 Task 5 要把条件复核解释继续挂进投后复盘正式 Tool；
    // 目的：让 review 层沿现有 stock dispatcher 主链被发现、调用和审计，而不是只停留在内部函数。
    let request = match serde_json::from_value::<SecurityPostTradeReviewRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_post_trade_review(&request) {
        Ok(result) => ToolResponse::ok_serialized(&result),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_decision_evidence_bundle(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<SecurityDecisionEvidenceBundleRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_decision_evidence_bundle(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_decision_committee(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<SecurityDecisionCommitteeRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_decision_committee(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_committee_member_agent(args: Value) -> ToolResponse {
    if let Ok(request) =
        serde_json::from_value::<SecurityDecisionCommitteeMemberAgentRequest>(args.clone())
    {
        return match security_committee_member_agent(&request) {
            Ok(result) => ToolResponse::ok(json!(result)),
            Err(error) => ToolResponse::error(error.to_string()),
        };
    }

    let request = match serde_json::from_value::<SecurityCommitteeVoteMemberAgentRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_committee_vote_member_agent(&request) {
        Ok(result) => ToolResponse::ok_serialized(&result),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_register_resonance_factor(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<RegisterResonanceFactorRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match register_resonance_factor(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_append_resonance_factor_series(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<AppendResonanceFactorSeriesRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match append_resonance_factor_series(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_append_resonance_event_tags(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<AppendResonanceEventTagsRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match append_resonance_event_tags(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_bootstrap_resonance_template_factors(args: Value) -> ToolResponse {
    let request =
        match serde_json::from_value::<BootstrapResonanceTemplateFactorsRequest>(args) {
            Ok(request) => request,
            Err(error) => {
                return ToolResponse::error(format!("request parsing failed: {error}"));
            }
        };

    match bootstrap_resonance_template_factors(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_evaluate_security_resonance(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<EvaluateSecurityResonanceRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match evaluate_security_resonance(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_analysis_resonance(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<SecurityAnalysisResonanceRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_analysis_resonance(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_record_security_signal_snapshot(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<RecordSecuritySignalSnapshotRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match record_security_signal_snapshot(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_backfill_security_signal_outcomes(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<BackfillSecuritySignalOutcomesRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match backfill_security_signal_outcomes(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_study_security_signal_analogs(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<StudySecuritySignalAnalogsRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match study_security_signal_analogs(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_signal_outcome_research_summary(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<SignalOutcomeResearchSummaryRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match signal_outcome_research_summary(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_chair_resolution(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<SecurityChairResolutionRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_chair_resolution(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_feature_snapshot(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<SecurityFeatureSnapshotRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_feature_snapshot(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_forward_outcome(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<SecurityForwardOutcomeRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_forward_outcome(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_scorecard_refit(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<SecurityScorecardRefitRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_scorecard_refit(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_scorecard_training(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<SecurityScorecardTrainingRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_scorecard_training(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_decision_submit_approval(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<SecurityDecisionSubmitApprovalRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_decision_submit_approval(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_decision_verify_package(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<SecurityDecisionVerifyPackageRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_decision_verify_package(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_decision_package_revision(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<SecurityDecisionPackageRevisionRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_decision_package_revision(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_record_post_meeting_conclusion(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<SecurityRecordPostMeetingConclusionRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_record_post_meeting_conclusion(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}
