use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::schema::{ConfidenceLevel, HeaderColumn, HeaderInference, SchemaState};
use crate::runtime_paths::workspace_runtime_dir;

// 2026-03-22: 这里定义源文件指纹，目的是在复用 table_ref 前先校验 Excel 是否已变化，避免旧确认态误用到新文件。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceFingerprint {
    pub file_size_bytes: u64,
    pub modified_unix_ms: u128,
}

// 2026-03-22: 这里定义可持久化的 table_ref 记录，目的是把确认过的整表/局部区域都沉淀成跨请求可复用句柄。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedTableRef {
    pub table_ref: String,
    pub source_path: String,
    pub sheet_name: String,
    // 2026-03-22: 这里保留显式区域，目的是让 load_table_region 生成的局部确认态可以被 preview/分析层原样回放。
    #[serde(default)]
    pub region: Option<String>,
    pub columns: Vec<String>,
    pub header_row_count: usize,
    pub data_start_row_index: usize,
    pub source_fingerprint: SourceFingerprint,
}

impl PersistedTableRef {
    // 2026-03-22: 这里从 confirmed schema 构造整表 table_ref，目的是保持既有 apply_header_schema 行为不变。
    pub fn from_confirmed_schema(
        table_ref: String,
        path: &str,
        sheet_name: &str,
        inference: &HeaderInference,
    ) -> Result<Self, TableRefStoreError> {
        Ok(Self {
            table_ref,
            source_path: path.to_string(),
            sheet_name: sheet_name.to_string(),
            region: None,
            columns: inference
                .columns
                .iter()
                .map(|column| column.canonical_name.clone())
                .collect(),
            header_row_count: inference.header_row_count,
            data_start_row_index: inference.data_start_row_index,
            source_fingerprint: fingerprint_for_path(path)?,
        })
    }

    // 2026-03-22: 这里为显式区域构造 table_ref，目的是把 inspect -> load region 的确认态正式升级成可持久化入口。
    pub fn from_region(
        table_ref: String,
        path: &str,
        sheet_name: &str,
        region: &str,
        columns: Vec<String>,
        header_row_count: usize,
    ) -> Result<Self, TableRefStoreError> {
        Ok(Self {
            table_ref,
            source_path: path.to_string(),
            sheet_name: sheet_name.to_string(),
            region: Some(region.to_string()),
            columns,
            // 2026-03-22: 这里把局部区域的数据起始行固定为 header_row_count，目的是让持久化元数据语义保持完整。
            data_start_row_index: header_row_count,
            header_row_count,
            source_fingerprint: fingerprint_for_path(path)?,
        })
    }

    // 2026-03-22: 这里提供测试专用构造器，目的是让持久化 round-trip 与 region 回放测试不必依赖 dispatcher 预热。
    pub fn new_for_test(
        table_ref: &str,
        path: &str,
        sheet_name: &str,
        columns: Vec<String>,
        header_row_count: usize,
        data_start_row_index: usize,
        region: Option<String>,
    ) -> Self {
        Self {
            table_ref: table_ref.to_string(),
            source_path: path.to_string(),
            sheet_name: sheet_name.to_string(),
            region,
            columns,
            header_row_count,
            data_start_row_index,
            source_fingerprint: fingerprint_for_path(path).unwrap_or(SourceFingerprint {
                file_size_bytes: 0,
                modified_unix_ms: 0,
            }),
        }
    }

    // 2026-03-22: 这里把持久化记录还原成 confirmed inference，目的是让整表 table_ref 继续复用既有加载主路径。
    pub fn to_confirmed_inference(&self) -> HeaderInference {
        HeaderInference {
            columns: self
                .columns
                .iter()
                .map(|column| HeaderColumn {
                    header_path: vec![column.clone()],
                    canonical_name: column.clone(),
                })
                .collect(),
            confidence: ConfidenceLevel::High,
            schema_state: SchemaState::Confirmed,
            header_row_count: self.header_row_count,
            data_start_row_index: self.data_start_row_index,
        }
    }

    // 2026-03-22: 这里显式标记当前句柄是否来自局部区域，目的是让回放层走对整表/局部两条加载分支。
    pub fn is_region_ref(&self) -> bool {
        self.region.is_some()
    }

    // 2026-03-22: 这里校验当前源文件是否与确认时一致，目的是在 Excel 被修改后直接拒绝旧 table_ref。
    pub fn validate_source_unchanged(&self) -> Result<(), TableRefStoreError> {
        let current = fingerprint_for_path(&self.source_path)?;
        if current != self.source_fingerprint {
            return Err(TableRefStoreError::StaleTableRef {
                table_ref: self.table_ref.clone(),
            });
        }

        Ok(())
    }
}

