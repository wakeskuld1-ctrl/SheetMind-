use excel_skill::ops::foundation::knowledge_bundle::KnowledgeBundle;
use excel_skill::ops::foundation::knowledge_record::{
    EvidenceRef, KnowledgeEdge, KnowledgeNode,
};
use excel_skill::ops::foundation::knowledge_repository::KnowledgeRepository;
use excel_skill::ops::foundation::metadata_schema::{
    ConceptMetadataPolicy, MetadataFieldDefinition, MetadataSchema, MetadataValueType,
};
use excel_skill::ops::foundation::metadata_validator::MetadataValidationIssue;
use excel_skill::ops::foundation::ontology_schema::{OntologyConcept, OntologyRelationType};
use excel_skill::ops::foundation::repository_metadata_audit::{
    RepositoryEvidenceHygieneDiagnostic, RepositoryMetadataAudit,
    RepositoryWeakLocatorReason, RepositoryWeakSourceRefReason,
};

// 2026-04-10 CST: 这里先补 repository-level audit 红测，原因是当前 foundation 只有节点级 validator，
// 还没有把问题提升成仓库级报告。目的：先钉住“聚合 issue + hygiene diagnostics”的正式契约，再做最小实现。
#[test]
fn repository_metadata_audit_aggregates_validator_issues_and_hygiene_diagnostics() {
    let schema = sample_schema();
    let repository = sample_repository();

    let report = RepositoryMetadataAudit::new(&schema).audit(&repository);

    assert_eq!(report.total_nodes, 7);
    assert_eq!(report.audited_nodes, 7);
    assert_eq!(report.issue_count, 4);

    assert_eq!(
        report.issue_type_counts.get("MissingRequiredField"),
        Some(&1usize)
    );
    assert_eq!(report.issue_type_counts.get("DisallowedField"), Some(&1usize));
    assert_eq!(report.issue_type_counts.get("AliasFieldUsed"), Some(&1usize));
    assert_eq!(report.issue_type_counts.get("DeprecatedFieldUsed"), Some(&1usize));

    assert_eq!(report.concept_issue_counts.get("revenue"), Some(&2usize));

    assert_eq!(
        report.issues[0].issue,
        MetadataValidationIssue::MissingRequiredField {
            node_id: "node-revenue-missing".to_string(),
            concept_id: "revenue".to_string(),
            field_key: "domain".to_string(),
        }
    );
    assert_eq!(
        report.issues[1].issue,
        MetadataValidationIssue::DisallowedField {
            node_id: "node-revenue-owner".to_string(),
            concept_id: "revenue".to_string(),
            field_key: "owner".to_string(),
        }
    );
    assert_eq!(
        report.issues[2].issue,
        MetadataValidationIssue::AliasFieldUsed {
            node_id: "node-revenue-governance".to_string(),
            alias_field_key: "old_domain".to_string(),
            canonical_field_key: "legacy_domain".to_string(),
        }
    );
    assert_eq!(
        report.issues[3].issue,
        MetadataValidationIssue::DeprecatedFieldUsed {
            node_id: "node-revenue-governance".to_string(),
            field_key: "legacy_domain".to_string(),
            replaced_by: Some("domain".to_string()),
        }
    );

    assert_eq!(
        report.hygiene_diagnostics,
        vec![
            RepositoryEvidenceHygieneDiagnostic::MissingEvidenceRef {
                node_id: "node-revenue-missing".to_string(),
            },
            RepositoryEvidenceHygieneDiagnostic::DuplicateEvidenceRefWithinNode {
                node_id: "node-revenue-owner".to_string(),
                source_ref: "sheet:sales".to_string(),
                locator: "A1:B12".to_string(),
                occurrence_count: 2,
            },
            RepositoryEvidenceHygieneDiagnostic::DuplicateEvidenceRef {
                source_ref: "sheet:sales".to_string(),
                locator: "A1:B12".to_string(),
                node_ids: vec![
                    "node-revenue-owner".to_string(),
                    "node-revenue-governance".to_string(),
                ],
            },
            RepositoryEvidenceHygieneDiagnostic::WeakLocator {
                node_id: "node-revenue-blank-locator".to_string(),
                source_ref: "sheet:finance".to_string(),
                locator: "   ".to_string(),
                reason: RepositoryWeakLocatorReason::Blank,
            },
            RepositoryEvidenceHygieneDiagnostic::WeakLocator {
                node_id: "node-revenue-short-locator".to_string(),
                source_ref: "sheet:ops".to_string(),
                locator: "A".to_string(),
                reason: RepositoryWeakLocatorReason::TooShort,
            },
            RepositoryEvidenceHygieneDiagnostic::WeakSourceRef {
                node_id: "node-revenue-governance".to_string(),
                source_ref: "".to_string(),
                locator: "A1:B12".to_string(),
                reason: RepositoryWeakSourceRefReason::Blank,
            },
            RepositoryEvidenceHygieneDiagnostic::WeakSourceRef {
                node_id: "node-revenue-short-source".to_string(),
                source_ref: "sh".to_string(),
                locator: "B20".to_string(),
                reason: RepositoryWeakSourceRefReason::TooShort,
            },
            RepositoryEvidenceHygieneDiagnostic::WeakSourceRef {
                node_id: "node-revenue-missing-namespace".to_string(),
                source_ref: "sheetsales".to_string(),
                locator: "C10".to_string(),
                reason: RepositoryWeakSourceRefReason::MissingNamespace,
            },
        ]
    );
}

