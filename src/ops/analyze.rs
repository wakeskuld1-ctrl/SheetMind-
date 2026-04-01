use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

use crate::frame::loader::LoadedTable;
use crate::ops::semantic::{
    classify_time_period, looks_like_amount_column_name, looks_like_date_column_name,
    looks_like_identifier_column_name, looks_like_time_column_name, parse_date_value,
    parse_time_value,
};
use crate::ops::summary::{ColumnSummary, summarize_table};
use polars::prelude::{AnyValue, DataType};

// 2026-03-21: 这里定义表级健康状态，目的是给桥接 Tool 一个稳定且可扩展的总体风险出口。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TableHealth {
    pub level: String,
    pub score: f64,
}

// 2026-03-21: 这里定义结构化 finding，目的是让后续 Skill 与建模 Tool 可以稳定编排诊断结果。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AnalyzeFinding {
    pub code: String,
    pub severity: String,
    pub scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<String>,
    pub message: String,
    pub suggestion: String,
}

// 2026-03-21: 这里单独定义业务观察，目的是把“质量诊断”和“轻量统计提示”分层输出，避免上层反解析摘要文案。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BusinessObservation {
    #[serde(rename = "type")]
    pub observation_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<String>,
    pub message: String,
}

// 2026-03-21: 这里定义面向用户的中文摘要，目的是让非 IT 用户无需理解 JSON 细节也能直接读结论。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HumanSummary {
    pub overall: String,
    #[serde(default)]
    pub major_issues: Vec<String>,
    #[serde(default)]
    pub quick_insights: Vec<String>,
    pub recommended_next_step: String,
}

// 2026-03-21: 这里定义 analyze_table 的统一返回结构，目的是固定桥接 Tool 的双层输出协议。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AnalyzeResult {
    pub row_count: usize,
    pub column_count: usize,
    pub table_health: TableHealth,
    // 2026-03-21: 这里强制保留空数组，目的是让外部协议从 V1 开始就稳定存在 structured_findings 字段。
    #[serde(default)]
    pub structured_findings: Vec<AnalyzeFinding>,
    // 2026-03-21: 这里单独暴露业务观察数组，目的是给分析建模层和问答层一个稳定的轻量统计入口。
    #[serde(default)]
    pub business_observations: Vec<BusinessObservation>,
    #[serde(default)]
    pub next_actions: Vec<String>,
    pub human_summary: HumanSummary,
}

