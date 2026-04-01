use std::collections::BTreeSet;

use polars::prelude::DataType;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::frame::loader::LoadedTable;

// 2026-03-21: 这里统一定义缺失处理策略，目的是让回归、分类和后续聚类都共享同一套建模前处理口径。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MissingStrategy {
    // 2026-03-21: 这里先只支持删行，目的是用最保守、最可解释的方式完成 V1 缺失处理。
    DropRows,
}

// 2026-03-21: 这里定义回归样本准备结果，目的是把线性回归和后续聚类等模型共同依赖的数值矩阵稳定暴露出来。
#[derive(Debug, Clone, PartialEq)]
pub struct PreparedRegressionDataset {
    pub features: Vec<String>,
    pub target: String,
    pub feature_matrix: Vec<Vec<f64>>,
    pub targets: Vec<f64>,
    pub row_count_used: usize,
    pub dropped_rows: usize,
}

// 2026-03-21: 这里定义二分类样本准备结果，目的是让逻辑回归直接消费统一的特征矩阵、标签向量和标签映射。
#[derive(Debug, Clone, PartialEq)]
pub struct PreparedBinaryClassificationDataset {
    pub features: Vec<String>,
    pub target: String,
    pub feature_matrix: Vec<Vec<f64>>,
    pub targets: Vec<f64>,
    pub positive_label: String,
    pub negative_label: String,
    pub row_count_used: usize,
    pub dropped_rows: usize,
}

// 2026-03-21: 这里定义聚类样本准备结果，目的是把聚类需要的纯特征矩阵与原始行索引统一暴露给算法层。
#[derive(Debug, Clone, PartialEq)]
pub struct PreparedClusteringDataset {
    pub features: Vec<String>,
    pub feature_matrix: Vec<Vec<f64>>,
    pub source_row_indices: Vec<usize>,
    pub row_count_used: usize,
    pub dropped_rows: usize,
}

// 2026-03-21: 这里统一定义建模前准备错误，目的是把列缺失、类型不匹配、标签不合法和样本问题分层暴露给上层模型 Tool。
#[derive(Debug, Error)]
pub enum ModelPrepError {
    #[error("建模至少需要 1 个特征列")]
    EmptyFeatures,
    #[error("找不到列: {0}")]
    MissingColumn(String),
    #[error("目标列 `{0}` 不能同时作为特征列")]
    TargetIncludedInFeatures(String),
    #[error("特征列 `{0}` 不是数值列，暂时不能用于建模")]
    NonNumericFeature(String),
    #[error("目标列 `{0}` 不是数值列，暂时不能做回归")]
    NonNumericRegressionTarget(String),
    #[error("目标列 `{0}` 不是可识别的二分类列，逻辑回归 V1 只支持二分类")]
    InvalidBinaryTarget(String),
    #[error("目标列 `{column}` 当前识别出了 {distinct_count} 个不同取值，逻辑回归 V1 只支持二分类")]
    NonBinaryTarget {
        column: String,
        distinct_count: usize,
    },
    #[error("正类标签 `{label}` 在目标列 `{column}` 中不存在，请确认后重试")]
    PositiveLabelNotFound { column: String, label: String },
    // 2026-03-22: 这里把单一类别错误改成带行动建议的人话，目的是让低 IT 用户知道应先检查目标列分布或更换目标列。
    #[error(
        "目标列 `{0}` 只有一个类别，暂时不能做逻辑回归；建议先看目标列分布，确认是否选错目标列，或更换为真正的二分类目标列后再试"
    )]
    SingleClassTarget(String),
    #[error("删除缺失值后，没有可用于建模的有效数据")]
    NoUsableRows,
    #[error("无法读取列 `{column}`: {message}")]
    ReadColumn { column: String, message: String },
}

