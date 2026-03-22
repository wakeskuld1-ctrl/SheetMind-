use std::collections::BTreeSet;

use serde::Serialize;
use thiserror::Error;

use crate::frame::loader::LoadedTable;
use crate::ops::semantic::looks_like_identifier_column_name;

// 2026-03-21: 这里定义 keep_mode 的中文选项，目的是让上层 Skill 直接用业务语言引导用户做保留范围选择。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LinkKeepModeOption {
    pub keep_mode: String,
    pub label: String,
    pub description: String,
}

// 2026-03-21: 这里定义单个显性关联候选，目的是把列对、覆盖率、原因和提问话术稳定暴露给 Skill。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TableLinkCandidate {
    pub left_column: String,
    pub right_column: String,
    pub confidence: String,
    pub match_row_count: usize,
    pub left_match_rate: f64,
    pub right_match_rate: f64,
    pub reason: String,
    pub question: String,
    #[serde(default)]
    pub keep_mode_options: Vec<LinkKeepModeOption>,
}

// 2026-03-21: 这里定义关系建议 Tool 的统一输出，目的是让问答界面既拿到候选，也拿到总览提示与下一步动作。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TableLinkSuggestionResult {
    #[serde(default)]
    pub candidates: Vec<TableLinkCandidate>,
    pub human_summary: String,
    pub recommended_next_step: String,
}

// 2026-03-21: 这里定义关系建议错误，目的是把底层读值失败统一转成上层可消费的业务错误。
#[derive(Debug, Error)]
pub enum TableLinkSuggestionError {
    #[error("无法读取列 `{column}`: {message}")]
    ReadColumn { column: String, message: String },
}

#[derive(Debug, Clone, PartialEq)]
struct ColumnMatchProfile {
    match_row_count: usize,
    left_match_rate: f64,
    right_match_rate: f64,
}

// 2026-03-21: 这里提供多表显性关联建议主入口，目的是在真正执行 join 之前先识别最明显的候选关系。
pub fn suggest_table_links(
    left: &LoadedTable,
    right: &LoadedTable,
    max_candidates: usize,
) -> Result<TableLinkSuggestionResult, TableLinkSuggestionError> {
    let mut candidates = Vec::<TableLinkCandidate>::new();

    for left_column in left.handle.columns() {
        for right_column in right.handle.columns() {
            if !is_obvious_link_pair(left_column, right_column) {
                continue;
            }

            let profile = build_match_profile(left, right, left_column, right_column)?;
            if !is_obvious_match(&profile) {
                continue;
            }

            candidates.push(TableLinkCandidate {
                left_column: left_column.clone(),
                right_column: right_column.clone(),
                confidence: confidence_label(left_column, right_column, &profile).to_string(),
                match_row_count: profile.match_row_count,
                left_match_rate: profile.left_match_rate,
                right_match_rate: profile.right_match_rate,
                reason: build_reason(left_column, right_column, &profile),
                question: build_question(left_column, right_column),
                keep_mode_options: default_keep_mode_options(),
            });
        }
    }

    // 2026-03-22: 这里补上“标识列优先”的稳定排序，目的是避免 user_id 与 region 同时命中时把业务主键排到后面。
    // 2026-03-21: 这里统一按置信度和覆盖率排序，目的是让 Skill 总是先看到最稳妥的显性关联建议。
    candidates.sort_by(|left_candidate, right_candidate| {
        confidence_rank(&right_candidate.confidence)
            .cmp(&confidence_rank(&left_candidate.confidence))
            .then_with(|| {
                link_priority_rank(&right_candidate.left_column, &right_candidate.right_column).cmp(
                    &link_priority_rank(&left_candidate.left_column, &left_candidate.right_column),
                )
            })
            .then_with(|| {
                right_candidate
                    .match_row_count
                    .cmp(&left_candidate.match_row_count)
            })
            .then_with(|| {
                right_candidate
                    .left_match_rate
                    .total_cmp(&left_candidate.left_match_rate)
            })
            .then_with(|| {
                right_candidate
                    .right_match_rate
                    .total_cmp(&left_candidate.right_match_rate)
            })
    });
    candidates.truncate(max_candidates.max(1));

    let human_summary = if candidates.is_empty() {
        "当前没有识别到足够明显的显性关联候选，建议先确认两张表里是否存在同名或明显对应的 ID/编号 列。".to_string()
    } else {
        format!(
            "当前识别到 {} 个较明显的显性关联候选，建议先确认第一个候选是否符合你的业务主键。",
            candidates.len()
        )
    };
    let recommended_next_step = if candidates.is_empty() {
        "建议先让用户确认哪两列是真正对应的标识列，再决定是否调用 join_tables。".to_string()
    } else {
        "建议先把第一个候选用业务语言问清楚，如果确认无误，再调用 join_tables。".to_string()
    };

    Ok(TableLinkSuggestionResult {
        candidates,
        human_summary,
        recommended_next_step,
    })
}

