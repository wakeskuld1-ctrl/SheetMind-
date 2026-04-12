use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::ops::stock::security_analysis_contextual::{
    SecurityAnalysisContextualError, SecurityAnalysisContextualRequest,
    SecurityAnalysisContextualResult, security_analysis_contextual,
};
use crate::ops::stock::security_disclosure_history_backfill::load_historical_disclosure_context;
use crate::ops::stock::security_external_proxy_backfill::resolve_effective_external_proxy_inputs;
use crate::ops::stock::security_fundamental_history_backfill::load_historical_fundamental_context;

const DEFAULT_DISCLOSURE_LIMIT: usize = 8;

// 2026-04-01 CST: 这里定义 fullstack 总 Tool 请求，原因是方案 1 已确定要把技术面、财报面、公告面合并成统一产品入口；
// 目的：让上层只打一枪就拿到完整证券分析骨架，同时复用现有 market/sector proxy 配置口径而不重造轮子。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityAnalysisFullstackRequest {
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
}

// 2026-04-01 CST: 这里定义 fullstack 总 Tool 结果，原因是产品主链需要稳定消费“技术 + 财报 + 公告 + 行业 + 综合结论”统一合同；
// 目的：避免 Skill / GUI / 其他 AI 继续在外层手工拼多个 Tool 返回，降低产品接线复杂度。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityAnalysisFullstackResult {
    pub symbol: String,
    pub technical_context: SecurityAnalysisContextualResult,
    pub fundamental_context: FundamentalContext,
    pub disclosure_context: DisclosureContext,
    pub industry_context: IndustryContext,
    pub integrated_conclusion: IntegratedConclusion,
}

// 2026-04-01 CST: 这里定义财报快照合同，原因是首版信息面先收口到“最近报告期 + 核心增长指标 + 风险提示”；
// 目的：先交付可消费的财报层，而不是一次性把全部财务表做成重型数据仓接口。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct FundamentalContext {
    pub status: String,
    pub source: String,
    pub latest_report_period: Option<String>,
    pub report_notice_date: Option<String>,
    pub headline: String,
    pub profit_signal: String,
    pub report_metrics: FundamentalMetrics,
    pub narrative: Vec<String>,
    pub risk_flags: Vec<String>,
}

// 2026-04-01 CST: 这里拆独立财报指标结构，原因是首版产品最需要的是少量高价值指标，而不是整张财报明细；
// 目的：给后续 GUI 和策略层保留稳定字段，不把 narrative 文案当成唯一数据来源。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct FundamentalMetrics {
    pub revenue: Option<f64>,
    pub revenue_yoy_pct: Option<f64>,
    pub net_profit: Option<f64>,
    pub net_profit_yoy_pct: Option<f64>,
    pub roe_pct: Option<f64>,
}

// 2026-04-01 CST: 这里定义公告摘要合同，原因是首版公告面重点是“最近披露了什么、有没有明显风险关键词”；
// 目的：先把公告层做成稳定摘要入口，而不是引入大模型做自由文本总结。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DisclosureContext {
    pub status: String,
    pub source: String,
    pub announcement_count: usize,
    pub headline: String,
    pub keyword_summary: Vec<String>,
    pub recent_announcements: Vec<DisclosureAnnouncement>,
    pub risk_flags: Vec<String>,
}

// 2026-04-01 CST: 这里定义单条公告对象，原因是产品后续需要展示时间线和点击跳转能力；
// 目的：把最近公告列表固化成结构化数组，避免只有一段模糊摘要。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DisclosureAnnouncement {
    pub published_at: String,
    pub title: String,
    pub article_code: Option<String>,
    pub category: Option<String>,
}

// 2026-04-01 CST: 这里定义行业上下文，原因是行业层首版先沿用 sector proxy 技术结论，但要给上层稳定的独立消费字段；
// 目的：把“行业环境”从 technical_context 的深层嵌套里提炼出来，便于后续继续叠加行业景气数据。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct IndustryContext {
    pub sector_symbol: String,
    pub proxy_bias: String,
    pub headline: String,
    pub rationale: Vec<String>,
    pub risk_flags: Vec<String>,
}

// 2026-04-01 CST: 这里定义总综合结论，原因是产品真正需要的是最终 stance，而不是调用方自己重新读完所有子结果再拼判断；
// 目的：把技术面与信息面冲突/共振关系收敛成统一 headline + rationale + risk_flags。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct IntegratedConclusion {
    pub stance: String,
    pub headline: String,
    pub rationale: Vec<String>,
    pub risk_flags: Vec<String>,
}

// 2026-04-01 CST: 这里集中定义 fullstack Tool 错误，原因是方案 1 只允许技术主链失败时中断，信息面源失败必须降级；
// 目的：清晰区分“主链不可用”和“信息面缺失但可继续返回”的产品语义。
#[derive(Debug, Error)]
pub enum SecurityAnalysisFullstackError {
    #[error("技术上下文分析失败: {0}")]
    Technical(#[from] SecurityAnalysisContextualError),
}

#[derive(Debug, Error)]
enum FundamentalFetchError {
    #[error("财报源请求失败: {0}")]
    Transport(String),
    #[error("财报源响应解析失败: {0}")]
    Parse(String),
    #[error("财报源没有返回可用数据")]
    Empty,
}

#[derive(Debug, Error)]
enum DisclosureFetchError {
    #[error("公告源请求失败: {0}")]
    Transport(String),
    #[error("公告源响应解析失败: {0}")]
    Parse(String),
    #[error("公告源没有返回可用数据")]
    Empty,
}

// 2026-04-12 CST: Add a governed-history row contract for live financial backfill,
// because the new historical-data path must persist multiple provider periods
// without duplicating field-normalization logic in every caller.
// Purpose: let stock history tools reuse the same decoded multi-period financial rows.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct LiveFundamentalHistoryRow {
    pub report_period: String,
    pub notice_date: Option<String>,
    pub source: String,
    pub report_metrics: FundamentalMetrics,
}

