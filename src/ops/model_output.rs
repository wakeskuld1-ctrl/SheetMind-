use serde::Serialize;

use crate::ops::model_prep::MissingStrategy;

// 2026-03-21: 这里统一定义分析建模层的人类摘要，目的是让回归、分类、聚类都能复用同一套双层输出骨架。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ModelHumanSummary {
    pub overall: String,
    #[serde(default)]
    pub key_points: Vec<String>,
    pub recommended_next_step: String,
}

// 2026-03-21: 这里统一定义建模数据摘要，目的是让 Skill 与高层 Tool 先读公共字段，再按模型类型读取专有内容。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ModelingDataSummary {
    pub feature_count: usize,
    pub row_count_used: usize,
    pub dropped_rows: usize,
    pub missing_strategy: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intercept_enabled: Option<bool>,
}

// 2026-03-21: 这里统一定义模型质量指标，目的是让不同模型都能用“指标名 + 指标值”的稳定协议输出核心质量分数。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ModelingMetric {
    pub name: String,
    pub value: f64,
}

// 2026-03-21: 这里统一定义模型质量摘要，目的是让上层先读主指标，再按需读取补充指标。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ModelingQualitySummary {
    pub primary_metric: ModelingMetric,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub secondary_metrics: Vec<ModelingMetric>,
}

// 2026-03-21: 这里集中生成公共数据摘要，目的是避免三类模型分别维护相同字段导致协议漂移。
pub fn build_data_summary(
    feature_count: usize,
    row_count_used: usize,
    dropped_rows: usize,
    missing_strategy: MissingStrategy,
    intercept_enabled: Option<bool>,
) -> ModelingDataSummary {
    ModelingDataSummary {
        feature_count,
        row_count_used,
        dropped_rows,
        missing_strategy: missing_strategy_label(missing_strategy).to_string(),
        intercept_enabled,
    }
}

// 2026-03-21: 这里统一输出缺失策略标签，目的是让 JSON 响应里不暴露内部枚举细节。
pub fn missing_strategy_label(strategy: MissingStrategy) -> &'static str {
    match strategy {
        MissingStrategy::DropRows => "drop_rows",
    }
}
