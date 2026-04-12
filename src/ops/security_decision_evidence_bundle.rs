use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use thiserror::Error;

use crate::ops::stock::security_analysis_contextual::SecurityAnalysisContextualResult;
use crate::ops::stock::security_analysis_fullstack::{
    DisclosureContext, FundamentalContext, IndustryContext, IntegratedConclusion,
    SecurityAnalysisFullstackError, SecurityAnalysisFullstackRequest,
    SecurityAnalysisFullstackResult, security_analysis_fullstack,
};
use crate::ops::stock::security_external_proxy_backfill::{
    SecurityExternalProxyBackfillError, resolve_effective_external_proxy_inputs,
};

// 2026-04-01 CST: 这里定义证券投决证据包请求，原因是方案 B 需要把研究链输出冻结成投决层的单一输入合同；
// 目的：让后续多头、空头与风险闸门都读取同一份证据，而不是各自直接碰 fullstack Tool 导致上下文漂移。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityDecisionEvidenceBundleRequest {
    pub symbol: String,
    #[serde(default)]
    pub market_symbol: Option<String>,
    #[serde(default)]
    pub sector_symbol: Option<String>,
    #[serde(default)]
    pub market_profile: Option<String>,
    #[serde(default)]
    pub sector_profile: Option<String>,
    #[serde(default)]
    pub as_of_date: Option<String>,
    #[serde(default = "default_lookback_days")]
    pub lookback_days: usize,
    #[serde(default = "default_disclosure_limit")]
    pub disclosure_limit: usize,
    #[serde(default)]
    pub external_proxy_inputs: Option<SecurityExternalProxyInputs>,
}

// 2026-04-11 CST: Add a governed external proxy payload contract, because Scheme B
// now needs gold ETF live proxy values to enter the formal evidence and scorecard
// chain without introducing a new out-of-band side channel.
// Purpose: let feature snapshots and approval flows freeze manually supplied proxy
// inputs in one auditable request object before historical backfill exists.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
pub struct SecurityExternalProxyInputs {
    #[serde(default)]
    pub yield_curve_proxy_status: Option<String>,
    #[serde(default)]
    pub yield_curve_slope_delta_bp_5d: Option<f64>,
    #[serde(default)]
    pub funding_liquidity_proxy_status: Option<String>,
    #[serde(default)]
    pub funding_liquidity_spread_delta_bp_5d: Option<f64>,
    #[serde(default)]
    pub gold_spot_proxy_status: Option<String>,
    #[serde(default)]
    pub gold_spot_proxy_return_5d: Option<f64>,
    #[serde(default)]
    pub usd_index_proxy_status: Option<String>,
    #[serde(default)]
    pub usd_index_proxy_return_5d: Option<f64>,
    #[serde(default)]
    pub real_rate_proxy_status: Option<String>,
    #[serde(default)]
    pub real_rate_proxy_delta_bp_5d: Option<f64>,
    #[serde(default)]
    pub fx_proxy_status: Option<String>,
    #[serde(default)]
    pub fx_return_5d: Option<f64>,
    #[serde(default)]
    pub overseas_market_proxy_status: Option<String>,
    #[serde(default)]
    pub overseas_market_return_5d: Option<f64>,
    #[serde(default)]
    pub market_session_gap_status: Option<String>,
    #[serde(default)]
    pub market_session_gap_days: Option<f64>,
    #[serde(default)]
    pub etf_fund_flow_proxy_status: Option<String>,
    #[serde(default)]
    pub etf_fund_flow_5d: Option<f64>,
    #[serde(default)]
    pub premium_discount_proxy_status: Option<String>,
    #[serde(default)]
    pub premium_discount_pct: Option<f64>,
    #[serde(default)]
    pub benchmark_relative_strength_status: Option<String>,
    #[serde(default)]
    pub benchmark_relative_return_5d: Option<f64>,
}

// 2026-04-01 CST: 这里定义证据质量摘要，原因是投决会需要先知道“证据完整度”再决定是否进入裁决；
// 目的：把技术、基本面、公告的可用性收敛成稳定字段，便于后续 Gate 和 Skill 统一消费。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityEvidenceQuality {
    pub technical_status: String,
    pub fundamental_status: String,
    pub disclosure_status: String,
    pub overall_status: String,
    pub risk_flags: Vec<String>,
}

// 2026-04-01 CST: 这里定义证券投决证据包结果，原因是我们要把研究链结果提升成投决层可冻结、可审计的对象；
// 目的：显式携带 analysis_date、data_gaps、evidence_hash，避免后续对话中悄悄混入新的事实或日期。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityDecisionEvidenceBundleResult {
    pub symbol: String,
    pub market_profile: Option<String>,
    pub sector_profile: Option<String>,
    #[serde(default)]
    pub external_proxy_inputs: Option<SecurityExternalProxyInputs>,
    pub analysis_date: String,
    pub technical_context: SecurityAnalysisContextualResult,
    pub fundamental_context: FundamentalContext,
    pub disclosure_context: DisclosureContext,
    pub industry_context: IndustryContext,
    pub integrated_conclusion: IntegratedConclusion,
    pub evidence_quality: SecurityEvidenceQuality,
    pub risk_notes: Vec<String>,
    pub data_gaps: Vec<String>,
    pub evidence_hash: String,
}

