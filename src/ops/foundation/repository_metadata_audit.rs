use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::ops::foundation::knowledge_record::EvidenceRef;
use crate::ops::foundation::knowledge_repository::KnowledgeRepository;
use crate::ops::foundation::metadata_schema::MetadataSchema;
use crate::ops::foundation::metadata_validator::{MetadataValidationIssue, MetadataValidator};

// 2026-04-10 CST: 这里定义 repository-level issue 明细，原因是 foundation 当前已经有节点级 validator，
// 但仓库级治理还缺少“问题属于哪个节点”的正式报告载体。目的：把节点上下文和 issue 一起挂进审计报告。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryMetadataAuditIssue {
    pub node_id: String,
    pub issue: MetadataValidationIssue,
}

// 2026-04-10 CST: 这里补充 weak locator 原因枚举，原因是本轮方案A要求把“弱 locator”从单点提示提升为可解释诊断。
// 目的：让后续 repository audit 报告能区分空白定位与过短定位，便于 AI 和人工治理时按原因收口。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RepositoryWeakLocatorReason {
    Blank,
    TooShort,
    SheetOnly,
    SingleCellOnly,
    AmbiguousKeyword,
    InvalidRangeFormat,
}

// 2026-04-10 CST: 这里补充 weak source_ref 原因枚举，原因是当前弱来源只会报“弱”，不足以支撑后续治理优先级判断。
// 目的：明确区分空白、过短、缺少 namespace 三类问题，保持诊断层只读但可追踪。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RepositoryWeakSourceRefReason {
    Blank,
    TooShort,
    MissingNamespace,
    EntityMissing,
    ContainsWhitespace,
    InvalidCharacter,
    UnknownNamespace,
}

// 2026-04-12 CST: Added a structured hygiene severity layer because the
// knowledge roaming foundation now needs machine-readable governance summary
// output in addition to raw diagnostics. Purpose: let downstream AI flows decide
// whether to continue navigation or stop for hygiene remediation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepositoryHygieneSeverity {
    Critical,
    Warning,
    Info,
}

// 2026-04-12 CST: Added repository hygiene summary modeling so audit results can
// expose stable aggregate counts without losing per-diagnostic detail. Purpose:
// keep the foundation report usable for both humans and automated orchestration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryEvidenceHygieneSummary {
    pub total_diagnostics: usize,
    pub severity_counts: BTreeMap<String, usize>,
    pub diagnostic_type_counts: BTreeMap<String, usize>,
    pub weak_locator_reason_counts: BTreeMap<String, usize>,
    pub weak_source_ref_reason_counts: BTreeMap<String, usize>,
    pub affected_node_count: usize,
    pub has_blocking_hygiene_issue: bool,
}

// 2026-04-12 CST: Added grouped hygiene views because the roaming foundation now
// needs one layer that is easier for AI routing than raw details or flat counts.
// Purpose: expose stable severity-first and node-first governance views.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryEvidenceHygieneViews {
    pub by_severity: Vec<RepositoryEvidenceHygieneSeverityGroup>,
    pub by_node: Vec<RepositoryEvidenceHygieneNodeGroup>,
}

// 2026-04-12 CST: Added reason-grouped hygiene views because downstream AI now
// needs to prioritize cleanup by weak-cause family, not only by node or global
// severity. Purpose: expose a compact reason-first routing surface without
// duplicating full diagnostic payloads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryEvidenceHygieneReasonViews {
    pub weak_locator_by_reason: Vec<RepositoryWeakLocatorReasonGroup>,
    pub weak_source_ref_by_reason: Vec<RepositoryWeakSourceRefReasonGroup>,
}

// 2026-04-12 CST: Added severity groups so downstream routing can quickly answer
// which class of hygiene issue should be handled first. Purpose: reduce the need
// for every caller to rebuild severity buckets from raw diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryEvidenceHygieneSeverityGroup {
    pub severity: RepositoryHygieneSeverity,
    pub diagnostic_count: usize,
    pub affected_node_count: usize,
    pub node_ids: Vec<String>,
}

// 2026-04-12 CST: Added node groups so governance flows can prioritize concrete
// cleanup targets. Purpose: expose highest severity and local issue shape per
// node without duplicating the full diagnostic payload again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryEvidenceHygieneNodeGroup {
    pub node_id: String,
    pub highest_severity: RepositoryHygieneSeverity,
    pub diagnostic_count: usize,
    pub diagnostic_type_counts: BTreeMap<String, usize>,
}

// 2026-04-12 CST: Added weak-locator reason groups so governance flows can rank
// locator hygiene by concrete cause. Purpose: let AI decide whether to fix blank,
// pseudo-range, or sheet-only references first without rebuilding aggregates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryWeakLocatorReasonGroup {
    pub reason: RepositoryWeakLocatorReason,
    pub severity: RepositoryHygieneSeverity,
    pub diagnostic_count: usize,
    pub affected_node_count: usize,
    pub node_ids: Vec<String>,
}

// 2026-04-12 CST: Added weak-source-ref reason groups so governance flows can
// separate blocking source issues from warning-level source issues. Purpose:
// keep reason-first routing aligned with the existing severity model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryWeakSourceRefReasonGroup {
    pub reason: RepositoryWeakSourceRefReason,
    pub severity: RepositoryHygieneSeverity,
    pub diagnostic_count: usize,
    pub affected_node_count: usize,
    pub node_ids: Vec<String>,
}

// 2026-04-10 CST: 这里定义最小 evidence hygiene diagnostics，原因是用户已经明确要求 foundation 继续补
// “重复证据、弱 locator、弱 source_ref”这类仓库级诊断。目的：先把只读诊断能力正式建模，不越界到自动修复。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepositoryEvidenceHygieneDiagnostic {
    // 2026-04-10 CST: 这里补 MissingEvidenceRef，原因是没有任何证据引用的节点会直接削弱 foundation 元数据可信度。
    // 目的：在 repository-level audit 中正式标出“无证据承接”的节点，后续再决定是否进入治理动作。
    MissingEvidenceRef {
        node_id: String,
    },
    // 2026-04-10 CST: 这里补同节点内重复证据诊断，原因是此前只覆盖跨节点重复，漏掉了节点内部自重复的 hygiene 问题。
    // 目的：把“单节点重复挂同一 evidence_ref”显式暴露出来，避免仓库治理只看跨节点冲突。
    DuplicateEvidenceRefWithinNode {
        node_id: String,
        source_ref: String,
        locator: String,
        occurrence_count: usize,
    },
    DuplicateEvidenceRef {
        source_ref: String,
        locator: String,
        node_ids: Vec<String>,
    },
    WeakLocator {
        node_id: String,
        source_ref: String,
        locator: String,
        reason: RepositoryWeakLocatorReason,
    },
    WeakSourceRef {
        node_id: String,
        source_ref: String,
        locator: String,
        reason: RepositoryWeakSourceRefReason,
    },
}

// 2026-04-10 CST: 这里定义 repository audit 报告对象，原因是方案B选的是“聚合审计报告”，
// 不是只返回扁平 issue 列表。目的：统一承载摘要、明细、聚合计数和 hygiene diagnostics。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryMetadataAuditReport {
    pub total_nodes: usize,
    pub audited_nodes: usize,
    pub issue_count: usize,
    pub issues: Vec<RepositoryMetadataAuditIssue>,
    pub issue_type_counts: BTreeMap<String, usize>,
    pub concept_issue_counts: BTreeMap<String, usize>,
    pub hygiene_diagnostics: Vec<RepositoryEvidenceHygieneDiagnostic>,
    pub hygiene_summary: RepositoryEvidenceHygieneSummary,
    pub hygiene_views: RepositoryEvidenceHygieneViews,
    pub hygiene_reason_views: RepositoryEvidenceHygieneReasonViews,
}