// 2026-03-21: 这里提供回归样本准备入口，目的是让线性回归等数值目标模型共享统一前处理链路。
pub fn prepare_regression_dataset(
    loaded: &LoadedTable,
    features: &[&str],
    target: &str,
    intercept: bool,
    missing_strategy: MissingStrategy,
) -> Result<PreparedRegressionDataset, ModelPrepError> {
    validate_feature_target_selection(features, target)?;
    let feature_values = features
        .iter()
        .map(|feature| extract_numeric_column(loaded, feature, false))
        .collect::<Result<Vec<_>, _>>()?;
    let target_values = extract_numeric_column(loaded, target, true)?;

    let prepared = match missing_strategy {
        MissingStrategy::DropRows => {
            build_numeric_rows(features, target, &feature_values, &target_values, intercept)?
        }
    };

    Ok(prepared)
}

// 2026-03-21: 这里提供二分类样本准备入口，目的是把标签识别、正类映射和缺失删行统一收口给逻辑回归复用。
pub fn prepare_binary_classification_dataset(
    loaded: &LoadedTable,
    features: &[&str],
    target: &str,
    intercept: bool,
    missing_strategy: MissingStrategy,
    positive_label: Option<&str>,
) -> Result<PreparedBinaryClassificationDataset, ModelPrepError> {
    validate_feature_target_selection(features, target)?;
    let feature_values = features
        .iter()
        .map(|feature| extract_numeric_column(loaded, feature, false))
        .collect::<Result<Vec<_>, _>>()?;
    let target_values = extract_binary_target_column(loaded, target, positive_label)?;

    let prepared = match missing_strategy {
        MissingStrategy::DropRows => build_binary_rows(
            features,
            target,
            &feature_values,
            &target_values.values,
            intercept,
            target_values.positive_label,
            target_values.negative_label,
        )?,
    };

    Ok(prepared)
}

// 2026-03-21: 这里提供聚类样本准备入口，目的是让聚类和回归/分类共享同一套数值列校验与缺失删行口径。
pub fn prepare_clustering_dataset(
    loaded: &LoadedTable,
    features: &[&str],
    missing_strategy: MissingStrategy,
) -> Result<PreparedClusteringDataset, ModelPrepError> {
    if features.is_empty() {
        return Err(ModelPrepError::EmptyFeatures);
    }

    let feature_values = features
        .iter()
        .map(|feature| extract_numeric_column(loaded, feature, false))
        .collect::<Result<Vec<_>, _>>()?;

    let prepared = match missing_strategy {
        MissingStrategy::DropRows => build_feature_only_rows(features, &feature_values)?,
    };

    Ok(prepared)
}

struct BinaryTargetValues {
    values: Vec<Option<f64>>,
    positive_label: String,
    negative_label: String,
}

// 2026-03-21: 这里集中校验特征列与目标列选择，目的是避免各个模型 Tool 重复实现同一套门禁逻辑。
fn validate_feature_target_selection(
    features: &[&str],
    target: &str,
) -> Result<(), ModelPrepError> {
    if features.is_empty() {
        return Err(ModelPrepError::EmptyFeatures);
    }

    if features.iter().any(|feature| *feature == target) {
        return Err(ModelPrepError::TargetIncludedInFeatures(target.to_string()));
    }

    Ok(())
}

// 2026-03-21: 这里统一读取数值列，目的是把回归与分类共同依赖的数值特征抽到公共层。
fn extract_numeric_column(
    loaded: &LoadedTable,
    column: &str,
    is_regression_target: bool,
) -> Result<Vec<Option<f64>>, ModelPrepError> {
    let series = loaded
        .dataframe
        .column(column)
        .map_err(|_| ModelPrepError::MissingColumn(column.to_string()))?
        .as_materialized_series();

    if !is_numeric_dtype(series.dtype()) {
        return if is_regression_target {
            Err(ModelPrepError::NonNumericRegressionTarget(
                column.to_string(),
            ))
        } else {
            Err(ModelPrepError::NonNumericFeature(column.to_string()))
        };
    }

    let casted = series
        .cast(&DataType::Float64)
        .map_err(|error| ModelPrepError::ReadColumn {
            column: column.to_string(),
            message: error.to_string(),
        })?;

    Ok(casted
        .f64()
        .map_err(|error| ModelPrepError::ReadColumn {
            column: column.to_string(),
            message: error.to_string(),
        })?
        .into_iter()
        .collect())
}

