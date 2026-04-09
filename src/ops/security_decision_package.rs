use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ops::stock::security_execution_journal::{
    SecurityExecutionJournalDocument, SecurityExecutionJournalResult, SecurityExecutionTradeInput,
};
use crate::ops::stock::security_execution_record::{
    SecurityExecutionRecordDocument, SecurityExecutionRecordResult,
};
use crate::ops::stock::security_post_trade_review::{
    SecurityPostTradeReviewDocument, SecurityPostTradeReviewError, SecurityPostTradeReviewRequest,
    SecurityPostTradeReviewResult, security_post_trade_review,
};
use crate::ops::stock::security_record_post_meeting_conclusion::{
    SecurityPostMeetingConclusionDocument, SecurityPostMeetingConclusionError,
    SecurityPostMeetingConclusionRequest, SecurityPostMeetingConclusionResult,
    security_record_post_meeting_conclusion,
};

// 2026-04-09 CST: 这里扩展 package 请求合同，原因是 Task 9 标准治理版要把 post_trade_review 正式挂入 package；
// 目的：让 package 自身就具备装配投前、投中、投后对象的最小上下文，而不是继续依赖外层二次拼装。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityDecisionPackageRequest {
    pub symbol: String,
    #[serde(default = "default_market_regime")]
    pub market_regime: String,
    #[serde(default = "default_sector_template")]
    pub sector_template: String,
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
    #[serde(default = "default_factor_lookback_days")]
    pub factor_lookback_days: usize,
    #[serde(default = "default_stop_loss_pct")]
    pub stop_loss_pct: f64,
    #[serde(default = "default_target_return_pct")]
    pub target_return_pct: f64,
    #[serde(default = "default_review_horizon_days")]
    pub review_horizon_days: usize,
    #[serde(default)]
    pub actual_entry_date: String,
    #[serde(default)]
    pub actual_entry_price: f64,
    #[serde(default)]
    pub actual_position_pct: f64,
    #[serde(default)]
    pub actual_exit_date: String,
    #[serde(default)]
    pub actual_exit_price: f64,
    #[serde(default)]
    pub exit_reason: String,
    #[serde(default)]
    pub execution_trades: Vec<SecurityExecutionTradeInput>,
    #[serde(default)]
    pub execution_journal_notes: Vec<String>,
    #[serde(default)]
    pub execution_record_notes: Vec<String>,
    #[serde(default = "default_min_risk_reward_ratio")]
    pub min_risk_reward_ratio: f64,
    #[serde(default = "default_created_at")]
    pub created_at: String,
    #[serde(default)]
    pub scorecard_model_path: Option<String>,
    #[serde(default)]
    pub execution_notes: Vec<String>,
    #[serde(default)]
    pub follow_up_actions: Vec<String>,
}

// 2026-04-09 CST: 这里保持 object_graph 为正式治理拓扑，原因是 verify 不应再靠外层猜测“哪些对象应在包里”；
// 目的：让 post_trade_review 进入 package 后，校验逻辑只围绕这份正式图谱执行。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityDecisionPackageObjectNode {
    pub object_type: String,
    pub object_id: String,
    pub parent_ref: Option<String>,
}

// 2026-04-09 CST: 这里保持 artifact_manifest 为正式挂载清单，原因是 object_graph 只表达对象关系，不表达文档绑定；
// 目的：让 verify 可以同时校验“对象已入包”和“正式文档已挂清单”两层事实。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityDecisionPackageArtifactEntry {
    pub document_type: String,
    pub artifact_id: String,
    pub symbol: String,
    pub analysis_date: String,
    pub bound_object_type: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SecurityDecisionPackageResult {
    pub package_id: String,
    pub contract_version: String,
    pub document_type: String,
    pub package_status: String,
    pub assembled_at: String,
    pub symbol: String,
    pub analysis_date: String,
    pub chair_resolution_result: SecurityPostMeetingConclusionResult,
    pub post_meeting_conclusion: SecurityPostMeetingConclusionDocument,
    pub post_trade_review_result: SecurityPostTradeReviewResult,
    pub post_trade_review: SecurityPostTradeReviewDocument,
    pub execution_journal_result: SecurityExecutionJournalResult,
    pub execution_journal: SecurityExecutionJournalDocument,
    pub execution_record_result: SecurityExecutionRecordResult,
    pub execution_record: SecurityExecutionRecordDocument,
    pub object_graph: Vec<SecurityDecisionPackageObjectNode>,
    pub artifact_manifest: Vec<SecurityDecisionPackageArtifactEntry>,
    pub package_summary: String,
}

