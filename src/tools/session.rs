use serde_json::{Value, json};

use crate::frame::loader::LoadedTable;
use crate::frame::source_file_ref_store::SourceFileRefStore;
use crate::frame::table_ref_store::PersistedTableRef;
use crate::runtime::local_memory::{
    EventLogInput, LocalMemoryRuntime, SchemaStatus, SessionStage, SessionState, SessionStatePatch,
};
use crate::tools::contracts::ToolResponse;

pub fn build_session_state_response(state: &SessionState) -> Value {
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

pub fn memory_runtime() -> Result<LocalMemoryRuntime, ToolResponse> {
    LocalMemoryRuntime::workspace_default().map_err(|error| ToolResponse::error(error.to_string()))
}

pub fn session_id_from_args(args: &Value) -> String {
    args.get("session_id")
        .and_then(|value| value.as_str())
        .unwrap_or("default")
        .to_string()
}

pub fn user_goal_from_args(args: &Value, fallback: &str) -> String {
    args.get("user_goal")
        .and_then(|value| value.as_str())
        .unwrap_or(fallback)
        .to_string()
}

pub fn selected_columns_from_args(args: &Value) -> Option<Vec<String>> {
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

pub fn sync_confirmed_table_state(
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
                message: Some("绾喛顓荤悰銊ャ仈閸氬骸鍑″┑鈧ú?table_ref".to_string()),
            },
        )
        .map_err(|error| ToolResponse::error(error.to_string()))?;
    Ok(())
}

pub fn sync_loaded_table_state(
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
                message: Some(format!("{tool_name} synced current stage state")),
            },
        )
        .map_err(|error| ToolResponse::error(error.to_string()))?;
    Ok(())
}

pub fn sync_output_handle_state(
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
                message: Some(format!("{tool_name} synced latest {handle_kind}")),
            },
        )
        .map_err(|error| ToolResponse::error(error.to_string()))?;
    Ok(())
}

pub fn current_file_ref_from_args(args: &Value) -> Option<String> {
    args.get("file_ref")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
}

pub fn current_sheet_index_from_args(args: &Value) -> Option<usize> {
    args.get("sheet_index")
        .and_then(|value| value.as_u64())
        .map(|value| value as usize)
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
    "unknown"
}

fn active_handle_ref_from_args(args: &Value) -> Option<String> {
    args.get("result_ref")
        .and_then(|value| value.as_str())
        .or_else(|| args.get("table_ref").and_then(|value| value.as_str()))
        .map(|value| value.to_string())
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
