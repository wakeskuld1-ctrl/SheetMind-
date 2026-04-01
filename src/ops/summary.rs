use std::collections::BTreeMap;

use polars::prelude::{AnyValue, DataType, Series};
use serde::Serialize;
use thiserror::Error;

use crate::frame::loader::LoadedTable;

// 2026-03-21: 这里定义高频值摘要项，目的是把文本列最常见取值稳定暴露给上层问答与决策能力。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TopValueCount {
    pub value: String,
    pub count: usize,
}

// 2026-03-21: 这里定义列级统计摘要，目的是让不同类型列都能通过一份稳定结构对外输出关键画像。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ColumnSummary {
    pub column: String,
    pub dtype: String,
    pub summary_kind: String,
    pub count: usize,
    pub null_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missing_rate: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distinct_count: Option<usize>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub top_values: Vec<TopValueCount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_number: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_number: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mean: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sum: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub true_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub false_count: Option<usize>,
}

// 2026-03-21: 这里定义统计摘要错误，目的是把缺列、类型读取失败和底层统计失败分开暴露给上层。
#[derive(Debug, Error)]
pub enum SummaryError {
    #[error("找不到列: {0}")]
    MissingColumn(String),
    #[error("无法统计列 {column}: {message}")]
    SummarizeColumn { column: String, message: String },
}

// 2026-03-21: 这里在已加载表上执行列级摘要，目的是让用户在进入建模前先快速得到稳定数据画像。
pub fn summarize_table(
    loaded: &LoadedTable,
    requested_columns: &[&str],
    top_k: usize,
) -> Result<Vec<ColumnSummary>, SummaryError> {
    let columns = if requested_columns.is_empty() {
        loaded
            .handle
            .columns()
            .iter()
            .map(|column| column.as_str())
            .collect::<Vec<_>>()
    } else {
        requested_columns.to_vec()
    };

    columns
        .into_iter()
        .map(|column| summarize_column(loaded, column, top_k))
        .collect()
}

// 2026-03-21: 这里按列类型分派摘要逻辑，目的是把数值、文本和布尔列的画像规则清晰拆开。
fn summarize_column(
    loaded: &LoadedTable,
    column: &str,
    top_k: usize,
) -> Result<ColumnSummary, SummaryError> {
    let series = loaded
        .dataframe
        .column(column)
        .map_err(|_| SummaryError::MissingColumn(column.to_string()))?
        .as_materialized_series();
    let null_count = series.null_count();
    let count = series.len().saturating_sub(null_count);

    if matches!(series.dtype(), DataType::Boolean) {
        summarize_boolean_column(series, column, count, null_count)
    } else if is_numeric_dtype(series.dtype()) {
        summarize_numeric_column(series, column, count, null_count)
    } else {
        summarize_string_column(series, column, top_k)
    }
}

// 2026-03-21: 这里汇总数值列指标，目的是输出后续建模和异常分析最常用的基础统计量。
fn summarize_numeric_column(
    series: &Series,
    column: &str,
    count: usize,
    null_count: usize,
) -> Result<ColumnSummary, SummaryError> {
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
        })?;

    let mut min_number = None::<f64>;
    let mut max_number = None::<f64>;
    let mut sum = 0.0_f64;

    for value in values.into_iter().flatten() {
        min_number = Some(match min_number {
            Some(current) => current.min(value),
            None => value,
        });
        max_number = Some(match max_number {
            Some(current) => current.max(value),
            None => value,
        });
        sum += value;
    }

    Ok(ColumnSummary {
        column: column.to_string(),
        dtype: dtype_label(series.dtype()).to_string(),
        summary_kind: "numeric".to_string(),
        count,
        null_count,
        // 2026-03-21: 这里补缺失率输出，目的是让上层问答和质量判断直接复用统一比例指标。
        missing_rate: calculate_missing_rate(series.len(), null_count),
        distinct_count: None,
        top_values: Vec::new(),
        min_number,
        max_number,
        mean: if count == 0 {
            None
        } else {
            Some(sum / count as f64)
        },
        sum: if count == 0 { None } else { Some(sum) },
        true_count: None,
        false_count: None,
    })
}

