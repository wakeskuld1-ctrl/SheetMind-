use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::ops::stock::security_approval_brief_signature::{
    verify_security_approval_brief_document, SecurityApprovalBriefSignatureEnvelope,
};
use crate::ops::stock::security_decision_approval_brief::SecurityDecisionApprovalBrief;
use crate::ops::stock::security_decision_approval_bridge::{
    PersistedApprovalRequest, PersistedDecisionCard,
};
use crate::ops::stock::security_decision_package::{
    sha256_for_bytes, SecurityDecisionPackageArtifact, SecurityDecisionPackageDocument,
};
use crate::ops::stock::security_post_meeting_conclusion::SecurityPostMeetingConclusion;

// 2026-04-02 CST: 这里定义证券审批包校验请求，原因是 P0-5 需要一个正式 Tool 来执行 package 路径、签名 secret 和报告落盘策略；
// 目的：把 verify 所需的最小输入参数收口到稳定合同，避免调用方手工拼接内部校验细节。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityDecisionVerifyPackageRequest {
    pub package_path: String,
    #[serde(default)]
    pub approval_brief_signing_key_secret: Option<String>,
    #[serde(default)]
    pub approval_brief_signing_key_secret_env: Option<String>,
    #[serde(default = "default_write_report")]
    pub write_report: bool,
}

