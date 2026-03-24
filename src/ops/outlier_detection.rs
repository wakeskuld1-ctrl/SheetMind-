use polars::prelude::{DataFrame, DataType, NamedFrom, Series};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::handles::TableHandle;
use crate::frame::loader::LoadedTable;

// 2026-03-25: 这里定义异常值检测方法枚举，原因是统计诊断层第一版先只支持最常见的 IQR 和 Z-Score 两种口径；目的是让上层 Skill 能显式说明“按哪种传统统计方法检测异常值”。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OutlierDetectionMethod {
    Iqr,
    Zscore,
}

// 2026-03-25: 这里定义单列异常摘要，原因是异常值检测不能只返回行标记；目的是让用户和上层 Skill 都能直接读取“哪一列有多少异常、阈值是多少”。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OutlierSummary {
    pub column: String,
    pub method: String,
    pub outlier_count: usize,
    pub outlier_ratio: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lower_bound: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upper_bound: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mean: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stddev: Option<f64>,
}

// 2026-03-25: 这里定义人类可读摘要，原因是低 IT 用户通常看不懂统计阈值本身；目的是把结构化结果再翻译成可以直接用于问答界面的中文结论。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OutlierHumanSummary {
    pub overall: String,
    #[serde(default)]
    pub key_points: Vec<String>,
    pub recommended_next_step: String,
}

// 2026-03-25: 这里定义异常值检测统一输出，原因是 dispatcher 需要同时返回摘要和可继续链式复用的新结果表；目的是让异常值标记既能解释也能继续被筛选或导出。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OutlierDetectionResult {
    pub method: String,
    pub row_count: usize,
    #[serde(default)]
    pub outlier_summaries: Vec<OutlierSummary>,
    pub human_summary: OutlierHumanSummary,
}

// 2026-03-25: 这里定义异常值检测错误类型，原因是第一版要明确区分缺列、非数值列和有效样本不足；目的是让用户收到可执行的中文报错，而不是模糊失败。
#[derive(Debug, Error)]
pub enum OutlierDetectionError {
    #[error("outlier_detection 至少需要一个 columns 参数")]
    EmptyColumns,
    #[error("列 `{column}` 不存在")]
    MissingColumn { column: String },
    #[error("列 `{column}` 不是可做异常值检测的数值列，请先做类型转换")]
    NonNumericColumn { column: String },
    #[error("列 `{column}` 没有足够的有效数值，暂时无法做异常值检测")]
    NoValidValues { column: String },
    #[error("无法写入异常标记列 `{column}`: {message}")]
    WriteColumn { column: String, message: String },
}

// 2026-03-25: 这里提供异常值检测主入口，原因是统计诊断型能力第二步要把“可疑极端记录”明确标出来；目的是让用户后续可以继续筛选、导出或人工复核这些记录。
pub fn outlier_detection(
    loaded: &LoadedTable,
    columns: &[&str],
    method: OutlierDetectionMethod,
) -> Result<(LoadedTable, OutlierDetectionResult), OutlierDetectionError> {
    if columns.is_empty() {
        return Err(OutlierDetectionError::EmptyColumns);
    }

    let mut dataframe = loaded.dataframe.clone();
    let mut summaries = Vec::with_capacity(columns.len());

    for column in columns {
        let values = extract_numeric_values(loaded, column)?;
        let (flags, summary) = match method {
            OutlierDetectionMethod::Iqr => detect_outliers_by_iqr(column, &values)?,
            OutlierDetectionMethod::Zscore => detect_outliers_by_zscore(column, &values)?,
        };
        let flag_column_name = format!("{column}__is_outlier");

        // 2026-03-25: 这里把逐行异常标记写回 DataFrame，原因是异常值检测不应该只停留在摘要层；目的是让结果表可以继续进入筛选、导出和汇报链路。
        dataframe
            .with_column(Series::new(flag_column_name.clone().into(), flags))
            .map_err(|error| OutlierDetectionError::WriteColumn {
                column: flag_column_name,
                message: error.to_string(),
            })?;
        summaries.push(summary);
    }

    let result = OutlierDetectionResult {
        method: outlier_method_label(method).to_string(),
        row_count: dataframe.height(),
        human_summary: build_outlier_human_summary(&summaries),
        outlier_summaries: summaries,
    };

    let output = LoadedTable {
        handle: rebuild_handle(loaded, &dataframe),
        dataframe,
    };
    Ok((output, result))
}

// 2026-03-25: 这里集中重建输出表句柄，原因是新结果表多了异常标记列；目的是让后续 result_ref 持久化时保留最新列清单。
fn rebuild_handle(loaded: &LoadedTable, dataframe: &DataFrame) -> TableHandle {
    TableHandle::new_confirmed(
        loaded.handle.source_path(),
        loaded.handle.sheet_name(),
        dataframe
            .get_column_names_str()
            .into_iter()
            .map(|name| name.to_string())
            .collect(),
    )
}

// 2026-03-25: 这里统一把目标列抽成可空数值向量，原因是异常值检测要在保留原行位置的同时跳过缺失值；目的是让后续写回布尔标记时仍然逐行对齐原表。
fn extract_numeric_values(
    loaded: &LoadedTable,
    column: &str,
) -> Result<Vec<Option<f64>>, OutlierDetectionError> {
    let series = loaded
        .dataframe
        .column(column)
        .map_err(|_| OutlierDetectionError::MissingColumn {
            column: column.to_string(),
        })?
        .as_materialized_series();
    let casted = series
        .cast(&DataType::Float64)
        .map_err(|_| OutlierDetectionError::NonNumericColumn {
            column: column.to_string(),
        })?;
    let values = casted
        .f64()
        .map_err(|_| OutlierDetectionError::NonNumericColumn {
            column: column.to_string(),
        })?
        .into_iter()
        .collect::<Vec<_>>();

    if values.iter().flatten().next().is_none() {
        return Err(OutlierDetectionError::NoValidValues {
            column: column.to_string(),
        });
    }

    Ok(values)
}

