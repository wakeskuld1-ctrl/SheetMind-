use serde_json::{Value, json};

use crate::frame::loader::LoadedTable;
use crate::ops::cast::{CastColumnSpec, cast_column_types, summarize_column_types};
use crate::ops::deduplicate_by_key::{DeduplicateKeep, OrderSpec, deduplicate_by_key};
use crate::ops::derive::{DerivationSpec, derive_columns};
use crate::ops::distinct_rows::{DistinctKeep, distinct_rows};
use crate::ops::fill_lookup::{FillLookupRule, fill_missing_from_lookup_by_keys};
use crate::ops::fill_missing_values::{FillMissingRule, fill_missing_values};
use crate::ops::filter::{FilterCondition, filter_rows};
use crate::ops::format_table_for_export::{ExportFormatOptions, format_table_for_export};
use crate::ops::group::{AggregationSpec, group_and_aggregate};
use crate::ops::lookup_values::{LookupSelect, lookup_values_by_keys};
use crate::ops::normalize_text::{NormalizeTextRule, normalize_text_columns};
use crate::ops::parse_datetime::{ParseDateTimeRule, parse_datetime_columns};
use crate::ops::pivot::{PivotAggregation, pivot_table};
use crate::ops::preview::preview_table;
use crate::ops::rename::{RenameColumnMapping, rename_columns};
use crate::ops::select::select_columns;
use crate::ops::sort::{SortSpec, sort_rows};
use crate::ops::top_n::top_n_rows;
use crate::ops::window::{WindowCalculation, WindowOrderSpec, window_calculation};
use crate::tools::contracts::ToolResponse;
use crate::tools::results;
use crate::tools::sources;

use super::shared::{apply_optional_casts, parse_casts, string_array};

pub(super) fn dispatch_preview_table(args: Value) -> ToolResponse {
    let limit = args
        .get("limit")
        .and_then(|value| value.as_u64())
        .unwrap_or(5) as usize;

    match sources::load_table_for_tool(&args, "preview_table") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => preview_loaded_table(&loaded, limit),
        Err(response) => response,
    }
}

pub(super) fn dispatch_select_columns(args: Value) -> ToolResponse {
    let Some(columns) = args.get("columns").and_then(|value| value.as_array()) else {
        return ToolResponse::error("invalid request parameters");
    };
    let requested_columns = columns
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>();

    match sources::load_table_for_tool(&args, "select_columns") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => {
            match select_columns(&loaded, &requested_columns) {
                Ok(selected) => results::respond_with_result_dataset(
                    "select_columns",
                    &args,
                    &selected,
                    json!({
                        "columns": selected.handle.columns(),
                        "row_count": selected.dataframe.height(),
                    }),
                ),
                Err(error) => ToolResponse::error(error.to_string()),
            }
        }
        Err(response) => response,
    }
}

pub(super) fn dispatch_filter_rows(args: Value) -> ToolResponse {
    let Some(conditions_value) = args.get("conditions") else {
        return ToolResponse::error("invalid request parameters");
    };
    let conditions = match serde_json::from_value::<Vec<FilterCondition>>(conditions_value.clone())
    {
        Ok(conditions) => conditions,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match sources::load_table_for_tool(&args, "filter_rows") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => match filter_rows(&loaded, &conditions) {
            Ok(filtered) => {
                results::respond_with_preview_and_result_ref("filter_rows", &args, &filtered, 5)
            }
            Err(error) => ToolResponse::error(error.to_string()),
        },
        Err(response) => response,
    }
}

