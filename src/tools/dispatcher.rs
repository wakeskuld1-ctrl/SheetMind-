use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;
use serde_json::{Value, json};

use crate::domain::handles::TableHandle;
use crate::domain::schema::{ConfidenceLevel, HeaderInference, infer_schema_state_label};
use crate::excel::header_inference::infer_header_schema;
use crate::excel::reader::{list_sheets, open_workbook};
use crate::excel::sheet_range::inspect_sheet_range;
use crate::frame::chart_ref_store::{
    ChartDraftStore, PersistedChartDraft, PersistedChartSeriesSpec, PersistedChartType,
};
use crate::frame::loader::{LoadedTable, load_confirmed_table, load_table_from_table_ref};
use crate::frame::region_loader::load_table_region;
use crate::frame::result_ref_store::{PersistedResultDataset, ResultRefStore};
use crate::frame::source_file_ref_store::{PersistedSourceFileRef, SourceFileRefStore};
use crate::frame::table_ref_store::{PersistedTableRef, TableRefStore};
use crate::frame::workbook_ref_store::{
    PersistedWorkbookDraft, WorkbookDraftStore, WorkbookSheetInput,
};
use crate::ops::analyze::analyze_table;
use crate::ops::append::append_tables;
use crate::ops::cast::{CastColumnSpec, cast_column_types, summarize_column_types};
use crate::ops::chart_svg::render_chart_svg;
use crate::ops::cluster_kmeans::cluster_kmeans;
use crate::ops::correlation_analysis::correlation_analysis;
use crate::ops::decision_assistant::decision_assistant;
use crate::ops::deduplicate_by_key::{DeduplicateKeep, OrderSpec, deduplicate_by_key};
use crate::ops::derive::{DerivationSpec, derive_columns};
use crate::ops::distinct_rows::{DistinctKeep, distinct_rows};
use crate::ops::distribution_analysis::distribution_analysis;
use crate::ops::export::{export_csv, export_excel, export_excel_workbook};
use crate::ops::fill_lookup::{FillLookupRule, fill_missing_from_lookup_by_keys};
use crate::ops::fill_missing_values::{FillMissingRule, fill_missing_values};
use crate::ops::filter::{FilterCondition, filter_rows};
use crate::ops::format_table_for_export::{ExportFormatOptions, format_table_for_export};
use crate::ops::group::{AggregationSpec, group_and_aggregate};
use crate::ops::join::{JoinKeepMode, join_preflight, join_tables};
use crate::ops::linear_regression::linear_regression;
use crate::ops::logistic_regression::logistic_regression;
use crate::ops::lookup_values::{LookupSelect, lookup_values_by_keys};
use crate::ops::model_prep::MissingStrategy;
use crate::ops::multi_table_plan::suggest_multi_table_plan;
use crate::ops::normalize_text::{NormalizeTextRule, normalize_text_columns};
use crate::ops::outlier_detection::{OutlierDetectionMethod, outlier_detection};
use crate::ops::parse_datetime::{ParseDateTimeRule, parse_datetime_columns};
use crate::ops::pivot::{PivotAggregation, pivot_table};
use crate::ops::preview::preview_table;
use crate::ops::rename::{RenameColumnMapping, rename_columns};
use crate::ops::report_delivery::{
    ReportDeliveryChart, ReportDeliveryChartSeries, ReportDeliveryChartType,
    ReportDeliveryLegendPosition, ReportDeliveryRequest, ReportDeliverySection,
    build_report_delivery_draft, chart_ref_to_report_delivery_chart,
};
use crate::ops::select::select_columns;
use crate::ops::sort::{SortSpec, sort_rows};
use crate::ops::stat_summary::stat_summary;
use crate::ops::summary::summarize_table;
use crate::ops::table_links::suggest_table_links;
use crate::ops::table_workflow::suggest_table_workflow;
use crate::ops::top_n::top_n_rows;
use crate::ops::trend_analysis::trend_analysis;
use crate::ops::window::{WindowCalculation, WindowOrderSpec, window_calculation};
use crate::runtime::local_memory::{
    EventLogInput, LocalMemoryRuntime, SchemaStatus, SessionStage, SessionStatePatch,
};
use crate::tools::contracts::{ToolRequest, ToolResponse};

pub fn dispatch(request: ToolRequest) -> ToolResponse {
    let ToolRequest { tool, args } = request;
    let args = resolve_result_ref_bindings(args);

    match tool.as_str() {
        "open_workbook" => dispatch_open_workbook(args),
        "list_sheets" => dispatch_list_sheets(args),
        "inspect_sheet_range" => dispatch_inspect_sheet_range(args),
        "load_table_region" => dispatch_load_table_region(args),
        "normalize_table" => dispatch_normalize_table(args),
        "apply_header_schema" => dispatch_apply_header_schema(args),
        "get_session_state" => dispatch_get_session_state(args),
        "update_session_state" => dispatch_update_session_state(args),
        "preview_table" => dispatch_preview_table(args),
        "select_columns" => dispatch_select_columns(args),
        "normalize_text_columns" => dispatch_normalize_text_columns(args),
        "rename_columns" => dispatch_rename_columns(args),
        "fill_missing_values" => dispatch_fill_missing_values(args),
        "distinct_rows" => dispatch_distinct_rows(args),
        "deduplicate_by_key" => dispatch_deduplicate_by_key(args),
        "format_table_for_export" => dispatch_format_table_for_export(args),
        "fill_missing_from_lookup" => dispatch_fill_missing_from_lookup(args),
        "parse_datetime_columns" => dispatch_parse_datetime_columns(args),
        "lookup_values" => dispatch_lookup_values(args),
        "window_calculation" => dispatch_window_calculation(args),
        "filter_rows" => dispatch_filter_rows(args),
        "cast_column_types" => dispatch_cast_column_types(args),
        "derive_columns" => dispatch_derive_columns(args),
        "group_and_aggregate" => dispatch_group_and_aggregate(args),
        "pivot_table" => dispatch_pivot_table(args),
        "sort_rows" => dispatch_sort_rows(args),
        "top_n" => dispatch_top_n(args),
        "compose_workbook" => dispatch_compose_workbook(args),
        "report_delivery" => dispatch_report_delivery(args),
        "build_chart" => dispatch_build_chart(args),
        "export_chart_image" => dispatch_export_chart_image(args),
        "export_csv" => dispatch_export_csv(args),
        "export_excel" => dispatch_export_excel(args),
        "export_excel_workbook" => dispatch_export_excel_workbook(args),
        "join_preflight" => dispatch_join_preflight(args),
        "join_tables" => dispatch_join_tables(args),
        "suggest_table_links" => dispatch_suggest_table_links(args),
        "suggest_table_workflow" => dispatch_suggest_table_workflow(args),
        "suggest_multi_table_plan" => dispatch_suggest_multi_table_plan(args),
        "execute_suggested_tool_call" => dispatch_execute_suggested_tool_call(args),
        "execute_multi_table_plan" => dispatch_execute_multi_table_plan(args),
        "append_tables" => dispatch_append_tables(args),
        "summarize_table" => dispatch_summarize_table(args),
        "analyze_table" => dispatch_analyze_table(args),
        "stat_summary" => dispatch_stat_summary(args),
        "correlation_analysis" => dispatch_correlation_analysis(args),
        "outlier_detection" => dispatch_outlier_detection(args),
        "distribution_analysis" => dispatch_distribution_analysis(args),
        "trend_analysis" => dispatch_trend_analysis(args),
        "linear_regression" => dispatch_linear_regression(args),
        "logistic_regression" => dispatch_logistic_regression(args),
        "cluster_kmeans" => dispatch_cluster_kmeans(args),
        "decision_assistant" => dispatch_decision_assistant(args),
        _ => ToolResponse::error(format!("不支持的 tool: {}", tool)),
    }
}

fn resolve_result_ref_bindings(args: Value) -> Value {
    let Some(bindings_value) = args.get("result_ref_bindings").cloned() else {
        return args;
    };
    let Some(bindings) = bindings_value.as_object() else {
        return args;
    };
    if bindings.is_empty() {
        return args;
    }

    let bindings = bindings
        .iter()
        .filter_map(|(alias, value)| value.as_str().map(|resolved| (alias.clone(), resolved)))
        .collect::<BTreeMap<_, _>>();
    if bindings.is_empty() {
        return args;
    }

    rewrite_result_ref_aliases(args, &bindings)
}

fn rewrite_result_ref_aliases(value: Value, bindings: &BTreeMap<String, &str>) -> Value {
    match value {
        Value::Object(map) => {
            let mut rewritten = serde_json::Map::with_capacity(map.len());
            for (key, child) in map {
                if key == "result_ref_bindings" {
                    rewritten.insert(key, child);
                    continue;
                }

                let next_value = if key == "result_ref" {
                    match child {
                        Value::String(reference) => bindings
                            .get(&reference)
                            .map(|resolved| Value::String((*resolved).to_string()))
                            .unwrap_or(Value::String(reference)),
                        other => rewrite_result_ref_aliases(other, bindings),
                    }
                } else {
                    rewrite_result_ref_aliases(child, bindings)
                };
                rewritten.insert(key, next_value);
            }
            Value::Object(rewritten)
        }
        Value::Array(items) => Value::Array(
            items
                .into_iter()
                .map(|item| rewrite_result_ref_aliases(item, bindings))
                .collect(),
        ),
        other => other,
    }
}