// 2026-04-12 CST: Add a governed-history row contract for live disclosure backfill,
// because the new historical-data path must persist paged announcement rows
// without rebuilding parsing logic outside fullstack.
// Purpose: let stock history tools reuse the same decoded multi-page announcement rows.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct LiveDisclosureHistoryRow {
    pub published_at: String,
    pub title: String,
    pub article_code: Option<String>,
    pub category: Option<String>,
    pub source: String,
}

// 2026-04-01 CST: 这里实现 fullstack 主入口，原因是用户已确认要把大盘、行业、财报、公告并进产品主链；
// 目的：在不污染底层技术面 Tool 的前提下，交付一个真正可直接面向产品调用的综合证券分析入口。
pub fn security_analysis_fullstack(
    request: &SecurityAnalysisFullstackRequest,
) -> Result<SecurityAnalysisFullstackResult, SecurityAnalysisFullstackError> {
    let technical_request = SecurityAnalysisContextualRequest {
        symbol: request.symbol.clone(),
        market_symbol: request.market_symbol.clone(),
        sector_symbol: request.sector_symbol.clone(),
        market_profile: request.market_profile.clone(),
        sector_profile: request.sector_profile.clone(),
        as_of_date: request.as_of_date.clone(),
        lookback_days: request.lookback_days,
    };
    let technical_context = security_analysis_contextual(&technical_request)?;
    // 2026-04-13 CST: Prefer governed ETF-native information synthesis here, because
    // the user explicitly required ETF symbols to stop inheriting stock-only
    // information gaps once governed proxy history is already available.
    // Purpose: let fullstack/committee/chair treat complete ETF proxy evidence as
    // formal information input instead of auto-downgrading to `technical_only`.
    let etf_information_context = synthesize_etf_information_context(request);
    let (fundamental_context, disclosure_context) =
        if let Some((fundamental_context, disclosure_context)) = etf_information_context {
            (fundamental_context, disclosure_context)
        } else {
            let fundamental_context = match load_historical_fundamental_context(
                &request.symbol,
                request.as_of_date.as_deref(),
            ) {
                Ok(Some(context)) => context,
                Ok(None) => match fetch_fundamental_context(&request.symbol) {
                    Ok(context) => context,
                    Err(error) => build_unavailable_fundamental_context(error.to_string()),
                },
                Err(error) => build_unavailable_fundamental_context(error.to_string()),
            };
            let disclosure_context = match load_historical_disclosure_context(
                &request.symbol,
                request.as_of_date.as_deref(),
                request.disclosure_limit.max(1),
            ) {
                Ok(Some(context)) => context,
                Ok(None) => {
                    match fetch_disclosure_context(&request.symbol, request.disclosure_limit.max(1))
                    {
                        Ok(context) => context,
                        Err(error) => build_unavailable_disclosure_context(error.to_string()),
                    }
                }
                Err(error) => build_unavailable_disclosure_context(error.to_string()),
            };
            (fundamental_context, disclosure_context)
        };
    let industry_context = build_industry_context(&technical_context);
    let integrated_conclusion = build_integrated_conclusion(
        &technical_context,
        &fundamental_context,
        &disclosure_context,
        &industry_context,
    );

    Ok(SecurityAnalysisFullstackResult {
        symbol: request.symbol.clone(),
        technical_context,
        fundamental_context,
        disclosure_context,
        industry_context,
        integrated_conclusion,
    })
}

// 2026-04-01 CST: 这里抓取财报快照，原因是首版财报面不做本地持久化，先把免费公开源聚合到总 Tool；
// 目的：先交付“最新报告期核心指标”能力，后续再决定是否沉淀到 SQLite。
fn fetch_fundamental_context(symbol: &str) -> Result<FundamentalContext, FundamentalFetchError> {
    let url = build_financial_url(symbol);
    let body = http_get_text(&url, "financials").map_err(FundamentalFetchError::Transport)?;
    let payload = serde_json::from_str::<Value>(&body)
        .map_err(|error| FundamentalFetchError::Parse(error.to_string()))?;
    let latest = extract_latest_financial_row(&payload)?;

    let latest_report_period = financial_string(
        latest,
        &["REPORT_DATE", "REPORTDATE", "REPORT_DATE_NAME", "date"],
    )
    .map(normalize_date_like);
    let report_notice_date =
        financial_string(latest, &["NOTICE_DATE", "NOTICEDATE", "latestNoticeDate"])
            .map(normalize_date_like);
    let metrics = FundamentalMetrics {
        revenue: financial_number(
            latest,
            &[
                "TOTAL_OPERATE_INCOME",
                "TOTALOPERATEINCOME",
                "营业总收入",
                "yyzsr",
            ],
        ),
        revenue_yoy_pct: financial_number(latest, &["YSTZ", "YYZSR_GTHR", "营业总收入同比"]),
        net_profit: financial_number(
            latest,
            &["PARENT_NETPROFIT", "PARENTNETPROFIT", "归母净利润", "gsjlr"],
        ),
        net_profit_yoy_pct: financial_number(
            latest,
            &["SJLTZ", "NETPROFIT_GTHR", "归母净利润同比"],
        ),
        roe_pct: financial_number(latest, &["ROEJQ", "ROE_WEIGHTED", "jqjzcsyl"]),
    };
    let profit_signal = classify_fundamental_signal(&metrics);
    let (headline, narrative, risk_flags) = build_fundamental_narrative(&metrics, &profit_signal);

    Ok(FundamentalContext {
        status: "available".to_string(),
        source: "eastmoney_financials".to_string(),
        latest_report_period,
        report_notice_date,
        headline,
        profit_signal,
        report_metrics: metrics,
        narrative,
        risk_flags,
    })
}

