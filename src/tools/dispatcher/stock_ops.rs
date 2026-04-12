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
use crate::ops::stock::security_chair_resolution::{
    SecurityChairResolutionRequest, security_chair_resolution,
};
use crate::ops::stock::security_condition_review::{
    SecurityConditionReviewRequest, security_condition_review,
};
use crate::ops::stock::security_decision_committee::{
    SecurityCommitteeMemberAgentRequest, SecurityDecisionCommitteeRequest,
    security_committee_member_agent, security_decision_committee,
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
use crate::ops::stock::security_disclosure_history_backfill::{
    SecurityDisclosureHistoryBackfillRequest, security_disclosure_history_backfill,
};
use crate::ops::stock::security_disclosure_history_live_backfill::{
    SecurityDisclosureHistoryLiveBackfillRequest, security_disclosure_history_live_backfill,
};
use crate::ops::stock::security_execution_record::{
    SecurityExecutionRecordRequest, security_execution_record,
};
use crate::ops::stock::security_external_proxy_backfill::{
    SecurityExternalProxyBackfillRequest, security_external_proxy_backfill,
};
use crate::ops::stock::security_external_proxy_history_import::{
    SecurityExternalProxyHistoryImportRequest, security_external_proxy_history_import,
};
use crate::ops::stock::security_feature_snapshot::{
    SecurityFeatureSnapshotRequest, security_feature_snapshot,
};
use crate::ops::stock::security_forward_outcome::{
    SecurityForwardOutcomeRequest, security_forward_outcome,
};
use crate::ops::stock::security_fundamental_history_backfill::{
    SecurityFundamentalHistoryBackfillRequest, security_fundamental_history_backfill,
};
use crate::ops::stock::security_fundamental_history_live_backfill::{
    SecurityFundamentalHistoryLiveBackfillRequest, security_fundamental_history_live_backfill,
};
use crate::ops::stock::security_history_expansion::{
    SecurityHistoryExpansionRequest, security_history_expansion,
};
use crate::ops::stock::security_master_scorecard::{
    SecurityMasterScorecardRequest, security_master_scorecard,
};
use crate::ops::stock::security_model_promotion::{
    SecurityModelPromotionRequest, security_model_promotion,
};
use crate::ops::stock::security_post_trade_review::{
    SecurityPostTradeReviewRequest, security_post_trade_review,
};
use crate::ops::stock::security_real_data_validation_backfill::{
    SecurityRealDataValidationBackfillRequest, security_real_data_validation_backfill,
};
use crate::ops::stock::security_record_post_meeting_conclusion::{
    SecurityRecordPostMeetingConclusionRequest, security_record_post_meeting_conclusion,
};
use crate::ops::stock::security_scorecard_refit_run::{
    SecurityScorecardRefitRequest, security_scorecard_refit,
};
use crate::ops::stock::security_scorecard_training::{
    SecurityScorecardTrainingRequest, security_scorecard_training,
};
use crate::ops::stock::security_shadow_evaluation::{
    SecurityShadowEvaluationRequest, security_shadow_evaluation,
};
use crate::ops::stock::sync_stock_price_history::{
    SyncStockPriceHistoryRequest, sync_stock_price_history,
};
use crate::ops::stock::technical_consultation_basic::{
    TechnicalConsultationBasicRequest, technical_consultation_basic,
};
use crate::tools::contracts::ToolResponse;

pub(super) fn dispatch_import_stock_price_history(args: Value) -> ToolResponse {
    // 2026-03-31 CST: 这里把股票历史导入请求收口到 stock dispatcher，原因是股票导入不再属于 foundation 分析域。
    // 目的：让 “CSV -> SQLite” 的股票入口单独沿 stock 模块扩展，而不再继续挂在 analysis dispatcher 里。
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
    // 2026-03-31 CST: 这里把股票历史同步请求收口到 stock dispatcher，原因是 provider 顺序和行情补数属于股票域内部细节。
    // 目的：避免后续继续在 foundation 分发层追加股票专属解析分支。
    let request = match serde_json::from_value::<SyncStockPriceHistoryRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match sync_stock_price_history(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_technical_consultation_basic(args: Value) -> ToolResponse {
    // 2026-03-31 CST: 这里把股票技术咨询请求收口到 stock dispatcher，原因是技术面咨询已经是独立业务模块而非通用分析底座。
    // 目的：确保后续新增指标、评分和多周期分析时，都沿 stock 业务域演进，不再反向污染 foundation。
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
    // 2026-04-01 CST: 这里新增上层综合证券分析 Tool 的 stock dispatcher 入口，原因是用户已批准在技术面 Tool 上层叠加大盘与板块环境分析。
    // 目的：保持 `technical_consultation_basic` 边界不变，同时为 CLI / Skill 暴露统一的综合证券分析入口。
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
    // 2026-04-01 CST: 这里接入 fullstack 总 Tool 的 stock dispatcher 入口，原因是方案 1 已确定新增独立上层聚合入口；
    // 目的：保持底层技术面 Tool 边界不变，同时为 CLI / Skill 暴露“技术 + 财报 + 公告 + 行业”的统一总入口。
    let request = match serde_json::from_value::<SecurityAnalysisFullstackRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_analysis_fullstack(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_decision_evidence_bundle(args: Value) -> ToolResponse {
    // 2026-04-01 CST: 这里新增证券投决证据包 dispatcher 入口，原因是方案 B 先要把研究链结果冻结成统一证据对象；
    // 目的：让 CLI / Skill / 后续审批层都能稳定调起“同源证据冻结”，而不是重复手工拼接 fullstack 返回。
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
    // 2026-04-01 CST: 这里新增证券投决会 dispatcher 入口，原因是顶层产品需要一个单次请求完成“证据 -> 正反方 -> 闸门 -> 裁决”的总入口；
    // 目的：把研究结论升级成结构化投决结果，并为对话式 Skill 提供稳定的主调用点。
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
    let request = match serde_json::from_value::<SecurityCommitteeMemberAgentRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_committee_member_agent(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_chair_resolution(args: Value) -> ToolResponse {
    // 2026-04-09 CST: 这里新增主席正式裁决 dispatcher 入口，原因是 Task 1 要让“最终正式动作”拥有独立主链入口；
    // 目的：让 CLI / Skill 通过统一 stock dispatcher 就能拿到 committee / scorecard / chair_resolution 三线结果。
    let request = match serde_json::from_value::<SecurityChairResolutionRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_chair_resolution(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_condition_review(args: Value) -> ToolResponse {
    // 2026-04-12 CST: Add the formal condition-review dispatcher entry, because
    // P8 needs review triggers to travel through the same public stock router as
    // approval and scorecard artifacts.
    // Purpose: expose a stable CLI entry for replayable intraperiod review objects.
    let request = match serde_json::from_value::<SecurityConditionReviewRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_condition_review(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_execution_record(args: Value) -> ToolResponse {
    // 2026-04-12 CST: Add the formal execution-record dispatcher entry, because
    // P8 needs execution events to share the same public stock router as review
    // and approval artifacts.
    // Purpose: expose a stable CLI path for replayable execution facts.
    let request = match serde_json::from_value::<SecurityExecutionRecordRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_execution_record(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_post_trade_review(args: Value) -> ToolResponse {
    // 2026-04-12 CST: Add the formal post-trade review dispatcher entry, because
    // P8 needs replayable review closure to share the same public stock router as
    // approvals, reviews, and execution facts.
    // Purpose: expose a stable CLI path for layered post-trade review artifacts.
    let request = match serde_json::from_value::<SecurityPostTradeReviewRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_post_trade_review(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_feature_snapshot(args: Value) -> ToolResponse {
    // 2026-04-09 CST: 这里新增特征快照 dispatcher 入口，原因是 Task 2 要把训练底座的快照对象正式暴露到主链；
    // 目的：让 CLI / Skill / 后续训练入口能通过统一 stock dispatcher 获取冻结特征快照。
    let request = match serde_json::from_value::<SecurityFeatureSnapshotRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_feature_snapshot(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_external_proxy_backfill(args: Value) -> ToolResponse {
    // 2026-04-11 CST: Add the formal dispatcher entry for dated external-proxy
    // backfill, because P4 needs historical proxy writes to travel through the same
    // governed stock router as snapshots, outcomes, and approvals.
    // Purpose: prevent dated proxy ingestion from turning into an out-of-band script.
    let request = match serde_json::from_value::<SecurityExternalProxyBackfillRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_external_proxy_backfill(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_external_proxy_history_import(args: Value) -> ToolResponse {
    // 2026-04-12 CST: Add the formal dispatcher entry for file-based proxy-history import,
    // because Historical Data Phase 1 needs real ETF proxy batches to enter the same
    // public stock router as other governed history tools.
    // Purpose: prevent real proxy-history imports from turning into shell-only sidecars.
    let request = match serde_json::from_value::<SecurityExternalProxyHistoryImportRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_external_proxy_history_import(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_fundamental_history_backfill(args: Value) -> ToolResponse {
    // 2026-04-12 CST: Add the governed stock fundamental-history dispatcher entry,
    // because Historical Data Phase 1 needs one public stock route for replayable
    // financial snapshot writes before fullstack can prefer governed history.
    // Purpose: expose a stable CLI path for stock fundamental-history batches.
    let request = match serde_json::from_value::<SecurityFundamentalHistoryBackfillRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_fundamental_history_backfill(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_fundamental_history_live_backfill(args: Value) -> ToolResponse {
    // 2026-04-12 CST: Add the governed live financial-history dispatcher entry,
    // because Historical Data Phase 1 needs a public provider-to-governed bridge
    // before validation and shadow can consume thicker stock information history.
    // Purpose: expose one stable CLI path for multi-period live financial imports.
    let request =
        match serde_json::from_value::<SecurityFundamentalHistoryLiveBackfillRequest>(args) {
            Ok(request) => request,
            Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
        };

    match security_fundamental_history_live_backfill(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_disclosure_history_backfill(args: Value) -> ToolResponse {
    // 2026-04-12 CST: Add the governed stock disclosure-history dispatcher entry,
    // because Historical Data Phase 1 needs one public stock route for replayable
    // announcement batches before fullstack can prefer governed history.
    // Purpose: expose a stable CLI path for stock disclosure-history batches.
    let request = match serde_json::from_value::<SecurityDisclosureHistoryBackfillRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_disclosure_history_backfill(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_disclosure_history_live_backfill(args: Value) -> ToolResponse {
    // 2026-04-12 CST: Add the governed live disclosure-history dispatcher entry,
    // because Historical Data Phase 1 needs a public provider-to-governed bridge
    // before validation and shadow can consume thicker stock information history.
    // Purpose: expose one stable CLI path for multi-page live disclosure imports.
    let request = match serde_json::from_value::<SecurityDisclosureHistoryLiveBackfillRequest>(args)
    {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_disclosure_history_live_backfill(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_real_data_validation_backfill(args: Value) -> ToolResponse {
    // 2026-04-12 CST: Add the governed real-data validation dispatcher entry, because
    // validation-slice refresh should run through the same public stock router as
    // history backfill and lifecycle replay artifacts.
    // Purpose: expose one stable CLI path for live-compatible validation data refresh.
    let request = match serde_json::from_value::<SecurityRealDataValidationBackfillRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_real_data_validation_backfill(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_history_expansion(args: Value) -> ToolResponse {
    // 2026-04-11 CST: Add the governed history-expansion dispatcher entry, because
    // P5 needs one formal route for proxy-coverage growth records before shadow
    // evaluation starts reading them.
    // Purpose: keep history-expansion on the same stock mainline as training and approval.
    let request = match serde_json::from_value::<SecurityHistoryExpansionRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_history_expansion(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_shadow_evaluation(args: Value) -> ToolResponse {
    // 2026-04-11 CST: Add the governed shadow-evaluation dispatcher entry, because
    // P5 requires one first-class review object between model registry and promotion.
    // Purpose: let CLI and Skills persist promotion-readiness reviews on the stock chain.
    let request = match serde_json::from_value::<SecurityShadowEvaluationRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_shadow_evaluation(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_model_promotion(args: Value) -> ToolResponse {
    // 2026-04-11 CST: Add the governed model-promotion dispatcher entry, because
    // P5 needs auditable candidate/shadow/champion transitions instead of implicit
    // registry-only state changes.
    // Purpose: expose grade decisions as a formal stock tool for approval consumers.
    let request = match serde_json::from_value::<SecurityModelPromotionRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_model_promotion(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_forward_outcome(args: Value) -> ToolResponse {
    // 2026-04-09 CST: 这里新增未来标签回填 dispatcher 入口，原因是 Task 3 需要把 snapshot 绑定的多期限标签正式暴露到 stock 主链；
    // 目的：让 CLI / Skill / 后续训练入口都能通过统一 dispatcher 获取正式 forward_outcome 结果。
    let request = match serde_json::from_value::<SecurityForwardOutcomeRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_forward_outcome(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_master_scorecard(args: Value) -> ToolResponse {
    // 2026-04-11 CST: 这里新增 master_scorecard dispatcher 入口，原因是方案 C 已确认总卡要成为正式 CLI Tool，
    // 而不是继续停留在内部聚合函数。
    // 目的：让 CLI / Skill / 持仓报告可以通过统一 stock dispatcher 调用未来几日赚钱效益总卡。
    let request = match serde_json::from_value::<SecurityMasterScorecardRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_master_scorecard(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_security_scorecard_refit(args: Value) -> ToolResponse {
    // 2026-04-09 CST: 这里新增 scorecard refit dispatcher 入口，原因是 Task 4 需要把离线重估正式挂入证券主链路由；
    // 目的：让 CLI / Skill / 后续训练编排通过统一 stock dispatcher 获取 refit_run 与 model_registry 结果。
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
    // 2026-04-09 CST: 这里新增正式 scorecard training dispatcher 入口，原因是 Task 5 需要把训练主链接入统一 stock 路由；
    // 目的：让 CLI / Skill / 回算编排都能通过同一个 dispatcher 获取 artifact、refit_run 与 model_registry。
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
    // 2026-04-02 CST: 这里新增证券审批提交 dispatcher 入口，原因是 P0-1 要让证券投决结果正式进入审批主线；
    // 目的：把 “committee -> approval objects -> runtime files” 收口成一个稳定 Tool，而不是外层分散拼接。
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
    // 2026-04-02 CST: 这里新增证券审批包校验 dispatcher 入口，原因是正式 decision package 已经生成，需要一个主链可调用的核验 Tool；
    // 目的：把 “读取 package -> 校验工件/哈希/签名/治理绑定 -> 返回 verification report” 收口成稳定产品入口。
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
    // 2026-04-02 CST: 这里新增证券审批包版本化 dispatcher 入口，原因是 decision package 需要随着审批动作生成新版本；
    // 目的：把 “读取旧 package -> 重建 manifest -> 生成 v2 package -> 可选重跑 verify” 收口成正式主链 Tool。
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
    // 2026-04-08 CST: 这里接入会后结论记录 dispatcher 入口，原因是 Task 3 需要正式的会后治理 Tool；
    // 目的：让 CLI / Skill 通过统一 stock dispatcher 完成“结论落盘 + revision”动作。
    let request = match serde_json::from_value::<SecurityRecordPostMeetingConclusionRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_record_post_meeting_conclusion(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}
