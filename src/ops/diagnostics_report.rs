use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::frame::loader::LoadedTable;
use crate::ops::correlation_analysis::{CorrelationAnalysisResult, correlation_analysis};
use crate::ops::distribution_analysis::{DistributionAnalysisResult, distribution_analysis};
use crate::ops::outlier_detection::{
    OutlierDetectionMethod, OutlierDetectionResult, outlier_detection,
};
use crate::ops::trend_analysis::{TrendAnalysisResult, trend_analysis};

// 2026-03-28 23:52 CST: 这里定义组合诊断里的相关性 section 请求，原因是高层 Tool 需要把目标列和候选特征列一起收口成强类型配置；
// 目的是让 dispatcher 不再手工散落解析字段，同时给后续 Rust 侧高层交付保留稳定合同。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DiagnosticsCorrelationRequest {
    pub target_column: String,
    #[serde(default)]
    pub feature_columns: Vec<String>,
}

// 2026-03-28 23:52 CST: 这里定义组合诊断里的异常值 section 请求，原因是高层 Tool 需要沿用已有 method/columns 配置；
// 目的是让 outlier_detection 能被无缝复用，而不是重新发明一套平行参数结构。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DiagnosticsOutlierRequest {
    #[serde(default)]
    pub columns: Vec<String>,
    #[serde(default = "default_outlier_method")]
    pub method: OutlierDetectionMethod,
}

// 2026-03-28 23:52 CST: 这里定义组合诊断里的分布 section 请求，原因是分布分析需要列名与 bins 这两个最小入口；
// 目的是保持组合 Tool 第一版只暴露必要参数，避免过早做成复杂编排器。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DiagnosticsDistributionRequest {
    pub column: String,
    #[serde(default = "default_distribution_bins")]
    pub bins: usize,
}

// 2026-03-28 23:52 CST: 这里定义组合诊断里的趋势 section 请求，原因是趋势分析天然依赖 time/value 两列；
// 目的是让高层 Tool 可以在同一份 result_ref 上按需补时间观察，而不是要求调用方分开起第二次请求。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DiagnosticsTrendRequest {
    pub time_column: String,
    pub value_column: String,
}

// 2026-03-28 23:52 CST: 这里定义组合诊断总请求，原因是高层 Tool 第一版要允许调用方只开部分 section；
// 目的是让“同一份表，按需要做哪些观察”变成可选配置，而不是强制四件套全开。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
pub struct DiagnosticsReportRequest {
    #[serde(default)]
    pub report_name: Option<String>,
    #[serde(default)]
    pub correlation: Option<DiagnosticsCorrelationRequest>,
    #[serde(default)]
    pub outlier: Option<DiagnosticsOutlierRequest>,
    #[serde(default)]
    pub distribution: Option<DiagnosticsDistributionRequest>,
    #[serde(default)]
    pub trend: Option<DiagnosticsTrendRequest>,
}

// 2026-03-28 23:52 CST: 这里定义每个 section 的外部状态摘要，原因是组合 Tool 需要在总览层告诉调用方哪些 section 成功、哪些降级；
// 目的是让 CLI/Skill 不用再读完整大对象也能先拿到一眼可扫的执行结果。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DiagnosticsSectionStatus {
    pub key: String,
    pub title: String,
    pub status: String,
    pub summary: String,
}

// 2026-03-28 23:52 CST: 这里定义组合诊断统一输出，原因是高层 Tool 的价值就在于把散的诊断结论收口成一个稳定 JSON 包；
// 目的是让后续 workbook、Skill 和 CLI 都消费同一个合同，而不是各自再拼装摘要。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DiagnosticsReportResult {
    pub report_name: String,
    pub report_status: String,
    pub row_count: usize,
    pub section_count: usize,
    pub available_section_count: usize,
    pub overall_summary: String,
    #[serde(default)]
    pub key_findings: Vec<String>,
    #[serde(default)]
    pub recommended_actions: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub sections: Vec<DiagnosticsSectionStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_section: Option<CorrelationAnalysisResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outlier_section: Option<OutlierDetectionResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distribution_section: Option<DistributionAnalysisResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trend_section: Option<TrendAnalysisResult>,
}

