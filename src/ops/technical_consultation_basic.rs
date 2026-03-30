use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::runtime::stock_history_store::{
    StockHistoryRow, StockHistoryStore, StockHistoryStoreError,
};

const MIN_REQUIRED_HISTORY_ROWS: usize = 200;
const DEFAULT_LOOKBACK_DAYS: usize = 260;

// 2026-03-28 CST：这里定义技术面基础咨询请求，原因是新 Tool 需要稳定的强类型输入合同；
// 目的：把 dispatcher 的弱类型 JSON 参数收口为业务层可维护的 Rust 请求结构。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TechnicalConsultationBasicRequest {
    pub symbol: String,
    #[serde(default)]
    pub as_of_date: Option<String>,
    #[serde(default = "default_lookback_days")]
    pub lookback_days: usize,
}

// 2026-03-28 CST：这里定义技术面基础咨询结果，原因是 CLI / Skill / 后续 AI 都要消费同一份稳定 JSON 合同；
// 目的：先把第一版业务输出固定住，后续继续补指标时只做增量扩展，不再反复改对外结构。
// 2026-03-29 CST：这里追加 `volume_confirmation`，原因是下一刀要把技术面补成“方向 + 强度 + 量能确认”；
// 目的：让调用方直接拿到量价是否共振的结构化判断，而不是只读文案。
// 2026-03-29 CST：这里追加 `divergence_signal`，原因是量价确认之后下一步最自然的是补第一版价格-OBV 背离识别；
// 目的：让上层能直接知道当前是否存在顶部或底部背离风险，而不需要再重复解析价格和 OBV 关系。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TechnicalConsultationBasicResult {
    pub symbol: String,
    pub as_of_date: String,
    pub history_row_count: usize,
    pub trend_bias: String,
    pub trend_strength: String,
    pub volume_confirmation: String,
    // 2026-03-29 09:45 CST: 这里新增资金流信号字段，原因是 MFI 第一版需要以结构化语义进入咨询结果；
    // 目的：让上层 Skill / AI 直接消费 overbought / oversold / neutral，而不是自行解释快照数值。
    pub money_flow_signal: String,
    // 2026-03-30 09:35 CST: 这里新增均值回归信号字段，原因是 CCI(20) 第一版已确认要以结构化语义进入咨询结果；
    // 目的：让上层 Skill / AI 直接消费 overbought / oversold / neutral 的均值回归判断，而不是自行翻译数值阈值。
    pub mean_reversion_signal: String,
    // 2026-03-30 10:45 CST: 这里新增区间位置结构化信号，原因是 Williams %R(14) 第一版已确认要正式进入对外咨询合同；
    // 目的：让上层 Skill / AI 直接消费近期区间高位/低位/中性的语义，不再自己翻译 -20 / -80 阈值。
    pub range_position_signal: String,
    // 2026-03-29 23:25 CST: 这里新增布林带位置信号字段，原因是布林带第一版要正式把价格相对上下轨的位置语义接入对外合同；
    // 目的：让上层 Skill / AI 直接消费上轨突破风险、下轨反抽候选与中性位置，而不是自己再翻译 `close/boll_upper/boll_lower`。
    pub bollinger_position_signal: String,
    // 2026-03-29 10:35 CST: 这里新增布林带中轨语义字段，原因是当前合同只有上下轨极端位置，还缺少“中轨支撑/压制”的中间层表达；
    // 目的：让上层 Skill / AI 直接消费 `midline_support_bias / midline_resistance_bias / neutral`，统一布林带三层位置语义。
    pub bollinger_midline_signal: String,
    // 2026-03-29 23:25 CST: 这里新增布林带带宽状态字段，原因是现有 `volatility_state` 还不够表达布林带自身的收敛/扩张语义；
    // 目的：让调用方直接拿到 `expanding / contracting / normal`，避免后续外部重复解析带宽阈值。
    pub bollinger_bandwidth_signal: String,
    pub divergence_signal: String,
    pub timing_signal: String,
    pub rsrs_signal: String,
    pub momentum_signal: String,
    pub volatility_state: String,
    pub summary: String,
    pub recommended_actions: Vec<String>,
    pub watch_points: Vec<String>,
    pub indicator_snapshot: TechnicalIndicatorSnapshot,
    pub data_window_summary: DataWindowSummary,
}

// 2026-03-28 CST：这里单独定义指标快照结构，原因是技术面一条线后续会被多个 Skill / Tool 复用；
// 目的：让“咨询结论”和“原始指标”解耦，便于后续做更细的展示、审计和二次路由。
// 2026-03-29 CST：这里追加 ADX / +DI / -DI，原因是方案 A 要把趋势判断升级成“方向 + 强度”；
// 目的：让下游 AI 能直接拿到趋势强度快照，而不是再次自己解释 OHLC 序列。
// 2026-03-29 CST：这里追加 OBV 与量能均值快照，原因是本轮要补量价确认能力；
// 目的：把“价格方向是否得到量能配合”正式暴露给上层，避免外部自己重复计算成交量特征。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TechnicalIndicatorSnapshot {
    pub close: f64,
    pub ema_10: f64,
    pub sma_50: f64,
    pub sma_200: f64,
    pub adx_14: f64,
    pub plus_di_14: f64,
    pub minus_di_14: f64,
    pub obv: f64,
    pub volume_sma_20: f64,
    pub volume_ratio_20: f64,
    // 2026-03-29 09:45 CST: 这里新增 MFI 快照，原因是技术面中级指标本轮要继续沿同一份 OHLCV 历史窗口补齐资金流维度；
    // 目的：把最终语义结论与底层数值同时暴露给外部，方便后续审计、展示和二次路由。
    pub mfi_14: f64,
    // 2026-03-30 09:35 CST: 这里新增 CCI(20) 快照，原因是用户已批准把均值回归能力继续沿 Rust 主线接入技术面咨询；
    // 目的：把最终语义结论与底层 CCI 数值同时暴露给外部，便于后续审计、展示和二次路由。
    pub cci_20: f64,
    // 2026-03-30 10:45 CST: 这里新增 Williams %R(14) 快照，原因是区间位置能力这轮要沿现有合同稳定接入；
    // 目的：把最终区间语义与底层 williams_r 数值一起暴露给外部，便于后续展示、审计和二次路由。
    pub williams_r_14: f64,
    // 2026-03-29 23:25 CST: 这里新增布林带带宽快照，原因是布林带第一版既要给结构化语义，也要保留底层可审计数值；
    // 目的：把 `(boll_upper - boll_lower) / abs(boll_middle)` 直接暴露给外部，便于后续展示、审计与二次路由。
    pub boll_width_ratio_20: f64,
    pub macd: f64,
    pub macd_signal: f64,
    pub macd_histogram: f64,
    pub rsi_14: f64,
    pub k_9: f64,
    pub d_9: f64,
    pub j_9: f64,
    pub rsrs_beta_18: f64,
    pub rsrs_zscore_18_60: f64,
    pub boll_upper: f64,
    pub boll_middle: f64,
    pub boll_lower: f64,
    pub atr_14: f64,
}

// 2026-03-28 CST：这里补充数据窗口摘要，原因是技术咨询结论必须让后续 AI 知道本次判断覆盖了多长历史；
// 目的：减少排障时反复追问“这次结论到底基于多少数据、截止到哪一天”。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DataWindowSummary {
    pub requested_lookback_days: usize,
    pub loaded_row_count: usize,
    pub start_date: String,
    pub end_date: String,
}

// 2026-03-28 CST：这里集中定义咨询层错误，原因是读取历史、数据不足、指标计算都可能失败；
// 目的：继续保持面向用户的中文错误口径，并把“无数据”和“历史不足”明确区分。
#[derive(Debug, Error)]
pub enum TechnicalConsultationBasicError {
    #[error("{0}")]
    Store(#[from] StockHistoryStoreError),
    #[error("股票 `{symbol}` 没有可用的历史数据")]
    EmptyHistory { symbol: String },
    #[error("历史数据不足，至少需要 {required} 条，当前只有 {actual} 条")]
    InsufficientHistory { required: usize, actual: usize },
    #[error("技术指标计算失败: {0}")]
    IndicatorCalculation(String),
}

// 2026-03-28 CST：这里提供技术面基础咨询主入口，原因是当前要把 SQLite 历史正式接上 Rust 业务 Tool；
// 目的：在不引入额外脚本运行时和额外架构层的前提下，先交付一版可用、可解释、可回归的技术面能力。
pub fn technical_consultation_basic(
    request: &TechnicalConsultationBasicRequest,
) -> Result<TechnicalConsultationBasicResult, TechnicalConsultationBasicError> {
    let store = StockHistoryStore::workspace_default()?;
    let lookback_days = request.lookback_days.max(MIN_REQUIRED_HISTORY_ROWS);
    let rows = store.load_recent_rows(
        &request.symbol,
        request.as_of_date.as_deref(),
        lookback_days,
    )?;

    if rows.is_empty() {
        return Err(TechnicalConsultationBasicError::EmptyHistory {
            symbol: request.symbol.clone(),
        });
    };

    if rows.len() < MIN_REQUIRED_HISTORY_ROWS {
        return Err(TechnicalConsultationBasicError::InsufficientHistory {
            required: MIN_REQUIRED_HISTORY_ROWS,
            actual: rows.len(),
        });
    }

    let indicator_snapshot = build_indicator_snapshot(&rows)?;
    Ok(build_consultation_result(
        request,
        &rows,
        indicator_snapshot,
    ))
}

// 2026-03-28 CST：这里把默认回看窗口固定为 260，原因是第一版既要覆盖 200 日长期均线，又要给短周期指标留缓冲；
// 目的：让调用方即使不传参数，也能拿到相对稳定的基础咨询输出。
fn default_lookback_days() -> usize {
    DEFAULT_LOOKBACK_DAYS
}

// 2026-03-28 CST：这里集中构造指标快照，原因是咨询结论和指标计算需要分层，便于后续扩展更多指标；
// 目的：先让“指标怎么算”和“结论怎么说”各自独立，避免逻辑互相缠绕。
// 2026-03-29 CST：这里把 OBV 与量能均值并入快照，原因是本轮量价确认仍然要复用同一份历史窗口；
// 目的：继续保持所有原始指标都在同一处产出，避免后续分类逻辑里重复扫描 rows。
fn build_indicator_snapshot(
    rows: &[StockHistoryRow],
) -> Result<TechnicalIndicatorSnapshot, TechnicalConsultationBasicError> {
    let closes = rows.iter().map(|row| row.close).collect::<Vec<_>>();
    let volumes = rows.iter().map(|row| row.volume as f64).collect::<Vec<_>>();

    let close = *closes.last().ok_or_else(|| {
        TechnicalConsultationBasicError::IndicatorCalculation("缺少收盘价".to_string())
    })?;
    let ema_10 = ema_last(&closes, 10)?;
    let sma_50 = sma_last(&closes, 50)?;
    let sma_200 = sma_last(&closes, 200)?;
    let (adx_14, plus_di_14, minus_di_14) = adx_snapshot(rows, 14)?;
    let obv = obv_last(rows)?;
    let volume_sma_20 = sma_last(&volumes, 20)?;
    let volume_ratio_20 = if volume_sma_20.abs() <= f64::EPSILON {
        0.0
    } else {
        rows.last()
            .map(|row| row.volume as f64 / volume_sma_20)
            .unwrap_or(0.0)
    };
    let (macd, macd_signal, macd_histogram) = macd_snapshot(&closes)?;
    let rsi_14 = rsi_last(&closes, 14)?;
    // 2026-03-29 09:45 CST: 这里新增 MFI(14) 快照计算，原因是本轮已确认继续补技术面的资金流维度；
    // 目的：保持 OHLCV -> SQLite -> Rust 指标主线不变，让资金流判断也复用同一份历史窗口。
    let mfi_14 = mfi_last(rows, 14)?;
    // 2026-03-30 09:35 CST: 这里新增 CCI(20) 快照计算，原因是方案 A 这轮继续补均值回归维度；
    // 目的：继续复用同一份 OHLC 历史窗口输出中级指标，避免引入第二套实现或新的运行时依赖。
    let cci_20 = cci_last(rows, 20)?;
    // 2026-03-30 10:45 CST: 这里新增 Williams %R(14) 快照计算，原因是用户已批准先补区间位置这一层能力；
    // 目的：继续复用同一份 OHLC 历史窗口输出中级指标，避免引入额外实现分叉。
    let williams_r_14 = williams_r_last(rows, 14)?;
    let (k_9, d_9, j_9) = kdj_snapshot(rows, 9)?;
    // 2026-03-29 22:35 CST: 这里新增 RSRS beta/zscore 快照，原因是用户已确认 RSRS 要直接接进咨询输出；
    // 目的：继续沿同一份 OHLC 历史窗口产出中级指标，不新开额外实现或第二条数据链。
    let (rsrs_beta_18, rsrs_zscore_18_60) = rsrs_snapshot(rows, 18, 60)?;
    let (boll_upper, boll_middle, boll_lower) = bollinger_last(&closes, 20, 2.0)?;
    // 2026-03-29 23:25 CST: 这里新增布林带带宽快照计算，原因是这轮要基于现有上下轨/中轨快照补一层结构化带宽语义；
    // 目的：继续复用同一份 BOLL 结果，不重开第二套公式链路，同时给 summary / actions / watch_points 提供稳定阈值输入。
    let boll_width_ratio_20 = if boll_middle.abs() <= f64::EPSILON {
        0.0
    } else {
        (boll_upper - boll_lower) / boll_middle.abs()
    };
    let atr_14 = atr_last(rows, 14)?;

    Ok(TechnicalIndicatorSnapshot {
        close,
        ema_10,
        sma_50,
        sma_200,
        adx_14,
        plus_di_14,
        minus_di_14,
        obv,
        volume_sma_20,
        volume_ratio_20,
        mfi_14,
        cci_20,
        williams_r_14,
        boll_width_ratio_20,
        macd,
        macd_signal,
        macd_histogram,
        rsi_14,
        k_9,
        d_9,
        j_9,
        rsrs_beta_18,
        rsrs_zscore_18_60,
        boll_upper,
        boll_middle,
        boll_lower,
        atr_14,
    })
}

// 2026-03-28 CST：这里集中生成业务结论，原因是趋势 / 动量 / 波动几类判断都基于同一份指标快照；
// 目的：把第一版咨询输出收口成稳定合同，保证后续 Skill 可以直接消费。
// 2026-03-29 CST：这里新增趋势强度，原因是 ADX 上线后需要把“方向”和“强弱”分开表达；
// 目的：弱趋势时自动降级为 sideways，避免 AI 把短期均线排列误判成成熟趋势。
// 2026-03-29 CST：这里再接入量能确认，原因是价格方向和趋势强度之外，还需要知道量价是否共振；
// 目的：让后续输出能明确区分“放量确认”“缩量走强”“量能中性”。
// 2026-03-29 CST：这里继续接入第一版背离识别，原因是技术面下一步要能表达“表面走强但量能未确认”的风险；
// 目的：把价格与 OBV 的明显背离收口成稳定字段，便于后续 Skill / AI 直接消费。
fn build_consultation_result(
    request: &TechnicalConsultationBasicRequest,
    rows: &[StockHistoryRow],
    indicator_snapshot: TechnicalIndicatorSnapshot,
) -> TechnicalConsultationBasicResult {
    let trend_strength = classify_trend_strength(&indicator_snapshot).to_string();
    let trend_bias = classify_trend_bias(&indicator_snapshot).to_string();
    let volume_confirmation =
        classify_volume_confirmation(&trend_bias, &trend_strength, &indicator_snapshot).to_string();
    // 2026-03-29 09:45 CST: 这里把资金流信号接入结果构造，原因是 MFI 第一版要和趋势/量能一样进入稳定对外合同；
    // 目的：让后续 summary / actions / watch_points 都能直接复用结构化 `money_flow_signal`，而不是临时读数值拼文案。
    let money_flow_signal = classify_money_flow_signal(&indicator_snapshot).to_string();
    // 2026-03-30 09:35 CST: 这里把 CCI 均值回归信号接入结果构造，原因是 CCI 第一版不能只停留在数值快照层；
    // 目的：让 summary / actions / watch_points 与对外 JSON 合同都直接复用 `mean_reversion_signal`。
    let mean_reversion_signal = classify_mean_reversion_signal(&indicator_snapshot).to_string();
    // 2026-03-30 10:45 CST: 这里把 Williams %R 区间位置信号接入结果构造，原因是第一版不能只停留在快照数值层；
    // 目的：让 summary / actions / watch_points 与对外 JSON 合同都直接复用 `range_position_signal`。
    let range_position_signal = classify_range_position_signal(&indicator_snapshot).to_string();
    // 2026-03-29 23:25 CST: 这里把布林带位置与带宽信号接入结果构造，原因是布林带第一版要沿现有包装链最小增量进入正式咨询输出；
    // 目的：让摘要、动作建议、观察点与对外 JSON 合同都直接复用结构化布林带语义，而不是临时读快照拼文本。
    let bollinger_position_signal =
        classify_bollinger_position_signal(&indicator_snapshot).to_string();
    // 2026-03-29 10:35 CST: 这里新增布林带中轨分类接线，原因是方案 A 需要在现有布林带第一版之上补齐中间层位置语义；
    // 目的：让 summary / actions / watch_points 与对外 JSON 合同都能复用同一份中轨支撑/压制判断。
    let bollinger_midline_signal =
        classify_bollinger_midline_signal(&indicator_snapshot).to_string();
    let bollinger_bandwidth_signal =
        classify_bollinger_bandwidth_signal(&indicator_snapshot).to_string();
    let divergence_signal = classify_divergence_signal(rows).to_string();
    let timing_signal = classify_timing_signal(&indicator_snapshot).to_string();
    // 2026-03-29 22:35 CST: 这里把 RSRS 信号分类接入结果构造，原因是这轮已经确定要一起进入咨询输出；
    // 目的：把斜率强化/走弱沉淀为稳定字段，让外部不必再次读取快照自行解释。
    let rsrs_signal = classify_rsrs_signal(&indicator_snapshot).to_string();
    let momentum_signal = classify_momentum_signal(&indicator_snapshot).to_string();
    let volatility_state = classify_volatility_state(&indicator_snapshot).to_string();
    let summary = build_summary_with_timing_and_rsrs_and_money_flow_and_mean_reversion_and_range_position_and_bollinger(
        &trend_bias,
        &trend_strength,
        &volume_confirmation,
        &money_flow_signal,
        &mean_reversion_signal,
        &range_position_signal,
        &bollinger_position_signal,
        &bollinger_midline_signal,
        &bollinger_bandwidth_signal,
        &divergence_signal,
        &timing_signal,
        &rsrs_signal,
        &momentum_signal,
        &volatility_state,
    );
    let recommended_actions =
        build_recommended_actions_with_timing_and_rsrs_and_money_flow_and_mean_reversion_and_range_position_and_bollinger(
        &trend_bias,
        &trend_strength,
        &volume_confirmation,
        &money_flow_signal,
        &mean_reversion_signal,
            &range_position_signal,
            &bollinger_position_signal,
            &bollinger_midline_signal,
            &bollinger_bandwidth_signal,
            &divergence_signal,
        &timing_signal,
        &rsrs_signal,
        &momentum_signal,
        &volatility_state,
    );
    let watch_points = build_watch_points_with_timing_and_rsrs_and_money_flow_and_mean_reversion_and_range_position_and_bollinger(
        &trend_bias,
        &trend_strength,
        &volume_confirmation,
        &money_flow_signal,
        &mean_reversion_signal,
        &range_position_signal,
        &bollinger_position_signal,
        &bollinger_midline_signal,
        &bollinger_bandwidth_signal,
        &divergence_signal,
        &timing_signal,
        // 2026-03-30 CST: 这里把 RSRS 参数继续放在快照参数之前，原因是当前观察点构造函数签名已经固定为“分类信号 -> 快照 -> 波动状态”。
        // 目的：修复最近一轮字段扩展后调用点没有同步更新导致的编译阻塞，保证授权页相关测试可以继续执行。
        &rsrs_signal,
        &indicator_snapshot,
        &volatility_state,
    );
    let start_date = rows
        .first()
        .map(|row| row.trade_date.clone())
        .unwrap_or_default();
    let end_date = rows
        .last()
        .map(|row| row.trade_date.clone())
        .unwrap_or_default();

    return TechnicalConsultationBasicResult {
        symbol: request.symbol.clone(),
        as_of_date: end_date.clone(),
        history_row_count: rows.len(),
        trend_bias,
        trend_strength,
        volume_confirmation,
        money_flow_signal,
        mean_reversion_signal,
        range_position_signal,
        bollinger_position_signal,
        bollinger_midline_signal,
        bollinger_bandwidth_signal,
        divergence_signal,
        timing_signal,
        rsrs_signal,
        momentum_signal,
        volatility_state,
        summary,
        recommended_actions,
        watch_points,
        indicator_snapshot,
        data_window_summary: DataWindowSummary {
            requested_lookback_days: request.lookback_days,
            loaded_row_count: rows.len(),
            start_date,
            end_date,
        },
    };
}

// 2026-03-28 CST：这里用均线结构判断趋势方向，原因是第一版只需要可解释、可测试的基础规则；
// 目的：先把 bullish / bearish / sideways 的口径稳定下来，再逐步加更高级的规则。
// 2026-03-29 CST：这里接入 ADX 弱趋势保护，原因是均线偶尔排成多空结构不代表趋势已经成立；
// 目的：当 ADX 仍然偏弱时，把方向统一降级为 sideways，减少误导性结论。
fn classify_trend_bias(snapshot: &TechnicalIndicatorSnapshot) -> &'static str {
    if classify_trend_strength(snapshot) == "weak" {
        return "sideways";
    }

