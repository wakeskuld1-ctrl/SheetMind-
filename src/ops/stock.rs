// 2026-04-02 CST: 这里重写 stock 模块边界声明，原因是当前仓库已经形成 foundation / stock 两条主业务线；
// 目的：把证券域的模块导出收口到一个清晰入口，方便后续继续新增模板级共振工具而不在模块声明里反复插补。
#[path = "import_stock_price_history.rs"]
pub mod import_stock_price_history;

#[path = "security_analysis_contextual.rs"]
pub mod security_analysis_contextual;

#[path = "security_analysis_fullstack.rs"]
pub mod security_analysis_fullstack;

#[path = "security_decision_briefing.rs"]
pub mod security_decision_briefing;

// 2026-04-09 CST: 这里保留 4-8 形成的计划记录与调仓事件模块，原因是现有 runtime、测试与交接文档仍在直接引用。
// 目的：本轮先做兼容合并，避免在推进新主链时把旧执行链路直接打断。
#[path = "security_position_plan_record.rs"]
pub mod security_position_plan_record;

#[path = "security_record_position_adjustment.rs"]
pub mod security_record_position_adjustment;

// 2026-04-09 CST: 这里接入新的仓位与执行主链模块，原因是 4-9 起 position_plan / execution_record / journal 已经成为正式 Tool。
// 目的：让标准化交付能力沿单票、账户、执行与复盘主线持续演进，同时保留旧对象做平滑过渡。
#[path = "security_position_plan.rs"]
pub mod security_position_plan;

#[path = "security_portfolio_position_plan.rs"]
pub mod security_portfolio_position_plan;

#[path = "security_post_trade_review.rs"]
pub mod security_post_trade_review;

#[path = "security_execution_record.rs"]
pub mod security_execution_record;

#[path = "security_execution_journal.rs"]
pub mod security_execution_journal;

// 2026-04-09 CST: 这里新增统一日期/补数门禁辅助模块，原因是用户要求把“本地优先 -> 自动补数 -> 最近交易日回退 -> 显式日期说明”内建到 Tool/Contract。
// 目的：供 technical/fullstack/briefing/position_plan 同源复用，避免规则继续只停留在 Skill 文档口头层。
#[path = "stock_analysis_data_guard.rs"]
pub mod stock_analysis_data_guard;

// 2026-04-02 CST: 这里接入正式投决会模块，原因是 committee payload 已经升级为统一事实包，下一跳必须有确定的消费端；
// 目的：让 stock 业务域对外暴露稳定的 security_committee_vote 入口，而不是让上层继续手工拼接投票逻辑。
#[path = "security_committee_vote.rs"]
pub mod security_committee_vote;

#[path = "security_analysis_resonance.rs"]
pub mod security_analysis_resonance;

// 2026-04-02 CST: 这里接入模板级共振因子同步模块，原因是银行宏观共振底座需要正式补齐“模板补数”主链，
// 目的：让模板因子同步和后续评估、briefing 一样，全部沿 stock 业务域统一扩展。
#[path = "sync_template_resonance_factors.rs"]
pub mod sync_template_resonance_factors;

#[path = "signal_outcome_research.rs"]
pub mod signal_outcome_research;

#[path = "sync_stock_price_history.rs"]
pub mod sync_stock_price_history;

#[path = "technical_consultation_basic.rs"]
pub mod technical_consultation_basic;

#[path = "security_decision_evidence_bundle.rs"]
pub mod security_decision_evidence_bundle;

#[path = "security_risk_gates.rs"]
pub mod security_risk_gates;

#[path = "security_decision_card.rs"]
pub mod security_decision_card;

#[path = "security_decision_committee.rs"]
pub mod security_decision_committee;

#[path = "security_scorecard.rs"]
pub mod security_scorecard;

#[path = "security_chair_resolution.rs"]
pub mod security_chair_resolution;

#[path = "security_record_post_meeting_conclusion.rs"]
pub mod security_record_post_meeting_conclusion;

#[path = "security_decision_package.rs"]
pub mod security_decision_package;

#[path = "security_decision_verify_package.rs"]
pub mod security_decision_verify_package;

#[path = "security_decision_package_revision.rs"]
pub mod security_decision_package_revision;

#[path = "security_feature_snapshot.rs"]
pub mod security_feature_snapshot;

#[path = "security_forward_outcome.rs"]
pub mod security_forward_outcome;

#[path = "security_scorecard_model_registry.rs"]
pub mod security_scorecard_model_registry;

#[path = "security_scorecard_refit_run.rs"]
pub mod security_scorecard_refit_run;

// 2026-04-09 CST: 这里挂入正式训练入口模块，原因是 Task 5 需要把离线 scorecard 训练纳入证券主链边界；
// 目的：让训练能力与 snapshot、forward_outcome、refit 处于同一 stock 域内持续演进，避免回退到脚本式管理。
#[path = "security_scorecard_training.rs"]
pub mod security_scorecard_training;
