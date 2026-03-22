use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::{Value, json};

use crate::domain::handles::TableHandle;
use crate::domain::schema::{ConfidenceLevel, HeaderInference, infer_schema_state_label};
use crate::excel::header_inference::infer_header_schema;
use crate::excel::reader::{list_sheets, open_workbook};
use crate::excel::sheet_range::inspect_sheet_range;
use crate::frame::loader::{LoadedTable, load_confirmed_table, load_table_from_table_ref};
use crate::frame::region_loader::load_table_region;
use crate::frame::result_ref_store::{PersistedResultDataset, ResultRefStore};
use crate::frame::source_file_ref_store::{
    PersistedSourceFileRef, SourceFileRefStore,
};
use crate::frame::table_ref_store::{PersistedTableRef, TableRefStore};
use crate::frame::workbook_ref_store::{
    PersistedWorkbookDraft, WorkbookDraftStore, WorkbookSheetInput,
};
use crate::ops::analyze::analyze_table;
use crate::ops::append::append_tables;
use crate::ops::cast::{CastColumnSpec, cast_column_types, summarize_column_types};
use crate::ops::cluster_kmeans::cluster_kmeans;
use crate::ops::decision_assistant::decision_assistant;
use crate::ops::deduplicate_by_key::{DeduplicateKeep, OrderSpec, deduplicate_by_key};
use crate::ops::derive::{DerivationSpec, derive_columns};
use crate::ops::distinct_rows::{DistinctKeep, distinct_rows};
use crate::ops::export::{export_csv, export_excel, export_excel_workbook};
use crate::ops::fill_lookup::{FillLookupRule, fill_missing_from_lookup_by_keys};
use crate::ops::fill_missing_values::{FillMissingRule, fill_missing_values};
use crate::ops::filter::{FilterCondition, filter_rows};
use crate::ops::format_table_for_export::{ExportFormatOptions, format_table_for_export};
use crate::ops::group::{AggregationSpec, group_and_aggregate};
use crate::ops::join::{JoinKeepMode, join_tables};
use crate::ops::linear_regression::linear_regression;
use crate::ops::logistic_regression::logistic_regression;
use crate::ops::lookup_values::{LookupSelect, lookup_values_by_keys};
use crate::ops::model_prep::MissingStrategy;
use crate::ops::multi_table_plan::suggest_multi_table_plan;
use crate::ops::normalize_text::{NormalizeTextRule, normalize_text_columns};
use crate::ops::parse_datetime::{ParseDateTimeRule, parse_datetime_columns};
use crate::ops::pivot::{PivotAggregation, pivot_table};
use crate::ops::preview::preview_table;
use crate::ops::rename::{RenameColumnMapping, rename_columns};
use crate::ops::select::select_columns;
use crate::ops::sort::{SortSpec, sort_rows};
use crate::ops::stat_summary::stat_summary;
use crate::ops::summary::summarize_table;
use crate::ops::table_links::suggest_table_links;
use crate::ops::table_workflow::suggest_table_workflow;
use crate::ops::top_n::top_n_rows;
use crate::ops::window::{WindowCalculation, WindowOrderSpec, window_calculation};
use crate::runtime::local_memory::{
    EventLogInput, LocalMemoryRuntime, SchemaStatus, SessionStage, SessionStatePatch,
};
use crate::tools::contracts::{ToolRequest, ToolResponse};

