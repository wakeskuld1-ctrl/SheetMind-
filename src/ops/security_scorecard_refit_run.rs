use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::ops::stock::security_scorecard_model_registry::{
    SecurityScorecardCandidateArtifactInput, SecurityScorecardModelRegistry,
    SecurityScorecardModelRegistryError, build_security_scorecard_model_registry,
    sanitize_identifier,
};

// 2026-04-09 CST: 这里新增 refit Tool 请求合同，原因是 Task 4 需要把“离线重估一次”正式收口成稳定接口，
// 目的：把市场范围、样本窗口、标签版本和 candidate artifact 一次性冻结下来，形成后续训练发布链的最小入口。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityScorecardRefitRequest {
    #[serde(default = "default_created_at")]
    pub created_at: String,
    #[serde(default)]
    pub refit_runtime_root: Option<String>,
    pub market_scope: String,
    pub instrument_scope: String,
    pub feature_set_version: String,
    pub label_definition_version: String,
    pub train_range: String,
    pub valid_range: String,
    pub test_range: String,
    pub candidate_artifact: SecurityScorecardCandidateArtifactInput,
    #[serde(default)]
    pub comparison_to_champion_json: Option<Value>,
    #[serde(default)]
    pub promotion_decision: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityScorecardRefitRun {
    pub refit_run_id: String,
    pub contract_version: String,
    pub document_type: String,
    pub market_scope: String,
    pub instrument_scope: String,
    pub feature_set_version: String,
    pub label_definition_version: String,
    pub train_range: String,
    pub valid_range: String,
    pub test_range: String,
    pub candidate_artifact_path: String,
    #[serde(default)]
    pub candidate_registry_ref: Option<String>,
    #[serde(default)]
    pub comparison_to_champion_json: Option<Value>,
    #[serde(default)]
    pub promotion_decision: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SecurityScorecardRefitResult {
    pub refit_run: SecurityScorecardRefitRun,
    pub model_registry: SecurityScorecardModelRegistry,
    pub refit_run_path: String,
    pub model_registry_path: String,
}

#[derive(Debug, Error)]
pub enum SecurityScorecardRefitError {
    #[error("security scorecard refit build failed: {0}")]
    Build(String),
    #[error("security scorecard model registry build failed: {0}")]
    Registry(#[from] SecurityScorecardModelRegistryError),
    #[error("security scorecard refit persist failed: {0}")]
    Persist(String),
}

pub fn security_scorecard_refit(
    request: &SecurityScorecardRefitRequest,
) -> Result<SecurityScorecardRefitResult, SecurityScorecardRefitError> {
    validate_request(request)?;

    let model_registry = build_security_scorecard_model_registry(
        &request.market_scope,
        &request.instrument_scope,
        &request.train_range,
        &request.valid_range,
        &request.test_range,
        &request.candidate_artifact,
    )?;
    let refit_run = build_security_scorecard_refit_run(request, &model_registry);

    let runtime_root = resolve_runtime_root(request);
    let refit_run_path = runtime_root.join("scorecard_refit_runs").join(format!(
        "{}.json",
        sanitize_identifier(&refit_run.refit_run_id)
    ));
    let model_registry_path = runtime_root.join("scorecard_model_registry").join(format!(
        "{}__{}.json",
        sanitize_identifier(&model_registry.model_id),
        sanitize_identifier(&model_registry.model_version)
    ));

    persist_json(&refit_run_path, &refit_run)?;
    persist_json(&model_registry_path, &model_registry)?;

    Ok(SecurityScorecardRefitResult {
        refit_run,
        model_registry,
        refit_run_path: refit_run_path.to_string_lossy().to_string(),
        model_registry_path: model_registry_path.to_string_lossy().to_string(),
    })
}

fn build_security_scorecard_refit_run(
    request: &SecurityScorecardRefitRequest,
    model_registry: &SecurityScorecardModelRegistry,
) -> SecurityScorecardRefitRun {
    SecurityScorecardRefitRun {
        refit_run_id: format!(
            "refit-{}-{}-{}",
            sanitize_identifier(&request.market_scope),
            sanitize_identifier(&request.instrument_scope),
            sanitize_identifier(&request.created_at),
        ),
        contract_version: "security_scorecard_refit_run.v1".to_string(),
        document_type: "security_scorecard_refit_run".to_string(),
        market_scope: request.market_scope.clone(),
        instrument_scope: request.instrument_scope.clone(),
        feature_set_version: request.feature_set_version.clone(),
        label_definition_version: request.label_definition_version.clone(),
        train_range: request.train_range.clone(),
        valid_range: request.valid_range.clone(),
        test_range: request.test_range.clone(),
        candidate_artifact_path: request.candidate_artifact.artifact_path.clone(),
        candidate_registry_ref: Some(model_registry.registry_id.clone()),
        comparison_to_champion_json: request.comparison_to_champion_json.clone(),
        promotion_decision: request.promotion_decision.clone(),
        created_at: request.created_at.clone(),
    }
}

fn validate_request(
    request: &SecurityScorecardRefitRequest,
) -> Result<(), SecurityScorecardRefitError> {
    for (field_name, field_value) in [
        ("market_scope", request.market_scope.trim()),
        ("instrument_scope", request.instrument_scope.trim()),
        ("feature_set_version", request.feature_set_version.trim()),
        (
            "label_definition_version",
            request.label_definition_version.trim(),
        ),
        ("train_range", request.train_range.trim()),
        ("valid_range", request.valid_range.trim()),
        ("test_range", request.test_range.trim()),
    ] {
        if field_value.is_empty() {
            return Err(SecurityScorecardRefitError::Build(format!(
                "{field_name} cannot be empty"
            )));
        }
    }
    Ok(())
}

fn persist_json<T: Serialize>(path: &Path, value: &T) -> Result<(), SecurityScorecardRefitError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| SecurityScorecardRefitError::Persist(error.to_string()))?;
    }
    let payload = serde_json::to_vec_pretty(value)
        .map_err(|error| SecurityScorecardRefitError::Persist(error.to_string()))?;
    fs::write(path, payload)
        .map_err(|error| SecurityScorecardRefitError::Persist(error.to_string()))
}

fn resolve_runtime_root(request: &SecurityScorecardRefitRequest) -> PathBuf {
    request
        .refit_runtime_root
        .as_ref()
        .map(|path| PathBuf::from(path.trim()))
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| {
            PathBuf::from(".worktrees")
                .join("SheetMind-Scenes-inspect")
                .join(".sheetmind_scenes_runtime")
        })
}

fn default_created_at() -> String {
    chrono::Utc::now().to_rfc3339()
}
