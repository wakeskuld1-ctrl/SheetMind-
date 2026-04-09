use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::tools::contracts::{
    SecurityConditionReviewFollowUpAction, SecurityConditionReviewTriggerType,
};

// 2026-04-10 CST: 这里新增条件复核请求合同，原因是证券主链在没有实时数据前提下，
// 需要一个正式对象承接投中阶段的“条件是否仍成立”判断；目的：把 decision / approval / position plan / package 绑定到统一复核入口。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityConditionReviewRequest {
    pub symbol: String,
    pub analysis_date: String,
    pub decision_ref: String,
    pub approval_ref: String,
    pub position_plan_ref: String,
    pub package_path: String,
    pub review_trigger_type: SecurityConditionReviewTriggerType,
    pub review_trigger_summary: String,
    #[serde(default = "default_created_at")]
    pub created_at: String,
}

// 2026-04-10 CST: 这里新增条件复核正式文档，原因是投中阶段不能只靠对话临时解释“为什么继续执行或为什么重开会”；
// 目的：把触发原因、建议动作、复核发现和摘要固定为可落盘、可挂接、可审计的正式对象。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityConditionReviewDocument {
    pub condition_review_id: String,
    pub contract_version: String,
    pub generated_at: String,
    pub symbol: String,
    pub analysis_date: String,
    pub decision_ref: String,
    pub approval_ref: String,
    pub position_plan_ref: String,
    pub package_path: String,
    pub review_trigger_type: SecurityConditionReviewTriggerType,
    pub review_trigger_summary: String,
    pub recommended_follow_up_action: SecurityConditionReviewFollowUpAction,
    pub review_findings: Vec<String>,
    pub review_summary: String,
}

// 2026-04-10 CST: 这里保留最小结果包裹层，原因是后续 CLI / dispatcher / package 链更适合消费统一 result 外壳；
// 目的：避免后续直接返回 document 时再回头改外层合同。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityConditionReviewResult {
    pub condition_review: SecurityConditionReviewDocument,
}

#[derive(Debug, Error)]
pub enum SecurityConditionReviewError {
    #[error("security condition review build failed: {0}")]
    Build(String),
}

pub fn security_condition_review(
    request: &SecurityConditionReviewRequest,
) -> Result<SecurityConditionReviewResult, SecurityConditionReviewError> {
    validate_request(request)?;
    let follow_up_action = derive_follow_up_action(request);
    let review_findings = build_review_findings(request, &follow_up_action);
    let review_summary = build_review_summary(request, &follow_up_action);
    let trigger_label = trigger_label(&request.review_trigger_type);

    Ok(SecurityConditionReviewResult {
        condition_review: SecurityConditionReviewDocument {
            condition_review_id: format!(
                "condition-review:{}:{}:{}:v1",
                request.symbol, request.analysis_date, trigger_label
            ),
            contract_version: "security_condition_review.v1".to_string(),
            generated_at: normalize_created_at(&request.created_at),
            symbol: request.symbol.clone(),
            analysis_date: request.analysis_date.clone(),
            decision_ref: request.decision_ref.clone(),
            approval_ref: request.approval_ref.clone(),
            position_plan_ref: request.position_plan_ref.clone(),
            package_path: request.package_path.clone(),
            review_trigger_type: request.review_trigger_type.clone(),
            review_trigger_summary: request.review_trigger_summary.clone(),
            recommended_follow_up_action: follow_up_action,
            review_findings,
            review_summary,
        },
    })
}

// 2026-04-10 CST: 这里先做最小必填校验，原因是条件复核对象如果缺失决策引用，
// 后续无法挂回 package / execution / review 主链；目的：先把正式引用边界卡住。
fn validate_request(
    request: &SecurityConditionReviewRequest,
) -> Result<(), SecurityConditionReviewError> {
    for (field, value) in [
        ("symbol", request.symbol.trim()),
        ("analysis_date", request.analysis_date.trim()),
        ("decision_ref", request.decision_ref.trim()),
        ("approval_ref", request.approval_ref.trim()),
        ("position_plan_ref", request.position_plan_ref.trim()),
        ("package_path", request.package_path.trim()),
        ("review_trigger_summary", request.review_trigger_summary.trim()),
    ] {
        if value.is_empty() {
            return Err(SecurityConditionReviewError::Build(format!(
                "{field} cannot be empty"
            )));
        }
    }
    Ok(())
}

