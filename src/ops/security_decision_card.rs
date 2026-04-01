use serde::Serialize;

use crate::ops::stock::security_decision_evidence_bundle::SecurityDecisionEvidenceBundleResult;
use crate::ops::stock::security_risk_gates::{SecurityDecisionRiskProfile, SecurityRiskGateResult};

// 2026-04-01 CST: 这里定义投决立场摘要，原因是顶层 committee 需要把多头和空头的初判以结构化对象保留下来；
// 目的：让“独立立场”不再只是自由文本，而能被投决卡、Skill 和后续审阅层稳定消费。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SecurityDecisionThesis {
    pub thesis_label: String,
    pub headline: String,
    pub confidence: String,
    pub thesis_points: Vec<String>,
    pub invalidation_conditions: Vec<String>,
    pub cited_risks: Vec<String>,
}

// 2026-04-01 CST: 这里定义证券投决卡，原因是研究结论、正反方和闸门结果需要最终沉淀为一个统一对象；
// 目的：为后续审批、复核和用户输出提供单一裁决载体，而不是继续拼接多份分散结果。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SecurityDecisionCard {
    pub decision_id: String,
    pub symbol: String,
    pub analysis_date: String,
    pub status: String,
    pub direction: String,
    pub confidence_score: f64,
    pub expected_return_range: String,
    pub downside_risk: String,
    pub position_size_suggestion: String,
    pub required_next_actions: Vec<String>,
    pub final_recommendation: String,
}

// 2026-04-01 CST: 这里统一生成证券投决卡，原因是 committee 顶层应该专注组织流程，而不是内嵌裁决细节；
// 目的：把状态归类、仓位建议和最终话术收口到一个明确模块中。
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

    let direction = match bundle.integrated_conclusion.stance.as_str() {
        "negative" | "watchful_negative" | "bearish" => "avoid".to_string(),
        _ => "long".to_string(),
    };

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
        direction,
        confidence_score,
        expected_return_range,
        downside_risk,
        position_size_suggestion,
        required_next_actions,
        final_recommendation,
    }
}

// 2026-04-01 CST: 这里把多源信息压成一个简单置信分，原因是投决卡需要稳定的数值字段给后续 UI/审批使用；
// 目的：先提供可解释的 v1 分值，再为后续更复杂的打分模型预留位置。
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

// 2026-04-01 CST: 这里集中生成后续动作，原因是投决会输出需要告诉下一步该补什么而不是只给状态；
// 目的：让用户和后续 AI 在 blocked / needs_more_evidence 场景下有明确动作列表。
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

// 2026-04-01 CST: 这里集中生成最终裁决话术，原因是状态、仓位和风报比说明不应分散在顶层 Tool 里手写；
// 目的：保持 CLI、Skill 与后续 UI 输出的一致口径。
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
            risk_gates.iter().filter(|gate| gate.result == "warn").count()
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