// 2026-04-12 CST: Added a versioned export contract because downstream AI and
// upper layers should consume a stable DTO instead of binding to the internal
// repository audit report shape directly. Purpose: reserve room for future
// internal evolution while keeping one clear v1 contract surface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryMetadataAuditExportDtoV1 {
    pub total_nodes: usize,
    pub audited_nodes: usize,
    pub issue_count: usize,
    pub issues: Vec<RepositoryMetadataAuditIssueDtoV1>,
    pub issue_type_counts: BTreeMap<String, usize>,
    pub concept_issue_counts: BTreeMap<String, usize>,
    pub hygiene_diagnostics: Vec<RepositoryEvidenceHygieneDiagnosticDtoV1>,
    pub hygiene_summary: RepositoryEvidenceHygieneSummaryDtoV1,
    pub hygiene_views: RepositoryEvidenceHygieneViewsDtoV1,
    pub hygiene_reason_views: RepositoryEvidenceHygieneReasonViewsDtoV1,
}

// 2026-04-12 CST: Added issue DTO modeling so the v1 export layer can carry
// validator findings without exposing the internal issue enum directly.
// Purpose: keep the exported contract stable even if validator issue internals
// evolve later.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryMetadataAuditIssueDtoV1 {
    pub node_id: String,
    pub issue_type: String,
    pub concept_id: Option<String>,
    pub field_key: Option<String>,
    pub alias_field_key: Option<String>,
    pub canonical_field_key: Option<String>,
    pub replaced_by: Option<String>,
}

// 2026-04-12 CST: Added diagnostic DTO modeling because downstream consumers
// need stable strings and fields rather than the internal hygiene enum itself.
// Purpose: export the existing hygiene detail layer without leaking enum layout.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryEvidenceHygieneDiagnosticDtoV1 {
    pub diagnostic_type: String,
    pub severity: RepositoryHygieneSeverityDtoV1,
    pub node_id: Option<String>,
    pub node_ids: Vec<String>,
    pub source_ref: Option<String>,
    pub locator: Option<String>,
    pub reason: Option<String>,
    pub occurrence_count: Option<usize>,
}

// 2026-04-12 CST: Added a DTO severity enum so the export layer can remain
// strongly typed while still exposing stable string semantics to callers.
// Purpose: avoid passing internal severity enums through the external contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepositoryHygieneSeverityDtoV1 {
    Critical,
    Warning,
    Info,
}

// 2026-04-12 CST: Added summary DTO modeling so the export layer mirrors the
// current aggregate contract without exposing internal structs by reference.
// Purpose: keep v1 self-contained and stable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryEvidenceHygieneSummaryDtoV1 {
    pub total_diagnostics: usize,
    pub severity_counts: BTreeMap<String, usize>,
    pub diagnostic_type_counts: BTreeMap<String, usize>,
    pub weak_locator_reason_counts: BTreeMap<String, usize>,
    pub weak_source_ref_reason_counts: BTreeMap<String, usize>,
    pub affected_node_count: usize,
    pub has_blocking_hygiene_issue: bool,
}

// 2026-04-12 CST: Added grouped view DTO modeling so upper layers can rely on
// one versioned projection of the existing grouped governance contract.
// Purpose: separate internal report structs from the stable export surface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryEvidenceHygieneViewsDtoV1 {
    pub by_severity: Vec<RepositoryEvidenceHygieneSeverityGroupDtoV1>,
    pub by_node: Vec<RepositoryEvidenceHygieneNodeGroupDtoV1>,
}

// 2026-04-12 CST: Added reason-view DTO modeling because reason-first routing is
// one of the key downstream AI entry points. Purpose: export it under the same
// stable v1 contract as the rest of hygiene output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryEvidenceHygieneReasonViewsDtoV1 {
    pub weak_locator_by_reason: Vec<RepositoryWeakLocatorReasonGroupDtoV1>,
    pub weak_source_ref_by_reason: Vec<RepositoryWeakSourceRefReasonGroupDtoV1>,
}

// 2026-04-12 CST: Added severity-group DTOs so the export layer preserves the
// already-locked grouping and ordering semantics. Purpose: avoid forcing callers
// to rebuild severity buckets outside foundation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryEvidenceHygieneSeverityGroupDtoV1 {
    pub severity: RepositoryHygieneSeverityDtoV1,
    pub diagnostic_count: usize,
    pub affected_node_count: usize,
    pub node_ids: Vec<String>,
}

// 2026-04-12 CST: Added node-group DTOs so the export layer preserves the
// stable node-priority routing surface from the internal report.
// Purpose: keep consumers insulated from internal struct changes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryEvidenceHygieneNodeGroupDtoV1 {
    pub node_id: String,
    pub highest_severity: RepositoryHygieneSeverityDtoV1,
    pub diagnostic_count: usize,
    pub diagnostic_type_counts: BTreeMap<String, usize>,
}

// 2026-04-12 CST: Added weak-locator reason DTO groups so downstream contracts
// can consume reason-family routing without binding to internal enums.
// Purpose: keep reason semantics explicit in the exported model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryWeakLocatorReasonGroupDtoV1 {
    pub reason: RepositoryWeakLocatorReasonDtoV1,
    pub severity: RepositoryHygieneSeverityDtoV1,
    pub diagnostic_count: usize,
    pub affected_node_count: usize,
    pub node_ids: Vec<String>,
}

// 2026-04-12 CST: Added weak-source-ref reason DTO groups so exported reason
// routing remains versioned and strongly typed. Purpose: mirror the current
// internal contract without leaking internal enum definitions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryWeakSourceRefReasonGroupDtoV1 {
    pub reason: RepositoryWeakSourceRefReasonDtoV1,
    pub severity: RepositoryHygieneSeverityDtoV1,
    pub diagnostic_count: usize,
    pub affected_node_count: usize,
    pub node_ids: Vec<String>,
}

// 2026-04-12 CST: Added weak-locator DTO reasons so v1 can expose stable string
// semantics and remain decoupled from internal enums. Purpose: make callers read
// one exported vocabulary instead of the internal type system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepositoryWeakLocatorReasonDtoV1 {
    Blank,
    TooShort,
    SheetOnly,
    SingleCellOnly,
    AmbiguousKeyword,
    InvalidRangeFormat,
}

// 2026-04-12 CST: Added weak-source-ref DTO reasons for the same boundary
// reason as locator DTO reasons. Purpose: preserve a clear exported vocabulary
// for source-ref hygiene routing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepositoryWeakSourceRefReasonDtoV1 {
    Blank,
    TooShort,
    MissingNamespace,
    EntityMissing,
    ContainsWhitespace,
    InvalidCharacter,
    UnknownNamespace,
}

// 2026-04-10 CST: 这里定义 repository audit 入口，原因是 foundation 当前需要一个只读聚合器，
// 复用现有 MetadataValidator 完成仓库级审计。目的：把“repository + schema -> audit report”固定成正式能力入口。
pub struct RepositoryMetadataAudit<'a> {
    schema: &'a MetadataSchema,
}

impl<'a> RepositoryMetadataAudit<'a> {
    // 2026-04-10 CST: 这里提供最小构造函数，原因是 repository audit 当前只依赖 schema，
    // 不应额外引入 runtime、store 或业务态依赖。目的：保持 foundation 审计层单一职责。
    pub fn new(schema: &'a MetadataSchema) -> Self {
        Self { schema }
    }

    // 2026-04-10 CST: 这里执行 repository-level metadata audit，原因是 validator linkage 完成后，
    // 下一步就是把节点级信号提升到仓库级可观察报告。目的：输出正式聚合审计结果，但不做任何自动迁移动作。
    pub fn audit(&self, repository: &KnowledgeRepository) -> RepositoryMetadataAuditReport {
        let validator = MetadataValidator::new(self.schema);
        let mut issues = Vec::new();
        let mut issue_type_counts = BTreeMap::new();
        let mut concept_issue_counts = BTreeMap::new();

        for node in &repository.bundle().nodes {
            for issue in validator.validate_node(node) {
                let issue_type_key = issue_type_key(&issue).to_string();
                *issue_type_counts.entry(issue_type_key).or_insert(0) += 1;

                if let Some(concept_id) = concept_id_for_issue(&issue) {
                    *concept_issue_counts
                        .entry(concept_id.to_string())
                        .or_insert(0) += 1;
                }

                issues.push(RepositoryMetadataAuditIssue {
                    node_id: node.id.clone(),
                    issue,
                });
            }
        }

        let hygiene_diagnostics = collect_hygiene_diagnostics(repository);
        let hygiene_summary = summarize_hygiene_diagnostics(&hygiene_diagnostics);
        let hygiene_views = build_hygiene_views(&hygiene_diagnostics);
        let hygiene_reason_views = build_hygiene_reason_views(&hygiene_diagnostics);

        RepositoryMetadataAuditReport {
            total_nodes: repository.bundle().nodes.len(),
            audited_nodes: repository.bundle().nodes.len(),
            issue_count: issues.len(),
            issues,
            issue_type_counts,
            concept_issue_counts,
            hygiene_diagnostics,
            hygiene_summary,
            hygiene_views,
            hygiene_reason_views,
        }
    }
}

