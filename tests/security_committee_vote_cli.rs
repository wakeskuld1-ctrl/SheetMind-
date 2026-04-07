mod common;

use excel_skill::ops::stock::security_committee_vote::{
    SecurityCommitteeVoteRequest, security_committee_vote,
};
use excel_skill::ops::stock::security_decision_briefing::{
    CommitteeEvidenceChecks, CommitteeExecutionDigest, CommitteeHistoricalDigest, CommitteePayload,
    CommitteeRecommendationDigest, CommitteeResonanceDigest, ExecutionPlan,
};
use serde_json::json;
use std::collections::HashSet;

use crate::common::run_cli_with_json;

// 2026-04-02 CST: 这里集中构造投决会测试用 committee payload，原因是 vote Tool 的核心边界是“只能消费统一事实包”，
// 目的：让后续输入校验、角色投票、聚合规则都能基于同一份结构化 payload 演进，而不是每条测试各自手写不同形状的 JSON。
fn build_committee_payload() -> CommitteePayload {
    CommitteePayload {
        symbol: "601857.SH".to_string(),
        analysis_date: "2025-08-08".to_string(),
        recommended_action: "add_on_strength".to_string(),
        confidence: "medium".to_string(),
        key_risks: vec![
            "财报关键同比指标不完整".to_string(),
            "阻力突破后仍需量能确认".to_string(),
        ],
        minority_objection_points: vec!["油价共振存在短线回撤扰动".to_string()],
        evidence_version: "security-decision-briefing:601857.SH:2025-08-08:v1".to_string(),
        briefing_digest: "顺风结构延续，但仍需确认突破质量".to_string(),
        committee_schema_version: "committee-payload:v1".to_string(),
        recommendation_digest: CommitteeRecommendationDigest {
            final_stance: "watchful_positive".to_string(),
            action_bias: "add_on_strength".to_string(),
            summary: "趋势与共振偏多，但执行上仍需确认量价配合".to_string(),
            confidence: "medium".to_string(),
        },
        execution_digest: CommitteeExecutionDigest {
            add_trigger_price: 12.36,
            add_trigger_volume_ratio: 1.24,
            add_position_pct: 0.12,
            reduce_trigger_price: 11.42,
            reduce_position_pct: 0.08,
            stop_loss_price: 10.98,
            invalidation_price: 10.55,
            rejection_zone: "12.36-12.74".to_string(),
            watch_points: vec![
                "若量比不足则不追价".to_string(),
                "跌破短承接位先减仓".to_string(),
            ],
            explanation: vec!["执行阈值来自阻力位、均线与布林中轨".to_string()],
        },
        resonance_digest: CommitteeResonanceDigest {
            resonance_score: 0.81,
            action_bias: "add_on_strength".to_string(),
            top_positive_driver_names: vec!["布伦特原油".to_string(), "石油石化板块".to_string()],
            top_negative_driver_names: vec!["库存扰动".to_string()],
            event_override_titles: vec!["分红预案公告".to_string()],
        },
        evidence_checks: CommitteeEvidenceChecks {
            fundamental_ready: true,
            technical_ready: true,
            resonance_ready: true,
            execution_ready: true,
            briefing_ready: true,
        },
        historical_digest: CommitteeHistoricalDigest {
            status: "unavailable".to_string(),
            historical_confidence: "unknown".to_string(),
            analog_sample_count: 0,
            analog_win_rate_10d: None,
            expected_return_window: None,
            expected_drawdown_window: None,
            research_limitations: vec!["历史研究层尚未接入 committee payload".to_string()],
        },
    }
}

// 2026-04-02 CST: 这里专门保留 execution plan 构造器，原因是 committee payload 目前同时保留旧字段与新子层，
// 目的：后续如果 briefing 与 vote 共用 execution 结构，可以在不改测试意图的前提下统一替换实现。
#[allow(dead_code)]
fn build_execution_plan() -> ExecutionPlan {
    ExecutionPlan {
        add_trigger_price: 12.36,
        add_trigger_volume_ratio: 1.24,
        add_position_pct: 0.12,
        reduce_trigger_price: 11.42,
        rejection_zone: "12.36-12.74".to_string(),
        reduce_position_pct: 0.08,
        stop_loss_price: 10.98,
        invalidation_price: 10.55,
        watch_points: vec![
            "若量比不足则不追价".to_string(),
            "跌破短承接位先减仓".to_string(),
        ],
        explanation: vec!["执行阈值来自阻力位、均线与布林中轨".to_string()],
    }
}