// 2026-04-13 CST: Add an ETF-native information synthesis branch, because governed
// ETF proxy history now exists while stock-style financial/disclosure records do not.
// Purpose: keep ETF conclusions from being permanently trapped in stock-only
// information gaps when proxy evidence is already complete and auditable.
fn synthesize_etf_information_context(
    request: &SecurityAnalysisFullstackRequest,
) -> Option<(FundamentalContext, DisclosureContext)> {
    let instrument_subscope = resolve_fullstack_etf_subscope(
        &request.symbol,
        request.market_profile.as_deref(),
        request.sector_profile.as_deref(),
    )?;
    let as_of_date = request.as_of_date.as_deref()?;
    let proxy_inputs =
        resolve_effective_external_proxy_inputs(&request.symbol, Some(as_of_date), None).ok()??;
    if !etf_proxy_information_is_complete(instrument_subscope, &proxy_inputs) {
        return None;
    }

    Some(build_etf_information_contexts(
        instrument_subscope,
        as_of_date,
        &proxy_inputs,
    ))
}

// 2026-04-13 CST: Keep ETF family detection local to fullstack, because this layer
// must stay independent from the evidence-bundle module that already depends on it.
// Purpose: avoid a circular dependency while still letting fullstack speak ETF-native
// information semantics before the decision layer freezes the evidence bundle.
fn resolve_fullstack_etf_subscope(
    symbol: &str,
    market_profile: Option<&str>,
    sector_profile: Option<&str>,
) -> Option<&'static str> {
    let code = symbol.split('.').next().unwrap_or_default();
    if !(code.starts_with('5') || code.starts_with('1')) {
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

// 2026-04-13 CST: Define one completeness gate for ETF proxy evidence, because the
// synthesized ETF information layer should only claim availability when its required
// proxy contracts are actually bound for the current replay date.
// Purpose: stop ETF semantics from becoming too optimistic while still letting
// complete governed proxy history count as formal information evidence.
fn etf_proxy_information_is_complete(
    instrument_subscope: &str,
    proxy_inputs: &crate::ops::stock::security_decision_evidence_bundle::SecurityExternalProxyInputs,
) -> bool {
    match instrument_subscope {
        "treasury_etf" => {
            proxy_status_is_bound(proxy_inputs.yield_curve_proxy_status.as_deref())
                && proxy_status_is_bound(proxy_inputs.funding_liquidity_proxy_status.as_deref())
        }
        "gold_etf" => {
            proxy_status_is_bound(proxy_inputs.gold_spot_proxy_status.as_deref())
                && proxy_status_is_bound(proxy_inputs.usd_index_proxy_status.as_deref())
                && proxy_status_is_bound(proxy_inputs.real_rate_proxy_status.as_deref())
        }
        "cross_border_etf" => {
            proxy_status_is_bound(proxy_inputs.fx_proxy_status.as_deref())
                && proxy_status_is_bound(proxy_inputs.overseas_market_proxy_status.as_deref())
                && proxy_status_is_bound(proxy_inputs.market_session_gap_status.as_deref())
        }
        "equity_etf" => {
            proxy_status_is_bound(proxy_inputs.etf_fund_flow_proxy_status.as_deref())
                && proxy_status_is_bound(proxy_inputs.premium_discount_proxy_status.as_deref())
                && proxy_status_is_bound(proxy_inputs.benchmark_relative_strength_status.as_deref())
        }
        _ => false,
    }
}

// 2026-04-13 CST: Normalize proxy binding state checks here, because ETF governed
// history uses placeholder and not-applicable markers during earlier backfill stages.
// Purpose: give the ETF information synthesis path one strict rule for when a proxy
// field should count as real evidence instead of a placeholder contract.
fn proxy_status_is_bound(status: Option<&str>) -> bool {
    matches!(
        status,
        Some("manual_bound") | Some("historical_bound") | Some("bound")
    )
}

// 2026-04-13 CST: Build ETF-native information contexts from governed proxy inputs,
// because the current fullstack contract still expects fundamental/disclosure-style
// slots even when the instrument is an ETF.
// Purpose: let the rest of the mainline keep its existing contracts while consuming
// ETF-specific evidence instead of forcing stock-only information semantics.
fn build_etf_information_contexts(
    instrument_subscope: &str,
    as_of_date: &str,
    proxy_inputs: &crate::ops::stock::security_decision_evidence_bundle::SecurityExternalProxyInputs,
) -> (FundamentalContext, DisclosureContext) {
    let (profit_signal, proxy_headline, proxy_keywords, proxy_risks) =
        summarize_etf_proxy_signal(instrument_subscope, proxy_inputs);
    let information_source = "governed_etf_proxy_information".to_string();
    let narrative = vec![
        format!(
            "ETF information synthesis used governed proxy evidence for `{instrument_subscope}` at `{as_of_date}`."
        ),
        proxy_headline.clone(),
    ];
    let fundamental_context = FundamentalContext {
        status: "available".to_string(),
        source: information_source.clone(),
        latest_report_period: Some(as_of_date.to_string()),
        report_notice_date: Some(as_of_date.to_string()),
        headline: format!("ETF proxy information is available: {proxy_headline}"),
        profit_signal,
        report_metrics: FundamentalMetrics {
            revenue: None,
            revenue_yoy_pct: None,
            net_profit: None,
            net_profit_yoy_pct: None,
            roe_pct: None,
        },
        narrative,
        risk_flags: proxy_risks.clone(),
    };
    let disclosure_context = DisclosureContext {
        status: "available".to_string(),
        source: information_source,
        announcement_count: proxy_keywords.len(),
        headline: format!(
            "ETF proxy evidence covers {} signal dimensions",
            proxy_keywords.len()
        ),
        keyword_summary: proxy_keywords,
        recent_announcements: vec![],
        risk_flags: proxy_risks,
    };

    (fundamental_context, disclosure_context)
}

// 2026-04-13 CST: Collapse ETF proxy values into one governed information summary,
// because the current integrated conclusion path still expects a compact signal and
// a small keyword/risk set instead of raw proxy fields.
// Purpose: create one ETF-native summary that is auditable, compact, and stable
// enough to enter the existing balanced-scorecard and chair chains.
fn summarize_etf_proxy_signal(
    instrument_subscope: &str,
    proxy_inputs: &crate::ops::stock::security_decision_evidence_bundle::SecurityExternalProxyInputs,
) -> (String, String, Vec<String>, Vec<String>) {
    match instrument_subscope {
        "treasury_etf" => {
            let slope = proxy_inputs
                .yield_curve_slope_delta_bp_5d
                .unwrap_or_default();
            let funding = proxy_inputs
                .funding_liquidity_spread_delta_bp_5d
                .unwrap_or_default();
            let score = if slope <= 0.0 { 1 } else { -1 } + if funding <= 0.0 { 1 } else { -1 };
            let profit_signal = if score > 0 {
                "positive"
            } else if score < 0 {
                "negative"
            } else {
                "neutral"
            };
            let risks = if score < 0 {
                vec!["Treasury ETF proxy mix is still cautious on rates/liquidity.".to_string()]
            } else {
                vec![]
            };
            (
                profit_signal.to_string(),
                format!(
                    "yield-curve delta {:.2}bp and funding-liquidity delta {:.2}bp are governed and replayable",
                    slope, funding
                ),
                vec![
                    "yield_curve_proxy".to_string(),
                    "funding_liquidity_proxy".to_string(),
                ],
                risks,
            )
        }
        "gold_etf" => {
            let gold = proxy_inputs.gold_spot_proxy_return_5d.unwrap_or_default();
            let usd = proxy_inputs.usd_index_proxy_return_5d.unwrap_or_default();
            let real_rate = proxy_inputs.real_rate_proxy_delta_bp_5d.unwrap_or_default();
            let score = if gold > 0.0 { 1 } else { -1 }
                + if usd < 0.0 { 1 } else { -1 }
                + if real_rate < 0.0 { 1 } else { -1 };
            let profit_signal = if score > 0 {
                "positive"
            } else if score < 0 {
                "negative"
            } else {
                "neutral"
            };
            let risks = if score < 0 {
                vec!["Gold ETF proxy mix is not yet supportive enough.".to_string()]
            } else {
                vec![]
            };
            (
                profit_signal.to_string(),
                format!(
                    "gold {:.4}, USD {:.4}, and real-rate delta {:.2}bp are governed and replayable",
                    gold, usd, real_rate
                ),
                vec![
                    "gold_spot_proxy".to_string(),
                    "usd_index_proxy".to_string(),
                    "real_rate_proxy".to_string(),
                ],
                risks,
            )
        }
        "cross_border_etf" => {
            let fx = proxy_inputs.fx_return_5d.unwrap_or_default();
            let overseas = proxy_inputs.overseas_market_return_5d.unwrap_or_default();
            let gap = proxy_inputs.market_session_gap_days.unwrap_or_default();
            let score = if overseas > 0.0 { 1 } else { -1 }
                + if fx >= 0.0 { 1 } else { -1 }
                + if gap <= 1.0 { 1 } else { -1 };
            let profit_signal = if score > 0 {
                "positive"
            } else if score < 0 {
                "negative"
            } else {
                "neutral"
            };
            let risks = if score < 0 {
                vec![
                    "Cross-border ETF proxy mix still shows fragile overseas alignment."
                        .to_string(),
                ]
            } else {
                vec![]
            };
            (
                profit_signal.to_string(),
                format!(
                    "FX {:.4}, overseas market {:.4}, and session gap {:.0} day(s) are governed and replayable",
                    fx, overseas, gap
                ),
                vec![
                    "fx_proxy".to_string(),
                    "overseas_market_proxy".to_string(),
                    "market_session_gap".to_string(),
                ],
                risks,
            )
        }
        _ => {
            let fund_flow = proxy_inputs.etf_fund_flow_5d.unwrap_or_default();
            let premium = proxy_inputs.premium_discount_pct.unwrap_or_default();
            let relative = proxy_inputs
                .benchmark_relative_return_5d
                .unwrap_or_default();
            let score = if fund_flow >= 0.0 { 1 } else { -1 }
                + if premium.abs() <= 0.01 { 1 } else { -1 }
                + if relative > 0.0 { 1 } else { -1 };
            let profit_signal = if score > 0 {
                "positive"
            } else if score < 0 {
                "negative"
            } else {
                "neutral"
            };
            let risks = if score < 0 {
                vec![
                    "Equity ETF proxy mix still lacks enough flow/relative-strength support."
                        .to_string(),
                ]
            } else {
                vec![]
            };
            (
                profit_signal.to_string(),
                format!(
                    "fund flow {:.4}, premium-discount {:.4}, and benchmark-relative return {:.4} are governed and replayable",
                    fund_flow, premium, relative
                ),
                vec![
                    "etf_fund_flow_proxy".to_string(),
                    "premium_discount_proxy".to_string(),
                    "benchmark_relative_proxy".to_string(),
                ],
                risks,
            )
        }
    }
}

// 2026-04-12 CST: Expose a governed-history helper for validation backfill,
// because slice builders need one formal way to fetch live-compatible financial
// context before persisting it into slice-local governed storage.
// Purpose: avoid duplicating Eastmoney financial parsing outside fullstack.
pub(crate) fn fetch_live_fundamental_context_for_governed_history(
    symbol: &str,
) -> Result<FundamentalContext, String> {
    fetch_fundamental_context(symbol).map_err(|error| error.to_string())
}

// 2026-04-12 CST: Expose multi-period financial rows for governed backfill,
// because the new stock history tool must capture more than the latest snapshot
// before replay and promotion can trust the data thickness.
// Purpose: centralize EastMoney multi-period financial parsing inside fullstack.
pub(crate) fn fetch_live_fundamental_history_rows_for_governed_history(
    symbol: &str,
) -> Result<Vec<LiveFundamentalHistoryRow>, String> {
    let url = build_financial_url(symbol);
    let body = http_get_text(&url, "financials").map_err(|error| error.to_string())?;
    let payload = serde_json::from_str::<Value>(&body)
        .map_err(|error| format!("failed to parse financial payload: {error}"))?;
    let rows = extract_financial_rows(&payload).map_err(|error| error.to_string())?;
    let mut decoded_rows = Vec::new();
    let mut covered_periods = BTreeSet::new();

    for row in rows {
        let Some(report_period) = financial_string(
            row,
            &["REPORT_DATE", "REPORTDATE", "REPORT_DATE_NAME", "date"],
        )
        .map(normalize_date_like) else {
            continue;
        };
        if !covered_periods.insert(report_period.clone()) {
            continue;
        }
        let notice_date = financial_string(row, &["NOTICE_DATE", "NOTICEDATE", "latestNoticeDate"])
            .map(normalize_date_like);
        let report_metrics = FundamentalMetrics {
            revenue: financial_number(
                row,
                &[
                    "TOTAL_OPERATE_INCOME",
                    "TOTALOPERATEINCOME",
                    "钀ヤ笟鎬绘敹鍏?",
                    "yyzsr",
                ],
            ),
            revenue_yoy_pct: financial_number(
                row,
                &["YSTZ", "YYZSR_GTHR", "钀ヤ笟鎬绘敹鍏ュ悓姣?"],
            ),
            net_profit: financial_number(
                row,
                &[
                    "PARENT_NETPROFIT",
                    "PARENTNETPROFIT",
                    "褰掓瘝鍑€鍒╂鼎",
                    "gsjlr",
                ],
            ),
            net_profit_yoy_pct: financial_number(
                row,
                &["SJLTZ", "NETPROFIT_GTHR", "褰掓瘝鍑€鍒╂鼎鍚屾瘮"],
            ),
            roe_pct: financial_number(row, &["ROEJQ", "ROE_WEIGHTED", "jqjzcsyl"]),
        };
        decoded_rows.push(LiveFundamentalHistoryRow {
            report_period,
            notice_date,
            source: "eastmoney_financials".to_string(),
            report_metrics,
        });
    }

    if decoded_rows.is_empty() {
        return Err("no governed financial history rows were decoded".to_string());
    }

    Ok(decoded_rows)
}

// 2026-04-01 CST: 这里抓取最近公告摘要，原因是公告面是首版最有价值、且最适合走免费公开源的事件层信息；
// 目的：让产品在技术面之外还能同步看见“最近披露了什么”，而不是继续人工外查。
fn fetch_disclosure_context(
    symbol: &str,
    disclosure_limit: usize,
) -> Result<DisclosureContext, DisclosureFetchError> {
    let url = build_announcement_url(symbol, disclosure_limit);
    let body = http_get_text(&url, "announcements").map_err(DisclosureFetchError::Transport)?;
    let payload = serde_json::from_str::<Value>(&body)
        .map_err(|error| DisclosureFetchError::Parse(error.to_string()))?;
    let notices = extract_announcement_list(&payload)?;
    if notices.is_empty() {
        return Err(DisclosureFetchError::Empty);
    }

    let recent_announcements = notices
        .iter()
        .take(disclosure_limit)
        .map(|notice| DisclosureAnnouncement {
            published_at: notice
                .published_at
                .clone()
                .map(normalize_date_like)
                .unwrap_or_default(),
            title: notice.title.clone(),
            article_code: notice.article_code.clone(),
            category: notice.category.clone(),
        })
        .collect::<Vec<_>>();
    let keyword_summary = build_disclosure_keyword_summary(&recent_announcements);
    let risk_flags = build_disclosure_risk_flags(&recent_announcements);
    let headline = build_disclosure_headline(&recent_announcements, &risk_flags);

    Ok(DisclosureContext {
        status: "available".to_string(),
        source: "eastmoney_announcements".to_string(),
        announcement_count: recent_announcements.len(),
        headline,
        keyword_summary,
        recent_announcements,
        risk_flags,
    })
}

// 2026-04-12 CST: Expose a governed-history helper for validation backfill,
// because slice builders need one formal way to fetch live-compatible disclosure
// context before persisting it into slice-local governed storage.
// Purpose: avoid duplicating announcement parsing outside fullstack.
pub(crate) fn fetch_live_disclosure_context_for_governed_history(
    symbol: &str,
    disclosure_limit: usize,
) -> Result<DisclosureContext, String> {
    fetch_disclosure_context(symbol, disclosure_limit).map_err(|error| error.to_string())
}

// 2026-04-12 CST: Expose multi-page disclosure rows for governed backfill,
// because the new stock history tool must capture more than one recent page
// before replay can trust announcement coverage.
// Purpose: centralize paged announcement parsing inside fullstack.
pub(crate) fn fetch_live_disclosure_history_rows_for_governed_history(
    symbol: &str,
    page_size: usize,
    max_pages: usize,
) -> Result<Vec<LiveDisclosureHistoryRow>, String> {
    let effective_page_size = page_size.max(1).min(50);
    let effective_max_pages = max_pages.max(1);
    let mut decoded_rows = Vec::new();
    let mut seen_record_refs = BTreeSet::new();

    for page_index in 1..=effective_max_pages {
        let url = build_announcement_page_url(symbol, effective_page_size, page_index);
        let body = http_get_text(&url, "announcements").map_err(|error| error.to_string())?;
        let payload = serde_json::from_str::<Value>(&body)
            .map_err(|error| format!("failed to parse disclosure payload: {error}"))?;
        let notices = extract_announcement_list(&payload).map_err(|error| error.to_string())?;
        if notices.is_empty() {
            break;
        }

        let notice_count = notices.len();
        for notice in notices {
            let record_key = notice
                .article_code
                .clone()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| {
                    format!(
                        "{}:{}",
                        notice.published_at.clone().unwrap_or_default(),
                        notice.title
                    )
                });
            if !seen_record_refs.insert(record_key) {
                continue;
            }
            decoded_rows.push(LiveDisclosureHistoryRow {
                published_at: notice
                    .published_at
                    .clone()
                    .map(normalize_date_like)
                    .unwrap_or_default(),
                title: notice.title,
                article_code: notice.article_code,
                category: notice.category,
                source: "eastmoney_announcements".to_string(),
            });
        }

        if notice_count < effective_page_size {
            break;
        }
    }

    if decoded_rows.is_empty() {
        return Err("no governed disclosure history rows were decoded".to_string());
    }

    Ok(decoded_rows)
}

