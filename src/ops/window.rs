use polars::prelude::{AnyValue, DataFrame, DataType, NamedFrom, Series, SortMultipleOptions};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::handles::TableHandle;
use crate::frame::loader::LoadedTable;

const WINDOW_ROW_INDEX_COLUMN: &str = "__window_row_index";

// 2026-03-23: 这里定义窗口排序规格，原因是窗口函数必须先稳定表达“按哪些列、什么方向”排序；目的是把 Skill 层的自然语言意图收口成确定计算顺序。
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct WindowOrderSpec {
    pub column: String,
    #[serde(default)]
    pub descending: bool,
}

// 2026-03-23: 这里扩展第一版窗口函数枚举，原因是分析建模层已经需要 shift、percent_rank 和 rolling；目的是在不引入复杂 DSL 的前提下覆盖高频窗口能力。
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowCalculationKind {
    RowNumber,
    Rank,
    CumulativeSum,
    Lag,
    Lead,
    PercentRank,
    RollingSum,
    RollingMean,
}

// 2026-03-23: 这里为单个窗口计算补充 offset/window_size，原因是 lag/lead 与 rolling 都需要最小参数；目的是继续沿用统一结构化请求而不是额外再造一个子协议。
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct WindowCalculation {
    pub kind: WindowCalculationKind,
    #[serde(default)]
    pub source_column: Option<String>,
    pub output_column: String,
    #[serde(default)]
    pub offset: Option<usize>,
    #[serde(default)]
    pub window_size: Option<usize>,
}

