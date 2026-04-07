use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use thiserror::Error;

use crate::ops::stock::security_analysis_contextual::SecurityAnalysisContextualResult;
use crate::ops::stock::security_analysis_fullstack::{
    security_analysis_fullstack, DisclosureContext, FundamentalContext, IndustryContext,
    IntegratedConclusion, SecurityAnalysisFullstackError, SecurityAnalysisFullstackRequest,
    SecurityAnalysisFullstackResult,
};

// 2026-04-01 CST: 这里定义证券投决证据包请求，原因是方案 B 需要把研究链输出冻结成投决层的单一输入合同；
// 目的：让后续多头、空头与风险闸门都读取同一份证据，而不是各自直接碰 fullstack Tool 导致上下文漂移。
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

// 2026-04-01 CST: 这里定义证据质量摘要，原因是投决会需要先知道“证据完整度”再决定是否进入裁决；
// 目的：把技术、基本面、公告的可用性收敛成稳定字段，便于后续 Gate 和 Skill 统一消费。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SecurityEvidenceQuality {
    pub technical_status: String,
    pub fundamental_status: String,
    pub disclosure_status: String,
    pub overall_status: String,
    pub risk_flags: Vec<String>,
}

// 2026-04-01 CST: 这里定义证券投决证据包结果，原因是我们要把研究链结果提升成投决层可冻结、可审计的对象；
// 目的：显式携带 analysis_date、data_gaps、evidence_hash，避免后续对话中悄悄混入新的事实或日期。
#[derive(Debug, Clone, PartialEq, Serialize)]
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

// 2026-04-01 CST: 这里单独定义证据包错误，原因是投决层不应该直接泄露 fullstack 内部错误类型细节；
// 目的：保留研究层失败原因，同时给 dispatcher 和后续 Skill 一个清晰、单一的错误边界。
#[derive(Debug, Error)]
pub enum SecurityDecisionEvidenceBundleError {
    #[error("证券投决证据冻结失败: {0}")]
    Fullstack(#[from] SecurityAnalysisFullstackError),
}

// 2026-04-01 CST: 这里实现证券投决证据冻结入口，原因是所有正反方立场必须先基于同一份静态证据包；
// 目的：把 current research chain 提升成 investment decision workbench 可消费的稳定中间层，而不是继续直接暴露研究工具本身。
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

// 2026-04-01 CST: 这里把 fullstack 结果映射成投决证据包，原因是研究层和投决层虽然复用数据，但合同职责不同；
// 目的：集中补齐 analysis_date、quality、data_gaps 和 hash，避免这些逻辑散落在 Skill 或 dispatcher 里。
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
    } = analysis;

    let analysis_date = technical_context.stock_analysis.as_of_date.clone();
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

// 2026-04-01 CST: 这里集中定义证据缺口规则，原因是投决会需要显式知道缺了什么而不是只看 upstream status；
// 目的：把“不可用信息源”翻译成用户和 AI 都能解释的 data gap 语义。
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

// 2026-04-01 CST: 这里统一收集证据层风险提示，原因是后续双立场与闸门都要引用同一组风险摘要；
// 目的：避免 bull/bear 在不同位置重复拼接风险，导致观点口径不一致。
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

// 2026-04-01 CST: 这里把多源可用性收敛成质量摘要，原因是风控闸门只需要看稳定状态而不是重新解读全部子对象；
// 目的：为 data completeness、approval gating 和最终输出提供统一的质量刻度。
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

// 2026-04-01 CST: 这里生成证据哈希，原因是单次对话里的正反方必须围绕同一份冻结证据而不是隐式更新的数据；
// 目的：给 Skill 和后续审计链一个可比对的“证据版本标识”。
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

// 2026-04-01 CST: 这里统一做去重，原因是多层研究链可能同时指出同一个风险点；
// 目的：保持输出精炼，避免后续正反方摘要被重复风险淹没。
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