// 2026-04-01 CST: 这里抽行业上下文，原因是行业代理当前已经由 contextual Tool 产出完整技术结论；
// 目的：先把行业层提炼成单独对象，后续继续叠加行业景气源时不用改动总合同形状。
fn build_industry_context(technical_context: &SecurityAnalysisContextualResult) -> IndustryContext {
    IndustryContext {
        sector_symbol: technical_context.sector_symbol.clone(),
        proxy_bias: technical_context
            .sector_analysis
            .consultation_conclusion
            .bias
            .clone(),
        headline: technical_context
            .sector_analysis
            .consultation_conclusion
            .headline
            .clone(),
        rationale: technical_context
            .sector_analysis
            .consultation_conclusion
            .rationale
            .clone(),
        risk_flags: technical_context
            .sector_analysis
            .consultation_conclusion
            .risk_flags
            .clone(),
    }
}

// 2026-04-01 CST: 这里统一做综合 stance 判断，原因是产品需要一个最后可执行结论，而不是每个前端自己写一套拼装规则；
// 目的：把技术顺逆风、财报强弱和公告风险收敛成可稳定复用的总判断。
fn build_integrated_conclusion(
    technical_context: &SecurityAnalysisContextualResult,
    fundamental_context: &FundamentalContext,
    disclosure_context: &DisclosureContext,
    industry_context: &IndustryContext,
) -> IntegratedConclusion {
    let technical_alignment = technical_context.contextual_conclusion.alignment.as_str();
    let has_info_gap =
        fundamental_context.status != "available" || disclosure_context.status != "available";
    let has_disclosure_risk = !disclosure_context.risk_flags.is_empty();
    let has_fundamental_risk = fundamental_context.profit_signal == "negative";

    let stance = if has_info_gap {
        "technical_only".to_string()
    } else if technical_alignment == "tailwind"
        && fundamental_context.profit_signal == "positive"
        && !has_disclosure_risk
    {
        "constructive".to_string()
    } else if technical_alignment == "mixed"
        && fundamental_context.profit_signal == "positive"
        && !has_disclosure_risk
    {
        "watchful_positive".to_string()
    } else if technical_alignment == "headwind" || has_fundamental_risk || has_disclosure_risk {
        "cautious".to_string()
    } else {
        "mixed_watch".to_string()
    };

    let headline = match stance.as_str() {
        "constructive" => "技术环境与财报快照同向，当前更适合按偏积极综合结论跟踪。".to_string(),
        "watchful_positive" => {
            "财报快照偏正面，但技术环境尚未完全顺风，当前更适合边观察边等待确认。".to_string()
        }
        "technical_only" => {
            "信息面源暂不可用，当前只能基于技术与行业代理给出临时结论。".to_string()
        }
        "cautious" => "技术、财报或公告层至少有一项未形成正向共振，当前宜保持谨慎。".to_string(),
        _ => "当前综合信息尚未形成单边优势，更适合作为观察性结论使用。".to_string(),
    };

    let mut rationale = vec![
        format!(
            "技术层当前为 {}，行业代理 {} 的 headline 为：{}",
            technical_context.contextual_conclusion.alignment,
            industry_context.sector_symbol,
            industry_context.headline
        ),
        format!(
            "财报层状态为 {}，利润信号为 {}。",
            fundamental_context.status, fundamental_context.profit_signal
        ),
        format!(
            "公告层状态为 {}，最近纳入 {} 条公告摘要。",
            disclosure_context.status, disclosure_context.announcement_count
        ),
    ];
    if fundamental_context.status == "available" {
        rationale.push(fundamental_context.headline.clone());
    }
    if disclosure_context.status == "available" {
        rationale.push(disclosure_context.headline.clone());
    }

    let mut risk_flags = Vec::new();
    if has_info_gap {
        risk_flags.push("财报面或公告面当前缺失，综合判断存在信息盲区".to_string());
    }
    risk_flags.extend(fundamental_context.risk_flags.clone());
    risk_flags.extend(disclosure_context.risk_flags.clone());
    if technical_alignment == "headwind" {
        risk_flags.push("技术环境仍处逆风，信息面正向也不宜直接替代价格确认".to_string());
    }

    IntegratedConclusion {
        stance,
        headline,
        rationale,
        risk_flags,
    }
}

