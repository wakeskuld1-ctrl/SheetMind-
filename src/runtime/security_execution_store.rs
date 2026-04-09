use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::{Connection, params};
use thiserror::Error;

use crate::runtime_paths::workspace_runtime_dir;
use crate::tools::contracts::{
    SecurityPositionPlanRecordResult, SecurityRecordPositionAdjustmentResult,
};

// 2026-04-08 CST: 这里新增证券执行层 runtime store，原因是仓位计划、调仓事件与投后复盘已经形成正式 ref 链；
// 目的：把 plan / adjustment 的落盘与回读统一收口到独立存储层，避免各个 Tool 各自手写 JSON 文件或重复拼接查询逻辑。
#[derive(Debug, Clone)]
pub struct SecurityExecutionStore {
    db_path: PathBuf,
}

// 2026-04-08 CST: 这里集中定义执行层存储错误，原因是 Task 6 同时涉及建库、写入、回读和 JSON 反序列化；
// 目的：为上层 Tool 返回清晰中文错误，并把执行链存储问题和业务规则问题明确分开。
#[derive(Debug, Error)]
pub enum SecurityExecutionStoreError {
    #[error("无法确定证券执行层 SQLite 所在目录: {0}")]
    ResolveRuntimeDir(String),
    #[error("无法创建证券执行层 SQLite 目录: {0}")]
    CreateRuntimeDir(String),
    #[error("无法打开证券执行层 SQLite: {0}")]
    OpenDatabase(String),
    #[error("无法初始化证券执行层表结构: {0}")]
    BootstrapSchema(String),
    #[error("无法写入仓位计划记录: {0}")]
    WritePositionPlan(String),
    #[error("无法读取仓位计划记录: {0}")]
    ReadPositionPlan(String),
    #[error("无法写入调仓事件记录: {0}")]
    WriteAdjustmentEvent(String),
    #[error("无法读取调仓事件记录: {0}")]
    ReadAdjustmentEvent(String),
    #[error("无法序列化证券执行层对象: {0}")]
    SerializePayload(String),
    #[error("无法反序列化证券执行层对象: {0}")]
    DeserializePayload(String),
}

