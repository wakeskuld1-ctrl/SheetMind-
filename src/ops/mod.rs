// 2026-03-31 CST: 这里把 ops 显式拆成 foundation / stock 两个业务域，原因是用户已经确认底座能力和股票能力不能继续串台。
// 目的：先在模块层建立清晰边界，再通过兼容 re-export 维持现有调用不炸，后续新增能力按模块归属进入对应域。
pub mod foundation;
pub mod stock;

// 2026-03-31 CST: 这里保留 foundation 能力的兼容导出，原因是当前仓库已有大量 `crate::ops::...` 引用，不能一轮全改。
// 目的：让旧代码先稳定通过，同时把“真正声明模块”的位置固定到 foundation。
pub use foundation::analyze;
pub use foundation::append;
pub use foundation::capacity_assessment;
pub use foundation::capacity_assessment_excel_report;
pub use foundation::capacity_assessment_from_inventory;
pub use foundation::cast;
pub use foundation::chart_svg;
pub use foundation::cluster_kmeans;
pub use foundation::correlation_analysis;
pub use foundation::decision_assistant;
pub use foundation::deduplicate_by_key;
pub use foundation::derive;
pub use foundation::diagnostics_report;
pub use foundation::diagnostics_report_excel_report;
pub use foundation::distinct_rows;
pub use foundation::distribution_analysis;
pub use foundation::excel_chart_writer;
pub use foundation::export;
pub use foundation::fill_lookup;
pub use foundation::fill_missing_values;
pub use foundation::filter;
pub use foundation::format_table_for_export;
pub use foundation::group;
pub use foundation::join;
pub use foundation::linear_regression;
pub use foundation::logistic_regression;
pub use foundation::lookup_values;
pub use foundation::model_output;
pub use foundation::model_prep;
pub use foundation::multi_table_plan;
pub use foundation::normalize_text;
pub use foundation::outlier_detection;
pub use foundation::parse_datetime;
pub use foundation::pivot;
pub use foundation::preview;
pub use foundation::rename;
pub use foundation::report_delivery;
pub use foundation::select;
pub use foundation::semantic;
pub use foundation::sort;
pub use foundation::ssh_inventory;
pub use foundation::stat_summary;
pub use foundation::summary;
pub use foundation::table_links;
pub use foundation::table_workflow;
pub use foundation::top_n;
pub use foundation::trend_analysis;
pub use foundation::window;

// 2026-03-31 CST: 这里保留 stock 能力的兼容导出，原因是现有测试和调用方仍通过 `crate::ops::...` 访问股票链路。
// 目的：先建立模块边界和新归属，再逐步把调用点迁到 `crate::ops::stock::...`。
pub use stock::import_stock_price_history;
pub use stock::security_analysis_contextual;
pub use stock::security_analysis_fullstack;
pub use stock::security_decision_card;
pub use stock::security_decision_approval_bridge;
pub use stock::security_decision_approval_brief;
pub use stock::security_approval_brief_signature;
// 2026-04-02 CST: 这里导出证券审批包能力，原因是外层仍通过 `crate::ops::...` 兼容访问股票链路；
// 目的：不要求调用方立刻改路径，也能使用新的 decision package 构造能力。
pub use stock::security_decision_package;
// 2026-04-02 CST: 这里导出证券审批包版本化能力，原因是外层调用方仍通过 `crate::ops::...` 兼容访问股票链路；
// 目的：让新的 revision Tool 不要求调用方立刻改模块路径，也能沿现有入口被消费。
pub use stock::security_decision_package_revision;
pub use stock::security_decision_committee;
pub use stock::security_decision_evidence_bundle;
pub use stock::security_decision_submit_approval;
// 2026-04-02 CST: 这里导出证券审批包校验能力，原因是外层调用方仍通过 `crate::ops::...` 兼容访问股票链路；
// 目的：让新的 verify Tool 不要求调用方立刻改模块路径，也能沿现有入口被消费。
pub use stock::security_decision_verify_package;
pub use stock::security_position_plan;
pub use stock::security_risk_gates;
pub use stock::sync_stock_price_history;
pub use stock::technical_consultation_basic;