#[derive(Debug, Error)]
pub enum SecurityDecisionPackageError {
    #[error("security decision package post meeting preparation failed: {0}")]
    PostMeeting(#[from] SecurityPostMeetingConclusionError),
    #[error("security decision package post trade review preparation failed: {0}")]
    PostTradeReview(#[from] SecurityPostTradeReviewError),
}

pub fn security_decision_package(
    request: &SecurityDecisionPackageRequest,
) -> Result<SecurityDecisionPackageResult, SecurityDecisionPackageError> {
    // 2026-04-09 CST: 这里先装配 post_trade_review，原因是 Task 9 标准治理版要把投后对象正式并入 package；
    // 目的：让 package / verify / revision 在同一份结果里直接消费 review，而不是依赖外层手工拼接。
    let post_trade_review_result = security_post_trade_review(&SecurityPostTradeReviewRequest {
        symbol: request.symbol.clone(),
        market_symbol: request.market_symbol.clone(),
        sector_symbol: request.sector_symbol.clone(),
        market_regime: request.market_regime.clone(),
        sector_template: request.sector_template.clone(),
        market_profile: request.market_profile.clone(),
        sector_profile: request.sector_profile.clone(),
        as_of_date: request.as_of_date.clone(),
        review_horizon_days: request.review_horizon_days,
        lookback_days: request.lookback_days,
        factor_lookback_days: request.factor_lookback_days,
        disclosure_limit: request.disclosure_limit,
        stop_loss_pct: request.stop_loss_pct,
        target_return_pct: request.target_return_pct,
        actual_entry_date: request.actual_entry_date.clone(),
        actual_entry_price: request.actual_entry_price,
        actual_position_pct: request.actual_position_pct,
        actual_exit_date: request.actual_exit_date.clone(),
        actual_exit_price: request.actual_exit_price,
        exit_reason: request.exit_reason.clone(),
        execution_trades: request.execution_trades.clone(),
        execution_journal_notes: request.execution_journal_notes.clone(),
        execution_record_notes: request.execution_record_notes.clone(),
        // 2026-04-09 CST: 这里显式补空账户计划字段，原因是 security_post_trade_review 已扩展支持账户级偏差回写；
        // 目的：保持 decision_package 旧链路继续可编译、可运行，在未传账户计划时按 None 处理。
        portfolio_position_plan_document: None,
        created_at: request.created_at.clone(),
    })?;
    let post_meeting_request = SecurityPostMeetingConclusionRequest {
        symbol: request.symbol.clone(),
        market_symbol: request.market_symbol.clone(),
        sector_symbol: request.sector_symbol.clone(),
        market_profile: request.market_profile.clone(),
        sector_profile: request.sector_profile.clone(),
        as_of_date: request.as_of_date.clone(),
        lookback_days: request.lookback_days,
        disclosure_limit: request.disclosure_limit,
        stop_loss_pct: request.stop_loss_pct,
        target_return_pct: request.target_return_pct,
        min_risk_reward_ratio: request.min_risk_reward_ratio,
        created_at: request.created_at.clone(),
        scorecard_model_path: request.scorecard_model_path.clone(),
        execution_notes: request.execution_notes.clone(),
        follow_up_actions: request.follow_up_actions.clone(),
    };
    let post_meeting_result = security_record_post_meeting_conclusion(&post_meeting_request)?;
    let object_graph = build_object_graph(&post_meeting_result, &post_trade_review_result);
    let artifact_manifest =
        build_artifact_manifest(&post_meeting_result, &post_trade_review_result);
    let symbol = post_meeting_result.post_meeting_conclusion.symbol.clone();
    let analysis_date = post_meeting_result
        .post_meeting_conclusion
        .analysis_date
        .clone();
    let package_id = format!(
        "decision-package-{}",
        post_meeting_result.post_meeting_conclusion.decision_id
    );

    Ok(SecurityDecisionPackageResult {
        package_id,
        contract_version: "security_decision_package.v1".to_string(),
        document_type: "security_decision_package".to_string(),
        package_status: "assembled".to_string(),
        assembled_at: normalize_created_at(&request.created_at),
        symbol: symbol.clone(),
        analysis_date: analysis_date.clone(),
        chair_resolution_result: post_meeting_result.clone(),
        post_meeting_conclusion: post_meeting_result.post_meeting_conclusion.clone(),
        post_trade_review_result: post_trade_review_result.clone(),
        post_trade_review: post_trade_review_result.post_trade_review.clone(),
        execution_journal_result: post_trade_review_result.execution_journal_result.clone(),
        execution_journal: post_trade_review_result.execution_journal.clone(),
        execution_record_result: post_trade_review_result.execution_record_result.clone(),
        execution_record: post_trade_review_result.execution_record.clone(),
        object_graph,
        artifact_manifest,
        package_summary: format!(
            "decision package 已装配 evidence -> committee -> scorecard -> chair -> post_meeting -> execution_journal -> execution_record -> post_trade_review 全链对象，分析日期为 {}。",
            analysis_date
        ),
    })
}

fn build_object_graph(
    post_meeting_result: &SecurityPostMeetingConclusionResult,
    post_trade_review_result: &SecurityPostTradeReviewResult,
) -> Vec<SecurityDecisionPackageObjectNode> {
    let committee = &post_meeting_result.chair_resolution_result.committee_result;
    let scorecard = &post_meeting_result.chair_resolution_result.scorecard;
    let chair = &post_meeting_result.chair_resolution_result.chair_resolution;
    let post_meeting = &post_meeting_result.post_meeting_conclusion;
    let execution_journal = &post_trade_review_result.execution_journal;
    let execution_record = &post_trade_review_result.execution_record;
    let post_trade_review = &post_trade_review_result.post_trade_review;
    vec![
        SecurityDecisionPackageObjectNode {
            object_type: "evidence_bundle".to_string(),
            object_id: committee.evidence_bundle.evidence_hash.clone(),
            parent_ref: None,
        },
        SecurityDecisionPackageObjectNode {
            object_type: "committee_result".to_string(),
            object_id: committee.committee_session_ref.clone(),
            parent_ref: Some(committee.evidence_bundle.evidence_hash.clone()),
        },
        SecurityDecisionPackageObjectNode {
            object_type: "decision_card".to_string(),
            object_id: committee.decision_card.decision_id.clone(),
            parent_ref: Some(committee.committee_session_ref.clone()),
        },
        SecurityDecisionPackageObjectNode {
            object_type: "scorecard".to_string(),
            object_id: scorecard.scorecard_id.clone(),
            parent_ref: Some(committee.decision_card.decision_id.clone()),
        },
        SecurityDecisionPackageObjectNode {
            object_type: "chair_resolution".to_string(),
            object_id: chair.chair_resolution_id.clone(),
            parent_ref: Some(scorecard.scorecard_id.clone()),
        },
        SecurityDecisionPackageObjectNode {
            object_type: "post_meeting_conclusion".to_string(),
            object_id: post_meeting.post_meeting_conclusion_id.clone(),
            parent_ref: Some(chair.chair_resolution_id.clone()),
        },
        SecurityDecisionPackageObjectNode {
            object_type: "execution_journal".to_string(),
            object_id: execution_journal.execution_journal_id.clone(),
            parent_ref: Some(post_meeting.post_meeting_conclusion_id.clone()),
        },
        SecurityDecisionPackageObjectNode {
            object_type: "execution_record".to_string(),
            object_id: execution_record.execution_record_id.clone(),
            parent_ref: Some(execution_journal.execution_journal_id.clone()),
        },
        SecurityDecisionPackageObjectNode {
            object_type: "post_trade_review".to_string(),
            object_id: post_trade_review.review_id.clone(),
            parent_ref: Some(execution_record.execution_record_id.clone()),
        },
    ]
}

fn build_artifact_manifest(
    post_meeting_result: &SecurityPostMeetingConclusionResult,
    post_trade_review_result: &SecurityPostTradeReviewResult,
) -> Vec<SecurityDecisionPackageArtifactEntry> {
    let committee = &post_meeting_result.chair_resolution_result.committee_result;
    let scorecard = &post_meeting_result.chair_resolution_result.scorecard;
    let chair = &post_meeting_result.chair_resolution_result.chair_resolution;
    let post_meeting = &post_meeting_result.post_meeting_conclusion;
    let execution_journal = &post_trade_review_result.execution_journal;
    let execution_record = &post_trade_review_result.execution_record;
    let post_trade_review = &post_trade_review_result.post_trade_review;
    vec![
        SecurityDecisionPackageArtifactEntry {
            document_type: "security_decision_evidence_bundle".to_string(),
            artifact_id: committee.evidence_bundle.evidence_hash.clone(),
            symbol: committee.symbol.clone(),
            analysis_date: committee.analysis_date.clone(),
            bound_object_type: "evidence_bundle".to_string(),
        },
        SecurityDecisionPackageArtifactEntry {
            document_type: "security_decision_card".to_string(),
            artifact_id: committee.decision_card.decision_id.clone(),
            symbol: committee.symbol.clone(),
            analysis_date: committee.analysis_date.clone(),
            bound_object_type: "decision_card".to_string(),
        },
        SecurityDecisionPackageArtifactEntry {
            document_type: "security_scorecard".to_string(),
            artifact_id: scorecard.scorecard_id.clone(),
            symbol: scorecard.symbol.clone(),
            analysis_date: scorecard.analysis_date.clone(),
            bound_object_type: "scorecard".to_string(),
        },
        SecurityDecisionPackageArtifactEntry {
            document_type: "security_chair_resolution".to_string(),
            artifact_id: chair.chair_resolution_id.clone(),
            symbol: chair.symbol.clone(),
            analysis_date: chair.analysis_date.clone(),
            bound_object_type: "chair_resolution".to_string(),
        },
        SecurityDecisionPackageArtifactEntry {
            document_type: "security_post_meeting_conclusion".to_string(),
            artifact_id: post_meeting.post_meeting_conclusion_id.clone(),
            symbol: post_meeting.symbol.clone(),
            analysis_date: post_meeting.analysis_date.clone(),
            bound_object_type: "post_meeting_conclusion".to_string(),
        },
        SecurityDecisionPackageArtifactEntry {
            document_type: "security_execution_journal".to_string(),
            artifact_id: execution_journal.execution_journal_id.clone(),
            symbol: execution_journal.symbol.clone(),
            analysis_date: execution_journal.analysis_date.clone(),
            bound_object_type: "execution_journal".to_string(),
        },
        SecurityDecisionPackageArtifactEntry {
            document_type: "security_execution_record".to_string(),
            artifact_id: execution_record.execution_record_id.clone(),
            symbol: execution_record.symbol.clone(),
            analysis_date: execution_record.analysis_date.clone(),
            bound_object_type: "execution_record".to_string(),
        },
        SecurityDecisionPackageArtifactEntry {
            document_type: "security_post_trade_review".to_string(),
            artifact_id: post_trade_review.review_id.clone(),
            symbol: post_trade_review.symbol.clone(),
            analysis_date: post_trade_review.analysis_date.clone(),
            bound_object_type: "post_trade_review".to_string(),
        },
    ]
}

fn normalize_created_at(value: &str) -> String {
    if value.trim().is_empty() {
        Utc::now().to_rfc3339()
    } else {
        value.trim().to_string()
    }
}

fn default_created_at() -> String {
    Utc::now().to_rfc3339()
}

fn default_market_regime() -> String {
    "a_share".to_string()
}

fn default_sector_template() -> String {
    "bank".to_string()
}

fn default_lookback_days() -> usize {
    260
}

fn default_disclosure_limit() -> usize {
    8
}

fn default_factor_lookback_days() -> usize {
    120
}

fn default_stop_loss_pct() -> f64 {
    0.05
}

fn default_target_return_pct() -> f64 {
    0.12
}

fn default_review_horizon_days() -> usize {
    20
}

fn default_min_risk_reward_ratio() -> f64 {
    2.0
}