// 2026-04-01 CST: 这里统一构造财报降级对象，原因是免费信息源波动不能拖垮主链；
// 目的：让上层明确知道“为什么缺失、缺的是哪一层”，而不是直接拿到一个泛化错误。
fn build_unavailable_fundamental_context(message: String) -> FundamentalContext {
    FundamentalContext {
        status: "unavailable".to_string(),
        source: "eastmoney_financials".to_string(),
        latest_report_period: None,
        report_notice_date: None,
        headline: "财报快照当前不可用，综合结论已退化为技术优先。".to_string(),
        profit_signal: "unknown".to_string(),
        report_metrics: FundamentalMetrics {
            revenue: None,
            revenue_yoy_pct: None,
            net_profit: None,
            net_profit_yoy_pct: None,
            roe_pct: None,
        },
        narrative: vec!["免费财报源当前未返回可消费数据，已跳过财报层聚合。".to_string()],
        risk_flags: vec![message],
    }
}

// 2026-04-01 CST: 这里统一构造公告降级对象，原因是公告源异常同样属于可降级信息层；
// 目的：保证产品主链在外部源波动时仍返回结构完整的 JSON 合同。
fn build_unavailable_disclosure_context(message: String) -> DisclosureContext {
    DisclosureContext {
        status: "unavailable".to_string(),
        source: "eastmoney_announcements".to_string(),
        announcement_count: 0,
        headline: "公告摘要当前不可用，综合结论未纳入事件驱动层信息。".to_string(),
        keyword_summary: vec![],
        recent_announcements: vec![],
        risk_flags: vec![message],
    }
}

