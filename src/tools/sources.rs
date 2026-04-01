use serde::Deserialize;
use serde_json::{Value, json};

use crate::domain::handles::TableHandle;
use crate::domain::schema::{ConfidenceLevel, HeaderInference, infer_schema_state_label};
use crate::excel::header_inference::infer_header_schema;
use crate::excel::reader::WorkbookSummary;
use crate::frame::loader::{LoadedTable, load_confirmed_table, load_table_from_table_ref};
use crate::frame::result_ref_store::ResultRefStore;
use crate::frame::source_file_ref_store::{PersistedSourceFileRef, SourceFileRefStore};
use crate::frame::table_ref_store::TableRefStore;
use crate::tools::contracts::ToolResponse;

pub(crate) enum OperationLoad {
    NeedsConfirmation(ToolResponse),
    Loaded(LoadedTable),
}

#[derive(Debug, Deserialize)]
pub(crate) struct NestedTableSource {
    pub(crate) path: Option<String>,
    pub(crate) sheet: Option<String>,
    pub(crate) file_ref: Option<String>,
    pub(crate) sheet_index: Option<usize>,
    pub(crate) table_ref: Option<String>,
    pub(crate) result_ref: Option<String>,
}

struct ResolvedSheetSource {
    path: String,
    sheet_name: String,
}

pub(crate) fn load_nested_table_source(
    value: &Value,
    tool: &str,
    field_name: &str,
) -> Result<OperationLoad, ToolResponse> {
    let source = parse_nested_table_source(value, tool, field_name)?;
    load_nested_table_source_from_parsed(&source, tool, field_name)
}

pub(crate) fn parse_nested_table_source(
    value: &Value,
    tool: &str,
    field_name: &str,
) -> Result<NestedTableSource, ToolResponse> {
    serde_json::from_value::<NestedTableSource>(value.clone()).map_err(|error| {
        ToolResponse::error(format!("{tool} 的 {field_name} 参数解析失败: {error}"))
    })
}

pub(crate) fn load_nested_table_source_from_parsed(
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

pub(crate) fn nested_source_payload(source: &NestedTableSource) -> Value {
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

pub(crate) fn source_refs_from_nested_source(source: &NestedTableSource) -> Vec<String> {
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

pub(crate) fn load_table_for_analysis(
    args: &Value,
    tool: &str,
) -> Result<OperationLoad, ToolResponse> {
    load_table_for_tool(args, tool)
}

pub(crate) fn load_table_for_tool(args: &Value, tool: &str) -> Result<OperationLoad, ToolResponse> {
    if let Some(result_ref) = args.get("result_ref").and_then(|value| value.as_str()) {
        return load_result_from_ref(result_ref).map_err(ToolResponse::error);
    }

    if let Some(table_ref) = args.get("table_ref").and_then(|value| value.as_str()) {
        return load_table_from_ref(table_ref).map_err(ToolResponse::error);
    }

    let (path, sheet_name) = resolve_sheet_source(args, tool)?;
    load_sheet_for_operation(&path, &sheet_name).map_err(ToolResponse::error)
}

pub(crate) fn build_opened_file_response(args: &Value, summary: WorkbookSummary) -> ToolResponse {
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

pub(crate) fn resolve_sheet_source(
    args: &Value,
    tool: &str,
) -> Result<(String, String), ToolResponse> {
    let resolved = if let Some(file_ref) = args.get("file_ref").and_then(|value| value.as_str()) {
        let Some(sheet_index) = args.get("sheet_index").and_then(|value| value.as_u64()) else {
            return Err(ToolResponse::error(format!(
                "{tool} 缺少 sheet_index 参数，或请改传 path + sheet"
            )));
        };
        resolve_sheet_source_from_file_ref(file_ref, sheet_index as usize)
            .map_err(ToolResponse::error)?
    } else {
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
        ResolvedSheetSource {
            path: path.to_string(),
            sheet_name: sheet.to_string(),
        }
    };

    Ok((resolved.path, resolved.sheet_name))
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
