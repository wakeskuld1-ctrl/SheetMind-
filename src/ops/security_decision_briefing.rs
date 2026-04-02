use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::ops::stock::security_analysis_resonance::{
    SecurityAnalysisResonanceError, SecurityAnalysisResonanceRequest,
    SecurityAnalysisResonanceResult, security_analysis_resonance,
};
use crate::ops::stock::security_committee_vote::{
    SecurityCommitteeVoteError, SecurityCommitteeVoteResult, SecurityCommitteeVoteRequest,
    security_committee_vote,
};
use crate::ops::stock::signal_outcome_research::{
    SignalOutcomeResearchSummaryRequest, signal_outcome_research_summary,
};

// 2026-04-02 CST: 这里先定义 security_decision_briefing 的请求合同，原因是本轮第一步只需要先把 briefing Tool 的输入边界稳定下来，
// 目的：让后续 assembler、dispatcher 和 Skill 都围绕同一份强类型请求扩展，而不是继续在各层散落弱类型 JSON 参数解释。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityDecisionBriefingRequest {
    pub symbol: String,
    #[serde(default)]
    pub market_symbol: Option<String>,
    #[serde(default)]
    pub sector_symbol: Option<String>,
    pub market_regime: String,
    pub sector_template: String,
    #[serde(default)]
    pub as_of_date: Option<String>,
    #[serde(default = "default_lookback_days")]
    pub lookback_days: usize,
    #[serde(default = "default_factor_lookback_days")]
    pub factor_lookback_days: usize,
    #[serde(default = "default_disclosure_limit")]
    pub disclosure_limit: usize,
}

// 2026-04-02 CST: 这里先定义 briefing 顶层响应合同，原因是计划第一步要求先把咨询场景与投决场景共享的事实载体钉住，
// 目的：确保后续就算逐步补 assembler、执行层和 committee payload，也不会再改动对外字段骨架。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SecurityDecisionBriefingResult {
    pub symbol: String,
    pub analysis_date: String,
    pub summary: String,
    pub evidence_version: String,
    pub fundamental_brief: BriefingLayer,
    pub technical_brief: BriefingLayer,
    pub resonance_brief: BriefingLayer,
    pub execution_plan: ExecutionPlan,
    pub committee_payload: CommitteePayload,
    pub committee_recommendations: CommitteeRecommendations,
}

// 2026-04-02 CST: 这里新增 briefing 默认携带的投决会建议集合，原因是用户明确要求普通个股分析报告也要默认带出投决建议，
// 目的：让上层用户不需要先知道“是否进入投决会”，而是在同一份 briefing 里直接看到 standard / strict / advisory 三种正式口径。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CommitteeRecommendations {
    pub default_mode: String,
    pub report_focus: String,
    pub standard: CommitteeRecommendationEntry,
    pub strict: CommitteeRecommendationEntry,
    pub advisory: CommitteeRecommendationEntry,
}

// 2026-04-02 CST: 这里把每种 committee 模式的适用场景和正式 vote 结果收口到同一结构里，原因是报告侧既要解释“什么时候看哪种建议”，
// 目的：也要保证展示内容直接复用正式 `security_committee_vote` 输出，而不是额外手写一份易漂移的摘要。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CommitteeRecommendationEntry {
    pub scenario: String,
    pub vote: SecurityCommitteeVoteResult,
}

// 2026-04-02 CST: 这里先定义交易执行层合同，原因是计划后续会在同一 briefing 中补齐可执行阈值而不是只给抽象方向判断，
// 目的：先把字段边界稳定住，后续再把每个阈值映射到真实技术指标来源。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExecutionPlan {
    pub add_trigger_price: f64,
    pub add_trigger_volume_ratio: f64,
    pub add_position_pct: f64,
    pub reduce_trigger_price: f64,
    pub rejection_zone: String,
    pub reduce_position_pct: f64,
    pub stop_loss_price: f64,
    pub invalidation_price: f64,
    pub watch_points: Vec<String>,
    pub explanation: Vec<String>,
}

