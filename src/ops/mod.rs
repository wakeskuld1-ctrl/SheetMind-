// 2026-04-02 CST: 这里重写 ops 顶层导出，原因是当前仓库已经形成 foundation / stock 两条主业务线；
// 目的：在保持旧引用兼容的前提下，把新引入的模板级共振同步能力纳入统一导出面。
pub mod foundation;
pub mod stock;

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

pub use stock::import_stock_price_history;
pub use stock::security_analysis_contextual;
pub use stock::security_analysis_fullstack;
pub use stock::security_analysis_resonance;
pub use stock::security_decision_briefing;
pub use stock::security_execution_journal;
pub use stock::security_execution_record;
pub use stock::security_portfolio_position_plan;
pub use stock::security_position_plan;
pub use stock::security_position_plan_record;
pub use stock::security_post_trade_review;
pub use stock::security_record_position_adjustment;

// 2026-04-02 CST: 这里补顶层投决会导出，原因是现有测试和上层调用大量通过 crate::ops::... 访问 stock 能力；
// 目的：保持旧引用习惯不变，同时把 security_committee_vote 纳入统一导出面。
pub use stock::security_committee_vote;
pub use stock::signal_outcome_research;
pub use stock::sync_stock_price_history;

// 2026-04-02 CST: 这里补模板级共振同步能力的兼容导出，原因是现有测试和调用点仍大量沿 `crate::ops::...` 引用 stock 能力；
// 目的：让新底座能力进入主干导出面，后续 CLI / Skill / 测试可直接复用。
pub use stock::security_chair_resolution;
pub use stock::security_decision_card;
pub use stock::security_decision_committee;
pub use stock::security_decision_evidence_bundle;
pub use stock::security_decision_package;
pub use stock::security_decision_package_revision;
pub use stock::security_decision_verify_package;
pub use stock::security_feature_snapshot;
pub use stock::security_forward_outcome;
pub use stock::security_record_post_meeting_conclusion;
pub use stock::security_risk_gates;
pub use stock::security_scorecard;
pub use stock::security_scorecard_model_registry;
pub use stock::security_scorecard_refit_run;
pub use stock::security_scorecard_training;
pub use stock::sync_template_resonance_factors;
pub use stock::technical_consultation_basic;
