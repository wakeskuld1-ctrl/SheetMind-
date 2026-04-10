use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::ops::stock::security_analysis_contextual::{
    SecurityAnalysisContextualError, SecurityAnalysisContextualRequest,
    SecurityAnalysisContextualResult, security_analysis_contextual,
};
use crate::ops::stock::stock_analysis_data_guard::StockAnalysisDateGuard;

const DEFAULT_DISCLOSURE_LIMIT: usize = 8;
const DEFAULT_SINA_FINANCIAL_URL_BASE: &str =
    "https://vip.stock.finance.sina.com.cn/corp/go.php/vFD_FinancialGuideLine";
const DEFAULT_SINA_ANNOUNCEMENT_URL_BASE: &str =
    "https://vip.stock.finance.sina.com.cn/corp/go.php/vCB_AllBulletin";

// 2026-04-02 CST: 这里重写 fullstack 请求结构旁的说明，原因是当前证券分析主链已经从“单东财抓取”升级成“多源降级聚合”；
// 目的：让调用方继续沿用原有入参，但底层可以透明切到东财、官方备源和新浪备源，不再把网络可达性暴露给上层。
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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityAnalysisFullstackResult {
    pub symbol: String,
    // 2026-04-08 CST: 这里新增统一分析日期字段，原因是方案 C 要把公共合同从 briefing 下沉到 fullstack 层；
    // 目的：让后续 briefing / committee / agent 可以直接消费 fullstack 顶层日期，而不必回钻 nested technical_context。
    pub analysis_date: String,
    // 2026-04-08 CST: 这里新增证据版本字段，原因是 fullstack 聚合了技术面、财报和公告，需要稳定的证据快照版本号；
    // 目的：为后续链路提供统一的事实版本引用，避免只靠 symbol 或隐式嵌套字段判断版本。
    pub evidence_version: String,
    pub analysis_date_guard: StockAnalysisDateGuard,
    pub technical_context: SecurityAnalysisContextualResult,
    pub fundamental_context: FundamentalContext,
    pub disclosure_context: DisclosureContext,
    pub industry_context: IndustryContext,
    pub integrated_conclusion: IntegratedConclusion,
}

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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct FundamentalMetrics {
    pub revenue: Option<f64>,
    pub revenue_yoy_pct: Option<f64>,
    pub net_profit: Option<f64>,
    pub net_profit_yoy_pct: Option<f64>,
    pub roe_pct: Option<f64>,
}

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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DisclosureAnnouncement {
    pub published_at: String,
    pub title: String,
    pub article_code: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct IndustryContext {
    pub sector_symbol: String,
    pub proxy_bias: String,
    pub headline: String,
    pub rationale: Vec<String>,
    pub risk_flags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct IntegratedConclusion {
    pub stance: String,
    pub headline: String,
    pub rationale: Vec<String>,
    pub risk_flags: Vec<String>,
}

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

#[derive(Debug, Clone, PartialEq)]
struct RawAnnouncement {
    published_at: Option<String>,
    title: String,
    article_code: Option<String>,
    category: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
struct ParsedHtmlRow {
    html: String,
    cells: Vec<String>,
}

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
    // 2026-04-08 CST: 这里沿用技术上下文的统一日期生成 fullstack 顶层合同字段，原因是聚合链路必须对齐同一分析时点；
    // 目的：确保顶层 fullstack 合同能稳定暴露 `analysis_date / evidence_version`，供更高层直接复用。
    let analysis_date = technical_context.analysis_date.clone();
    let evidence_version = format!(
        "security-analysis-fullstack:{}:{}:v1",
        request.symbol, analysis_date
    );

    Ok(SecurityAnalysisFullstackResult {
        symbol: request.symbol.clone(),
        analysis_date,
        evidence_version,
        analysis_date_guard: technical_context.analysis_date_guard.clone(),
        technical_context,
        fundamental_context,
        disclosure_context,
        industry_context,
        integrated_conclusion,
    })
}

// 2026-04-02 CST: 这里把财报抓取改成三层 provider 链，原因是用户现场已经确认东财在本机网络下会稳定 TLS 失败；
// 目的：先走东财主源，再尝试可插拔官方源，最后退到新浪公开页，尽量把 technical_only 缩到真正全链路都失效时。
fn fetch_fundamental_context(symbol: &str) -> Result<FundamentalContext, FundamentalFetchError> {
    let mut attempt_errors = Vec::new();

    match fetch_fundamental_from_eastmoney(symbol) {
        Ok(context) => return Ok(context),
        Err(error) => attempt_errors.push(format!("eastmoney_financials: {error}")),
    }

    if let Some(url) = build_optional_official_financial_url(symbol) {
        match fetch_fundamental_from_official_json(&url) {
            Ok(context) => return Ok(context),
            Err(error) => attempt_errors.push(format!("official_financials: {error}")),
        }
    }

    match fetch_fundamental_from_sina_resilient(symbol) {
        Ok(context) => return Ok(context),
        Err(error) => attempt_errors.push(format!("sina_financial_guideline: {error}")),
    }

    Err(FundamentalFetchError::Transport(attempt_errors.join(" | ")))
}

// 2026-04-02 CST: 这里把公告抓取也改成三层 provider 链，原因是单一公告源比财报源更容易受 TLS、限流和前端改版影响；
// 目的：把公告摘要稳定在“主源失败仍可继续返回最近公告”的产品语义上，避免上层每次都手动补公告信息。
fn fetch_disclosure_context(
    symbol: &str,
    disclosure_limit: usize,
) -> Result<DisclosureContext, DisclosureFetchError> {
    let mut attempt_errors = Vec::new();

    match fetch_disclosure_from_eastmoney(symbol, disclosure_limit) {
        Ok(context) => return Ok(context),
        Err(error) => attempt_errors.push(format!("eastmoney_announcements: {error}")),
    }

    if let Some(url) = build_optional_official_announcement_url(symbol, disclosure_limit) {
        match fetch_disclosure_from_official_json(&url, disclosure_limit) {
            Ok(context) => return Ok(context),
            Err(error) => attempt_errors.push(format!("official_announcements: {error}")),
        }
    }

    match fetch_disclosure_from_sina(symbol, disclosure_limit) {
        Ok(context) => return Ok(context),
        Err(error) => attempt_errors.push(format!("sina_announcements: {error}")),
    }

    Err(DisclosureFetchError::Transport(attempt_errors.join(" | ")))
}

fn fetch_fundamental_from_eastmoney(
    symbol: &str,
) -> Result<FundamentalContext, FundamentalFetchError> {
    let url = build_eastmoney_financial_url(symbol);
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

    Ok(build_available_fundamental_context(
        "eastmoney_financials",
        latest_report_period,
        report_notice_date,
        metrics,
    ))
}

fn fetch_fundamental_from_official_json(
    url: &str,
) -> Result<FundamentalContext, FundamentalFetchError> {
    let body =
        http_get_text(url, "official_financials").map_err(FundamentalFetchError::Transport)?;
    let payload = serde_json::from_str::<Value>(&body)
        .map_err(|error| FundamentalFetchError::Parse(error.to_string()))?;
    let root = extract_first_object(&payload).ok_or(FundamentalFetchError::Empty)?;
    let metrics_node = root
        .get("report_metrics")
        .or_else(|| root.get("metrics"))
        .unwrap_or(root);
    let metrics = FundamentalMetrics {
        revenue: json_number(
            metrics_node,
            &["revenue", "operate_income", "total_operate_income"],
        ),
        revenue_yoy_pct: json_number(metrics_node, &["revenue_yoy_pct", "operate_income_yoy_pct"]),
        net_profit: json_number(metrics_node, &["net_profit", "parent_netprofit", "profit"]),
        net_profit_yoy_pct: json_number(metrics_node, &["net_profit_yoy_pct", "profit_yoy_pct"]),
        roe_pct: json_number(metrics_node, &["roe_pct", "roe"]),
    };
    if metrics == empty_fundamental_metrics()
        && json_string(root, &["latest_report_period", "report_period"]).is_none()
    {
        return Err(FundamentalFetchError::Empty);
    }

    Ok(build_available_fundamental_context(
        root.get("source")
            .and_then(Value::as_str)
            .unwrap_or("official_financials"),
        json_string(root, &["latest_report_period", "report_period"]).map(normalize_date_like),
        json_string(root, &["report_notice_date", "notice_date"]).map(normalize_date_like),
        metrics,
    ))
}

#[allow(dead_code)]
fn fetch_fundamental_from_sina(symbol: &str) -> Result<FundamentalContext, FundamentalFetchError> {
    let url = build_sina_financial_url(symbol);
    let body = http_get_text(&url, "sina_financials").map_err(FundamentalFetchError::Transport)?;
    let rows = parse_html_rows_with_raw(&body);
    if rows.is_empty() {
        return Err(FundamentalFetchError::Parse(
            "新浪财务页没有可解析表格".to_string(),
        ));
    }

    let mut latest_report_period = None;
    let mut metrics = FundamentalMetrics {
        revenue: None,
        revenue_yoy_pct: None,
        net_profit: None,
        net_profit_yoy_pct: None,
        roe_pct: None,
    };

    for row in rows {
        let Some(label) = row.cells.first().map(|value| value.trim()) else {
            continue;
        };
        // 2026-04-02 CST: 这里给新浪财报解析补“标签归一化 + typecode 锚点”双保险，原因是线上页面在当前链路里可能出现乱码标签；
        // 目的：只要关键行的结构锚点还在，就继续识别净利增速、ROE 和报告期，避免财报备源被误判为空。
        let _normalized_label = normalize_sina_financial_label(label);
        let first_value = row
            .cells
            .iter()
            .skip(1)
            .find(|value| !value.trim().is_empty() && value.trim() != "--")
            .map(|value| value.trim().to_string());

        if label.contains("报告日期") {
            latest_report_period = first_value.map(normalize_date_like);
            continue;
        }
        if label.contains("营业总收入(元)")
            || label.contains("主营业务收入(元)")
            || label.contains("营业收入(元)")
        {
            metrics.revenue = first_value.as_deref().and_then(parse_number_text);
            continue;
        }
        if label.contains("营业总收入增长率(%)")
            || label.contains("主营业务收入增长率(%)")
            || label.contains("营业收入增长率(%)")
        {
            metrics.revenue_yoy_pct = first_value.as_deref().and_then(parse_number_text);
            continue;
        }
        if label.contains("归母净利润(元)") || label.contains("净利润(元)") {
            metrics.net_profit = first_value.as_deref().and_then(parse_number_text);
            continue;
        }
        if label.contains("归母净利润增长率(%)") || label.contains("净利润增长率(%)")
        {
            metrics.net_profit_yoy_pct = first_value.as_deref().and_then(parse_number_text);
            continue;
        }
        if label.contains("净资产收益率(%)") || label.contains("加权净资产收益率(%)")
        {
            if metrics.roe_pct.is_none() {
                metrics.roe_pct = first_value.as_deref().and_then(parse_number_text);
            }
        }
    }

    if latest_report_period.is_none() && metrics == empty_fundamental_metrics() {
        return Err(FundamentalFetchError::Empty);
    }

    Ok(build_available_fundamental_context(
        "sina_financial_guideline",
        latest_report_period,
        None,
        metrics,
    ))
}

// 2026-04-02 CST: 这里新增更稳的新浪财报解析分支，原因是线上真实页面在当前链路里会出现乱码标签，但 HTML 结构与 typecode 仍稳定；
// 目的：通过“日期行识别 + financialratios typecode + 归一化标签”三层兜底，让财报备源在真实环境中恢复可用。
fn fetch_fundamental_from_sina_resilient(
    symbol: &str,
) -> Result<FundamentalContext, FundamentalFetchError> {
    let url = build_sina_financial_url(symbol);
    let body = http_get_text(&url, "sina_financials").map_err(FundamentalFetchError::Transport)?;
    let rows = parse_html_rows_with_raw(&body);
    if rows.is_empty() {
        return Err(FundamentalFetchError::Parse(
            "新浪财务页没有可解析表格".to_string(),
        ));
    }

    let mut latest_report_period = None;
    let mut metrics = FundamentalMetrics {
        revenue: None,
        revenue_yoy_pct: None,
        net_profit: None,
        net_profit_yoy_pct: None,
        roe_pct: None,
    };

    for row in rows {
        let Some(label) = row.cells.first().map(|value| value.trim()) else {
            continue;
        };
        let normalized_label = normalize_sina_financial_label(label);
        let first_value = row
            .cells
            .iter()
            .skip(1)
            .find(|value| !value.trim().is_empty() && value.trim() != "--")
            .map(|value| value.trim().to_string());

        if is_sina_report_period_row(&normalized_label, &row.cells) {
            latest_report_period = first_value.map(normalize_date_like);
            continue;
        }
        if is_sina_financial_metric_row(
            &normalized_label,
            &row.html,
            &[],
            &["营业总收入元", "主营业务收入元", "营业收入元"],
        ) {
            metrics.revenue = first_value.as_deref().and_then(parse_number_text);
            continue;
        }
        if is_sina_financial_metric_row(
            &normalized_label,
            &row.html,
            &["financialratios43"],
            &["营业总收入增长率", "主营业务收入增长率", "营业收入增长率"],
        ) {
            metrics.revenue_yoy_pct = first_value.as_deref().and_then(parse_number_text);
            continue;
        }
        if is_sina_financial_metric_row(
            &normalized_label,
            &row.html,
            &["financialratios57", "financialratios65"],
            &["归母净利润元", "净利润元", "扣除非经常性损益后的净利润元"],
        ) {
            metrics.net_profit = first_value.as_deref().and_then(parse_number_text);
            continue;
        }
        if is_sina_financial_metric_row(
            &normalized_label,
            &row.html,
            &["financialratios44"],
            &["归母净利润增长率", "净利润增长率"],
        ) {
            metrics.net_profit_yoy_pct = first_value.as_deref().and_then(parse_number_text);
            continue;
        }
        if is_sina_financial_metric_row(
            &normalized_label,
            &row.html,
            &["financialratios59", "financialratios62"],
            &["净资产收益率", "加权净资产收益率"],
        ) {
            if metrics.roe_pct.is_none() {
                metrics.roe_pct = first_value.as_deref().and_then(parse_number_text);
            }
        }
    }

    if latest_report_period.is_none() && metrics == empty_fundamental_metrics() {
        return Err(FundamentalFetchError::Empty);
    }

    Ok(build_available_fundamental_context(
        "sina_financial_guideline",
        latest_report_period,
        None,
        metrics,
    ))
}

fn fetch_disclosure_from_eastmoney(
    symbol: &str,
    disclosure_limit: usize,
) -> Result<DisclosureContext, DisclosureFetchError> {
    let url = build_eastmoney_announcement_url(symbol, disclosure_limit);
    let body = http_get_text(&url, "announcements").map_err(DisclosureFetchError::Transport)?;
    let payload = serde_json::from_str::<Value>(&body)
        .map_err(|error| DisclosureFetchError::Parse(error.to_string()))?;
    let notices = extract_announcement_list(&payload)?;
    if notices.is_empty() {
        return Err(DisclosureFetchError::Empty);
    }

    Ok(build_available_disclosure_context(
        "eastmoney_announcements",
        notices,
        disclosure_limit,
    ))
}

fn fetch_disclosure_from_official_json(
    url: &str,
    disclosure_limit: usize,
) -> Result<DisclosureContext, DisclosureFetchError> {
    let body =
        http_get_text(url, "official_announcements").map_err(DisclosureFetchError::Transport)?;
    let payload = serde_json::from_str::<Value>(&body)
        .map_err(|error| DisclosureFetchError::Parse(error.to_string()))?;
    let root = extract_first_object(&payload).ok_or(DisclosureFetchError::Empty)?;
    let list = root
        .get("recent_announcements")
        .or_else(|| root.get("announcements"))
        .and_then(Value::as_array)
        .ok_or_else(|| {
            DisclosureFetchError::Parse("官方公告备源缺少 announcements 数组".to_string())
        })?;

    let notices = list
        .iter()
        .filter_map(|item| {
            let title = json_string(item, &["title"])?;
            Some(RawAnnouncement {
                published_at: json_string(item, &["published_at", "notice_date", "date"]),
                title,
                article_code: json_string(item, &["article_code", "id", "code", "url"]),
                category: json_string(item, &["category"]),
            })
        })
        .collect::<Vec<_>>();
    if notices.is_empty() {
        return Err(DisclosureFetchError::Empty);
    }

    Ok(build_available_disclosure_context(
        root.get("source")
            .and_then(Value::as_str)
            .unwrap_or("official_announcements"),
        notices,
        disclosure_limit,
    ))
}

fn fetch_disclosure_from_sina(
    symbol: &str,
    disclosure_limit: usize,
) -> Result<DisclosureContext, DisclosureFetchError> {
    let url = build_sina_announcement_url(symbol);
    let body =
        http_get_text(&url, "sina_announcements").map_err(DisclosureFetchError::Transport)?;
    let regex = Regex::new(
        r#"(?is)(\d{4}-\d{2}-\d{2})\s*&nbsp;\s*<a[^>]*href=['"]([^'"]+)['"][^>]*>(.*?)</a>"#,
    )
    .map_err(|error| DisclosureFetchError::Parse(error.to_string()))?;

    let notices = regex
        .captures_iter(&body)
        .filter_map(|capture| {
            let title = strip_html_tags(capture.get(3)?.as_str());
            if title.trim().is_empty() {
                return None;
            }
            let href = capture.get(2)?.as_str().trim();
            Some(RawAnnouncement {
                published_at: Some(capture.get(1)?.as_str().to_string()),
                title,
                article_code: extract_query_param(href, "id")
                    .or_else(|| Some(to_absolute_sina_url(href))),
                category: Some("公司公告".to_string()),
            })
        })
        .collect::<Vec<_>>();
    if notices.is_empty() {
        return Err(DisclosureFetchError::Empty);
    }

    Ok(build_available_disclosure_context(
        "sina_announcements",
        notices,
        disclosure_limit,
    ))
}

fn build_available_fundamental_context(
    source: &str,
    latest_report_period: Option<String>,
    report_notice_date: Option<String>,
    metrics: FundamentalMetrics,
) -> FundamentalContext {
    let profit_signal = classify_fundamental_signal(&metrics);
    let (headline, narrative, risk_flags) = build_fundamental_narrative(&metrics, &profit_signal);

    FundamentalContext {
        status: "available".to_string(),
        source: source.to_string(),
        latest_report_period,
        report_notice_date,
        headline,
        profit_signal,
        report_metrics: metrics,
        narrative,
        risk_flags,
    }
}

fn build_available_disclosure_context(
    source: &str,
    notices: Vec<RawAnnouncement>,
    disclosure_limit: usize,
) -> DisclosureContext {
    let recent_announcements = notices
        .into_iter()
        .take(disclosure_limit)
        .map(|notice| DisclosureAnnouncement {
            published_at: notice
                .published_at
                .map(normalize_date_like)
                .unwrap_or_default(),
            title: notice.title,
            article_code: notice.article_code,
            category: notice.category,
        })
        .collect::<Vec<_>>();
    let keyword_summary = build_disclosure_keyword_summary(&recent_announcements);
    let risk_flags = build_disclosure_risk_flags(&recent_announcements);
    let headline = build_disclosure_headline(&recent_announcements, &risk_flags);

    DisclosureContext {
        status: "available".to_string(),
        source: source.to_string(),
        announcement_count: recent_announcements.len(),
        headline,
        keyword_summary,
        recent_announcements,
        risk_flags,
    }
}

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
        "constructive" => {
            "技术环境、财报快照和公告节奏形成了同向共振，当前更适合按偏积极的综合结论跟踪。"
                .to_string()
        }
        "watchful_positive" => {
            "财报快照偏正面，但技术环境仍在确认阶段，当前更适合作为边观察边等待确认的结论使用。"
                .to_string()
        }
        "technical_only" => {
            "信息面主链当前不可用，当前结论暂时只能以技术面和行业代理为主。".to_string()
        }
        "cautious" => {
            "技术面、财报面或公告面至少有一层没有形成正向共振，当前更适合保持谨慎。".to_string()
        }
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
        risk_flags.push("技术环境仍处逆风，信息面正向也不能直接替代价格确认".to_string());
    }

    IntegratedConclusion {
        stance,
        headline,
        rationale,
        risk_flags,
    }
}

