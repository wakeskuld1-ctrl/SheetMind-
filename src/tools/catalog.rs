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
    // 2026-03-31 CST: 这里把股票能力从通用目录里独立分组，原因是用户已经明确要求底座能力与股票业务域隔离；
    // 目的：让 catalog 在保持平铺兼容输出的同时，也能明确告知调用方这些能力属于 stock 模块。
    "technical_consultation_basic",
    "security_analysis_contextual",
    "security_analysis_fullstack",
    "security_decision_evidence_bundle",
    "security_decision_committee",
    "security_committee_member_agent",
    // 2026-04-09 CST: 这里把主席正式裁决 Tool 暴露到 stock 目录，原因是 Task 1 要把最终正式决议对象单独产品化；
    // 目的：让 CLI / Skill / 后续治理链稳定发现主席线入口。
    "security_chair_resolution",
    // 2026-04-09 CST: 这里把特征快照 Tool 暴露到 stock 目录，原因是 Task 2 要把训练底座对象正式产品化；
    // 目的：让 feature_snapshot 成为可发现、可复用的一等能力。
    "security_feature_snapshot",
    // 2026-04-09 CST: 这里把未来标签回填 Tool 暴露到 stock 目录，原因是 Task 3 要让 forward_outcome 成为正式训练底座入口；
    // 目的：让 CLI / Skill / 回算流水线可以稳定发现 snapshot 绑定的多期限标签能力。
    "security_forward_outcome",
    "security_scorecard_refit",
    // 2026-04-09 CST: 这里把正式 scorecard training Tool 暴露到 stock 目录，原因是 Task 5 需要统一发现训练主链入口；
    // 目的：让 CLI、Skill 与后续回算编排都能稳定发现“训练 -> artifact -> refit”这条正式能力链。
    "security_scorecard_training",
    "security_decision_submit_approval",
    "security_decision_verify_package",
    "security_decision_package_revision",
    // 2026-04-08 CST: 这里把会后结论记录 Tool 暴露到 stock 目录，原因是红测要求 catalog 可发现正式会后治理入口；
    // 目的：让 CLI / Skill / 后续编排都能稳定发现该能力。
    "security_record_post_meeting_conclusion",
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
    "security_decision_evidence_bundle",
    "security_decision_committee",
    "security_committee_member_agent",
    // 2026-04-09 CST: 这里把主席正式裁决 Tool 暴露到总目录，原因是主 dispatcher 与 tool_catalog 仍依赖全量 TOOL_NAMES；
    // 目的：确保主席线成为一等正式能力，而不是只能内部调用的隐藏模块。
    "security_chair_resolution",
    // 2026-04-09 CST: 这里把特征快照 Tool 暴露到总目录，原因是主 dispatcher 与 tool_catalog 仍依赖全量 TOOL_NAMES；
    // 目的：确保训练底座入口从一开始就是主链可发现能力。
    "security_feature_snapshot",
    // 2026-04-09 CST: 这里把未来标签回填 Tool 暴露到总目录，原因是主 dispatcher 的 tool_catalog 仍依赖全量 TOOL_NAMES；
    // 目的：确保 forward_outcome 像 feature_snapshot 一样成为主链可发现的一等能力。
    "security_forward_outcome",
    "security_scorecard_refit",
    // 2026-04-09 CST: 这里把正式 scorecard training Tool 暴露到总目录，原因是主 dispatcher 的 tool_catalog 仍依赖全量 TOOL_NAMES；
    // 目的：确保训练入口和 snapshot/forward_outcome/refit 一样成为主链可发现能力。
    "security_scorecard_training",
    "security_decision_submit_approval",
    "security_decision_verify_package",
    "security_decision_package_revision",
    // 2026-04-08 CST: 这里把会后结论记录 Tool 暴露到总目录，原因是主 dispatcher 仍依赖全量 TOOL_NAMES 做发现；
    // 目的：保持 tool catalog 与实际 dispatcher 能力一致。
    "security_record_post_meeting_conclusion",
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
