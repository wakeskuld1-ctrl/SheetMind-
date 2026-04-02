use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ops::stock::security_decision_briefing::CommitteePayload;

// 2026-04-02 CST: 这里定义正式投决会请求合同，原因是方案 B 明确要求 vote Tool 只能消费统一 committee payload，
// 目的：把 committee_mode 与 meeting_id 收口在强类型请求里，避免 dispatcher/Skill 在外层拼装第二套事实。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityCommitteeVoteRequest {
    pub committee_payload: CommitteePayload,
    #[serde(default = "default_committee_mode")]
    pub committee_mode: String,
    #[serde(default)]
    pub meeting_id: Option<String>,
}

// 2026-04-02 CST: 这里定义单个委员投票结构，原因是用户要求完整方案而不是最小返回值，
// 目的：保留角色、票型、信心、理由、关注点、阻断项与条件，后续无论 CLI 还是 GUI 都能直接解释。
// 2026-04-02 CST: 这里补单个委员投票结构的反序列化能力，原因是端到端 CLI 测试和后续 GUI/自动化链需要把结构化投票结果回读成强类型；
// 目的：让 vote Tool 的正式输出既可稳定序列化，也可被上层安全反解校验，而不是停留在松散 JSON。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CommitteeMemberVote {
    pub role: String,
    pub vote: String,
    pub confidence: String,
    pub rationale: String,
    pub focus_points: Vec<String>,
    pub blockers: Vec<String>,
    pub conditions: Vec<String>,
}

// 2026-04-02 CST: 这里定义投决会结构化结果，原因是测试已经把最终聚合字段钉死成正式合同，
// 目的：让 catalog/dispatcher/Skill 与后续 review 都围绕同一份输出结构演进，而不是继续返还松散 JSON。
// 2026-04-02 CST: 这里补投决会结果结构的反序列化能力，原因是 briefing -> vote 的整链回归需要直接校验 CLI 输出的正式合同；
// 目的：让 `SecurityCommitteeVoteResult` 成为真正可往返的合同对象，便于测试、GUI 和后续编排层复用。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityCommitteeVoteResult {
    pub symbol: String,
    pub analysis_date: String,
    pub evidence_version: String,
    pub committee_mode: String,
    pub final_decision: String,
    pub final_action: String,
    pub final_confidence: String,
    pub approval_ratio: f64,
    pub quorum_met: bool,
    pub veto_triggered: bool,
    pub veto_role: Option<String>,
    pub votes: Vec<CommitteeMemberVote>,
    pub conditions: Vec<String>,
    pub key_disagreements: Vec<String>,
    pub warnings: Vec<String>,
    pub meeting_digest: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommitteeMode {
    Standard,
    Strict,
    Advisory,
}

#[derive(Debug, Error)]
pub enum SecurityCommitteeVoteError {
    #[error("committee_mode `{0}` 不受支持，仅支持 standard/strict/advisory")]
    UnsupportedCommitteeMode(String),
    #[error("committee_payload.evidence_version 不能为空")]
    MissingEvidenceVersion,
    #[error("committee_payload.committee_schema_version `{0}` 不受支持")]
    UnsupportedCommitteeSchemaVersion(String),
    #[error("committee_payload 不完整: {0}")]
    IncompleteCommitteePayload(String),
}

pub fn security_committee_vote(
    request: &SecurityCommitteeVoteRequest,
) -> Result<SecurityCommitteeVoteResult, SecurityCommitteeVoteError> {
    let committee_mode = parse_committee_mode(&request.committee_mode)?;
    validate_committee_payload(&request.committee_payload)?;
    let warnings = build_vote_warnings(&request.committee_payload);
    let votes = build_committee_votes(&request.committee_payload, committee_mode);
    Ok(aggregate_committee_votes(
        &request.committee_payload,
        committee_mode,
        votes,
        warnings,
    ))
}

fn default_committee_mode() -> String {
    "standard".to_string()
}

// 2026-04-02 CST: 这里先把 mode 解析成内部枚举，原因是投决规则在 standard/strict/advisory 下差异明确，
// 目的：避免后续聚合逻辑继续散落字符串判断，降低规则扩展时的耦合与漏判风险。
fn parse_committee_mode(mode: &str) -> Result<CommitteeMode, SecurityCommitteeVoteError> {
    match mode.trim() {
        "standard" => Ok(CommitteeMode::Standard),
        "strict" => Ok(CommitteeMode::Strict),
        "advisory" => Ok(CommitteeMode::Advisory),
        other => Err(SecurityCommitteeVoteError::UnsupportedCommitteeMode(
            other.to_string(),
        )),
    }
}