// 2026-04-01 CST: 这里单独定义证据包错误，原因是投决层不应该直接泄露 fullstack 内部错误类型细节；
// 目的：保留研究层失败原因，同时给 dispatcher 和后续 Skill 一个清晰、单一的错误边界。
#[derive(Debug, Error)]
pub enum SecurityDecisionEvidenceBundleError {
    #[error("证券投决证据冻结失败: {0}")]
    Fullstack(#[from] SecurityAnalysisFullstackError),
    #[error("security decision evidence bundle historical proxy hydration failed: {0}")]
    HistoricalProxy(#[from] SecurityExternalProxyBackfillError),
}

// 2026-04-01 CST: 这里实现证券投决证据冻结入口，原因是所有正反方立场必须先基于同一份静态证据包；
// 目的：把 current research chain 提升成 investment decision workbench 可消费的稳定中间层，而不是继续直接暴露研究工具本身。
pub fn security_decision_evidence_bundle(
    request: &SecurityDecisionEvidenceBundleRequest,
) -> Result<SecurityDecisionEvidenceBundleResult, SecurityDecisionEvidenceBundleError> {
    let fullstack_request = SecurityAnalysisFullstackRequest {
        symbol: request.symbol.clone(),
        market_symbol: request.market_symbol.clone(),
        sector_symbol: request.sector_symbol.clone(),
        market_profile: request.market_profile.clone(),
        sector_profile: request.sector_profile.clone(),
        as_of_date: request.as_of_date.clone(),
        lookback_days: request.lookback_days,
        disclosure_limit: request.disclosure_limit,
    };
    let analysis = security_analysis_fullstack(&fullstack_request)?;
    // 2026-04-12 UTC+08: Resolve historical ETF proxy inputs after fullstack picks
    // the effective analysis date, because latest production-style runs often omit
    // `as_of_date` and rely on the runtime to freeze to the latest trading day.
    // Purpose: make feature snapshot, scorecard, and chair_resolution share the same
    // effective proxy date even when the caller did not pass `as_of_date` explicitly.
    let effective_as_of_date = request
        .as_of_date
        .clone()
        .unwrap_or_else(|| analysis.technical_context.stock_analysis.as_of_date.clone());
    let effective_external_proxy_inputs = resolve_effective_external_proxy_inputs(
        &request.symbol,
        Some(effective_as_of_date.as_str()),
        request.external_proxy_inputs.clone(),
    )?;
    let mut effective_request = request.clone();
    effective_request.as_of_date = Some(effective_as_of_date);
    effective_request.external_proxy_inputs = effective_external_proxy_inputs;
    Ok(build_evidence_bundle(&effective_request, analysis))
}

// 2026-04-11 CST: Keep the ETF differentiating feature list close to the unified
// evidence seed, because ETF/equity model families now need one shared source of
// truth for which raw snapshot fields count as ETF-specific signals.
// Purpose: let feature snapshot, training, and runtime scorecard reuse the same ETF
// feature family instead of drifting into separate hard-coded lists.
pub(crate) const ETF_DIFFERENTIATING_FEATURES: &[&str] = &[
    "close_vs_sma50",
    "close_vs_sma200",
    "volume_ratio_20",
    "mfi_14",
    "cci_20",
    "williams_r_14",
    "boll_width_ratio_20",
    "atr_14",
    "rsrs_zscore_18_60",
    "support_gap_pct_20",
    "resistance_gap_pct_20",
];

// 2026-04-11 CST: Reserve governed placeholder contracts for ETF external proxies,
// because Scheme C needs treasury, gold, cross-border, and equity ETF pools to keep
// stable field names before real external feeds are wired in.
// Purpose: let evidence seed, training, and runtime governance share one auditable
// contract for future rate, FX, gold, fund-flow, and overseas proxy binding.
pub(crate) const TREASURY_ETF_PROXY_FIELDS: &[&str] =
    &["yield_curve_proxy_status", "funding_liquidity_proxy_status"];
pub(crate) const TREASURY_ETF_PROXY_NUMERIC_FIELDS: &[&str] = &[
    "yield_curve_slope_delta_bp_5d",
    "funding_liquidity_spread_delta_bp_5d",
];
pub(crate) const GOLD_ETF_PROXY_FIELDS: &[&str] = &[
    "gold_spot_proxy_status",
    "usd_index_proxy_status",
    "real_rate_proxy_status",
];
pub(crate) const GOLD_ETF_PROXY_NUMERIC_FIELDS: &[&str] = &[
    "gold_spot_proxy_return_5d",
    "usd_index_proxy_return_5d",
    "real_rate_proxy_delta_bp_5d",
];
pub(crate) const CROSS_BORDER_ETF_PROXY_FIELDS: &[&str] = &[
    "fx_proxy_status",
    "overseas_market_proxy_status",
    "market_session_gap_status",
];
pub(crate) const CROSS_BORDER_ETF_PROXY_NUMERIC_FIELDS: &[&str] = &[
    "fx_return_5d",
    "overseas_market_return_5d",
    "market_session_gap_days",
];
pub(crate) const EQUITY_ETF_PROXY_FIELDS: &[&str] = &[
    "etf_fund_flow_proxy_status",
    "premium_discount_proxy_status",
    "benchmark_relative_strength_status",
];
pub(crate) const EQUITY_ETF_PROXY_NUMERIC_FIELDS: &[&str] = &[
    "etf_fund_flow_5d",
    "premium_discount_pct",
    "benchmark_relative_return_5d",
];

// 2026-04-11 CST: Split ETF differentiating signals into governed sub-pool families,
// because Scheme B now requires equity ETF, treasury ETF, gold ETF, and cross-border
// ETF to stop sharing one generic minimum factor entrance.
// Purpose: give training and runtime one auditable source of truth for the minimum
// feature family each ETF sub-pool must carry before later external factors arrive.
pub(crate) const EQUITY_ETF_FEATURE_FAMILY: &[&str] = &[
    "close_vs_sma50",
    "close_vs_sma200",
    "volume_ratio_20",
    "support_gap_pct_20",
    "resistance_gap_pct_20",
    "rsrs_zscore_18_60",
    "etf_fund_flow_proxy_status",
    "etf_fund_flow_5d",
    "premium_discount_proxy_status",
    "premium_discount_pct",
    "benchmark_relative_strength_status",
    "benchmark_relative_return_5d",
];
pub(crate) const TREASURY_ETF_FEATURE_FAMILY: &[&str] = &[
    "close_vs_sma200",
    "boll_width_ratio_20",
    "atr_14",
    "rsrs_zscore_18_60",
    "yield_curve_proxy_status",
    "yield_curve_slope_delta_bp_5d",
    "funding_liquidity_proxy_status",
    "funding_liquidity_spread_delta_bp_5d",
];
pub(crate) const GOLD_ETF_FEATURE_FAMILY: &[&str] = &[
    "volume_ratio_20",
    "mfi_14",
    "cci_20",
    "williams_r_14",
    "atr_14",
    "gold_spot_proxy_status",
    "usd_index_proxy_status",
    "real_rate_proxy_status",
    "gold_spot_proxy_return_5d",
    "usd_index_proxy_return_5d",
    "real_rate_proxy_delta_bp_5d",
];
pub(crate) const CROSS_BORDER_ETF_FEATURE_FAMILY: &[&str] = &[
    "close_vs_sma50",
    "close_vs_sma200",
    "volume_ratio_20",
    "support_gap_pct_20",
    "resistance_gap_pct_20",
    "fx_proxy_status",
    "fx_return_5d",
    "overseas_market_proxy_status",
    "overseas_market_return_5d",
    "market_session_gap_status",
    "market_session_gap_days",
];

// 2026-04-11 CST: Centralize ETF symbol detection in the evidence layer, because
// training and runtime scorecard now both need the same ETF/equity branch without
// drifting into duplicated prefix checks.
// Purpose: keep ETF sub-pool classification consistent across feature seed,
// training artifact identity, and scorecard runtime guardrails.
pub(crate) fn is_etf_symbol(symbol: &str) -> bool {
    let code = symbol.split('.').next().unwrap_or_default();
    code.starts_with('5') || code.starts_with('1')
}

// 2026-04-11 CST: Add a governed ETF sub-pool classifier, because Scheme C splits
// ETF modeling into equity, treasury, gold, and cross-border families instead of
// one generic ETF bucket.
// Purpose: let training and runtime share one auditable ETF family resolution rule
// before deeper external ETF factors are introduced.
pub(crate) fn resolve_etf_subscope(
    symbol: &str,
    market_profile: Option<&str>,
    sector_profile: Option<&str>,
) -> Option<&'static str> {
    if !is_etf_symbol(symbol) {
        return None;
    }

    let normalized_context = [market_profile, sector_profile]
        .into_iter()
        .flatten()
        .map(|value| value.to_ascii_lowercase())
        .collect::<Vec<_>>();
    let contains_any = |needles: &[&str]| {
        normalized_context
            .iter()
            .any(|value| needles.iter().any(|needle| value.contains(needle)))
    };
    let code = symbol.split('.').next().unwrap_or_default();

    if contains_any(&[
        "bond_etf",
        "treasury",
        "gov_bond",
        "government_bond",
        "local_gov",
    ]) || matches!(code, "511010" | "511060")
    {
        Some("treasury_etf")
    } else if contains_any(&["gold_etf", "gold", "precious_metal"]) || code.starts_with("518") {
        Some("gold_etf")
    } else if contains_any(&[
        "cross_border",
        "cross-border",
        "qdii",
        "nikkei",
        "japan",
        "nasdaq",
        "sp500",
        "overseas",
        "global",
        "foreign",
    ]) {
        Some("cross_border_etf")
    } else {
        Some("equity_etf")
    }
}

// 2026-04-11 CST: Centralize ETF sub-pool minimum feature-family lookup, because
// training and runtime must validate the same structural factor entrance for each
// ETF pool instead of drifting into separate hard-coded lists.
// Purpose: let approval and scorecard reject artifacts that claim the correct ETF
// sub-pool label but still omit that pool's required minimum factors.
pub(crate) fn required_etf_feature_family(
    instrument_subscope: Option<&str>,
) -> &'static [&'static str] {
    match instrument_subscope {
        Some("treasury_etf") => TREASURY_ETF_FEATURE_FAMILY,
        Some("gold_etf") => GOLD_ETF_FEATURE_FAMILY,
        Some("cross_border_etf") => CROSS_BORDER_ETF_FEATURE_FAMILY,
        Some("equity_etf") => EQUITY_ETF_FEATURE_FAMILY,
        _ => ETF_DIFFERENTIATING_FEATURES,
    }
}

