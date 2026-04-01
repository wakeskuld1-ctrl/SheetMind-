use serde_json::Value;

use crate::frame::loader::LoadedTable;
use crate::ops::cast::{CastColumnSpec, cast_column_types};
use crate::ops::model_prep::MissingStrategy;
use crate::tools::contracts::ToolResponse;

pub(super) fn string_array<'a>(args: &'a Value, field: &str) -> Vec<&'a str> {
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

pub(super) fn parse_casts(
    args: &Value,
    field: &str,
    _tool: &str,
) -> Result<Vec<CastColumnSpec>, ToolResponse> {
    match args.get(field) {
        Some(casts_value) => serde_json::from_value::<Vec<CastColumnSpec>>(casts_value.clone())
            .map_err(|error| ToolResponse::error(format!("request parsing failed: {error}"))),
        None => Ok(Vec::new()),
    }
}

pub(super) fn parse_missing_strategy(
    args: &Value,
    _tool: &str,
) -> Result<MissingStrategy, ToolResponse> {
    match args.get("missing_strategy") {
        Some(strategy_value) => serde_json::from_value::<MissingStrategy>(strategy_value.clone())
            .map_err(|error| ToolResponse::error(format!("request parsing failed: {error}"))),
        None => Ok(MissingStrategy::DropRows),
    }
}

pub(super) fn apply_optional_casts(
    loaded: LoadedTable,
    casts: &[CastColumnSpec],
) -> Result<LoadedTable, String> {
    if casts.is_empty() {
        Ok(loaded)
    } else {
        cast_column_types(&loaded, casts).map_err(|error| error.to_string())
    }
}