// 2026-03-22: 这里集中分发 Tool 请求，目的是让 CLI 只负责 JSON 收发，而把具体能力下沉到各自操作层。
pub fn dispatch(request: ToolRequest) -> ToolResponse {
    match request.tool.as_str() {
        "open_workbook" => dispatch_open_workbook(request.args),
        // 2026-03-22: 这里接入独立 list_sheets 入口，目的是把工作簿结构探查从 open_workbook 中显式拆成标准 I/O Tool。
        "list_sheets" => dispatch_list_sheets(request.args),
        "inspect_sheet_range" => dispatch_inspect_sheet_range(request.args),
        "load_table_region" => dispatch_load_table_region(request.args),
        "normalize_table" => dispatch_normalize_table(request.args),
        "apply_header_schema" => dispatch_apply_header_schema(request.args),
        // 2026-03-22: 这里接入会话状态读取入口，目的是让总入口 Skill 能先从本地独立记忆层恢复当前上下文。
        "get_session_state" => dispatch_get_session_state(request.args),
        // 2026-03-22: 这里接入会话状态写入入口，目的是让总入口 Skill 能显式维护当前阶段、目标和激活句柄。
        "update_session_state" => dispatch_update_session_state(request.args),
        "preview_table" => dispatch_preview_table(request.args),
        "select_columns" => dispatch_select_columns(request.args),
        "normalize_text_columns" => dispatch_normalize_text_columns(request.args),
        "rename_columns" => dispatch_rename_columns(request.args),
        "fill_missing_values" => dispatch_fill_missing_values(request.args),
        "distinct_rows" => dispatch_distinct_rows(request.args),
        "deduplicate_by_key" => dispatch_deduplicate_by_key(request.args),
        "format_table_for_export" => dispatch_format_table_for_export(request.args),
        "fill_missing_from_lookup" => dispatch_fill_missing_from_lookup(request.args),
        "parse_datetime_columns" => dispatch_parse_datetime_columns(request.args),
        "lookup_values" => dispatch_lookup_values(request.args),
        "window_calculation" => dispatch_window_calculation(request.args),
        "filter_rows" => dispatch_filter_rows(request.args),
        "cast_column_types" => dispatch_cast_column_types(request.args),
        "derive_columns" => dispatch_derive_columns(request.args),
        "group_and_aggregate" => dispatch_group_and_aggregate(request.args),
        "pivot_table" => dispatch_pivot_table(request.args),
        "sort_rows" => dispatch_sort_rows(request.args),
        "top_n" => dispatch_top_n(request.args),
        "compose_workbook" => dispatch_compose_workbook(request.args),
        "export_csv" => dispatch_export_csv(request.args),
        "export_excel" => dispatch_export_excel(request.args),
        "export_excel_workbook" => dispatch_export_excel_workbook(request.args),
        "join_tables" => dispatch_join_tables(request.args),
        // 2026-03-21: 这里接入显性关联候选分发，目的是让 CLI 先给上层返回“先建议、再执行”的多表入口。
        "suggest_table_links" => dispatch_suggest_table_links(request.args),
        // 2026-03-22: 这里接入多表流程建议分发，目的是先判断更像追加还是关联，再交给上层 Skill 做确认。
        "suggest_table_workflow" => dispatch_suggest_table_workflow(request.args),
        // 2026-03-22: 这里接入多表顺序建议分发，目的是让 CLI 直接返回“先追加、再关联”的保守计划步骤。
        "suggest_multi_table_plan" => dispatch_suggest_multi_table_plan(request.args),
        "append_tables" => dispatch_append_tables(request.args),
        "summarize_table" => dispatch_summarize_table(request.args),
        "analyze_table" => dispatch_analyze_table(request.args),
        "stat_summary" => dispatch_stat_summary(request.args),
        "linear_regression" => dispatch_linear_regression(request.args),
        "logistic_regression" => dispatch_logistic_regression(request.args),
        "cluster_kmeans" => dispatch_cluster_kmeans(request.args),
        "decision_assistant" => dispatch_decision_assistant(request.args),
        _ => ToolResponse::error(format!("暂不支持的 tool: {}", request.tool)),
    }
}