// 2026-04-02 CST: 这里先定义 committee payload 合同，原因是第一阶段虽然不实现投票引擎，但必须先把投决入口的数据口径稳定下来，
// 目的：让咨询模式和投决模式都消费同一份 factual payload，避免上层 Agent 各自再拼装一套事实。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CommitteePayload {
    pub symbol: String,
    pub analysis_date: String,
    pub recommended_action: String,
    pub confidence: String,
    pub key_risks: Vec<String>,
    pub minority_objection_points: Vec<String>,
    pub evidence_version: String,
    pub briefing_digest: String,
    pub committee_schema_version: String,
    pub recommendation_digest: CommitteeRecommendationDigest,
    pub execution_digest: CommitteeExecutionDigest,
    pub resonance_digest: CommitteeResonanceDigest,
    pub evidence_checks: CommitteeEvidenceChecks,
    pub historical_digest: CommitteeHistoricalDigest,
}

// 2026-04-02 CST: 这里把投决建议摘要收口成独立子层，原因是后续 chair 与基本面/技术面角色都要读取同一份推荐事实，
// 目的：让 vote Tool 只消费 committee payload，不再回头扫描 briefing 其他层拼接推荐语义。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CommitteeRecommendationDigest {
    pub final_stance: String,
    pub action_bias: String,
    pub summary: String,
    pub confidence: String,
}

// 2026-04-02 CST: 这里把执行层复制成 committee digest，原因是 execution reviewer 需要结构化阈值而不是只看 briefing 摘要，
// 目的：把“怎么做、在哪做、什么情况下撤”固定为可投票、可展示、可留痕的事实子层。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CommitteeExecutionDigest {
    pub add_trigger_price: f64,
    pub add_trigger_volume_ratio: f64,
    pub add_position_pct: f64,
    pub reduce_trigger_price: f64,
    pub reduce_position_pct: f64,
    pub stop_loss_price: f64,
    pub invalidation_price: f64,
    pub rejection_zone: String,
    pub watch_points: Vec<String>,
    pub explanation: Vec<String>,
}

// 2026-04-02 CST: 这里把共振层压成 committee 摘要，原因是投决角色更关心“驱动是什么、反向扰动是什么”，
// 目的：在不暴露整层复杂对象的前提下，把共振证据稳定映射成投票输入。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CommitteeResonanceDigest {
    pub resonance_score: f64,
    pub action_bias: String,
    pub top_positive_driver_names: Vec<String>,
    pub top_negative_driver_names: Vec<String>,
    pub event_override_titles: Vec<String>,
}

// 2026-04-02 CST: 这里显式写出证据就绪检查，原因是风险官与主席需要先判断“是否已具备正式表决条件”，
// 目的：把 briefing 各层 readiness 变成确定性的布尔信号，而不是让不同角色各自猜测。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CommitteeEvidenceChecks {
    pub fundamental_ready: bool,
    pub technical_ready: bool,
    pub resonance_ready: bool,
    pub execution_ready: bool,
    pub briefing_ready: bool,
}

// 2026-04-02 CST: 这里预留历史研究摘要，原因是方案 B 要让 vote Tool 先支持 unavailable 边界，再平滑接入研究增强，
// 目的：避免 signal outcome 研究层后续接入时再次改动 committee payload 主合同。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CommitteeHistoricalDigest {
    pub status: String,
    pub historical_confidence: String,
    pub analog_sample_count: usize,
    pub analog_win_rate_10d: Option<f64>,
    pub expected_return_window: Option<String>,
    pub expected_drawdown_window: Option<String>,
    pub research_limitations: Vec<String>,
}