#[test]
fn security_committee_vote_is_cataloged() {
    let output = run_cli_with_json(r#"{"tool":"tool_catalog","args":{}}"#);
    let tool_catalog = output["data"]["tool_catalog"]
        .as_array()
        .expect("tool_catalog should be an array");
    assert!(
        tool_catalog
            .iter()
            .filter_map(|item| item.as_str())
            .any(|item| item == "security_committee_vote")
    );
    let stock_catalog = output["data"]["tool_catalog_modules"]["stock"]
        .as_array()
        .expect("stock tool group should be an array");
    assert!(
        stock_catalog
            .iter()
            .filter_map(|item| item.as_str())
            .any(|item| item == "security_committee_vote")
    );
}

#[test]
fn security_committee_vote_rejects_invalid_payload() {
    let mut payload = build_committee_payload();
    payload.evidence_version.clear();

    let error = security_committee_vote(&SecurityCommitteeVoteRequest {
        committee_payload: payload,
        committee_mode: "standard".to_string(),
        meeting_id: None,
    })
    .expect_err("missing evidence_version should be rejected");
    assert!(
        error.to_string().contains("evidence_version"),
        "error should mention evidence_version, got: {error}"
    );
}

#[test]
fn security_committee_vote_emits_fixed_member_votes() {
    let result = security_committee_vote(&SecurityCommitteeVoteRequest {
        committee_payload: build_committee_payload(),
        committee_mode: "standard".to_string(),
        meeting_id: Some("meeting-001".to_string()),
    })
    .expect("valid payload should vote successfully");

    assert_eq!(result.votes.len(), 7);
    for role in [
        "chair",
        "fundamental_reviewer",
        "technical_reviewer",
        "event_reviewer",
        "valuation_reviewer",
        "risk_officer",
        "execution_reviewer",
    ] {
        let vote = result
            .votes
            .iter()
            .find(|item| item.role == role)
            .expect("expected fixed committee role to exist");
        assert!(!vote.vote.is_empty());
        assert!(!vote.confidence.is_empty());
        assert!(!vote.rationale.is_empty());
        assert!(!vote.member_id.is_empty());
        assert!(!vote.execution_instance_id.is_empty());
    }
}

#[test]
fn security_committee_vote_applies_aggregation_rules() {
    let result = security_committee_vote(&SecurityCommitteeVoteRequest {
        committee_payload: build_committee_payload(),
        committee_mode: "standard".to_string(),
        meeting_id: None,
    })
    .expect("valid payload should aggregate successfully");

    assert!(result.quorum_met);
    assert!(result.approval_ratio >= 0.0);
    assert!(
        [
            "approved",
            "approved_with_conditions",
            "deferred",
            "rejected"
        ]
        .contains(&result.final_decision.as_str())
    );
}

#[test]
fn security_committee_vote_surfaces_conditions_and_disagreements() {
    let mut payload = build_committee_payload();
    payload.historical_digest.research_limitations =
        vec!["历史样本不足，需降低结论确信度".to_string()];

    let result = security_committee_vote(&SecurityCommitteeVoteRequest {
        committee_payload: payload,
        committee_mode: "standard".to_string(),
        meeting_id: None,
    })
    .expect("valid payload should aggregate successfully");

    assert!(
        !result.warnings.is_empty(),
        "warnings should expose historical limitations"
    );
    assert!(
        result.conditions.iter().all(|item| !item.trim().is_empty()),
        "conditions should not contain empty text"
    );
    assert!(
        result
            .key_disagreements
            .iter()
            .all(|item| !item.trim().is_empty()),
        "key_disagreements should not contain empty text"
    );
}

#[test]
fn security_committee_vote_cli_returns_structured_result() {
    let payload = serde_json::to_value(build_committee_payload())
        .expect("committee payload should serialize");
    let request = json!({
        "tool": "security_committee_vote",
        "args": {
            "committee_payload": payload,
            "committee_mode": "standard",
            "meeting_id": "meeting-cli-001"
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    for field_name in [
        "symbol",
        "analysis_date",
        "evidence_version",
        "committee_mode",
        "final_decision",
        "final_action",
        "final_confidence",
        "approval_ratio",
        "veto_triggered",
        "votes",
        "conditions",
        "key_disagreements",
        "warnings",
    ] {
        assert!(
            output["data"].get(field_name).is_some(),
            "CLI vote result should expose `{field_name}`"
        );
    }
}

#[test]
fn security_committee_vote_exposes_seven_seat_independent_execution() {
    let payload = serde_json::to_value(build_committee_payload())
        .expect("committee payload should serialize");
    let request = json!({
        "tool": "security_committee_vote",
        "args": {
            "committee_payload": payload,
            "committee_mode": "standard",
            "meeting_id": "meeting-cli-seven-seat-001"
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["committee_engine"],
        "seven_seat_committee_v3"
    );
    assert_eq!(output["data"]["deliberation_seat_count"], 6);
    assert_eq!(output["data"]["risk_seat_count"], 1);

    let votes = output["data"]["votes"]
        .as_array()
        .expect("votes should be an array");
    assert_eq!(votes.len(), 7);

    let mut evidence_versions = HashSet::new();
    let mut execution_instance_ids = HashSet::new();
    let mut process_ids = HashSet::new();
    let mut risk_seat_count = 0_usize;

    for vote in votes {
        assert!(vote["member_id"].is_string());
        assert!(vote["seat_kind"].is_string());
        assert_eq!(vote["execution_mode"], "child_process");
        assert!(vote["vote"].is_string());
        assert!(vote["rationale"].is_string());

        evidence_versions.insert(
            vote["evidence_version"]
                .as_str()
                .expect("evidence_version should exist")
                .to_string(),
        );
        execution_instance_ids.insert(
            vote["execution_instance_id"]
                .as_str()
                .expect("execution_instance_id should exist")
                .to_string(),
        );
        process_ids.insert(
            vote["process_id"]
                .as_u64()
                .expect("process_id should exist"),
        );

        if vote["seat_kind"] == "risk_control" {
            risk_seat_count += 1;
        }
    }

    assert_eq!(risk_seat_count, 1);
    assert_eq!(evidence_versions.len(), 1);
    assert_eq!(execution_instance_ids.len(), 7);
    assert_eq!(process_ids.len(), 7);
}