// 2026-03-21: 这里汇总文本列指标，目的是把空白和占位缺失值先过滤掉，再输出更贴近 Excel 用户心智的离散分布。
fn summarize_string_column(
    series: &Series,
    column: &str,
    top_k: usize,
) -> Result<ColumnSummary, SummaryError> {
    let mut counts = BTreeMap::<String, usize>::new();
    let mut missing_count = 0_usize;

    for row_index in 0..series.len() {
        let value = series
            .get(row_index)
            .map_err(|error| SummaryError::SummarizeColumn {
                column: column.to_string(),
                message: error.to_string(),
            })?;

        match value {
            AnyValue::Null => {
                missing_count += 1;
            }
            _ => {
                let rendered = series
                    .str_value(row_index)
                    .map_err(|error| SummaryError::SummarizeColumn {
                        column: column.to_string(),
                        message: error.to_string(),
                    })?
                    .into_owned();

                if is_missing_text(&rendered) {
                    missing_count += 1;
                } else {
                    *counts.entry(rendered).or_default() += 1;
                }
            }
        }
    }

    let distinct_count = counts.len();
    let count = series.len().saturating_sub(missing_count);
    let mut top_values = counts
        .into_iter()
        .map(|(value, count)| TopValueCount { value, count })
        .collect::<Vec<_>>();
    top_values.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.value.cmp(&right.value))
    });
    top_values.truncate(top_k);

    Ok(ColumnSummary {
        column: column.to_string(),
        dtype: dtype_label(series.dtype()).to_string(),
        summary_kind: "string".to_string(),
        count,
        null_count: missing_count,
        // 2026-03-21: 这里沿用统一缺失率计算，目的是把空白与占位值也纳入同一口径的质量指标。
        missing_rate: calculate_missing_rate(series.len(), missing_count),
        distinct_count: Some(distinct_count),
        top_values,
        min_number: None,
        max_number: None,
        mean: None,
        sum: None,
        true_count: None,
        false_count: None,
    })
}

// 2026-03-21: 这里汇总布尔列指标，目的是为规则判断和二值标签字段提供直接可用的画像。
fn summarize_boolean_column(
    series: &Series,
    column: &str,
    count: usize,
    null_count: usize,
) -> Result<ColumnSummary, SummaryError> {
    let values = series
        .bool()
        .map_err(|error| SummaryError::SummarizeColumn {
            column: column.to_string(),
            message: error.to_string(),
        })?;

    let mut true_count = 0_usize;
    let mut false_count = 0_usize;

    for value in values.into_iter().flatten() {
        if value {
            true_count += 1;
        } else {
            false_count += 1;
        }
    }

    Ok(ColumnSummary {
        column: column.to_string(),
        dtype: dtype_label(series.dtype()).to_string(),
        summary_kind: "boolean".to_string(),
        count,
        null_count,
        // 2026-03-21: 这里让布尔列也返回缺失率，目的是避免上层还要单独推算质量指标。
        missing_rate: calculate_missing_rate(series.len(), null_count),
        distinct_count: None,
        top_values: Vec::new(),
        min_number: None,
        max_number: None,
        mean: None,
        sum: None,
        true_count: Some(true_count),
        false_count: Some(false_count),
    })
}

// 2026-03-21: 这里集中维护 V1 的文本缺失规则，目的是让空白和常见占位值先在摘要层统一口径。
fn is_missing_text(value: &str) -> bool {
    let normalized = value.trim();
    if normalized.is_empty() {
        return true;
    }

    matches!(normalized, "N/A" | "NA" | "null" | "NULL")
}

// 2026-03-21: 这里统一计算缺失率，目的是避免不同列类型各自重复实现比例逻辑并产生口径漂移。
fn calculate_missing_rate(total_count: usize, missing_count: usize) -> Option<f64> {
    if total_count == 0 {
        None
    } else {
        Some(missing_count as f64 / total_count as f64)
    }
}

// 2026-03-21: 这里集中维护 dtype 文案映射，目的是避免直接把 Polars 内部缩写暴露给上层。
fn dtype_label(dtype: &DataType) -> &'static str {
    match dtype {
        DataType::String => "string",
        DataType::Boolean => "boolean",
        DataType::Int64 => "int64",
        DataType::Int32 => "int32",
        DataType::Int16 => "int16",
        DataType::Int8 => "int8",
        DataType::UInt64 => "uint64",
        DataType::UInt32 => "uint32",
        DataType::UInt16 => "uint16",
        DataType::UInt8 => "uint8",
        DataType::Float64 => "float64",
        DataType::Float32 => "float32",
        _ => "unknown",
    }
}

// 2026-03-21: 这里统一判断数值 dtype，目的是让后续扩展更多整型/浮点型时不需要分散修改摘要主流程。
fn is_numeric_dtype(dtype: &DataType) -> bool {
    matches!(
        dtype,
        DataType::Int64
            | DataType::Int32
            | DataType::Int16
            | DataType::Int8
            | DataType::UInt64
            | DataType::UInt32
            | DataType::UInt16
            | DataType::UInt8
            | DataType::Float64
            | DataType::Float32
    )
}
