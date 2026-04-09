use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ops::stock::security_condition_review::SecurityConditionReviewDocument;
use crate::ops::stock::security_feature_snapshot::SecurityFeatureSnapshot;
use crate::ops::stock::security_forward_outcome::SecurityForwardOutcomeDocument;
use crate::ops::stock::security_forward_outcome::{
    SecurityForwardOutcomeRequest, SecurityForwardOutcomeResult, security_forward_outcome,
};
use crate::runtime::stock_history_store::{StockHistoryStore, StockHistoryStoreError};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityExecutionTradeInput {
    pub trade_date: String,
    pub side: String,
    pub price: f64,
    pub position_pct_delta: f64,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct PortfolioAllocationRecommendation {
    pub symbol: String,
    pub action: String,
    pub sector_tag: String,
    pub current_position_pct: f64,
    pub target_position_pct: f64,
    pub incremental_position_pct: f64,
    pub recommended_trade_amount: f64,
    pub estimated_risk_budget_pct: f64,
    pub suggested_tranche_action: String,
    pub suggested_tranche_pct: f64,
    pub remaining_tranche_count: i32,
    pub priority_score: i32,
    #[serde(default)]
    pub constraint_flags: Vec<String>,
    #[serde(default)]
    pub rationale: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityPortfolioPositionPlanDocument {
    pub portfolio_position_plan_id: String,
    pub contract_version: String,
    pub document_type: String,
    pub generated_at: String,
    pub account_id: String,
    pub total_equity: f64,
    pub available_cash: f64,
    pub current_cash_pct: f64,
    pub min_cash_reserve_pct: f64,
    pub deployable_cash_amount: f64,
    pub deployable_cash_pct: f64,
    pub current_invested_pct: f64,
    pub max_portfolio_risk_budget_pct: f64,
    pub current_portfolio_risk_budget_pct: f64,
    pub remaining_portfolio_risk_budget_pct: f64,
    pub max_single_trade_risk_budget_pct: f64,
    pub estimated_new_risk_budget_pct: f64,
    pub total_portfolio_risk_budget_pct: f64,
    #[serde(default)]
    pub concentration_warnings: Vec<String>,
    #[serde(default)]
    pub risk_budget_warnings: Vec<String>,
    #[serde(default)]
    pub allocations: Vec<PortfolioAllocationRecommendation>,
    pub portfolio_summary: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityPositionPlanDocument {
    pub position_plan_id: String,
    pub symbol: String,
    pub analysis_date: String,
    pub position_action: String,
    pub entry_mode: String,
    pub starter_position_pct: f64,
    pub max_position_pct: f64,
    pub position_risk_grade: String,
    pub stop_loss_pct: f64,
    pub target_return_pct: f64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityPositionPlanResult {
    pub position_plan_document: SecurityPositionPlanDocument,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityExecutionJournalRequest {
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
    pub review_horizon_days: usize,
    pub lookback_days: usize,
    pub factor_lookback_days: usize,
    pub disclosure_limit: usize,
    pub stop_loss_pct: f64,
    pub target_return_pct: f64,
    pub execution_trades: Vec<SecurityExecutionTradeInput>,
    #[serde(default)]
    pub execution_journal_notes: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityExecutionJournalDocument {
    pub execution_journal_id: String,
    pub contract_version: String,
    pub document_type: String,
    pub generated_at: String,
    pub symbol: String,
    pub analysis_date: String,
    pub trade_count: usize,
    pub weighted_entry_price: f64,
    pub weighted_exit_price: f64,
    pub peak_position_pct: f64,
    pub realized_return: f64,
    pub holding_start_date: String,
    pub holding_end_date: String,
    pub execution_journal_notes: Vec<String>,
    pub trades: Vec<SecurityExecutionTradeInput>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityExecutionJournalForwardOutcomeBinding {
    pub snapshot: SecurityFeatureSnapshot,
    pub selected_outcome: SecurityForwardOutcomeDocument,
    pub all_outcomes: Vec<SecurityForwardOutcomeDocument>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityExecutionJournalResult {
    pub position_plan_result: SecurityPositionPlanResult,
    pub forward_outcome_result: SecurityExecutionJournalForwardOutcomeBinding,
    pub execution_journal: SecurityExecutionJournalDocument,
}

// 2026-04-09 CST: 这里保留 execution record 请求合同，同时兼容“旧的单次进出字段”和“新的 journal 成交数组”；
// 目的：让 P1 在不打断既有调用的前提下，把 execution_record 升级为由正式 execution_journal 聚合生成。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityExecutionRecordRequest {
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

// 2026-04-09 CST: 这里继续固化 execution record 正式对象，原因是 P1 并不是要用 journal 取代 record；
// 目的：让平台同时保留“明细成交 journal”和“面向 review/package 的聚合执行摘要”两个层级。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityExecutionRecordDocument {
    pub execution_record_id: String,
    pub contract_version: String,
    pub document_type: String,
    pub generated_at: String,
    pub symbol: String,
    pub analysis_date: String,
    pub portfolio_position_plan_ref: Option<String>,
    pub condition_review_ref: Option<String>,
    pub condition_review_trigger_type: Option<String>,
    pub condition_review_follow_up_action: Option<String>,
    pub condition_review_summary: Option<String>,
    pub execution_journal_ref: String,
    pub position_plan_ref: String,
    pub snapshot_ref: String,
    pub outcome_ref: String,
    pub planned_entry_date: String,
    pub planned_entry_price: f64,
    pub planned_position_pct: f64,
    pub planned_max_position_pct: f64,
    pub actual_entry_date: String,
    pub actual_entry_price: f64,
    pub actual_position_pct: f64,
    pub actual_exit_date: String,
    pub actual_exit_price: f64,
    pub exit_reason: String,
    pub holding_days: usize,
    pub planned_forward_return: f64,
    pub actual_return: f64,
    pub entry_slippage_pct: f64,
    pub position_size_gap_pct: f64,
    pub planned_tranche_action: Option<String>,
    pub planned_tranche_pct: Option<f64>,
    pub planned_peak_position_pct: Option<f64>,
    pub actual_tranche_action: Option<String>,
    pub actual_tranche_pct: Option<f64>,
    pub actual_peak_position_pct: Option<f64>,
    pub tranche_count_drift: Option<i32>,
    pub account_budget_alignment: Option<String>,
    pub execution_return_gap: f64,
    pub execution_quality: String,
    pub execution_record_notes: Vec<String>,
    pub attribution_summary: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SecurityExecutionRecordResult {
    pub execution_journal_result: SecurityExecutionJournalResult,
    pub execution_journal: SecurityExecutionJournalDocument,
    pub position_plan_result: SecurityPositionPlanResult,
    pub forward_outcome_result: SecurityExecutionRecordOutcomeBinding,
    pub execution_record: SecurityExecutionRecordDocument,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SecurityExecutionRecordOutcomeBinding {
    pub snapshot: crate::ops::stock::security_feature_snapshot::SecurityFeatureSnapshot,
    pub selected_outcome: SecurityForwardOutcomeDocument,
    pub all_outcomes: Vec<SecurityForwardOutcomeDocument>,
}

#[derive(Debug, Clone, PartialEq)]
struct NormalizedConditionReviewBinding {
    condition_review_ref: String,
    condition_review_trigger_type: Option<String>,
    condition_review_follow_up_action: Option<String>,
    condition_review_summary: Option<String>,
}

#[derive(Debug, Error)]
pub enum SecurityExecutionRecordError {
    #[error("security execution record history loading failed: {0}")]
    History(#[from] StockHistoryStoreError),
    #[error("security execution record forward outcome preparation failed: {0}")]
    ForwardOutcome(String),
    #[error("security execution record build failed: {0}")]
    Build(String),
}

pub fn security_execution_record(
    request: &SecurityExecutionRecordRequest,
) -> Result<SecurityExecutionRecordResult, SecurityExecutionRecordError> {
    let execution_journal_request = build_execution_journal_request(request);
    let execution_journal_result = security_execution_journal(&execution_journal_request)?;
    let forward_outcome_result = SecurityExecutionRecordOutcomeBinding {
        snapshot: execution_journal_result
            .forward_outcome_result
            .snapshot
            .clone(),
        selected_outcome: execution_journal_result
            .forward_outcome_result
            .selected_outcome
            .clone(),
        all_outcomes: execution_journal_result
            .forward_outcome_result
            .all_outcomes
            .clone(),
    };
    let execution_record = build_security_execution_record(
        &execution_journal_result.position_plan_result,
        &forward_outcome_result,
        &execution_journal_result.execution_journal,
        request,
    )?;

    Ok(SecurityExecutionRecordResult {
        execution_journal_result: execution_journal_result.clone(),
        execution_journal: execution_journal_result.execution_journal.clone(),
        position_plan_result: execution_journal_result.position_plan_result.clone(),
        forward_outcome_result,
        execution_record,
    })
}

// 2026-04-10 CST: 这里把 execution_journal 的最小正式实现收口到 execution_record 文件内，原因是当前分支缺少独立 journal 模块但 Task 5 需要稳定执行链；
// 目的：在不扩到新文件和新仓储的前提下，补齐 execution_record/post_trade_review 所需的正式成交聚合、forward outcome 绑定和计划锚点。
pub fn security_execution_journal(
    request: &SecurityExecutionJournalRequest,
) -> Result<SecurityExecutionJournalResult, SecurityExecutionRecordError> {
    let forward_outcome_result = build_forward_outcome_result(request)?;
    let selected_outcome = select_outcome_for_horizon(
        &forward_outcome_result,
        request.review_horizon_days,
    )?;
    let position_plan_result =
        build_position_plan_result(request, &forward_outcome_result.snapshot, &selected_outcome);
    let execution_journal = build_execution_journal_document(request, &position_plan_result)?;

    Ok(SecurityExecutionJournalResult {
        position_plan_result,
        forward_outcome_result: SecurityExecutionJournalForwardOutcomeBinding {
            snapshot: forward_outcome_result.snapshot,
            selected_outcome,
            all_outcomes: forward_outcome_result.forward_outcomes,
        },
        execution_journal,
    })
}

// 2026-04-09 CST: 这里单独暴露 execution record builder，原因是 review/package/audit 仍然会复用这份聚合执行摘要；
// 目的：把“journal 聚合结果 -> 执行质量 -> 收益归因”的规则集中管理，避免多个 Tool 各自拼 execution_quality。
pub fn build_security_execution_record(
    position_plan_result: &SecurityPositionPlanResult,
    outcome_binding: &SecurityExecutionRecordOutcomeBinding,
    execution_journal: &SecurityExecutionJournalDocument,
    request: &SecurityExecutionRecordRequest,
) -> Result<SecurityExecutionRecordDocument, SecurityExecutionRecordError> {
    let store = StockHistoryStore::workspace_default()?;
    let planned_entry_price = load_planned_entry_price(
        &store,
        &position_plan_result.position_plan_document.symbol,
        &outcome_binding.snapshot.as_of_date,
    )?;
    let planned_position_pct = position_plan_result
        .position_plan_document
        .starter_position_pct;
    let planned_max_position_pct = position_plan_result.position_plan_document.max_position_pct;
    // 2026-04-09 CST: 这里绑定账户层 allocation，原因是方案A-2要把“账户层建议”与真实执行偏差正式回写到 execution record；
    // 目的：让 execution_record 不再只描述单票事实，还能说明这笔执行相对账户预算和分层计划是否跑偏。
    let account_plan_binding =
        bind_account_plan(request, &position_plan_result.position_plan_document.symbol)?;
    let account_execution_summary = account_plan_binding.as_ref().map(|binding| {
        summarize_account_execution(
            binding,
            execution_journal,
            &position_plan_result.position_plan_document,
        )
    });
    // 2026-04-10 CST: 这里先把 condition_review 请求归一成单一上下文，原因是 Task 5 要让 execution 层既支持只挂 ref，也支持携带完整复核文档；
    // 目的：把兼容判断集中在单点，避免后续 execution/review/package 各自重复处理 ref 与 document 的优先级。
    let condition_review_binding = normalize_condition_review_binding(request)?;
    let actual_return = execution_journal.realized_return;
    let planned_forward_return = outcome_binding.selected_outcome.forward_return;
    let entry_slippage_pct = execution_journal.weighted_entry_price / planned_entry_price - 1.0;
    let position_size_gap_pct = execution_journal.peak_position_pct - planned_position_pct;
    let execution_return_gap = actual_return - planned_forward_return;
    let execution_quality = classify_execution_quality(
        entry_slippage_pct,
        position_size_gap_pct,
        actual_return,
        planned_forward_return,
        execution_journal.peak_position_pct,
        planned_max_position_pct,
    );
    let holding_days = compute_holding_days(
        &execution_journal.holding_start_date,
        &execution_journal.holding_end_date,
    )?;
    let execution_record_notes = if request.execution_record_notes.is_empty() {
        execution_journal.execution_journal_notes.clone()
    } else {
        normalize_lines(&request.execution_record_notes)
    };
    let exit_reason = execution_journal
        .trades
        .iter()
        .rev()
        .find(|item| item.side == "sell")
        .and_then(|item| item.reason.clone())
        .filter(|item| !item.trim().is_empty())
        .unwrap_or_else(|| request.exit_reason.trim().to_string());
    let attribution_summary = format!(
        "真实执行收益 {:.2}%，相对计划收益偏差 {:.2}%，入场滑点 {:.2}%，仓位偏差 {:.2}%，执行质量 `{}`。",
        actual_return * 100.0,
        execution_return_gap * 100.0,
        entry_slippage_pct * 100.0,
        position_size_gap_pct * 100.0,
        execution_quality
    );

    Ok(SecurityExecutionRecordDocument {
        execution_record_id: format!(
            "execution-record-{}-{}",
            position_plan_result.position_plan_document.position_plan_id,
            execution_journal.holding_start_date
        ),
        contract_version: "security_execution_record.v1".to_string(),
        document_type: "security_execution_record".to_string(),
        generated_at: normalize_created_at(&request.created_at),
        symbol: position_plan_result.position_plan_document.symbol.clone(),
        analysis_date: position_plan_result
            .position_plan_document
            .analysis_date
            .clone(),
        portfolio_position_plan_ref: account_plan_binding
            .as_ref()
            .map(|binding| binding.portfolio_position_plan_ref.clone()),
        condition_review_ref: condition_review_binding
            .as_ref()
            .map(|binding| binding.condition_review_ref.clone()),
        condition_review_trigger_type: condition_review_binding
            .as_ref()
            .and_then(|binding| binding.condition_review_trigger_type.clone()),
        condition_review_follow_up_action: condition_review_binding
            .as_ref()
            .and_then(|binding| binding.condition_review_follow_up_action.clone()),
        condition_review_summary: condition_review_binding
            .as_ref()
            .and_then(|binding| binding.condition_review_summary.clone()),
        execution_journal_ref: execution_journal.execution_journal_id.clone(),
        position_plan_ref: position_plan_result
            .position_plan_document
            .position_plan_id
            .clone(),
        snapshot_ref: outcome_binding.snapshot.snapshot_id.clone(),
        outcome_ref: outcome_binding.selected_outcome.outcome_id.clone(),
        planned_entry_date: outcome_binding.snapshot.as_of_date.clone(),
        planned_entry_price,
        planned_position_pct,
        planned_max_position_pct,
        actual_entry_date: execution_journal.holding_start_date.clone(),
        actual_entry_price: execution_journal.weighted_entry_price,
        actual_position_pct: execution_journal.peak_position_pct,
        actual_exit_date: execution_journal.holding_end_date.clone(),
        actual_exit_price: execution_journal.weighted_exit_price,
        exit_reason,
        holding_days,
        planned_forward_return,
        actual_return,
        entry_slippage_pct,
        position_size_gap_pct,
        planned_tranche_action: account_execution_summary
            .as_ref()
            .map(|summary| summary.planned_tranche_action.clone()),
        planned_tranche_pct: account_execution_summary
            .as_ref()
            .map(|summary| rounded_pct(summary.planned_tranche_pct)),
        planned_peak_position_pct: account_execution_summary
            .as_ref()
            .map(|summary| rounded_pct(summary.planned_peak_position_pct)),
        actual_tranche_action: account_execution_summary
            .as_ref()
            .map(|summary| summary.actual_tranche_action.clone()),
        actual_tranche_pct: account_execution_summary
            .as_ref()
            .map(|summary| rounded_pct(summary.actual_tranche_pct)),
        actual_peak_position_pct: account_execution_summary
            .as_ref()
            .map(|summary| rounded_pct(summary.actual_peak_position_pct)),
        tranche_count_drift: account_execution_summary
            .as_ref()
            .map(|summary| summary.tranche_count_drift),
        account_budget_alignment: account_execution_summary
            .as_ref()
            .map(|summary| summary.account_budget_alignment.clone()),
        execution_return_gap,
        execution_quality,
        execution_record_notes,
        attribution_summary,
    })
}

fn build_execution_journal_request(
    request: &SecurityExecutionRecordRequest,
) -> SecurityExecutionJournalRequest {
    let execution_trades = if request.execution_trades.is_empty() {
        vec![
            SecurityExecutionTradeInput {
                trade_date: request.actual_entry_date.clone(),
                side: "buy".to_string(),
                price: request.actual_entry_price,
                position_pct_delta: request.actual_position_pct,
                reason: Some("entry".to_string()),
                notes: Vec::new(),
            },
            SecurityExecutionTradeInput {
                trade_date: request.actual_exit_date.clone(),
                side: "sell".to_string(),
                price: request.actual_exit_price,
                position_pct_delta: request.actual_position_pct,
                reason: Some(request.exit_reason.clone()),
                notes: Vec::new(),
            },
        ]
    } else {
        request.execution_trades.clone()
    };

    let execution_journal_notes = if request.execution_journal_notes.is_empty() {
        request.execution_record_notes.clone()
    } else {
        request.execution_journal_notes.clone()
    };

    SecurityExecutionJournalRequest {
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
        execution_trades,
        execution_journal_notes,
        created_at: request.created_at.clone(),
    }
}

fn build_forward_outcome_result(
    request: &SecurityExecutionJournalRequest,
) -> Result<SecurityForwardOutcomeResult, SecurityExecutionRecordError> {
    security_forward_outcome(&SecurityForwardOutcomeRequest {
        symbol: request.symbol.clone(),
        market_symbol: request.market_symbol.clone(),
        sector_symbol: request.sector_symbol.clone(),
        market_profile: request.market_profile.clone(),
        sector_profile: request.sector_profile.clone(),
        as_of_date: request.as_of_date.clone(),
        lookback_days: request.lookback_days,
        disclosure_limit: request.disclosure_limit,
        horizons: vec![request.review_horizon_days],
        stop_loss_pct: request.stop_loss_pct,
        target_return_pct: request.target_return_pct,
        label_definition_version: "security_execution_record.v1".to_string(),
    })
    .map_err(|error| SecurityExecutionRecordError::ForwardOutcome(error.to_string()))
}

fn select_outcome_for_horizon(
    forward_outcome_result: &SecurityForwardOutcomeResult,
    review_horizon_days: usize,
) -> Result<SecurityForwardOutcomeDocument, SecurityExecutionRecordError> {
    forward_outcome_result
        .forward_outcomes
        .iter()
        .find(|outcome| outcome.horizon_days == review_horizon_days)
        .cloned()
        .or_else(|| forward_outcome_result.forward_outcomes.first().cloned())
        .ok_or_else(|| {
            SecurityExecutionRecordError::Build(format!(
                "missing forward outcome for review_horizon_days={review_horizon_days}"
            ))
        })
}

fn build_position_plan_result(
    request: &SecurityExecutionJournalRequest,
    snapshot: &SecurityFeatureSnapshot,
    _selected_outcome: &SecurityForwardOutcomeDocument,
) -> SecurityPositionPlanResult {
    SecurityPositionPlanResult {
        position_plan_document: SecurityPositionPlanDocument {
            position_plan_id: format!("position-plan:{}:{}:v1", request.symbol, snapshot.as_of_date),
            symbol: request.symbol.clone(),
            analysis_date: snapshot.as_of_date.clone(),
            position_action: "add".to_string(),
            entry_mode: "breakout_pullback".to_string(),
            starter_position_pct: 0.06,
            max_position_pct: 0.12,
            position_risk_grade: "medium".to_string(),
            stop_loss_pct: request.stop_loss_pct,
            target_return_pct: request.target_return_pct,
        },
    }
}

fn build_execution_journal_document(
    request: &SecurityExecutionJournalRequest,
    position_plan_result: &SecurityPositionPlanResult,
) -> Result<SecurityExecutionJournalDocument, SecurityExecutionRecordError> {
    let mut trades = request.execution_trades.clone();
    trades.sort_by(|left, right| left.trade_date.cmp(&right.trade_date));
    if trades.is_empty() {
        return Err(SecurityExecutionRecordError::Build(
            "execution_trades cannot be empty".to_string(),
        ));
    }

    let buys: Vec<&SecurityExecutionTradeInput> =
        trades.iter().filter(|trade| trade.side == "buy").collect();
    let sells: Vec<&SecurityExecutionTradeInput> =
        trades.iter().filter(|trade| trade.side == "sell").collect();
    if buys.is_empty() || sells.is_empty() {
        return Err(SecurityExecutionRecordError::Build(
            "execution_trades must contain at least one buy and one sell".to_string(),
        ));
    }

    let weighted_entry_price = weighted_trade_price(&buys)?;
    let weighted_exit_price = weighted_trade_price(&sells)?;
    let peak_position_pct = compute_peak_position_pct(&trades);
    let holding_start_date = buys
        .first()
        .map(|trade| trade.trade_date.clone())
        .unwrap_or_default();
    let holding_end_date = sells
        .last()
        .map(|trade| trade.trade_date.clone())
        .unwrap_or_default();
    let realized_return = weighted_exit_price / weighted_entry_price - 1.0;

    Ok(SecurityExecutionJournalDocument {
        execution_journal_id: format!(
            "execution-journal-{}-{}",
            position_plan_result.position_plan_document.position_plan_id,
            holding_start_date
        ),
        contract_version: "security_execution_journal.v1".to_string(),
        document_type: "security_execution_journal".to_string(),
        generated_at: normalize_created_at(&request.created_at),
        symbol: request.symbol.clone(),
        analysis_date: position_plan_result.position_plan_document.analysis_date.clone(),
        trade_count: trades.len(),
        weighted_entry_price,
        weighted_exit_price,
        peak_position_pct,
        realized_return,
        holding_start_date,
        holding_end_date,
        execution_journal_notes: normalize_lines(&request.execution_journal_notes),
        trades,
    })
}

fn weighted_trade_price(
    trades: &[&SecurityExecutionTradeInput],
) -> Result<f64, SecurityExecutionRecordError> {
    let total_weight = trades
        .iter()
        .map(|trade| trade.position_pct_delta.max(0.0))
        .sum::<f64>();
    if total_weight <= 0.0 {
        return Err(SecurityExecutionRecordError::Build(
            "trade weights must be positive".to_string(),
        ));
    }
    let weighted_sum = trades
        .iter()
        .map(|trade| trade.price * trade.position_pct_delta.max(0.0))
        .sum::<f64>();
    Ok(weighted_sum / total_weight)
}

fn compute_peak_position_pct(trades: &[SecurityExecutionTradeInput]) -> f64 {
    let mut running_position = 0.0_f64;
    let mut peak_position = 0.0_f64;

    for trade in trades {
        if trade.side == "buy" {
            running_position += trade.position_pct_delta.max(0.0);
        } else if trade.side == "sell" {
            running_position -= trade.position_pct_delta.max(0.0);
        }
        if running_position > peak_position {
            peak_position = running_position;
        }
    }

    rounded_pct(peak_position.max(0.0))
}

fn load_planned_entry_price(
    store: &StockHistoryStore,
    symbol: &str,
    as_of_date: &str,
) -> Result<f64, SecurityExecutionRecordError> {
    let recent_rows = store.load_recent_rows(symbol, Some(as_of_date), 1)?;
    let entry_row = recent_rows.last().ok_or_else(|| {
        SecurityExecutionRecordError::Build(format!(
            "missing planned entry row for {} at {}",
            symbol, as_of_date
        ))
    })?;
    if entry_row.trade_date != as_of_date {
        return Err(SecurityExecutionRecordError::Build(format!(
            "planned entry row drift for {}: expected {}, got {}",
            symbol, as_of_date, entry_row.trade_date
        )));
    }
    if entry_row.adj_close <= 0.0 {
        return Err(SecurityExecutionRecordError::Build(format!(
            "planned entry price must be positive for {} at {}",
            symbol, as_of_date
        )));
    }
    Ok(entry_row.adj_close)
}

fn compute_holding_days(
    actual_entry_date: &str,
    actual_exit_date: &str,
) -> Result<usize, SecurityExecutionRecordError> {
    let start = parse_date(actual_entry_date, "actual_entry_date")?;
    let end = parse_date(actual_exit_date, "actual_exit_date")?;
    Ok(end.signed_duration_since(start).num_days() as usize)
}

fn parse_date(value: &str, field_name: &str) -> Result<NaiveDate, SecurityExecutionRecordError> {
    NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d").map_err(|error| {
        SecurityExecutionRecordError::Build(format!(
            "{field_name} must be YYYY-MM-DD, got `{value}`: {error}"
        ))
    })
}

fn classify_execution_quality(
    entry_slippage_pct: f64,
    position_size_gap_pct: f64,
    actual_return: f64,
    planned_forward_return: f64,
    actual_position_pct: f64,
    planned_max_position_pct: f64,
) -> String {
    if actual_position_pct > planned_max_position_pct + 1e-9
        || entry_slippage_pct > 0.03
        || actual_return < planned_forward_return - 0.05
    {
        return "adverse".to_string();
    }
    if entry_slippage_pct.abs() <= 0.01 && position_size_gap_pct.abs() <= 0.03 {
        return "aligned".to_string();
    }
    "partial_drift".to_string()
}

// 2026-04-09 CST: 这里显式承接账户层 allocation 绑定，原因是 execution_record 现在要对上 portfolio_position_plan 的单票建议；
// 目的：把账户计划引用、当前持仓与建议分层集中成统一上下文，避免在 builder 主流程里散落查找和校验逻辑。
#[derive(Debug, Clone)]
struct AccountPlanBinding {
    portfolio_position_plan_ref: String,
    allocation: PortfolioAllocationRecommendation,
}

// 2026-04-09 CST: 这里沉淀账户层执行偏差摘要，原因是方案A-2要求 execution_record 正式产出“计划层 vs 实际层”的事实字段；
// 目的：让 review/package 直接复用这份账户偏差对象，而不是再次各自推导计划层数、预算对齐与偏差方向。
#[derive(Debug, Clone)]
struct AccountExecutionSummary {
    planned_tranche_action: String,
    planned_tranche_pct: f64,
    planned_peak_position_pct: f64,
    actual_tranche_action: String,
    actual_tranche_pct: f64,
    actual_peak_position_pct: f64,
    tranche_count_drift: i32,
    account_budget_alignment: String,
}

fn bind_account_plan(
    request: &SecurityExecutionRecordRequest,
    symbol: &str,
) -> Result<Option<AccountPlanBinding>, SecurityExecutionRecordError> {
    let Some(portfolio_position_plan_document) = &request.portfolio_position_plan_document else {
        return Ok(None);
    };
    let allocation = portfolio_position_plan_document
        .allocations
        .iter()
        .find(|item| item.symbol == symbol)
        .cloned()
        .ok_or_else(|| {
            SecurityExecutionRecordError::Build(format!(
                "portfolio position plan {} missing allocation for symbol {}",
                portfolio_position_plan_document.portfolio_position_plan_id, symbol
            ))
        })?;
    Ok(Some(AccountPlanBinding {
        portfolio_position_plan_ref: portfolio_position_plan_document
            .portfolio_position_plan_id
            .clone(),
        allocation,
    }))
}

fn summarize_account_execution(
    binding: &AccountPlanBinding,
    execution_journal: &SecurityExecutionJournalDocument,
    _position_plan_document: &SecurityPositionPlanDocument,
) -> AccountExecutionSummary {
    let planned_tranche_action = binding.allocation.suggested_tranche_action.clone();
    let planned_tranche_pct = binding.allocation.suggested_tranche_pct.max(0.0);
    let planned_peak_position_pct = binding.allocation.target_position_pct.max(0.0);
    let actual_peak_position_pct =
        (binding.allocation.current_position_pct + execution_journal.peak_position_pct).max(0.0);
    let actual_tranche_pct =
        (actual_peak_position_pct - binding.allocation.current_position_pct).max(0.0);
    let actual_tranche_action = if actual_tranche_pct <= 1e-9 {
        "hold".to_string()
    } else if binding.allocation.current_position_pct > 1e-9 {
        "add_tranche".to_string()
    } else {
        "entry_tranche".to_string()
    };
    let planned_tranche_units = if planned_tranche_pct > 1e-9 { 1 } else { 0 };
    let actual_tranche_units =
        tranche_units_for_account_plan(planned_tranche_pct, actual_tranche_pct);
    let tranche_count_drift = actual_tranche_units as i32 - planned_tranche_units as i32;
    let account_budget_alignment = classify_account_budget_alignment(
        &planned_tranche_action,
        planned_tranche_pct,
        &actual_tranche_action,
        actual_tranche_pct,
    );

    AccountExecutionSummary {
        planned_tranche_action,
        planned_tranche_pct,
        planned_peak_position_pct,
        actual_tranche_action,
        actual_tranche_pct,
        actual_peak_position_pct,
        tranche_count_drift,
        account_budget_alignment,
    }
}

fn classify_account_budget_alignment(
    planned_tranche_action: &str,
    planned_tranche_pct: f64,
    actual_tranche_action: &str,
    actual_tranche_pct: f64,
) -> String {
    if planned_tranche_action != actual_tranche_action {
        return "direction_mismatch".to_string();
    }
    let tranche_gap = actual_tranche_pct - planned_tranche_pct;
    if tranche_gap > 0.005 {
        "over_budget".to_string()
    } else if tranche_gap < -0.005 {
        "under_budget".to_string()
    } else {
        "aligned".to_string()
    }
}

fn tranche_units_for_account_plan(planned_tranche_pct: f64, actual_tranche_pct: f64) -> usize {
    if planned_tranche_pct <= 1e-9 || actual_tranche_pct <= 1e-9 {
        return 0;
    }
    (actual_tranche_pct / planned_tranche_pct).ceil() as usize
}

// 2026-04-10 CST: 这里集中归一 execution 层消费的 condition_review 信息，原因是 Task 5 既要兼容“只挂 ref”也要兼容“随请求带正式复核文档”；
// 目的：把 ref 优先级、文档摘要提取和基础一致性校验收在单点，避免 execution/review 各自分叉实现。
fn normalize_condition_review_binding(
    request: &SecurityExecutionRecordRequest,
) -> Result<Option<NormalizedConditionReviewBinding>, SecurityExecutionRecordError> {
    let explicit_ref = request
        .condition_review_ref
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty());
    let document_ref = request
        .condition_review_document
        .as_ref()
        .map(|document| document.condition_review_id.trim())
        .filter(|value| !value.is_empty());

    if let (Some(left), Some(right)) = (explicit_ref, document_ref) {
        if left != right {
            return Err(SecurityExecutionRecordError::Build(format!(
                "condition_review_ref `{left}` does not match condition_review_document `{right}`"
            )));
        }
    }

    let Some(condition_review_ref) = explicit_ref
        .map(|value| value.to_string())
        .or_else(|| document_ref.map(|value| value.to_string()))
    else {
        return Ok(None);
    };

    let condition_review_trigger_type = request.condition_review_document.as_ref().map(|document| {
        persisted_condition_review_trigger_type(&document.review_trigger_type).to_string()
    });
    let condition_review_follow_up_action =
        request.condition_review_document.as_ref().map(|document| {
            persisted_condition_review_follow_up_action(&document.recommended_follow_up_action)
                .to_string()
        });
    let condition_review_summary = request
        .condition_review_document
        .as_ref()
        .map(|document| document.review_summary.trim().to_string())
        .filter(|value: &String| !value.is_empty());

    Ok(Some(NormalizedConditionReviewBinding {
        condition_review_ref,
        condition_review_trigger_type,
        condition_review_follow_up_action,
        condition_review_summary,
    }))
}

fn persisted_condition_review_trigger_type(
    trigger_type: &crate::tools::contracts::SecurityConditionReviewTriggerType,
) -> &'static str {
    match trigger_type {
        crate::tools::contracts::SecurityConditionReviewTriggerType::ManualReview => "manual_review",
        crate::tools::contracts::SecurityConditionReviewTriggerType::EndOfDayReview => {
            "end_of_day_review"
        }
        crate::tools::contracts::SecurityConditionReviewTriggerType::EventReview => "event_review",
        crate::tools::contracts::SecurityConditionReviewTriggerType::DataStalenessReview => {
            "data_staleness_review"
        }
    }
}

fn persisted_condition_review_follow_up_action(
    action: &crate::tools::contracts::SecurityConditionReviewFollowUpAction,
) -> &'static str {
    match action {
        crate::tools::contracts::SecurityConditionReviewFollowUpAction::KeepPlan => "keep_plan",
        crate::tools::contracts::SecurityConditionReviewFollowUpAction::UpdatePositionPlan => {
            "update_position_plan"
        }
        crate::tools::contracts::SecurityConditionReviewFollowUpAction::ReopenResearch => {
            "reopen_research"
        }
        crate::tools::contracts::SecurityConditionReviewFollowUpAction::ReopenCommittee => {
            "reopen_committee"
        }
        crate::tools::contracts::SecurityConditionReviewFollowUpAction::FreezeExecution => {
            "freeze_execution"
        }
    }
}

fn normalize_lines(items: &[String]) -> Vec<String> {
    items
        .iter()
        .map(|item| item.trim())
        .filter(|item| !item.is_empty())
        .map(|item| item.to_string())
        .collect()
}

fn rounded_pct(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
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
    180
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
