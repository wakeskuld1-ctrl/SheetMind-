use polars::prelude::{AnyValue, Column, DataFrame, NamedFrom, Series};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::handles::TableHandle;
use crate::frame::loader::LoadedTable;

// 2026-03-23: 这里定义文本替换对，目的是把“原值 -> 新值”的替换规则沉淀成稳定的结构化输入，便于 Skill 只传参数不掺实现细节。
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ReplacePair {
    pub from: String,
    pub to: String,
}

// 2026-03-23: 这里定义单列文本标准化规则，目的是把 join / lookup 前常见的清洗动作收口成可组合原子能力。
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct NormalizeTextRule {
    pub column: String,
    #[serde(default)]
    pub trim: bool,
    #[serde(default)]
    pub collapse_whitespace: bool,
    #[serde(default)]
    pub lowercase: bool,
    #[serde(default)]
    pub uppercase: bool,
    #[serde(default)]
    pub remove_chars: Vec<String>,
    #[serde(default)]
    pub replace_pairs: Vec<ReplacePair>,
}

// 2026-03-23: 这里定义文本标准化错误，目的是把缺列、重复规则和结果表构建失败分层暴露给上层。
#[derive(Debug, Error)]
pub enum NormalizeTextError {
    #[error("normalize_text_columns 至少需要一条 rules 规则")]
    EmptyRules,
    #[error("normalize_text_columns 找不到列: {0}")]
    MissingColumn(String),
    #[error("normalize_text_columns 存在重复列规则: {0}")]
    DuplicateRule(String),
    #[error("normalize_text_columns 无法读取列 `{column}` 第 {row_index} 行的值: {message}")]
    ReadValue {
        column: String,
        row_index: usize,
        message: String,
    },
    #[error("normalize_text_columns 无法构建结果表: {0}")]
    BuildFrame(String),
}

// 2026-03-23: 这里执行按列文本标准化，目的是把文本清洗从 Skill 侧前移到可测试、可复用的 Rust 计算层。
pub fn normalize_text_columns(
    loaded: &LoadedTable,
    rules: &[NormalizeTextRule],
) -> Result<LoadedTable, NormalizeTextError> {
    if rules.is_empty() {
        return Err(NormalizeTextError::EmptyRules);
    }

    ensure_unique_rules(rules)?;
    for rule in rules {
        ensure_column_exists(&loaded.dataframe, &rule.column)?;
    }

    let mut frame_columns = Vec::<Column>::with_capacity(loaded.dataframe.width());
    for column_name in loaded.handle.columns() {
        let source_column = loaded
            .dataframe
            .column(column_name)
            .map_err(|_| NormalizeTextError::MissingColumn(column_name.clone()))?;
        let maybe_rule = rules.iter().find(|rule| rule.column == *column_name);

        match maybe_rule {
            Some(rule) => {
                let values = (0..loaded.dataframe.height())
                    .map(|row_index| {
                        read_optional_string(source_column, row_index)
                            .map(|value| value.map(|item| apply_rule(item, rule)))
                    })
                    .collect::<Result<Vec<Option<String>>, NormalizeTextError>>()?;
                frame_columns.push(Series::new(column_name.clone().into(), values).into());
            }
            None => frame_columns.push(source_column.clone()),
        }
    }

    let dataframe = DataFrame::new(frame_columns)
        .map_err(|error| NormalizeTextError::BuildFrame(error.to_string()))?;
    let handle = TableHandle::new_confirmed(
        loaded.handle.source_path(),
        loaded.handle.sheet_name(),
        loaded.handle.columns().to_vec(),
    );

    Ok(LoadedTable { handle, dataframe })
}

// 2026-03-23: 这里先做重复规则校验，目的是避免同一列的多份配置在执行时出现不可解释的覆盖顺序。
fn ensure_unique_rules(rules: &[NormalizeTextRule]) -> Result<(), NormalizeTextError> {
    let mut seen = std::collections::BTreeSet::<String>::new();
    for rule in rules {
        if !seen.insert(rule.column.clone()) {
            return Err(NormalizeTextError::DuplicateRule(rule.column.clone()));
        }
    }
    Ok(())
}

// 2026-03-23: 这里统一校验目标列存在，目的是在真正逐行清洗前先返回更友好的缺列错误。
fn ensure_column_exists(dataframe: &DataFrame, column: &str) -> Result<(), NormalizeTextError> {
    if dataframe.column(column).is_err() {
        return Err(NormalizeTextError::MissingColumn(column.to_string()));
    }
    Ok(())
}

// 2026-03-23: 这里统一把任意列值读成可选字符串，目的是兼容字符串列与后续可能出现的数值列文本清洗场景。
fn read_optional_string(
    column: &Column,
    row_index: usize,
) -> Result<Option<String>, NormalizeTextError> {
    let series = column.as_materialized_series();
    match series.get(row_index) {
        Ok(AnyValue::Null) => Ok(None),
        Ok(_) => series
            .str_value(row_index)
            .map(|value| Some(value.into_owned()))
            .map_err(|error| NormalizeTextError::ReadValue {
                column: series.name().to_string(),
                row_index,
                message: error.to_string(),
            }),
        Err(error) => Err(NormalizeTextError::ReadValue {
            column: series.name().to_string(),
            row_index,
            message: error.to_string(),
        }),
    }
}

// 2026-03-23: 这里把字符串按固定顺序应用规则，目的是让清洗行为稳定、可预测，避免同样输入在不同轮执行结果漂移。
fn apply_rule(mut value: String, rule: &NormalizeTextRule) -> String {
    if rule.trim {
        value = value.trim().to_string();
    }
    if rule.collapse_whitespace {
        value = value.split_whitespace().collect::<Vec<_>>().join(" ");
    }
    if rule.lowercase {
        value = value.to_lowercase();
    }
    if rule.uppercase {
        value = value.to_uppercase();
    }
    for item in &rule.remove_chars {
        value = value.replace(item, "");
    }
    for pair in &rule.replace_pairs {
        value = value.replace(&pair.from, &pair.to);
    }
    value
}
