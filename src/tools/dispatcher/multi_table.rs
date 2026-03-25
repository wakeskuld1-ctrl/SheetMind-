use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::{Value, json};

use crate::frame::loader::LoadedTable;
use crate::frame::workbook_ref_store::{
    PersistedWorkbookDraft, WorkbookDraftStore, WorkbookSheetInput,
};
use crate::ops::append::append_tables;
use crate::ops::join::{JoinKeepMode, join_tables};
use crate::ops::multi_table_plan::suggest_multi_table_plan;
use crate::ops::table_links::suggest_table_links;
use crate::ops::table_workflow::suggest_table_workflow;
use crate::tools::contracts::ToolResponse;
use crate::tools::results;
use crate::tools::session;
use crate::tools::sources;

use super::shared::{apply_optional_casts, parse_casts};

#[derive(Debug, Deserialize)]
struct ComposeWorkbookWorksheetArg {
    sheet_name: String,
    source: sources::NestedTableSource,
}

#[derive(Debug, Deserialize)]
struct MultiPlanTableInput {
    path: Option<String>,
    sheet: Option<String>,
    table_ref: Option<String>,
    result_ref: Option<String>,
    alias: Option<String>,
}

pub(super) fn dispatch_compose_workbook(args: Value) -> ToolResponse {
    let Some(worksheets_value) = args.get("worksheets") else {
        return ToolResponse::error("compose_workbook 缺少 worksheets 参数");
    };
    let worksheet_args = match serde_json::from_value::<Vec<ComposeWorkbookWorksheetArg>>(
        worksheets_value.clone(),
    ) {
        Ok(worksheet_args) => worksheet_args,
        Err(error) => {
            return ToolResponse::error(format!(
                "compose_workbook request parsing failed: {error}"
            ));
        }
    };

    let mut sheet_inputs = Vec::<WorkbookSheetInput>::with_capacity(worksheet_args.len());
    for worksheet_arg in worksheet_args {
        let loaded = match sources::load_nested_table_source_from_parsed(
            &worksheet_arg.source,
            "compose_workbook",
            &worksheet_arg.sheet_name,
        ) {
            Ok(sources::OperationLoad::NeedsConfirmation(response)) => return response,
            Ok(sources::OperationLoad::Loaded(loaded)) => loaded,
            Err(response) => return response,
        };
        sheet_inputs.push(WorkbookSheetInput {
            sheet_name: worksheet_arg.sheet_name,
            source_refs: sources::source_refs_from_nested_source(&worksheet_arg.source),
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
    if let Err(response) =
        session::sync_output_handle_state(&args, &workbook_ref, "workbook_ref", "compose_workbook")
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

pub(super) fn dispatch_join_tables(args: Value) -> ToolResponse {
    let Some(left_value) = args.get("left") else {
        return ToolResponse::error("join_tables 缺少 left 参数");
    };
    let Some(right_value) = args.get("right") else {
        return ToolResponse::error("invalid request parameters");
    };
    let Some(left_on) = args.get("left_on").and_then(|value| value.as_str()) else {
        return ToolResponse::error("invalid request parameters");
    };
    let Some(right_on) = args.get("right_on").and_then(|value| value.as_str()) else {
        return ToolResponse::error("invalid request parameters");
    };
    let limit = args
        .get("limit")
        .and_then(|value| value.as_u64())
        .unwrap_or(5) as usize;

    let keep_mode = match args.get("keep_mode") {
        Some(mode_value) => match serde_json::from_value::<JoinKeepMode>(mode_value.clone()) {
            Ok(mode) => mode,
            Err(error) => {
                return ToolResponse::error(format!("join_tables request parsing failed: {error}"));
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

    match sources::load_nested_table_source(left_value, "join_tables", "left") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(left_loaded)) => {
            match sources::load_nested_table_source(right_value, "join_tables", "right") {
                Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
                Ok(sources::OperationLoad::Loaded(right_loaded)) => {
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
                                Ok(joined) => results::respond_with_preview_and_result_ref(
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

pub(super) fn dispatch_suggest_table_links(args: Value) -> ToolResponse {
    let Some(left_value) = args.get("left") else {
        return ToolResponse::error("invalid request parameters");
    };
    let Some(right_value) = args.get("right") else {
        return ToolResponse::error("invalid request parameters");
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

    match sources::load_nested_table_source(left_value, "suggest_table_links", "left") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(left_loaded)) => {
            match sources::load_nested_table_source(right_value, "suggest_table_links", "right") {
                Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
                Ok(sources::OperationLoad::Loaded(right_loaded)) => {
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

pub(super) fn dispatch_suggest_table_workflow(args: Value) -> ToolResponse {
    let Some(left_value) = args.get("left") else {
        return ToolResponse::error("invalid request parameters");
    };
    let Some(right_value) = args.get("right") else {
        return ToolResponse::error("invalid request parameters");
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

    match sources::load_nested_table_source(left_value, "suggest_table_workflow", "left") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(left_loaded)) => {
            match sources::load_nested_table_source(right_value, "suggest_table_workflow", "right")
            {
                Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
                Ok(sources::OperationLoad::Loaded(right_loaded)) => {
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

pub(super) fn dispatch_suggest_multi_table_plan(args: Value) -> ToolResponse {
    let Some(tables_value) = args.get("tables") else {
        return ToolResponse::error("invalid request parameters");
    };
    let table_inputs =
        match serde_json::from_value::<Vec<MultiPlanTableInput>>(tables_value.clone()) {
            Ok(inputs) => inputs,
            Err(error) => {
                return ToolResponse::error(format!(
                    "suggest_multi_table_plan request parsing failed: {error}"
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
        let source = sources::NestedTableSource {
            path: input.path,
            sheet: input.sheet,
            file_ref: None,
            sheet_index: None,
            table_ref: input.table_ref,
            result_ref: input.result_ref,
        };
        let source_payload = sources::nested_source_payload(&source);
        source_payloads.insert(table_ref.clone(), source_payload);
        match sources::load_nested_table_source_from_parsed(
            &source,
            "suggest_multi_table_plan",
            "tables",
        ) {
            Ok(sources::OperationLoad::NeedsConfirmation(response)) => return response,
            Ok(sources::OperationLoad::Loaded(loaded)) => loaded_tables.push((table_ref, loaded)),
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

pub(super) fn dispatch_append_tables(args: Value) -> ToolResponse {
    let Some(top_value) = args.get("top") else {
        return ToolResponse::error("invalid request parameters");
    };
    let Some(bottom_value) = args.get("bottom") else {
        return ToolResponse::error("invalid request parameters");
    };
    let limit = args
        .get("limit")
        .and_then(|value| value.as_u64())
        .unwrap_or(5) as usize;

    match sources::load_nested_table_source(top_value, "append_tables", "top") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(top_loaded)) => {
            match sources::load_nested_table_source(bottom_value, "append_tables", "bottom") {
                Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
                Ok(sources::OperationLoad::Loaded(bottom_loaded)) => {
                    match append_tables(&top_loaded, &bottom_loaded) {
                        Ok(appended) => results::respond_with_preview_and_result_ref(
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
            "join_tables" => {
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
