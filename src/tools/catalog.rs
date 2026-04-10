pub const FOUNDATION_TOOL_NAMES: &[&str] = &[
    "license_activate",
    "license_status",
    "license_deactivate",
    // 2026-04-10 CST: 这里把 repository 级 metadata audit 正式挂进 foundation 目录，原因是当前通用知识库治理主线已经具备
    // schema/validator/versioning/migration contract，但还缺 CLI / Skill 可发现的整库审计入口；目的：让上层直接发现标准化 metadata 治理能力。
    "foundation_repository_metadata_audit",
    // 2026-04-10 CST: 这里把 repository metadata audit gate 挂进 foundation 目录，原因是方案A下一步要让上层直接消费“能否放行”的治理判断，
    // 目的：让 CLI / Skill 不必先拿 audit 报告再自行重复写 blocking 规则，统一由标准 foundation gate 收口。
    "foundation_repository_metadata_audit_gate",
    // 2026-04-10 CST: 这里把 repository metadata audit batch 挂进 foundation 目录，原因是方案A第一刀已经明确要把单仓库 gate 提升成批量入口，
    // 目的：让 CLI / Skill 可以在保持通用能力边界的前提下，直接对多个 repository layout 执行批次审计。
    "foundation_repository_metadata_audit_batch",
    // 2026-04-10 CST: 这里把 foundation repository import gate 挂进 foundation 目录，原因是方案B1要把 batch 结果继续提升为导入接入层标准入口，
    // 目的：让上层不必再手工解释 accepted / rejected 列表与阻塞原因，而是直接消费统一导入门禁结果。
    "foundation_repository_import_gate",
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
    "security_decision_evidence_bundle",
    "security_decision_committee",
    "security_committee_member_agent",
    "security_chair_resolution",
    "security_record_post_meeting_conclusion",
    "security_decision_package",
    "security_decision_verify_package",
    "security_decision_package_revision",
    "security_feature_snapshot",
    "security_forward_outcome",
    "security_scorecard_refit",
    // 2026-04-09 CST: 这里把正式 scorecard training Tool 暴露到 stock 目录，原因是 Task 5 需要统一发现训练主链入口；
    // 目的：让 CLI、Skill 与后续回算编排都能稳定发现“训练 -> artifact -> refit”这条正式能力链。
    "security_scorecard_training",
    // 2026-04-09 CST: 这里把 security_position_plan 暴露到 stock 目录，原因是 Task 7 要把 briefing 内仓位层正式对象化为独立 Tool；
    // 目的：让 Skill/CLI 可以直接发现“分析 -> 仓位计划”主链，而不是继续手工从 briefing 结果里拆字段。
    "security_position_plan",
    "security_portfolio_position_plan",
    // 2026-04-10 CST: 这里把账户 open snapshot 暴露到 stock 目录，原因是方案B要把账户层自动回接上一轮 execution_record 做成独立对象；
    // 目的：让上层显式发现“runtime 持仓状态 -> 账户计划”这条中间桥接能力，而不是隐式读库。
    "security_account_open_position_snapshot",
    // 2026-04-09 CST: 这里把 security_post_trade_review 暴露到 stock 目录，原因是 Task 8 要补齐正式投后复盘层；
    // 目的：让上层可以沿 position_plan -> post_trade_review 直接走完整投后主链。
    "security_post_trade_review",
    // 2026-04-09 CST: 这里把 security_execution_record 暴露到 stock 目录，原因是 Task 10 要让真实执行对象成为正式一等 Tool；
    // 目的：让 review/package/governance 不再只能消费建议层结果，而能显式绑定真实执行收益归因对象。
    "security_execution_record",
    // 2026-04-09 CST: 这里把 security_execution_journal 暴露到 stock 目录，原因是 P1 要把多笔成交升级为正式一等对象；
    // 目的：让上层先拿到结构化成交明细，再由 execution_record 聚合，而不是继续把分批成交塞进文本备注。
    "security_execution_journal",
    // 2026-04-02 CST: 这里把 security_decision_briefing 接入 stock 工具目录，原因是统一 briefing 已经成为咨询与投决的共同事实入口；
    // 目的：让 CLI、Skill 和后续 GUI 可以先发现 briefing Tool，再围绕它建立统一路由。
    "security_decision_briefing",
    // 2026-04-10 CST: 这里补挂审批提交与条件复核 Tool 名称，原因是当前分支已导入实现文件但尚未进入正式目录；
    // 目的：让 CLI / Skill / AI 可以稳定发现“投决提交审批”和“投中条件复核”两个主链入口，而不是继续靠内部模块名硬调。
    "security_decision_submit_approval",
    "security_condition_review",
    // 2026-04-10 CST: 这里补挂轻量会后结论 Tool，原因是当前分支缺的是独立 formal object 入口，不是替换旧 record Tool；
    // 目的：让 CLI / Skill 同时能发现“轻量对象化结论”和“较重会后记录”两种能力。
    "security_post_meeting_conclusion",
    // 2026-04-08 CST: 这里把 security_position_plan_record 接入 stock 目录，原因是证券主链正在从投前建议推进到投中执行闭环；
    // 目的：让上层可以在正式 Tool 目录里发现“把仓位建议升级成正式计划对象”的入口。
    "security_position_plan_record",
    "security_post_trade_review",
    "security_record_position_adjustment",
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
    // 2026-04-10 CST: 这里同步把 foundation repository metadata audit 写入扁平总目录，原因是 tool_catalog 顶层仍然依赖 TOOL_NAMES 输出，
    // 目的：保证新 foundation Tool 在分组目录和全局目录两层都能被稳定发现。
    "foundation_repository_metadata_audit",
    // 2026-04-10 CST: 这里同步把 repository metadata audit gate 写进扁平总目录，原因是顶层 tool_catalog 仍然依赖 TOOL_NAMES 输出，
    // 目的：保证 gate 在分组目录和全局目录两层都能稳定被发现，避免只进 foundation 分组却漏掉全局目录。
    "foundation_repository_metadata_audit_gate",
    // 2026-04-10 CST: 这里同步把 repository metadata audit batch 写进扁平总目录，原因是批量入口和单仓库入口一样都需要被顶层 tool_catalog 发现，
    // 目的：保证后续批处理编排可以沿统一目录读取这个 A1 标准能力。
    "foundation_repository_metadata_audit_batch",
    // 2026-04-10 CST: 这里同步把 foundation repository import gate 写进扁平总目录，原因是方案B1同样需要被顶层 tool_catalog 稳定发现，
    // 目的：保证导入链接入层既能出现在 foundation 分组里，也能出现在全局目录里。
    "foundation_repository_import_gate",
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
    "security_chair_resolution",
    "security_record_post_meeting_conclusion",
    "security_decision_package",
    "security_decision_verify_package",
    "security_decision_package_revision",
    "security_feature_snapshot",
    "security_forward_outcome",
    "security_scorecard_refit",
    // 2026-04-09 CST: 这里把正式 scorecard training Tool 暴露到总目录，原因是主 dispatcher 的 tool_catalog 仍依赖全量 TOOL_NAMES；
    // 目的：确保训练入口和 snapshot/forward_outcome/refit 一样成为主链可发现能力。
    "security_scorecard_training",
    // 2026-04-09 CST: 这里把 security_position_plan 补进扁平总目录，原因是主 dispatcher 的 tool_catalog 仍依赖 TOOL_NAMES；
    // 目的：确保独立仓位计划对象和 briefing 一样成为一等可发现能力。
    "security_position_plan",
    "security_portfolio_position_plan",
    "security_account_open_position_snapshot",
    // 2026-04-09 CST: 这里把 security_post_trade_review 补进扁平总目录，原因是投后复盘需要成为正式一等 Tool；
    // 目的：避免后续继续由外层手工拼接 forward_outcome 与 position_plan 做“伪复盘”。
    "security_post_trade_review",
    // 2026-04-09 CST: 这里把 security_execution_record 补进扁平总目录，原因是主 dispatcher 的 tool_catalog 仍依赖 TOOL_NAMES；
    // 目的：确保真实执行记录对象和 review/package 一样成为正式可发现能力。
    "security_execution_record",
    // 2026-04-09 CST: 这里把 security_execution_journal 补进扁平总目录，原因是主 dispatcher 的 tool_catalog 仍依赖 TOOL_NAMES；
    // 目的：确保多笔成交 journal 能和 execution_record 一样被 CLI / Skill 正式发现。
    "security_execution_journal",
    // 2026-04-02 CST: 这里把 security_decision_briefing 接入扁平总目录，原因是现有 tool_catalog 契约仍要求返回统一可发现工具列表；
    // 目的：确保 briefing Tool 不仅存在于 stock 分组，也能在顶层目录中被外层调用方稳定发现。
    "security_decision_briefing",
    // 2026-04-10 CST: 这里同步把审批提交与条件复核写入顶层目录，原因是主 dispatcher 的 tool_catalog 仍依赖扁平 TOOL_NAMES；
    // 目的：确保外层发现逻辑在 stock 分组和全局目录两层都能看到这两个新增 Tool。
    "security_decision_submit_approval",
    "security_condition_review",
    // 2026-04-10 CST: 这里同步把轻量会后结论写入顶层目录，原因是外层发现仍依赖 TOOL_NAMES，
    // 目的：避免对象已并回但 catalog 不可见，导致后续 Skill/AI 无法稳定调用。
    "security_post_meeting_conclusion",
    // 2026-04-08 CST: 这里把 security_position_plan_record 补进顶层总目录，原因是后续 CLI / Skill 仍依赖统一 catalog 判断可发现性；
    // 目的：保证仓位计划记录入口既能在 stock 分组中发现，也能在平铺目录里稳定暴露。
    "security_position_plan_record",
    "security_post_trade_review",
    "security_record_position_adjustment",
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
