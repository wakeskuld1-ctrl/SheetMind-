use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::{Value, json};
use thiserror::Error;

use crate::frame::loader::LoadedTable;
use crate::ops::append::{AppendError, append_tables};
use crate::ops::join::{JoinError, JoinKeepMode, join_tables};
use crate::ops::table_links::{TableLinkSuggestionError, suggest_table_links};

// 2026-03-22: 这里定义多表计划步骤，目的是把“先追加、再关联”的顺序建议稳定暴露给上层 Skill。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MultiTablePlanStep {
    pub step_id: String,
    pub action: String,
    #[serde(default)]
    pub input_refs: Vec<String>,
    pub result_ref: String,
    pub confidence: String,
    pub reason: String,
    pub question: String,
    pub suggested_tool_call: Value,
}

// 2026-03-22: 这里定义多表计划总输出，目的是让问答界面一次拿到步骤、未决表与下一步提示。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MultiTablePlanResult {
    #[serde(default)]
    pub steps: Vec<MultiTablePlanStep>,
    #[serde(default)]
    pub unresolved_refs: Vec<String>,
    pub human_summary: String,
    pub recommended_next_step: String,
}

#[derive(Debug, Error)]
pub enum MultiTablePlanError {
    #[error("suggest_multi_table_plan 至少需要两张表")]
    NotEnoughTables,
    #[error(transparent)]
    Append(#[from] AppendError),
    #[error(transparent)]
    Join(#[from] JoinError),
    #[error(transparent)]
    LinkSuggestion(#[from] TableLinkSuggestionError),
}

pub struct PlanningTable {
    pub table_ref: String,
    pub loaded: LoadedTable,
}

enum ExecutionRef {
    Source { path: String, sheet: String },
    Result { result_ref: String },
}

struct PlanNode {
    table_ref: String,
    execution_ref: ExecutionRef,
    loaded: LoadedTable,
}

// 2026-03-22: 这里提供多表顺序建议主入口，目的是先把同结构表合并，再把显性可关联表串起来，剩余表保留待确认。
pub fn suggest_multi_table_plan(
    tables: Vec<(String, LoadedTable)>,
    max_link_candidates: usize,
) -> Result<MultiTablePlanResult, MultiTablePlanError> {
    if tables.len() < 2 {
        return Err(MultiTablePlanError::NotEnoughTables);
    }

    let mut nodes = tables
        .into_iter()
        .map(|(table_ref, loaded)| PlanNode {
            table_ref,
            execution_ref: ExecutionRef::Source {
                path: loaded.handle.source_path().to_string(),
                sheet: loaded.handle.sheet_name().to_string(),
            },
            loaded,
        })
        .collect::<Vec<_>>();
    let mut steps = Vec::<MultiTablePlanStep>::new();
    let mut step_index = 1usize;

    nodes = build_append_steps(nodes, &mut steps, &mut step_index)?;
    nodes = build_join_steps(nodes, max_link_candidates, &mut steps, &mut step_index)?;

    let unresolved_refs = nodes
        .into_iter()
        .map(|node| node.table_ref)
        .collect::<Vec<_>>();
    let human_summary = if steps.is_empty() {
        "当前没有形成足够明确的多表处理计划，建议先确认哪些表属于同结构数据，哪些表需要通过标识列互相关联。".to_string()
    } else {
        format!(
            "当前已生成 {} 步多表处理建议，建议先按步骤顺序确认并执行，剩余未决表再单独处理。",
            steps.len()
        )
    };
    let recommended_next_step = if steps.is_empty() {
        "建议先人工确认每张表的用途，再决定是否追加或关联。".to_string()
    } else {
        "建议先从第一步开始确认；如果确认无误，再按计划顺序执行 append_tables 或 join_tables。"
            .to_string()
    };

    Ok(MultiTablePlanResult {
        steps,
        unresolved_refs,
        human_summary,
        recommended_next_step,
    })
}

// 2026-03-22: 这里先按列集合分组构造追加链，目的是把同结构批次数据优先合并成代表表，减少后续多表关联噪声。
fn build_append_steps(
    nodes: Vec<PlanNode>,
    steps: &mut Vec<MultiTablePlanStep>,
    step_index: &mut usize,
) -> Result<Vec<PlanNode>, MultiTablePlanError> {
    let mut group_indexes = BTreeMap::<String, usize>::new();
    let mut groups = Vec::<Vec<PlanNode>>::new();
    for node in nodes {
        let mut sorted_columns = node.loaded.handle.columns().to_vec();
        sorted_columns.sort();
        let group_key = sorted_columns.join("|");
        match group_indexes.get(&group_key).copied() {
            Some(group_index) => groups[group_index].push(node),
            None => {
                group_indexes.insert(group_key, groups.len());
                groups.push(vec![node]);
            }
        }
    }

    let mut next_nodes = Vec::<PlanNode>::new();
    for mut group in groups {
        if group.len() == 1 {
            next_nodes.push(group.remove(0));
            continue;
        }

        let mut current = group.remove(0);
        for next in group {
            let step_id = format!("step_{}", *step_index);
            let result_ref = format!("{}_result", step_id);
            let appended = append_tables(&current.loaded, &next.loaded)?;

            steps.push(MultiTablePlanStep {
                step_id: step_id.clone(),
                action: "append_tables".to_string(),
                input_refs: vec![current.table_ref.clone(), next.table_ref.clone()],
                result_ref: result_ref.clone(),
                confidence: "high".to_string(),
                reason: "这两张表列结构一致，建议先纵向追加，形成统一代表表后再继续后续分析。"
                    .to_string(),
                question: format!(
                    "是否先把 `{}` 的数据追加到 `{}` 下方，形成统一结果后再继续下一步？",
                    next.table_ref, current.table_ref
                ),
                suggested_tool_call: json!({
                    "tool": "append_tables",
                    "args": {
                        "top": execution_ref_payload(&current.execution_ref),
                        "bottom": execution_ref_payload(&next.execution_ref),
                    }
                }),
            });

            current = PlanNode {
                table_ref: result_ref.clone(),
                execution_ref: ExecutionRef::Result { result_ref },
                loaded: appended,
            };
            *step_index += 1;
        }

        next_nodes.push(current);
    }

    Ok(next_nodes)
}

// 2026-03-22: 这里按最明显关联候选继续构造 join 链，目的是把追加后的代表表进一步串成保守可执行的多表计划。
fn build_join_steps(
    mut nodes: Vec<PlanNode>,
    max_link_candidates: usize,
    steps: &mut Vec<MultiTablePlanStep>,
    step_index: &mut usize,
) -> Result<Vec<PlanNode>, MultiTablePlanError> {
    loop {
        let Some((left_index, right_index, left_on, right_on, reason, question, confidence)) =
            find_best_join_pair(&nodes, max_link_candidates)?
        else {
            break;
        };

        let high_index = left_index.max(right_index);
        let low_index = left_index.min(right_index);
        let right_node = nodes.remove(high_index);
        let left_node = nodes.remove(low_index);

        let step_id = format!("step_{}", *step_index);
        let result_ref = format!("{}_result", step_id);
        let joined = join_tables(
            &left_node.loaded,
            &right_node.loaded,
            &left_on,
            &right_on,
            JoinKeepMode::MatchedOnly,
        )?;

        steps.push(MultiTablePlanStep {
            step_id: step_id.clone(),
            action: "join_tables".to_string(),
            input_refs: vec![left_node.table_ref.clone(), right_node.table_ref.clone()],
            result_ref: result_ref.clone(),
            confidence,
            reason,
            question,
            suggested_tool_call: json!({
                "tool": "join_tables",
                "args": {
                    "left": execution_ref_payload(&left_node.execution_ref),
                    "right": execution_ref_payload(&right_node.execution_ref),
                    "left_on": left_on,
                    "right_on": right_on,
                    "keep_mode": "matched_only",
                }
            }),
        });

        nodes.push(PlanNode {
            table_ref: result_ref.clone(),
            execution_ref: ExecutionRef::Result { result_ref },
            loaded: joined,
        });
        *step_index += 1;
    }

    Ok(nodes)
}

// 2026-03-22: 这里在当前代表表之间寻找最明显的显性关联候选，目的是保持多表计划器的顺序建议足够保守且可解释。
fn find_best_join_pair(
    nodes: &[PlanNode],
    max_link_candidates: usize,
) -> Result<Option<(usize, usize, String, String, String, String, String)>, MultiTablePlanError> {
    let mut best_pair: Option<(
        usize,
        usize,
        String,
        String,
        String,
        String,
        String,
        usize,
        usize,
    )> = None;

    for left_index in 0..nodes.len() {
        for right_index in (left_index + 1)..nodes.len() {
            let link_result = suggest_table_links(
                &nodes[left_index].loaded,
                &nodes[right_index].loaded,
                max_link_candidates,
            )?;
            let Some(candidate) = link_result.candidates.first() else {
                continue;
            };

            let confidence_rank = match candidate.confidence.as_str() {
                "high" => 2,
                "medium" => 1,
                _ => 0,
            };
            let match_rows = candidate.match_row_count;

            // 2026-03-22: 这里显式跳过 confidence 原文、只取排序权重和命中行数，目的是修复候选元组解包错位导致的编译失败。
            let should_replace = match &best_pair {
                Some((_, _, _, _, _, _, _, best_rank, best_match_rows)) => {
                    confidence_rank > *best_rank
                        || (confidence_rank == *best_rank && match_rows > *best_match_rows)
                }
                None => true,
            };

            if should_replace {
                best_pair = Some((
                    left_index,
                    right_index,
                    candidate.left_column.clone(),
                    candidate.right_column.clone(),
                    candidate.reason.clone(),
                    candidate.question.clone(),
                    candidate.confidence.clone(),
                    confidence_rank,
                    match_rows,
                ));
            }
        }
    }

    Ok(best_pair.map(
        |(left_index, right_index, left_on, right_on, reason, question, confidence, _, _)| {
            (
                left_index,
                right_index,
                left_on,
                right_on,
                reason,
                question,
                confidence,
            )
        },
    ))
}

// 2026-03-22: 这里把原始表引用和中间结果引用统一转成 JSON，目的是让计划步骤既能指向源表，也能指向前一步结果。
fn execution_ref_payload(reference: &ExecutionRef) -> Value {
    match reference {
        ExecutionRef::Source { path, sheet } => json!({
            "path": path,
            "sheet": sheet,
        }),
        ExecutionRef::Result { result_ref } => json!({
            "result_ref": result_ref,
        }),
    }
}