impl RepositoryMetadataAuditExportDtoV1 {
    // 2026-04-12 CST: Added a one-way conversion entry point because the export
    // contract should depend on the internal report, not the other way around.
    // Purpose: keep v1 as a stable projection layer that can survive internal
    // report evolution.
    pub fn from_report(report: &RepositoryMetadataAuditReport) -> Self {
        Self {
            total_nodes: report.total_nodes,
            audited_nodes: report.audited_nodes,
            issue_count: report.issue_count,
            issues: report
                .issues
                .iter()
                .map(RepositoryMetadataAuditIssueDtoV1::from_issue)
                .collect(),
            issue_type_counts: report.issue_type_counts.clone(),
            concept_issue_counts: report.concept_issue_counts.clone(),
            hygiene_diagnostics: report
                .hygiene_diagnostics
                .iter()
                .map(RepositoryEvidenceHygieneDiagnosticDtoV1::from_diagnostic)
                .collect(),
            hygiene_summary: RepositoryEvidenceHygieneSummaryDtoV1::from_summary(
                &report.hygiene_summary,
            ),
            hygiene_views: RepositoryEvidenceHygieneViewsDtoV1::from_views(&report.hygiene_views),
            hygiene_reason_views: RepositoryEvidenceHygieneReasonViewsDtoV1::from_reason_views(
                &report.hygiene_reason_views,
            ),
        }
    }
}

impl RepositoryMetadataAuditIssueDtoV1 {
    // 2026-04-12 CST: Added issue conversion so v1 callers receive stable field
    // names instead of the raw validator enum. Purpose: keep the export layer
    // simple and resilient to validator implementation changes.
    fn from_issue(issue: &RepositoryMetadataAuditIssue) -> Self {
        match &issue.issue {
            MetadataValidationIssue::AliasFieldUsed {
                alias_field_key,
                canonical_field_key,
                ..
            } => Self {
                node_id: issue.node_id.clone(),
                issue_type: "AliasFieldUsed".to_string(),
                concept_id: None,
                field_key: None,
                alias_field_key: Some(alias_field_key.clone()),
                canonical_field_key: Some(canonical_field_key.clone()),
                replaced_by: None,
            },
            MetadataValidationIssue::DeprecatedFieldUsed {
                field_key,
                replaced_by,
                ..
            } => Self {
                node_id: issue.node_id.clone(),
                issue_type: "DeprecatedFieldUsed".to_string(),
                concept_id: None,
                field_key: Some(field_key.clone()),
                alias_field_key: None,
                canonical_field_key: None,
                replaced_by: replaced_by.clone(),
            },
            MetadataValidationIssue::MissingConceptPolicy { concept_id, .. } => Self {
                node_id: issue.node_id.clone(),
                issue_type: "MissingConceptPolicy".to_string(),
                concept_id: Some(concept_id.clone()),
                field_key: None,
                alias_field_key: None,
                canonical_field_key: None,
                replaced_by: None,
            },
            MetadataValidationIssue::MissingRequiredField {
                concept_id,
                field_key,
                ..
            } => Self {
                node_id: issue.node_id.clone(),
                issue_type: "MissingRequiredField".to_string(),
                concept_id: Some(concept_id.clone()),
                field_key: Some(field_key.clone()),
                alias_field_key: None,
                canonical_field_key: None,
                replaced_by: None,
            },
            MetadataValidationIssue::DisallowedField {
                concept_id,
                field_key,
                ..
            } => Self {
                node_id: issue.node_id.clone(),
                issue_type: "DisallowedField".to_string(),
                concept_id: Some(concept_id.clone()),
                field_key: Some(field_key.clone()),
                alias_field_key: None,
                canonical_field_key: None,
                replaced_by: None,
            },
            MetadataValidationIssue::InvalidAllowedValue {
                field_key,
                ..
            } => Self {
                node_id: issue.node_id.clone(),
                issue_type: "InvalidAllowedValue".to_string(),
                concept_id: None,
                field_key: Some(field_key.clone()),
                alias_field_key: None,
                canonical_field_key: None,
                replaced_by: None,
            },
            MetadataValidationIssue::InvalidValueType {
                field_key,
                ..
            } => Self {
                node_id: issue.node_id.clone(),
                issue_type: "InvalidValueType".to_string(),
                concept_id: None,
                field_key: Some(field_key.clone()),
                alias_field_key: None,
                canonical_field_key: None,
                replaced_by: None,
            },
        }
    }
}

impl RepositoryEvidenceHygieneDiagnosticDtoV1 {
    // 2026-04-12 CST: Added hygiene diagnostic conversion so callers can inspect
    // stable detail records without depending on internal enum shapes.
    // Purpose: preserve detail export while keeping the contract versioned.
    fn from_diagnostic(diagnostic: &RepositoryEvidenceHygieneDiagnostic) -> Self {
        match diagnostic {
            RepositoryEvidenceHygieneDiagnostic::MissingEvidenceRef { node_id } => Self {
                diagnostic_type: "MissingEvidenceRef".to_string(),
                severity: RepositoryHygieneSeverityDtoV1::from_internal(
                    &hygiene_severity_for_diagnostic(diagnostic),
                ),
                node_id: Some(node_id.clone()),
                node_ids: vec![node_id.clone()],
                source_ref: None,
                locator: None,
                reason: None,
                occurrence_count: None,
            },
            RepositoryEvidenceHygieneDiagnostic::DuplicateEvidenceRefWithinNode {
                node_id,
                source_ref,
                locator,
                occurrence_count,
            } => Self {
                diagnostic_type: "DuplicateEvidenceRefWithinNode".to_string(),
                severity: RepositoryHygieneSeverityDtoV1::from_internal(
                    &hygiene_severity_for_diagnostic(diagnostic),
                ),
                node_id: Some(node_id.clone()),
                node_ids: vec![node_id.clone()],
                source_ref: Some(source_ref.clone()),
                locator: Some(locator.clone()),
                reason: None,
                occurrence_count: Some(*occurrence_count),
            },
            RepositoryEvidenceHygieneDiagnostic::DuplicateEvidenceRef {
                source_ref,
                locator,
                node_ids,
            } => Self {
                diagnostic_type: "DuplicateEvidenceRef".to_string(),
                severity: RepositoryHygieneSeverityDtoV1::from_internal(
                    &hygiene_severity_for_diagnostic(diagnostic),
                ),
                node_id: None,
                node_ids: node_ids.clone(),
                source_ref: Some(source_ref.clone()),
                locator: Some(locator.clone()),
                reason: None,
                occurrence_count: None,
            },
            RepositoryEvidenceHygieneDiagnostic::WeakLocator {
                node_id,
                source_ref,
                locator,
                reason,
            } => Self {
                diagnostic_type: "WeakLocator".to_string(),
                severity: RepositoryHygieneSeverityDtoV1::from_internal(
                    &hygiene_severity_for_diagnostic(diagnostic),
                ),
                node_id: Some(node_id.clone()),
                node_ids: vec![node_id.clone()],
                source_ref: Some(source_ref.clone()),
                locator: Some(locator.clone()),
                reason: Some(weak_locator_reason_key(reason).to_string()),
                occurrence_count: None,
            },
            RepositoryEvidenceHygieneDiagnostic::WeakSourceRef {
                node_id,
                source_ref,
                locator,
                reason,
            } => Self {
                diagnostic_type: "WeakSourceRef".to_string(),
                severity: RepositoryHygieneSeverityDtoV1::from_internal(
                    &hygiene_severity_for_diagnostic(diagnostic),
                ),
                node_id: Some(node_id.clone()),
                node_ids: vec![node_id.clone()],
                source_ref: Some(source_ref.clone()),
                locator: Some(locator.clone()),
                reason: Some(weak_source_ref_reason_key(reason).to_string()),
                occurrence_count: None,
            },
        }
    }
}