fn build_financial_url(symbol: &str) -> String {
    let base = std::env::var("EXCEL_SKILL_EASTMONEY_FINANCIAL_URL_BASE").unwrap_or_else(|_| {
        // 2026-04-12 CST: Switch the default EastMoney financial endpoint to ZYZBAjaxNew,
        // because MainTargetAjax now returns 406/HTML in live traffic while the current
        // finance-analysis page loads governed key metrics from ZYZBAjaxNew.
        // Purpose: keep real governed financial-history backfill working outside mocked tests.
        "https://emweb.securities.eastmoney.com/PC_HSF10/NewFinanceAnalysis/ZYZBAjaxNew".to_string()
    });
    let separator = if base.contains('?') { "&" } else { "?" };
    format!(
        "{base}{separator}type=1&code={}",
        normalize_eastmoney_code(symbol)
    )
}

fn build_announcement_url(symbol: &str, disclosure_limit: usize) -> String {
    build_announcement_page_url(symbol, disclosure_limit.min(20), 1)
}

// 2026-04-12 CST: Split page-aware announcement URL building into one helper,
// because multi-page governed backfill must request arbitrary page indexes
// instead of being locked to the first page only.
// Purpose: let recent-context fetch and historical backfill share one URL contract.
fn build_announcement_page_url(symbol: &str, page_size: usize, page_index: usize) -> String {
    let base = std::env::var("EXCEL_SKILL_EASTMONEY_ANNOUNCEMENT_URL_BASE")
        .unwrap_or_else(|_| "https://np-anotice-stock.eastmoney.com/api/security/ann".to_string());
    let separator = if base.contains('?') { "&" } else { "?" };
    format!(
        "{base}{separator}sr=-1&page_size={}&page_index={}&ann_type=A&stock_list={}",
        page_size.min(50).max(1),
        page_index.max(1),
        normalize_plain_stock_code(symbol)
    )
}