// 2026-04-02 CST: 这里集中校验事实包完整性，原因是 vote Tool 不能在 payload 不完整时自行脑补或回源，
// 目的：把所有“是否允许进入表决”的硬门槛收口到单点校验，确保 dispatcher 与上层 Skill 得到一致错误。
fn validate_committee_payload(
    payload: &CommitteePayload,
) -> Result<(), SecurityCommitteeVoteError> {
    if payload.evidence_version.trim().is_empty() {
        return Err(SecurityCommitteeVoteError::MissingEvidenceVersion);
    }
    if payload.committee_schema_version.trim() != "committee-payload:v1" {
        return Err(
            SecurityCommitteeVoteError::UnsupportedCommitteeSchemaVersion(
                payload.committee_schema_version.clone(),
            ),
        );
    }
    if payload.briefing_digest.trim().is_empty() {
        return Err(SecurityCommitteeVoteError::IncompleteCommitteePayload(
            "briefing_digest 不能为空".to_string(),
        ));
    }
    if payload.symbol.trim().is_empty() || payload.analysis_date.trim().is_empty() {
        return Err(SecurityCommitteeVoteError::IncompleteCommitteePayload(
            "symbol 与 analysis_date 不能为空".to_string(),
        ));
    }
    if payload.key_risks.is_empty() {
        return Err(SecurityCommitteeVoteError::IncompleteCommitteePayload(
            "key_risks 不能为空".to_string(),
        ));
    }
    Ok(())
}

// 2026-04-02 CST: 这里统一生成 warning，原因是历史研究 unavailable 与 readiness 边界应被显式暴露而不是默默吞掉，
// 目的：让投决会结果即使继续输出结论，也能把证据缺口和边界条件一并带给上层。
fn build_vote_warnings(payload: &CommitteePayload) -> Vec<String> {
    let mut warnings = Vec::new();
    if payload.historical_digest.status != "available" {
        push_unique_text(
            &mut warnings,
            format!(
                "历史研究层当前为 {}，委员会将以现有 briefing 事实包继续表决。",
                payload.historical_digest.status
            ),
        );
    }
    for limitation in &payload.historical_digest.research_limitations {
        push_unique_text(&mut warnings, limitation.clone());
    }
    if !payload.evidence_checks.fundamental_ready {
        push_unique_text(
            &mut warnings,
            "财报/基本面证据尚未完全就绪，基本面委员票权将偏保守。".to_string(),
        );
    }
    if !payload.evidence_checks.briefing_ready {
        push_unique_text(
            &mut warnings,
            "briefing 尚未标记为 ready，委员会结果应视作无效草案。".to_string(),
        );
    }
    warnings
}

// 2026-04-02 CST: 这里按固定 5 个角色生成投票，原因是完整方案要求固定角色、固定职责、固定票面，
// 目的：确保同一份 payload 每次都能得到稳定可解释的表决轨迹，而不是由外层 Agent 自由发挥角色分工。
fn build_committee_votes(
    payload: &CommitteePayload,
    committee_mode: CommitteeMode,
) -> Vec<CommitteeMemberVote> {
    vec![
        build_chair_vote(payload),
        build_fundamental_vote(payload),
        build_technical_vote(payload),
        build_risk_vote(payload, committee_mode),
        build_execution_vote(payload, committee_mode),
    ]
}

fn build_chair_vote(payload: &CommitteePayload) -> CommitteeMemberVote {
    let conditions = standard_conditions(payload);
    let blockers = if payload.evidence_checks.briefing_ready {
        Vec::new()
    } else {
        vec!["briefing 未完成 ready 校验".to_string()]
    };
    let vote = if blockers.is_empty() && conditions.is_empty() {
        "approve"
    } else if blockers.is_empty() {
        "conditional_approve"
    } else {
        "defer"
    };
    CommitteeMemberVote {
        role: "chair".to_string(),
        vote: vote.to_string(),
        confidence: normalize_confidence(&payload.confidence).to_string(),
        rationale: format!(
            "主席位基于 briefing 摘要与综合建议形成总体结论，当前推荐 {}。",
            payload.recommended_action
        ),
        focus_points: vec![
            payload.recommendation_digest.summary.clone(),
            payload.briefing_digest.clone(),
        ],
        blockers,
        conditions,
    }
}

