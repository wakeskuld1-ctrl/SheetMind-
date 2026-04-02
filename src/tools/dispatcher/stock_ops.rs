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
use crate::ops::stock::security_committee_vote::{
    SecurityCommitteeVoteRequest, security_committee_vote,
};
use crate::ops::stock::security_decision_briefing::{
    SecurityDecisionBriefingRequest, security_decision_briefing,
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
use crate::tools::contracts::ToolResponse;

pub(super) fn dispatch_import_stock_price_history(args: Value) -> ToolResponse {
    // 2026-03-31 CST：这里把股票历史导入请求收口到 stock dispatcher，原因是股票导入已不再属于 foundation 分析域；
    // 目的：让 “CSV -> SQLite” 的股票入口单独沿 stock 模块扩展，而不是继续挂在通用分析分发层里。
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
    // 2026-03-31 CST：这里把股票历史同步请求收口到 stock dispatcher，原因是 provider 顺序和补数逻辑属于股票域内部细节；
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

// 2026-04-02 CST: 这里补模板级共振因子同步 dispatcher，原因是方案C要求“模板补数”必须走正式 stock Tool 主链；
// 目的：让银行宏观代理序列的同步、转换和落库不再依赖外部脚本，而是可以被 CLI / Skill 稳定发现和调用。
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
    // 2026-03-31 CST：这里把股票技术面咨询请求收口到 stock dispatcher，原因是技术面咨询已是独立业务模块；
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
    // 2026-04-01 CST：这里接入综合证券分析 contextual Tool，原因是用户已批准在技术面上层叠加大盘与板块环境；
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
    // 2026-04-01 CST：这里接入 fullstack Tool，原因是既有主链已经确定要把技术、财报、公告和行业统一聚合；
    // 目的：让 CLI / Skill 直接消费完整证券分析结果，而不是在外层继续手工拼接信息面。
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
    // 2026-04-02 CST: 这里接入 security_decision_briefing 的 stock dispatcher 分支，原因是统一 briefing 已经成为咨询与投决共用的事实入口；
    // 目的：让 CLI / Skill 可以直接走正式 Tool 主链拿到单一 briefing，而不是在外层手工串 fullstack 与 resonance。
    let request = match serde_json::from_value::<SecurityDecisionBriefingRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match security_decision_briefing(&request) {
        Ok(result) => ToolResponse::ok_serialized(&result),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

// 2026-04-02 CST: 这里接入 security_committee_vote 的 stock dispatcher，原因是投决会必须沿正式 Tool 主链暴露，
// 目的：让上层只传 committee payload / committee_mode 就能拿到结构化表决结果，而不是再去拼第二套流程。
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

pub(super) fn dispatch_register_resonance_factor(args: Value) -> ToolResponse {
    // 2026-04-02 CST：这里接入因子注册入口，原因是方案 3 已确认先做“平台底层”，而不是只做一次性分析输出；
    // 目的：让新共振想法可以先注册为正式因子，再落序列、跑评估和进入分析主链。
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
    // 2026-04-02 CST：这里接入因子序列写库入口，原因是用户要求“算出来以后写到数据库里，再把相关性强的拉出来评估”；
    // 目的：把价格、运价、汇率等候选因子沉淀成正式日度序列资产。
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
    // 2026-04-02 CST：这里接入事件标签写库入口，原因是事件标签已被纳入第一版平台而不是后补；
    // 目的：让地缘、政策、运输瓶颈等非价格事件也能通过正式 Tool 主链进入平台。
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
    // 2026-04-02 CST：这里接入模板池初始化入口，原因是第二阶段方案 B 要把传统行业候选因子池正式暴露给 Tool 主链；
    // 目的：让 Agent/Skill 可以先初始化行业底座，再做独立评估或最终分析。
    let request = match serde_json::from_value::<BootstrapResonanceTemplateFactorsRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match bootstrap_resonance_template_factors(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_evaluate_security_resonance(args: Value) -> ToolResponse {
    // 2026-04-02 CST：这里接入独立评估入口，原因是第二阶段已经确认“研究评估”和“fullstack 最终分析”需要拆开；
    // 目的：让 Agent/Skill 可以只跑共振评估并落快照，而不是所有场景都强绑信息面抓取。
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
    // 2026-04-02 CST：这里接入共振平台分析入口，原因是用户已经明确要求国际与行业证券分析必须显式暴露共振驱动；
    // 目的：复用 fullstack 主链，再把板块、商品、事件和注册因子一起聚合成正式分析结果。
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
    // 2026-04-02 CST: 这里接入 research snapshot Tool，原因是方案C第一批任务要求把“当前完整指标状态”做成正式研究平台入口，
    // 目的：让上层先能稳定触发并落库 snapshot，后续再围绕同一主键扩展 forward returns 与 analog study。
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
    // 2026-04-02 CST: 这里接入 forward returns 回填 Tool，原因是方案C第二步要求把 snapshot 后续收益研究做成正式平台链路，
    // 目的：让研究层可以围绕已落库快照统一回填 1/3/5/10/20 日结果，而不是每次由上层临时扫描历史。
    let request = match serde_json::from_value::<BackfillSecuritySignalOutcomesRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match backfill_security_signal_outcomes(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

// 2026-04-02 CST: 这里接入历史相似研究 Tool，原因是用户明确要求把银行体系内“共振 + MACD/RSRS 等技术状态相似”
// 的样本统计做成正式平台入口；目的：让上层能用统一 Tool 主链生成并持久化 analog study，而不是手工离线统计。
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

// 2026-04-02 CST: 这里接入历史研究摘要读取 Tool，原因是 security_decision_briefing / committee payload
// 要读取统一研究结论，而不是各层自行拼接；目的：让咨询与投决共享同一份 historical digest 数据源。
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