#[cfg(test)]
mod tests {
    use super::required_etf_feature_family;

    #[test]
    fn required_etf_feature_family_includes_external_proxy_contracts() {
        // 2026-04-11 CST: Add a red test for ETF external proxy contracts, reason:
        // Scheme C now needs every ETF sub-pool to reserve auditable proxy-field slots
        // before real macro, FX, gold, or overseas feeds are connected.
        // Purpose: lock the minimum external proxy contract for each ETF pool so later
        // data ingestion can bind into stable field names without changing the schema.
        let treasury_family = required_etf_feature_family(Some("treasury_etf"));
        let gold_family = required_etf_feature_family(Some("gold_etf"));
        let cross_border_family = required_etf_feature_family(Some("cross_border_etf"));
        let equity_family = required_etf_feature_family(Some("equity_etf"));

        assert!(treasury_family.contains(&"yield_curve_proxy_status"));
        assert!(treasury_family.contains(&"funding_liquidity_proxy_status"));
        assert!(treasury_family.contains(&"yield_curve_slope_delta_bp_5d"));
        assert!(treasury_family.contains(&"funding_liquidity_spread_delta_bp_5d"));
        assert!(gold_family.contains(&"gold_spot_proxy_status"));
        assert!(gold_family.contains(&"usd_index_proxy_status"));
        assert!(gold_family.contains(&"real_rate_proxy_status"));
        assert!(gold_family.contains(&"gold_spot_proxy_return_5d"));
        assert!(gold_family.contains(&"usd_index_proxy_return_5d"));
        assert!(gold_family.contains(&"real_rate_proxy_delta_bp_5d"));
        assert!(cross_border_family.contains(&"fx_proxy_status"));
        assert!(cross_border_family.contains(&"overseas_market_proxy_status"));
        assert!(cross_border_family.contains(&"fx_return_5d"));
        assert!(cross_border_family.contains(&"overseas_market_return_5d"));
        assert!(cross_border_family.contains(&"market_session_gap_status"));
        assert!(cross_border_family.contains(&"market_session_gap_days"));
        assert!(equity_family.contains(&"etf_fund_flow_proxy_status"));
        assert!(equity_family.contains(&"etf_fund_flow_5d"));
        assert!(equity_family.contains(&"premium_discount_proxy_status"));
        assert!(equity_family.contains(&"premium_discount_pct"));
        assert!(equity_family.contains(&"benchmark_relative_strength_status"));
        assert!(equity_family.contains(&"benchmark_relative_return_5d"));
    }
}

