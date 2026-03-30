use serde_json::{Value, json};

use crate::ops::analyze::analyze_table;
use crate::ops::capacity_assessment::{CapacityAssessmentRequest, capacity_assessment};
use crate::ops::capacity_assessment_excel_report::{
    CapacityAssessmentExcelReportRequest, capacity_assessment_excel_report,
};
use crate::ops::capacity_assessment_from_inventory::{
    CapacityAssessmentFromInventoryRequest, capacity_assessment_from_inventory,
};
use crate::ops::cluster_kmeans::cluster_kmeans;
use crate::ops::correlation_analysis::correlation_analysis;
use crate::ops::decision_assistant::decision_assistant;
use crate::ops::diagnostics_report::{DiagnosticsReportRequest, diagnostics_report};
use crate::ops::diagnostics_report_excel_report::{
    DiagnosticsReportExcelReportRequest, diagnostics_report_excel_report,
};
use crate::ops::distribution_analysis::distribution_analysis;
use crate::ops::import_stock_price_history::{
    ImportStockPriceHistoryRequest, import_stock_price_history,
};
use crate::ops::linear_regression::linear_regression;
use crate::ops::logistic_regression::logistic_regression;
use crate::ops::outlier_detection::{OutlierDetectionMethod, outlier_detection};
use crate::ops::preview::preview_table;
use crate::ops::ssh_inventory::{SshInventoryRequest, ssh_inventory};
use crate::ops::stat_summary::stat_summary;
use crate::ops::summary::summarize_table;
use crate::ops::sync_stock_price_history::{
    SyncStockPriceHistoryRequest, sync_stock_price_history,
};
use crate::ops::technical_consultation_basic::{
    TechnicalConsultationBasicRequest, technical_consultation_basic,
};
use crate::ops::trend_analysis::trend_analysis;
use crate::runtime::local_memory::SessionStage;
use crate::tools::contracts::ToolResponse;
use crate::tools::session;
use crate::tools::sources;

use super::shared::{apply_optional_casts, parse_casts, parse_missing_strategy, string_array};

