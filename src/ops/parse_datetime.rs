use chrono::{Datelike, Duration, NaiveDate};
use polars::prelude::{AnyValue, Column, DataFrame, NamedFrom, Series};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::handles::TableHandle;
use crate::frame::loader::LoadedTable;
use crate::ops::semantic::{ParsedDate, ParsedTime, parse_date_value, parse_time_value};

// 2026-03-23: 这里定义日期时间标准化目标类型，目的是把“转日期”与“转日期时间”沉淀成稳定契约，便于 Skill 只传参数不关心实现。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DateTimeTargetType {
    Date,
    DateTime,
}

impl DateTimeTargetType {
    // 2026-03-23: 这里统一返回目标类型标签，目的是让错误信息能直接说明当前列期望的标准化口径。
    fn as_label(&self) -> &'static str {
        match self {
            Self::Date => "date",
            Self::DateTime => "datetime",
        }
    }
}

// 2026-03-23: 这里定义单列日期时间解析规则，目的是把时间标准化从 Skill 提示词搬到可测试、可复用的 Tool 输入结构里。
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ParseDateTimeRule {
    pub column: String,
    pub target_type: DateTimeTargetType,
}

// 2026-03-23: 这里定义日期时间标准化错误，目的是把缺列、重复规则、脏值和结果构建失败分层暴露给上层调用方。
#[derive(Debug, Error)]
pub enum ParseDateTimeError {
    #[error("parse_datetime_columns 至少需要一条 rules 规则")]
    EmptyRules,
    #[error("parse_datetime_columns 找不到列: {0}")]
    MissingColumn(String),
    #[error("parse_datetime_columns 存在重复列规则: {0}")]
    DuplicateRule(String),
    #[error("parse_datetime_columns 无法读取列`{column}`第{row_index}行的值: {message}")]
    ReadValue {
        column: String,
        row_index: usize,
        message: String,
    },
    #[error(
        "parse_datetime_columns 无法把列`{column}`第{row_index}行的值`{value}`解析为{target_type}"
    )]
    InvalidValue {
        column: String,
        row_index: usize,
        value: String,
        target_type: &'static str,
    },
    #[error("parse_datetime_columns 无法构建结果表: {0}")]
    BuildFrame(String),
}

// 2026-03-23: 这里执行按列日期时间标准化，目的是把日期类分析前置成独立 Tool，避免窗口、趋势与建模重复处理时间口径。
pub fn parse_datetime_columns(
    loaded: &LoadedTable,
    rules: &[ParseDateTimeRule],
) -> Result<LoadedTable, ParseDateTimeError> {
    if rules.is_empty() {
        return Err(ParseDateTimeError::EmptyRules);
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
            .map_err(|_| ParseDateTimeError::MissingColumn(column_name.clone()))?;
        let maybe_rule = rules.iter().find(|rule| rule.column == *column_name);

        match maybe_rule {
            Some(rule) => {
                let values = (0..loaded.dataframe.height())
                    .map(|row_index| {
                        let current_value = read_optional_string(source_column, row_index)?;
                        match current_value {
                            None => Ok(None),
                            Some(value) if value.trim().is_empty() => Ok(Some(value)),
                            Some(value) => parse_value(&value, rule).map(Some).map_err(|_| {
                                ParseDateTimeError::InvalidValue {
                                    column: rule.column.clone(),
                                    row_index,
                                    value,
                                    target_type: rule.target_type.as_label(),
                                }
                            }),
                        }
                    })
                    .collect::<Result<Vec<Option<String>>, ParseDateTimeError>>()?;
                frame_columns.push(Series::new(column_name.clone().into(), values).into());
            }
            None => frame_columns.push(source_column.clone()),
        }
    }

    let dataframe = DataFrame::new(frame_columns)
        .map_err(|error| ParseDateTimeError::BuildFrame(error.to_string()))?;
    let handle = TableHandle::new_confirmed(
        loaded.handle.source_path(),
        loaded.handle.sheet_name(),
        loaded.handle.columns().to_vec(),
    );

    Ok(LoadedTable { handle, dataframe })
}

// 2026-03-23: 这里先做重复规则校验，目的是避免同一列出现多套时间口径导致执行结果不可解释。
fn ensure_unique_rules(rules: &[ParseDateTimeRule]) -> Result<(), ParseDateTimeError> {
    let mut seen = std::collections::BTreeSet::<String>::new();
    for rule in rules {
        if !seen.insert(rule.column.clone()) {
            return Err(ParseDateTimeError::DuplicateRule(rule.column.clone()));
        }
    }
    Ok(())
}

