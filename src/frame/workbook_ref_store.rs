use std::fs;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use polars::prelude::DataFrame;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::frame::result_ref_store::{
    PersistedResultColumn, PersistedResultDataset, ResultRefStoreError,
};
use crate::runtime_paths::workspace_runtime_dir;

// 2026-03-22: 这里定义 workbook 草稿输入 sheet，目的是把 dispatcher 组装好的多表快照统一压成可持久化结构前的最小中间形态。
pub struct WorkbookSheetInput {
    pub sheet_name: String,
    pub source_refs: Vec<String>,
    pub dataframe: DataFrame,
}

// 2026-03-22: 这里定义持久化的 workbook sheet，目的是让多 Sheet 导出可以脱离原始来源直接重放。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersistedWorkbookSheet {
    pub sheet_name: String,
    #[serde(default)]
    pub source_refs: Vec<String>,
    pub row_count: usize,
    pub columns: Vec<PersistedResultColumn>,
}

impl PersistedWorkbookSheet {
    // 2026-03-22: 这里从 DataFrame 构造 sheet 快照，目的是让 compose_workbook 把每张表直接冻结成可导出的本地草稿。
    pub fn from_dataframe(
        sheet_name: &str,
        source_refs: Vec<String>,
        dataframe: &DataFrame,
    ) -> Result<Self, WorkbookRefStoreError> {
        let dataset = PersistedResultDataset::from_dataframe(
            "__workbook_sheet__",
            "compose_workbook",
            source_refs.clone(),
            dataframe,
        )
        .map_err(WorkbookRefStoreError::SnapshotSheet)?;

        Ok(Self {
            sheet_name: sheet_name.to_string(),
            source_refs,
            row_count: dataset.row_count,
            columns: dataset.columns,
        })
    }

    // 2026-03-22: 这里把 sheet 快照恢复成 DataFrame，目的是让导出层可以直接消费 compose_workbook 的落盘结果。
    pub fn to_dataframe(&self) -> Result<DataFrame, WorkbookRefStoreError> {
        PersistedResultDataset {
            result_ref: "__workbook_sheet__".to_string(),
            produced_by: "compose_workbook".to_string(),
            source_refs: self.source_refs.clone(),
            row_count: self.row_count,
            columns: self.columns.clone(),
        }
        .to_dataframe()
        .map_err(WorkbookRefStoreError::RestoreWorkbook)
    }
}

// 2026-03-22: 这里定义持久化的 workbook 草稿，目的是把多 Sheet 输出计划提升成可跨请求复用的稳定句柄。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersistedWorkbookDraft {
    pub workbook_ref: String,
    pub worksheets: Vec<PersistedWorkbookSheet>,
}

impl PersistedWorkbookDraft {
    // 2026-03-22: 这里从多张 sheet 输入构造 workbook 草稿，目的是集中校验 sheet 名并把数据快照一次性落盘。
    pub fn from_sheet_inputs(
        workbook_ref: &str,
        worksheets: Vec<WorkbookSheetInput>,
    ) -> Result<Self, WorkbookRefStoreError> {
        if worksheets.is_empty() {
            return Err(WorkbookRefStoreError::EmptyWorksheets);
        }

        let mut seen = std::collections::BTreeSet::<String>::new();
        let mut persisted = Vec::<PersistedWorkbookSheet>::with_capacity(worksheets.len());
        for worksheet in worksheets {
            if worksheet.sheet_name.trim().is_empty() {
                return Err(WorkbookRefStoreError::InvalidSheetName(
                    "sheet_name 不能为空".to_string(),
                ));
            }
            if !seen.insert(worksheet.sheet_name.clone()) {
                return Err(WorkbookRefStoreError::DuplicateSheetName(
                    worksheet.sheet_name.clone(),
                ));
            }
            persisted.push(PersistedWorkbookSheet::from_dataframe(
                &worksheet.sheet_name,
                worksheet.source_refs,
                &worksheet.dataframe,
            )?);
        }

        Ok(Self {
            workbook_ref: workbook_ref.to_string(),
            worksheets: persisted,
        })
    }
}