impl RepositoryHygieneSeverityDtoV1 {
    // 2026-04-12 CST: Added stable string access because tests and future JSON
    // serializers should not rely on Rust debug formatting. Purpose: keep the
    // exported vocabulary explicit and deterministic.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Critical => "Critical",
            Self::Warning => "Warning",
            Self::Info => "Info",
        }
    }

    fn from_internal(severity: &RepositoryHygieneSeverity) -> Self {
        match severity {
            RepositoryHygieneSeverity::Critical => Self::Critical,
            RepositoryHygieneSeverity::Warning => Self::Warning,
            RepositoryHygieneSeverity::Info => Self::Info,
        }
    }
}

impl RepositoryEvidenceHygieneSummaryDtoV1 {
    // 2026-04-12 CST: Added summary conversion so the export contract mirrors
    // the already-locked aggregate semantics exactly. Purpose: make v1 a stable
    // projection instead of a reinterpretation layer.
    fn from_summary(summary: &RepositoryEvidenceHygieneSummary) -> Self {
        Self {
            total_diagnostics: summary.total_diagnostics,
            severity_counts: summary.severity_counts.clone(),
            diagnostic_type_counts: summary.diagnostic_type_counts.clone(),
            weak_locator_reason_counts: summary.weak_locator_reason_counts.clone(),
            weak_source_ref_reason_counts: summary.weak_source_ref_reason_counts.clone(),
            affected_node_count: summary.affected_node_count,
            has_blocking_hygiene_issue: summary.has_blocking_hygiene_issue,
        }
    }
}

impl RepositoryEvidenceHygieneViewsDtoV1 {
    // 2026-04-12 CST: Added grouped-view conversion because grouped routing is
    // one of the main consumer surfaces for the foundation audit output.
    // Purpose: export the internal grouped views without exposing their structs.
    fn from_views(views: &RepositoryEvidenceHygieneViews) -> Self {
        Self {
            by_severity: views
                .by_severity
                .iter()
                .map(|group| RepositoryEvidenceHygieneSeverityGroupDtoV1 {
                    severity: RepositoryHygieneSeverityDtoV1::from_internal(&group.severity),
                    diagnostic_count: group.diagnostic_count,
                    affected_node_count: group.affected_node_count,
                    node_ids: group.node_ids.clone(),
                })
                .collect(),
            by_node: views
                .by_node
                .iter()
                .map(|group| RepositoryEvidenceHygieneNodeGroupDtoV1 {
                    node_id: group.node_id.clone(),
                    highest_severity: RepositoryHygieneSeverityDtoV1::from_internal(
                        &group.highest_severity,
                    ),
                    diagnostic_count: group.diagnostic_count,
                    diagnostic_type_counts: group.diagnostic_type_counts.clone(),
                })
                .collect(),
        }
    }
}

impl RepositoryEvidenceHygieneReasonViewsDtoV1 {
    // 2026-04-12 CST: Added reason-view conversion so v1 exports the reason-first
    // governance surface with the same ordering and count semantics as the
    // internal report. Purpose: stabilize downstream AI routing.
    fn from_reason_views(reason_views: &RepositoryEvidenceHygieneReasonViews) -> Self {
        Self {
            weak_locator_by_reason: reason_views
                .weak_locator_by_reason
                .iter()
                .map(|group| RepositoryWeakLocatorReasonGroupDtoV1 {
                    reason: RepositoryWeakLocatorReasonDtoV1::from_internal(&group.reason),
                    severity: RepositoryHygieneSeverityDtoV1::from_internal(&group.severity),
                    diagnostic_count: group.diagnostic_count,
                    affected_node_count: group.affected_node_count,
                    node_ids: group.node_ids.clone(),
                })
                .collect(),
            weak_source_ref_by_reason: reason_views
                .weak_source_ref_by_reason
                .iter()
                .map(|group| RepositoryWeakSourceRefReasonGroupDtoV1 {
                    reason: RepositoryWeakSourceRefReasonDtoV1::from_internal(&group.reason),
                    severity: RepositoryHygieneSeverityDtoV1::from_internal(&group.severity),
                    diagnostic_count: group.diagnostic_count,
                    affected_node_count: group.affected_node_count,
                    node_ids: group.node_ids.clone(),
                })
                .collect(),
        }
    }
}

impl RepositoryWeakLocatorReasonDtoV1 {
    // 2026-04-12 CST: Added stable string access for exported locator reasons so
    // callers can consume explicit vocabulary values. Purpose: avoid depending on
    // debug output or internal enum names.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Blank => "Blank",
            Self::TooShort => "TooShort",
            Self::SheetOnly => "SheetOnly",
            Self::SingleCellOnly => "SingleCellOnly",
            Self::AmbiguousKeyword => "AmbiguousKeyword",
            Self::InvalidRangeFormat => "InvalidRangeFormat",
        }
    }

    fn from_internal(reason: &RepositoryWeakLocatorReason) -> Self {
        match reason {
            RepositoryWeakLocatorReason::Blank => Self::Blank,
            RepositoryWeakLocatorReason::TooShort => Self::TooShort,
            RepositoryWeakLocatorReason::SheetOnly => Self::SheetOnly,
            RepositoryWeakLocatorReason::SingleCellOnly => Self::SingleCellOnly,
            RepositoryWeakLocatorReason::AmbiguousKeyword => Self::AmbiguousKeyword,
            RepositoryWeakLocatorReason::InvalidRangeFormat => Self::InvalidRangeFormat,
        }
    }
}

impl RepositoryWeakSourceRefReasonDtoV1 {
    // 2026-04-12 CST: Added stable string access for exported source-ref reasons
    // so the v1 contract has an explicit, versioned vocabulary.
    // Purpose: keep external consumers away from internal enum formatting.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Blank => "Blank",
            Self::TooShort => "TooShort",
            Self::MissingNamespace => "MissingNamespace",
            Self::EntityMissing => "EntityMissing",
            Self::ContainsWhitespace => "ContainsWhitespace",
            Self::InvalidCharacter => "InvalidCharacter",
            Self::UnknownNamespace => "UnknownNamespace",
        }
    }

    fn from_internal(reason: &RepositoryWeakSourceRefReason) -> Self {
        match reason {
            RepositoryWeakSourceRefReason::Blank => Self::Blank,
            RepositoryWeakSourceRefReason::TooShort => Self::TooShort,
            RepositoryWeakSourceRefReason::MissingNamespace => Self::MissingNamespace,
            RepositoryWeakSourceRefReason::EntityMissing => Self::EntityMissing,
            RepositoryWeakSourceRefReason::ContainsWhitespace => Self::ContainsWhitespace,
            RepositoryWeakSourceRefReason::InvalidCharacter => Self::InvalidCharacter,
            RepositoryWeakSourceRefReason::UnknownNamespace => Self::UnknownNamespace,
        }
    }
}

// 2026-04-10 CST: 这里收口 issue 类型名映射，原因是 repository audit 需要做按 issue 类型聚合，
// 但当前不想再额外引入一套重复的 issue kind 枚举。目的：稳定输出聚合键，同时复用现有 validator 契约。
fn issue_type_key(issue: &MetadataValidationIssue) -> &'static str {
    match issue {
        MetadataValidationIssue::AliasFieldUsed { .. } => "AliasFieldUsed",
        MetadataValidationIssue::DeprecatedFieldUsed { .. } => "DeprecatedFieldUsed",
        MetadataValidationIssue::MissingConceptPolicy { .. } => "MissingConceptPolicy",
        MetadataValidationIssue::MissingRequiredField { .. } => "MissingRequiredField",
        MetadataValidationIssue::DisallowedField { .. } => "DisallowedField",
        MetadataValidationIssue::InvalidAllowedValue { .. } => "InvalidAllowedValue",
        MetadataValidationIssue::InvalidValueType { .. } => "InvalidValueType",
    }
}

