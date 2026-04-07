use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::ops::import_stock_price_history::ImportDateRange;
use crate::runtime::stock_history_store::{
    StockHistoryImportSummary, StockHistoryRow, StockHistoryStore, StockHistoryStoreError,
};

// 2026-03-29 CST: 这里定义 HTTP 股票历史同步请求，原因是腾讯/新浪双源同步需要独立于 CSV 导入的强类型合同；
// 目的：把 provider 顺序、日期区间和复权参数收口到一个稳定入口里，避免 dispatcher 手工散落解析。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SyncStockPriceHistoryRequest {
    pub symbol: String,
    pub start_date: String,
    pub end_date: String,
    #[serde(default = "default_adjustment")]
    pub adjustment: String,
    #[serde(default = "default_sync_providers")]
    pub providers: Vec<String>,
}

// 2026-03-29 CST: 这里定义 HTTP 股票历史同步结果，原因是外部 EXE 和后续 Skill 都需要知道最终命中的 provider；
// 目的：让调用方不仅知道导入成功，还知道这次实际走了腾讯还是新浪。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SyncStockPriceHistoryResult {
    pub symbol: String,
    pub provider_used: String,
    pub imported_row_count: usize,
    pub database_path: String,
    pub table_name: String,
    pub date_range: ImportDateRange,
}

// 2026-03-29 CST: 这里集中定义 HTTP 股票同步错误，原因是 symbol 归一化、HTTP、解析、日期和落库任一环节都可能失败；
// 目的：把外部老接口的不稳定性翻译成中文、可定位、可聚合的错误，而不是直接把底层异常抛给用户。
#[derive(Debug, Error)]
pub enum SyncStockPriceHistoryError {
    #[error("仅支持 qfq 前复权，当前收到 `{0}`")]
    UnsupportedAdjustment(String),
    #[error("股票代码 `{0}` 不是可识别的 A 股代码")]
    UnsupportedSymbol(String),
    #[error("开始日期 `{0}` 不是合法的 YYYY-MM-DD")]
    InvalidStartDate(String),
    #[error("结束日期 `{0}` 不是合法的 YYYY-MM-DD")]
    InvalidEndDate(String),
    #[error("开始日期不能晚于结束日期")]
    InvalidDateRange,
    #[error("未提供可用的 provider")]
    EmptyProviders,
    #[error("不支持的 provider: {0}")]
    UnsupportedProvider(String),
    #[error("provider `{provider}` 请求失败: {message}")]
    ProviderTransport { provider: String, message: String },
    #[error("provider `{provider}` 返回错误: {message}")]
    ProviderApi { provider: String, message: String },
    #[error("provider `{provider}` 响应解析失败: {message}")]
    ProviderParse { provider: String, message: String },
    #[error("provider `{provider}` 返回为空，未找到可导入日线")]
    ProviderEmpty { provider: String },
    #[error("所有 provider 均失败: {0}")]
    AllProvidersFailed(String),
    #[error("{0}")]
    Store(#[from] StockHistoryStoreError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SyncProvider {
    Tencent,
    Sina,
}

impl SyncProvider {
    fn as_str(self) -> &'static str {
        match self {
            SyncProvider::Tencent => "tencent",
            SyncProvider::Sina => "sina",
        }
    }

    fn from_name(value: &str) -> Result<Self, SyncStockPriceHistoryError> {
        match value.trim().to_lowercase().as_str() {
            "tencent" => Ok(Self::Tencent),
            "sina" => Ok(Self::Sina),
            other => Err(SyncStockPriceHistoryError::UnsupportedProvider(
                other.to_string(),
            )),
        }
    }
}

