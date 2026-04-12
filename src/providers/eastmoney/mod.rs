pub mod cache;
pub mod client;
pub mod types;

use types::CapitalFlowSnapshot;

pub fn fetch_capital_flow_snapshot(symbol: &str) -> Result<CapitalFlowSnapshot, String> {
    let base = std::env::var("EXCEL_SKILL_EASTMONEY_CAPITAL_FLOW_URL_BASE")
        .map_err(|_| "capital flow endpoint not configured".to_string())?;
    let separator = if base.contains('?') { "&" } else { "?" };
    let url = format!("{base}{separator}symbol={symbol}");
    let payload = client::http_get_json(&url, "capital_flow")?;
    types::parse_capital_flow_snapshot(symbol, &payload)
}