// 2026-04-10 CST: 这里先固化最小动作分流，原因是 Task 1 只要求先让 manual_review 有正式返回，
// 但 Task 2 立刻会扩展到四类触发模式；目的：现在就把分流逻辑集中到单点，避免下一步返工。
fn derive_follow_up_action(
    request: &SecurityConditionReviewRequest,
) -> SecurityConditionReviewFollowUpAction {
    let summary = request.review_trigger_summary.as_str();
    if contains_any(summary, &["冻结", "停牌", "止损", "重大负面"]) {
        return SecurityConditionReviewFollowUpAction::FreezeExecution;
    }

    match request.review_trigger_type {
        SecurityConditionReviewTriggerType::ManualReview => {
            SecurityConditionReviewFollowUpAction::KeepPlan
        }
        SecurityConditionReviewTriggerType::EndOfDayReview => {
            SecurityConditionReviewFollowUpAction::UpdatePositionPlan
        }
        SecurityConditionReviewTriggerType::EventReview => {
            SecurityConditionReviewFollowUpAction::ReopenCommittee
        }
        SecurityConditionReviewTriggerType::DataStalenessReview => {
            SecurityConditionReviewFollowUpAction::ReopenResearch
        }
    }
}

fn build_review_findings(
    request: &SecurityConditionReviewRequest,
    follow_up_action: &SecurityConditionReviewFollowUpAction,
) -> Vec<String> {
    vec![
        format!(
            "trigger_type={}",
            trigger_label(&request.review_trigger_type)
        ),
        format!("follow_up_action={}", follow_up_action_label(follow_up_action)),
        request.review_trigger_summary.clone(),
    ]
}

fn build_review_summary(
    request: &SecurityConditionReviewRequest,
    follow_up_action: &SecurityConditionReviewFollowUpAction,
) -> String {
    format!(
        "{} 在 {} 触发下完成条件复核，当前建议动作为 {}。",
        request.symbol,
        trigger_label(&request.review_trigger_type),
        follow_up_action_label(follow_up_action)
    )
}

fn trigger_label(trigger_type: &SecurityConditionReviewTriggerType) -> &'static str {
    match trigger_type {
        SecurityConditionReviewTriggerType::ManualReview => "manual_review",
        SecurityConditionReviewTriggerType::EndOfDayReview => "end_of_day_review",
        SecurityConditionReviewTriggerType::EventReview => "event_review",
        SecurityConditionReviewTriggerType::DataStalenessReview => "data_staleness_review",
    }
}

fn follow_up_action_label(action: &SecurityConditionReviewFollowUpAction) -> &'static str {
    match action {
        SecurityConditionReviewFollowUpAction::KeepPlan => "keep_plan",
        SecurityConditionReviewFollowUpAction::UpdatePositionPlan => "update_position_plan",
        SecurityConditionReviewFollowUpAction::ReopenResearch => "reopen_research",
        SecurityConditionReviewFollowUpAction::ReopenCommittee => "reopen_committee",
        SecurityConditionReviewFollowUpAction::FreezeExecution => "freeze_execution",
    }
}

fn contains_any(summary: &str, keywords: &[&str]) -> bool {
    keywords.iter().any(|keyword| summary.contains(keyword))
}

fn default_created_at() -> String {
    Utc::now().to_rfc3339()
}

fn normalize_created_at(created_at: &str) -> String {
    let trimmed = created_at.trim();
    if trimmed.is_empty() {
        default_created_at()
    } else {
        trimmed.to_string()
    }
}
