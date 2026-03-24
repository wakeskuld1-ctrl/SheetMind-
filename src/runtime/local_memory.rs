use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::frame::table_ref_store::PersistedTableRef;
use crate::runtime_paths::workspace_runtime_dir;

// 2026-03-22: 这里定义会话阶段枚举，目的是把 orchestrator 三层路由状态收口成稳定的本地持久值。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SessionStage {
    #[default]
    Unknown,
    TableProcessing,
    AnalysisModeling,
    DecisionAssistant,
}

impl SessionStage {
    // 2026-03-22: 这里统一阶段枚举的数据库值，目的是避免 dispatcher 和 runtime 各自拼接字符串造成不一致。
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::TableProcessing => "table_processing",
            Self::AnalysisModeling => "analysis_modeling",
            Self::DecisionAssistant => "decision_assistant",
        }
    }

    // 2026-03-22: 这里集中做数据库字符串到枚举的映射，目的是让异常值在运行时层就能被明确拦截。
    fn from_db_value(value: &str) -> Result<Self, LocalMemoryError> {
        match value {
            "unknown" => Ok(Self::Unknown),
            "table_processing" => Ok(Self::TableProcessing),
            "analysis_modeling" => Ok(Self::AnalysisModeling),
            "decision_assistant" => Ok(Self::DecisionAssistant),
            _ => Err(LocalMemoryError::InvalidStage(value.to_string())),
        }
    }
}

// 2026-03-22: 这里定义 schema 摘要状态，目的是把“未知 / 待确认 / 已确认”这三个门禁态持久化到本地 runtime。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SchemaStatus {
    #[default]
    Unknown,
    PendingConfirmation,
    Confirmed,
}

impl SchemaStatus {
    // 2026-03-22: 这里统一 schema 状态的数据库值，目的是让 Tool 自动同步和 Skill 显式读写使用同一套编码。
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::PendingConfirmation => "pending_confirmation",
            Self::Confirmed => "confirmed",
        }
    }

    // 2026-03-22: 这里集中做数据库字符串到 schema 状态的映射，目的是在 runtime 层提前发现脏数据。
    fn from_db_value(value: &str) -> Result<Self, LocalMemoryError> {
        match value {
            "unknown" => Ok(Self::Unknown),
            "pending_confirmation" => Ok(Self::PendingConfirmation),
            "confirmed" => Ok(Self::Confirmed),
            _ => Err(LocalMemoryError::InvalidSchemaStatus(value.to_string())),
        }
    }
}

// 2026-03-22: 这里定义对外暴露的会话状态结构，目的是让 orchestrator 读到的状态字段与 Skill 协议保持一致。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionState {
    pub session_id: String,
    pub current_workbook: Option<String>,
    pub current_sheet: Option<String>,
    // 2026-03-23: ?????????????????????????????????????????? Skill ???????? Sheet ????????????????
    pub current_file_ref: Option<String>,
    // 2026-03-23: ?????? Sheet ??????????????????? Sheet ????????????????? Sheet??
    pub current_sheet_index: Option<usize>,
    pub current_stage: SessionStage,
    pub schema_status: SchemaStatus,
    pub active_table_ref: Option<String>,
    // 2026-03-23: 这里新增显式激活句柄引用，原因是 table_ref/result_ref/workbook_ref 已经共享同一条链式执行主路径；目的是让会话状态能准确指向“当前最新结果”而不再复用 active_table_ref 承载多重语义。
    pub active_handle_ref: Option<String>,
    // 2026-03-23: 这里新增激活句柄类型，原因是上层 Skill 需要直接知道当前激活对象属于哪类句柄；目的是减少仅靠前缀推断带来的歧义并增强可解释性。
    pub active_handle_kind: Option<String>,
    pub last_user_goal: Option<String>,
    pub selected_columns: Vec<String>,
}

impl SessionState {
    // 2026-03-22: 这里提供默认空状态构造器，目的是让首次读取 session 时也能返回结构稳定的摘要对象。
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            current_workbook: None,
            current_sheet: None,
            current_file_ref: None,
            current_sheet_index: None,
            current_stage: SessionStage::Unknown,
            schema_status: SchemaStatus::Unknown,
            active_table_ref: None,
            active_handle_ref: None,
            active_handle_kind: None,
            last_user_goal: None,
            selected_columns: Vec::new(),
        }
    }
}

