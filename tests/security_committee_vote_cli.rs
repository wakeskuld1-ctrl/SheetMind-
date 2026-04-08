mod common;

use excel_skill::ops::stock::security_committee_vote::{
    SecurityCommitteeVoteRequest, security_committee_vote,
};
use excel_skill::ops::stock::security_decision_briefing::{
    CommitteeEvidenceChecks, CommitteeExecutionDigest, CommitteeHistoricalDigest, CommitteePayload,
    CommitteeRecommendationDigest, CommitteeResonanceDigest, CommitteeRiskBreakdown,
    CommitteeRiskItem, CommitteeSubjectProfile, ExecutionPlan, OddsBrief, PositionPlan,
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
        // 2026-04-08 CST: 这里默认补齐个股 subject_profile，原因是 CommitteePayload 已升级为统一承载 ETF/个股主链解释。
        // 目的：让测试夹具和正式合同保持一致，并为后续 ETF 夹具复用留下稳定入口。
        subject_profile: CommitteeSubjectProfile::default(),
        // 2026-04-08 CST: 这里补齐结构化风险合同，原因是 CommitteePayload 已升级为 risk_breakdown 主合同；
        // 目的：让 committee_vote 测试夹具与 briefing 正式输出保持同一风险边界，并让 key_risks 退化为兼容摘要。
        risk_breakdown: CommitteeRiskBreakdown {
            technical: vec![CommitteeRiskItem {
                category: "technical".to_string(),
                severity: "medium".to_string(),
                headline: "阻力突破后仍需量能确认".to_string(),
                rationale: "当前趋势转强，但若量比不足，突破有效性仍需二次确认。".to_string(),
            }],
            fundamental: vec![CommitteeRiskItem {
                category: "fundamental".to_string(),
                severity: "medium".to_string(),
                headline: "财报关键同比指标不完整".to_string(),
                rationale: "基本面席位仍需补齐核心财务口径，避免在信息缺口下放大仓位。".to_string(),
            }],
            resonance: vec![CommitteeRiskItem {
                category: "resonance".to_string(),
                severity: "medium".to_string(),
                headline: "油价共振存在短线回撤扰动".to_string(),
                rationale: "共振驱动虽偏多，但上游商品扰动仍可能带来短线波动。".to_string(),
            }],
            execution: vec![CommitteeRiskItem {
                category: "execution".to_string(),
                severity: "low".to_string(),
                headline: "量比不足时不追价".to_string(),
                rationale: "执行层需要严格遵守加仓与减仓阈值，避免信号失真时扩仓。".to_string(),
            }],
        },
        // 2026-04-08 CST: 这里同步 key_risks 到 risk_breakdown 的固定派生口径，原因是方案 B 已要求旧摘要字段必须与主合同严格一致；
        // 目的：让“合法 payload”夹具本身成为新门禁的正样本，避免测试继续依赖过时的手工摘要顺序。
        key_risks: vec![
            "阻力突破后仍需量能确认".to_string(),
            "财报关键同比指标不完整".to_string(),
            "油价共振存在短线回撤扰动".to_string(),
            "量比不足时不追价".to_string(),
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
            analog_loss_rate_10d: None,
            analog_flat_rate_10d: None,
            analog_avg_return_10d: None,
            analog_median_return_10d: None,
            analog_avg_win_return_10d: None,
            analog_avg_loss_return_10d: None,
            analog_payoff_ratio_10d: None,
            analog_expectancy_10d: None,
            expected_return_window: None,
            expected_drawdown_window: None,
            research_limitations: vec!["历史研究层尚未接入 committee payload".to_string()],
        },
        // 2026-04-08 CST: 这里补齐赔率摘要夹具，原因是 committee_payload 已开始承载 briefing 同源的赔率层；
        // 目的：让 vote Tool 的输入夹具与新 briefing 合同保持一致，同时保留历史研究 unavailable 时的最小默认边界。
        odds_digest: OddsBrief {
            status: "unavailable".to_string(),
            historical_confidence: "unknown".to_string(),
            sample_count: 0,
            win_rate_10d: None,
            loss_rate_10d: None,
            flat_rate_10d: None,
            avg_return_10d: None,
            median_return_10d: None,
            avg_win_return_10d: None,
            avg_loss_return_10d: None,
            payoff_ratio_10d: None,
            expectancy_10d: None,
            expected_return_window: None,
            expected_drawdown_window: None,
            odds_grade: "pending_research".to_string(),
            confidence_grade: "unknown".to_string(),
            rationale: vec!["历史研究未就绪时，赔率层仅允许输出等待或观察仓语义。".to_string()],
            research_limitations: vec!["历史研究层尚未接入 committee payload".to_string()],
        },
        // 2026-04-08 CST: 这里补齐仓位摘要夹具，原因是 committee_payload 已开始承载 briefing 同源的仓位层；
        // 目的：让 vote Tool 在不改投票规则的前提下，也能消费完整的新合同事实包。
        position_digest: PositionPlan {
            position_action: "pilot_only".to_string(),
            entry_mode: "breakout_confirmation".to_string(),
            starter_position_pct: 0.04,
            max_position_pct: 0.08,
            add_on_trigger: "站上 12.36 且量比达到 1.24 后再追加。".to_string(),
            reduce_on_trigger: "跌破 11.42 后先减仓观察。".to_string(),
            hard_stop_trigger: "跌破 10.98 或 10.55 时结束当前交易假设。".to_string(),
            liquidity_cap: "单次执行不超过计划仓位的 50%".to_string(),
            position_risk_grade: "high".to_string(),
            regime_adjustment: "历史研究 unavailable 时，默认按观察仓执行。".to_string(),
            execution_notes: vec!["执行上仍需等待放量确认，不宜一次性打满。".to_string()],
            rationale: vec!["仓位层在历史研究缺失时应主动降档，不放大主观判断。".to_string()],
        },
    }
}

