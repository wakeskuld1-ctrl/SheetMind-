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