fn dispatch_open_workbook(args: Value) -> ToolResponse {
    let Some(path) = args.get("path").and_then(|value| value.as_str()) else {
        return ToolResponse::error("open_workbook 缺少 path 参数");
    };

    match open_workbook(path) {
        Ok(summary) => build_opened_file_response(&args, summary),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

fn dispatch_list_sheets(args: Value) -> ToolResponse {
    let Some(path) = args.get("path").and_then(|value| value.as_str()) else {
        return ToolResponse::error("list_sheets 缺少 path 参数");
    };

    match list_sheets(path) {
        Ok(summary) => build_opened_file_response(&args, summary),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

fn dispatch_inspect_sheet_range(args: Value) -> ToolResponse {
    let source = match resolve_sheet_source(&args, "inspect_sheet_range") {
        Ok(source) => source,
        Err(response) => return response,
    };
    let sample_rows = args
        .get("sample_rows")
        .and_then(|value| value.as_u64())
        .unwrap_or(5) as usize;

    match inspect_sheet_range(&source.path, &source.sheet_name, sample_rows) {
        Ok(inspection) => ToolResponse::ok(json!(inspection)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

fn dispatch_load_table_region(args: Value) -> ToolResponse {
    let source = match resolve_sheet_source(&args, "load_table_region") {
        Ok(source) => source,
        Err(response) => return response,
    };
    let Some(region) = args.get("range").and_then(|value| value.as_str()) else {
        return ToolResponse::error("load_table_region 缺少 range 参数");
    };
    let header_row_count = args
        .get("header_row_count")
        .and_then(|value| value.as_u64())
        .unwrap_or(1) as usize;

    match load_table_region(&source.path, &source.sheet_name, region, header_row_count) {
        Ok(loaded) => {
            let table_ref = TableRefStore::create_table_ref();
            let persisted = match PersistedTableRef::from_region(
                table_ref.clone(),
                &source.path,
                &source.sheet_name,
                region,
                loaded.handle.columns().to_vec(),
                header_row_count,
            ) {
                Ok(persisted) => persisted,
                Err(error) => return ToolResponse::error(error.to_string()),
            };
            let store = match TableRefStore::workspace_default() {
                Ok(store) => store,
                Err(error) => return ToolResponse::error(error.to_string()),
            };
            if let Err(error) = store.save(&persisted) {
                return ToolResponse::error(error.to_string());
            }
            if let Err(response) = sync_confirmed_table_state(&args, &persisted, "查看区域加载结果")
            {
                return response;
            }

            match preview_table(&loaded.dataframe, 5) {
                Ok(preview) => respond_with_result_dataset(
                    "load_table_region",
                    &args,
                    &loaded,
                    json!({
                        "path": source.path,
                        "sheet": source.sheet_name,
                        "range": region,
                        "header_row_count": header_row_count,
                        "table_ref": persisted.table_ref,
                        "columns": preview.columns,
                        "rows": preview.rows,
                        "row_count": loaded.dataframe.height(),
                    }),
                ),
                Err(error) => ToolResponse::error(error.to_string()),
            }
        }
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

fn dispatch_normalize_table(args: Value) -> ToolResponse {
    let source = match resolve_sheet_source(&args, "normalize_table") {
        Ok(source) => source,
        Err(response) => return response,
    };

    match infer_header_schema(&source.path, &source.sheet_name) {
        Ok(inference) => build_inference_response(&source.sheet_name, inference),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

fn dispatch_apply_header_schema(args: Value) -> ToolResponse {
    let source = match resolve_sheet_source(&args, "apply_header_schema") {
        Ok(source) => source,
        Err(response) => return response,
    };

    match infer_header_schema(&source.path, &source.sheet_name) {
        Ok(inference) => {
            let forced_inference = HeaderInference {
                columns: inference.columns.clone(),
                confidence: ConfidenceLevel::High,
                schema_state: crate::domain::schema::SchemaState::Confirmed,
                header_row_count: inference.header_row_count,
                data_start_row_index: inference.data_start_row_index,
            };

            match load_confirmed_table(&source.path, &source.sheet_name, &forced_inference) {
                Ok(loaded) => {
                    let row_count = loaded.dataframe.height();
                    let table_ref = TableRefStore::create_table_ref();
                    let persisted = match PersistedTableRef::from_confirmed_schema(
                        table_ref.clone(),
                        &source.path,
                        &source.sheet_name,
                        &forced_inference,
                    ) {
                        Ok(persisted) => persisted,
                        Err(error) => return ToolResponse::error(error.to_string()),
                    };
                    let store = match TableRefStore::workspace_default() {
                        Ok(store) => store,
                        Err(error) => return ToolResponse::error(error.to_string()),
                    };
                    if let Err(error) = store.save(&persisted) {
                        return ToolResponse::error(error.to_string());
                    }
                    if let Err(response) =
                        sync_confirmed_table_state(&args, &persisted, "查看确认后的表")
                    {
                        return response;
                    }

                    ToolResponse::ok(json!({
                        "table_id": table_ref,
                        "table_ref": persisted.table_ref,
                        "schema_state": infer_schema_state_label(loaded.handle.schema_state()),
                        "sheet": loaded.handle.sheet_name(),
                        "columns": forced_inference.columns,
                        "row_count": row_count,
                    }))
                }
                Err(error) => ToolResponse::error(error.to_string()),
            }
        }
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

#[derive(Debug, Deserialize, Default)]
struct UpdateSessionStateInput {
    session_id: Option<String>,
    current_workbook: Option<String>,
    current_sheet: Option<String>,
    current_file_ref: Option<String>,
    current_sheet_index: Option<usize>,
    current_stage: Option<SessionStage>,
    schema_status: Option<SchemaStatus>,
    active_table_ref: Option<String>,
    active_handle_ref: Option<String>,
    active_handle_kind: Option<String>,
    last_user_goal: Option<String>,
    selected_columns: Option<Vec<String>>,
}

fn dispatch_get_session_state(args: Value) -> ToolResponse {
    let runtime = match memory_runtime() {
        Ok(runtime) => runtime,
        Err(response) => return response,
    };
    let session_id = session_id_from_args(&args);

    match runtime.get_session_state(&session_id) {
        Ok(state) => ToolResponse::ok(build_session_state_response(&state)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

fn dispatch_update_session_state(args: Value) -> ToolResponse {
    let payload = match serde_json::from_value::<UpdateSessionStateInput>(args.clone()) {
        Ok(payload) => payload,
        Err(error) => {
            return ToolResponse::error(format!("update_session_state 参数解析失败: {error}"));
        }
    };
    let runtime = match memory_runtime() {
        Ok(runtime) => runtime,
        Err(response) => return response,
    };
    let session_id = payload.session_id.unwrap_or_else(|| "default".to_string());
    let patch = SessionStatePatch {
        current_workbook: payload.current_workbook,
        current_sheet: payload.current_sheet,
        current_file_ref: payload.current_file_ref,
        current_sheet_index: payload.current_sheet_index,
        current_stage: payload.current_stage,
        schema_status: payload.schema_status,
        active_table_ref: payload
            .active_table_ref
            .or_else(|| payload.active_handle_ref.clone()),
        active_handle_ref: payload.active_handle_ref,
        active_handle_kind: payload.active_handle_kind,
        last_user_goal: payload.last_user_goal,
        selected_columns: payload.selected_columns,
    };

    match runtime.update_session_state(&session_id, &patch) {
        Ok(state) => {
            let _ = runtime.append_event(
                &session_id,
                &EventLogInput {
                    event_type: "session_state_updated".to_string(),
                    stage: Some(state.current_stage.clone()),
                    tool_name: Some("update_session_state".to_string()),
                    status: "ok".to_string(),
                    message: Some("总入口显式更新会话状态".to_string()),
                },
            );
            ToolResponse::ok(build_session_state_response(&state))
        }
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

fn build_session_state_response(state: &crate::runtime::local_memory::SessionState) -> Value {
    let mut payload = json!(state);
    if let Some(object) = payload.as_object_mut() {
        let active_handle_ref = state
            .active_handle_ref
            .clone()
            .or_else(|| state.active_table_ref.clone());
        let active_handle_kind = state.active_handle_kind.clone().or_else(|| {
            active_handle_ref
                .as_deref()
                .map(classify_handle_kind)
                .map(str::to_string)
        });
        object.insert(
            "active_handle_ref".to_string(),
            active_handle_ref
                .clone()
                .map(Value::String)
                .unwrap_or(Value::Null),
        );
        object.insert(
            "active_handle".to_string(),
            active_handle_ref
                .as_ref()
                .map(|reference| {
                    let kind = active_handle_kind
                        .clone()
                        .unwrap_or_else(|| classify_handle_kind(reference).to_string());
                    json!({
                        "ref": reference,
                        "kind": kind,
                    })
                })
                .unwrap_or(Value::Null),
        );
    }
    payload
}

fn classify_handle_kind(reference: &str) -> &'static str {
    if reference.starts_with("result_") {
        return "result_ref";
    }
    if reference.starts_with("table_") {
        return "table_ref";
    }
    if reference.starts_with("workbook_") {
        return "workbook_ref";
    }
    if reference.starts_with("chart_") {
        return "chart_ref";
    }
    "unknown"
}

fn dispatch_preview_table(args: Value) -> ToolResponse {
    let limit = args
        .get("limit")
        .and_then(|value| value.as_u64())
        .unwrap_or(5) as usize;

    match load_table_for_tool(&args, "preview_table") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => preview_loaded_table(&loaded, limit),
        Err(response) => response,
    }
}

fn dispatch_select_columns(args: Value) -> ToolResponse {
    let Some(columns) = args.get("columns").and_then(|value| value.as_array()) else {
        return ToolResponse::error("select_columns 缺少 columns 参数");
    };
    let requested_columns = columns
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>();

    match load_table_for_tool(&args, "select_columns") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match select_columns(&loaded, &requested_columns) {
            Ok(selected) => respond_with_result_dataset(
                "select_columns",
                &args,
                &selected,
                json!({
                    "columns": selected.handle.columns(),
                    "row_count": selected.dataframe.height(),
                }),
            ),
            Err(error) => ToolResponse::error(error.to_string()),
        },
        Err(response) => response,
    }
}

fn dispatch_filter_rows(args: Value) -> ToolResponse {
    let Some(conditions_value) = args.get("conditions") else {
        return ToolResponse::error("filter_rows 缺少 conditions 参数");
    };
    let conditions = match serde_json::from_value::<Vec<FilterCondition>>(conditions_value.clone())
    {
        Ok(conditions) => conditions,
        Err(error) => {
            return ToolResponse::error(format!("filter_rows 条件解析失败: {error}"));
        }
    };

    match load_table_for_tool(&args, "filter_rows") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match filter_rows(&loaded, &conditions) {
            Ok(filtered) => respond_with_preview_and_result_ref("filter_rows", &args, &filtered, 5),
            Err(error) => ToolResponse::error(error.to_string()),
        },
        Err(response) => response,
    }
}

fn dispatch_cast_column_types(args: Value) -> ToolResponse {
    let Some(casts_value) = args.get("casts") else {
        return ToolResponse::error("cast_column_types 缺少 casts 参数");
    };
    let casts = match serde_json::from_value::<Vec<CastColumnSpec>>(casts_value.clone()) {
        Ok(casts) => casts,
        Err(error) => {
            return ToolResponse::error(format!("cast_column_types 参数解析失败: {error}"));
        }
    };

    match load_table_for_tool(&args, "cast_column_types") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match cast_column_types(&loaded, &casts) {
            Ok(casted) => respond_with_result_dataset(
                "cast_column_types",
                &args,
                &casted,
                json!({
                    "columns": casted.handle.columns(),
                    "column_types": summarize_column_types(&casted.dataframe),
                    "row_count": casted.dataframe.height(),
                }),
            ),
            Err(error) => ToolResponse::error(error.to_string()),
        },
        Err(response) => response,
    }
}

fn dispatch_derive_columns(args: Value) -> ToolResponse {
    let Some(derivations_value) = args.get("derivations") else {
        return ToolResponse::error("derive_columns 缺少 derivations 参数");
    };
    let derivations = match serde_json::from_value::<Vec<DerivationSpec>>(derivations_value.clone())
    {
        Ok(derivations) => derivations,
        Err(error) => {
            return ToolResponse::error(format!("derive_columns 参数解析失败: {error}"));
        }
    };
    let casts = match parse_casts(&args, "casts", "derive_columns") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match load_table_for_tool(&args, "derive_columns") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => match derive_columns(&prepared_loaded, &derivations) {
                Ok(derived) => respond_with_preview_and_result_ref(
                    "derive_columns",
                    &args,
                    &derived,
                    derived.dataframe.height(),
                ),
                Err(error) => ToolResponse::error(error.to_string()),
            },
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

fn dispatch_group_and_aggregate(args: Value) -> ToolResponse {
    let Some(group_by_value) = args.get("group_by").and_then(|value| value.as_array()) else {
        return ToolResponse::error("group_and_aggregate 缺少 group_by 参数");
    };
    let Some(aggregations_value) = args.get("aggregations") else {
        return ToolResponse::error("group_and_aggregate 缺少 aggregations 参数");
    };
    let group_by = group_by_value
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>();
    let aggregations =
        match serde_json::from_value::<Vec<AggregationSpec>>(aggregations_value.clone()) {
            Ok(aggregations) => aggregations,
            Err(error) => {
                return ToolResponse::error(format!("group_and_aggregate 参数解析失败: {error}"));
            }
        };
    let casts = match parse_casts(&args, "casts", "group_and_aggregate") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match load_table_for_tool(&args, "group_and_aggregate") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => {
                match group_and_aggregate(&prepared_loaded, &group_by, &aggregations) {
                    Ok(grouped) => respond_with_preview_and_result_ref(
                        "group_and_aggregate",
                        &args,
                        &grouped,
                        grouped.dataframe.height(),
                    ),
                    Err(error) => ToolResponse::error(error.to_string()),
                }
            }
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

fn dispatch_pivot_table(args: Value) -> ToolResponse {
    let rows = string_array(&args, "rows");
    let columns = string_array(&args, "columns");
    let values = string_array(&args, "values");
    let Some(aggregation_value) = args.get("aggregation") else {
        return ToolResponse::error("pivot_table 缺少 aggregation 参数");
    };
    let aggregation = match serde_json::from_value::<PivotAggregation>(aggregation_value.clone()) {
        Ok(aggregation) => aggregation,
        Err(error) => {
            return ToolResponse::error(format!("pivot_table 参数解析失败: {error}"));
        }
    };
    let casts = match parse_casts(&args, "casts", "pivot_table") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match load_table_for_tool(&args, "pivot_table") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => {
                match pivot_table(&prepared_loaded, &rows, &columns, &values, aggregation) {
                    Ok(pivoted) => {
                        respond_with_preview_and_result_ref("pivot_table", &args, &pivoted, 20)
                    }
                    Err(error) => ToolResponse::error(error.to_string()),
                }
            }
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

fn dispatch_sort_rows(args: Value) -> ToolResponse {
    let Some(sorts_value) = args.get("sorts") else {
        return ToolResponse::error("sort_rows 缺少 sorts 参数");
    };
    let limit = args
        .get("limit")
        .and_then(|value| value.as_u64())
        .unwrap_or(5) as usize;
    let sorts = match serde_json::from_value::<Vec<SortSpec>>(sorts_value.clone()) {
        Ok(sorts) => sorts,
        Err(error) => return ToolResponse::error(format!("sort_rows 参数解析失败: {error}")),
    };
    let casts = match parse_casts(&args, "casts", "sort_rows") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match load_table_for_tool(&args, "sort_rows") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => match sort_rows(&prepared_loaded, &sorts) {
                Ok(sorted) => {
                    respond_with_preview_and_result_ref("sort_rows", &args, &sorted, limit)
                }
                Err(error) => ToolResponse::error(error.to_string()),
            },
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

fn dispatch_top_n(args: Value) -> ToolResponse {
    let Some(sorts_value) = args.get("sorts") else {
        return ToolResponse::error("top_n 缺少 sorts 参数");
    };
    let Some(n) = args.get("n").and_then(|value| value.as_u64()) else {
        return ToolResponse::error("top_n 缺少 n 参数");
    };
    let sorts = match serde_json::from_value::<Vec<SortSpec>>(sorts_value.clone()) {
        Ok(sorts) => sorts,
        Err(error) => return ToolResponse::error(format!("top_n 参数解析失败: {error}")),
    };
    let casts = match parse_casts(&args, "casts", "top_n") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match load_table_for_tool(&args, "top_n") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => match top_n_rows(&prepared_loaded, &sorts, n as usize) {
                Ok(top_rows) => respond_with_preview_and_result_ref(
                    "top_n",
                    &args,
                    &top_rows,
                    top_rows.dataframe.height(),
                ),
                Err(error) => ToolResponse::error(error.to_string()),
            },
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

#[derive(Debug, Deserialize)]
struct ComposeWorkbookWorksheetArg {
    sheet_name: String,
    source: NestedTableSource,
}

#[derive(Debug, Deserialize)]
struct ReportDeliverySectionArg {
    #[serde(default)]
    sheet_name: Option<String>,
    #[serde(default)]
    format: Option<ExportFormatOptions>,
    source: NestedTableSource,
}

#[derive(Debug, Deserialize)]
struct ReportDeliveryChartArg {
    #[serde(default)]
    chart_ref: Option<String>,
    #[serde(default)]
    chart_type: Option<ReportDeliveryChartType>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    category_column: Option<String>,
    #[serde(default)]
    value_column: Option<String>,
    #[serde(default)]
    series: Vec<ReportDeliveryChartSeriesArg>,
    #[serde(default)]
    show_legend: bool,
    #[serde(default)]
    legend_position: Option<ReportDeliveryLegendPosition>,
    #[serde(default)]
    chart_style: Option<u8>,
    #[serde(default)]
    x_axis_name: Option<String>,
    #[serde(default)]
    y_axis_name: Option<String>,
    #[serde(default)]
    anchor_row: Option<u32>,
    #[serde(default)]
    anchor_col: Option<u16>,
}

#[derive(Debug, Deserialize)]
struct ReportDeliveryChartSeriesArg {
    value_column: String,
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ReportDeliveryArgs {
    report_name: Option<String>,
    #[serde(default)]
    report_subtitle: Option<String>,
    summary: ReportDeliverySectionArg,
    analysis: ReportDeliverySectionArg,
    #[serde(default = "default_true")]
    include_chart_sheet: bool,
    #[serde(default)]
    chart_sheet_name: Option<String>,
    #[serde(default)]
    charts: Vec<ReportDeliveryChartArg>,
}

#[derive(Debug, Deserialize)]
struct BuildChartSeriesArg {
    value_column: String,
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BuildChartArgs {
    source: NestedTableSource,
    chart_type: PersistedChartType,
    #[serde(default)]
    title: Option<String>,
    category_column: String,
    #[serde(default)]
    value_column: Option<String>,
    #[serde(default)]
    series: Vec<BuildChartSeriesArg>,
    #[serde(default)]
    x_axis_name: Option<String>,
    #[serde(default)]
    y_axis_name: Option<String>,
    #[serde(default)]
    show_legend: bool,
    #[serde(default)]
    width: Option<u32>,
    #[serde(default)]
    height: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct ExportChartImageArgs {
    chart_ref: String,
    output_path: String,
}

fn default_true() -> bool {
    true
}

fn dispatch_compose_workbook(args: Value) -> ToolResponse {
    let Some(worksheets_value) = args.get("worksheets") else {
        return ToolResponse::error("compose_workbook 缺少 worksheets 参数");
    };
    let worksheet_args = match serde_json::from_value::<Vec<ComposeWorkbookWorksheetArg>>(
        worksheets_value.clone(),
    ) {
        Ok(worksheet_args) => worksheet_args,
        Err(error) => {
            return ToolResponse::error(format!(
                "compose_workbook 的 worksheets 参数解析失败: {error}"
            ));
        }
    };

    let mut sheet_inputs = Vec::<WorkbookSheetInput>::with_capacity(worksheet_args.len());
    for worksheet_arg in worksheet_args {
        let loaded = match load_nested_table_source_from_parsed(
            &worksheet_arg.source,
            "compose_workbook",
            &worksheet_arg.sheet_name,
        ) {
            Ok(OperationLoad::NeedsConfirmation(response)) => return response,
            Ok(OperationLoad::Loaded(loaded)) => loaded,
            Err(response) => return response,
        };
        sheet_inputs.push(WorkbookSheetInput {
            sheet_name: worksheet_arg.sheet_name,
            source_refs: source_refs_from_nested_source(&worksheet_arg.source),
            dataframe: loaded.dataframe,
            title: None,
            subtitle: None,
            data_start_row: 0,
        });
    }

    let workbook_ref = WorkbookDraftStore::create_workbook_ref();
    let draft = match PersistedWorkbookDraft::from_sheet_inputs(&workbook_ref, sheet_inputs) {
        Ok(draft) => draft,
        Err(error) => return ToolResponse::error(error.to_string()),
    };
    let store = match WorkbookDraftStore::workspace_default() {
        Ok(store) => store,
        Err(error) => return ToolResponse::error(error.to_string()),
    };
    if let Err(error) = store.save(&draft) {
        return ToolResponse::error(error.to_string());
    }
    if let Err(response) =
        sync_output_handle_state(&args, &workbook_ref, "workbook_ref", "compose_workbook")
    {
        return response;
    }

    ToolResponse::ok(json!({
        "workbook_ref": workbook_ref,
        "sheet_count": draft.worksheets.len(),
        "sheet_names": draft
            .worksheets
            .iter()
            .map(|worksheet| worksheet.sheet_name.clone())
            .collect::<Vec<_>>(),
    }))
}

fn dispatch_report_delivery(args: Value) -> ToolResponse {
    let delivery_args = match serde_json::from_value::<ReportDeliveryArgs>(args.clone()) {
        Ok(delivery_args) => delivery_args,
        Err(error) => {
            return ToolResponse::error(format!("report_delivery 参数解析失败: {error}"));
        }
    };

    let summary_loaded = match load_nested_table_source_from_parsed(
        &delivery_args.summary.source,
        "report_delivery",
        "summary",
    ) {
        Ok(OperationLoad::NeedsConfirmation(response)) => return response,
        Ok(OperationLoad::Loaded(loaded)) => loaded,
        Err(response) => return response,
    };
    let summary_loaded = match apply_report_delivery_section_format(
        summary_loaded,
        delivery_args.summary.format.as_ref(),
    ) {
        Ok(loaded) => loaded,
        Err(response) => return response,
    };
    let analysis_loaded = match load_nested_table_source_from_parsed(
        &delivery_args.analysis.source,
        "report_delivery",
        "analysis",
    ) {
        Ok(OperationLoad::NeedsConfirmation(response)) => return response,
        Ok(OperationLoad::Loaded(loaded)) => loaded,
        Err(response) => return response,
    };
    let analysis_loaded = match apply_report_delivery_section_format(
        analysis_loaded,
        delivery_args.analysis.format.as_ref(),
    ) {
        Ok(loaded) => loaded,
        Err(response) => return response,
    };
    let summary_source_refs = source_refs_from_nested_source(&delivery_args.summary.source);
    let analysis_source_refs = source_refs_from_nested_source(&delivery_args.analysis.source);
    let charts = match resolve_report_delivery_charts(
        delivery_args.charts,
        &analysis_loaded.dataframe,
        &analysis_source_refs,
    ) {
        Ok(charts) => charts,
        Err(response) => return response,
    };

    let workbook_ref = WorkbookDraftStore::create_workbook_ref();
    let report_name = delivery_args
        .report_name
        .unwrap_or_else(|| "标准分析汇报".to_string());
    let draft = match build_report_delivery_draft(
        &workbook_ref,
        ReportDeliveryRequest {
            report_name: report_name.clone(),
            report_subtitle: delivery_args.report_subtitle,
            summary: ReportDeliverySection {
                sheet_name: delivery_args
                    .summary
                    .sheet_name
                    .unwrap_or_else(|| "摘要页".to_string()),
                source_refs: summary_source_refs,
                dataframe: summary_loaded.dataframe,
            },
            analysis: ReportDeliverySection {
                sheet_name: delivery_args
                    .analysis
                    .sheet_name
                    .unwrap_or_else(|| "分析结果页".to_string()),
                source_refs: analysis_source_refs,
                dataframe: analysis_loaded.dataframe,
            },
            include_chart_sheet: delivery_args.include_chart_sheet,
            chart_sheet_name: delivery_args
                .chart_sheet_name
                .unwrap_or_else(|| "图表页".to_string()),
            charts,
        },
    ) {
        Ok(draft) => draft,
        Err(error) => return ToolResponse::error(error.to_string()),
    };

    let store = match WorkbookDraftStore::workspace_default() {
        Ok(store) => store,
        Err(error) => return ToolResponse::error(error.to_string()),
    };
    if let Err(error) = store.save(&draft) {
        return ToolResponse::error(error.to_string());
    }
    if let Err(response) =
        sync_output_handle_state(&args, &workbook_ref, "workbook_ref", "report_delivery")
    {
        return response;
    }

    ToolResponse::ok(json!({
        "workbook_ref": workbook_ref,
        "report_name": report_name,
        "template": "standard_report_v2",
        "sheet_count": draft.worksheets.len(),
        "chart_count": draft.charts.len(),
        "sheet_names": draft
            .worksheets
            .iter()
            .map(|worksheet| worksheet.sheet_name.clone())
            .collect::<Vec<_>>(),
    }))
}

fn apply_report_delivery_section_format(
    loaded: LoadedTable,
    format: Option<&ExportFormatOptions>,
) -> Result<LoadedTable, ToolResponse> {
    let Some(format) = format else {
        return Ok(loaded);
    };
    format_table_for_export(&loaded, format).map_err(|error| ToolResponse::error(error.to_string()))
}

fn resolve_report_delivery_charts(
    chart_args: Vec<ReportDeliveryChartArg>,
    analysis_dataframe: &polars::prelude::DataFrame,
    analysis_source_refs: &[String],
) -> Result<Vec<ReportDeliveryChart>, ToolResponse> {
    chart_args
        .into_iter()
        .map(|chart| resolve_report_delivery_chart(chart, analysis_dataframe, analysis_source_refs))
        .collect()
}

fn resolve_report_delivery_chart(
    chart: ReportDeliveryChartArg,
    analysis_dataframe: &polars::prelude::DataFrame,
    analysis_source_refs: &[String],
) -> Result<ReportDeliveryChart, ToolResponse> {
    if let Some(chart_ref) = chart.chart_ref.as_ref() {
        let store = ChartDraftStore::workspace_default()
            .map_err(|error| ToolResponse::error(error.to_string()))?;
        let draft = store
            .load(chart_ref)
            .map_err(|error| ToolResponse::error(error.to_string()))?;
        if !chart_ref_matches_analysis(&draft, analysis_dataframe, analysis_source_refs) {
            return Err(ToolResponse::error(
                "report_delivery 的 chart_ref 与 analysis 数据不一致",
            ));
        }
        return Ok(chart_ref_to_report_delivery_chart(&draft));
    }

    let chart_type = chart
        .chart_type
        .ok_or_else(|| ToolResponse::error("report_delivery 的 chart_type 不能为空"))?;
    Ok(ReportDeliveryChart {
        chart_ref: None,
        source_refs: vec![],
        chart_type,
        title: chart.title,
        category_column: chart.category_column.unwrap_or_default(),
        value_column: chart.value_column.unwrap_or_default(),
        series: chart
            .series
            .into_iter()
            .map(|series| ReportDeliveryChartSeries {
                value_column: series.value_column,
                name: series.name,
            })
            .collect(),
        show_legend: chart.show_legend,
        legend_position: chart.legend_position,
        chart_style: chart.chart_style,
        x_axis_name: chart.x_axis_name,
        y_axis_name: chart.y_axis_name,
        anchor_row: chart.anchor_row,
        anchor_col: chart.anchor_col,
    })
}

fn chart_ref_matches_analysis(
    draft: &PersistedChartDraft,
    analysis_dataframe: &polars::prelude::DataFrame,
    analysis_source_refs: &[String],
) -> bool {
    if analysis_dataframe.column(&draft.category_column).is_err() {
        return false;
    }
    if draft
        .series
        .iter()
        .any(|item| analysis_dataframe.column(&item.value_column).is_err())
    {
        return false;
    }
    if draft
        .source_refs
        .iter()
        .any(|source_ref| analysis_source_refs.iter().any(|item| item == source_ref))
    {
        return true;
    }
    draft.dataset.row_count == analysis_dataframe.height()
}
fn dispatch_build_chart(args: Value) -> ToolResponse {
    let chart_args = match serde_json::from_value::<BuildChartArgs>(args.clone()) {
        Ok(chart_args) => chart_args,
        Err(error) => {
            return ToolResponse::error(format!("build_chart 参数解析失败: {error}"));
        }
    };

    let loaded =
        match load_nested_table_source_from_parsed(&chart_args.source, "build_chart", "source") {
            Ok(OperationLoad::NeedsConfirmation(response)) => return response,
            Ok(OperationLoad::Loaded(loaded)) => loaded,
            Err(response) => return response,
        };

    let mut series = chart_args
        .series
        .into_iter()
        .map(|item| PersistedChartSeriesSpec {
            value_column: item.value_column,
            name: item.name,
        })
        .collect::<Vec<_>>();
    if series.is_empty() {
        if let Some(value_column) = chart_args.value_column.as_ref() {
            if !value_column.trim().is_empty() {
                series.push(PersistedChartSeriesSpec {
                    value_column: value_column.clone(),
                    name: None,
                });
            }
        }
    }
    if series.is_empty() {
        return ToolResponse::error("build_chart 至少需要一个数值系列");
    }

    let chart_ref = ChartDraftStore::create_chart_ref();
    let draft = match PersistedChartDraft::from_dataframe_with_layout(
        &chart_ref,
        "build_chart",
        source_refs_from_nested_source(&chart_args.source),
        &loaded.dataframe,
        chart_args.chart_type.clone(),
        chart_args.title.clone(),
        &chart_args.category_column,
        chart_args.x_axis_name.clone(),
        chart_args.y_axis_name.clone(),
        chart_args.show_legend,
        chart_args.width.unwrap_or(900),
        chart_args.height.unwrap_or(520),
        series.clone(),
    ) {
        Ok(draft) => draft,
        Err(error) => return ToolResponse::error(error.to_string()),
    };

    let store = match ChartDraftStore::workspace_default() {
        Ok(store) => store,
        Err(error) => return ToolResponse::error(error.to_string()),
    };
    if let Err(error) = store.save(&draft) {
        return ToolResponse::error(error.to_string());
    }
    if let Err(response) = sync_output_handle_state(&args, &chart_ref, "chart_ref", "build_chart") {
        return response;
    }

    ToolResponse::ok(json!({
        "chart_ref": chart_ref,
        "chart_type": serde_json::to_value(&chart_args.chart_type).unwrap_or_else(|_| Value::String("unknown".to_string())),
        "title": chart_args.title,
        "category_column": chart_args.category_column,
        "series_count": series.len(),
        "row_count": loaded.dataframe.height(),
    }))
}
fn dispatch_export_chart_image(args: Value) -> ToolResponse {
    let export_args = match serde_json::from_value::<ExportChartImageArgs>(args.clone()) {
        Ok(export_args) => export_args,
        Err(error) => {
            return ToolResponse::error(format!("export_chart_image 参数解析失败: {error}"));
        }
    };

    if export_args.chart_ref.trim().is_empty() {
        return ToolResponse::error("export_chart_image 缺少 chart_ref 参数");
    }
    if export_args.output_path.trim().is_empty() {
        return ToolResponse::error("export_chart_image 缺少 output_path 参数");
    }
    if !export_args
        .output_path
        .to_ascii_lowercase()
        .ends_with(".svg")
    {
        return ToolResponse::error("export_chart_image 目前只支持导出 svg");
    }

    let store = match ChartDraftStore::workspace_default() {
        Ok(store) => store,
        Err(error) => return ToolResponse::error(error.to_string()),
    };
    let draft = match store.load(&export_args.chart_ref) {
        Ok(draft) => draft,
        Err(error) => return ToolResponse::error(error.to_string()),
    };
    let svg = match render_chart_svg(&draft) {
        Ok(svg) => svg,
        Err(error) => return ToolResponse::error(error.to_string()),
    };

    if let Some(parent) = Path::new(&export_args.output_path).parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            return ToolResponse::error(format!("无法创建图表导出目录: {error}"));
        }
    }
    if let Err(error) = fs::write(&export_args.output_path, svg.as_bytes()) {
        return ToolResponse::error(format!("无法写出图表 SVG: {error}"));
    }
    if let Err(response) = sync_output_handle_state(
        &args,
        &export_args.chart_ref,
        "chart_ref",
        "export_chart_image",
    ) {
        return response;
    }

    ToolResponse::ok(json!({
        "chart_ref": export_args.chart_ref,
        "output_path": export_args.output_path,
        "format": "svg",
    }))
}

fn dispatch_export_csv(args: Value) -> ToolResponse {
    let Some(output_path) = args.get("output_path").and_then(|value| value.as_str()) else {
        return ToolResponse::error("export_csv 缺少 output_path 参数");
    };

    match load_table_for_tool(&args, "export_csv") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match export_csv(&loaded, output_path) {
            Ok(()) => ToolResponse::ok(json!({
                "output_path": output_path,
                "row_count": loaded.dataframe.height(),
                "column_count": loaded.dataframe.width(),
                "format": "csv",
            })),
            Err(error) => ToolResponse::error(error.to_string()),
        },
        Err(response) => response,
    }
}

fn dispatch_export_excel(args: Value) -> ToolResponse {
    let Some(output_path) = args.get("output_path").and_then(|value| value.as_str()) else {
        return ToolResponse::error("export_excel 缺少 output_path 参数");
    };
    let sheet_name = args
        .get("sheet_name")
        .and_then(|value| value.as_str())
        .unwrap_or("Report");

    match load_table_for_tool(&args, "export_excel") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match export_excel(&loaded, output_path, sheet_name) {
            Ok(()) => ToolResponse::ok(json!({
                "output_path": output_path,
                "sheet_name": sheet_name,
                "row_count": loaded.dataframe.height(),
                "column_count": loaded.dataframe.width(),
                "format": "xlsx",
            })),
            Err(error) => ToolResponse::error(error.to_string()),
        },
        Err(response) => response,
    }
}

fn dispatch_export_excel_workbook(args: Value) -> ToolResponse {
    let Some(workbook_ref) = args.get("workbook_ref").and_then(|value| value.as_str()) else {
        return ToolResponse::error("export_excel_workbook 缺少 workbook_ref 参数");
    };
    let Some(output_path) = args.get("output_path").and_then(|value| value.as_str()) else {
        return ToolResponse::error("export_excel_workbook 缺少 output_path 参数");
    };

    let store = match WorkbookDraftStore::workspace_default() {
        Ok(store) => store,
        Err(error) => return ToolResponse::error(error.to_string()),
    };
    let draft = match store.load(workbook_ref) {
        Ok(draft) => draft,
        Err(error) => return ToolResponse::error(error.to_string()),
    };

    match export_excel_workbook(&draft, output_path) {
        Ok(()) => ToolResponse::ok(json!({
            "workbook_ref": workbook_ref,
            "output_path": output_path,
            "sheet_count": draft.worksheets.len(),
            "sheet_names": draft
                .worksheets
                .iter()
                .map(|worksheet| worksheet.sheet_name.clone())
                .collect::<Vec<_>>(),
            "format": "xlsx",
        })),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

fn dispatch_join_tables(args: Value) -> ToolResponse {
    let Some(left_value) = args.get("left") else {
        return ToolResponse::error("join_tables 缺少 left 参数");
    };
    let Some(right_value) = args.get("right") else {
        return ToolResponse::error("join_tables 缺少 right 参数");
    };
    let Some(left_on) = args.get("left_on").and_then(|value| value.as_str()) else {
        return ToolResponse::error("join_tables 缺少 left_on 参数");
    };
    let Some(right_on) = args.get("right_on").and_then(|value| value.as_str()) else {
        return ToolResponse::error("join_tables 缺少 right_on 参数");
    };
    let limit = args
        .get("limit")
        .and_then(|value| value.as_u64())
        .unwrap_or(5) as usize;

    let keep_mode = match args.get("keep_mode") {
        Some(mode_value) => match serde_json::from_value::<JoinKeepMode>(mode_value.clone()) {
            Ok(mode) => mode,
            Err(error) => {
                return ToolResponse::error(format!(
                    "join_tables 的 keep_mode 参数解析失败: {error}"
                ));
            }
        },
        None => JoinKeepMode::MatchedOnly,
    };
    let left_casts = match parse_casts(&args, "left_casts", "join_tables") {
        Ok(casts) => casts,
        Err(response) => return response,
    };
    let right_casts = match parse_casts(&args, "right_casts", "join_tables") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match load_nested_table_source(left_value, "join_tables", "left") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(left_loaded)) => {
            match load_nested_table_source(right_value, "join_tables", "right") {
                Ok(OperationLoad::NeedsConfirmation(response)) => response,
                Ok(OperationLoad::Loaded(right_loaded)) => {
                    let prepared_left = apply_optional_casts(left_loaded, &left_casts);
                    let prepared_right = apply_optional_casts(right_loaded, &right_casts);

                    match (prepared_left, prepared_right) {
                        (Ok(prepared_left), Ok(prepared_right)) => {
                            match join_tables(
                                &prepared_left,
                                &prepared_right,
                                left_on,
                                right_on,
                                keep_mode,
                            ) {
                                Ok(joined) => respond_with_preview_and_result_ref(
                                    "join_tables",
                                    &args,
                                    &joined,
                                    limit,
                                ),
                                Err(error) => ToolResponse::error(error.to_string()),
                            }
                        }
                        (Err(error), _) | (_, Err(error)) => ToolResponse::error(error),
                    }
                }
                Err(response) => response,
            }
        }
        Err(response) => response,
    }
}

fn dispatch_join_preflight(args: Value) -> ToolResponse {
    let Some(left_value) = args.get("left") else {
        return ToolResponse::error("join_preflight 缺少 left 参数");
    };
    let Some(right_value) = args.get("right") else {
        return ToolResponse::error("join_preflight 缺少 right 参数");
    };
    let Some(left_on) = args.get("left_on").and_then(|value| value.as_str()) else {
        return ToolResponse::error("join_preflight 缺少 left_on 参数");
    };
    let Some(right_on) = args.get("right_on").and_then(|value| value.as_str()) else {
        return ToolResponse::error("join_preflight 缺少 right_on 参数");
    };
    let limit = args
        .get("limit")
        .and_then(|value| value.as_u64())
        .unwrap_or(5) as usize;
    let confirm_join = args
        .get("confirm_join")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);

    let keep_mode = match args.get("keep_mode") {
        Some(mode_value) => match serde_json::from_value::<JoinKeepMode>(mode_value.clone()) {
            Ok(mode) => mode,
            Err(error) => {
                return ToolResponse::error(format!(
                    "join_preflight 的 keep_mode 参数解析失败: {error}"
                ));
            }
        },
        None => JoinKeepMode::MatchedOnly,
    };
    let left_casts = match parse_casts(&args, "left_casts", "join_preflight") {
        Ok(casts) => casts,
        Err(response) => return response,
    };
    let right_casts = match parse_casts(&args, "right_casts", "join_preflight") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match load_nested_table_source(left_value, "join_preflight", "left") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(left_loaded)) => {
            match load_nested_table_source(right_value, "join_preflight", "right") {
                Ok(OperationLoad::NeedsConfirmation(response)) => response,
                Ok(OperationLoad::Loaded(right_loaded)) => {
                    let prepared_left = apply_optional_casts(left_loaded, &left_casts);
                    let prepared_right = apply_optional_casts(right_loaded, &right_casts);

                    match (prepared_left, prepared_right) {
                        (Ok(prepared_left), Ok(prepared_right)) => match join_preflight(
                            &prepared_left,
                            &prepared_right,
                            left_on,
                            right_on,
                            keep_mode.clone(),
                            limit,
                        ) {
                            Ok(result) => {
                                let mut payload = json!(result);
                                let suggested_join_tool_call =
                                    build_suggested_join_tool_call_for_preflight(
                                        &args,
                                        keep_mode.clone(),
                                    );
                                payload["suggested_join_tool_call"] =
                                    suggested_join_tool_call.clone();
                                if confirm_join {
                                    payload["confirmed_join_tool_call"] = suggested_join_tool_call;
                                    payload["recommended_next_step"] = json!(
                                        "已确认预检结果，可直接执行 confirmed_join_tool_call。"
                                    );
                                }
                                ToolResponse::ok(payload)
                            }
                            Err(error) => ToolResponse::error(error.to_string()),
                        },
                        (Err(error), _) | (_, Err(error)) => ToolResponse::error(error),
                    }
                }
                Err(response) => response,
            }
        }
        Err(response) => response,
    }
}

fn build_suggested_join_tool_call_for_preflight(args: &Value, keep_mode: JoinKeepMode) -> Value {
    let mut join_args = serde_json::Map::new();
    if let Some(left) = args.get("left").cloned() {
        join_args.insert("left".to_string(), left);
    }
    if let Some(right) = args.get("right").cloned() {
        join_args.insert("right".to_string(), right);
    }
    if let Some(left_on) = args.get("left_on").cloned() {
        join_args.insert("left_on".to_string(), left_on);
    }
    if let Some(right_on) = args.get("right_on").cloned() {
        join_args.insert("right_on".to_string(), right_on);
    }
    join_args.insert("keep_mode".to_string(), json!(keep_mode));
    if let Some(limit) = args.get("limit").cloned() {
        join_args.insert("limit".to_string(), limit);
    }
    if let Some(left_casts) = args.get("left_casts").cloned() {
        join_args.insert("left_casts".to_string(), left_casts);
    }
    if let Some(right_casts) = args.get("right_casts").cloned() {
        join_args.insert("right_casts".to_string(), right_casts);
    }
    if let Some(bindings) = args.get("result_ref_bindings").cloned() {
        join_args.insert("result_ref_bindings".to_string(), bindings);
    }

    json!({
        "tool": "join_tables",
        "args": Value::Object(join_args),
    })
}

fn dispatch_normalize_text_columns(args: Value) -> ToolResponse {
    let Some(rules_value) = args.get("rules") else {
        return ToolResponse::error("normalize_text_columns 缺少 rules 参数");
    };
    let rules = match serde_json::from_value::<Vec<NormalizeTextRule>>(rules_value.clone()) {
        Ok(rules) => rules,
        Err(error) => {
            return ToolResponse::error(format!("normalize_text_columns 参数解析失败: {error}"));
        }
    };

    match load_table_for_tool(&args, "normalize_text_columns") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match normalize_text_columns(&loaded, &rules) {
            Ok(normalized) => {
                respond_with_preview_and_result_ref("normalize_text_columns", &args, &normalized, 5)
            }
            Err(error) => ToolResponse::error(error.to_string()),
        },
        Err(response) => response,
    }
}

fn dispatch_parse_datetime_columns(args: Value) -> ToolResponse {
    let Some(rules_value) = args.get("rules") else {
        return ToolResponse::error("parse_datetime_columns 缺少 rules 参数");
    };
    let rules = match serde_json::from_value::<Vec<ParseDateTimeRule>>(rules_value.clone()) {
        Ok(rules) => rules,
        Err(error) => {
            return ToolResponse::error(format!("parse_datetime_columns 参数解析失败: {error}"));
        }
    };

    match load_table_for_tool(&args, "parse_datetime_columns") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match parse_datetime_columns(&loaded, &rules) {
            Ok(parsed) => {
                respond_with_preview_and_result_ref("parse_datetime_columns", &args, &parsed, 20)
            }
            Err(error) => ToolResponse::error(error.to_string()),
        },
        Err(response) => response,
    }
}

fn dispatch_lookup_values(args: Value) -> ToolResponse {
    let Some(base_value) = args.get("base") else {
        return ToolResponse::error("lookup_values 缺少 base 参数");
    };
    let Some(lookup_value) = args.get("lookup") else {
        return ToolResponse::error("lookup_values 缺少 lookup 参数");
    };
    let base_keys = match parse_lookup_key_args(&args, "base_on", "base_keys", "lookup_values") {
        Ok(keys) => keys,
        Err(response) => return response,
    };
    let lookup_keys =
        match parse_lookup_key_args(&args, "lookup_on", "lookup_keys", "lookup_values") {
            Ok(keys) => keys,
            Err(response) => return response,
        };
    let Some(selects_value) = args.get("selects") else {
        return ToolResponse::error("lookup_values 缺少 selects 参数");
    };
    let selects = match serde_json::from_value::<Vec<LookupSelect>>(selects_value.clone()) {
        Ok(selects) => selects,
        Err(error) => {
            return ToolResponse::error(format!("lookup_values 参数解析失败: {error}"));
        }
    };

    match load_nested_table_source(base_value, "lookup_values", "base") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(base_loaded)) => {
            match load_nested_table_source(lookup_value, "lookup_values", "lookup") {
                Ok(OperationLoad::NeedsConfirmation(response)) => response,
                Ok(OperationLoad::Loaded(lookup_loaded)) => {
                    let base_key_refs = base_keys.iter().map(String::as_str).collect::<Vec<_>>();
                    let lookup_key_refs =
                        lookup_keys.iter().map(String::as_str).collect::<Vec<_>>();
                    match lookup_values_by_keys(
                        &base_loaded,
                        &lookup_loaded,
                        &base_key_refs,
                        &lookup_key_refs,
                        &selects,
                    ) {
                        Ok(looked_up) => respond_with_preview_and_result_ref(
                            "lookup_values",
                            &args,
                            &looked_up,
                            20,
                        ),
                        Err(error) => ToolResponse::error(error.to_string()),
                    }
                }
                Err(response) => response,
            }
        }
        Err(response) => response,
    }
}

fn dispatch_window_calculation(args: Value) -> ToolResponse {
    let Some(order_by_value) = args.get("order_by") else {
        return ToolResponse::error("window_calculation 缺少 order_by 参数");
    };
    let Some(calculations_value) = args.get("calculations") else {
        return ToolResponse::error("window_calculation 缺少 calculations 参数");
    };
    let partition_by = string_array(&args, "partition_by");
    let order_by = match serde_json::from_value::<Vec<WindowOrderSpec>>(order_by_value.clone()) {
        Ok(order_by) => order_by,
        Err(error) => {
            return ToolResponse::error(format!("window_calculation 参数解析失败: {error}"));
        }
    };
    let calculations =
        match serde_json::from_value::<Vec<WindowCalculation>>(calculations_value.clone()) {
            Ok(calculations) => calculations,
            Err(error) => {
                return ToolResponse::error(format!("window_calculation 参数解析失败: {error}"));
            }
        };
    let casts = match parse_casts(&args, "casts", "window_calculation") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match load_table_for_tool(&args, "window_calculation") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => {
                match window_calculation(&prepared_loaded, &partition_by, &order_by, &calculations)
                {
                    Ok(calculated) => respond_with_preview_and_result_ref(
                        "window_calculation",
                        &args,
                        &calculated,
                        20,
                    ),
                    Err(error) => ToolResponse::error(error.to_string()),
                }
            }
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

fn dispatch_rename_columns(args: Value) -> ToolResponse {
    let Some(mappings_value) = args.get("mappings") else {
        return ToolResponse::error("rename_columns 缺少 mappings 参数");
    };
    let mappings = match serde_json::from_value::<Vec<RenameColumnMapping>>(mappings_value.clone())
    {
        Ok(mappings) => mappings,
        Err(error) => {
            return ToolResponse::error(format!("rename_columns 参数解析失败: {error}"));
        }
    };

    match load_table_for_tool(&args, "rename_columns") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match rename_columns(&loaded, &mappings) {
            Ok(renamed) => {
                respond_with_preview_and_result_ref("rename_columns", &args, &renamed, 5)
            }
            Err(error) => ToolResponse::error(error.to_string()),
        },
        Err(response) => response,
    }
}

fn dispatch_fill_missing_values(args: Value) -> ToolResponse {
    let Some(rules_value) = args.get("rules") else {
        return ToolResponse::error("fill_missing_values 缺少 rules 参数");
    };
    let rules = match serde_json::from_value::<Vec<FillMissingRule>>(rules_value.clone()) {
        Ok(rules) => rules,
        Err(error) => {
            return ToolResponse::error(format!("fill_missing_values 参数解析失败: {error}"));
        }
    };

    match load_table_for_tool(&args, "fill_missing_values") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match fill_missing_values(&loaded, &rules) {
            Ok(filled) => {
                respond_with_preview_and_result_ref("fill_missing_values", &args, &filled, 20)
            }
            Err(error) => ToolResponse::error(error.to_string()),
        },
        Err(response) => response,
    }
}

fn dispatch_distinct_rows(args: Value) -> ToolResponse {
    let subset = string_array(&args, "subset");
    let keep = match args.get("keep") {
        Some(value) => match serde_json::from_value::<DistinctKeep>(value.clone()) {
            Ok(keep) => keep,
            Err(error) => {
                return ToolResponse::error(format!("distinct_rows 参数解析失败: {error}"));
            }
        },
        None => DistinctKeep::First,
    };

    match load_table_for_tool(&args, "distinct_rows") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match distinct_rows(&loaded, &subset, keep) {
            Ok(distincted) => {
                respond_with_preview_and_result_ref("distinct_rows", &args, &distincted, 20)
            }
            Err(error) => ToolResponse::error(error.to_string()),
        },
        Err(response) => response,
    }
}

fn dispatch_deduplicate_by_key(args: Value) -> ToolResponse {
    let keys = string_array(&args, "keys");
    let order_by = match args.get("order_by") {
        Some(value) => match serde_json::from_value::<Vec<OrderSpec>>(value.clone()) {
            Ok(order_by) => order_by,
            Err(error) => {
                return ToolResponse::error(format!("deduplicate_by_key 参数解析失败: {error}"));
            }
        },
        None => Vec::new(),
    };
    let keep = match args.get("keep") {
        Some(value) => match serde_json::from_value::<DeduplicateKeep>(value.clone()) {
            Ok(keep) => keep,
            Err(error) => {
                return ToolResponse::error(format!("deduplicate_by_key 参数解析失败: {error}"));
            }
        },
        None => DeduplicateKeep::First,
    };

    match load_table_for_tool(&args, "deduplicate_by_key") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => {
            match deduplicate_by_key(&loaded, &keys, &order_by, keep) {
                Ok(deduplicated) => respond_with_preview_and_result_ref(
                    "deduplicate_by_key",
                    &args,
                    &deduplicated,
                    20,
                ),
                Err(error) => ToolResponse::error(error.to_string()),
            }
        }
        Err(response) => response,
    }
}

fn dispatch_format_table_for_export(args: Value) -> ToolResponse {
    let options = match serde_json::from_value::<ExportFormatOptions>(args.clone()) {
        Ok(options) => options,
        Err(error) => {
            return ToolResponse::error(format!("format_table_for_export 参数解析失败: {error}"));
        }
    };

    match load_table_for_tool(&args, "format_table_for_export") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match format_table_for_export(&loaded, &options) {
            Ok(formatted) => respond_with_preview_and_result_ref(
                "format_table_for_export",
                &args,
                &formatted,
                20,
            ),
            Err(error) => ToolResponse::error(error.to_string()),
        },
        Err(response) => response,
    }
}

fn dispatch_fill_missing_from_lookup(args: Value) -> ToolResponse {
    let Some(base_value) = args.get("base") else {
        return ToolResponse::error("fill_missing_from_lookup 缺少 base 参数");
    };
    let Some(lookup_value) = args.get("lookup") else {
        return ToolResponse::error("fill_missing_from_lookup 缺少 lookup 参数");
    };
    let base_keys =
        match parse_lookup_key_args(&args, "base_on", "base_keys", "fill_missing_from_lookup") {
            Ok(keys) => keys,
            Err(response) => return response,
        };
    let lookup_keys = match parse_lookup_key_args(
        &args,
        "lookup_on",
        "lookup_keys",
        "fill_missing_from_lookup",
    ) {
        Ok(keys) => keys,
        Err(response) => return response,
    };
    let Some(fills_value) = args.get("fills") else {
        return ToolResponse::error("fill_missing_from_lookup 缺少 fills 参数");
    };
    let fills = match serde_json::from_value::<Vec<FillLookupRule>>(fills_value.clone()) {
        Ok(fills) => fills,
        Err(error) => {
            return ToolResponse::error(format!("fill_missing_from_lookup 参数解析失败: {error}"));
        }
    };

    match load_nested_table_source(base_value, "fill_missing_from_lookup", "base") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(base_loaded)) => {
            match load_nested_table_source(lookup_value, "fill_missing_from_lookup", "lookup") {
                Ok(OperationLoad::NeedsConfirmation(response)) => response,
                Ok(OperationLoad::Loaded(lookup_loaded)) => {
                    let base_key_refs = base_keys.iter().map(String::as_str).collect::<Vec<_>>();
                    let lookup_key_refs =
                        lookup_keys.iter().map(String::as_str).collect::<Vec<_>>();
                    match fill_missing_from_lookup_by_keys(
                        &base_loaded,
                        &lookup_loaded,
                        &base_key_refs,
                        &lookup_key_refs,
                        &fills,
                    ) {
                        Ok(filled) => respond_with_preview_and_result_ref(
                            "fill_missing_from_lookup",
                            &args,
                            &filled,
                            5,
                        ),
                        Err(error) => ToolResponse::error(error.to_string()),
                    }
                }
                Err(response) => response,
            }
        }
        Err(response) => response,
    }
}

fn dispatch_suggest_table_links(args: Value) -> ToolResponse {
    let Some(left_value) = args.get("left") else {
        return ToolResponse::error("suggest_table_links 缺少 left 参数");
    };
    let Some(right_value) = args.get("right") else {
        return ToolResponse::error("suggest_table_links 缺少 right 参数");
    };
    let max_candidates = args
        .get("max_candidates")
        .and_then(|value| value.as_u64())
        .unwrap_or(3) as usize;

    let left_casts = match parse_casts(&args, "left_casts", "suggest_table_links") {
        Ok(casts) => casts,
        Err(response) => return response,
    };
    let right_casts = match parse_casts(&args, "right_casts", "suggest_table_links") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match load_nested_table_source(left_value, "suggest_table_links", "left") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(left_loaded)) => {
            match load_nested_table_source(right_value, "suggest_table_links", "right") {
                Ok(OperationLoad::NeedsConfirmation(response)) => response,
                Ok(OperationLoad::Loaded(right_loaded)) => {
                    let prepared_left = apply_optional_casts(left_loaded, &left_casts);
                    let prepared_right = apply_optional_casts(right_loaded, &right_casts);

                    match (prepared_left, prepared_right) {
                        (Ok(prepared_left), Ok(prepared_right)) => {
                            match suggest_table_links(
                                &prepared_left,
                                &prepared_right,
                                max_candidates,
                            ) {
                                Ok(result) => ToolResponse::ok(json!(result)),
                                Err(error) => ToolResponse::error(error.to_string()),
                            }
                        }
                        (Err(error), _) | (_, Err(error)) => ToolResponse::error(error),
                    }
                }
                Err(response) => response,
            }
        }
        Err(response) => response,
    }
}

fn dispatch_suggest_table_workflow(args: Value) -> ToolResponse {
    let Some(left_value) = args.get("left") else {
        return ToolResponse::error("suggest_table_workflow 缺少 left 参数");
    };
    let Some(right_value) = args.get("right") else {
        return ToolResponse::error("suggest_table_workflow 缺少 right 参数");
    };
    let max_link_candidates = args
        .get("max_link_candidates")
        .and_then(|value| value.as_u64())
        .unwrap_or(3) as usize;

    let left_casts = match parse_casts(&args, "left_casts", "suggest_table_workflow") {
        Ok(casts) => casts,
        Err(response) => return response,
    };
    let right_casts = match parse_casts(&args, "right_casts", "suggest_table_workflow") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match load_nested_table_source(left_value, "suggest_table_workflow", "left") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(left_loaded)) => {
            match load_nested_table_source(right_value, "suggest_table_workflow", "right") {
                Ok(OperationLoad::NeedsConfirmation(response)) => response,
                Ok(OperationLoad::Loaded(right_loaded)) => {
                    let prepared_left = apply_optional_casts(left_loaded, &left_casts);
                    let prepared_right = apply_optional_casts(right_loaded, &right_casts);

                    match (prepared_left, prepared_right) {
                        (Ok(prepared_left), Ok(prepared_right)) => {
                            match suggest_table_workflow(
                                &prepared_left,
                                &prepared_right,
                                max_link_candidates,
                            ) {
                                Ok(result) => {
                                    let mut payload = json!(result);
                                    rewrite_workflow_suggested_tool_call_sources(
                                        &mut payload,
                                        left_value.clone(),
                                        right_value.clone(),
                                    );
                                    ToolResponse::ok(payload)
                                }
                                Err(error) => ToolResponse::error(error.to_string()),
                            }
                        }
                        (Err(error), _) | (_, Err(error)) => ToolResponse::error(error),
                    }
                }
                Err(response) => response,
            }
        }
        Err(response) => response,
    }
}

