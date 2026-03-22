use std::collections::{BTreeMap, BTreeSet};

use polars::prelude::{AnyValue, Column, DataFrame, DataType, NamedFrom, Series};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::handles::TableHandle;
use crate::frame::loader::LoadedTable;

// 2026-03-23: 这里定义第一版透视聚合方式，原因是先覆盖 Excel 用户最常见的 sum / count / mean 心智模型。
// 2026-03-23: 这里保留精简枚举，目的是让后续 CLI 与 skill 链路在不扩散复杂度的前提下稳定复用。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PivotAggregation {
    Sum,
    Count,
    Mean,
}

// 2026-03-23: 这里定义透视错误，原因是把列缺失、类型不匹配与构表失败清晰暴露给上层。
// 2026-03-23: 这里继续保留中文错误文案，目的是让 CLI 直接面向业务用户时也可读。
#[derive(Debug, Error)]
pub enum PivotError {
    #[error("pivot_table 至少需要一列 rows")]
    EmptyRows,
    #[error("pivot_table 至少需要一列 columns")]
    EmptyColumns,
    #[error("pivot_table 至少需要一列 values")]
    EmptyValues,
    #[error("pivot_table 第一版仅支持单个 values 列")]
    MultipleValuesUnsupported,
    #[error("pivot_table 找不到列: {0}")]
    MissingColumn(String),
    #[error("pivot_table 的 `{column}` 不是数值列，无法执行 {aggregation:?}")]
    NonNumericValueColumn {
        column: String,
        aggregation: PivotAggregation,
    },
    #[error("pivot_table 无法读取 `{column}` 第 {row_index} 行的值: {message}")]
    ReadValue {
        column: String,
        row_index: usize,
        message: String,
    },
    #[error("pivot_table 无法构建结果表: {0}")]
    BuildFrame(String),
}

