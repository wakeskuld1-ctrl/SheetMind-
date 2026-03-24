use serde::Serialize;
use thiserror::Error;

use crate::frame::loader::LoadedTable;

// 2026-03-25: 这里定义分箱结果，原因是分布分析第一版除了摘要还需要给出稳定的 histogram 结构；目的是让上层 Skill 能继续解释“主要集中在哪个区间”。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DistributionBin {
    pub index: usize,
    pub label: String,
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub count: usize,
    pub ratio: f64,
}

// 2026-03-25: 这里定义分布统计摘要，原因是建模前观察要同时看到中心位置、离散程度和偏态方向；目的是把第一版最关键的分布信息结构化输出。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DistributionSummary {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub median: f64,
    pub q1: f64,
    pub q3: f64,
    pub stddev: f64,
    pub skewness: f64,
}

// 2026-03-25: 这里定义人类可读摘要，原因是业务用户更容易理解“分布偏不偏、数据主要堆在哪里”；目的是让问答层不用二次拼装说明文本。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DistributionHumanSummary {
    pub overall: String,
    #[serde(default)]
    pub key_points: Vec<String>,
    pub recommended_next_step: String,
}

// 2026-03-25: 这里定义分布分析统一输出，原因是 dispatcher 和 Skill 都需要稳定读取列名、分箱结果和摘要；目的是给后续趋势/异常/建模前观察建立一致协议。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DistributionAnalysisResult {
    pub column: String,
    pub row_count: usize,
    pub non_null_count: usize,
    pub null_count: usize,
    pub bin_count: usize,
    pub distribution_summary: DistributionSummary,
    #[serde(default)]
    pub bins: Vec<DistributionBin>,
    pub human_summary: DistributionHumanSummary,
}

// 2026-03-25: 这里定义分布分析错误，原因是第一版要把缺列、非数值列和非法 bins 直接说明白；目的是让普通用户知道该先清洗什么问题。
#[derive(Debug, Error)]
pub enum DistributionAnalysisError {
    #[error("distribution_analysis 缺少 column 参数")]
    MissingColumnArg,
    #[error("distribution_analysis 的 bins 必须大于 0")]
    InvalidBinCount,
    #[error("列 `{column}` 不存在")]
    MissingColumn { column: String },
    #[error("列 `{column}` 不是可做分布分析的数值列，请先做类型转换")]
    NonNumericColumn { column: String },
    #[error("列 `{column}` 没有足够的有效数值，暂时无法做分布分析")]
    NoValidValues { column: String },
}

// 2026-03-25: 这里提供分布分析主入口，原因是统计诊断型能力第三步要先回答“数据是集中、分散还是偏态”；目的是让建模前观察先有稳定的传统统计底座。
pub fn distribution_analysis(
    loaded: &LoadedTable,
    column: &str,
    bins: usize,
) -> Result<DistributionAnalysisResult, DistributionAnalysisError> {
    if column.trim().is_empty() {
        return Err(DistributionAnalysisError::MissingColumnArg);
    }
    if bins == 0 {
        return Err(DistributionAnalysisError::InvalidBinCount);
    }

    let values = extract_numeric_values(loaded, column)?;
    let valid_values = values.iter().flatten().copied().collect::<Vec<_>>();
    if valid_values.is_empty() {
        return Err(DistributionAnalysisError::NoValidValues {
            column: column.to_string(),
        });
    }

    let mut sorted = valid_values.clone();
    sorted.sort_by(|left, right| left.total_cmp(right));

    let row_count = loaded.dataframe.height();
    let non_null_count = valid_values.len();
    let null_count = row_count.saturating_sub(non_null_count);
    let mean = valid_values.iter().sum::<f64>() / non_null_count as f64;
    let variance = valid_values
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / non_null_count as f64;
    let stddev = variance.sqrt();
    let summary = DistributionSummary {
        min: sorted[0],
        max: *sorted.last().unwrap_or(&sorted[0]),
        mean,
        median: quantile(&sorted, 0.5),
        q1: quantile(&sorted, 0.25),
        q3: quantile(&sorted, 0.75),
        stddev,
        skewness: calculate_skewness(&valid_values, mean, stddev),
    };
    let histogram = build_histogram(&valid_values, bins, &summary);
    let human_summary = build_human_summary(column, &summary, &histogram, non_null_count);

    Ok(DistributionAnalysisResult {
        column: column.to_string(),
        row_count,
        non_null_count,
        null_count,
        bin_count: bins,
        distribution_summary: summary,
        bins: histogram,
        human_summary,
    })
}

