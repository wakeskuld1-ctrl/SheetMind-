use polars::prelude::{AnyValue, DataType};
use serde::Serialize;
use thiserror::Error;

use crate::frame::loader::LoadedTable;

// 2026-03-25: 这里定义趋势点结构，原因是趋势分析第一版不仅要给摘要，还要给按时间排序后的点位；目的是让上层 Skill 可以直接复述趋势走向并保留可视化基础数据。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TrendPoint {
    pub time: String,
    pub value: f64,
}

// 2026-03-25: 这里定义趋势人类摘要，原因是业务用户更容易理解“整体在上升还是下降”；目的是把起止变化翻译成可直接展示的中文结论。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TrendHumanSummary {
    pub overall: String,
    #[serde(default)]
    pub key_points: Vec<String>,
    pub recommended_next_step: String,
}

// 2026-03-25: 这里定义趋势分析统一输出，原因是 dispatcher 和 Skill 都需要稳定读取趋势方向、变化量和排序点位；目的是让趋势观察成为统计诊断层的标准协议之一。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TrendAnalysisResult {
    pub time_column: String,
    pub value_column: String,
    pub row_count: usize,
    pub point_count: usize,
    pub direction: String,
    pub start_time: String,
    pub end_time: String,
    pub start_value: f64,
    pub end_value: f64,
    pub absolute_change: f64,
    pub change_rate: f64,
    #[serde(default)]
    pub points: Vec<TrendPoint>,
    pub human_summary: TrendHumanSummary,
}

// 2026-03-25: 这里定义趋势分析错误类型，原因是第一版要把缺列、非数值列和有效点不足直白告诉用户；目的是让低 IT 用户知道下一步该先清洗什么。
#[derive(Debug, Error)]
pub enum TrendAnalysisError {
    #[error("trend_analysis 缺少 time_column 参数")]
    MissingTimeColumnArg,
    #[error("trend_analysis 缺少 value_column 参数")]
    MissingValueColumnArg,
    #[error("列 `{column}` 不存在")]
    MissingColumn { column: String },
    #[error("列 `{column}` 不是可做趋势分析的数值列，请先做类型转换")]
    NonNumericColumn { column: String },
    #[error("`{time_column}` 和 `{value_column}` 可用的趋势点不足，至少需要 2 个有效点")]
    NotEnoughPoints {
        time_column: String,
        value_column: String,
    },
}

// 2026-03-25: 这里提供趋势分析主入口，原因是统计诊断层需要回答“时间上整体是在涨还是跌”；目的是让建模前观察先具备最小时间趋势判断能力。
pub fn trend_analysis(
    loaded: &LoadedTable,
    time_column: &str,
    value_column: &str,
) -> Result<TrendAnalysisResult, TrendAnalysisError> {
    if time_column.trim().is_empty() {
        return Err(TrendAnalysisError::MissingTimeColumnArg);
    }
    if value_column.trim().is_empty() {
        return Err(TrendAnalysisError::MissingValueColumnArg);
    }

    let points = extract_trend_points(loaded, time_column, value_column)?;
    if points.len() < 2 {
        return Err(TrendAnalysisError::NotEnoughPoints {
            time_column: time_column.to_string(),
            value_column: value_column.to_string(),
        });
    }

    let start = &points[0];
    let end = &points[points.len() - 1];
    let absolute_change = end.value - start.value;
    let change_rate = if start.value.abs() <= 1e-12 {
        0.0
    } else {
        absolute_change / start.value
    };
    let direction = classify_direction(absolute_change).to_string();
    let human_summary = build_human_summary(
        time_column,
        value_column,
        start,
        end,
        absolute_change,
        change_rate,
        &direction,
    );

    Ok(TrendAnalysisResult {
        time_column: time_column.to_string(),
        value_column: value_column.to_string(),
        row_count: loaded.dataframe.height(),
        point_count: points.len(),
        direction,
        start_time: start.time.clone(),
        end_time: end.time.clone(),
        start_value: start.value,
        end_value: end.value,
        absolute_change,
        change_rate,
        points,
        human_summary,
    })
}

