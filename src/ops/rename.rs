use polars::prelude::{Column, DataFrame};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::handles::TableHandle;
use crate::frame::loader::LoadedTable;

// 2026-03-23: 这里定义列改名映射，目的是把字段口径统一前置成显式配置，而不是散落在 Skill 提示词里。
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct RenameColumnMapping {
    pub from: String,
    pub to: String,
}

// 2026-03-23: 这里定义列改名错误，目的是把缺列、重复映射与目标冲突显式暴露给上层。
#[derive(Debug, Error)]
pub enum RenameColumnError {
    #[error("rename_columns 至少需要一条 mappings 映射")]
    EmptyMappings,
    #[error("rename_columns 找不到列: {0}")]
    MissingColumn(String),
    #[error("rename_columns 存在重复源列映射: {0}")]
    DuplicateSource(String),
    #[error("rename_columns 目标列名冲突: {0}")]
    DuplicateTarget(String),
    #[error("rename_columns 无法构建结果表: {0}")]
    BuildFrame(String),
}

// 2026-03-23: 这里执行显式列改名，目的是把字段标准化沉淀成独立 Tool，便于后续聚合、分析和导出复用统一列口径。
pub fn rename_columns(
    loaded: &LoadedTable,
    mappings: &[RenameColumnMapping],
) -> Result<LoadedTable, RenameColumnError> {
    if mappings.is_empty() {
        return Err(RenameColumnError::EmptyMappings);
    }

    ensure_unique_sources(mappings)?;
    let final_names = build_final_column_names(loaded.handle.columns(), mappings)?;
    let mut frame_columns = Vec::<Column>::with_capacity(loaded.dataframe.width());

    for (index, source_name) in loaded.handle.columns().iter().enumerate() {
        let mut column = loaded
            .dataframe
            .column(source_name)
            .map_err(|_| RenameColumnError::MissingColumn(source_name.clone()))?
            .clone();
        if final_names[index] != *source_name {
            column.rename(final_names[index].clone().into());
        }
        frame_columns.push(column);
    }

    let dataframe = DataFrame::new(frame_columns)
        .map_err(|error| RenameColumnError::BuildFrame(error.to_string()))?;
    let handle = TableHandle::new_confirmed(
        loaded.handle.source_path(),
        loaded.handle.sheet_name(),
        final_names.clone(),
    );

    Ok(LoadedTable { handle, dataframe })
}

// 2026-03-23: 这里先锁定同一源列只能映射一次，目的是避免 rename 配置存在二义性。
fn ensure_unique_sources(mappings: &[RenameColumnMapping]) -> Result<(), RenameColumnError> {
    let mut seen = std::collections::BTreeSet::<String>::new();
    for mapping in mappings {
        if !seen.insert(mapping.from.clone()) {
            return Err(RenameColumnError::DuplicateSource(mapping.from.clone()));
        }
    }
    Ok(())
}

// 2026-03-23: 这里先计算最终列名集合，目的是在真正改名之前就把缺列和目标冲突一次性校验清楚。
fn build_final_column_names(
    current_columns: &[String],
    mappings: &[RenameColumnMapping],
) -> Result<Vec<String>, RenameColumnError> {
    let mut final_names = current_columns.to_vec();

    for mapping in mappings {
        let Some(position) = current_columns
            .iter()
            .position(|column| column == &mapping.from)
        else {
            return Err(RenameColumnError::MissingColumn(mapping.from.clone()));
        };
        final_names[position] = mapping.to.clone();
    }

    let mut seen = std::collections::BTreeSet::<String>::new();
    for name in &final_names {
        if !seen.insert(name.clone()) {
            return Err(RenameColumnError::DuplicateTarget(name.clone()));
        }
    }

    Ok(final_names)
}
