use polars::prelude::{AnyValue, Column, DataFrame, NamedFrom, Series};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::handles::TableHandle;
use crate::frame::loader::LoadedTable;

// 2026-03-22: 这里定义补空策略枚举，目的是把“固定值/补零/前值填补”收口成稳定契约，避免 Skill 自己描述计算细节。
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FillMissingStrategy {
    Constant,
    Zero,
    ForwardFill,
}

// 2026-03-22: 这里定义单列补空规则，目的是让不同列可以显式采用不同缺失值处理策略。
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct FillMissingRule {
    pub column: String,
    pub strategy: FillMissingStrategy,
    #[serde(default)]
    pub value: Option<String>,
}

// 2026-03-22: 这里定义通用补空错误，目的是把缺列、重复规则、无效常量和结果表构建失败分层暴露给上层。
#[derive(Debug, Error)]
pub enum FillMissingError {
    #[error("fill_missing_values 至少需要一条 rules 规则")]
    EmptyRules,
    #[error("fill_missing_values 找不到列: {0}")]
    MissingColumn(String),
    #[error("fill_missing_values 存在重复列规则: {0}")]
    DuplicateRule(String),
    #[error("fill_missing_values 的 constant 策略缺少 value: {0}")]
    MissingConstantValue(String),
    #[error("fill_missing_values 无法读取列`{column}`第{row_index}行的值: {message}")]
    ReadValue {
        column: String,
        row_index: usize,
        message: String,
    },
    #[error("fill_missing_values 无法构建结果表: {0}")]
    BuildFrame(String),
}

// 2026-03-22: 这里执行按列补空，目的是把 Excel 用户最常见的空值处理从 Prompt 描述下沉成可测试的 Rust Tool。
pub fn fill_missing_values(
    loaded: &LoadedTable,
    rules: &[FillMissingRule],
) -> Result<LoadedTable, FillMissingError> {
    if rules.is_empty() {
        return Err(FillMissingError::EmptyRules);
    }

    ensure_unique_rules(rules)?;
    validate_rules(loaded, rules)?;

    let mut frame_columns = Vec::<Column>::with_capacity(loaded.dataframe.width());
    for column_name in loaded.handle.columns() {
        let source_column = loaded
            .dataframe
            .column(column_name)
            .map_err(|_| FillMissingError::MissingColumn(column_name.clone()))?;
        let maybe_rule = rules.iter().find(|rule| rule.column == *column_name);

        match maybe_rule {
            Some(rule) => {
                let values = apply_rule_to_column(source_column, rule, loaded.dataframe.height())?;
                frame_columns.push(Series::new(column_name.clone().into(), values).into());
            }
            None => frame_columns.push(source_column.clone()),
        }
    }

    let dataframe = DataFrame::new(frame_columns)
        .map_err(|error| FillMissingError::BuildFrame(error.to_string()))?;
    let handle = TableHandle::new_confirmed(
        loaded.handle.source_path(),
        loaded.handle.sheet_name(),
        loaded.handle.columns().to_vec(),
    );

    Ok(LoadedTable { handle, dataframe })
}

// 2026-03-22: 这里先限制同一列只允许一条补空规则，目的是保持行为可解释，避免多条规则覆盖顺序不透明。
fn ensure_unique_rules(rules: &[FillMissingRule]) -> Result<(), FillMissingError> {
    let mut seen = std::collections::BTreeSet::<String>::new();
    for rule in rules {
        if !seen.insert(rule.column.clone()) {
            return Err(FillMissingError::DuplicateRule(rule.column.clone()));
        }
    }
    Ok(())
}

// 2026-03-22: 这里集中校验列存在与 constant 参数完整性，目的是在逐行处理前先返回更友好的输入错误。
fn validate_rules(loaded: &LoadedTable, rules: &[FillMissingRule]) -> Result<(), FillMissingError> {
    for rule in rules {
        if loaded.dataframe.column(&rule.column).is_err() {
            return Err(FillMissingError::MissingColumn(rule.column.clone()));
        }
        if matches!(rule.strategy, FillMissingStrategy::Constant) && rule.value.is_none() {
            return Err(FillMissingError::MissingConstantValue(rule.column.clone()));
        }
    }
    Ok(())
}

// 2026-03-22: 这里按单列执行补空，目的是把不同策略的逐行处理逻辑从主流程里拆开，降低后续扩展成本。
fn apply_rule_to_column(
    column: &Column,
    rule: &FillMissingRule,
    row_count: usize,
) -> Result<Vec<Option<String>>, FillMissingError> {
    let mut last_seen = Option::<String>::None;
    let mut output = Vec::<Option<String>>::with_capacity(row_count);

    for row_index in 0..row_count {
        let current = read_optional_string(column, row_index)?;
        let next_value = match rule.strategy {
            FillMissingStrategy::Constant => {
                if is_missing(&current) {
                    rule.value.clone()
                } else {
                    current.clone()
                }
            }
            FillMissingStrategy::Zero => {
                if is_missing(&current) {
                    Some("0".to_string())
                } else {
                    current.clone()
                }
            }
            FillMissingStrategy::ForwardFill => {
                if is_missing(&current) {
                    last_seen.clone().or(current.clone())
                } else {
                    current.clone()
                }
            }
        };

        // 2026-03-22: 这里用补空后的非缺失值推进 last_seen，目的是让 forward_fill 在连续空值段里能稳定沿用最近一次有效值。
        if !is_missing(&next_value) {
            last_seen = next_value.clone();
        }
        output.push(next_value);
    }

    Ok(output)
}

// 2026-03-22: 这里统一把任意列值读取成可选字符串，目的是兼容字符列以及后续可能传入的数值列。
fn read_optional_string(
    column: &Column,
    row_index: usize,
) -> Result<Option<String>, FillMissingError> {
    let series = column.as_materialized_series();
    match series.get(row_index) {
        Ok(AnyValue::Null) => Ok(None),
        Ok(_) => series
            .str_value(row_index)
            .map(|value| Some(value.into_owned()))
            .map_err(|error| FillMissingError::ReadValue {
                column: series.name().to_string(),
                row_index,
                message: error.to_string(),
            }),
        Err(error) => Err(FillMissingError::ReadValue {
            column: series.name().to_string(),
            row_index,
            message: error.to_string(),
        }),
    }
}

// 2026-03-22: 这里把 null、空串和纯空白统一视为缺失，目的是和 Excel 用户“空着就算没填”的直觉保持一致。
fn is_missing(value: &Option<String>) -> bool {
    value
        .as_ref()
        .map(|item| item.trim().is_empty())
        .unwrap_or(true)
}
