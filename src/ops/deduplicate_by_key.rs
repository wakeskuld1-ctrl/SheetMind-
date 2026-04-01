use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use polars::prelude::{AnyValue, Column, DataFrame, NamedFrom, Series};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::handles::TableHandle;
use crate::frame::loader::LoadedTable;

// 2026-03-22: 这里定义按业务键去重时的保留策略，目的是把“保留首条 / 末条”的语义显式收口为稳定参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeduplicateKeep {
    First,
    Last,
}

// 2026-03-22: 这里定义排序方向，目的是让“保留最新 / 最大 / 最小”这类业务语义可以先通过显式排序表达。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderDirection {
    Asc,
    Desc,
}

// 2026-03-22: 这里定义单个排序规则，目的是把按哪个列、什么方向排序收敛成可复用的结构。
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct OrderSpec {
    pub column: String,
    pub direction: OrderDirection,
}

// 2026-03-22: 这里定义主键去重错误，目的是把 keys、order_by、读值和结果构建失败分层暴露给上层。
#[derive(Debug, Error)]
pub enum DeduplicateByKeyError {
    #[error("deduplicate_by_key 至少需要一个 keys 列")]
    EmptyKeys,
    #[error("deduplicate_by_key 的 keys 存在重复列: {0}")]
    DuplicateKeyColumn(String),
    #[error("deduplicate_by_key 的 order_by 存在重复列: {0}")]
    DuplicateOrderColumn(String),
    #[error("deduplicate_by_key 找不到列: {0}")]
    MissingColumn(String),
    #[error("deduplicate_by_key 无法读取列 `{column}` 第 {row_index} 行的值: {message}")]
    ReadValue {
        column: String,
        row_index: usize,
        message: String,
    },
    #[error("deduplicate_by_key 无法构建结果表: {0}")]
    BuildFrame(String),
}

// 2026-03-22: 这里执行按业务键去重，目的是把“先按规则排序、再按 key 保留一条”的真实 Excel 清洗需求沉淀成独立 Tool 底座。
pub fn deduplicate_by_key(
    loaded: &LoadedTable,
    keys: &[&str],
    order_by: &[OrderSpec],
    keep: DeduplicateKeep,
) -> Result<LoadedTable, DeduplicateByKeyError> {
    let key_columns = resolve_key_columns(loaded, keys)?;
    validate_order_columns(loaded, order_by)?;

    let sorted_row_indexes = sorted_row_indexes(loaded, order_by)?;
    let kept_row_indexes = select_kept_rows(loaded, &key_columns, &sorted_row_indexes, keep)?;
    let frame_columns = build_frame_columns(loaded, &kept_row_indexes)?;
    let dataframe = DataFrame::new(frame_columns)
        .map_err(|error| DeduplicateByKeyError::BuildFrame(error.to_string()))?;
    let handle = TableHandle::new_confirmed(
        loaded.handle.source_path(),
        loaded.handle.sheet_name(),
        loaded.handle.columns().to_vec(),
    );

    Ok(LoadedTable { handle, dataframe })
}

// 2026-03-22: 这里先校验 key 列集合，目的是避免“空 keys”或重复 keys 这种二义性配置进入真实计算路径。
fn resolve_key_columns(
    loaded: &LoadedTable,
    keys: &[&str],
) -> Result<Vec<String>, DeduplicateByKeyError> {
    if keys.is_empty() {
        return Err(DeduplicateByKeyError::EmptyKeys);
    }

    let mut seen = BTreeSet::<String>::new();
    let mut resolved = Vec::<String>::with_capacity(keys.len());
    for key in keys {
        if !seen.insert((*key).to_string()) {
            return Err(DeduplicateByKeyError::DuplicateKeyColumn(
                (*key).to_string(),
            ));
        }
        if loaded.dataframe.column(key).is_err() {
            return Err(DeduplicateByKeyError::MissingColumn((*key).to_string()));
        }
        resolved.push((*key).to_string());
    }

    Ok(resolved)
}

// 2026-03-22: 这里单独校验排序列，目的是把“保留最新记录”依赖的排序口径提前锁住，而不是等到底层排序时报模糊错误。
fn validate_order_columns(
    loaded: &LoadedTable,
    order_by: &[OrderSpec],
) -> Result<(), DeduplicateByKeyError> {
    let mut seen = BTreeSet::<String>::new();
    for spec in order_by {
        if !seen.insert(spec.column.clone()) {
            return Err(DeduplicateByKeyError::DuplicateOrderColumn(
                spec.column.clone(),
            ));
        }
        if loaded.dataframe.column(&spec.column).is_err() {
            return Err(DeduplicateByKeyError::MissingColumn(spec.column.clone()));
        }
    }
    Ok(())
}

