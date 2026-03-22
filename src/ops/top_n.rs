use thiserror::Error;

use crate::frame::loader::LoadedTable;
use crate::ops::sort::{SortError, SortSpec, sort_rows};

// 2026-03-21: 这里定义 top_n 错误，目的是把无效截取数量与排序阶段失败区分开暴露给上层。
#[derive(Debug, Error)]
pub enum TopNError {
    // 2026-03-21: n 必须大于 0，目的是避免返回语义含糊的空结果并帮助用户直接修正参数。
    #[error("top_n 的 n 必须大于 0")]
    InvalidCount,
    // 2026-03-21: 这里复用排序错误，目的是让 top_n 保持“先排序、后截取”的单一职责组合逻辑。
    #[error(transparent)]
    Sort(#[from] SortError),
}

// 2026-03-21: 这里执行 top_n，目的是把“按指定规则排序后取前 N 条”沉淀成独立可复用能力。
pub fn top_n_rows(
    loaded: &LoadedTable,
    sorts: &[SortSpec],
    n: usize,
) -> Result<LoadedTable, TopNError> {
    if n == 0 {
        return Err(TopNError::InvalidCount);
    }

    let sorted = sort_rows(loaded, sorts)?;
    let dataframe = sorted.dataframe.slice(0, n);

    Ok(LoadedTable {
        // 2026-03-21: 截取前 N 行不会改变列结构与表身份语义，因此直接复用排序后的句柄元数据。
        handle: sorted.handle,
        dataframe,
    })
}