// 2026-04-10 CST: 这里提取可归属 concept 的 issue，原因是 repository audit 只需要对真正带 concept 语义的 issue
// 做 concept 聚合，没必要给 alias、type、value 之类问题虚构归属。目的：保持 concept_issue_counts 口径清晰。
fn concept_id_for_issue(issue: &MetadataValidationIssue) -> Option<&str> {
    match issue {
        MetadataValidationIssue::MissingConceptPolicy { concept_id, .. } => Some(concept_id),
        MetadataValidationIssue::MissingRequiredField { concept_id, .. } => Some(concept_id),
        MetadataValidationIssue::DisallowedField { concept_id, .. } => Some(concept_id),
        MetadataValidationIssue::AliasFieldUsed { .. }
        | MetadataValidationIssue::DeprecatedFieldUsed { .. }
        | MetadataValidationIssue::InvalidAllowedValue { .. }
        | MetadataValidationIssue::InvalidValueType { .. } => None,
    }
}

// 2026-04-10 CST: 这里集中收集最小 evidence hygiene diagnostics，原因是仓库级诊断不属于节点 validator 的职责，
// 更适合在 repository audit 层统一观察。目的：补上 duplicate evidence、weak locator、weak source_ref 等只读诊断。
fn collect_hygiene_diagnostics(
    repository: &KnowledgeRepository,
) -> Vec<RepositoryEvidenceHygieneDiagnostic> {
    let mut evidence_node_index: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
    let mut missing_evidence_diagnostics = Vec::new();
    let mut duplicate_within_node_diagnostics = Vec::new();
    let mut weak_locator_diagnostics = Vec::new();
    let mut weak_source_ref_diagnostics = Vec::new();

    for node in &repository.bundle().nodes {
        // 2026-04-10 CST: 这里先判定节点是否完全缺少 evidence_refs，原因是这类问题属于节点级完整性缺口，但更适合在 repository audit 层统一暴露。
        // 目的：把“缺证据节点”收敛到 hygiene diagnostics，避免污染节点级 metadata validator 职责。
        if node.evidence_refs.is_empty() {
            missing_evidence_diagnostics.push(
                RepositoryEvidenceHygieneDiagnostic::MissingEvidenceRef {
                    node_id: node.id.clone(),
                },
            );
        }

        let mut node_evidence_counts: BTreeMap<(String, String), usize> = BTreeMap::new();

        for evidence_ref in &node.evidence_refs {
            record_duplicate_candidate(&mut evidence_node_index, &node.id, evidence_ref);
            *node_evidence_counts
                .entry((
                    evidence_ref.source_ref.clone(),
                    evidence_ref.locator.clone(),
                ))
                .or_insert(0) += 1;

            let source_ref_reason = weak_source_ref_reason(&evidence_ref.source_ref);

            // 2026-04-11 CST: 这里让 locator 结构诊断依赖 source_ref 先基本可用，原因是来源已经空白/过短/缺 namespace 时，
            // 再继续给 locator 打结构弱提示会把同一条 evidence 的主问题稀释掉。目的：保持 diagnostics 主次清楚，避免重复噪声。
            if source_ref_reason.is_none()
                && let Some(reason) = weak_locator_reason(&evidence_ref.locator)
            {
                weak_locator_diagnostics.push(RepositoryEvidenceHygieneDiagnostic::WeakLocator {
                    node_id: node.id.clone(),
                    source_ref: evidence_ref.source_ref.clone(),
                    locator: evidence_ref.locator.clone(),
                    reason,
                });
            }

            if let Some(reason) = source_ref_reason {
                weak_source_ref_diagnostics.push(
                    RepositoryEvidenceHygieneDiagnostic::WeakSourceRef {
                        node_id: node.id.clone(),
                        source_ref: evidence_ref.source_ref.clone(),
                        locator: evidence_ref.locator.clone(),
                        reason,
                    },
                );
            }
        }

        // 2026-04-10 CST: 这里在节点扫描结束后再收集同节点重复，原因是需要保留 occurrence_count，而这依赖节点内累计次数。
        // 目的：保证 DuplicateEvidenceRefWithinNode 既稳定有序，又不影响跨节点重复索引的原有职责。
        for ((source_ref, locator), occurrence_count) in node_evidence_counts {
            if occurrence_count > 1 {
                duplicate_within_node_diagnostics.push(
                    RepositoryEvidenceHygieneDiagnostic::DuplicateEvidenceRefWithinNode {
                        node_id: node.id.clone(),
                        source_ref,
                        locator,
                        occurrence_count,
                    },
                );
            }
        }
    }

    let mut diagnostics = Vec::new();
    diagnostics.extend(missing_evidence_diagnostics);
    diagnostics.extend(duplicate_within_node_diagnostics);

    for ((source_ref, locator), node_ids) in evidence_node_index {
        if node_ids.len() > 1 {
            diagnostics.push(RepositoryEvidenceHygieneDiagnostic::DuplicateEvidenceRef {
                source_ref,
                locator,
                node_ids,
            });
        }
    }

    diagnostics.extend(weak_locator_diagnostics);
    diagnostics.extend(weak_source_ref_diagnostics);
    diagnostics
}

// 2026-04-12 CST: Added a dedicated summary pass over hygiene diagnostics so the
// report can provide stable aggregate governance signals without changing the
// existing detail-list contract. Purpose: support AI-driven routing on top of the
// foundation roaming audit output.
fn summarize_hygiene_diagnostics(
    diagnostics: &[RepositoryEvidenceHygieneDiagnostic],
) -> RepositoryEvidenceHygieneSummary {
    let mut severity_counts = BTreeMap::new();
    let mut diagnostic_type_counts = BTreeMap::new();
    let mut weak_locator_reason_counts = BTreeMap::new();
    let mut weak_source_ref_reason_counts = BTreeMap::new();
    let mut affected_node_ids = BTreeSet::new();
    let mut has_blocking_hygiene_issue = false;

    for diagnostic in diagnostics {
        let severity = hygiene_severity_for_diagnostic(diagnostic);
        let severity_key = hygiene_severity_key(&severity).to_string();
        *severity_counts.entry(severity_key).or_insert(0) += 1;

        if matches!(severity, RepositoryHygieneSeverity::Critical) {
            has_blocking_hygiene_issue = true;
        }

        let diagnostic_type_key = hygiene_diagnostic_type_key(diagnostic).to_string();
        *diagnostic_type_counts.entry(diagnostic_type_key).or_insert(0) += 1;

        if let Some(node_id) = hygiene_diagnostic_node_id(diagnostic) {
            affected_node_ids.insert(node_id.to_string());
        } else if let RepositoryEvidenceHygieneDiagnostic::DuplicateEvidenceRef { node_ids, .. } =
            diagnostic
        {
            for node_id in node_ids {
                affected_node_ids.insert(node_id.clone());
            }
        }

        if let RepositoryEvidenceHygieneDiagnostic::WeakLocator { reason, .. } = diagnostic {
            let reason_key = weak_locator_reason_key(reason).to_string();
            *weak_locator_reason_counts.entry(reason_key).or_insert(0) += 1;
        }

        if let RepositoryEvidenceHygieneDiagnostic::WeakSourceRef { reason, .. } = diagnostic {
            let reason_key = weak_source_ref_reason_key(reason).to_string();
            *weak_source_ref_reason_counts.entry(reason_key).or_insert(0) += 1;
        }
    }

    RepositoryEvidenceHygieneSummary {
        total_diagnostics: diagnostics.len(),
        severity_counts,
        diagnostic_type_counts,
        weak_locator_reason_counts,
        weak_source_ref_reason_counts,
        affected_node_count: affected_node_ids.len(),
        has_blocking_hygiene_issue,
    }
}

// 2026-04-12 CST: Added grouped hygiene-view construction so the repository
// audit can serve execution-friendly routing views alongside the aggregate
// summary. Purpose: keep this transformation centralized and deterministic.
fn build_hygiene_views(
    diagnostics: &[RepositoryEvidenceHygieneDiagnostic],
) -> RepositoryEvidenceHygieneViews {
    let by_severity = build_hygiene_views_by_severity(diagnostics);
    let by_node = build_hygiene_views_by_node(diagnostics);

    RepositoryEvidenceHygieneViews {
        by_severity,
        by_node,
    }
}

