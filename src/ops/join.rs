use std::collections::BTreeMap;

use polars::prelude::{Column, DataFrame, NamedFrom, Series};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::handles::TableHandle;
use crate::frame::loader::LoadedTable;
use crate::ops::preview::preview_table;

// 2026-03-22: 这里定义显性关联的保留模式，目的是把技术性 join 语义翻译成业务用户更容易理解的结果保留策略。
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JoinKeepMode {
    // 2026-03-22: 这里只保留两边都能关联上的记录，目的是对应“只看关联成功的数据”。
    MatchedOnly,
    // 2026-03-22: 这里保留左表全部记录，目的是对应“以左表为主，右表信息有则补充”。
    KeepLeft,
    // 2026-03-22: 这里保留右表全部记录，目的是对应“以右表为主，左表信息有则补充”。
    KeepRight,
}

// 2026-03-22: 这里定义显性关联错误，目的是把缺键、缺列、读值失败和结果构建失败分层暴露给上层。
#[derive(Debug, Error)]
pub enum JoinError {
    // 2026-03-22: 这里要求显式提供左表关联列，目的是避免不完整请求进入底层计算。
    #[error("join_tables 缺少左表关联列")]
    EmptyLeftKey,
    // 2026-03-22: 这里要求显式提供右表关联列，目的是避免不完整请求进入底层计算。
    #[error("join_tables 缺少右表关联列")]
    EmptyRightKey,
    // 2026-03-22: 这里单独暴露缺列信息，目的是帮助用户快速修正显性关联配置。
    #[error("{side}找不到列: {column}")]
    MissingColumn { side: &'static str, column: String },
    // 2026-03-22: 这里包装单元格读取失败，目的是避免底层细节直接泄漏到 Tool 层。
    #[error("无法读取{side}列`{column}`的值: {message}")]
    ReadValue {
        side: &'static str,
        column: String,
        message: String,
    },
    // 2026-03-22: 这里包装结果 DataFrame 构建失败，目的是统一返回稳定的业务错误语义。
    #[error("无法构建关联结果表: {0}")]
    BuildFrame(String),
    #[error("无法生成 join 预览: {0}")]
    Preview(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct JoinPreflightPreview {
    pub matched_row_count: usize,
    pub left_unmatched_row_count: usize,
    pub right_unmatched_row_count: usize,
    pub left_duplicate_key_count: usize,
    pub right_duplicate_key_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct JoinPreflightResult {
    pub left_on: String,
    pub right_on: String,
    pub selected_keep_mode: JoinKeepMode,
    pub join_preview: JoinPreflightPreview,
    pub output_row_count_by_keep_mode: BTreeMap<String, usize>,
    #[serde(default)]
    pub result_columns: Vec<String>,
    #[serde(default)]
    pub preview_rows: Vec<BTreeMap<String, String>>,
    #[serde(default)]
    pub risk_summary: Vec<String>,
    pub human_summary: String,
}

// 2026-03-22: 这里执行显性等值关联，目的是把多表分析真正推进到计算闭环，而不只停留在关系提示层。
pub fn join_tables(
    left: &LoadedTable,
    right: &LoadedTable,
    left_on: &str,
    right_on: &str,
    keep_mode: JoinKeepMode,
) -> Result<LoadedTable, JoinError> {
    if left_on.trim().is_empty() {
        return Err(JoinError::EmptyLeftKey);
    }
    if right_on.trim().is_empty() {
        return Err(JoinError::EmptyRightKey);
    }

    ensure_column_exists(&left.dataframe, left_on, "左表")?;
    ensure_column_exists(&right.dataframe, right_on, "右表")?;

    let left_columns = left.handle.columns().to_vec();
    let right_output =
        build_right_output_columns(&left_columns, right.handle.columns(), left_on, right_on);
    let result_columns = build_result_columns(&left_columns, &right_output);
    let right_index = build_right_index(&right.dataframe, right_on)?;
    let mut matched_right_rows = vec![false; right.dataframe.height()];
    let mut merged_rows = Vec::<Vec<Option<String>>>::new();

    for left_row_index in 0..left.dataframe.height() {
        // 2026-03-23: 这里改为读取规范化关联键，目的是让左表查索引时按统一数值语义匹配，而不是直接依赖字符串展示值。
        let left_key = read_join_key(&left.dataframe, left_on, left_row_index, "左表")?;
        match right_index.get(&left_key) {
            Some(right_row_indexes) => {
                for &right_row_index in right_row_indexes {
                    matched_right_rows[right_row_index] = true;
                    merged_rows.push(build_joined_row(
                        left,
                        right,
                        &left_columns,
                        &right_output,
                        Some(left_row_index),
                        Some(right_row_index),
                    )?);
                }
            }
            None if matches!(keep_mode, JoinKeepMode::KeepLeft) => {
                merged_rows.push(build_joined_row(
                    left,
                    right,
                    &left_columns,
                    &right_output,
                    Some(left_row_index),
                    None,
                )?);
            }
            None => {}
        }
    }

    if matches!(keep_mode, JoinKeepMode::KeepRight) {
        for (right_row_index, matched) in matched_right_rows.iter().enumerate() {
            if !matched {
                merged_rows.push(build_joined_row(
                    left,
                    right,
                    &left_columns,
                    &right_output,
                    None,
                    Some(right_row_index),
                )?);
            }
        }
    }

    let dataframe = build_joined_dataframe(&result_columns, &merged_rows)?;
    let handle = TableHandle::new_confirmed(
        format!(
            "{} + {}",
            left.handle.source_path(),
            right.handle.source_path()
        ),
        format!(
            "{} + {}",
            left.handle.sheet_name(),
            right.handle.sheet_name()
        ),
        result_columns,
    );

    Ok(LoadedTable { handle, dataframe })
}

pub fn join_preflight(
    left: &LoadedTable,
    right: &LoadedTable,
    left_on: &str,
    right_on: &str,
    keep_mode: JoinKeepMode,
    limit: usize,
) -> Result<JoinPreflightResult, JoinError> {
    if left_on.trim().is_empty() {
        return Err(JoinError::EmptyLeftKey);
    }
    if right_on.trim().is_empty() {
        return Err(JoinError::EmptyRightKey);
    }

    ensure_column_exists(&left.dataframe, left_on, "左表")?;
    ensure_column_exists(&right.dataframe, right_on, "右表")?;

    let matched_only = join_tables(left, right, left_on, right_on, JoinKeepMode::MatchedOnly)?;
    let keep_left = join_tables(left, right, left_on, right_on, JoinKeepMode::KeepLeft)?;
    let keep_right = join_tables(left, right, left_on, right_on, JoinKeepMode::KeepRight)?;
    let selected_preview = match keep_mode {
        JoinKeepMode::MatchedOnly => &matched_only,
        JoinKeepMode::KeepLeft => &keep_left,
        JoinKeepMode::KeepRight => &keep_right,
    };

    let matched_row_count = matched_only.dataframe.height();
    let left_unmatched_row_count = keep_left
        .dataframe
        .height()
        .saturating_sub(matched_row_count);
    let right_unmatched_row_count = keep_right
        .dataframe
        .height()
        .saturating_sub(matched_row_count);
    let left_duplicate_key_count = duplicate_key_count(&left.dataframe, left_on, "左表")?;
    let right_duplicate_key_count = duplicate_key_count(&right.dataframe, right_on, "右表")?;

    let mut output_row_count_by_keep_mode = BTreeMap::<String, usize>::new();
    output_row_count_by_keep_mode.insert("matched_only".to_string(), matched_row_count);
    output_row_count_by_keep_mode.insert("keep_left".to_string(), keep_left.dataframe.height());
    output_row_count_by_keep_mode.insert("keep_right".to_string(), keep_right.dataframe.height());

    let preview = preview_table(&selected_preview.dataframe, limit)
        .map_err(|error| JoinError::Preview(error.to_string()))?;
    let risk_summary = build_join_preflight_risk_summary(
        left_on,
        right_on,
        left_unmatched_row_count,
        right_unmatched_row_count,
        left_duplicate_key_count,
        right_duplicate_key_count,
        left.dataframe.height(),
        right.dataframe.height(),
    );

    Ok(JoinPreflightResult {
        left_on: left_on.to_string(),
        right_on: right_on.to_string(),
        selected_keep_mode: keep_mode.clone(),
        join_preview: JoinPreflightPreview {
            matched_row_count,
            left_unmatched_row_count,
            right_unmatched_row_count,
            left_duplicate_key_count,
            right_duplicate_key_count,
        },
        output_row_count_by_keep_mode,
        result_columns: selected_preview.handle.columns().to_vec(),
        preview_rows: preview.rows,
        risk_summary: risk_summary.clone(),
        human_summary: build_join_preflight_summary(
            &keep_mode,
            selected_preview.dataframe.height(),
            keep_left.dataframe.height(),
            keep_right.dataframe.height(),
            &risk_summary,
        ),
    })
}

// 2026-03-22: 这里单独校验列存在性，目的是在真正关联前先返回更友好的缺列错误。
fn ensure_column_exists(
    dataframe: &DataFrame,
    column: &str,
    side: &'static str,
) -> Result<(), JoinError> {
    if dataframe.column(column).is_err() {
        return Err(JoinError::MissingColumn {
            side,
            column: column.to_string(),
        });
    }
    Ok(())
}

// 2026-03-22: 这里为右表输出列生成稳定名字，目的是避免同名列覆盖左表原始值。
fn build_right_output_columns(
    left_columns: &[String],
    right_columns: &[String],
    left_on: &str,
    right_on: &str,
) -> Vec<(String, String)> {
    let mut occupied_names = left_columns.to_vec();
    let mut output = Vec::new();

    for right_column in right_columns {
        if right_column == right_on && left_on == right_on {
            continue;
        }

        let mut candidate = right_column.clone();
        while occupied_names.iter().any(|name| name == &candidate) {
            candidate.push_str("_right");
        }

        occupied_names.push(candidate.clone());
        output.push((right_column.clone(), candidate));
    }

    output
}

// 2026-03-22: 这里把左右输出列合并成最终 schema，目的是让结果表在空结果时也保持稳定结构。
fn build_result_columns(left_columns: &[String], right_output: &[(String, String)]) -> Vec<String> {
    let mut columns = left_columns.to_vec();
    columns.extend(
        right_output
            .iter()
            .map(|(_, output_name)| output_name.clone()),
    );
    columns
}

// 2026-03-22: 这里为右表建立关联键索引，目的是避免左表每一行都全表扫描右表。
fn build_right_index(
    dataframe: &DataFrame,
    key_column: &str,
) -> Result<BTreeMap<String, Vec<usize>>, JoinError> {
    let mut index = BTreeMap::<String, Vec<usize>>::new();
    for row_index in 0..dataframe.height() {
        // 2026-03-23: 这里改为先规范化右表关联键，目的是让 1 与 1.0 这类数值等价键先落到同一个索引桶里。
        let key = read_join_key(dataframe, key_column, row_index, "右表")?;
        index.entry(key).or_default().push(row_index);
    }
    Ok(index)
}

// 2026-03-22: 这里构建单行关联结果，目的是把左保留、右保留和匹配成功三类行拼装逻辑统一收口。
fn build_joined_row(
    left: &LoadedTable,
    right: &LoadedTable,
    left_columns: &[String],
    right_output: &[(String, String)],
    left_row_index: Option<usize>,
    right_row_index: Option<usize>,
) -> Result<Vec<Option<String>>, JoinError> {
    let mut row = Vec::with_capacity(left_columns.len() + right_output.len());

    for left_column in left_columns {
        row.push(match left_row_index {
            Some(row_index) => Some(read_cell(&left.dataframe, left_column, row_index, "左表")?),
            None => None,
        });
    }

    for (right_source_column, _) in right_output {
        row.push(match right_row_index {
            Some(row_index) => Some(read_cell(
                &right.dataframe,
                right_source_column,
                row_index,
                "右表",
            )?),
            None => None,
        });
    }

    Ok(row)
}

// 2026-03-22: 这里把行式结果重新组装成 DataFrame，目的是让结果能继续复用 preview、sort、group 等后续能力。
fn build_joined_dataframe(
    columns: &[String],
    rows: &[Vec<Option<String>>],
) -> Result<DataFrame, JoinError> {
    let mut frame_columns = Vec::<Column>::with_capacity(columns.len());

    for (column_index, column_name) in columns.iter().enumerate() {
        let values = rows
            .iter()
            .map(|row| row.get(column_index).cloned().unwrap_or(None))
            .collect::<Vec<Option<String>>>();
        frame_columns.push(Series::new(column_name.clone().into(), values).into());
    }

    DataFrame::new(frame_columns).map_err(|error| JoinError::BuildFrame(error.to_string()))
}

// 2026-03-22: 这里统一读取任意类型列的展示值，目的是让 join 在后续接入数值列后也能继续工作。
fn read_cell(
    dataframe: &DataFrame,
    column: &str,
    row_index: usize,
    side: &'static str,
) -> Result<String, JoinError> {
    read_series(dataframe, column, side)?
        .str_value(row_index)
        .map(|value| value.into_owned())
        .map_err(|error| JoinError::ReadValue {
            side,
            column: column.to_string(),
            message: error.to_string(),
        })
}

// 2026-03-23: 这里把关联键读取单独收口，目的是只在键比较时做最小类型对齐，而不改变结果表的原始展示值。
fn read_join_key(
    dataframe: &DataFrame,
    column: &str,
    row_index: usize,
    side: &'static str,
) -> Result<String, JoinError> {
    let series = read_series(dataframe, column, side)?;
    let display_value = series
        .str_value(row_index)
        .map(|value| value.into_owned())
        .map_err(|error| JoinError::ReadValue {
            side,
            column: column.to_string(),
            message: error.to_string(),
        })?;

    Ok(normalize_join_key(series.dtype(), &display_value))
}

// 2026-03-23: 这里抽出底层 Series 读取，目的是让展示值读取和关联键读取共用同一套缺列错误出口。
fn read_series<'a>(
    dataframe: &'a DataFrame,
    column: &str,
    side: &'static str,
) -> Result<&'a polars::prelude::Series, JoinError> {
    dataframe
        .column(column)
        .map_err(|_| JoinError::MissingColumn {
            side,
            column: column.to_string(),
        })
        .map(|column| column.as_materialized_series())
}

// 2026-03-23: 这里只对浮点型键做规范化，目的是把 1 与 1.0 视为同一数值，同时避免把字符串业务编码强行改写。
fn normalize_join_key(dtype: &polars::prelude::DataType, display_value: &str) -> String {
    let trimmed = display_value.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("null") {
        return String::new();
    }

    match dtype {
        polars::prelude::DataType::Float32 | polars::prelude::DataType::Float64 => trimmed
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite())
            .map(|value| format!("{value}"))
            .unwrap_or_else(|| trimmed.to_string()),
        _ => trimmed.to_string(),
    }
}

fn duplicate_key_count(
    dataframe: &DataFrame,
    key_column: &str,
    side: &'static str,
) -> Result<usize, JoinError> {
    let mut counts = BTreeMap::<String, usize>::new();
    for row_index in 0..dataframe.height() {
        let key = read_join_key(dataframe, key_column, row_index, side)?;
        if key.is_empty() {
            continue;
        }
        *counts.entry(key).or_default() += 1;
    }

    Ok(counts
        .values()
        .map(|count| count.saturating_sub(1))
        .sum::<usize>())
}

fn build_join_preflight_risk_summary(
    left_on: &str,
    right_on: &str,
    left_unmatched_row_count: usize,
    right_unmatched_row_count: usize,
    left_duplicate_key_count: usize,
    right_duplicate_key_count: usize,
    left_row_count: usize,
    right_row_count: usize,
) -> Vec<String> {
    let mut summary = Vec::<String>::new();

    if left_unmatched_row_count > 0 || right_unmatched_row_count > 0 {
        summary.push(format!(
            "按 `{}` -> `{}` 预估仍有 {} / {} 条记录无法匹配。",
            left_on, right_on, left_unmatched_row_count, right_unmatched_row_count
        ));
    }

    if left_duplicate_key_count > 0 || right_duplicate_key_count > 0 {
        summary.push(format!(
            "两边存在重复键（A 表 {} 条、B 表 {} 条），执行后可能扩成多对多结果。",
            left_duplicate_key_count, right_duplicate_key_count
        ));
    }

    if left_row_count > 0 && right_row_count > 0 {
        let left_match_rate =
            1.0 - (left_unmatched_row_count as f64 / left_row_count.max(1) as f64);
        let right_match_rate =
            1.0 - (right_unmatched_row_count as f64 / right_row_count.max(1) as f64);
        if left_match_rate < 0.8 || right_match_rate < 0.8 {
            summary.push(format!(
                "当前匹配覆盖率约为 {:.0}% / {:.0}%，建议先看预览结果再决定是否正式关联。",
                left_match_rate * 100.0,
                right_match_rate * 100.0
            ));
        }
    }

    summary
}

fn build_join_preflight_summary(
    keep_mode: &JoinKeepMode,
    selected_row_count: usize,
    keep_left_row_count: usize,
    keep_right_row_count: usize,
    risk_summary: &[String],
) -> String {
    let mode_label = match keep_mode {
        JoinKeepMode::MatchedOnly => "只保留两边都有的数据",
        JoinKeepMode::KeepLeft => "优先保留 A 表",
        JoinKeepMode::KeepRight => "优先保留 B 表",
    };
    let risk_suffix = if risk_summary.is_empty() {
        "当前没有明显的 join 风险提示。".to_string()
    } else {
        format!("主要风险：{}", risk_summary.join("；"))
    };

    format!(
        "预估{}会得到 {} 行结果；如果优先保留 A 表 / B 表，则大约会得到 {} / {} 行。{}",
        mode_label, selected_row_count, keep_left_row_count, keep_right_row_count, risk_suffix
    )
}