// 2026-03-21: 这里集中把目标列识别成二分类标签，目的是避免逻辑回归自己再处理文本/布尔/数值目标映射。
fn extract_binary_target_column(
    loaded: &LoadedTable,
    column: &str,
    positive_label: Option<&str>,
) -> Result<BinaryTargetValues, ModelPrepError> {
    let series = loaded
        .dataframe
        .column(column)
        .map_err(|_| ModelPrepError::MissingColumn(column.to_string()))?
        .as_materialized_series();

    match series.dtype() {
        DataType::Boolean => extract_boolean_binary_target(series, column, positive_label),
        dtype if is_numeric_dtype(dtype) => {
            extract_numeric_binary_target(series, column, positive_label)
        }
        _ => extract_text_binary_target(series, column, positive_label),
    }
}

// 2026-03-21: 这里处理布尔目标列，目的是让 true/false 直接进入逻辑回归而不需要上层先手工编码。
fn extract_boolean_binary_target(
    series: &polars::prelude::Series,
    column: &str,
    positive_label: Option<&str>,
) -> Result<BinaryTargetValues, ModelPrepError> {
    let requested_positive = positive_label.unwrap_or("true");
    if !matches!(requested_positive, "true" | "false") {
        return Err(ModelPrepError::PositiveLabelNotFound {
            column: column.to_string(),
            label: requested_positive.to_string(),
        });
    }

    let positive_is_true = requested_positive == "true";
    let values = series
        .bool()
        .map_err(|error| ModelPrepError::ReadColumn {
            column: column.to_string(),
            message: error.to_string(),
        })?
        .into_iter()
        .map(|value| value.map(|flag| if flag == positive_is_true { 1.0 } else { 0.0 }))
        .collect::<Vec<_>>();

    ensure_binary_has_both_classes(column, &values)?;

    Ok(BinaryTargetValues {
        values,
        positive_label: requested_positive.to_string(),
        negative_label: if positive_is_true { "false" } else { "true" }.to_string(),
    })
}

// 2026-03-21: 这里处理数值目标列，目的是只允许最保守的 0/1 二值进入逻辑回归。
fn extract_numeric_binary_target(
    series: &polars::prelude::Series,
    column: &str,
    positive_label: Option<&str>,
) -> Result<BinaryTargetValues, ModelPrepError> {
    let casted = series
        .cast(&DataType::Float64)
        .map_err(|error| ModelPrepError::ReadColumn {
            column: column.to_string(),
            message: error.to_string(),
        })?;
    let raw_values = casted
        .f64()
        .map_err(|error| ModelPrepError::ReadColumn {
            column: column.to_string(),
            message: error.to_string(),
        })?
        .into_iter()
        .collect::<Vec<_>>();

    let mut distinct = BTreeSet::new();
    for value in raw_values.iter().flatten() {
        if !approximately_binary(*value) {
            return Err(ModelPrepError::InvalidBinaryTarget(column.to_string()));
        }
        distinct.insert(if *value >= 0.5 { 1_i32 } else { 0_i32 });
    }

    if distinct.len() > 2 {
        return Err(ModelPrepError::NonBinaryTarget {
            column: column.to_string(),
            distinct_count: distinct.len(),
        });
    }

    let requested_positive = positive_label.unwrap_or("1");
    if !matches!(requested_positive, "0" | "1") {
        return Err(ModelPrepError::PositiveLabelNotFound {
            column: column.to_string(),
            label: requested_positive.to_string(),
        });
    }

    let positive_is_one = requested_positive == "1";
    let values = raw_values
        .into_iter()
        .map(|value| {
            value.map(|item| {
                let is_one = item >= 0.5;
                if is_one == positive_is_one { 1.0 } else { 0.0 }
            })
        })
        .collect::<Vec<_>>();

    ensure_binary_has_both_classes(column, &values)?;

    Ok(BinaryTargetValues {
        values,
        positive_label: requested_positive.to_string(),
        negative_label: if positive_is_one { "0" } else { "1" }.to_string(),
    })
}