// 2026-04-02 CST: 这里定义证券审批包校验结果，原因是调用方需要同时拿到报告正文与落盘路径；
// 目的：让 CLI / Skill / 后续审批治理都能一次获得“是否有效、为什么、报告在哪”。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SecurityDecisionVerifyPackageResult {
    pub report_id: String,
    pub contract_version: String,
    pub generated_at: String,
    pub package_path: String,
    pub package_id: String,
    pub decision_ref: String,
    pub approval_ref: String,
    pub package_valid: bool,
    pub artifact_checks: Vec<SecurityDecisionPackageArtifactCheck>,
    pub hash_checks: Vec<SecurityDecisionPackageHashCheck>,
    pub signature_checks: Vec<SecurityDecisionPackageSignatureCheck>,
    pub governance_checks: SecurityDecisionPackageGovernanceCheck,
    pub issues: Vec<String>,
    pub recommended_action: String,
    pub verification_report_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SecurityDecisionPackageArtifactCheck {
    pub artifact_role: String,
    pub path: String,
    pub required: bool,
    pub present: bool,
    pub exists_on_disk: bool,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SecurityDecisionPackageHashCheck {
    pub artifact_role: String,
    pub manifest_sha256: String,
    pub actual_sha256: String,
    pub matched: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SecurityDecisionPackageSignatureCheck {
    pub artifact_role: String,
    pub algorithm: String,
    pub key_id: String,
    pub payload_sha256_matched: bool,
    pub signature_valid: bool,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SecurityDecisionPackageGovernanceCheck {
    pub decision_ref_matched: bool,
    pub approval_ref_matched: bool,
    pub evidence_hash_matched: bool,
    pub governance_hash_matched: bool,
    // 2026-04-08 CST: 这里新增会后结论校验位，原因是 Task 11 已经把 post_meeting_conclusion 正式挂进 package；
    // 目的：让 verify 能明确区分“绑定一致”“brief 配对一致”“关键字段完整”三种治理语义，而不是只停留在文件存在性层面。
    pub post_meeting_conclusion_binding_consistent: bool,
    pub post_meeting_conclusion_brief_paired: bool,
    pub post_meeting_conclusion_complete: bool,
}

// 2026-04-02 CST: 这里定义校验错误边界，原因是 verify 阶段既可能失败在路径解析，也可能失败在落盘；
// 目的：让 dispatcher 继续拿到单一错误口径，同时把“包本身无效”和“工具执行失败”分开。
#[derive(Debug, thiserror::Error)]
pub enum SecurityDecisionVerifyPackageError {
    #[error("证券审批包校验执行失败: {0}")]
    Verify(String),
}

// 2026-04-02 CST: 这里实现正式证券审批包校验入口，原因是 P0-5 要把 package 从“可生成”升级成“可核验”；
// 目的：统一执行 manifest、哈希、签名与治理绑定校验，并生成正式 verification report。
pub fn security_decision_verify_package(
    request: &SecurityDecisionVerifyPackageRequest,
) -> Result<SecurityDecisionVerifyPackageResult, SecurityDecisionVerifyPackageError> {
    let package_path = PathBuf::from(request.package_path.trim());
    if request.package_path.trim().is_empty() {
        return Err(SecurityDecisionVerifyPackageError::Verify(
            "package_path cannot be empty".to_string(),
        ));
    }

    let package: SecurityDecisionPackageDocument = serde_json::from_slice(
        &fs::read(&package_path)
            .map_err(|error| SecurityDecisionVerifyPackageError::Verify(error.to_string()))?,
    )
    .map_err(|error| SecurityDecisionVerifyPackageError::Verify(error.to_string()))?;

    let mut issues = Vec::new();
    let artifact_checks = build_artifact_checks(&package.artifact_manifest, &mut issues);
    let hash_checks = build_hash_checks(&package.artifact_manifest, &mut issues);
    let signature_checks = build_signature_checks(
        &package.artifact_manifest,
        request,
        &mut issues,
    )?;
    let governance_checks =
        build_governance_checks(&package, &package.artifact_manifest, &mut issues)?;
    let package_valid = issues.is_empty();
    let recommended_action = recommend_action(package_valid, &artifact_checks, &signature_checks);

    let mut result = SecurityDecisionVerifyPackageResult {
        report_id: format!("verification-{}", package.decision_id),
        contract_version: "security_decision_package_verification.v1".to_string(),
        generated_at: Utc::now().to_rfc3339(),
        package_path: package_path.to_string_lossy().to_string(),
        package_id: package.package_id.clone(),
        decision_ref: package.decision_ref.clone(),
        approval_ref: package.approval_ref.clone(),
        package_valid,
        artifact_checks,
        hash_checks,
        signature_checks,
        governance_checks,
        issues,
        recommended_action,
        verification_report_path: None,
    };

    if request.write_report {
        let report_path = resolve_verification_report_path(&package_path, &package.decision_id)?;
        persist_json(&report_path, &result)?;
        result.verification_report_path = Some(report_path.to_string_lossy().to_string());
    }

    Ok(result)
}

fn build_artifact_checks(
    artifacts: &[SecurityDecisionPackageArtifact],
    issues: &mut Vec<String>,
) -> Vec<SecurityDecisionPackageArtifactCheck> {
    let mut checks = Vec::new();
    for artifact in artifacts {
        let exists_on_disk = artifact.present
            && !artifact.path.trim().is_empty()
            && Path::new(&artifact.path).exists();
        let (status, message) = if artifact.required && !artifact.present {
            issues.push(format!("required artifact `{}` is not present", artifact.artifact_role));
            ("failed".to_string(), "required artifact missing from manifest".to_string())
        } else if artifact.present && !exists_on_disk {
            issues.push(format!(
                "artifact `{}` expected at `{}` but file does not exist",
                artifact.artifact_role, artifact.path
            ));
            ("failed".to_string(), "artifact file missing on disk".to_string())
        } else if !artifact.present && !artifact.required {
            ("warning".to_string(), "optional artifact not present".to_string())
        } else {
            ("passed".to_string(), "artifact present on disk".to_string())
        };
        checks.push(SecurityDecisionPackageArtifactCheck {
            artifact_role: artifact.artifact_role.clone(),
            path: artifact.path.clone(),
            required: artifact.required,
            present: artifact.present,
            exists_on_disk,
            status,
            message,
        });
    }
    checks
}

fn build_hash_checks(
    artifacts: &[SecurityDecisionPackageArtifact],
    issues: &mut Vec<String>,
) -> Vec<SecurityDecisionPackageHashCheck> {
    let mut checks = Vec::new();
    for artifact in artifacts {
        if !artifact.present || artifact.path.trim().is_empty() {
            continue;
        }
        let actual_sha256 = match fs::read(&artifact.path) {
            Ok(payload) => compute_manifest_compatible_sha256(artifact, &payload),
            Err(_) => String::new(),
        };
        let matched = !actual_sha256.is_empty() && actual_sha256 == artifact.sha256;
        if !matched {
            issues.push(format!(
                "artifact `{}` sha256 mismatch or unreadable",
                artifact.artifact_role
            ));
        }
        checks.push(SecurityDecisionPackageHashCheck {
            artifact_role: artifact.artifact_role.clone(),
            manifest_sha256: artifact.sha256.clone(),
            actual_sha256,
            matched,
        });
    }
    checks
}

// 2026-04-02 CST: 这里把 verify 阶段的哈希口径对齐到 package manifest，原因是 manifest 中 JSON 工件的哈希来自结构化 payload 而不是 pretty 文件字节；
// 目的：让 happy path 既能准确复现提交时摘要，又不会因为格式化空白差异产生误报；篡改内容时仍会稳定失配。
fn compute_manifest_compatible_sha256(
    artifact: &SecurityDecisionPackageArtifact,
    payload: &[u8],
) -> String {
    if artifact.path.ends_with(".json") {
        if let Ok(value) = serde_json::from_slice::<serde_json::Value>(payload) {
            if let Ok(sha256) = crate::ops::stock::security_decision_package::sha256_for_json_value(&value) {
                return sha256;
            }
        }
    }
    sha256_for_bytes(payload)
}

fn build_signature_checks(
    artifacts: &[SecurityDecisionPackageArtifact],
    request: &SecurityDecisionVerifyPackageRequest,
    issues: &mut Vec<String>,
) -> Result<Vec<SecurityDecisionPackageSignatureCheck>, SecurityDecisionVerifyPackageError> {
    let signature_artifact = artifacts
        .iter()
        .find(|artifact| artifact.artifact_role == "approval_brief_signature");
    let approval_brief_artifact = artifacts
        .iter()
        .find(|artifact| artifact.artifact_role == "approval_brief");

    let Some(signature_artifact) = signature_artifact else {
        return Ok(Vec::new());
    };
    if !signature_artifact.present {
        return Ok(vec![SecurityDecisionPackageSignatureCheck {
            artifact_role: "approval_brief_signature".to_string(),
            algorithm: "hmac_sha256".to_string(),
            key_id: String::new(),
            payload_sha256_matched: true,
            signature_valid: true,
            message: "optional signature artifact not present".to_string(),
        }]);
    }

    let Some(approval_brief_artifact) = approval_brief_artifact else {
        issues.push("approval_brief_signature exists but approval_brief artifact is missing".to_string());
        return Ok(vec![SecurityDecisionPackageSignatureCheck {
            artifact_role: "approval_brief_signature".to_string(),
            algorithm: "hmac_sha256".to_string(),
            key_id: String::new(),
            payload_sha256_matched: false,
            signature_valid: false,
            message: "approval brief artifact missing".to_string(),
        }]);
    };

    let secret = resolve_optional_signing_secret(request)?;
    let brief: SecurityDecisionApprovalBrief = serde_json::from_slice(
        &fs::read(&approval_brief_artifact.path)
            .map_err(|error| SecurityDecisionVerifyPackageError::Verify(error.to_string()))?,
    )
    .map_err(|error| SecurityDecisionVerifyPackageError::Verify(error.to_string()))?;
    let envelope: SecurityApprovalBriefSignatureEnvelope = serde_json::from_slice(
        &fs::read(&signature_artifact.path)
            .map_err(|error| SecurityDecisionVerifyPackageError::Verify(error.to_string()))?,
    )
    .map_err(|error| SecurityDecisionVerifyPackageError::Verify(error.to_string()))?;

    let verification = verify_security_approval_brief_document(&brief, &envelope, &secret);
    let (payload_sha256_matched, signature_valid, message) = match verification {
        Ok(()) => (true, true, "approval brief detached signature verified".to_string()),
        Err(error) => {
            issues.push(format!("approval brief detached signature verification failed: {error}"));
            let payload_matches = !error.contains("payload sha256 mismatch");
            (payload_matches, false, error)
        }
    };

    Ok(vec![SecurityDecisionPackageSignatureCheck {
        artifact_role: "approval_brief_signature".to_string(),
        algorithm: envelope.algorithm.clone(),
        key_id: envelope.key_id.clone(),
        payload_sha256_matched,
        signature_valid,
        message,
    }])
}

fn build_governance_checks(
    package: &SecurityDecisionPackageDocument,
    artifacts: &[SecurityDecisionPackageArtifact],
    issues: &mut Vec<String>,
) -> Result<SecurityDecisionPackageGovernanceCheck, SecurityDecisionVerifyPackageError> {
    let decision_card = load_optional_json::<PersistedDecisionCard>(artifacts, "decision_card")?;
    let approval_request =
        load_optional_json::<PersistedApprovalRequest>(artifacts, "approval_request")?;
    let approval_brief =
        load_optional_json::<SecurityDecisionApprovalBrief>(artifacts, "approval_brief")?;
    let post_meeting_conclusion =
        load_optional_json::<SecurityPostMeetingConclusion>(artifacts, "post_meeting_conclusion")?;

    let decision_ref_matched = decision_card
        .as_ref()
        .map(|card| card.decision_ref == package.decision_ref)
        .unwrap_or(false)
        && approval_request
            .as_ref()
            .and_then(|request| request.decision_ref.clone())
            .map(|value| value == package.decision_ref)
            .unwrap_or(false)
        && approval_brief
            .as_ref()
            .map(|brief| brief.decision_ref == package.decision_ref)
            .unwrap_or(false);
    if !decision_ref_matched {
        issues.push("governance decision_ref mismatch across package artifacts".to_string());
    }

    let approval_ref_matched = approval_request
        .as_ref()
        .map(|request| request.approval_ref == package.approval_ref)
        .unwrap_or(false)
        && approval_brief
            .as_ref()
            .map(|brief| brief.approval_ref == package.approval_ref)
            .unwrap_or(false);
    if !approval_ref_matched {
        issues.push("governance approval_ref mismatch across package artifacts".to_string());
    }

    let evidence_hash_matched = approval_request
        .as_ref()
        .and_then(|request| request.evidence_hash.clone())
        .map(|value| value == package.governance_binding.evidence_hash)
        .unwrap_or(false)
        && approval_brief
            .as_ref()
            .map(|brief| brief.evidence_hash == package.governance_binding.evidence_hash)
            .unwrap_or(false);
    if !evidence_hash_matched {
        issues.push("governance evidence_hash mismatch across package artifacts".to_string());
    }

    let governance_hash_matched = approval_request
        .as_ref()
        .and_then(|request| request.governance_hash.clone())
        .map(|value| value == package.governance_binding.governance_hash)
        .unwrap_or(false)
        && approval_brief
            .as_ref()
            .map(|brief| brief.governance_hash == package.governance_binding.governance_hash)
            .unwrap_or(false);
    if !governance_hash_matched {
        issues.push("governance governance_hash mismatch across package artifacts".to_string());
    }

    let has_post_meeting_binding = package
        .object_graph
        .post_meeting_conclusion_ref
        .as_ref()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
        || package
            .object_graph
            .post_meeting_conclusion_path
            .as_ref()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
        || artifact_is_present(artifacts, "post_meeting_conclusion");

    let post_meeting_conclusion_binding_consistent = if !has_post_meeting_binding {
        true
    } else {
        post_meeting_conclusion
            .as_ref()
            .map(|conclusion| {
                package
                    .object_graph
                    .post_meeting_conclusion_ref
                    .as_ref()
                    .map(|value| value == &conclusion.conclusion_id)
                    .unwrap_or(false)
                    && package
                        .object_graph
                        .post_meeting_conclusion_path
                        .as_ref()
                        .map(|value| value == &find_artifact_path(artifacts, "post_meeting_conclusion"))
                        .unwrap_or(false)
                    && conclusion.decision_ref == package.decision_ref
                    && conclusion.approval_ref == package.approval_ref
                    && conclusion.governance_binding.decision_ref == package.decision_ref
                    && conclusion.governance_binding.approval_ref == package.approval_ref
            })
            .unwrap_or(false)
    };
    if !post_meeting_conclusion_binding_consistent {
        issues.push("post_meeting_conclusion binding mismatch across package artifacts".to_string());
    }

    let post_meeting_conclusion_brief_paired = if !has_post_meeting_binding {
        true
    } else {
        post_meeting_conclusion
            .as_ref()
            .map(|conclusion| {
                conclusion.source_brief_ref == package.object_graph.approval_brief_ref
                    && conclusion.brief_pairing.pre_meeting_brief_ref
                        == package.object_graph.approval_brief_ref
                    && conclusion.brief_pairing.pre_meeting_brief_path
                        == package.object_graph.approval_brief_path
            })
            .unwrap_or(false)
    };
    if !post_meeting_conclusion_brief_paired {
        issues.push("post_meeting_conclusion brief pairing mismatch".to_string());
    }

    let post_meeting_conclusion_complete = if !has_post_meeting_binding {
        true
    } else {
        post_meeting_conclusion
            .as_ref()
            .map(|conclusion| {
                !conclusion.conclusion_id.trim().is_empty()
                    && !conclusion.source_package_path.trim().is_empty()
                    && !conclusion.source_brief_ref.trim().is_empty()
                    && !conclusion.decision_ref.trim().is_empty()
                    && !conclusion.approval_ref.trim().is_empty()
                    && !conclusion.governance_binding.source_package_path.trim().is_empty()
                    && !conclusion.brief_pairing.pre_meeting_brief_ref.trim().is_empty()
                    && !conclusion.brief_pairing.pre_meeting_brief_path.trim().is_empty()
                    && supported_post_meeting_disposition(&conclusion.final_disposition)
            })
            .unwrap_or(false)
    };
    if !post_meeting_conclusion_complete {
        issues.push("post_meeting_conclusion formal contract is incomplete".to_string());
    }

    Ok(SecurityDecisionPackageGovernanceCheck {
        decision_ref_matched,
        approval_ref_matched,
        evidence_hash_matched,
        governance_hash_matched,
        post_meeting_conclusion_binding_consistent,
        post_meeting_conclusion_brief_paired,
        post_meeting_conclusion_complete,
    })
}

fn artifact_is_present(artifacts: &[SecurityDecisionPackageArtifact], artifact_role: &str) -> bool {
    artifacts
        .iter()
        .any(|artifact| artifact.artifact_role == artifact_role && artifact.present)
}

fn find_artifact_path(artifacts: &[SecurityDecisionPackageArtifact], artifact_role: &str) -> String {
    artifacts
        .iter()
        .find(|artifact| artifact.artifact_role == artifact_role && artifact.present)
        .map(|artifact| artifact.path.clone())
        .unwrap_or_default()
}

fn supported_post_meeting_disposition(value: &str) -> bool {
    matches!(
        value.trim(),
        "approve" | "reject" | "needs_more_evidence" | "approve_with_override"
    )
}

fn load_optional_json<T: for<'de> Deserialize<'de>>(
    artifacts: &[SecurityDecisionPackageArtifact],
    artifact_role: &str,
) -> Result<Option<T>, SecurityDecisionVerifyPackageError> {
    let Some(artifact) = artifacts
        .iter()
        .find(|artifact| artifact.artifact_role == artifact_role)
    else {
        return Ok(None);
    };
    if !artifact.present || artifact.path.trim().is_empty() {
        return Ok(None);
    }

    let payload = fs::read(&artifact.path)
        .map_err(|error| SecurityDecisionVerifyPackageError::Verify(error.to_string()))?;
    let value = serde_json::from_slice(&payload)
        .map_err(|error| SecurityDecisionVerifyPackageError::Verify(error.to_string()))?;
    Ok(Some(value))
}

fn recommend_action(
    package_valid: bool,
    artifact_checks: &[SecurityDecisionPackageArtifactCheck],
    signature_checks: &[SecurityDecisionPackageSignatureCheck],
) -> String {
    if !package_valid {
        return "quarantine_and_rebuild".to_string();
    }
    let has_optional_warning = artifact_checks.iter().any(|item| item.status == "warning")
        || signature_checks.iter().any(|item| !item.signature_valid);
    if has_optional_warning {
        "review_with_warning".to_string()
    } else {
        "proceed_with_review".to_string()
    }
}

fn resolve_optional_signing_secret(
    request: &SecurityDecisionVerifyPackageRequest,
) -> Result<String, SecurityDecisionVerifyPackageError> {
    if let Some(secret) = request.approval_brief_signing_key_secret.as_ref() {
        if !secret.trim().is_empty() {
            return Ok(secret.trim().to_string());
        }
        return Err(SecurityDecisionVerifyPackageError::Verify(
            "approval brief verification secret cannot be empty".to_string(),
        ));
    }

    if let Some(env_key) = request.approval_brief_signing_key_secret_env.as_ref() {
        let value = std::env::var(env_key)
            .map_err(|error| SecurityDecisionVerifyPackageError::Verify(error.to_string()))?;
        if !value.trim().is_empty() {
            return Ok(value);
        }
    }

    Err(SecurityDecisionVerifyPackageError::Verify(
        "approval brief signature exists but no verification secret was provided".to_string(),
    ))
}

fn resolve_verification_report_path(
    package_path: &Path,
    decision_id: &str,
) -> Result<PathBuf, SecurityDecisionVerifyPackageError> {
    let runtime_root = package_path
        .parent()
        .and_then(|path| path.parent())
        .ok_or_else(|| {
            SecurityDecisionVerifyPackageError::Verify(
                "failed to derive runtime root from package path".to_string(),
            )
        })?;
    Ok(runtime_root
        .join("decision_packages_verification")
        .join(format!("{decision_id}.verification.json")))
}

fn persist_json<T: Serialize>(
    path: &Path,
    value: &T,
) -> Result<(), SecurityDecisionVerifyPackageError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| SecurityDecisionVerifyPackageError::Verify(error.to_string()))?;
    }
    let payload = serde_json::to_vec_pretty(value)
        .map_err(|error| SecurityDecisionVerifyPackageError::Verify(error.to_string()))?;
    fs::write(path, payload)
        .map_err(|error| SecurityDecisionVerifyPackageError::Verify(error.to_string()))
}

fn default_write_report() -> bool {
    true
}