    if snapshot.close > snapshot.ema_10
        && snapshot.ema_10 > snapshot.sma_50
        && snapshot.sma_50 > snapshot.sma_200
        && snapshot.plus_di_14 >= snapshot.minus_di_14
    {
        "bullish"
    } else if snapshot.close < snapshot.ema_10
        && snapshot.ema_10 < snapshot.sma_50
        && snapshot.sma_50 < snapshot.sma_200
        && snapshot.minus_di_14 >= snapshot.plus_di_14
    {
        "bearish"
    } else {
        "sideways"
    }
}

// 2026-03-29 CST：这里新增趋势强度判定，原因是方案 A 的核心不是再加一个数字，而是把趋势强弱结构化；
// 目的：让调用方可以直接使用 strong / moderate / weak，而不必自己再解释 ADX 阈值。
fn classify_trend_strength(snapshot: &TechnicalIndicatorSnapshot) -> &'static str {
    if snapshot.adx_14 >= 25.0 {
        "strong"
    } else if snapshot.adx_14 < 20.0 {
        "weak"
    } else {
        "moderate"
    }
}

// 2026-03-29 CST：这里新增量能确认判断，原因是价格方向与 ADX 强弱仍不足以表达量价是否共振；
// 目的：把量能配合拆成 `confirmed / weakening / neutral`，供摘要和建议直接消费。
fn classify_volume_confirmation(
    trend_bias: &str,
    trend_strength: &str,
    snapshot: &TechnicalIndicatorSnapshot,
) -> &'static str {
    let obv_direction_positive = snapshot.obv >= 0.0;
    let aligned_with_trend = match trend_bias {
        "bullish" => obv_direction_positive,
        "bearish" => !obv_direction_positive,
        _ => false,
    };

    if trend_bias != "sideways"
        && trend_strength != "weak"
        && snapshot.volume_ratio_20 >= 1.0
        && aligned_with_trend
    {
        "confirmed"
    } else if snapshot.volume_ratio_20 < 0.95 {
        "weakening"
    } else {
        "neutral"
    }
}

// 2026-03-29 CST：这里新增第一版背离判断，原因是量价确认之后最自然的下一步是识别价格与 OBV 是否已经开始脱节；
// 目的：先用近 20 日高低点与 OBV 配合关系，稳定输出 `bearish_divergence / bullish_divergence / none`。
fn classify_divergence_signal(rows: &[StockHistoryRow]) -> &'static str {
    const RECENT_WINDOW: usize = 10;
    const BASELINE_WINDOW: usize = 20;

    if rows.len() < RECENT_WINDOW + BASELINE_WINDOW {
        return "none";
    }

    let closes = rows.iter().map(|row| row.close).collect::<Vec<_>>();
    let obv_values = obv_series(rows);
    let latest_price_window = &closes[closes.len() - BASELINE_WINDOW..];
    let latest_obv_window = &obv_values[obv_values.len() - BASELINE_WINDOW..];
    let current_close = *latest_price_window.last().unwrap_or(&0.0);
    let current_obv = *latest_obv_window.last().unwrap_or(&0.0);
    let previous_19_price_window = &latest_price_window[..latest_price_window.len() - 1];
    let previous_19_obv_window = &latest_obv_window[..latest_obv_window.len() - 1];

    // 2026-03-29 16:55 CST: 这里把背离比较改成“最近 10 日”对“前 20 日”，原因是测试样本的最后一天可能只是新高后的回踩；
    // 目的：只要近期价格高点继续上移而 OBV 高点没有同步上移，就稳定识别第一层顶背离，不把最后一根是否收在最高点当成必要条件。
    let recent_price_window = &closes[closes.len() - RECENT_WINDOW..];
    let previous_price_window =
        &closes[closes.len() - (RECENT_WINDOW + BASELINE_WINDOW)..closes.len() - RECENT_WINDOW];
    let recent_obv_window = &obv_values[obv_values.len() - RECENT_WINDOW..];
    let previous_obv_window = &obv_values
        [obv_values.len() - (RECENT_WINDOW + BASELINE_WINDOW)..obv_values.len() - RECENT_WINDOW];

    let recent_price_high = recent_price_window
        .iter()
        .fold(f64::MIN, |current, value| current.max(*value));
    let previous_price_high = previous_price_window
        .iter()
        .fold(f64::MIN, |current, value| current.max(*value));
    let recent_price_low = recent_price_window
        .iter()
        .fold(f64::MAX, |current, value| current.min(*value));
    let previous_price_low = previous_price_window
        .iter()
        .fold(f64::MAX, |current, value| current.min(*value));
    let recent_obv_high = recent_obv_window
        .iter()
        .fold(f64::MIN, |current, value| current.max(*value));
    let previous_obv_high = previous_obv_window
        .iter()
        .fold(f64::MIN, |current, value| current.max(*value));
    let recent_obv_low = recent_obv_window
        .iter()
        .fold(f64::MAX, |current, value| current.min(*value));
    let previous_obv_low = previous_obv_window
        .iter()
        .fold(f64::MAX, |current, value| current.min(*value));
    let previous_19_price_high = previous_19_price_window
        .iter()
        .fold(f64::MIN, |current, value| current.max(*value));
    let previous_19_price_low = previous_19_price_window
        .iter()
        .fold(f64::MAX, |current, value| current.min(*value));
    let previous_19_obv_high = previous_19_obv_window
        .iter()
        .fold(f64::MIN, |current, value| current.max(*value));
    let previous_19_obv_low = previous_19_obv_window
        .iter()
        .fold(f64::MAX, |current, value| current.min(*value));

    // 2026-03-29 18:45 CST：这里追加“当前点优先”的背离判断，原因是本轮 bullish_divergence 红测说明
    // 仅比较“最近一段窗口高低点”会漏掉当前价格刚创新低、但 OBV 已经先行修复的底背离；
    // 目的：先用最小改动把顶部与底部背离都补齐到同一份稳定合同里，同时保留既有近期窗口回看能力。
    if current_close > previous_19_price_high && current_obv < previous_19_obv_high {
        "bearish_divergence"
    } else if current_close < previous_19_price_low && current_obv > previous_19_obv_low {
        "bullish_divergence"
    } else if recent_price_high > previous_price_high && recent_obv_high < previous_obv_high {
        "bearish_divergence"
    } else if recent_price_low < previous_price_low && recent_obv_low > previous_obv_low {
        "bullish_divergence"
    } else {
        "none"
    }
}

// 2026-03-28 CST：这里用 MACD 和 RSI 组合判断动量，原因是单看一个指标容易误判；
// 目的：给第一版咨询提供简单但常见的动量语义标签。
fn classify_momentum_signal(snapshot: &TechnicalIndicatorSnapshot) -> &'static str {
    if snapshot.macd > snapshot.macd_signal && snapshot.rsi_14 >= 55.0 {
        "positive"
    } else if snapshot.macd < snapshot.macd_signal && snapshot.rsi_14 <= 45.0 {
        "negative"
    } else {
        "neutral"
    }
}

// 2026-03-28 CST：这里用 ATR、布林带宽度和价格越带情况判断波动，原因是第一版要稳定识别高波动；
// 目的：在不引入更复杂波动模型的前提下，先给结论补上风险温度标签。
fn classify_timing_signal(snapshot: &TechnicalIndicatorSnapshot) -> &'static str {
    // 2026-03-29 22:00 CST: 这里把第一版择时规则收敛为“KDJ 交叉 + RSI/MACD 确认”，原因是单看 K/D/J 绝对值会把单边强趋势误报成回落；
    // 目的：保持 KDJ 仍是核心触发条件，同时用现有 RSI/MACD 快照过滤掉明显不符合语义的假阳性。
    if snapshot.rsi_14 <= 30.0
        && snapshot.k_9 > snapshot.d_9
        && snapshot.j_9 > snapshot.k_9
        && snapshot.macd_histogram > 0.0
    {
        "oversold_rebound"
    } else if snapshot.rsi_14 >= 70.0
        && snapshot.k_9 < snapshot.d_9
        && snapshot.j_9 < snapshot.k_9
        && snapshot.macd_histogram < 0.0
    {
        "overbought_pullback"
    } else {
        "neutral"
    }
}