// 2026-04-09 CST: 这里新增证据包到原子特征种子的统一映射，原因是 Task 2 要把 feature_snapshot 的“当时可见特征”
// 收口在研究层正式中间对象上；目的：避免 feature_snapshot / scorecard / training 各自重复拼同一批证据字段。
pub fn build_evidence_bundle_feature_seed(
    bundle: &SecurityDecisionEvidenceBundleResult,
) -> BTreeMap<String, Value> {
    let mut features = BTreeMap::new();
    let indicator_snapshot = &bundle.technical_context.stock_analysis.indicator_snapshot;
    let instrument_subscope = resolve_etf_subscope(
        &bundle.symbol,
        bundle.market_profile.as_deref(),
        bundle.sector_profile.as_deref(),
    );
    features.insert(
        "integrated_stance".to_string(),
        Value::String(bundle.integrated_conclusion.stance.clone()),
    );
    features.insert(
        "technical_alignment".to_string(),
        Value::String(
            bundle
                .technical_context
                .contextual_conclusion
                .alignment
                .clone(),
        ),
    );
    features.insert(
        "technical_status".to_string(),
        Value::String(bundle.evidence_quality.technical_status.clone()),
    );
    features.insert(
        "fundamental_status".to_string(),
        Value::String(bundle.fundamental_context.status.clone()),
    );
    features.insert(
        "fundamental_available".to_string(),
        json!(bundle.fundamental_context.status == "available"),
    );
    features.insert(
        "disclosure_status".to_string(),
        Value::String(bundle.disclosure_context.status.clone()),
    );
    features.insert(
        "disclosure_available".to_string(),
        json!(bundle.disclosure_context.status == "available"),
    );
    features.insert(
        "overall_evidence_status".to_string(),
        Value::String(bundle.evidence_quality.overall_status.clone()),
    );
    features.insert("data_gap_count".to_string(), json!(bundle.data_gaps.len()));
    features.insert(
        "risk_note_count".to_string(),
        json!(bundle.risk_notes.len()),
    );
    // 2026-04-11 CST: Add ETF-ready numeric differentiators from the already-loaded
    // technical snapshot, because the previous four-feature seed was too coarse and
    // could collapse different ETF symbols into the same score bucket.
    // Purpose: keep ETF modeling inside the existing local-history pipeline while
    // exposing enough numeric structure for separate ETF training/runtime behavior.
    features.insert(
        "close_vs_sma50".to_string(),
        json!(ratio_gap(
            indicator_snapshot.close,
            indicator_snapshot.sma_50
        )),
    );
    features.insert(
        "close_vs_sma200".to_string(),
        json!(ratio_gap(
            indicator_snapshot.close,
            indicator_snapshot.sma_200
        )),
    );
    features.insert(
        "volume_ratio_20".to_string(),
        json!(indicator_snapshot.volume_ratio_20),
    );
    features.insert("mfi_14".to_string(), json!(indicator_snapshot.mfi_14));
    features.insert("cci_20".to_string(), json!(indicator_snapshot.cci_20));
    features.insert(
        "williams_r_14".to_string(),
        json!(indicator_snapshot.williams_r_14),
    );
    features.insert(
        "boll_width_ratio_20".to_string(),
        json!(indicator_snapshot.boll_width_ratio_20),
    );
    features.insert("atr_14".to_string(), json!(indicator_snapshot.atr_14));
    features.insert(
        "rsrs_zscore_18_60".to_string(),
        json!(indicator_snapshot.rsrs_zscore_18_60),
    );
    features.insert(
        "support_gap_pct_20".to_string(),
        json!(ratio_gap(
            indicator_snapshot.close,
            indicator_snapshot.support_level_20
        )),
    );
    features.insert(
        "resistance_gap_pct_20".to_string(),
        json!(ratio_gap(
            indicator_snapshot.resistance_level_20,
            indicator_snapshot.close
        )),
    );
    features.insert(
        "analysis_date".to_string(),
        Value::String(bundle.analysis_date.clone()),
    );
    features.insert(
        "market_profile".to_string(),
        bundle
            .market_profile
            .clone()
            .map(Value::String)
            .unwrap_or(Value::Null),
    );
    features.insert(
        "sector_profile".to_string(),
        bundle
            .sector_profile
            .clone()
            .map(Value::String)
            .unwrap_or(Value::Null),
    );
    features.insert(
        "instrument_subscope".to_string(),
        instrument_subscope
            .map(|value| Value::String(value.to_string()))
            .unwrap_or(Value::Null),
    );
    // 2026-04-11 CST: Write ETF external proxy status placeholders into the unified
    // raw feature seed, because Scheme C must freeze the schema before real macro,
    // FX, gold, and overseas feeds are connected.
    // Purpose: keep future external bindings backward-compatible with current
    // training/runtime consumers and make missing proxy contracts auditable today.
    append_etf_proxy_contract_features(
        &mut features,
        instrument_subscope,
        bundle.external_proxy_inputs.as_ref(),
    );
    features
}

