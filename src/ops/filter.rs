// 2026-03-21: 引入 NewChunkedArray trait，目的是让布尔掩码可以稳定构造，修复 filter_rows 首版编译失败。
use polars::prelude::{BooleanChunked, NewChunkedArray};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::handles::TableHandle;
use crate::frame::loader::LoadedTable;

// 2026-03-21: 这里定义过滤条件，目的是把用户或 Tool 层的筛选意图转成稳定、可测试的结构化输入。
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct FilterCondition {
    // 2026-03-21: 指定目标列名，目的是在 DataFrame 上明确定位要参与筛选的字段。
    pub column: String,
    // 2026-03-21: 指定比较操作符，目的是后续逐步扩展大于、小于、包含等操作时保持接口稳定。
    pub operator: FilterOperator,
    // 2026-03-21: 指定比较值，目的是在首版全部按字符串载入时先走一致的比较路径。
    pub value: String,
}

// 2026-03-21: 这里定义过滤操作符，目的是先支持最小可用的等值筛选，再逐步扩展更多运算。
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterOperator {
    // 2026-03-21: 先支持等值比较，目的是覆盖 Excel 用户最常见的维度筛选场景。
    Equals,
}

// 2026-03-21: 这里定义过滤错误，目的是把空条件、缺列和 DataFrame 过滤失败显式暴露给上层。
#[derive(Debug, Error)]
pub enum FilterError {
    // 2026-03-21: 没有条件时直接拒绝，目的是避免返回语义不清的“原表即结果”。
    #[error("filter_rows 至少需要一个条件")]
    EmptyConditions,
    // 2026-03-21: 缺列时返回明确错误，目的是帮助用户快速修正字段名或表头映射。
    #[error("找不到列: {0}")]
    MissingColumn(String),
    // 2026-03-21: 包装底层筛选失败，目的是把 Polars 错误统一翻译成 Tool 级错误。
    #[error("无法过滤数据: {0}")]
    FilterFrame(String),
}

// 2026-03-21: 这里对已加载表执行条件过滤，目的是让后续聚合、透视和关联前都能先做稳定数据裁剪。
pub fn filter_rows(
    loaded: &LoadedTable,
    conditions: &[FilterCondition],
) -> Result<LoadedTable, FilterError> {
    if conditions.is_empty() {
        return Err(FilterError::EmptyConditions);
    }

    let mut mask = vec![true; loaded.dataframe.height()];

    for condition in conditions {
        let column = loaded
            .dataframe
            .column(&condition.column)
            .map_err(|_| FilterError::MissingColumn(condition.column.clone()))?;
        let series = column.as_materialized_series();

        for (row_index, row_match) in mask.iter_mut().enumerate() {
            if !*row_match {
                continue;
            }

            let cell_value = series
                .str_value(row_index)
                .map(|value| value.into_owned())
                .map_err(|error| FilterError::FilterFrame(error.to_string()))?;
            let matches = match condition.operator {
                FilterOperator::Equals => cell_value == condition.value,
            };
            *row_match = matches;
        }
    }

    let mask_chunked = BooleanChunked::from_slice("filter_mask".into(), &mask);
    let dataframe = loaded
        .dataframe
        .filter(&mask_chunked)
        .map_err(|error| FilterError::FilterFrame(error.to_string()))?;
    let handle = TableHandle::new_confirmed(
        loaded.handle.source_path(),
        loaded.handle.sheet_name(),
        loaded.handle.columns().to_vec(),
    );

    Ok(LoadedTable { handle, dataframe })
}
