use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::ops::stock::security_decision_approval_brief::SecurityDecisionApprovalBrief;
use crate::ops::stock::security_decision_package::SecurityDecisionPackageDocument;
use crate::ops::stock::security_decision_package_revision::{
    SecurityDecisionPackageRevisionRequest, security_decision_package_revision,
};
use crate::ops::stock::security_post_meeting_conclusion::{
    SecurityPostMeetingConclusionBuildInput, build_security_post_meeting_conclusion,
};

// 2026-04-08 CST: 这里新增会后结论记录 Tool 请求合同，原因是 Task 3 需要独立的“会后治理动作入口”；
// 目的：把 package_path、最终处置、复核与签名透传等参数集中收口，避免外层自己串 revision 细节。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityRecordPostMeetingConclusionRequest {
    pub package_path: String,
    pub final_disposition: String,
    pub disposition_reason: String,
    #[serde(default)]
    pub key_reasons: Vec<String>,
    #[serde(default)]
    pub required_follow_ups: Vec<String>,
    #[serde(default)]
    pub reviewer_notes: String,
    pub reviewer: String,
    pub reviewer_role: String,
    #[serde(default = "default_revision_reason")]
    pub revision_reason: String,
    #[serde(default = "default_reverify_after_revision")]
    pub reverify_after_revision: bool,
    #[serde(default)]
    pub approval_brief_signing_key_secret: Option<String>,
    #[serde(default)]
    pub approval_brief_signing_key_secret_env: Option<String>,
}

// 2026-04-08 CST: 这里定义会后结论 Tool 响应合同，原因是 CLI/Skill 需要一次拿到对象、路径和 revision 结果；
// 目的：先满足红测与后续编排使用场景，不要求调用方二次回读磁盘。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SecurityRecordPostMeetingConclusionResult {
    pub post_meeting_conclusion: Value,
    pub post_meeting_conclusion_path: String,
    pub decision_package: Value,
    pub decision_package_path: String,
    pub package_version: u32,
    pub previous_package_path: String,
    pub revision_reason: String,
    pub verification_report_path: Option<String>,
}

#[derive(Debug, Error)]
pub enum SecurityRecordPostMeetingConclusionError {
    #[error("记录证券会后结论失败: {0}")]
    Record(String),
}

