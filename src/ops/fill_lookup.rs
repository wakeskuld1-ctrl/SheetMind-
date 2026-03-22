use std::collections::{BTreeMap, BTreeSet};

use polars::prelude::{AnyValue, Column, DataFrame, NamedFrom, Series};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::handles::TableHandle;
use crate::frame::loader::LoadedTable;

// 2026-03-23: 这里定义单个回填规则，目的是把“主表哪一列用查值表哪一列补齐”收口成稳定契约。
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct FillLookupRule {
    pub base_column: String,
    pub lookup_column: String,
}

// 2026-03-23: 这里定义 lookup 回填错误，目的是把缺列、重复键和读值失败分层暴露给上层。
#[derive(Debug, Error)]
pub enum FillLookupError {
    #[error("fill_missing_from_lookup 至少需要一条 fills 规则")]
    EmptyFills,
    #[error("fill_missing_from_lookup 缺少 base_on 键列")]
    EmptyBaseKey,
    #[error("fill_missing_from_lookup 缺少 lookup_on 键列")]
    EmptyLookupKey,
    #[error("fill_missing_from_lookup 至少需要一条 base_keys 键列")]
    EmptyBaseKeys,
    #[error("fill_missing_from_lookup 至少需要一条 lookup_keys 键列")]
    EmptyLookupKeys,
    #[error("fill_missing_from_lookup 的 base_keys 与 lookup_keys 键列数量不一致")]
    KeyArityMismatch,
    #[error("fill_missing_from_lookup 找不到{side}列: {column}")]
    MissingColumn { side: &'static str, column: String },
    #[error("fill_missing_from_lookup 存在重复回填目标列: {0}")]
    DuplicateBaseColumn(String),
    #[error("fill_missing_from_lookup 的 lookup 键不唯一: {key_column} = {key}")]
    DuplicateLookupKey { key_column: String, key: String },
    #[error(
        "fill_missing_from_lookup 无法读取{side}列 `{column}` 第 {row_index} 行的值: {message}"
    )]
    ReadValue {
        side: &'static str,
        column: String,
        row_index: usize,
        message: String,
    },
    #[error("fill_missing_from_lookup 无法构建结果表: {0}")]
    BuildFrame(String),
}

// 2026-03-23: 这里执行显式 lookup 回填，目的是把“只补空、不扩表”的主数据补齐能力从 join 场景中独立出来。
pub fn fill_missing_from_lookup(
    base: &LoadedTable,
    lookup: &LoadedTable,
    base_on: &str,
    lookup_on: &str,
    fills: &[FillLookupRule],
) -> Result<LoadedTable, FillLookupError> {
    // 2026-03-23: 这里让旧单键入口转调复合键实现，目的是新增能力时不破坏现有调用协议。
    fill_missing_from_lookup_by_keys(base, lookup, &[base_on], &[lookup_on], fills)
}

// 2026-03-23: 这里新增复合键回填入口，目的是支持“客户ID + 月份”这类主数据补齐场景。
pub fn fill_missing_from_lookup_by_keys(
    base: &LoadedTable,
    lookup: &LoadedTable,
    base_keys: &[&str],
    lookup_keys: &[&str],
    fills: &[FillLookupRule],
) -> Result<LoadedTable, FillLookupError> {
    if fills.is_empty() {
        return Err(FillLookupError::EmptyFills);
    }
    if base_keys.is_empty() {
        return Err(FillLookupError::EmptyBaseKeys);
    }
    if lookup_keys.is_empty() {
        return Err(FillLookupError::EmptyLookupKeys);
    }
    if base_keys.len() != lookup_keys.len() {
        return Err(FillLookupError::KeyArityMismatch);
    }

    ensure_unique_fill_targets(fills)?;
    for base_key in base_keys {
        if base_key.trim().is_empty() {
            return Err(FillLookupError::EmptyBaseKey);
        }
        ensure_column_exists(&base.dataframe, base_key, "base")?;
    }
    for lookup_key in lookup_keys {
        if lookup_key.trim().is_empty() {
            return Err(FillLookupError::EmptyLookupKey);
        }
        ensure_column_exists(&lookup.dataframe, lookup_key, "lookup")?;
    }
    for fill in fills {
        ensure_column_exists(&base.dataframe, &fill.base_column, "base")?;
        ensure_column_exists(&lookup.dataframe, &fill.lookup_column, "lookup")?;
    }

    let lookup_index = build_lookup_index(lookup, lookup_keys, fills)?;
    let mut frame_columns = Vec::<Column>::with_capacity(base.dataframe.width());

    for base_column_name in base.handle.columns() {
        let base_column = base.dataframe.column(base_column_name).map_err(|_| {
            FillLookupError::MissingColumn {
                side: "base",
                column: base_column_name.clone(),
            }
        })?;
        let maybe_fill = fills
            .iter()
            .find(|fill| fill.base_column == *base_column_name);

        match maybe_fill {
            Some(fill) => {
                let values = (0..base.dataframe.height())
                    .map(|row_index| {
                        let current_value = read_optional_string(base_column, row_index, "base")?;
                        if !is_missing(&current_value) {
                            return Ok(current_value);
                        }

                        let Some(key) =
                            read_composite_key(&base.dataframe, base_keys, row_index, "base")?
                        else {
                            return Ok(current_value);
                        };

                        Ok(lookup_index
                            .get(&key)
                            .and_then(|row| row.get(&fill.lookup_column))
                            .cloned()
                            .flatten()
                            .or(current_value))
                    })
                    .collect::<Result<Vec<Option<String>>, FillLookupError>>()?;
                frame_columns.push(Series::new(base_column_name.clone().into(), values).into());
            }
            None => frame_columns.push(base_column.clone()),
        }
    }

    let dataframe = DataFrame::new(frame_columns)
        .map_err(|error| FillLookupError::BuildFrame(error.to_string()))?;
    let handle = TableHandle::new_confirmed(
        base.handle.source_path(),
        base.handle.sheet_name(),
        base.handle.columns().to_vec(),
    );

    Ok(LoadedTable { handle, dataframe })
}

