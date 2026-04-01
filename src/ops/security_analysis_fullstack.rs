use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::ops::stock::security_analysis_contextual::{
    security_analysis_contextual, SecurityAnalysisContextualError,
    SecurityAnalysisContextualRequest, SecurityAnalysisContextualResult,
};

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
#[derive(Debug, Clone, PartialEq, Serialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FundamentalMetrics {
    pub revenue: Option<f64>,
    pub revenue_yoy_pct: Option<f64>,
    pub net_profit: Option<f64>,
    pub net_profit_yoy_pct: Option<f64>,
    pub roe_pct: Option<f64>,
}

// 2026-04-01 CST: 这里定义公告摘要合同，原因是首版公告面重点是“最近披露了什么、有没有明显风险关键词”；
// 目的：先把公告层做成稳定摘要入口，而不是引入大模型做自由文本总结。
#[derive(Debug, Clone, PartialEq, Serialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DisclosureAnnouncement {
    pub published_at: String,
    pub title: String,
    pub article_code: Option<String>,
    pub category: Option<String>,
}

// 2026-04-01 CST: 这里定义行业上下文，原因是行业层首版先沿用 sector proxy 技术结论，但要给上层稳定的独立消费字段；
// 目的：把“行业环境”从 technical_context 的深层嵌套里提炼出来，便于后续继续叠加行业景气数据。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct IndustryContext {
    pub sector_symbol: String,
    pub proxy_bias: String,
    pub headline: String,
    pub rationale: Vec<String>,
    pub risk_flags: Vec<String>,
}

// 2026-04-01 CST: 这里定义总综合结论，原因是产品真正需要的是最终 stance，而不是调用方自己重新读完所有子结果再拼判断；
// 目的：把技术面与信息面冲突/共振关系收敛成统一 headline + rationale + risk_flags。
#[derive(Debug, Clone, PartialEq, Serialize)]
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
    let fundamental_context = match fetch_fundamental_context(&request.symbol) {
        Ok(context) => context,
        Err(error) => build_unavailable_fundamental_context(error.to_string()),
    };
    let disclosure_context =
        match fetch_disclosure_context(&request.symbol, request.disclosure_limit.max(1)) {
            Ok(context) => context,
            Err(error) => build_unavailable_disclosure_context(error.to_string()),
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
        "https://emweb.securities.eastmoney.com/PC_HSF10/NewFinanceAnalysis/MainTargetAjax"
            .to_string()
    });
    let separator = if base.contains('?') { "&" } else { "?" };
    format!(
        "{base}{separator}type=1&code={}",
        normalize_eastmoney_code(symbol)
    )
}

fn build_announcement_url(symbol: &str, disclosure_limit: usize) -> String {
    let base = std::env::var("EXCEL_SKILL_EASTMONEY_ANNOUNCEMENT_URL_BASE")
        .unwrap_or_else(|_| "https://np-anotice-stock.eastmoney.com/api/security/ann".to_string());
    let separator = if base.contains('?') { "&" } else { "?" };
    format!(
        "{base}{separator}sr=-1&page_size={}&page_index=1&ann_type=A&stock_list={}",
        disclosure_limit.min(20),
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
    if let Some(rows) = payload.as_array() {
        return rows.first().ok_or(FundamentalFetchError::Empty);
    }
    if let Some(rows) = payload.get("data").and_then(|value| value.as_array()) {
        return rows.first().ok_or(FundamentalFetchError::Empty);
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
