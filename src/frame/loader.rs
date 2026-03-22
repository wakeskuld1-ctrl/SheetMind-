use calamine::{Reader, open_workbook_auto};
use polars::prelude::{Column, DataFrame, NamedFrom, PolarsError, Series};
use thiserror::Error;

use crate::domain::handles::TableHandle;
use crate::domain::schema::{HeaderInference, SchemaState};
use crate::frame::region_loader::load_table_region;
use crate::frame::table_ref_store::PersistedTableRef;

// 2026-03-21: 这里定义已加载表对象，目的是把句柄元数据与 Polars DataFrame 绑定在一起，供后续 Tool 复用。
pub struct LoadedTable {
    // 2026-03-21: 这里保留确认后的表句柄，目的是让后续 Tool 同时访问 schema 元数据与来源信息。
    pub handle: TableHandle,
    // 2026-03-21: 这里保留实际 DataFrame，目的是让筛选、聚合、关联等计算统一基于 Polars 执行。
    pub dataframe: DataFrame,
}

// 2026-03-21: 这里定义表加载错误，目的是把 schema 门禁、Excel 读取与 DataFrame 构造错误分开收口。
#[derive(Debug, Error)]
pub enum FrameLoadError {
    #[error("schema 尚未确认，不能加载为数据表")]
    UnconfirmedSchema,
    #[error("无法打开工作簿: {0}")]
    OpenWorkbook(String),
    #[error("无法读取工作表: {0}")]
    ReadSheet(String),
    #[error("无法构建 DataFrame: {0}")]
    BuildFrame(String),
    // 2026-03-22: 这里包装 table_ref 加载失败，目的是让分析/建模层可以直接透传稳定中文错误。
    #[error("无法根据 table_ref 加载已确认表: {0}")]
    TableRef(String),
}

// 2026-03-21: 这里把确认后的表头推断结果正式加载为 Polars DataFrame，目的是为所有计算 Tool 提供统一承载层。
pub fn load_confirmed_table(
    path: &str,
    sheet_name: &str,
    inference: &HeaderInference,
) -> Result<LoadedTable, FrameLoadError> {
    if !matches!(inference.schema_state, SchemaState::Confirmed) {
        return Err(FrameLoadError::UnconfirmedSchema);
    }

    let mut workbook = open_workbook_auto(path)
        .map_err(|error| FrameLoadError::OpenWorkbook(error.to_string()))?;
    let range = workbook
        .worksheet_range(sheet_name)
        .map_err(|error| FrameLoadError::ReadSheet(error.to_string()))?;

    let width = inference.columns.len();
    let mut columns = vec![Vec::<String>::new(); width];

    for row in range.rows().skip(inference.data_start_row_index) {
        // 2026-03-21: 这里把每行补齐到 schema 列宽，目的是让 Excel 缺省单元格也能稳定装入 DataFrame。
        for (column_index, target_column) in columns.iter_mut().enumerate() {
            let value = row
                .get(column_index)
                .map(|cell| cell.to_string())
                .unwrap_or_default();
            target_column.push(value);
        }
    }

    let frame_columns = inference
        .columns
        .iter()
        .zip(columns)
        .map(|(header, values)| Series::new((&header.canonical_name).into(), values).into())
        .collect::<Vec<Column>>();

    let dataframe = DataFrame::new(frame_columns).map_err(map_polars_error)?;
    let handle = TableHandle::new_confirmed(
        path,
        sheet_name,
        inference
            .columns
            .iter()
            .map(|column| column.canonical_name.clone())
            .collect(),
    );

    Ok(LoadedTable { handle, dataframe })
}

// 2026-03-22: 这里通过持久化 table_ref 恢复确认态加载，目的是让整表与局部区域都能跨请求稳定复用。
pub fn load_table_from_table_ref(
    persisted: &PersistedTableRef,
) -> Result<LoadedTable, FrameLoadError> {
    persisted
        .validate_source_unchanged()
        .map_err(|error| FrameLoadError::TableRef(error.to_string()))?;

    // 2026-03-22: 这里先分流 region table_ref，目的是避免显式区域回放时退化回整张 Sheet。
    if persisted.is_region_ref() {
        let region = persisted
            .region
            .as_deref()
            .ok_or_else(|| FrameLoadError::TableRef("region table_ref 缺少区域信息".to_string()))?;
        return load_table_region(
            &persisted.source_path,
            &persisted.sheet_name,
            region,
            persisted.header_row_count,
        )
        .map_err(|error| FrameLoadError::TableRef(error.to_string()));
    }

    let inference = persisted.to_confirmed_inference();
    load_confirmed_table(&persisted.source_path, &persisted.sheet_name, &inference)
}

// 2026-03-21: 这里单独抽出 Polars 错误映射，目的是避免加载主流程里堆满重复模板代码。
fn map_polars_error(error: PolarsError) -> FrameLoadError {
    FrameLoadError::BuildFrame(error.to_string())
}
