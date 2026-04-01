use serde::Serialize;
use thiserror::Error;

use crate::frame::loader::LoadedTable;
use crate::ops::model_output::{
    ModelHumanSummary, ModelingMetric, ModelingQualitySummary, build_data_summary,
};
use crate::ops::model_prep::{MissingStrategy, ModelPrepError, prepare_clustering_dataset};

// 2026-03-21: 这里定义聚类成员归属，目的是让上层 Skill 能把分群结果稳定映射回输入样本行。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ClusterAssignment {
    pub source_row_index: usize,
    pub cluster_id: usize,
}

// 2026-03-21: 这里定义每个分组的样本规模，目的是让用户先看到各群体大小是否均衡。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ClusterSize {
    pub cluster_id: usize,
    pub row_count: usize,
    pub share: f64,
}

// 2026-03-21: 这里定义单个特征在中心点上的取值，目的是让中心点结果可以按“列名 -> 中心值”被稳定消费。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ClusterCenterValue {
    pub feature: String,
    pub value: f64,
}

// 2026-03-21: 这里定义单个簇的中心点，目的是让终端和 Skill 都能直接解释每一组的典型画像。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ClusterCenter {
    pub cluster_id: usize,
    #[serde(default)]
    pub values: Vec<ClusterCenterValue>,
}

// 2026-03-21: 这里定义 KMeans 的统一输出结构，目的是把聚类能力接入与回归/分类一致的分析建模层协议。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ClusterKmeansResult {
    pub model_kind: String,
    pub problem_type: String,
    pub features: Vec<String>,
    pub cluster_count: usize,
    #[serde(default)]
    pub assignments: Vec<ClusterAssignment>,
    #[serde(default)]
    pub cluster_sizes: Vec<ClusterSize>,
    #[serde(default)]
    pub cluster_centers: Vec<ClusterCenter>,
    pub row_count_used: usize,
    pub dropped_rows: usize,
    pub data_summary: crate::ops::model_output::ModelingDataSummary,
    pub quality_summary: ModelingQualitySummary,
    #[serde(default)]
    pub assumptions: Vec<String>,
    pub human_summary: ModelHumanSummary,
}

// 2026-03-21: 这里定义聚类错误，目的是把输入问题与训练阶段问题都转成低 IT 用户可读的中文提示。
#[derive(Debug, Error)]
pub enum ClusterKmeansError {
    #[error(transparent)]
    Prepare(#[from] ModelPrepError),
    #[error("聚类至少需要 2 个分组，当前收到 {0} 组")]
    InvalidClusterCount(usize),
    #[error("当前分组数是 {clusters}，但有效样本只有 {rows} 行，不能这样分组")]
    ClusterCountExceedsRows { clusters: usize, rows: usize },
    #[error(
        "当前有效样本里只有 {distinct_points} 个不同的数据点，无法稳定分成 {requested_clusters} 组"
    )]
    NotEnoughDistinctPoints {
        requested_clusters: usize,
        distinct_points: usize,
    },
    #[error("当前分组数过大或样本过于集中，至少有 1 个分组没有分到数据，请减少分组数后重试")]
    EmptyCluster,
}

