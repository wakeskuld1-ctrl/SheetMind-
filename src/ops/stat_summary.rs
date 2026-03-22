use polars::prelude::DataType;
use serde::Serialize;

use crate::frame::loader::LoadedTable;
use crate::ops::summary::{SummaryError, TopValueCount, summarize_table};

// 2026-03-21: 这里定义表级概览结构，目的是把当前表的字段类型分布稳定暴露给后续建模入口。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TableOverview {
    pub numeric_columns: usize,
    pub categorical_columns: usize,
    pub boolean_columns: usize,
}

// 2026-03-21: 这里定义数值列统计摘要，目的是为回归、聚类和异常值前检查提供稳定桥接统计量。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NumericStatSummary {
    pub column: String,
    pub count: usize,
    pub null_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missing_rate: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub q1: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub median: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mean: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub q3: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sum: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zero_ratio: Option<f64>,
}

// 2026-03-21: 这里定义类别列统计摘要，目的是让后续问答和建模前检查都能直接理解离散分布集中度。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CategoricalStatSummary {
    pub column: String,
    pub count: usize,
    pub null_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missing_rate: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distinct_count: Option<usize>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub top_values: Vec<TopValueCount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_share: Option<f64>,
}

// 2026-03-21: 这里定义布尔列统计摘要，目的是把标签分布和规则字段分布稳定桥接给后续分析层。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BooleanStatSummary {
    pub column: String,
    pub count: usize,
    pub null_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missing_rate: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub true_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub false_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub true_ratio: Option<f64>,
}

// 2026-03-21: 这里定义面向用户的中文统计摘要，目的是让非 IT 用户无需读结构化字段也能理解结果。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StatHumanSummary {
    pub overall: String,
    #[serde(default)]
    pub key_points: Vec<String>,
    pub recommended_next_step: String,
}

// 2026-03-21: 这里定义统计桥接 Tool 的统一输出结构，目的是固定后续分析建模层消费的协议。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StatSummaryResult {
    pub row_count: usize,
    pub column_count: usize,
    pub table_overview: TableOverview,
    #[serde(default)]
    pub numeric_summaries: Vec<NumericStatSummary>,
    #[serde(default)]
    pub categorical_summaries: Vec<CategoricalStatSummary>,
    #[serde(default)]
    pub boolean_summaries: Vec<BooleanStatSummary>,
    pub human_summary: StatHumanSummary,
}

// 2026-03-21: 这里提供统计桥接主入口，目的是把基础画像上升为更适合建模前检查的统计摘要结果。
pub fn stat_summary(
    loaded: &LoadedTable,
    requested_columns: &[&str],
    top_k: usize,
) -> Result<StatSummaryResult, SummaryError> {
    let summaries = summarize_table(loaded, requested_columns, top_k)?;
    let mut table_overview = TableOverview {
        numeric_columns: 0,
        categorical_columns: 0,
        boolean_columns: 0,
    };
    let mut numeric_summaries = Vec::new();
    let mut categorical_summaries = Vec::new();
    let mut boolean_summaries = Vec::new();

    for summary in &summaries {
        match summary.summary_kind.as_str() {
            "numeric" => {
                table_overview.numeric_columns += 1;
                numeric_summaries
                    .push(build_numeric_stat_summary(loaded, summary.column.as_str())?);
            }
            "boolean" => {
                table_overview.boolean_columns += 1;
                boolean_summaries.push(BooleanStatSummary {
                    column: summary.column.clone(),
                    count: summary.count,
                    null_count: summary.null_count,
                    missing_rate: summary.missing_rate,
                    true_count: summary.true_count,
                    false_count: summary.false_count,
                    // 2026-03-21: 这里按有效布尔值口径计算 true_ratio，目的是让标签分布不受缺失行干扰。
                    true_ratio: calculate_ratio(summary.true_count, summary.count),
                });
            }
            _ => {
                table_overview.categorical_columns += 1;
                categorical_summaries.push(CategoricalStatSummary {
                    column: summary.column.clone(),
                    count: summary.count,
                    null_count: summary.null_count,
                    missing_rate: summary.missing_rate,
                    distinct_count: summary.distinct_count,
                    top_values: summary.top_values.clone(),
                    // 2026-03-21: 这里按有效类别值口径计算 top_share，目的是让主值集中度可直接用于业务提示。
                    top_share: summary
                        .top_values
                        .first()
                        .map(|top_value| top_value.count)
                        .and_then(|top_count| calculate_ratio(Some(top_count), summary.count)),
                });
            }
        }
    }

    let key_points = build_key_points(
        &numeric_summaries,
        &categorical_summaries,
        &boolean_summaries,
    );
    let overall = if summaries.is_empty() {
        "这张表当前没有可统计的列，请先确认工作表内容。".to_string()
    } else {
        "这张表已经具备基础统计摘要，可进入下一步分析。".to_string()
    };
    let recommended_next_step = if table_overview.numeric_columns > 0 {
        "建议先选择目标列和特征列，再进入建模前检查。".to_string()
    } else {
        "建议先确认需要分析的指标列，再进入下一步分析。".to_string()
    };

    Ok(StatSummaryResult {
        row_count: loaded.dataframe.height(),
        column_count: if requested_columns.is_empty() {
            loaded.dataframe.width()
        } else {
            requested_columns.len()
        },
        table_overview,
        numeric_summaries,
        categorical_summaries,
        boolean_summaries,
        human_summary: StatHumanSummary {
            overall,
            key_points,
            recommended_next_step,
        },
    })
}