// 2026-03-21: 这里提供 V1 的规则诊断主入口，目的是复用 summarize_table 的稳定画像，再叠加轻量质量与分布检查。
// 2026-03-21: 本次补充排序与展示压缩逻辑，目的是兼顾机器可编排的完整 finding 和用户可读的简洁摘要。
pub fn analyze_table(
    loaded: &LoadedTable,
    requested_columns: &[&str],
    top_k: usize,
) -> AnalyzeResult {
    let column_count = if requested_columns.is_empty() {
        loaded.dataframe.width()
    } else {
        requested_columns.len()
    };
    let effective_top_k = top_k.max(1);
    // 2026-03-21: 这里直接复用 summarize_table，目的是先从已有列画像能力派生第一批结构化诊断和业务观察。
    let summaries = summarize_table(loaded, requested_columns, effective_top_k).unwrap_or_default();
    let mut structured_findings = build_findings_from_summaries(&summaries);

    // 2026-03-21: 这里检测整行重复，目的是在建模和聚合前先暴露会放大样本权重的重复记录风险。
    let duplicate_row_count = count_duplicate_rows(loaded);
    if duplicate_row_count > 0 {
        structured_findings.push(AnalyzeFinding {
            code: "duplicate_rows".to_string(),
            severity: "high".to_string(),
            scope: "table".to_string(),
            column: None,
            message: format!("这张表存在 {} 行重复记录", duplicate_row_count),
            suggestion: "建议先去重，再继续聚合、关联或建模。".to_string(),
        });
    }

    // 2026-03-21: 这里检测候选键重复和空值，目的是在进入 join 或建模前先暴露标识列风险。
    for candidate_key in detect_candidate_keys(loaded) {
        let (blank_count, duplicate_count) = inspect_candidate_key(loaded, &candidate_key);

        if duplicate_count > 0 {
            structured_findings.push(AnalyzeFinding {
                code: "duplicate_candidate_key".to_string(),
                severity: "high".to_string(),
                scope: "column".to_string(),
                column: Some(candidate_key.clone()),
                message: format!("{} 列存在重复标识值", candidate_key),
                suggestion: "建议先确认这列是否真的应作为唯一标识，再决定关联或建模方式。"
                    .to_string(),
            });
        }

        if blank_count > 0 {
            structured_findings.push(AnalyzeFinding {
                code: "blank_candidate_key".to_string(),
                severity: "high".to_string(),
                scope: "column".to_string(),
                column: Some(candidate_key.clone()),
                message: format!("{} 列存在空白标识值", candidate_key),
                suggestion: "建议先补齐或排除空标识记录，避免影响后续关联和建模。".to_string(),
            });
        }
    }

    // 2026-03-21: 这里补充数值列分布扫描，目的是让桥接 Tool 能给出零值占比和异常值提醒。
    for summary in &summaries {
        if summary.summary_kind != "numeric" {
            continue;
        }

        let zero_ratio = calculate_zero_ratio(loaded, &summary.column);
        if zero_ratio >= 0.80 {
            structured_findings.push(AnalyzeFinding {
                code: "high_zero_ratio".to_string(),
                severity: "medium".to_string(),
                scope: "column".to_string(),
                column: Some(summary.column.clone()),
                message: format!("{} 列有大量 0 值", summary.column),
                suggestion: "建议先确认 0 值代表真实业务含义，还是缺失/默认占位。".to_string(),
            });
        }

        if has_outlier_by_iqr(loaded, &summary.column) {
            structured_findings.push(AnalyzeFinding {
                code: "outlier_suspected".to_string(),
                severity: "medium".to_string(),
                scope: "column".to_string(),
                column: Some(summary.column.clone()),
                message: format!("{} 列疑似存在异常值", summary.column),
                suggestion: "建议先核查极端值来源，再决定是否直接用于统计或建模。".to_string(),
            });
        }
    }

    // 2026-03-21: 这里统一排序完整 finding，目的是让外部编排和回归测试拿到稳定顺序。
    sort_findings(&mut structured_findings);
    // 2026-03-21: 这里压缩展示视图，目的是让 human_summary 对同一列只突出最重要的问题。
    let display_findings = compress_display_findings(&structured_findings);
    let business_observations = build_business_observations(loaded, &summaries, effective_top_k);
    let quick_insights = build_quick_insights(&business_observations);
    let next_actions = build_next_actions(&display_findings);

    let table_health = if structured_findings.iter().any(|finding| {
        matches!(
            finding.code.as_str(),
            "all_missing" | "duplicate_rows" | "duplicate_candidate_key" | "blank_candidate_key"
        )
    }) {
        TableHealth {
            level: "risky".to_string(),
            score: 0.35,
        }
    } else if structured_findings.is_empty() {
        TableHealth {
            level: "good".to_string(),
            score: 1.0,
        }
    } else {
        TableHealth {
            level: "warning".to_string(),
            score: 0.7,
        }
    };

    let major_issues = display_findings
        .iter()
        .take(3)
        .map(|finding| finding.message.clone())
        .collect::<Vec<_>>();

    AnalyzeResult {
        row_count: loaded.dataframe.height(),
        column_count,
        table_health: table_health.clone(),
        structured_findings,
        business_observations,
        next_actions: next_actions.clone(),
        human_summary: HumanSummary {
            overall: build_overall_summary(&table_health, &major_issues),
            major_issues,
            quick_insights,
            recommended_next_step: next_actions
                .first()
                .cloned()
                .unwrap_or_else(|| "建议继续检查缺失、重复和值分布，再进入后续分析。".to_string()),
        },
    }
}