fn http_get_text(url: &str, source_label: &str) -> Result<String, String> {
    match ureq::get(url).set("Accept", "application/json").call() {
        Ok(response) => response.into_string().map_err(|error| error.to_string()),
        Err(ureq::Error::Status(status, response)) => {
            let body = response.into_string().unwrap_or_default();
            Err(if body.is_empty() {
                format!("{source_label} HTTP {status}")
            } else {
                format!("{source_label} HTTP {status}: {body}")
            })
        }
        Err(ureq::Error::Transport(error)) => Err(error.to_string()),
    }
}

fn extract_latest_financial_row<'a>(
    payload: &'a Value,
) -> Result<&'a Value, FundamentalFetchError> {
    let rows = extract_financial_rows(payload)?;
    rows.first().copied().ok_or(FundamentalFetchError::Empty)
}

// 2026-04-12 CST: Centralize financial-row extraction, because the new live
// history backfill must decode all available provider rows while the legacy
// latest-context fetch still only needs the first row.
// Purpose: avoid duplicating payload-shape handling for latest and historical paths.
fn extract_financial_rows<'a>(payload: &'a Value) -> Result<Vec<&'a Value>, FundamentalFetchError> {
    if let Some(rows) = payload.as_array() {
        return Ok(rows.iter().collect());
    }
    if let Some(rows) = payload.get("data").and_then(|value| value.as_array()) {
        return Ok(rows.iter().collect());
    }
    Err(FundamentalFetchError::Parse(
        "财报源返回结构不符合预期".to_string(),
    ))
}

fn extract_announcement_list(
    payload: &Value,
) -> Result<Vec<RawAnnouncement>, DisclosureFetchError> {
    let list = payload
        .get("data")
        .and_then(|value| value.get("list"))
        .and_then(|value| value.as_array())
        .ok_or_else(|| DisclosureFetchError::Parse("公告源返回结构不符合预期".to_string()))?;

    Ok(list
        .iter()
        .filter_map(|item| {
            let title = item
                .get("title")
                .and_then(|value| value.as_str())?
                .trim()
                .to_string();
            if title.is_empty() {
                return None;
            }
            Some(RawAnnouncement {
                published_at: item
                    .get("notice_date")
                    .and_then(|value| value.as_str())
                    .map(|value| value.to_string()),
                title,
                article_code: item
                    .get("art_code")
                    .and_then(|value| value.as_str())
                    .map(|value| value.to_string()),
                category: item
                    .get("columns")
                    .and_then(|value| value.as_array())
                    .and_then(|columns| columns.first())
                    .and_then(|value| value.get("column_name"))
                    .and_then(|value| value.as_str())
                    .map(|value| value.to_string()),
            })
        })
        .collect())
}

