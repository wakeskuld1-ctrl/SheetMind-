use serde::Deserialize;
use serde_json::{Value, json};

use crate::domain::schema::{ConfidenceLevel, HeaderInference, infer_schema_state_label};
use crate::excel::header_inference::infer_header_schema;
use crate::excel::reader::{list_sheets, open_workbook};
use crate::excel::sheet_range::inspect_sheet_range;
use crate::frame::loader::load_confirmed_table;
use crate::frame::region_loader::load_table_region;
use crate::frame::table_ref_store::{PersistedTableRef, TableRefStore};
use crate::frame::workbook_ref_store::WorkbookDraftStore;
use crate::ops::export::{export_csv, export_excel, export_excel_workbook};
use crate::ops::preview::preview_table;
use crate::runtime::local_memory::{EventLogInput, SchemaStatus, SessionStage, SessionStatePatch};
use crate::tools::contracts::ToolResponse;
use crate::tools::results;
use crate::tools::session;
use crate::tools::sources;

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

pub(super) fn dispatch_open_workbook(args: Value) -> ToolResponse {
    let Some(path) = args.get("path").and_then(|value| value.as_str()) else {
        return ToolResponse::error("open_workbook \u{7f3a}\u{5c11} path \u{53c2}\u{6570}");
    };

    match open_workbook(path) {
        Ok(summary) => sources::build_opened_file_response(&args, summary),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_list_sheets(args: Value) -> ToolResponse {
    let Some(path) = args.get("path").and_then(|value| value.as_str()) else {
        // 2026-03-25: 模块化切流后保持与旧 dispatcher 完全一致的参数缺失报错，避免 CLI 回归测试和上层 Skill 文案判断失配。
        return ToolResponse::error("list_sheets 缺少 path 参数");
    };

    match list_sheets(path) {
        Ok(summary) => sources::build_opened_file_response(&args, summary),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_inspect_sheet_range(args: Value) -> ToolResponse {
    let (path, sheet_name) = match sources::resolve_sheet_source(&args, "inspect_sheet_range") {
        Ok(source) => source,
        Err(response) => return response,
    };
    let sample_rows = args
        .get("sample_rows")
        .and_then(|value| value.as_u64())
        .unwrap_or(5) as usize;

    match inspect_sheet_range(&path, &sheet_name, sample_rows) {
        Ok(inspection) => ToolResponse::ok(json!(inspection)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_load_table_region(args: Value) -> ToolResponse {
    let (path, sheet_name) = match sources::resolve_sheet_source(&args, "load_table_region") {
        Ok(source) => source,
        Err(response) => return response,
    };
    let Some(region) = args.get("range").and_then(|value| value.as_str()) else {
        // 2026-03-25: 保留历史稳定报错文案，目的为维持旧链路兼容并锁住 UTF-8 中文断言。
        return ToolResponse::error("load_table_region 缺少 range 参数");
    };
    let header_row_count = args
        .get("header_row_count")
        .and_then(|value| value.as_u64())
        .unwrap_or(1) as usize;

    match load_table_region(&path, &sheet_name, region, header_row_count) {
        Ok(loaded) => {
            let table_ref = TableRefStore::create_table_ref();
            let persisted = match PersistedTableRef::from_region(
                table_ref.clone(),
                &path,
                &sheet_name,
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
            if let Err(response) =
                session::sync_confirmed_table_state(&args, &persisted, "table loaded")
            {
                return response;
            }

            match preview_table(&loaded.dataframe, 5) {
                Ok(preview) => results::respond_with_result_dataset(
                    "load_table_region",
                    &args,
                    &loaded,
                    json!({
                        "path": path,
                        "sheet": sheet_name,
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

pub(super) fn dispatch_normalize_table(args: Value) -> ToolResponse {
    let (path, sheet_name) = match sources::resolve_sheet_source(&args, "normalize_table") {
        Ok(source) => source,
        Err(response) => return response,
    };

    match infer_header_schema(&path, &sheet_name) {
        Ok(inference) => build_inference_response(&sheet_name, inference),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_apply_header_schema(args: Value) -> ToolResponse {
    let (path, sheet_name) = match sources::resolve_sheet_source(&args, "apply_header_schema") {
        Ok(source) => source,
        Err(response) => return response,
    };

    match infer_header_schema(&path, &sheet_name) {
        Ok(inference) => {
            let forced_inference = HeaderInference {
                columns: inference.columns.clone(),
                confidence: ConfidenceLevel::High,
                schema_state: crate::domain::schema::SchemaState::Confirmed,
                header_row_count: inference.header_row_count,
                data_start_row_index: inference.data_start_row_index,
            };

            match load_confirmed_table(&path, &sheet_name, &forced_inference) {
                Ok(loaded) => {
                    let row_count = loaded.dataframe.height();
                    let table_ref = TableRefStore::create_table_ref();
                    let persisted = match PersistedTableRef::from_confirmed_schema(
                        table_ref.clone(),
                        &path,
                        &sheet_name,
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
                        session::sync_confirmed_table_state(&args, &persisted, "schema applied")
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

pub(super) fn dispatch_get_session_state(args: Value) -> ToolResponse {
    let runtime = match session::memory_runtime() {
        Ok(runtime) => runtime,
        Err(response) => return response,
    };
    let session_id = session::session_id_from_args(&args);

    match runtime.get_session_state(&session_id) {
        Ok(state) => ToolResponse::ok(session::build_session_state_response(&state)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_update_session_state(args: Value) -> ToolResponse {
    let payload = match serde_json::from_value::<UpdateSessionStateInput>(args.clone()) {
        Ok(payload) => payload,
        Err(error) => {
            return ToolResponse::error(format!(
                "update_session_state \u{53c2}\u{6570}\u{89e3}\u{6790}\u{5931}\u{8d25}: {error}"
            ));
        }
    };
    let runtime = match session::memory_runtime() {
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
                    message: Some("session state updated".to_string()),
                },
            );
            ToolResponse::ok(session::build_session_state_response(&state))
        }
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

pub(super) fn dispatch_export_csv(args: Value) -> ToolResponse {
    let Some(output_path) = args.get("output_path").and_then(|value| value.as_str()) else {
        return ToolResponse::error("invalid request parameters");
    };

    match sources::load_table_for_tool(&args, "export_csv") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => match export_csv(&loaded, output_path) {
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

pub(super) fn dispatch_export_excel(args: Value) -> ToolResponse {
    let Some(output_path) = args.get("output_path").and_then(|value| value.as_str()) else {
        return ToolResponse::error("invalid request parameters");
    };
    let sheet_name = args
        .get("sheet_name")
        .and_then(|value| value.as_str())
        .unwrap_or("Report");

    match sources::load_table_for_tool(&args, "export_excel") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => {
            match export_excel(&loaded, output_path, sheet_name) {
                Ok(()) => ToolResponse::ok(json!({
                    "output_path": output_path,
                    "sheet_name": sheet_name,
                    "row_count": loaded.dataframe.height(),
                    "column_count": loaded.dataframe.width(),
                    "format": "xlsx",
                })),
                Err(error) => ToolResponse::error(error.to_string()),
            }
        }
        Err(response) => response,
    }
}

pub(super) fn dispatch_export_excel_workbook(args: Value) -> ToolResponse {
    let Some(workbook_ref) = args.get("workbook_ref").and_then(|value| value.as_str()) else {
        return ToolResponse::error("invalid request parameters");
    };
    let Some(output_path) = args.get("output_path").and_then(|value| value.as_str()) else {
        return ToolResponse::error("invalid request parameters");
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