#[derive(Debug, Error)]
pub enum SecurityDecisionBriefingError {
    #[error("security_decision_briefing 复用共振分析失败: {0}")]
    Resonance(#[from] SecurityAnalysisResonanceError),
    #[error("security_decision_briefing 序列化子层失败: {0}")]
    Serialization(String),
    #[error("security_decision_briefing 生成默认投决会建议失败: {0}")]
    CommitteeVote(#[from] SecurityCommitteeVoteError),
}

// 2026-04-02 CST: 这里先定义 briefing 子层允许承载的结构化对象类型，原因是合同红测阶段需要锁定子层字段存在性而不是业务计算结果，
// 目的：为后续测试和调试保留轻量级占位载体，同时不阻断后续替换成真实事实层对象。
pub type BriefingLayer = Value;

// 2026-04-02 CST: 这里补 security_decision_briefing assembler 主入口，原因是第二步需要把已有 technical/fullstack/resonance
// 事实层装配成单一 briefing 结构；目的：让后续咨询、交易执行和 committee payload 都围绕同一份事实载体继续扩展。
pub fn security_decision_briefing(
    request: &SecurityDecisionBriefingRequest,
) -> Result<SecurityDecisionBriefingResult, SecurityDecisionBriefingError> {
    let resonance_request = SecurityAnalysisResonanceRequest {
        symbol: request.symbol.clone(),
        market_symbol: request.market_symbol.clone(),
        sector_symbol: request.sector_symbol.clone(),
        market_regime: request.market_regime.clone(),
        sector_template: request.sector_template.clone(),
        as_of_date: request.as_of_date.clone(),
        lookback_days: request.lookback_days,
        factor_lookback_days: request.factor_lookback_days,
        disclosure_limit: request.disclosure_limit,
    };
    let resonance_analysis = security_analysis_resonance(&resonance_request)?;
    assemble_security_decision_briefing(resonance_analysis)
}

// 2026-04-02 CST: 这里把 assembler 收口成单独函数，原因是后续还要继续给 execution_plan 与 committee_payload 做增量增强；
// 目的：让“复用既有事实层”和“生成 briefing 子层”分离，降低后续扩展交易执行层时的修改面。
fn assemble_security_decision_briefing(
    analysis: SecurityAnalysisResonanceResult,
) -> Result<SecurityDecisionBriefingResult, SecurityDecisionBriefingError> {
    let analysis_date = analysis
        .base_analysis
        .technical_context
        .stock_analysis
        .as_of_date
        .clone();
    let evidence_version = build_evidence_version(&analysis.symbol, &analysis_date);
    let summary = build_summary(&analysis);
    let technical_brief =
        serialize_layer(&analysis.base_analysis.technical_context.stock_analysis)?;
    let fundamental_brief = serialize_layer(&analysis.base_analysis.fundamental_context)?;
    let resonance_brief = serialize_layer(&analysis.resonance_context)?;
    let execution_plan = build_execution_plan(&analysis);
    let committee_payload = build_committee_payload(
        &analysis,
        &analysis_date,
        &summary,
        &evidence_version,
        &execution_plan,
    );
    let committee_recommendations = build_committee_recommendations(&committee_payload)?;

    Ok(SecurityDecisionBriefingResult {
        symbol: analysis.symbol,
        analysis_date,
        summary,
        evidence_version,
        fundamental_brief,
        technical_brief,
        resonance_brief,
        execution_plan,
        committee_payload,
        committee_recommendations,
    })
}

// 2026-04-02 CST: 这里统一做子层序列化，原因是 briefing 当前阶段只需要稳定输出结构化 JSON 合同，不需要把内部源对象继续暴露为耦合类型；
// 目的：让 assembler 在保持事实层完整度的同时，把对外合同稳定收口成可测试的 JSON 子对象。
fn serialize_layer<T: Serialize>(
    value: &T,
) -> Result<BriefingLayer, SecurityDecisionBriefingError> {
    serde_json::to_value(value)
        .map_err(|error| SecurityDecisionBriefingError::Serialization(error.to_string()))
}

// 2026-04-02 CST: 这里先生成统一 evidence_version，原因是 committee payload 阶段一已经要求咨询与投决共享同一份事实版本标识；
// 目的：让后续 vote Tool 可以用同一版本号识别 briefing 是否来自同一份底稿。
fn build_evidence_version(symbol: &str, analysis_date: &str) -> String {
    format!("security-decision-briefing:{symbol}:{analysis_date}:v1")
}

// 2026-04-02 CST: 这里先把 integrated_conclusion 和 resonance bias 拼成简报摘要，原因是当前 assembler 阶段需要先交付一个稳定 summary 字段；
// 目的：让上层调用方能先拿到“综合结论 + 共振偏向”的单句摘要，后续再继续增强交易执行与投决内容。
fn build_summary(analysis: &SecurityAnalysisResonanceResult) -> String {
    format!(
        "{}；共振动作偏向为 {}。",
        analysis.base_analysis.integrated_conclusion.headline,
        analysis.resonance_context.action_bias
    )
}

// 2026-04-02 CST: 这里把 execution_plan 正式改为指标派生结构，原因是 Task 3 要求 briefing 不再停留在占位阈值，而要输出可执行交易门槛；
// 目的：把阻力位、量比门槛、承接位、强弱分界与趋势失效位统一收口到 briefing 内，避免上层 Agent 再手工拼阈值。
fn build_execution_plan(analysis: &SecurityAnalysisResonanceResult) -> ExecutionPlan {
    let snapshot = &analysis
        .base_analysis
        .technical_context
        .stock_analysis
        .indicator_snapshot;
    let action_bias = analysis.resonance_context.action_bias.as_str();
    let add_trigger_price = round_price(snapshot.resistance_level_20.max(snapshot.close));
    let add_trigger_volume_ratio = round_ratio(snapshot.volume_ratio_20.max(1.05));
    let add_position_pct = match action_bias {
        "add_on_strength" => 0.12,
        "hold_and_confirm" => 0.06,
        "watch_conflict" => 0.04,
        _ => 0.03,
    };
    let reduce_trigger_price = round_price(snapshot.ema_10.min(snapshot.close));
    let reduce_position_pct = match action_bias {
        "reduce_or_exit" => 0.20,
        "watch_conflict" => 0.12,
        _ => 0.08,
    };
    let stop_loss_price = round_price(snapshot.boll_middle);
    let invalidation_price = round_price(snapshot.sma_50);
    let rejection_zone = format!(
        "{:.2}-{:.2}",
        snapshot.resistance_level_20,
        snapshot.resistance_level_20 + snapshot.atr_14.max(0.01)
    );
    let mut watch_points = analysis
        .base_analysis
        .technical_context
        .stock_analysis
        .watch_points
        .clone();
    watch_points.push(format!(
        "若量比未达到 {:.2} 以上，突破阻力位后不追价。",
        add_trigger_volume_ratio
    ));
    watch_points.push(format!(
        "若跌破 {:.2} 的短承接位，优先执行减仓观察。",
        reduce_trigger_price
    ));

    ExecutionPlan {
        add_trigger_price,
        add_trigger_volume_ratio,
        add_position_pct,
        reduce_trigger_price,
        rejection_zone,
        reduce_position_pct,
        stop_loss_price,
        invalidation_price,
        watch_points,
        explanation: vec![
            format!(
                "加仓触发价取自 resistance_level_20={:.2}，对应近期 20 日阻力位突破确认。",
                snapshot.resistance_level_20
            ),
            format!(
                "放量门槛基于 volume_ratio_20={:.2} 设定，避免无量突破误判。",
                snapshot.volume_ratio_20
            ),
            format!(
                "减仓触发价取自 ema_10={:.2}，用于识别短趋势承接是否失守。",
                snapshot.ema_10
            ),
            format!(
                "止损价取自 boll_middle={:.2}，用于监控强弱分界是否被跌破。",
                snapshot.boll_middle
            ),
            format!(
                "失效价取自 sma_50={:.2}，用于识别中期趋势是否正式破坏。",
                snapshot.sma_50
            ),
        ],
    }
}

// 2026-04-02 CST: 这里补一个价格四舍五入助手，原因是 execution_plan 里的阈值是给上层直接阅读和执行的，不适合暴露过长浮点尾数；
// 目的：把技术层原始数值统一整理成更稳定的交易价位显示，同时不改变其指标来源。
fn round_price(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

// 2026-04-02 CST: 这里补一个比例四舍五入助手，原因是量比与仓位比例属于执行阈值，过长小数会降低可读性与可执行性；
// 目的：让 execution_plan 输出稳定、易读，又不丢掉核心方向性信息。
fn round_ratio(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

// 2026-04-02 CST: 这里先补 committee payload 的最小装配，原因是第一阶段虽然还不做投票引擎，但必须先让投决端拿到稳定事实入口；
// 目的：确保咨询模式和投决模式共享同一份 symbol / date / evidence_version / risk digest，而不是各自重拼底稿。
fn build_committee_payload(
    analysis: &SecurityAnalysisResonanceResult,
    analysis_date: &str,
    summary: &str,
    evidence_version: &str,
    execution_plan: &ExecutionPlan,
) -> CommitteePayload {
    // 2026-04-02 CST: 这里先把综合结论压成 recommendation digest，原因是投决会后续必须只消费 committee payload 而不能回头扫 briefing 细节，
    // 目的：让 chair 与各 reviewer 在同一份结构化推荐事实上表态，避免继续围绕扁平摘要重复解释。
    let recommendation_digest = CommitteeRecommendationDigest {
        final_stance: analysis.base_analysis.integrated_conclusion.stance.clone(),
        action_bias: analysis.resonance_context.action_bias.clone(),
        summary: analysis
            .base_analysis
            .integrated_conclusion
            .headline
            .clone(),
        confidence: analysis
            .base_analysis
            .technical_context
            .stock_analysis
            .consultation_conclusion
            .confidence
            .clone(),
    };
    // 2026-04-02 CST: 这里把 execution_plan 显式复制进 committee digest，原因是 execution reviewer 需要读到确定阈值而不是再次推导，
    // 目的：把“何时加减仓、何时失效”的边界固定为投票输入，后续 CLI/Skill/GUI 都复用同一份执行事实。
    let execution_digest = CommitteeExecutionDigest {
        add_trigger_price: execution_plan.add_trigger_price,
        add_trigger_volume_ratio: execution_plan.add_trigger_volume_ratio,
        add_position_pct: execution_plan.add_position_pct,
        reduce_trigger_price: execution_plan.reduce_trigger_price,
        reduce_position_pct: execution_plan.reduce_position_pct,
        stop_loss_price: execution_plan.stop_loss_price,
        invalidation_price: execution_plan.invalidation_price,
        rejection_zone: execution_plan.rejection_zone.clone(),
        watch_points: execution_plan.watch_points.clone(),
        explanation: execution_plan.explanation.clone(),
    };
    // 2026-04-02 CST: 这里把共振上下文压缩成角色可直接消费的摘要，原因是投决层更关心驱动项、负向扰动和事件覆盖而不是底层评估细节，
    // 目的：在不泄露整层内部结构的前提下，保留足够的驱动解释能力支撑固定角色投票。
    let resonance_digest = CommitteeResonanceDigest {
        resonance_score: analysis.resonance_context.resonance_score,
        action_bias: analysis.resonance_context.action_bias.clone(),
        top_positive_driver_names: analysis
            .resonance_context
            .top_positive_resonances
            .iter()
            .map(|driver| driver.display_name.clone())
            .collect(),
        top_negative_driver_names: analysis
            .resonance_context
            .top_negative_resonances
            .iter()
            .map(|driver| driver.display_name.clone())
            .collect(),
        event_override_titles: analysis
            .resonance_context
            .event_overrides
            .iter()
            .map(|event| event.title.clone())
            .collect(),
    };
    // 2026-04-02 CST: 这里把 readiness 状态显式固化出来，原因是 risk officer 与 chair 需要先判断“是否具备正式表决条件”，
    // 目的：把各层是否齐备从隐式推断变成确定布尔信号，减少不同角色各自猜测事实边界。
    let evidence_checks = CommitteeEvidenceChecks {
        fundamental_ready: analysis.base_analysis.fundamental_context.status == "available",
        technical_ready: true,
        resonance_ready: true,
        execution_ready: true,
        briefing_ready: true,
    };
    // 2026-04-02 CST: 这里先把历史研究层以 unavailable 占位接入，原因是方案 B 允许先把 vote engine 跑通，再平滑接 signal outcome 增强，
    // 目的：先固定 committee payload 主合同，避免后续历史研究接入时再次破坏 briefing/vote 的事实边界。
    let historical_digest = build_historical_digest(&analysis.symbol, analysis_date);

    CommitteePayload {
        symbol: analysis.symbol.clone(),
        analysis_date: analysis_date.to_string(),
        recommended_action: analysis.resonance_context.action_bias.clone(),
        confidence: analysis
            .base_analysis
            .technical_context
            .stock_analysis
            .consultation_conclusion
            .confidence
            .clone(),
        key_risks: analysis
            .base_analysis
            .integrated_conclusion
            .risk_flags
            .clone(),
        minority_objection_points: analysis
            .resonance_context
            .top_negative_resonances
            .iter()
            .take(2)
            .map(|driver| format!("{} 存在负向共振或背离风险", driver.display_name))
            .collect(),
        evidence_version: evidence_version.to_string(),
        briefing_digest: summary.to_string(),
        committee_schema_version: "committee-payload:v1".to_string(),
        recommendation_digest,
        execution_digest,
        resonance_digest,
        evidence_checks,
        historical_digest,
    }
}

// 2026-04-02 CST: 这里把 briefing 默认投决建议正式收口成三种模式，原因是用户要求“默认就要投决会，并在报告里直接写出建议”，
// 目的：让个股报告、严格交易建议和已有持仓判断都能沿同一份 committee_payload 生成，而不是再由上层 Agent 手工改写。
fn build_committee_recommendations(
    committee_payload: &CommitteePayload,
) -> Result<CommitteeRecommendations, SecurityDecisionBriefingError> {
    Ok(CommitteeRecommendations {
        default_mode: "standard".to_string(),
        report_focus: "stock_analysis_report".to_string(),
        standard: build_committee_recommendation_entry(
            committee_payload,
            "standard",
            "个股分析报告默认投决会建议",
        )?,
        strict: build_committee_recommendation_entry(
            committee_payload,
            "strict",
            "涉及金额与买卖动作的严格交易建议",
        )?,
        advisory: build_committee_recommendation_entry(
            committee_payload,
            "advisory",
            "已有持仓判断与持仓处置建议",
        )?,
    })
}

// 2026-04-02 CST: 这里集中复用正式 vote Tool 生成 briefing 内嵌建议，原因是报告里的建议必须与独立 `security_committee_vote` 完全同口径，
// 目的：避免 report 层和 vote Tool 各自产出一份不同结论，重新引入用户刚刚明确反对的“双事实 / 双口径”问题。
fn build_committee_recommendation_entry(
    committee_payload: &CommitteePayload,
    committee_mode: &str,
    scenario: &str,
) -> Result<CommitteeRecommendationEntry, SecurityDecisionBriefingError> {
    let vote = security_committee_vote(&SecurityCommitteeVoteRequest {
        committee_payload: committee_payload.clone(),
        committee_mode: committee_mode.to_string(),
        meeting_id: Some(format!(
            "briefing-{}-{}",
            committee_payload.symbol, committee_mode
        )),
    })?;
    Ok(CommitteeRecommendationEntry {
        scenario: scenario.to_string(),
        vote,
    })
}

// 2026-04-02 CST: 这里把历史研究摘要正式接回 committee payload，原因是用户明确要求咨询和投决看到的信息必须一致，
// 不能一边说“有历史相似研究”，另一边 payload 还永远 unavailable；目的：让历史胜率、预期收益与回撤区间进入统一交付物。
fn build_historical_digest(symbol: &str, analysis_date: &str) -> CommitteeHistoricalDigest {
    match signal_outcome_research_summary(&SignalOutcomeResearchSummaryRequest {
        symbol: symbol.to_string(),
        snapshot_date: Some(analysis_date.to_string()),
        study_key: "bank_resonance_core_technical_v1".to_string(),
    }) {
        Ok(summary) => CommitteeHistoricalDigest {
            status: summary.status,
            historical_confidence: summary.historical_confidence,
            analog_sample_count: summary.analog_sample_count,
            analog_win_rate_10d: summary.analog_win_rate_10d,
            expected_return_window: summary.expected_return_window,
            expected_drawdown_window: summary.expected_drawdown_window,
            research_limitations: summary.research_limitations,
        },
        Err(_) => CommitteeHistoricalDigest {
            status: "unavailable".to_string(),
            historical_confidence: "unknown".to_string(),
            analog_sample_count: 0,
            analog_win_rate_10d: None,
            expected_return_window: None,
            expected_drawdown_window: None,
            research_limitations: vec![
                "历史研究摘要读取失败，当前按 unavailable 处理。".to_string(),
            ],
        },
    }
}

// 2026-04-02 CST: 这里先提供占位默认值函数，原因是请求合同在 assembler 和 dispatcher 接入前也需要可稳定反序列化，
// 目的：让合同层先独立成立，后续只做增量实现而不再回头拆字段默认规则。
fn default_lookback_days() -> usize {
    180
}

// 2026-04-02 CST: 这里把 factor lookback 的默认值先收口到 briefing 请求层，原因是 briefing 后续会统一协调 technical/fullstack/resonance 的观察窗口，
// 目的：避免调用方在正式接入前就必须显式传完整参数集合。
fn default_factor_lookback_days() -> usize {
    120
}

// 2026-04-02 CST: 这里把公告披露上限默认值也先钉在 briefing 请求层，原因是 briefing 未来要复用 fullstack 的信息面窗口而不是临时拼参数，
// 目的：让合同阶段就拥有稳定、可测试的默认行为。
fn default_disclosure_limit() -> usize {
    6
}