// 2026-03-29 22:35 CST: 这里新增 RSRS 第一版信号分类，原因是 RSRS 不能只停留在两个数值快照上；
// 目的：先用 beta + zscore 的可解释规则把“斜率强化 / 压力转强 / 中性”稳定收口，后续再按样本继续细化。
fn classify_rsrs_signal(snapshot: &TechnicalIndicatorSnapshot) -> &'static str {
    if snapshot.rsrs_zscore_18_60 >= 0.7 && snapshot.rsrs_beta_18 >= 1.0 {
        "bullish_breakout"
    } else if snapshot.rsrs_zscore_18_60 <= -0.7 && snapshot.rsrs_beta_18 <= 1.0 {
        "bearish_pressure"
    } else {
        "neutral"
    }
}

// 2026-03-29 09:45 CST: 这里新增 MFI 第一版信号分类，原因是资金流能力不能只停留在一个快照数值上；
// 目的：先把资金过热、资金过冷与中性状态收敛成稳定枚举，后续再继续扩展更细的资金流结构。
fn classify_money_flow_signal(snapshot: &TechnicalIndicatorSnapshot) -> &'static str {
    if snapshot.mfi_14 >= 80.0 {
        "overbought_distribution"
    } else if snapshot.mfi_14 <= 20.0 {
        "oversold_accumulation"
    } else {
        "neutral"
    }
}

// 2026-03-30 09:35 CST: 这里新增 CCI 第一版均值回归信号分类，原因是均值回归能力不能只暴露一个快照数值；
// 目的：先把价格偏离均值的上沿风险、下沿修复候选与中性状态收敛成稳定枚举，便于上层直接消费。
fn classify_mean_reversion_signal(snapshot: &TechnicalIndicatorSnapshot) -> &'static str {
    if snapshot.cci_20 >= 100.0 {
        "overbought_reversal_risk"
    } else if snapshot.cci_20 <= -100.0 {
        "oversold_rebound_candidate"
    } else {
        "neutral"
    }
}

// 2026-03-30 10:45 CST: 这里新增 Williams %R 区间位置分类，原因是区间位置能力第一版需要稳定输出结构化语义；
// 目的：把 `>= -20 / <= -80 / 其余` 固化成对外合同，避免上层重复翻译阈值。
fn classify_range_position_signal(snapshot: &TechnicalIndicatorSnapshot) -> &'static str {
    if snapshot.williams_r_14 >= -20.0 {
        "overbought_pullback_risk"
    } else if snapshot.williams_r_14 <= -80.0 {
        "oversold_rebound_candidate"
    } else {
        "neutral"
    }
}

// 2026-03-29 23:25 CST: 这里新增布林带位置分类，原因是用户已经批准把布林带第一版正式并入基础技术咨询；
// 目的：把 `close` 相对上下轨的位置固化成稳定合同，避免上层继续重复解析 `boll_upper / boll_lower`。
fn classify_bollinger_position_signal(snapshot: &TechnicalIndicatorSnapshot) -> &'static str {
    if snapshot.close >= snapshot.boll_upper {
        "upper_band_breakout_risk"
    } else if snapshot.close <= snapshot.boll_lower {
        "lower_band_rebound_candidate"
    } else {
        "neutral"
    }
}

// 2026-03-29 10:35 CST: 这里新增布林带中轨分类，原因是上下轨极端语义之外还需要表达价格相对中轨的偏多/偏空位置；
// 目的：把 `close` 相对 `boll_middle` 的中间层语义固化为稳定合同，同时避免与上下轨极端分类重复。
fn classify_bollinger_midline_signal(snapshot: &TechnicalIndicatorSnapshot) -> &'static str {
    if classify_bollinger_position_signal(snapshot) != "neutral" {
        "neutral"
    } else if snapshot.close > snapshot.boll_middle {
        "midline_support_bias"
    } else if snapshot.close < snapshot.boll_middle {
        "midline_resistance_bias"
    } else {
        "neutral"
    }
}

// 2026-03-29 23:25 CST: 这里新增布林带带宽分类，原因是布林带家族除了上下轨位置，还需要表达波动收敛与扩张状态；
// 目的：把 `boll_width_ratio_20` 统一翻译成 `expanding / contracting / normal`，供摘要、建议和观察点直接复用。
fn classify_bollinger_bandwidth_signal(snapshot: &TechnicalIndicatorSnapshot) -> &'static str {
    if snapshot.boll_width_ratio_20 >= 0.12 {
        "expanding"
    } else if snapshot.boll_width_ratio_20 <= 0.05 {
        "contracting"
    } else {
        "normal"
    }
}

fn classify_volatility_state(snapshot: &TechnicalIndicatorSnapshot) -> &'static str {
    let atr_ratio = if snapshot.close.abs() > f64::EPSILON {
        snapshot.atr_14 / snapshot.close.abs()
    } else {
        0.0
    };
    let boll_width_ratio = snapshot.boll_width_ratio_20;
    let out_of_band =
        snapshot.close >= snapshot.boll_upper || snapshot.close <= snapshot.boll_lower;

    if atr_ratio >= 0.03 || boll_width_ratio >= 0.18 || out_of_band {
        "high"
    } else {
        "normal"
    }
}

// 2026-03-28 CST：这里统一生成摘要文本，原因是上层 AI / Skill 需要先拿到一句可直接展示的中文结论；
// 目的：减少后续消费方重复拼装趋势、动量、波动说明。
// 2026-03-29 CST：这里把趋势强度接进摘要，原因是 ADX 上线后不能只说方向不说强弱；
// 目的：让最终中文摘要直接体现“偏多但强度弱”与“偏多且强度强”的差别。
// 2026-03-29 CST：这里再接入量能确认文案，原因是量价共振是当前下一步最重要的补口；
// 目的：让摘要一句话就能说明当前趋势是否已经得到成交量支持。
// 2026-03-29 CST：这里继续接入背离文案，原因是价格与 OBV 脱节时应该在最终摘要里明确提示；
// 目的：让调用方即使只展示一行摘要，也不会漏掉背离风险。
fn build_summary(
    trend_bias: &str,
    trend_strength: &str,
    volume_confirmation: &str,
    divergence_signal: &str,
    momentum_signal: &str,
    volatility_state: &str,
) -> String {
    let trend_text = match trend_bias {
        "bullish" => "整体偏多",
        "bearish" => "整体偏空",
        _ => "整体偏震荡",
    };
    let strength_text = match trend_strength {
        "strong" => "趋势强度较强",
        "moderate" => "趋势强度中等",
        _ => "趋势强度偏弱",
    };
    let volume_text = match volume_confirmation {
        "confirmed" => "量能配合较好",
        "weakening" => "量能确认不足",
        _ => "量能表现中性",
    };
    let divergence_text = match divergence_signal {
        "bearish_divergence" => "存在顶背离迹象",
        "bullish_divergence" => "存在底背离迹象",
        _ => "未见明显背离",
    };
    let momentum_text = match momentum_signal {
        "positive" => "动量偏强",
        "negative" => "动量偏弱",
        _ => "动量中性",
    };
    let volatility_text = match volatility_state {
        "high" => "波动偏高",
        _ => "波动处于常态",
    };

    format!(
        "{}，{}，{}，{}，{}，{}。",
        trend_text, strength_text, volume_text, divergence_text, momentum_text, volatility_text
    )
}

// 2026-03-28 CST：这里按状态生成建议动作，原因是第一版咨询不只要给指标，还要给下一步关注方向；
// 目的：先沉淀成结构化字符串数组，后续 Skill 可以直接重排或翻译成更自然的话术。
// 2026-03-29 CST：这里把趋势强度接入建议，原因是弱趋势场景不应该继续复用强趋势追随文案；
// 目的：把 ADX 强弱转成直接可执行的观察、等待和纪律建议。
// 2026-03-29 CST：这里再接入量能确认分支，原因是放量与缩量下的执行建议应当明确区分；
// 目的：让“顺势跟随”与“等待量能补上”有不同的结构化输出。
// 2026-03-29 CST：这里继续接入背离分支，原因是背离出现时执行建议应优先强调确认与防守；
// 目的：把“趋势仍在”与“趋势虽在但内部已背离”的建议分开。
fn build_recommended_actions(
    trend_bias: &str,
    trend_strength: &str,
    volume_confirmation: &str,
    divergence_signal: &str,
    momentum_signal: &str,
    volatility_state: &str,
) -> Vec<String> {
    let mut actions = Vec::new();

    match trend_bias {
        "bullish" => actions.push("优先按顺势思路观察回踩 10 日 EMA 后的承接力度".to_string()),
        "bearish" => actions.push("优先控制逆势博弈仓位，等待均线结构修复后再评估".to_string()),
        _ => actions.push("优先等待方向明确，再考虑顺势跟随".to_string()),
    }

    match trend_strength {
        "strong" => actions
            .push("ADX 显示趋势强度已经成型，可优先关注顺势延续而不是频繁抄底摸顶".to_string()),
        "moderate" => {
            actions.push("ADX 仍在中性区间，建议继续等待 DI 方向和价格结构共振确认".to_string())
        }
        _ => actions
            .push("ADX 仍在弱势区间，建议以等待方向确认为主，避免把震荡误判成趋势".to_string()),
    }

    match volume_confirmation {
        "confirmed" => actions
            .push("量能已与当前方向形成配合，可优先关注趋势延续而不是单纯猜测反转".to_string()),
        "weakening" => actions.push(
            "当前价格方向尚未得到量能充分确认，建议先观察放量再决定是否继续顺势参与".to_string(),
        ),
        _ => {
            actions.push("量能暂未给出明显增量信号，建议把成交量变化与价格突破一起跟踪".to_string())
        }
    }

    match divergence_signal {
        "bearish_divergence" => actions.push("价格与 OBV 已出现顶背离迹象，建议优先确认强势是否还能继续，不要把新高直接当成无条件加仓信号".to_string()),
        "bullish_divergence" => actions.push("价格与 OBV 已出现底背离迹象，建议结合止跌结构观察是否正在形成更稳的反转基础".to_string()),
        _ => {}
    }

    match momentum_signal {
        "positive" => actions.push("保留对强势延续的跟踪，但避免在短线过热位置追高".to_string()),
        "negative" => actions.push("优先观察动量是否继续走弱，避免过早抢反弹".to_string()),
        _ => actions.push("结合后续放量与均线突破情况确认下一步方向".to_string()),
    }

    if volatility_state == "high" {
        actions.push("提高波动容忍和止损纪律，避免在剧烈摆动中频繁反向操作".to_string());
    }

    actions
}

// 2026-03-28 CST：这里单独生成观察点，原因是推荐动作更偏“怎么做”，而观察点更偏“看什么”；
// 目的：让后续 Skill 在展示时可以区分执行建议和监控提示。
// 2026-03-29 CST：这里新增 ADX 与 DI 观察点，原因是趋势强度上线后需要把强度变化纳入监控；
// 目的：让后续 AI 直接知道要盯住 ADX 升降和 +DI/-DI 交叉，而不是只看均线位置。
// 2026-03-29 CST：这里再补量能观察点，原因是量价确认上线后需要提示是否继续放量或继续缩量；
// 目的：把“价格是否延续”和“量能是否跟随”拆成两个可独立追踪的监控维度。
// 2026-03-29 CST：这里继续补背离观察点，原因是背离上线后需要显式提示“价格高低点”和“OBV 高低点”是否重新同步；
// 目的：让下游直接知道背离解除或强化该看什么。
fn build_watch_points(
    trend_bias: &str,
    trend_strength: &str,
    volume_confirmation: &str,
    divergence_signal: &str,
    snapshot: &TechnicalIndicatorSnapshot,
    volatility_state: &str,
) -> Vec<String> {
    let mut watch_points = Vec::new();

    match trend_bias {
        "bullish" => {
            watch_points.push("留意收盘价是否继续站稳 10 日 EMA 之上".to_string());
            watch_points.push("留意 50 日均线与 200 日均线的多头排列是否保持".to_string());
        }
        "bearish" => {
            watch_points.push("留意反弹是否仍被 10 日 EMA 压制".to_string());
            watch_points.push("留意 50 日均线与 200 日均线的空头排列是否延续".to_string());
        }
        _ => {
            watch_points.push("留意价格是否突破布林带中轨并形成连续方向".to_string());
            watch_points.push("留意价格是否摆脱 50 日均线附近的反复拉锯".to_string());
        }
    }

    match trend_strength {
        "strong" => watch_points
            .push("留意 ADX 是否继续保持在 25 之上，以确认趋势强度没有明显衰减".to_string()),
        "moderate" => {
            watch_points.push("留意 ADX 是否上穿 25，以确认当前方向是否从尝试转向成型".to_string())
        }
        _ => watch_points.push("留意 ADX 是否从 20 以下回升，并等待真正的方向突破".to_string()),
    }

    if snapshot.plus_di_14 >= snapshot.minus_di_14 {
        watch_points.push("留意 +DI 是否继续高于 -DI，以确认多头方向没有被破坏".to_string());
    } else {
        watch_points.push("留意 -DI 是否继续高于 +DI，以确认空头方向是否仍在延续".to_string());
    }

    match volume_confirmation {
        "confirmed" => watch_points
            .push("留意成交量是否继续维持在 20 日均量之上，确认量价共振没有减弱".to_string()),
        "weakening" => watch_points
            .push("留意成交量是否重新回到 20 日均量之上，避免缩量趋势快速失真".to_string()),
        _ => watch_points
            .push("留意量比是否从中性转为放量，判断当前价格突破是否会获得更多资金确认".to_string()),
    }

    match divergence_signal {
        "bearish_divergence" => watch_points
            .push("留意价格是否继续创新高而 OBV 仍未同步抬升，防止顶背离继续扩大".to_string()),
        "bullish_divergence" => watch_points
            .push("留意价格是否继续创新低但 OBV 已不再创新低，确认底背离是否开始兑现".to_string()),
        _ => {}
    }

    if volatility_state == "high" {
        watch_points.push("留意 ATR 是否继续抬升，避免把高波动误判成有效趋势".to_string());
    }

    watch_points
}

// 2026-03-28 CST：这里实现简单均线，原因是 50/200 日均线是第一版趋势判断的基础骨架；
// 目的：优先提供稳定、可验证的均值口径，而不是一次性引入复杂库依赖。
// 2026-03-29 21:35 CST: 这里新增带 KDJ 择时的摘要包装，原因是旧摘要函数已经稳定承载趋势/量能/背离文案；
// 目的：在不重写旧逻辑的前提下，把 timing_signal 以最小增量并入正式对外合同。
fn build_summary_with_timing(
    trend_bias: &str,
    trend_strength: &str,
    volume_confirmation: &str,
    divergence_signal: &str,
    timing_signal: &str,
    momentum_signal: &str,
    volatility_state: &str,
) -> String {
    let base_summary = build_summary(
        trend_bias,
        trend_strength,
        volume_confirmation,
        divergence_signal,
        momentum_signal,
        volatility_state,
    );
    let timing_text = match timing_signal {
        "oversold_rebound" => "鐭嚎鏈夎秴鍗栦慨澶嶈抗璞?",
        "overbought_pullback" => "鐭嚎鏈夐珮浣嶅洖钀借抗璞?",
        _ => "鐭嚎鎷╂椂淇″彿涓€?",
    };

    format!("{base_summary} {timing_text}銆?")
}

