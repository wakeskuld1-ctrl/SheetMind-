use serde::Serialize;
use thiserror::Error;

use crate::frame::loader::LoadedTable;
use crate::ops::analyze::{AnalyzeFinding, AnalyzeResult, TableHealth, analyze_table};
use crate::ops::stat_summary::{StatSummaryResult, stat_summary};
use crate::ops::summary::SummaryError;

// 2026-03-21: 这里定义阻塞风险结构，目的是把“哪些问题会卡住后续动作”单独暴露给问答层。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BlockingRisk {
    pub code: String,
    pub severity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<String>,
    pub message: String,
    pub why_blocking: String,
}

// 2026-03-21: 这里定义优先动作，目的是让低 IT 用户直接看到先做什么、为什么做、建议调用什么 Tool。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PriorityAction {
    pub priority: String,
    pub title: String,
    pub reason: String,
    #[serde(default)]
    pub suggested_tools: Vec<String>,
}

// 2026-03-21: 这里定义下一步 Tool 建议，目的是把决策助手与下层原子能力稳定串起来。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NextToolSuggestion {
    pub tool: String,
    pub status: String,
    pub reason: String,
}

// 2026-03-21: 这里定义人类摘要，目的是让决策助手也遵循双层输出：结构化结果 + 中文摘要。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DecisionHumanSummary {
    pub overall: String,
    #[serde(default)]
    pub immediate_actions: Vec<String>,
    pub recommended_next_step: String,
}

// 2026-03-21: 这里定义决策助手统一输出，目的是把统计诊断与下一步建议合并成一个稳定高层 Tool。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DecisionAssistantResult {
    pub assistant_kind: String,
    pub table_health: TableHealth,
    #[serde(default)]
    pub blocking_risks: Vec<BlockingRisk>,
    #[serde(default)]
    pub priority_actions: Vec<PriorityAction>,
    #[serde(default)]
    pub business_highlights: Vec<String>,
    #[serde(default)]
    pub next_tool_suggestions: Vec<NextToolSuggestion>,
    pub human_summary: DecisionHumanSummary,
}