// 2026-04-08 CST: 这里实现会后结论记录主入口，原因是红测要求“独立对象落盘 + 驱动 package revision”同时成立；
// 目的：先把最小 happy path 走通，后续再把 post_meeting_conclusion 正式挂入 package 对象图和 verify 链路。
pub fn security_record_post_meeting_conclusion(
    request: &SecurityRecordPostMeetingConclusionRequest,
) -> Result<SecurityRecordPostMeetingConclusionResult, SecurityRecordPostMeetingConclusionError> {
    if request.package_path.trim().is_empty() {
        return Err(SecurityRecordPostMeetingConclusionError::Record(
            "package_path cannot be empty".to_string(),
        ));
    }
    if !is_supported_final_disposition(&request.final_disposition) {
        return Err(SecurityRecordPostMeetingConclusionError::Record(format!(
            "unsupported final_disposition `{}`",
            request.final_disposition
        )));
    }

    let package_path = PathBuf::from(request.package_path.trim());
    let package: SecurityDecisionPackageDocument =
        serde_json::from_slice(&fs::read(&package_path).map_err(|error| {
            SecurityRecordPostMeetingConclusionError::Record(error.to_string())
        })?)
        .map_err(|error| SecurityRecordPostMeetingConclusionError::Record(error.to_string()))?;
    let approval_brief: SecurityDecisionApprovalBrief = serde_json::from_slice(
        &fs::read(&package.object_graph.approval_brief_path)
            .map_err(|error| SecurityRecordPostMeetingConclusionError::Record(error.to_string()))?,
    )
    .map_err(|error| SecurityRecordPostMeetingConclusionError::Record(error.to_string()))?;

    let post_meeting_conclusion_path =
        resolve_post_meeting_conclusion_path(&package_path, &package.decision_id)?;
    let revision_reason = normalized_revision_reason(&request.revision_reason);
    let conclusion =
        build_security_post_meeting_conclusion(SecurityPostMeetingConclusionBuildInput {
            generated_at: chrono::Utc::now().to_rfc3339(),
            scene_name: package.scene_name.clone(),
            decision_id: package.decision_id.clone(),
            decision_ref: package.decision_ref.clone(),
            approval_ref: package.approval_ref.clone(),
            symbol: package.symbol.clone(),
            analysis_date: package.analysis_date.clone(),
            source_package_path: package_path.to_string_lossy().to_string(),
            source_package_version: package.package_version,
            source_brief_ref: approval_brief.brief_id.clone(),
            source_brief_path: package.object_graph.approval_brief_path.clone(),
            final_disposition: request.final_disposition.trim().to_string(),
            disposition_reason: request.disposition_reason.trim().to_string(),
            key_reasons: sanitize_string_list(&request.key_reasons),
            required_follow_ups: sanitize_string_list(&request.required_follow_ups),
            reviewer_notes: request.reviewer_notes.trim().to_string(),
            reviewer: request.reviewer.trim().to_string(),
            reviewer_role: request.reviewer_role.trim().to_string(),
            revision_reason: revision_reason.clone(),
        });
    persist_json(&post_meeting_conclusion_path, &conclusion)?;

    let revision = security_decision_package_revision(&SecurityDecisionPackageRevisionRequest {
        package_path: package_path.to_string_lossy().to_string(),
        revision_reason: revision_reason.clone(),
        reverify_after_revision: request.reverify_after_revision,
        // 2026-04-12 CST: Keep lifecycle attachment paths empty for post-meeting
        // package revisions unless this flow explicitly records them, because P8
        // introduced optional lifecycle attachments on the shared revision contract.
        // Purpose: preserve backward-compatible package revision behavior for conclusion-only updates.
        condition_review_path: None,
        execution_record_path: None,
        post_trade_review_path: None,
        approval_brief_signing_key_secret: request.approval_brief_signing_key_secret.clone(),
        approval_brief_signing_key_secret_env: request
            .approval_brief_signing_key_secret_env
            .clone(),
    })
    .map_err(|error| SecurityRecordPostMeetingConclusionError::Record(error.to_string()))?;

    Ok(SecurityRecordPostMeetingConclusionResult {
        post_meeting_conclusion: serde_json::to_value(&conclusion)
            .map_err(|error| SecurityRecordPostMeetingConclusionError::Record(error.to_string()))?,
        post_meeting_conclusion_path: post_meeting_conclusion_path.to_string_lossy().to_string(),
        decision_package: revision.decision_package,
        decision_package_path: revision.decision_package_path,
        package_version: revision.package_version,
        previous_package_path: revision.previous_package_path,
        revision_reason: revision.revision_reason,
        verification_report_path: revision.verification_report_path,
    })
}

fn resolve_post_meeting_conclusion_path(
    package_path: &Path,
    decision_id: &str,
) -> Result<PathBuf, SecurityRecordPostMeetingConclusionError> {
    let runtime_root = find_runtime_root(package_path)?;
    Ok(runtime_root
        .join("post_meeting_conclusions")
        .join(format!("{decision_id}.json")))
}

fn find_runtime_root(
    package_path: &Path,
) -> Result<PathBuf, SecurityRecordPostMeetingConclusionError> {
    for ancestor in package_path.ancestors() {
        if ancestor
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name == "decision_packages")
            .unwrap_or(false)
        {
            return ancestor.parent().map(Path::to_path_buf).ok_or_else(|| {
                SecurityRecordPostMeetingConclusionError::Record(
                    "failed to locate runtime root from package path".to_string(),
                )
            });
        }
    }

    Err(SecurityRecordPostMeetingConclusionError::Record(
        "failed to locate decision_packages directory from package path".to_string(),
    ))
}

fn persist_json<T: Serialize>(
    path: &Path,
    value: &T,
) -> Result<(), SecurityRecordPostMeetingConclusionError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| SecurityRecordPostMeetingConclusionError::Record(error.to_string()))?;
    }
    let payload = serde_json::to_vec_pretty(value)
        .map_err(|error| SecurityRecordPostMeetingConclusionError::Record(error.to_string()))?;
    fs::write(path, payload)
        .map_err(|error| SecurityRecordPostMeetingConclusionError::Record(error.to_string()))
}

fn sanitize_string_list(values: &[String]) -> Vec<String> {
    values
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

fn is_supported_final_disposition(value: &str) -> bool {
    matches!(
        value.trim(),
        "approve" | "reject" | "needs_more_evidence" | "approve_with_override"
    )
}

fn normalized_revision_reason(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        default_revision_reason()
    } else {
        trimmed.to_string()
    }
}

fn default_revision_reason() -> String {
    "post_meeting_conclusion_recorded".to_string()
}

fn default_reverify_after_revision() -> bool {
    true
}
