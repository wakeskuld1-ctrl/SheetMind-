#[path = "import_stock_price_history.rs"]
pub mod import_stock_price_history;

#[path = "security_analysis_contextual.rs"]
pub mod security_analysis_contextual;

#[path = "security_analysis_fullstack.rs"]
pub mod security_analysis_fullstack;

#[path = "security_decision_briefing.rs"]
pub mod security_decision_briefing;

#[path = "security_approval_brief_signature.rs"]
pub mod security_approval_brief_signature;

#[path = "security_decision_approval_bridge.rs"]
pub mod security_decision_approval_bridge;

#[path = "security_decision_approval_brief.rs"]
pub mod security_decision_approval_brief;

#[path = "security_decision_card.rs"]
pub mod security_decision_card;

#[path = "security_decision_package.rs"]
pub mod security_decision_package;

#[path = "security_scorecard.rs"]
pub mod security_scorecard;

#[path = "security_position_plan_record.rs"]
pub mod security_position_plan_record;

#[path = "security_record_position_adjustment.rs"]
pub mod security_record_position_adjustment;

#[path = "security_committee_vote.rs"]
pub mod security_committee_vote;

#[path = "security_condition_review.rs"]
pub mod security_condition_review;

// 2026-04-10 CST: 这里导出 execution_record 模块，原因是 Task 5 需要把条件复核继续挂进执行主链并供 dispatcher 调用；
// 目的：让 stock 工具分发层能显式引用 execution_record，而不是停留在未导出的内部模块状态。
#[path = "security_execution_record.rs"]
pub mod security_execution_record;

// 2026-04-10 CST: 这里导出 post_trade_review 模块，原因是 Task 5 需要把条件复核解释正式接进投后复盘主链；
// 目的：让 dispatcher 与后续 package/review 链能稳定引用 post_trade_review 正式实现。
#[path = "security_post_trade_review.rs"]
pub mod security_post_trade_review;

#[path = "security_analysis_resonance.rs"]
pub mod security_analysis_resonance;

#[path = "sync_template_resonance_factors.rs"]
pub mod sync_template_resonance_factors;

#[path = "signal_outcome_research.rs"]
pub mod signal_outcome_research;

#[path = "security_chair_resolution.rs"]
pub mod security_chair_resolution;

#[path = "security_feature_snapshot.rs"]
pub mod security_feature_snapshot;

#[path = "security_forward_outcome.rs"]
pub mod security_forward_outcome;

#[path = "security_scorecard_model_registry.rs"]
pub mod security_scorecard_model_registry;

#[path = "security_scorecard_refit_run.rs"]
pub mod security_scorecard_refit_run;

#[path = "security_scorecard_training.rs"]
pub mod security_scorecard_training;

#[path = "security_decision_package_revision.rs"]
pub mod security_decision_package_revision;

#[path = "security_post_meeting_conclusion.rs"]
pub mod security_post_meeting_conclusion;

#[path = "security_decision_committee.rs"]
pub mod security_decision_committee;

#[path = "security_decision_evidence_bundle.rs"]
pub mod security_decision_evidence_bundle;

#[path = "security_decision_submit_approval.rs"]
pub mod security_decision_submit_approval;

#[path = "security_record_post_meeting_conclusion.rs"]
pub mod security_record_post_meeting_conclusion;

#[path = "security_decision_verify_package.rs"]
pub mod security_decision_verify_package;

#[path = "security_position_plan.rs"]
pub mod security_position_plan;

#[path = "security_risk_gates.rs"]
pub mod security_risk_gates;

#[path = "sync_stock_price_history.rs"]
pub mod sync_stock_price_history;

#[path = "technical_consultation_basic.rs"]
pub mod technical_consultation_basic;
