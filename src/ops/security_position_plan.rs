use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ops::stock::security_decision_briefing::{
    SecurityDecisionBriefingCore, SecurityDecisionBriefingError, SecurityDecisionBriefingRequest,
    security_decision_briefing_core,
};
use crate::ops::stock::stock_analysis_data_guard::StockAnalysisDateGuard;

// 2026-04-09 CST: 这里新增 security_position_plan 请求合同，原因是 Task 7 要把 briefing 里的仓位层提升为独立正式 Tool；
// 目的：复用同一份输入边界生成正式仓位文档，避免再维护第二套平行请求结构。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityPositionPlanRequest {
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
    #[serde(default = "default_created_at")]
    pub created_at: String,
}

// 2026-04-09 CST: 这里固化独立仓位计划文档，原因是用户要求投中层不能只停留在 briefing 的嵌套字段里；
// 目的：把推荐动作、赔率结论与仓位执行约束收敛成可被 Skill、投决会和后续 package 直接引用的正式对象。
// 2026-04-09 CST: 这里补入分层模板字段，原因是方案A-1要把“试仓 / 加仓 / 减仓模板”正式对象化；
// 目的：让单票层不只给 starter/max，还能明确告诉上层每一层怎么走、最多几层。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityPositionPlanDocument {
    pub position_plan_id: String,
    pub contract_version: String,
    pub document_type: String,
    pub generated_at: String,
    pub symbol: String,
    pub analysis_date: String,
    pub analysis_date_guard: StockAnalysisDateGuard,
    pub evidence_version: String,
    pub briefing_ref: String,
    pub committee_payload_ref: String,
    pub recommended_action: String,
    pub confidence: String,
    pub odds_grade: String,
    pub historical_confidence: String,
    pub confidence_grade: String,
    pub position_action: String,
    pub entry_mode: String,
    pub starter_position_pct: f64,
    pub max_position_pct: f64,
    #[serde(default)]
    pub entry_tranche_pct: f64,
    #[serde(default)]
    pub add_tranche_pct: f64,
    #[serde(default)]
    pub reduce_tranche_pct: f64,
    #[serde(default)]
    pub max_tranche_count: usize,
    #[serde(default)]
    pub tranche_template: String,
    #[serde(default)]
    pub tranche_trigger_rules: Vec<String>,
    #[serde(default)]
    pub cooldown_rule: String,
    pub add_on_trigger: String,
    pub reduce_on_trigger: String,
    pub hard_stop_trigger: String,
    pub liquidity_cap: String,
    pub position_risk_grade: String,
    pub regime_adjustment: String,
    pub execution_notes: Vec<String>,
    pub rationale: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SecurityPositionPlanResult {
    pub briefing_core: SecurityDecisionBriefingCore,
    pub position_plan_document: SecurityPositionPlanDocument,
}

