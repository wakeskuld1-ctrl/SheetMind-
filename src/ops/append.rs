use std::collections::BTreeSet;

use polars::prelude::DataFrame;
use thiserror::Error;

use crate::domain::handles::TableHandle;
use crate::frame::loader::LoadedTable;

// 2026-03-21: 这里定义纵向追加错误，目的是把列集合不一致和底层 DataFrame 拼接失败清晰暴露给上层。
#[derive(Debug, Error)]
pub enum AppendError {
    // 2026-03-21: 这里要求两张表必须拥有完全相同的列名集合，目的是避免把不同结构的数据误拼到一起。
    #[error("append_tables 要求两张表包含完全相同的列名")]
    SchemaMismatch,
    // 2026-03-21: 这里包装列重排失败，目的是把“列存在性”问题与真正的纵向追加失败区分开。
    #[error("无法按列名对齐待追加表: {0}")]
    AlignColumns(String),
    // 2026-03-21: 这里统一包装 DataFrame 纵向拼接失败，目的是让 Tool 层只处理稳定业务语义错误。
    #[error("无法完成纵向追加: {0}")]
    AppendFrame(String),
}

// 2026-03-21: 这里执行 V1.1 纵向追加，目的是允许两张表列顺序不同，但结果仍以首表 schema 为准。
pub fn append_tables(top: &LoadedTable, bottom: &LoadedTable) -> Result<LoadedTable, AppendError> {
    ensure_same_column_set(top.handle.columns(), bottom.handle.columns())?;

    let aligned_bottom = reorder_dataframe_by_columns(&bottom.dataframe, top.handle.columns())?;
    let dataframe = top
        .dataframe
        .vstack(&aligned_bottom)
        .map_err(|error| AppendError::AppendFrame(error.to_string()))?;

    let handle = TableHandle::new_confirmed(
        format!(
            "{} + {}",
            top.handle.source_path(),
            bottom.handle.source_path()
        ),
        format!(
            "{} + {}",
            top.handle.sheet_name(),
            bottom.handle.sheet_name()
        ),
        top.handle.columns().to_vec(),
    );

    Ok(LoadedTable { handle, dataframe })
}

// 2026-03-21: 这里单独校验列名集合一致性，目的是允许列顺序不同，但继续拒绝缺列/多列场景。
fn ensure_same_column_set(left: &[String], right: &[String]) -> Result<(), AppendError> {
    let left_set = left.iter().cloned().collect::<BTreeSet<_>>();
    let right_set = right.iter().cloned().collect::<BTreeSet<_>>();

    if left_set == right_set {
        Ok(())
    } else {
        Err(AppendError::SchemaMismatch)
    }
}

// 2026-03-21: 这里按首表列顺序重排待追加表，目的是保证追加结果 schema 稳定且可继续串接后续 Tool。
fn reorder_dataframe_by_columns(
    dataframe: &DataFrame,
    target_columns: &[String],
) -> Result<DataFrame, AppendError> {
    dataframe
        .select(target_columns.iter().map(|column| column.as_str()))
        .map_err(|error| AppendError::AlignColumns(error.to_string()))
}
