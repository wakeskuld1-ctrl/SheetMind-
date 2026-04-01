use serde::Serialize;
use thiserror::Error;

use crate::frame::loader::LoadedTable;
use crate::ops::model_output::{
    ModelHumanSummary, ModelingMetric, ModelingQualitySummary, build_data_summary,
};
use crate::ops::model_prep::{MissingStrategy, ModelPrepError, prepare_regression_dataset};

// 2026-03-21: 这里定义特征系数结构，目的是让上层按“列名 -> 系数”稳定消费回归结果，而不是依赖位置猜测。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RegressionCoefficient {
    pub feature: String,
    pub value: f64,
}

// 2026-03-21: 这里定义线性回归统一输出，目的是让它和逻辑回归、聚类共享同一层建模总览协议。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LinearRegressionResult {
    pub model_kind: String,
    pub problem_type: String,
    pub features: Vec<String>,
    pub target: String,
    #[serde(default)]
    pub coefficients: Vec<RegressionCoefficient>,
    pub intercept: f64,
    pub r2: f64,
    pub row_count_used: usize,
    pub dropped_rows: usize,
    pub data_summary: crate::ops::model_output::ModelingDataSummary,
    pub quality_summary: ModelingQualitySummary,
    #[serde(default)]
    pub assumptions: Vec<String>,
    pub human_summary: ModelHumanSummary,
}

// 2026-03-21: 这里定义线性回归错误，目的是把公共样本准备错误和求解错误分层暴露给上层。
#[derive(Debug, Error)]
pub enum LinearRegressionError {
    #[error(transparent)]
    Prepare(#[from] ModelPrepError),
    #[error(
        "删除缺失值后，只剩 {actual} 行有效数据，样本太少，暂时不能建模；至少需要 {required} 行"
    )]
    NotEnoughRows { required: usize, actual: usize },
    #[error("特征列之间过于重复，当前模型算不稳定，请减少重复含义的列后重试")]
    SingularMatrix,
}

// 2026-03-21: 这里提供线性回归主入口，目的是让线性回归复用统一样本准备并输出统一建模总览字段。
pub fn linear_regression(
    loaded: &LoadedTable,
    features: &[&str],
    target: &str,
    intercept: bool,
    missing_strategy: MissingStrategy,
) -> Result<LinearRegressionResult, LinearRegressionError> {
    let prepared =
        prepare_regression_dataset(loaded, features, target, intercept, missing_strategy)?;
    let min_required_rows = (features.len() + usize::from(intercept)).max(3);
    if prepared.targets.len() < min_required_rows {
        return Err(LinearRegressionError::NotEnoughRows {
            required: min_required_rows,
            actual: prepared.targets.len(),
        });
    }

    let beta = solve_ols(&prepared.feature_matrix, &prepared.targets)?;
    let predictions = predict_all(&prepared.feature_matrix, &beta);
    let r2 = calculate_r2(&prepared.targets, &predictions);
    let model_intercept = if intercept { beta[0] } else { 0.0 };
    let coefficient_offset = usize::from(intercept);
    let coefficients = prepared
        .features
        .iter()
        .enumerate()
        .map(|(index, feature)| RegressionCoefficient {
            feature: feature.clone(),
            value: beta[index + coefficient_offset],
        })
        .collect::<Vec<_>>();
    let assumptions = build_assumptions(intercept, missing_strategy);
    let human_summary = build_human_summary(
        target,
        &coefficients,
        r2,
        prepared.targets.len(),
        prepared.dropped_rows,
        model_intercept,
    );

    Ok(LinearRegressionResult {
        model_kind: "linear_regression".to_string(),
        problem_type: "regression".to_string(),
        features: prepared.features,
        target: prepared.target,
        coefficients,
        intercept: model_intercept,
        r2,
        row_count_used: prepared.targets.len(),
        dropped_rows: prepared.dropped_rows,
        data_summary: build_data_summary(
            features.len(),
            prepared.targets.len(),
            prepared.dropped_rows,
            missing_strategy,
            Some(intercept),
        ),
        quality_summary: ModelingQualitySummary {
            primary_metric: ModelingMetric {
                name: "r2".to_string(),
                value: r2,
            },
            secondary_metrics: vec![ModelingMetric {
                name: "intercept".to_string(),
                value: model_intercept,
            }],
        },
        assumptions,
        human_summary,
    })
}

// 2026-03-21: 这里用正规方程求解 OLS，目的是继续以依赖最少的方式提供稳定的传统统计计算。
fn solve_ols(
    design_matrix: &[Vec<f64>],
    targets: &[f64],
) -> Result<Vec<f64>, LinearRegressionError> {
    let parameter_count = design_matrix.first().map(|row| row.len()).unwrap_or(0);
    let mut xtx = vec![vec![0.0; parameter_count]; parameter_count];
    let mut xty = vec![0.0; parameter_count];

    for (row, target) in design_matrix.iter().zip(targets.iter().copied()) {
        for left in 0..parameter_count {
            xty[left] += row[left] * target;
            for right in 0..parameter_count {
                xtx[left][right] += row[left] * row[right];
            }
        }
    }

    solve_linear_system(xtx, xty)
}

