use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::ops::stock::security_decision_package::{
    SecurityDecisionPackageArtifact, SecurityDecisionPackageBuildInput,
    SecurityDecisionPackageDocument, build_security_decision_package, sha256_for_bytes,
    sha256_for_json_value,
};
use crate::ops::stock::security_decision_verify_package::{
    SecurityDecisionVerifyPackageRequest, security_decision_verify_package,
};

// 2026-04-02 CST: 这里定义审批包版本化请求，原因是 P0-6 需要一个正式 Tool 把旧 package 升级成下一版本；
// 目的：把版本化所需的包路径、修订原因和是否重跑校验等参数统一收口，避免调用方自己拼接内部步骤。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityDecisionPackageRevisionRequest {
    pub package_path: String,
    #[serde(default = "default_revision_reason")]
    pub revision_reason: String,
    #[serde(default = "default_reverify_after_revision")]
    pub reverify_after_revision: bool,
    #[serde(default)]
    pub approval_brief_signing_key_secret: Option<String>,
    #[serde(default)]
    pub approval_brief_signing_key_secret_env: Option<String>,
}

// 2026-04-02 CST: 这里定义审批包版本化结果，原因是上层调用方除了新 package 以外，还需要知道 lineage 和可选 verification 工件；
// 目的：让 CLI / Skill 能直接消费 v2 package 结果，而不再手工拼路径和再调一次 verify。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SecurityDecisionPackageRevisionResult {
    pub decision_package: Value,
    pub decision_package_path: String,
    pub package_version: u32,
    pub previous_package_path: String,
    pub revision_reason: String,
    pub trigger_event_summary: String,
    pub verification_report_path: Option<String>,
}

#[derive(Debug, Error)]
pub enum SecurityDecisionPackageRevisionError {
    #[error("证券审批包版本化失败: {0}")]
    Revision(String),
}