fn dispatch_open_workbook(args: Value) -> ToolResponse {
    let Some(path) = args.get("path").and_then(|value| value.as_str()) else {
        // 2026-03-23: 这里收口 open_workbook 缺参报错，原因是历史乱码导致 UTF-8 文案回归失败；目的是保证 CLI 对普通用户输出稳定可读的中文提示。
        return ToolResponse::error("open_workbook 缺少 path 参数");
    };

    match open_workbook(path) {
        // 2026-03-23: ???????????? file_ref??????????????? Sheet??????????? Sheet ????????????????? path/sheet ?????
        Ok(summary) => build_opened_file_response(&args, summary),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

fn dispatch_list_sheets(args: Value) -> ToolResponse {
    let Some(path) = args.get("path").and_then(|value| value.as_str()) else {
        return ToolResponse::error("list_sheets ?? path ??");
    };

    match list_sheets(path) {
        // 2026-03-23: ??? list_sheets ? open_workbook ????? file_ref ???????????????????? Sheet?????????????????????????????????
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
        return ToolResponse::error("load_table_region ?? range ??");
    };
    let header_row_count = args
        .get("header_row_count")
        .and_then(|value| value.as_u64())
        .unwrap_or(1) as usize;

    match load_table_region(&source.path, &source.sheet_name, region, header_row_count) {
        Ok(loaded) => {
            let table_ref = TableRefStore::create_table_ref();
            // 2026-03-22: ?????????????? table_ref???????????????? preview/??????
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
            if let Err(response) = sync_confirmed_table_state(
                &args,
                &persisted,
                "?????????????????",
            ) {
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
            // 2026-03-22: ?????????? confirmed??????????? table_ref ?????????????
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
                    if let Err(response) = sync_confirmed_table_state(
                        &args,
                        &persisted,
                        "?????????????",
                    ) {
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

// 2026-03-23: 这里补回会话状态更新入参结构，原因是编译阻塞暴露出该定义缺失；目的是恢复 update_session_state 分发链路并让后续测试能继续推进。
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
        active_table_ref: payload.active_handle_ref.or(payload.active_table_ref),
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

// 2026-03-22: 这里为会话状态补充激活句柄摘要，目的是在保留 active_table_ref 兼容字段的同时，对上层 Skill 暴露更清晰的激活上下文。
fn build_session_state_response(state: &crate::runtime::local_memory::SessionState) -> Value {
    let mut payload = json!(state);
    if let Some(object) = payload.as_object_mut() {
        let active_handle_ref = state.active_table_ref.clone();
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
                    json!({
                        "ref": reference,
                        "kind": classify_handle_kind(reference),
                    })
                })
                .unwrap_or(Value::Null),
        );
    }
    payload
}

// 2026-03-22: 这里按句柄前缀推断激活对象类型，目的是让 table_ref/result_ref/workbook_ref 在会话摘要里一眼可辨。
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
        Err(error) => return ToolResponse::error(format!("filter_rows 条件解析失败: {error}")),
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

// 2026-03-23: 这里接入透视分发，目的是把 Excel 用户熟悉的 pivot 能力纳入现有单表 Tool 链式体验。
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

// 2026-03-22: 这里接入 workbook 草稿组装分发，目的是把多张表快照装配成一个可复用的多 Sheet 交付句柄。
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

// 2026-03-22: 这里接入多 Sheet workbook 导出分发，目的是让 compose_workbook 产出的 workbook_ref 可以真正落成 xlsx 文件。
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

    // 2026-03-22: 这里把 join 的左右输入统一收口到嵌套来源解析器，目的是让显性关联既能吃原始表，也能直接吃 table_ref/result_ref。
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

// 2026-03-23: 这里接入文本标准化分发，目的是把 join / lookup 前的文本清洗沉淀成统一单表 Tool 入口。
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

// 2026-03-23: 这里接入日期时间标准化分发，目的是让时间列清洗继续沿用现有 result_ref 链式复用体验。
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

// 2026-03-23: 这里接入轻量查值分发，目的是让主表不变行的带列场景沿用现有双来源与 result_ref 链式体验。
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
        Err(error) => return ToolResponse::error(format!("lookup_values 参数解析失败: {error}")),
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

// 2026-03-23: 这里接入窗口计算分发，目的是让组内序号、排名和累计值沿用单表 Tool 的统一输入与 result_ref 体验。
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

// 2026-03-23: 这里接入列改名分发，目的是让字段口径统一继续沿用现有 result_ref 链式复用体验。
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

// 2026-03-22: 这里接入通用补空分发，目的是让常量补空、补零和前值填补沿用现有单表 Tool 链式体验。
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

// 2026-03-22: 这里接入通用去重分发，目的是让整行去重和按子集列去重沿用现有单表 Tool 的链式体验。
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

// 2026-03-22: 这里接入按业务键去重分发，目的是让“先按排序规则选最新/最早记录”的真实业务清洗场景进入统一 Tool 入口。
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

// 2026-03-22: 这里接入导出前整理分发，目的是让列顺序、表头别名和输出裁剪沿用现有单表 Tool 的链式体验。
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

// 2026-03-23: 这里接入 lookup 回填分发，目的是让 base / lookup 双来源场景也能复用统一 JSON Tool 入口。
fn dispatch_fill_missing_from_lookup(args: Value) -> ToolResponse {
    let Some(base_value) = args.get("base") else {
        return ToolResponse::error("fill_missing_from_lookup 缺少 base 参数");
    };
    let Some(lookup_value) = args.get("lookup") else {
        return ToolResponse::error("fill_missing_from_lookup 缺少 lookup 参数");
    };
    let base_keys = match parse_lookup_key_args(
        &args,
        "base_on",
        "base_keys",
        "fill_missing_from_lookup",
    ) {
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
    // 2026-03-21: 这里限制候选数量可配置，目的是让问答界面先看最稳的少量建议，避免一次返回过多噪声。
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

    // 2026-03-23: 这里把关系建议层也升级为嵌套来源输入，目的是让显性关联候选可以直接消费 table_ref 和 result_ref。
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
    // 2026-03-22: 这里限制最多返回多少个关联候选，目的是让工作流层输出保持精简，优先呈现最稳的少量建议。
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

    // 2026-03-23: 这里把工作流建议层也升级为嵌套来源输入，目的是让建议调用骨架能承接上游句柄而不退化回原始路径。
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
                                    // 2026-03-23: 这里把建议调用中的来源骨架改回用户原始输入，目的是避免 table_ref/result_ref 在建议层退化回 path+sheet。
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

fn dispatch_suggest_multi_table_plan(args: Value) -> ToolResponse {
    #[derive(Debug, Deserialize)]
    struct MultiPlanTableInput {
        path: Option<String>,
        sheet: Option<String>,
        table_ref: Option<String>,
        result_ref: Option<String>,
        alias: Option<String>,
    }

    let Some(tables_value) = args.get("tables") else {
        return ToolResponse::error("suggest_multi_table_plan 缺少 tables 参数");
    };
    let table_inputs =
        match serde_json::from_value::<Vec<MultiPlanTableInput>>(tables_value.clone()) {
            Ok(inputs) => inputs,
            Err(error) => {
                return ToolResponse::error(format!(
                    "suggest_multi_table_plan 的 tables 参数解析失败: {error}"
                ));
            }
        };
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
            // 2026-03-23: 这里显式补齐 file_ref/sheet_index 缺省值，原因是 NestedTableSource 已扩展新入口；目的是保持旧的多表计划输入仍可稳定构造。
            file_ref: None,
            sheet_index: None,
            table_ref: input.table_ref,
            result_ref: input.result_ref,
        };
        let source_payload = nested_source_payload(&source);
        // 2026-03-23: 这里把多表计划器的每个原始输入来源先缓存下来，目的是后面把建议调用骨架恢复成用户最初传入的来源类型。
        source_payloads.insert(table_ref.clone(), source_payload);
        match load_nested_table_source_from_parsed(&source, "suggest_multi_table_plan", "tables") {
            Ok(OperationLoad::NeedsConfirmation(response)) => return response,
            Ok(OperationLoad::Loaded(loaded)) => loaded_tables.push((table_ref, loaded)),
            Err(response) => return response,
        }
    }

    match suggest_multi_table_plan(loaded_tables, max_link_candidates) {
        Ok(result) => {
            let mut payload = json!(result);
            rewrite_multi_table_plan_suggested_tool_call_sources(&mut payload, &source_payloads);
            ToolResponse::ok(payload)
        }
        Err(error) => ToolResponse::error(error.to_string()),
    }
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

    // 2026-03-22: 这里把 append 的上下输入统一收口到嵌套来源解析器，目的是让“先追加再关联”链路能直接消费上一份 result_ref。
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
                            "鏌ョ湅缁熻鎽樿",
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
                            "鏌ョ湅缁熻鎽樿",
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
                        "鎵ц閫昏緫鍥炲綊",
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
    // 2026-03-23: 这里新增 file_ref，原因是后续多表场景也可能需要复用“已打开文件 + 第几个 Sheet”；目的是让嵌套来源与顶层来源保持同一套稳妥入口。
    file_ref: Option<String>,
    // 2026-03-23: 这里新增 sheet_index，原因是 file_ref 模式下不能再要求上层重复传 Sheet 名；目的是让多表来源也能按“第几个 Sheet”继续。
    sheet_index: Option<usize>,
    table_ref: Option<String>,
    result_ref: Option<String>,
}

// 2026-03-23: 这里定义单表来源解析结果，原因是 dispatcher 需要把 path+sheet 与 file_ref+sheet_index 两套入口统一到同一内部结构；目的是新增新入口时不破坏旧分支。
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

// 2026-03-22: 这里为 join/append 等多表 Tool 统一解析 path、table_ref、result_ref 三种来源。
fn load_nested_table_source(
    value: &Value,
    tool: &str,
    field_name: &str,
) -> Result<OperationLoad, ToolResponse> {
    let source = parse_nested_table_source(value, tool, field_name)?;
    load_nested_table_source_from_parsed(&source, tool, field_name)
}

// 2026-03-23: 这里把嵌套来源解析拆成独立步骤，目的是让“先加载”与“保留原始来源骨架”两类需求复用同一套参数校验。
fn parse_nested_table_source(
    value: &Value,
    tool: &str,
    field_name: &str,
) -> Result<NestedTableSource, ToolResponse> {
    serde_json::from_value::<NestedTableSource>(value.clone()).map_err(|error| {
        ToolResponse::error(format!("{tool} 的 {field_name} 参数解析失败: {error}"))
    })
}

// 2026-03-23: 这里复用已解析来源继续装载表，目的是让多表计划器既能读取数据，也能同步拿到原始来源定义。
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
            "{tool} 的 {field_name} 需要提供 path+sheet、file_ref+sheet_index、table_ref 或 result_ref"
        ))),
    }
}

