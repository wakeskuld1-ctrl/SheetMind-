mod common;

use serde_json::json;

use crate::common::run_cli_with_json;

#[test]
fn tool_catalog_includes_security_execution_record() {
    let output = run_cli_with_json("");

    // 2026-04-12 CST: Add a discovery red test for the formal execution-record tool,
    // because P8 needs lifecycle execution events to become first-class stock artifacts.
    // Purpose: lock catalog visibility before wiring the execution chain.
    assert!(
        output["data"]["tool_catalog"]
            .as_array()
            .expect("tool catalog should be an array")
            .iter()
            .any(|tool| tool == "security_execution_record")
    );
}

#[test]
fn security_execution_record_cli_returns_structured_result() {
    let request = json!({
        "tool": "security_execution_record",
        "args": {
            "symbol": "601916.SH",
            "analysis_date": "2026-04-10",
            "decision_ref": "decision:601916.SH:2026-04-10:v1",
            "approval_ref": "approval:601916.SH:2026-04-10:v1",
            "position_plan_ref": "position-plan:601916.SH:2026-04-10:v1",
            "condition_review_ref": "condition-review:601916.SH:2026-04-10:manual_review:v1",
            "execution_action": "build",
            "execution_status": "planned",
            "executed_gross_pct": 0.06,
            "execution_summary": "首仓 6%，等待盘中触发后执行",
            "created_at": "2026-04-12T10:30:00+08:00"
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-04-12 CST: Add a contract red test for structured execution records,
    // because P8 needs executable lifecycle events to be replayable and auditable.
    // Purpose: force the tool to emit a stable execution document with explicit bindings.
    assert_eq!(output["status"], "ok", "execution record output: {output}");
    assert_eq!(
        output["data"]["execution_record"]["document_type"],
        "security_execution_record"
    );
    assert_eq!(
        output["data"]["execution_record"]["execution_action"],
        "build"
    );
    assert_eq!(
        output["data"]["execution_record"]["binding"]["condition_review_ref"],
        "condition-review:601916.SH:2026-04-10:manual_review:v1"
    );
}