// 2026-03-21: 这里处理文本目标列，目的是在保守前提下支持“成功/失败”等非技术用户更常见的标签形式。
fn extract_text_binary_target(
    series: &polars::prelude::Series,
    column: &str,
    positive_label: Option<&str>,
) -> Result<BinaryTargetValues, ModelPrepError> {
    let mut distinct = BTreeSet::new();
    let mut raw_values = Vec::with_capacity(series.len());

    for row_index in 0..series.len() {
        let value = series
            .get(row_index)
            .map_err(|error| ModelPrepError::ReadColumn {
                column: column.to_string(),
                message: error.to_string(),
            })?;

        if value.is_null() {
            raw_values.push(None);
            continue;
        }

        let rendered = series
            .str_value(row_index)
            .map_err(|error| ModelPrepError::ReadColumn {
                column: column.to_string(),
                message: error.to_string(),
            })?
            .trim()
            .to_string();

        if rendered.is_empty() {
            raw_values.push(None);
            continue;
        }

        distinct.insert(rendered.clone());
        raw_values.push(Some(rendered));
    }

    if distinct.len() != 2 {
        return if distinct.len() <= 1 {
            Err(ModelPrepError::SingleClassTarget(column.to_string()))
        } else {
            Err(ModelPrepError::NonBinaryTarget {
                column: column.to_string(),
                distinct_count: distinct.len(),
            })
        };
    }

    let labels = distinct.into_iter().collect::<Vec<_>>();
    let positive = match positive_label {
        Some(label) if labels.iter().any(|existing| existing == label) => label.to_string(),
        Some(label) => {
            return Err(ModelPrepError::PositiveLabelNotFound {
                column: column.to_string(),
                label: label.to_string(),
            });
        }
        None => labels[1].clone(),
    };
    let negative = labels
        .iter()
        .find(|label| *label != &positive)
        .cloned()
        .ok_or_else(|| ModelPrepError::SingleClassTarget(column.to_string()))?;

    let values = raw_values
        .into_iter()
        .map(|value| value.map(|item| if item == positive { 1.0 } else { 0.0 }))
        .collect::<Vec<_>>();

    ensure_binary_has_both_classes(column, &values)?;

    Ok(BinaryTargetValues {
        values,
        positive_label: positive,
        negative_label: negative,
    })
}

// 2026-03-21: 这里按统一删行策略构造回归样本，目的是让所有数值模型都共享同一套有效行筛选语义。
fn build_numeric_rows(
    features: &[&str],
    target: &str,
    feature_values: &[Vec<Option<f64>>],
    target_values: &[Option<f64>],
    intercept: bool,
) -> Result<PreparedRegressionDataset, ModelPrepError> {
    let mut feature_matrix = Vec::new();
    let mut targets = Vec::new();
    let mut dropped_rows = 0_usize;

    for row_index in 0..target_values.len() {
        let Some(target_value) = target_values[row_index] else {
            dropped_rows += 1;
            continue;
        };

        let mut row = Vec::with_capacity(features.len() + usize::from(intercept));
        if intercept {
            row.push(1.0);
        }

        let mut complete = true;
        for feature in feature_values {
            match feature[row_index] {
                Some(value) => row.push(value),
                None => {
                    complete = false;
                    break;
                }
            }
        }

        if complete {
            feature_matrix.push(row);
            targets.push(target_value);
        } else {
            dropped_rows += 1;
        }
    }

    if targets.is_empty() {
        return Err(ModelPrepError::NoUsableRows);
    }

    Ok(PreparedRegressionDataset {
        features: features
            .iter()
            .map(|feature| (*feature).to_string())
            .collect(),
        target: target.to_string(),
        row_count_used: targets.len(),
        dropped_rows,
        feature_matrix,
        targets,
    })
}

