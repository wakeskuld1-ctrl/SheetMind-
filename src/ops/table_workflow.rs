use std::collections::BTreeSet;

use serde::Serialize;
use serde_json::{Value, json};
use thiserror::Error;

use crate::frame::loader::LoadedTable;
use crate::ops::table_links::{TableLinkCandidate, TableLinkSuggestionError, suggest_table_links};

// 2026-03-22: 这里定义纵向追加候选，目的是把“结构相同可直接上下拼接”的判断稳定暴露给上层 Skill。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TableAppendCandidate {
    pub confidence: String,
    #[serde(default)]
    pub shared_columns: Vec<String>,
    pub left_row_count: usize,
    pub right_row_count: usize,
    pub reason: String,
    pub question: String,
}

// 2026-03-22: 这里定义多表流程建议的统一输出，目的是让上层一次拿到推荐动作、追加候选和关联候选。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TableWorkflowSuggestionResult {
    pub recommended_action: String,
    pub action_reason: String,
    pub human_summary: String,
    pub recommended_next_step: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub append_candidate: Option<TableAppendCandidate>,
    #[serde(default)]
    pub link_candidates: Vec<TableLinkCandidate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_tool_call: Option<SuggestedToolCall>,
}

// 2026-03-22: 这里定义工作流建议错误，目的是把底层显性关联建议失败统一抬升到工作流层。
#[derive(Debug, Error)]
pub enum TableWorkflowSuggestionError {
    #[error(transparent)]
    LinkSuggestion(#[from] TableLinkSuggestionError),
}

// 2026-03-22: 这里定义建议调用骨架，目的是让 Skill 可以直接复用推荐动作和参数，而不是再次手工拼 JSON。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SuggestedToolCall {
    pub tool: String,
    pub args: Value,
}

// 2026-03-22: 这里提供两表下一步动作建议主入口，目的是先判断更像追加、关联还是继续确认，再由 Skill 发问。
pub fn suggest_table_workflow(
    left: &LoadedTable,
    right: &LoadedTable,
    max_link_candidates: usize,
) -> Result<TableWorkflowSuggestionResult, TableWorkflowSuggestionError> {
    let append_candidate = build_append_candidate(left, right);
    let link_result = suggest_table_links(left, right, max_link_candidates)?;

    if let Some(append_candidate) = append_candidate {
        return Ok(TableWorkflowSuggestionResult {
            recommended_action: "append_tables".to_string(),
            action_reason: "两张表列结构一致，更像同一类数据的分批补充。".to_string(),
            human_summary:
                "当前更像是结构相同的数据表，建议优先考虑纵向追加，而不是直接做字段关联。"
                    .to_string(),
            recommended_next_step:
                "建议先确认是否把 B 表追加到 A 表下方；如果确认无误，再调用 append_tables。"
                    .to_string(),
            append_candidate: Some(append_candidate),
            link_candidates: link_result.candidates,
            suggested_tool_call: Some(build_append_tool_call(left, right)),
        });
    }

    if !link_result.candidates.is_empty() {
        let first_candidate = &link_result.candidates[0];
        return Ok(TableWorkflowSuggestionResult {
            recommended_action: "join_preflight".to_string(),
            action_reason: "两张表结构不同，但已经识别到明显的显性关联候选。".to_string(),
            human_summary:
                "当前更像是两张互相补信息的表，建议先确认显性关联列，再决定是否执行关联。"
                    .to_string(),
            recommended_next_step:
                "建议先把第一个关联候选用业务语言问清楚；如果确认无误，先调用 join_preflight 预览风险与结果，再决定是否执行 join_tables。"
                    .to_string(),
            append_candidate: None,
            suggested_tool_call: Some(build_join_tool_call(left, right, first_candidate)),
            link_candidates: link_result.candidates,
        });
    }

    Ok(TableWorkflowSuggestionResult {
        recommended_action: "manual_confirmation".to_string(),
        action_reason: "当前既没有识别到结构一致追加候选，也没有识别到足够明显的显性关联候选。".to_string(),
        human_summary:
            "当前没有识别到明显的追加或关联动作，建议继续确认这两张表是否属于同结构数据，或者是否存在对应的 ID 列。"
                .to_string(),
        recommended_next_step:
            "建议先确认两张表是要上下拼接，还是要用某个标识列互相关联，再决定下一步。".to_string(),
        append_candidate: None,
        link_candidates: link_result.candidates,
        suggested_tool_call: None,
    })
}

// 2026-03-22: 这里判断结构一致追加候选，目的是把“列集合相同”的高置信度追加场景从 Skill 猜测下沉到 Tool 计算层。
fn build_append_candidate(left: &LoadedTable, right: &LoadedTable) -> Option<TableAppendCandidate> {
    let left_columns = left.handle.columns();
    let right_columns = right.handle.columns();

    let left_set = left_columns.iter().cloned().collect::<BTreeSet<_>>();
    let right_set = right_columns.iter().cloned().collect::<BTreeSet<_>>();

    if left_set != right_set {
        return None;
    }

    Some(TableAppendCandidate {
        confidence: "high".to_string(),
        shared_columns: left_columns.to_vec(),
        left_row_count: left.dataframe.height(),
        right_row_count: right.dataframe.height(),
        reason: "两张表的列结构一致，更像同一类数据在不同工作表或工作簿里的分批记录。".to_string(),
        question: "这两张表结构相同，是否把 B 表的数据追加到 A 表下方？".to_string(),
    })
}

// 2026-03-22: 这里生成追加执行骨架，目的是让上层确认后可直接把建议转成 append_tables 调用。
fn build_append_tool_call(left: &LoadedTable, right: &LoadedTable) -> SuggestedToolCall {
    SuggestedToolCall {
        tool: "append_tables".to_string(),
        args: json!({
            "top": {
                "path": left.handle.source_path(),
                "sheet": left.handle.sheet_name(),
            },
            "bottom": {
                "path": right.handle.source_path(),
                "sheet": right.handle.sheet_name(),
            }
        }),
    }
}

// 2026-03-22: 这里生成关联预检骨架，目的是让上层确认首个候选后先做 join_preflight，再决定是否执行 join_tables。
fn build_join_tool_call(
    left: &LoadedTable,
    right: &LoadedTable,
    candidate: &TableLinkCandidate,
) -> SuggestedToolCall {
    SuggestedToolCall {
        tool: "join_preflight".to_string(),
        args: json!({
            "left": {
                "path": left.handle.source_path(),
                "sheet": left.handle.sheet_name(),
            },
            "right": {
                "path": right.handle.source_path(),
                "sheet": right.handle.sheet_name(),
            },
            "left_on": candidate.left_column,
            "right_on": candidate.right_column,
            "keep_mode": "matched_only"
        }),
    }
}
