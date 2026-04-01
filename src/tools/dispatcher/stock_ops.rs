use serde_json::{json, Value};

use crate::ops::stock::import_stock_price_history::{
    import_stock_price_history, ImportStockPriceHistoryRequest,
};
use crate::ops::stock::security_analysis_contextual::{
    security_analysis_contextual, SecurityAnalysisContextualRequest,
};
use crate::ops::stock::security_analysis_fullstack::{
    security_analysis_fullstack, SecurityAnalysisFullstackRequest,
};
use crate::ops::stock::security_decision_committee::{
    security_decision_committee, SecurityDecisionCommitteeRequest,
};
use crate::ops::stock::security_decision_evidence_bundle::{
    security_decision_evidence_bundle, SecurityDecisionEvidenceBundleRequest,
};
use crate::ops::stock::security_decision_submit_approval::{
    security_decision_submit_approval, SecurityDecisionSubmitApprovalRequest,
};
use crate::ops::stock::security_decision_package_revision::{
    security_decision_package_revision, SecurityDecisionPackageRevisionRequest,
};
use crate::ops::stock::security_decision_verify_package::{
    security_decision_verify_package, SecurityDecisionVerifyPackageRequest,
};
use crate::ops::stock::sync_stock_price_history::{
    sync_stock_price_history, SyncStockPriceHistoryRequest,
};
use crate::ops::stock::technical_consultation_basic::{
    technical_consultation_basic, TechnicalConsultationBasicRequest,
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