// 2026-03-25: 这里把目标列统一抽成可空数值向量，原因是分布分析只接受数值列但仍需保留空值统计；目的是同时支持 non_null_count 和 null_count 输出。
fn extract_numeric_values(
    loaded: &LoadedTable,
    column: &str,
) -> Result<Vec<Option<f64>>, DistributionAnalysisError> {
    let series = loaded
        .dataframe
        .column(column)
        .map_err(|_| DistributionAnalysisError::MissingColumn {
            column: column.to_string(),
        })?
        .as_materialized_series();
    let casted = series
        .cast(&polars::prelude::DataType::Float64)
        .map_err(|_| DistributionAnalysisError::NonNumericColumn {
            column: column.to_string(),
        })?;
    let values = casted
        .f64()
        .map_err(|_| DistributionAnalysisError::NonNumericColumn {
            column: column.to_string(),
        })?
        .into_iter()
        .collect::<Vec<_>>();

    Ok(values)
}

// 2026-03-25: 这里统一计算分位数，原因是 median/q1/q3 都依赖相同插值逻辑；目的是让分布摘要输出保持稳定一致。
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

// 2026-03-25: 这里计算偏度，原因是第一版需要回答“分布是否明显右偏/左偏”；目的是让 Skill 能自然翻译成业务上可理解的观察结论。
fn calculate_skewness(values: &[f64], mean: f64, stddev: f64) -> f64 {
    if stddev.abs() <= 1e-12 {
        return 0.0;
    }

    values
        .iter()
        .map(|value| ((value - mean) / stddev).powi(3))
        .sum::<f64>()
        / values.len() as f64
}

// 2026-03-25: 这里构造等宽分箱，原因是第一版 histogram 先以最稳妥的传统等宽区间为主；目的是让分布分析输出稳定、易解释、易导出。
fn build_histogram(values: &[f64], bin_count: usize, summary: &DistributionSummary) -> Vec<DistributionBin> {
    if values.is_empty() {
        return Vec::new();
    }

    let min = summary.min;
    let max = summary.max;
    let width = if (max - min).abs() <= 1e-12 {
        1.0
    } else {
        (max - min) / bin_count as f64
    };
    let mut counts = vec![0usize; bin_count];

    for value in values {
        let index = if (max - min).abs() <= 1e-12 {
            0
        } else if (*value - max).abs() <= 1e-12 {
            bin_count - 1
        } else {
            (((*value - min) / width).floor() as usize).min(bin_count - 1)
        };
        counts[index] += 1;
    }

    counts
        .into_iter()
        .enumerate()
        .map(|(index, count)| {
            let lower_bound = min + width * index as f64;
            let upper_bound = if index + 1 == bin_count {
                max
            } else {
                min + width * (index + 1) as f64
            };

            DistributionBin {
                index,
                label: format!("[{lower_bound:.2}, {upper_bound:.2}]"),
                lower_bound,
                upper_bound,
                count,
                ratio: count as f64 / values.len() as f64,
            }
        })
        .collect()
}

// 2026-03-25: 这里生成分布观察摘要，原因是统计结果最终要让非技术用户也能看懂；目的是把偏态、集中区间和下一步动作整理成可直接展示的文本。
fn build_human_summary(
    column: &str,
    summary: &DistributionSummary,
    bins: &[DistributionBin],
    non_null_count: usize,
) -> DistributionHumanSummary {
    let dominant_bin = bins
        .iter()
        .max_by(|left, right| left.count.cmp(&right.count).then_with(|| left.index.cmp(&right.index)));
    let distribution_tone = if summary.skewness > 1.0 {
        "明显右偏"
    } else if summary.skewness < -1.0 {
        "明显左偏"
    } else {
        "整体较平稳"
    };

    let mut key_points = vec![format!(
        "`{column}` 共纳入 {} 个有效数值，中位数为 {:.4}，四分位区间约为 [{:.4}, {:.4}]",
        non_null_count, summary.median, summary.q1, summary.q3
    )];
    if let Some(bin) = dominant_bin {
        key_points.push(format!(
            "`{column}` 主要集中在区间 {}，占比约为 {:.2}%",
            bin.label,
            bin.ratio * 100.0
        ));
    }

    DistributionHumanSummary {
        overall: format!(
            "`{column}` 的分布观察已完成：当前数据{distribution_tone}，可据此判断是否需要进一步做异常值处理或建模前变换。"
        ),
        key_points,
        recommended_next_step:
            "建议先结合异常值检测结果一起判断是否存在极端值拉长尾部，再决定是否做对数变换、分箱或继续建模。"
                .to_string(),
    }
}