#[derive(Debug, Error)]
pub enum SecurityPositionPlanError {
    #[error("security_position_plan briefing assembly failed: {0}")]
    Briefing(#[from] SecurityDecisionBriefingError),
}

pub fn security_position_plan(
    request: &SecurityPositionPlanRequest,
) -> Result<SecurityPositionPlanResult, SecurityPositionPlanError> {
    let briefing_request = SecurityDecisionBriefingRequest {
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
    let briefing_core = security_decision_briefing_core(&briefing_request)?;
    let position_plan_document = build_security_position_plan_document(&briefing_core, request);

    Ok(SecurityPositionPlanResult {
        briefing_core,
        position_plan_document,
    })
}

// 2026-04-09 CST: 这里单独暴露仓位文档 builder，原因是后续 package / committee / audit 可能需要复用正式仓位对象化逻辑；
// 目的：继续保持“briefing 为事实源、position_plan 为正式文档壳”的单向依赖，不把算法散落到多处。
pub fn build_security_position_plan_document(
    briefing_core: &SecurityDecisionBriefingCore,
    request: &SecurityPositionPlanRequest,
) -> SecurityPositionPlanDocument {
    let position_plan = &briefing_core.position_plan;
    let execution_plan = &briefing_core.execution_plan;
    let odds_brief = &briefing_core.odds_brief;
    let committee_payload = &briefing_core.committee_payload;
    let briefing_ref = briefing_core.evidence_version.clone();
    let committee_payload_ref = format!(
        "committee-payload:{}:{}",
        briefing_core.symbol, briefing_core.analysis_date
    );
    let entry_tranche_pct = position_plan.starter_position_pct;
    let add_tranche_pct = execution_plan.add_position_pct;
    let reduce_tranche_pct = execution_plan.reduce_position_pct;
    let max_tranche_count = derive_max_tranche_count(
        position_plan.starter_position_pct,
        position_plan.max_position_pct,
        execution_plan.add_position_pct,
    );

    SecurityPositionPlanDocument {
        position_plan_id: format!(
            "position-plan-{}-{}",
            briefing_core.symbol, briefing_core.analysis_date
        ),
        contract_version: "security_position_plan.v1".to_string(),
        document_type: "security_position_plan".to_string(),
        generated_at: normalize_created_at(&request.created_at),
        symbol: briefing_core.symbol.clone(),
        analysis_date: briefing_core.analysis_date.clone(),
        analysis_date_guard: briefing_core.analysis_date_guard.clone(),
        evidence_version: briefing_core.evidence_version.clone(),
        briefing_ref,
        committee_payload_ref,
        recommended_action: committee_payload.recommended_action.clone(),
        confidence: committee_payload.confidence.clone(),
        odds_grade: odds_brief.odds_grade.clone(),
        historical_confidence: odds_brief.historical_confidence.clone(),
        confidence_grade: odds_brief.confidence_grade.clone(),
        position_action: position_plan.position_action.clone(),
        entry_mode: position_plan.entry_mode.clone(),
        starter_position_pct: position_plan.starter_position_pct,
        max_position_pct: position_plan.max_position_pct,
        entry_tranche_pct,
        add_tranche_pct,
        reduce_tranche_pct,
        max_tranche_count,
        tranche_template: "starter_plus_adds".to_string(),
        tranche_trigger_rules: vec![
            format!(
                "首层按 {:.0}% 建立试仓，只在 `{}` 对应场景成立后执行。",
                entry_tranche_pct * 100.0,
                position_plan.entry_mode
            ),
            format!(
                "后续每层按 {:.0}% 推进，并以 `{}` 作为加仓确认。",
                add_tranche_pct * 100.0,
                position_plan.add_on_trigger
            ),
            format!(
                "若触发 `{}`，先按 {:.0}% 节奏减仓。",
                position_plan.reduce_on_trigger,
                reduce_tranche_pct * 100.0
            ),
        ],
        cooldown_rule: "同一交易日不连续执行两次同方向加仓，至少等待一个确认周期。".to_string(),
        add_on_trigger: position_plan.add_on_trigger.clone(),
        reduce_on_trigger: position_plan.reduce_on_trigger.clone(),
        hard_stop_trigger: position_plan.hard_stop_trigger.clone(),
        liquidity_cap: position_plan.liquidity_cap.clone(),
        position_risk_grade: position_plan.position_risk_grade.clone(),
        regime_adjustment: position_plan.regime_adjustment.clone(),
        execution_notes: position_plan.execution_notes.clone(),
        rationale: position_plan.rationale.clone(),
    }
}

fn derive_max_tranche_count(
    starter_position_pct: f64,
    max_position_pct: f64,
    add_tranche_pct: f64,
) -> usize {
    if starter_position_pct <= 0.0 || max_position_pct <= 0.0 {
        return 0;
    }
    if max_position_pct <= starter_position_pct || add_tranche_pct <= 0.0 {
        return 1;
    }
    let remaining = (max_position_pct - starter_position_pct).max(0.0);
    1 + (remaining / add_tranche_pct).ceil() as usize
}

fn normalize_created_at(value: &str) -> String {
    if value.trim().is_empty() {
        Utc::now().to_rfc3339()
    } else {
        value.trim().to_string()
    }
}

fn default_created_at() -> String {
    Utc::now().to_rfc3339()
}

fn default_lookback_days() -> usize {
    180
}

fn default_factor_lookback_days() -> usize {
    120
}

fn default_disclosure_limit() -> usize {
    6
}
