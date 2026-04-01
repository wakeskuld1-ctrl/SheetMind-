use std::collections::{BTreeMap, BTreeSet};

use polars::prelude::{AnyValue, Column, DataFrame, NamedFrom, Series};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::handles::TableHandle;
use crate::frame::loader::LoadedTable;

// 2026-03-22: 这里定义去重保留策略，目的是把“保留第一条/最后一条”显式收口成稳定参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DistinctKeep {
    First,
    Last,
}

// 2026-03-22: 这里定义去重错误，目的是把缺列、读值失败和结果构建失败分层暴露给上层。
#[derive(Debug, Error)]
pub enum DistinctRowsError {
    #[error("distinct_rows 的 subset 存在重复列: {0}")]
    DuplicateSubsetColumn(String),
    #[error("distinct_rows 找不到列: {0}")]
    MissingColumn(String),
    #[error("distinct_rows 无法读取列`{column}`第{row_index}行的值: {message}")]
    ReadValue {
        column: String,
        row_index: usize,
        message: String,
    },
    #[error("distinct_rows 无法构建结果表: {0}")]
    BuildFrame(String),
}

// 2026-03-22: 这里执行整行/子集列去重，目的是把 Excel 用户高频使用的“删除重复项”沉淀成显式 Tool 能力。
pub fn distinct_rows(
    loaded: &LoadedTable,
    subset: &[&str],
    keep: DistinctKeep,
) -> Result<LoadedTable, DistinctRowsError> {
    let key_columns = resolve_key_columns(loaded, subset)?;
    let row_indexes = collect_distinct_row_indexes(loaded, &key_columns, keep)?;
    let frame_columns = build_frame_columns(loaded, &row_indexes)?;
    let dataframe = DataFrame::new(frame_columns)
        .map_err(|error| DistinctRowsError::BuildFrame(error.to_string()))?;
    let handle = TableHandle::new_confirmed(
        loaded.handle.source_path(),
        loaded.handle.sheet_name(),
        loaded.handle.columns().to_vec(),
    );

    Ok(LoadedTable { handle, dataframe })
}

// 2026-03-22: 这里统一解析去重键列，目的是让“不传 subset 就按整行去重”的默认语义保持清晰。
fn resolve_key_columns(
    loaded: &LoadedTable,
    subset: &[&str],
) -> Result<Vec<String>, DistinctRowsError> {
    if subset.is_empty() {
        return Ok(loaded.handle.columns().to_vec());
    }

    let mut seen = BTreeSet::<String>::new();
    let mut columns = Vec::<String>::with_capacity(subset.len());
    for column in subset {
        if !seen.insert((*column).to_string()) {
            return Err(DistinctRowsError::DuplicateSubsetColumn(
                (*column).to_string(),
            ));
        }
        if loaded.dataframe.column(column).is_err() {
            return Err(DistinctRowsError::MissingColumn((*column).to_string()));
        }
        columns.push((*column).to_string());
    }
    Ok(columns)
}

// 2026-03-22: 这里先只计算保留行号，目的是把“去重口径”与“结果重建”拆开，便于后续扩展更多 keep 策略。
fn collect_distinct_row_indexes(
    loaded: &LoadedTable,
    key_columns: &[String],
    keep: DistinctKeep,
) -> Result<Vec<usize>, DistinctRowsError> {
    let mut keep_by_key = BTreeMap::<String, usize>::new();

    for row_index in 0..loaded.dataframe.height() {
        let signature = build_row_signature(loaded, key_columns, row_index)?;
        match keep {
            DistinctKeep::First => {
                keep_by_key.entry(signature).or_insert(row_index);
            }
            DistinctKeep::Last => {
                keep_by_key.insert(signature, row_index);
            }
        }
    }

    let mut row_indexes = keep_by_key.into_values().collect::<Vec<_>>();
    row_indexes.sort_unstable();
    Ok(row_indexes)
}

// 2026-03-22: 这里把指定行号重建成结果表，目的是避免引入更重的 DataFrame 取行依赖，同时保持结果顺序稳定。
fn build_frame_columns(
    loaded: &LoadedTable,
    row_indexes: &[usize],
) -> Result<Vec<Column>, DistinctRowsError> {
    let mut frame_columns = Vec::<Column>::with_capacity(loaded.dataframe.width());
    for column_name in loaded.handle.columns() {
        let source_column = loaded
            .dataframe
            .column(column_name)
            .map_err(|_| DistinctRowsError::MissingColumn(column_name.clone()))?;
        let values = row_indexes
            .iter()
            .map(|row_index| read_optional_string(source_column, *row_index))
            .collect::<Result<Vec<Option<String>>, DistinctRowsError>>()?;
        frame_columns.push(Series::new(column_name.clone().into(), values).into());
    }
    Ok(frame_columns)
}

// 2026-03-22: 这里按键列构造行签名，目的是让整行去重与子集列去重都落在同一套判重逻辑上。
fn build_row_signature(
    loaded: &LoadedTable,
    key_columns: &[String],
    row_index: usize,
) -> Result<String, DistinctRowsError> {
    let mut values = Vec::<Option<String>>::with_capacity(key_columns.len());
    for column_name in key_columns {
        let source_column = loaded
            .dataframe
            .column(column_name)
            .map_err(|_| DistinctRowsError::MissingColumn(column_name.clone()))?;
        values.push(read_optional_string(source_column, row_index)?);
    }
    Ok(serde_json::to_string(&values).unwrap_or_default())
}

// 2026-03-22: 这里统一把任意列值读取成可选字符串，目的是兼容字符列和后续可能出现的数值列去重场景。
fn read_optional_string(
    column: &Column,
    row_index: usize,
) -> Result<Option<String>, DistinctRowsError> {
    let series = column.as_materialized_series();
    match series.get(row_index) {
        Ok(AnyValue::Null) => Ok(None),
        Ok(_) => series
            .str_value(row_index)
            .map(|value| Some(value.into_owned()))
            .map_err(|error| DistinctRowsError::ReadValue {
                column: series.name().to_string(),
                row_index,
                message: error.to_string(),
            }),
        Err(error) => Err(DistinctRowsError::ReadValue {
            column: series.name().to_string(),
            row_index,
            message: error.to_string(),
        }),
    }
}