// 2026-03-21: 这里按统一删行策略构造二分类样本，目的是让标签映射和数值特征在进入逻辑回归前就已准备完毕。
fn build_binary_rows(
    features: &[&str],
    target: &str,
    feature_values: &[Vec<Option<f64>>],
    target_values: &[Option<f64>],
    intercept: bool,
    positive_label: String,
    negative_label: String,
) -> Result<PreparedBinaryClassificationDataset, ModelPrepError> {
    let mut feature_matrix = Vec::new();
    let mut targets = Vec::new();
    let mut dropped_rows = 0_usize;

    for row_index in 0..target_values.len() {
        let Some(target_value) = target_values[row_index] else {
            dropped_rows += 1;
            continue;
        };

        let mut row = Vec::with_capacity(features.len() + usize::from(intercept));
        if intercept {
            row.push(1.0);
        }

        let mut complete = true;
        for feature in feature_values {
            match feature[row_index] {
                Some(value) => row.push(value),
                None => {
                    complete = false;
                    break;
                }
            }
        }

        if complete {
            feature_matrix.push(row);
            targets.push(target_value);
        } else {
            dropped_rows += 1;
        }
    }

    if targets.is_empty() {
        return Err(ModelPrepError::NoUsableRows);
    }

    ensure_binary_has_both_classes(
        target,
        &targets.iter().copied().map(Some).collect::<Vec<_>>(),
    )?;

    let row_count_used = targets.len();

    Ok(PreparedBinaryClassificationDataset {
        features: features
            .iter()
            .map(|feature| (*feature).to_string())
            .collect(),
        target: target.to_string(),
        feature_matrix,
        targets,
        positive_label,
        negative_label,
        row_count_used,
        dropped_rows,
    })
}

// 2026-03-21: 这里按统一删行口径构造聚类样本，目的是确保聚类与回归/分类面对缺失值时保持同一行为。
fn build_feature_only_rows(
    features: &[&str],
    feature_values: &[Vec<Option<f64>>],
) -> Result<PreparedClusteringDataset, ModelPrepError> {
    let row_total = feature_values
        .first()
        .map(|values| values.len())
        .unwrap_or(0);
    let mut feature_matrix = Vec::new();
    let mut source_row_indices = Vec::new();
    let mut dropped_rows = 0_usize;

    for row_index in 0..row_total {
        let mut row = Vec::with_capacity(features.len());
        let mut complete = true;

        for feature in feature_values {
            match feature[row_index] {
                Some(value) => row.push(value),
                None => {
                    complete = false;
                    break;
                }
            }
        }

        if complete {
            feature_matrix.push(row);
            source_row_indices.push(row_index);
        } else {
            dropped_rows += 1;
        }
    }

    if feature_matrix.is_empty() {
        return Err(ModelPrepError::NoUsableRows);
    }

    Ok(PreparedClusteringDataset {
        features: features
            .iter()
            .map(|feature| (*feature).to_string())
            .collect(),
        feature_matrix,
        source_row_indices: source_row_indices.clone(),
        row_count_used: source_row_indices.len(),
        dropped_rows,
    })
}

// 2026-03-21: 这里集中校验二分类是否同时存在正负两类，目的是避免逻辑回归只看到单类样本时再晚失败。
fn ensure_binary_has_both_classes(
    column: &str,
    values: &[Option<f64>],
) -> Result<(), ModelPrepError> {
    let mut has_zero = false;
    let mut has_one = false;

    for value in values.iter().flatten() {
        if *value >= 0.5 {
            has_one = true;
        } else {
            has_zero = true;
        }
    }

    match (has_zero, has_one) {
        (true, true) => Ok(()),
        _ => Err(ModelPrepError::SingleClassTarget(column.to_string())),
    }
}

// 2026-03-21: 这里集中维护 V1 允许进入建模层的数值类型，目的是保证特征列统一走可靠数值计算链路。
fn is_numeric_dtype(dtype: &DataType) -> bool {
    matches!(
        dtype,
        DataType::Float64
            | DataType::Float32
            | DataType::Int64
            | DataType::Int32
            | DataType::Int16
            | DataType::Int8
            | DataType::UInt64
            | DataType::UInt32
            | DataType::UInt16
            | DataType::UInt8
    )
}

// 2026-03-21: 这里保守判断数值标签是否接近 0/1，目的是避免把任意两个数值错当成二值标签。
fn approximately_binary(value: f64) -> bool {
    (value - 0.0).abs() <= 1e-9 || (value - 1.0).abs() <= 1e-9
}
