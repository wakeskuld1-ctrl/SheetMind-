use serde::{Deserialize, Serialize};

use crate::providers::eastmoney::cache::{CAPITAL_FLOW_NAMESPACE, CAPITAL_FLOW_TTL_SECONDS};
use crate::providers::eastmoney::fetch_capital_flow_snapshot;
use crate::providers::eastmoney::types::CapitalFlowSnapshot;
use crate::runtime::eastmoney_budget_store::{EastMoneyBudgetScope, EastMoneyBudgetStore};
use crate::runtime::eastmoney_cache_store::EastMoneyCacheStore;

// 2026-04-11 UTC+08: Added a stock-domain capital-flow context so contextual analysis can show
// funding pressure without directly depending on provider details; purpose: keep SRP boundaries clear.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CapitalFlowContext {
    pub status: String,
    pub source: String,
    pub headline: String,
    pub main_net_inflow: Option<f64>,
    pub super_order_net_inflow: Option<f64>,
    pub budget_status: String,
    pub cache_hit: bool,
    pub risk_flags: Vec<String>,
}

pub fn fetch_capital_flow_context(symbol: &str) -> CapitalFlowContext {
    if let Ok(cache_store) = EastMoneyCacheStore::workspace_default() {
        if let Ok(Some(cached)) =
            cache_store.get::<CapitalFlowSnapshot>(CAPITAL_FLOW_NAMESPACE, symbol)
        {
            return available_capital_flow_context(
                cached.payload,
                "cache_hit".to_string(),
                cached.cache_hit,
            );
        }
    }

    let budget_store = match EastMoneyBudgetStore::workspace_default() {
        Ok(store) => store,
        Err(error) => {
            return unavailable_capital_flow_context(
                "budget_store_unavailable".to_string(),
                vec![error],
            );
        }
    };
    let budget = match budget_store.consume(EastMoneyBudgetScope::CapitalFlow) {
        Ok(result) => result,
        Err(error) => {
            return unavailable_capital_flow_context(
                "budget_store_unavailable".to_string(),
                vec![error],
            );
        }
    };

    if budget.status == "budget_exhausted" {
        return CapitalFlowContext {
            status: "budget_exhausted".to_string(),
            source: "eastmoney_capital_flow".to_string(),
            headline: "capital flow query skipped because the daily EastMoney budget is exhausted"
                .to_string(),
            main_net_inflow: None,
            super_order_net_inflow: None,
            budget_status: budget.status,
            cache_hit: false,
            risk_flags: vec!["EastMoney capital-flow daily budget exhausted".to_string()],
        };
    }

    match fetch_capital_flow_snapshot(symbol) {
        Ok(snapshot) => {
            if let Ok(cache_store) = EastMoneyCacheStore::workspace_default() {
                let _ = cache_store.put(
                    CAPITAL_FLOW_NAMESPACE,
                    symbol,
                    CAPITAL_FLOW_TTL_SECONDS,
                    &snapshot,
                );
            }
            available_capital_flow_context(snapshot, budget.status, false)
        }
        Err(error) => unavailable_capital_flow_context("available".to_string(), vec![error]),
    }
}

fn available_capital_flow_context(
    snapshot: CapitalFlowSnapshot,
    budget_status: String,
    cache_hit: bool,
) -> CapitalFlowContext {
    CapitalFlowContext {
        status: "available".to_string(),
        source: "eastmoney_capital_flow".to_string(),
        headline: snapshot.headline,
        main_net_inflow: snapshot.main_net_inflow,
        super_order_net_inflow: snapshot.super_order_net_inflow,
        budget_status,
        cache_hit,
        risk_flags: Vec::new(),
    }
}

fn unavailable_capital_flow_context(
    budget_status: String,
    risk_flags: Vec<String>,
) -> CapitalFlowContext {
    CapitalFlowContext {
        status: "unavailable".to_string(),
        source: "eastmoney_capital_flow".to_string(),
        headline: "capital flow snapshot is currently unavailable; contextual analysis fell back to technical-only environmental signals".to_string(),
        main_net_inflow: None,
        super_order_net_inflow: None,
        budget_status,
        cache_hit: false,
        risk_flags,
    }
}