// 2026-03-21: 这里定义决策助手错误，目的是把底层统计摘要异常继续转为统一中文错误。
#[derive(Debug, Error)]
pub enum DecisionAssistantError {
    #[error(transparent)]
    Summary(#[from] SummaryError),
}

// 2026-03-21: 这里提供决策助手主入口，目的是让上层在 V1 先拿到“质量诊断优先”的规则化决策建议。
pub fn decision_assistant(
    loaded: &LoadedTable,
    requested_columns: &[&str],
    top_k: usize,
) -> Result<DecisionAssistantResult, DecisionAssistantError> {
    let analyze_result = analyze_table(loaded, requested_columns, top_k);
    let stat_result = stat_summary(loaded, requested_columns, top_k)?;
    let blocking_risks = build_blocking_risks(&analyze_result);
    let priority_actions = build_priority_actions(&analyze_result, &blocking_risks);
    let business_highlights = build_business_highlights(&analyze_result, &stat_result);
    let next_tool_suggestions = build_next_tool_suggestions(&blocking_risks, &stat_result);
    let human_summary =
        build_human_summary(&analyze_result, &priority_actions, &next_tool_suggestions);

    Ok(DecisionAssistantResult {
        assistant_kind: "quality_diagnostic".to_string(),
        table_health: analyze_result.table_health,
        blocking_risks,
        priority_actions,
        business_highlights,
        next_tool_suggestions,
        human_summary,
    })
}

// 2026-03-21: 这里集中识别阻塞性 finding，目的是把真正会卡住关联、建模和统计解释的问题优先挑出来。
fn build_blocking_risks(analyze_result: &AnalyzeResult) -> Vec<BlockingRisk> {
    analyze_result
        .structured_findings
        .iter()
        .filter(|finding| is_blocking_finding(finding))
        .map(|finding| BlockingRisk {
            code: finding.code.clone(),
            severity: finding.severity.clone(),
            column: finding.column.clone(),
            message: finding.message.clone(),
            why_blocking: blocking_reason(finding.code.as_str()).to_string(),
        })
        .collect()
}

// 2026-03-21: 这里生成优先动作，目的是把“先清洗、再分析”的顺序明确告诉用户，而不是只丢风险列表。
fn build_priority_actions(
    analyze_result: &AnalyzeResult,
    blocking_risks: &[BlockingRisk],
) -> Vec<PriorityAction> {
    let mut actions = Vec::new();

    if !blocking_risks.is_empty() {
        actions.push(PriorityAction {
            priority: "high".to_string(),
            title: "先处理高风险数据质量问题".to_string(),
            reason: format!(
                "当前至少有 {} 项阻塞风险会直接影响后续关联、统计或建模结果。",
                blocking_risks.len()
            ),
            suggested_tools: vec!["analyze_table".to_string(), "filter_rows".to_string()],
        });
    }

    if analyze_result.structured_findings.iter().any(|finding| {
        matches!(
            finding.code.as_str(),
            "high_zero_ratio" | "outlier_suspected" | "high_category_imbalance"
        )
    }) {
        actions.push(PriorityAction {
            priority: if blocking_risks.is_empty() {
                "high".to_string()
            } else {
                "medium".to_string()
            },
            title: "再检查分布异常与极端值".to_string(),
            reason: "当前表中已经出现分布失衡、零值堆积或异常值迹象，直接建模容易放大偏差。"
                .to_string(),
            suggested_tools: vec![
                "stat_summary".to_string(),
                "group_and_aggregate".to_string(),
            ],
        });
    }

    if actions.is_empty() {
        actions.push(PriorityAction {
            priority: "medium".to_string(),
            title: "可以进入下一步分析建模".to_string(),
            reason: "当前没有识别出明显阻塞项，可以开始尝试统计摘要、聚类或回归分析。".to_string(),
            suggested_tools: vec!["stat_summary".to_string(), "cluster_kmeans".to_string()],
        });
    }

    actions
}

// 2026-03-21: 这里汇总业务亮点，目的是在质量诊断之外顺带给用户几条可直接读的业务观察。
fn build_business_highlights(
    analyze_result: &AnalyzeResult,
    stat_result: &StatSummaryResult,
) -> Vec<String> {
    let mut highlights = analyze_result
        .business_observations
        .iter()
        .map(|observation| observation.message.clone())
        .collect::<Vec<_>>();

    for key_point in &stat_result.human_summary.key_points {
        if !highlights.iter().any(|item| item == key_point) {
            highlights.push(key_point.clone());
        }
    }

    highlights.truncate(6);
    highlights
}

// 2026-03-21: 这里生成下一步 Tool 建议，目的是把“诊断完以后可以做什么”规则化输出出来。
fn build_next_tool_suggestions(
    blocking_risks: &[BlockingRisk],
    stat_result: &StatSummaryResult,
) -> Vec<NextToolSuggestion> {
    let has_blocking = !blocking_risks.is_empty();
    let mut suggestions = Vec::new();

    if stat_result.table_overview.numeric_columns >= 2 {
        suggestions.push(NextToolSuggestion {
            tool: "cluster_kmeans".to_string(),
            status: readiness_label(has_blocking).to_string(),
            reason: readiness_reason(
                has_blocking,
                "当前至少有两个数值列，可以尝试做分群，观察样本结构是否天然分层。",
            )
            .to_string(),
        });
        suggestions.push(NextToolSuggestion {
            tool: "linear_regression".to_string(),
            status: readiness_label(has_blocking).to_string(),
            reason: readiness_reason(
                has_blocking,
                "当前有多个数值列，可以选择一个数值目标列做线性影响分析。",
            )
            .to_string(),
        });
    }

    if stat_result.table_overview.numeric_columns >= 1
        && (stat_result.table_overview.categorical_columns
            + stat_result.table_overview.boolean_columns)
            >= 1
    {
        suggestions.push(NextToolSuggestion {
            tool: "logistic_regression".to_string(),
            status: readiness_label(has_blocking).to_string(),
            reason: readiness_reason(
                has_blocking,
                "当前同时存在数值列和类别列，若能确认二分类目标列，可进入逻辑回归。",
            )
            .to_string(),
        });
    }

    if stat_result.table_overview.numeric_columns >= 1
        && stat_result.table_overview.categorical_columns >= 1
    {
        suggestions.push(NextToolSuggestion {
            tool: "group_and_aggregate".to_string(),
            status: readiness_label(false).to_string(),
            reason: "如果还不准备直接建模，可以先按维度分组汇总，验证业务统计差异。".to_string(),
        });
    }

    suggestions
}

// 2026-03-21: 这里集中生成人类摘要，目的是让用户直接看到“先做什么，再做什么”的决策顺序。
fn build_human_summary(
    analyze_result: &AnalyzeResult,
    priority_actions: &[PriorityAction],
    next_tool_suggestions: &[NextToolSuggestion],
) -> DecisionHumanSummary {
    let immediate_actions = priority_actions
        .iter()
        .take(3)
        .map(|action| action.title.clone())
        .collect::<Vec<_>>();
    let recommended_next_step = next_tool_suggestions
        .iter()
        .find(|suggestion| suggestion.status == "ready")
        .map(|suggestion| format!("建议优先尝试 `{}`。", suggestion.tool))
        .unwrap_or_else(|| "建议先解决高风险数据质量问题，再进入下一步分析建模。".to_string());

    DecisionHumanSummary {
        overall: if analyze_result.table_health.level == "risky" {
            "这张表当前存在需要优先处理的质量风险，建议先清洗再分析，不要直接进入建模。".to_string()
        } else {
            "这张表当前没有明显阻塞项，可以优先进入统计分析或建模探索。".to_string()
        },
        immediate_actions,
        recommended_next_step,
    }
}

// 2026-03-21: 这里集中维护阻塞型 finding 规则，目的是避免高层决策判断散落在多个函数中。
fn is_blocking_finding(finding: &AnalyzeFinding) -> bool {
    matches!(
        finding.code.as_str(),
        "all_missing"
            | "high_missing_rate"
            | "duplicate_rows"
            | "duplicate_candidate_key"
            | "blank_candidate_key"
    )
}

// 2026-03-21: 这里统一解释为什么算阻塞，目的是让用户不是只看到风险名称，而是知道它会卡住什么步骤。
fn blocking_reason(code: &str) -> &'static str {
    match code {
        "all_missing" => "整列没有有效值，这会直接降低后续统计和建模的可信度。",
        "high_missing_rate" => "缺失过多会导致有效样本大幅减少，后续结果容易失真。",
        "duplicate_rows" => "重复记录会放大样本权重，影响统计汇总、聚类和建模。",
        "duplicate_candidate_key" => "候选键重复会让关联和去重判断变得不稳定。",
        "blank_candidate_key" => "候选键为空会让关联、主键判断和样本定位失效。",
        _ => "该问题会影响后续分析结果的稳定性。",
    }
}

// 2026-03-21: 这里统一输出建议状态标签，目的是让上层一眼区分“现在能做”还是“先别做”。
fn readiness_label(has_blocking: bool) -> &'static str {
    if has_blocking { "wait" } else { "ready" }
}

// 2026-03-21: 这里统一拼接建议原因，目的是把阻塞提醒与动作价值合并成一句直白提示。
fn readiness_reason<'a>(has_blocking: bool, ready_reason: &'a str) -> std::borrow::Cow<'a, str> {
    if has_blocking {
        std::borrow::Cow::Borrowed("这一步有价值，但建议先处理阻塞风险后再做，避免结果失真。")
    } else {
        std::borrow::Cow::Borrowed(ready_reason)
    }
}