#[derive(Debug, Deserialize)]
struct MultiPlanTableInput {
    path: Option<String>,
    sheet: Option<String>,
    table_ref: Option<String>,
    result_ref: Option<String>,
    alias: Option<String>,
}

fn dispatch_suggest_multi_table_plan(args: Value) -> ToolResponse {
    match build_multi_table_plan_payload(&args) {
        Ok(payload) => ToolResponse::ok(payload),
        Err(response) => response,
    }
}

fn build_multi_table_plan_payload(args: &Value) -> Result<Value, ToolResponse> {
    let Some(tables_value) = args.get("tables") else {
        return Err(ToolResponse::error(
            "suggest_multi_table_plan missing required `tables` argument",
        ));
    };
    let table_inputs = serde_json::from_value::<Vec<MultiPlanTableInput>>(tables_value.clone())
        .map_err(|error| {
            ToolResponse::error(format!(
                "suggest_multi_table_plan failed to parse `tables`: {error}"
            ))
        })?;
    let max_link_candidates = args
        .get("max_link_candidates")
        .and_then(|value| value.as_u64())
        .unwrap_or(3) as usize;

    let mut loaded_tables = Vec::<(String, LoadedTable)>::new();
    let mut source_payloads = BTreeMap::<String, Value>::new();
    for (index, input) in table_inputs.into_iter().enumerate() {
        let table_ref = input
            .alias
            .unwrap_or_else(|| format!("table_{}", index + 1));
        let source = NestedTableSource {
            path: input.path,
            sheet: input.sheet,
            file_ref: None,
            sheet_index: None,
            table_ref: input.table_ref,
            result_ref: input.result_ref,
        };
        let source_payload = nested_source_payload(&source);
        source_payloads.insert(table_ref.clone(), source_payload);
        match load_nested_table_source_from_parsed(&source, "suggest_multi_table_plan", "tables") {
            Ok(OperationLoad::NeedsConfirmation(response)) => return Err(response),
            Ok(OperationLoad::Loaded(loaded)) => loaded_tables.push((table_ref, loaded)),
            Err(response) => return Err(response),
        }
    }

    let mut payload = json!(
        suggest_multi_table_plan(loaded_tables, max_link_candidates)
            .map_err(|error| ToolResponse::error(error.to_string()))?
    );
    rewrite_multi_table_plan_suggested_tool_call_sources(&mut payload, &source_payloads);
    Ok(payload)
}