fn build_fundamental_vote(payload: &CommitteePayload) -> CommitteeMemberVote {
    let mut conditions = Vec::new();
    let mut blockers = Vec::new();
    let vote = if !payload.evidence_checks.fundamental_ready {
        blockers.push("基本面证据未就绪，需补齐财报与公告快照。".to_string());
        "defer"
    } else if payload.key_risks.is_empty() {
        "approve"
    } else {
        if payload
            .key_risks
            .iter()
            .any(|risk| risk.contains("财报") || risk.contains("同比"))
        {
            conditions.push("补核财报同比口径与关键盈利指标后再放大仓位。".to_string());
        }
        "conditional_approve"
    };
    CommitteeMemberVote {
        role: "fundamental_reviewer".to_string(),
        vote: vote.to_string(),
        confidence: normalize_confidence(&payload.recommendation_digest.confidence).to_string(),
        rationale: "基本面委员重点复核财报、公告与综合风险旗标。".to_string(),
        focus_points: payload
            .key_risks
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>(),
        blockers,
        conditions,
    }
}

fn build_technical_vote(payload: &CommitteePayload) -> CommitteeMemberVote {
    let mut conditions = Vec::new();
    let mut blockers = Vec::new();
    let vote = if payload.resonance_digest.action_bias == "reduce_or_exit"
        || payload.resonance_digest.resonance_score <= 0.42
    {
        blockers.push("技术/共振面当前不支持继续执行偏积极动作。".to_string());
        "reject"
    } else if payload.resonance_digest.resonance_score >= 0.70 {
        if payload.recommendation_digest.action_bias == "hold_and_confirm" {
            conditions.push("等待放量确认后再按 execution_digest 执行。".to_string());
            "conditional_approve"
        } else {
            "approve"
        }
    } else {
        conditions.push("技术确认不足，需等待下一轮信号增强。".to_string());
        "defer"
    };
    CommitteeMemberVote {
        role: "technical_reviewer".to_string(),
        vote: vote.to_string(),
        confidence: normalize_confidence(&payload.confidence).to_string(),
        rationale: format!(
            "技术委员基于 resonance_score={:.2} 与 action_bias={} 做出表态。",
            payload.resonance_digest.resonance_score, payload.resonance_digest.action_bias
        ),
        focus_points: payload.resonance_digest.top_positive_driver_names.clone(),
        blockers,
        conditions,
    }
}

fn build_risk_vote(
    payload: &CommitteePayload,
    committee_mode: CommitteeMode,
) -> CommitteeMemberVote {
    let mut conditions = standard_conditions(payload);
    let mut blockers = Vec::new();
    let execution_invalid = has_execution_red_flag(payload);
    let vote = if !payload.evidence_checks.briefing_ready || execution_invalid {
        blockers.push("事实包 readiness 或执行阈值存在硬缺口。".to_string());
        "reject"
    } else if matches!(committee_mode, CommitteeMode::Strict)
        && payload.historical_digest.status != "available"
    {
        conditions.push("strict 模式下需补入历史研究层再进入正式执行。".to_string());
        "defer"
    } else if payload.key_risks.len() >= 4
        || payload.recommendation_digest.action_bias == "reduce_or_exit"
    {
        blockers.push("风险旗标过多或建议已转向减仓/退出。".to_string());
        "reject"
    } else if conditions.is_empty() {
        "approve"
    } else {
        "conditional_approve"
    };
    CommitteeMemberVote {
        role: "risk_officer".to_string(),
        vote: vote.to_string(),
        confidence: "high".to_string(),
        rationale: "风险官重点检查证据齐备度、风险旗标与执行失效边界。".to_string(),
        focus_points: vec![
            format!("key_risks={}", payload.key_risks.len()),
            format!("historical_status={}", payload.historical_digest.status),
        ],
        blockers,
        conditions,
    }
}

fn build_execution_vote(
    payload: &CommitteePayload,
    committee_mode: CommitteeMode,
) -> CommitteeMemberVote {
    let mut conditions = Vec::new();
    let mut blockers = Vec::new();
    let execution_invalid = has_execution_red_flag(payload);
    let vote = if execution_invalid {
        blockers.push("execution_digest 阈值冲突，无法形成可执行交易计划。".to_string());
        "reject"
    } else if matches!(committee_mode, CommitteeMode::Strict)
        && payload.execution_digest.watch_points.len() < 2
    {
        conditions.push("strict 模式下需补足执行 watch points。".to_string());
        "defer"
    } else if payload.execution_digest.watch_points.is_empty() {
        conditions.push("补齐执行 watch points 后再落地。".to_string());
        "conditional_approve"
    } else if payload.historical_digest.status != "available" {
        conditions.push("先按小仓位试单，再等待历史研究层补齐。".to_string());
        "conditional_approve"
    } else {
        "approve"
    };
    CommitteeMemberVote {
        role: "execution_reviewer".to_string(),
        vote: vote.to_string(),
        confidence: "high".to_string(),
        rationale: "执行委员确认加减仓阈值、止损位与失效位是否可直接落地。".to_string(),
        focus_points: vec![
            format!(
                "add_trigger={:.2}",
                payload.execution_digest.add_trigger_price
            ),
            format!("stop_loss={:.2}", payload.execution_digest.stop_loss_price),
        ],
        blockers,
        conditions,
    }
}