// 2026-03-28 23:52 CST: 这里定义高层 Tool 的最小错误类型，原因是第一版只需要明确拦住“一个 section 都没配置”的空请求；
// 目的是把真正的局部 section 失败留给降级逻辑处理，而不是让整包直接报错。
#[derive(Debug, Error)]
pub enum DiagnosticsReportError {
    #[error("diagnostics_report 至少需要配置一个诊断 section")]
    EmptySections,
}

// 2026-03-28 23:52 CST: 这里提供组合诊断主入口，原因是我们要把四个独立统计观察能力收口成一个高层 Tool；
// 目的是在不重写底层算法的前提下，形成“统一摘要 + 分 section 结果 + 降级 warning”的稳定交付合同。
pub fn diagnostics_report(
    loaded: &LoadedTable,
    request: &DiagnosticsReportRequest,
) -> Result<DiagnosticsReportResult, DiagnosticsReportError> {
    let mut section_count = 0usize;
    let mut available_section_count = 0usize;
    let mut key_findings = Vec::new();
    let mut recommended_actions = Vec::new();
    let mut warnings = Vec::new();
    let mut sections = Vec::new();

    let mut correlation_section = None;
    let mut outlier_section = None;
    let mut distribution_section = None;
    let mut trend_section = None;

    // 2026-03-28 23:52 CST: 这里按 section 独立执行相关性分析，原因是组合 Tool 需要把局部失败收敛成 warning；
    // 目的是保证个别字段缺失时，其它可用 section 仍能继续交付。
    if let Some(correlation_request) = request.correlation.as_ref() {
        section_count += 1;
        let feature_columns = correlation_request
            .feature_columns
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        match correlation_analysis(loaded, &correlation_request.target_column, &feature_columns) {
            Ok(result) => {
                available_section_count += 1;
                sections.push(DiagnosticsSectionStatus {
                    key: "correlation".to_string(),
                    title: "相关性分析".to_string(),
                    status: "ok".to_string(),
                    summary: result.human_summary.overall.clone(),
                });
                collect_correlation_findings(&result, &mut key_findings);
                push_unique_action(
                    &mut recommended_actions,
                    result.human_summary.recommended_next_step.clone(),
                );
                correlation_section = Some(result);
            }
            Err(error) => {
                let message = format!("correlation_analysis unavailable: {error}");
                warnings.push(message.clone());
                sections.push(DiagnosticsSectionStatus {
                    key: "correlation".to_string(),
                    title: "相关性分析".to_string(),
                    status: "unavailable".to_string(),
                    summary: message,
                });
            }
        }
    }

    // 2026-03-28 23:52 CST: 这里按 section 独立执行异常值分析，原因是 outlier_detection 自带标记表和摘要；
    // 目的是在组合 Tool 第一版先复用摘要层，而不额外引入新的结果持久化分支。
    if let Some(outlier_request) = request.outlier.as_ref() {
        section_count += 1;
        let columns = outlier_request
            .columns
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        match outlier_detection(loaded, &columns, outlier_request.method) {
            Ok((_, result)) => {
                available_section_count += 1;
                sections.push(DiagnosticsSectionStatus {
                    key: "outlier".to_string(),
                    title: "异常值诊断".to_string(),
                    status: "ok".to_string(),
                    summary: result.human_summary.overall.clone(),
                });
                collect_outlier_findings(&result, &mut key_findings);
                push_unique_action(
                    &mut recommended_actions,
                    result.human_summary.recommended_next_step.clone(),
                );
                outlier_section = Some(result);
            }
            Err(error) => {
                let message = format!("outlier_detection unavailable: {error}");
                warnings.push(message.clone());
                sections.push(DiagnosticsSectionStatus {
                    key: "outlier".to_string(),
                    title: "异常值诊断".to_string(),
                    status: "unavailable".to_string(),
                    summary: message,
                });
            }
        }
    }

    // 2026-03-28 23:52 CST: 这里按 section 独立执行分布分析，原因是分布观察常常和异常值一起构成建模前诊断；
    // 目的是让偏态、集中区间和分箱结果能够进入统一总览，而不是分散在单独调用里。
    if let Some(distribution_request) = request.distribution.as_ref() {
        section_count += 1;
        match distribution_analysis(
            loaded,
            &distribution_request.column,
            distribution_request.bins,
        ) {
            Ok(result) => {
                available_section_count += 1;
                sections.push(DiagnosticsSectionStatus {
                    key: "distribution".to_string(),
                    title: "分布分析".to_string(),
                    status: "ok".to_string(),
                    summary: result.human_summary.overall.clone(),
                });
                collect_distribution_findings(&result, &mut key_findings);
                push_unique_action(
                    &mut recommended_actions,
                    result.human_summary.recommended_next_step.clone(),
                );
                distribution_section = Some(result);
            }
            Err(error) => {
                let message = format!("distribution_analysis unavailable: {error}");
                warnings.push(message.clone());
                sections.push(DiagnosticsSectionStatus {
                    key: "distribution".to_string(),
                    title: "分布分析".to_string(),
                    status: "unavailable".to_string(),
                    summary: message,
                });
            }
        }
    }

    // 2026-03-28 23:52 CST: 这里按 section 独立执行趋势分析，原因是趋势观察天然容易受时间列缺失影响；
    // 目的是把时间维失败控制在局部 warning，而不是拖垮整个组合诊断结果。
    if let Some(trend_request) = request.trend.as_ref() {
        section_count += 1;
        match trend_analysis(
            loaded,
            &trend_request.time_column,
            &trend_request.value_column,
        ) {
            Ok(result) => {
                available_section_count += 1;
                sections.push(DiagnosticsSectionStatus {
                    key: "trend".to_string(),
                    title: "趋势分析".to_string(),
                    status: "ok".to_string(),
                    summary: result.human_summary.overall.clone(),
                });
                collect_trend_findings(&result, &mut key_findings);
                push_unique_action(
                    &mut recommended_actions,
                    result.human_summary.recommended_next_step.clone(),
                );
                trend_section = Some(result);
            }
            Err(error) => {
                let message = format!("trend_analysis unavailable: {error}");
                warnings.push(message.clone());
                sections.push(DiagnosticsSectionStatus {
                    key: "trend".to_string(),
                    title: "趋势分析".to_string(),
                    status: "unavailable".to_string(),
                    summary: message,
                });
            }
        }
    }

    if section_count == 0 {
        return Err(DiagnosticsReportError::EmptySections);
    }

    let report_status = if available_section_count == 0 {
        "unavailable"
    } else if warnings.is_empty() {
        "ok"
    } else {
        "degraded"
    };
    let report_name = request
        .report_name
        .clone()
        .unwrap_or_else(|| "统计诊断组合报告".to_string());

    // 2026-03-28 23:52 CST: 这里统一生成总览摘要，原因是高层 Tool 的核心不是 section 罗列，而是要先交付一段可扫读的整体判断；
    // 目的是让 CLI 和后续 workbook 先拿到“本次诊断是否完整、发现集中在哪”这一层总论。
    let overall_summary = build_overall_summary(
        report_status,
        section_count,
        available_section_count,
        warnings.len(),
    );

    if key_findings.is_empty() {
        key_findings
            .push("当前没有成功产出可用诊断结论，请优先检查字段配置和数值列质量。".to_string());
    }
    if recommended_actions.is_empty() {
        recommended_actions
            .push("建议先补齐可用字段后重新运行组合诊断，再决定是否继续做建模或交付。".to_string());
    }

    Ok(DiagnosticsReportResult {
        report_name,
        report_status: report_status.to_string(),
        row_count: loaded.dataframe.height(),
        section_count,
        available_section_count,
        overall_summary,
        key_findings,
        recommended_actions,
        warnings,
        sections,
        correlation_section,
        outlier_section,
        distribution_section,
        trend_section,
    })
}