// 2026-04-11 CST: Normalize price-distance features through one helper, because ETF
// and equity feature seeds both need a stable zero-safe ratio transform.
// Purpose: avoid duplicating ad-hoc denominator handling across ETF differentiators.
fn ratio_gap(numerator: f64, denominator: f64) -> f64 {
    if denominator.abs() <= f64::EPSILON {
        0.0
    } else {
        numerator / denominator - 1.0
    }
}

// 2026-04-11 CST: Centralize ETF proxy placeholder emission, because four ETF
// sub-pools now share one contract-writing rule even though each pool owns
// different external drivers.
// Purpose: keep the raw feature schema stable while marking the active ETF pool as
// awaiting proxy binding and every inactive pool as not applicable.
fn append_etf_proxy_contract_features(
    features: &mut BTreeMap<String, Value>,
    instrument_subscope: Option<&str>,
    external_proxy_inputs: Option<&SecurityExternalProxyInputs>,
) {
    for (subscope, proxy_fields) in [
        ("treasury_etf", TREASURY_ETF_PROXY_FIELDS),
        ("gold_etf", GOLD_ETF_PROXY_FIELDS),
        ("cross_border_etf", CROSS_BORDER_ETF_PROXY_FIELDS),
        ("equity_etf", EQUITY_ETF_PROXY_FIELDS),
    ] {
        for feature_name in proxy_fields {
            let status = match *feature_name {
                "yield_curve_proxy_status" => resolve_proxy_status(
                    external_proxy_inputs
                        .and_then(|inputs| inputs.yield_curve_proxy_status.as_ref()),
                    external_proxy_inputs
                        .and_then(|inputs| inputs.yield_curve_slope_delta_bp_5d)
                        .is_some(),
                    instrument_subscope == Some(subscope),
                ),
                "funding_liquidity_proxy_status" => resolve_proxy_status(
                    external_proxy_inputs
                        .and_then(|inputs| inputs.funding_liquidity_proxy_status.as_ref()),
                    external_proxy_inputs
                        .and_then(|inputs| inputs.funding_liquidity_spread_delta_bp_5d)
                        .is_some(),
                    instrument_subscope == Some(subscope),
                ),
                "gold_spot_proxy_status" => resolve_proxy_status(
                    external_proxy_inputs.and_then(|inputs| inputs.gold_spot_proxy_status.as_ref()),
                    external_proxy_inputs
                        .and_then(|inputs| inputs.gold_spot_proxy_return_5d)
                        .is_some(),
                    instrument_subscope == Some(subscope),
                ),
                "usd_index_proxy_status" => resolve_proxy_status(
                    external_proxy_inputs.and_then(|inputs| inputs.usd_index_proxy_status.as_ref()),
                    external_proxy_inputs
                        .and_then(|inputs| inputs.usd_index_proxy_return_5d)
                        .is_some(),
                    instrument_subscope == Some(subscope),
                ),
                "real_rate_proxy_status" => resolve_proxy_status(
                    external_proxy_inputs.and_then(|inputs| inputs.real_rate_proxy_status.as_ref()),
                    external_proxy_inputs
                        .and_then(|inputs| inputs.real_rate_proxy_delta_bp_5d)
                        .is_some(),
                    instrument_subscope == Some(subscope),
                ),
                "fx_proxy_status" => resolve_proxy_status(
                    external_proxy_inputs.and_then(|inputs| inputs.fx_proxy_status.as_ref()),
                    external_proxy_inputs
                        .and_then(|inputs| inputs.fx_return_5d)
                        .is_some(),
                    instrument_subscope == Some(subscope),
                ),
                "overseas_market_proxy_status" => resolve_proxy_status(
                    external_proxy_inputs
                        .and_then(|inputs| inputs.overseas_market_proxy_status.as_ref()),
                    external_proxy_inputs
                        .and_then(|inputs| inputs.overseas_market_return_5d)
                        .is_some(),
                    instrument_subscope == Some(subscope),
                ),
                "market_session_gap_status" => resolve_proxy_status(
                    external_proxy_inputs
                        .and_then(|inputs| inputs.market_session_gap_status.as_ref()),
                    external_proxy_inputs
                        .and_then(|inputs| inputs.market_session_gap_days)
                        .is_some(),
                    instrument_subscope == Some(subscope),
                ),
                "etf_fund_flow_proxy_status" => resolve_proxy_status(
                    external_proxy_inputs
                        .and_then(|inputs| inputs.etf_fund_flow_proxy_status.as_ref()),
                    external_proxy_inputs
                        .and_then(|inputs| inputs.etf_fund_flow_5d)
                        .is_some(),
                    instrument_subscope == Some(subscope),
                ),
                "premium_discount_proxy_status" => resolve_proxy_status(
                    external_proxy_inputs
                        .and_then(|inputs| inputs.premium_discount_proxy_status.as_ref()),
                    external_proxy_inputs
                        .and_then(|inputs| inputs.premium_discount_pct)
                        .is_some(),
                    instrument_subscope == Some(subscope),
                ),
                "benchmark_relative_strength_status" => resolve_proxy_status(
                    external_proxy_inputs
                        .and_then(|inputs| inputs.benchmark_relative_strength_status.as_ref()),
                    external_proxy_inputs
                        .and_then(|inputs| inputs.benchmark_relative_return_5d)
                        .is_some(),
                    instrument_subscope == Some(subscope),
                ),
                _ => {
                    if instrument_subscope == Some(subscope) {
                        "placeholder_unbound".to_string()
                    } else {
                        "not_applicable".to_string()
                    }
                }
            };
            features.insert((*feature_name).to_string(), Value::String(status));
        }
    }
    append_treasury_etf_proxy_numeric_features(
        features,
        instrument_subscope,
        external_proxy_inputs,
    );
    append_gold_etf_proxy_numeric_features(features, instrument_subscope, external_proxy_inputs);
    append_cross_border_etf_proxy_numeric_features(
        features,
        instrument_subscope,
        external_proxy_inputs,
    );
    append_equity_etf_proxy_numeric_features(features, instrument_subscope, external_proxy_inputs);
}