// 2026-03-21: 这里把数值列的桥接统计集中实现，目的是让分位数、中位数和零值占比口径在一个地方维护。
fn build_numeric_stat_summary(
    loaded: &LoadedTable,
    column: &str,
) -> Result<NumericStatSummary, SummaryError> {
    let series = loaded
        .dataframe
        .column(column)
        .map_err(|_| SummaryError::MissingColumn(column.to_string()))?
        .as_materialized_series();
    let casted =
        series
            .cast(&DataType::Float64)
            .map_err(|error| SummaryError::SummarizeColumn {
                column: column.to_string(),
                message: error.to_string(),
            })?;
    let values = casted
        .f64()
        .map_err(|error| SummaryError::SummarizeColumn {
            column: column.to_string(),
            message: error.to_string(),
        })?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    let count = values.len();
    let null_count = series.len().saturating_sub(count);
    let sum = if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f64>())
    };
    let mean = sum.map(|total| total / count as f64);
    let mut sorted_values = values.clone();
    sorted_values.sort_by(|left, right| left.total_cmp(right));

    Ok(NumericStatSummary {
        column: column.to_string(),
        count,
        null_count,
        missing_rate: calculate_optional_ratio(null_count, series.len()),
        min: sorted_values.first().copied(),
        q1: calculate_quantile(&sorted_values, 0.25),
        median: calculate_quantile(&sorted_values, 0.50),
        mean,
        q3: calculate_quantile(&sorted_values, 0.75),
        max: sorted_values.last().copied(),
        sum,
        // 2026-03-21: 这里按有效数值口径计算 zero_ratio，目的是避免缺失值把 0 值占比稀释掉。
        zero_ratio: Some(calculate_zero_ratio(&values)),
    })
}

// 2026-03-21: 这里生成中文关键统计点，目的是让终端问答界面先拿到几条可直接展示的核心观察。
fn build_key_points(
    numeric_summaries: &[NumericStatSummary],
    categorical_summaries: &[CategoricalStatSummary],
    boolean_summaries: &[BooleanStatSummary],
) -> Vec<String> {
    let mut key_points = Vec::new();

    for summary in categorical_summaries {
        if let (Some(top_value), Some(top_share)) = (summary.top_values.first(), summary.top_share)
        {
            if top_share >= 0.50 {
                key_points.push(format!("{} 主要集中在 {}", summary.column, top_value.value));
            }
        }
    }

    for summary in numeric_summaries {
        if let (Some(mean), Some(median)) = (summary.mean, summary.median) {
            if median > 0.0 && mean >= median * 1.5 {
                key_points.push(format!(
                    "{} 列有明显长尾，高值记录会拉高均值",
                    summary.column
                ));
                continue;
            }
        }

        if let Some(zero_ratio) = summary.zero_ratio {
            if zero_ratio >= 0.50 {
                key_points.push(format!("{} 列有较多 0 值记录", summary.column));
            }
        }
    }

    for summary in boolean_summaries {
        if let Some(true_ratio) = summary.true_ratio {
            if true_ratio >= 0.80 {
                key_points.push(format!("{} 列大部分为 true", summary.column));
            } else if true_ratio <= 0.20 {
                key_points.push(format!("{} 列大部分为 false", summary.column));
            }
        }
    }

    key_points.truncate(3);
    key_points
}

// 2026-03-21: 这里统一计算可选比例，目的是避免多处重复写“总数为 0 时返回 None”的保护逻辑。
fn calculate_optional_ratio(numerator: usize, denominator: usize) -> Option<f64> {
    if denominator == 0 {
        None
    } else {
        Some(numerator as f64 / denominator as f64)
    }
}

// 2026-03-21: 这里兼容“值可能不存在”的比例计算，目的是复用到 top_share 和 true_ratio 这类可选场景。
fn calculate_ratio(numerator: Option<usize>, denominator: usize) -> Option<f64> {
    numerator.and_then(|value| calculate_optional_ratio(value, denominator))
}

// 2026-03-21: 这里统一计算数值列 zero_ratio，目的是把零值口径固定在有效数值集合之上。
fn calculate_zero_ratio(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let zero_count = values
        .iter()
        .filter(|value| value.abs() <= f64::EPSILON)
        .count();
    zero_count as f64 / values.len() as f64
}

// 2026-03-21: 这里用线性插值计算分位数，目的是让奇数和偶数样本下的 q1/median/q3 都保持稳定口径。
fn calculate_quantile(sorted_values: &[f64], probability: f64) -> Option<f64> {
    if sorted_values.is_empty() {
        return None;
    }

    if sorted_values.len() == 1 {
        return sorted_values.first().copied();
    }

    let last_index = (sorted_values.len() - 1) as f64;
    let position = probability.clamp(0.0, 1.0) * last_index;
    let lower_index = position.floor() as usize;
    let upper_index = position.ceil() as usize;
    let lower_value = sorted_values[lower_index];
    let upper_value = sorted_values[upper_index];

    if lower_index == upper_index {
        Some(lower_value)
    } else {
        Some(lower_value + (upper_value - lower_value) * (position - lower_index as f64))
    }
}