// 2026-03-22: 这里定义 workbook 草稿存储错误，目的是把快照、恢复和文件读写错误统一翻译成可读中文信息。
#[derive(Debug, Error)]
pub enum WorkbookRefStoreError {
    #[error("compose_workbook 至少需要一张 worksheet")]
    EmptyWorksheets,
    #[error("compose_workbook 的 sheet_name 不合法: {0}")]
    InvalidSheetName(String),
    #[error("compose_workbook 存在重复 sheet_name: {0}")]
    DuplicateSheetName(String),
    #[error("无法为 workbook 草稿创建存储目录: {0}")]
    CreateStoreDir(String),
    #[error("无法保存 workbook_ref `{workbook_ref}`: {message}")]
    SaveWorkbook {
        workbook_ref: String,
        message: String,
    },
    #[error("无法读取 workbook_ref `{workbook_ref}`: {message}")]
    LoadWorkbook {
        workbook_ref: String,
        message: String,
    },
    #[error("无法快照 worksheet: {0}")]
    SnapshotSheet(ResultRefStoreError),
    #[error("无法恢复 workbook 草稿中的 worksheet: {0}")]
    RestoreWorkbook(ResultRefStoreError),
}

// 2026-03-22: 这里定义 workbook_ref 存储入口，目的是给 compose/export 提供独立于 result_ref 的工作簿级句柄管理。
#[derive(Debug, Clone)]
pub struct WorkbookDraftStore {
    root_dir: PathBuf,
}

impl WorkbookDraftStore {
    // 2026-03-22: 这里允许用显式目录创建 workbook store，目的是让测试与生产都能复用同一套持久化逻辑。
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }

    // 2026-03-22: 这里提供默认 workbook 草稿目录，目的是让 compose/export 两个 Tool 共享稳定的落盘位置。
    pub fn workspace_default() -> Result<Self, WorkbookRefStoreError> {
        let runtime_dir = workspace_runtime_dir().map_err(WorkbookRefStoreError::CreateStoreDir)?;
        Ok(Self::new(runtime_dir.join("workbook_refs")))
    }

    // 2026-03-22: 这里统一生成 workbook_ref，目的是把多 Sheet 草稿和单表 result_ref 做清晰区分。
    pub fn create_workbook_ref() -> String {
        let timestamp = UNIX_EPOCH
            .elapsed()
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        format!("workbook_{}_{}", std::process::id(), timestamp)
    }

    // 2026-03-22: 这里保存 workbook 草稿，目的是让 compose_workbook 与 export_excel_workbook 之间通过稳定句柄衔接。
    pub fn save(&self, draft: &PersistedWorkbookDraft) -> Result<(), WorkbookRefStoreError> {
        fs::create_dir_all(&self.root_dir)
            .map_err(|error| WorkbookRefStoreError::CreateStoreDir(error.to_string()))?;
        let payload = serde_json::to_vec_pretty(draft).map_err(|error| {
            WorkbookRefStoreError::SaveWorkbook {
                workbook_ref: draft.workbook_ref.clone(),
                message: error.to_string(),
            }
        })?;
        fs::write(self.file_path(&draft.workbook_ref), payload).map_err(|error| {
            WorkbookRefStoreError::SaveWorkbook {
                workbook_ref: draft.workbook_ref.clone(),
                message: error.to_string(),
            }
        })
    }

    // 2026-03-22: 这里读取 workbook 草稿，目的是让导出动作只依赖 workbook_ref 就能重放多 Sheet 快照。
    pub fn load(
        &self,
        workbook_ref: &str,
    ) -> Result<PersistedWorkbookDraft, WorkbookRefStoreError> {
        let payload = fs::read(self.file_path(workbook_ref)).map_err(|error| {
            WorkbookRefStoreError::LoadWorkbook {
                workbook_ref: workbook_ref.to_string(),
                message: error.to_string(),
            }
        })?;
        serde_json::from_slice(&payload).map_err(|error| WorkbookRefStoreError::LoadWorkbook {
            workbook_ref: workbook_ref.to_string(),
            message: error.to_string(),
        })
    }

    // 2026-03-22: 这里统一拼接 workbook 草稿文件路径，目的是保证所有入口遵守同一命名规则。
    fn file_path(&self, workbook_ref: &str) -> PathBuf {
        self.root_dir.join(format!("{workbook_ref}.json"))
    }
}