// 2026-04-11 CST: Freeze treasury ETF numeric proxy values in the raw feature schema,
// because Scheme B now needs yield-curve and funding-liquidity proxies to travel
// through the governed evidence path before real historical proxy backfill exists.
// Purpose: keep treasury ETF live proxy inputs auditable inside the same raw-feature
// contract consumed by scorecard, committee, and approval.
fn append_treasury_etf_proxy_numeric_features(
    features: &mut BTreeMap<String, Value>,
    instrument_subscope: Option<&str>,
    external_proxy_inputs: Option<&SecurityExternalProxyInputs>,
) {
    let active_treasury_proxy_inputs = if instrument_subscope == Some("treasury_etf") {
        external_proxy_inputs
    } else {
        None
    };
    for feature_name in TREASURY_ETF_PROXY_NUMERIC_FIELDS {
        features.insert(
            (*feature_name).to_string(),
            json!(
                treasury_proxy_numeric_value(active_treasury_proxy_inputs, feature_name)
                    .unwrap_or(0.0)
            ),
        );
    }
}

// 2026-04-11 CST: Keep treasury ETF numeric proxy extraction behind one helper,
// because manual proxy rollout should make later rate/liquidity proxy additions
// append-only instead of scattering string matches across the evidence layer.
// Purpose: reduce drift risk when treasury ETF proxy coverage grows.
fn treasury_proxy_numeric_value(
    external_proxy_inputs: Option<&SecurityExternalProxyInputs>,
    feature_name: &str,
) -> Option<f64> {
    let inputs = external_proxy_inputs?;
    match feature_name {
        "yield_curve_slope_delta_bp_5d" => inputs.yield_curve_slope_delta_bp_5d,
        "funding_liquidity_spread_delta_bp_5d" => inputs.funding_liquidity_spread_delta_bp_5d,
        _ => None,
    }
}

// 2026-04-11 CST: Freeze gold ETF numeric proxy values in the raw feature schema,
// because Scheme B needs gold price, USD, and real-rate proxies to reach scorecard
// consumers before historical proxy backfill exists.
// Purpose: keep live gold ETF analysis inside the same governed raw-feature contract
// as the rest of the security decision pipeline.
fn append_gold_etf_proxy_numeric_features(
    features: &mut BTreeMap<String, Value>,
    instrument_subscope: Option<&str>,
    external_proxy_inputs: Option<&SecurityExternalProxyInputs>,
) {
    // 2026-04-11 CST: Reuse the governed numeric gold-proxy field registry here,
    // because the first manual binding rollout duplicated three inserts and left
    // the shared field list unused.
    // Purpose: keep the raw-feature schema DRY so later gold proxy expansion only
    // changes the registry/helper pair instead of drifting across multiple sites.
    let active_gold_proxy_inputs = if instrument_subscope == Some("gold_etf") {
        external_proxy_inputs
    } else {
        None
    };
    for feature_name in GOLD_ETF_PROXY_NUMERIC_FIELDS {
        features.insert(
            (*feature_name).to_string(),
            json!(gold_proxy_numeric_value(active_gold_proxy_inputs, feature_name).unwrap_or(0.0)),
        );
    }
}

// 2026-04-11 CST: Keep gold ETF numeric proxy extraction behind one helper,
// because manual proxy rollout now shares the same field registry with training
// and runtime governance.
// Purpose: make future gold-proxy additions append-only and easier to audit.
fn gold_proxy_numeric_value(
    external_proxy_inputs: Option<&SecurityExternalProxyInputs>,
    feature_name: &str,
) -> Option<f64> {
    let inputs = external_proxy_inputs?;
    match feature_name {
        "gold_spot_proxy_return_5d" => inputs.gold_spot_proxy_return_5d,
        "usd_index_proxy_return_5d" => inputs.usd_index_proxy_return_5d,
        "real_rate_proxy_delta_bp_5d" => inputs.real_rate_proxy_delta_bp_5d,
        _ => None,
    }
}

// 2026-04-11 CST: Freeze cross-border ETF numeric proxy values in the raw feature
// schema, because Scheme B now needs FX, overseas market, and session-gap inputs
// to travel through the same governed evidence path as treasury/gold ETF proxies.
// Purpose: keep QDII and overseas-linked ETF live proxy inputs auditable before
// real historical proxy backfill and external feed capture are wired in.
fn append_cross_border_etf_proxy_numeric_features(
    features: &mut BTreeMap<String, Value>,
    instrument_subscope: Option<&str>,
    external_proxy_inputs: Option<&SecurityExternalProxyInputs>,
) {
    let active_cross_border_proxy_inputs = if instrument_subscope == Some("cross_border_etf") {
        external_proxy_inputs
    } else {
        None
    };
    for feature_name in CROSS_BORDER_ETF_PROXY_NUMERIC_FIELDS {
        features.insert(
            (*feature_name).to_string(),
            json!(
                cross_border_proxy_numeric_value(active_cross_border_proxy_inputs, feature_name)
                    .unwrap_or(0.0)
            ),
        );
    }
}