#[derive(Debug, Clone)]
struct ProviderRows {
    provider: SyncProvider,
    rows: Vec<StockHistoryRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProviderSymbol {
    prefixed_symbol: String,
}

#[derive(Debug, Clone, PartialEq)]
struct SyncDateWindow {
    start_date: NaiveDate,
    end_date: NaiveDate,
}

#[derive(Debug, Deserialize)]
struct TencentEnvelope {
    code: i32,
    #[serde(default)]
    msg: String,
    data: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct SinaKlineRow {
    day: String,
    open: String,
    high: String,
    low: String,
    close: String,
    volume: String,
}

// 2026-03-29 CST: 这里提供 HTTP 股票历史同步主入口，原因是用户已经确认要把腾讯 + 新浪并入现有股票历史底座；
// 目的：继续沿 `HTTP/CSV -> SQLite -> technical_consultation_basic` 主线增量推进，而不是另开新的行情存储链路。
pub fn sync_stock_price_history(
    request: &SyncStockPriceHistoryRequest,
) -> Result<SyncStockPriceHistoryResult, SyncStockPriceHistoryError> {
    if request.adjustment.trim().to_lowercase() != "qfq" {
        return Err(SyncStockPriceHistoryError::UnsupportedAdjustment(
            request.adjustment.clone(),
        ));
    }

    let provider_symbol = normalize_provider_symbol(&request.symbol)?;
    let window = parse_sync_date_window(&request.start_date, &request.end_date)?;
    let providers = parse_provider_order(&request.providers)?;

    let mut provider_errors = Vec::new();
    for provider in providers {
        match fetch_provider_rows(provider, &provider_symbol, &window, &request.adjustment) {
            Ok(provider_rows) => {
                let store = StockHistoryStore::workspace_default()?;
                let summary = store.import_rows(
                    &request.symbol,
                    &format!(
                        "{}_http_{}",
                        provider_rows.provider.as_str(),
                        request.adjustment
                    ),
                    &provider_rows.rows,
                )?;
                return Ok(build_sync_result(
                    request,
                    &store,
                    &summary,
                    provider_rows.provider,
                ));
            }
            Err(error) => provider_errors.push(error.to_string()),
        }
    }

    Err(SyncStockPriceHistoryError::AllProvidersFailed(
        provider_errors.join(" | "),
    ))
}

// 2026-03-29 CST: 这里统一构造 HTTP 同步回执，原因是 provider 成功和 SQLite 摘要都需要对外返回；
// 目的：固定外部 JSON 合同，避免存储层字段直接泄漏到 CLI 外部。
fn build_sync_result(
    request: &SyncStockPriceHistoryRequest,
    store: &StockHistoryStore,
    summary: &StockHistoryImportSummary,
    provider: SyncProvider,
) -> SyncStockPriceHistoryResult {
    SyncStockPriceHistoryResult {
        symbol: request.symbol.clone(),
        provider_used: provider.as_str().to_string(),
        imported_row_count: summary.imported_row_count,
        database_path: store.db_path().display().to_string(),
        table_name: "stock_price_history".to_string(),
        date_range: ImportDateRange {
            start_date: summary.start_date.clone(),
            end_date: summary.end_date.clone(),
        },
    }
}

// 2026-03-29 CST: 这里解析 provider 顺序，原因是这轮已经明确要先腾讯后新浪，但仍要允许请求方显式改优先级；
// 目的：把“顺序可配、范围受控”收口成稳定规则，而不是在业务主函数里到处判空和小写转换。
fn parse_provider_order(
    providers: &[String],
) -> Result<Vec<SyncProvider>, SyncStockPriceHistoryError> {
    if providers.is_empty() {
        return Err(SyncStockPriceHistoryError::EmptyProviders);
    }

    providers
        .iter()
        .map(|provider| SyncProvider::from_name(provider))
        .collect()
}

// 2026-03-29 CST: 这里解析日期窗口，原因是腾讯/新浪默认都可能返回比请求更多的日线；
// 目的：统一把外部源数据裁到目标时间范围内，保证后续 SQLite 只写入本次需要的窗口。
fn parse_sync_date_window(
    start_date: &str,
    end_date: &str,
) -> Result<SyncDateWindow, SyncStockPriceHistoryError> {
    let start_date = NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
        .map_err(|_| SyncStockPriceHistoryError::InvalidStartDate(start_date.to_string()))?;
    let end_date = NaiveDate::parse_from_str(end_date, "%Y-%m-%d")
        .map_err(|_| SyncStockPriceHistoryError::InvalidEndDate(end_date.to_string()))?;

    if start_date > end_date {
        return Err(SyncStockPriceHistoryError::InvalidDateRange);
    }

    Ok(SyncDateWindow {
        start_date,
        end_date,
    })
}

// 2026-03-29 CST: 这里统一归一化 A 股 symbol，原因是腾讯/新浪老接口都要求 `sh600519` / `sz000001` 这类前缀格式；
// 目的：兼容现有 `.SH / .SZ` 主线 symbol，同时不给后续 provider 解析层增加重复判断。
fn normalize_provider_symbol(symbol: &str) -> Result<ProviderSymbol, SyncStockPriceHistoryError> {
    let trimmed = symbol.trim().to_uppercase();
    let (code, exchange) = if let Some((code, exchange)) = trimmed.split_once('.') {
        (code.to_string(), exchange.to_string())
    } else if trimmed.len() == 6 {
        let exchange = if trimmed.starts_with(['6', '9']) {
            "SH".to_string()
        } else if trimmed.starts_with(['0', '2', '3']) {
            "SZ".to_string()
        } else {
            return Err(SyncStockPriceHistoryError::UnsupportedSymbol(
                symbol.to_string(),
            ));
        };
        (trimmed, exchange)
    } else {
        return Err(SyncStockPriceHistoryError::UnsupportedSymbol(
            symbol.to_string(),
        ));
    };

    let prefix = match exchange.as_str() {
        "SH" => "sh",
        "SZ" => "sz",
        _ => {
            return Err(SyncStockPriceHistoryError::UnsupportedSymbol(
                symbol.to_string(),
            ));
        }
    };

    Ok(ProviderSymbol {
        prefixed_symbol: format!("{prefix}{code}"),
    })
}

// 2026-03-29 CST: 这里按 provider 分派 HTTP 拉取，原因是双源虽然共用输出结构，但 URL 和解析规则完全不同；
// 目的：把“谁负责请求、谁负责解析”边界固定下来，减少后续继续加 provider 时的耦合。
fn fetch_provider_rows(
    provider: SyncProvider,
    provider_symbol: &ProviderSymbol,
    window: &SyncDateWindow,
    adjustment: &str,
) -> Result<ProviderRows, SyncStockPriceHistoryError> {
    let rows = match provider {
        SyncProvider::Tencent => fetch_tencent_rows(provider_symbol, window, adjustment)?,
        SyncProvider::Sina => fetch_sina_rows(provider_symbol, window)?,
    };

    if rows.is_empty() {
        return Err(SyncStockPriceHistoryError::ProviderEmpty {
            provider: provider.as_str().to_string(),
        });
    }

    Ok(ProviderRows { provider, rows })
}

// 2026-03-29 CST: 这里实现腾讯 fqkline 拉取，原因是这轮已确认腾讯应作为第一优先 provider；
// 目的：先打通“腾讯成功 -> SQLite”主路径，再用同一输出结构承接后备 provider。
fn fetch_tencent_rows(
    provider_symbol: &ProviderSymbol,
    window: &SyncDateWindow,
    adjustment: &str,
) -> Result<Vec<StockHistoryRow>, SyncStockPriceHistoryError> {
    let url = build_tencent_url(provider_symbol, window, adjustment);
    let body = http_get_text(SyncProvider::Tencent, &url)?;
    let envelope = serde_json::from_str::<TencentEnvelope>(&body).map_err(|error| {
        SyncStockPriceHistoryError::ProviderParse {
            provider: SyncProvider::Tencent.as_str().to_string(),
            message: error.to_string(),
        }
    })?;

    if envelope.code != 0 {
        return Err(SyncStockPriceHistoryError::ProviderApi {
            provider: SyncProvider::Tencent.as_str().to_string(),
            message: if envelope.msg.is_empty() {
                body
            } else {
                envelope.msg
            },
        });
    }

    let data = envelope
        .data
        .ok_or_else(|| SyncStockPriceHistoryError::ProviderEmpty {
            provider: SyncProvider::Tencent.as_str().to_string(),
        })?;
    let provider_data = data.get(&provider_symbol.prefixed_symbol).ok_or_else(|| {
        SyncStockPriceHistoryError::ProviderParse {
            provider: SyncProvider::Tencent.as_str().to_string(),
            message: "响应里缺少目标 symbol".to_string(),
        }
    })?;
    let field_name = format!("{}day", adjustment.trim().to_lowercase());
    let kline_rows = provider_data
        .get(&field_name)
        .or_else(|| provider_data.get("qfqday"))
        .and_then(|value| value.as_array())
        .ok_or_else(|| SyncStockPriceHistoryError::ProviderParse {
            provider: SyncProvider::Tencent.as_str().to_string(),
            message: "腾讯响应里缺少日线数组".to_string(),
        })?;

    let mut rows = Vec::new();
    for item in kline_rows {
        let values = item
            .as_array()
            .ok_or_else(|| SyncStockPriceHistoryError::ProviderParse {
                provider: SyncProvider::Tencent.as_str().to_string(),
                message: "腾讯日线行不是数组".to_string(),
            })?;
        if values.len() < 6 {
            return Err(SyncStockPriceHistoryError::ProviderParse {
                provider: SyncProvider::Tencent.as_str().to_string(),
                message: "腾讯日线字段数量不足".to_string(),
            });
        }
        let trade_date = value_as_str(values, 0, SyncProvider::Tencent)?;
        if !date_in_window(trade_date, window)? {
            continue;
        }
        let close = parse_provider_f64(
            SyncProvider::Tencent,
            value_as_str(values, 2, SyncProvider::Tencent)?,
        )?;
        rows.push(StockHistoryRow {
            trade_date: trade_date.to_string(),
            open: parse_provider_f64(
                SyncProvider::Tencent,
                value_as_str(values, 1, SyncProvider::Tencent)?,
            )?,
            close,
            high: parse_provider_f64(
                SyncProvider::Tencent,
                value_as_str(values, 3, SyncProvider::Tencent)?,
            )?,
            low: parse_provider_f64(
                SyncProvider::Tencent,
                value_as_str(values, 4, SyncProvider::Tencent)?,
            )?,
            adj_close: close,
            volume: parse_provider_i64(
                SyncProvider::Tencent,
                value_as_str(values, 5, SyncProvider::Tencent)?,
            )?,
        });
    }

    Ok(rows)
}

// 2026-03-29 CST: 这里实现新浪 KLine 拉取，原因是方案 2 明确要求腾讯失败时有稳定降级路径；
// 目的：把新浪收口成“只负责后备原始行情源”的最小实现，而不是引入第二套技术指标逻辑。
fn fetch_sina_rows(
    provider_symbol: &ProviderSymbol,
    window: &SyncDateWindow,
) -> Result<Vec<StockHistoryRow>, SyncStockPriceHistoryError> {
    let url = build_sina_url(provider_symbol, window);
    let body = http_get_text(SyncProvider::Sina, &url)?;
    let parsed_rows = serde_json::from_str::<Vec<SinaKlineRow>>(&body).map_err(|error| {
        SyncStockPriceHistoryError::ProviderParse {
            provider: SyncProvider::Sina.as_str().to_string(),
            message: error.to_string(),
        }
    })?;

    let mut rows = Vec::new();
    for item in parsed_rows {
        if !date_in_window(&item.day, window)? {
            continue;
        }
        let close = parse_provider_f64(SyncProvider::Sina, &item.close)?;
        rows.push(StockHistoryRow {
            trade_date: item.day,
            open: parse_provider_f64(SyncProvider::Sina, &item.open)?,
            high: parse_provider_f64(SyncProvider::Sina, &item.high)?,
            low: parse_provider_f64(SyncProvider::Sina, &item.low)?,
            close,
            adj_close: close,
            volume: parse_provider_i64(SyncProvider::Sina, &item.volume)?,
        });
    }

    Ok(rows)
}

// 2026-03-29 CST: 这里统一拼腾讯 URL，原因是线上默认地址和测试替换地址都需要共用一个出口；
// 目的：让测试可以通过环境变量把真实 HTTP 调用替换成本地假服务，而生产默认行为不受影响。
fn build_tencent_url(
    provider_symbol: &ProviderSymbol,
    window: &SyncDateWindow,
    adjustment: &str,
) -> String {
    if let Ok(url) = std::env::var("EXCEL_SKILL_TENCENT_KLINE_URL") {
        return url;
    }

    format!(
        "https://web.ifzq.gtimg.cn/appstock/app/fqkline/get?param={},day,{},{},640,{}",
        provider_symbol.prefixed_symbol,
        window.start_date.format("%Y-%m-%d"),
        window.end_date.format("%Y-%m-%d"),
        adjustment.trim().to_lowercase(),
    )
}

// 2026-03-29 CST: 这里统一拼新浪 URL，原因是线上默认地址和测试替换地址都需要共用一个出口；
// 目的：让新浪降级路径既能跑本地夹具，也能在生产环境尽量维持最小可用。
fn build_sina_url(provider_symbol: &ProviderSymbol, window: &SyncDateWindow) -> String {
    if let Ok(url) = std::env::var("EXCEL_SKILL_SINA_KLINE_URL") {
        return url;
    }

    let datalen = ((window.end_date - window.start_date).num_days().max(30) + 30) as i64;
    format!(
        "https://money.finance.sina.com.cn/quotes_service/api/json_v2.php/CN_MarketData.getKLineData?symbol={}&scale=240&ma=no&datalen={}",
        provider_symbol.prefixed_symbol, datalen
    )
}

// 2026-03-29 CST: 这里统一执行 GET 请求，原因是腾讯和新浪都走简单 GET；
// 目的：把状态码、网络异常和 body 读取错误统一翻译成 provider 级中文错误。
fn http_get_text(provider: SyncProvider, url: &str) -> Result<String, SyncStockPriceHistoryError> {
    match ureq::get(url).set("Accept", "application/json").call() {
        Ok(response) => {
            response
                .into_string()
                .map_err(|error| SyncStockPriceHistoryError::ProviderTransport {
                    provider: provider.as_str().to_string(),
                    message: error.to_string(),
                })
        }
        Err(ureq::Error::Status(status, response)) => {
            let body = response.into_string().unwrap_or_default();
            Err(SyncStockPriceHistoryError::ProviderApi {
                provider: provider.as_str().to_string(),
                message: if body.is_empty() {
                    format!("HTTP {status}")
                } else {
                    format!("HTTP {status}: {body}")
                },
            })
        }
        Err(ureq::Error::Transport(error)) => Err(SyncStockPriceHistoryError::ProviderTransport {
            provider: provider.as_str().to_string(),
            message: error.to_string(),
        }),
    }
}

// 2026-03-29 CST: 这里统一读取腾讯数组里的字符串字段，原因是老接口 JSON 结构不是强类型对象；
// 目的：把索引越界、类型错误集中翻译成清晰的 provider 解析错误。
fn value_as_str<'a>(
    values: &'a [Value],
    index: usize,
    provider: SyncProvider,
) -> Result<&'a str, SyncStockPriceHistoryError> {
    values
        .get(index)
        .and_then(|value| value.as_str())
        .ok_or_else(|| SyncStockPriceHistoryError::ProviderParse {
            provider: provider.as_str().to_string(),
            message: format!("第 {index} 个字段不是字符串"),
        })
}