// 2026-03-21: 这里集中从列画像生成初始 finding，目的是把列级规则与后续表级补充检查拆开，减少主流程噪音。
fn build_findings_from_summaries(summaries: &[ColumnSummary]) -> Vec<AnalyzeFinding> {
    let mut findings = Vec::new();

    for summary in summaries {
        if summary.missing_rate == Some(1.0) {
            findings.push(AnalyzeFinding {
                code: "all_missing".to_string(),
                severity: "high".to_string(),
                scope: "column".to_string(),
                column: Some(summary.column.clone()),
                message: format!("{} 列全部为空", summary.column),
                suggestion: "建议先确认这列是否还有保留价值，或在分析前先排除。".to_string(),
            });
            continue;
        }

        if summary.missing_rate.unwrap_or(0.0) >= 0.30 {
            findings.push(AnalyzeFinding {
                code: "high_missing_rate".to_string(),
                severity: "high".to_string(),
                scope: "column".to_string(),
                column: Some(summary.column.clone()),
                message: format!("{} 列缺失较多", summary.column),
                suggestion: "建议先补齐这列，或在分析前评估是否需要排除。".to_string(),
            });
        }

        if summary.summary_kind == "string"
            && summary.count > 0
            && summary
                .top_values
                .first()
                .map(|top_value| top_value.count as f64 / summary.count as f64 >= 0.80)
                .unwrap_or(false)
        {
            findings.push(AnalyzeFinding {
                code: "high_category_imbalance".to_string(),
                severity: "medium".to_string(),
                scope: "column".to_string(),
                column: Some(summary.column.clone()),
                message: format!("{} 列分布非常集中", summary.column),
                suggestion: "建议先确认这列是否存在类别失衡，再决定后续分析方式。".to_string(),
            });
        }

        if summary.distinct_count == Some(1) && summary.count > 0 {
            findings.push(AnalyzeFinding {
                code: "single_value_column".to_string(),
                severity: "medium".to_string(),
                scope: "column".to_string(),
                column: Some(summary.column.clone()),
                message: format!("{} 列几乎没有变化", summary.column),
                suggestion: "这列信息量较低，后续分析时可评估是否保留。".to_string(),
            });
        }
    }

    findings
}

// 2026-03-21: 这里构造总评文案，目的是把风险等级翻译成更直接的中文引导。
fn build_overall_summary(table_health: &TableHealth, major_issues: &[String]) -> String {
    match table_health.level.as_str() {
        "risky" => "这张表存在需要优先关注的质量风险，建议先清洗再继续分析。".to_string(),
        "warning" if !major_issues.is_empty() => {
            "这张表可以继续分析，但建议先处理几个明显的数据风险。".to_string()
        }
        _ => "这张表已经完成基础加载，可以继续做更深入的质量诊断。".to_string(),
    }
}

// 2026-03-21: 这里统计重复行数量，目的是用最直白的方式暴露样本重复风险。
fn count_duplicate_rows(loaded: &LoadedTable) -> usize {
    let columns = loaded.handle.columns();
    let mut seen = BTreeSet::<Vec<String>>::new();
    let mut duplicate_count = 0_usize;

    for row_index in 0..loaded.dataframe.height() {
        let mut row = Vec::with_capacity(columns.len());
        for column in columns {
            let value = loaded
                .dataframe
                .column(column)
                .ok()
                .and_then(|column| column.as_materialized_series().str_value(row_index).ok())
                .map(|value| value.into_owned())
                .unwrap_or_default();
            row.push(value);
        }

        if !seen.insert(row) {
            duplicate_count += 1;
        }
    }

    duplicate_count
}

// 2026-03-21: 这里识别候选键列，目的是先用更保守的命名规则覆盖显性 ID/编号 场景。
// 2026-03-21: 本次改成按 token/后缀判断，目的是避免把 notes 这类普通列因包含 no 而误判成键列。
fn detect_candidate_keys(loaded: &LoadedTable) -> Vec<String> {
    loaded
        .handle
        .columns()
        .iter()
        .filter(|column| looks_like_candidate_key(column))
        .cloned()
        .collect()
}

// 2026-03-21: 这里封装候选键命名判断，目的是让误判修复点集中而不是散落在调用处。
fn looks_like_candidate_key(column: &str) -> bool {
    looks_like_identifier_column_name(column)
}

// 2026-03-21: 这里检查候选键的空值和重复值，目的是把唯一标识风险沉淀成结构化 finding。
fn inspect_candidate_key(loaded: &LoadedTable, column: &str) -> (usize, usize) {
    let Some(series) = loaded
        .dataframe
        .column(column)
        .ok()
        .map(|column| column.as_materialized_series())
    else {
        return (0, 0);
    };

    let mut blank_count = 0_usize;
    let mut counts = BTreeMap::<String, usize>::new();

    for row_index in 0..series.len() {
        match series.get(row_index) {
            Ok(AnyValue::Null) => {
                blank_count += 1;
            }
            Ok(_) => {
                let value = series
                    .str_value(row_index)
                    .map(|value| value.into_owned())
                    .unwrap_or_default();
                let normalized = value.trim();

                if normalized.is_empty() {
                    blank_count += 1;
                } else {
                    *counts.entry(normalized.to_string()).or_default() += 1;
                }
            }
            Err(_) => {
                blank_count += 1;
            }
        }
    }

    let duplicate_count = counts.values().filter(|count| **count > 1).count();
    (blank_count, duplicate_count)
}