// 2026-03-22: 这里定义会话状态 patch，目的是支持 orchestrator 和关键 Tool 只更新自己关心的字段。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SessionStatePatch {
    pub current_workbook: Option<String>,
    pub current_sheet: Option<String>,
    // 2026-03-23: ????? patch ??????????????????? Tool ???? file_ref ??????????? orchestrator ?????????????
    pub current_file_ref: Option<String>,
    // 2026-03-23: ????? patch ???? Sheet ?????????????? Sheet?????????????????? Sheet ??
    pub current_sheet_index: Option<usize>,
    pub current_stage: Option<SessionStage>,
    pub schema_status: Option<SchemaStatus>,
    pub active_table_ref: Option<String>,
    // 2026-03-23: 这里允许 patch 单独更新激活句柄，原因是会产生新 result_ref/workbook_ref 的 Tool 需要只回写最新输出对象；目的是把输入确认态 table_ref 与输出中间句柄解耦。
    pub active_handle_ref: Option<String>,
    // 2026-03-23: 这里允许 patch 显式标注句柄类型，原因是会话摘要需要稳定暴露句柄类别；目的是避免 dispatcher 和 runtime 各自重复猜测类型。
    pub active_handle_kind: Option<String>,
    pub last_user_goal: Option<String>,
    pub selected_columns: Option<Vec<String>>,
}

// 2026-03-22: 这里定义本地事件日志输入，目的是给关键状态切换留下最小留痕而不把全文结果塞进数据库。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventLogInput {
    pub event_type: String,
    pub stage: Option<SessionStage>,
    pub tool_name: Option<String>,
    pub status: String,
    pub message: Option<String>,
}

// 2026-03-22: 这里定义 runtime 错误类型，目的是把目录、建库、序列化和脏数据问题明确区分出来。
#[derive(Debug, Error)]
pub enum LocalMemoryError {
    #[error("无法确定当前工作目录，不能初始化本地记忆层: {0}")]
    ResolveWorkingDirectory(String),
    #[error("无法创建本地记忆层目录: {0}")]
    CreateRuntimeDir(String),
    #[error("无法打开本地记忆库: {0}")]
    OpenDatabase(String),
    #[error("无法初始化本地记忆库结构: {0}")]
    BootstrapSchema(String),
    #[error("无法读取会话状态: {0}")]
    ReadSessionState(String),
    #[error("无法写入会话状态: {0}")]
    UpdateSessionState(String),
    #[error("无法镜像 table_ref 元数据: {0}")]
    MirrorTableRef(String),
    #[error("无法写入事件日志: {0}")]
    AppendEvent(String),
    #[error("无法序列化本地记忆层数据: {0}")]
    SerializePayload(String),
    #[error("无法反序列化本地记忆层数据: {0}")]
    DeserializePayload(String),
    #[error("本地记忆层出现未知阶段值: {0}")]
    InvalidStage(String),
    #[error("本地记忆层出现未知 schema 状态值: {0}")]
    InvalidSchemaStatus(String),
}

// 2026-03-22: 这里定义 SQLite runtime 入口，目的是把 session_state、table_ref 镜像和事件留痕统一收口到独立模块。
#[derive(Debug, Clone)]
pub struct LocalMemoryRuntime {
    db_path: PathBuf,
}