// 2026-04-11 CST: Keep cross-border ETF numeric proxy extraction behind one helper,
// because manual FX and overseas-session rollout should stay append-only when later
// Nikkei, US, or currency-linked inputs expand.
// Purpose: reduce drift risk by centralizing the governed field-name mapping.
fn cross_border_proxy_numeric_value(
    external_proxy_inputs: Option<&SecurityExternalProxyInputs>,
    feature_name: &str,
) -> Option<f64> {
    let inputs = external_proxy_inputs?;
    match feature_name {
        "fx_return_5d" => inputs.fx_return_5d,
        "overseas_market_return_5d" => inputs.overseas_market_return_5d,
        "market_session_gap_days" => inputs.market_session_gap_days,
        _ => None,
    }
}

// 2026-04-11 CST: Freeze equity ETF numeric proxy values in the raw feature schema,
// because Scheme B now needs fund-flow, premium-discount, and benchmark-relative
// inputs to travel through the same governed evidence path as other ETF sub-pools.
// Purpose: keep equity ETF live proxy inputs auditable before real ETF flow and
// discount data ingestion are wired in.
fn append_equity_etf_proxy_numeric_features(
    features: &mut BTreeMap<String, Value>,
    instrument_subscope: Option<&str>,
    external_proxy_inputs: Option<&SecurityExternalProxyInputs>,
) {
    let active_equity_proxy_inputs = if instrument_subscope == Some("equity_etf") {
        external_proxy_inputs
    } else {
        None
    };
    for feature_name in EQUITY_ETF_PROXY_NUMERIC_FIELDS {
        features.insert(
            (*feature_name).to_string(),
            json!(
                equity_proxy_numeric_value(active_equity_proxy_inputs, feature_name).unwrap_or(0.0)
            ),
        );
    }
}

// 2026-04-11 CST: Keep equity ETF numeric proxy extraction behind one helper,
// because manual fund-flow and premium-discount rollout should stay append-only
// when later ETF market-structure inputs expand.
// Purpose: reduce drift risk by centralizing the governed field-name mapping.
fn equity_proxy_numeric_value(
    external_proxy_inputs: Option<&SecurityExternalProxyInputs>,
    feature_name: &str,
) -> Option<f64> {
    let inputs = external_proxy_inputs?;
    match feature_name {
        "etf_fund_flow_5d" => inputs.etf_fund_flow_5d,
        "premium_discount_pct" => inputs.premium_discount_pct,
        "benchmark_relative_return_5d" => inputs.benchmark_relative_return_5d,
        _ => None,
    }
}

fn resolve_proxy_status(
    explicit_status: Option<&String>,
    has_numeric_binding: bool,
    is_active_pool: bool,
) -> String {
    if let Some(status) = explicit_status {
        status.clone()
    } else if is_active_pool && has_numeric_binding {
        "manual_bound".to_string()
    } else if is_active_pool {
        "placeholder_unbound".to_string()
    } else {
        "not_applicable".to_string()
    }
}

// 2026-04-01 CST: 这里把 fullstack 结果映射成投决证据包，原因是研究层和投决层虽然复用数据，但合同职责不同；
// 目的：集中补齐 analysis_date、quality、data_gaps 和 hash，避免这些逻辑散落在 Skill 或 dispatcher 里。
fn build_evidence_bundle(
    request: &SecurityDecisionEvidenceBundleRequest,
    analysis: SecurityAnalysisFullstackResult,
) -> SecurityDecisionEvidenceBundleResult {
    let SecurityAnalysisFullstackResult {
        symbol,
        technical_context,
        fundamental_context,
        disclosure_context,
        industry_context,
        integrated_conclusion,
    } = analysis;

    let analysis_date = technical_context.stock_analysis.as_of_date.clone();
    let data_gaps = collect_data_gaps(&fundamental_context, &disclosure_context);
    let risk_notes = collect_risk_notes(
        &technical_context,
        &fundamental_context,
        &disclosure_context,
        &industry_context,
        &integrated_conclusion,
        &data_gaps,
    );
    let evidence_quality =
        build_evidence_quality(&fundamental_context, &disclosure_context, &risk_notes);
    let evidence_hash = build_evidence_hash(
        &symbol,
        &analysis_date,
        &integrated_conclusion.stance,
        &evidence_quality,
        &data_gaps,
        request,
    );

    SecurityDecisionEvidenceBundleResult {
        symbol,
        market_profile: request.market_profile.clone(),
        sector_profile: request.sector_profile.clone(),
        external_proxy_inputs: request.external_proxy_inputs.clone(),
        analysis_date,
        technical_context,
        fundamental_context,
        disclosure_context,
        industry_context,
        integrated_conclusion,
        evidence_quality,
        risk_notes,
        data_gaps,
        evidence_hash,
    }
}

// 2026-04-01 CST: 这里集中定义证据缺口规则，原因是投决会需要显式知道缺了什么而不是只看 upstream status；
// 目的：把“不可用信息源”翻译成用户和 AI 都能解释的 data gap 语义。
fn collect_data_gaps(
    fundamental_context: &FundamentalContext,
    disclosure_context: &DisclosureContext,
) -> Vec<String> {
    let mut data_gaps = Vec::new();

    if fundamental_context.status != "available" {
        data_gaps.push(format!(
            "基本面上下文当前不可用：{}",
            fundamental_context.headline
        ));
    }
    if disclosure_context.status != "available" {
        data_gaps.push(format!(
            "公告上下文当前不可用：{}",
            disclosure_context.headline
        ));
    }

    data_gaps
}