// 2026-03-28 23:52 CST: 这里集中生成总览描述，原因是不同 section 的执行结果需要被压缩成一句高层判断；
// 目的是把“完整 / 降级 / 不可用”三种状态稳定翻译成统一中文口径。
fn build_overall_summary(
    report_status: &str,
    section_count: usize,
    available_section_count: usize,
    warning_count: usize,
) -> String {
    match report_status {
        "ok" => format!(
            "本次组合诊断已完成，共配置 {section_count} 个诊断 section，全部成功产出结果，可直接进入业务解读或交付整理。"
        ),
        "degraded" => format!(
            "本次组合诊断以降级模式完成，共配置 {section_count} 个 section，其中 {available_section_count} 个成功、{warning_count} 个降级，请结合 warning 补查缺失字段。"
        ),
        _ => format!(
            "本次组合诊断未产出可用 section，共尝试 {section_count} 个配置，请先修正字段或数据质量问题后重试。"
        ),
    }
}

// 2026-03-28 23:52 CST: 这里抽取相关性层的关键发现，原因是总报告要把最强正负相关直接抬到摘要层；
// 目的是让调用方不用再自己从 correlations 数组里二次找“最值得先看”的字段。
fn collect_correlation_findings(
    result: &CorrelationAnalysisResult,
    key_findings: &mut Vec<String>,
) {
    if let Some(item) = result.top_positive.first() {
        key_findings.push(format!(
            "相关性：`{}` 与 `{}` 呈最强正相关，系数约为 {:.4}。",
            item.feature_column, result.target_column, item.coefficient
        ));
    }
    if let Some(item) = result.top_negative.first() {
        key_findings.push(format!(
            "相关性：`{}` 与 `{}` 呈最强负相关，系数约为 {:.4}。",
            item.feature_column, result.target_column, item.coefficient
        ));
    }
}