// 2026-03-25: 这里实现 IQR 异常值检测，原因是它对业务表中的极端值更稳健；目的是优先满足 Excel 场景下“明显离群点”识别。
fn detect_outliers_by_iqr(
    column: &str,
    values: &[Option<f64>],
) -> Result<(Vec<bool>, OutlierSummary), OutlierDetectionError> {
    let valid_values = values.iter().flatten().copied().collect::<Vec<_>>();
    if valid_values.is_empty() {
        return Err(OutlierDetectionError::NoValidValues {
            column: column.to_string(),
        });
    }

    let mut sorted = valid_values.clone();
    sorted.sort_by(|left, right| left.total_cmp(right));
    let q1 = quantile(&sorted, 0.25);
    let q3 = quantile(&sorted, 0.75);
    let iqr = q3 - q1;
    let lower_bound = q1 - 1.5 * iqr;
    let upper_bound = q3 + 1.5 * iqr;
    let flags = values
        .iter()
        .map(|value| {
            value
                .map(|item| item < lower_bound || item > upper_bound)
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    let outlier_count = flags.iter().filter(|flag| **flag).count();

    Ok((
        flags,
        OutlierSummary {
            column: column.to_string(),
            method: outlier_method_label(OutlierDetectionMethod::Iqr).to_string(),
            outlier_count,
            outlier_ratio: outlier_count as f64 / values.len() as f64,
            lower_bound: Some(lower_bound),
            upper_bound: Some(upper_bound),
            mean: None,
            stddev: None,
        },
    ))
}

// 2026-03-25: 这里实现 Z-Score 异常值检测，原因是第一版需要给用户一个传统标准差口径备选；目的是让方法选择保持最小但不单一。
fn detect_outliers_by_zscore(
    column: &str,
    values: &[Option<f64>],
) -> Result<(Vec<bool>, OutlierSummary), OutlierDetectionError> {
    let valid_values = values.iter().flatten().copied().collect::<Vec<_>>();
    if valid_values.is_empty() {
        return Err(OutlierDetectionError::NoValidValues {
            column: column.to_string(),
        });
    }

    let mean = valid_values.iter().sum::<f64>() / valid_values.len() as f64;
    let variance = valid_values
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / valid_values.len() as f64;
    let stddev = variance.sqrt();
    let flags = if stddev.abs() <= 1e-12 {
        vec![false; values.len()]
    } else {
        values
            .iter()
            .map(|value| {
                value
                    .map(|item| ((item - mean) / stddev).abs() > 3.0)
                    .unwrap_or(false)
            })
            .collect::<Vec<_>>()
    };
    let outlier_count = flags.iter().filter(|flag| **flag).count();

    Ok((
        flags,
        OutlierSummary {
            column: column.to_string(),
            method: outlier_method_label(OutlierDetectionMethod::Zscore).to_string(),
            outlier_count,
            outlier_ratio: outlier_count as f64 / values.len() as f64,
            lower_bound: None,
            upper_bound: None,
            mean: Some(mean),
            stddev: Some(stddev),
        },
    ))
}

// 2026-03-25: 这里统一计算分位数，原因是 IQR 需要稳定的 Q1/Q3 口径；目的是避免后续统计诊断 Tool 各自复制分位数逻辑。
fn quantile(sorted_values: &[f64], quantile: f64) -> f64 {
    if sorted_values.len() == 1 {
        return sorted_values[0];
    }

    let position = quantile.clamp(0.0, 1.0) * (sorted_values.len() - 1) as f64;
    let lower_index = position.floor() as usize;
    let upper_index = position.ceil() as usize;
    if lower_index == upper_index {
        sorted_values[lower_index]
    } else {
        let lower_value = sorted_values[lower_index];
        let upper_value = sorted_values[upper_index];
        lower_value + (upper_value - lower_value) * (position - lower_index as f64)
    }
}

// 2026-03-25: 这里统一生成人类摘要，原因是异常值诊断需要先告诉用户“有没有明显异常、下一步怎么处理”；目的是降低业务用户理解门槛。
fn build_outlier_human_summary(summaries: &[OutlierSummary]) -> OutlierHumanSummary {
    let total_outliers = summaries.iter().map(|summary| summary.outlier_count).sum::<usize>();
    let key_points = summaries
        .iter()
        .filter(|summary| summary.outlier_count > 0)
        .map(|summary| {
            format!(
                "`{}` 检测到 {} 个异常值，占比约为 {:.2}%",
                summary.column,
                summary.outlier_count,
                summary.outlier_ratio * 100.0
            )
        })
        .collect::<Vec<_>>();

    let overall = if total_outliers == 0 {
        "当前检测列没有发现明显异常值，可以继续做分布观察或建模前检查。".to_string()
    } else {
        format!("系统已检测到 {total_outliers} 个可疑异常值，建议先核查来源再继续分析。")
    };

    OutlierHumanSummary {
        overall,
        key_points,
        recommended_next_step:
            "建议先筛出异常标记为 true 的记录做人工复核，再决定是否剔除、修正或单独解释。"
                .to_string(),
    }
}

// 2026-03-25: 这里统一输出方法标签，原因是外层 JSON 协议要稳定返回机器可读方法名；目的是减少 dispatcher 和 Skill 的协议漂移。
fn outlier_method_label(method: OutlierDetectionMethod) -> &'static str {
    match method {
        OutlierDetectionMethod::Iqr => "iqr",
        OutlierDetectionMethod::Zscore => "zscore",
    }
}
