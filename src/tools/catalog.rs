pub const TOOL_NAMES: &[&str] = &[
    "license_activate",
    "license_status",
    "license_deactivate",
    "open_workbook",
    "list_sheets",
    "inspect_sheet_range",
    "load_table_region",
    "normalize_table",
    "apply_header_schema",
    "get_session_state",
    "update_session_state",
    "preview_table",
    "select_columns",
    "normalize_text_columns",
    "rename_columns",
    "fill_missing_values",
    "distinct_rows",
    "deduplicate_by_key",
    "format_table_for_export",
    "fill_missing_from_lookup",
    "parse_datetime_columns",
    "lookup_values",
    "window_calculation",
    "filter_rows",
    "cast_column_types",
    "derive_columns",
    "group_and_aggregate",
    "pivot_table",
    "sort_rows",
    "top_n",
    "compose_workbook",
    "report_delivery",
    "build_chart",
    "export_chart_image",
    "export_csv",
    "export_excel",
    "export_excel_workbook",
    "join_tables",
    "suggest_table_links",
    "suggest_table_workflow",
    "suggest_multi_table_plan",
    "append_tables",
    "summarize_table",
    "analyze_table",
    "stat_summary",
    "correlation_analysis",
    "outlier_detection",
    "distribution_analysis",
    "trend_analysis",
    // 2026-03-28 CST: 这里注册股票技术面基础 Tool，原因是后续 CLI、AI 和 Skill 必须先能发现这条 Rust / exe 主线能力；
    // 目的：把 `technical_consultation_basic` 正式暴露到统一 catalog，而不是只停留在内部模块。
    "technical_consultation_basic",
    // 2026-03-28 CST: 这里注册股票历史 CSV 导入 Tool，原因是后续 Skill / CLI 必须先能发现这个 SQLite 落库入口；
    // 目的：把股票历史主线能力正式暴露到统一 catalog，而不是只停留在内部模块。
    "import_stock_price_history",
    // 2026-03-29 CST: 这里注册股票历史 HTTP 同步 Tool，原因是方案 2+3 已确认要把腾讯/新浪并入同一 SQLite 主线；
    // 目的：让 CLI、Skill 和后续自动化都能正式发现 `sync_stock_price_history`。
    "sync_stock_price_history",
    // 2026-03-29 00:08 CST：这里注册组合诊断 Excel 报表 Tool，原因是 workbook-first 交付入口也必须先被 catalog 发现；
    // 目的：让 CLI 和后续 AI 能直接调用“一步出 workbook/xlsx”的高层诊断交付能力。
    "diagnostics_report_excel_report",
    // 2026-03-28 23:54 CST: 这里注册统计诊断组合 Tool，原因是新的高层诊断入口必须先被 catalog 发现；
    // 目的是让 CLI、Skill 和后续交付层都能直接发现“统一组合诊断”能力。
    "diagnostics_report",
    // 2026-03-28 10:42 CST: 这里注册容量评估 Tool，原因是测试和 CLI 入口都要求新场景能被发现；目的是避免只实现底层算子却忘记对外暴露能力。
    "capacity_assessment",
    // 2026-03-28 16:55 CST: 这里注册容量桥接 Tool，原因是要让 SSH 盘点结果可以直接收口成容量分析；目的是减少调用方手工拼装 inventory_evidence。
    "capacity_assessment_from_inventory",
    // 2026-03-28 16:12 CST: 这里注册受限 SSH 盘点 Tool，原因是测试和容量链路都需要发现这个安全入口；目的是把只读白名单采集能力正式暴露给 CLI 和编排层。
    "ssh_inventory",
    // 2026-03-28 22:19 CST: 这里注册容量评估 Excel 报表 Tool，原因是本轮要把容量结论真正落成 Excel 交付；
    // 目的是让目录层直接暴露“一步出 workbook/xlsx”的高层能力。
    "capacity_assessment_excel_report",
    "linear_regression",
    "logistic_regression",
    "cluster_kmeans",
    "decision_assistant",
];

pub fn tool_names() -> &'static [&'static str] {
    TOOL_NAMES
}

pub fn is_supported_tool(tool_name: &str) -> bool {
    TOOL_NAMES.contains(&tool_name)
}