// 2026-04-12 CST: Added grouped reason-view construction so the foundation
// audit can expose weak-cause-first governance entry points beside summary and
// node/severity views. Purpose: keep all reason aggregation deterministic and
// local to repository audit.
fn build_hygiene_reason_views(
    diagnostics: &[RepositoryEvidenceHygieneDiagnostic],
) -> RepositoryEvidenceHygieneReasonViews {
    let weak_locator_by_reason = build_weak_locator_reason_groups(diagnostics);
    let weak_source_ref_by_reason = build_weak_source_ref_reason_groups(diagnostics);

    RepositoryEvidenceHygieneReasonViews {
        weak_locator_by_reason,
        weak_source_ref_by_reason,
    }
}

// 2026-04-12 CST: Added severity-group construction because the summary counts
// alone are not enough to know which concrete nodes sit under each severity
// bucket. Purpose: provide a compact routing index by severity.
fn build_hygiene_views_by_severity(
    diagnostics: &[RepositoryEvidenceHygieneDiagnostic],
) -> Vec<RepositoryEvidenceHygieneSeverityGroup> {
    let mut groups = Vec::new();

    for severity in [
        RepositoryHygieneSeverity::Critical,
        RepositoryHygieneSeverity::Warning,
        RepositoryHygieneSeverity::Info,
    ] {
        let mut diagnostic_count = 0usize;
        let mut node_ids = BTreeSet::new();

        for diagnostic in diagnostics {
            if hygiene_severity_for_diagnostic(diagnostic) == severity {
                diagnostic_count += 1;
                for node_id in hygiene_diagnostic_node_ids(diagnostic) {
                    node_ids.insert(node_id.to_string());
                }
            }
        }

        groups.push(RepositoryEvidenceHygieneSeverityGroup {
            severity,
            diagnostic_count,
            affected_node_count: node_ids.len(),
            node_ids: node_ids.into_iter().collect(),
        });
    }

    groups
}

// 2026-04-12 CST: Added node-group construction because downstream governance
// often needs a sorted cleanup target list instead of flat diagnostics. Purpose:
// expose highest severity and per-node issue shape in one stable structure.
fn build_hygiene_views_by_node(
    diagnostics: &[RepositoryEvidenceHygieneDiagnostic],
) -> Vec<RepositoryEvidenceHygieneNodeGroup> {
    let mut diagnostic_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut highest_severity_by_node: BTreeMap<String, RepositoryHygieneSeverity> = BTreeMap::new();
    let mut diagnostic_type_counts_by_node: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();

    for diagnostic in diagnostics {
        let severity = hygiene_severity_for_diagnostic(diagnostic);
        let diagnostic_type_key = hygiene_diagnostic_type_key(diagnostic).to_string();

        for node_id in hygiene_diagnostic_node_ids(diagnostic) {
            *diagnostic_counts.entry(node_id.to_string()).or_insert(0) += 1;

            highest_severity_by_node
                .entry(node_id.to_string())
                .and_modify(|existing| {
                    if hygiene_severity_rank(&severity) < hygiene_severity_rank(existing) {
                        *existing = severity.clone();
                    }
                })
                .or_insert_with(|| severity.clone());

            *diagnostic_type_counts_by_node
                .entry(node_id.to_string())
                .or_default()
                .entry(diagnostic_type_key.clone())
                .or_insert(0) += 1;
        }
    }

    let mut groups = diagnostic_counts
        .into_iter()
        .map(|(node_id, diagnostic_count)| RepositoryEvidenceHygieneNodeGroup {
            highest_severity: highest_severity_by_node
                .remove(&node_id)
                .unwrap_or(RepositoryHygieneSeverity::Info),
            diagnostic_type_counts: diagnostic_type_counts_by_node
                .remove(&node_id)
                .unwrap_or_default(),
            node_id,
            diagnostic_count,
        })
        .collect::<Vec<_>>();

    groups.sort_by(|left, right| {
        hygiene_severity_rank(&left.highest_severity)
            .cmp(&hygiene_severity_rank(&right.highest_severity))
            .then_with(|| right.diagnostic_count.cmp(&left.diagnostic_count))
            .then_with(|| left.node_id.cmp(&right.node_id))
    });

    groups
}

// 2026-04-12 CST: Added weak-locator reason grouping so AI routing can see
// which locator hygiene pattern is most common without re-scanning diagnostics.
// Purpose: provide a stable reason-first view for locator cleanup.
fn build_weak_locator_reason_groups(
    diagnostics: &[RepositoryEvidenceHygieneDiagnostic],
) -> Vec<RepositoryWeakLocatorReasonGroup> {
    let mut groups: BTreeMap<RepositoryWeakLocatorReason, BTreeSet<String>> = BTreeMap::new();

    for diagnostic in diagnostics {
        if let RepositoryEvidenceHygieneDiagnostic::WeakLocator {
            node_id, reason, ..
        } = diagnostic
        {
            groups
                .entry(reason.clone())
                .or_default()
                .insert(node_id.clone());
        }
    }

    let mut groups = groups
        .into_iter()
        .map(|(reason, node_ids)| RepositoryWeakLocatorReasonGroup {
            severity: RepositoryHygieneSeverity::Warning,
            diagnostic_count: node_ids.len(),
            affected_node_count: node_ids.len(),
            node_ids: node_ids.into_iter().collect(),
            reason,
        })
        .collect::<Vec<_>>();

    groups.sort_by(|left, right| {
        hygiene_severity_rank(&left.severity)
            .cmp(&hygiene_severity_rank(&right.severity))
            .then_with(|| right.diagnostic_count.cmp(&left.diagnostic_count))
            .then_with(|| weak_locator_reason_key(&left.reason).cmp(weak_locator_reason_key(&right.reason)))
    });

    groups
}

// 2026-04-12 CST: Added weak-source-ref reason grouping so the foundation can
// separate blocking source cleanup causes from warning-only ones. Purpose:
// expose one direct governance queue per source reason family.
fn build_weak_source_ref_reason_groups(
    diagnostics: &[RepositoryEvidenceHygieneDiagnostic],
) -> Vec<RepositoryWeakSourceRefReasonGroup> {
    let mut groups: BTreeMap<RepositoryWeakSourceRefReason, BTreeSet<String>> = BTreeMap::new();

    for diagnostic in diagnostics {
        if let RepositoryEvidenceHygieneDiagnostic::WeakSourceRef {
            node_id, reason, ..
        } = diagnostic
        {
            groups
                .entry(reason.clone())
                .or_default()
                .insert(node_id.clone());
        }
    }

    let mut groups = groups
        .into_iter()
        .map(|(reason, node_ids)| RepositoryWeakSourceRefReasonGroup {
            severity: hygiene_severity_for_weak_source_ref_reason(&reason),
            diagnostic_count: node_ids.len(),
            affected_node_count: node_ids.len(),
            node_ids: node_ids.into_iter().collect(),
            reason,
        })
        .collect::<Vec<_>>();

    groups.sort_by(|left, right| {
        hygiene_severity_rank(&left.severity)
            .cmp(&hygiene_severity_rank(&right.severity))
            .then_with(|| right.diagnostic_count.cmp(&left.diagnostic_count))
            .then_with(|| {
                weak_source_ref_reason_key(&left.reason)
                    .cmp(weak_source_ref_reason_key(&right.reason))
            })
    });

    groups
}