// 2026-03-21: 这里提供 KMeans 主入口，目的是补齐分析建模层 V1 最后一块传统聚类能力。
pub fn cluster_kmeans(
    loaded: &LoadedTable,
    features: &[&str],
    cluster_count: usize,
    max_iterations: usize,
    missing_strategy: MissingStrategy,
) -> Result<ClusterKmeansResult, ClusterKmeansError> {
    if cluster_count < 2 {
        return Err(ClusterKmeansError::InvalidClusterCount(cluster_count));
    }

    let prepared = prepare_clustering_dataset(loaded, features, missing_strategy)?;
    if cluster_count > prepared.row_count_used {
        return Err(ClusterKmeansError::ClusterCountExceedsRows {
            clusters: cluster_count,
            rows: prepared.row_count_used,
        });
    }

    let centers = choose_initial_centers(&prepared.feature_matrix, cluster_count)?;
    let (cluster_ids, centers, inertia) =
        run_kmeans(&prepared.feature_matrix, centers, max_iterations);
    let cluster_sizes = build_cluster_sizes(&cluster_ids, cluster_count, prepared.row_count_used);
    if cluster_sizes.iter().any(|item| item.row_count == 0) {
        return Err(ClusterKmeansError::EmptyCluster);
    }

    let assignments = prepared
        .source_row_indices
        .iter()
        .zip(cluster_ids.iter().copied())
        .map(|(source_row_index, cluster_id)| ClusterAssignment {
            source_row_index: *source_row_index,
            cluster_id,
        })
        .collect::<Vec<_>>();
    let cluster_centers = build_cluster_centers(&prepared.features, &centers);
    let quality_summary = build_quality_summary(inertia, &cluster_sizes);
    let assumptions = build_assumptions(missing_strategy);
    let human_summary = build_human_summary(
        cluster_count,
        prepared.row_count_used,
        prepared.dropped_rows,
        &prepared.features,
        &cluster_sizes,
        &cluster_centers,
    );

    Ok(ClusterKmeansResult {
        model_kind: "cluster_kmeans".to_string(),
        problem_type: "clustering".to_string(),
        features: prepared.features.clone(),
        cluster_count,
        assignments,
        cluster_sizes,
        cluster_centers,
        row_count_used: prepared.row_count_used,
        dropped_rows: prepared.dropped_rows,
        data_summary: build_data_summary(
            prepared.features.len(),
            prepared.row_count_used,
            prepared.dropped_rows,
            missing_strategy,
            None,
        ),
        quality_summary,
        assumptions,
        human_summary,
    })
}

// 2026-03-21: 这里使用确定性 farthest-point 初始化，目的是避免引入随机数依赖并让测试结果稳定可复现。
fn choose_initial_centers(
    feature_matrix: &[Vec<f64>],
    cluster_count: usize,
) -> Result<Vec<Vec<f64>>, ClusterKmeansError> {
    let mut centers = Vec::with_capacity(cluster_count);
    let mut chosen_indices = Vec::<usize>::with_capacity(cluster_count);

    centers.push(feature_matrix[0].clone());
    chosen_indices.push(0);

    while centers.len() < cluster_count {
        let mut best_index = None;
        let mut best_distance = -1.0_f64;

        for (row_index, row) in feature_matrix.iter().enumerate() {
            if chosen_indices.contains(&row_index) {
                continue;
            }

            let nearest_distance = centers
                .iter()
                .map(|center| squared_distance(row, center))
                .fold(f64::INFINITY, f64::min);

            if nearest_distance > best_distance + 1e-12 {
                best_distance = nearest_distance;
                best_index = Some(row_index);
            }
        }

        let Some(best_index) = best_index else {
            return Err(ClusterKmeansError::NotEnoughDistinctPoints {
                requested_clusters: cluster_count,
                distinct_points: centers.len(),
            });
        };

        if best_distance <= 1e-12 {
            return Err(ClusterKmeansError::NotEnoughDistinctPoints {
                requested_clusters: cluster_count,
                distinct_points: centers.len(),
            });
        }

        centers.push(feature_matrix[best_index].clone());
        chosen_indices.push(best_index);
    }

    Ok(centers)
}