// 2026-03-23: 这里扩展窗口错误类型，原因是新增 shift/rolling 后“缺 source、缺窗口大小、参数为 0”都需要显式拦截；目的是让上层 Skill 收到稳定、可解释的中文报错。
#[derive(Debug, Error)]
pub enum WindowCalculationError {
    #[error("window_calculation 至少需要一条 order_by 规则")]
    EmptyOrderBy,
    #[error("window_calculation 至少需要一条 calculations 规则")]
    EmptyCalculations,
    #[error("window_calculation 找不到列: {0}")]
    MissingColumn(String),
    #[error("window_calculation 存在重复输出列: {0}")]
    DuplicateOutputColumn(String),
    #[error("window_calculation 输出列与原表列冲突: {0}")]
    OutputColumnConflict(String),
    #[error("window_calculation 的 {kind} 缺少 source_column")]
    MissingSourceColumn { kind: &'static str },
    #[error("window_calculation 的 {kind} 需要 window_size")]
    MissingWindowSize { kind: &'static str },
    #[error("window_calculation 的 {kind} offset 必须大于 0，当前为 {offset}")]
    InvalidOffset { kind: &'static str, offset: usize },
    #[error("window_calculation 的 {kind} window_size 必须大于 0，当前为 {window_size}")]
    InvalidWindowSize {
        kind: &'static str,
        window_size: usize,
    },
    #[error("window_calculation 的 `{column}` 不是数值列，无法执行 {kind}")]
    NonNumericSource { column: String, kind: &'static str },
    #[error("window_calculation 无法读取列 `{column}` 第 {row_index} 行的值: {message}")]
    ReadValue {
        column: String,
        row_index: usize,
        message: String,
    },
    #[error("window_calculation 无法完成排序: {0}")]
    SortFrame(String),
    #[error("window_calculation 无法构建结果表: {0}")]
    BuildFrame(String),
}

// 2026-03-23: 这里执行窗口计算主入口，原因是表处理层到分析建模层的桥接需要统一承载排序、分区和结果回填；目的是让上层 Tool 只管声明计算，不关心 Polars 细节。
pub fn window_calculation(
    loaded: &LoadedTable,
    partition_by: &[&str],
    order_by: &[WindowOrderSpec],
    calculations: &[WindowCalculation],
) -> Result<LoadedTable, WindowCalculationError> {
    if order_by.is_empty() {
        return Err(WindowCalculationError::EmptyOrderBy);
    }
    if calculations.is_empty() {
        return Err(WindowCalculationError::EmptyCalculations);
    }

    for column in partition_by {
        ensure_column_exists(&loaded.dataframe, column)?;
    }
    for order in order_by {
        ensure_column_exists(&loaded.dataframe, &order.column)?;
    }
    ensure_unique_output_columns(loaded.handle.columns(), calculations)?;
    validate_calculations(&loaded.dataframe, calculations)?;

    let sorted = build_sorted_window_frame(loaded, partition_by, order_by)?;
    let mut frame_columns = loaded.dataframe.get_columns().to_vec();
    let mut output_columns = loaded.handle.columns().to_vec();

    for calculation in calculations {
        let values = compute_window_values(&sorted, partition_by, order_by, calculation)?;
        frame_columns.push(Series::new(calculation.output_column.clone().into(), values).into());
        output_columns.push(calculation.output_column.clone());
    }

    let dataframe = DataFrame::new(frame_columns)
        .map_err(|error| WindowCalculationError::BuildFrame(error.to_string()))?;
    let handle = TableHandle::new_confirmed(
        loaded.handle.source_path(),
        loaded.handle.sheet_name(),
        output_columns,
    );

    Ok(LoadedTable { handle, dataframe })
}

// 2026-03-23: 这里统一校验输入列存在性，原因是缺列时尽早失败比在扫描中途报错更好定位；目的是给 Skill 层一个稳定的前置门禁。
fn ensure_column_exists(dataframe: &DataFrame, column: &str) -> Result<(), WindowCalculationError> {
    if dataframe.column(column).is_err() {
        return Err(WindowCalculationError::MissingColumn(column.to_string()));
    }
    Ok(())
}

// 2026-03-23: 这里先拦截输出列冲突，原因是窗口结果列只允许显式新增；目的是避免调用链里悄悄覆盖原始业务列。
fn ensure_unique_output_columns(
    base_columns: &[String],
    calculations: &[WindowCalculation],
) -> Result<(), WindowCalculationError> {
    let mut seen = std::collections::BTreeSet::<String>::new();
    for calculation in calculations {
        if base_columns
            .iter()
            .any(|column| column == &calculation.output_column)
        {
            return Err(WindowCalculationError::OutputColumnConflict(
                calculation.output_column.clone(),
            ));
        }
        if !seen.insert(calculation.output_column.clone()) {
            return Err(WindowCalculationError::DuplicateOutputColumn(
                calculation.output_column.clone(),
            ));
        }
    }
    Ok(())
}

// 2026-03-23: 这里集中做 calculation 级参数校验，原因是新增多种窗口函数后各自门禁不同；目的是把“缺 source / 缺窗口 / 非数值列”统一收口在执行前。
fn validate_calculations(
    dataframe: &DataFrame,
    calculations: &[WindowCalculation],
) -> Result<(), WindowCalculationError> {
    for calculation in calculations {
        match calculation.kind {
            WindowCalculationKind::RowNumber
            | WindowCalculationKind::Rank
            | WindowCalculationKind::PercentRank => {}
            WindowCalculationKind::CumulativeSum => {
                validate_numeric_source(dataframe, calculation, "cumulative_sum")?;
            }
            WindowCalculationKind::Lag => {
                validate_shift_source(dataframe, calculation, "lag")?;
            }
            WindowCalculationKind::Lead => {
                validate_shift_source(dataframe, calculation, "lead")?;
            }
            WindowCalculationKind::RollingSum => {
                validate_numeric_rolling_source(dataframe, calculation, "rolling_sum")?;
            }
            WindowCalculationKind::RollingMean => {
                validate_numeric_rolling_source(dataframe, calculation, "rolling_mean")?;
            }
        }
    }
    Ok(())
}

// 2026-03-23: 这里校验 shift 类函数的 source/offset，原因是 lag/lead 没有 source 就无法工作；目的是把错误前移到参数阶段而不是中途扫描阶段。
fn validate_shift_source(
    dataframe: &DataFrame,
    calculation: &WindowCalculation,
    kind: &'static str,
) -> Result<(), WindowCalculationError> {
    let source_column = calculation
        .source_column
        .as_deref()
        .ok_or(WindowCalculationError::MissingSourceColumn { kind })?;
    ensure_column_exists(dataframe, source_column)?;
    if let Some(offset) = calculation.offset {
        if offset == 0 {
            return Err(WindowCalculationError::InvalidOffset { kind, offset });
        }
    }
    Ok(())
}

// 2026-03-23: 这里校验数值 source，原因是累计和滚动统计都必须建立在数值列之上；目的是让窗口计算与后续建模共享一致的数据质量门禁。
fn validate_numeric_source(
    dataframe: &DataFrame,
    calculation: &WindowCalculation,
    kind: &'static str,
) -> Result<(), WindowCalculationError> {
    let source_column = calculation
        .source_column
        .as_deref()
        .ok_or(WindowCalculationError::MissingSourceColumn { kind })?;
    let column = dataframe
        .column(source_column)
        .map_err(|_| WindowCalculationError::MissingColumn(source_column.to_string()))?;
    if !is_numeric_dtype(column.dtype()) {
        return Err(WindowCalculationError::NonNumericSource {
            column: source_column.to_string(),
            kind,
        });
    }
    Ok(())
}

// 2026-03-23: 这里在数值门禁之外继续校验 window_size，原因是 rolling 必须知道窗口宽度；目的是避免 window_size=0 这类语义含混的输入进入执行阶段。
fn validate_numeric_rolling_source(
    dataframe: &DataFrame,
    calculation: &WindowCalculation,
    kind: &'static str,
) -> Result<(), WindowCalculationError> {
    validate_numeric_source(dataframe, calculation, kind)?;
    match calculation.window_size {
        Some(window_size) if window_size > 0 => Ok(()),
        Some(window_size) => Err(WindowCalculationError::InvalidWindowSize { kind, window_size }),
        None => Err(WindowCalculationError::MissingWindowSize { kind }),
    }
}

// 2026-03-23: 这里先把原表排序成“窗口视图”，原因是所有窗口函数都要按统一顺序计算；目的是计算完成后仍能借助原行索引回填到用户熟悉的原表位置。
fn build_sorted_window_frame(
    loaded: &LoadedTable,
    partition_by: &[&str],
    order_by: &[WindowOrderSpec],
) -> Result<DataFrame, WindowCalculationError> {
    let mut sortable = loaded.dataframe.clone();
    let row_indexes = (0..loaded.dataframe.height() as u32).collect::<Vec<u32>>();
    sortable
        .with_column(Series::new(WINDOW_ROW_INDEX_COLUMN.into(), row_indexes))
        .map_err(|error| WindowCalculationError::BuildFrame(error.to_string()))?;

    let mut sort_columns = partition_by
        .iter()
        .map(|item| (*item).to_string())
        .collect::<Vec<_>>();
    sort_columns.extend(order_by.iter().map(|item| item.column.clone()));
    let mut descending = vec![false; partition_by.len()];
    descending.extend(order_by.iter().map(|item| item.descending));

    sortable
        .sort(
            sort_columns.iter().map(String::as_str).collect::<Vec<_>>(),
            SortMultipleOptions::default()
                .with_order_descending_multi(descending)
                .with_maintain_order(true),
        )
        .map_err(|error| WindowCalculationError::SortFrame(error.to_string()))
}

// 2026-03-23: 这里按分区切片计算单个窗口指标，原因是 lag/lead 与 rolling 都需要看到同分区前后文；目的是用最小实现覆盖多个窗口函数而不引入额外执行框架。
fn compute_window_values(
    sorted: &DataFrame,
    partition_by: &[&str],
    order_by: &[WindowOrderSpec],
    calculation: &WindowCalculation,
) -> Result<Vec<String>, WindowCalculationError> {
    let mut values = vec![String::new(); sorted.height()];
    let mut partition_start = 0usize;

    while partition_start < sorted.height() {
        let partition_key = read_key(sorted, partition_by, partition_start)?;
        let mut partition_end = partition_start + 1;
        while partition_end < sorted.height()
            && read_key(sorted, partition_by, partition_end)? == partition_key
        {
            partition_end += 1;
        }

        apply_partition_calculation(
            sorted,
            order_by,
            calculation,
            partition_start,
            partition_end,
            &mut values,
        )?;
        partition_start = partition_end;
    }

    Ok(values)
}

// 2026-03-23: 这里把单个分区的窗口计算拆到独立函数，原因是不同 kind 的状态机差异已经变大；目的是让后续继续补窗口能力时只改局部而不污染主循环。
fn apply_partition_calculation(
    sorted: &DataFrame,
    order_by: &[WindowOrderSpec],
    calculation: &WindowCalculation,
    partition_start: usize,
    partition_end: usize,
    values: &mut [String],
) -> Result<(), WindowCalculationError> {
    let partition_len = partition_end - partition_start;
    let original_indexes = (partition_start..partition_end)
        .map(|row_index| read_row_index(sorted, row_index))
        .collect::<Result<Vec<_>, _>>()?;

    match calculation.kind {
        WindowCalculationKind::RowNumber => {
            for (offset, original_index) in original_indexes.iter().enumerate() {
                values[*original_index] = (offset + 1).to_string();
            }
        }
        WindowCalculationKind::Rank => {
            let dense_ranks = dense_rank_values(sorted, order_by, partition_start, partition_end)?;
            for (offset, original_index) in original_indexes.iter().enumerate() {
                values[*original_index] = dense_ranks[offset].to_string();
            }
        }
        WindowCalculationKind::CumulativeSum => {
            let source_column = calculation.source_column.as_deref().ok_or(
                WindowCalculationError::MissingSourceColumn {
                    kind: "cumulative_sum",
                },
            )?;
            let numeric_values =
                read_numeric_partition(sorted, source_column, partition_start, partition_end)?;
            let mut running_sum = 0.0_f64;
            for (offset, original_index) in original_indexes.iter().enumerate() {
                running_sum += numeric_values[offset];
                values[*original_index] = format_numeric(running_sum);
            }
        }
        WindowCalculationKind::Lag => {
            let source_column = calculation
                .source_column
                .as_deref()
                .ok_or(WindowCalculationError::MissingSourceColumn { kind: "lag" })?;
            let source_values =
                read_string_partition(sorted, source_column, partition_start, partition_end)?;
            let offset = calculation.offset.unwrap_or(1);
            for (index, original_index) in original_indexes.iter().enumerate() {
                values[*original_index] = index
                    .checked_sub(offset)
                    .and_then(|target| source_values.get(target).cloned())
                    .unwrap_or_default();
            }
        }
        WindowCalculationKind::Lead => {
            let source_column = calculation
                .source_column
                .as_deref()
                .ok_or(WindowCalculationError::MissingSourceColumn { kind: "lead" })?;
            let source_values =
                read_string_partition(sorted, source_column, partition_start, partition_end)?;
            let offset = calculation.offset.unwrap_or(1);
            for (index, original_index) in original_indexes.iter().enumerate() {
                values[*original_index] = source_values
                    .get(index + offset)
                    .cloned()
                    .unwrap_or_default();
            }
        }
        WindowCalculationKind::PercentRank => {
            let standard_ranks =
                standard_rank_values(sorted, order_by, partition_start, partition_end)?;
            for (offset, original_index) in original_indexes.iter().enumerate() {
                let rendered = if partition_len <= 1 {
                    "0".to_string()
                } else {
                    let percent_rank = (standard_ranks[offset] - 1) as f64
                        / (partition_len.saturating_sub(1)) as f64;
                    format_numeric(percent_rank)
                };
                values[*original_index] = rendered;
            }
        }
        WindowCalculationKind::RollingSum => {
            let source_column = calculation.source_column.as_deref().ok_or(
                WindowCalculationError::MissingSourceColumn {
                    kind: "rolling_sum",
                },
            )?;
            let window_size =
                calculation
                    .window_size
                    .ok_or(WindowCalculationError::MissingWindowSize {
                        kind: "rolling_sum",
                    })?;
            let numeric_values =
                read_numeric_partition(sorted, source_column, partition_start, partition_end)?;
            let prefix_sums = build_prefix_sums(&numeric_values);
            for (offset, original_index) in original_indexes.iter().enumerate() {
                let window_start = (offset + 1).saturating_sub(window_size);
                let sum = prefix_sums[offset + 1] - prefix_sums[window_start];
                values[*original_index] = format_numeric(sum);
            }
        }
        WindowCalculationKind::RollingMean => {
            let source_column = calculation.source_column.as_deref().ok_or(
                WindowCalculationError::MissingSourceColumn {
                    kind: "rolling_mean",
                },
            )?;
            let window_size =
                calculation
                    .window_size
                    .ok_or(WindowCalculationError::MissingWindowSize {
                        kind: "rolling_mean",
                    })?;
            let numeric_values =
                read_numeric_partition(sorted, source_column, partition_start, partition_end)?;
            let prefix_sums = build_prefix_sums(&numeric_values);
            for (offset, original_index) in original_indexes.iter().enumerate() {
                let window_start = (offset + 1).saturating_sub(window_size);
                let sum = prefix_sums[offset + 1] - prefix_sums[window_start];
                let count = offset + 1 - window_start;
                values[*original_index] = format_numeric(sum / count as f64);
            }
        }
    }

    Ok(())
}

// 2026-03-23: 这里单独计算 dense rank，原因是现有 rank 已被测试锁定为 dense rank；目的是在扩展 percent_rank 时继续保持旧行为不回归。
fn dense_rank_values(
    sorted: &DataFrame,
    order_by: &[WindowOrderSpec],
    partition_start: usize,
    partition_end: usize,
) -> Result<Vec<i64>, WindowCalculationError> {
    let mut ranks = Vec::<i64>::with_capacity(partition_end - partition_start);
    let mut current_rank = 0_i64;
    let mut previous_order_key: Option<Vec<String>> = None;

    for row_index in partition_start..partition_end {
        let order_key = read_order_key(sorted, order_by, row_index)?;
        if previous_order_key.as_ref() != Some(&order_key) {
            current_rank += 1;
            previous_order_key = Some(order_key);
        }
        ranks.push(current_rank);
    }

    Ok(ranks)
}

// 2026-03-23: 这里单独计算标准 rank，原因是 percent_rank 需要“并列占同名次、下一名跳位”的标准定义；目的是不影响现有 dense rank 输出的兼容性。
fn standard_rank_values(
    sorted: &DataFrame,
    order_by: &[WindowOrderSpec],
    partition_start: usize,
    partition_end: usize,
) -> Result<Vec<i64>, WindowCalculationError> {
    let mut ranks = Vec::<i64>::with_capacity(partition_end - partition_start);
    let mut current_rank = 1_i64;
    let mut previous_order_key: Option<Vec<String>> = None;

    for (offset, row_index) in (partition_start..partition_end).enumerate() {
        let order_key = read_order_key(sorted, order_by, row_index)?;
        if previous_order_key.as_ref() != Some(&order_key) {
            current_rank = offset as i64 + 1;
            previous_order_key = Some(order_key);
        }
        ranks.push(current_rank);
    }

    Ok(ranks)
}

// 2026-03-23: 这里把分区内字符串源列先抽成向量，原因是 lag/lead 需要随机访问前后行；目的是避免在每次偏移时重复读列。
fn read_string_partition(
    dataframe: &DataFrame,
    column: &str,
    partition_start: usize,
    partition_end: usize,
) -> Result<Vec<String>, WindowCalculationError> {
    (partition_start..partition_end)
        .map(|row_index| read_string_cell(dataframe, column, row_index))
        .collect()
}

// 2026-03-23: 这里把分区内数值源列先抽成向量，原因是 cumulative/rolling 都要反复访问数值；目的是把读取错误集中到一个入口并便于后续前缀和复用。
fn read_numeric_partition(
    dataframe: &DataFrame,
    column: &str,
    partition_start: usize,
    partition_end: usize,
) -> Result<Vec<f64>, WindowCalculationError> {
    (partition_start..partition_end)
        .map(|row_index| read_numeric_cell(dataframe, column, row_index))
        .collect()
}

// 2026-03-23: 这里构造前缀和，原因是 rolling_sum/rolling_mean 直接用前缀和比每行重复累加更稳；目的是在保持实现简单的同时避免不必要的重复计算。
fn build_prefix_sums(values: &[f64]) -> Vec<f64> {
    let mut prefix_sums = Vec::<f64>::with_capacity(values.len() + 1);
    prefix_sums.push(0.0);
    for value in values {
        let next = prefix_sums.last().copied().unwrap_or_default() + value;
        prefix_sums.push(next);
    }
    prefix_sums
}

// 2026-03-23: 这里读取分区键，原因是窗口分区切换必须依赖统一键读取；目的是让“无分区”和“多列分区”都走同一套实现。
fn read_key(
    dataframe: &DataFrame,
    columns: &[&str],
    row_index: usize,
) -> Result<Vec<String>, WindowCalculationError> {
    columns
        .iter()
        .map(|column| read_string_cell(dataframe, column, row_index))
        .collect()
}

// 2026-03-23: 这里读取排序键，原因是 dense rank 与 standard rank 都依赖稳定的 order_by 组合键；目的是让并列判断和排序来源完全一致。
fn read_order_key(
    dataframe: &DataFrame,
    order_by: &[WindowOrderSpec],
    row_index: usize,
) -> Result<Vec<String>, WindowCalculationError> {
    order_by
        .iter()
        .map(|order| read_string_cell(dataframe, &order.column, row_index))
        .collect()
}

// 2026-03-23: 这里读取原始行索引，原因是窗口视图排序后必须准确回填；目的是保证最终结果仍然对齐用户原始表顺序。
fn read_row_index(
    dataframe: &DataFrame,
    row_index: usize,
) -> Result<usize, WindowCalculationError> {
    let series = dataframe
        .column(WINDOW_ROW_INDEX_COLUMN)
        .map_err(|_| WindowCalculationError::MissingColumn(WINDOW_ROW_INDEX_COLUMN.to_string()))?
        .as_materialized_series();
    match series.get(row_index) {
        Ok(AnyValue::UInt32(value)) => Ok(value as usize),
        Ok(AnyValue::UInt64(value)) => Ok(value as usize),
        Ok(AnyValue::Int32(value)) if value >= 0 => Ok(value as usize),
        Ok(AnyValue::Int64(value)) if value >= 0 => Ok(value as usize),
        Ok(other) => Err(WindowCalculationError::ReadValue {
            column: WINDOW_ROW_INDEX_COLUMN.to_string(),
            row_index,
            message: format!("无法识别的行索引类型: {other:?}"),
        }),
        Err(error) => Err(WindowCalculationError::ReadValue {
            column: WINDOW_ROW_INDEX_COLUMN.to_string(),
            row_index,
            message: error.to_string(),
        }),
    }
}

// 2026-03-23: 这里统一把单元格读成字符串，原因是分区键、排序键和 shift 输出都需要文本化表达；目的是让不同列类型在窗口层共享同一读取口径。
fn read_string_cell(
    dataframe: &DataFrame,
    column: &str,
    row_index: usize,
) -> Result<String, WindowCalculationError> {
    let series = dataframe
        .column(column)
        .map_err(|_| WindowCalculationError::MissingColumn(column.to_string()))?
        .as_materialized_series();
    match series.get(row_index) {
        Ok(AnyValue::Null) => Ok(String::new()),
        Ok(_) => series
            .str_value(row_index)
            .map(|value| value.into_owned())
            .map_err(|error| WindowCalculationError::ReadValue {
                column: column.to_string(),
                row_index,
                message: error.to_string(),
            }),
        Err(error) => Err(WindowCalculationError::ReadValue {
            column: column.to_string(),
            row_index,
            message: error.to_string(),
        }),
    }
}

// 2026-03-23: 这里统一读取数值单元格，原因是累计与滚动统计都依赖稳定数值口径；目的是把空值归零、类型判断和错误信息集中在一个入口。
fn read_numeric_cell(
    dataframe: &DataFrame,
    column: &str,
    row_index: usize,
) -> Result<f64, WindowCalculationError> {
    let series = dataframe
        .column(column)
        .map_err(|_| WindowCalculationError::MissingColumn(column.to_string()))?
        .as_materialized_series();
    match series.get(row_index) {
        Ok(AnyValue::Int8(value)) => Ok(value as f64),
        Ok(AnyValue::Int16(value)) => Ok(value as f64),
        Ok(AnyValue::Int32(value)) => Ok(value as f64),
        Ok(AnyValue::Int64(value)) => Ok(value as f64),
        Ok(AnyValue::UInt8(value)) => Ok(value as f64),
        Ok(AnyValue::UInt16(value)) => Ok(value as f64),
        Ok(AnyValue::UInt32(value)) => Ok(value as f64),
        Ok(AnyValue::UInt64(value)) => Ok(value as f64),
        Ok(AnyValue::Float32(value)) => Ok(value as f64),
        Ok(AnyValue::Float64(value)) => Ok(value),
        Ok(AnyValue::Null) => Ok(0.0),
        Ok(other) => Err(WindowCalculationError::ReadValue {
            column: column.to_string(),
            row_index,
            message: format!("非数值类型: {other:?}"),
        }),
        Err(error) => Err(WindowCalculationError::ReadValue {
            column: column.to_string(),
            row_index,
            message: error.to_string(),
        }),
    }
}

// 2026-03-23: 这里统一判断数值列类型，原因是 cumulative/rolling 在执行前要有一致门禁；目的是防止文本列误入分析路径。
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

// 2026-03-23: 这里统一格式化数值输出，原因是窗口计算既可能产出整数也可能产出小数；目的是让结果预览更贴近 Excel 用户直觉并避免多余尾零。
fn format_numeric(value: f64) -> String {
    if (value.fract()).abs() < f64::EPSILON {
        format!("{}", value as i64)
    } else {
        let mut rendered = format!("{value:.6}");
        while rendered.contains('.') && rendered.ends_with('0') {
            rendered.pop();
        }
        if rendered.ends_with('.') {
            rendered.pop();
        }
        rendered
    }
}