fn build_unavailable_fundamental_context(message: String) -> FundamentalContext {
    FundamentalContext {
        status: "unavailable".to_string(),
        source: "multi_source_fallback".to_string(),
        latest_report_period: None,
        report_notice_date: None,
        headline: "财报快照当前不可用，综合结论已退化为技术优先。".to_string(),
        profit_signal: "unknown".to_string(),
        report_metrics: empty_fundamental_metrics(),
        narrative: vec!["多源财报抓取均未返回可消费数据，当前已跳过财报层聚合。".to_string()],
        risk_flags: vec![message],
    }
}

fn build_unavailable_disclosure_context(message: String) -> DisclosureContext {
    DisclosureContext {
        status: "unavailable".to_string(),
        source: "multi_source_fallback".to_string(),
        announcement_count: 0,
        headline: "公告摘要当前不可用，综合结论未纳入事件驱动层信息。".to_string(),
        keyword_summary: vec![],
        recent_announcements: vec![],
        risk_flags: vec![message],
    }
}

fn build_eastmoney_financial_url(symbol: &str) -> String {
    let base = std::env::var("EXCEL_SKILL_EASTMONEY_FINANCIAL_URL_BASE").unwrap_or_else(|_| {
        "https://emweb.securities.eastmoney.com/PC_HSF10/NewFinanceAnalysis/MainTargetAjax"
            .to_string()
    });
    append_query_params(
        &base,
        &[
            ("type", "1".to_string()),
            ("code", normalize_eastmoney_code(symbol)),
        ],
    )
}