impl SecurityExecutionStore {
    // 2026-04-08 CST: 这里允许显式指定执行层数据库路径，原因是测试隔离和后续多环境部署都可能需要自定义路径；
    // 目的：让执行层存储既能跟随 workspace 默认 runtime，也能在定向测试里落到独立目录。
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }

    // 2026-04-08 CST: 这里提供默认执行层数据库入口，原因是 plan / adjustment / review Tool 都需要共享同一份执行链事实源；
    // 目的：让三个 Tool 自动收敛到统一 runtime，而不是每个 Tool 单独维护自己的落盘路径。
    pub fn workspace_default() -> Result<Self, SecurityExecutionStoreError> {
        if let Ok(path) = std::env::var("EXCEL_SKILL_SECURITY_EXECUTION_DB") {
            return Ok(Self::new(PathBuf::from(path)));
        }

        let runtime_dir =
            workspace_runtime_dir().map_err(SecurityExecutionStoreError::ResolveRuntimeDir)?;
        Ok(Self::new(runtime_dir.join("security_execution.db")))
    }

    // 2026-04-08 CST: 这里暴露数据库路径，原因是后续测试和排障都需要确认执行链是否真实落盘；
    // 目的：让上层在必要时能直接核对 runtime 文件位置，减少“到底写到哪里去了”的排障成本。
    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    // 2026-04-08 CST: 这里统一打开执行层连接并自动建表，原因是当前三个 Tool 都应该共享同一套初始化逻辑；
    // 目的：避免 plan / adjustment / review 各自复制建库代码，确保 schema 只维护一份。
    pub fn open_connection(&self) -> Result<Connection, SecurityExecutionStoreError> {
        if let Some(parent) = self.db_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| SecurityExecutionStoreError::CreateRuntimeDir(error.to_string()))?;
        }

        let connection = Connection::open(&self.db_path)
            .map_err(|error| SecurityExecutionStoreError::OpenDatabase(error.to_string()))?;
        connection
            .busy_timeout(Duration::from_secs(5))
            .map_err(|error| SecurityExecutionStoreError::OpenDatabase(error.to_string()))?;
        bootstrap_schema(&connection)?;
        Ok(connection)
    }

    // 2026-04-08 CST: 这里补仓位计划记录落盘，原因是投后复盘必须能只凭 position_plan_ref 回读正式计划对象；
    // 目的：让 position_plan_record 不再只是回声式 Tool，而是真正成为后续执行与复盘的锚点。
    pub fn upsert_position_plan(
        &self,
        record: &SecurityPositionPlanRecordResult,
    ) -> Result<(), SecurityExecutionStoreError> {
        let payload = serde_json::to_string(record)
            .map_err(|error| SecurityExecutionStoreError::SerializePayload(error.to_string()))?;
        let connection = self.open_connection()?;
        connection
            .execute(
                "INSERT INTO security_position_plan_records (
                    position_plan_ref,
                    symbol,
                    analysis_date,
                    decision_ref,
                    approval_ref,
                    evidence_version,
                    payload_json
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                ON CONFLICT(position_plan_ref) DO UPDATE SET
                    symbol = excluded.symbol,
                    analysis_date = excluded.analysis_date,
                    decision_ref = excluded.decision_ref,
                    approval_ref = excluded.approval_ref,
                    evidence_version = excluded.evidence_version,
                    payload_json = excluded.payload_json,
                    updated_at = CURRENT_TIMESTAMP",
                params![
                    record.position_plan_ref,
                    record.symbol,
                    record.analysis_date,
                    record.decision_ref,
                    record.approval_ref,
                    record.evidence_version,
                    payload,
                ],
            )
            .map_err(|error| SecurityExecutionStoreError::WritePositionPlan(error.to_string()))?;
        Ok(())
    }

    // 2026-04-08 CST: 这里补仓位计划回读，原因是 post_trade_review 需要只凭 ref 从执行层恢复正式计划事实；
    // 目的：让复盘链路不依赖调用方重复携带完整计划 payload，保持正式引用对象语义。
    pub fn load_position_plan(
        &self,
        position_plan_ref: &str,
    ) -> Result<Option<SecurityPositionPlanRecordResult>, SecurityExecutionStoreError> {
        let connection = self.open_connection()?;
        let mut statement = connection
            .prepare(
                "SELECT payload_json
                 FROM security_position_plan_records
                 WHERE position_plan_ref = ?1
                 LIMIT 1",
            )
            .map_err(|error| SecurityExecutionStoreError::ReadPositionPlan(error.to_string()))?;
        let mut rows = statement
            .query(params![position_plan_ref])
            .map_err(|error| SecurityExecutionStoreError::ReadPositionPlan(error.to_string()))?;

        let Some(row) = rows
            .next()
            .map_err(|error| SecurityExecutionStoreError::ReadPositionPlan(error.to_string()))?
        else {
            return Ok(None);
        };

        let payload: String = row
            .get(0)
            .map_err(|error| SecurityExecutionStoreError::ReadPositionPlan(error.to_string()))?;
        serde_json::from_str::<SecurityPositionPlanRecordResult>(&payload)
            .map(Some)
            .map_err(|error| SecurityExecutionStoreError::DeserializePayload(error.to_string()))
    }

    // 2026-04-08 CST: 这里补调仓事件落盘，原因是投后复盘要从 adjustment_event_ref 链接回每次实际执行动作；
    // 目的：让 security_record_position_adjustment 产出的正式事件对象能被后续聚合与审计反查，而不是停留在单次响应里。
    pub fn upsert_adjustment_event(
        &self,
        record: &SecurityRecordPositionAdjustmentResult,
    ) -> Result<(), SecurityExecutionStoreError> {
        let payload = serde_json::to_string(record)
            .map_err(|error| SecurityExecutionStoreError::SerializePayload(error.to_string()))?;
        let connection = self.open_connection()?;
        connection
            .execute(
                "INSERT INTO security_position_adjustment_events (
                    adjustment_event_ref,
                    position_plan_ref,
                    symbol,
                    event_date,
                    decision_ref,
                    approval_ref,
                    evidence_version,
                    payload_json
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                ON CONFLICT(adjustment_event_ref) DO UPDATE SET
                    position_plan_ref = excluded.position_plan_ref,
                    symbol = excluded.symbol,
                    event_date = excluded.event_date,
                    decision_ref = excluded.decision_ref,
                    approval_ref = excluded.approval_ref,
                    evidence_version = excluded.evidence_version,
                    payload_json = excluded.payload_json,
                    updated_at = CURRENT_TIMESTAMP",
                params![
                    record.adjustment_event_ref,
                    record.position_plan_ref,
                    record.symbol,
                    record.event_date,
                    record.decision_ref,
                    record.approval_ref,
                    record.evidence_version,
                    payload,
                ],
            )
            .map_err(|error| SecurityExecutionStoreError::WriteAdjustmentEvent(error.to_string()))?;
        Ok(())
    }

    // 2026-04-08 CST: 这里补调仓事件回读，原因是复盘 Tool 需要顺着多条 adjustment_event_ref 回收完整事件链；
    // 目的：把事件恢复逻辑集中在存储层，避免复盘 Tool 再自己拼 SQL 或手工解析落盘 JSON。
    pub fn load_adjustment_event(
        &self,
        adjustment_event_ref: &str,
    ) -> Result<Option<SecurityRecordPositionAdjustmentResult>, SecurityExecutionStoreError> {
        let connection = self.open_connection()?;
        let mut statement = connection
            .prepare(
                "SELECT payload_json
                 FROM security_position_adjustment_events
                 WHERE adjustment_event_ref = ?1
                 LIMIT 1",
            )
            .map_err(|error| SecurityExecutionStoreError::ReadAdjustmentEvent(error.to_string()))?;
        let mut rows = statement
            .query(params![adjustment_event_ref])
            .map_err(|error| SecurityExecutionStoreError::ReadAdjustmentEvent(error.to_string()))?;

        let Some(row) = rows
            .next()
            .map_err(|error| SecurityExecutionStoreError::ReadAdjustmentEvent(error.to_string()))?
        else {
            return Ok(None);
        };

        let payload: String = row
            .get(0)
            .map_err(|error| SecurityExecutionStoreError::ReadAdjustmentEvent(error.to_string()))?;
        serde_json::from_str::<SecurityRecordPositionAdjustmentResult>(&payload)
            .map(Some)
            .map_err(|error| SecurityExecutionStoreError::DeserializePayload(error.to_string()))
    }
}

