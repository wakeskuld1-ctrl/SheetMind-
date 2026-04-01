use polars::chunked_array::cast::CastOptions;
use polars::prelude::{DataFrame, DataType};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::handles::TableHandle;
use crate::frame::loader::LoadedTable;

// 2026-03-21: 这里定义单列类型转换请求，目的是把 Tool 层输入稳定映射成 DataFrame 可执行的显式转换计划。
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct CastColumnSpec {
    // 2026-03-21: 指定目标列名，目的是避免把用户意图和底层列位置强绑定，便于后续列重排后继续复用。
    pub column: String,
    // 2026-03-21: 指定目标类型，目的是让 V1 先走显式、可控的转换路径，降低自动推断误判风险。
    pub target_type: CastTargetType,
}

// 2026-03-21: 这里定义 V1 支持的显式目标类型，目的是优先覆盖后续聚合与建模最常见的基础类型。
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CastTargetType {
    // 2026-03-21: 保留字符串类型，目的是允许后续把数值列重新回退为展示友好的字符串列。
    String,
    // 2026-03-21: 先支持 64 位整数，目的是覆盖 Excel 中金额、计数、ID 之外的大多数整数分析列。
    Int64,
    // 2026-03-21: 先支持 64 位浮点数，目的是为均值、回归和聚类等后续分析能力提供数值底座。
    Float64,
    // 2026-03-21: 先支持布尔类型，目的是让二分类标记列能被明确转成 true/false 语义。
    Boolean,
}

impl CastTargetType {
    // 2026-03-21: 这里统一维护目标类型到 Polars 类型的映射，目的是把外部协议与底层实现解耦。
    fn to_polars_dtype(&self) -> DataType {
        match self {
            CastTargetType::String => DataType::String,
            CastTargetType::Int64 => DataType::Int64,
            CastTargetType::Float64 => DataType::Float64,
            CastTargetType::Boolean => DataType::Boolean,
        }
    }

    // 2026-03-21: 这里提供用户侧稳定类型标签，目的是让 CLI 与后续问答界面都复用统一文案。
    fn label(&self) -> &'static str {
        match self {
            CastTargetType::String => "string",
            CastTargetType::Int64 => "int64",
            CastTargetType::Float64 => "float64",
            CastTargetType::Boolean => "boolean",
        }
    }
}

// 2026-03-21: 这里定义列类型摘要，目的是让 Tool 层可以直接返回“列 -> 类型”的稳定 JSON 结果。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ColumnTypeSummary {
    // 2026-03-21: 返回列名，目的是让用户或 Skill 明确知道哪一列已经被转换成什么类型。
    pub column: String,
    // 2026-03-21: 返回类型标签，目的是避免直接暴露 Polars 内部缩写给非技术用户。
    pub dtype: String,
}

// 2026-03-21: 这里定义类型转换错误，目的是把空请求、缺列和具体转换失败分开暴露给上层。
#[derive(Debug, Error)]
pub enum CastError {
    // 2026-03-21: 空转换计划直接拒绝，目的是避免返回语义模糊的“没有变化但算成功”结果。
    #[error("cast_column_types 至少需要一个转换定义")]
    EmptyCasts,
    // 2026-03-21: 缺列时返回明确错误，目的是帮助用户快速修正字段名或表头映射。
    #[error("找不到列: {0}")]
    MissingColumn(String),
    // 2026-03-21: 转换失败时显式给出列名和目标类型，目的是方便用户定位坏值或错误配置。
    #[error("列 {column} 无法转换为 {target_type}: {message}")]
    CastColumn {
        column: String,
        target_type: String,
        message: String,
    },
    // 2026-03-21: 包装 DataFrame 列替换失败，目的是把底层结构错误统一收口成 Tool 级语义。
    #[error("无法写回列 {column}: {message}")]
    ReplaceColumn { column: String, message: String },
}

// 2026-03-21: 这里对已加载表执行显式类型转换，目的是为聚合、统计和建模提供真正的数值化承载层。
pub fn cast_column_types(
    loaded: &LoadedTable,
    casts: &[CastColumnSpec],
) -> Result<LoadedTable, CastError> {
    if casts.is_empty() {
        return Err(CastError::EmptyCasts);
    }

    let mut dataframe = loaded.dataframe.clone();

    for cast in casts {
        let column_index = dataframe
            .get_column_index(&cast.column)
            .ok_or_else(|| CastError::MissingColumn(cast.column.clone()))?;
        let casted_series = dataframe
            .column(&cast.column)
            .map_err(|_| CastError::MissingColumn(cast.column.clone()))?
            .as_materialized_series()
            .cast_with_options(&cast.target_type.to_polars_dtype(), CastOptions::Strict)
            .map_err(|error| CastError::CastColumn {
                column: cast.column.clone(),
                target_type: cast.target_type.label().to_string(),
                message: error.to_string(),
            })?;

        dataframe
            .replace_column(column_index, casted_series)
            .map_err(|error| CastError::ReplaceColumn {
                column: cast.column.clone(),
                message: error.to_string(),
            })?;
    }

    let handle = TableHandle::new_confirmed(
        loaded.handle.source_path(),
        loaded.handle.sheet_name(),
        loaded.handle.columns().to_vec(),
    );

    Ok(LoadedTable { handle, dataframe })
}

// 2026-03-21: 这里抽出列类型摘要生成逻辑，目的是让 CLI 返回与后续问答界面展示共用同一份结果。
pub fn summarize_column_types(dataframe: &DataFrame) -> Vec<ColumnTypeSummary> {
    dataframe
        .get_columns()
        .iter()
        .map(|column| ColumnTypeSummary {
            column: column.name().to_string(),
            dtype: dtype_label(column.as_materialized_series().dtype()).to_string(),
        })
        .collect()
}

// 2026-03-21: 这里把 Polars 类型翻译成稳定文案，目的是避免把 `i64`、`str` 这类技术缩写直接暴露给用户。
fn dtype_label(dtype: &DataType) -> &'static str {
    match dtype {
        DataType::String => "string",
        DataType::Int64 => "int64",
        DataType::Float64 => "float64",
        DataType::Boolean => "boolean",
        DataType::Int32 => "int32",
        DataType::Float32 => "float32",
        DataType::UInt64 => "uint64",
        DataType::UInt32 => "uint32",
        DataType::UInt16 => "uint16",
        DataType::UInt8 => "uint8",
        DataType::Int16 => "int16",
        DataType::Int8 => "int8",
        _ => "unknown",
    }
}