fn build_eastmoney_announcement_url(symbol: &str, disclosure_limit: usize) -> String {
    let base = std::env::var("EXCEL_SKILL_EASTMONEY_ANNOUNCEMENT_URL_BASE")
        .unwrap_or_else(|_| "https://np-anotice-stock.eastmoney.com/api/security/ann".to_string());
    append_query_params(
        &base,
        &[
            ("sr", "-1".to_string()),
            ("page_size", disclosure_limit.min(20).to_string()),
            ("page_index", "1".to_string()),
            ("ann_type", "A".to_string()),
            ("stock_list", normalize_plain_stock_code(symbol)),
        ],
    )
}

fn build_optional_official_financial_url(symbol: &str) -> Option<String> {
    std::env::var("EXCEL_SKILL_OFFICIAL_FINANCIAL_URL_BASE")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(|base| append_query_params(&base, &[("symbol", symbol.to_string())]))
}

fn build_optional_official_announcement_url(
    symbol: &str,
    disclosure_limit: usize,
) -> Option<String> {
    std::env::var("EXCEL_SKILL_OFFICIAL_ANNOUNCEMENT_URL_BASE")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(|base| {
            append_query_params(
                &base,
                &[
                    ("symbol", symbol.to_string()),
                    ("limit", disclosure_limit.to_string()),
                ],
            )
        })
}

