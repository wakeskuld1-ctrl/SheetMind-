use crate::tools::catalog;
use crate::tools::contracts::{ToolRequest, ToolResponse};

mod analysis_ops;
mod multi_table;
mod shared;
mod single_table;
mod workbook_io;

pub fn dispatch(request: ToolRequest) -> ToolResponse {
    if !catalog::is_supported_tool(request.tool.as_str()) {
        return ToolResponse::error(format!("unsupported tool: {}", request.tool));
    }

    match request.tool.as_str() {
        "open_workbook" => workbook_io::dispatch_open_workbook(request.args),
        "list_sheets" => workbook_io::dispatch_list_sheets(request.args),
        "inspect_sheet_range" => workbook_io::dispatch_inspect_sheet_range(request.args),
        "load_table_region" => workbook_io::dispatch_load_table_region(request.args),
        "normalize_table" => workbook_io::dispatch_normalize_table(request.args),
        "apply_header_schema" => workbook_io::dispatch_apply_header_schema(request.args),
        "get_session_state" => workbook_io::dispatch_get_session_state(request.args),
        "update_session_state" => workbook_io::dispatch_update_session_state(request.args),
        "preview_table" => single_table::dispatch_preview_table(request.args),
        "select_columns" => single_table::dispatch_select_columns(request.args),
        "normalize_text_columns" => single_table::dispatch_normalize_text_columns(request.args),
        "rename_columns" => single_table::dispatch_rename_columns(request.args),
        "fill_missing_values" => single_table::dispatch_fill_missing_values(request.args),
        "distinct_rows" => single_table::dispatch_distinct_rows(request.args),
        "deduplicate_by_key" => single_table::dispatch_deduplicate_by_key(request.args),
        "format_table_for_export" => single_table::dispatch_format_table_for_export(request.args),
        "fill_missing_from_lookup" => single_table::dispatch_fill_missing_from_lookup(request.args),
        "parse_datetime_columns" => single_table::dispatch_parse_datetime_columns(request.args),
        "lookup_values" => single_table::dispatch_lookup_values(request.args),
        "window_calculation" => single_table::dispatch_window_calculation(request.args),
        "filter_rows" => single_table::dispatch_filter_rows(request.args),
        "cast_column_types" => single_table::dispatch_cast_column_types(request.args),
        "derive_columns" => single_table::dispatch_derive_columns(request.args),
        "group_and_aggregate" => single_table::dispatch_group_and_aggregate(request.args),
        "pivot_table" => single_table::dispatch_pivot_table(request.args),
        "sort_rows" => single_table::dispatch_sort_rows(request.args),
        "top_n" => single_table::dispatch_top_n(request.args),
        "compose_workbook" => multi_table::dispatch_compose_workbook(request.args),
        "export_csv" => workbook_io::dispatch_export_csv(request.args),
        "export_excel" => workbook_io::dispatch_export_excel(request.args),
        "export_excel_workbook" => workbook_io::dispatch_export_excel_workbook(request.args),
        "join_tables" => multi_table::dispatch_join_tables(request.args),
        "suggest_table_links" => multi_table::dispatch_suggest_table_links(request.args),
        "suggest_table_workflow" => multi_table::dispatch_suggest_table_workflow(request.args),
        "suggest_multi_table_plan" => multi_table::dispatch_suggest_multi_table_plan(request.args),
        "append_tables" => multi_table::dispatch_append_tables(request.args),
        "summarize_table" => analysis_ops::dispatch_summarize_table(request.args),
        "analyze_table" => analysis_ops::dispatch_analyze_table(request.args),
        "stat_summary" => analysis_ops::dispatch_stat_summary(request.args),
        "linear_regression" => analysis_ops::dispatch_linear_regression(request.args),
        "logistic_regression" => analysis_ops::dispatch_logistic_regression(request.args),
        "cluster_kmeans" => analysis_ops::dispatch_cluster_kmeans(request.args),
        "decision_assistant" => analysis_ops::dispatch_decision_assistant(request.args),
        _ => unreachable!("registered tool must be routed by dispatch"),
    }
}