// 2026-03-23: 这里统一校验目标列存在，目的是在逐行解析前先返回更友好的缺列错误。
fn ensure_column_exists(dataframe: &DataFrame, column: &str) -> Result<(), ParseDateTimeError> {
    if dataframe.column(column).is_err() {
        return Err(ParseDateTimeError::MissingColumn(column.to_string()));
    }
    Ok(())
}

// 2026-03-23: 这里统一把任意列值读成可选字符串，目的是兼容字符串列以及 Excel 原生日期序列这类数值输入。
fn read_optional_string(
    column: &Column,
    row_index: usize,
) -> Result<Option<String>, ParseDateTimeError> {
    let series = column.as_materialized_series();
    match series.get(row_index) {
        Ok(AnyValue::Null) => Ok(None),
        Ok(_) => series
            .str_value(row_index)
            .map(|value| Some(value.into_owned()))
            .map_err(|error| ParseDateTimeError::ReadValue {
                column: series.name().to_string(),
                row_index,
                message: error.to_string(),
            }),
        Err(error) => Err(ParseDateTimeError::ReadValue {
            column: series.name().to_string(),
            row_index,
            message: error.to_string(),
        }),
    }
}

// 2026-03-22: 这里按目标类型解析单个值，目的是把文本日期、真实日历校验和 Excel 序列值支持统一收口到一个入口。
fn parse_value(value: &str, rule: &ParseDateTimeRule) -> Result<String, ()> {
    match rule.target_type {
        DateTimeTargetType::Date => parse_date_string(value),
        DateTimeTargetType::DateTime => parse_datetime_string(value),
    }
}

// 2026-03-22: 这里先走文本日期，再兜底 Excel 序列值，目的是兼容普通文本台账和原生序列值两类来源。
fn parse_date_string(value: &str) -> Result<String, ()> {
    if let Some(date) = parse_date_value(value) {
        return Ok(date.to_iso_string());
    }

    parse_excel_serial_datetime(value)
        .map(|(date, _)| date.to_iso_string())
        .ok_or(())
}

// 2026-03-22: 这里解析日期时间文本，文本失败后再兜底 Excel 序列值，目的是补齐普通业务表最常见的两类输入。
fn parse_datetime_string(value: &str) -> Result<String, ()> {
    let trimmed = value.trim();
    let (date_part, time_part) = split_datetime_parts(trimmed);
    if let Some(date) = parse_date_value(date_part) {
        let time = match time_part {
            Some(part) if !part.trim().is_empty() => parse_time_value(part).ok_or(())?,
            _ => ParsedTime {
                hour: 0,
                minute: 0,
                second: 0,
            },
        };

        return Ok(format!(
            "{} {:02}:{:02}:{:02}",
            date.to_iso_string(),
            time.hour,
            time.minute,
            time.second
        ));
    }

    let (date, time) = parse_excel_serial_datetime(trimmed).ok_or(())?;
    Ok(format!(
        "{} {:02}:{:02}:{:02}",
        date.to_iso_string(),
        time.hour,
        time.minute,
        time.second
    ))
}

// 2026-03-23: 这里把日期与时间片段切开，目的是既支持纯日期补零点，也避免把“日期+脏后缀”误判成合法时间。
fn split_datetime_parts(value: &str) -> (&str, Option<&str>) {
    if let Some((date_part, time_part)) = value.split_once('T') {
        return (date_part.trim(), Some(time_part.trim()));
    }

    if let Some(index) = value.find(char::is_whitespace) {
        let (date_part, remainder) = value.split_at(index);
        return (date_part.trim(), Some(remainder.trim()));
    }

    (value.trim(), None)
}

// 2026-03-22: 这里解析 Excel 1900 系统序列值，目的是让 parse_datetime_columns 能直接处理用户表中的原生日期数字。
fn parse_excel_serial_datetime(value: &str) -> Option<(ParsedDate, ParsedTime)> {
    let serial = value.trim().parse::<f64>().ok()?;
    if !serial.is_finite() || serial < 0.0 {
        return None;
    }

    let whole_days = serial.floor() as i64;
    let fractional = serial - whole_days as f64;
    let mut seconds = (fractional * 86_400.0).round() as i64;
    let mut date = NaiveDate::from_ymd_opt(1899, 12, 30)? + Duration::days(whole_days);
    if seconds >= 86_400 {
        date += Duration::days(1);
        seconds -= 86_400;
    }

    let parsed_date = ParsedDate {
        year: date.year() as u32,
        month: date.month(),
        day: date.day(),
    };
    let hour = (seconds / 3600) as u32;
    let minute = ((seconds % 3600) / 60) as u32;
    let second = (seconds % 60) as u32;
    let parsed_time = ParsedTime {
        hour,
        minute,
        second,
    };

    Some((parsed_date, parsed_time))
}