pub(super) fn dispatch_cast_column_types(args: Value) -> ToolResponse {
    let Some(casts_value) = args.get("casts") else {
        return ToolResponse::error("invalid request parameters");
    };
    let casts = match serde_json::from_value::<Vec<CastColumnSpec>>(casts_value.clone()) {
        Ok(casts) => casts,
        Err(error) => {
            return ToolResponse::error(format!("request parsing failed: {error}"));
        }
    };

    match sources::load_table_for_tool(&args, "cast_column_types") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => match cast_column_types(&loaded, &casts) {
            Ok(casted) => results::respond_with_result_dataset(
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

pub(super) fn dispatch_derive_columns(args: Value) -> ToolResponse {
    let Some(derivations_value) = args.get("derivations") else {
        return ToolResponse::error("invalid request parameters");
    };
    let derivations = match serde_json::from_value::<Vec<DerivationSpec>>(derivations_value.clone())
    {
        Ok(derivations) => derivations,
        Err(error) => {
            return ToolResponse::error(format!("request parsing failed: {error}"));
        }
    };
    let casts = match parse_casts(&args, "casts", "derive_columns") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match sources::load_table_for_tool(&args, "derive_columns") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => match derive_columns(&prepared_loaded, &derivations) {
                Ok(derived) => results::respond_with_preview_and_result_ref(
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

pub(super) fn dispatch_group_and_aggregate(args: Value) -> ToolResponse {
    let Some(group_by_value) = args.get("group_by").and_then(|value| value.as_array()) else {
        return ToolResponse::error("invalid request parameters");
    };
    let Some(aggregations_value) = args.get("aggregations") else {
        return ToolResponse::error("invalid request parameters");
    };
    let group_by = group_by_value
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>();
    let aggregations =
        match serde_json::from_value::<Vec<AggregationSpec>>(aggregations_value.clone()) {
            Ok(aggregations) => aggregations,
            Err(error) => {
                return ToolResponse::error(format!("request parsing failed: {error}"));
            }
        };
    let casts = match parse_casts(&args, "casts", "group_and_aggregate") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match sources::load_table_for_tool(&args, "group_and_aggregate") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => {
                match group_and_aggregate(&prepared_loaded, &group_by, &aggregations) {
                    Ok(grouped) => results::respond_with_preview_and_result_ref(
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

pub(super) fn dispatch_pivot_table(args: Value) -> ToolResponse {
    let rows = string_array(&args, "rows");
    let columns = string_array(&args, "columns");
    let values = string_array(&args, "values");
    let Some(aggregation_value) = args.get("aggregation") else {
        return ToolResponse::error("invalid request parameters");
    };
    let aggregation = match serde_json::from_value::<PivotAggregation>(aggregation_value.clone()) {
        Ok(aggregation) => aggregation,
        Err(error) => {
            return ToolResponse::error(format!("request parsing failed: {error}"));
        }
    };
    let casts = match parse_casts(&args, "casts", "pivot_table") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match sources::load_table_for_tool(&args, "pivot_table") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => {
                match pivot_table(&prepared_loaded, &rows, &columns, &values, aggregation) {
                    Ok(pivoted) => results::respond_with_preview_and_result_ref(
                        "pivot_table",
                        &args,
                        &pivoted,
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

pub(super) fn dispatch_sort_rows(args: Value) -> ToolResponse {
    let Some(sorts_value) = args.get("sorts") else {
        return ToolResponse::error("invalid request parameters");
    };
    let limit = args
        .get("limit")
        .and_then(|value| value.as_u64())
        .unwrap_or(5) as usize;
    let sorts = match serde_json::from_value::<Vec<SortSpec>>(sorts_value.clone()) {
        Ok(sorts) => sorts,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };
    let casts = match parse_casts(&args, "casts", "sort_rows") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match sources::load_table_for_tool(&args, "sort_rows") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => match sort_rows(&prepared_loaded, &sorts) {
                Ok(sorted) => {
                    results::respond_with_preview_and_result_ref("sort_rows", &args, &sorted, limit)
                }
                Err(error) => ToolResponse::error(error.to_string()),
            },
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

pub(super) fn dispatch_top_n(args: Value) -> ToolResponse {
    let Some(sorts_value) = args.get("sorts") else {
        return ToolResponse::error("invalid request parameters");
    };
    let Some(n) = args.get("n").and_then(|value| value.as_u64()) else {
        return ToolResponse::error("invalid request parameters");
    };
    let sorts = match serde_json::from_value::<Vec<SortSpec>>(sorts_value.clone()) {
        Ok(sorts) => sorts,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };
    let casts = match parse_casts(&args, "casts", "top_n") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match sources::load_table_for_tool(&args, "top_n") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => match top_n_rows(&prepared_loaded, &sorts, n as usize) {
                Ok(top_rows) => results::respond_with_preview_and_result_ref(
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

pub(super) fn dispatch_normalize_text_columns(args: Value) -> ToolResponse {
    let Some(rules_value) = args.get("rules") else {
        return ToolResponse::error("invalid request parameters");
    };
    let rules = match serde_json::from_value::<Vec<NormalizeTextRule>>(rules_value.clone()) {
        Ok(rules) => rules,
        Err(error) => {
            return ToolResponse::error(format!("request parsing failed: {error}"));
        }
    };

    match sources::load_table_for_tool(&args, "normalize_text_columns") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => match normalize_text_columns(&loaded, &rules)
        {
            Ok(normalized) => results::respond_with_preview_and_result_ref(
                "normalize_text_columns",
                &args,
                &normalized,
                5,
            ),
            Err(error) => ToolResponse::error(error.to_string()),
        },
        Err(response) => response,
    }
}

pub(super) fn dispatch_parse_datetime_columns(args: Value) -> ToolResponse {
    let Some(rules_value) = args.get("rules") else {
        return ToolResponse::error("invalid request parameters");
    };
    let rules = match serde_json::from_value::<Vec<ParseDateTimeRule>>(rules_value.clone()) {
        Ok(rules) => rules,
        Err(error) => {
            return ToolResponse::error(format!("request parsing failed: {error}"));
        }
    };

    match sources::load_table_for_tool(&args, "parse_datetime_columns") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => match parse_datetime_columns(&loaded, &rules)
        {
            Ok(parsed) => results::respond_with_preview_and_result_ref(
                "parse_datetime_columns",
                &args,
                &parsed,
                20,
            ),
            Err(error) => ToolResponse::error(error.to_string()),
        },
        Err(response) => response,
    }
}

pub(super) fn dispatch_lookup_values(args: Value) -> ToolResponse {
    let Some(base_value) = args.get("base") else {
        return ToolResponse::error("invalid request parameters");
    };
    let Some(lookup_value) = args.get("lookup") else {
        return ToolResponse::error("invalid request parameters");
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
        return ToolResponse::error("invalid request parameters");
    };
    let selects = match serde_json::from_value::<Vec<LookupSelect>>(selects_value.clone()) {
        Ok(selects) => selects,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match sources::load_nested_table_source(base_value, "lookup_values", "base") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(base_loaded)) => {
            match sources::load_nested_table_source(lookup_value, "lookup_values", "lookup") {
                Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
                Ok(sources::OperationLoad::Loaded(lookup_loaded)) => {
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
                        Ok(looked_up) => results::respond_with_preview_and_result_ref(
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

pub(super) fn dispatch_window_calculation(args: Value) -> ToolResponse {
    let Some(order_by_value) = args.get("order_by") else {
        return ToolResponse::error("invalid request parameters");
    };
    let Some(calculations_value) = args.get("calculations") else {
        return ToolResponse::error("invalid request parameters");
    };
    let partition_by = string_array(&args, "partition_by");
    let order_by = match serde_json::from_value::<Vec<WindowOrderSpec>>(order_by_value.clone()) {
        Ok(order_by) => order_by,
        Err(error) => {
            return ToolResponse::error(format!("request parsing failed: {error}"));
        }
    };
    let calculations =
        match serde_json::from_value::<Vec<WindowCalculation>>(calculations_value.clone()) {
            Ok(calculations) => calculations,
            Err(error) => {
                return ToolResponse::error(format!("request parsing failed: {error}"));
            }
        };
    let casts = match parse_casts(&args, "casts", "window_calculation") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match sources::load_table_for_tool(&args, "window_calculation") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => {
                match window_calculation(&prepared_loaded, &partition_by, &order_by, &calculations)
                {
                    Ok(calculated) => results::respond_with_preview_and_result_ref(
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

pub(super) fn dispatch_rename_columns(args: Value) -> ToolResponse {
    let Some(mappings_value) = args.get("mappings") else {
        return ToolResponse::error("invalid request parameters");
    };
    let mappings = match serde_json::from_value::<Vec<RenameColumnMapping>>(mappings_value.clone())
    {
        Ok(mappings) => mappings,
        Err(error) => {
            return ToolResponse::error(format!("request parsing failed: {error}"));
        }
    };

    match sources::load_table_for_tool(&args, "rename_columns") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => match rename_columns(&loaded, &mappings) {
            Ok(renamed) => {
                results::respond_with_preview_and_result_ref("rename_columns", &args, &renamed, 5)
            }
            Err(error) => ToolResponse::error(error.to_string()),
        },
        Err(response) => response,
    }
}

pub(super) fn dispatch_fill_missing_values(args: Value) -> ToolResponse {
    let Some(rules_value) = args.get("rules") else {
        return ToolResponse::error("invalid request parameters");
    };
    let rules = match serde_json::from_value::<Vec<FillMissingRule>>(rules_value.clone()) {
        Ok(rules) => rules,
        Err(error) => {
            return ToolResponse::error(format!("request parsing failed: {error}"));
        }
    };

    match sources::load_table_for_tool(&args, "fill_missing_values") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => match fill_missing_values(&loaded, &rules) {
            Ok(filled) => results::respond_with_preview_and_result_ref(
                "fill_missing_values",
                &args,
                &filled,
                20,
            ),
            Err(error) => ToolResponse::error(error.to_string()),
        },
        Err(response) => response,
    }
}

pub(super) fn dispatch_distinct_rows(args: Value) -> ToolResponse {
    let subset = string_array(&args, "subset");
    let keep = match args.get("keep") {
        Some(value) => match serde_json::from_value::<DistinctKeep>(value.clone()) {
            Ok(keep) => keep,
            Err(error) => {
                return ToolResponse::error(format!("request parsing failed: {error}"));
            }
        },
        None => DistinctKeep::First,
    };

    match sources::load_table_for_tool(&args, "distinct_rows") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => match distinct_rows(&loaded, &subset, keep) {
            Ok(distincted) => results::respond_with_preview_and_result_ref(
                "distinct_rows",
                &args,
                &distincted,
                20,
            ),
            Err(error) => ToolResponse::error(error.to_string()),
        },
        Err(response) => response,
    }
}

pub(super) fn dispatch_deduplicate_by_key(args: Value) -> ToolResponse {
    let keys = string_array(&args, "keys");
    let order_by = match args.get("order_by") {
        Some(value) => match serde_json::from_value::<Vec<OrderSpec>>(value.clone()) {
            Ok(order_by) => order_by,
            Err(error) => {
                return ToolResponse::error(format!("request parsing failed: {error}"));
            }
        },
        None => Vec::new(),
    };
    let keep = match args.get("keep") {
        Some(value) => match serde_json::from_value::<DeduplicateKeep>(value.clone()) {
            Ok(keep) => keep,
            Err(error) => {
                return ToolResponse::error(format!("request parsing failed: {error}"));
            }
        },
        None => DeduplicateKeep::First,
    };

    match sources::load_table_for_tool(&args, "deduplicate_by_key") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => {
            match deduplicate_by_key(&loaded, &keys, &order_by, keep) {
                Ok(deduplicated) => results::respond_with_preview_and_result_ref(
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

pub(super) fn dispatch_format_table_for_export(args: Value) -> ToolResponse {
    let options = match serde_json::from_value::<ExportFormatOptions>(args.clone()) {
        Ok(options) => options,
        Err(error) => {
            return ToolResponse::error(format!("request parsing failed: {error}"));
        }
    };

    match sources::load_table_for_tool(&args, "format_table_for_export") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => {
            match format_table_for_export(&loaded, &options) {
                Ok(formatted) => results::respond_with_preview_and_result_ref(
                    "format_table_for_export",
                    &args,
                    &formatted,
                    20,
                ),
                Err(error) => ToolResponse::error(error.to_string()),
            }
        }
        Err(response) => response,
    }
}

pub(super) fn dispatch_fill_missing_from_lookup(args: Value) -> ToolResponse {
    let Some(base_value) = args.get("base") else {
        return ToolResponse::error("invalid request parameters");
    };
    let Some(lookup_value) = args.get("lookup") else {
        return ToolResponse::error("invalid request parameters");
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
        return ToolResponse::error("invalid request parameters");
    };
    let fills = match serde_json::from_value::<Vec<FillLookupRule>>(fills_value.clone()) {
        Ok(fills) => fills,
        Err(error) => {
            return ToolResponse::error(format!("request parsing failed: {error}"));
        }
    };

    match sources::load_nested_table_source(base_value, "fill_missing_from_lookup", "base") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(base_loaded)) => {
            match sources::load_nested_table_source(
                lookup_value,
                "fill_missing_from_lookup",
                "lookup",
            ) {
                Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
                Ok(sources::OperationLoad::Loaded(lookup_loaded)) => {
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
                        Ok(filled) => results::respond_with_preview_and_result_ref(
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
            "{tool} cannot accept both {single_field} and {multi_field}"
        )));
    }

    if let Some(value) = single_value {
        if value.trim().is_empty() {
            return Err(ToolResponse::error(format!(
                "{tool} missing required parameter: {single_field}"
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
                "{tool} missing required parameter: {multi_field}"
            )));
        }
        return Ok(keys);
    }

    Err(ToolResponse::error(format!(
        "{tool} missing required parameter: {single_field}"
    )))
}