// 2026-03-29 22:35 CST: 这里新增带 RSRS 的摘要包装，原因是用户已经批准把 RSRS 一起接进咨询输出；
// 目的：继续沿包装函数路径做最小增量，把 RSRS 文案并到已有 timing 摘要里，而不重写老摘要函数。
fn build_summary_with_timing_and_rsrs(
    trend_bias: &str,
    trend_strength: &str,
    volume_confirmation: &str,
    divergence_signal: &str,
    timing_signal: &str,
    rsrs_signal: &str,
    momentum_signal: &str,
    volatility_state: &str,
) -> String {
    let base_summary = build_summary_with_timing(
        trend_bias,
        trend_strength,
        volume_confirmation,
        divergence_signal,
        timing_signal,
        momentum_signal,
        volatility_state,
    );
    let rsrs_text = match rsrs_signal {
        "bullish_breakout" => "RSRS 显示近期斜率强化，突破延续概率偏高。",
        "bearish_pressure" => "RSRS 显示近期斜率走弱，短线压力正在抬升。",
        // 2026-03-30 00:18 CST: 这里把 neutral 文案改成“未形成共振”，原因是方案 A 这轮要先锁 RSRS 方向不一致边界；
        // 目的：让上层明确知道 neutral 不只是“没有信号”，还可能是 beta 与 zscore 尚未同向共振。
        _ => "RSRS 暂未形成 beta 与 zscore 的同向共振，斜率结构仍待确认。",
    };

    format!("{base_summary} {rsrs_text}")
}

// 2026-03-29 09:45 CST: 这里新增 MFI 摘要包装，原因是资金流能力要与现有 timing / RSRS 一样进入最终一句话摘要；
// 目的：继续沿包装函数方式做最小增量，把 MFI 文案并入现有咨询结果而不重写旧摘要链路。
fn build_summary_with_timing_and_rsrs_and_money_flow(
    trend_bias: &str,
    trend_strength: &str,
    volume_confirmation: &str,
    money_flow_signal: &str,
    divergence_signal: &str,
    timing_signal: &str,
    rsrs_signal: &str,
    momentum_signal: &str,
    volatility_state: &str,
) -> String {
    let base_summary = build_summary_with_timing_and_rsrs(
        trend_bias,
        trend_strength,
        volume_confirmation,
        divergence_signal,
        timing_signal,
        rsrs_signal,
        momentum_signal,
        volatility_state,
    );
    let money_flow_text = match money_flow_signal {
        "overbought_distribution" => "MFI 显示短线资金已进入高位过热区，需提防分配与回吐压力。",
        "oversold_accumulation" => "MFI 显示短线资金已落入低位超卖区，可继续观察是否出现吸筹修复。",
        _ => "MFI 显示资金流状态中性，尚未形成明显过热或过冷。",
    };

    format!("{base_summary} {money_flow_text}")
}

// 2026-03-30 09:35 CST: 这里新增带 CCI 的摘要包装，原因是均值回归信号已确定要沿现有包装链最小接入咨询输出；
// 目的：在不重写旧摘要函数的前提下，把 CCI 文案追加到已有 timing / RSRS / MFI 摘要之后，保持架构稳定。
fn build_summary_with_timing_and_rsrs_and_money_flow_and_mean_reversion(
    trend_bias: &str,
    trend_strength: &str,
    volume_confirmation: &str,
    money_flow_signal: &str,
    mean_reversion_signal: &str,
    divergence_signal: &str,
    timing_signal: &str,
    rsrs_signal: &str,
    momentum_signal: &str,
    volatility_state: &str,
) -> String {
    let base_summary = build_summary_with_timing_and_rsrs_and_money_flow(
        trend_bias,
        trend_strength,
        volume_confirmation,
        money_flow_signal,
        divergence_signal,
        timing_signal,
        rsrs_signal,
        momentum_signal,
        volatility_state,
    );
    let mean_reversion_text = match mean_reversion_signal {
        "overbought_reversal_risk" => {
            "CCI 显示价格已明显偏离均值上沿，短线存在均值回归与高位回落风险。"
        }
        "oversold_rebound_candidate" => {
            "CCI 显示价格已明显偏离均值下沿，可继续观察是否出现均值回归式反抽修复。"
        }
        _ => "CCI 显示价格偏离度仍处中性区间，均值回归信号尚未形成。",
    };

    format!("{base_summary} {mean_reversion_text}")
}

// 2026-03-30 10:45 CST: 这里新增带 Williams %R 的摘要包装，原因是区间位置能力进入咨询层后需要直接写入最终一行摘要；
// 目的：让上层即使只展示 summary，也能知道当前处在近期区间高位、低位还是中性位置。
fn build_summary_with_timing_and_rsrs_and_money_flow_and_mean_reversion_and_range_position(
    trend_bias: &str,
    trend_strength: &str,
    volume_confirmation: &str,
    money_flow_signal: &str,
    mean_reversion_signal: &str,
    range_position_signal: &str,
    divergence_signal: &str,
    timing_signal: &str,
    rsrs_signal: &str,
    momentum_signal: &str,
    volatility_state: &str,
) -> String {
    let base_summary = build_summary_with_timing_and_rsrs_and_money_flow_and_mean_reversion(
        trend_bias,
        trend_strength,
        volume_confirmation,
        money_flow_signal,
        mean_reversion_signal,
        divergence_signal,
        timing_signal,
        rsrs_signal,
        momentum_signal,
        volatility_state,
    );
    let range_position_text = match range_position_signal {
        "overbought_pullback_risk" => {
            "Williams %R 显示收盘已贴近近期区间上沿，短线需防范高位回落与追高承接不足。"
        }
        "oversold_rebound_candidate" => {
            "Williams %R 显示收盘已逼近近期区间下沿，可继续观察是否出现低位反抽修复。"
        }
        _ => "Williams %R 显示当前仍处近期区间中性位置，区间方向信号尚未形成。",
    };

    format!("{base_summary} {range_position_text}")
}

// 2026-03-29 21:35 CST: 这里新增带 KDJ 择时的动作建议包装，原因是 timing_signal 不能只作为枚举输出；
// 目的：让上层直接拿到“超卖修复 / 高位回落 / 中性”的执行建议，而不是自己再翻译 KDJ。
// 2026-03-29 23:25 CST: 这里新增布林带摘要包装，原因是布林带第一版要沿现有 summary 包装链最小增量接入；
// 目的：即使上层只展示一行摘要，也能同时看到布林带位置与带宽的结构化语义。
fn build_summary_with_timing_and_rsrs_and_money_flow_and_mean_reversion_and_range_position_and_bollinger(
    trend_bias: &str,
    trend_strength: &str,
    volume_confirmation: &str,
    money_flow_signal: &str,
    mean_reversion_signal: &str,
    range_position_signal: &str,
    bollinger_position_signal: &str,
    bollinger_midline_signal: &str,
    bollinger_bandwidth_signal: &str,
    divergence_signal: &str,
    timing_signal: &str,
    rsrs_signal: &str,
    momentum_signal: &str,
    volatility_state: &str,
) -> String {
    let base_summary =
        build_summary_with_timing_and_rsrs_and_money_flow_and_mean_reversion_and_range_position(
            trend_bias,
            trend_strength,
            volume_confirmation,
            money_flow_signal,
            mean_reversion_signal,
            range_position_signal,
            divergence_signal,
            timing_signal,
            rsrs_signal,
            momentum_signal,
            volatility_state,
        );
    let bollinger_position_text = match bollinger_position_signal {
        "upper_band_breakout_risk" => {
            "布林带显示收盘已经贴近或站上上轨，短线需要防范冲高后回落与追高承接不足。"
        }
        "lower_band_rebound_candidate" => {
            "布林带显示收盘已经贴近或跌破下轨，可继续观察是否出现下轨附近的反抽修复。"
        }
        _ => "布林带显示价格仍运行在上下轨之间，位置端暂未形成极端突破信号。",
    };
    let bollinger_midline_text = match bollinger_midline_signal {
        "midline_support_bias" => "布林带中轨仍提供支撑，价格暂时保持在中轨上方运行。",
        "midline_resistance_bias" => "布林带中轨暂时形成压制，价格仍在中轨下方运行。",
        _ => "布林带中轨附近暂未形成明确的支撑或压制偏向。",
    };
    let bollinger_bandwidth_text = match bollinger_bandwidth_signal {
        "expanding" => "布林带带宽正在扩张，说明波动正在放大。",
        "contracting" => "布林带带宽正在收敛，说明波动暂时压缩。",
        _ => "布林带带宽维持常态，波动扩张与收敛都不明显。",
    };

    format!(
        "{base_summary} {bollinger_position_text} {bollinger_midline_text} {bollinger_bandwidth_text}"
    )
}

fn build_recommended_actions_with_timing(
    trend_bias: &str,
    trend_strength: &str,
    volume_confirmation: &str,
    divergence_signal: &str,
    timing_signal: &str,
    momentum_signal: &str,
    volatility_state: &str,
) -> Vec<String> {
    let mut actions = build_recommended_actions(
        trend_bias,
        trend_strength,
        volume_confirmation,
        divergence_signal,
        momentum_signal,
        volatility_state,
    );

    let timing_action = match timing_signal {
        "oversold_rebound" => {
            "KDJ 鏄剧ず鐭嚎瓒呭崠鍚庢鍦ㄤ慨澶嶏紝鍙紭鍏堝叧娉ㄥ弽寮圭殑杩炵画鎬у拰鑳藉惁绔欑ǔ杩戞湡鏀拺"
        }
        "overbought_pullback" => {
            "KDJ 鏄剧ず鐭嚎楂樹綅鍥炶惤锛屽缓璁厛绛夊緟鎯呯华鍜屼环鏍肩粨鏋勫啀搴﹀紑濮嬬ǔ瀹氬悗鍐嶈瘎浼?"
        }
        _ => {
            "KDJ 鏆傛湭缁欏嚭鏄庣‘鐭嚎鎷╂椂浼樺娍锛屽缓璁户缁粨鍚堣秼鍔裤€侀噺鑳藉拰鍏抽敭浠蜂綅涓€璧疯窡韪?"
        }
    };
    actions.push(timing_action.to_string());
    actions
}

// 2026-03-29 22:35 CST: 这里新增带 RSRS 的动作建议包装，原因是这轮已经明确 RSRS 不能只停留在快照层；
// 目的：让调用方直接拿到与 RSRS 信号对应的执行建议，而不是自己再翻译 zscore 阈值。
fn build_recommended_actions_with_timing_and_rsrs(
    trend_bias: &str,
    trend_strength: &str,
    volume_confirmation: &str,
    divergence_signal: &str,
    timing_signal: &str,
    rsrs_signal: &str,
    momentum_signal: &str,
    volatility_state: &str,
) -> Vec<String> {
    let mut actions = build_recommended_actions_with_timing(
        trend_bias,
        trend_strength,
        volume_confirmation,
        divergence_signal,
        timing_signal,
        momentum_signal,
        volatility_state,
    );

    let rsrs_action = match rsrs_signal {
        "bullish_breakout" => {
            "RSRS 显示斜率强化，优先观察顺势突破后的回踩承接，而不是在尚未确认前急于逆势博弈"
        }
        "bearish_pressure" => {
            "RSRS 显示斜率走弱，优先确认近期支撑是否失守，并把防守纪律放在追价之前"
        }
        // 2026-03-30 00:18 CST: 这里补 neutral“共振未形成”提示，原因是仅写中性区间不足以解释 mismatch 边界；
        // 目的：让调用方知道下一步该观察的是 beta 与 zscore 是否同向，而不是把 neutral 误读成完全无效信号。
        _ => {
            "RSRS 暂未形成 beta 与 zscore 的同向共振，建议继续结合趋势、量能和关键价位确认是否出现新的斜率强化"
        }
    };
    actions.push(rsrs_action.to_string());
    actions
}

// 2026-03-29 09:45 CST: 这里新增 MFI 动作建议包装，原因是资金流极值不仅是快照数字，还应给出下一步执行提示；
// 目的：让 `money_flow_signal` 直接落到推荐动作，而不是要求上层再把 80/20 阈值重新翻译成中文建议。
fn build_recommended_actions_with_timing_and_rsrs_and_money_flow(
    trend_bias: &str,
    trend_strength: &str,
    volume_confirmation: &str,
    money_flow_signal: &str,
    divergence_signal: &str,
    timing_signal: &str,
    rsrs_signal: &str,
    momentum_signal: &str,
    volatility_state: &str,
) -> Vec<String> {
    let mut actions = build_recommended_actions_with_timing_and_rsrs(
        trend_bias,
        trend_strength,
        volume_confirmation,
        divergence_signal,
        timing_signal,
        rsrs_signal,
        momentum_signal,
        volatility_state,
    );

    let money_flow_action = match money_flow_signal {
        "overbought_distribution" => {
            "MFI 已进入过热区，优先确认高位放量后的承接质量，不要把持续拉升直接等同于无风险追高"
        }
        "oversold_accumulation" => {
            "MFI 已进入超卖区，优先观察止跌与回补量能是否同步出现，再决定是否参与低位修复"
        }
        _ => "MFI 仍处中性区间，建议继续结合趋势、量能与关键价位等待更清晰的资金流方向",
    };
    actions.push(money_flow_action.to_string());
    actions
}

// 2026-03-30 09:35 CST: 这里新增带 CCI 的动作建议包装，原因是均值回归信号进入咨询层后必须直接给出可执行建议；
// 目的：让调用方直接拿到与 `mean_reversion_signal` 对应的操作提示，而不是继续由上层翻译 100 / -100 阈值。
fn build_recommended_actions_with_timing_and_rsrs_and_money_flow_and_mean_reversion(
    trend_bias: &str,
    trend_strength: &str,
    volume_confirmation: &str,
    money_flow_signal: &str,
    mean_reversion_signal: &str,
    divergence_signal: &str,
    timing_signal: &str,
    rsrs_signal: &str,
    momentum_signal: &str,
    volatility_state: &str,
) -> Vec<String> {
    let mut actions = build_recommended_actions_with_timing_and_rsrs_and_money_flow(
        trend_bias,
        trend_strength,
        volume_confirmation,
        money_flow_signal,
        divergence_signal,
        timing_signal,
        rsrs_signal,
        momentum_signal,
        volatility_state,
    );

    let mean_reversion_action = match mean_reversion_signal {
        "overbought_reversal_risk" => {
            "CCI 已进入上沿极值区，优先确认冲高后的承接与回踩质量，不要把短线过度偏离直接等同于可持续追高"
        }
        "oversold_rebound_candidate" => {
            "CCI 已进入下沿极值区，优先观察止跌与反抽是否同步出现，再决定是否参与均值回归修复"
        }
        _ => "CCI 仍处中性区间，建议继续结合趋势、量能与关键价位等待更清晰的均值回归信号",
    };
    actions.push(mean_reversion_action.to_string());
    actions
}