fn build_etf_committee_payload() -> CommitteePayload {
    let mut payload = build_committee_payload();
    // 2026-04-08 CST: 这里构造 ETF 夹具，原因是本轮要锁住“ETF 不再因缺个股财报而直接 defer”的回归边界。
    // 目的：用最小测试覆盖新的 ETF 基本面席位语义，避免后续主链回退到旧个股规则。
    payload.symbol = "159866.SZ".to_string();
    payload.subject_profile = CommitteeSubjectProfile {
        asset_class: "etf".to_string(),
        market_scope: "china".to_string(),
        committee_focus: "fund_review".to_string(),
    };
    payload.evidence_checks.fundamental_ready = false;
    // 2026-04-08 CST: 这里同步 ETF 夹具的兼容摘要，原因是 fundamental 风险 headline 已被 ETF 专用语义覆盖；
    // 目的：确保 ETF 合法夹具也满足“key_risks 严格来自 risk_breakdown 派生”的新合同。
    payload.key_risks = vec![
        "阻力突破后仍需量能确认".to_string(),
        "ETF 当前缺少跟踪误差、底层指数与申赎结构的专用研究。".to_string(),
        "油价共振存在短线回撤扰动".to_string(),
        "量比不足时不追价".to_string(),
    ];
    payload.risk_breakdown.fundamental = vec![CommitteeRiskItem {
        category: "fundamental".to_string(),
        severity: "medium".to_string(),
        headline: "ETF 当前缺少跟踪误差、底层指数与申赎结构的专用研究。".to_string(),
        rationale: "ETF 投决重点不在单一公司财报，而在跟踪质量、指数结构、流动性与申赎机制；当前主链先把这类缺口显式暴露为专项研究待补。"
            .to_string(),
    }];
    payload.recommendation_digest.summary =
        "日经 ETF 仍在区间内震荡，倾向先观察跟踪质量与量价确认。".to_string();
    payload
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
fn security_committee_vote_rejects_risk_breakdown_category_mismatch() {
    let mut payload = build_committee_payload();
    // 2026-04-08 CST: 这里故意制造分类错桶样例，原因是方案 B 要把 risk_breakdown 提升为强一致性门禁主源。
    // 目的：锁住“technical 桶里不能塞 fundamental 分类”这类结构化风险合同错误，避免脏 payload 混进投决会。
    payload.risk_breakdown.technical[0].category = "fundamental".to_string();

    let error = security_committee_vote(&SecurityCommitteeVoteRequest {
        committee_payload: payload,
        committee_mode: "standard".to_string(),
        meeting_id: None,
    })
    .expect_err("category mismatch should be rejected");
    assert!(
        error.to_string().contains("risk_breakdown"),
        "error should mention risk_breakdown, got: {error}"
    );
}

#[test]
fn security_committee_vote_rejects_key_risks_not_matching_risk_breakdown() {
    let mut payload = build_committee_payload();
    // 2026-04-08 CST: 这里故意制造 key_risks 与 risk_breakdown 不一致，原因是方案 B 要求旧摘要字段必须严格来自主合同派生。
    // 目的：防止外部调用方继续手工维护第二套风险事实，导致摘要与结构化风险并行漂移。
    payload.key_risks = vec!["外部手工拼接的风险摘要".to_string()];

    let error = security_committee_vote(&SecurityCommitteeVoteRequest {
        committee_payload: payload,
        committee_mode: "standard".to_string(),
        meeting_id: None,
    })
    .expect_err("key_risks mismatch should be rejected");
    assert!(
        error.to_string().contains("key_risks"),
        "error should mention key_risks, got: {error}"
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

#[test]
fn security_committee_vote_etf_fundamental_reviewer_uses_fund_review_semantics() {
    let result = security_committee_vote(&SecurityCommitteeVoteRequest {
        committee_payload: build_etf_committee_payload(),
        committee_mode: "standard".to_string(),
        meeting_id: Some("meeting-etf-001".to_string()),
    })
    .expect("ETF payload should vote successfully");

    let vote = result
        .votes
        .iter()
        .find(|item| item.role == "fundamental_reviewer")
        .expect("fundamental_reviewer should exist");

    assert_ne!(vote.vote, "defer");
    assert!(
        vote.rationale.contains("ETF"),
        "ETF rationale should use fund-review semantics, got: {}",
        vote.rationale
    );
}
