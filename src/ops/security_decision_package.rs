use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::ops::stock::security_condition_review::SecurityConditionReviewDocument;
use crate::tools::contracts::{
    SecurityConditionReviewFollowUpAction, SecurityConditionReviewTriggerType,
};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityDecisionPackageDocument {
    pub package_id: String,
    pub contract_version: String,
    pub created_at: String,
    pub package_version: u32,
    pub previous_package_path: Option<String>,
    pub revision_reason: String,
    pub trigger_event_summary: String,
    pub scene_name: String,
    pub decision_id: String,
    pub decision_ref: String,
    pub approval_ref: String,
    pub symbol: String,
    pub analysis_date: String,
    pub package_status: String,
    pub object_graph: SecurityDecisionPackageObjectGraph,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition_review_digest: Option<SecurityDecisionPackageConditionReviewDigest>,
    pub artifact_manifest: Vec<SecurityDecisionPackageArtifact>,
    pub governance_binding: SecurityDecisionPackageGovernanceBinding,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityDecisionPackageObjectGraph {
    pub decision_ref: String,
    pub approval_ref: String,
    pub position_plan_ref: String,
    pub approval_brief_ref: String,
    pub scorecard_ref: String,
    pub decision_card_path: String,
    pub approval_request_path: String,
    pub position_plan_path: String,
    pub approval_brief_path: String,
    pub scorecard_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition_review_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityDecisionPackageConditionReviewDigest {
    pub condition_review_ref: String,
    pub generated_at: String,
    pub review_trigger_type: String,
    pub recommended_follow_up_action: String,
    pub decision_ref: String,
    pub approval_ref: String,
    pub position_plan_ref: String,
    pub symbol: String,
    pub analysis_date: String,
    pub review_summary: String,
}

impl SecurityDecisionPackageConditionReviewDigest {
    // 2026-04-10 CST: 这里提供从正式 condition review 文档到 package digest 的单点映射，原因是方案 B 要求 package 既保留 ref 主锚点，
    // 目的：又能给审批/复盘/AI 提供可直接消费的最小摘要，但仍避免在多处重复手拼 trigger/action 文案。
    pub fn from_condition_review(document: &SecurityConditionReviewDocument) -> Self {
        Self {
            condition_review_ref: document.condition_review_id.clone(),
            generated_at: document.generated_at.clone(),
            review_trigger_type: condition_review_trigger_label(&document.review_trigger_type)
                .to_string(),
            recommended_follow_up_action:
                condition_review_action_label(&document.recommended_follow_up_action).to_string(),
            decision_ref: document.decision_ref.clone(),
            approval_ref: document.approval_ref.clone(),
            position_plan_ref: document.position_plan_ref.clone(),
            symbol: document.symbol.clone(),
            analysis_date: document.analysis_date.clone(),
            review_summary: document.review_summary.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityDecisionPackageArtifact {
    pub artifact_role: String,
    pub path: String,
    pub sha256: String,
    pub contract_version: String,
    pub required: bool,
    pub present: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityDecisionPackageGovernanceBinding {
    pub evidence_hash: String,
    pub governance_hash: String,
    pub decision_ref: String,
    pub approval_ref: String,
    pub package_scope: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SecurityDecisionPackageBuildInput {
    pub created_at: String,
    pub package_version: u32,
    pub previous_package_path: Option<String>,
    pub revision_reason: String,
    pub trigger_event_summary: String,
    pub scene_name: String,
    pub decision_id: String,
    pub decision_ref: String,
    pub approval_ref: String,
    pub symbol: String,
    pub analysis_date: String,
    pub decision_status: String,
    pub approval_status: String,
    pub position_plan_ref: String,
    pub approval_brief_ref: String,
    pub scorecard_ref: String,
    pub decision_card_path: String,
    pub approval_request_path: String,
    pub position_plan_path: String,
    pub approval_brief_path: String,
    pub scorecard_path: String,
    pub condition_review_ref: Option<String>,
    pub condition_review_digest: Option<SecurityDecisionPackageConditionReviewDigest>,
    pub evidence_hash: String,
    pub governance_hash: String,
    pub artifact_manifest: Vec<SecurityDecisionPackageArtifact>,
}

pub fn build_security_decision_package(
    input: SecurityDecisionPackageBuildInput,
) -> SecurityDecisionPackageDocument {
    // 2026-04-10 CST: 这里在 package builder 内统一归一化 condition review 绑定，原因是 Task 4 要把 ref 与 digest 一起挂进正式 package，
    // 目的：确保调用方即使只传一侧或两侧同时传入，也能在 builder 出口拿到自一致的正式 package 合同。
    let normalized_condition_review_digest = input
        .condition_review_digest
        .map(|mut digest| {
            if digest.condition_review_ref.trim().is_empty() {
                if let Some(ref_value) = input.condition_review_ref.as_ref() {
                    digest.condition_review_ref = ref_value.clone();
                }
            }
            digest
        });
    let normalized_condition_review_ref = normalized_condition_review_digest
        .as_ref()
        .map(|digest| digest.condition_review_ref.clone())
        .or(input.condition_review_ref.clone());

    SecurityDecisionPackageDocument {
        package_id: format!("pkg-{}", input.decision_id),
        contract_version: "security_decision_package.v1".to_string(),
        created_at: normalize_created_at(&input.created_at),
        package_version: input.package_version.max(1),
        previous_package_path: input.previous_package_path,
        revision_reason: input.revision_reason,
        trigger_event_summary: input.trigger_event_summary,
        scene_name: input.scene_name,
        decision_id: input.decision_id,
        decision_ref: input.decision_ref.clone(),
        approval_ref: input.approval_ref.clone(),
        symbol: input.symbol,
        analysis_date: input.analysis_date,
        package_status: derive_package_status(&input.decision_status, &input.approval_status),
        object_graph: SecurityDecisionPackageObjectGraph {
            decision_ref: input.decision_ref.clone(),
            approval_ref: input.approval_ref.clone(),
            position_plan_ref: input.position_plan_ref,
            approval_brief_ref: input.approval_brief_ref,
            scorecard_ref: input.scorecard_ref,
            decision_card_path: input.decision_card_path,
            approval_request_path: input.approval_request_path,
            position_plan_path: input.position_plan_path,
            approval_brief_path: input.approval_brief_path,
            scorecard_path: input.scorecard_path,
            condition_review_ref: normalized_condition_review_ref,
        },
        condition_review_digest: normalized_condition_review_digest,
        artifact_manifest: input.artifact_manifest,
        governance_binding: SecurityDecisionPackageGovernanceBinding {
            evidence_hash: input.evidence_hash,
            governance_hash: input.governance_hash,
            decision_ref: input.decision_ref,
            approval_ref: input.approval_ref,
            package_scope: "security_decision_submit_approval".to_string(),
        },
    }
}

pub fn sha256_for_json_value(value: &serde_json::Value) -> Result<String, String> {
    let payload = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    Ok(sha256_for_bytes(&payload))
}

pub fn sha256_for_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn derive_package_status(decision_status: &str, approval_status: &str) -> String {
    match (decision_status, approval_status) {
        (_, "Approved") => "approved_bundle_ready".to_string(),
        (_, "Rejected") => "rejected_bundle_ready".to_string(),
        (_, "ApprovedWithOverride") => "approved_with_override_bundle_ready".to_string(),
        (_, "NeedsMoreEvidence") => "needs_follow_up".to_string(),
        ("blocked", _) => "needs_follow_up".to_string(),
        ("ready_for_review", "Pending") => "review_bundle_ready".to_string(),
        _ => "pending_review_materials".to_string(),
    }
}

fn normalize_created_at(value: &str) -> String {
    if value.trim().is_empty() {
        Utc::now().to_rfc3339()
    } else {
        value.trim().to_string()
    }
}

fn condition_review_trigger_label(trigger_type: &SecurityConditionReviewTriggerType) -> &'static str {
    match trigger_type {
        SecurityConditionReviewTriggerType::ManualReview => "manual_review",
        SecurityConditionReviewTriggerType::EndOfDayReview => "end_of_day_review",
        SecurityConditionReviewTriggerType::EventReview => "event_review",
        SecurityConditionReviewTriggerType::DataStalenessReview => "data_staleness_review",
    }
}

fn condition_review_action_label(
    action: &SecurityConditionReviewFollowUpAction,
) -> &'static str {
    match action {
        SecurityConditionReviewFollowUpAction::KeepPlan => "keep_plan",
        SecurityConditionReviewFollowUpAction::UpdatePositionPlan => "update_position_plan",
        SecurityConditionReviewFollowUpAction::ReopenResearch => "reopen_research",
        SecurityConditionReviewFollowUpAction::ReopenCommittee => "reopen_committee",
        SecurityConditionReviewFollowUpAction::FreezeExecution => "freeze_execution",
    }
}