// 2026-03-30 10:45 CST: 这里新增带 Williams %R 的动作建议包装，原因是区间位置能力进入合同后也需要给出可执行建议；
// 目的：让调用方直接拿到与 `range_position_signal` 对应的操作提示，而不是自己再翻译区间阈值。
fn build_recommended_actions_with_timing_and_rsrs_and_money_flow_and_mean_reversion_and_range_position(
    trend_bias: &str,
    trend_strength: &str,
    volume_confirmation: &str,
    money_flow_signal: &str,
    mean_reversion_signal: &str,
    range_position_signal: &str,
    divergence_signal: &str,
    timing_signal: &str,
    rsrs_signal: &str,
    momentum_signal: &str,
    volatility_state: &str,
) -> Vec<String> {
    let mut actions =
        build_recommended_actions_with_timing_and_rsrs_and_money_flow_and_mean_reversion(
            trend_bias,
            trend_strength,
            volume_confirmation,
            money_flow_signal,
            mean_reversion_signal,
            divergence_signal,
            timing_signal,
            rsrs_signal,
            momentum_signal,
            volatility_state,
        );

    let range_position_action = match range_position_signal {
        "overbought_pullback_risk" => {
            "Williams %R 已进入区间上沿，优先确认冲高后是否出现放量滞涨与回踩承接，不要把贴近上沿直接等同于还能继续追高"
        }
        "oversold_rebound_candidate" => {
            "Williams %R 已进入区间下沿，优先观察止跌与反抽是否同步出现，再决定是否参与低位修复"
        }
        _ => "Williams %R 仍处区间中性位置，建议继续结合趋势、量能与关键价位等待更清晰的区间方向信号",
    };
    actions.push(range_position_action.to_string());
    actions
}

// 2026-03-29 21:35 CST: 这里新增带 KDJ 择时的观察点包装，原因是 timing_signal 已经进入结构化输出；
// 目的：明确后续要盯的 K/D/J 变化，避免上层只拿到状态标签却不知道如何跟踪。
// 2026-03-29 23:25 CST: 这里新增布林带动作建议包装，原因是布林带第一版进入合同后还需要给出可执行提示；
// 目的：让调用方直接拿到与 `bollinger_position_signal` 和 `bollinger_bandwidth_signal` 对应的中文建议，而不是自己再翻译阈值。
fn build_recommended_actions_with_timing_and_rsrs_and_money_flow_and_mean_reversion_and_range_position_and_bollinger(
    trend_bias: &str,
    trend_strength: &str,
    volume_confirmation: &str,
    money_flow_signal: &str,
    mean_reversion_signal: &str,
    range_position_signal: &str,
    bollinger_position_signal: &str,
    bollinger_midline_signal: &str,
    bollinger_bandwidth_signal: &str,
    divergence_signal: &str,
    timing_signal: &str,
    rsrs_signal: &str,
    momentum_signal: &str,
    volatility_state: &str,
) -> Vec<String> {
    let mut actions =
        build_recommended_actions_with_timing_and_rsrs_and_money_flow_and_mean_reversion_and_range_position(
            trend_bias,
            trend_strength,
            volume_confirmation,
            money_flow_signal,
            mean_reversion_signal,
            range_position_signal,
            divergence_signal,
            timing_signal,
            rsrs_signal,
            momentum_signal,
            volatility_state,
        );

    let bollinger_position_action = match bollinger_position_signal {
        "upper_band_breakout_risk" => {
            "布林带上轨已经被触及或突破，优先确认放量后能否继续站稳，避免把短线冲顶直接等同于无风险突破。"
        }
        "lower_band_rebound_candidate" => {
            "布林带下轨已经被触及或跌破，优先观察止跌与反抽是否同步出现，再决定是否参与低位修复。"
        }
        _ => "布林带位置仍在中性区间，建议继续等待价格向上下轨给出更清晰的方向确认。",
    };
    let bollinger_midline_action = match bollinger_midline_signal {
        "midline_support_bias" => {
            "价格仍运行在布林带中轨上方，可优先观察中轨回踩后的承接是否延续，再决定是否继续按偏多节奏跟踪。"
        }
        "midline_resistance_bias" => {
            "价格仍运行在布林带中轨下方，优先确认中轨附近反抽是否再次受压，再决定是否继续等待更清晰的转强信号。"
        }
        _ => "价格正在布林带中轨附近反复拉扯，建议继续等待中轨支撑或压制哪一侧先被有效确认。",
    };
    let bollinger_bandwidth_action = match bollinger_bandwidth_signal {
        "expanding" => "布林带带宽正在扩张，执行上要预留更大的波动缓冲，不要用过紧的节奏处理仓位。",
        "contracting" => "布林带带宽正在收敛，优先等待放量扩张后的方向选择，而不是在窄幅区间内频繁追价。",
        _ => "布林带带宽维持常态，可继续结合趋势、量能与关键价位做常规确认。",
    };
    actions.push(bollinger_position_action.to_string());
    actions.push(bollinger_midline_action.to_string());
    actions.push(bollinger_bandwidth_action.to_string());
    actions
}

fn build_watch_points_with_timing(
    trend_bias: &str,
    trend_strength: &str,
    volume_confirmation: &str,
    divergence_signal: &str,
    timing_signal: &str,
    snapshot: &TechnicalIndicatorSnapshot,
    volatility_state: &str,
) -> Vec<String> {
    let mut watch_points = build_watch_points(
        trend_bias,
        trend_strength,
        volume_confirmation,
        divergence_signal,
        snapshot,
        volatility_state,
    );

    let timing_watch_point = match timing_signal {
        "oversold_rebound" => {
            "鐣欐剰 K 绾挎槸鍚︾户缁繚鎸佸湪 D 绾夸箣涓婏紝骞剁‘璁?J 鍊兼槸鍚︾户缁粠浣庝綅淇"
        }
        "overbought_pullback" => {
            "鐣欐剰 K 绾挎槸鍚︾户缁綆浜?D 绾匡紝骞剁‘璁?J 鍊兼槸鍚﹀湪楂樹綅鍖洪棿缁х画鍥炶惤"
        }
        _ => {
            "鐣欐剰 KDJ 鏄惁鍑虹幇鏂扮殑浣庝綅閲戝弶鎴栭珮浣嶆鍙夛紝鍒ゆ柇鐭嚎鎷╂椂鐘舵€佹槸鍚﹀紑濮嬪彉鍖?"
        }
    };
    watch_points.push(timing_watch_point.to_string());
    watch_points
}

// 2026-03-29 22:35 CST: 这里新增带 RSRS 的观察点包装，原因是 RSRS 进咨询层后必须明确后续监控口径；
// 目的：让上层直接知道应该盯 zscore 是否延续强化或快速回落，而不是只有标签没有跟踪线索。
fn build_watch_points_with_timing_and_rsrs(
    trend_bias: &str,
    trend_strength: &str,
    volume_confirmation: &str,
    divergence_signal: &str,
    timing_signal: &str,
    rsrs_signal: &str,
    snapshot: &TechnicalIndicatorSnapshot,
    volatility_state: &str,
) -> Vec<String> {
    let mut watch_points = build_watch_points_with_timing(
        trend_bias,
        trend_strength,
        volume_confirmation,
        divergence_signal,
        timing_signal,
        snapshot,
        volatility_state,
    );

    let rsrs_watch_point = match rsrs_signal {
        "bullish_breakout" => {
            "留意 RSRS zscore 是否继续保持在正向强化区间，确认斜率突破没有快速回落为中性"
        }
        "bearish_pressure" => {
            "留意 RSRS zscore 是否继续处于负向区间，并确认近期压力没有被新的放量突破迅速扭转"
        }
        // 2026-03-30 00:18 CST: 这里把 neutral 观察点改成“关注共振形成”，原因是方案 A 本轮优先补的是 mismatch 边界；
        // 目的：把后续监控口径明确收敛到 beta 与 zscore 是否完成同向共振，而不是泛泛观察中性区间。
        _ => {
            "留意 RSRS 的 beta 与 zscore 是否开始形成同向共振，判断斜率结构是否从当前中性状态转向强化或走弱"
        }
    };
    watch_points.push(rsrs_watch_point.to_string());
    watch_points
}

// 2026-03-29 09:45 CST: 这里新增 MFI 观察点包装，原因是资金流信号进入对外合同后需要给出明确监控口径；
// 目的：把后续应该关注的 MFI 回落、修复与中性突破条件显式写进 watch_points，减少上层二次解释成本。
fn build_watch_points_with_timing_and_rsrs_and_money_flow(
    trend_bias: &str,
    trend_strength: &str,
    volume_confirmation: &str,
    money_flow_signal: &str,
    divergence_signal: &str,
    timing_signal: &str,
    rsrs_signal: &str,
    snapshot: &TechnicalIndicatorSnapshot,
    volatility_state: &str,
) -> Vec<String> {
    let mut watch_points = build_watch_points_with_timing_and_rsrs(
        trend_bias,
        trend_strength,
        volume_confirmation,
        divergence_signal,
        timing_signal,
        rsrs_signal,
        snapshot,
        volatility_state,
    );

    let money_flow_watch_point = match money_flow_signal {
        "overbought_distribution" => {
            "留意 MFI 是否继续停留在 80 上方并伴随价格滞涨，确认高位资金过热是否开始转化为分配压力"
        }
        "oversold_accumulation" => {
            "留意 MFI 是否从 20 下方回升并配合价格止跌，确认低位超卖是否开始转向修复吸筹"
        }
        _ => "留意 MFI 是否从中性区间继续向 80 或 20 两端扩张，判断资金流是否正在形成新的过热或过冷结构",
    };
    watch_points.push(money_flow_watch_point.to_string());
    watch_points
}

// 2026-03-30 09:35 CST: 这里新增带 CCI 的观察点包装，原因是均值回归信号纳入对外合同后需要明确后续监控口径；
// 目的：把 CCI 回落、修复与中性扩张条件显式写进 watch_points，减少上层再次解释成本。
fn build_watch_points_with_timing_and_rsrs_and_money_flow_and_mean_reversion(
    trend_bias: &str,
    trend_strength: &str,
    volume_confirmation: &str,
    money_flow_signal: &str,
    mean_reversion_signal: &str,
    divergence_signal: &str,
    timing_signal: &str,
    rsrs_signal: &str,
    snapshot: &TechnicalIndicatorSnapshot,
    volatility_state: &str,
) -> Vec<String> {
    let mut watch_points = build_watch_points_with_timing_and_rsrs_and_money_flow(
        trend_bias,
        trend_strength,
        volume_confirmation,
        money_flow_signal,
        divergence_signal,
        timing_signal,
        rsrs_signal,
        snapshot,
        volatility_state,
    );

    let mean_reversion_watch_point = match mean_reversion_signal {
        "overbought_reversal_risk" => {
            "留意 CCI 是否仍继续停留在 100 上方并伴随价格滞涨，确认高位偏离是否开始回归均值"
        }
        "oversold_rebound_candidate" => {
            "留意 CCI 是否从 -100 下方回升并配合价格止跌，确认低位偏离是否开始进入修复反抽"
        }
        _ => "留意 CCI 是否从中性区间继续向 100 或 -100 两端扩张，判断均值回归信号是否正在形成",
    };
    watch_points.push(mean_reversion_watch_point.to_string());
    watch_points
}

// 2026-03-30 10:45 CST: 这里新增带 Williams %R 的观察点包装，原因是区间位置能力纳入对外合同后需要明确后续跟踪口径；
// 目的：把近期区间上沿/下沿/中性扩张条件显式写进 watch_points，减少上层二次解释成本。
fn build_watch_points_with_timing_and_rsrs_and_money_flow_and_mean_reversion_and_range_position(
    trend_bias: &str,
    trend_strength: &str,
    volume_confirmation: &str,
    money_flow_signal: &str,
    mean_reversion_signal: &str,
    range_position_signal: &str,
    divergence_signal: &str,
    timing_signal: &str,
    rsrs_signal: &str,
    snapshot: &TechnicalIndicatorSnapshot,
    volatility_state: &str,
) -> Vec<String> {
    let mut watch_points =
        build_watch_points_with_timing_and_rsrs_and_money_flow_and_mean_reversion(
            trend_bias,
            trend_strength,
            volume_confirmation,
            money_flow_signal,
            mean_reversion_signal,
            divergence_signal,
            timing_signal,
            rsrs_signal,
            snapshot,
            volatility_state,
        );

    let range_position_watch_point = match range_position_signal {
        "overbought_pullback_risk" => {
            "留意 Williams %R 是否仍停留在 -20 上方附近并伴随价格冲高乏力，确认近期区间高位是否开始转弱回落"
        }
        "oversold_rebound_candidate" => {
            "留意 Williams %R 是否从 -80 下方回升并配合价格止跌，确认近期区间低位是否开始进入修复反抽"
        }
        _ => "留意 Williams %R 是否继续在中性区间内收敛或向 -20、-80 两端扩张，判断近期区间位置是否正在形成新的方向倾向",
    };
    watch_points.push(range_position_watch_point.to_string());
    watch_points
}

// 2026-03-29 23:25 CST: 这里新增布林带观察点包装，原因是布林带第一版进入正式咨询输出后还需要明确后续监控口径；
// 目的：把布林带位置与带宽的后续观察点显式写入 `watch_points`，减少上层二次解释成本。
fn build_watch_points_with_timing_and_rsrs_and_money_flow_and_mean_reversion_and_range_position_and_bollinger(
    trend_bias: &str,
    trend_strength: &str,
    volume_confirmation: &str,
    money_flow_signal: &str,
    mean_reversion_signal: &str,
    range_position_signal: &str,
    bollinger_position_signal: &str,
    bollinger_midline_signal: &str,
    bollinger_bandwidth_signal: &str,
    divergence_signal: &str,
    timing_signal: &str,
    rsrs_signal: &str,
    snapshot: &TechnicalIndicatorSnapshot,
    volatility_state: &str,
) -> Vec<String> {
    let mut watch_points =
        build_watch_points_with_timing_and_rsrs_and_money_flow_and_mean_reversion_and_range_position(
            trend_bias,
            trend_strength,
            volume_confirmation,
            money_flow_signal,
            mean_reversion_signal,
            range_position_signal,
            divergence_signal,
            timing_signal,
            rsrs_signal,
            snapshot,
            volatility_state,
        );

    let bollinger_position_watch_point = match bollinger_position_signal {
        "upper_band_breakout_risk" => {
            "留意布林带上轨附近能否继续站稳，并确认价格冲高时是否出现承接减弱或快速回落。"
        }
        "lower_band_rebound_candidate" => {
            "留意布林带下轨附近是否出现止跌与反抽同步修复，确认低位是否开始脱离极端弱势。"
        }
        _ => "留意价格是否继续在布林带中轨附近整理，或开始向上下轨一端扩张形成新的位置倾向。",
    };
    let bollinger_midline_watch_point = match bollinger_midline_signal {
        "midline_support_bias" => {
            "留意布林带中轨回踩时能否继续获得承接，确认中轨支撑是否仍然有效。"
        }
        "midline_resistance_bias" => {
            "留意价格反抽布林带中轨时是否再次受压，确认中轨压制是否仍然成立。"
        }
        _ => "留意价格是否继续围绕布林带中轨反复震荡，等待中轨支撑或压制方向先被明确打破。",
    };
    let bollinger_bandwidth_watch_point = match bollinger_bandwidth_signal {
        "expanding" => "留意布林带带宽是否继续扩张，判断当前波动放大是趋势延续还是情绪冲击。",
        "contracting" => "留意布林带带宽是否从收敛转向放大，确认窄幅整理后是否出现新的方向选择。",
        _ => "留意布林带带宽是否偏离常态，观察波动是否开始从平稳转向扩张或进一步收敛。",
    };
    watch_points.push(bollinger_position_watch_point.to_string());
    watch_points.push(bollinger_midline_watch_point.to_string());
    watch_points.push(bollinger_bandwidth_watch_point.to_string());
    watch_points
}

