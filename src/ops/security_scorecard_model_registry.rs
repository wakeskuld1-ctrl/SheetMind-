use std::fs;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use thiserror::Error;

// 2026-04-09 CST: 这里新增 scorecard model registry 正式合同，原因是 Task 4 需要把 candidate artifact 从“临时文件”升级为正式注册对象，
// 目的：让后续训练入口、晋级治理和线上消费都能基于稳定字段追踪模型来源。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityScorecardModelRegistry {
    pub registry_id: String,
    pub contract_version: String,
    pub document_type: String,
    pub model_id: String,
    pub market_scope: String,
    pub instrument_scope: String,
    pub horizon_days: usize,
    pub target_head: String,
    pub model_version: String,
    pub status: String,
    pub training_window: String,
    pub validation_window: String,
    pub oot_window: String,
    pub artifact_path: String,
    pub artifact_sha256: String,
    pub metrics_summary_json: Value,
    pub published_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityScorecardCandidateArtifactInput {
    pub model_id: String,
    pub model_version: String,
    pub horizon_days: usize,
    pub target_head: String,
    #[serde(default = "default_registry_status")]
    pub status: String,
    pub artifact_path: String,
    #[serde(default)]
    pub metrics_summary_json: Value,
    #[serde(default)]
    pub published_at: Option<String>,
}

#[derive(Debug, Error)]
pub enum SecurityScorecardModelRegistryError {
    #[error("security scorecard model registry build failed: {0}")]
    Build(String),
}

pub fn build_security_scorecard_model_registry(
    market_scope: &str,
    instrument_scope: &str,
    training_window: &str,
    validation_window: &str,
    oot_window: &str,
    candidate: &SecurityScorecardCandidateArtifactInput,
) -> Result<SecurityScorecardModelRegistry, SecurityScorecardModelRegistryError> {
    let artifact_path = candidate.artifact_path.trim();
    if artifact_path.is_empty() {
        return Err(SecurityScorecardModelRegistryError::Build(
            "candidate artifact path cannot be empty".to_string(),
        ));
    }

    let artifact_payload = fs::read(artifact_path).map_err(|error| {
        SecurityScorecardModelRegistryError::Build(format!(
            "failed to read candidate artifact `{artifact_path}`: {error}"
        ))
    })?;
    let artifact_sha256 = sha256_for_bytes(&artifact_payload);

    Ok(SecurityScorecardModelRegistry {
        registry_id: format!(
            "registry-{}-{}-{}d-{}",
            sanitize_identifier(&candidate.model_id),
            sanitize_identifier(&candidate.model_version),
            candidate.horizon_days,
            sanitize_identifier(&candidate.target_head),
        ),
        contract_version: "security_scorecard_model_registry.v1".to_string(),
        document_type: "security_scorecard_model_registry".to_string(),
        model_id: candidate.model_id.clone(),
        market_scope: market_scope.to_string(),
        instrument_scope: instrument_scope.to_string(),
        horizon_days: candidate.horizon_days,
        target_head: candidate.target_head.clone(),
        model_version: candidate.model_version.clone(),
        status: candidate.status.clone(),
        training_window: training_window.to_string(),
        validation_window: validation_window.to_string(),
        oot_window: oot_window.to_string(),
        artifact_path: artifact_path.to_string(),
        artifact_sha256,
        metrics_summary_json: candidate.metrics_summary_json.clone(),
        published_at: candidate.published_at.clone(),
    })
}

fn sha256_for_bytes(payload: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(payload);
    format!("{:x}", hasher.finalize())
}

pub fn sanitize_identifier(raw: &str) -> String {
    raw.chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' => ch,
            _ => '_',
        })
        .collect()
}

fn default_registry_status() -> String {
    "candidate".to_string()
}