// 2026-04-10 CST: 这里集中构造 repository audit 的最小 schema，原因是红测只关心 repository 级聚合，
// 不应把无关字段和复杂治理噪音混进样本。目的：让红测失败原因稳定指向 audit 缺失，而不是 fixture 过重。
fn sample_schema() -> MetadataSchema {
    MetadataSchema::new(
        vec![
            MetadataFieldDefinition::new("domain", MetadataValueType::String)
                .with_allowed_value("finance")
                .with_allowed_value("operations")
                .with_alias("biz_domain"),
            MetadataFieldDefinition::new("legacy_domain", MetadataValueType::String)
                .with_allowed_value("finance")
                .with_allowed_value("operations")
                .deprecated()
                .with_replaced_by("domain")
                .with_alias("old_domain"),
            MetadataFieldDefinition::new("owner", MetadataValueType::String),
        ],
        vec![
            ConceptMetadataPolicy::new("revenue")
                .with_allowed_field("domain")
                .with_allowed_field("legacy_domain")
                .with_required_field("domain"),
        ],
    )
    .expect("sample schema should be valid")
}

// 2026-04-10 CST: 这里集中构造 repository audit 的最小样本仓库，原因是我们需要同时覆盖节点级 issue 聚合
// 和 evidence hygiene diagnostics。目的：用一个小样本同时触发 required/disallowed/alias/deprecated 与重复证据、弱定位、弱来源。
fn sample_repository() -> KnowledgeRepository {
    KnowledgeRepository::new(KnowledgeBundle::new(
        "foundation.v1",
        vec![OntologyConcept::new("revenue", "Revenue")],
        vec![],
        vec![
            KnowledgeNode::new(
                "node-revenue-missing",
                "Revenue Missing Domain",
                "Body",
            )
            .with_concept_id("revenue"),
            KnowledgeNode::new(
                "node-revenue-owner",
                "Revenue Owner Field",
                "Body",
            )
            .with_concept_id("revenue")
            .with_metadata_entry("domain", "finance")
            .with_metadata_entry("owner", "ops-team")
            .with_evidence_ref(EvidenceRef::new("sheet:sales", "A1:B12"))
            .with_evidence_ref(EvidenceRef::new("sheet:sales", "A1:B12")),
            KnowledgeNode::new(
                "node-revenue-governance",
                "Revenue Governance",
                "Body",
            )
            .with_concept_id("revenue")
            .with_metadata_entry("old_domain", "finance")
            .with_evidence_ref(EvidenceRef::new("", "A1:B12"))
            .with_evidence_ref(EvidenceRef::new("sheet:sales", "A1:B12")),
            KnowledgeNode::new(
                "node-revenue-blank-locator",
                "Revenue Blank Locator",
                "Body",
            )
            .with_concept_id("revenue")
            .with_metadata_entry("domain", "finance")
            .with_evidence_ref(EvidenceRef::new("sheet:finance", "   ")),
            KnowledgeNode::new(
                "node-revenue-short-locator",
                "Revenue Short Locator",
                "Body",
            )
            .with_concept_id("revenue")
            .with_metadata_entry("domain", "finance")
            .with_evidence_ref(EvidenceRef::new("sheet:ops", "A")),
            KnowledgeNode::new(
                "node-revenue-short-source",
                "Revenue Short Source",
                "Body",
            )
            .with_concept_id("revenue")
            .with_metadata_entry("domain", "finance")
            .with_evidence_ref(EvidenceRef::new("sh", "B20")),
            KnowledgeNode::new(
                "node-revenue-missing-namespace",
                "Revenue Missing Namespace",
                "Body",
            )
            .with_concept_id("revenue")
            .with_metadata_entry("domain", "finance")
            .with_evidence_ref(EvidenceRef::new("sheetsales", "C10")),
        ],
        vec![KnowledgeEdge::new(
            "node-revenue-owner",
            "node-revenue-governance",
            OntologyRelationType::References,
        )],
    ))
    .expect("sample repository should be valid")
}