// 2026-03-21: 这里计算 0 值占比，目的是把业务里常见的“默认值过多”问题直接暴露成质量诊断。
fn calculate_zero_ratio(loaded: &LoadedTable, column: &str) -> f64 {
    let values = collect_numeric_values(loaded, column);
    if values.is_empty() {
        return 0.0;
    }

    let zero_count = values
        .iter()
        .filter(|value| value.abs() < f64::EPSILON)
        .count();
    zero_count as f64 / values.len() as f64
}

// 2026-03-21: 这里用 IQR 做轻量异常值检测，目的是先给 V1 一个可解释、可测试的异常提醒能力。
fn has_outlier_by_iqr(loaded: &LoadedTable, column: &str) -> bool {
    let mut values = collect_numeric_values(loaded, column);
    if values.len() < 4 {
        return false;
    }

    values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let q1 = percentile(&values, 0.25);
    let q3 = percentile(&values, 0.75);
    let iqr = q3 - q1;
    let lower = q1 - 1.5 * iqr;
    let upper = q3 + 1.5 * iqr;

    values.iter().any(|value| *value < lower || *value > upper)
}

// 2026-03-21: 这里收集数值列有效值，目的是复用到零值占比和异常值等多类轻量统计规则。
fn collect_numeric_values(loaded: &LoadedTable, column: &str) -> Vec<f64> {
    let Some(series) = loaded
        .dataframe
        .column(column)
        .ok()
        .map(|column| column.as_materialized_series())
    else {
        return Vec::new();
    };
    let casted = match series.dtype() {
        DataType::Float64 => series.clone(),
        _ => match series.cast(&DataType::Float64) {
            Ok(casted) => casted,
            Err(_) => return Vec::new(),
        },
    };

    match casted.f64() {
        Ok(values) => values.into_iter().flatten().collect(),
        Err(_) => Vec::new(),
    }
}

// 2026-03-21: 这里用线性插值近似分位点，目的是在不引入额外依赖的情况下完成稳定的 IQR 计算。
fn percentile(values: &[f64], p: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    if values.len() == 1 {
        return values[0];
    }

    let position = (values.len() - 1) as f64 * p;
    let lower_index = position.floor() as usize;
    let upper_index = position.ceil() as usize;
    if lower_index == upper_index {
        values[lower_index]
    } else {
        let weight = position - lower_index as f64;
        values[lower_index] * (1.0 - weight) + values[upper_index] * weight
    }
}

// 2026-03-21: 这里统一排序完整 finding，目的是让 JSON 输出顺序固定，方便 Skill 与测试稳定消费。
fn sort_findings(findings: &mut [AnalyzeFinding]) {
    findings.sort_by(|left, right| {
        finding_priority(left)
            .cmp(&finding_priority(right))
            .then_with(|| left.column.cmp(&right.column))
            .then_with(|| left.code.cmp(&right.code))
    });
}

// 2026-03-21: 这里定义 finding 优先级，目的是让“阻塞分析”的问题在展示和推荐里优先出现。
fn finding_priority(finding: &AnalyzeFinding) -> usize {
    match finding.code.as_str() {
        "all_missing" => 0,
        "duplicate_rows" => 1,
        "duplicate_candidate_key" => 2,
        "blank_candidate_key" => 3,
        "high_missing_rate" => 4,
        "outlier_suspected" => 5,
        "high_zero_ratio" => 6,
        "high_category_imbalance" => 7,
        "single_value_column" => 8,
        _ => 99,
    }
}

// 2026-03-21: 这里压缩展示层 finding，目的是同一列只突出最值得先处理的一条问题给终端用户。
fn compress_display_findings(findings: &[AnalyzeFinding]) -> Vec<AnalyzeFinding> {
    let mut display_findings = Vec::new();
    let mut seen_columns = BTreeSet::<String>::new();
    let mut seen_table_findings = BTreeSet::<String>::new();

    for finding in findings {
        if let Some(column) = &finding.column {
            if seen_columns.insert(column.clone()) {
                display_findings.push(finding.clone());
            }
        } else if seen_table_findings.insert(finding.code.clone()) {
            display_findings.push(finding.clone());
        }
    }

    display_findings
}