// 2026-03-21: 这里执行标准 KMeans 迭代，目的是在不额外引库的前提下完成稳定可解释的传统聚类计算。
fn run_kmeans(
    feature_matrix: &[Vec<f64>],
    mut centers: Vec<Vec<f64>>,
    max_iterations: usize,
) -> (Vec<usize>, Vec<Vec<f64>>, f64) {
    let cluster_count = centers.len();
    let feature_count = centers.first().map(|row| row.len()).unwrap_or(0);
    let mut cluster_ids = vec![usize::MAX; feature_matrix.len()];
    let iterations = max_iterations.max(1);

    for _ in 0..iterations {
        let mut changed = false;
        for (row_index, row) in feature_matrix.iter().enumerate() {
            let nearest_cluster = nearest_center(row, &centers);
            if cluster_ids[row_index] != nearest_cluster {
                cluster_ids[row_index] = nearest_cluster;
                changed = true;
            }
        }

        let mut sums = vec![vec![0.0_f64; feature_count]; cluster_count];
        let mut counts = vec![0_usize; cluster_count];
        for (row, cluster_id) in feature_matrix.iter().zip(cluster_ids.iter().copied()) {
            counts[cluster_id] += 1;
            for (feature_index, value) in row.iter().enumerate() {
                sums[cluster_id][feature_index] += value;
            }
        }

        for cluster_id in 0..cluster_count {
            if counts[cluster_id] == 0 {
                continue;
            }
            for feature_index in 0..feature_count {
                centers[cluster_id][feature_index] =
                    sums[cluster_id][feature_index] / counts[cluster_id] as f64;
            }
        }

        if !changed {
            break;
        }
    }

    let inertia = feature_matrix
        .iter()
        .zip(cluster_ids.iter().copied())
        .map(|(row, cluster_id)| squared_distance(row, &centers[cluster_id]))
        .sum::<f64>();

    (cluster_ids, centers, inertia)
}

// 2026-03-21: 这里计算最近中心点，目的是让聚类归属判断逻辑集中在一个稳定函数里。
fn nearest_center(row: &[f64], centers: &[Vec<f64>]) -> usize {
    let mut best_cluster = 0_usize;
    let mut best_distance = squared_distance(row, &centers[0]);

    for (cluster_id, center) in centers.iter().enumerate().skip(1) {
        let distance = squared_distance(row, center);
        if distance < best_distance {
            best_cluster = cluster_id;
            best_distance = distance;
        }
    }

    best_cluster
}

// 2026-03-21: 这里统一计算欧式平方距离，目的是避免初始化、归属和质量计算重复手写同一逻辑。
fn squared_distance(left: &[f64], right: &[f64]) -> f64 {
    left.iter()
        .zip(right.iter())
        .map(|(left_value, right_value)| (left_value - right_value).powi(2))
        .sum()
}

// 2026-03-21: 这里生成簇规模摘要，目的是让上层先看各群体大小是否极端失衡。
fn build_cluster_sizes(
    cluster_ids: &[usize],
    cluster_count: usize,
    row_count_used: usize,
) -> Vec<ClusterSize> {
    let mut counts = vec![0_usize; cluster_count];
    for cluster_id in cluster_ids.iter().copied() {
        counts[cluster_id] += 1;
    }

    counts
        .into_iter()
        .enumerate()
        .map(|(cluster_id, row_count)| ClusterSize {
            cluster_id,
            row_count,
            share: row_count as f64 / row_count_used as f64,
        })
        .collect()
}

// 2026-03-21: 这里把中心点向量翻译成带列名的结构，目的是让结果不依赖位置猜测即可解释。
fn build_cluster_centers(features: &[String], centers: &[Vec<f64>]) -> Vec<ClusterCenter> {
    centers
        .iter()
        .enumerate()
        .map(|(cluster_id, center)| ClusterCenter {
            cluster_id,
            values: features
                .iter()
                .zip(center.iter().copied())
                .map(|(feature, value)| ClusterCenterValue {
                    feature: feature.clone(),
                    value,
                })
                .collect(),
        })
        .collect()
}