// 2026-04-12 CST: Added a stable severity mapping helper so hygiene-summary
// rules stay centralized and testable. Purpose: avoid scattering governance
// severity heuristics through the audit flow.
fn hygiene_severity_for_diagnostic(
    diagnostic: &RepositoryEvidenceHygieneDiagnostic,
) -> RepositoryHygieneSeverity {
    match diagnostic {
        RepositoryEvidenceHygieneDiagnostic::MissingEvidenceRef { .. } => {
            RepositoryHygieneSeverity::Critical
        }
        RepositoryEvidenceHygieneDiagnostic::DuplicateEvidenceRefWithinNode { .. }
        | RepositoryEvidenceHygieneDiagnostic::DuplicateEvidenceRef { .. }
        | RepositoryEvidenceHygieneDiagnostic::WeakLocator { .. } => {
            RepositoryHygieneSeverity::Warning
        }
        RepositoryEvidenceHygieneDiagnostic::WeakSourceRef { reason, .. } => match reason {
            RepositoryWeakSourceRefReason::Blank
            | RepositoryWeakSourceRefReason::TooShort
            | RepositoryWeakSourceRefReason::MissingNamespace
            | RepositoryWeakSourceRefReason::EntityMissing => {
                RepositoryHygieneSeverity::Critical
            }
            RepositoryWeakSourceRefReason::ContainsWhitespace
            | RepositoryWeakSourceRefReason::InvalidCharacter
            | RepositoryWeakSourceRefReason::UnknownNamespace => {
                RepositoryHygieneSeverity::Warning
            }
        },
    }
}

// 2026-04-12 CST: Added a dedicated weak-source reason severity helper because
// reason-view groups need the same blocking semantics without rebuilding fake
// diagnostics. Purpose: keep source reason severity logic centralized.
fn hygiene_severity_for_weak_source_ref_reason(
    reason: &RepositoryWeakSourceRefReason,
) -> RepositoryHygieneSeverity {
    match reason {
        RepositoryWeakSourceRefReason::Blank
        | RepositoryWeakSourceRefReason::TooShort
        | RepositoryWeakSourceRefReason::MissingNamespace
        | RepositoryWeakSourceRefReason::EntityMissing => RepositoryHygieneSeverity::Critical,
        RepositoryWeakSourceRefReason::ContainsWhitespace
        | RepositoryWeakSourceRefReason::InvalidCharacter
        | RepositoryWeakSourceRefReason::UnknownNamespace => RepositoryHygieneSeverity::Warning,
    }
}

// 2026-04-12 CST: Added a stable string key helper for severity aggregation so
// report snapshots remain deterministic. Purpose: match the existing report style
// that uses named count maps instead of serialized enums.
fn hygiene_severity_key(severity: &RepositoryHygieneSeverity) -> &'static str {
    match severity {
        RepositoryHygieneSeverity::Critical => "Critical",
        RepositoryHygieneSeverity::Warning => "Warning",
        RepositoryHygieneSeverity::Info => "Info",
    }
}

// 2026-04-12 CST: Added diagnostic type key mapping because the summary needs a
// compact aggregate view of hygiene classes. Purpose: preserve detail semantics
// while exposing stable bucket names to downstream AI consumers.
fn hygiene_diagnostic_type_key(
    diagnostic: &RepositoryEvidenceHygieneDiagnostic,
) -> &'static str {
    match diagnostic {
        RepositoryEvidenceHygieneDiagnostic::MissingEvidenceRef { .. } => "MissingEvidenceRef",
        RepositoryEvidenceHygieneDiagnostic::DuplicateEvidenceRefWithinNode { .. } => {
            "DuplicateEvidenceRefWithinNode"
        }
        RepositoryEvidenceHygieneDiagnostic::DuplicateEvidenceRef { .. } => "DuplicateEvidenceRef",
        RepositoryEvidenceHygieneDiagnostic::WeakLocator { .. } => "WeakLocator",
        RepositoryEvidenceHygieneDiagnostic::WeakSourceRef { .. } => "WeakSourceRef",
    }
}

// 2026-04-12 CST: Added node-id extraction helper so affected-node counting
// stays centralized. Purpose: keep summary assembly simple and avoid repeated
// pattern matching in the aggregation loop.
fn hygiene_diagnostic_node_id(
    diagnostic: &RepositoryEvidenceHygieneDiagnostic,
) -> Option<&str> {
    match diagnostic {
        RepositoryEvidenceHygieneDiagnostic::MissingEvidenceRef { node_id }
        | RepositoryEvidenceHygieneDiagnostic::DuplicateEvidenceRefWithinNode { node_id, .. }
        | RepositoryEvidenceHygieneDiagnostic::WeakLocator { node_id, .. }
        | RepositoryEvidenceHygieneDiagnostic::WeakSourceRef { node_id, .. } => Some(node_id),
        RepositoryEvidenceHygieneDiagnostic::DuplicateEvidenceRef { .. } => None,
    }
}

// 2026-04-12 CST: Added multi-node extraction because some repository hygiene
// diagnostics are shared across several nodes. Purpose: reuse one canonical path
// for grouped-view construction and affected-node counting.
fn hygiene_diagnostic_node_ids(
    diagnostic: &RepositoryEvidenceHygieneDiagnostic,
) -> Vec<&str> {
    match diagnostic {
        RepositoryEvidenceHygieneDiagnostic::DuplicateEvidenceRef { node_ids, .. } => {
            node_ids.iter().map(String::as_str).collect()
        }
        _ => hygiene_diagnostic_node_id(diagnostic).into_iter().collect(),
    }
}

// 2026-04-12 CST: Added explicit severity ranking so node-group sorting stays
// deterministic and independent of enum declaration order. Purpose: keep tests
// and downstream ordering stable.
fn hygiene_severity_rank(severity: &RepositoryHygieneSeverity) -> usize {
    match severity {
        RepositoryHygieneSeverity::Critical => 0,
        RepositoryHygieneSeverity::Warning => 1,
        RepositoryHygieneSeverity::Info => 2,
    }
}

// 2026-04-12 CST: Added explicit reason-to-key mapping for weak locator summary
// output. Purpose: keep aggregate naming stable even if enum display behavior
// changes later.
fn weak_locator_reason_key(reason: &RepositoryWeakLocatorReason) -> &'static str {
    match reason {
        RepositoryWeakLocatorReason::Blank => "Blank",
        RepositoryWeakLocatorReason::TooShort => "TooShort",
        RepositoryWeakLocatorReason::SheetOnly => "SheetOnly",
        RepositoryWeakLocatorReason::SingleCellOnly => "SingleCellOnly",
        RepositoryWeakLocatorReason::AmbiguousKeyword => "AmbiguousKeyword",
        RepositoryWeakLocatorReason::InvalidRangeFormat => "InvalidRangeFormat",
    }
}

// 2026-04-12 CST: Added explicit reason-to-key mapping for weak source summary
// output. Purpose: make the new hygiene-summary contract deterministic for tests
// and downstream governance logic.
fn weak_source_ref_reason_key(reason: &RepositoryWeakSourceRefReason) -> &'static str {
    match reason {
        RepositoryWeakSourceRefReason::Blank => "Blank",
        RepositoryWeakSourceRefReason::TooShort => "TooShort",
        RepositoryWeakSourceRefReason::MissingNamespace => "MissingNamespace",
        RepositoryWeakSourceRefReason::EntityMissing => "EntityMissing",
        RepositoryWeakSourceRefReason::ContainsWhitespace => "ContainsWhitespace",
        RepositoryWeakSourceRefReason::InvalidCharacter => "InvalidCharacter",
        RepositoryWeakSourceRefReason::UnknownNamespace => "UnknownNamespace",
    }
}