// 2026-03-25: 这里抽取并排序趋势点，原因是趋势分析要先建立稳定的时间序列视图；目的是让输入行顺序不影响最终趋势判断。
fn extract_trend_points(
    loaded: &LoadedTable,
    time_column: &str,
    value_column: &str,
) -> Result<Vec<TrendPoint>, TrendAnalysisError> {
    let time_series = loaded
        .dataframe
        .column(time_column)
        .map_err(|_| TrendAnalysisError::MissingColumn {
            column: time_column.to_string(),
        })?
        .as_materialized_series()
        .clone();
    let value_series = loaded
        .dataframe
        .column(value_column)
        .map_err(|_| TrendAnalysisError::MissingColumn {
            column: value_column.to_string(),
        })?
        .as_materialized_series()
        .cast(&DataType::Float64)
        .map_err(|_| TrendAnalysisError::NonNumericColumn {
            column: value_column.to_string(),
        })?;
    let values = value_series
        .f64()
        .map_err(|_| TrendAnalysisError::NonNumericColumn {
            column: value_column.to_string(),
        })?;

    let mut points = time_series
        .iter()
        .zip(values.into_iter())
        .filter_map(|(time_value, numeric_value)| {
            numeric_value.and_then(|value| {
                any_value_to_label(&time_value).map(|time| TrendPoint { time, value })
            })
        })
        .collect::<Vec<_>>();

    points.sort_by(|left, right| left.time.cmp(&right.time));
    Ok(points)
}

// 2026-03-25: 这里统一把时间列值转成展示标签，原因是第一版趋势分析要兼容文本、整数和日期序列的最小读取；目的是让排序后的点位能稳定回传给上层。
fn any_value_to_label(value: &AnyValue<'_>) -> Option<String> {
    match value {
        AnyValue::Null => None,
        // 2026-03-25: 这里对字符串值单独去掉调试引号，原因是 AnyValue::to_string() 在字符串场景会带上外层引号；目的是让趋势点返回的时间标签保持干净可读。
        AnyValue::String(text) => Some((*text).to_string()),
        // 2026-03-25: 这里兼容拥有所有权的字符串值，原因是不同 Polars 场景下字符串 AnyValue 可能是借用或拥有两种形态；目的是避免趋势标签在运行时分支下表现不一致。
        AnyValue::StringOwned(text) => Some(text.as_str().to_string()),
        _ => {
            let label = value.to_string();
            if label.trim().is_empty() {
                None
            } else {
                Some(label)
            }
        }
    }
}

// 2026-03-25: 这里统一判断趋势方向，原因是趋势分析第一版只需要稳定区分上升、下降和持平；目的是让上层 Skill 能先给用户明确结论。
fn classify_direction(change: f64) -> &'static str {
    if change > 1e-12 {
        "upward"
    } else if change < -1e-12 {
        "downward"
    } else {
        "flat"
    }
}

// 2026-03-25: 这里生成趋势人类摘要，原因是时间趋势判断最终要以业务可读的话术交付；目的是让结果不只停留在数值字段本身。
fn build_human_summary(
    time_column: &str,
    value_column: &str,
    start: &TrendPoint,
    end: &TrendPoint,
    absolute_change: f64,
    change_rate: f64,
    direction: &str,
) -> TrendHumanSummary {
    let direction_label = match direction {
        "upward" => "整体呈上升趋势",
        "downward" => "整体呈下降趋势",
        _ => "整体较为平稳",
    };

    TrendHumanSummary {
        overall: format!(
            "`{value_column}` 基于 `{time_column}` 的趋势分析已完成：从 {} 到 {}，{}。",
            start.time, end.time, direction_label
        ),
        key_points: vec![
            format!(
                "起点值为 {:.4}，终点值为 {:.4}，绝对变化为 {:.4}",
                start.value, end.value, absolute_change
            ),
            format!("相对起点的变化率约为 {:.2}%", change_rate * 100.0),
        ],
        recommended_next_step:
            "建议继续结合分布和异常值结果，判断趋势变化是稳定增长、短期波动还是被极端值拉动。"
                .to_string(),
    }
}
