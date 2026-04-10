use serde::{Deserialize, Serialize};

use crate::ops::stock::security_decision_evidence_bundle::SecurityDecisionEvidenceBundleResult;
use crate::ops::stock::security_risk_gates::{SecurityDecisionRiskProfile, SecurityRiskGateResult};

// 2026-04-09 CST: 这里定义投决立场摘要，原因是 committee 需要保留多头与空头两条独立论证对象，
// 目的：让后续投委会、主席线和审批线可以稳定引用 thesis，而不是只剩下一段拼接文本。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityDecisionThesis {
    pub thesis_label: String,
    pub headline: String,
    pub confidence: String,
    pub thesis_points: Vec<String>,
    pub invalidation_conditions: Vec<String>,
    pub cited_risks: Vec<String>,
}

// 2026-04-09 CST: 这里定义七席委员会单席意见合同，原因是 Task 1 需要把委员会对象正式化而不是继续沿用旧 vote 汇总，
// 目的：为后续 chair / package / verify 提供稳定席位级输入对象。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityCommitteeMemberOpinion {
    pub member_id: String,
    pub seat_name: String,
    pub seat_kind: String,
    pub market_tilt_profile: String,
    pub vote: String,
    pub confidence: String,
    pub reasoning: String,
    pub supporting_points: Vec<String>,
    pub counter_points: Vec<String>,
    pub key_risks: Vec<String>,
    pub what_changes_my_mind: Vec<String>,
    pub execution_mode: String,
    pub execution_instance_id: String,
    pub process_id: u32,
    pub evidence_hash: String,
}

// 2026-04-09 CST: 这里定义委员会计票摘要，原因是正式 committee 需要沉淀多数票结构，
// 目的：让 chair 与后续治理链读取稳定票型，而不是重新遍历席位对象手工统计。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityCommitteeVoteTally {
    pub deliberation_seat_count: usize,
    pub risk_seat_count: usize,
    pub buy_count: usize,
    pub hold_count: usize,
    pub reduce_count: usize,
    pub avoid_count: usize,
    pub abstain_count: usize,
    pub majority_vote: String,
    pub majority_count: usize,
}

// 2026-04-09 CST: 这里定义风控席否决摘要，原因是 committee 最终结论不仅看多数票，还要看风控席状态，
// 目的：给 chair 与后续审批治理提供稳定的 veto 合同。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityCommitteeRiskVeto {
    pub seat_name: String,
    pub vote: String,
    pub status: String,
    pub reason: String,
}

// 2026-04-09 CST: 这里定义正式投决卡，原因是 committee 需要一个统一的裁决载体供 chair 和后续链路消费，
// 目的：把动作、暴露方向、置信度和后续动作沉淀成稳定对象。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityDecisionCard {
    pub decision_id: String,
    pub symbol: String,
    pub analysis_date: String,
    pub status: String,
    pub recommendation_action: String,
    pub exposure_side: String,
    pub direction: String,
    pub confidence_score: f64,
    pub expected_return_range: String,
    pub downside_risk: String,
    pub position_size_suggestion: String,
    pub required_next_actions: Vec<String>,
    pub final_recommendation: String,
}

// 2026-04-09 CST: 这里统一生成正式投决卡，原因是 committee 顶层应专注组织流程而不内嵌裁决细节，
// 目的：把初步动作、仓位建议和说明话术收口到单一 builder，后续 committee 再覆盖最终动作语义。
pub fn build_security_decision_card(
    bundle: &SecurityDecisionEvidenceBundleResult,
    bull_case: &SecurityDecisionThesis,
    bear_case: &SecurityDecisionThesis,
    risk_gates: &[SecurityRiskGateResult],
    risk_profile: &SecurityDecisionRiskProfile,
) -> SecurityDecisionCard {
    let has_blocking_fail = risk_gates
        .iter()
        .any(|gate| gate.blocking && gate.result == "fail");
    let has_warn = risk_gates.iter().any(|gate| gate.result == "warn");

    let status = if has_blocking_fail {
        "blocked".to_string()
    } else if bundle.evidence_quality.overall_status != "complete" || has_warn {
        "needs_more_evidence".to_string()
    } else {
        "ready_for_review".to_string()
    };

    let recommendation_action = derive_preliminary_recommendation_action(bundle).to_string();
    let exposure_side = derive_exposure_side_from_action(&recommendation_action).to_string();
    let direction = exposure_side.clone();

    let confidence_score = score_confidence(bundle, risk_gates);
    let position_size_suggestion = match status.as_str() {
        "blocked" => "none".to_string(),
        "needs_more_evidence" => "pilot".to_string(),
        _ => "starter".to_string(),
    };

    let required_next_actions = collect_next_actions(risk_gates, bull_case, bear_case);
    let expected_return_range = format!(
        "{:.1}% - {:.1}%",
        risk_profile.target_return_pct * 100.0,
        risk_profile.target_return_pct * 100.0 * 1.5
    );
    let downside_risk = format!("{:.1}%", risk_profile.stop_loss_pct * 100.0);
    let final_recommendation =
        build_final_recommendation(&status, &position_size_suggestion, risk_profile, risk_gates);

    SecurityDecisionCard {
        decision_id: format!("{}-{}", bundle.symbol, bundle.analysis_date),
        symbol: bundle.symbol.clone(),
        analysis_date: bundle.analysis_date.clone(),
        status,
        recommendation_action,
        exposure_side,
        direction,
        confidence_score,
        expected_return_range,
        downside_risk,
        position_size_suggestion,
        required_next_actions,
        final_recommendation,
    }
}

