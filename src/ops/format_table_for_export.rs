use std::collections::BTreeSet;

use polars::prelude::{Column, DataFrame};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::handles::TableHandle;
use crate::frame::loader::LoadedTable;
use crate::ops::rename::{RenameColumnError, RenameColumnMapping, rename_columns};

// 2026-03-22: 这里定义导出前整理选项，目的是把“列顺序、表头别名、是否裁剪多余列”收口成一个稳定契约。
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ExportFormatOptions {
    #[serde(default)]
    pub column_order: Vec<String>,
    #[serde(default)]
    pub rename_mappings: Vec<RenameColumnMapping>,
    #[serde(default)]
    pub drop_unspecified_columns: bool,
}

// 2026-03-22: 这里定义导出整理错误，目的是把列顺序配置错误、结果构建失败和重命名错误分层暴露。
#[derive(Debug, Error)]
pub enum FormatTableForExportError {
    #[error("format_table_for_export 的 column_order 存在重复列: {0}")]
    DuplicateColumnOrder(String),
    #[error("format_table_for_export 找不到列: {0}")]
    MissingColumn(String),
    #[error("format_table_for_export 无法构建结果表: {0}")]
    BuildFrame(String),
    #[error("{0}")]
    Rename(#[from] RenameColumnError),
}

// 2026-03-22: 这里执行导出前整理，目的是把“交付给客户看的列布局”在真正写文件前先沉淀成可复用结果表。
pub fn format_table_for_export(
    loaded: &LoadedTable,
    options: &ExportFormatOptions,
) -> Result<LoadedTable, FormatTableForExportError> {
    let ordered_columns = resolve_column_order(loaded, options)?;
    let frame_columns = build_ordered_columns(loaded, &ordered_columns)?;
    let dataframe = DataFrame::new(frame_columns)
        .map_err(|error| FormatTableForExportError::BuildFrame(error.to_string()))?;
    let ordered_loaded = LoadedTable {
        // 2026-03-22: 这里先按原列名重建 handle，目的是让后续 rename 校验仍基于真实源列口径执行。
        handle: TableHandle::new_confirmed(
            loaded.handle.source_path(),
            loaded.handle.sheet_name(),
            ordered_columns,
        ),
        dataframe,
    };

    if options.rename_mappings.is_empty() {
        Ok(ordered_loaded)
    } else {
        rename_columns(&ordered_loaded, &options.rename_mappings).map_err(Into::into)
    }
}

// 2026-03-22: 这里先解析最终导出列顺序，目的是让“显式列顺序 + 可选保留剩余列”的语义在进入数据重建前就固定下来。
fn resolve_column_order(
    loaded: &LoadedTable,
    options: &ExportFormatOptions,
) -> Result<Vec<String>, FormatTableForExportError> {
    let mut seen = BTreeSet::<String>::new();
    let mut ordered = Vec::<String>::new();

    for column_name in &options.column_order {
        if !seen.insert(column_name.clone()) {
            return Err(FormatTableForExportError::DuplicateColumnOrder(
                column_name.clone(),
            ));
        }
        if loaded.dataframe.column(column_name).is_err() {
            return Err(FormatTableForExportError::MissingColumn(
                column_name.clone(),
            ));
        }
        ordered.push(column_name.clone());
    }

    if !options.drop_unspecified_columns {
        for column_name in loaded.handle.columns() {
            if seen.insert(column_name.clone()) {
                ordered.push(column_name.clone());
            }
        }
    }

    if ordered.is_empty() {
        ordered.extend(loaded.handle.columns().iter().cloned());
    }

    Ok(ordered)
}

// 2026-03-22: 这里按确定好的列顺序重建表，目的是让 workbook 组装层拿到的就是稳定、面向交付的列布局。
fn build_ordered_columns(
    loaded: &LoadedTable,
    ordered_columns: &[String],
) -> Result<Vec<Column>, FormatTableForExportError> {
    let mut frame_columns = Vec::<Column>::with_capacity(ordered_columns.len());
    for column_name in ordered_columns {
        let column = loaded
            .dataframe
            .column(column_name)
            .map_err(|_| FormatTableForExportError::MissingColumn(column_name.clone()))?
            .clone();
        frame_columns.push(column);
    }
    Ok(frame_columns)
}