// 2026-03-21: 这里构造少量业务观察，目的是在质量诊断之外补一层可直接消费的轻量统计结论。
// 2026-03-21: 本次按“基础观察优先、扩展观察后补”的顺序生成，目的是兼容旧测试同时扩展新类型。
fn build_business_observations(
    loaded: &LoadedTable,
    summaries: &[ColumnSummary],
    max_items: usize,
) -> Vec<BusinessObservation> {
    let mut observations = Vec::new();
    let mut seen = BTreeSet::<(String, String)>::new();

    push_business_observations(
        &mut observations,
        &mut seen,
        build_top_category_observations(summaries),
        max_items,
    );
    push_business_observations(
        &mut observations,
        &mut seen,
        build_numeric_range_observations(summaries),
        max_items,
    );
    push_business_observations(
        &mut observations,
        &mut seen,
        build_dominant_dimension_observations(summaries),
        max_items,
    );
    push_business_observations(
        &mut observations,
        &mut seen,
        build_numeric_center_observations(loaded, summaries),
        max_items,
    );
    push_business_observations(
        &mut observations,
        &mut seen,
        build_date_observations(loaded, summaries),
        max_items,
    );
    push_business_observations(
        &mut observations,
        &mut seen,
        build_time_observations(loaded, summaries),
        max_items,
    );
    push_business_observations(
        &mut observations,
        &mut seen,
        build_amount_observations(loaded, summaries),
        max_items,
    );

    observations
}

// 2026-03-21: 这里分阶段追加观察，目的是让 top_k 成为统一门禁，而不是每类观察各算各的。
fn push_business_observations(
    target: &mut Vec<BusinessObservation>,
    seen: &mut BTreeSet<(String, String)>,
    candidates: Vec<BusinessObservation>,
    max_items: usize,
) {
    for observation in candidates {
        if target.len() >= max_items {
            break;
        }

        let key = (
            observation.observation_type.clone(),
            observation.column.clone().unwrap_or_default(),
        );
        if seen.insert(key) {
            target.push(observation);
        }
    }
}

// 2026-03-21: 这里生成人类最容易理解的主类别观察，目的是保留原有业务观察契约。
fn build_top_category_observations(summaries: &[ColumnSummary]) -> Vec<BusinessObservation> {
    summaries
        .iter()
        .filter(|summary| summary.summary_kind == "string" && summary.count > 0)
        .filter_map(|summary| {
            summary
                .top_values
                .first()
                .map(|top_value| BusinessObservation {
                    observation_type: "top_category".to_string(),
                    column: Some(summary.column.clone()),
                    message: format!(
                        "{} 列最常见的是 {}，约占 {:.0}%",
                        summary.column,
                        top_value.value,
                        top_value.count as f64 / summary.count as f64 * 100.0
                    ),
                })
        })
        .collect()
}

// 2026-03-21: 这里保留数值范围观察，目的是继续给分析建模层提供最基础的数值尺度提示。
fn build_numeric_range_observations(summaries: &[ColumnSummary]) -> Vec<BusinessObservation> {
    summaries
        .iter()
        .filter(|summary| summary.summary_kind == "numeric")
        .filter_map(|summary| match (summary.min_number, summary.max_number) {
            (Some(min_number), Some(max_number)) => Some(BusinessObservation {
                observation_type: "numeric_range".to_string(),
                column: Some(summary.column.clone()),
                message: format!(
                    "{} 列的取值范围大致在 {} 到 {}",
                    summary.column,
                    format_number(min_number),
                    format_number(max_number)
                ),
            }),
            _ => None,
        })
        .collect()
}

// 2026-03-21: 这里新增 dominant_dimension 观察，目的是把“主分布维度”从 finding 里拆出来作为轻量业务提示。
fn build_dominant_dimension_observations(summaries: &[ColumnSummary]) -> Vec<BusinessObservation> {
    summaries
        .iter()
        .filter(|summary| summary.summary_kind == "string" && summary.count > 0)
        .filter_map(|summary| {
            summary.top_values.first().and_then(|top_value| {
                let share = top_value.count as f64 / summary.count as f64;
                if share >= 0.80 {
                    Some(BusinessObservation {
                        observation_type: "dominant_dimension".to_string(),
                        column: Some(summary.column.clone()),
                        message: format!(
                            "{} 列当前主要由 {} 主导，约占 {:.0}%",
                            summary.column,
                            top_value.value,
                            share * 100.0
                        ),
                    })
                } else {
                    None
                }
            })
        })
        .collect()
}