fn derive_preliminary_recommendation_action(
    bundle: &SecurityDecisionEvidenceBundleResult,
) -> &'static str {
    match bundle.integrated_conclusion.stance.as_str() {
        "negative" | "watchful_negative" | "bearish" => "avoid",
        "technical_only" | "neutral" => "hold",
        _ => "buy",
    }
}

// 2026-04-09 CST: 这里集中维护动作到暴露方向的映射，原因是 committee、scorecard、chair 都会依赖这一层方向语义，
// 目的：避免不同模块手写映射，导致 action 与 side 再次漂移。
pub fn derive_exposure_side_from_action(action: &str) -> &'static str {
    match action {
        "buy" | "hold" | "reduce" => "long",
        "short" => "short",
        "hedge" => "hedge",
        _ => "neutral",
    }
}

fn score_confidence(
    bundle: &SecurityDecisionEvidenceBundleResult,
    risk_gates: &[SecurityRiskGateResult],
) -> f64 {
    let mut score = match bundle.integrated_conclusion.stance.as_str() {
        "positive" => 0.78,
        "watchful_positive" => 0.66,
        "neutral" => 0.52,
        _ => 0.35,
    };

    if bundle.technical_context.contextual_conclusion.alignment == "tailwind" {
        score += 0.08;
    }
    if bundle.evidence_quality.overall_status != "complete" {
        score -= 0.08;
    }
    score -= risk_gates
        .iter()
        .filter(|gate| gate.result == "warn")
        .count() as f64
        * 0.05;
    if risk_gates
        .iter()
        .any(|gate| gate.blocking && gate.result == "fail")
    {
        score -= 0.15;
    }

    score.clamp(0.0, 0.95)
}

fn collect_next_actions(
    risk_gates: &[SecurityRiskGateResult],
    bull_case: &SecurityDecisionThesis,
    bear_case: &SecurityDecisionThesis,
) -> Vec<String> {
    let mut actions = Vec::new();
    for gate in risk_gates {
        if let Some(remediation) = gate.remediation.as_ref() {
            actions.push(remediation.clone());
        }
    }
    actions.push(format!(
        "继续跟踪多头失效条件：{}",
        bull_case.invalidation_conditions.join("；")
    ));
    actions.push(format!(
        "继续核对空头挑战点：{}",
        bear_case.thesis_points.join("；")
    ));
    dedupe_strings(&mut actions);
    actions
}

fn build_final_recommendation(
    status: &str,
    position_size_suggestion: &str,
    risk_profile: &SecurityDecisionRiskProfile,
    risk_gates: &[SecurityRiskGateResult],
) -> String {
    let ratio = if risk_profile.stop_loss_pct <= f64::EPSILON {
        0.0
    } else {
        risk_profile.target_return_pct / risk_profile.stop_loss_pct
    };
    match status {
        "blocked" => format!(
            "当前不建议进入执行建议，核心原因是风报比仅为 {:.2}，尚未达到投决会的最低要求。",
            ratio
        ),
        "needs_more_evidence" => format!(
            "当前仅建议以 {} 级别观察或试探，虽然风报比为 {:.2}，但仍有 {} 个闸门处于提醒状态。",
            position_size_suggestion,
            ratio,
            risk_gates
                .iter()
                .filter(|gate| gate.result == "warn")
                .count()
        ),
        _ => format!(
            "当前可进入审阅状态，建议以 {} 仓位方案启动，核心依据是风报比为 {:.2} 且主要闸门已通过。",
            position_size_suggestion, ratio
        ),
    }
}

fn dedupe_strings(values: &mut Vec<String>) {
    let mut deduped = Vec::new();
    for value in values.drain(..) {
        if !deduped.contains(&value) {
            deduped.push(value);
        }
    }
    *values = deduped;
}
