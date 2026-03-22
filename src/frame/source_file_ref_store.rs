use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::frame::table_ref_store::SourceFingerprint;

// 2026-03-23: 这里定义按序号暴露给后续流程复用的 Sheet 目录项，原因是入口链路可能不稳定传递中文 Sheet 名；目的是让后续 Tool 可以改走“第几个 Sheet”而不是重复依赖名称字符串。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedSourceSheet {
    pub sheet_index: usize,
    pub sheet_name: String,
}

// 2026-03-23: 这里定义源文件引用记录，原因是后续单表入口需要在不重复传中文路径和 Sheet 名的情况下继续；目的是把文件路径、实际工作路径与 Sheet 目录沉淀成可跨请求复用的最小快照。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedSourceFileRef {
    pub file_ref: String,
    pub original_path: String,
    pub working_path: String,
    pub recovery_applied: bool,
    pub sheets: Vec<PersistedSourceSheet>,
    pub source_fingerprint: SourceFingerprint,
}

impl PersistedSourceFileRef {
    // 2026-03-23: 这里从已打开文件构造持久化记录，原因是 open_workbook/list_sheets 成功后需要立即生成后续可复用入口；目的是让 Skill 后续直接按“第几个 Sheet”继续流转。
    pub fn from_opened_file(
        file_ref: String,
        original_path: &str,
        working_path: &str,
        sheet_names: &[String],
        recovery_applied: bool,
    ) -> Result<Self, SourceFileRefStoreError> {
        let sheets = sheet_names
            .iter()
            .enumerate()
            .map(|(index, sheet_name)| PersistedSourceSheet {
                // 2026-03-23: 这里采用 1-based 序号，原因是对外描述需要贴近“第 1 个 Sheet”的自然表达；目的是减少非 IT 用户的心智转换成本。
                sheet_index: index + 1,
                sheet_name: sheet_name.clone(),
            })
            .collect::<Vec<_>>();

        Ok(Self {
            file_ref,
            original_path: original_path.to_string(),
            working_path: working_path.to_string(),
            recovery_applied,
            sheets,
            source_fingerprint: fingerprint_for_path(working_path)?,
        })
    }

    // 2026-03-23: 这里按 Sheet 序号解析名称，原因是后续 Tool 将优先按“第几个 Sheet”继续；目的是在 Rust 进程内部再恢复真实 Sheet 名，避免外层重复传递中文字符串。
    pub fn sheet_name_for_index(&self, sheet_index: usize) -> Result<String, SourceFileRefStoreError> {
        self.sheets
            .iter()
            .find(|sheet| sheet.sheet_index == sheet_index)
            .map(|sheet| sheet.sheet_name.clone())
            .ok_or_else(|| SourceFileRefStoreError::MissingSheetIndex {
                file_ref: self.file_ref.clone(),
                sheet_index,
            })
    }

    // 2026-03-23: 这里校验当前工作文件是否仍与建档时一致，原因是 file_ref 可能跨请求被复用；目的是避免旧引用误用到已经被替换的新文件上。
    pub fn validate_source_unchanged(&self) -> Result<(), SourceFileRefStoreError> {
        let current = fingerprint_for_path(&self.working_path)?;
        if current != self.source_fingerprint {
            return Err(SourceFileRefStoreError::StaleFileRef {
                file_ref: self.file_ref.clone(),
            });
        }

        Ok(())
    }
}

// 2026-03-23: 这里定义 file_ref 存储错误，原因是需要把持久化、缺失与过期三类问题明确区分；目的是给上层返回可解释、可追踪的中文错误信息。
#[derive(Debug, Error)]
pub enum SourceFileRefStoreError {
    #[error("无法创建 file_ref 存储目录: {0}")]
    CreateStoreDir(String),
    #[error("无法确定当前工作目录，不能初始化 file_ref 存储: {0}")]
    ResolveWorkingDirectory(String),
    #[error("无法保存 file_ref `{file_ref}`: {message}")]
    SaveFileRef { file_ref: String, message: String },
    #[error("无法读取 file_ref `{file_ref}`: {message}")]
    LoadFileRef { file_ref: String, message: String },
    #[error("file_ref `{file_ref}` 对应的文件已变化，请重新打开文件后再继续")]
    StaleFileRef { file_ref: String },
    #[error("file_ref `{file_ref}` 中找不到第 {sheet_index} 个 Sheet")]
    MissingSheetIndex { file_ref: String, sheet_index: usize },
    #[error("无法读取 file_ref 源文件元数据: {0}")]
    ReadSourceMetadata(String),
}