// 2026-03-23: 这里把来源定义重新压成最小 JSON 骨架，目的是让建议调用能原样保留用户输入的来源类型而不混入 alias 等计划字段。
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

// 2026-03-22: 这里把单个嵌套来源翻译成血缘引用列表，目的是让 workbook 草稿也能保留每张 sheet 的上游来源说明。
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

// 2026-03-22: 这里为分析建模层统一接入双入口，目的是让 Tool 同时兼容旧的 path+sheet 和新的 table_ref。
fn load_table_for_analysis(args: &Value, tool: &str) -> Result<OperationLoad, ToolResponse> {
    load_table_for_tool(args, tool)
}

// 2026-03-22: 这里为单表类 Tool 统一接入三种输入，目的是让表处理层和分析层都能复用 path+sheet、table_ref、result_ref。
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

// 2026-03-22: 这里从 table_ref 恢复持久化确认态，目的是让分析建模层跳过重复 schema 推断。
fn load_table_from_ref(table_ref: &str) -> Result<OperationLoad, String> {
    let store = TableRefStore::workspace_default().map_err(|error| error.to_string())?;
    let persisted = store.load(table_ref).map_err(|error| error.to_string())?;
    load_table_from_table_ref(&persisted)
        .map(OperationLoad::Loaded)
        .map_err(|error| error.to_string())
}

