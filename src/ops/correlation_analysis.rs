use serde::Serialize;
use thiserror::Error;

use crate::frame::loader::LoadedTable;

// 2026-03-25: 这里定义单列相关性结果，原因是上层 Skill 需要稳定读取“目标列 vs 特征列”的排序结果；目的是让统计诊断层先形成可复用的结构化协议。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CorrelationItem {
    pub feature_column: String,
    pub coefficient: f64,
    pub paired_row_count: usize,
}

// 2026-03-25: 这里定义相关性分析的人类摘要，原因是低 IT 用户不会直接消费纯系数数组；目的是让分析建模层先具备“先给观察结论”的能力。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CorrelationHumanSummary {
    pub overall: String,
    #[serde(default)]
    pub key_points: Vec<String>,
    pub recommended_next_step: String,
}

// 2026-03-25: 这里定义相关性分析统一输出，原因是 dispatcher 和 Skill 都需要稳定字段来读取方法、目标列、排序结果与摘要；目的是减少后续协议漂移。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CorrelationAnalysisResult {
    pub method: String,
    pub target_column: String,
    pub row_count: usize,
    pub feature_count: usize,
    #[serde(default)]
    pub correlations: Vec<CorrelationItem>,
    #[serde(default)]
    pub top_positive: Vec<CorrelationItem>,
    #[serde(default)]
    pub top_negative: Vec<CorrelationItem>,
    pub human_summary: CorrelationHumanSummary,
}

// 2026-03-25: 这里定义相关性分析错误，原因是第一版要给出比“计算失败”更直白的中文提示；目的是让用户知道是列类型问题、样本不足还是列恒定。
#[derive(Debug, Error)]
pub enum CorrelationAnalysisError {
    #[error("correlation_analysis 至少需要一个 feature_columns 参数")]
    EmptyFeatureColumns,
    #[error("列 `{column}` 不存在")]
    MissingColumn { column: String },
    #[error("列 `{column}` 不是可计算相关性的数值列，请先做类型转换")]
    NonNumericColumn { column: String },
    #[error("列 `{column}` 在成对有效样本中只有 {actual} 行，至少需要 {required} 行才能计算相关性")]
    NotEnoughPairedRows {
        column: String,
        required: usize,
        actual: usize,
    },
    #[error("列 `{column}` 在当前有效样本中没有波动，暂时无法计算相关性")]
    ConstantColumn { column: String },
}

// 2026-03-25: 这里提供相关性分析主入口，原因是统计诊断型能力要先补“目标列和候选特征列的关系排序”；目的是把分析建模层前置观察正式沉到 Tool 层。
pub fn correlation_analysis(
    loaded: &LoadedTable,
    target_column: &str,
    feature_columns: &[&str],
) -> Result<CorrelationAnalysisResult, CorrelationAnalysisError> {
    if feature_columns.is_empty() {
        return Err(CorrelationAnalysisError::EmptyFeatureColumns);
    }

    let target_values = extract_numeric_column(loaded, target_column)?;
    let mut correlations = feature_columns
        .iter()
        .map(|feature_column| {
            build_correlation_item(target_column, &target_values, loaded, feature_column)
        })
        .collect::<Result<Vec<_>, _>>()?;

    // 2026-03-25: 这里按相关系数从高到低排序，原因是第一版要先让“最强正相关”直接排在最前面；目的是便于问答层直接给用户一个最直观的观察顺序。
    correlations.sort_by(|left, right| {
        right
            .coefficient
            .total_cmp(&left.coefficient)
            .then_with(|| left.feature_column.cmp(&right.feature_column))
    });

    let top_positive = correlations
        .iter()
        .filter(|item| item.coefficient >= 0.0)
        .cloned()
        .collect::<Vec<_>>();
    let mut top_negative = correlations
        .iter()
        .filter(|item| item.coefficient < 0.0)
        .cloned()
        .collect::<Vec<_>>();
    top_negative.sort_by(|left, right| {
        left.coefficient
            .total_cmp(&right.coefficient)
            .then_with(|| left.feature_column.cmp(&right.feature_column))
    });

    let human_summary = build_human_summary(
        target_column,
        loaded.dataframe.height(),
        &top_positive,
        &top_negative,
    );

    Ok(CorrelationAnalysisResult {
        method: "pearson".to_string(),
        target_column: target_column.to_string(),
        row_count: loaded.dataframe.height(),
        feature_count: feature_columns.len(),
        correlations,
        top_positive,
        top_negative,
        human_summary,
    })
}