fn sma_last(values: &[f64], period: usize) -> Result<f64, TechnicalConsultationBasicError> {
    if values.len() < period {
        return Err(TechnicalConsultationBasicError::InsufficientHistory {
            required: period,
            actual: values.len(),
        });
    }

    let window = &values[values.len() - period..];
    Ok(window.iter().sum::<f64>() / period as f64)
}

// 2026-03-28 CST：这里实现 EMA，原因是 10 日 EMA 和 MACD 都需要复用同一套平滑逻辑；
// 目的：把第一版指数均线口径固定在本模块里，方便后续继续扩展指标而不分散实现。
fn ema_last(values: &[f64], period: usize) -> Result<f64, TechnicalConsultationBasicError> {
    if values.len() < period {
        return Err(TechnicalConsultationBasicError::InsufficientHistory {
            required: period,
            actual: values.len(),
        });
    }

    let alpha = 2.0 / (period as f64 + 1.0);
    let mut ema = values[0];
    for value in values.iter().skip(1) {
        ema = alpha * value + (1.0 - alpha) * ema;
    }
    Ok(ema)
}

// 2026-03-28 CST：这里输出整条 EMA 序列，原因是 MACD 需要完整的快线 / 慢线轨迹而不是只有最后一个点；
// 目的：继续复用统一平滑公式，避免 MACD 维护第二份实现。
fn ema_series(values: &[f64], period: usize) -> Result<Vec<f64>, TechnicalConsultationBasicError> {
    if values.len() < period {
        return Err(TechnicalConsultationBasicError::InsufficientHistory {
            required: period,
            actual: values.len(),
        });
    }

    let alpha = 2.0 / (period as f64 + 1.0);
    let mut output = Vec::with_capacity(values.len());
    let mut ema = values[0];
    output.push(ema);

    for value in values.iter().skip(1) {
        ema = alpha * value + (1.0 - alpha) * ema;
        output.push(ema);
    }

    Ok(output)
}

// 2026-03-28 CST：这里实现 MACD 快照，原因是第一版动量判断必须同时看到快慢线与柱体；
// 目的：一次计算出 macd / macd_signal / macd_histogram，供咨询层和外部展示层共用。
fn macd_snapshot(values: &[f64]) -> Result<(f64, f64, f64), TechnicalConsultationBasicError> {
    let ema_12 = ema_series(values, 12)?;
    let ema_26 = ema_series(values, 26)?;
    let macd_series = ema_12
        .iter()
        .zip(ema_26.iter())
        .map(|(fast, slow)| fast - slow)
        .collect::<Vec<_>>();
    let signal_series = ema_series(&macd_series, 9)?;

    let macd = *macd_series.last().ok_or_else(|| {
        TechnicalConsultationBasicError::IndicatorCalculation("MACD 序列为空".to_string())
    })?;
    let macd_signal = *signal_series.last().ok_or_else(|| {
        TechnicalConsultationBasicError::IndicatorCalculation("MACD 信号线序列为空".to_string())
    })?;

    Ok((macd, macd_signal, macd - macd_signal))
}

// 2026-03-28 CST：这里实现 RSI，原因是第一版动量判断需要一个和 MACD 互补的超买超卖参考；
// 目的：先采用常见的 Wilder 平滑口径，保证结果可解释且符合常见技术分析习惯。
fn rsi_last(values: &[f64], period: usize) -> Result<f64, TechnicalConsultationBasicError> {
    if values.len() < period + 1 {
        return Err(TechnicalConsultationBasicError::InsufficientHistory {
            required: period + 1,
            actual: values.len(),
        });
    }

    let mut gains = 0.0;
    let mut losses = 0.0;
    for index in 1..=period {
        let diff = values[index] - values[index - 1];
        if diff >= 0.0 {
            gains += diff;
        } else {
            losses += diff.abs();
        }
    }

    let mut average_gain = gains / period as f64;
    let mut average_loss = losses / period as f64;

    for index in (period + 1)..values.len() {
        let diff = values[index] - values[index - 1];
        let gain = diff.max(0.0);
        let loss = (-diff).max(0.0);
        average_gain = ((average_gain * (period as f64 - 1.0)) + gain) / period as f64;
        average_loss = ((average_loss * (period as f64 - 1.0)) + loss) / period as f64;
    }

    if average_loss.abs() <= f64::EPSILON {
        return Ok(100.0);
    }

    let relative_strength = average_gain / average_loss;
    Ok(100.0 - (100.0 / (1.0 + relative_strength)))
}

// 2026-03-29 09:45 CST: 这里新增 MFI(14) 计算，原因是中级技术面下一步已经明确选择资金流维度；
// 目的：继续用本地 Rust 公式在同一份 OHLCV 历史上输出 MFI，避免重新引入脚本运行时或外部指标 API 依赖。
fn mfi_last(
    rows: &[StockHistoryRow],
    period: usize,
) -> Result<f64, TechnicalConsultationBasicError> {
    if rows.len() < period + 1 {
        return Err(TechnicalConsultationBasicError::InsufficientHistory {
            required: period + 1,
            actual: rows.len(),
        });
    }

    let start_index = rows.len() - period - 1;
    let mut positive_money_flow = 0.0;
    let mut negative_money_flow = 0.0;

    for index in (start_index + 1)..rows.len() {
        let previous_typical_price = typical_price(&rows[index - 1]);
        let current_typical_price = typical_price(&rows[index]);
        let raw_money_flow = current_typical_price * rows[index].volume as f64;

        if current_typical_price > previous_typical_price {
            positive_money_flow += raw_money_flow;
        } else if current_typical_price < previous_typical_price {
            negative_money_flow += raw_money_flow;
        }
    }

    if positive_money_flow.abs() <= f64::EPSILON && negative_money_flow.abs() <= f64::EPSILON {
        return Ok(50.0);
    }
    if negative_money_flow.abs() <= f64::EPSILON {
        return Ok(100.0);
    }
    if positive_money_flow.abs() <= f64::EPSILON {
        return Ok(0.0);
    }

    let money_ratio = positive_money_flow / negative_money_flow;
    Ok(100.0 - (100.0 / (1.0 + money_ratio)))
}

// 2026-03-28 CST：这里实现布林带，原因是第一版既要给波动信息，也要给区间参考；
// 目的：输出中轨和上下轨，供咨询层判断高波动与横盘状态。
// 2026-03-30 09:35 CST: 这里新增 CCI(20) 计算，原因是技术面中级指标下一步已确认按均值回归维度推进；
// 目的：继续使用本地 Rust 公式在同一份 OHLC 历史上输出 CCI，避免重新引入脚本运行时或外部技术指标依赖。
fn cci_last(
    rows: &[StockHistoryRow],
    period: usize,
) -> Result<f64, TechnicalConsultationBasicError> {
    if rows.len() < period {
        return Err(TechnicalConsultationBasicError::InsufficientHistory {
            required: period,
            actual: rows.len(),
        });
    }

    let window = &rows[rows.len() - period..];
    let typical_prices = window.iter().map(typical_price).collect::<Vec<_>>();
    let typical_price_sma = typical_prices.iter().sum::<f64>() / period as f64;
    let mean_deviation = typical_prices
        .iter()
        .map(|value| (value - typical_price_sma).abs())
        .sum::<f64>()
        / period as f64;

    if mean_deviation.abs() <= f64::EPSILON {
        return Ok(0.0);
    }

    let latest_typical_price = *typical_prices.last().ok_or_else(|| {
        TechnicalConsultationBasicError::IndicatorCalculation("CCI 典型价格序列为空".to_string())
    })?;
    Ok((latest_typical_price - typical_price_sma) / (0.015 * mean_deviation))
}

// 2026-03-30 10:45 CST: 这里新增 Williams %R(14) 计算，原因是区间位置能力第一版需要使用近期最高/最低价定位收盘所处位置；
// 目的：继续在本地 Rust 公式层完成指标，保持 EXE / Skill 主链不引入额外脚本运行时或外部依赖。
fn williams_r_last(
    rows: &[StockHistoryRow],
    period: usize,
) -> Result<f64, TechnicalConsultationBasicError> {
    if rows.len() < period {
        return Err(TechnicalConsultationBasicError::InsufficientHistory {
            required: period,
            actual: rows.len(),
        });
    }

    let window = &rows[rows.len() - period..];
    let highest_high = window
        .iter()
        .map(|row| row.high)
        .fold(f64::MIN, |current, value| current.max(value));
    let lowest_low = window
        .iter()
        .map(|row| row.low)
        .fold(f64::MAX, |current, value| current.min(value));
    let latest_close = window.last().map(|row| row.close).ok_or_else(|| {
        TechnicalConsultationBasicError::IndicatorCalculation("Williams %R 窗口为空".to_string())
    })?;
    let range = highest_high - lowest_low;

    if range.abs() <= f64::EPSILON {
        return Ok(-50.0);
    }

    Ok(((highest_high - latest_close) / range) * -100.0)
}

fn bollinger_last(
    values: &[f64],
    period: usize,
    multiplier: f64,
) -> Result<(f64, f64, f64), TechnicalConsultationBasicError> {
    if values.len() < period {
        return Err(TechnicalConsultationBasicError::InsufficientHistory {
            required: period,
            actual: values.len(),
        });
    }

    let window = &values[values.len() - period..];
    let middle = window.iter().sum::<f64>() / period as f64;
    let variance = window
        .iter()
        .map(|value| {
            let diff = value - middle;
            diff * diff
        })
        .sum::<f64>()
        / period as f64;
    let deviation = variance.sqrt();

    Ok((
        middle + multiplier * deviation,
        middle,
        middle - multiplier * deviation,
    ))
}

// 2026-03-28 CST：这里实现 ATR，原因是第一版波动判断需要一个能反映真实波动区间的指标；
// 目的：用常见的 Wilder 平滑口径给出 atr_14，便于识别剧烈波动场景。
fn atr_last(
    rows: &[StockHistoryRow],
    period: usize,
) -> Result<f64, TechnicalConsultationBasicError> {
    if rows.len() < period + 1 {
        return Err(TechnicalConsultationBasicError::InsufficientHistory {
            required: period + 1,
            actual: rows.len(),
        });
    }

    let true_ranges = true_ranges(rows);
    let mut atr = true_ranges[..period].iter().sum::<f64>() / period as f64;
    for true_range in true_ranges.iter().skip(period) {
        atr = ((atr * (period as f64 - 1.0)) + true_range) / period as f64;
    }

    Ok(atr)
}

// 2026-03-29 CST：这里实现 OBV，原因是量价确认第一版需要一个和价格方向联动的累计量能指标；
// 目的：先用最经典的 OBV 口径给出量价方向性快照，便于后续继续补更多量价类指标。
fn obv_last(rows: &[StockHistoryRow]) -> Result<f64, TechnicalConsultationBasicError> {
    if rows.len() < 2 {
        return Err(TechnicalConsultationBasicError::InsufficientHistory {
            required: 2,
            actual: rows.len(),
        });
    }

    Ok(*obv_series(rows).last().unwrap_or(&0.0))
}

// 2026-03-29 CST：这里抽出 OBV 序列，原因是背离识别需要的不只是最后一个累计值，还要看最近一段高低点关系；
// 目的：继续把量价类公式集中在本模块里，方便后续继续补背离或量价共振能力。
fn obv_series(rows: &[StockHistoryRow]) -> Vec<f64> {
    let mut series = Vec::with_capacity(rows.len());
    let mut obv = 0.0;
    series.push(obv);

    for index in 1..rows.len() {
        let current_row = &rows[index];
        let previous_row = &rows[index - 1];
        if current_row.close > previous_row.close {
            obv += current_row.volume as f64;
        } else if current_row.close < previous_row.close {
            obv -= current_row.volume as f64;
        }
        series.push(obv);
    }

    series
}

// 2026-03-29 CST：这里实现 ADX / +DI / -DI 快照，原因是方案 A 要补的不是单点数字，而是一组可解释的趋势强度指标；
// 目的：用 Wilder 平滑口径稳定输出强度结果，保证后续增加 RSRS / OBV 时仍沿用同一条 Rust 技术面主线。
// 2026-03-29 21:35 CST: 这里新增 KDJ 快照计算，原因是 technical_consultation_basic 第一版择时能力要稳定走 Rust 主线；
// 目的：直接在同一份 OHLC 历史窗口上输出 K/D/J，供 timing_signal 与上层 Skill 统一复用，避免引入第二套实现。
// 2026-03-29 22:35 CST: 这里新增 RSRS 快照计算，原因是中级指标第一版已经确定按单家族推进到 RSRS；
// 目的：用统一 Rust 口径在同一份历史窗口上输出 beta 与 zscore，避免外部 EXE 或 Skill 再重复实现。
fn rsrs_snapshot(
    rows: &[StockHistoryRow],
    regression_period: usize,
    zscore_period: usize,
) -> Result<(f64, f64), TechnicalConsultationBasicError> {
    let required = regression_period + zscore_period - 1;
    if rows.len() < required {
        return Err(TechnicalConsultationBasicError::InsufficientHistory {
            required,
            actual: rows.len(),
        });
    }

    let mut beta_series = Vec::with_capacity(rows.len() - regression_period + 1);
    for index in (regression_period - 1)..rows.len() {
        let window = &rows[index + 1 - regression_period..=index];
        beta_series.push(regression_slope_high_on_low(window)?);
    }

    let latest_beta = *beta_series.last().ok_or_else(|| {
        TechnicalConsultationBasicError::IndicatorCalculation("RSRS beta 序列为空".to_string())
    })?;
    let zscore_window = &beta_series[beta_series.len() - zscore_period..];
    let mean = zscore_window.iter().sum::<f64>() / zscore_window.len() as f64;
    let variance = zscore_window
        .iter()
        .map(|value| {
            let diff = value - mean;
            diff * diff
        })
        .sum::<f64>()
        / zscore_window.len() as f64;
    let std_dev = variance.sqrt();
    let zscore = if std_dev.abs() <= f64::EPSILON {
        0.0
    } else {
        (latest_beta - mean) / std_dev
    };

    Ok((latest_beta, zscore))
}