// 2026-04-02 CST: 这里集中做表决聚合，原因是标准/严格/咨询模式的 quorum、veto 与最终结论必须保持单点真值，
// 目的：让测试、CLI 与 Skill 都以同一套规则产出 final_decision / approval_ratio / warnings 等正式字段。
fn aggregate_committee_votes(
    payload: &CommitteePayload,
    committee_mode: CommitteeMode,
    votes: Vec<CommitteeMemberVote>,
    warnings: Vec<String>,
) -> SecurityCommitteeVoteResult {
    let total_votes = votes.len().max(1) as f64;
    let approval_votes = votes
        .iter()
        .filter(|vote| matches!(vote.vote.as_str(), "approve" | "conditional_approve"))
        .count();
    let conditional_votes = votes
        .iter()
        .filter(|vote| vote.vote == "conditional_approve")
        .count();
    let reject_votes = votes.iter().filter(|vote| vote.vote == "reject").count();
    let quorum_met = votes.len() >= 4;
    let veto_role = resolve_veto_role(&votes, committee_mode);
    let veto_triggered = veto_role.is_some();
    let approval_ratio = round_ratio(approval_votes as f64 / total_votes);
    let conditions = collect_conditions(&votes);
    let key_disagreements = collect_key_disagreements(payload, &votes, &warnings);
    let final_decision = determine_final_decision(
        committee_mode,
        quorum_met,
        veto_triggered,
        approval_votes,
        conditional_votes,
        reject_votes,
    );
    let final_action = determine_final_action(payload, &final_decision);
    let final_confidence = determine_final_confidence(payload, &final_decision, &warnings);
    let meeting_digest = build_meeting_digest(
        payload,
        committee_mode,
        &final_decision,
        approval_ratio,
        veto_triggered,
    );

    SecurityCommitteeVoteResult {
        symbol: payload.symbol.clone(),
        analysis_date: payload.analysis_date.clone(),
        evidence_version: payload.evidence_version.clone(),
        committee_mode: committee_mode_label(committee_mode).to_string(),
        final_decision,
        final_action,
        final_confidence,
        approval_ratio,
        quorum_met,
        veto_triggered,
        veto_role,
        votes,
        conditions,
        key_disagreements,
        warnings,
        meeting_digest,
    }
}

fn resolve_veto_role(
    votes: &[CommitteeMemberVote],
    committee_mode: CommitteeMode,
) -> Option<String> {
    match committee_mode {
        CommitteeMode::Advisory => None,
        CommitteeMode::Standard => votes
            .iter()
            .find(|vote| vote.role == "risk_officer" && vote.vote == "reject")
            .map(|vote| vote.role.clone()),
        CommitteeMode::Strict => votes
            .iter()
            .find(|vote| {
                matches!(vote.role.as_str(), "risk_officer" | "execution_reviewer")
                    && vote.vote == "reject"
            })
            .map(|vote| vote.role.clone()),
    }
}

fn determine_final_decision(
    committee_mode: CommitteeMode,
    quorum_met: bool,
    veto_triggered: bool,
    approval_votes: usize,
    conditional_votes: usize,
    reject_votes: usize,
) -> String {
    if !quorum_met {
        return "deferred".to_string();
    }
    match committee_mode {
        CommitteeMode::Standard => {
            if veto_triggered {
                "rejected".to_string()
            } else if approval_votes >= 3 {
                if conditional_votes == 0 && reject_votes <= 1 {
                    "approved".to_string()
                } else {
                    "approved_with_conditions".to_string()
                }
            } else if reject_votes >= 3 {
                "rejected".to_string()
            } else {
                "deferred".to_string()
            }
        }
        CommitteeMode::Strict => {
            if veto_triggered {
                "rejected".to_string()
            } else if approval_votes >= 4 && conditional_votes == 0 {
                "approved".to_string()
            } else if approval_votes >= 4 {
                "approved_with_conditions".to_string()
            } else {
                "deferred".to_string()
            }
        }
        CommitteeMode::Advisory => {
            if approval_votes > reject_votes {
                if conditional_votes > 0 {
                    "approved_with_conditions".to_string()
                } else {
                    "approved".to_string()
                }
            } else if reject_votes > approval_votes {
                "rejected".to_string()
            } else {
                "deferred".to_string()
            }
        }
    }
}