// 2026-04-10 CST: 这里抽出 weak locator 判定，原因是方案A需要在不改动审计主流程结构的前提下扩充定位质量规则。
// 目的：让规则集中、可测试，并为后续继续补 locator 结构规则预留单点扩展口。
fn weak_locator_reason(locator: &str) -> Option<RepositoryWeakLocatorReason> {
    let trimmed = locator.trim();

    if trimmed.is_empty() {
        Some(RepositoryWeakLocatorReason::Blank)
    } else if trimmed.len() < 3 {
        Some(RepositoryWeakLocatorReason::TooShort)
    // 2026-04-11 CST: 这里先判定伪区间格式，原因是像 `A1-B10` 这类值看起来像正式定位，
    // 但不符合当前基础区间语法；若不先拦下，后续会被更弱规则误吞。目的：把“像范围但格式不合法”单独归因。
    } else if looks_like_invalid_range(trimmed) {
        Some(RepositoryWeakLocatorReason::InvalidRangeFormat)
    // 2026-04-11 CST: 这里判定只有 sheet 名的 locator，原因是这类定位有来源上下文但没有单元格上下文，
    // 不足以支撑稳定 evidence 回放。目的：把“只有工作表，没有坐标”的情况从泛化弱定位里分出来。
    } else if is_sheet_only_locator(trimmed) {
        Some(RepositoryWeakLocatorReason::SheetOnly)
    // 2026-04-11 CST: 这里判定单格 locator，原因是本轮方案A要求把“可定位但上下文过窄”的情况从 TooShort 中分离。
    // 目的：让单格引用成为明确的弱提示，而不是长度规则副产物。
    } else if is_single_cell_locator(trimmed) {
        Some(RepositoryWeakLocatorReason::SingleCellOnly)
    // 2026-04-11 CST: 这里判定模糊关键词 locator，原因是 `row/col/data/table/cell` 这类自然语言定位不稳定，
    // 后续 AI 和人工都难以复放。目的：把“词面像定位，结构却不稳定”的情况统一归因。
    } else if contains_ambiguous_locator_keyword(trimmed) {
        Some(RepositoryWeakLocatorReason::AmbiguousKeyword)
    } else {
        None
    }
}

// 2026-04-11 CST: 这里抽出伪区间检测，原因是 locator 结构诊断本轮只想支持最小可解释规则，不引入完整地址解析器。
// 目的：先稳定识别 `A1-B10` 这类最常见伪范围格式，保持实现简单可测。
fn looks_like_invalid_range(locator: &str) -> bool {
    locator.contains('-')
        && locator.chars().any(|ch| ch.is_ascii_digit())
        && locator.chars().any(|ch| ch.is_ascii_alphabetic())
}

// 2026-04-11 CST: 这里抽出 sheet-only 检测，原因是这类值在 Excel 语境里很常见，但单独保留 sheet 名并不能完成证据回放。
// 目的：给 repository audit 一个稳定可解释的“只有 sheet 上下文”诊断口径。
fn is_sheet_only_locator(locator: &str) -> bool {
    let lower = locator.to_ascii_lowercase();
    lower.starts_with("sheet")
        && !locator.contains('!')
        && !locator.contains(':')
        && !locator.contains('-')
}

// 2026-04-11 CST: 这里抽出单格坐标检测，原因是后续可能继续扩到区间、命名区域等规则，不能把模式判断散在主函数里。
// 目的：先把单格引用独立成一类弱定位，后续如扩充坐标解析时只改这里。
fn is_single_cell_locator(locator: &str) -> bool {
    let mut seen_letter = false;
    let mut seen_digit = false;
    let mut digit_started = false;

    for ch in locator.chars() {
        if ch.is_ascii_alphabetic() && !digit_started {
            seen_letter = true;
        } else if ch.is_ascii_digit() && seen_letter {
            seen_digit = true;
            digit_started = true;
        } else {
            return false;
        }
    }

    seen_letter && seen_digit
}

// 2026-04-11 CST: 这里抽出模糊关键词检测，原因是本轮只想覆盖最常见的自然语言定位噪声，不做复杂分词。
// 目的：用最小关键词集把明显不稳定的 locator 描述先识别出来。
fn contains_ambiguous_locator_keyword(locator: &str) -> bool {
    let lower = locator.to_ascii_lowercase();
    ["row", "col", "data", "table", "cell"]
        .iter()
        .any(|keyword| lower.contains(keyword))
}

// 2026-04-10 CST: 这里抽出 weak source_ref 判定，原因是后续 source_ref 规范会继续扩展，不能把条件散落在 audit 主循环里。
// 目的：先稳定空白、过短、缺 namespace 三类规则，后续如补 URI/handle 规范时只需扩展这里。
fn weak_source_ref_reason(source_ref: &str) -> Option<RepositoryWeakSourceRefReason> {
    let trimmed = source_ref.trim();

    if trimmed.is_empty() {
        Some(RepositoryWeakSourceRefReason::Blank)
    } else if trimmed.len() < 4 {
        Some(RepositoryWeakSourceRefReason::TooShort)
    } else if !trimmed.contains(':') {
        Some(RepositoryWeakSourceRefReason::MissingNamespace)
    // 2026-04-12 CST: Added source_ref structure diagnostics for the roaming
    // foundation layer. Purpose: distinguish a missing entity segment from other
    // weak source_ref cases so repository hygiene reports stay actionable.
    } else if has_missing_source_entity(trimmed) {
        Some(RepositoryWeakSourceRefReason::EntityMissing)
    // 2026-04-12 CST: Added whitespace detection after namespace parsing because
    // free-form handles reduce replay stability in the knowledge roaming graph.
    } else if contains_source_whitespace(trimmed) {
        Some(RepositoryWeakSourceRefReason::ContainsWhitespace)
    // 2026-04-12 CST: Added a minimal invalid-character gate to catch unstable
    // source handles before they silently spread through repository diagnostics.
    } else if contains_invalid_source_character(trimmed) {
        Some(RepositoryWeakSourceRefReason::InvalidCharacter)
    // 2026-04-12 CST: Added an allowlist check for the current foundation
    // namespaces so bottom-layer audit output can separate malformed refs from
    // refs that simply point to an unsupported source domain.
    } else if has_unknown_source_namespace(trimmed) {
        Some(RepositoryWeakSourceRefReason::UnknownNamespace)
    } else {
        None
    }
}

// 2026-04-12 CST: Extracted source_ref segment parsing so new structure rules can
// share the same split logic. Purpose: keep the repository audit implementation
// small and make future roaming-source namespace rules easier to extend safely.
fn split_source_ref(source_ref: &str) -> Option<(&str, &str)> {
    source_ref.split_once(':')
}

// 2026-04-12 CST: Added an explicit missing-entity check because `sheet:` style
// refs are structurally different from blank or namespace-missing refs. Purpose:
// keep diagnostics precise for repository cleanup.
fn has_missing_source_entity(source_ref: &str) -> bool {
    split_source_ref(source_ref)
        .map(|(_, entity)| entity.is_empty())
        .unwrap_or(false)
}

// 2026-04-12 CST: Added whitespace detection for source_ref entities because the
// current roaming foundation expects stable machine-oriented handles. Purpose:
// prevent human-readable free text from being treated as a strong source handle.
fn contains_source_whitespace(source_ref: &str) -> bool {
    split_source_ref(source_ref)
        .map(|(_, entity)| entity.chars().any(char::is_whitespace))
        .unwrap_or(false)
}

// 2026-04-12 CST: Added a minimal invalid-character rule instead of a full URI
// parser. Purpose: catch obviously unstable source handles while keeping the
// bottom-layer audit logic lightweight.
fn contains_invalid_source_character(source_ref: &str) -> bool {
    split_source_ref(source_ref)
        .map(|(_, entity)| {
            entity
                .chars()
                .any(|ch| !(ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | '/' | '#')))
        })
        .unwrap_or(false)
}

// 2026-04-12 CST: Added a small namespace allowlist that matches the currently
// supported repository evidence domains. Purpose: report unsupported source kinds
// without forcing a broader ontology or migration change in this task.
fn has_unknown_source_namespace(source_ref: &str) -> bool {
    split_source_ref(source_ref)
        .map(|(namespace, _)| {
            !matches!(namespace, "sheet" | "file" | "workbook" | "table" | "range")
        })
        .unwrap_or(false)
}

// 2026-04-10 CST: 这里抽重复证据候选登记辅助函数，原因是 duplicate evidence 只依赖 source_ref + locator 键，
// 不需要把弱来源或弱定位逻辑也混在一起。目的：让诊断收集逻辑保持清晰，并按仓库扫描顺序保留 node_ids。
fn record_duplicate_candidate(
    evidence_node_index: &mut BTreeMap<(String, String), Vec<String>>,
    node_id: &str,
    evidence_ref: &EvidenceRef,
) {
    let node_ids = evidence_node_index
        .entry((
            evidence_ref.source_ref.clone(),
            evidence_ref.locator.clone(),
        ))
        .or_default();

    if !node_ids.iter().any(|existing| existing == node_id) {
        node_ids.push(node_id.to_string());
    }
}
