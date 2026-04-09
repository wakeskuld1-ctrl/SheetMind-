use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::{Connection, params};
use thiserror::Error;

use crate::runtime_paths::workspace_runtime_dir;

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

#[derive(Debug, Clone, PartialEq)]
pub struct StockHistoryImportSummary {
    pub imported_row_count: usize,
    pub start_date: String,
    pub end_date: String,
}

#[derive(Debug, Clone)]
pub struct StockHistoryStore {
    db_path: PathBuf,
}

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
    #[error("无法读取股票历史数据: {0}")]
    ReadRows(String),
}

impl StockHistoryStore {
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }

    pub fn workspace_default() -> Result<Self, StockHistoryStoreError> {
        if let Ok(path) = std::env::var("EXCEL_SKILL_STOCK_DB") {
            return Ok(Self::new(PathBuf::from(path)));
        }

        let runtime_dir =
            workspace_runtime_dir().map_err(StockHistoryStoreError::ResolveRuntimeDir)?;
        Ok(Self::new(runtime_dir.join("stock_history.db")))
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

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

        rows.reverse();
        Ok(rows)
    }

    pub fn load_forward_rows(
        &self,
        symbol: &str,
        after_date: &str,
        forward_days: usize,
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
            .query_map(params![symbol, after_date, forward_days as i64], |row| {
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

    pub fn load_rows_after(
        &self,
        symbol: &str,
        after_date: &str,
        limit: usize,
    ) -> Result<Vec<StockHistoryRow>, StockHistoryStoreError> {
        self.load_forward_rows(symbol, after_date, limit)
    }

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
