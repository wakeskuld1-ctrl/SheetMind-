use std::fs;
use std::path::Path;

use polars::prelude::AnyValue;
use rust_xlsxwriter::Workbook;
use thiserror::Error;

use crate::frame::loader::LoadedTable;
use crate::frame::workbook_ref_store::PersistedWorkbookDraft;

// 2026-03-22: 这里定义导出错误，目的是把路径、CSV 写出和 Excel 写出错误统一翻译成上层可读中文错误。
#[derive(Debug, Error)]
pub enum ExportError {
    #[error("导出路径缺少父目录: {0}")]
    MissingParentDirectory(String),
    #[error("无法创建导出目录: {0}")]
    CreateOutputDir(String),
    #[error("无法写出 CSV: {0}")]
    WriteCsv(String),
    #[error("无法写出 Excel: {0}")]
    WriteExcel(String),
}

// 2026-03-22: 这里导出 CSV，目的是先补齐客户最容易接收的平面报表交付能力。
pub fn export_csv(loaded: &LoadedTable, output_path: &str) -> Result<(), ExportError> {
    ensure_parent_dir(output_path)?;

    let mut rows = Vec::<String>::new();
    rows.push(
        loaded
            .dataframe
            .get_column_names()
            .iter()
            .map(|name| escape_csv_field(name))
            .collect::<Vec<_>>()
            .join(","),
    );

    for row_index in 0..loaded.dataframe.height() {
        let row = loaded
            .dataframe
            .get_columns()
            .iter()
            .map(|column| {
                column
                    .as_materialized_series()
                    .str_value(row_index)
                    .map(|value| escape_csv_field(value.as_ref()))
                    .map_err(|error| ExportError::WriteCsv(error.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        rows.push(row.join(","));
    }

    fs::write(output_path, rows.join("\n"))
        .map_err(|error| ExportError::WriteCsv(error.to_string()))
}

// 2026-03-22: 这里导出 Excel，目的是补齐普通业务用户最熟悉的 `.xlsx` 交付形态。
pub fn export_excel(
    loaded: &LoadedTable,
    output_path: &str,
    sheet_name: &str,
) -> Result<(), ExportError> {
    ensure_parent_dir(output_path)?;

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet
        .set_name(sheet_name)
        .map_err(|error| ExportError::WriteExcel(error.to_string()))?;

    for (column_index, column_name) in loaded.dataframe.get_column_names().iter().enumerate() {
        worksheet
            .write_string(0, column_index as u16, column_name.as_str())
            .map_err(|error| ExportError::WriteExcel(error.to_string()))?;
    }

    for row_index in 0..loaded.dataframe.height() {
        for (column_index, column) in loaded.dataframe.get_columns().iter().enumerate() {
            // 2026-03-23: 这里按真实单元格类型写 Excel，原因是用户导出的透视表需要直接继续做求和、透视与排序统计。
            // 2026-03-23: 这里对 null 直接留空，目的是避免把缺失值导出成字符串 `null` 污染业务表。
            write_excel_cell(worksheet, column, row_index, column_index)
                .map_err(|error| ExportError::WriteExcel(error.to_string()))?;
        }
    }

    workbook
        .save(output_path)
        .map_err(|error| ExportError::WriteExcel(error.to_string()))
}

// 2026-03-22: 这里导出多 Sheet workbook 草稿，目的是把 compose_workbook 生成的交付计划真正写成客户可打开的标准 Excel。
pub fn export_excel_workbook(
    draft: &PersistedWorkbookDraft,
    output_path: &str,
) -> Result<(), ExportError> {
    ensure_parent_dir(output_path)?;

    let mut workbook = Workbook::new();
    for worksheet_snapshot in &draft.worksheets {
        let dataframe = worksheet_snapshot
            .to_dataframe()
            .map_err(|error| ExportError::WriteExcel(error.to_string()))?;
        let worksheet = workbook.add_worksheet();
        worksheet
            .set_name(&worksheet_snapshot.sheet_name)
            .map_err(|error| ExportError::WriteExcel(error.to_string()))?;

        for (column_index, column_name) in dataframe.get_column_names().iter().enumerate() {
            worksheet
                .write_string(0, column_index as u16, column_name.as_str())
                .map_err(|error| ExportError::WriteExcel(error.to_string()))?;
        }

        for row_index in 0..dataframe.height() {
            for (column_index, column) in dataframe.get_columns().iter().enumerate() {
                // 2026-03-23: 这里和单表导出共用同一套类型写出逻辑，原因是多 Sheet 交付也要保持空值为空白、数值为可统计单元格。
                write_excel_cell(worksheet, column, row_index, column_index)
                    .map_err(|error| ExportError::WriteExcel(error.to_string()))?;
            }
        }
    }

    workbook
        .save(output_path)
        .map_err(|error| ExportError::WriteExcel(error.to_string()))
}

// 2026-03-22: 这里确保导出目录存在，目的是让客户直接给一个新路径时也能顺利写出文件。
fn ensure_parent_dir(output_path: &str) -> Result<(), ExportError> {
    let path = Path::new(output_path);
    let Some(parent) = path.parent() else {
        return Err(ExportError::MissingParentDirectory(output_path.to_string()));
    };
    fs::create_dir_all(parent).map_err(|error| ExportError::CreateOutputDir(error.to_string()))
}

// 2026-03-22: 这里统一做 CSV 转义，目的是让包含逗号、双引号和换行的值也能被 Excel 正常打开。
fn escape_csv_field(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn write_excel_cell(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    column: &polars::prelude::Column,
    row_index: usize,
    column_index: usize,
) -> Result<(), rust_xlsxwriter::XlsxError> {
    let row = (row_index + 1) as u32;
    let col = column_index as u16;
    let series = column.as_materialized_series();

    match series.get(row_index) {
        Ok(AnyValue::Null) => Ok(()),
        // 2026-03-23: 这里显式把 xlsxwriter 的链式返回收口成 `Result<(), _>`，目的是适配当前版本 API 而不改变导出行为。
        Ok(AnyValue::Int8(value)) => {
            worksheet.write_number(row, col, value as f64)?;
            Ok(())
        }
        // 2026-03-23: 这里保持所有整数都写成 number，目的是让导出的 Excel 单元格继续可被求和和透视。
        Ok(AnyValue::Int16(value)) => {
            worksheet.write_number(row, col, value as f64)?;
            Ok(())
        }
        Ok(AnyValue::Int32(value)) => {
            worksheet.write_number(row, col, value as f64)?;
            Ok(())
        }
        Ok(AnyValue::Int64(value)) => {
            worksheet.write_number(row, col, value as f64)?;
            Ok(())
        }
        Ok(AnyValue::UInt8(value)) => {
            worksheet.write_number(row, col, value as f64)?;
            Ok(())
        }
        Ok(AnyValue::UInt16(value)) => {
            worksheet.write_number(row, col, value as f64)?;
            Ok(())
        }
        Ok(AnyValue::UInt32(value)) => {
            worksheet.write_number(row, col, value as f64)?;
            Ok(())
        }
        Ok(AnyValue::UInt64(value)) => {
            worksheet.write_number(row, col, value as f64)?;
            Ok(())
        }
        Ok(AnyValue::Float32(value)) => {
            worksheet.write_number(row, col, value as f64)?;
            Ok(())
        }
        Ok(AnyValue::Float64(value)) => {
            worksheet.write_number(row, col, value)?;
            Ok(())
        }
        Ok(AnyValue::Boolean(value)) => {
            worksheet.write_boolean(row, col, value)?;
            Ok(())
        }
        Ok(_) => {
            // 2026-03-23: 这里把 Polars 文本读取错误显式转成 xlsx 导出错误，目的是避免 `?` 走不到目标错误类型。
            let value = series
                .str_value(row_index)
                .map_err(|error| rust_xlsxwriter::XlsxError::ParameterError(error.to_string()))?;
            worksheet.write_string(row, col, value.as_ref())?;
            Ok(())
        }
        Err(error) => Err(rust_xlsxwriter::XlsxError::ParameterError(error.to_string())),
    }
}
