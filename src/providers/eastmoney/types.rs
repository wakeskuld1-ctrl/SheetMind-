use serde::{Deserialize, Serialize};

// 2026-04-11 UTC+08: Added a normalized capital-flow snapshot so upper layers can consume one
// stable contract even if the raw EastMoney payload varies; purpose: isolate parsing churn.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapitalFlowSnapshot {
    pub symbol: String,
    pub headline: String,
    pub main_net_inflow: Option<f64>,
    pub super_order_net_inflow: Option<f64>,
}

pub fn parse_capital_flow_snapshot(
    symbol: &str,
    payload: &serde_json::Value,
) -> Result<CapitalFlowSnapshot, String> {
    let root = payload.get("data").unwrap_or(payload);
    let headline = root
        .get("headline")
        .and_then(|value| value.as_str())
        .unwrap_or("capital flow snapshot available")
        .to_string();
    let main_net_inflow = find_number(
        root,
        &["main_net_inflow", "mainNetInflow", "MAIN_NET_INFLOW"],
    );
    let super_order_net_inflow = find_number(
        root,
        &[
            "super_order_net_inflow",
            "superOrderNetInflow",
            "SUPER_ORDER_NET_INFLOW",
        ],
    );

    if main_net_inflow.is_none() && super_order_net_inflow.is_none() {
        return Err("capital flow payload did not include inflow fields".to_string());
    }

    Ok(CapitalFlowSnapshot {
        symbol: symbol.to_string(),
        headline,
        main_net_inflow,
        super_order_net_inflow,
    })
}

fn find_number(root: &serde_json::Value, keys: &[&str]) -> Option<f64> {
    keys.iter().find_map(|key| {
        root.get(*key).and_then(|value| match value {
            serde_json::Value::Number(number) => number.as_f64(),
            serde_json::Value::String(text) => text.replace(',', "").parse::<f64>().ok(),
            _ => None,
        })
    })
}
