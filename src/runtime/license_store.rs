use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use rusqlite::{Connection, OptionalExtension, params};
use thiserror::Error;

use crate::runtime_paths::workspace_runtime_dir;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredLicenseState {
    pub license_key: String,
    pub license_key_masked: String,
    pub instance_id: String,
    pub instance_name: String,
    pub customer_email: Option<String>,
    pub store_id: Option<u64>,
    pub product_id: Option<u64>,
    pub variant_id: Option<u64>,
    pub license_status: String,
    pub last_validation_status: String,
    pub activated_at: String,
    pub validated_at: String,
    pub last_error: Option<String>,
}

#[derive(Debug, Error)]
pub enum LicenseStoreError {
    #[error("无法解析授权 runtime 路径: {0}")]
    ResolveRuntimePath(String),
    #[error("无法创建授权 runtime 目录: {0}")]
    CreateRuntimeDir(String),
    #[error("无法打开授权 SQLite: {0}")]
    OpenDatabase(String),
    #[error("无法初始化授权表结构: {0}")]
    BootstrapSchema(String),
    #[error("无法读取授权状态: {0}")]
    ReadState(String),
    #[error("无法写入授权状态: {0}")]
    SaveState(String),
    #[error("无法清空授权状态: {0}")]
    ClearState(String),
}

#[derive(Debug, Clone)]
pub struct LicenseStore {
    db_path: PathBuf,
}

impl LicenseStore {
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }

    pub fn workspace_default() -> Result<Self, LicenseStoreError> {
        // 2026-03-29 CST: 这里沿用当前 runtime.db 路径规则，原因是用户已经明确要继续走 Rust / exe / SQLite 主线；
        // 目的：让授权状态和现有本地运行时资产保持同一套落盘位置，减少维护分叉。
        if let Ok(path) = std::env::var("EXCEL_SKILL_RUNTIME_DB") {
            return Ok(Self::new(PathBuf::from(path)));
        }

        let runtime_dir = workspace_runtime_dir().map_err(LicenseStoreError::ResolveRuntimePath)?;
        Ok(Self::new(runtime_dir.join("runtime.db")))
    }

    pub fn load(&self) -> Result<Option<StoredLicenseState>, LicenseStoreError> {
        let connection = self.open_connection()?;
        connection
            .query_row(
                "SELECT license_key, license_key_masked, instance_id, instance_name, customer_email,
                        store_id, product_id, variant_id, license_status, last_validation_status,
                        activated_at, validated_at, last_error
                 FROM license_state
                 WHERE singleton_id = 1",
                [],
                |row| {
                    Ok(StoredLicenseState {
                        license_key: row.get(0)?,
                        license_key_masked: row.get(1)?,
                        instance_id: row.get(2)?,
                        instance_name: row.get(3)?,
                        customer_email: row.get(4)?,
                        store_id: row.get::<_, Option<i64>>(5)?.map(|value| value as u64),
                        product_id: row.get::<_, Option<i64>>(6)?.map(|value| value as u64),
                        variant_id: row.get::<_, Option<i64>>(7)?.map(|value| value as u64),
                        license_status: row.get(8)?,
                        last_validation_status: row.get(9)?,
                        activated_at: row.get(10)?,
                        validated_at: row.get(11)?,
                        last_error: row.get(12)?,
                    })
                },
            )
            .optional()
            .map_err(|error| LicenseStoreError::ReadState(error.to_string()))
    }

    pub fn save(&self, state: &StoredLicenseState) -> Result<(), LicenseStoreError> {
        let connection = self.open_connection()?;
        connection
            .execute(
                "INSERT INTO license_state (
                    singleton_id,
                    license_key,
                    license_key_masked,
                    instance_id,
                    instance_name,
                    customer_email,
                    store_id,
                    product_id,
                    variant_id,
                    license_status,
                    last_validation_status,
                    activated_at,
                    validated_at,
                    last_error
                 ) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                 ON CONFLICT(singleton_id) DO UPDATE SET
                    license_key = excluded.license_key,
                    license_key_masked = excluded.license_key_masked,
                    instance_id = excluded.instance_id,
                    instance_name = excluded.instance_name,
                    customer_email = excluded.customer_email,
                    store_id = excluded.store_id,
                    product_id = excluded.product_id,
                    variant_id = excluded.variant_id,
                    license_status = excluded.license_status,
                    last_validation_status = excluded.last_validation_status,
                    activated_at = excluded.activated_at,
                    validated_at = excluded.validated_at,
                    last_error = excluded.last_error,
                    updated_at = CURRENT_TIMESTAMP",
                params![
                    state.license_key,
                    state.license_key_masked,
                    state.instance_id,
                    state.instance_name,
                    state.customer_email,
                    state.store_id.map(|value| value as i64),
                    state.product_id.map(|value| value as i64),
                    state.variant_id.map(|value| value as i64),
                    state.license_status,
                    state.last_validation_status,
                    state.activated_at,
                    state.validated_at,
                    state.last_error,
                ],
            )
            .map_err(|error| LicenseStoreError::SaveState(error.to_string()))?;
        Ok(())
    }

    pub fn clear(&self) -> Result<(), LicenseStoreError> {
        let connection = self.open_connection()?;
        connection
            .execute("DELETE FROM license_state", [])
            .map_err(|error| LicenseStoreError::ClearState(error.to_string()))?;
        Ok(())
    }

    fn open_connection(&self) -> Result<Connection, LicenseStoreError> {
        if let Some(parent) = self.db_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| LicenseStoreError::CreateRuntimeDir(error.to_string()))?;
        }

        let connection = Connection::open(&self.db_path)
            .map_err(|error| LicenseStoreError::OpenDatabase(error.to_string()))?;
        connection
            .busy_timeout(Duration::from_secs(5))
            .map_err(|error| LicenseStoreError::OpenDatabase(error.to_string()))?;
        self.bootstrap_schema(&connection)?;
        Ok(connection)
    }

    fn bootstrap_schema(&self, connection: &Connection) -> Result<(), LicenseStoreError> {
        connection
            .execute_batch(
                "
                CREATE TABLE IF NOT EXISTS license_state (
                    singleton_id INTEGER PRIMARY KEY CHECK (singleton_id = 1),
                    license_key TEXT NOT NULL,
                    license_key_masked TEXT NOT NULL,
                    instance_id TEXT NOT NULL,
                    instance_name TEXT NOT NULL,
                    customer_email TEXT,
                    store_id INTEGER,
                    product_id INTEGER,
                    variant_id INTEGER,
                    license_status TEXT NOT NULL,
                    last_validation_status TEXT NOT NULL,
                    activated_at TEXT NOT NULL,
                    validated_at TEXT NOT NULL,
                    last_error TEXT,
                    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
                );
                ",
            )
            .map_err(|error| LicenseStoreError::BootstrapSchema(error.to_string()))
    }
}
