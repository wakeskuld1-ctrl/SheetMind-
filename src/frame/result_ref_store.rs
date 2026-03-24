use std::fs;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use polars::prelude::{DataFrame, DataType, NamedFrom, PolarsError, Series, TimeUnit};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::runtime_paths::workspace_runtime_dir;

// 2026-03-22: 这里定义结果集持久化的稳定物理列类型，目的是先把跨请求复用真正需要的基础数值/文本类型收口下来。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersistedColumnType {
    String,
    Int64,
    Float64,
    Boolean,
}

// 2026-03-22: 这里补充逻辑类型元数据，目的是在不破坏旧 result_ref 协议的前提下，逐步支持日期/时间这类高阶列语义。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersistedLogicalType {
    Date,
    DateTime {
        time_unit: PersistedTimeUnit,
        time_zone: Option<String>,
    },
}

// 2026-03-22: 这里把时间粒度单独收口成可序列化枚举，目的是避免把 Polars 内部枚举直接写进持久化协议。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersistedTimeUnit {
    Milliseconds,
    Microseconds,
    Nanoseconds,
}

// 2026-03-22: 这里定义结果集的列级持久化结构，目的是让 result_ref 能同时保留列名、物理类型、逻辑类型和逐行取值。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersistedResultColumn {
    pub name: String,
    pub dtype: PersistedColumnType,
    pub values: Vec<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logical_type: Option<PersistedLogicalType>,
}

// 2026-03-22: 这里定义可跨请求复用的中间结果记录，目的是把 DataFrame 结果升级成真正可继续消费的稳定句柄。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersistedResultDataset {
    pub result_ref: String,
    pub produced_by: String,
    #[serde(default)]
    pub source_refs: Vec<String>,
    pub row_count: usize,
    pub columns: Vec<PersistedResultColumn>,
}

impl PersistedResultDataset {
    // 2026-03-22: 这里从 DataFrame 构造持久化结果，目的是把链式执行里最关键的“中间结果可落盘”能力补出来。
    pub fn from_dataframe(
        result_ref: &str,
        produced_by: &str,
        source_refs: Vec<String>,
        dataframe: &DataFrame,
    ) -> Result<Self, ResultRefStoreError> {
        let columns = dataframe
            .get_columns()
            .iter()
            .map(PersistedResultColumn::from_series)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            result_ref: result_ref.to_string(),
            produced_by: produced_by.to_string(),
            source_refs,
            row_count: dataframe.height(),
            columns,
        })
    }

    // 2026-03-22: 这里把落盘结果恢复成 DataFrame，目的是让后续 Tool 能直接继续复用 result_ref，而不是总回退到原始 Excel。
    pub fn to_dataframe(&self) -> Result<DataFrame, ResultRefStoreError> {
        let columns = self
            .columns
            .iter()
            .map(PersistedResultColumn::to_series)
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(Into::into)
            .collect();

        DataFrame::new(columns)
            .map_err(|error| ResultRefStoreError::RestoreResult(error.to_string()))
    }
}