// 2026-03-22: 这里从 result_ref 恢复中间结果集，目的是让跨请求链式分析可以直接消费上一步结果而不必回退到原始 Excel。
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

// 2026-03-23: 这里把 open_workbook/list_sheets 成功结果统一升级成带 file_ref 的响应，原因是后续流程需要按“第几个 Sheet”继续；目的是在不移除旧字段的前提下增量补齐稳妥入口。
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

// 2026-03-23: 这里统一解析单表来源，原因是顶层 Tool 需要同时兼容 path+sheet 与 file_ref+sheet_index 两套入口；目的是新增稳妥入口时不破坏任何旧调用。
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

// 2026-03-23: 这里按 file_ref + sheet_index 恢复真实单表来源，原因是外层链路不稳定传递中文 Sheet 名；目的是把中文名称恢复留在 Rust 进程内部完成。
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

// 2026-03-22: 这里为会产生新表的单表 Tool 统一附加 result_ref，目的是让用户能直接把当前结果接到下一步分析里。
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

// 2026-03-22: 这里统一把结果表落成 result_ref 并回填到 JSON 响应，目的是把“看结果”和“继续复用结果”合并成同一次调用体验。
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

// 2026-03-23: 这里兼容单键与复合键参数，目的是让旧的 `*_on` 调用不破坏，同时允许新链路显式传多列键。
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
            "{tool} 不要同时传 {single_field} 和 {multi_field}"
        )));
    }

    if let Some(value) = single_value {
        if value.trim().is_empty() {
            return Err(ToolResponse::error(format!("{tool} 缺少 {single_field} 参数")));
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
            return Err(ToolResponse::error(format!("{tool} 缺少 {multi_field} 参数")));
        }
        return Ok(keys);
    }

    Err(ToolResponse::error(format!("{tool} 缺少 {single_field} 参数")))
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

// 2026-03-22: 这里统一持久化中间结果，目的是让表处理输出可以不依赖原始 Excel 再次读取就进入下一步分析。
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