pub(super) fn dispatch_summarize_table(args: Value) -> ToolResponse {
    let requested_columns = string_array(&args, "columns");
    let top_k = args
        .get("top_k")
        .and_then(|value| value.as_u64())
        .unwrap_or(5) as usize;
    let casts = match parse_casts(&args, "casts", "summarize_table") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match sources::load_table_for_analysis(&args, "summarize_table") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => {
                match summarize_table(&prepared_loaded, &requested_columns, top_k) {
                    Ok(summaries) => {
                        if let Err(response) = session::sync_loaded_table_state(
                            &args,
                            &prepared_loaded,
                            SessionStage::AnalysisModeling,
                            "analysis completed",
                            "summarize_table",
                            "analysis_completed",
                        ) {
                            return response;
                        }
                        ToolResponse::ok(json!({
                            "row_count": prepared_loaded.dataframe.height(),
                            "summaries": summaries,
                        }))
                    }
                    Err(error) => ToolResponse::error(error.to_string()),
                }
            }
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

pub(super) fn dispatch_analyze_table(args: Value) -> ToolResponse {
    let requested_columns = string_array(&args, "columns");
    let top_k = args
        .get("top_k")
        .and_then(|value| value.as_u64())
        .unwrap_or(5) as usize;
    let casts = match parse_casts(&args, "casts", "analyze_table") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match sources::load_table_for_analysis(&args, "analyze_table") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => {
                let result = analyze_table(&prepared_loaded, &requested_columns, top_k);
                if let Err(response) = session::sync_loaded_table_state(
                    &args,
                    &prepared_loaded,
                    SessionStage::AnalysisModeling,
                    "analysis completed",
                    "analyze_table",
                    "analysis_completed",
                ) {
                    return response;
                }
                ToolResponse::ok(json!(result))
            }
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

pub(super) fn dispatch_stat_summary(args: Value) -> ToolResponse {
    let requested_columns = string_array(&args, "columns");
    let top_k = args
        .get("top_k")
        .and_then(|value| value.as_u64())
        .unwrap_or(5) as usize;
    let casts = match parse_casts(&args, "casts", "stat_summary") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match sources::load_table_for_analysis(&args, "stat_summary") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => {
                match stat_summary(&prepared_loaded, &requested_columns, top_k) {
                    Ok(result) => {
                        if let Err(response) = session::sync_loaded_table_state(
                            &args,
                            &prepared_loaded,
                            SessionStage::AnalysisModeling,
                            "analysis completed",
                            "stat_summary",
                            "analysis_completed",
                        ) {
                            return response;
                        }
                        ToolResponse::ok(json!(result))
                    }
                    Err(error) => ToolResponse::error(error.to_string()),
                }
            }
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

pub(super) fn dispatch_correlation_analysis(args: Value) -> ToolResponse {
    let Some(target_column) = args.get("target_column").and_then(|value| value.as_str()) else {
        return ToolResponse::error("invalid request parameters");
    };
    let feature_columns = string_array(&args, "feature_columns");
    let casts = match parse_casts(&args, "casts", "correlation_analysis") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match sources::load_table_for_analysis(&args, "correlation_analysis") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => {
                match correlation_analysis(&prepared_loaded, target_column, &feature_columns) {
                    Ok(result) => {
                        if let Err(response) = session::sync_loaded_table_state(
                            &args,
                            &prepared_loaded,
                            SessionStage::AnalysisModeling,
                            "analysis completed",
                            "correlation_analysis",
                            "analysis_completed",
                        ) {
                            return response;
                        }
                        ToolResponse::ok(json!(result))
                    }
                    Err(error) => ToolResponse::error(error.to_string()),
                }
            }
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

pub(super) fn dispatch_outlier_detection(args: Value) -> ToolResponse {
    let columns = string_array(&args, "columns");
    let method = match args.get("method") {
        Some(value) => match serde_json::from_value::<OutlierDetectionMethod>(value.clone()) {
            Ok(method) => method,
            Err(error) => {
                return ToolResponse::error(format!("request parsing failed: {error}"));
            }
        },
        None => OutlierDetectionMethod::Iqr,
    };
    let casts = match parse_casts(&args, "casts", "outlier_detection") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match sources::load_table_for_analysis(&args, "outlier_detection") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => match outlier_detection(&prepared_loaded, &columns, method) {
                Ok((flagged_loaded, result)) => {
                    if let Err(response) = session::sync_loaded_table_state(
                        &args,
                        &prepared_loaded,
                        SessionStage::AnalysisModeling,
                        "analysis completed",
                        "outlier_detection",
                        "analysis_completed",
                    ) {
                        return response;
                    }
                    let preview = preview_table(&flagged_loaded.dataframe, 20);
                    let columns = preview
                        .as_ref()
                        .map(|item| item.columns.clone())
                        .unwrap_or_default();
                    let rows = preview.map(|item| item.rows).unwrap_or_default();
                    crate::tools::results::respond_with_result_dataset(
                        "outlier_detection",
                        &args,
                        &flagged_loaded,
                        json!({
                            "method": result.method,
                            "row_count": result.row_count,
                            "outlier_summaries": result.outlier_summaries,
                            "human_summary": result.human_summary,
                            "columns": columns,
                            "rows": rows
                        }),
                    )
                }
                Err(error) => ToolResponse::error(error.to_string()),
            },
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

pub(super) fn dispatch_distribution_analysis(args: Value) -> ToolResponse {
    let Some(column) = args.get("column").and_then(|value| value.as_str()) else {
        return ToolResponse::error("invalid request parameters");
    };
    let bins = args
        .get("bins")
        .and_then(|value| value.as_u64())
        .unwrap_or(10) as usize;
    let casts = match parse_casts(&args, "casts", "distribution_analysis") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match sources::load_table_for_analysis(&args, "distribution_analysis") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => match distribution_analysis(&prepared_loaded, column, bins) {
                Ok(result) => {
                    if let Err(response) = session::sync_loaded_table_state(
                        &args,
                        &prepared_loaded,
                        SessionStage::AnalysisModeling,
                        "analysis completed",
                        "distribution_analysis",
                        "analysis_completed",
                    ) {
                        return response;
                    }
                    ToolResponse::ok(json!(result))
                }
                Err(error) => ToolResponse::error(error.to_string()),
            },
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

pub(super) fn dispatch_trend_analysis(args: Value) -> ToolResponse {
    let Some(time_column) = args.get("time_column").and_then(|value| value.as_str()) else {
        return ToolResponse::error("invalid request parameters");
    };
    let Some(value_column) = args.get("value_column").and_then(|value| value.as_str()) else {
        return ToolResponse::error("invalid request parameters");
    };
    let casts = match parse_casts(&args, "casts", "trend_analysis") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match sources::load_table_for_analysis(&args, "trend_analysis") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => {
                match trend_analysis(&prepared_loaded, time_column, value_column) {
                    Ok(result) => {
                        if let Err(response) = session::sync_loaded_table_state(
                            &args,
                            &prepared_loaded,
                            SessionStage::AnalysisModeling,
                            "analysis completed",
                            "trend_analysis",
                            "analysis_completed",
                        ) {
                            return response;
                        }
                        ToolResponse::ok(json!(result))
                    }
                    Err(error) => ToolResponse::error(error.to_string()),
                }
            }
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

pub(super) fn dispatch_import_stock_price_history(args: Value) -> ToolResponse {
    // 2026-03-28 CST: 这里先把股票历史导入请求收口成强类型，原因是第一刀只做稳定的 CSV -> SQLite 合同；
    // 目的：避免 dispatcher 层散落手工取字段，并为后续 Skill 调用保留统一入口。
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
    // 2026-03-29 CST: 这里先把股票历史 HTTP 同步请求收口成强类型，原因是 provider 顺序、日期区间和复权参数都要稳定解析；
    // 目的：避免 dispatcher 层散落手工字段判断，并为后续继续加 provider 保留统一入口。
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
    // 2026-03-28 CST: 这里先把股票技术面基础请求收口成强类型，原因是新 Tool 也要沿现有 Rust 主链稳定暴露；
    // 目的：避免 dispatcher 层散落解析 `symbol / lookback_days`，并为后续 Skill 承接保留统一入口。
    let request = match serde_json::from_value::<TechnicalConsultationBasicRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match technical_consultation_basic(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_diagnostics_report(args: Value) -> ToolResponse {
    // 2026-03-28 23:54 CST: 这里先把组合诊断请求解析成强类型，原因是高层 Tool 同时承接多个 section 配置；
    // 目的是避免 dispatcher 手工拆四组弱类型字段，保持后续 Rust 高层能力扩展可控。
    let request = match serde_json::from_value::<DiagnosticsReportRequest>(args.clone()) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };
    let casts = match parse_casts(&args, "casts", "diagnostics_report") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match sources::load_table_for_analysis(&args, "diagnostics_report") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => match diagnostics_report(&prepared_loaded, &request) {
                Ok(result) => {
                    if let Err(response) = session::sync_loaded_table_state(
                        &args,
                        &prepared_loaded,
                        SessionStage::AnalysisModeling,
                        "analysis completed",
                        "diagnostics_report",
                        "analysis_completed",
                    ) {
                        return response;
                    }
                    ToolResponse::ok(json!(result))
                }
                Err(error) => ToolResponse::error(error.to_string()),
            },
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

pub(super) fn dispatch_diagnostics_report_excel_report(args: Value) -> ToolResponse {
    // 2026-03-29 00:08 CST：这里先把组合诊断 Excel 报表请求解析成强类型，原因是该 Tool 同时承接诊断参数和 workbook 交付参数；
    // 目的：避免 dispatcher 内再散落多组弱类型字段解析分支，并保持后续 Rust 交付 Tool 的扩展方式一致。
    let request = match serde_json::from_value::<DiagnosticsReportExcelReportRequest>(args.clone())
    {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };
    let casts = match parse_casts(&args, "casts", "diagnostics_report_excel_report") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match sources::load_table_for_analysis(&args, "diagnostics_report_excel_report") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => {
                match diagnostics_report_excel_report(&prepared_loaded, &request) {
                    Ok(result) => {
                        if let Err(response) = session::sync_output_handle_state(
                            &args,
                            &result.workbook_ref,
                            "workbook_ref",
                            "diagnostics_report_excel_report",
                        ) {
                            return response;
                        }
                        ToolResponse::ok(json!(result))
                    }
                    Err(error) => ToolResponse::error(error),
                }
            }
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

pub(super) fn dispatch_capacity_assessment(args: Value) -> ToolResponse {
    // 2026-03-28 10:43 CST: 这里先解析容量评估请求，原因是要把阈值和列映射从通用 JSON 参数收口为强类型；目的是减少后续场景扩展时的分支散落。
    let request = match serde_json::from_value::<CapacityAssessmentRequest>(args.clone()) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };
    let casts = match parse_casts(&args, "casts", "capacity_assessment") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match sources::load_table_for_analysis(&args, "capacity_assessment") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => {
                let result = capacity_assessment(&prepared_loaded, &request);
                if let Err(response) = session::sync_loaded_table_state(
                    &args,
                    &prepared_loaded,
                    SessionStage::DecisionAssistant,
                    "capacity assessment completed",
                    "capacity_assessment",
                    "capacity_assessment_completed",
                ) {
                    return response;
                }
                ToolResponse::ok(json!(result))
            }
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

pub(super) fn dispatch_capacity_assessment_from_inventory(args: Value) -> ToolResponse {
    // 2026-03-28 16:55 CST: 这里分开解析桥接请求和容量请求，原因是桥接层需要额外接收 SSH 盘点输入；目的是保持底层容量 Tool 的契约稳定。
    let bridge_request =
        match serde_json::from_value::<CapacityAssessmentFromInventoryRequest>(args.clone()) {
            Ok(request) => request,
            Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
        };
    let capacity_request = match serde_json::from_value::<CapacityAssessmentRequest>(args.clone()) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    let has_source = args.get("path").is_some()
        || args.get("sheet").is_some()
        || args.get("table_ref").is_some()
        || args.get("result_ref").is_some()
        || args.get("file_ref").is_some();

    if has_source {
        match sources::load_table_for_analysis(&args, "capacity_assessment_from_inventory") {
            Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
            Ok(sources::OperationLoad::Loaded(loaded)) => {
                match capacity_assessment_from_inventory(
                    Some(&loaded),
                    &bridge_request,
                    &capacity_request,
                ) {
                    Ok(result) => {
                        if let Err(response) = session::sync_loaded_table_state(
                            &args,
                            &loaded,
                            SessionStage::DecisionAssistant,
                            "capacity assessment from inventory completed",
                            "capacity_assessment_from_inventory",
                            "capacity_assessment_from_inventory_completed",
                        ) {
                            return response;
                        }
                        ToolResponse::ok(json!(result))
                    }
                    Err(error) => ToolResponse::error(error),
                }
            }
            Err(response) => response,
        }
    } else {
        match capacity_assessment_from_inventory(None, &bridge_request, &capacity_request) {
            Ok(result) => ToolResponse::ok(json!(result)),
            Err(error) => ToolResponse::error(error),
        }
    }
}

pub(super) fn dispatch_capacity_assessment_excel_report(args: Value) -> ToolResponse {
    // 2026-03-28 22:20 CST: 这里先把高层 Excel 报表请求解析成强类型，原因是该 Tool 同时承接分析、桥接和交付参数；
    // 目的是避免 dispatcher 内再散落多组弱类型字段解析分支。
    let request = match serde_json::from_value::<CapacityAssessmentExcelReportRequest>(args.clone())
    {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    let has_source = args.get("path").is_some()
        || args.get("sheet").is_some()
        || args.get("table_ref").is_some()
        || args.get("result_ref").is_some()
        || args.get("file_ref").is_some();

    // 2026-03-28 22:20 CST: 这里允许无表源执行，原因是用户明确要求“给不了 Excel 也要能输出决策思路”；
    // 目的是让 guidance-only / inventory-only 场景也能直接形成 Excel 交付物。
    let result = if has_source {
        match sources::load_table_for_analysis(&args, "capacity_assessment_excel_report") {
            Ok(sources::OperationLoad::NeedsConfirmation(response)) => return response,
            Ok(sources::OperationLoad::Loaded(loaded)) => {
                match capacity_assessment_excel_report(Some(&loaded), &request) {
                    Ok(result) => result,
                    Err(error) => return ToolResponse::error(error),
                }
            }
            Err(response) => return response,
        }
    } else {
        match capacity_assessment_excel_report(None, &request) {
            Ok(result) => result,
            Err(error) => return ToolResponse::error(error),
        }
    };

    if let Err(response) = session::sync_output_handle_state(
        &args,
        &result.workbook_ref,
        "workbook_ref",
        "capacity_assessment_excel_report",
    ) {
        return response;
    }

    ToolResponse::ok(json!(result))
}

pub(super) fn dispatch_ssh_inventory(args: Value) -> ToolResponse {
    // 2026-03-28 16:12 CST: 这里先把 JSON 参数收口成受限 SSH 请求，原因是要在分发层统一做强类型解析；目的是让白名单校验和执行逻辑都走同一条安全入口。
    let request = match serde_json::from_value::<SshInventoryRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    // 2026-03-28 16:12 CST: 这里直接调用受限 SSH 盘点实现，原因是该 Tool 不依赖工作簿上下文；目的是让调用方在没有 Excel 明细时也能先拿到部署与主机事实证据。
    match ssh_inventory(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error),
    }
}

pub(super) fn dispatch_linear_regression(args: Value) -> ToolResponse {
    let Some(features_value) = args.get("features").and_then(|value| value.as_array()) else {
        return ToolResponse::error("invalid request parameters");
    };
    let Some(target) = args.get("target").and_then(|value| value.as_str()) else {
        return ToolResponse::error("invalid request parameters");
    };
    let features = features_value
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>();
    let intercept = args
        .get("intercept")
        .and_then(|value| value.as_bool())
        .unwrap_or(true);
    let missing_strategy = match parse_missing_strategy(&args, "linear_regression") {
        Ok(strategy) => strategy,
        Err(response) => return response,
    };
    let casts = match parse_casts(&args, "casts", "linear_regression") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match sources::load_table_for_analysis(&args, "linear_regression") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => match linear_regression(
                &prepared_loaded,
                &features,
                target,
                intercept,
                missing_strategy,
            ) {
                Ok(result) => {
                    if let Err(response) = session::sync_loaded_table_state(
                        &args,
                        &prepared_loaded,
                        SessionStage::AnalysisModeling,
                        "modeling completed",
                        "linear_regression",
                        "modeling_completed",
                    ) {
                        return response;
                    }
                    ToolResponse::ok(json!(result))
                }
                Err(error) => ToolResponse::error(error.to_string()),
            },
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

pub(super) fn dispatch_logistic_regression(args: Value) -> ToolResponse {
    let Some(features_value) = args.get("features").and_then(|value| value.as_array()) else {
        return ToolResponse::error("invalid request parameters");
    };
    let Some(target) = args.get("target").and_then(|value| value.as_str()) else {
        return ToolResponse::error("invalid request parameters");
    };
    let features = features_value
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>();
    let intercept = args
        .get("intercept")
        .and_then(|value| value.as_bool())
        .unwrap_or(true);
    let positive_label = args.get("positive_label").and_then(|value| value.as_str());
    let missing_strategy = match parse_missing_strategy(&args, "logistic_regression") {
        Ok(strategy) => strategy,
        Err(response) => return response,
    };
    let casts = match parse_casts(&args, "casts", "logistic_regression") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match sources::load_table_for_analysis(&args, "logistic_regression") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => match logistic_regression(
                &prepared_loaded,
                &features,
                target,
                intercept,
                missing_strategy,
                positive_label,
            ) {
                Ok(result) => {
                    if let Err(response) = session::sync_loaded_table_state(
                        &args,
                        &prepared_loaded,
                        SessionStage::AnalysisModeling,
                        "modeling completed",
                        "logistic_regression",
                        "modeling_completed",
                    ) {
                        return response;
                    }
                    ToolResponse::ok(json!(result))
                }
                Err(error) => ToolResponse::error(error.to_string()),
            },
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

pub(super) fn dispatch_cluster_kmeans(args: Value) -> ToolResponse {
    let Some(features_value) = args.get("features").and_then(|value| value.as_array()) else {
        return ToolResponse::error("invalid request parameters");
    };
    let Some(cluster_count) = args.get("cluster_count").and_then(|value| value.as_u64()) else {
        return ToolResponse::error("invalid request parameters");
    };
    let features = features_value
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>();
    let max_iterations = args
        .get("max_iterations")
        .and_then(|value| value.as_u64())
        .unwrap_or(100) as usize;
    let missing_strategy = match parse_missing_strategy(&args, "cluster_kmeans") {
        Ok(strategy) => strategy,
        Err(response) => return response,
    };
    let casts = match parse_casts(&args, "casts", "cluster_kmeans") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match sources::load_table_for_analysis(&args, "cluster_kmeans") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => match cluster_kmeans(
                &prepared_loaded,
                &features,
                cluster_count as usize,
                max_iterations,
                missing_strategy,
            ) {
                Ok(result) => {
                    if let Err(response) = session::sync_loaded_table_state(
                        &args,
                        &prepared_loaded,
                        SessionStage::AnalysisModeling,
                        "modeling completed",
                        "cluster_kmeans",
                        "modeling_completed",
                    ) {
                        return response;
                    }
                    ToolResponse::ok(json!(result))
                }
                Err(error) => ToolResponse::error(error.to_string()),
            },
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

pub(super) fn dispatch_decision_assistant(args: Value) -> ToolResponse {
    let requested_columns = string_array(&args, "columns");
    let top_k = args
        .get("top_k")
        .and_then(|value| value.as_u64())
        .unwrap_or(5) as usize;
    let casts = match parse_casts(&args, "casts", "decision_assistant") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match sources::load_table_for_analysis(&args, "decision_assistant") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => {
                match decision_assistant(&prepared_loaded, &requested_columns, top_k) {
                    Ok(result) => {
                        if let Err(response) = session::sync_loaded_table_state(
                            &args,
                            &prepared_loaded,
                            SessionStage::DecisionAssistant,
                            "decision assistant completed",
                            "decision_assistant",
                            "decision_assistant_completed",
                        ) {
                            return response;
                        }
                        ToolResponse::ok(json!(result))
                    }
                    Err(error) => ToolResponse::error(error.to_string()),
                }
            }
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}