impl PersistedResultColumn {
    // 2026-03-22: 这里把运行时列压平成 JSON 值，目的是在保持协议简单的同时补齐 NaN/Infinity 与日期时间元数据。
    fn from_series(column: &polars::prelude::Column) -> Result<Self, ResultRefStoreError> {
        let series = column.as_materialized_series();
        if matches!(series.dtype(), DataType::Boolean) {
            return Ok(Self {
                name: column.name().to_string(),
                dtype: PersistedColumnType::Boolean,
                values: series
                    .bool()
                    .map_err(|error| ResultRefStoreError::SerializeResult(error.to_string()))?
                    .into_iter()
                    .map(|value| value.map(Value::Bool).unwrap_or(Value::Null))
                    .collect(),
                logical_type: None,
            });
        }

        // 2026-03-22: 这里把 Date 先收敛成整数天数再落盘，目的是让日期列跨请求恢复后仍保持 Date 语义。
        if matches!(series.dtype(), DataType::Date) {
            let casted = series
                .cast(&DataType::Int32)
                .map_err(|error| ResultRefStoreError::SerializeResult(error.to_string()))?;
            return Ok(Self {
                name: column.name().to_string(),
                dtype: PersistedColumnType::Int64,
                values: casted
                    .i32()
                    .map_err(|error| ResultRefStoreError::SerializeResult(error.to_string()))?
                    .into_iter()
                    .map(|value| {
                        value
                            .map(|days| Value::from(i64::from(days)))
                            .unwrap_or(Value::Null)
                    })
                    .collect(),
                logical_type: Some(PersistedLogicalType::Date),
            });
        }

        // 2026-03-22: 这里把 Datetime 先收敛成整数时间戳再落盘，目的是保住时间粒度与时区元数据。
        if let DataType::Datetime(time_unit, time_zone) = series.dtype() {
            let casted = series
                .cast(&DataType::Int64)
                .map_err(|error| ResultRefStoreError::SerializeResult(error.to_string()))?;
            return Ok(Self {
                name: column.name().to_string(),
                dtype: PersistedColumnType::Int64,
                values: casted
                    .i64()
                    .map_err(|error| ResultRefStoreError::SerializeResult(error.to_string()))?
                    .into_iter()
                    .map(|value| value.map(Value::from).unwrap_or(Value::Null))
                    .collect(),
                logical_type: Some(PersistedLogicalType::DateTime {
                    time_unit: PersistedTimeUnit::from_polars(*time_unit),
                    time_zone: time_zone.as_ref().map(|zone| zone.to_string()),
                }),
            });
        }

        if is_integer_dtype(series.dtype()) {
            let casted = series
                .cast(&DataType::Int64)
                .map_err(|error| ResultRefStoreError::SerializeResult(error.to_string()))?;
            return Ok(Self {
                name: column.name().to_string(),
                dtype: PersistedColumnType::Int64,
                values: casted
                    .i64()
                    .map_err(|error| ResultRefStoreError::SerializeResult(error.to_string()))?
                    .into_iter()
                    .map(|value| value.map(Value::from).unwrap_or(Value::Null))
                    .collect(),
                logical_type: None,
            });
        }

        if is_float_dtype(series.dtype()) {
            let casted = series
                .cast(&DataType::Float64)
                .map_err(|error| ResultRefStoreError::SerializeResult(error.to_string()))?;
            return Ok(Self {
                name: column.name().to_string(),
                dtype: PersistedColumnType::Float64,
                values: casted
                    .f64()
                    .map_err(|error| ResultRefStoreError::SerializeResult(error.to_string()))?
                    .into_iter()
                    .map(|value| value.map(serialize_f64_value).unwrap_or(Value::Null))
                    .collect(),
                logical_type: None,
            });
        }

        let mut values = Vec::with_capacity(series.len());
        for row_index in 0..series.len() {
            let rendered = series
                .get(row_index)
                .map_err(|error| ResultRefStoreError::SerializeResult(error.to_string()))?;
            if rendered.is_null() {
                values.push(Value::Null);
                continue;
            }

            values.push(Value::String(
                series
                    .str_value(row_index)
                    .map_err(|error| ResultRefStoreError::SerializeResult(error.to_string()))?
                    .into_owned(),
            ));
        }

        Ok(Self {
            name: column.name().to_string(),
            dtype: PersistedColumnType::String,
            values,
            logical_type: None,
        })
    }

    // 2026-03-22: 这里先按稳定物理类型恢复 Series，再按逻辑类型补回日期/时间语义，目的是兼顾兼容性与后续分析可用性。
    fn to_series(&self) -> Result<Series, ResultRefStoreError> {
        let physical = match self.dtype {
            PersistedColumnType::String => Series::new(
                self.name.clone().into(),
                self.values
                    .iter()
                    .map(|value| match value {
                        Value::Null => None,
                        Value::String(text) => Some(text.clone()),
                        other => Some(other.to_string()),
                    })
                    .collect::<Vec<Option<String>>>(),
            ),
            PersistedColumnType::Int64 => Series::new(
                self.name.clone().into(),
                self.values
                    .iter()
                    .map(parse_i64_value)
                    .collect::<Result<Vec<Option<i64>>, _>>()?,
            ),
            PersistedColumnType::Float64 => Series::new(
                self.name.clone().into(),
                self.values
                    .iter()
                    .map(parse_f64_value)
                    .collect::<Result<Vec<Option<f64>>, _>>()?,
            ),
            PersistedColumnType::Boolean => Series::new(
                self.name.clone().into(),
                self.values
                    .iter()
                    .map(parse_bool_value)
                    .collect::<Result<Vec<Option<bool>>, _>>()?,
            ),
        };

        apply_logical_type(self, physical)
    }
}

impl PersistedTimeUnit {
    // 2026-03-22: 这里集中转换时间粒度，目的是让落盘协议与 Polars 运行时保持解耦。
    fn from_polars(time_unit: TimeUnit) -> Self {
        match time_unit {
            TimeUnit::Milliseconds => Self::Milliseconds,
            TimeUnit::Microseconds => Self::Microseconds,
            TimeUnit::Nanoseconds => Self::Nanoseconds,
        }
    }