// 2026-03-23: 这里执行最小透视能力，原因是把 Excel 用户熟悉的宽表分析正式下沉到 Rust Tool 层。
// 2026-03-23: 这里将聚合结果直接保留为真实数值列，目的是后续 preview/export 都能复用同一份类型信息。
pub fn pivot_table(
    loaded: &LoadedTable,
    rows: &[&str],
    columns: &[&str],
    values: &[&str],
    aggregation: PivotAggregation,
) -> Result<LoadedTable, PivotError> {
    if rows.is_empty() {
        return Err(PivotError::EmptyRows);
    }
    if columns.is_empty() {
        return Err(PivotError::EmptyColumns);
    }
    if values.is_empty() {
        return Err(PivotError::EmptyValues);
    }
    if values.len() > 1 {
        return Err(PivotError::MultipleValuesUnsupported);
    }

    for column in rows {
        ensure_column_exists(&loaded.dataframe, column)?;
    }
    for column in columns {
        ensure_column_exists(&loaded.dataframe, column)?;
    }

    let value_column_name = values[0];
    let value_column = loaded
        .dataframe
        .column(value_column_name)
        .map_err(|_| PivotError::MissingColumn(value_column_name.to_string()))?;
    if matches!(aggregation, PivotAggregation::Sum | PivotAggregation::Mean)
        && !is_numeric_dtype(value_column.dtype())
    {
        return Err(PivotError::NonNumericValueColumn {
            column: value_column_name.to_string(),
            aggregation,
        });
    }

    let mut pivot_headers = BTreeSet::<String>::new();
    let mut aggregated = BTreeMap::<Vec<String>, BTreeMap<String, PivotAccumulator>>::new();

    for row_index in 0..loaded.dataframe.height() {
        let row_key = rows
            .iter()
            .map(|column| read_string_cell(&loaded.dataframe, column, row_index))
            .collect::<Result<Vec<String>, PivotError>>()?;
        let pivot_key = columns
            .iter()
            .map(|column| read_string_cell(&loaded.dataframe, column, row_index))
            .collect::<Result<Vec<String>, PivotError>>()?
            .join(" | ");
        pivot_headers.insert(pivot_key.clone());

        let entry = aggregated
            .entry(row_key)
            .or_default()
            .entry(pivot_key)
            .or_insert_with(PivotAccumulator::default);
        entry.consume(value_column, row_index, aggregation)?;
    }

    let pivot_headers = pivot_headers.into_iter().collect::<Vec<_>>();
    let row_keys = aggregated.keys().cloned().collect::<Vec<_>>();
    let mut frame_columns = Vec::<Column>::with_capacity(rows.len() + pivot_headers.len());

    for (dimension_index, row_name) in rows.iter().enumerate() {
        let values = row_keys
            .iter()
            .map(|key| key.get(dimension_index).cloned().unwrap_or_default())
            .collect::<Vec<String>>();
        frame_columns.push(Series::new((*row_name).into(), values).into());
    }

    for pivot_header in &pivot_headers {
        let typed_column = match aggregation {
            // 2026-03-23: 这里把 count 列保留为整型，原因是 Excel 后续统计时不应再把计数结果当文本。
            // 2026-03-23: 这里缺失交叉格继续保留为 None，目的是让 preview 为空、导出后为空白单元格。
            PivotAggregation::Count => Series::new(
                pivot_header.clone().into(),
                row_keys
                    .iter()
                    .map(|row_key| {
                        aggregated
                            .get(row_key)
                            .and_then(|cells| cells.get(pivot_header))
                            .map(PivotAccumulator::count_value)
                    })
                    .collect::<Vec<Option<i64>>>(),
            ),
            // 2026-03-23: 这里把 sum / mean 列保留为浮点型，原因是导出 Excel 时必须生成可继续求和/透视的真实数值单元格。
            // 2026-03-23: 这里对没有任何有效数值的交叉格返回 None，目的是避免把“没有数据”误写成 0。
            PivotAggregation::Sum | PivotAggregation::Mean => Series::new(
                pivot_header.clone().into(),
                row_keys
                    .iter()
                    .map(|row_key| {
                        aggregated
                            .get(row_key)
                            .and_then(|cells| cells.get(pivot_header))
                            .and_then(|cell| cell.numeric_value(aggregation))
                    })
                    .collect::<Vec<Option<f64>>>(),
            ),
        };
        frame_columns.push(typed_column.into());
    }

    let dataframe =
        DataFrame::new(frame_columns).map_err(|error| PivotError::BuildFrame(error.to_string()))?;
    let mut output_columns = rows.iter().map(|item| (*item).to_string()).collect::<Vec<_>>();
    output_columns.extend(pivot_headers.iter().cloned());
    let handle = TableHandle::new_confirmed(
        loaded.handle.source_path(),
        loaded.handle.sheet_name(),
        output_columns,
    );

    Ok(LoadedTable { handle, dataframe })
}

// 2026-03-23: 这里统一校验列存在，原因是在开始透视前先把缺列问题明确返回。
fn ensure_column_exists(dataframe: &DataFrame, column: &str) -> Result<(), PivotError> {
    if dataframe.column(column).is_err() {
        return Err(PivotError::MissingColumn(column.to_string()));
    }
    Ok(())
}

// 2026-03-23: 这里统一把维度列读成字符串，原因是让透视后的行键与列键保持稳定、可排序、可导出。
fn read_string_cell(
    dataframe: &DataFrame,
    column: &str,
    row_index: usize,
) -> Result<String, PivotError> {
    let series = dataframe
        .column(column)
        .map_err(|_| PivotError::MissingColumn(column.to_string()))?
        .as_materialized_series();
    match series.get(row_index) {
        Ok(AnyValue::Null) => Ok(String::new()),
        Ok(_) => series
            .str_value(row_index)
            .map(|value| value.into_owned())
            .map_err(|error| PivotError::ReadValue {
                column: column.to_string(),
                row_index,
                message: error.to_string(),
            }),
        Err(error) => Err(PivotError::ReadValue {
            column: column.to_string(),
            row_index,
            message: error.to_string(),
        }),
    }
}