// 2026-03-23: 这里先限制同一目标列只允许被一条规则回填，目的是避免多个 lookup 列竞争覆盖同一列。
fn ensure_unique_fill_targets(fills: &[FillLookupRule]) -> Result<(), FillLookupError> {
    let mut seen = BTreeSet::<String>::new();
    for fill in fills {
        if !seen.insert(fill.base_column.clone()) {
            return Err(FillLookupError::DuplicateBaseColumn(
                fill.base_column.clone(),
            ));
        }
    }
    Ok(())
}

// 2026-03-23: 这里统一校验列存在，目的是在开始逐行回填前先返回更友好的缺列错误。
fn ensure_column_exists(
    dataframe: &DataFrame,
    column: &str,
    side: &'static str,
) -> Result<(), FillLookupError> {
    if dataframe.column(column).is_err() {
        return Err(FillLookupError::MissingColumn {
            side,
            column: column.to_string(),
        });
    }
    Ok(())
}

// 2026-03-23: 这里先把 lookup 表建立成唯一键索引，目的是把回填阶段的每行查找保持为稳定映射而不是重复扫描。
fn build_lookup_index(
    lookup: &LoadedTable,
    lookup_keys: &[&str],
    fills: &[FillLookupRule],
) -> Result<BTreeMap<String, BTreeMap<String, Option<String>>>, FillLookupError> {
    let mut index = BTreeMap::<String, BTreeMap<String, Option<String>>>::new();
    let key_column_name = lookup_keys.join(" + ");

    for row_index in 0..lookup.dataframe.height() {
        let Some(key) = read_composite_key(&lookup.dataframe, lookup_keys, row_index, "lookup")? else {
            continue;
        };
        if index.contains_key(&key) {
            return Err(FillLookupError::DuplicateLookupKey {
                key_column: key_column_name.clone(),
                key,
            });
        }

        let mut row_map = BTreeMap::<String, Option<String>>::new();
        for fill in fills {
            let lookup_column = lookup.dataframe.column(&fill.lookup_column).map_err(|_| {
                FillLookupError::MissingColumn {
                    side: "lookup",
                    column: fill.lookup_column.clone(),
                }
            })?;
            row_map.insert(
                fill.lookup_column.clone(),
                read_optional_string(lookup_column, row_index, "lookup")?,
            );
        }
        index.insert(key, row_map);
    }

    Ok(index)
}

// 2026-03-23: 这里把多列键统一压成稳定复合键，目的是让回填先支持等值复合键而不引入更复杂策略。
fn read_composite_key(
    dataframe: &DataFrame,
    key_columns: &[&str],
    row_index: usize,
    side: &'static str,
) -> Result<Option<String>, FillLookupError> {
    let mut parts = Vec::<String>::with_capacity(key_columns.len());
    for key_column in key_columns {
        let column = dataframe
            .column(key_column)
            .map_err(|_| FillLookupError::MissingColumn {
                side,
                column: (*key_column).to_string(),
            })?;
        let Some(value) = read_optional_string(column, row_index, side)?
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
        else {
            return Ok(None);
        };
        parts.push(value);
    }

    Ok(Some(parts.join("\u{1f}")))
}

// 2026-03-23: 这里统一读取可选字符串，目的是兼容空值、字符串和后续可能出现的数值键列。
fn read_optional_string(
    column: &Column,
    row_index: usize,
    side: &'static str,
) -> Result<Option<String>, FillLookupError> {
    let series = column.as_materialized_series();
    match series.get(row_index) {
        Ok(AnyValue::Null) => Ok(None),
        Ok(_) => series
            .str_value(row_index)
            .map(|value| Some(value.into_owned()))
            .map_err(|error| FillLookupError::ReadValue {
                side,
                column: series.name().to_string(),
                row_index,
                message: error.to_string(),
            }),
        Err(error) => Err(FillLookupError::ReadValue {
            side,
            column: series.name().to_string(),
            row_index,
            message: error.to_string(),
        }),
    }
}

// 2026-03-23: 这里把 null、空串和纯空白统一视为缺失，目的是与 Excel 用户“空着就算没填”的直觉保持一致。
fn is_missing(value: &Option<String>) -> bool {
    value
        .as_ref()
        .map(|item| item.trim().is_empty())
        .unwrap_or(true)
}