    // 2026-03-22: 这里把持久化时间粒度恢复回 Polars 枚举，目的是让时间列回放后仍保留原始精度。
    fn to_polars(&self) -> TimeUnit {
        match self {
            Self::Milliseconds => TimeUnit::Milliseconds,
            Self::Microseconds => TimeUnit::Microseconds,
            Self::Nanoseconds => TimeUnit::Nanoseconds,
        }
    }
}

// 2026-03-22: 这里定义 result_ref 的落盘错误，目的是把“序列化失败 / 恢复失败 / 文件失败”拆成可读中文错误。
#[derive(Debug, Error)]
pub enum ResultRefStoreError {
    #[error("无法创建 result_ref 存储目录: {0}")]
    CreateStoreDir(String),
    #[error("无法保存 result_ref `{result_ref}`: {message}")]
    SaveResult { result_ref: String, message: String },
    #[error("无法读取 result_ref `{result_ref}`: {message}")]
    LoadResult { result_ref: String, message: String },
    #[error("无法序列化结果集: {0}")]
    SerializeResult(String),
    #[error("无法恢复结果集: {0}")]
    RestoreResult(String),
}

// 2026-03-22: 这里定义 result_ref 的文件存储入口，目的是先以最小代价建立中间结果的跨请求复用能力。
#[derive(Debug, Clone)]
pub struct ResultRefStore {
    root_dir: PathBuf,
}

impl ResultRefStore {
    // 2026-03-22: 这里允许测试显式传目录，目的是让持久化 round-trip 测试互不污染。
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }

    // 2026-03-22: 这里提供默认落盘目录，目的是让 dispatcher 和 Tool 统一复用同一套结果集存储位置。
    pub fn workspace_default() -> Result<Self, ResultRefStoreError> {
        let runtime_dir = workspace_runtime_dir().map_err(ResultRefStoreError::CreateStoreDir)?;
        Ok(Self::new(runtime_dir.join("result_refs")))
    }

    // 2026-03-22: 这里统一生成 result_ref，目的是把中间结果命名规则和 table_ref 区分开，便于调试与展示。
    pub fn create_result_ref() -> String {
        let timestamp = UNIX_EPOCH
            .elapsed()
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        format!("result_{}_{}", std::process::id(), timestamp)
    }

    // 2026-03-22: 这里保存结果集，目的是让多步分析中的中间 DataFrame 真正脱离当前进程内存。
    pub fn save(&self, record: &PersistedResultDataset) -> Result<(), ResultRefStoreError> {
        fs::create_dir_all(&self.root_dir)
            .map_err(|error| ResultRefStoreError::CreateStoreDir(error.to_string()))?;
        let payload =
            serde_json::to_vec_pretty(record).map_err(|error| ResultRefStoreError::SaveResult {
                result_ref: record.result_ref.clone(),
                message: error.to_string(),
            })?;
        fs::write(self.file_path(&record.result_ref), payload).map_err(|error| {
            ResultRefStoreError::SaveResult {
                result_ref: record.result_ref.clone(),
                message: error.to_string(),
            }
        })
    }

    // 2026-03-22: 这里按 result_ref 读回结果集，目的是给下一步分析和导出层提供稳定输入。
    pub fn load(&self, result_ref: &str) -> Result<PersistedResultDataset, ResultRefStoreError> {
        let payload = fs::read(self.file_path(result_ref)).map_err(|error| {
            ResultRefStoreError::LoadResult {
                result_ref: result_ref.to_string(),
                message: error.to_string(),
            }
        })?;
        serde_json::from_slice(&payload).map_err(|error| ResultRefStoreError::LoadResult {
            result_ref: result_ref.to_string(),
            message: error.to_string(),
        })
    }

    // 2026-03-22: 这里统一拼接记录路径，目的是确保所有入口都遵守同一套文件命名规则。
    fn file_path(&self, result_ref: &str) -> PathBuf {
        self.root_dir.join(format!("{result_ref}.json"))
    }
}

// 2026-03-22: 这里收口整数类型判断，目的是把结果持久化支持范围控制在 V1 真正常用的数值集内。
fn is_integer_dtype(dtype: &DataType) -> bool {
    matches!(
        dtype,
        DataType::Int64
            | DataType::Int32
            | DataType::Int16
            | DataType::Int8
            | DataType::UInt64
            | DataType::UInt32
            | DataType::UInt16
            | DataType::UInt8
    )
}

// 2026-03-22: 这里收口浮点类型判断，目的是让统计和建模结果在跨请求后仍保留连续数值语义。
fn is_float_dtype(dtype: &DataType) -> bool {
    matches!(dtype, DataType::Float64 | DataType::Float32)
}