// 2026-03-22: 这里先把行号按 order_by 排好，目的是避免在去重主循环里反复比较多列值，保持实现简单稳定。
fn sorted_row_indexes(
    loaded: &LoadedTable,
    order_by: &[OrderSpec],
) -> Result<Vec<usize>, DeduplicateByKeyError> {
    let mut row_indexes = (0..loaded.dataframe.height()).collect::<Vec<_>>();
    if order_by.is_empty() {
        return Ok(row_indexes);
    }

    row_indexes.sort_by(|left, right| compare_rows(loaded, order_by, *left, *right));
    Ok(row_indexes)
}

// 2026-03-22: 这里按排序后的行序决定每个 key 保留哪一条，目的是把排序语义与 key 聚合语义清晰拆开。
fn select_kept_rows(
    loaded: &LoadedTable,
    key_columns: &[String],
    sorted_row_indexes: &[usize],
    keep: DeduplicateKeep,
) -> Result<Vec<usize>, DeduplicateByKeyError> {
    let mut keep_by_key = BTreeMap::<String, usize>::new();

    for row_index in sorted_row_indexes {
        let signature = build_row_signature(loaded, key_columns, *row_index)?;
        match keep {
            DeduplicateKeep::First => {
                keep_by_key.entry(signature).or_insert(*row_index);
            }
            DeduplicateKeep::Last => {
                keep_by_key.insert(signature, *row_index);
            }
        }
    }

    let mut row_indexes = keep_by_key.into_values().collect::<Vec<_>>();
    row_indexes.sort_unstable();
    Ok(row_indexes)
}

// 2026-03-22: 这里把保留的行号重建成结果表，目的是沿用当前项目“显式重建结果 DataFrame”的一致风格。
fn build_frame_columns(
    loaded: &LoadedTable,
    row_indexes: &[usize],
) -> Result<Vec<Column>, DeduplicateByKeyError> {
    let mut frame_columns = Vec::<Column>::with_capacity(loaded.dataframe.width());
    for column_name in loaded.handle.columns() {
        let source_column = loaded
            .dataframe
            .column(column_name)
            .map_err(|_| DeduplicateByKeyError::MissingColumn(column_name.clone()))?;
        let values = row_indexes
            .iter()
            .map(|row_index| read_optional_string(source_column, *row_index))
            .collect::<Result<Vec<Option<String>>, DeduplicateByKeyError>>()?;
        frame_columns.push(Series::new(column_name.clone().into(), values).into());
    }
    Ok(frame_columns)
}

// 2026-03-22: 这里对两行做多列比较，目的是让 deduplicate_by_key 不依赖额外 DataFrame 排序特性也能稳定工作。
fn compare_rows(
    loaded: &LoadedTable,
    order_by: &[OrderSpec],
    left_row_index: usize,
    right_row_index: usize,
) -> Ordering {
    for spec in order_by {
        let Ok(column) = loaded.dataframe.column(&spec.column) else {
            continue;
        };
        let left_value = read_optional_string(column, left_row_index).unwrap_or(None);
        let right_value = read_optional_string(column, right_row_index).unwrap_or(None);
        let mut ordering = compare_optional_values(left_value.as_ref(), right_value.as_ref());
        if matches!(spec.direction, OrderDirection::Desc) {
            ordering = ordering.reverse();
        }
        if ordering != Ordering::Equal {
            return ordering;
        }
    }

    left_row_index.cmp(&right_row_index)
}

// 2026-03-22: 这里统一定义空值和非空值的排序关系，目的是让排序缺失值时也保持可解释、可重复的结果。
fn compare_optional_values(left: Option<&String>, right: Option<&String>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.cmp(right),
        (None, Some(_)) => Ordering::Less,
        (Some(_), None) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

// 2026-03-22: 这里按 key 列构造行签名，目的是让业务键去重和保留策略都能落在统一的键空间上。
fn build_row_signature(
    loaded: &LoadedTable,
    key_columns: &[String],
    row_index: usize,
) -> Result<String, DeduplicateByKeyError> {
    let mut values = Vec::<Option<String>>::with_capacity(key_columns.len());
    for column_name in key_columns {
        let source_column = loaded
            .dataframe
            .column(column_name)
            .map_err(|_| DeduplicateByKeyError::MissingColumn(column_name.clone()))?;
        values.push(read_optional_string(source_column, row_index)?);
    }
    Ok(serde_json::to_string(&values).unwrap_or_default())
}

// 2026-03-22: 这里统一把任意列值读取成可选字符串，目的是让 key 列和排序列都能兼容文本与数值场景。
fn read_optional_string(
    column: &Column,
    row_index: usize,
) -> Result<Option<String>, DeduplicateByKeyError> {
    let series = column.as_materialized_series();
    match series.get(row_index) {
        Ok(AnyValue::Null) => Ok(None),
        Ok(_) => series
            .str_value(row_index)
            .map(|value| Some(value.into_owned()))
            .map_err(|error| DeduplicateByKeyError::ReadValue {
                column: series.name().to_string(),
                row_index,
                message: error.to_string(),
            }),
        Err(error) => Err(DeduplicateByKeyError::ReadValue {
            column: series.name().to_string(),
            row_index,
            message: error.to_string(),
        }),
    }
}