// 2026-03-21: 这里新增 numeric_center 观察，目的是给桥接层补一层轻量中心统计，而不直接进入建模。
// 2026-03-21: 这里按数值跨度排序后再生成，目的是优先保留更有信息量的金额/指标列中心值。
fn build_numeric_center_observations(
    loaded: &LoadedTable,
    summaries: &[ColumnSummary],
) -> Vec<BusinessObservation> {
    let mut numeric_summaries = summaries
        .iter()
        .filter(|summary| summary.summary_kind == "numeric")
        .filter_map(
            |summary| match (summary.min_number, summary.max_number, summary.mean) {
                (Some(min_number), Some(max_number), Some(mean)) => {
                    Some((max_number - min_number, summary, mean))
                }
                _ => None,
            },
        )
        .collect::<Vec<_>>();

    numeric_summaries.sort_by(|left, right| {
        right
            .0
            .partial_cmp(&left.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    numeric_summaries
        .into_iter()
        .map(|(_, summary, mean)| {
            // 2026-03-21: 这里根据均值和中位数偏离度判断是否偏态，目的是在极端值场景下优先给出更稳健的中心统计。
            let chosen_center = calculate_numeric_center(loaded, summary);
            if chosen_center.label == "median_center" {
                BusinessObservation {
                    observation_type: "median_center".to_string(),
                    column: Some(summary.column.clone()),
                    message: format!(
                        "{} 列的中位数约为 {}",
                        summary.column,
                        format_number(chosen_center.value)
                    ),
                }
            } else {
                BusinessObservation {
                    observation_type: "numeric_center".to_string(),
                    column: Some(summary.column.clone()),
                    message: format!("{} 列的平均值约为 {}", summary.column, format_number(mean)),
                }
            }
        })
        .collect()
}

// 2026-03-21: 这里新增日期观察，目的是让用户先知道数据覆盖周期，以及记录是否主要集中在某个月份。
fn build_date_observations(
    loaded: &LoadedTable,
    summaries: &[ColumnSummary],
) -> Vec<BusinessObservation> {
    let mut observations = Vec::new();

    for summary in summaries {
        if summary.summary_kind != "string"
            || summary.count == 0
            || !is_likely_date_column(loaded, summary)
        {
            continue;
        }

        let mut dates = collect_string_values(loaded, &summary.column)
            .into_iter()
            .filter_map(|value| parse_date_value(&value))
            .collect::<Vec<_>>();
        if dates.is_empty() {
            continue;
        }

        dates.sort();
        if let (Some(min_date), Some(max_date)) = (dates.first(), dates.last()) {
            observations.push(BusinessObservation {
                observation_type: "date_range".to_string(),
                column: Some(summary.column.clone()),
                message: format!(
                    "{} 列覆盖范围大致从 {} 到 {}",
                    summary.column,
                    min_date.to_iso_string(),
                    max_date.to_iso_string()
                ),
            });
        }

        let mut month_counts = BTreeMap::<String, usize>::new();
        for date in &dates {
            *month_counts.entry(date.to_year_month()).or_default() += 1;
        }

        if let Some((month, count)) = month_counts
            .into_iter()
            .max_by(|left, right| left.1.cmp(&right.1).then_with(|| left.0.cmp(&right.0)))
        {
            let share = count as f64 / dates.len() as f64;
            if share >= 0.60 {
                observations.push(BusinessObservation {
                    observation_type: "date_concentration".to_string(),
                    column: Some(summary.column.clone()),
                    message: format!(
                        "{} 列的大部分记录集中在 {}，约占 {:.0}%",
                        summary.column,
                        month,
                        share * 100.0
                    ),
                });
            }
        }
    }

    observations
}

// 2026-03-21: 这里新增时间观察，目的是把用户最容易理解的高峰时段直接暴露出来。
fn build_time_observations(
    loaded: &LoadedTable,
    summaries: &[ColumnSummary],
) -> Vec<BusinessObservation> {
    let mut observations = Vec::new();

    for summary in summaries {
        if summary.summary_kind != "string"
            || summary.count == 0
            || !is_likely_time_column(loaded, summary)
        {
            continue;
        }

        let times = collect_string_values(loaded, &summary.column)
            .into_iter()
            .filter_map(|value| parse_time_value(&value))
            .collect::<Vec<_>>();
        if times.is_empty() {
            continue;
        }

        let mut period_counts = BTreeMap::<&'static str, usize>::new();
        let mut business_hour_count = 0_usize;
        for time in &times {
            *period_counts
                .entry(classify_time_period(time.hour))
                .or_default() += 1;
            if (9..=18).contains(&time.hour) {
                business_hour_count += 1;
            }
        }

        if let Some((period, count)) = period_counts
            .into_iter()
            .max_by(|left, right| left.1.cmp(&right.1).then_with(|| left.0.cmp(&right.0)))
        {
            let share = count as f64 / times.len() as f64;
            if share >= 0.50 {
                observations.push(BusinessObservation {
                    observation_type: "time_peak_period".to_string(),
                    column: Some(summary.column.clone()),
                    message: format!(
                        "{} 列主要集中在{}，约占 {:.0}%",
                        summary.column,
                        period,
                        share * 100.0
                    ),
                });
            }
        }

        let business_hour_ratio = business_hour_count as f64 / times.len() as f64;
        if business_hour_ratio >= 0.80 {
            observations.push(BusinessObservation {
                observation_type: "time_business_hour_pattern".to_string(),
                column: Some(summary.column.clone()),
                message: format!("{} 列的大部分记录发生在工作时段", summary.column),
            });
        }
    }

    observations
}

// 2026-03-21: 这里新增金额观察，目的是把常见区间、负数记录和长尾风险翻译成更业务化的提示。
fn build_amount_observations(
    loaded: &LoadedTable,
    summaries: &[ColumnSummary],
) -> Vec<BusinessObservation> {
    let mut observations = Vec::new();

    for summary in summaries {
        if summary.summary_kind != "numeric" || !looks_like_amount_column_name(&summary.column) {
            continue;
        }

        let mut values = collect_numeric_values(loaded, &summary.column);
        if values.is_empty() {
            continue;
        }

        values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
        let q1 = percentile(&values, 0.25);
        let q3 = percentile(&values, 0.75);
        observations.push(BusinessObservation {
            observation_type: "amount_typical_band".to_string(),
            column: Some(summary.column.clone()),
            message: format!(
                "{} 列的常见区间大致在 {} 到 {}",
                summary.column,
                format_number(q1),
                format_number(q3)
            ),
        });

        if values.iter().any(|value| *value < 0.0) {
            observations.push(BusinessObservation {
                observation_type: "amount_negative_presence".to_string(),
                column: Some(summary.column.clone()),
                message: format!(
                    "{} 列存在负数记录，建议留意退款、冲销或净额场景",
                    summary.column
                ),
            });
        }

        let chosen_center = calculate_numeric_center(loaded, summary);
        if chosen_center.label == "median_center" {
            observations.push(BusinessObservation {
                observation_type: "amount_skew_hint".to_string(),
                column: Some(summary.column.clone()),
                message: format!(
                    "{} 列少量高金额记录会明显拉高平均值，解读时更建议参考中位数",
                    summary.column
                ),
            });
        }
    }

    observations
}

// 2026-03-21: 这里为数值中心观察选择均值或中位数，目的是在偏态数据下优先输出更稳健的中心提示。
fn calculate_numeric_center(loaded: &LoadedTable, summary: &ColumnSummary) -> NumericCenterChoice {
    let mean = summary.mean.unwrap_or(0.0);
    let min_number = summary.min_number.unwrap_or(mean);
    let max_number = summary.max_number.unwrap_or(mean);
    let estimated_median = calculate_median(loaded, &summary.column).unwrap_or(mean);
    let range = (max_number - min_number).abs();
    let drift = (mean - estimated_median).abs();

    // 2026-03-21: 这里把偏态切换阈值放宽到 15%，目的是让“均值明显被极端值拉偏”的常见业务列更容易切到中位数。
    if range > 0.0 && drift / range >= 0.15 {
        NumericCenterChoice {
            label: "median_center",
            value: estimated_median,
        }
    } else {
        NumericCenterChoice {
            label: "numeric_center",
            value: mean,
        }
    }
}

// 2026-03-21: 这里给数值中心决策封一个小结构，目的是避免在业务观察构造时散落硬编码分支。
struct NumericCenterChoice {
    label: &'static str,
    value: f64,
}

// 2026-03-21: 这里直接从原始数值列计算中位数，目的是在不扩展 summary 契约的前提下先补稳健中心观察。
fn calculate_median(loaded: &LoadedTable, column: &str) -> Option<f64> {
    let mut values = collect_numeric_values(loaded, column);
    if values.is_empty() {
        return None;
    }

    values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let mid = values.len() / 2;

    if values.len() % 2 == 1 {
        Some(values[mid])
    } else {
        Some((values[mid - 1] + values[mid]) / 2.0)
    }
}

// 2026-03-21: 这里优先复用业务观察生成 quick_insights，目的是让人类摘要和结构化统计提示保持一致。
// 2026-03-21: 这里统一收集字符串列有效值，目的是让日期/时间语义识别与观察生成共享同一套输入口径。
fn collect_string_values(loaded: &LoadedTable, column: &str) -> Vec<String> {
    let Some(series) = loaded
        .dataframe
        .column(column)
        .ok()
        .map(|column| column.as_materialized_series())
    else {
        return Vec::new();
    };

    let mut values = Vec::new();
    for row_index in 0..series.len() {
        let Ok(value) = series.get(row_index) else {
            continue;
        };
        if matches!(value, AnyValue::Null) {
            continue;
        }

        let rendered = series
            .str_value(row_index)
            .map(|value| value.into_owned())
            .unwrap_or_default();
        let normalized = rendered.trim();
        if !normalized.is_empty() {
            values.push(normalized.to_string());
        }
    }

    values
}

// 2026-03-21: 这里用列名和样本解析成功率共同判断日期列，目的是减少普通文本列被误判成日期列。
fn is_likely_date_column(loaded: &LoadedTable, summary: &ColumnSummary) -> bool {
    if !looks_like_date_column_name(&summary.column) {
        return false;
    }

    has_parse_ratio(
        collect_string_values(loaded, &summary.column),
        parse_date_value,
    )
}

// 2026-03-21: 这里用列名和样本解析成功率共同判断时间列，目的是避免把普通文本标签误报成时段字段。
fn is_likely_time_column(loaded: &LoadedTable, summary: &ColumnSummary) -> bool {
    if !looks_like_time_column_name(&summary.column) {
        return false;
    }

    has_parse_ratio(
        collect_string_values(loaded, &summary.column),
        parse_time_value,
    )
}

// 2026-03-21: 这里统一计算解析成功率阈值，目的是把语义识别规则稳定收敛在一处维护。
fn has_parse_ratio<T, F>(values: Vec<String>, parser: F) -> bool
where
    F: Fn(&str) -> Option<T>,
{
    if values.is_empty() {
        return false;
    }

    let parsed_count = values
        .iter()
        .filter(|value| parser(value.as_str()).is_some())
        .count();
    parsed_count as f64 / values.len() as f64 >= 0.60
}

// 2026-03-21: 这里按观察优先级生成 quick_insights，目的是让摘要优先展示更有业务解释力的新增观察。
fn build_quick_insights(observations: &[BusinessObservation]) -> Vec<String> {
    let mut ranked = observations.to_vec();
    ranked.sort_by(|left, right| {
        business_observation_priority(&left.observation_type)
            .cmp(&business_observation_priority(&right.observation_type))
            .then_with(|| left.column.cmp(&right.column))
    });

    ranked
        .iter()
        .take(2)
        .map(|observation| observation.message.clone())
        .collect()
}

// 2026-03-21: 这里定义 quick_insights 的展示优先级，目的是让真正有解释力的观察先进入摘要。
fn business_observation_priority(observation_type: &str) -> usize {
    match observation_type {
        "amount_skew_hint" => 0,
        "time_peak_period" => 1,
        "date_concentration" => 2,
        "amount_typical_band" => 3,
        "date_range" => 4,
        "amount_negative_presence" => 5,
        _ => 10,
    }
}

// 2026-03-21: 这里把展示层 finding 转成下一步动作建议，目的是让 Skill 和问答界面能直接承接后续操作。
fn build_next_actions(findings: &[AnalyzeFinding]) -> Vec<String> {
    let mut next_actions = Vec::new();

    if findings
        .iter()
        .any(|finding| finding.code == "all_missing" || finding.code == "high_missing_rate")
    {
        next_actions.push("建议先处理缺失值较高的列，再继续分析。".to_string());
    }
    if findings.iter().any(|finding| {
        finding.code == "duplicate_rows" || finding.code == "duplicate_candidate_key"
    }) {
        next_actions.push("建议先去重或确认主键唯一性，再继续关联或建模。".to_string());
    }
    if findings.iter().any(|finding| {
        finding.code == "high_category_imbalance" || finding.code == "outlier_suspected"
    }) {
        next_actions.push("建议先核查分布异常和极端值，再决定是否直接用于统计或建模。".to_string());
    }

    if next_actions.is_empty() {
        next_actions.push("建议继续检查缺失、重复和值分布，再进入后续分析。".to_string());
    }

    next_actions
}

// 2026-03-21: 这里统一格式化数值，目的是让业务观察文案更适合直接展示给 Excel 用户。
fn format_number(value: f64) -> String {
    if (value.fract()).abs() < f64::EPSILON {
        format!("{:.0}", value)
    } else {
        format!("{:.2}", value)
    }
}
