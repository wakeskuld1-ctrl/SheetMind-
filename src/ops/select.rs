use polars::prelude::DataFrame;
use thiserror::Error;

use crate::domain::handles::TableHandle;
use crate::frame::loader::LoadedTable;

// 2026-03-21: 这里定义列选择错误，目的是把列不存在和 DataFrame 操作失败显式暴露给上层。
#[derive(Debug, Error)]
pub enum SelectError {
    // 2026-03-21: 当请求列为空时直接拒绝，目的是避免返回语义模糊的空表结果。
    #[error("select_columns 至少需要一个列名")]
    EmptySelection,
    // 2026-03-21: 当列不存在时返回明确错误，目的是帮助用户快速修正字段名拼写或映射问题。
    #[error("找不到列: {0}")]
    MissingColumn(String),
    // 2026-03-21: 包装底层 DataFrame 选择失败，目的是把 Polars 错误统一翻译成 Tool 级错误语义。
    #[error("无法选择列: {0}")]
    SelectFrame(String),
}

// 2026-03-21: 这里在已加载表上执行列选择，目的是把“表读取”与“表操作”拆成两个清晰步骤，符合 SRP。
pub fn select_columns(
    loaded: &LoadedTable,
    requested_columns: &[&str],
) -> Result<LoadedTable, SelectError> {
    if requested_columns.is_empty() {
        return Err(SelectError::EmptySelection);
    }

    ensure_columns_exist(&loaded.dataframe, requested_columns)?;

    let dataframe = loaded
        .dataframe
        .select(requested_columns.iter().copied())
        .map_err(|error| SelectError::SelectFrame(error.to_string()))?;
    let handle = TableHandle::new_confirmed(
        loaded.handle.source_path(),
        loaded.handle.sheet_name(),
        requested_columns
            .iter()
            .map(|column| (*column).to_string())
            .collect(),
    );

    Ok(LoadedTable { handle, dataframe })
}

// 2026-03-21: 单独校验列存在性，目的是在真正执行 DataFrame 选择前先返回更用户友好的缺列错误。
fn ensure_columns_exist(
    dataframe: &DataFrame,
    requested_columns: &[&str],
) -> Result<(), SelectError> {
    for column in requested_columns {
        if dataframe.column(column).is_err() {
            return Err(SelectError::MissingColumn((*column).to_string()));
        }
    }
    Ok(())
}
