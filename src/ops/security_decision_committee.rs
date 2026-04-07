use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ops::stock::security_decision_card::{
    build_security_decision_card, SecurityDecisionCard, SecurityDecisionThesis,
};
use crate::ops::stock::security_decision_evidence_bundle::{
    security_decision_evidence_bundle, SecurityDecisionEvidenceBundleError,
    SecurityDecisionEvidenceBundleRequest, SecurityDecisionEvidenceBundleResult,
};
use crate::ops::stock::security_risk_gates::{
    evaluate_security_risk_gates, SecurityDecisionRiskProfile, SecurityRiskGateResult,
};

// 2026-04-01 CST: 这里定义证券投决会请求，原因是用户输入除了标的和环境代理，还会携带止损与目标收益约束；
// 目的：把“研究请求”和“裁决参数”收进同一个 Tool 合同，支持单次调用完成投决流程。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityDecisionCommitteeRequest {
    pub symbol: String,
    #[serde(default)]
    pub market_symbol: Option<String>,
    #[serde(default)]
    pub sector_symbol: Option<String>,
    #[serde(default)]
    pub market_profile: Option<String>,
    #[serde(default)]
    pub sector_profile: Option<String>,
    #[serde(default)]
    pub as_of_date: Option<String>,
    #[serde(default = "default_lookback_days")]
    pub lookback_days: usize,
    #[serde(default = "default_disclosure_limit")]
    pub disclosure_limit: usize,
    #[serde(default = "default_stop_loss_pct")]
    pub stop_loss_pct: f64,
    #[serde(default = "default_target_return_pct")]
    pub target_return_pct: f64,
    #[serde(default = "default_min_risk_reward_ratio")]
    pub min_risk_reward_ratio: f64,
}

// 2026-04-01 CST: 这里定义证券投决会结果，原因是顶层 Tool 需要同时返回证据、正反方、闸门和投决卡；
// 目的：让一次请求能拿到完整投决闭环，而不是外层再手工拼装多个中间结果。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SecurityDecisionCommitteeResult {
    pub symbol: String,
    pub analysis_date: String,
    pub evidence_bundle: SecurityDecisionEvidenceBundleResult,
    pub bull_case: SecurityDecisionThesis,
    pub bear_case: SecurityDecisionThesis,
    pub risk_gates: Vec<SecurityRiskGateResult>,
    pub decision_card: SecurityDecisionCard,
}