// 2026-03-21: 这里只允许“明显特征”的列对进入候选打分，目的是让 V2.1 先走保守路线，不去猜复杂关系。
fn is_obvious_link_pair(left_column: &str, right_column: &str) -> bool {
    let left_normalized = normalize_name(left_column);
    let right_normalized = normalize_name(right_column);

    left_normalized == right_normalized
        || (looks_like_identifier_column_name(left_column)
            && looks_like_identifier_column_name(right_column)
            && identifier_stem(&left_normalized) == identifier_stem(&right_normalized)
            && !identifier_stem(&left_normalized).is_empty())
}

// 2026-03-21: 这里计算左右列的行级覆盖率，目的是把“值真的能对上多少”显式暴露给上层，而不是只看列名。
fn build_match_profile(
    left: &LoadedTable,
    right: &LoadedTable,
    left_column: &str,
    right_column: &str,
) -> Result<ColumnMatchProfile, TableLinkSuggestionError> {
    let left_values = read_non_blank_values(left, left_column)?;
    let right_values = read_non_blank_values(right, right_column)?;

    if left_values.is_empty() || right_values.is_empty() {
        return Ok(ColumnMatchProfile {
            match_row_count: 0,
            left_match_rate: 0.0,
            right_match_rate: 0.0,
        });
    }

    let right_set = right_values.iter().cloned().collect::<BTreeSet<_>>();
    let left_set = left_values.iter().cloned().collect::<BTreeSet<_>>();
    let left_match_count = left_values
        .iter()
        .filter(|value| right_set.contains(*value))
        .count();
    let right_match_count = right_values
        .iter()
        .filter(|value| left_set.contains(*value))
        .count();

    Ok(ColumnMatchProfile {
        match_row_count: left_match_count.min(right_match_count),
        left_match_rate: left_match_count as f64 / left_values.len() as f64,
        right_match_rate: right_match_count as f64 / right_values.len() as f64,
    })
}

// 2026-03-21: 这里限定进入结果的最小覆盖率，目的是只返回显性的候选，避免误导用户做错误关联。
fn is_obvious_match(profile: &ColumnMatchProfile) -> bool {
    profile.match_row_count > 0 && profile.left_match_rate >= 0.5 && profile.right_match_rate >= 0.5
}

// 2026-03-21: 这里用保守规则给置信度分级，目的是让同名且高覆盖的候选稳居第一优先级。
fn confidence_label(
    left_column: &str,
    right_column: &str,
    profile: &ColumnMatchProfile,
) -> &'static str {
    if normalize_name(left_column) == normalize_name(right_column)
        && profile.left_match_rate >= 0.6
        && profile.right_match_rate >= 0.6
    {
        "high"
    } else {
        "medium"
    }
}

