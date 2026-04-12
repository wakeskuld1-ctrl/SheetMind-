mod common;

use serde_json::json;

use crate::common::run_cli_with_json;

#[test]
fn tool_catalog_includes_security_post_trade_review() {
    let output = run_cli_with_json("");

    // 2026-04-12 CST: Add a discovery red test for the formal post-trade review tool,
    // because P8 needs replayable review artifacts rather than free-form closeout notes.
    // Purpose: lock catalog visibility before wiring layered review attribution.
    assert!(
        output["data"]["tool_catalog"]
            .as_array()
            .expect("tool catalog should be an array")
            .iter()
            .any(|tool| tool == "security_post_trade_review")
    );
}

#[test]
fn security_post_trade_review_cli_returns_structured_result() {
    let request = json!({
        "tool": "security_post_trade_review",
        "args": {
            "symbol": "601916.SH",
            "analysis_date": "2026-04-10",
            "decision_ref": "decision:601916.SH:2026-04-10:v1",
            "approval_ref": "approval:601916.SH:2026-04-10:v1",
            "position_plan_ref": "position-plan:601916.SH:2026-04-10:v1",
            "execution_record_ref": "execution-record:601916.SH:2026-04-10:build:v1",
            "review_status": "interim_review",
            "review_summary": "建仓后走势弱于预期，需要复盘原因并调整模型消费等级",
            "attribution": {
                "data_issue": false,
                "model_issue": true,
                "governance_issue": false,
                "execution_issue": false
            },
            "recommended_governance_action": "continue_shadow",
            "created_at": "2026-04-12T11:00:00+08:00"
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-04-12 CST: Add a contract red test for structured post-trade review,
    // because P8 needs layered attribution and formal governance follow-up instead
    // of leaving review semantics in prose only.
    // Purpose: force the tool to emit a stable review document with attribution fields.
    assert_eq!(output["status"], "ok", "post trade review output: {output}");
    assert_eq!(
        output["data"]["post_trade_review"]["document_type"],
        "security_post_trade_review"
    );
    assert_eq!(
        output["data"]["post_trade_review"]["attribution"]["model_issue"],
        true
    );
    assert_eq!(
        output["data"]["post_trade_review"]["recommended_governance_action"],
        "continue_shadow"
    );
    assert_eq!(
        output["data"]["post_trade_review"]["binding"]["execution_record_ref"],
        "execution-record:601916.SH:2026-04-10:build:v1"
    );
}