// 2026-03-22: 这里把 JSON 值恢复为 i64，目的是让落盘后的聚合结果还能继续参与整数运算。
fn parse_i64_value(value: &Value) -> Result<Option<i64>, ResultRefStoreError> {
    match value {
        Value::Null => Ok(None),
        Value::Number(number) => number.as_i64().map(Some).ok_or_else(|| {
            ResultRefStoreError::RestoreResult(format!("无法把 `{number}` 恢复为 int64"))
        }),
        Value::String(text) => text.parse::<i64>().map(Some).map_err(|error| {
            ResultRefStoreError::RestoreResult(format!("无法把 `{text}` 恢复为 int64: {error}"))
        }),
        other => Err(ResultRefStoreError::RestoreResult(format!(
            "无法把 `{other}` 恢复为 int64"
        ))),
    }
}

// 2026-03-22: 这里把 JSON 值恢复为 f64，目的是让均值、回归和评分结果可以跨请求继续复用。
fn parse_f64_value(value: &Value) -> Result<Option<f64>, ResultRefStoreError> {
    match value {
        Value::Null => Ok(None),
        Value::Number(number) => number.as_f64().map(Some).ok_or_else(|| {
            ResultRefStoreError::RestoreResult(format!("无法把 `{number}` 恢复为 float64"))
        }),
        Value::String(text) => parse_special_f64_value(text).map(Some).ok_or_else(|| {
            ResultRefStoreError::RestoreResult(format!("无法把 `{text}` 恢复为 float64"))
        }),
        other => Err(ResultRefStoreError::RestoreResult(format!(
            "无法把 `{other}` 恢复为 float64"
        ))),
    }
}

// 2026-03-22: 这里把 JSON 值恢复为 bool，目的是让标签列和二分类结果在链式分析里保持稳定。
fn parse_bool_value(value: &Value) -> Result<Option<bool>, ResultRefStoreError> {
    match value {
        Value::Null => Ok(None),
        Value::Bool(flag) => Ok(Some(*flag)),
        Value::String(text) => text.parse::<bool>().map(Some).map_err(|error| {
            ResultRefStoreError::RestoreResult(format!("无法把 `{text}` 恢复为 boolean: {error}"))
        }),
        other => Err(ResultRefStoreError::RestoreResult(format!(
            "无法把 `{other}` 恢复为 boolean"
        ))),
    }
}

// 2026-03-22: 这里单独编码 NaN/Infinity，目的是解决 JSON 标准数值不支持非有限浮点导致的静默丢值问题。
fn serialize_f64_value(value: f64) -> Value {
    if value.is_nan() {
        return Value::String("NaN".to_string());
    }
    if value.is_infinite() && value.is_sign_positive() {
        return Value::String("Infinity".to_string());
    }
    if value.is_infinite() && value.is_sign_negative() {
        return Value::String("-Infinity".to_string());
    }
    Value::from(value)
}

// 2026-03-22: 这里集中恢复特殊浮点文本，目的是让 NaN/Infinity 在跨请求回放后仍保留原始数值语义。
fn parse_special_f64_value(text: &str) -> Option<f64> {
    match text.trim() {
        "NaN" => Some(f64::NAN),
        "Infinity" => Some(f64::INFINITY),
        "-Infinity" => Some(f64::NEG_INFINITY),
        other => other.parse::<f64>().ok(),
    }
}

// 2026-03-22: 这里按逻辑类型再恢复日期/时间语义，目的是兼顾旧协议兼容与后续分析可用性。
fn apply_logical_type(
    column: &PersistedResultColumn,
    series: Series,
) -> Result<Series, ResultRefStoreError> {
    match &column.logical_type {
        None => Ok(series),
        Some(PersistedLogicalType::Date) => series
            .cast(&DataType::Int32)
            .and_then(|casted| casted.cast(&DataType::Date))
            .map_err(map_polars_error),
        Some(PersistedLogicalType::DateTime {
            time_unit,
            time_zone,
        }) => {
            // 2026-03-22: 这里先保守回放无时区 Datetime，目的是先把 V1/V2 当前真实用到的 naive datetime 稳定打通。
            let polars_time_zone = if time_zone.is_some() { None } else { None };
            series
                .cast(&DataType::Datetime(time_unit.to_polars(), polars_time_zone))
                .map_err(map_polars_error)
        }
    }
}

// 2026-03-22: 这里保留 Polars 错误到本模块错误的桥接点，目的是把时间逻辑恢复失败也统一收口成稳定错误。
#[allow(dead_code)]
fn map_polars_error(error: PolarsError) -> ResultRefStoreError {
    ResultRefStoreError::RestoreResult(error.to_string())
}
