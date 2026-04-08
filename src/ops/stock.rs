// 2026-04-02 CST: 这里重写 stock 模块边界声明，原因是当前仓库已经形成 foundation / stock 两条主业务线，
// 目的：把股票域的模块导出收口到一个清晰入口，方便后续继续新增模板级共振工具而不在模块声明里反复插补。
#[path = "import_stock_price_history.rs"]
pub mod import_stock_price_history;

#[path = "security_analysis_contextual.rs"]
pub mod security_analysis_contextual;

#[path = "security_analysis_fullstack.rs"]
pub mod security_analysis_fullstack;

#[path = "security_decision_briefing.rs"]
pub mod security_decision_briefing;

// 2026-04-08 CST: 这里接入仓位计划记录模块，原因是证券主链要把 briefing 的 `position_plan`
// 升级成正式可引用对象，目的：让后续调仓与复盘继续沿 stock 业务域统一扩展，而不是起平行对象层。
#[path = "security_position_plan_record.rs"]
pub mod security_position_plan_record;

// 2026-04-08 CST: 这里接入调仓事件记录模块，原因是证券主链要从“正式仓位计划”继续推进到“正式执行事件”，
// 目的：让多次调仓记录继续沿 stock 业务域统一扩展，而不是在外层另起平行对象层。
#[path = "security_record_position_adjustment.rs"]
pub mod security_record_position_adjustment;

// 2026-04-08 CST: 这里接入投后复盘模块，原因是证券主链已经从计划对象与调仓事件推进到正式复盘闭环；
// 目的：让 post_trade_review 继续沿 stock 业务域统一扩展，而不是在外层另起平行对象层。
#[path = "security_post_trade_review.rs"]
pub mod security_post_trade_review;

// 2026-04-02 CST: 这里接入正式投决会模块，原因是 committee payload 已经升级为统一事实包，下一跳必须有确定的消费端，
// 目的：让 stock 业务域对外暴露稳定的 security_committee_vote 入口，而不是让上层继续手工拼投票逻辑。
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
