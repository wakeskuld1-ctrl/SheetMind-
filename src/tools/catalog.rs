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
    // 2026-04-12 CST: Expose the formal condition-review tool, because P8
    // needs intraperiod review to become discoverable before execution and replay tools build on it.
    // Purpose: make lifecycle review a public stock capability instead of a hidden helper.
    "security_condition_review",
    // 2026-04-12 CST: Expose the formal execution-record tool, because P8
    // needs execution lifecycle events to become public stock capabilities.
    // Purpose: make execution events discoverable before package and review wiring expands.
    "security_execution_record",
    // 2026-04-12 CST: Expose the formal post-trade review tool, because P8
    // needs replayable review closure after execution events are recorded.
    // Purpose: make layered post-trade review a public stock capability.
    "security_post_trade_review",
    // 2026-04-11 CST: Expose the dated external-proxy backfill tool in the stock
    // catalog, because P4 needs governed proxy history writes to become a formal
    // discoverable capability before training and replay can depend on them.
    // Purpose: make CLI and Skill flows find proxy-backfill without hidden wiring.
    "security_external_proxy_backfill",
    // 2026-04-12 CST: Expose the file-based proxy-history import tool, because
    // Historical Data Phase 1 needs one discoverable bridge for real ETF proxy batches.
    // Purpose: let CLI and Skills find governed proxy-history import without hidden scripts.
    "security_external_proxy_history_import",
    // 2026-04-12 CST: Expose the governed stock fundamental-history backfill tool,
    // because Historical Data Phase 1 needs replayable financial snapshots to be
    // discoverable before fullstack and validation consume them.
    // Purpose: make stock information-history backfill a public stock capability.
    "security_fundamental_history_backfill",
    // 2026-04-12 CST: Expose the live stock fundamental-history backfill tool,
    // because Historical Data Phase 1 needs one discoverable provider-to-governed bridge
    // for multi-period financial history.
    // Purpose: make live governed financial backfill a public stock capability.
    "security_fundamental_history_live_backfill",
    // 2026-04-12 CST: Expose the governed stock disclosure-history backfill tool,
    // because Historical Data Phase 1 needs replayable announcement history to be
    // discoverable before fullstack and validation consume it.
    // Purpose: make stock disclosure-history backfill a public stock capability.
    "security_disclosure_history_backfill",
    // 2026-04-12 CST: Expose the live stock disclosure-history backfill tool,
    // because Historical Data Phase 1 needs one discoverable provider-to-governed bridge
    // for multi-page announcement history.
    // Purpose: make live governed disclosure backfill a public stock capability.
    "security_disclosure_history_live_backfill",
    // 2026-04-12 CST: Expose the governed real-data validation backfill tool, because
    // P11 needs a discoverable way to refresh validation slices with live-compatible data.
    // Purpose: make validation-slice refresh visible on the public stock catalog.
    "security_real_data_validation_backfill",
    // 2026-04-11 CST: Expose the governed history-expansion tool, because P5
    // promotion governance now needs one discoverable artifact that explains
    // how proxy-history coverage was expanded.
    // Purpose: let CLI and Skills find the expansion record without hidden wiring.
    "security_history_expansion",
    // 2026-04-11 CST: Expose the governed shadow-evaluation tool, because P5
    // requires a first-class review step between candidate metrics and promotion.
    // Purpose: make shadow readiness visible on the stock tool catalog.
    "security_shadow_evaluation",
    // 2026-04-11 CST: Expose the governed model-promotion tool, because P5
    // promotion decisions must be discoverable and auditable from the CLI layer.
    // Purpose: keep candidate/shadow/champion transitions on the public stock chain.
    "security_model_promotion",
    // 2026-04-09 CST: 这里把未来标签回填 Tool 暴露到 stock 目录，原因是 Task 3 要让 forward_outcome 成为正式训练底座入口；
    // 目的：让 CLI / Skill / 回算流水线可以稳定发现 snapshot 绑定的多期限标签能力。
    "security_forward_outcome",
    // 2026-04-11 CST: 这里把 master_scorecard Tool 暴露到 stock 目录，原因是方案 C 已确认要把
    // “未来几日赚钱效益总卡”正式产品化。
    // 目的：让 CLI / Skill / 持仓报告能够稳定发现总卡入口，而不是依赖口头流程。
    "security_master_scorecard",
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
    // 2026-04-12 CST: Expose the formal condition-review tool on the top-level
    // catalog, because lifecycle review must be discoverable from the main CLI list.
    // Purpose: keep tool discovery aligned with the new P8 lifecycle object set.
    "security_condition_review",
    // 2026-04-12 CST: Expose the formal execution-record tool on the top-level
    // catalog, because execution events are part of the public lifecycle from P8 onward.
    // Purpose: keep tool discovery aligned with the new execution object set.
    "security_execution_record",
    // 2026-04-12 CST: Expose the formal post-trade review tool on the top-level
    // catalog, because lifecycle review closure becomes public from P8 onward.
    // Purpose: keep discovery aligned with the new post-trade object set.
    "security_post_trade_review",
    // 2026-04-12 CST: Expose the file-based proxy-history import tool on the top-level
    // catalog, because public tool discovery still flows through TOOL_NAMES.
    // Purpose: keep governed proxy-history import aligned with the main CLI list.
    "security_external_proxy_history_import",
    // 2026-04-09 CST: 这里把未来标签回填 Tool 暴露到总目录，原因是主 dispatcher 的 tool_catalog 仍依赖全量 TOOL_NAMES；
    // 目的：确保 forward_outcome 像 feature_snapshot 一样成为主链可发现的一等能力。
    "security_forward_outcome",
    // 2026-04-11 CST: 这里把 master_scorecard Tool 暴露到总目录，原因是主 dispatcher 的 tool_catalog
    // 仍依赖全量 TOOL_NAMES。
    // 目的：确保未来几日赚钱效益总卡成为主链可发现的一等能力。
    "security_master_scorecard",
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
    // 2026-04-11 CST: Keep the dated external-proxy backfill tool discoverable on
    // the public catalog, because dispatcher tool discovery still flows through
    // TOOL_NAMES even when the stock sub-catalog already knows about the tool.
    // Purpose: make governed proxy-history backfill appear on the top-level CLI list.
    "security_external_proxy_backfill",
    // 2026-04-12 CST: Expose the governed stock fundamental-history backfill tool
    // on the top-level catalog, because dispatcher tool discovery still flows
    // through TOOL_NAMES.
    // Purpose: keep stock information-history backfill discoverable from the main CLI list.
    "security_fundamental_history_backfill",
    // 2026-04-12 CST: Expose the live stock fundamental-history backfill tool on the
    // top-level catalog, because public tool discovery still flows through TOOL_NAMES.
    // Purpose: keep live governed financial backfill aligned with the main CLI list.
    "security_fundamental_history_live_backfill",
    // 2026-04-12 CST: Expose the governed stock disclosure-history backfill tool
    // on the top-level catalog, because dispatcher tool discovery still flows
    // through TOOL_NAMES.
    // Purpose: keep stock disclosure-history backfill discoverable from the main CLI list.
    "security_disclosure_history_backfill",
    // 2026-04-12 CST: Expose the live stock disclosure-history backfill tool on the
    // top-level catalog, because public tool discovery still flows through TOOL_NAMES.
    // Purpose: keep live governed disclosure backfill aligned with the main CLI list.
    "security_disclosure_history_live_backfill",
    // 2026-04-12 CST: Expose the governed real-data validation backfill tool on the
    // top-level catalog, because tool discovery still flows through TOOL_NAMES.
    // Purpose: keep validation refresh aligned with the public CLI list.
    "security_real_data_validation_backfill",
    "security_history_expansion",
    "security_shadow_evaluation",
    "security_model_promotion",
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