// 2026-03-21: 这里生成人类可读原因，目的是让 Skill 不用自己二次拼接覆盖率说明。
fn build_reason(left_column: &str, right_column: &str, profile: &ColumnMatchProfile) -> String {
    format!(
        "`{}` 与 `{}` 列名特征明显相近，且两边分别有 {:.0}% / {:.0}% 的有效记录能对上。",
        left_column,
        right_column,
        profile.left_match_rate * 100.0,
        profile.right_match_rate * 100.0
    )
}

// 2026-03-21: 这里直接输出业务问题话术，目的是把技术层候选翻译成终端用户能听懂的确认问题。
fn build_question(left_column: &str, right_column: &str) -> String {
    format!(
        "是否用 A 表 `{}` 列去关联 B 表 `{}` 列？如果两边不一致，你希望只保留两边都有的数据，还是优先保留 A 表或 B 表？",
        left_column, right_column
    )
}

// 2026-03-21: 这里给出 keep_mode 的中文选项，目的是让显性 Join 的执行选择继续保持非技术表达。
fn default_keep_mode_options() -> Vec<LinkKeepModeOption> {
    vec![
        LinkKeepModeOption {
            keep_mode: "matched_only".to_string(),
            label: "只保留两边都有的数据".to_string(),
            description: "适合只看两张表都能对上的记录。".to_string(),
        },
        LinkKeepModeOption {
            keep_mode: "keep_left".to_string(),
            label: "优先保留 A 表".to_string(),
            description: "适合以 A 表为主，B 表能补充就补充。".to_string(),
        },
        LinkKeepModeOption {
            keep_mode: "keep_right".to_string(),
            label: "优先保留 B 表".to_string(),
            description: "适合以 B 表为主，A 表能补充就补充。".to_string(),
        },
    ]
}

// 2026-03-21: 这里统一读取非空白文本值，目的是让后续匹配逻辑只比较真实有效值，不把空白当作命中。
fn read_non_blank_values(
    loaded: &LoadedTable,
    column: &str,
) -> Result<Vec<String>, TableLinkSuggestionError> {
    let series = loaded
        .dataframe
        .column(column)
        .map_err(|error| TableLinkSuggestionError::ReadColumn {
            column: column.to_string(),
            message: error.to_string(),
        })?
        .as_materialized_series();
    let mut values = Vec::new();

    for row_index in 0..series.len() {
        let rendered = series
            .str_value(row_index)
            .map_err(|error| TableLinkSuggestionError::ReadColumn {
                column: column.to_string(),
                message: error.to_string(),
            })?
            .trim()
            .to_string();
        if rendered.is_empty() {
            continue;
        }
        values.push(rendered);
    }

    Ok(values)
}

// 2026-03-21: 这里统一做列名规范化，目的是兼容大小写、下划线和短横线差异。
fn normalize_name(column: &str) -> String {
    column.trim().to_lowercase().replace([' ', '_', '-'], "")
}

// 2026-03-21: 这里抽取标识列主体，目的是让 user_id / userid / user-code 这类命名更容易归到同一类保守候选。
fn identifier_stem(normalized: &str) -> String {
    normalized
        .trim_end_matches("id")
        .trim_end_matches("code")
        .trim_end_matches("no")
        .trim_end_matches("编号")
        .trim_end_matches("编码")
        .trim_end_matches("代码")
        .to_string()
}

// 2026-03-21: 这里统一映射置信度排序权重，目的是让候选排序稳定可预测。
fn confidence_rank(label: &str) -> usize {
    match label {
        "high" => 2,
        "medium" => 1,
        _ => 0,
    }
}

// 2026-03-22: 这里给显性关联候选补业务主键优先级，目的是在覆盖率接近时优先把 ID/编号类列稳定排在地区/名称等普通字段前面。
fn link_priority_rank(left_column: &str, right_column: &str) -> usize {
    if looks_like_identifier_column_name(left_column)
        && looks_like_identifier_column_name(right_column)
    {
        1
    } else {
        0
    }
}