// 2026-04-08 CST: 这里集中维护执行层 schema，原因是仓位计划与调仓事件已经构成一条稳定执行链；
// 目的：让后续复盘聚合和审计回查都复用同一份表结构，而不是临时新增零散文件存储。
fn bootstrap_schema(connection: &Connection) -> Result<(), SecurityExecutionStoreError> {
    connection
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS security_position_plan_records (
                position_plan_ref TEXT PRIMARY KEY,
                symbol TEXT NOT NULL,
                analysis_date TEXT NOT NULL,
                decision_ref TEXT NOT NULL,
                approval_ref TEXT NOT NULL,
                evidence_version TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE IF NOT EXISTS security_position_adjustment_events (
                adjustment_event_ref TEXT PRIMARY KEY,
                position_plan_ref TEXT NOT NULL,
                symbol TEXT NOT NULL,
                event_date TEXT NOT NULL,
                decision_ref TEXT NOT NULL,
                approval_ref TEXT NOT NULL,
                evidence_version TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            CREATE INDEX IF NOT EXISTS idx_security_position_plan_symbol_date
                ON security_position_plan_records(symbol, analysis_date);
            CREATE INDEX IF NOT EXISTS idx_security_adjustment_plan_date
                ON security_position_adjustment_events(position_plan_ref, event_date);",
        )
        .map_err(|error| SecurityExecutionStoreError::BootstrapSchema(error.to_string()))?;
    Ok(())
}
