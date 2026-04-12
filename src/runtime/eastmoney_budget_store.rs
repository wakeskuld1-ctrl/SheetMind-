use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::runtime_paths::workspace_runtime_dir;

// 2026-04-11 UTC+08: Added EastMoney budget scopes so the new free-tier limiter can track
// why and where quota is spent; purpose: keep capital-flow and event calls independently governable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EastMoneyBudgetScope {
    CapitalFlow,
    Events,
}

// 2026-04-11 UTC+08: Added explicit config so tests and runtime can share the same limiter contract;
// purpose: avoid hard-coded magic numbers leaking into business logic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EastMoneyBudgetStoreConfig {
    pub total_daily_limit: usize,
    pub capital_flow_daily_limit: usize,
    pub event_daily_limit: usize,
}

// 2026-04-11 UTC+08: Added structured consume result so upper layers can degrade gracefully instead
// of treating quota exhaustion as a transport failure; purpose: keep the stock analysis chain alive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EastMoneyBudgetConsumeResult {
    pub status: String,
    pub remaining_for_scope: usize,
    pub remaining_total: usize,
    pub consumed: bool,
}

#[derive(Debug, Clone)]
pub struct EastMoneyBudgetStore {
    path: PathBuf,
    config: EastMoneyBudgetStoreConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct EastMoneyBudgetLedger {
    day: String,
    capital_flow_used: usize,
    event_used: usize,
    total_used: usize,
}

impl EastMoneyBudgetStore {
    pub fn new(path: PathBuf, config: EastMoneyBudgetStoreConfig) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        Ok(Self { path, config })
    }

    pub fn workspace_default() -> Result<Self, String> {
        let runtime_dir = workspace_runtime_dir()?;
        let total_daily_limit = std::env::var("EXCEL_SKILL_EASTMONEY_DAILY_LIMIT")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(50);
        let capital_flow_daily_limit =
            std::env::var("EXCEL_SKILL_EASTMONEY_CAPITAL_FLOW_DAILY_LIMIT")
                .ok()
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(total_daily_limit.min(30));
        let event_daily_limit = std::env::var("EXCEL_SKILL_EASTMONEY_EVENT_DAILY_LIMIT")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(total_daily_limit.saturating_sub(capital_flow_daily_limit));

        Self::new(
            runtime_dir.join("eastmoney_budget.json"),
            EastMoneyBudgetStoreConfig {
                total_daily_limit,
                capital_flow_daily_limit,
                event_daily_limit,
            },
        )
    }

    pub fn consume(
        &self,
        scope: EastMoneyBudgetScope,
    ) -> Result<EastMoneyBudgetConsumeResult, String> {
        let mut ledger = self.read_ledger()?;
        let today = Local::now().format("%Y-%m-%d").to_string();
        if ledger.day != today {
            ledger = EastMoneyBudgetLedger {
                day: today,
                ..EastMoneyBudgetLedger::default()
            };
        }

        let (scope_used, scope_limit) = match scope {
            EastMoneyBudgetScope::CapitalFlow => (
                &mut ledger.capital_flow_used,
                self.config.capital_flow_daily_limit,
            ),
            EastMoneyBudgetScope::Events => (&mut ledger.event_used, self.config.event_daily_limit),
        };

        if ledger.total_used >= self.config.total_daily_limit || *scope_used >= scope_limit {
            return Ok(EastMoneyBudgetConsumeResult {
                status: "budget_exhausted".to_string(),
                remaining_for_scope: scope_limit.saturating_sub(*scope_used),
                remaining_total: self
                    .config
                    .total_daily_limit
                    .saturating_sub(ledger.total_used),
                consumed: false,
            });
        }

        *scope_used += 1;
        ledger.total_used += 1;
        let remaining_for_scope = scope_limit.saturating_sub(*scope_used);
        let remaining_total = self
            .config
            .total_daily_limit
            .saturating_sub(ledger.total_used);
        self.write_ledger(&ledger)?;

        Ok(EastMoneyBudgetConsumeResult {
            status: "available".to_string(),
            remaining_for_scope,
            remaining_total,
            consumed: true,
        })
    }

    fn read_ledger(&self) -> Result<EastMoneyBudgetLedger, String> {
        if !self.path.exists() {
            return Ok(EastMoneyBudgetLedger::default());
        }
        let body = fs::read_to_string(&self.path).map_err(|error| error.to_string())?;
        serde_json::from_str(&body).map_err(|error| error.to_string())
    }

    fn write_ledger(&self, ledger: &EastMoneyBudgetLedger) -> Result<(), String> {
        let body = serde_json::to_string_pretty(ledger).map_err(|error| error.to_string())?;
        fs::write(&self.path, body).map_err(|error| error.to_string())
    }
}