// 2026-03-21: 这里统一构造聚类质量摘要，目的是让上层总能先拿到主指标，再按需看补充指标。
fn build_quality_summary(inertia: f64, cluster_sizes: &[ClusterSize]) -> ModelingQualitySummary {
    let largest_cluster_share = cluster_sizes
        .iter()
        .map(|item| item.share)
        .fold(0.0_f64, f64::max);

    ModelingQualitySummary {
        primary_metric: ModelingMetric {
            name: "inertia".to_string(),
            value: inertia,
        },
        secondary_metrics: vec![ModelingMetric {
            name: "largest_cluster_share".to_string(),
            value: largest_cluster_share,
        }],
    }
}

// 2026-03-21: 这里统一维护聚类前提说明，目的是明确告诉用户当前 V1 的算法边界与解释边界。
fn build_assumptions(missing_strategy: MissingStrategy) -> Vec<String> {
    let mut assumptions = vec![
        "当前聚类只支持数值特征列".to_string(),
        "当前聚类使用用户给定的分组数，不自动选 K".to_string(),
        "当前结果反映样本相似分组，不等于业务因果结论".to_string(),
        "本轮暂不做轮廓系数、自动选 K、复杂可视化解释".to_string(),
    ];

    match missing_strategy {
        MissingStrategy::DropRows => {
            assumptions.push("遇到缺失值时，当前聚类会直接跳过整行样本".to_string());
        }
    }

    assumptions
}

// 2026-03-21: 这里集中生成人话摘要，目的是让低 IT 用户无需理解中心点 JSON 也能看懂聚类结果。
fn build_human_summary(
    cluster_count: usize,
    row_count_used: usize,
    dropped_rows: usize,
    features: &[String],
    cluster_sizes: &[ClusterSize],
    cluster_centers: &[ClusterCenter],
) -> ModelHumanSummary {
    let largest_cluster = cluster_sizes
        .iter()
        .max_by(|left, right| left.row_count.cmp(&right.row_count))
        .cloned();
    let most_separating_feature = pick_most_separating_feature(cluster_centers)
        .and_then(|feature_index| features.get(feature_index).cloned())
        .unwrap_or_else(|| "未识别".to_string());

    let mut key_points = vec![
        format!("本次聚类使用了 {row_count_used} 行有效样本，分成 {cluster_count} 组"),
        format!("当前最能区分各组的特征列是 `{most_separating_feature}`"),
    ];

    if let Some(largest_cluster) = largest_cluster {
        key_points.push(format!(
            "样本最多的是第 {} 组，占比 {:.2}%",
            largest_cluster.cluster_id,
            largest_cluster.share * 100.0
        ));
    }

    if dropped_rows > 0 {
        key_points.push(format!("有 {dropped_rows} 行因为缺失值被跳过"));
    }

    ModelHumanSummary {
        overall: format!(
            "系统已基于 {row_count_used} 行有效样本完成 {cluster_count} 组聚类，可以开始观察群体分层差异。"
        ),
        key_points,
        recommended_next_step: if dropped_rows > 0 {
            "建议先检查被跳过的缺失样本，再结合各组中心点确认分群是否符合业务直觉。".to_string()
        } else {
            "建议先阅读各组中心点与样本占比，再决定是否围绕某一组继续做业务分析或规则运营。"
                .to_string()
        },
    }
}

// 2026-03-21: 这里识别最能拉开各组差异的特征列，目的是给用户一个更直白的“分群主轴”提示。
fn pick_most_separating_feature(cluster_centers: &[ClusterCenter]) -> Option<usize> {
    let feature_count = cluster_centers.first()?.values.len();
    let mut best_feature_index = None;
    let mut best_span = -1.0_f64;

    for feature_index in 0..feature_count {
        let mut min_value = f64::INFINITY;
        let mut max_value = f64::NEG_INFINITY;
        for center in cluster_centers {
            let value = center.values.get(feature_index)?.value;
            min_value = min_value.min(value);
            max_value = max_value.max(value);
        }

        let span = max_value - min_value;
        if span > best_span {
            best_span = span;
            best_feature_index = Some(feature_index);
        }
    }

    best_feature_index
}