// 2026-03-29 22:35 CST: 这里单独抽出高点对低点的回归斜率，原因是 RSRS 计算需要稳定、可复核的核心公式；
// 目的：把 beta 口径收口到一个函数里，后续即使要补 R² 或修正项，也不需要在快照函数中散改。
fn regression_slope_high_on_low(
    window: &[StockHistoryRow],
) -> Result<f64, TechnicalConsultationBasicError> {
    let mean_low = window.iter().map(|row| row.low).sum::<f64>() / window.len() as f64;
    let mean_high = window.iter().map(|row| row.high).sum::<f64>() / window.len() as f64;
    let denominator = window
        .iter()
        .map(|row| {
            let diff = row.low - mean_low;
            diff * diff
        })
        .sum::<f64>();

    if denominator.abs() <= f64::EPSILON {
        return Err(TechnicalConsultationBasicError::IndicatorCalculation(
            "RSRS 回归斜率分母为 0".to_string(),
        ));
    }

    let numerator = window
        .iter()
        .map(|row| (row.low - mean_low) * (row.high - mean_high))
        .sum::<f64>();

    Ok(numerator / denominator)
}

fn kdj_snapshot(
    rows: &[StockHistoryRow],
    period: usize,
) -> Result<(f64, f64, f64), TechnicalConsultationBasicError> {
    if rows.len() < period {
        return Err(TechnicalConsultationBasicError::InsufficientHistory {
            required: period,
            actual: rows.len(),
        });
    }

    let mut k = 50.0;
    let mut d = 50.0;
    let mut j = 50.0;

    for index in (period - 1)..rows.len() {
        let window = &rows[index + 1 - period..=index];
        let highest_high = window
            .iter()
            .fold(f64::MIN, |current, row| current.max(row.high));
        let lowest_low = window
            .iter()
            .fold(f64::MAX, |current, row| current.min(row.low));
        let range = highest_high - lowest_low;
        let rsv = if range.abs() <= f64::EPSILON {
            50.0
        } else {
            (((rows[index].close - lowest_low) / range) * 100.0).clamp(0.0, 100.0)
        };

        k = ((2.0 * k) + rsv) / 3.0;
        d = ((2.0 * d) + k) / 3.0;
        j = 3.0 * k - 2.0 * d;
    }

    Ok((k, d, j))
}

fn adx_snapshot(
    rows: &[StockHistoryRow],
    period: usize,
) -> Result<(f64, f64, f64), TechnicalConsultationBasicError> {
    if rows.len() < period * 2 {
        return Err(TechnicalConsultationBasicError::InsufficientHistory {
            required: period * 2,
            actual: rows.len(),
        });
    }

    let true_ranges = true_ranges(rows);
    let (plus_dm_values, minus_dm_values) = directional_movements(rows);

    let mut smoothed_tr = true_ranges[..period].iter().sum::<f64>();
    let mut smoothed_plus_dm = plus_dm_values[..period].iter().sum::<f64>();
    let mut smoothed_minus_dm = minus_dm_values[..period].iter().sum::<f64>();
    let mut dx_values = Vec::with_capacity(true_ranges.len().saturating_sub(period) + 1);

    for index in (period - 1)..true_ranges.len() {
        if index >= period {
            smoothed_tr = smoothed_tr - (smoothed_tr / period as f64) + true_ranges[index];
            smoothed_plus_dm =
                smoothed_plus_dm - (smoothed_plus_dm / period as f64) + plus_dm_values[index];
            smoothed_minus_dm =
                smoothed_minus_dm - (smoothed_minus_dm / period as f64) + minus_dm_values[index];
        }

        let plus_di = directional_index(smoothed_plus_dm, smoothed_tr);
        let minus_di = directional_index(smoothed_minus_dm, smoothed_tr);
        let denominator = plus_di + minus_di;
        let dx = if denominator.abs() <= f64::EPSILON {
            0.0
        } else {
            ((plus_di - minus_di).abs() / denominator) * 100.0
        };
        dx_values.push(dx);
    }

    if dx_values.len() < period {
        return Err(TechnicalConsultationBasicError::IndicatorCalculation(
            "ADX 序列长度不足".to_string(),
        ));
    }

    let mut adx = dx_values[..period].iter().sum::<f64>() / period as f64;
    for dx in dx_values.iter().skip(period) {
        adx = ((adx * (period as f64 - 1.0)) + dx) / period as f64;
    }

    let plus_di_14 = directional_index(smoothed_plus_dm, smoothed_tr);
    let minus_di_14 = directional_index(smoothed_minus_dm, smoothed_tr);
    Ok((adx, plus_di_14, minus_di_14))
}

// 2026-03-29 CST：这里抽出真实波动区间序列，原因是 ATR 与 ADX 都依赖同一套 true range；
// 目的：减少重复公式，避免后续继续扩展趋势类指标时把口径写散。
fn true_ranges(rows: &[StockHistoryRow]) -> Vec<f64> {
    let mut true_ranges = Vec::with_capacity(rows.len().saturating_sub(1));
    for index in 1..rows.len() {
        let current_row = &rows[index];
        let previous_row = &rows[index - 1];
        let high_low = current_row.high - current_row.low;
        let high_close = (current_row.high - previous_row.close).abs();
        let low_close = (current_row.low - previous_row.close).abs();
        true_ranges.push(high_low.max(high_close).max(low_close));
    }
    true_ranges
}

// 2026-03-29 CST：这里抽出方向性波动序列，原因是 +DI / -DI 都基于同一份日级上升动量和下降动量；
// 目的：把 ADX 相关公式拆开，后续继续补趋势类指标时更容易复用和测试。
fn directional_movements(rows: &[StockHistoryRow]) -> (Vec<f64>, Vec<f64>) {
    let mut plus_dm_values = Vec::with_capacity(rows.len().saturating_sub(1));
    let mut minus_dm_values = Vec::with_capacity(rows.len().saturating_sub(1));

    for index in 1..rows.len() {
        let current_row = &rows[index];
        let previous_row = &rows[index - 1];
        let upward_move = current_row.high - previous_row.high;
        let downward_move = previous_row.low - current_row.low;
        let plus_dm = if upward_move > downward_move && upward_move > 0.0 {
            upward_move
        } else {
            0.0
        };
        let minus_dm = if downward_move > upward_move && downward_move > 0.0 {
            downward_move
        } else {
            0.0
        };

        plus_dm_values.push(plus_dm);
        minus_dm_values.push(minus_dm);
    }

    (plus_dm_values, minus_dm_values)
}

// 2026-03-29 CST：这里抽出 DI 百分比换算，原因是 ADX 与 +DI/-DI 都依赖同一个平滑分母；
// 目的：减少重复公式，保证不同趋势相关输出共享同一口径。
fn directional_index(smoothed_dm: f64, smoothed_tr: f64) -> f64 {
    if smoothed_tr.abs() <= f64::EPSILON {
        0.0
    } else {
        (smoothed_dm / smoothed_tr) * 100.0
    }
}