fn financial_string(row: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        row.get(*key)
            .and_then(|value| value.as_str())
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

fn financial_number(row: &Value, keys: &[&str]) -> Option<f64> {
    keys.iter()
        .find_map(|key| row.get(*key))
        .and_then(value_as_f64)
}

fn value_as_f64(value: &Value) -> Option<f64> {
    if let Some(number) = value.as_f64() {
        return Some(number);
    }
    value
        .as_str()
        .and_then(|text| text.replace(',', "").trim().parse::<f64>().ok())
}

fn classify_fundamental_signal(metrics: &FundamentalMetrics) -> String {
    match (metrics.revenue_yoy_pct, metrics.net_profit_yoy_pct) {
        (Some(revenue), Some(profit)) if revenue >= 0.0 && profit >= 0.0 => "positive".to_string(),
        (Some(revenue), Some(profit)) if revenue < 0.0 && profit < 0.0 => "negative".to_string(),
        (Some(_), Some(_)) => "mixed".to_string(),
        _ => "unknown".to_string(),
    }
}

fn build_fundamental_narrative(
    metrics: &FundamentalMetrics,
    profit_signal: &str,
) -> (String, Vec<String>, Vec<String>) {
    let revenue_text = metrics
        .revenue_yoy_pct
        .map(|value| format!("营收同比 {:.2}%", value))
        .unwrap_or_else(|| "营收同比暂缺".to_string());
    let profit_text = metrics
        .net_profit_yoy_pct
        .map(|value| format!("归母净利润同比 {:.2}%", value))
        .unwrap_or_else(|| "归母净利润同比暂缺".to_string());
    let roe_text = metrics
        .roe_pct
        .map(|value| format!("ROE {:.2}%", value))
        .unwrap_or_else(|| "ROE 暂缺".to_string());

    let headline = match profit_signal {
        "positive" => "最新财报显示营收和归母净利润保持同比增长。".to_string(),
        "negative" => "最新财报显示营收和归母净利润同步承压。".to_string(),
        "mixed" => "最新财报的收入与利润表现分化，需防止单项指标掩盖真实经营波动。".to_string(),
        _ => "最新财报只返回了部分指标，当前更适合把财报层视作辅助观察。".to_string(),
    };

    let narrative = vec![
        headline.clone(),
        format!("{revenue_text}，{profit_text}。"),
        format!("盈利质量观察位可继续结合 {roe_text} 与后续现金流披露确认。"),
    ];

    let mut risk_flags = Vec::new();
    if metrics.net_profit_yoy_pct.is_some_and(|value| value < 0.0) {
        risk_flags.push("归母净利润同比为负，后续估值修复弹性可能受限".to_string());
    }
    if metrics.revenue_yoy_pct.is_some_and(|value| value < 0.0) {
        risk_flags.push("营收同比为负，需警惕需求或价格压力继续传导".to_string());
    }
    if metrics.roe_pct.is_some_and(|value| value < 8.0) {
        risk_flags.push("ROE 偏低，盈利效率仍需后续报告进一步验证".to_string());
    }
    if metrics.revenue_yoy_pct.is_none() || metrics.net_profit_yoy_pct.is_none() {
        risk_flags.push("财报关键同比指标不完整，当前解读存在缺口".to_string());
    }

    (headline, narrative, risk_flags)
}

fn build_disclosure_keyword_summary(notices: &[DisclosureAnnouncement]) -> Vec<String> {
    let mut summary = Vec::new();
    if notices
        .iter()
        .any(|notice| contains_any(&notice.title, &["年度报告", "年报"]))
    {
        summary.push("最近公告包含年度报告".to_string());
    }
    if notices
        .iter()
        .any(|notice| contains_any(&notice.title, &["利润分配", "分红"]))
    {
        summary.push("最近公告包含利润分配或分红信息".to_string());
    }
    if notices
        .iter()
        .any(|notice| contains_any(&notice.title, &["回购", "增持"]))
    {
        summary.push("最近公告包含回购或增持类事项".to_string());
    }
    if summary.is_empty() {
        summary.push("最近公告暂未识别出高频正向事件关键词".to_string());
    }
    summary
}

fn build_disclosure_risk_flags(notices: &[DisclosureAnnouncement]) -> Vec<String> {
    let risk_keywords = [
        ("减持", "最近公告含减持事项，需留意筹码压力"),
        ("问询", "最近公告含问询事项，需留意监管关注点"),
        ("诉讼", "最近公告含诉讼事项，需留意经营不确定性"),
        ("终止", "最近公告含终止事项，需留意原有催化是否失效"),
        (
            "风险提示",
            "最近公告含风险提示，需关注公司主动披露的不确定性",
        ),
        ("预亏", "最近公告含预亏信息，需重新评估盈利预期"),
        ("亏损", "最近公告含亏损相关信息，需警惕业绩压力"),
    ];
    let mut flags = Vec::new();
    for notice in notices {
        for (keyword, message) in risk_keywords {
            if notice.title.contains(keyword) && !flags.iter().any(|flag| flag == message) {
                flags.push(message.to_string());
            }
        }
    }
    flags
}

fn build_disclosure_headline(notices: &[DisclosureAnnouncement], risk_flags: &[String]) -> String {
    if !risk_flags.is_empty() {
        return "最近公告中已出现需要重点复核的风险关键词，信息面不宜按纯正向理解。".to_string();
    }
    if notices
        .iter()
        .any(|notice| contains_any(&notice.title, &["年度报告", "年报"]))
    {
        return "最近公告以定期披露为主，信息面暂未看到明显负向事件。".to_string();
    }
    if notices
        .iter()
        .any(|notice| contains_any(&notice.title, &["回购", "增持"]))
    {
        return "最近公告含回购或增持类事项，事件层对情绪存在一定支撑。".to_string();
    }
    "最近公告以常规定期披露和公司事项为主，暂未识别到强风险事件。".to_string()
}

fn contains_any(title: &str, keywords: &[&str]) -> bool {
    keywords.iter().any(|keyword| title.contains(keyword))
}

fn normalize_eastmoney_code(symbol: &str) -> String {
    let normalized = symbol.trim().to_uppercase();
    if let Some((code, exchange)) = normalized.split_once('.') {
        return format!("{exchange}{code}");
    }
    if normalized.len() == 6 {
        let exchange = if normalized.starts_with(['6', '9']) {
            "SH"
        } else {
            "SZ"
        };
        return format!("{exchange}{normalized}");
    }
    normalized
}

fn normalize_plain_stock_code(symbol: &str) -> String {
    symbol
        .trim()
        .split('.')
        .next()
        .unwrap_or(symbol)
        .to_string()
}

fn normalize_date_like(value: String) -> String {
    value.chars().take(10).collect()
}

fn default_lookback_days() -> usize {
    260
}

fn default_disclosure_limit() -> usize {
    DEFAULT_DISCLOSURE_LIMIT
}

#[derive(Debug, Clone, PartialEq)]
struct RawAnnouncement {
    published_at: Option<String>,
    title: String,
    article_code: Option<String>,
    category: Option<String>,
}