// 2026-04-01 CST: 这里统一收集证据层风险提示，原因是后续双立场与闸门都要引用同一组风险摘要；
// 目的：避免 bull/bear 在不同位置重复拼接风险，导致观点口径不一致。
fn collect_risk_notes(
    technical_context: &SecurityAnalysisContextualResult,
    fundamental_context: &FundamentalContext,
    disclosure_context: &DisclosureContext,
    industry_context: &IndustryContext,
    integrated_conclusion: &IntegratedConclusion,
    data_gaps: &[String],
) -> Vec<String> {
    let mut risk_notes = Vec::new();
    risk_notes.extend(technical_context.contextual_conclusion.risk_flags.clone());
    risk_notes.extend(fundamental_context.risk_flags.clone());
    risk_notes.extend(disclosure_context.risk_flags.clone());
    risk_notes.extend(industry_context.risk_flags.clone());
    risk_notes.extend(integrated_conclusion.risk_flags.clone());
    risk_notes.extend(data_gaps.iter().cloned());
    dedupe_strings(&mut risk_notes);
    risk_notes
}

// 2026-04-01 CST: 这里把多源可用性收敛成质量摘要，原因是风控闸门只需要看稳定状态而不是重新解读全部子对象；
// 目的：为 data completeness、approval gating 和最终输出提供统一的质量刻度。
fn build_evidence_quality(
    fundamental_context: &FundamentalContext,
    disclosure_context: &DisclosureContext,
    risk_notes: &[String],
) -> SecurityEvidenceQuality {
    let technical_status = "available".to_string();
    let fundamental_status = fundamental_context.status.clone();
    let disclosure_status = disclosure_context.status.clone();
    let overall_status = if fundamental_status == "available" && disclosure_status == "available" {
        "complete".to_string()
    } else {
        "degraded".to_string()
    };

    SecurityEvidenceQuality {
        technical_status,
        fundamental_status,
        disclosure_status,
        overall_status,
        risk_flags: risk_notes.to_vec(),
    }
}

// 2026-04-01 CST: 这里生成证据哈希，原因是单次对话里的正反方必须围绕同一份冻结证据而不是隐式更新的数据；
// 目的：给 Skill 和后续审计链一个可比对的“证据版本标识”。
fn build_evidence_hash(
    symbol: &str,
    analysis_date: &str,
    stance: &str,
    evidence_quality: &SecurityEvidenceQuality,
    data_gaps: &[String],
    request: &SecurityDecisionEvidenceBundleRequest,
) -> String {
    let mut hasher = DefaultHasher::new();
    symbol.hash(&mut hasher);
    analysis_date.hash(&mut hasher);
    stance.hash(&mut hasher);
    evidence_quality.overall_status.hash(&mut hasher);
    evidence_quality.fundamental_status.hash(&mut hasher);
    evidence_quality.disclosure_status.hash(&mut hasher);
    data_gaps.hash(&mut hasher);
    request.market_symbol.hash(&mut hasher);
    request.sector_symbol.hash(&mut hasher);
    request.market_profile.hash(&mut hasher);
    request.sector_profile.hash(&mut hasher);
    request.lookback_days.hash(&mut hasher);
    request.disclosure_limit.hash(&mut hasher);
    hash_external_proxy_inputs(request.external_proxy_inputs.as_ref(), &mut hasher);
    format!("sec-{:016x}", hasher.finish())
}

fn hash_external_proxy_inputs(
    external_proxy_inputs: Option<&SecurityExternalProxyInputs>,
    hasher: &mut DefaultHasher,
) {
    let Some(inputs) = external_proxy_inputs else {
        return;
    };
    inputs.yield_curve_proxy_status.hash(hasher);
    inputs
        .yield_curve_slope_delta_bp_5d
        .map(f64::to_bits)
        .hash(hasher);
    inputs.funding_liquidity_proxy_status.hash(hasher);
    inputs
        .funding_liquidity_spread_delta_bp_5d
        .map(f64::to_bits)
        .hash(hasher);
    inputs.gold_spot_proxy_status.hash(hasher);
    inputs
        .gold_spot_proxy_return_5d
        .map(f64::to_bits)
        .hash(hasher);
    inputs.usd_index_proxy_status.hash(hasher);
    inputs
        .usd_index_proxy_return_5d
        .map(f64::to_bits)
        .hash(hasher);
    inputs.real_rate_proxy_status.hash(hasher);
    inputs
        .real_rate_proxy_delta_bp_5d
        .map(f64::to_bits)
        .hash(hasher);
    inputs.fx_proxy_status.hash(hasher);
    inputs.fx_return_5d.map(f64::to_bits).hash(hasher);
    inputs.overseas_market_proxy_status.hash(hasher);
    inputs
        .overseas_market_return_5d
        .map(f64::to_bits)
        .hash(hasher);
    inputs.market_session_gap_status.hash(hasher);
    inputs
        .market_session_gap_days
        .map(f64::to_bits)
        .hash(hasher);
    inputs.etf_fund_flow_proxy_status.hash(hasher);
    inputs.etf_fund_flow_5d.map(f64::to_bits).hash(hasher);
    inputs.premium_discount_proxy_status.hash(hasher);
    inputs.premium_discount_pct.map(f64::to_bits).hash(hasher);
    inputs.benchmark_relative_strength_status.hash(hasher);
    inputs
        .benchmark_relative_return_5d
        .map(f64::to_bits)
        .hash(hasher);
}

// 2026-04-01 CST: 这里统一做去重，原因是多层研究链可能同时指出同一个风险点；
// 目的：保持输出精炼，避免后续正反方摘要被重复风险淹没。
fn dedupe_strings(values: &mut Vec<String>) {
    let mut deduped = Vec::new();
    for value in values.drain(..) {
        if !deduped.contains(&value) {
            deduped.push(value);
        }
    }
    *values = deduped;
}

fn default_lookback_days() -> usize {
    260
}

fn default_disclosure_limit() -> usize {
    8
}