impl LocalMemoryRuntime {
    // 2026-03-22: 这里允许显式传入数据库路径，目的是让测试能隔离 runtime 文件，也让未来二进制可定制落盘位置。
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }

    // 2026-03-22: 这里提供工作区默认 runtime 路径，目的是让当前仓库开发态和 CLI 真实请求共享同一套本地记忆层。
    pub fn workspace_default() -> Result<Self, LocalMemoryError> {
        if let Ok(path) = std::env::var("EXCEL_SKILL_RUNTIME_DB") {
            return Ok(Self::new(PathBuf::from(path)));
        }

        let runtime_dir =
            workspace_runtime_dir().map_err(LocalMemoryError::ResolveWorkingDirectory)?;
        Ok(Self::new(runtime_dir.join("runtime.db")))
    }

    // 2026-03-22: 这里读取会话状态，目的是让 orchestrator 能在每轮开始时拿到真实本地摘要而不是依赖大模型记忆。
    pub fn get_session_state(&self, session_id: &str) -> Result<SessionState, LocalMemoryError> {
        let connection = self.open_connection()?;
        let row = connection
            .query_row(
                "SELECT current_workbook, current_sheet, current_file_ref, current_sheet_index, current_stage, schema_status, active_table_ref, active_handle_ref, active_handle_kind, last_user_goal, selected_columns_json FROM session_state WHERE session_id = ?1",
                [session_id],
                |row| {
                    let stage = SessionStage::from_db_value(row.get::<_, String>(4)?.as_str())
                        .map_err(to_sql_error)?;
                    let schema_status =
                        SchemaStatus::from_db_value(row.get::<_, String>(5)?.as_str())
                            .map_err(to_sql_error)?;
                    let selected_columns_json: String = row.get(10)?;
                    let selected_columns =
                        serde_json::from_str::<Vec<String>>(&selected_columns_json)
                            .map_err(|error| {
                                to_sql_error(LocalMemoryError::DeserializePayload(
                                    error.to_string(),
                                ))
                            })?;

                    Ok(SessionState {
                        session_id: session_id.to_string(),
                        current_workbook: row.get(0)?,
                        current_sheet: row.get(1)?,
                        current_file_ref: row.get(2)?,
                        current_sheet_index: row.get::<_, Option<i64>>(3)?.map(|value| value as usize),
                        current_stage: stage,
                        schema_status,
                        active_table_ref: row.get(6)?,
                        active_handle_ref: row.get(7)?,
                        active_handle_kind: row.get(8)?,
                        last_user_goal: row.get(9)?,
                        selected_columns,
                    })
                },
            )
            .optional()
            .map_err(|error| LocalMemoryError::ReadSessionState(error.to_string()))?;

        Ok(row.unwrap_or_else(|| SessionState::new(session_id)))
    }

    // 2026-03-22: 这里按 patch 更新会话状态，目的是让 Skill 和 Tool 都能只写自己本轮真正掌握的那部分上下文。
    pub fn update_session_state(
        &self,
        session_id: &str,
        patch: &SessionStatePatch,
    ) -> Result<SessionState, LocalMemoryError> {
        let mut next_state = self.get_session_state(session_id)?;
        if let Some(current_workbook) = &patch.current_workbook {
            next_state.current_workbook = Some(current_workbook.clone());
        }
        if let Some(current_sheet) = &patch.current_sheet {
            next_state.current_sheet = Some(current_sheet.clone());
        }
        if let Some(current_file_ref) = &patch.current_file_ref {
            next_state.current_file_ref = Some(current_file_ref.clone());
        }
        if let Some(current_sheet_index) = patch.current_sheet_index {
            next_state.current_sheet_index = Some(current_sheet_index);
        }
        if let Some(current_stage) = &patch.current_stage {
            next_state.current_stage = current_stage.clone();
        }
        if let Some(schema_status) = &patch.schema_status {
            next_state.schema_status = schema_status.clone();
        }
        if let Some(active_table_ref) = &patch.active_table_ref {
            next_state.active_table_ref = Some(active_table_ref.clone());
        }
        if let Some(active_handle_ref) = &patch.active_handle_ref {
            next_state.active_handle_ref = Some(active_handle_ref.clone());
        }
        if let Some(active_handle_kind) = &patch.active_handle_kind {
            next_state.active_handle_kind = Some(active_handle_kind.clone());
        }
        if let Some(last_user_goal) = &patch.last_user_goal {
            next_state.last_user_goal = Some(last_user_goal.clone());
        }
        if let Some(selected_columns) = &patch.selected_columns {
            next_state.selected_columns = selected_columns.clone();
        }

        let connection = self.open_connection()?;
        let selected_columns_json = serde_json::to_string(&next_state.selected_columns)
            .map_err(|error| LocalMemoryError::SerializePayload(error.to_string()))?;
        connection
            .execute(
                "INSERT INTO sessions (session_id, status) VALUES (?1, 'active')
                 ON CONFLICT(session_id) DO UPDATE SET updated_at = CURRENT_TIMESTAMP",
                [session_id],
            )
            .map_err(|error| LocalMemoryError::UpdateSessionState(error.to_string()))?;
        connection
            .execute(
                "INSERT INTO session_state (
                    session_id,
                    current_workbook,
                    current_sheet,
                    current_file_ref,
                    current_sheet_index,
                    current_stage,
                    schema_status,
                    active_table_ref,
                    active_handle_ref,
                    active_handle_kind,
                    last_user_goal,
                    selected_columns_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                 ON CONFLICT(session_id) DO UPDATE SET
                    current_workbook = excluded.current_workbook,
                    current_sheet = excluded.current_sheet,
                    current_file_ref = excluded.current_file_ref,
                    current_sheet_index = excluded.current_sheet_index,
                    current_stage = excluded.current_stage,
                    schema_status = excluded.schema_status,
                    active_table_ref = excluded.active_table_ref,
                    active_handle_ref = excluded.active_handle_ref,
                    active_handle_kind = excluded.active_handle_kind,
                    last_user_goal = excluded.last_user_goal,
                    selected_columns_json = excluded.selected_columns_json,
                    updated_at = CURRENT_TIMESTAMP",
                params![
                    next_state.session_id,
                    next_state.current_workbook,
                    next_state.current_sheet,
                    next_state.current_file_ref,
                    next_state.current_sheet_index.map(|value| value as i64),
                    next_state.current_stage.as_str(),
                    next_state.schema_status.as_str(),
                    next_state.active_table_ref,
                    next_state.active_handle_ref,
                    next_state.active_handle_kind,
                    next_state.last_user_goal,
                    selected_columns_json,
                ],
            )
            .map_err(|error| LocalMemoryError::UpdateSessionState(error.to_string()))?;

        Ok(next_state)
    }

    // 2026-03-22: 这里镜像确认态 table_ref 元数据，目的是让后续 orchestrator 和审计层能直接从 SQLite 了解当前激活句柄来源。
    pub fn mirror_table_ref(&self, record: &PersistedTableRef) -> Result<(), LocalMemoryError> {
        let connection = self.open_connection()?;
        let columns_json = serde_json::to_string(&record.columns)
            .map_err(|error| LocalMemoryError::SerializePayload(error.to_string()))?;
        let fingerprint_json = serde_json::to_string(&record.source_fingerprint)
            .map_err(|error| LocalMemoryError::SerializePayload(error.to_string()))?;

        connection
            .execute(
                "INSERT INTO table_refs (
                    table_ref,
                    source_path,
                    sheet_name,
                    region,
                    columns_json,
                    header_row_count,
                    data_start_row_index,
                    source_fingerprint_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                 ON CONFLICT(table_ref) DO UPDATE SET
                    source_path = excluded.source_path,
                    sheet_name = excluded.sheet_name,
                    region = excluded.region,
                    columns_json = excluded.columns_json,
                    header_row_count = excluded.header_row_count,
                    data_start_row_index = excluded.data_start_row_index,
                    source_fingerprint_json = excluded.source_fingerprint_json,
                    last_used_at = CURRENT_TIMESTAMP",
                params![
                    record.table_ref,
                    record.source_path,
                    record.sheet_name,
                    record.region,
                    columns_json,
                    record.header_row_count as i64,
                    record.data_start_row_index as i64,
                    fingerprint_json,
                ],
            )
            .map_err(|error| LocalMemoryError::MirrorTableRef(error.to_string()))?;

        Ok(())
    }

    // 2026-03-22: 这里写入轻量事件日志，目的是为跨层状态切换提供最小留痕，而不是把大段结果全文写进数据库。
    pub fn append_event(
        &self,
        session_id: &str,
        event: &EventLogInput,
    ) -> Result<(), LocalMemoryError> {
        let connection = self.open_connection()?;
        connection
            .execute(
                "INSERT INTO event_logs (
                    session_id,
                    event_type,
                    stage,
                    tool_name,
                    status,
                    message
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    session_id,
                    event.event_type,
                    event.stage.as_ref().map(SessionStage::as_str),
                    event.tool_name,
                    event.status,
                    event.message,
                ],
            )
            .map_err(|error| LocalMemoryError::AppendEvent(error.to_string()))?;

        Ok(())
    }

    // 2026-03-22: 这里统一打开并初始化 SQLite 连接，目的是让所有读写入口共享同一套建库与超时设置。
    fn open_connection(&self) -> Result<Connection, LocalMemoryError> {
        if let Some(parent) = self.db_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| LocalMemoryError::CreateRuntimeDir(error.to_string()))?;
        }

        let connection = Connection::open(&self.db_path)
            .map_err(|error| LocalMemoryError::OpenDatabase(error.to_string()))?;
        connection
            .busy_timeout(Duration::from_secs(5))
            .map_err(|error| LocalMemoryError::OpenDatabase(error.to_string()))?;
        self.bootstrap_schema(&connection)?;
        Ok(connection)
    }

    // 2026-03-22: 这里集中初始化最小 runtime 表结构，目的是让 session_state、table_refs 和事件日志在首次使用时自动就绪。
    fn bootstrap_schema(&self, connection: &Connection) -> Result<(), LocalMemoryError> {
        connection
            .execute_batch(
                "
                CREATE TABLE IF NOT EXISTS sessions (
                    session_id TEXT PRIMARY KEY,
                    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    status TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS session_state (
                    session_id TEXT PRIMARY KEY,
                    current_workbook TEXT,
                    current_sheet TEXT,
                    current_file_ref TEXT,
                    current_sheet_index INTEGER,
                    current_stage TEXT NOT NULL,
                    schema_status TEXT NOT NULL,
                    active_table_ref TEXT,
                    active_handle_ref TEXT,
                    active_handle_kind TEXT,
                    last_user_goal TEXT,
                    selected_columns_json TEXT NOT NULL,
                    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY(session_id) REFERENCES sessions(session_id)
                );

                CREATE TABLE IF NOT EXISTS table_refs (
                    table_ref TEXT PRIMARY KEY,
                    source_path TEXT NOT NULL,
                    sheet_name TEXT NOT NULL,
                    region TEXT,
                    columns_json TEXT NOT NULL,
                    header_row_count INTEGER NOT NULL,
                    data_start_row_index INTEGER NOT NULL,
                    source_fingerprint_json TEXT NOT NULL,
                    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    last_used_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
                );

                CREATE TABLE IF NOT EXISTS event_logs (
                    event_id INTEGER PRIMARY KEY AUTOINCREMENT,
                    session_id TEXT NOT NULL,
                    event_type TEXT NOT NULL,
                    stage TEXT,
                    tool_name TEXT,
                    status TEXT NOT NULL,
                    message TEXT,
                    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
                );
                ",
            )
            .map_err(|error| LocalMemoryError::BootstrapSchema(error.to_string()))?;
        self.ensure_session_state_optional_columns(connection)?;
        self.ensure_table_refs_region_column(connection)
    }

    // 2026-03-22: 这里补齐 table_refs 的 region 列迁移，目的是兼容旧 runtime 库也能镜像显式区域 table_ref。
    // 2026-03-23: ???? runtime ????? file_ref ? sheet_index ???????????????????????????????????????????
    fn ensure_session_state_optional_columns(
        &self,
        connection: &Connection,
    ) -> Result<(), LocalMemoryError> {
        let mut statement = connection
            .prepare("PRAGMA table_info(session_state)")
            .map_err(|error| LocalMemoryError::BootstrapSchema(error.to_string()))?;
        let columns = statement
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(|error| LocalMemoryError::BootstrapSchema(error.to_string()))?
            .filter_map(Result::ok)
            .collect::<Vec<_>>();

        if !columns
            .iter()
            .any(|column_name| column_name == "current_file_ref")
        {
            connection
                .execute(
                    "ALTER TABLE session_state ADD COLUMN current_file_ref TEXT",
                    [],
                )
                .map_err(|error| LocalMemoryError::BootstrapSchema(error.to_string()))?;
        }
        if !columns
            .iter()
            .any(|column_name| column_name == "current_sheet_index")
        {
            connection
                .execute(
                    "ALTER TABLE session_state ADD COLUMN current_sheet_index INTEGER",
                    [],
                )
                .map_err(|error| LocalMemoryError::BootstrapSchema(error.to_string()))?;
        }
        if !columns
            .iter()
            .any(|column_name| column_name == "active_handle_ref")
        {
            // 2026-03-23: 这里兼容旧 runtime 库补充 active_handle_ref 列，原因是历史库只保存 active_table_ref；目的是在不清库的前提下给多步链式闭环增加“最新激活句柄”维度。
            connection
                .execute(
                    "ALTER TABLE session_state ADD COLUMN active_handle_ref TEXT",
                    [],
                )
                .map_err(|error| LocalMemoryError::BootstrapSchema(error.to_string()))?;
        }
        if !columns
            .iter()
            .any(|column_name| column_name == "active_handle_kind")
        {
            // 2026-03-23: 这里兼容旧 runtime 库补充 active_handle_kind 列，原因是上层要区分 table/result/workbook 三类句柄；目的是避免老库升级后状态摘要仍依赖前缀猜测。
            connection
                .execute(
                    "ALTER TABLE session_state ADD COLUMN active_handle_kind TEXT",
                    [],
                )
                .map_err(|error| LocalMemoryError::BootstrapSchema(error.to_string()))?;
        }

        Ok(())
    }

    fn ensure_table_refs_region_column(
        &self,
        connection: &Connection,
    ) -> Result<(), LocalMemoryError> {
        let mut statement = connection
            .prepare("PRAGMA table_info(table_refs)")
            .map_err(|error| LocalMemoryError::BootstrapSchema(error.to_string()))?;
        let has_region = statement
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(|error| LocalMemoryError::BootstrapSchema(error.to_string()))?
            .filter_map(Result::ok)
            .any(|column_name| column_name == "region");

        if !has_region {
            connection
                .execute("ALTER TABLE table_refs ADD COLUMN region TEXT", [])
                .map_err(|error| LocalMemoryError::BootstrapSchema(error.to_string()))?;
        }

        Ok(())
    }
}

// 2026-03-22: 这里统一把非 rusqlite 错误桥接成 SQL 转换错误，目的是复用 query_row 的映射闭包而不扩散样板代码。
fn to_sql_error(error: LocalMemoryError) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
}
