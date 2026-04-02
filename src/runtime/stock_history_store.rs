use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::{Connection, params};
use thiserror::Error;

use crate::runtime_paths::workspace_runtime_dir;

// 2026-03-28 CST: 这里定义单条股票日线记录，原因是 CSV 导入和后续技术指标计算都会复用同一份标准化历史结构；
// 目的：把“文本 CSV 行”先收口成稳定的 Rust 结构，避免后面每个 Tool 都重复解析字段。
#[derive(Debug, Clone, PartialEq)]
pub struct StockHistoryRow {
    pub trade_date: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub adj_close: f64,
    pub volume: i64,
}

// 2026-03-28 CST: 这里定义导入摘要，原因是外部 EXE Tool 需要回执导入结果，而不是只做静默落库；
// 目的：让后续 Skill、CLI 和交接链路都能直接消费统一的导入结果合同。
#[derive(Debug, Clone, PartialEq)]
pub struct StockHistoryImportSummary {
    pub imported_row_count: usize,
    pub start_date: String,
    pub end_date: String,
}

// 2026-03-28 CST: 这里定义股票历史 SQLite Store，原因是用户已经明确历史数据要走 SQLite；
// 目的：把股票历史表和 session/runtime 记忆分离，既复用同一个 runtime 根目录，又不把两类表硬耦合到一起。
#[derive(Debug, Clone)]
pub struct StockHistoryStore {
    db_path: PathBuf,
}

// 2026-03-28 CST: 这里集中定义股票历史存储层错误，原因是 CSV 解析层和 SQLite 层都可能失败；
// 目的：让上层 Tool 能拿到清晰、中文、可定位的问题信息。
#[derive(Debug, Error)]
pub enum StockHistoryStoreError {
    #[error("无法确定股票历史 SQLite 所在目录: {0}")]
    ResolveRuntimeDir(String),
    #[error("无法创建股票历史 SQLite 目录: {0}")]
    CreateRuntimeDir(String),
    #[error("无法打开股票历史 SQLite: {0}")]
    OpenDatabase(String),
    #[error("无法初始化股票历史表结构: {0}")]
    BootstrapSchema(String),
    #[error("股票历史数据不能为空")]
    EmptyRows,
    #[error("无法写入股票历史数据: {0}")]
    WriteRows(String),
    // 2026-03-28 CST：这里补充历史读取错误类型，原因是技术面咨询 Tool 已经开始直接依赖 SQLite 历史查询；
    // 目的：把“写入失败”和“读取失败”在存储层明确拆开，便于上层返回更准确的中文错误。
    #[error("无法读取股票历史数据: {0}")]
    ReadRows(String),
}

