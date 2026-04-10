use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::ops::stock::security_decision_evidence_bundle::{
    SecurityDecisionEvidenceBundleError, SecurityDecisionEvidenceBundleRequest,
    build_evidence_bundle_feature_seed, security_decision_evidence_bundle,
};

// 2026-04-09 CST: 这里新增特征快照请求合同，原因是 Task 2 要把“分析时点可见特征冻结”独立成正式 Tool，
// 目的：让后续训练 / 回算 / 主席线都能从统一入口拿到稳定快照，而不是每次临时拼字段。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityFeatureSnapshotRequest {
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
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityFeatureSnapshot {
    pub snapshot_id: String,
    pub contract_version: String,
    pub document_type: String,
    pub symbol: String,
    pub market: String,
    pub instrument_type: String,
    pub as_of_date: String,
    pub data_cutoff_at: String,
    pub feature_set_version: String,
    pub raw_features_json: BTreeMap<String, Value>,
    pub group_features_json: BTreeMap<String, Value>,
    pub data_quality_flags: Vec<String>,
    pub snapshot_hash: String,
}

#[derive(Debug, Error)]
pub enum SecurityFeatureSnapshotError {
    #[error("security feature snapshot evidence preparation failed: {0}")]
    Evidence(#[from] SecurityDecisionEvidenceBundleError),
    #[error("security feature snapshot build failed: {0}")]
    Build(String),
}

pub fn security_feature_snapshot(
    request: &SecurityFeatureSnapshotRequest,
) -> Result<SecurityFeatureSnapshot, SecurityFeatureSnapshotError> {
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
    let raw_features_json = build_evidence_bundle_feature_seed(&evidence_bundle);
    let group_features_json = build_group_features(request, &raw_features_json);
    let data_quality_flags = build_data_quality_flags(&evidence_bundle);
    let snapshot_hash = build_snapshot_hash(
        &request.symbol,
        &evidence_bundle.analysis_date,
        &raw_features_json,
        &group_features_json,
        &data_quality_flags,
    )?;

    Ok(SecurityFeatureSnapshot {
        snapshot_id: format!(
            "snapshot-{}-{}",
            request.symbol, evidence_bundle.analysis_date
        ),
        contract_version: "security_feature_snapshot.v1".to_string(),
        document_type: "security_feature_snapshot".to_string(),
        symbol: request.symbol.clone(),
        market: derive_market(&request.symbol),
        instrument_type: derive_instrument_type(&request.symbol),
        as_of_date: evidence_bundle.analysis_date.clone(),
        data_cutoff_at: evidence_bundle.analysis_date.clone(),
        feature_set_version: "security_feature_snapshot.v1".to_string(),
        raw_features_json,
        group_features_json,
        data_quality_flags,
        snapshot_hash,
    })
}

fn build_group_features(
    request: &SecurityFeatureSnapshotRequest,
    raw_features_json: &BTreeMap<String, Value>,
) -> BTreeMap<String, Value> {
    let mut groups = BTreeMap::new();
    groups.insert("M".to_string(), json!({
        "market_profile": request.market_profile.clone().unwrap_or_else(|| "unknown".to_string()),
        "integrated_stance": raw_features_json.get("integrated_stance").cloned().unwrap_or(Value::Null),
        "technical_alignment": raw_features_json.get("technical_alignment").cloned().unwrap_or(Value::Null),
    }));
    groups.insert("F".to_string(), json!({
        "fundamental_status": raw_features_json.get("fundamental_status").cloned().unwrap_or(Value::Null),
        "fundamental_available": raw_features_json.get("fundamental_available").cloned().unwrap_or(Value::Null),
    }));
    groups.insert(
        "V".to_string(),
        json!({ "valuation_status": "not_populated_v1" }),
    );
    groups.insert("T".to_string(), json!({
        "technical_alignment": raw_features_json.get("technical_alignment").cloned().unwrap_or(Value::Null),
        "technical_status": raw_features_json.get("technical_status").cloned().unwrap_or(Value::Null),
    }));
    groups.insert(
        "Q".to_string(),
        json!({ "flow_status": "not_populated_v1" }),
    );
    groups.insert("E".to_string(), json!({
        "disclosure_status": raw_features_json.get("disclosure_status").cloned().unwrap_or(Value::Null),
        "disclosure_available": raw_features_json.get("disclosure_available").cloned().unwrap_or(Value::Null),
    }));
    groups.insert("R".to_string(), json!({
        "overall_evidence_status": raw_features_json.get("overall_evidence_status").cloned().unwrap_or(Value::Null),
        "data_gap_count": raw_features_json.get("data_gap_count").cloned().unwrap_or(Value::Null),
        "risk_note_count": raw_features_json.get("risk_note_count").cloned().unwrap_or(Value::Null),
    }));
    groups.insert(
        "X".to_string(),
        json!({ "trading_structure_status": "not_populated_v1" }),
    );
    groups
}

fn build_data_quality_flags(
    evidence_bundle: &crate::ops::stock::security_decision_evidence_bundle::SecurityDecisionEvidenceBundleResult,
) -> Vec<String> {
    let mut flags = Vec::new();
    flags.push(format!(
        "overall_status:{}",
        evidence_bundle.evidence_quality.overall_status
    ));
    flags.extend(
        evidence_bundle
            .data_gaps
            .iter()
            .map(|gap| format!("data_gap:{gap}")),
    );
    flags.extend(
        evidence_bundle
            .evidence_quality
            .risk_flags
            .iter()
            .take(4)
            .map(|flag| format!("risk_flag:{flag}")),
    );
    dedupe_strings(&mut flags);
    flags
}

fn build_snapshot_hash(
    symbol: &str,
    as_of_date: &str,
    raw_features_json: &BTreeMap<String, Value>,
    group_features_json: &BTreeMap<String, Value>,
    data_quality_flags: &[String],
) -> Result<String, SecurityFeatureSnapshotError> {
    let payload = json!({
        "symbol": symbol,
        "as_of_date": as_of_date,
        "raw_features_json": raw_features_json,
        "group_features_json": group_features_json,
        "data_quality_flags": data_quality_flags,
    });
    let bytes = serde_json::to_vec(&payload)
        .map_err(|error| SecurityFeatureSnapshotError::Build(error.to_string()))?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(format!("snapshot-{:x}", hasher.finalize()))
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

fn derive_market(symbol: &str) -> String {
    if symbol.ends_with(".SH") || symbol.ends_with(".SZ") {
        "A_SHARE".to_string()
    } else {
        "UNKNOWN".to_string()
    }
}

fn derive_instrument_type(symbol: &str) -> String {
    let code = symbol.split('.').next().unwrap_or_default();
    if code.starts_with('5') || code.starts_with('1') {
        "ETF".to_string()
    } else {
        "EQUITY".to_string()
    }
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
