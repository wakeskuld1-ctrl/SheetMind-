use std::collections::BTreeSet;

use polars::prelude::{Column, DataFrame};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::handles::TableHandle;
use crate::frame::loader::LoadedTable;
use crate::frame::workbook_ref_store::{
    PersistedWorkbookColumnConditionalFormatRule, PersistedWorkbookColumnNumberFormatRule,
};
use crate::ops::rename::{RenameColumnError, RenameColumnMapping, rename_columns};

// 2026-03-24: 这里重写导出整理选项定义，原因是历史乱码把 derive 属性吞进注释后已经破坏编译；
// 目的是先恢复 format_table_for_export 的稳定语法边界，再继续承接数字格式元数据。
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ExportFormatOptions {
    #[serde(default)]
    pub column_order: Vec<String>,
    #[serde(default)]
    pub rename_mappings: Vec<RenameColumnMapping>,
    #[serde(default)]
    pub number_formats: Vec<PersistedWorkbookColumnNumberFormatRule>,
    #[serde(default)]
    pub conditional_formats: Vec<PersistedWorkbookColumnConditionalFormatRule>,
    #[serde(default)]
    pub drop_unspecified_columns: bool,
}

// 2026-03-24: 这里重写导出整理错误类型，原因是历史坏行把 Error derive 一起吞掉；
// 目的是保留现有行为边界，同时让上层 dispatcher 能继续复用这些错误语义。
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

// 2026-03-24: 这里恢复导出前整理主流程，原因是历史注释已把函数签名损坏；
// 目的是继续维持“列顺序 -> 重建 DataFrame -> 可选重命名”的稳定执行顺序。
pub fn format_table_for_export(
    loaded: &LoadedTable,
    options: &ExportFormatOptions,
) -> Result<LoadedTable, FormatTableForExportError> {
    let ordered_columns = resolve_column_order(loaded, options)?;
    let frame_columns = build_ordered_columns(loaded, &ordered_columns)?;
    let dataframe = DataFrame::new(frame_columns)
        .map_err(|error| FormatTableForExportError::BuildFrame(error.to_string()))?;

    let ordered_loaded = LoadedTable {
        // 2026-03-24: 这里按最终列顺序重建 handle，原因是后续 rename 校验必须基于可见列集合；
        // 目的是保证 format_table_for_export 输出的 handle 与 dataframe 一致。
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

// 2026-03-24: 这里解析最终列顺序，原因是导出层既要支持显式列顺序，也要支持保留剩余列；
// 目的是先把列布局语义冻结，再进入 DataFrame 重建阶段。
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

// 2026-03-24: 这里按既定列顺序重建 Polars 列集合，原因是导出整理只应做结构编排，不应改值；
// 目的是让上层 workbook/report_delivery 直接拿到稳定的结果表。
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