fn resolve_multi_table_plan_for_execution(args: &Value) -> Result<Value, ToolResponse> {
    if let Some(plan) = args.get("plan").cloned() {
        let Some(plan_object) = plan.as_object() else {
            return Err(ToolResponse::error(
                "execute_multi_table_plan `plan` must be an object",
            ));
        };
        if plan_object
            .get("steps")
            .and_then(Value::as_array)
            .is_none()
        {
            return Err(ToolResponse::error(
                "execute_multi_table_plan `plan.steps` must be an array",
            ));
        }
        return Ok(plan);
    }

    build_multi_table_plan_payload(args)
}

fn dispatch_execute_suggested_tool_call(args: Value) -> ToolResponse {
    let Some(tool_call_value) = args.get("tool_call").cloned() else {
        return ToolResponse::error("execute_suggested_tool_call 缺少 tool_call 参数");
    };
    let mut tool_call = match serde_json::from_value::<ToolRequest>(tool_call_value) {
        Ok(request) => request,
        Err(error) => {
            return ToolResponse::error(format!(
                "execute_suggested_tool_call 的 tool_call 参数解析失败: {error}"
            ));
        }
    };
    if tool_call.tool.trim().is_empty() {
        return ToolResponse::error("execute_suggested_tool_call 的 tool_call.tool 不能为空");
    }
    if tool_call.tool == "execute_suggested_tool_call" {
        return ToolResponse::error(
            "execute_suggested_tool_call 不支持递归调用自身，请改为传入具体 tool",
        );
    }

    if let Some(overrides) = args.get("arg_overrides").cloned() {
        let Some(overrides_object) = overrides.as_object() else {
            return ToolResponse::error("execute_suggested_tool_call 的 arg_overrides 必须是对象");
        };
        match &mut tool_call.args {
            Value::Null => {
                tool_call.args = Value::Object(overrides_object.clone());
            }
            Value::Object(call_args) => {
                for (key, value) in overrides_object {
                    call_args.insert(key.clone(), value.clone());
                }
            }
            _ => {
                return ToolResponse::error(
                    "execute_suggested_tool_call 的 tool_call.args 必须是对象",
                );
            }
        }
    }

    if let Some(bindings) = args.get("result_ref_bindings").cloned() {
        let Some(bindings_object) = bindings.as_object() else {
            return ToolResponse::error(
                "execute_suggested_tool_call 的 result_ref_bindings 必须是对象",
            );
        };
        match &mut tool_call.args {
            Value::Null => {
                tool_call.args = json!({
                    "result_ref_bindings": bindings_object
                });
            }
            Value::Object(call_args) => {
                let entry = call_args
                    .entry("result_ref_bindings".to_string())
                    .or_insert_with(|| json!({}));
                let Some(call_bindings) = entry.as_object_mut() else {
                    return ToolResponse::error(
                        "execute_suggested_tool_call 的 tool_call.args.result_ref_bindings 必须是对象",
                    );
                };
                for (key, value) in bindings_object {
                    call_bindings.insert(key.clone(), value.clone());
                }
            }
            _ => {
                return ToolResponse::error(
                    "execute_suggested_tool_call 的 tool_call.args 必须是对象",
                );
            }
        }
    }

    dispatch(tool_call)
}