// 2026-03-29 CST: 这里统一解析 provider 数值字段，原因是腾讯和新浪老接口都把数值塞成字符串；
// 目的：避免每个 provider 分支都重复写字符串转数值逻辑。
fn parse_provider_f64(
    provider: SyncProvider,
    value: &str,
) -> Result<f64, SyncStockPriceHistoryError> {
    value
        .parse::<f64>()
        .map_err(|error| SyncStockPriceHistoryError::ProviderParse {
            provider: provider.as_str().to_string(),
            message: format!("无法解析数值 `{value}`: {error}"),
        })
}

// 2026-03-29 CST: 这里统一解析 provider 成交量字段，原因是 volume 最终需要以整数落 SQLite；
// 目的：兼容字符串整数和带小数的字符串成交量，保持现有 store 结构不变。
fn parse_provider_i64(
    provider: SyncProvider,
    value: &str,
) -> Result<i64, SyncStockPriceHistoryError> {
    if let Ok(parsed) = value.parse::<i64>() {
        return Ok(parsed);
    }

    value
        .parse::<f64>()
        .map(|value| value.round() as i64)
        .map_err(|error| SyncStockPriceHistoryError::ProviderParse {
            provider: provider.as_str().to_string(),
            message: format!("无法解析成交量 `{value}`: {error}"),
        })
}

// 2026-03-29 CST: 这里统一按请求窗口过滤日期，原因是新浪默认返回长度型窗口、腾讯也可能超出目标区间；
// 目的：保证最终落库的仍是调用方明确请求的日期范围。
fn date_in_window(
    trade_date: &str,
    window: &SyncDateWindow,
) -> Result<bool, SyncStockPriceHistoryError> {
    let parsed = NaiveDate::parse_from_str(trade_date, "%Y-%m-%d").map_err(|error| {
        SyncStockPriceHistoryError::ProviderParse {
            provider: "date_filter".to_string(),
            message: format!("无法解析日期 `{trade_date}`: {error}"),
        }
    })?;
    Ok(parsed >= window.start_date && parsed <= window.end_date)
}

fn default_adjustment() -> String {
    "qfq".to_string()
}

fn default_sync_providers() -> Vec<String> {
    vec!["tencent".to_string(), "sina".to_string()]
}