// 2026-03-25: 这里把列统一抽成可空数值向量，原因是相关性计算只接受数值列，但仍要兼容缺失值；目的是把“取列 + 数值化 + 空值保留”集中在一个地方维护。
fn extract_numeric_column(
    loaded: &LoadedTable,
    column: &str,
) -> Result<Vec<Option<f64>>, CorrelationAnalysisError> {
    let series = loaded
        .dataframe
        .column(column)
        .map_err(|_| CorrelationAnalysisError::MissingColumn {
            column: column.to_string(),
        })?
        .as_materialized_series();
    let casted = series
        .cast(&polars::prelude::DataType::Float64)
        .map_err(|_| CorrelationAnalysisError::NonNumericColumn {
            column: column.to_string(),
        })?;
    let values = casted
        .f64()
        .map_err(|_| CorrelationAnalysisError::NonNumericColumn {
            column: column.to_string(),
        })?
        .into_iter()
        .collect::<Vec<_>>();

    Ok(values)
}

// 2026-03-25: 这里为每个特征列构造单条相关性记录，原因是第一版输出需要逐列说明系数和有效样本数；目的是让后续 top_positive/top_negative 都能复用同一底层计算。
fn build_correlation_item(
    target_column: &str,
    target_values: &[Option<f64>],
    loaded: &LoadedTable,
    feature_column: &str,
) -> Result<CorrelationItem, CorrelationAnalysisError> {
    let feature_values = extract_numeric_column(loaded, feature_column)?;
    let paired_values = target_values
        .iter()
        .zip(feature_values.iter())
        .filter_map(|(target, feature)| target.zip(*feature))
        .collect::<Vec<_>>();

    if paired_values.len() < 2 {
        return Err(CorrelationAnalysisError::NotEnoughPairedRows {
            column: feature_column.to_string(),
            required: 2,
            actual: paired_values.len(),
        });
    }

    let coefficient = calculate_pearson(target_column, feature_column, &paired_values)?;

    Ok(CorrelationItem {
        feature_column: feature_column.to_string(),
        coefficient,
        paired_row_count: paired_values.len(),
    })
}

// 2026-03-25: 这里手工实现 Pearson 相关系数，原因是第一版要继续遵守“传统计算在 Rust Tool 层完成”；目的是先给相关性分析提供零外部运行时依赖的稳定实现。
fn calculate_pearson(
    target_column: &str,
    feature_column: &str,
    paired_values: &[(f64, f64)],
) -> Result<f64, CorrelationAnalysisError> {
    let sample_count = paired_values.len() as f64;
    let mut sum_x = 0.0_f64;
    let mut sum_y = 0.0_f64;
    let mut sum_xx = 0.0_f64;
    let mut sum_yy = 0.0_f64;
    let mut sum_xy = 0.0_f64;

    for (x, y) in paired_values {
        sum_x += x;
        sum_y += y;
        sum_xx += x * x;
        sum_yy += y * y;
        sum_xy += x * y;
    }

    let numerator = sample_count * sum_xy - sum_x * sum_y;
    let target_scale = sample_count * sum_xx - sum_x.powi(2);
    let feature_scale = sample_count * sum_yy - sum_y.powi(2);

    if target_scale.abs() <= 1e-12 {
        return Err(CorrelationAnalysisError::ConstantColumn {
            column: target_column.to_string(),
        });
    }
    if feature_scale.abs() <= 1e-12 {
        return Err(CorrelationAnalysisError::ConstantColumn {
            column: feature_column.to_string(),
        });
    }

    let denominator = (target_scale * feature_scale).sqrt();
    let coefficient = (numerator / denominator).clamp(-1.0, 1.0);
    Ok(coefficient)
}

// 2026-03-25: 这里集中生成人话摘要，原因是 Skill 需要先拿到“最强正相关/负相关”的自然语言说明；目的是减少上层重复解释结构化结果的负担。
fn build_human_summary(
    target_column: &str,
    row_count: usize,
    top_positive: &[CorrelationItem],
    top_negative: &[CorrelationItem],
) -> CorrelationHumanSummary {
    let strongest_positive = top_positive.first();
    let strongest_negative = top_negative.first();
    let mut key_points = Vec::new();

    if let Some(item) = strongest_positive {
        key_points.push(format!(
            "`{}` 与 `{}` 呈最强正相关，系数约为 {:.4}",
            item.feature_column, target_column, item.coefficient
        ));
    }
    if let Some(item) = strongest_negative {
        key_points.push(format!(
            "`{}` 与 `{}` 呈最强负相关，系数约为 {:.4}",
            item.feature_column, target_column, item.coefficient
        ));
    }

    let overall = if strongest_positive.is_none() && strongest_negative.is_none() {
        format!("当前没有足够的数值列与 `{target_column}` 计算相关关系。")
    } else {
        format!(
            "系统已基于 {row_count} 行数据完成 `{target_column}` 的相关性分析，可先查看最强正相关和负相关字段。"
        )
    };

    CorrelationHumanSummary {
        overall,
        key_points,
        recommended_next_step:
            "建议先结合业务语义确认强相关字段，再决定是否继续做回归、聚类或异常诊断。".to_string(),
    }
}