// 2026-03-22: 这里统一抽取当前输入来源，目的是让 result_ref 能把单表和多表链路里的所有上游句柄都记完整。
fn source_refs_from_args(args: &Value) -> Vec<String> {
    let mut refs = Vec::<String>::new();
    collect_source_refs(args, &mut refs);
    refs
}

// 2026-03-22: 这里递归抽取请求里的来源句柄，目的是让 join/append 生成的新结果也能保留左右两边乃至混合来源的血缘。
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

// 2026-03-22: 这里做顺序保留去重，目的是让 source_refs 可读且稳定，不会因为同一个来源重复出现而污染血缘展示。
fn push_unique_source_ref(refs: &mut Vec<String>, candidate: String) {
    if !refs.iter().any(|existing| existing == &candidate) {
        refs.push(candidate);
    }
}

// 2026-03-23: 这里回填两表工作流建议里的来源骨架，目的是让上层直接执行建议调用时继续沿用 table_ref/result_ref，而不是回退到原始路径。
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

// 2026-03-23: 这里回填多表计划器每一步的来源骨架，目的是让计划里的 suggested_tool_call 能保留原始来源类型并继续引用 step_n_result。
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
            "join_tables" => {
                suggested_args.insert("left".to_string(), first_source);
                suggested_args.insert("right".to_string(), second_source);
            }
            _ => {}
        }
    }
}

// 2026-03-23: 这里把 alias 和 step_n_result 统一翻译成实际来源骨架，目的是让多表计划步骤既能回指原表，也能回指前一步结果。
fn planned_source_payload(reference: &str, source_payloads: &BTreeMap<String, Value>) -> Value {
    source_payloads
        .get(reference)
        .cloned()
        .unwrap_or_else(|| json!({ "result_ref": reference }))
}

// 2026-03-22: 这里统一创建本地记忆层入口，目的是让 dispatcher 所有会话状态读写共享同一套路径解析与错误出口。
fn memory_runtime() -> Result<LocalMemoryRuntime, ToolResponse> {
    LocalMemoryRuntime::workspace_default().map_err(|error| ToolResponse::error(error.to_string()))
}

// 2026-03-22: 这里统一解析 session_id，目的是让显式多会话和默认单会话两种模式都能复用同一套入口。
fn session_id_from_args(args: &Value) -> String {
    args.get("session_id")
        .and_then(|value| value.as_str())
        .unwrap_or("default")
        .to_string()
}

// 2026-03-22: 这里统一获取用户目标兜底值，目的是让总入口显式写入和关键 Tool 自动同步都能保留可解释的目标摘要。
fn user_goal_from_args(args: &Value, fallback: &str) -> String {
    args.get("user_goal")
        .and_then(|value| value.as_str())
        .unwrap_or(fallback)
        .to_string()
}

// 2026-03-22: 这里统一提取当前请求的关心列，目的是让分析建模层和决策层都能把列上下文同步到本地记忆层。
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

// 2026-03-22: 这里集中同步确认态 table_ref 与会话摘要，目的是把表处理层建立的 confirmed 状态直接沉淀到本地独立记忆层。
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

// 2026-03-22: 这里集中同步分析/决策层的会话状态，目的是让总入口下一轮能直接判断用户已经处在哪个层级。
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
                active_table_ref: active_handle_ref_from_args(args),
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

// 2026-03-22: 这里统一抽取当前激活句柄，目的是让会话状态至少能记住本轮是从 table_ref 还是 result_ref 继续下来的。
fn active_handle_ref_from_args(args: &Value) -> Option<String> {
    args.get("result_ref")
        .and_then(|value| value.as_str())
        .or_else(|| args.get("table_ref").and_then(|value| value.as_str()))
        .map(|value| value.to_string())
}

// 2026-03-23: ?????? file_ref?????? Tool ??????????????????????????????????????
fn current_file_ref_from_args(args: &Value) -> Option<String> {
    args.get("file_ref")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
}

// 2026-03-23: ????????? Sheet???? file_ref ?????????????????????????????? Sheet ???
fn current_sheet_index_from_args(args: &Value) -> Option<usize> {
    args.get("sheet_index")
        .and_then(|value| value.as_u64())
        .map(|value| value as usize)
}

// 2026-03-23: ????????????????????????????????? ASCII ??????????????????????????
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

