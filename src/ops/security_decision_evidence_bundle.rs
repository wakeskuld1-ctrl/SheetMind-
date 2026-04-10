use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use thiserror::Error;

use crate::ops::stock::security_analysis_contextual::SecurityAnalysisContextualResult;
use crate::ops::stock::security_analysis_fullstack::{
    DisclosureContext, FundamentalContext, IndustryContext, IntegratedConclusion,
    SecurityAnalysisFullstackError, SecurityAnalysisFullstackRequest,
    SecurityAnalysisFullstackResult, security_analysis_fullstack,
};

// 2026-04-09 CST: 这里新增正式证据包请求合同，原因是 Task 1-2 的 committee 与 snapshot 都不能再直接读取 fullstack 临时结果；
// 目的：把证券研究链冻结成稳定中间层，后续 chair / snapshot / training 都围绕这一层取数，避免语义漂移。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityDecisionEvidenceBundleRequest {
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
}

// 2026-04-09 CST: 这里定义证据质量摘要，原因是 committee / snapshot 都需要先判断“证据是否完整”，
// 目的：把多源研究结果压缩成稳定可复用的质量刻度，而不是在每个 Tool 里重复写完整度判断。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityEvidenceQuality {
    pub technical_status: String,
    pub fundamental_status: String,
    pub disclosure_status: String,
    pub overall_status: String,
    pub risk_flags: Vec<String>,
}

// 2026-04-09 CST: 这里定义正式证据包结果，原因是 committee 与 feature_snapshot 都需要共享同一份冻结研究对象，
// 目的：把 analysis_date / data_gaps / evidence_hash 固化下来，避免同一轮分析出现“不同 Tool 看到不同事实”的问题。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityDecisionEvidenceBundleResult {
    pub symbol: String,
    pub analysis_date: String,
    pub technical_context: SecurityAnalysisContextualResult,
    pub fundamental_context: FundamentalContext,
    pub disclosure_context: DisclosureContext,
    pub industry_context: IndustryContext,
    pub integrated_conclusion: IntegratedConclusion,
    pub evidence_quality: SecurityEvidenceQuality,
    pub risk_notes: Vec<String>,
    pub data_gaps: Vec<String>,
    pub evidence_hash: String,
}