impl StockHistoryStore {
    // 2026-03-28 CST: 这里允许显式指定数据库路径，原因是测试和后续命令行打包都可能需要自定义落盘位置；
    // 目的：保留“同一逻辑，不同落盘目录”的扩展点，同时不增加当前业务复杂度。
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }

    // 2026-03-28 CST: 这里提供工作区默认数据库路径，原因是当前第一刀要和现有 runtime 根目录保持一致；
    // 目的：让 `EXCEL_SKILL_RUNTIME_DB` 的测试隔离能力能自动覆盖股票历史落盘，而不再新增一套测试环境变量。
    pub fn workspace_default() -> Result<Self, StockHistoryStoreError> {
        if let Ok(path) = std::env::var("EXCEL_SKILL_STOCK_DB") {
            return Ok(Self::new(PathBuf::from(path)));
        }

        let runtime_dir =
            workspace_runtime_dir().map_err(StockHistoryStoreError::ResolveRuntimeDir)?;
        Ok(Self::new(runtime_dir.join("stock_history.db")))
    }

    // 2026-03-28 CST: 这里暴露数据库路径，原因是导入 Tool 回执需要告诉上层数据实际落到哪里；
    // 目的：方便后续 Skill/交接/排障定位 SQLite 文件。
    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    // 2026-03-28 CST: 这里集中执行 upsert 导入，原因是“同一 symbol + trade_date 覆盖”是历史补数和重导入的核心规则；
    // 目的：保证技术指标计算读取到的总是一份去重后的日线历史，而不是重复交易日。
    pub fn import_rows(
        &self,
        symbol: &str,
        source: &str,
        rows: &[StockHistoryRow],
    ) -> Result<StockHistoryImportSummary, StockHistoryStoreError> {
        if rows.is_empty() {
            return Err(StockHistoryStoreError::EmptyRows);
        }

        let mut connection = self.open_connection()?;
        let transaction = connection
            .transaction()
            .map_err(|error| StockHistoryStoreError::WriteRows(error.to_string()))?;

        for row in rows {
            transaction
                .execute(
                    "INSERT INTO stock_price_history (
                        symbol,
                        trade_date,
                        open,
                        high,
                        low,
                        close,
                        adj_close,
                        volume,
                        source
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                    ON CONFLICT(symbol, trade_date) DO UPDATE SET
                        open = excluded.open,
                        high = excluded.high,
                        low = excluded.low,
                        close = excluded.close,
                        adj_close = excluded.adj_close,
                        volume = excluded.volume,
                        source = excluded.source,
                        updated_at = CURRENT_TIMESTAMP",
                    params![
                        symbol,
                        row.trade_date,
                        row.open,
                        row.high,
                        row.low,
                        row.close,
                        row.adj_close,
                        row.volume,
                        source,
                    ],
                )
                .map_err(|error| StockHistoryStoreError::WriteRows(error.to_string()))?;
        }

        transaction
            .commit()
            .map_err(|error| StockHistoryStoreError::WriteRows(error.to_string()))?;

        let start_date = rows
            .iter()
            .map(|row| row.trade_date.as_str())
            .min()
            .expect("rows should not be empty")
            .to_string();
        let end_date = rows
            .iter()
            .map(|row| row.trade_date.as_str())
            .max()
            .expect("rows should not be empty")
            .to_string();

        Ok(StockHistoryImportSummary {
            imported_row_count: rows.len(),
            start_date,
            end_date,
        })
    }

    // 2026-03-28 CST: 这里统一打开并初始化股票历史 SQLite，原因是上层 Tool 不应该关心建库和建表细节；
    // 目的：让导入 Tool 和后续技术咨询 Tool 共用同一套持久层入口。
    // 2026-03-28 CST：这里新增最近历史读取方法，原因是 `technical_consultation_basic` 需要沿现有 SQLite 主线直接取最近窗口数据；
    // 目的：统一收口“按 symbol + 截止日期 + 回看窗口读取升序历史”的 SQL，避免上层 Tool 重复拼接查询。
    pub fn load_recent_rows(
        &self,
        symbol: &str,
        as_of_date: Option<&str>,
        lookback_days: usize,
    ) -> Result<Vec<StockHistoryRow>, StockHistoryStoreError> {
        let connection = self.open_connection()?;
        let mut statement = connection
            .prepare(
                "
                SELECT trade_date, open, high, low, close, adj_close, volume
                FROM stock_price_history
                WHERE symbol = ?1
                  AND (?2 IS NULL OR trade_date <= ?2)
                ORDER BY trade_date DESC
                LIMIT ?3
                ",
            )
            .map_err(|error| StockHistoryStoreError::ReadRows(error.to_string()))?;

        let mapped_rows = statement
            .query_map(params![symbol, as_of_date, lookback_days as i64], |row| {
                Ok(StockHistoryRow {
                    trade_date: row.get(0)?,
                    open: row.get(1)?,
                    high: row.get(2)?,
                    low: row.get(3)?,
                    close: row.get(4)?,
                    adj_close: row.get(5)?,
                    volume: row.get(6)?,
                })
            })
            .map_err(|error| StockHistoryStoreError::ReadRows(error.to_string()))?;

        let mut rows = Vec::new();
        for row in mapped_rows {
            rows.push(row.map_err(|error| StockHistoryStoreError::ReadRows(error.to_string()))?);
        }

        // 2026-03-28 CST：这里反转成升序结果，原因是 SQLite 倒序取最近窗口最直接；
        // 目的：保证上层指标计算永远面对“从旧到新”的稳定输入，减少重复排序逻辑。
        rows.reverse();
        Ok(rows)
    }

    // 2026-04-02 CST: 这里补“某个快照日之后”的顺序历史读取，原因是 forward returns 回填必须拿到 snapshot 之后的未来窗口，
    // 目的：把未来收益研究也继续收口在 stock history store，而不是在上层研究模块直接拼 SQL。
    pub fn load_rows_after(
        &self,
        symbol: &str,
        after_date: &str,
        limit: usize,
    ) -> Result<Vec<StockHistoryRow>, StockHistoryStoreError> {
        let connection = self.open_connection()?;
        let mut statement = connection
            .prepare(
                "
                SELECT trade_date, open, high, low, close, adj_close, volume
                FROM stock_price_history
                WHERE symbol = ?1
                  AND trade_date > ?2
                ORDER BY trade_date ASC
                LIMIT ?3
                ",
            )
            .map_err(|error| StockHistoryStoreError::ReadRows(error.to_string()))?;

        let mapped_rows = statement
            .query_map(params![symbol, after_date, limit as i64], |row| {
                Ok(StockHistoryRow {
                    trade_date: row.get(0)?,
                    open: row.get(1)?,
                    high: row.get(2)?,
                    low: row.get(3)?,
                    close: row.get(4)?,
                    adj_close: row.get(5)?,
                    volume: row.get(6)?,
                })
            })
            .map_err(|error| StockHistoryStoreError::ReadRows(error.to_string()))?;

        let mut rows = Vec::new();
        for row in mapped_rows {
            rows.push(row.map_err(|error| StockHistoryStoreError::ReadRows(error.to_string()))?);
        }
        Ok(rows)
    }

    // 2026-04-02 CST: 这里补“按日期区间读取升序行情”的统一入口，原因是模板级共振因子同步需要把多个代理 symbol 对齐到同一时间窗口；
    // 目的：继续把日期过滤和升序输出收口在 stock history store，避免上层同步逻辑重复手写 SQL 与排序规则。
    pub fn load_rows_in_range(
        &self,
        symbol: &str,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<StockHistoryRow>, StockHistoryStoreError> {
        let connection = self.open_connection()?;
        let mut statement = connection
            .prepare(
                "
                SELECT trade_date, open, high, low, close, adj_close, volume
                FROM stock_price_history
                WHERE symbol = ?1
                  AND trade_date >= ?2
                  AND trade_date <= ?3
                ORDER BY trade_date ASC
                ",
            )
            .map_err(|error| StockHistoryStoreError::ReadRows(error.to_string()))?;

        let mapped_rows = statement
            .query_map(params![symbol, start_date, end_date], |row| {
                Ok(StockHistoryRow {
                    trade_date: row.get(0)?,
                    open: row.get(1)?,
                    high: row.get(2)?,
                    low: row.get(3)?,
                    close: row.get(4)?,
                    adj_close: row.get(5)?,
                    volume: row.get(6)?,
                })
            })
            .map_err(|error| StockHistoryStoreError::ReadRows(error.to_string()))?;

        let mut rows = Vec::new();
        for row in mapped_rows {
            rows.push(row.map_err(|error| StockHistoryStoreError::ReadRows(error.to_string()))?);
        }
        Ok(rows)
    }

    fn open_connection(&self) -> Result<Connection, StockHistoryStoreError> {
        if let Some(parent) = self.db_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| StockHistoryStoreError::CreateRuntimeDir(error.to_string()))?;
        }

        let connection = Connection::open(&self.db_path)
            .map_err(|error| StockHistoryStoreError::OpenDatabase(error.to_string()))?;
        connection
            .busy_timeout(Duration::from_secs(5))
            .map_err(|error| StockHistoryStoreError::OpenDatabase(error.to_string()))?;
        self.bootstrap_schema(&connection)?;
        Ok(connection)
    }

    // 2026-03-28 CST: 这里初始化股票历史表，原因是第一刀只需要最小历史表，不应把指标缓存和咨询结果表一起硬塞进来；
    // 目的：先稳住 `stock_price_history` 主表，再逐步往上叠技术面能力。
    fn bootstrap_schema(&self, connection: &Connection) -> Result<(), StockHistoryStoreError> {
        connection
            .execute_batch(
                "
                CREATE TABLE IF NOT EXISTS stock_price_history (
                    symbol TEXT NOT NULL,
                    trade_date TEXT NOT NULL,
                    open REAL NOT NULL,
                    high REAL NOT NULL,
                    low REAL NOT NULL,
                    close REAL NOT NULL,
                    adj_close REAL NOT NULL,
                    volume INTEGER NOT NULL,
                    source TEXT NOT NULL,
                    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    PRIMARY KEY(symbol, trade_date)
                );

                CREATE INDEX IF NOT EXISTS idx_stock_price_history_symbol_date
                ON stock_price_history(symbol, trade_date);
                ",
            )
            .map_err(|error| StockHistoryStoreError::BootstrapSchema(error.to_string()))?;
        Ok(())
    }
}