// 2026-03-23: 这里定义 file_ref 磁盘存储入口，原因是打开文件后的上下文要跨请求延续；目的是把“文件 + Sheet 序号目录”沉淀到独立存储层而不影响既有 table_ref/workbook_ref 语义。
#[derive(Debug, Clone)]
pub struct SourceFileRefStore {
    root_dir: PathBuf,
}

impl SourceFileRefStore {
    // 2026-03-23: 这里允许显式指定存储目录，原因是测试与生产都需要复用同一套持久化逻辑；目的是保持存储层可测试且职责单一。
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }

    // 2026-03-23: 这里提供工作区默认目录，原因是真实 CLI 请求需要跨轮次复用 file_ref；目的是让打开文件后的上下文默认落盘到统一位置。
    pub fn workspace_default() -> Result<Self, SourceFileRefStoreError> {
        let current_dir = std::env::current_dir()
            .map_err(|error| SourceFileRefStoreError::ResolveWorkingDirectory(error.to_string()))?;
        Ok(Self::new(
            current_dir.join(".excel_skill_runtime").join("file_refs"),
        ))
    }

    // 2026-03-23: 这里统一生成 file_ref，原因是需要与 table_ref/result_ref/workbook_ref 做明显区分；目的是让 dispatcher 能稳定识别“这是一个打开过的文件引用”。
    pub fn create_file_ref() -> String {
        let timestamp = UNIX_EPOCH
            .elapsed()
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        format!("file_{}_{}", std::process::id(), timestamp)
    }

    // 2026-03-23: 这里持久化 file_ref 记录，原因是 open_workbook/list_sheets 之后的流程需要继续复用；目的是让后续调用只凭 file_ref + sheet_index 就能恢复文件上下文。
    pub fn save(&self, record: &PersistedSourceFileRef) -> Result<(), SourceFileRefStoreError> {
        fs::create_dir_all(&self.root_dir)
            .map_err(|error| SourceFileRefStoreError::CreateStoreDir(error.to_string()))?;
        let payload = serde_json::to_vec_pretty(record).map_err(|error| {
            SourceFileRefStoreError::SaveFileRef {
                file_ref: record.file_ref.clone(),
                message: error.to_string(),
            }
        })?;
        fs::write(self.file_path(&record.file_ref), payload).map_err(|error| {
            SourceFileRefStoreError::SaveFileRef {
                file_ref: record.file_ref.clone(),
                message: error.to_string(),
            }
        })
    }

    // 2026-03-23: 这里按 file_ref 读取记录，原因是后续 Tool 需要恢复“当前文件 + 第几个 Sheet”的内部上下文；目的是避免再次依赖路径和 Sheet 名字符串。
    pub fn load(&self, file_ref: &str) -> Result<PersistedSourceFileRef, SourceFileRefStoreError> {
        let payload = fs::read(self.file_path(file_ref)).map_err(|error| {
            SourceFileRefStoreError::LoadFileRef {
                file_ref: file_ref.to_string(),
                message: error.to_string(),
            }
        })?;
        serde_json::from_slice(&payload).map_err(|error| SourceFileRefStoreError::LoadFileRef {
            file_ref: file_ref.to_string(),
            message: error.to_string(),
        })
    }

    // 2026-03-23: 这里统一拼接 file_ref 文件路径，原因是所有调用方都要遵守同一命名规则；目的是保持持久化布局稳定且可预期。
    fn file_path(&self, file_ref: &str) -> PathBuf {
        self.root_dir.join(format!("{file_ref}.json"))
    }
}

// 2026-03-23: 这里复用文件指纹逻辑，原因是 file_ref 也需要做源文件变更校验；目的是保证跨请求复用时不会静默读取到已被替换的文件。
fn fingerprint_for_path(path: impl AsRef<Path>) -> Result<SourceFingerprint, SourceFileRefStoreError> {
    let metadata = fs::metadata(path.as_ref())
        .map_err(|error| SourceFileRefStoreError::ReadSourceMetadata(error.to_string()))?;
    let modified = metadata
        .modified()
        .map_err(|error| SourceFileRefStoreError::ReadSourceMetadata(error.to_string()))?;
    let modified_unix_ms = modified
        .duration_since(UNIX_EPOCH)
        .map_err(|error| SourceFileRefStoreError::ReadSourceMetadata(error.to_string()))?
        .as_millis();

    Ok(SourceFingerprint {
        file_size_bytes: metadata.len(),
        modified_unix_ms,
    })
}
