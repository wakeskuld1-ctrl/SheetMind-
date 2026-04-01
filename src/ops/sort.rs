use polars::prelude::{DataFrame, SortMultipleOptions};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::handles::TableHandle;
use crate::frame::loader::LoadedTable;

// 2026-03-21: 这里定义单个排序请求，目的是把用户侧“按哪列、什么方向排序”的意图稳定映射成可执行结构。
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct SortSpec {
    // 2026-03-21: 指定排序列名，目的是避免把排序逻辑和列位置耦合，便于后续列裁剪后继续复用。
    pub column: String,
    // 2026-03-21: 指定是否降序，目的是用最直白的布尔协议承载升降序，而不暴露 SQL 术语给用户。
    pub descending: bool,
}

// 2026-03-21: 这里定义排序错误，目的是把空排序计划、缺列和底层执行失败区分开返回给上层。
#[derive(Debug, Error)]
pub enum SortError {
    // 2026-03-21: 空排序计划直接拒绝，目的是避免返回“看似成功但没有任何行为”的模糊结果。
    #[error("sort_rows 至少需要一个排序定义")]
    EmptySorts,
    // 2026-03-21: 缺列时返回明确错误，目的是帮助用户快速修正字段名或前置表头确认结果。
    #[error("找不到列: {0}")]
    MissingColumn(String),
    // 2026-03-21: 包装底层排序失败，目的是让 Tool 层只处理稳定的业务语义错误。
    #[error("无法完成排序: {0}")]
    SortFrame(String),
}

// 2026-03-21: 这里执行显式多列排序，目的是为表预览、聚合后排序和后续 top_n 能力提供统一底座。
pub fn sort_rows(loaded: &LoadedTable, sorts: &[SortSpec]) -> Result<LoadedTable, SortError> {
    if sorts.is_empty() {
        return Err(SortError::EmptySorts);
    }

    ensure_columns_exist(&loaded.dataframe, sorts)?;

    let sort_columns = sorts
        .iter()
        .map(|sort| sort.column.as_str())
        .collect::<Vec<_>>();
    let descending = sorts.iter().map(|sort| sort.descending).collect::<Vec<_>>();
    let dataframe = loaded
        .dataframe
        .sort(
            sort_columns,
            SortMultipleOptions::default()
                .with_order_descending_multi(descending)
                .with_maintain_order(true),
        )
        .map_err(|error| SortError::SortFrame(error.to_string()))?;

    let handle = TableHandle::new_confirmed(
        loaded.handle.source_path(),
        loaded.handle.sheet_name(),
        loaded.handle.columns().to_vec(),
    );

    Ok(LoadedTable { handle, dataframe })
}

// 2026-03-21: 这里提前校验排序列是否存在，目的是在进入 Polars 前先返回更友好的中文错误。
fn ensure_columns_exist(dataframe: &DataFrame, sorts: &[SortSpec]) -> Result<(), SortError> {
    for sort in sorts {
        if dataframe.column(&sort.column).is_err() {
            return Err(SortError::MissingColumn(sort.column.clone()));
        }
    }

    Ok(())
}
