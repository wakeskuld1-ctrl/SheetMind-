mod common;

use excel_skill::ops::stock::security_condition_review::{
    SecurityConditionReviewRequest, security_condition_review,
};
use excel_skill::tools::contracts::{
    SecurityConditionReviewFollowUpAction, SecurityConditionReviewTriggerType,
};
use serde_json::json;

use crate::common::run_cli_with_json;

// 2026-04-10 CST: 这里补 tool_catalog 可发现性红测，原因是 Task 3 的目标是把 security_condition_review 升级成正式 CLI Tool；
// 目的：先锁住“目录能发现”这层对外合同，避免后续只接了内部实现却没有暴露给 CLI / Skill 主链。
#[test]
fn security_condition_review_is_cataloged() {
    let output = run_cli_with_json(r#"{"tool":"tool_catalog","args":{}}"#);
    let tool_catalog = output["data"]["tool_catalog"]
        .as_array()
        .expect("tool_catalog should be an array");
    assert!(
        tool_catalog
            .iter()
            .filter_map(|item| item.as_str())
            .any(|item| item == "security_condition_review")
    );

    let stock_catalog = output["data"]["tool_catalog_modules"]["stock"]
        .as_array()
        .expect("stock tool group should be an array");
    assert!(
        stock_catalog
            .iter()
            .filter_map(|item| item.as_str())
            .any(|item| item == "security_condition_review")
    );
}

// 2026-04-10 CST: 这里补 CLI 路由红测，原因是 Task 3 不只是 catalog 注册，还必须证明 JSON 请求能走到正式 dispatcher；
// 目的：锁住“请求解析 -> stock dispatcher -> condition review 结果输出”的最小主链，后续重构时也能防止路由漂移。
#[test]
fn security_condition_review_cli_returns_structured_result() {
    let request = json!({
        "tool": "security_condition_review",
        "args": {
            "symbol": "601916.SH",
            "analysis_date": "2026-04-08",
            "decision_ref": "decision:601916.SH:2026-04-08:v1",
            "approval_ref": "approval:601916.SH:2026-04-08:v1",
            "position_plan_ref": "position-plan:601916.SH:2026-04-08:v1",
            "package_path": "artifacts/decision_packages/601916.SH-2026-04-08.json",
            "review_trigger_type": "manual_review",
            "review_trigger_summary": "manual intraday review for existing position plan",
            "created_at": "2026-04-10T09:30:00Z"
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["condition_review"]["recommended_follow_up_action"],
        "keep_plan"
    );
    assert_eq!(
        output["data"]["condition_review"]["condition_review_id"],
        "condition-review:601916.SH:2026-04-08:manual_review:v1"
    );
}

#[test]
fn security_condition_review_manual_review_contract() {
    let result = security_condition_review(&SecurityConditionReviewRequest {
        symbol: "601916.SH".to_string(),
        analysis_date: "2026-04-08".to_string(),
        decision_ref: "decision:601916.SH:2026-04-08:v1".to_string(),
        approval_ref: "approval:601916.SH:2026-04-08:v1".to_string(),
        position_plan_ref: "position-plan:601916.SH:2026-04-08:v1".to_string(),
        package_path: "artifacts/decision_packages/601916.SH-2026-04-08.json".to_string(),
        review_trigger_type: SecurityConditionReviewTriggerType::ManualReview,
        review_trigger_summary: "盘中人工复核：检查原有持仓计划是否仍成立".to_string(),
        created_at: "2026-04-10T09:30:00Z".to_string(),
    })
    .expect("manual review should produce a condition review result");

    assert_eq!(result.condition_review.symbol, "601916.SH");
    assert_eq!(result.condition_review.analysis_date, "2026-04-08");
    assert_eq!(
        result.condition_review.decision_ref,
        "decision:601916.SH:2026-04-08:v1"
    );
    assert_eq!(
        result.condition_review.approval_ref,
        "approval:601916.SH:2026-04-08:v1"
    );
    assert_eq!(
        result.condition_review.position_plan_ref,
        "position-plan:601916.SH:2026-04-08:v1"
    );
    assert_eq!(
        result.condition_review.recommended_follow_up_action,
        SecurityConditionReviewFollowUpAction::KeepPlan
    );
    assert!(
        !result.condition_review.review_summary.trim().is_empty(),
        "review_summary should not be empty"
    );
}

#[test]
fn security_condition_review_end_of_day_review_updates_position_plan() {
    let result = security_condition_review(&SecurityConditionReviewRequest {
        symbol: "601916.SH".to_string(),
        analysis_date: "2026-04-08".to_string(),
        decision_ref: "decision:601916.SH:2026-04-08:v1".to_string(),
        approval_ref: "approval:601916.SH:2026-04-08:v1".to_string(),
        position_plan_ref: "position-plan:601916.SH:2026-04-08:v1".to_string(),
        package_path: "artifacts/decision_packages/601916.SH-2026-04-08.json".to_string(),
        review_trigger_type: SecurityConditionReviewTriggerType::EndOfDayReview,
        review_trigger_summary: "收盘后复核：加仓条件需要按最新收盘结构微调".to_string(),
        created_at: "2026-04-10T15:05:00Z".to_string(),
    })
    .expect("end_of_day_review should produce a condition review result");

    assert_eq!(
        result.condition_review.recommended_follow_up_action,
        SecurityConditionReviewFollowUpAction::UpdatePositionPlan
    );
    assert!(
        result
            .condition_review
            .review_summary
            .contains("end_of_day_review"),
        "review_summary should mention end_of_day_review"
    );
}

#[test]
fn security_condition_review_event_review_reopens_committee() {
    let result = security_condition_review(&SecurityConditionReviewRequest {
        symbol: "159866.SZ".to_string(),
        analysis_date: "2026-04-08".to_string(),
        decision_ref: "decision:159866.SZ:2026-04-08:v1".to_string(),
        approval_ref: "approval:159866.SZ:2026-04-08:v1".to_string(),
        position_plan_ref: "position-plan:159866.SZ:2026-04-08:v1".to_string(),
        package_path: "artifacts/decision_packages/159866.SZ-2026-04-08.json".to_string(),
        review_trigger_type: SecurityConditionReviewTriggerType::EventReview,
        review_trigger_summary: "事件复核：日本市场政策预期发生变化，需要重新评估投决口径".to_string(),
        created_at: "2026-04-10T11:30:00Z".to_string(),
    })
    .expect("event_review should produce a condition review result");

    assert_eq!(
        result.condition_review.recommended_follow_up_action,
        SecurityConditionReviewFollowUpAction::ReopenCommittee
    );
}

#[test]
fn security_condition_review_event_review_can_freeze_execution() {
    let result = security_condition_review(&SecurityConditionReviewRequest {
        symbol: "159866.SZ".to_string(),
        analysis_date: "2026-04-08".to_string(),
        decision_ref: "decision:159866.SZ:2026-04-08:v1".to_string(),
        approval_ref: "approval:159866.SZ:2026-04-08:v1".to_string(),
        position_plan_ref: "position-plan:159866.SZ:2026-04-08:v1".to_string(),
        package_path: "artifacts/decision_packages/159866.SZ-2026-04-08.json".to_string(),
        review_trigger_type: SecurityConditionReviewTriggerType::EventReview,
        review_trigger_summary: "事件复核：基金临时停牌，先冻结执行并等待进一步公告".to_string(),
        created_at: "2026-04-10T11:45:00Z".to_string(),
    })
    .expect("freeze event review should produce a condition review result");

    assert_eq!(
        result.condition_review.recommended_follow_up_action,
        SecurityConditionReviewFollowUpAction::FreezeExecution
    );
    assert!(
        result
            .condition_review
            .review_findings
            .iter()
            .any(|item| item.contains("freeze_execution")),
        "review_findings should carry freeze_execution label"
    );
}

#[test]
fn security_condition_review_data_staleness_review_reopens_research() {
    let result = security_condition_review(&SecurityConditionReviewRequest {
        symbol: "601916.SH".to_string(),
        analysis_date: "2026-04-08".to_string(),
        decision_ref: "decision:601916.SH:2026-04-08:v1".to_string(),
        approval_ref: "approval:601916.SH:2026-04-08:v1".to_string(),
        position_plan_ref: "position-plan:601916.SH:2026-04-08:v1".to_string(),
        package_path: "artifacts/decision_packages/601916.SH-2026-04-08.json".to_string(),
        review_trigger_type: SecurityConditionReviewTriggerType::DataStalenessReview,
        review_trigger_summary: "数据过期复核：原有公告与财报快照已过期，需要先补研究证据".to_string(),
        created_at: "2026-04-10T16:00:00Z".to_string(),
    })
    .expect("data_staleness_review should produce a condition review result");

    assert_eq!(
        result.condition_review.recommended_follow_up_action,
        SecurityConditionReviewFollowUpAction::ReopenResearch
    );
}
