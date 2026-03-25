use serde_json::{Value, json};

use crate::frame::loader::LoadedTable;
use crate::frame::result_ref_store::{PersistedResultDataset, ResultRefStore};
use crate::ops::preview::preview_table;
use crate::tools::contracts::ToolResponse;
use crate::tools::session;

pub(crate) fn respond_with_preview_and_result_ref(
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

pub(crate) fn respond_with_result_dataset(
    tool_name: &str,
    args: &Value,
    loaded: &LoadedTable,
    payload: Value,
) -> ToolResponse {
    let result_ref = match persist_result_dataset(tool_name, args, loaded) {
        Ok(result_ref) => result_ref,
        Err(response) => return response,
    };
    if let Err(response) =
        session::sync_output_handle_state(args, &result_ref, "result_ref", tool_name)
    {
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