// 2026-03-29 09:45 CST: 这里抽出典型价格公式，原因是 MFI 在每一步都要复用同一口径的 `(H + L + C) / 3`；
// 目的：把资金流的核心中间量固定在一个函数里，后续若继续补 CMF / ADTM 等指标也能复用同一表达。
fn typical_price(row: &StockHistoryRow) -> f64 {
    (row.high + row.low + row.close) / 3.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rsrs_test_snapshot(beta: f64, zscore: f64) -> TechnicalIndicatorSnapshot {
        TechnicalIndicatorSnapshot {
            close: 100.0,
            ema_10: 99.0,
            sma_50: 98.0,
            sma_200: 96.0,
            adx_14: 28.0,
            plus_di_14: 24.0,
            minus_di_14: 18.0,
            obv: 1_000_000.0,
            volume_sma_20: 950_000.0,
            volume_ratio_20: 1.05,
            mfi_14: 54.0,
            cci_20: 0.0,
            williams_r_14: -50.0,
            boll_width_ratio_20: 0.08,
            macd: 1.2,
            macd_signal: 0.8,
            macd_histogram: 0.4,
            rsi_14: 56.0,
            k_9: 52.0,
            d_9: 49.0,
            j_9: 58.0,
            rsrs_beta_18: beta,
            rsrs_zscore_18_60: zscore,
            boll_upper: 104.0,
            boll_middle: 100.0,
            boll_lower: 96.0,
            atr_14: 2.1,
        }
    }

    fn money_flow_test_snapshot(mfi_14: f64) -> TechnicalIndicatorSnapshot {
        // 2026-03-29 CST: 这里新增 MFI 源码级边界快照辅助函数，原因是这轮要先把 80/20 精确阈值与阈值内侧 neutral 合同钉死；
        // 目的：避免为了边界硬化再去折腾整套 CLI OHLCV 几何，让 MFI 第一版能先在最小测试面稳定收口。
        TechnicalIndicatorSnapshot {
            mfi_14,
            ..rsrs_test_snapshot(1.0, 0.0)
        }
    }

    fn mean_reversion_test_snapshot(cci_20: f64) -> TechnicalIndicatorSnapshot {
        // 2026-03-30 CST: 这里新增 CCI 源码级边界快照辅助函数，原因是这轮要先把 100 / -100 精确阈值与阈值内侧 neutral 合同钉死；
        // 目的：避免为了 CCI 边界硬化反复折腾整套 CLI OHLCV 几何，让均值回归第一版先在最小测试面稳定收口。
        TechnicalIndicatorSnapshot {
            cci_20,
            ..rsrs_test_snapshot(1.0, 0.0)
        }
    }

    fn range_position_test_snapshot(williams_r_14: f64) -> TechnicalIndicatorSnapshot {
        // 2026-03-30 CST: 这里新增 Williams %R 源码级边界快照辅助函数，原因是这轮要先把 -20 / -80 精确阈值与阈值内侧 neutral 合同钉死；
        // 目的：避免为了区间位置边界硬化反复折腾整套 CLI OHLC 几何，让 Williams %R 第一版先在最小测试面稳定收口。
        TechnicalIndicatorSnapshot {
            williams_r_14,
            ..rsrs_test_snapshot(1.0, 0.0)
        }
    }

    fn bollinger_test_snapshot(
        close: f64,
        boll_upper: f64,
        boll_middle: f64,
        boll_lower: f64,
        boll_width_ratio_20: f64,
    ) -> TechnicalIndicatorSnapshot {
        // 2026-03-29 23:25 CST: 这里新增布林带源码级边界快照辅助函数，原因是这轮要先把位置与带宽阈值边界钉死；
        // 目的：避免为了布林带边界硬化反复折腾整套 CLI OHLC 几何，让第一版先在最小测试面稳定收口。
        TechnicalIndicatorSnapshot {
            close,
            boll_upper,
            boll_middle,
            boll_lower,
            boll_width_ratio_20,
            ..rsrs_test_snapshot(1.0, 0.0)
        }
    }

    fn zero_volume_test_rows(row_count: usize) -> Vec<StockHistoryRow> {
        // 2026-03-29 CST: 这里新增零成交量历史夹具，原因是现有 CLI 已经证明 “不会 NaN”，但还没把精确回落值 50.0 钉成源码合同；
        // 目的：直接在公式层锁住 “正负资金流都为 0 -> MFI 返回 50.0” 的边界，避免后续继续补指标时把 neutral fallback 改漂。
        (0..row_count)
            .map(|offset| StockHistoryRow {
                trade_date: format!("2025-01-{:02}", offset + 1),
                open: 100.0,
                high: 101.0,
                low: 99.0,
                close: 100.0,
                adj_close: 100.0,
                volume: 0,
            })
            .collect()
    }

    #[test]
    fn mfi_classifies_overbought_distribution_at_exact_upper_threshold() {
        // 2026-03-29 CST: 这里补 MFI 上阈值精确命中单测，原因是第一版资金流规则已明确 80 为闭区间边界；
        // 目的：确保 mfi_14 恰好等于 80.0 时必须进入 overbought_distribution，而不是因阈值写法漂移落回 neutral。
        let snapshot = money_flow_test_snapshot(80.0);

        assert_eq!(
            classify_money_flow_signal(&snapshot),
            "overbought_distribution"
        );
    }

    #[test]
    fn mfi_stays_neutral_just_below_upper_threshold() {
        // 2026-03-29 CST: 这里补 MFI 上阈值内侧单测，原因是精确命中锁住以后，还需要确认 79.99 不会被误判成过热分配；
        // 目的：把 “只差一档但未过线” 的资金流场景固定为 neutral，避免后续把 >= / > 的边界悄悄改松。
        let snapshot = money_flow_test_snapshot(79.99);

        assert_eq!(classify_money_flow_signal(&snapshot), "neutral");
    }

    #[test]
    fn mfi_classifies_oversold_accumulation_at_exact_lower_threshold() {
        // 2026-03-29 CST: 这里补 MFI 下阈值精确命中单测，原因是第一版资金流规则对 20 也是闭区间边界；
        // 目的：确保 mfi_14 恰好等于 20.0 时必须进入 oversold_accumulation，而不是在低位边界含糊处理。
        let snapshot = money_flow_test_snapshot(20.0);

        assert_eq!(
            classify_money_flow_signal(&snapshot),
            "oversold_accumulation"
        );
    }

    #[test]
    fn mfi_stays_neutral_just_above_lower_threshold() {
        // 2026-03-29 CST: 这里补 MFI 下阈值内侧单测，原因是要和 79.99 的上阈值内侧场景成对出现，锁住 20.01 仍属 neutral；
        // 目的：防止后续把 <= 20.0 的判定线无意放宽，导致尚未过线的偏弱资金流被提前打成 oversold_accumulation。
        let snapshot = money_flow_test_snapshot(20.01);

        assert_eq!(classify_money_flow_signal(&snapshot), "neutral");
    }

    #[test]
    fn cci_classifies_overbought_reversal_risk_at_exact_upper_threshold() {
        // 2026-03-30 CST: 这里补 CCI 上阈值精确命中单测，原因是第一版均值回归规则已明确 100 为闭区间边界；
        // 目的：确保 cci_20 恰好等于 100.0 时必须进入 overbought_reversal_risk，而不是因阈值写法漂移落回 neutral。
        let snapshot = mean_reversion_test_snapshot(100.0);

        assert_eq!(
            classify_mean_reversion_signal(&snapshot),
            "overbought_reversal_risk"
        );
    }

    #[test]
    fn cci_stays_neutral_just_below_upper_threshold() {
        // 2026-03-30 CST: 这里补 CCI 上阈值内侧单测，原因是精确命中锁住以后，还需要确认 99.99 不会被误判成高位回落风险；
        // 目的：把“只差一点但未越线”的上沿场景固定为 neutral，避免后续把 >= / > 的边界悄悄改松。
        let snapshot = mean_reversion_test_snapshot(99.99);

        assert_eq!(classify_mean_reversion_signal(&snapshot), "neutral");
    }

    #[test]
    fn cci_classifies_oversold_rebound_candidate_at_exact_lower_threshold() {
        // 2026-03-30 CST: 这里补 CCI 下阈值精确命中单测，原因是第一版均值回归规则对 -100 也是闭区间边界；
        // 目的：确保 cci_20 恰好等于 -100.0 时必须进入 oversold_rebound_candidate，而不是在低位边界含糊处理。
        let snapshot = mean_reversion_test_snapshot(-100.0);

        assert_eq!(
            classify_mean_reversion_signal(&snapshot),
            "oversold_rebound_candidate"
        );
    }

    #[test]
    fn cci_stays_neutral_just_above_lower_threshold() {
        // 2026-03-30 CST: 这里补 CCI 下阈值内侧单测，原因是要和 99.99 的上阈值内侧场景成对出现，锁住 -99.99 仍属 neutral；
        // 目的：防止后续把 <= -100.0 的判定线无意放宽，导致尚未越线的弱偏离样本被提前判成低位修复候选。
        let snapshot = mean_reversion_test_snapshot(-99.99);

        assert_eq!(classify_mean_reversion_signal(&snapshot), "neutral");
    }

    #[test]
    fn williams_r_classifies_overbought_pullback_risk_at_exact_upper_threshold() {
        // 2026-03-30 CST: 这里补 Williams %R 上阈值精确命中单测，原因是第一版区间位置规则已明确 -20 为闭区间边界；
        // 目的：确保 williams_r_14 恰好等于 -20.0 时必须进入 overbought_pullback_risk，而不是因阈值写法漂移落回 neutral。
        let snapshot = range_position_test_snapshot(-20.0);

        assert_eq!(
            classify_range_position_signal(&snapshot),
            "overbought_pullback_risk"
        );
    }

    #[test]
    fn williams_r_stays_neutral_just_below_upper_threshold() {
        // 2026-03-30 CST: 这里补 Williams %R 上阈值内侧单测，原因是 exact-threshold 锁住以后还需要确认 -20.01 不会被误判成高位回落风险；
        // 目的：把“只差一点但未触线”的上沿场景固定为 neutral，避免后续有人把 >= / > 的边界悄悄改松。
        let snapshot = range_position_test_snapshot(-20.01);

        assert_eq!(classify_range_position_signal(&snapshot), "neutral");
    }

    #[test]
    fn williams_r_classifies_oversold_rebound_candidate_at_exact_lower_threshold() {
        // 2026-03-30 CST: 这里补 Williams %R 下阈值精确命中单测，原因是区间位置第一版对 -80 也是闭区间边界；
        // 目的：确保 williams_r_14 恰好等于 -80.0 时必须进入 oversold_rebound_candidate，而不是在低位边界含糊处理。
        let snapshot = range_position_test_snapshot(-80.0);

        assert_eq!(
            classify_range_position_signal(&snapshot),
            "oversold_rebound_candidate"
        );
    }

    #[test]
    fn williams_r_stays_neutral_just_above_lower_threshold() {
        // 2026-03-30 CST: 这里补 Williams %R 下阈值内侧单测，原因是要和 -20.01 的上沿内侧场景成对出现，锁住 -79.99 仍属 neutral；
        // 目的：防止后续把 <= -80.0 的判定线无意放宽，导致尚未触线的弱低位样本被提前打成 oversold_rebound_candidate。
        let snapshot = range_position_test_snapshot(-79.99);

        assert_eq!(classify_range_position_signal(&snapshot), "neutral");
    }

    #[test]
    fn bollinger_classifies_upper_band_breakout_risk_at_exact_upper_band() {
        // 2026-03-29 23:25 CST: 这里补布林带上轨精确命中单测，原因是第一版规则明确把 `close == boll_upper` 视为上轨突破风险边界；
        // 目的：确保上轨边界不会因为后续阈值写法调整被悄悄改成严格大于才触发。
        let snapshot = bollinger_test_snapshot(104.0, 104.0, 100.0, 96.0, 0.12);

        assert_eq!(
            classify_bollinger_position_signal(&snapshot),
            "upper_band_breakout_risk"
        );
    }

    #[test]
    fn bollinger_classifies_lower_band_rebound_candidate_at_exact_lower_band() {
        // 2026-03-29 23:25 CST: 这里补布林带下轨精确命中单测，原因是第一版规则明确把 `close == boll_lower` 视为下轨反抽候选边界；
        // 目的：确保下轨边界不会因为比较符号漂移而把精确命中样本误落回 neutral。
        let snapshot = bollinger_test_snapshot(96.0, 104.0, 100.0, 96.0, 0.12);

        assert_eq!(
            classify_bollinger_position_signal(&snapshot),
            "lower_band_rebound_candidate"
        );
    }

    #[test]
    fn bollinger_midline_classifies_support_bias_above_middle_band() {
        // 2026-03-29 10:35 CST: 这里新增布林带中轨支撑单测，原因是方案 A 需要先锁住“高于中轨但未触发上轨极端”的中间层语义；
        // 目的：确保中轨上方样本正式进入 `midline_support_bias`，而不是继续被吞进 neutral。
        let snapshot = bollinger_test_snapshot(101.2, 104.0, 100.0, 96.0, 0.08);

        assert_eq!(
            classify_bollinger_midline_signal(&snapshot),
            "midline_support_bias"
        );
    }

    #[test]
    fn bollinger_midline_classifies_resistance_bias_below_middle_band() {
        // 2026-03-29 10:35 CST: 这里新增布林带中轨压制单测，原因是中轨下方运行需要与下轨极端回补场景分开建模；
        // 目的：确保中轨下方样本正式进入 `midline_resistance_bias`，形成与支撑偏多对称的稳定合同。
        let snapshot = bollinger_test_snapshot(98.8, 104.0, 100.0, 96.0, 0.08);

        assert_eq!(
            classify_bollinger_midline_signal(&snapshot),
            "midline_resistance_bias"
        );
    }

    #[test]
    fn bollinger_bandwidth_classifies_expanding_at_exact_upper_threshold() {
        // 2026-03-29 23:25 CST: 这里补布林带扩张阈值单测，原因是第一版把 0.12 作为 expanding 的闭区间边界；
        // 目的：确保带宽阈值精确命中时正式进入 expanding，而不是因 >= / > 漂移落回 normal。
        let snapshot = bollinger_test_snapshot(100.0, 106.0, 100.0, 94.0, 0.12);

        assert_eq!(classify_bollinger_bandwidth_signal(&snapshot), "expanding");
    }

    #[test]
    fn bollinger_bandwidth_stays_normal_just_below_expanding_threshold() {
        // 2026-03-29 23:25 CST: 这里补布林带扩张阈值内侧单测，原因是精确边界锁住以后还需要确认 0.1199 不会被误判成扩张；
        // 目的：把“只差一点但未越线”的场景固定为 normal，避免后续阈值被无意放宽。
        let snapshot = bollinger_test_snapshot(100.0, 105.99, 100.0, 94.01, 0.1199);

        assert_eq!(classify_bollinger_bandwidth_signal(&snapshot), "normal");
    }

    #[test]
    fn bollinger_bandwidth_classifies_contracting_at_exact_lower_threshold() {
        // 2026-03-29 23:25 CST: 这里补布林带收敛阈值单测，原因是第一版把 0.05 作为 contracting 的闭区间边界；
        // 目的：确保带宽精确等于 0.05 时正式进入 contracting，而不是在低波动边界含糊处理。
        let snapshot = bollinger_test_snapshot(100.0, 102.5, 100.0, 97.5, 0.05);

        assert_eq!(classify_bollinger_bandwidth_signal(&snapshot), "contracting");
    }

    #[test]
    fn bollinger_bandwidth_stays_normal_just_above_contracting_threshold() {
        // 2026-03-29 23:25 CST: 这里补布林带收敛阈值内侧单测，原因是要和 expanding 内侧样本成对出现，锁住 0.0501 仍属 normal；
        // 目的：防止后续把 <= 0.05 的判断线无意放宽，导致尚未进入收敛区的样本被提前归类。
        let snapshot = bollinger_test_snapshot(100.0, 102.505, 100.0, 97.495, 0.0501);

        assert_eq!(classify_bollinger_bandwidth_signal(&snapshot), "normal");
    }

    #[test]
    fn mfi_returns_exact_neutral_50_when_recent_window_has_only_zero_volume() {
        // 2026-03-29 CST: 这里补零成交量精确回落值单测，原因是现有 CLI 只锁了 finite，但还没锁住公式层的 neutral fallback 数值；
        // 目的：明确正负资金流都为 0 时必须返回 50.0，后续继续扩展资金流家族时也不能把这个稳定中性口径改漂。
        let rows = zero_volume_test_rows(15);

        assert_eq!(mfi_last(&rows, 14).expect("mfi should stay computable"), 50.0);
    }

    #[test]
    fn rsrs_classifies_bullish_breakout_at_exact_positive_threshold() {
        // 2026-03-29 CST: 这里补正向精确阈值边界单测，原因是上一轮只锁了 mismatch neutral，尚未把 0.70 / 1.00 的精确命中点钉成合同。
        // 目的：明确 zscore 恰好等于 0.7 且 beta 恰好等于 1.0 时，分类必须进入 bullish_breakout，而不是因边界漂移落回 neutral。
        let snapshot = rsrs_test_snapshot(1.0, 0.7);

        assert_eq!(classify_rsrs_signal(&snapshot), "bullish_breakout");
    }

    #[test]
    fn rsrs_stays_neutral_just_below_positive_threshold() {
        // 2026-03-29 CST: 这里补正向阈值内侧单测，原因是 exact-threshold 锁住以后，还需要确认 0.69 不会被误判成突破。
        // 目的：把“只差一档但未过线”的正向场景固定为 neutral，避免后续有人把 >= / > 的边界悄悄改松。
        let snapshot = rsrs_test_snapshot(1.01, 0.69);

        assert_eq!(classify_rsrs_signal(&snapshot), "neutral");
    }

    #[test]
    fn rsrs_classifies_bearish_pressure_at_exact_negative_threshold() {
        // 2026-03-29 CST: 这里补负向精确阈值边界单测，原因是当前规则对 -0.7 / 1.0 也是闭区间，应该和正向边界一样有明确合同。
        // 目的：明确 zscore 恰好等于 -0.7 且 beta 恰好等于 1.0 时，分类必须进入 bearish_pressure，而不是在负向边界上含糊处理。
        let snapshot = rsrs_test_snapshot(1.0, -0.7);

        assert_eq!(classify_rsrs_signal(&snapshot), "bearish_pressure");
    }

    #[test]
    fn rsrs_stays_neutral_just_above_negative_threshold() {
        // 2026-03-29 CST: 这里补负向阈值内侧单测，原因是要和 0.69 的正向场景成对出现，锁住 -0.69 仍属 neutral。
        // 目的：防止后续把 <= -0.7 的判定线无意放宽，导致尚未过线的弱负向样本被提前打成 bearish_pressure。
        let snapshot = rsrs_test_snapshot(0.99, -0.69);

        assert_eq!(classify_rsrs_signal(&snapshot), "neutral");
    }

    #[test]
    fn rsrs_neutral_guidance_mentions_resonance_when_zscore_is_positive_but_beta_below_one() {
        let snapshot = rsrs_test_snapshot(0.95, 1.25);
        let rsrs_signal = classify_rsrs_signal(&snapshot);

        // 2026-03-30 00:18 CST: 这里先锁正向 mismatch 的最小单测，原因是 CLI 夹具在 RSRS 整窗回归下几何噪声太大；
        // 目的：直接把“zscore 偏强但 beta 仍未站上 1 -> neutral 且文案提示未形成共振”钉成稳定合同。
        assert_eq!(rsrs_signal, "neutral");
        assert!(
            build_summary_with_timing_and_rsrs(
                "bullish",
                "strong",
                "confirmed",
                "none",
                "neutral",
                rsrs_signal,
                "positive",
                "normal"
            )
            .contains("共振")
        );
        assert!(
            build_recommended_actions_with_timing_and_rsrs(
                "bullish",
                "strong",
                "confirmed",
                "none",
                "neutral",
                rsrs_signal,
                "positive",
                "normal"
            )
            .iter()
            .any(|text| text.contains("共振"))
        );
        assert!(
            // 2026-03-29 CST: 修正观察点测试的入参顺序，原因是这里需要在 rsrs_signal 后先传 snapshot。
            // 目的：让正向 mismatch 的 neutral 文案断言能够稳定命中 RSRS 观察点逻辑并恢复编译。
            build_watch_points_with_timing_and_rsrs(
                "bullish",
                "strong",
                "confirmed",
                "none",
                "neutral",
                rsrs_signal,
                &snapshot,
                "normal"
            )
            .iter()
            .any(|text| text.contains("共振"))
        );
    }

    #[test]
    fn rsrs_neutral_guidance_mentions_resonance_when_zscore_is_negative_but_beta_above_one() {
        let snapshot = rsrs_test_snapshot(1.05, -1.25);
        let rsrs_signal = classify_rsrs_signal(&snapshot);

        // 2026-03-30 00:18 CST: 这里再锁负向 mismatch 的最小单测，原因是 bearish_pressure 的边界也需要用稳定单测收口；
        // 目的：确保“zscore 转弱但 beta 仍高于 1 -> neutral 且文案提示未形成共振”不会被后续文案改动冲掉。
        assert_eq!(rsrs_signal, "neutral");
        assert!(
            build_summary_with_timing_and_rsrs(
                "bearish",
                "moderate",
                "weakening",
                "none",
                "neutral",
                rsrs_signal,
                "negative",
                "high"
            )
            .contains("共振")
        );
        assert!(
            build_recommended_actions_with_timing_and_rsrs(
                "bearish",
                "moderate",
                "weakening",
                "none",
                "neutral",
                rsrs_signal,
                "negative",
                "high"
            )
            .iter()
            .any(|text| text.contains("共振"))
        );
        assert!(
            // 2026-03-29 CST: 同步修正负向 mismatch 观察点测试的入参顺序，原因是此前把状态字符串误放到了 snapshot 位置。
            // 目的：保证 bearish neutral 场景也能覆盖“未形成共振”的观察点文案，而不是卡在编译阶段。
            build_watch_points_with_timing_and_rsrs(
                "bearish",
                "moderate",
                "weakening",
                "none",
                "neutral",
                rsrs_signal,
                &snapshot,
                "high"
            )
            .iter()
            .any(|text| text.contains("共振"))
        );
    }
}
