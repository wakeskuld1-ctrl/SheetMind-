use serde::Serialize;
use thiserror::Error;

use crate::frame::loader::LoadedTable;
use crate::ops::linear_regression::RegressionCoefficient;
use crate::ops::model_output::{
    ModelHumanSummary, ModelingMetric, ModelingQualitySummary, build_data_summary,
};
use crate::ops::model_prep::{
    MissingStrategy, ModelPrepError, PreparedBinaryClassificationDataset,
    prepare_binary_classification_dataset,
};

// 2026-03-21: 这里定义类别分布摘要，目的是让非技术用户先看到正负类样本是否失衡。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ClassBalanceSummary {
    pub positive_count: usize,
    pub negative_count: usize,
    pub positive_rate: f64,
}

// 2026-03-21: 这里定义逻辑回归统一输出，目的是让分类模型和回归/聚类共享同一层建模总览协议。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LogisticRegressionResult {
    pub model_kind: String,
    pub problem_type: String,
    pub features: Vec<String>,
    pub target: String,
    pub positive_label: String,
    #[serde(default)]
    pub coefficients: Vec<RegressionCoefficient>,
    pub intercept: f64,
    pub row_count_used: usize,
    pub dropped_rows: usize,
    pub class_balance: ClassBalanceSummary,
    pub training_accuracy: f64,
    pub data_summary: crate::ops::model_output::ModelingDataSummary,
    pub quality_summary: ModelingQualitySummary,
    #[serde(default)]
    pub assumptions: Vec<String>,
    pub human_summary: ModelHumanSummary,
}

// 2026-03-21: 这里定义逻辑回归错误，目的是把公共样本准备错误与训练阶段错误分层暴露给上层。
#[derive(Debug, Error)]
pub enum LogisticRegressionError {
    #[error(transparent)]
    Prepare(#[from] ModelPrepError),
    #[error(
        "删除缺失值后，只剩 {actual} 行有效数据，样本太少，暂时不能建模；至少需要 {required} 行"
    )]
    NotEnoughRows { required: usize, actual: usize },
}

// 2026-03-21: 这里提供逻辑回归主入口，目的是让二分类建模能力完整下沉到 Tool 计算层。
pub fn logistic_regression(
    loaded: &LoadedTable,
    features: &[&str],
    target: &str,
    intercept: bool,
    missing_strategy: MissingStrategy,
    positive_label: Option<&str>,
) -> Result<LogisticRegressionResult, LogisticRegressionError> {
    let prepared = prepare_binary_classification_dataset(
        loaded,
        features,
        target,
        intercept,
        missing_strategy,
        positive_label,
    )?;
    let min_required_rows = (features.len() + usize::from(intercept) + 1).max(4);
    if prepared.targets.len() < min_required_rows {
        return Err(LogisticRegressionError::NotEnoughRows {
            required: min_required_rows,
            actual: prepared.targets.len(),
        });
    }

    let beta = train_logistic_model(&prepared.feature_matrix, &prepared.targets);
    let probabilities = predict_probabilities(&prepared.feature_matrix, &beta);
    let training_accuracy = calculate_accuracy(&prepared.targets, &probabilities);
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
    let class_balance = build_class_balance(&prepared);
    let assumptions = build_assumptions(intercept, missing_strategy);
    let human_summary = build_human_summary(
        &prepared.target,
        &prepared.positive_label,
        &coefficients,
        training_accuracy,
        prepared.row_count_used,
        prepared.dropped_rows,
        &class_balance,
    );

    Ok(LogisticRegressionResult {
        model_kind: "logistic_regression".to_string(),
        problem_type: "classification".to_string(),
        features: prepared.features,
        target: prepared.target,
        positive_label: prepared.positive_label,
        coefficients,
        intercept: model_intercept,
        row_count_used: prepared.row_count_used,
        dropped_rows: prepared.dropped_rows,
        class_balance: class_balance.clone(),
        training_accuracy,
        data_summary: build_data_summary(
            features.len(),
            prepared.row_count_used,
            prepared.dropped_rows,
            missing_strategy,
            Some(intercept),
        ),
        quality_summary: ModelingQualitySummary {
            primary_metric: ModelingMetric {
                name: "training_accuracy".to_string(),
                value: training_accuracy,
            },
            secondary_metrics: vec![ModelingMetric {
                name: "positive_rate".to_string(),
                value: class_balance.positive_rate,
            }],
        },
        assumptions,
        human_summary,
    })
}

// 2026-03-21: 这里使用批量梯度下降训练逻辑回归，目的是先用依赖可控、容易验证的方式完成 V1 二分类闭环。
fn train_logistic_model(feature_matrix: &[Vec<f64>], targets: &[f64]) -> Vec<f64> {
    let parameter_count = feature_matrix.first().map(|row| row.len()).unwrap_or(0);
    let mut beta = vec![0.0_f64; parameter_count];
    let average_norm = feature_matrix
        .iter()
        .map(|row| row.iter().map(|value| value * value).sum::<f64>())
        .sum::<f64>()
        / feature_matrix.len() as f64;
    let learning_rate = 1.0 / average_norm.max(1.0);
    let max_iterations = 20_000_usize;
    let tolerance = 1e-8_f64;

    for _ in 0..max_iterations {
        let mut gradient = vec![0.0_f64; parameter_count];

        for (row, target) in feature_matrix.iter().zip(targets.iter().copied()) {
            let prediction = sigmoid(dot(row, &beta));
            let error = prediction - target;
            for (index, value) in row.iter().enumerate() {
                gradient[index] += error * value;
            }
        }

        let mut max_change = 0.0_f64;
        for index in 0..parameter_count {
            let step = learning_rate * gradient[index] / feature_matrix.len() as f64;
            beta[index] -= step;
            max_change = max_change.max(step.abs());
        }

        if max_change <= tolerance {
            break;
        }
    }

    beta
}