fn build_sina_financial_url(symbol: &str) -> String {
    let plain = normalize_plain_stock_code(symbol);
    match std::env::var("EXCEL_SKILL_SINA_FINANCIAL_URL_BASE") {
        Ok(base) if !base.trim().is_empty() => {
            append_query_params(&base, &[("symbol", symbol.to_string()), ("stockid", plain)])
        }
        _ => format!("{DEFAULT_SINA_FINANCIAL_URL_BASE}/stockid/{plain}/displaytype/4.phtml"),
    }
}

fn build_sina_announcement_url(symbol: &str) -> String {
    let plain = normalize_plain_stock_code(symbol);
    match std::env::var("EXCEL_SKILL_SINA_ANNOUNCEMENT_URL_BASE") {
        Ok(base) if !base.trim().is_empty() => {
            append_query_params(&base, &[("symbol", symbol.to_string()), ("stockid", plain)])
        }
        _ => format!("{DEFAULT_SINA_ANNOUNCEMENT_URL_BASE}/stockid/{plain}.phtml"),
    }
}

fn append_query_params(base: &str, params: &[(&str, String)]) -> String {
    let separator = if base.contains('?') { '&' } else { '?' };
    let query = params
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("&");
    format!("{base}{separator}{query}")
}

fn http_get_text(url: &str, source_label: &str) -> Result<String, String> {
    match ureq::get(url)
        .set("Accept", "text/html,application/json;q=0.9,*/*;q=0.8")
        .call()
    {
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
        .and_then(Value::as_array)
        .ok_or_else(|| DisclosureFetchError::Parse("公告源返回结构不符合预期".to_string()))?;

    Ok(list
        .iter()
        .filter_map(|item| {
            let title = item
                .get("title")
                .and_then(Value::as_str)?
                .trim()
                .to_string();
            if title.is_empty() {
                return None;
            }
            Some(RawAnnouncement {
                published_at: item
                    .get("notice_date")
                    .and_then(Value::as_str)
                    .map(ToString::to_string),
                title,
                article_code: item
                    .get("art_code")
                    .and_then(Value::as_str)
                    .map(ToString::to_string),
                category: item
                    .get("columns")
                    .and_then(Value::as_array)
                    .and_then(|columns| columns.first())
                    .and_then(|value| value.get("column_name"))
                    .and_then(Value::as_str)
                    .map(ToString::to_string),
            })
        })
        .collect())
}

fn extract_first_object(payload: &Value) -> Option<&Value> {
    if payload.is_object() {
        return Some(payload);
    }
    payload.as_array().and_then(|rows| rows.first())
}

fn financial_string(row: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        row.get(*key)
            .and_then(Value::as_str)
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

fn financial_number(row: &Value, keys: &[&str]) -> Option<f64> {
    keys.iter()
        .find_map(|key| row.get(*key))
        .and_then(value_as_f64)
}

fn json_string(row: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        row.get(*key)
            .and_then(Value::as_str)
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

fn json_number(row: &Value, keys: &[&str]) -> Option<f64> {
    keys.iter()
        .find_map(|key| row.get(*key))
        .and_then(value_as_f64)
}

fn value_as_f64(value: &Value) -> Option<f64> {
    if let Some(number) = value.as_f64() {
        return Some(number);
    }
    value.as_str().and_then(parse_number_text)
}

fn parse_number_text(text: &str) -> Option<f64> {
    text.replace(',', "").trim().parse::<f64>().ok()
}

#[allow(dead_code)]
fn parse_html_table_rows(html: &str) -> Vec<Vec<String>> {
    let row_regex = Regex::new(r"(?is)<tr[^>]*>(.*?)</tr>").expect("row regex should compile");
    let cell_regex =
        Regex::new(r"(?is)<t[dh][^>]*>(.*?)</t[dh]>").expect("cell regex should compile");

    row_regex
        .captures_iter(html)
        .filter_map(|row_capture| {
            let row_html = row_capture.get(1)?.as_str();
            let cells = cell_regex
                .captures_iter(row_html)
                .filter_map(|cell_capture| cell_capture.get(1))
                .map(|cell| strip_html_tags(cell.as_str()))
                .collect::<Vec<_>>();
            if cells.is_empty() { None } else { Some(cells) }
        })
        .collect()
}

fn parse_html_rows_with_raw(html: &str) -> Vec<ParsedHtmlRow> {
    let row_regex = Regex::new(r"(?is)<tr[^>]*>(.*?)</tr>").expect("row regex should compile");
    let cell_regex =
        Regex::new(r"(?is)<t[dh][^>]*>(.*?)</t[dh]>").expect("cell regex should compile");

    row_regex
        .captures_iter(html)
        .filter_map(|row_capture| {
            let row_html = row_capture.get(1)?.as_str();
            let cells = cell_regex
                .captures_iter(row_html)
                .filter_map(|cell_capture| cell_capture.get(1))
                .map(|cell| strip_html_tags(cell.as_str()))
                .collect::<Vec<_>>();
            if cells.is_empty() {
                None
            } else {
                Some(ParsedHtmlRow {
                    html: row_html.to_string(),
                    cells,
                })
            }
        })
        .collect()
}

fn normalize_sina_financial_label(label: &str) -> String {
    label
        .replace("&nbsp;", "")
        .replace(" ", "")
        .replace('\u{a0}', "")
        .replace('\t', "")
        .replace('\r', "")
        .replace('\n', "")
        .replace('（', "(")
        .replace('）', ")")
        .replace('_', "")
        .replace(':', "")
        .replace('：', "")
}

fn is_sina_report_period_row(normalized_label: &str, cells: &[String]) -> bool {
    if normalized_label.contains("报告日期") {
        return true;
    }

    let date_like_count = cells
        .iter()
        .skip(1)
        .take(4)
        .filter(|value| looks_like_report_date(value))
        .count();

    date_like_count >= 2
}

fn looks_like_report_date(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.len() == 10
        && trimmed.chars().nth(4) == Some('-')
        && trimmed.chars().nth(7) == Some('-')
        && trimmed
            .chars()
            .enumerate()
            .all(|(idx, ch)| matches!(idx, 4 | 7) || ch.is_ascii_digit())
}

fn is_sina_financial_metric_row(
    normalized_label: &str,
    row_html: &str,
    typecodes: &[&str],
    label_keywords: &[&str],
) -> bool {
    typecodes.iter().any(|code| row_html.contains(code))
        || label_keywords
            .iter()
            .any(|keyword| normalized_label.contains(keyword))
}

fn strip_html_tags(html: &str) -> String {
    let tag_regex = Regex::new(r"(?is)<[^>]+>").expect("tag regex should compile");
    tag_regex
        .replace_all(html, "")
        .replace("&nbsp;", " ")
        .replace("&#160;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .trim()
        .to_string()
}

fn extract_query_param(url: &str, key: &str) -> Option<String> {
    let query = url.split('?').nth(1)?;
    query.split('&').find_map(|segment| {
        let (segment_key, value) = segment.split_once('=')?;
        if segment_key == key {
            Some(value.to_string())
        } else {
            None
        }
    })
}

fn to_absolute_sina_url(url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("https://vip.stock.finance.sina.com.cn{url}")
    }
}

fn classify_fundamental_signal(metrics: &FundamentalMetrics) -> String {
    let positives = [metrics.revenue_yoy_pct, metrics.net_profit_yoy_pct]
        .into_iter()
        .flatten()
        .filter(|value| *value >= 0.0)
        .count();
    let negatives = [metrics.revenue_yoy_pct, metrics.net_profit_yoy_pct]
        .into_iter()
        .flatten()
        .filter(|value| *value < 0.0)
        .count();

    match (positives, negatives) {
        (0, 0) => "unknown".to_string(),
        (0, _) => "negative".to_string(),
        (_, 0) => "positive".to_string(),
        _ => "mixed".to_string(),
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
        "positive" => "最新财报快照显示核心盈利指标仍保持正向。".to_string(),
        "negative" => "最新财报快照显示核心盈利指标正在承压。".to_string(),
        "mixed" => "最新财报快照的收入与利润表现分化，需避免单一指标误导。".to_string(),
        _ => "最新财报快照只返回了部分指标，当前更适合作为辅助观察。".to_string(),
    };

    let narrative = vec![
        headline.clone(),
        format!("{revenue_text}，{profit_text}。"),
        format!("盈利质量可继续结合 {roe_text} 与后续现金流披露确认。"),
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
        summary.push("最近公告包含年度报告或定期报告".to_string());
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
        return "最近公告中已经出现需要重点复核的风险关键词，信息面不宜按纯正向理解。".to_string();
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

fn empty_fundamental_metrics() -> FundamentalMetrics {
    FundamentalMetrics {
        revenue: None,
        revenue_yoy_pct: None,
        net_profit: None,
        net_profit_yoy_pct: None,
        roe_pct: None,
    }
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
