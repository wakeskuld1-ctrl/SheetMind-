use std::collections::BTreeMap;

use polars::prelude::{AnyValue, DataFrame};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TablePreview {
    pub columns: Vec<String>,
    pub rows: Vec<BTreeMap<String, String>>,
}

#[derive(Debug, Error)]
pub enum PreviewError {
    #[error("无法生成表预览: {0}")]
    ReadValue(String),
}

pub fn preview_table(dataframe: &DataFrame, limit: usize) -> Result<TablePreview, PreviewError> {
    let columns = dataframe
        .get_column_names_str()
        .into_iter()
        .map(|name| name.to_string())
        .collect::<Vec<_>>();
    let row_limit = limit.min(dataframe.height());
    let materialized_columns = dataframe.get_columns();
    let mut rows = Vec::with_capacity(row_limit);

    for row_index in 0..row_limit {
        let mut row = BTreeMap::new();
        for column in materialized_columns {
            // 2026-03-23: 这里显式把 null 预览成空字符串，原因是用户不希望在结果视图里再看到字面量 `null`。
            // 2026-03-23: 这里同时把数值列渲染成紧凑文本，目的是避免 100.0 这类展示噪音影响 Excel 风格阅读。
            let series = column.as_materialized_series();
            let value = match series.get(row_index) {
                Ok(AnyValue::Null) => String::new(),
                Ok(AnyValue::Int8(value)) => value.to_string(),
                Ok(AnyValue::Int16(value)) => value.to_string(),
                Ok(AnyValue::Int32(value)) => value.to_string(),
                Ok(AnyValue::Int64(value)) => value.to_string(),
                Ok(AnyValue::UInt8(value)) => value.to_string(),
                Ok(AnyValue::UInt16(value)) => value.to_string(),
                Ok(AnyValue::UInt32(value)) => value.to_string(),
                Ok(AnyValue::UInt64(value)) => value.to_string(),
                Ok(AnyValue::Float32(value)) => format_preview_number(value as f64),
                Ok(AnyValue::Float64(value)) => format_preview_number(value),
                Ok(AnyValue::Boolean(value)) => value.to_string(),
                Ok(_) => series
                    .str_value(row_index)
                    .map(|cell| cell.into_owned())
                    .map_err(|error| PreviewError::ReadValue(error.to_string()))?,
                Err(error) => return Err(PreviewError::ReadValue(error.to_string())),
            };
            row.insert(column.name().to_string(), value);
        }
        rows.push(row);
    }

    Ok(TablePreview { columns, rows })
}

fn format_preview_number(value: f64) -> String {
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