// 2026-03-22: 这里定义 table_ref 存储错误，目的是把持久化失败、缺失与过期句柄转成上层可解释中文错误。
#[derive(Debug, Error)]
pub enum TableRefStoreError {
    #[error("无法创建 table_ref 存储目录: {0}")]
    CreateStoreDir(String),
    #[error("无法读取 table_ref `{table_ref}`: {message}")]
    LoadTableRef { table_ref: String, message: String },
    #[error("无法保存 table_ref `{table_ref}`: {message}")]
    SaveTableRef { table_ref: String, message: String },
    #[error("table_ref `{table_ref}` 对应的源文件已变化，请重新确认表头后再继续分析建模")]
    StaleTableRef { table_ref: String },
    #[error("无法读取 table_ref 源文件元数据: {0}")]
    ReadSourceMetadata(String),
    #[error("无法确定当前工作目录，不能初始化 table_ref 存储: {0}")]
    ResolveWorkingDirectory(String),
}

// 2026-03-22: 这里定义磁盘存储入口，目的是把 table_ref 的保存与读取统一收口成独立模块。
#[derive(Debug, Clone)]
pub struct TableRefStore {
    root_dir: PathBuf,
}

impl TableRefStore {
    // 2026-03-22: 这里允许测试显式传目录，目的是让 round-trip 测试可以在受控路径下运行。
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }

    // 2026-03-22: 这里提供工作区默认目录，目的是让真实 CLI 请求能在同一工作目录下跨请求复用 table_ref。
    pub fn workspace_default() -> Result<Self, TableRefStoreError> {
        let runtime_dir =
            workspace_runtime_dir().map_err(TableRefStoreError::ResolveWorkingDirectory)?;
        Ok(Self::new(runtime_dir.join("table_refs")))
    }

    // 2026-03-22: 这里持久化 table_ref 记录，目的是让确认态真正落盘，而不是只停留在进程内存里。
    pub fn save(&self, record: &PersistedTableRef) -> Result<(), TableRefStoreError> {
        fs::create_dir_all(&self.root_dir)
            .map_err(|error| TableRefStoreError::CreateStoreDir(error.to_string()))?;
        let payload = serde_json::to_vec_pretty(record).map_err(|error| {
            TableRefStoreError::SaveTableRef {
                table_ref: record.table_ref.clone(),
                message: error.to_string(),
            }
        })?;
        fs::write(self.file_path(&record.table_ref), payload).map_err(|error| {
            TableRefStoreError::SaveTableRef {
                table_ref: record.table_ref.clone(),
                message: error.to_string(),
            }
        })
    }

    // 2026-03-22: 这里从磁盘读取 table_ref 记录，目的是让后续请求只凭句柄就能恢复确认态上下文。
    pub fn load(&self, table_ref: &str) -> Result<PersistedTableRef, TableRefStoreError> {
        let payload = fs::read(self.file_path(table_ref)).map_err(|error| {
            TableRefStoreError::LoadTableRef {
                table_ref: table_ref.to_string(),
                message: error.to_string(),
            }
        })?;
        serde_json::from_slice(&payload).map_err(|error| TableRefStoreError::LoadTableRef {
            table_ref: table_ref.to_string(),
            message: error.to_string(),
        })
    }

    // 2026-03-22: 这里统一生成新 table_ref，目的是避免上层手写 ID 规则导致不一致。
    pub fn create_table_ref() -> String {
        let timestamp = UNIX_EPOCH
            .elapsed()
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        format!("table_{}_{}", std::process::id(), timestamp)
    }

    // 2026-03-22: 这里统一计算记录文件路径，目的是让保存和读取共享同一命名规则。
    fn file_path(&self, table_ref: &str) -> PathBuf {
        self.root_dir.join(format!("{table_ref}.json"))
    }
}

// 2026-03-22: 这里抽出源文件指纹计算，目的是让保存与校验复用同一套元数据采集逻辑。
fn fingerprint_for_path(path: impl AsRef<Path>) -> Result<SourceFingerprint, TableRefStoreError> {
    let metadata = fs::metadata(path.as_ref())
        .map_err(|error| TableRefStoreError::ReadSourceMetadata(error.to_string()))?;
    let modified = metadata
        .modified()
        .map_err(|error| TableRefStoreError::ReadSourceMetadata(error.to_string()))?;
    let modified_unix_ms = modified
        .duration_since(UNIX_EPOCH)
        .map_err(|error| TableRefStoreError::ReadSourceMetadata(error.to_string()))?
        .as_millis();

    Ok(SourceFingerprint {
        file_size_bytes: metadata.len(),
        modified_unix_ms,
    })
}