// 2026-04-01 CST: 这里单独定义投决会错误边界，原因是顶层 Tool 需要用自己的语言描述“证据冻结失败”；
// 目的：给 dispatcher 和 Skill 一个稳定错误口径，不泄露太多内部实现细节。
#[derive(Debug, Error)]
pub enum SecurityDecisionCommitteeError {
    #[error("证券投决会证据准备失败: {0}")]
    Evidence(#[from] SecurityDecisionEvidenceBundleError),
}

// 2026-04-01 CST: 这里实现证券投决会总入口，原因是我们要把研究、正反方、闸门和裁决收进一个可复用的 Tool；
// 目的：让同一对话能够通过“单次冻结证据 + 双立场独立生成 + 风控闸门裁决”拿到结构化投决结果。
pub fn security_decision_committee(
    request: &SecurityDecisionCommitteeRequest,
) -> Result<SecurityDecisionCommitteeResult, SecurityDecisionCommitteeError> {
    let evidence_request = SecurityDecisionEvidenceBundleRequest {
        symbol: request.symbol.clone(),
        market_symbol: request.market_symbol.clone(),
        sector_symbol: request.sector_symbol.clone(),
        market_profile: request.market_profile.clone(),
        sector_profile: request.sector_profile.clone(),
        as_of_date: request.as_of_date.clone(),
        lookback_days: request.lookback_days,
        disclosure_limit: request.disclosure_limit,
    };
    let evidence_bundle = security_decision_evidence_bundle(&evidence_request)?;

    // 2026-04-01 CST: 这里刻意让多头与空头只依赖冻结后的 evidence_bundle，原因是单对话内也要尽量保持双方初判独立；
    // 目的：避免后续把另一方的生成结果反向污染本方结论，退化成一边写一边改口的伪博弈。
    let bull_case = build_bull_case(&evidence_bundle);
    let bear_case = build_bear_case(&evidence_bundle);
    let risk_profile = SecurityDecisionRiskProfile {
        stop_loss_pct: request.stop_loss_pct,
        target_return_pct: request.target_return_pct,
        min_risk_reward_ratio: request.min_risk_reward_ratio,
    };
    let risk_gates = evaluate_security_risk_gates(&evidence_bundle, &risk_profile);
    let decision_card = build_security_decision_card(
        &evidence_bundle,
        &bull_case,
        &bear_case,
        &risk_gates,
        &risk_profile,
    );

    Ok(SecurityDecisionCommitteeResult {
        symbol: evidence_bundle.symbol.clone(),
        analysis_date: evidence_bundle.analysis_date.clone(),
        evidence_bundle,
        bull_case,
        bear_case,
        risk_gates,
        decision_card,
    })
}

// 2026-04-01 CST: 这里生成多头立场摘要，原因是投决会需要一个只论证“为什么可以做”的独立对象；
// 目的：把研究链里的支持证据提炼成结构化 thesis，而不是直接给最终买卖建议。
fn build_bull_case(bundle: &SecurityDecisionEvidenceBundleResult) -> SecurityDecisionThesis {
    let stock_conclusion = &bundle
        .technical_context
        .stock_analysis
        .consultation_conclusion;
    let mut thesis_points = vec![
        stock_conclusion.headline.clone(),
        bundle
            .technical_context
            .contextual_conclusion
            .headline
            .clone(),
        bundle.integrated_conclusion.headline.clone(),
    ];
    if bundle.fundamental_context.status == "available" {
        thesis_points.push(bundle.fundamental_context.headline.clone());
    }
    if bundle.disclosure_context.status == "available" {
        thesis_points.push(bundle.disclosure_context.headline.clone());
    }
    dedupe_strings(&mut thesis_points);

    let mut invalidation_conditions = stock_conclusion.risk_flags.clone();
    invalidation_conditions.extend(
        bundle
            .technical_context
            .contextual_conclusion
            .risk_flags
            .clone(),
    );
    if invalidation_conditions.is_empty() {
        invalidation_conditions.push("个股失去技术面确认或环境共振时，原多头论证失效".to_string());
    }
    dedupe_strings(&mut invalidation_conditions);

    SecurityDecisionThesis {
        thesis_label: "bullish_thesis".to_string(),
        headline: format!(
            "{}，当前更接近“有条件通过研究审阅”的多头论证",
            bundle.integrated_conclusion.headline
        ),
        confidence: match bundle.integrated_conclusion.stance.as_str() {
            "positive" => "high".to_string(),
            "watchful_positive" => "medium".to_string(),
            _ => "guarded".to_string(),
        },
        thesis_points,
        invalidation_conditions,
        cited_risks: bundle.risk_notes.clone(),
    }
}

// 2026-04-01 CST: 这里生成空头挑战摘要，原因是投决会必须有一个专门挑错、找失效条件的对立对象；
// 目的：把单边乐观结论拉回到“有哪些证据不足或风险被低估”这一层。
fn build_bear_case(bundle: &SecurityDecisionEvidenceBundleResult) -> SecurityDecisionThesis {
    let mut thesis_points = Vec::new();
    if !bundle.data_gaps.is_empty() {
        thesis_points.extend(bundle.data_gaps.clone());
    }
    thesis_points.extend(bundle.risk_notes.iter().take(4).cloned());
    if thesis_points.is_empty() {
        thesis_points
            .push("当前未发现足以直接否决的强空头证据，但仍需防止研究结论过度乐观".to_string());
    }
    dedupe_strings(&mut thesis_points);

    let invalidation_conditions = vec![
        "如果后续基本面与公告持续确认且环境维持顺风，则本轮空头挑战权重下降".to_string(),
        "如果个股回踩后仍守住关键支撑并延续量价确认，则不宜继续按高强度反对处理".to_string(),
    ];

    SecurityDecisionThesis {
        thesis_label: "bearish_challenge".to_string(),
        headline: "当前需要重点核查证据缺口、事件风险与环境变化，而不是直接把研究偏强等同于可执行"
            .to_string(),
        confidence: if bundle.data_gaps.is_empty() {
            "medium".to_string()
        } else {
            "high".to_string()
        },
        thesis_points,
        invalidation_conditions,
        cited_risks: bundle.risk_notes.clone(),
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

fn default_lookback_days() -> usize {
    260
}

fn default_disclosure_limit() -> usize {
    8
}

fn default_stop_loss_pct() -> f64 {
    0.05
}

fn default_target_return_pct() -> f64 {
    0.12
}

fn default_min_risk_reward_ratio() -> f64 {
    2.0
}
