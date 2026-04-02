pub const FOUNDATION_TOOL_NAMES: &[&str] = &[
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
    "diagnostics_report_excel_report",
    "diagnostics_report",
    "capacity_assessment",
    "capacity_assessment_from_inventory",
    "ssh_inventory",
    "capacity_assessment_excel_report",
    "linear_regression",
    "logistic_regression",
    "cluster_kmeans",
    "decision_assistant",
];

pub const STOCK_TOOL_NAMES: &[&str] = &[
    // 2026-03-31 CST: 这里把股票能力从通用目录中独立分组，原因是用户已经明确要求底座能力与股票业务域隔离。
    // 目的：让 catalog 在保持平铺兼容输出的同时，也能明确告诉调用方这些能力属于 stock 模块。
    "technical_consultation_basic",
    "security_analysis_contextual",
    "security_analysis_fullstack",
    // 2026-04-02 CST: 这里把 security_decision_briefing 接入 stock 工具目录，原因是统一 briefing 已经成为咨询与投决的共同事实入口；
    // 目的：让 CLI、Skill 和后续 GUI 可以先发现 briefing Tool，再围绕它建立统一路由。
    "security_decision_briefing",
    // 2026-04-02 CST: 这里把 security_committee_vote 加进 stock 目录，原因是正式投决会已经成为 briefing 之后的标准主链，
    // 目的：让 tool_catalog 能显式暴露“briefing -> committee vote”的连续发现路径。
    "security_committee_vote",
    "register_resonance_factor",
    "append_resonance_factor_series",
    "append_resonance_event_tags",
    "sync_template_resonance_factors",
    "bootstrap_resonance_template_factors",
    "evaluate_security_resonance",
    "security_analysis_resonance",
    "record_security_signal_snapshot",
    "backfill_security_signal_outcomes",
    "study_security_signal_analogs",
    "signal_outcome_research_summary",
    "import_stock_price_history",
    "sync_stock_price_history",
];

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
    "technical_consultation_basic",
    "security_analysis_contextual",
    "security_analysis_fullstack",
    // 2026-04-02 CST: 这里把 security_decision_briefing 接入扁平总目录，原因是现有 tool_catalog 契约仍要求返回统一可发现工具列表；
    // 目的：确保 briefing Tool 不仅存在于 stock 分组，也能在顶层目录中被外层调用方稳定发现。
    "security_decision_briefing",
    // 2026-04-02 CST: 这里把 security_committee_vote 补进顶层总目录，原因是 CLI/Skill 依赖统一 catalog 判断可发现性，
    // 目的：保证新 Tool 同时出现在 stock 分组和全局工具清单里。
    "security_committee_vote",
    "register_resonance_factor",
    "append_resonance_factor_series",
    "append_resonance_event_tags",
    "sync_template_resonance_factors",
    "bootstrap_resonance_template_factors",
    "evaluate_security_resonance",
    "security_analysis_resonance",
    "record_security_signal_snapshot",
    "backfill_security_signal_outcomes",
    "study_security_signal_analogs",
    "signal_outcome_research_summary",
    "import_stock_price_history",
    "sync_stock_price_history",
    "diagnostics_report_excel_report",
    "diagnostics_report",
    "capacity_assessment",
    "capacity_assessment_from_inventory",
    "ssh_inventory",
    "capacity_assessment_excel_report",
    "linear_regression",
    "logistic_regression",
    "cluster_kmeans",
    "decision_assistant",
];

pub fn tool_names() -> &'static [&'static str] {
    TOOL_NAMES
}

pub fn foundation_tool_names() -> &'static [&'static str] {
    FOUNDATION_TOOL_NAMES
}

pub fn stock_tool_names() -> &'static [&'static str] {
    STOCK_TOOL_NAMES
}

pub fn is_supported_tool(tool_name: &str) -> bool {
    TOOL_NAMES.contains(&tool_name)
}

pub fn is_stock_tool(tool_name: &str) -> bool {
    STOCK_TOOL_NAMES.contains(&tool_name)
}
