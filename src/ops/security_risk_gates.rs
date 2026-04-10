use serde::Serialize;

use crate::ops::stock::security_decision_evidence_bundle::SecurityDecisionEvidenceBundleResult;

// 2026-04-09 CST: 这里定义投决层风险参数，原因是 committee 需要把用户给定的止损和目标收益收口成统一风控输入，
// 目的：把风险约束从顶层 Tool 剥离出来，后续 chair / approval / replay 都能复用同一套口径。
#[derive(Debug, Clone, PartialEq)]
pub struct SecurityDecisionRiskProfile {
    pub stop_loss_pct: f64,
    pub target_return_pct: f64,
    pub min_risk_reward_ratio: f64,
}

// 2026-04-09 CST: 这里定义单个风险闸门结果，原因是 committee 与后续治理层都需要消费一致 Gate 合同，
// 目的：把日期、完整度、环境、事件和风报比这些裁决前检查沉淀成稳定结构。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SecurityRiskGateResult {
    pub gate_name: String,
    pub result: String,
    pub blocking: bool,
    pub reason: String,
    pub metric_snapshot: Vec<String>,
    pub remediation: Option<String>,
}

// 2026-04-09 CST: 这里集中评估证券投决闸门，原因是“能研究”和“能执行”必须在这里被明确拆开，
// 目的：为 committee 的风险否决、chair 的执行约束和后续投后复盘提供统一前置判断。
pub fn evaluate_security_risk_gates(
    bundle: &SecurityDecisionEvidenceBundleResult,
    risk_profile: &SecurityDecisionRiskProfile,
) -> Vec<SecurityRiskGateResult> {
    vec![
        analysis_date_gate(bundle),
        data_completeness_gate(bundle),
        market_alignment_gate(bundle),
        event_risk_gate(bundle),
        risk_reward_gate(risk_profile),
    ]
}

fn analysis_date_gate(bundle: &SecurityDecisionEvidenceBundleResult) -> SecurityRiskGateResult {
    let is_valid = !bundle.analysis_date.trim().is_empty();
    SecurityRiskGateResult {
        gate_name: "analysis_date_gate".to_string(),
        result: if is_valid { "pass" } else { "fail" }.to_string(),
        blocking: !is_valid,
        reason: if is_valid {
            format!("分析日期已冻结为 {}", bundle.analysis_date)
        } else {
            "分析日期缺失，无法确认当前结论对应的交易窗口".to_string()
        },
        metric_snapshot: vec![format!("analysis_date={}", bundle.analysis_date)],
        remediation: if is_valid {
            None
        } else {
            Some("重新拉取证券研究链并确认 analysis_date".to_string())
        },
    }
}

fn data_completeness_gate(bundle: &SecurityDecisionEvidenceBundleResult) -> SecurityRiskGateResult {
    let has_gap = !bundle.data_gaps.is_empty();
    SecurityRiskGateResult {
        gate_name: "data_completeness_gate".to_string(),
        result: if has_gap { "warn" } else { "pass" }.to_string(),
        blocking: false,
        reason: if has_gap {
            "存在外部信息缺口，研究可继续但需要降低结论确定性".to_string()
        } else {
            "技术面、基本面、公告面均已取得当前可用证据".to_string()
        },
        metric_snapshot: vec![
            format!("overall_status={}", bundle.evidence_quality.overall_status),
            format!("data_gaps={}", bundle.data_gaps.len()),
        ],
        remediation: if has_gap {
            Some("补齐基本面或公告信息后再提升结论等级".to_string())
        } else {
            None
        },
    }
}

