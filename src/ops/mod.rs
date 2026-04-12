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
pub use stock::security_approval_brief_signature;
pub use stock::security_decision_approval_bridge;
pub use stock::security_decision_approval_brief;
pub use stock::security_decision_card;
// 2026-04-02 CST: 这里导出证券审批包能力，原因是外层仍通过 `crate::ops::...` 兼容访问股票链路；
// 目的：不要求调用方立刻改路径，也能使用新的 decision package 构造能力。
pub use stock::security_decision_package;
pub use stock::security_scorecard;
// 2026-04-11 CST: 这里导出正式 master_scorecard 能力，原因是现有外层兼容调用仍可能通过 `crate::ops::...`
// 访问证券主链对象。
// 目的：在不打断旧路径的前提下，让未来几日赚钱效益总卡进入统一兼容出口。
pub use stock::security_master_scorecard;
// 2026-04-09 CST: 这里导出主席正式裁决能力，原因是外层兼容调用仍可能走 `crate::ops::...`；
// 目的：在不要求调用方立刻改路径的前提下，把主席线纳入正式兼容出口。
pub use stock::security_chair_resolution;
// 2026-04-09 CST: 这里导出正式特征快照能力，原因是后续训练/回算/治理层都可能通过兼容路径访问；
// 目的：在不打断旧调用路径的前提下，把 feature_snapshot 纳入正式兼容出口。
pub use stock::security_feature_snapshot;
// 2026-04-12 CST: Re-export the formal condition-review capability, because
// compatibility callers may still resolve stock lifecycle tools through `crate::ops::...`.
// Purpose: keep the new review object reachable without import rewrites.
pub use stock::security_condition_review;
// 2026-04-12 CST: Re-export the formal execution-record capability, because
// compatibility callers may still resolve stock lifecycle tools through `crate::ops::...`.
// Purpose: keep the new execution object reachable without import rewrites.
pub use stock::security_execution_record;
// 2026-04-12 CST: Re-export the formal post-trade review capability, because
// compatibility callers may still resolve stock lifecycle tools through `crate::ops::...`.
// Purpose: keep the new review object reachable without import rewrites.
pub use stock::security_post_trade_review;
// 2026-04-09 CST: 这里导出正式未来标签回填能力，原因是后续训练/回算/复盘链路都可能先通过 `crate::ops::...` 兼容路径访问；
// 目的：在不打断旧调用路径的前提下，把 forward_outcome 纳入统一兼容出口。
pub use stock::security_forward_outcome;
// 2026-04-11 CST: Re-export the dated external-proxy backfill capability,
// because P4 historical replay consumers may still resolve stock tools via
// `crate::ops::...` compatibility paths.
// Purpose: keep the new governed proxy-backfill tool discoverable on the old path.
pub use stock::security_external_proxy_backfill;
// 2026-04-12 CST: Re-export the file-based proxy-history import capability, because
// compatibility callers may still resolve new stock history tools through `crate::ops::...`.
// Purpose: keep governed proxy-history import reachable without import rewrites.
pub use stock::security_external_proxy_history_import;
// 2026-04-12 CST: Re-export the live stock fundamental-history backfill capability,
// because compatibility callers may still resolve new stock history tools through `crate::ops::...`.
// Purpose: keep live-to-governed financial-history ingestion reachable without import rewrites.
pub use stock::security_fundamental_history_live_backfill;
// 2026-04-12 CST: Re-export the live stock disclosure-history backfill capability,
// because compatibility callers may still resolve new stock history tools through `crate::ops::...`.
// Purpose: keep live-to-governed disclosure-history ingestion reachable without import rewrites.
pub use stock::security_disclosure_history_live_backfill;
// 2026-04-12 CST: Re-export the governed real-data validation backfill capability,
// because compatibility callers may still resolve new stock tools through `crate::ops::...`.
// Purpose: keep the validation-slice refresh entry reachable without import rewrites.
pub use stock::security_real_data_validation_backfill;
// 2026-04-11 CST: Re-export the governed history-expansion capability, because
// legacy compatibility paths may still resolve stock governance tools through
// `crate::ops::...` during the P5 transition.
// Purpose: keep history-expansion discoverable on the old compatibility path.
pub use stock::security_history_expansion;
// 2026-04-11 CST: Re-export the governed shadow-evaluation capability, because
// approval and future orchestration may still use compatibility imports.
// Purpose: preserve one stable access path while the stock domain evolves.
pub use stock::security_shadow_evaluation;
// 2026-04-11 CST: Re-export the governed model-promotion capability, because
// P5 promotion consumers should remain reachable on existing compatibility paths.
// Purpose: avoid forcing immediate import rewrites outside the stock module boundary.
pub use stock::security_model_promotion;
pub use stock::security_scorecard_refit_run;
// 2026-04-09 CST: 这里导出正式训练入口能力，原因是外层兼容调用仍可能沿用 `crate::ops::...` 路径；
// 目的：在不打断旧调用方式的前提下，把 Task 5 新能力纳入统一兼容出口。
pub use stock::security_scorecard_training;
// 2026-04-02 CST: 这里导出证券审批包版本化能力，原因是外层调用方仍通过 `crate::ops::...` 兼容访问股票链路；
// 目的：让新的 revision Tool 不要求调用方立刻改模块路径，也能沿现有入口被消费。
pub use stock::security_decision_package_revision;
// 2026-04-08 CST: 这里导出会后结论对象与记录入口，原因是现有外层兼容调用仍可能走 `crate::ops::...`；
// 目的：在不打断旧路径的前提下，把 Task 3 新能力纳入统一兼容出口。
pub use stock::security_decision_committee;
pub use stock::security_decision_evidence_bundle;
pub use stock::security_decision_submit_approval;
pub use stock::security_post_meeting_conclusion;
pub use stock::security_record_post_meeting_conclusion;
// 2026-04-02 CST: 这里导出证券审批包校验能力，原因是外层调用方仍通过 `crate::ops::...` 兼容访问股票链路；
// 目的：让新的 verify Tool 不要求调用方立刻改模块路径，也能沿现有入口被消费。
pub use stock::security_decision_verify_package;
pub use stock::security_position_plan;
pub use stock::security_risk_gates;
pub use stock::sync_stock_price_history;
pub use stock::technical_consultation_basic;