// 2026-03-21: 这里批量计算逻辑回归概率，目的是统一服务训练准确率和后续可能扩展的概率输出。
fn predict_probabilities(feature_matrix: &[Vec<f64>], beta: &[f64]) -> Vec<f64> {
    feature_matrix
        .iter()
        .map(|row| sigmoid(dot(row, beta)))
        .collect()
}

// 2026-03-21: 这里统一计算训练准确率，目的是先提供最容易理解的分类质量指标。
fn calculate_accuracy(targets: &[f64], probabilities: &[f64]) -> f64 {
    let correct = targets
        .iter()
        .zip(probabilities.iter())
        .filter(|(target, probability)| {
            let predicted = if **probability >= 0.5 { 1.0 } else { 0.0 };
            (predicted - **target).abs() <= 1e-9
        })
        .count();
    correct as f64 / targets.len() as f64
}

// 2026-03-21: 这里统一计算类别分布，目的是把是否失衡的关键信息直接暴露给上层。
fn build_class_balance(prepared: &PreparedBinaryClassificationDataset) -> ClassBalanceSummary {
    let positive_count = prepared
        .targets
        .iter()
        .filter(|value| **value >= 0.5)
        .count();
    let negative_count = prepared.targets.len().saturating_sub(positive_count);
    ClassBalanceSummary {
        positive_count,
        negative_count,
        positive_rate: positive_count as f64 / prepared.targets.len() as f64,
    }
}

// 2026-03-21: 这里集中生成人话摘要，目的是让低 IT 用户直接看懂当前分类模型的基本情况。
fn build_human_summary(
    target: &str,
    positive_label: &str,
    coefficients: &[RegressionCoefficient],
    training_accuracy: f64,
    row_count_used: usize,
    dropped_rows: usize,
    class_balance: &ClassBalanceSummary,
) -> ModelHumanSummary {
    let dominant_feature = coefficients
        .iter()
        .max_by(|left, right| left.value.abs().total_cmp(&right.value.abs()))
        .map(|coefficient| coefficient.feature.clone())
        .unwrap_or_else(|| "未识别".to_string());
    let mut key_points = vec![
        format!("本次建模使用了 {row_count_used} 行有效样本"),
        format!("当前正类按 `{positive_label}` 处理"),
        format!("训练集准确率为 {:.4}", training_accuracy),
        format!("影响 `{target}` 是否属于正类最明显的特征列是 `{dominant_feature}`"),
    ];

    if dropped_rows > 0 {
        key_points.push(format!("有 {dropped_rows} 行因为缺失值被跳过"));
    }

    if class_balance.positive_rate >= 0.8 || class_balance.positive_rate <= 0.2 {
        key_points.push("当前类别分布明显不均衡，解读结果时要注意样本偏斜。".to_string());
    }

    ModelHumanSummary {
        overall: format!(
            "系统已基于 {row_count_used} 行有效样本完成二分类逻辑回归，当前正类按 `{positive_label}` 处理，训练集准确率为 {:.4}。",
            training_accuracy
        ),
        key_points,
        recommended_next_step: if dropped_rows > 0 {
            "建议先检查被跳过的缺失样本，再决定是否补值后重新建模。".to_string()
        } else {
            "建议继续结合业务含义检查主要系数方向是否符合预期。".to_string()
        },
    }
}

// 2026-03-21: 这里统一维护逻辑回归前提说明，目的是让 Skill 和用户明确知道 V1 的边界。
fn build_assumptions(intercept: bool, missing_strategy: MissingStrategy) -> Vec<String> {
    let mut assumptions = vec![
        "当前模型只支持数值型特征列".to_string(),
        "当前目标列只支持二分类".to_string(),
        "当前结果反映分类相关性，不等于因果关系".to_string(),
        "本轮明确不做 AUC、混淆矩阵全展开、阈值调优、正则化和多分类 softmax".to_string(),
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

// 2026-03-21: 这里统一计算向量点积，目的是避免训练和预测重复手写同一段基础数值逻辑。
fn dot(left: &[f64], right: &[f64]) -> f64 {
    left.iter().zip(right.iter()).map(|(x, y)| x * y).sum()
}

// 2026-03-21: 这里统一实现数值稳定的 sigmoid，目的是避免大数输入时出现溢出。
fn sigmoid(value: f64) -> f64 {
    if value >= 0.0 {
        let exp = (-value).exp();
        1.0 / (1.0 + exp)
    } else {
        let exp = value.exp();
        exp / (1.0 + exp)
    }
}