fn dispatch_execute_multi_table_plan(args: Value) -> ToolResponse {
    let auto_confirm_join = args
        .get("auto_confirm_join")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);

    let plan_payload = match resolve_multi_table_plan_for_execution(&args) {
        Ok(payload) => payload,
        Err(response) => return response,
    };
    let Some(steps) = plan_payload.get("steps").and_then(Value::as_array) else {
        return ToolResponse::error("execute_multi_table_plan expected array field `steps`");
    };

    let mut bindings = args
        .get("result_ref_bindings")
        .and_then(Value::as_object)
        .map(|bindings| {
            bindings
                .iter()
                .filter_map(|(key, value)| {
                    value
                        .as_str()
                        .map(|resolved| (key.clone(), resolved.to_string()))
                })
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let mut executed_steps = Vec::<Value>::new();
    let mut execution_status = "completed".to_string();
    let mut stop_reason: Option<String> = None;
    let mut stopped_at_step_id: Option<String> = None;
    let mut latest_result_ref: Option<String> = None;

    for step in steps {
        let step_id = step
            .get("step_id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let action = step
            .get("action")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let step_result_ref_alias = step
            .get("result_ref")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();

        let missing_aliases = step
            .get("pending_result_bindings")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.get("alias").and_then(Value::as_str))
                    .filter(|alias| !bindings.contains_key(*alias))
                    .map(|alias| alias.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if !missing_aliases.is_empty() {
            execution_status = "stopped_missing_result_bindings".to_string();
            stop_reason = Some(format!(
                "step `{}` missing result_ref_bindings: {}",
                step_id,
                missing_aliases.join(", ")
            ));
            stopped_at_step_id = Some(step_id.clone());
            executed_steps.push(json!({
                "step_id": step_id,
                "action": action,
                "status": "skipped",
                "reason": stop_reason.clone(),
                "missing_aliases": missing_aliases,
            }));
            break;
        }

        if !auto_confirm_join {
            let needs_preflight_confirmation = step
                .get("execution_status")
                .and_then(Value::as_str)
                .map(|status| status.starts_with("needs_preflight_confirmation"))
                .unwrap_or(false);
            if needs_preflight_confirmation {
                execution_status = "stopped_needs_preflight_confirmation".to_string();
                stop_reason = Some(format!(
                    "step `{}` requires preflight confirmation, set auto_confirm_join=true to continue",
                    step_id
                ));
                stopped_at_step_id = Some(step_id.clone());
                executed_steps.push(json!({
                    "step_id": step_id,
                    "action": action,
                    "status": "skipped",
                    "reason": stop_reason.clone(),
                }));
                break;
            }
        }

        let Some(suggested_tool_call) = step.get("suggested_tool_call").cloned() else {
            execution_status = "failed".to_string();
            stop_reason = Some(format!("step `{}` missing suggested_tool_call", step_id));
            stopped_at_step_id = Some(step_id.clone());
            executed_steps.push(json!({
                "step_id": step_id,
                "action": action,
                "status": "failed",
                "reason": stop_reason.clone(),
            }));
            break;
        };

        let mut exec_args = json!({
            "tool_call": suggested_tool_call,
            "result_ref_bindings": bindings,
        });
        if auto_confirm_join && action == "join_preflight" {
            exec_args["arg_overrides"] = json!({
                "confirm_join": true
            });
        }
        let response = dispatch_execute_suggested_tool_call(exec_args);

        if response.status != "ok" {
            execution_status = "failed".to_string();
            stop_reason = response.error.clone();
            stopped_at_step_id = Some(step_id.clone());
            executed_steps.push(json!({
                "step_id": step_id,
                "action": action,
                "status": "failed",
                "error": response.error,
            }));
            break;
        }

        let output_result_ref = response
            .data
            .get("result_ref")
            .and_then(Value::as_str)
            .map(|item| item.to_string());
        if let Some(result_ref) = output_result_ref.clone() {
            latest_result_ref = Some(result_ref.clone());
            if !step_result_ref_alias.is_empty() {
                bindings.insert(step_result_ref_alias, result_ref);
            }
        }

        executed_steps.push(json!({
            "step_id": step_id,
            "action": action,
            "status": "ok",
            "response": response.data,
            "output_result_ref": output_result_ref,
        }));
    }

    ToolResponse::ok(json!({
        "execution_status": execution_status,
        "stop_reason": stop_reason,
        "stopped_at_step_id": stopped_at_step_id,
        "auto_confirm_join": auto_confirm_join,
        "executed_steps": executed_steps,
        "result_ref_bindings": bindings,
        "latest_result_ref": latest_result_ref,
        "plan": plan_payload,
    }))
}

fn dispatch_append_tables(args: Value) -> ToolResponse {
    let Some(top_value) = args.get("top") else {
        return ToolResponse::error("append_tables 缺少 top 参数");
    };
    let Some(bottom_value) = args.get("bottom") else {
        return ToolResponse::error("append_tables 缺少 bottom 参数");
    };
    let limit = args
        .get("limit")
        .and_then(|value| value.as_u64())
        .unwrap_or(5) as usize;

    match load_nested_table_source(top_value, "append_tables", "top") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(top_loaded)) => {
            match load_nested_table_source(bottom_value, "append_tables", "bottom") {
                Ok(OperationLoad::NeedsConfirmation(response)) => response,
                Ok(OperationLoad::Loaded(bottom_loaded)) => {
                    match append_tables(&top_loaded, &bottom_loaded) {
                        Ok(appended) => respond_with_preview_and_result_ref(
                            "append_tables",
                            &args,
                            &appended,
                            limit,
                        ),
                        Err(error) => ToolResponse::error(error.to_string()),
                    }
                }
                Err(response) => response,
            }
        }
        Err(response) => response,
    }
}