// 2026-04-02 CST: 这里实现正式审批包版本化入口，原因是审批包需要随着审批动作生成后续版本，而不是停留在初始提交态；
// 目的：读取旧 package 与最新审批工件，生成新版本 package，并在需要时立即附带新的 verification report。
pub fn security_decision_package_revision(
    request: &SecurityDecisionPackageRevisionRequest,
) -> Result<SecurityDecisionPackageRevisionResult, SecurityDecisionPackageRevisionError> {
    if request.package_path.trim().is_empty() {
        return Err(SecurityDecisionPackageRevisionError::Revision(
            "package_path cannot be empty".to_string(),
        ));
    }

    let previous_package_path = PathBuf::from(request.package_path.trim());
    let previous_package: SecurityDecisionPackageDocument = serde_json::from_slice(
        &fs::read(&previous_package_path)
            .map_err(|error| SecurityDecisionPackageRevisionError::Revision(error.to_string()))?,
    )
    .map_err(|error| SecurityDecisionPackageRevisionError::Revision(error.to_string()))?;

    let updated_artifact_manifest = rebuild_artifact_manifest(&previous_package.artifact_manifest)?;
    let trigger_event_summary = infer_trigger_event_summary(&updated_artifact_manifest);
    let next_version = previous_package.package_version.saturating_add(1);
    let revised_package_path = resolve_revision_package_path(
        &previous_package_path,
        &previous_package.decision_id,
        next_version,
    )?;

    let approval_request_status =
        infer_approval_status(&updated_artifact_manifest).unwrap_or_else(|| "Pending".to_string());
    let revised_package = build_security_decision_package(SecurityDecisionPackageBuildInput {
        created_at: chrono::Utc::now().to_rfc3339(),
        package_version: next_version,
        previous_package_path: Some(previous_package_path.to_string_lossy().to_string()),
        revision_reason: request.revision_reason.trim().to_string(),
        trigger_event_summary: trigger_event_summary.clone(),
        scene_name: previous_package.scene_name.clone(),
        decision_id: previous_package.decision_id.clone(),
        decision_ref: previous_package.decision_ref.clone(),
        approval_ref: previous_package.approval_ref.clone(),
        symbol: previous_package.symbol.clone(),
        analysis_date: previous_package.analysis_date.clone(),
        decision_status: previous_package.package_status.clone(),
        approval_status: approval_request_status,
        // 2026-04-08 CST: 这里沿用上一版 package 的对象图绑定，原因是 Task 1 新增的显式对象图不能在 revision 时丢失；
        // 目的：确保 package 版本升级只更新版本与 manifest，而不破坏已经冻结的正式对象引用。
        position_plan_ref: previous_package.object_graph.position_plan_ref.clone(),
        approval_brief_ref: previous_package.object_graph.approval_brief_ref.clone(),
        scorecard_ref: previous_package.object_graph.scorecard_ref.clone(),
        // 2026-04-10 CST: 这里沿用上一版 package 的 condition_review 锚点，原因是 Task 4 要让投中复核进入正式 package 修订链；
        // 目的：保证 revision 只更新版本与 manifest，不丢失已经绑定的复核引用和复核摘要，便于后续追责与复盘。
        condition_review_ref: previous_package.object_graph.condition_review_ref.clone(),
        condition_review_digest: previous_package.condition_review_digest.clone(),
        decision_card_path: previous_package.object_graph.decision_card_path.clone(),
        approval_request_path: previous_package.object_graph.approval_request_path.clone(),
        position_plan_path: previous_package.object_graph.position_plan_path.clone(),
        approval_brief_path: previous_package.object_graph.approval_brief_path.clone(),
        scorecard_path: previous_package.object_graph.scorecard_path.clone(),
        evidence_hash: previous_package.governance_binding.evidence_hash.clone(),
        governance_hash: previous_package.governance_binding.governance_hash.clone(),
        artifact_manifest: updated_artifact_manifest,
    });

    persist_json(&revised_package_path, &revised_package)?;

    let verification_report_path = if request.reverify_after_revision {
        let verification =
            security_decision_verify_package(&SecurityDecisionVerifyPackageRequest {
                package_path: revised_package_path.to_string_lossy().to_string(),
                approval_brief_signing_key_secret: request
                    .approval_brief_signing_key_secret
                    .clone(),
                approval_brief_signing_key_secret_env: request
                    .approval_brief_signing_key_secret_env
                    .clone(),
                write_report: true,
            })
            .map_err(|error| SecurityDecisionPackageRevisionError::Revision(error.to_string()))?;
        verification.verification_report_path
    } else {
        None
    };

    Ok(SecurityDecisionPackageRevisionResult {
        decision_package: serde_json::to_value(&revised_package)
            .map_err(|error| SecurityDecisionPackageRevisionError::Revision(error.to_string()))?,
        decision_package_path: revised_package_path.to_string_lossy().to_string(),
        package_version: revised_package.package_version,
        previous_package_path: previous_package_path.to_string_lossy().to_string(),
        revision_reason: revised_package.revision_reason.clone(),
        trigger_event_summary,
        verification_report_path,
    })
}

fn rebuild_artifact_manifest(
    previous_artifacts: &[SecurityDecisionPackageArtifact],
) -> Result<Vec<SecurityDecisionPackageArtifact>, SecurityDecisionPackageRevisionError> {
    let mut rebuilt = Vec::new();
    for artifact in previous_artifacts {
        if !artifact.present || artifact.path.trim().is_empty() {
            rebuilt.push(SecurityDecisionPackageArtifact {
                artifact_role: artifact.artifact_role.clone(),
                path: artifact.path.clone(),
                sha256: String::new(),
                contract_version: artifact.contract_version.clone(),
                required: artifact.required,
                present: false,
            });
            continue;
        }

        let payload = fs::read(&artifact.path)
            .map_err(|error| SecurityDecisionPackageRevisionError::Revision(error.to_string()))?;
        let sha256 = compute_manifest_compatible_sha256(artifact, &payload)
            .map_err(SecurityDecisionPackageRevisionError::Revision)?;
        rebuilt.push(SecurityDecisionPackageArtifact {
            artifact_role: artifact.artifact_role.clone(),
            path: artifact.path.clone(),
            sha256,
            contract_version: artifact.contract_version.clone(),
            required: artifact.required,
            present: true,
        });
    }
    Ok(rebuilt)
}