// 2026-03-23: 这里判断数值类型，原因是在 sum / mean 前明确拦住文本列透视。
fn is_numeric_dtype(dtype: &DataType) -> bool {
    matches!(
        dtype,
        DataType::Int8
            | DataType::Int16
            | DataType::Int32
            | DataType::Int64
            | DataType::UInt8
            | DataType::UInt16
            | DataType::UInt32
            | DataType::UInt64
            | DataType::Float32
            | DataType::Float64
    )
}

// 2026-03-23: 这里用一个最小累加器承接 sum / count / mean，原因是避免把透视逻辑拆散到多套重复循环里。
// 2026-03-23: 这里额外记录 has_value，目的是区分“交叉格不存在”与“交叉格存在但数值之和刚好为 0”。
#[derive(Debug, Clone, Default)]
struct PivotAccumulator {
    sum: f64,
    count: i64,
    has_value: bool,
}

impl PivotAccumulator {
    // 2026-03-23: 这里逐行吸收单元格值，原因是把不同聚合方式统一映射到同一份中间状态。
    // 2026-03-23: 这里对 null 采取“跳过而不是写 0”，目的是保证空值不会污染 sum / mean 的业务含义。
    fn consume(
        &mut self,
        value_column: &Column,
        row_index: usize,
        aggregation: PivotAggregation,
    ) -> Result<(), PivotError> {
        match aggregation {
            PivotAggregation::Count => {
                let series = value_column.as_materialized_series();
                if !matches!(series.get(row_index), Ok(AnyValue::Null)) {
                    self.count += 1;
                    self.has_value = true;
                }
                Ok(())
            }
            PivotAggregation::Sum | PivotAggregation::Mean => {
                if let Some(numeric_value) = read_numeric_cell(value_column, row_index)? {
                    self.sum += numeric_value;
                    self.count += 1;
                    self.has_value = true;
                }
                Ok(())
            }
        }
    }

    // 2026-03-23: 这里把 count 结果显式转成 Option<i64>，原因是空交叉格必须保留为空，而不是伪造 0。
    fn count_value(&self) -> i64 {
        self.count
    }

    // 2026-03-23: 这里统一输出数值聚合结果，原因是把 sum / mean 的空值判定收敛到一个出口。
    // 2026-03-23: 这里对没有有效值的 mean 返回 None，目的是避免出现误导性的 0 均值。
    fn numeric_value(&self, aggregation: PivotAggregation) -> Option<f64> {
        if !self.has_value {
            return None;
        }

        match aggregation {
            PivotAggregation::Count => None,
            PivotAggregation::Sum => Some(self.sum),
            PivotAggregation::Mean => Some(self.sum / self.count as f64),
        }
    }
}

// 2026-03-23: 这里把数值列统一读成 Option<f64>，原因是让 sum / mean 共用一套聚合路径。
// 2026-03-23: 这里对 null 返回 None，目的是从源头保留“空值就是空值”的语义。
fn read_numeric_cell(column: &Column, row_index: usize) -> Result<Option<f64>, PivotError> {
    let series = column.as_materialized_series();
    match series.get(row_index) {
        Ok(AnyValue::Int8(value)) => Ok(Some(value as f64)),
        Ok(AnyValue::Int16(value)) => Ok(Some(value as f64)),
        Ok(AnyValue::Int32(value)) => Ok(Some(value as f64)),
        Ok(AnyValue::Int64(value)) => Ok(Some(value as f64)),
        Ok(AnyValue::UInt8(value)) => Ok(Some(value as f64)),
        Ok(AnyValue::UInt16(value)) => Ok(Some(value as f64)),
        Ok(AnyValue::UInt32(value)) => Ok(Some(value as f64)),
        Ok(AnyValue::UInt64(value)) => Ok(Some(value as f64)),
        Ok(AnyValue::Float32(value)) => Ok(Some(value as f64)),
        Ok(AnyValue::Float64(value)) => Ok(Some(value)),
        Ok(AnyValue::Null) => Ok(None),
        Ok(other) => Err(PivotError::ReadValue {
            column: series.name().to_string(),
            row_index,
            message: format!("非数值类型: {other:?}"),
        }),
        Err(error) => Err(PivotError::ReadValue {
            column: series.name().to_string(),
            row_index,
            message: error.to_string(),
        }),
    }
}
