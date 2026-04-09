use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

use crate::ops::stock::security_condition_review::SecurityConditionReviewDocument;
use crate::ops::stock::security_execution_record::{
    SecurityExecutionJournalDocument, SecurityExecutionJournalResult, SecurityExecutionRecordError,
    SecurityExecutionRecordRequest, SecurityExecutionRecordResult, SecurityExecutionTradeInput,
    SecurityPortfolioPositionPlanDocument, SecurityPositionPlanResult, security_execution_record,
};

// 2026-04-09 CST: 这里新增投后复盘请求合同，原因是 Task 8 要把“投前仓位建议 + 未来结果”正式收口成投后对象；
// 目的：让复盘 Tool 通过统一输入边界复用 position_plan 与 forward_outcome 主链，而不是继续由外层手工拼接。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityPostTradeReviewRequest {
    pub symbol: String,
    #[serde(default)]
    pub market_symbol: Option<String>,
    #[serde(default)]
    pub sector_symbol: Option<String>,
    pub market_regime: String,
    pub sector_template: String,
    #[serde(default)]
    pub market_profile: Option<String>,
    #[serde(default)]
    pub sector_profile: Option<String>,
    #[serde(default)]
    pub as_of_date: Option<String>,
    #[serde(default = "default_review_horizon_days")]
    pub review_horizon_days: usize,
    #[serde(default = "default_lookback_days")]
    pub lookback_days: usize,
    #[serde(default = "default_factor_lookback_days")]
    pub factor_lookback_days: usize,
    #[serde(default = "default_disclosure_limit")]
    pub disclosure_limit: usize,
    #[serde(default = "default_stop_loss_pct")]
    pub stop_loss_pct: f64,
    #[serde(default = "default_target_return_pct")]
    pub target_return_pct: f64,
    #[serde(default)]
    pub actual_entry_date: String,
    #[serde(default)]
    pub actual_entry_price: f64,
    #[serde(default)]
    pub actual_position_pct: f64,
    #[serde(default)]
    pub actual_exit_date: String,
    #[serde(default)]
    pub actual_exit_price: f64,
    #[serde(default)]
    pub exit_reason: String,
    #[serde(default)]
    pub execution_trades: Vec<SecurityExecutionTradeInput>,
    #[serde(default)]
    pub execution_journal_notes: Vec<String>,
    #[serde(default)]
    pub execution_record_notes: Vec<String>,
    #[serde(default)]
    pub condition_review_ref: Option<String>,
    #[serde(default)]
    pub condition_review_document: Option<SecurityConditionReviewDocument>,
    #[serde(default)]
    pub portfolio_position_plan_document: Option<SecurityPortfolioPositionPlanDocument>,
    #[serde(default = "default_created_at")]
    pub created_at: String,
}

