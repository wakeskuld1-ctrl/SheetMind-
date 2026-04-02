// 2026-04-02 CST: 这里重写 stock 模块边界声明，原因是方案C已经把股票分析扩展到“行情 + 共振平台 + 研究平台”组合能力；
// 目的：把股票域的模块导出收口到一个清晰入口，方便后续继续新增模板级共振工具而不在模块声明里反复插补。
#[path = "import_stock_price_history.rs"]
pub mod import_stock_price_history;

#[path = "security_analysis_contextual.rs"]
pub mod security_analysis_contextual;

#[path = "security_analysis_fullstack.rs"]
pub mod security_analysis_fullstack;

#[path = "security_decision_briefing.rs"]
pub mod security_decision_briefing;

// 2026-04-02 CST: 这里接入正式投决会模块，原因是 committee payload 已经升级为统一事实包，下一跳必须有确定的消费端，
// 目的：让 stock 业务域对外暴露稳定的 security_committee_vote 入口，而不是让上层继续手工拼投票逻辑。
#[path = "security_committee_vote.rs"]
pub mod security_committee_vote;

#[path = "security_analysis_resonance.rs"]
pub mod security_analysis_resonance;

// 2026-04-02 CST: 这里接入模板级共振因子同步模块，原因是银行宏观共振底座需要正式补出“模板补数”主链；
// 目的：让模板因子同步和后续评估、briefing 一样，全部沿 stock 业务域统一扩展。
#[path = "sync_template_resonance_factors.rs"]
pub mod sync_template_resonance_factors;

#[path = "signal_outcome_research.rs"]
pub mod signal_outcome_research;

#[path = "sync_stock_price_history.rs"]
pub mod sync_stock_price_history;

#[path = "technical_consultation_basic.rs"]
pub mod technical_consultation_basic;
