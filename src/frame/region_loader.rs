use calamine::{Reader, open_workbook_auto};
use polars::prelude::{Column, DataFrame, NamedFrom, PolarsError, Series};
use thiserror::Error;

use crate::domain::handles::TableHandle;
use crate::excel::header_inference::canonicalize_header_paths;
use crate::excel::sheet_range::{parse_excel_region, region_cell_value};
use crate::frame::loader::LoadedTable;

// 2026-03-22: 这里定义显式区域加载错误，目的是把区域语法、表头参数和 DataFrame 构造失败明确区分开。
#[derive(Debug, Error)]
pub enum RegionLoadError {
    #[error("header_row_count 必须大于 0")]
    InvalidHeaderRowCount,
    #[error("无法打开工作簿: {0}")]
    OpenWorkbook(String),
    #[error("无法读取工作表: {0}")]
    ReadSheet(String),
    #[error("{0}")]
    InvalidRegion(String),
    #[error("指定区域的表头行为空，请重新选择区域或调整 header_row_count")]
    EmptyHeader,
    #[error("无法构建 DataFrame: {0}")]
    BuildFrame(String),
}

// 2026-03-22: 这里实现显式区域加载，目的是把用户确认过的区域稳定装载为后续可复用的 DataFrame。
pub fn load_table_region(
    path: &str,
    sheet_name: &str,
    region_text: &str,
    header_row_count: usize,
) -> Result<LoadedTable, RegionLoadError> {
    if header_row_count == 0 {
        return Err(RegionLoadError::InvalidHeaderRowCount);
    }

    let region = parse_excel_region(region_text)
        .map_err(|error| RegionLoadError::InvalidRegion(error.to_string()))?;
    if header_row_count > region.row_count() {
        return Err(RegionLoadError::InvalidRegion(
            "header_row_count 超出了指定区域的行数".to_string(),
        ));
    }

    let mut workbook = open_workbook_auto(path)
        .map_err(|error| RegionLoadError::OpenWorkbook(error.to_string()))?;
    let sheet_range = workbook
        .worksheet_range(sheet_name)
        .map_err(|error| RegionLoadError::ReadSheet(error.to_string()))?;

    let header_paths = build_header_paths(&sheet_range, &region, header_row_count);
    if header_paths.iter().all(|path| path.is_empty()) {
        return Err(RegionLoadError::EmptyHeader);
    }

    let (columns, _) = canonicalize_header_paths(header_paths);
    let mut values_by_column = vec![Vec::<String>::new(); region.column_count()];

    for row_offset in header_row_count..region.row_count() {
        for (column_index, target_column) in values_by_column.iter_mut().enumerate() {
            target_column.push(region_cell_value(
                &sheet_range,
                &region,
                row_offset + 1,
                column_index + 1,
            ));
        }
    }

    let frame_columns = columns
        .iter()
        .zip(values_by_column)
        .map(|(header, values)| Series::new((&header.canonical_name).into(), values).into())
        .collect::<Vec<Column>>();
    let dataframe = DataFrame::new(frame_columns).map_err(map_polars_error)?;
    let handle = TableHandle::new_confirmed(
        path,
        sheet_name,
        columns
            .iter()
            .map(|column| column.canonical_name.clone())
            .collect(),
    );

    Ok(LoadedTable { handle, dataframe })
}

fn build_header_paths(
    sheet_range: &calamine::Range<calamine::Data>,
    region: &crate::excel::sheet_range::ExcelRegion,
    header_row_count: usize,
) -> Vec<Vec<String>> {
    (0..region.column_count())
        .map(|column_index| {
            (0..header_row_count)
                .map(|row_index| {
                    region_cell_value(sheet_range, region, row_index + 1, column_index + 1)
                })
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>()
        })
        .collect()
}

fn map_polars_error(error: PolarsError) -> RegionLoadError {
    RegionLoadError::BuildFrame(error.to_string())
}