fn dispatch_summarize_table(args: Value) -> ToolResponse {
    let requested_columns = string_array(&args, "columns");
    let top_k = args
        .get("top_k")
        .and_then(|value| value.as_u64())
        .unwrap_or(5) as usize;
    let casts = match parse_casts(&args, "casts", "summarize_table") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match load_table_for_analysis(&args, "summarize_table") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => {
                match summarize_table(&prepared_loaded, &requested_columns, top_k) {
                    Ok(summaries) => {
                        if let Err(response) = sync_loaded_table_state(
                            &args,
                            &prepared_loaded,
                            SessionStage::AnalysisModeling,
                            "查看统计摘要",
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

fn dispatch_analyze_table(args: Value) -> ToolResponse {
    let requested_columns = string_array(&args, "columns");
    let top_k = args
        .get("top_k")
        .and_then(|value| value.as_u64())
        .unwrap_or(5) as usize;
    let casts = match parse_casts(&args, "casts", "analyze_table") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match load_table_for_analysis(&args, "analyze_table") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => {
                let result = analyze_table(&prepared_loaded, &requested_columns, top_k);
                if let Err(response) = sync_loaded_table_state(
                    &args,
                    &prepared_loaded,
                    SessionStage::AnalysisModeling,
                    "查看分析诊断",
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

fn dispatch_stat_summary(args: Value) -> ToolResponse {
    let requested_columns = string_array(&args, "columns");
    let top_k = args
        .get("top_k")
        .and_then(|value| value.as_u64())
        .unwrap_or(5) as usize;
    let casts = match parse_casts(&args, "casts", "stat_summary") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match load_table_for_analysis(&args, "stat_summary") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => {
                match stat_summary(&prepared_loaded, &requested_columns, top_k) {
                    Ok(result) => {
                        if let Err(response) = sync_loaded_table_state(
                            &args,
                            &prepared_loaded,
                            SessionStage::AnalysisModeling,
                            "查看统计摘要",
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

fn dispatch_correlation_analysis(args: Value) -> ToolResponse {
    let Some(target_column) = args.get("target_column").and_then(|value| value.as_str()) else {
        return ToolResponse::error("correlation_analysis 缺少 target_column 参数");
    };
    let feature_columns = string_array(&args, "feature_columns");
    let casts = match parse_casts(&args, "casts", "correlation_analysis") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match load_table_for_analysis(&args, "correlation_analysis") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => {
                match correlation_analysis(&prepared_loaded, target_column, &feature_columns) {
                    Ok(result) => {
                        if let Err(response) = sync_loaded_table_state(
                            &args,
                            &prepared_loaded,
                            SessionStage::AnalysisModeling,
                            "查看相关性分析",
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

fn dispatch_outlier_detection(args: Value) -> ToolResponse {
    let columns = string_array(&args, "columns");
    let method = match args.get("method") {
        Some(value) => match serde_json::from_value::<OutlierDetectionMethod>(value.clone()) {
            Ok(method) => method,
            Err(error) => {
                return ToolResponse::error(format!(
                    "outlier_detection 的 method 参数解析失败: {error}"
                ));
            }
        },
        None => OutlierDetectionMethod::Iqr,
    };
    let casts = match parse_casts(&args, "casts", "outlier_detection") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match load_table_for_analysis(&args, "outlier_detection") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => match outlier_detection(&prepared_loaded, &columns, method) {
                Ok((flagged_loaded, result)) => {
                    if let Err(response) = sync_loaded_table_state(
                        &args,
                        &prepared_loaded,
                        SessionStage::AnalysisModeling,
                        "查看异常值诊断",
                        "outlier_detection",
                        "analysis_completed",
                    ) {
                        return response;
                    }
                    let preview = match preview_table(&flagged_loaded.dataframe, 20) {
                        Ok(preview) => preview,
                        Err(error) => return ToolResponse::error(error.to_string()),
                    };
                    respond_with_result_dataset(
                        "outlier_detection",
                        &args,
                        &flagged_loaded,
                        json!({
                            "method": result.method,
                            "row_count": result.row_count,
                            "outlier_summaries": result.outlier_summaries,
                            "human_summary": result.human_summary,
                            "columns": preview.columns,
                            "rows": preview.rows
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

fn dispatch_distribution_analysis(args: Value) -> ToolResponse {
    let Some(column) = args.get("column").and_then(|value| value.as_str()) else {
        return ToolResponse::error("distribution_analysis 缺少 column 参数");
    };
    let bins = args
        .get("bins")
        .and_then(|value| value.as_u64())
        .unwrap_or(10) as usize;
    let casts = match parse_casts(&args, "casts", "distribution_analysis") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match load_table_for_analysis(&args, "distribution_analysis") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => match distribution_analysis(&prepared_loaded, column, bins) {
                Ok(result) => {
                    if let Err(response) = sync_loaded_table_state(
                        &args,
                        &prepared_loaded,
                        SessionStage::AnalysisModeling,
                        "查看分布分析",
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

fn dispatch_trend_analysis(args: Value) -> ToolResponse {
    let Some(time_column) = args.get("time_column").and_then(|value| value.as_str()) else {
        return ToolResponse::error("trend_analysis 缺少 time_column 参数");
    };
    let Some(value_column) = args.get("value_column").and_then(|value| value.as_str()) else {
        return ToolResponse::error("trend_analysis 缺少 value_column 参数");
    };
    let casts = match parse_casts(&args, "casts", "trend_analysis") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match load_table_for_analysis(&args, "trend_analysis") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => {
                match trend_analysis(&prepared_loaded, time_column, value_column) {
                    Ok(result) => {
                        if let Err(response) = sync_loaded_table_state(
                            &args,
                            &prepared_loaded,
                            SessionStage::AnalysisModeling,
                            "查看趋势分析",
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

fn dispatch_linear_regression(args: Value) -> ToolResponse {
    let Some(features_value) = args.get("features").and_then(|value| value.as_array()) else {
        return ToolResponse::error("linear_regression 缺少 features 参数");
    };
    let Some(target) = args.get("target").and_then(|value| value.as_str()) else {
        return ToolResponse::error("linear_regression 缺少 target 参数");
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

    match load_table_for_analysis(&args, "linear_regression") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => match linear_regression(
                &prepared_loaded,
                &features,
                target,
                intercept,
                missing_strategy,
            ) {
                Ok(result) => {
                    if let Err(response) = sync_loaded_table_state(
                        &args,
                        &prepared_loaded,
                        SessionStage::AnalysisModeling,
                        "执行线性回归分析",
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

fn dispatch_logistic_regression(args: Value) -> ToolResponse {
    let Some(features_value) = args.get("features").and_then(|value| value.as_array()) else {
        return ToolResponse::error("logistic_regression 缺少 features 参数");
    };
    let Some(target) = args.get("target").and_then(|value| value.as_str()) else {
        return ToolResponse::error("logistic_regression 缺少 target 参数");
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

    match load_table_for_analysis(&args, "logistic_regression") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => match logistic_regression(
                &prepared_loaded,
                &features,
                target,
                intercept,
                missing_strategy,
                positive_label,
            ) {
                Ok(result) => {
                    if let Err(response) = sync_loaded_table_state(
                        &args,
                        &prepared_loaded,
                        SessionStage::AnalysisModeling,
                        "执行逻辑回归分析",
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
fn dispatch_cluster_kmeans(args: Value) -> ToolResponse {
    let Some(features_value) = args.get("features").and_then(|value| value.as_array()) else {
        return ToolResponse::error("cluster_kmeans 缺少 features 参数");
    };
    let Some(cluster_count) = args.get("cluster_count").and_then(|value| value.as_u64()) else {
        return ToolResponse::error("cluster_kmeans 缺少 cluster_count 参数");
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

    match load_table_for_analysis(&args, "cluster_kmeans") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => match cluster_kmeans(
                &prepared_loaded,
                &features,
                cluster_count as usize,
                max_iterations,
                missing_strategy,
            ) {
                Ok(result) => {
                    if let Err(response) = sync_loaded_table_state(
                        &args,
                        &prepared_loaded,
                        SessionStage::AnalysisModeling,
                        "执行聚类分析",
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

fn dispatch_decision_assistant(args: Value) -> ToolResponse {
    let requested_columns = string_array(&args, "columns");
    let top_k = args
        .get("top_k")
        .and_then(|value| value.as_u64())
        .unwrap_or(5) as usize;
    let casts = match parse_casts(&args, "casts", "decision_assistant") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match load_table_for_analysis(&args, "decision_assistant") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => {
                match decision_assistant(&prepared_loaded, &requested_columns, top_k) {
                    Ok(result) => {
                        if let Err(response) = sync_loaded_table_state(
                            &args,
                            &prepared_loaded,
                            SessionStage::DecisionAssistant,
                            "获取下一步决策建议",
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

enum OperationLoad {
    NeedsConfirmation(ToolResponse),
    Loaded(LoadedTable),
}

#[derive(Debug, Deserialize)]
struct NestedTableSource {
    path: Option<String>,
    sheet: Option<String>,
    file_ref: Option<String>,
    sheet_index: Option<usize>,
    table_ref: Option<String>,
    result_ref: Option<String>,
}

struct ResolvedSheetSource {
    path: String,
    sheet_name: String,
}

fn load_sheet_for_operation(path: &str, sheet: &str) -> Result<OperationLoad, String> {
    match infer_header_schema(path, sheet) {
        Ok(inference) => {
            if !matches!(inference.confidence, ConfidenceLevel::High) {
                return Ok(OperationLoad::NeedsConfirmation(build_inference_response(
                    sheet, inference,
                )));
            }

            load_confirmed_table(path, sheet, &inference)
                .map(OperationLoad::Loaded)
                .map_err(|error| error.to_string())
        }
        Err(error) => Err(error.to_string()),
    }
}

fn load_nested_table_source(
    value: &Value,
    tool: &str,
    field_name: &str,
) -> Result<OperationLoad, ToolResponse> {
    let source = parse_nested_table_source(value, tool, field_name)?;
    load_nested_table_source_from_parsed(&source, tool, field_name)
}

fn parse_nested_table_source(
    value: &Value,
    tool: &str,
    field_name: &str,
) -> Result<NestedTableSource, ToolResponse> {
    serde_json::from_value::<NestedTableSource>(value.clone()).map_err(|error| {
        ToolResponse::error(format!("{tool} 的 {field_name} 参数解析失败: {error}"))
    })
}

fn load_nested_table_source_from_parsed(
    source: &NestedTableSource,
    tool: &str,
    field_name: &str,
) -> Result<OperationLoad, ToolResponse> {
    if let Some(result_ref) = source.result_ref.as_deref() {
        return load_result_from_ref(result_ref).map_err(ToolResponse::error);
    }

    if let Some(table_ref) = source.table_ref.as_deref() {
        return load_table_from_ref(table_ref).map_err(ToolResponse::error);
    }

    if let Some(file_ref) = source.file_ref.as_deref() {
        let Some(sheet_index) = source.sheet_index else {
            return Err(ToolResponse::error(format!(
                "{tool} 的 {field_name} 缺少 sheet_index 参数"
            )));
        };
        let resolved = resolve_sheet_source_from_file_ref(file_ref, sheet_index)
            .map_err(ToolResponse::error)?;
        return load_sheet_for_operation(&resolved.path, &resolved.sheet_name)
            .map_err(ToolResponse::error);
    }

    match (source.path.as_deref(), source.sheet.as_deref()) {
        (Some(path), Some(sheet)) => {
            load_sheet_for_operation(path, sheet).map_err(ToolResponse::error)
        }
        (Some(_), None) => Err(ToolResponse::error(format!(
            "{tool} 的 {field_name} 缺少 sheet 参数"
        ))),
        (None, Some(_)) => Err(ToolResponse::error(format!(
            "{tool} 的 {field_name} 缺少 path 参数"
        ))),
        (None, None) => Err(ToolResponse::error(format!(
            "{tool} 需要提供 path+sheet、file_ref+sheet_index、table_ref 或 result_ref"
        ))),
    }
}

fn nested_source_payload(source: &NestedTableSource) -> Value {
    if let Some(table_ref) = source.table_ref.as_ref() {
        return json!({ "table_ref": table_ref });
    }
    if let Some(result_ref) = source.result_ref.as_ref() {
        return json!({ "result_ref": result_ref });
    }
    if let (Some(file_ref), Some(sheet_index)) = (source.file_ref.as_ref(), source.sheet_index) {
        return json!({
            "file_ref": file_ref,
            "sheet_index": sheet_index,
        });
    }
    if let (Some(path), Some(sheet)) = (source.path.as_ref(), source.sheet.as_ref()) {
        return json!({
            "path": path,
            "sheet": sheet,
        });
    }

    json!({})
}

fn source_refs_from_nested_source(source: &NestedTableSource) -> Vec<String> {
    if let Some(table_ref) = source.table_ref.as_ref() {
        return vec![table_ref.clone()];
    }
    if let Some(result_ref) = source.result_ref.as_ref() {
        return vec![result_ref.clone()];
    }
    if let (Some(file_ref), Some(sheet_index)) = (source.file_ref.as_ref(), source.sheet_index) {
        return vec![format!("{file_ref}#{sheet_index}")];
    }
    if let (Some(path), Some(sheet)) = (source.path.as_ref(), source.sheet.as_ref()) {
        return vec![format!("{path}#{sheet}")];
    }
    Vec::new()
}

fn load_table_for_analysis(args: &Value, tool: &str) -> Result<OperationLoad, ToolResponse> {
    load_table_for_tool(args, tool)
}

fn load_table_for_tool(args: &Value, tool: &str) -> Result<OperationLoad, ToolResponse> {
    if let Some(result_ref) = args.get("result_ref").and_then(|value| value.as_str()) {
        return load_result_from_ref(result_ref).map_err(ToolResponse::error);
    }

    if let Some(table_ref) = args.get("table_ref").and_then(|value| value.as_str()) {
        return load_table_from_ref(table_ref).map_err(ToolResponse::error);
    }

    let source = resolve_sheet_source(args, tool)?;
    load_sheet_for_operation(&source.path, &source.sheet_name).map_err(ToolResponse::error)
}

fn load_table_from_ref(table_ref: &str) -> Result<OperationLoad, String> {
    let store = TableRefStore::workspace_default().map_err(|error| error.to_string())?;
    let persisted = store.load(table_ref).map_err(|error| error.to_string())?;
    load_table_from_table_ref(&persisted)
        .map(OperationLoad::Loaded)
        .map_err(|error| error.to_string())
}

fn load_result_from_ref(result_ref: &str) -> Result<OperationLoad, String> {
    let store = ResultRefStore::workspace_default().map_err(|error| error.to_string())?;
    let persisted = store.load(result_ref).map_err(|error| error.to_string())?;
    let dataframe = persisted
        .to_dataframe()
        .map_err(|error| error.to_string())?;
    let handle = TableHandle::new_confirmed(
        format!("result://{result_ref}"),
        persisted.produced_by.clone(),
        persisted
            .columns
            .iter()
            .map(|column| column.name.clone())
            .collect(),
    );

    Ok(OperationLoad::Loaded(LoadedTable { handle, dataframe }))
}

fn build_opened_file_response(
    args: &Value,
    summary: crate::excel::reader::WorkbookSummary,
) -> ToolResponse {
    let original_path = args
        .get("original_path")
        .and_then(|value| value.as_str())
        .unwrap_or(summary.path.as_str());
    let recovery_applied = args
        .get("recovery_applied")
        .and_then(|value| value.as_bool())
        .unwrap_or(original_path != summary.path);
    let file_ref = SourceFileRefStore::create_file_ref();
    let persisted = match PersistedSourceFileRef::from_opened_file(
        file_ref.clone(),
        original_path,
        &summary.path,
        &summary.sheet_names,
        recovery_applied,
    ) {
        Ok(persisted) => persisted,
        Err(error) => return ToolResponse::error(error.to_string()),
    };
    let store = match SourceFileRefStore::workspace_default() {
        Ok(store) => store,
        Err(error) => return ToolResponse::error(error.to_string()),
    };
    if let Err(error) = store.save(&persisted) {
        return ToolResponse::error(error.to_string());
    }

    ToolResponse::ok(json!({
        "path": summary.path,
        "original_path": persisted.original_path,
        "working_path": persisted.working_path,
        "recovery_applied": persisted.recovery_applied,
        "file_ref": persisted.file_ref,
        "sheet_names": summary.sheet_names,
        "sheets": persisted.sheets,
    }))
}

fn resolve_sheet_source(args: &Value, tool: &str) -> Result<ResolvedSheetSource, ToolResponse> {
    if let Some(file_ref) = args.get("file_ref").and_then(|value| value.as_str()) {
        let Some(sheet_index) = args.get("sheet_index").and_then(|value| value.as_u64()) else {
            return Err(ToolResponse::error(format!(
                "{tool} 缺少 sheet_index 参数，或请改传 path + sheet"
            )));
        };
        return resolve_sheet_source_from_file_ref(file_ref, sheet_index as usize)
            .map_err(ToolResponse::error);
    }

    let Some(path) = args.get("path").and_then(|value| value.as_str()) else {
        return Err(ToolResponse::error(format!(
            "{tool} 缺少 path 参数，或请改传 file_ref + sheet_index / table_ref"
        )));
    };
    let Some(sheet) = args.get("sheet").and_then(|value| value.as_str()) else {
        return Err(ToolResponse::error(format!(
            "{tool} 缺少 sheet 参数，或请改传 file_ref + sheet_index / table_ref"
        )));
    };

    Ok(ResolvedSheetSource {
        path: path.to_string(),
        sheet_name: sheet.to_string(),
    })
}

fn resolve_sheet_source_from_file_ref(
    file_ref: &str,
    sheet_index: usize,
) -> Result<ResolvedSheetSource, String> {
    let store = SourceFileRefStore::workspace_default().map_err(|error| error.to_string())?;
    let persisted = store.load(file_ref).map_err(|error| error.to_string())?;
    persisted
        .validate_source_unchanged()
        .map_err(|error| error.to_string())?;
    let sheet_name = persisted
        .sheet_name_for_index(sheet_index)
        .map_err(|error| error.to_string())?;

    Ok(ResolvedSheetSource {
        path: persisted.working_path,
        sheet_name,
    })
}

fn build_inference_response(sheet: &str, inference: HeaderInference) -> ToolResponse {
    let payload = json!({
        "sheet": sheet,
        "confidence": confidence_label(&inference.confidence),
        "header_row_count": inference.header_row_count,
        "columns": inference.columns,
        "schema_state": infer_schema_state_label(&inference.schema_state),
    });

    if matches!(inference.confidence, ConfidenceLevel::High) {
        ToolResponse::ok(payload)
    } else {
        ToolResponse::needs_confirmation(payload)
    }
}

fn confidence_label(confidence: &ConfidenceLevel) -> &'static str {
    match confidence {
        ConfidenceLevel::High => "high",
        ConfidenceLevel::Medium => "medium",
        ConfidenceLevel::Low => "low",
    }
}

fn preview_loaded_table(loaded: &LoadedTable, limit: usize) -> ToolResponse {
    match preview_table(&loaded.dataframe, limit) {
        Ok(preview) => ToolResponse::ok(json!({
            "columns": preview.columns,
            "rows": preview.rows,
            "row_count": loaded.dataframe.height(),
        })),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

fn respond_with_preview_and_result_ref(
    tool_name: &str,
    args: &Value,
    loaded: &LoadedTable,
    limit: usize,
) -> ToolResponse {
    match preview_table(&loaded.dataframe, limit) {
        Ok(preview) => respond_with_result_dataset(
            tool_name,
            args,
            loaded,
            json!({
                "columns": preview.columns,
                "rows": preview.rows,
                "row_count": loaded.dataframe.height(),
            }),
        ),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

fn respond_with_result_dataset(
    tool_name: &str,
    args: &Value,
    loaded: &LoadedTable,
    payload: Value,
) -> ToolResponse {
    let result_ref = match persist_result_dataset(tool_name, args, loaded) {
        Ok(result_ref) => result_ref,
        Err(response) => return response,
    };
    if let Err(response) = sync_output_handle_state(args, &result_ref, "result_ref", tool_name) {
        return response;
    }

    let mut object = match payload {
        Value::Object(object) => object,
        _ => {
            return ToolResponse::error(format!(
                "{tool_name} 返回结果不是对象，无法附加 result_ref"
            ));
        }
    };
    object.insert("result_ref".to_string(), Value::String(result_ref));
    ToolResponse::ok(Value::Object(object))
}

fn string_array<'a>(args: &'a Value, field: &str) -> Vec<&'a str> {
    args.get(field)
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|value| value.as_str())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn parse_lookup_key_args(
    args: &Value,
    single_field: &str,
    multi_field: &str,
    tool: &str,
) -> Result<Vec<String>, ToolResponse> {
    let single_value = args.get(single_field).and_then(|value| value.as_str());
    let multi_values = args.get(multi_field).and_then(|value| value.as_array());

    if single_value.is_some() && multi_values.is_some() {
        return Err(ToolResponse::error(format!(
            "{tool} 不能同时传 {single_field} 和 {multi_field}"
        )));
    }

    if let Some(value) = single_value {
        if value.trim().is_empty() {
            return Err(ToolResponse::error(format!(
                "{tool} 缺少 {single_field} 参数"
            )));
        }
        return Ok(vec![value.to_string()]);
    }

    if let Some(values) = multi_values {
        let keys = values
            .iter()
            .filter_map(|value| value.as_str())
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>();
        if keys.is_empty() {
            return Err(ToolResponse::error(format!(
                "{tool} 缺少 {multi_field} 参数"
            )));
        }
        return Ok(keys);
    }

    Err(ToolResponse::error(format!(
        "{tool} 缺少 {single_field} 参数"
    )))
}

fn parse_casts(args: &Value, field: &str, tool: &str) -> Result<Vec<CastColumnSpec>, ToolResponse> {
    match args.get(field) {
        Some(casts_value) => serde_json::from_value::<Vec<CastColumnSpec>>(casts_value.clone())
            .map_err(|error| {
                ToolResponse::error(format!("{tool} 的 {field} 参数解析失败: {error}"))
            }),
        None => Ok(Vec::new()),
    }
}

fn parse_missing_strategy(args: &Value, tool: &str) -> Result<MissingStrategy, ToolResponse> {
    match args.get("missing_strategy") {
        Some(strategy_value) => serde_json::from_value::<MissingStrategy>(strategy_value.clone())
            .map_err(|error| {
                ToolResponse::error(format!("{tool} 的 missing_strategy 参数解析失败: {error}"))
            }),
        None => Ok(MissingStrategy::DropRows),
    }
}

fn apply_optional_casts(
    loaded: LoadedTable,
    casts: &[CastColumnSpec],
) -> Result<LoadedTable, String> {
    if casts.is_empty() {
        Ok(loaded)
    } else {
        cast_column_types(&loaded, casts).map_err(|error| error.to_string())
    }
}

fn persist_result_dataset(
    tool_name: &str,
    args: &Value,
    loaded: &LoadedTable,
) -> Result<String, ToolResponse> {
    let result_ref = ResultRefStore::create_result_ref();
    let store = ResultRefStore::workspace_default()
        .map_err(|error| ToolResponse::error(error.to_string()))?;
    let record = PersistedResultDataset::from_dataframe(
        &result_ref,
        tool_name,
        source_refs_from_args(args),
        &loaded.dataframe,
    )
    .map_err(|error| ToolResponse::error(error.to_string()))?;
    store
        .save(&record)
        .map_err(|error| ToolResponse::error(error.to_string()))?;
    Ok(result_ref)
}

fn source_refs_from_args(args: &Value) -> Vec<String> {
    let mut refs = Vec::<String>::new();
    collect_source_refs(args, &mut refs);
    refs
}

fn collect_source_refs(value: &Value, refs: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            if let Some(result_ref) = map.get("result_ref").and_then(|item| item.as_str()) {
                push_unique_source_ref(refs, result_ref.to_string());
            }
            if let Some(table_ref) = map.get("table_ref").and_then(|item| item.as_str()) {
                push_unique_source_ref(refs, table_ref.to_string());
            }
            if let (Some(path), Some(sheet)) = (
                map.get("path").and_then(|item| item.as_str()),
                map.get("sheet").and_then(|item| item.as_str()),
            ) {
                push_unique_source_ref(refs, format!("{path}#{sheet}"));
            }
            if let Some(file_ref) = map.get("file_ref").and_then(|item| item.as_str()) {
                let source_ref = map
                    .get("sheet_index")
                    .and_then(|item| item.as_u64())
                    .map(|sheet_index| format!("{file_ref}#{sheet_index}"))
                    .unwrap_or_else(|| file_ref.to_string());
                push_unique_source_ref(refs, source_ref);
            }

            for child in map.values() {
                collect_source_refs(child, refs);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_source_refs(item, refs);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

fn push_unique_source_ref(refs: &mut Vec<String>, candidate: String) {
    if !refs.iter().any(|existing| existing == &candidate) {
        refs.push(candidate);
    }
}

fn first_table_ref_in_value(value: &Value) -> Option<String> {
    match value {
        Value::Object(map) => {
            if let Some(table_ref) = map.get("table_ref").and_then(|item| item.as_str()) {
                return Some(table_ref.to_string());
            }
            map.values().find_map(first_table_ref_in_value)
        }
        Value::Array(items) => items.iter().find_map(first_table_ref_in_value),
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => None,
    }
}

fn rewrite_workflow_suggested_tool_call_sources(
    payload: &mut Value,
    left_source: Value,
    right_source: Value,
) {
    let Some(suggested_args) = payload
        .get_mut("suggested_tool_call")
        .and_then(|call| call.get_mut("args"))
        .and_then(Value::as_object_mut)
    else {
        return;
    };

    if suggested_args.contains_key("top") && suggested_args.contains_key("bottom") {
        suggested_args.insert("top".to_string(), left_source);
        suggested_args.insert("bottom".to_string(), right_source);
        return;
    }

    if suggested_args.contains_key("left") && suggested_args.contains_key("right") {
        suggested_args.insert("left".to_string(), left_source);
        suggested_args.insert("right".to_string(), right_source);
    }
}

fn rewrite_multi_table_plan_suggested_tool_call_sources(
    payload: &mut Value,
    source_payloads: &BTreeMap<String, Value>,
) {
    let Some(steps) = payload.get_mut("steps").and_then(Value::as_array_mut) else {
        return;
    };

    for step in steps {
        let Some(action) = step
            .get("action")
            .and_then(Value::as_str)
            .map(|value| value.to_string())
        else {
            continue;
        };
        let input_refs = step
            .get("input_refs")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(|item| item.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if input_refs.len() < 2 {
            continue;
        }

        let Some(suggested_args) = step
            .get_mut("suggested_tool_call")
            .and_then(|call| call.get_mut("args"))
            .and_then(Value::as_object_mut)
        else {
            continue;
        };

        let first_source = planned_source_payload(&input_refs[0], source_payloads);
        let second_source = planned_source_payload(&input_refs[1], source_payloads);

        match action.as_str() {
            "append_tables" => {
                suggested_args.insert("top".to_string(), first_source);
                suggested_args.insert("bottom".to_string(), second_source);
            }
            "join_preflight" | "join_tables" => {
                suggested_args.insert("left".to_string(), first_source);
                suggested_args.insert("right".to_string(), second_source);
            }
            _ => {}
        }
    }
}

fn planned_source_payload(reference: &str, source_payloads: &BTreeMap<String, Value>) -> Value {
    source_payloads
        .get(reference)
        .cloned()
        .unwrap_or_else(|| json!({ "result_ref": reference }))
}

fn memory_runtime() -> Result<LocalMemoryRuntime, ToolResponse> {
    LocalMemoryRuntime::workspace_default().map_err(|error| ToolResponse::error(error.to_string()))
}

fn session_id_from_args(args: &Value) -> String {
    args.get("session_id")
        .and_then(|value| value.as_str())
        .unwrap_or("default")
        .to_string()
}

fn user_goal_from_args(args: &Value, fallback: &str) -> String {
    args.get("user_goal")
        .and_then(|value| value.as_str())
        .unwrap_or(fallback)
        .to_string()
}

fn selected_columns_from_args(args: &Value) -> Option<Vec<String>> {
    let mut selected = Vec::<String>::new();

    if let Some(columns) = args.get("columns").and_then(|value| value.as_array()) {
        selected.extend(
            columns
                .iter()
                .filter_map(|value| value.as_str())
                .map(|value| value.to_string()),
        );
    }

    if let Some(features) = args.get("features").and_then(|value| value.as_array()) {
        selected.extend(
            features
                .iter()
                .filter_map(|value| value.as_str())
                .map(|value| value.to_string()),
        );
    }

    if let Some(target) = args.get("target").and_then(|value| value.as_str()) {
        if !selected.iter().any(|column| column == target) {
            selected.push(target.to_string());
        }
    }

    if selected.is_empty() {
        None
    } else {
        Some(selected)
    }
}

fn sync_confirmed_table_state(
    args: &Value,
    persisted: &PersistedTableRef,
    fallback_goal: &str,
) -> Result<(), ToolResponse> {
    let runtime = memory_runtime()?;
    let session_id = session_id_from_args(args);
    runtime
        .mirror_table_ref(persisted)
        .map_err(|error| ToolResponse::error(error.to_string()))?;
    runtime
        .update_session_state(
            &session_id,
            &SessionStatePatch {
                current_workbook: Some(current_workbook_for_session(args, &persisted.source_path)),
                current_sheet: Some(persisted.sheet_name.clone()),
                current_file_ref: current_file_ref_from_args(args),
                current_sheet_index: current_sheet_index_from_args(args),
                current_stage: Some(SessionStage::TableProcessing),
                schema_status: Some(SchemaStatus::Confirmed),
                active_table_ref: Some(persisted.table_ref.clone()),
                active_handle_ref: None,
                active_handle_kind: None,
                last_user_goal: Some(user_goal_from_args(args, fallback_goal)),
                selected_columns: Some(persisted.columns.clone()),
            },
        )
        .map_err(|error| ToolResponse::error(error.to_string()))?;
    runtime
        .append_event(
            &session_id,
            &EventLogInput {
                event_type: "schema_confirmed".to_string(),
                stage: Some(SessionStage::TableProcessing),
                tool_name: Some("apply_header_schema".to_string()),
                status: "ok".to_string(),
                message: Some("确认表头后已激活 table_ref".to_string()),
            },
        )
        .map_err(|error| ToolResponse::error(error.to_string()))?;
    Ok(())
}

fn sync_loaded_table_state(
    args: &Value,
    loaded: &LoadedTable,
    stage: SessionStage,
    fallback_goal: &str,
    tool_name: &str,
    event_type: &str,
) -> Result<(), ToolResponse> {
    let runtime = memory_runtime()?;
    let session_id = session_id_from_args(args);
    let current_state = runtime
        .get_session_state(&session_id)
        .map_err(|error| ToolResponse::error(error.to_string()))?;
    let input_handle_ref = active_handle_ref_from_args(args)
        .or_else(|| first_table_ref_in_value(args))
        .or_else(|| current_state.active_handle_ref.clone());
    let stable_table_ref =
        first_table_ref_in_value(args).or(current_state.active_table_ref.clone());
    runtime
        .update_session_state(
            &session_id,
            &SessionStatePatch {
                current_workbook: Some(current_workbook_for_session(
                    args,
                    loaded.handle.source_path(),
                )),
                current_sheet: Some(loaded.handle.sheet_name().to_string()),
                current_file_ref: current_file_ref_from_args(args),
                current_sheet_index: current_sheet_index_from_args(args),
                current_stage: Some(stage.clone()),
                schema_status: Some(SchemaStatus::Confirmed),
                active_table_ref: stable_table_ref,
                active_handle_ref: input_handle_ref.clone(),
                active_handle_kind: input_handle_ref
                    .as_deref()
                    .map(classify_handle_kind)
                    .map(str::to_string),
                last_user_goal: Some(user_goal_from_args(args, fallback_goal)),
                selected_columns: selected_columns_from_args(args),
            },
        )
        .map_err(|error| ToolResponse::error(error.to_string()))?;
    runtime
        .append_event(
            &session_id,
            &EventLogInput {
                event_type: event_type.to_string(),
                stage: Some(stage),
                tool_name: Some(tool_name.to_string()),
                status: "ok".to_string(),
                message: Some(format!("{tool_name} 已同步当前层级状态")),
            },
        )
        .map_err(|error| ToolResponse::error(error.to_string()))?;
    Ok(())
}

fn active_handle_ref_from_args(args: &Value) -> Option<String> {
    args.get("result_ref")
        .and_then(|value| value.as_str())
        .or_else(|| args.get("table_ref").and_then(|value| value.as_str()))
        .map(|value| value.to_string())
}

fn sync_output_handle_state(
    args: &Value,
    handle_ref: &str,
    handle_kind: &str,
    tool_name: &str,
) -> Result<(), ToolResponse> {
    let runtime = memory_runtime()?;
    let session_id = session_id_from_args(args);
    let current_state = runtime
        .get_session_state(&session_id)
        .map_err(|error| ToolResponse::error(error.to_string()))?;
    let stable_table_ref =
        first_table_ref_in_value(args).or(current_state.active_table_ref.clone());

    runtime
        .update_session_state(
            &session_id,
            &SessionStatePatch {
                current_workbook: None,
                current_sheet: None,
                current_file_ref: None,
                current_sheet_index: None,
                current_stage: None,
                schema_status: None,
                active_table_ref: stable_table_ref,
                active_handle_ref: Some(handle_ref.to_string()),
                active_handle_kind: Some(handle_kind.to_string()),
                last_user_goal: None,
                selected_columns: None,
            },
        )
        .map_err(|error| ToolResponse::error(error.to_string()))?;
    runtime
        .append_event(
            &session_id,
            &EventLogInput {
                event_type: "active_handle_updated".to_string(),
                stage: None,
                tool_name: Some(tool_name.to_string()),
                status: "ok".to_string(),
                message: Some(format!("{tool_name} 更新当前句柄为 {handle_kind}")),
            },
        )
        .map_err(|error| ToolResponse::error(error.to_string()))?;
    Ok(())
}

fn current_file_ref_from_args(args: &Value) -> Option<String> {
    args.get("file_ref")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
}

fn current_sheet_index_from_args(args: &Value) -> Option<usize> {
    args.get("sheet_index")
        .and_then(|value| value.as_u64())
        .map(|value| value as usize)
}

fn current_workbook_for_session(args: &Value, fallback_path: &str) -> String {
    let Some(file_ref) = current_file_ref_from_args(args) else {
        return fallback_path.to_string();
    };
    let Ok(store) = SourceFileRefStore::workspace_default() else {
        return fallback_path.to_string();
    };
    let Ok(record) = store.load(&file_ref) else {
        return fallback_path.to_string();
    };
    record.original_path
}
