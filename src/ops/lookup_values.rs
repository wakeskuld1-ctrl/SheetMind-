use std::collections::{BTreeMap, BTreeSet};

use polars::prelude::{AnyValue, Column, DataFrame, NamedFrom, Series};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::handles::TableHandle;
use crate::frame::loader::LoadedTable;

// 2026-03-23: 这里定义单个查值输出规则，目的是把“从 lookup 表取哪一列并输出成什么列名”沉淀成稳定契约。
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct LookupSelect {
    pub lookup_column: String,
    pub output_column: String,
}

// 2026-03-23: 这里定义轻量查值错误，目的是把缺列、重复 key、输出列冲突和读值失败分层暴露给上层。
#[derive(Debug, Error)]
pub enum LookupValuesError {
    #[error("lookup_values 至少需要一条 selects 规则")]
    EmptySelects,
    #[error("lookup_values 缺少 base_on 键列")]
    EmptyBaseKey,
    #[error("lookup_values 缺少 lookup_on 键列")]
    EmptyLookupKey,
    #[error("lookup_values 至少需要一条 base_keys 键列")]
    EmptyBaseKeys,
    #[error("lookup_values 至少需要一条 lookup_keys 键列")]
    EmptyLookupKeys,
    #[error("lookup_values 的 base_keys 与 lookup_keys 键列数量不一致")]
    KeyArityMismatch,
    #[error("lookup_values 找不到{side}列: {column}")]
    MissingColumn { side: &'static str, column: String },
    #[error("lookup_values 存在重复输出列: {0}")]
    DuplicateOutputColumn(String),
    #[error("lookup_values 输出列与主表现有列冲突: {0}")]
    OutputColumnConflict(String),
    #[error("lookup_values 的 lookup 键不唯一: {key_column} = {key}")]
    DuplicateLookupKey { key_column: String, key: String },
    #[error("lookup_values 无法读取{side}列`{column}`第{row_index}行的值: {message}")]
    ReadValue {
        side: &'static str,
        column: String,
        row_index: usize,
        message: String,
    },
    #[error("lookup_values 无法构建结果表: {0}")]
    BuildFrame(String),
}

// 2026-03-23: 这里执行轻量查值带列，目的是提供贴近 Excel VLOOKUP/XLOOKUP 心智、但不暴露 join 术语的基础 Tool。
pub fn lookup_values(
    base: &LoadedTable,
    lookup: &LoadedTable,
    base_on: &str,
    lookup_on: &str,
    selects: &[LookupSelect],
) -> Result<LoadedTable, LookupValuesError> {
    // 2026-03-23: 这里让旧单键入口转调复合键实现，目的是新增能力时不破坏现有 Skill / Tool 契约。
    lookup_values_by_keys(base, lookup, &[base_on], &[lookup_on], selects)
}

// 2026-03-23: 这里新增复合键查值入口，目的是支持“客户ID + 月份”这类真实业务场景，而不引入 join 语义。
pub fn lookup_values_by_keys(
    base: &LoadedTable,
    lookup: &LoadedTable,
    base_keys: &[&str],
    lookup_keys: &[&str],
    selects: &[LookupSelect],
) -> Result<LoadedTable, LookupValuesError> {
    if selects.is_empty() {
        return Err(LookupValuesError::EmptySelects);
    }
    if base_keys.is_empty() {
        return Err(LookupValuesError::EmptyBaseKeys);
    }
    if lookup_keys.is_empty() {
        return Err(LookupValuesError::EmptyLookupKeys);
    }
    if base_keys.len() != lookup_keys.len() {
        return Err(LookupValuesError::KeyArityMismatch);
    }

    ensure_unique_output_columns(selects)?;
    for base_key in base_keys {
        if base_key.trim().is_empty() {
            return Err(LookupValuesError::EmptyBaseKey);
        }
        ensure_column_exists(&base.dataframe, base_key, "base")?;
    }
    for lookup_key in lookup_keys {
        if lookup_key.trim().is_empty() {
            return Err(LookupValuesError::EmptyLookupKey);
        }
        ensure_column_exists(&lookup.dataframe, lookup_key, "lookup")?;
    }
    for select in selects {
        ensure_column_exists(&lookup.dataframe, &select.lookup_column, "lookup")?;
        ensure_output_column_is_available(base.handle.columns(), &select.output_column)?;
    }

    let lookup_index = build_lookup_index(lookup, lookup_keys, selects)?;
    let mut frame_columns = Vec::<Column>::with_capacity(base.dataframe.width() + selects.len());

    for base_column_name in base.handle.columns() {
        let base_column = base.dataframe.column(base_column_name).map_err(|_| {
            LookupValuesError::MissingColumn {
                side: "base",
                column: base_column_name.clone(),
            }
        })?;
        frame_columns.push(base_column.clone());
    }

    for select in selects {
        let values = (0..base.dataframe.height())
            .map(|row_index| {
                let matched_value =
                    read_composite_key(&base.dataframe, base_keys, row_index, "base")?
                        .and_then(|key| lookup_index.get(&key))
                        .and_then(|row| row.get(&select.lookup_column))
                        .cloned()
                        .flatten()
                        .unwrap_or_default();
                Ok(Some(matched_value))
            })
            .collect::<Result<Vec<Option<String>>, LookupValuesError>>()?;
        frame_columns.push(Series::new(select.output_column.clone().into(), values).into());
    }

    let dataframe = DataFrame::new(frame_columns)
        .map_err(|error| LookupValuesError::BuildFrame(error.to_string()))?;
    let mut output_columns = base.handle.columns().to_vec();
    output_columns.extend(selects.iter().map(|select| select.output_column.clone()));
    let handle = TableHandle::new_confirmed(
        base.handle.source_path(),
        base.handle.sheet_name(),
        output_columns,
    );

    Ok(LoadedTable { handle, dataframe })
}

// 2026-03-23: 这里先限制输出列名唯一，目的是避免多个 lookup 列竞争写入同一目标列导致结果不可解释。
fn ensure_unique_output_columns(selects: &[LookupSelect]) -> Result<(), LookupValuesError> {
    let mut seen = BTreeSet::<String>::new();
    for select in selects {
        if !seen.insert(select.output_column.clone()) {
            return Err(LookupValuesError::DuplicateOutputColumn(
                select.output_column.clone(),
            ));
        }
    }
    Ok(())
}

// 2026-03-23: 这里统一校验输入列存在，目的是在逐行查值前先返回更友好的缺列错误。
fn ensure_column_exists(
    dataframe: &DataFrame,
    column: &str,
    side: &'static str,
) -> Result<(), LookupValuesError> {
    if dataframe.column(column).is_err() {
        return Err(LookupValuesError::MissingColumn {
            side,
            column: column.to_string(),
        });
    }
    Ok(())
}

// 2026-03-23: 这里先挡住与主表现有列的冲突，目的是防止 lookup_values 静默覆盖主表字段而破坏“轻量带列”语义。
fn ensure_output_column_is_available(
    base_columns: &[String],
    output_column: &str,
) -> Result<(), LookupValuesError> {
    if base_columns.iter().any(|column| column == output_column) {
        return Err(LookupValuesError::OutputColumnConflict(
            output_column.to_string(),
        ));
    }
    Ok(())
}

// 2026-03-23: 这里先把 lookup 表构造成唯一键字典，目的是让主表逐行带值保持稳定 O(1) 查找。
fn build_lookup_index(
    lookup: &LoadedTable,
    lookup_keys: &[&str],
    selects: &[LookupSelect],
) -> Result<BTreeMap<String, BTreeMap<String, Option<String>>>, LookupValuesError> {
    let mut index = BTreeMap::<String, BTreeMap<String, Option<String>>>::new();
    let key_column_name = lookup_keys.join(" + ");

    for row_index in 0..lookup.dataframe.height() {
        let Some(key) = read_composite_key(&lookup.dataframe, lookup_keys, row_index, "lookup")?
        else {
            continue;
        };
        if index.contains_key(&key) {
            return Err(LookupValuesError::DuplicateLookupKey {
                key_column: key_column_name.clone(),
                key,
            });
        }

        let mut row_map = BTreeMap::<String, Option<String>>::new();
        for select in selects {
            let lookup_column = lookup
                .dataframe
                .column(&select.lookup_column)
                .map_err(|_| LookupValuesError::MissingColumn {
                    side: "lookup",
                    column: select.lookup_column.clone(),
                })?;
            row_map.insert(
                select.lookup_column.clone(),
                read_optional_string(lookup_column, row_index, "lookup")?,
            );
        }
        index.insert(key, row_map);
    }

    Ok(index)
}

// 2026-03-23: 这里把多列键统一压成稳定复合键，目的是让 lookup/fill 先支持等值复合键而不把 dispatcher 变复杂。
fn read_composite_key(
    dataframe: &DataFrame,
    key_columns: &[&str],
    row_index: usize,
    side: &'static str,
) -> Result<Option<String>, LookupValuesError> {
    let mut parts = Vec::<String>::with_capacity(key_columns.len());
    for key_column in key_columns {
        let column =
            dataframe
                .column(key_column)
                .map_err(|_| LookupValuesError::MissingColumn {
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

// 2026-03-23: 这里统一把任意列值读取成可选字符串，目的是兼容文本列与后续可能作为 key 的数值列。
fn read_optional_string(
    column: &Column,
    row_index: usize,
    side: &'static str,
) -> Result<Option<String>, LookupValuesError> {
    let series = column.as_materialized_series();
    match series.get(row_index) {
        Ok(AnyValue::Null) => Ok(None),
        Ok(_) => series
            .str_value(row_index)
            .map(|value| Some(value.into_owned()))
            .map_err(|error| LookupValuesError::ReadValue {
                side,
                column: series.name().to_string(),
                row_index,
                message: error.to_string(),
            }),
        Err(error) => Err(LookupValuesError::ReadValue {
            side,
            column: series.name().to_string(),
            row_index,
            message: error.to_string(),
        }),
    }
}