fn determine_final_action(payload: &CommitteePayload, final_decision: &str) -> String {
    match final_decision {
        "approved" => payload.recommended_action.clone(),
        "approved_with_conditions" => format!("{}_with_conditions", payload.recommended_action),
        "deferred" => "wait_for_next_review".to_string(),
        "rejected" => "do_not_execute".to_string(),
        _ => "wait_for_next_review".to_string(),
    }
}

fn determine_final_confidence(
    payload: &CommitteePayload,
    final_decision: &str,
    warnings: &[String],
) -> String {
    match final_decision {
        "approved" if warnings.is_empty() => normalize_confidence(&payload.confidence).to_string(),
        "approved" | "approved_with_conditions" => {
            downgrade_confidence(&payload.recommendation_digest.confidence).to_string()
        }
        "deferred" => "low".to_string(),
        "rejected" => "high".to_string(),
        _ => "low".to_string(),
    }
}

fn build_meeting_digest(
    payload: &CommitteePayload,
    committee_mode: CommitteeMode,
    final_decision: &str,
    approval_ratio: f64,
    veto_triggered: bool,
) -> String {
    format!(
        "{} 在 {} 模式下形成 {}，approval_ratio={:.2}，veto_triggered={}，建议动作为 {}。",
        payload.symbol,
        committee_mode_label(committee_mode),
        final_decision,
        approval_ratio,
        veto_triggered,
        payload.recommended_action
    )
}

fn collect_conditions(votes: &[CommitteeMemberVote]) -> Vec<String> {
    let mut conditions = Vec::new();
    for vote in votes {
        for condition in &vote.conditions {
            push_unique_text(&mut conditions, condition.clone());
        }
    }
    conditions
}

fn collect_key_disagreements(
    payload: &CommitteePayload,
    votes: &[CommitteeMemberVote],
    warnings: &[String],
) -> Vec<String> {
    let mut disagreements = Vec::new();
    for point in &payload.minority_objection_points {
        push_unique_text(&mut disagreements, point.clone());
    }
    for vote in votes {
        if matches!(
            vote.vote.as_str(),
            "reject" | "defer" | "conditional_approve"
        ) {
            push_unique_text(
                &mut disagreements,
                format!("{}: {}", vote.role, vote.rationale),
            );
        }
        for blocker in &vote.blockers {
            push_unique_text(
                &mut disagreements,
                format!("{} blocker: {}", vote.role, blocker),
            );
        }
    }
    if disagreements.is_empty() {
        for warning in warnings {
            push_unique_text(&mut disagreements, warning.clone());
        }
    }
    disagreements
}

fn standard_conditions(payload: &CommitteePayload) -> Vec<String> {
    let mut conditions = Vec::new();
    if payload.historical_digest.status != "available" {
        push_unique_text(
            &mut conditions,
            "历史研究层未接入前，按较小仓位和更短复核节奏执行。".to_string(),
        );
    }
    if !payload.key_risks.is_empty() {
        push_unique_text(
            &mut conditions,
            "严格遵守 execution_digest 里的加仓、减仓与止损阈值。".to_string(),
        );
    }
    if !payload.resonance_digest.event_override_titles.is_empty() {
        push_unique_text(
            &mut conditions,
            "若事件覆盖项发生反转，需立即重新召开投决会。".to_string(),
        );
    }
    conditions
}

fn has_execution_red_flag(payload: &CommitteePayload) -> bool {
    payload.execution_digest.add_trigger_price <= 0.0
        || payload.execution_digest.stop_loss_price <= 0.0
        || payload.execution_digest.add_trigger_price <= payload.execution_digest.stop_loss_price
        || payload.execution_digest.invalidation_price >= payload.execution_digest.add_trigger_price
}

fn normalize_confidence(confidence: &str) -> &str {
    match confidence {
        "high" | "medium" | "low" => confidence,
        _ => "medium",
    }
}

fn downgrade_confidence(confidence: &str) -> &str {
    match normalize_confidence(confidence) {
        "high" => "medium",
        "medium" => "low",
        _ => "low",
    }
}

fn committee_mode_label(mode: CommitteeMode) -> &'static str {
    match mode {
        CommitteeMode::Standard => "standard",
        CommitteeMode::Strict => "strict",
        CommitteeMode::Advisory => "advisory",
    }
}

fn round_ratio(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn push_unique_text(target: &mut Vec<String>, candidate: String) {
    if candidate.trim().is_empty() {
        return;
    }
    if !target.iter().any(|existing| existing == &candidate) {
        target.push(candidate);
    }
}