fn market_alignment_gate(bundle: &SecurityDecisionEvidenceBundleResult) -> SecurityRiskGateResult {
    let alignment = bundle
        .technical_context
        .contextual_conclusion
        .alignment
        .as_str();
    let (result, reason) = match alignment {
        "tailwind" => (
            "pass".to_string(),
            "个股与大盘、板块同向，共振环境支持进入审阅".to_string(),
        ),
        "headwind" => (
            "warn".to_string(),
            "个股与环境明显逆向，后续仓位建议需要保持保守".to_string(),
        ),
        _ => (
            "warn".to_string(),
            "环境与个股并未完全共振，仍需等待更多确认".to_string(),
        ),
    };
    SecurityRiskGateResult {
        gate_name: "market_alignment_gate".to_string(),
        result,
        blocking: false,
        reason,
        metric_snapshot: vec![format!("alignment={alignment}")],
        remediation: if alignment == "tailwind" {
            None
        } else {
            Some("等待市场环境与个股方向进一步共振".to_string())
        },
    }
}

fn event_risk_gate(bundle: &SecurityDecisionEvidenceBundleResult) -> SecurityRiskGateResult {
    let event_risk_count = collect_explicit_event_risks(bundle).len();
    let has_gap = bundle.evidence_quality.overall_status != "complete";
    SecurityRiskGateResult {
        gate_name: "event_risk_gate".to_string(),
        result: if event_risk_count > 0 || has_gap {
            "warn".to_string()
        } else {
            "pass".to_string()
        },
        blocking: false,
        reason: if event_risk_count > 0 {
            format!("信息面记录到 {event_risk_count} 条事件风险，需要在执行前人工复核")
        } else if has_gap {
            "信息面存在降级，当前事件风险识别不完整".to_string()
        } else {
            "当前信息面未发现额外需要阻断决策的事件型风险".to_string()
        },
        metric_snapshot: vec![
            format!("event_risk_count={event_risk_count}"),
            format!("overall_status={}", bundle.evidence_quality.overall_status),
        ],
        remediation: if event_risk_count > 0 || has_gap {
            Some("补看最新公告、财报或重大事项后再执行".to_string())
        } else {
            None
        },
    }
}

fn collect_explicit_event_risks(bundle: &SecurityDecisionEvidenceBundleResult) -> Vec<String> {
    let keywords = [
        "问询", "诉讼", "处罚", "减持", "违约", "退市", "爆雷", "亏损",
    ];
    let mut risks = Vec::new();

    for risk in bundle
        .fundamental_context
        .risk_flags
        .iter()
        .chain(bundle.disclosure_context.risk_flags.iter())
        .chain(bundle.industry_context.risk_flags.iter())
    {
        if keywords.iter().any(|keyword| risk.contains(keyword)) {
            risks.push(risk.clone());
        }
    }

    risks
}

fn risk_reward_gate(risk_profile: &SecurityDecisionRiskProfile) -> SecurityRiskGateResult {
    let ratio = if risk_profile.stop_loss_pct <= f64::EPSILON {
        0.0
    } else {
        risk_profile.target_return_pct / risk_profile.stop_loss_pct
    };
    let passed = ratio >= risk_profile.min_risk_reward_ratio;
    SecurityRiskGateResult {
        gate_name: "risk_reward_gate".to_string(),
        result: if passed { "pass" } else { "fail" }.to_string(),
        blocking: !passed,
        reason: if passed {
            format!(
                "目标收益/止损比为 {:.2}，达到最小风报比 {:.2}",
                ratio, risk_profile.min_risk_reward_ratio
            )
        } else {
            format!(
                "目标收益/止损比仅为 {:.2}，低于最小风报比 {:.2}",
                ratio, risk_profile.min_risk_reward_ratio
            )
        },
        metric_snapshot: vec![
            format!("stop_loss_pct={:.4}", risk_profile.stop_loss_pct),
            format!("target_return_pct={:.4}", risk_profile.target_return_pct),
            format!("risk_reward_ratio={ratio:.2}"),
        ],
        remediation: if passed {
            None
        } else {
            Some("提高目标收益或缩小止损后再进入执行建议".to_string())
        },
    }
}