// 2026-04-09 CST: 这里固化最小正式投后复盘文档，原因是平台要补齐“投后”这一段，而不能只停留在 forward_outcome 数值回填；
// 目的：把收益兑现、回撤、执行偏差与后续调整提示装配成一份可留痕、可审阅、可继续治理的正式对象。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityPostTradeReviewDocument {
    pub review_id: String,
    pub contract_version: String,
    pub document_type: String,
    pub generated_at: String,
    pub symbol: String,
    pub analysis_date: String,
    pub snapshot_date: String,
    pub review_horizon_days: usize,
    pub position_plan_ref: String,
    pub condition_review_ref: Option<String>,
    pub condition_review_trigger_type: Option<String>,
    pub condition_review_follow_up_action: Option<String>,
    pub condition_review_summary: Option<String>,
    pub condition_review_interpretation: Option<String>,
    pub snapshot_ref: String,
    pub outcome_ref: String,
    pub execution_journal_ref: String,
    pub execution_record_ref: String,
    pub planned_position: serde_json::Value,
    pub actual_result_window: String,
    pub realized_return: f64,
    pub executed_return: f64,
    pub max_drawdown_realized: f64,
    pub max_runup_realized: f64,
    pub thesis_status: String,
    pub execution_deviation: String,
    pub execution_return_gap: f64,
    pub account_plan_alignment: Option<String>,
    pub tranche_discipline: Option<String>,
    pub budget_drift_reason: Option<String>,
    pub model_miss_reason: String,
    pub next_account_adjustment_hint: Option<String>,
    pub next_adjustment_hint: String,
    pub review_summary: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SecurityPostTradeReviewResult {
    pub position_plan_result: SecurityPositionPlanResult,
    pub forward_outcome_result: SecurityPostTradeReviewOutcomeBinding,
    pub execution_journal_result: SecurityExecutionJournalResult,
    pub execution_journal: SecurityExecutionJournalDocument,
    pub execution_record_result: SecurityExecutionRecordResult,
    pub execution_record:
        crate::ops::stock::security_execution_record::SecurityExecutionRecordDocument,
    pub post_trade_review: SecurityPostTradeReviewDocument,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SecurityPostTradeReviewOutcomeBinding {
    pub snapshot: crate::ops::stock::security_feature_snapshot::SecurityFeatureSnapshot,
    pub selected_outcome:
        crate::ops::stock::security_forward_outcome::SecurityForwardOutcomeDocument,
    pub all_outcomes:
        Vec<crate::ops::stock::security_forward_outcome::SecurityForwardOutcomeDocument>,
}

#[derive(Debug, Error)]
pub enum SecurityPostTradeReviewError {
    #[error("security post trade review execution preparation failed: {0}")]
    ExecutionRecord(#[from] SecurityExecutionRecordError),
    #[error("security post trade review build failed: {0}")]
    Build(String),
}

pub fn security_post_trade_review(
    request: &SecurityPostTradeReviewRequest,
) -> Result<SecurityPostTradeReviewResult, SecurityPostTradeReviewError> {
    let execution_record_result = security_execution_record(&SecurityExecutionRecordRequest {
        symbol: request.symbol.clone(),
        market_symbol: request.market_symbol.clone(),
        sector_symbol: request.sector_symbol.clone(),
        market_regime: request.market_regime.clone(),
        sector_template: request.sector_template.clone(),
        market_profile: request.market_profile.clone(),
        sector_profile: request.sector_profile.clone(),
        as_of_date: request.as_of_date.clone(),
        review_horizon_days: request.review_horizon_days,
        lookback_days: request.lookback_days,
        factor_lookback_days: request.factor_lookback_days,
        disclosure_limit: request.disclosure_limit,
        stop_loss_pct: request.stop_loss_pct,
        target_return_pct: request.target_return_pct,
        actual_entry_date: request.actual_entry_date.clone(),
        actual_entry_price: request.actual_entry_price,
        actual_position_pct: request.actual_position_pct,
        actual_exit_date: request.actual_exit_date.clone(),
        actual_exit_price: request.actual_exit_price,
        exit_reason: request.exit_reason.clone(),
        execution_trades: request.execution_trades.clone(),
        execution_journal_notes: request.execution_journal_notes.clone(),
        execution_record_notes: request.execution_record_notes.clone(),
        condition_review_ref: request.condition_review_ref.clone(),
        condition_review_document: request.condition_review_document.clone(),
        portfolio_position_plan_document: request.portfolio_position_plan_document.clone(),
        created_at: request.created_at.clone(),
    })?;
    let outcome_binding = SecurityPostTradeReviewOutcomeBinding {
        snapshot: execution_record_result
            .forward_outcome_result
            .snapshot
            .clone(),
        selected_outcome: execution_record_result
            .forward_outcome_result
            .selected_outcome
            .clone(),
        all_outcomes: execution_record_result
            .forward_outcome_result
            .all_outcomes
            .clone(),
    };
    let post_trade_review =
        build_security_post_trade_review(&execution_record_result, &outcome_binding, request)?;

    Ok(SecurityPostTradeReviewResult {
        position_plan_result: execution_record_result.position_plan_result.clone(),
        forward_outcome_result: outcome_binding,
        execution_journal_result: execution_record_result.execution_journal_result.clone(),
        execution_journal: execution_record_result.execution_journal.clone(),
        execution_record_result: execution_record_result.clone(),
        execution_record: execution_record_result.execution_record.clone(),
        post_trade_review,
    })
}

// 2026-04-09 CST: 这里单独暴露投后复盘 builder，原因是后续 package / audit / replay 可能继续复用正式复盘文档装配；
// 目的：保持复盘规则集中，避免后续在多个 Tool 中各自拼 thesis_status 与调整提示。
pub fn build_security_post_trade_review(
    execution_record_result: &SecurityExecutionRecordResult,
    outcome_binding: &SecurityPostTradeReviewOutcomeBinding,
    request: &SecurityPostTradeReviewRequest,
) -> Result<SecurityPostTradeReviewDocument, SecurityPostTradeReviewError> {
    let position_plan_document = &execution_record_result
        .position_plan_result
        .position_plan_document;
    let selected_outcome = &outcome_binding.selected_outcome;
    let execution_record = &execution_record_result.execution_record;
    let thesis_status = classify_thesis_status(selected_outcome);
    let execution_deviation = execution_record.execution_quality.clone();
    // 2026-04-09 CST: 这里把 execution_record 的账户偏差继续上卷到 review，原因是方案A-2要求投后层给出正式治理语言；
    // 目的：让 review 直接落地账户计划对齐、分层纪律和后续动作提示，而不是只停留在收益/回撤描述。
    let account_plan_alignment = execution_record.account_budget_alignment.clone();
    let tranche_discipline = account_plan_alignment
        .as_ref()
        .map(|alignment| classify_tranche_discipline(alignment));
    let budget_drift_reason = account_plan_alignment
        .as_ref()
        .map(|alignment| derive_budget_drift_reason(alignment));
    let next_account_adjustment_hint = account_plan_alignment
        .as_ref()
        .map(|alignment| derive_next_account_adjustment_hint(alignment));
    let model_miss_reason =
        derive_model_miss_reason(selected_outcome, &thesis_status, &execution_deviation);
    let next_adjustment_hint = derive_next_adjustment_hint(
        &thesis_status,
        position_plan_document.position_risk_grade.as_str(),
        selected_outcome,
        &execution_deviation,
    );
    let planned_position = json!({
        "position_action": position_plan_document.position_action,
        "entry_mode": position_plan_document.entry_mode,
        "starter_position_pct": position_plan_document.starter_position_pct,
        "max_position_pct": position_plan_document.max_position_pct,
        "position_risk_grade": position_plan_document.position_risk_grade,
    });
    // 2026-04-10 CST: 这里把 execution 层沉淀的 condition_review 再提升成 review 解释语句，原因是 Task 5 要让投后复盘直接消费最近一次投中复核；
    // 目的：让 review 输出既保留正式 ref，也补上“为什么当时继续执行/为什么要改计划”的治理口径。
    let condition_review_interpretation =
        derive_condition_review_interpretation(execution_record);

    Ok(SecurityPostTradeReviewDocument {
        review_id: format!(
            "post-trade-review-{}-{}d",
            position_plan_document.position_plan_id, request.review_horizon_days
        ),
        contract_version: "security_post_trade_review.v1".to_string(),
        document_type: "security_post_trade_review".to_string(),
        generated_at: normalize_created_at(&request.created_at),
        symbol: position_plan_document.symbol.clone(),
        analysis_date: position_plan_document.analysis_date.clone(),
        snapshot_date: outcome_binding.snapshot.as_of_date.clone(),
        review_horizon_days: selected_outcome.horizon_days,
        position_plan_ref: position_plan_document.position_plan_id.clone(),
        condition_review_ref: execution_record.condition_review_ref.clone(),
        condition_review_trigger_type: execution_record.condition_review_trigger_type.clone(),
        condition_review_follow_up_action: execution_record
            .condition_review_follow_up_action
            .clone(),
        condition_review_summary: execution_record.condition_review_summary.clone(),
        condition_review_interpretation,
        snapshot_ref: outcome_binding.snapshot.snapshot_id.clone(),
        outcome_ref: selected_outcome.outcome_id.clone(),
        execution_journal_ref: execution_record.execution_journal_ref.clone(),
        execution_record_ref: execution_record.execution_record_id.clone(),
        planned_position,
        actual_result_window: format!("{}d", selected_outcome.horizon_days),
        realized_return: selected_outcome.forward_return,
        executed_return: execution_record.actual_return,
        max_drawdown_realized: selected_outcome.max_drawdown,
        max_runup_realized: selected_outcome.max_runup,
        thesis_status: thesis_status.clone(),
        execution_deviation,
        execution_return_gap: execution_record.execution_return_gap,
        account_plan_alignment,
        tranche_discipline,
        budget_drift_reason,
        model_miss_reason,
        next_account_adjustment_hint,
        next_adjustment_hint: next_adjustment_hint.clone(),
        review_summary: format!(
            "投后复盘显示 {} 在 {} 日窗口内计划收益 {:.2}%，真实执行收益 {:.2}%，最大回撤 {:.2}%，结论为 `{}`，执行偏差 `{}`，后续建议 `{}`。",
            position_plan_document.symbol,
            selected_outcome.horizon_days,
            selected_outcome.forward_return * 100.0,
            execution_record.actual_return * 100.0,
            selected_outcome.max_drawdown * 100.0,
            thesis_status,
            execution_record.execution_quality,
            next_adjustment_hint
        ),
    })
}

fn classify_thesis_status(
    selected_outcome: &crate::ops::stock::security_forward_outcome::SecurityForwardOutcomeDocument,
) -> String {
    if selected_outcome.hit_stop_first {
        "broken".to_string()
    } else if selected_outcome.forward_return > 0.0 && selected_outcome.max_drawdown <= 0.08 {
        "validated".to_string()
    } else if selected_outcome.forward_return > 0.0 {
        "mixed".to_string()
    } else {
        "broken".to_string()
    }
}

fn derive_model_miss_reason(
    selected_outcome: &crate::ops::stock::security_forward_outcome::SecurityForwardOutcomeDocument,
    thesis_status: &str,
    execution_deviation: &str,
) -> String {
    if execution_deviation == "adverse" && thesis_status == "validated" {
        return "execution_slippage_overrode_valid_thesis".to_string();
    }
    if thesis_status == "validated" {
        return "none".to_string();
    }
    if selected_outcome.hit_stop_first {
        return "stop_loss_triggered_before_thesis_played_out".to_string();
    }
    if selected_outcome.forward_return <= 0.0 {
        return "negative_forward_return_within_review_window".to_string();
    }
    "reward_realized_but_path_quality_weakened".to_string()
}

fn derive_condition_review_interpretation(
    execution_record: &crate::ops::stock::security_execution_record::SecurityExecutionRecordDocument,
) -> Option<String> {
    let condition_review_ref = execution_record.condition_review_ref.as_ref()?;
    let follow_up_action = execution_record
        .condition_review_follow_up_action
        .as_deref()
        .unwrap_or("unknown");
    let trigger_type = execution_record
        .condition_review_trigger_type
        .as_deref()
        .unwrap_or("unknown");
    let summary = execution_record
        .condition_review_summary
        .as_deref()
        .unwrap_or("未提供复核摘要");

    Some(format!(
        "最近一次条件复核 `{}` 来自 `{}`，给出的后续动作是 `{}`，复核摘要为：{}",
        condition_review_ref, trigger_type, follow_up_action, summary
    ))
}

fn derive_next_adjustment_hint(
    thesis_status: &str,
    position_risk_grade: &str,
    selected_outcome: &crate::ops::stock::security_forward_outcome::SecurityForwardOutcomeDocument,
    execution_deviation: &str,
) -> String {
    if execution_deviation == "adverse" {
        return "保留研究结论前，先收紧入场滑点、仓位偏差和退出纪律。".to_string();
    }
    match thesis_status {
        "validated" if selected_outcome.max_runup >= 0.10 => {
            "保留主假设，后续同类机会可维持原仓位框架。".to_string()
        }
        "validated" => "保留主假设，但后续仍需观察是否真正形成持续优势。".to_string(),
        "mixed" if position_risk_grade == "high" => {
            "降低同类高风险机会的初始仓位，并提高对回撤路径的约束。".to_string()
        }
        "mixed" => "保留方向判断，但后续应下调仓位或延后加仓确认。".to_string(),
        _ => "将同类情形降级处理，后续优先等待更强确认或直接缩小风险暴露。".to_string(),
    }
}

fn classify_tranche_discipline(account_plan_alignment: &str) -> String {
    match account_plan_alignment {
        "aligned" => "disciplined".to_string(),
        "under_budget" => "underfilled".to_string(),
        "over_budget" => "overfilled".to_string(),
        _ => "offside".to_string(),
    }
}

fn derive_budget_drift_reason(account_plan_alignment: &str) -> String {
    match account_plan_alignment {
        "aligned" => "none".to_string(),
        "under_budget" => "planned_tranche_not_fully_executed".to_string(),
        "over_budget" => "executed_tranche_exceeded_account_budget".to_string(),
        _ => "execution_direction_conflicted_with_account_plan".to_string(),
    }
}

fn derive_next_account_adjustment_hint(account_plan_alignment: &str) -> String {
    match account_plan_alignment {
        "aligned" => "账户层计划与执行一致，后续继续按既定预算和层数纪律推进。".to_string(),
        "under_budget" => {
            "若研究结论未变，下次应先确认未执行原因，再决定是否补齐原计划层数。".to_string()
        }
        "over_budget" => {
            "下次同类机会先回到计划层数，未重新通过账户预算复核前不要继续追加强度。".to_string()
        }
        _ => "先暂停沿用原账户动作，复核方向、预算与分层模板后再恢复执行。".to_string(),
    }
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

fn default_review_horizon_days() -> usize {
    20
}

fn default_lookback_days() -> usize {
    260
}

fn default_factor_lookback_days() -> usize {
    120
}

fn default_disclosure_limit() -> usize {
    6
}

fn default_stop_loss_pct() -> f64 {
    0.05
}

fn default_target_return_pct() -> f64 {
    0.12
}