// 2026-04-09 CST: 这里单独定义证据包错误边界，原因是上层 Tool 不应该直接暴露 fullstack 内部错误实现，
// 目的：给 dispatcher 和后续治理层统一错误口径，避免错误文本跟着底层结构变化。
#[derive(Debug, Error)]
pub enum SecurityDecisionEvidenceBundleError {
    #[error("证券投决证据冻结失败: {0}")]
    Fullstack(#[from] SecurityAnalysisFullstackError),
}

// 2026-04-09 CST: 这里实现正式证据冻结入口，原因是 Task 1-4 的所有新对象都要基于同一份中间证据层，
// 目的：先把 fullstack 结果冻结成单一正式对象，再往上生长 committee、snapshot 与 training 底座。
pub fn security_decision_evidence_bundle(
    request: &SecurityDecisionEvidenceBundleRequest,
) -> Result<SecurityDecisionEvidenceBundleResult, SecurityDecisionEvidenceBundleError> {
    let fullstack_request = SecurityAnalysisFullstackRequest {
        symbol: request.symbol.clone(),
        market_symbol: request.market_symbol.clone(),
        sector_symbol: request.sector_symbol.clone(),
        market_profile: request.market_profile.clone(),
        sector_profile: request.sector_profile.clone(),
        as_of_date: request.as_of_date.clone(),
        lookback_days: request.lookback_days,
        disclosure_limit: request.disclosure_limit,
    };
    let analysis = security_analysis_fullstack(&fullstack_request)?;
    Ok(build_evidence_bundle(request, analysis))
}

// 2026-04-09 CST: 这里新增证据包到原子特征种子的统一映射，原因是 Task 2 的 snapshot 与后续 scorecard 都需要稳定原子特征，
// 目的：把特征抽取口径收口在证据层，避免 snapshot、scorecard、training 各自重复拼字段。
pub fn build_evidence_bundle_feature_seed(
    bundle: &SecurityDecisionEvidenceBundleResult,
) -> BTreeMap<String, Value> {
    let mut features = BTreeMap::new();
    features.insert(
        "integrated_stance".to_string(),
        Value::String(bundle.integrated_conclusion.stance.clone()),
    );
    features.insert(
        "technical_alignment".to_string(),
        Value::String(
            bundle
                .technical_context
                .contextual_conclusion
                .alignment
                .clone(),
        ),
    );
    features.insert(
        "technical_status".to_string(),
        Value::String(bundle.evidence_quality.technical_status.clone()),
    );
    features.insert(
        "fundamental_status".to_string(),
        Value::String(bundle.fundamental_context.status.clone()),
    );
    features.insert(
        "fundamental_available".to_string(),
        json!(bundle.fundamental_context.status == "available"),
    );
    features.insert(
        "disclosure_status".to_string(),
        Value::String(bundle.disclosure_context.status.clone()),
    );
    features.insert(
        "disclosure_available".to_string(),
        json!(bundle.disclosure_context.status == "available"),
    );
    features.insert(
        "overall_evidence_status".to_string(),
        Value::String(bundle.evidence_quality.overall_status.clone()),
    );
    features.insert("data_gap_count".to_string(), json!(bundle.data_gaps.len()));
    features.insert(
        "risk_note_count".to_string(),
        json!(bundle.risk_notes.len()),
    );
    features.insert(
        "analysis_date".to_string(),
        Value::String(bundle.analysis_date.clone()),
    );
    features
}

// 2026-04-09 CST: 这里集中把 fullstack 映射成正式证据包，原因是研究层与治理层虽然复用事实，但对象职责不同，
// 目的：统一补 analysis_date、quality、data_gaps 与 evidence_hash，避免这些逻辑散落到多个上层 Tool。
fn build_evidence_bundle(
    request: &SecurityDecisionEvidenceBundleRequest,
    analysis: SecurityAnalysisFullstackResult,
) -> SecurityDecisionEvidenceBundleResult {
    let SecurityAnalysisFullstackResult {
        symbol,
        technical_context,
        fundamental_context,
        disclosure_context,
        industry_context,
        integrated_conclusion,
        ..
    } = analysis;

    let analysis_date = technical_context.analysis_date.clone();
    let data_gaps = collect_data_gaps(&fundamental_context, &disclosure_context);
    let risk_notes = collect_risk_notes(
        &technical_context,
        &fundamental_context,
        &disclosure_context,
        &industry_context,
        &integrated_conclusion,
        &data_gaps,
    );
    let evidence_quality =
        build_evidence_quality(&fundamental_context, &disclosure_context, &risk_notes);
    let evidence_hash = build_evidence_hash(
        &symbol,
        &analysis_date,
        &integrated_conclusion.stance,
        &evidence_quality,
        &data_gaps,
        request,
    );

    SecurityDecisionEvidenceBundleResult {
        symbol,
        analysis_date,
        technical_context,
        fundamental_context,
        disclosure_context,
        industry_context,
        integrated_conclusion,
        evidence_quality,
        risk_notes,
        data_gaps,
        evidence_hash,
    }
}

// 2026-04-09 CST: 这里集中定义证据缺口规则，原因是 snapshot 与 committee 都需要显式知道“缺了什么”，
// 目的：把上游 unavailable 状态翻译成稳定 data_gap 语义，方便回放、训练和投决解释复用。
fn collect_data_gaps(
    fundamental_context: &FundamentalContext,
    disclosure_context: &DisclosureContext,
) -> Vec<String> {
    let mut data_gaps = Vec::new();

    if fundamental_context.status != "available" {
        data_gaps.push(format!(
            "基本面上下文当前不可用：{}",
            fundamental_context.headline
        ));
    }
    if disclosure_context.status != "available" {
        data_gaps.push(format!(
            "公告上下文当前不可用：{}",
            disclosure_context.headline
        ));
    }

    data_gaps
}

// 2026-04-09 CST: 这里统一收集证据层风险提示，原因是 committee、chair、snapshot 都需要复用同一组风险摘要，
// 目的：避免不同对象各自挑选风险字段，最终导致上层治理链口径不一致。
fn collect_risk_notes(
    technical_context: &SecurityAnalysisContextualResult,
    fundamental_context: &FundamentalContext,
    disclosure_context: &DisclosureContext,
    industry_context: &IndustryContext,
    integrated_conclusion: &IntegratedConclusion,
    data_gaps: &[String],
) -> Vec<String> {
    let mut risk_notes = Vec::new();
    risk_notes.extend(technical_context.contextual_conclusion.risk_flags.clone());
    risk_notes.extend(fundamental_context.risk_flags.clone());
    risk_notes.extend(disclosure_context.risk_flags.clone());
    risk_notes.extend(industry_context.risk_flags.clone());
    risk_notes.extend(integrated_conclusion.risk_flags.clone());
    risk_notes.extend(data_gaps.iter().cloned());
    dedupe_strings(&mut risk_notes);
    risk_notes
}

// 2026-04-09 CST: 这里把多源可用性收敛成质量摘要，原因是 committee/snapshot 只需要稳定状态，而不是重复解释所有子对象，
// 目的：为风险闸门、数据质量标记和后续训练过滤提供统一输入。
fn build_evidence_quality(
    fundamental_context: &FundamentalContext,
    disclosure_context: &DisclosureContext,
    risk_notes: &[String],
) -> SecurityEvidenceQuality {
    let technical_status = "available".to_string();
    let fundamental_status = fundamental_context.status.clone();
    let disclosure_status = disclosure_context.status.clone();
    let overall_status = if fundamental_status == "available" && disclosure_status == "available" {
        "complete".to_string()
    } else {
        "degraded".to_string()
    };

    SecurityEvidenceQuality {
        technical_status,
        fundamental_status,
        disclosure_status,
        overall_status,
        risk_flags: risk_notes.to_vec(),
    }
}

// 2026-04-09 CST: 这里生成证据哈希，原因是新治理链要求 committee / snapshot / chair 都围绕同一份冻结证据演进，
// 目的：给后续回放、审计和对齐校验提供稳定证据版本锚点。
fn build_evidence_hash(
    symbol: &str,
    analysis_date: &str,
    stance: &str,
    evidence_quality: &SecurityEvidenceQuality,
    data_gaps: &[String],
    request: &SecurityDecisionEvidenceBundleRequest,
) -> String {
    let mut hasher = DefaultHasher::new();
    symbol.hash(&mut hasher);
    analysis_date.hash(&mut hasher);
    stance.hash(&mut hasher);
    evidence_quality.overall_status.hash(&mut hasher);
    evidence_quality.fundamental_status.hash(&mut hasher);
    evidence_quality.disclosure_status.hash(&mut hasher);
    data_gaps.hash(&mut hasher);
    request.market_symbol.hash(&mut hasher);
    request.sector_symbol.hash(&mut hasher);
    request.market_profile.hash(&mut hasher);
    request.sector_profile.hash(&mut hasher);
    request.lookback_days.hash(&mut hasher);
    request.disclosure_limit.hash(&mut hasher);
    format!("sec-{:016x}", hasher.finish())
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