// 2026-03-21: 这里使用带主元选择的高斯消元，目的是在不引入额外数值库的前提下尽量稳定处理小规模矩阵。
fn solve_linear_system(
    mut matrix: Vec<Vec<f64>>,
    mut vector: Vec<f64>,
) -> Result<Vec<f64>, LinearRegressionError> {
    let size = vector.len();
    let epsilon = 1e-10_f64;

    for pivot_index in 0..size {
        let mut best_row = pivot_index;
        let mut best_value = matrix[pivot_index][pivot_index].abs();

        for candidate_row in (pivot_index + 1)..size {
            let candidate_value = matrix[candidate_row][pivot_index].abs();
            if candidate_value > best_value {
                best_row = candidate_row;
                best_value = candidate_value;
            }
        }

        if best_value <= epsilon {
            return Err(LinearRegressionError::SingularMatrix);
        }

        if best_row != pivot_index {
            matrix.swap(pivot_index, best_row);
            vector.swap(pivot_index, best_row);
        }

        let pivot_value = matrix[pivot_index][pivot_index];
        for column_index in pivot_index..size {
            matrix[pivot_index][column_index] /= pivot_value;
        }
        vector[pivot_index] /= pivot_value;

        for row_index in 0..size {
            if row_index == pivot_index {
                continue;
            }

            let factor = matrix[row_index][pivot_index];
            if factor.abs() <= epsilon {
                continue;
            }

            for column_index in pivot_index..size {
                matrix[row_index][column_index] -= factor * matrix[pivot_index][column_index];
            }
            vector[row_index] -= factor * vector[pivot_index];
        }
    }

    Ok(vector)
}

// 2026-03-21: 这里批量计算预测值，目的是统一服务 R2 计算和后续可能扩展的残差诊断。
fn predict_all(design_matrix: &[Vec<f64>], beta: &[f64]) -> Vec<f64> {
    design_matrix
        .iter()
        .map(|row| {
            row.iter()
                .zip(beta.iter())
                .map(|(value, weight)| value * weight)
                .sum()
        })
        .collect()
}

// 2026-03-21: 这里统一计算 R2，目的是给用户一个最常用且最容易理解的拟合质量指标。
fn calculate_r2(actual: &[f64], predicted: &[f64]) -> f64 {
    let mean = actual.iter().sum::<f64>() / actual.len() as f64;
    let total_sum_of_squares = actual
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>();
    let residual_sum_of_squares = actual
        .iter()
        .zip(predicted.iter())
        .map(|(actual, predicted)| (actual - predicted).powi(2))
        .sum::<f64>();

    if total_sum_of_squares.abs() <= 1e-10 {
        if residual_sum_of_squares.abs() <= 1e-10 {
            1.0
        } else {
            0.0
        }
    } else {
        1.0 - residual_sum_of_squares / total_sum_of_squares
    }
}

// 2026-03-21: 这里生成结构化前提说明，目的是让 Skill 和用户都知道当前结果建立在什么条件下。
fn build_assumptions(intercept: bool, missing_strategy: MissingStrategy) -> Vec<String> {
    let mut assumptions = vec![
        "当前模型只支持数值型特征列和数值型目标列".to_string(),
        "当前结果反映线性相关关系，不等于因果关系".to_string(),
    ];

    if intercept {
        assumptions.push("当前模型已包含截距项".to_string());
    } else {
        assumptions.push("当前模型未包含截距项".to_string());
    }

    match missing_strategy {
        MissingStrategy::DropRows => {
            assumptions.push("遇到缺失值时，当前模型会直接跳过整行样本".to_string());
        }
    }

    assumptions
}

// 2026-03-21: 这里集中生成人话摘要，目的是把统计结果翻译成低 IT 用户更容易理解的描述。
fn build_human_summary(
    target: &str,
    coefficients: &[RegressionCoefficient],
    r2: f64,
    row_count_used: usize,
    dropped_rows: usize,
    intercept: f64,
) -> ModelHumanSummary {
    let dominant_feature = coefficients
        .iter()
        .max_by(|left, right| left.value.abs().total_cmp(&right.value.abs()))
        .map(|coefficient| coefficient.feature.clone())
        .unwrap_or_else(|| "未识别".to_string());
    let mut key_points = vec![
        format!("本次建模使用了 {row_count_used} 行有效样本"),
        format!("对 `{target}` 影响最明显的特征列是 `{dominant_feature}`"),
        format!("当前模型的 R2 为 {:.4}", r2),
    ];

    if dropped_rows > 0 {
        key_points.push(format!("有 {dropped_rows} 行因为缺失值被跳过"));
    }

    ModelHumanSummary {
        overall: format!(
            "系统已基于 {row_count_used} 行有效样本完成线性回归，当前模型对 `{target}` 的拟合度为 {:.4}。",
            r2
        ),
        key_points,
        recommended_next_step: if dropped_rows > 0 {
            "建议先检查被跳过的缺失样本，再决定是否补值后重新建模。".to_string()
        } else if intercept.abs() > 1e-10 {
            "建议继续结合业务含义检查主要系数方向是否符合预期。".to_string()
        } else {
            "建议继续检查主要系数方向和量级是否符合业务预期。".to_string()
        },
    }
}