fn compute_manifest_compatible_sha256(
    artifact: &SecurityDecisionPackageArtifact,
    payload: &[u8],
) -> Result<String, String> {
    if artifact.path.ends_with(".json") {
        let value: Value = serde_json::from_slice(payload).map_err(|error| error.to_string())?;
        return sha256_for_json_value(&value);
    }
    Ok(sha256_for_bytes(payload))
}

fn infer_trigger_event_summary(artifacts: &[SecurityDecisionPackageArtifact]) -> String {
    let Some(events_artifact) = artifacts
        .iter()
        .find(|artifact| artifact.artifact_role == "approval_events" && artifact.present)
    else {
        return "approval package revised without approval event summary".to_string();
    };
    let Ok(payload) = fs::read(&events_artifact.path) else {
        return "approval package revised without approval event summary".to_string();
    };
    let Ok(value) = serde_json::from_slice::<Value>(&payload) else {
        return "approval package revised without approval event summary".to_string();
    };
    let Some(last_event) = value.as_array().and_then(|items| items.last()) else {
        return "approval package revised without approval event summary".to_string();
    };
    let reviewer = last_event
        .get("reviewer")
        .and_then(|value| value.as_str())
        .unwrap_or("unknown_reviewer");
    let action = last_event
        .get("action")
        .and_then(|value| value.as_str())
        .unwrap_or("unknown_action");
    let timestamp = last_event
        .get("timestamp")
        .and_then(|value| value.as_str())
        .unwrap_or("unknown_timestamp");
    format!("{reviewer} {action} at {timestamp}")
}

fn infer_approval_status(artifacts: &[SecurityDecisionPackageArtifact]) -> Option<String> {
    let artifact = artifacts
        .iter()
        .find(|artifact| artifact.artifact_role == "approval_request" && artifact.present)?;
    let payload = fs::read(&artifact.path).ok()?;
    let value = serde_json::from_slice::<Value>(&payload).ok()?;
    value.get("status")?.as_str().map(|value| value.to_string())
}

fn resolve_revision_package_path(
    previous_package_path: &Path,
    decision_id: &str,
    next_version: u32,
) -> Result<PathBuf, SecurityDecisionPackageRevisionError> {
    let decision_packages_dir = find_decision_packages_dir(previous_package_path)?;
    let version_dir = decision_packages_dir.join(decision_id);
    Ok(version_dir.join(format!("v{next_version}.json")))
}

fn find_decision_packages_dir(
    package_path: &Path,
) -> Result<PathBuf, SecurityDecisionPackageRevisionError> {
    for ancestor in package_path.ancestors() {
        if ancestor
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name == "decision_packages")
            .unwrap_or(false)
        {
            return Ok(ancestor.to_path_buf());
        }
    }
    Err(SecurityDecisionPackageRevisionError::Revision(
        "failed to locate decision_packages directory from package path".to_string(),
    ))
}

fn persist_json<T: Serialize>(
    path: &Path,
    value: &T,
) -> Result<(), SecurityDecisionPackageRevisionError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| SecurityDecisionPackageRevisionError::Revision(error.to_string()))?;
    }
    let payload = serde_json::to_vec_pretty(value)
        .map_err(|error| SecurityDecisionPackageRevisionError::Revision(error.to_string()))?;
    fs::write(path, payload)
        .map_err(|error| SecurityDecisionPackageRevisionError::Revision(error.to_string()))
}

fn default_revision_reason() -> String {
    "approval_state_transition".to_string()
}

fn default_reverify_after_revision() -> bool {
    true
}
