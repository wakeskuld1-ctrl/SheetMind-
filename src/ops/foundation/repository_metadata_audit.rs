use std::collections::BTreeMap;

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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepositoryWeakLocatorReason {
    Blank,
    TooShort,
}

// 2026-04-10 CST: 这里补充 weak source_ref 原因枚举，原因是当前弱来源只会报“弱”，不足以支撑后续治理优先级判断。
// 目的：明确区分空白、过短、缺少 namespace 三类问题，保持诊断层只读但可追踪。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepositoryWeakSourceRefReason {
    Blank,
    TooShort,
    MissingNamespace,
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

        RepositoryMetadataAuditReport {
            total_nodes: repository.bundle().nodes.len(),
            audited_nodes: repository.bundle().nodes.len(),
            issue_count: issues.len(),
            issues,
            issue_type_counts,
            concept_issue_counts,
            hygiene_diagnostics,
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
                .entry((evidence_ref.source_ref.clone(), evidence_ref.locator.clone()))
                .or_insert(0) += 1;

            if let Some(reason) = weak_locator_reason(&evidence_ref.locator) {
                weak_locator_diagnostics.push(RepositoryEvidenceHygieneDiagnostic::WeakLocator {
                    node_id: node.id.clone(),
                    source_ref: evidence_ref.source_ref.clone(),
                    locator: evidence_ref.locator.clone(),
                    reason,
                });
            }

            if let Some(reason) = weak_source_ref_reason(&evidence_ref.source_ref) {
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

// 2026-04-10 CST: 这里抽出 weak locator 判定，原因是方案A需要在不改动审计主流程结构的前提下扩充定位质量规则。
// 目的：让规则集中、可测试，并为后续继续补 locator 结构规则预留单点扩展口。
fn weak_locator_reason(locator: &str) -> Option<RepositoryWeakLocatorReason> {
    let trimmed = locator.trim();

    if trimmed.is_empty() {
        Some(RepositoryWeakLocatorReason::Blank)
    } else if trimmed.len() < 3 {
        Some(RepositoryWeakLocatorReason::TooShort)
    } else {
        None
    }
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
    } else {
        None
    }
}

// 2026-04-10 CST: 这里抽重复证据候选登记辅助函数，原因是 duplicate evidence 只依赖 source_ref + locator 键，
// 不需要把弱来源或弱定位逻辑也混在一起。目的：让诊断收集逻辑保持清晰，并按仓库扫描顺序保留 node_ids。
fn record_duplicate_candidate(
    evidence_node_index: &mut BTreeMap<(String, String), Vec<String>>,
    node_id: &str,
    evidence_ref: &EvidenceRef,
) {
    let node_ids = evidence_node_index
        .entry((evidence_ref.source_ref.clone(), evidence_ref.locator.clone()))
        .or_default();

    if !node_ids.iter().any(|existing| existing == node_id) {
        node_ids.push(node_id.to_string());
    }
}