// 2026-03-28 23:52 CST: 这里抽取异常值层的关键发现，原因是异常数量和占比往往就是业务方最先问的观察点；
// 目的是把 outlier_summaries 里最关键的结论直接提到总览层。
fn collect_outlier_findings(result: &OutlierDetectionResult, key_findings: &mut Vec<String>) {
    if let Some(summary) = result
        .outlier_summaries
        .iter()
        .max_by_key(|item| item.outlier_count)
    {
        if summary.outlier_count > 0 {
            key_findings.push(format!(
                "异常值：`{}` 检测到 {} 个异常点，占比约 {:.2}%。",
                summary.column,
                summary.outlier_count,
                summary.outlier_ratio * 100.0
            ));
        } else {
            key_findings.push("异常值：当前检测列未发现明显异常点。".to_string());
        }
    }
}

// 2026-03-28 23:52 CST: 这里抽取分布层的关键发现，原因是偏态与主集中区间是分布分析里最适合上收的两类信息；
// 目的是让总报告先告诉用户“偏不偏、主要堆在哪”，再决定是否深入 bins 明细。
fn collect_distribution_findings(
    result: &DistributionAnalysisResult,
    key_findings: &mut Vec<String>,
) {
    let tone = if result.distribution_summary.skewness > 1.0 {
        "明显右偏"
    } else if result.distribution_summary.skewness < -1.0 {
        "明显左偏"
    } else {
        "整体较平稳"
    };
    key_findings.push(format!(
        "分布：`{}` 当前 {}，中位数约为 {:.4}。",
        result.column, tone, result.distribution_summary.median
    ));
}

// 2026-03-28 23:52 CST: 这里抽取趋势层的关键发现，原因是趋势方向和变化幅度天然属于高层摘要信息；
// 目的是让用户一眼先看到“上涨/下滑/持平”的主判断，再决定是否展开 points 明细。
fn collect_trend_findings(result: &TrendAnalysisResult, key_findings: &mut Vec<String>) {
    let direction_label = match result.direction.as_str() {
        "upward" => "整体上升",
        "downward" => "整体下降",
        _ => "整体持平",
    };
    key_findings.push(format!(
        "趋势：`{}` 基于 `{}` {}，绝对变化 {:.4}，变化率约 {:.2}%。",
        result.value_column,
        result.time_column,
        direction_label,
        result.absolute_change,
        result.change_rate * 100.0
    ));
}

// 2026-03-28 23:52 CST: 这里统一去重追加动作建议，原因是四个 section 的 recommended_next_step 可能出现重复口径；
// 目的是让组合报告第一版保持可读，不把相同建议连续输出多次。
fn push_unique_action(actions: &mut Vec<String>, candidate: String) {
    if !actions.iter().any(|item| item == &candidate) {
        actions.push(candidate);
    }
}

// 2026-03-28 23:52 CST: 这里统一给异常值 section 提供默认方法，原因是第一版组合 Tool 应该与单 Tool 保持一致默认口径；
// 目的是让调用方在未显式指定时仍然沿用现有 IQR 逻辑。
fn default_outlier_method() -> OutlierDetectionMethod {
    OutlierDetectionMethod::Iqr
}

// 2026-03-28 23:52 CST: 这里统一给分布 section 提供默认 bins，原因是组合 Tool 不应该强迫调用方每次都填写直方图分箱数；
// 目的是保持第一版 JSON 输入简洁，同时与已有分布分析习惯兼容。
fn default_distribution_bins() -> usize {
    10
}
