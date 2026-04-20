use excel_skill::ops::foundation::knowledge_bundle::KnowledgeBundle;
use excel_skill::ops::foundation::knowledge_record::{EvidenceRef, KnowledgeEdge, KnowledgeNode};
use excel_skill::ops::foundation::knowledge_repository::KnowledgeRepository;
use excel_skill::ops::foundation::metadata_schema::{
    ConceptMetadataPolicy, MetadataFieldDefinition, MetadataSchema, MetadataValueType,
};
use excel_skill::ops::foundation::metadata_validator::MetadataValidationIssue;
use excel_skill::ops::foundation::ontology_schema::{OntologyConcept, OntologyRelationType};
use excel_skill::ops::foundation::repository_metadata_audit::{
    RepositoryEvidenceHygieneDiagnostic, RepositoryMetadataAudit,
    RepositoryMetadataAuditExportDtoV1, RepositoryWeakLocatorReason, RepositoryWeakSourceRefReason,
};

// 2026-04-10 CST: 这里先补 repository-level audit 红测，原因是当前 foundation 只有节点级 validator，
// 还没有把问题提升成仓库级报告。目的：先钉住“聚合 issue + hygiene diagnostics”的正式契约，再做最小实现。
#[test]
fn repository_metadata_audit_aggregates_validator_issues_and_hygiene_diagnostics() {
    let schema = sample_schema();
    let repository = sample_repository();

    let report = RepositoryMetadataAudit::new(&schema).audit(&repository);

    assert_eq!(report.total_nodes, 15);
    assert_eq!(report.audited_nodes, 15);
    assert_eq!(report.issue_count, 4);

    assert_eq!(
        report.issue_type_counts.get("MissingRequiredField"),
        Some(&1usize)
    );
    assert_eq!(
        report.issue_type_counts.get("DisallowedField"),
        Some(&1usize)
    );
    assert_eq!(
        report.issue_type_counts.get("AliasFieldUsed"),
        Some(&1usize)
    );
    assert_eq!(
        report.issue_type_counts.get("DeprecatedFieldUsed"),
        Some(&1usize)
    );

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
            RepositoryEvidenceHygieneDiagnostic::WeakLocator {
                node_id: "node-revenue-sheet-only-locator".to_string(),
                source_ref: "sheet:sales".to_string(),
                locator: "Sheet1".to_string(),
                reason: RepositoryWeakLocatorReason::SheetOnly,
            },
            RepositoryEvidenceHygieneDiagnostic::WeakLocator {
                node_id: "node-revenue-single-cell-locator".to_string(),
                source_ref: "sheet:sales".to_string(),
                locator: "B20".to_string(),
                reason: RepositoryWeakLocatorReason::SingleCellOnly,
            },
            RepositoryEvidenceHygieneDiagnostic::WeakLocator {
                node_id: "node-revenue-ambiguous-locator".to_string(),
                source_ref: "sheet:ops".to_string(),
                locator: "row 3".to_string(),
                reason: RepositoryWeakLocatorReason::AmbiguousKeyword,
            },
            RepositoryEvidenceHygieneDiagnostic::WeakLocator {
                node_id: "node-revenue-invalid-range-locator".to_string(),
                source_ref: "sheet:ops".to_string(),
                locator: "A1-B10".to_string(),
                reason: RepositoryWeakLocatorReason::InvalidRangeFormat,
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
            RepositoryEvidenceHygieneDiagnostic::WeakSourceRef {
                node_id: "node-revenue-missing-entity-source".to_string(),
                source_ref: "sheet:".to_string(),
                locator: "D10:D20".to_string(),
                reason: RepositoryWeakSourceRefReason::EntityMissing,
            },
            RepositoryEvidenceHygieneDiagnostic::WeakSourceRef {
                node_id: "node-revenue-whitespace-source".to_string(),
                source_ref: "sheet: sales q1".to_string(),
                locator: "E10:E20".to_string(),
                reason: RepositoryWeakSourceRefReason::ContainsWhitespace,
            },
            RepositoryEvidenceHygieneDiagnostic::WeakSourceRef {
                node_id: "node-revenue-invalid-char-source".to_string(),
                source_ref: "sheet:sales?2024".to_string(),
                locator: "F10:F20".to_string(),
                reason: RepositoryWeakSourceRefReason::InvalidCharacter,
            },
            RepositoryEvidenceHygieneDiagnostic::WeakSourceRef {
                node_id: "node-revenue-unknown-namespace-source".to_string(),
                source_ref: "blob:sales".to_string(),
                locator: "G10:G20".to_string(),
                reason: RepositoryWeakSourceRefReason::UnknownNamespace,
            },
        ]
    );

    assert_eq!(report.hygiene_summary.total_diagnostics, 16);
    assert_eq!(
        report.hygiene_summary.severity_counts.get("Critical"),
        Some(&5usize)
    );
    assert_eq!(
        report.hygiene_summary.severity_counts.get("Warning"),
        Some(&11usize)
    );
    assert_eq!(
        report
            .hygiene_summary
            .diagnostic_type_counts
            .get("MissingEvidenceRef"),
        Some(&1usize)
    );
    assert_eq!(
        report
            .hygiene_summary
            .diagnostic_type_counts
            .get("DuplicateEvidenceRefWithinNode"),
        Some(&1usize)
    );
    assert_eq!(
        report
            .hygiene_summary
            .diagnostic_type_counts
            .get("DuplicateEvidenceRef"),
        Some(&1usize)
    );
    assert_eq!(
        report
            .hygiene_summary
            .diagnostic_type_counts
            .get("WeakLocator"),
        Some(&6usize)
    );
    assert_eq!(
        report
            .hygiene_summary
            .diagnostic_type_counts
            .get("WeakSourceRef"),
        Some(&7usize)
    );
    assert_eq!(
        report
            .hygiene_summary
            .weak_locator_reason_counts
            .get("Blank"),
        Some(&1usize)
    );
    assert_eq!(
        report
            .hygiene_summary
            .weak_locator_reason_counts
            .get("TooShort"),
        Some(&1usize)
    );
    assert_eq!(
        report
            .hygiene_summary
            .weak_locator_reason_counts
            .get("SheetOnly"),
        Some(&1usize)
    );
    assert_eq!(
        report
            .hygiene_summary
            .weak_locator_reason_counts
            .get("SingleCellOnly"),
        Some(&1usize)
    );
    assert_eq!(
        report
            .hygiene_summary
            .weak_locator_reason_counts
            .get("AmbiguousKeyword"),
        Some(&1usize)
    );
    assert_eq!(
        report
            .hygiene_summary
            .weak_locator_reason_counts
            .get("InvalidRangeFormat"),
        Some(&1usize)
    );
    assert_eq!(
        report
            .hygiene_summary
            .weak_source_ref_reason_counts
            .get("Blank"),
        Some(&1usize)
    );
    assert_eq!(
        report
            .hygiene_summary
            .weak_source_ref_reason_counts
            .get("TooShort"),
        Some(&1usize)
    );
    assert_eq!(
        report
            .hygiene_summary
            .weak_source_ref_reason_counts
            .get("MissingNamespace"),
        Some(&1usize)
    );
    assert_eq!(
        report
            .hygiene_summary
            .weak_source_ref_reason_counts
            .get("EntityMissing"),
        Some(&1usize)
    );
    assert_eq!(
        report
            .hygiene_summary
            .weak_source_ref_reason_counts
            .get("ContainsWhitespace"),
        Some(&1usize)
    );
    assert_eq!(
        report
            .hygiene_summary
            .weak_source_ref_reason_counts
            .get("InvalidCharacter"),
        Some(&1usize)
    );
    assert_eq!(
        report
            .hygiene_summary
            .weak_source_ref_reason_counts
            .get("UnknownNamespace"),
        Some(&1usize)
    );
    assert_eq!(report.hygiene_summary.affected_node_count, 15);
    assert!(report.hygiene_summary.has_blocking_hygiene_issue);

    assert_eq!(report.hygiene_views.by_severity.len(), 3);
    assert_eq!(report.hygiene_views.by_severity[0].diagnostic_count, 5);
    assert_eq!(report.hygiene_views.by_severity[0].affected_node_count, 5);
    assert_eq!(
        report.hygiene_views.by_severity[0].node_ids,
        vec![
            "node-revenue-governance".to_string(),
            "node-revenue-missing".to_string(),
            "node-revenue-missing-entity-source".to_string(),
            "node-revenue-missing-namespace".to_string(),
            "node-revenue-short-source".to_string(),
        ]
    );
    assert_eq!(report.hygiene_views.by_severity[1].diagnostic_count, 11);
    assert_eq!(report.hygiene_views.by_severity[1].affected_node_count, 11);
    assert_eq!(report.hygiene_views.by_severity[2].diagnostic_count, 0);
    assert_eq!(report.hygiene_views.by_severity[2].affected_node_count, 0);

    assert_eq!(
        report.hygiene_views.by_node[0].node_id,
        "node-revenue-governance".to_string()
    );
    assert_eq!(report.hygiene_views.by_node[0].diagnostic_count, 2);
    assert_eq!(
        report.hygiene_views.by_node[0]
            .diagnostic_type_counts
            .get("WeakSourceRef"),
        Some(&1usize)
    );
    assert_eq!(
        report.hygiene_views.by_node[0]
            .diagnostic_type_counts
            .get("DuplicateEvidenceRef"),
        Some(&1usize)
    );

    assert_eq!(
        report.hygiene_views.by_node[1].node_id,
        "node-revenue-missing".to_string()
    );
    assert_eq!(report.hygiene_views.by_node[1].diagnostic_count, 1);
    assert_eq!(
        report.hygiene_views.by_node[2].node_id,
        "node-revenue-missing-entity-source".to_string()
    );
    assert_eq!(
        report.hygiene_views.by_node[3].node_id,
        "node-revenue-missing-namespace".to_string()
    );
    assert_eq!(
        report.hygiene_views.by_node[4].node_id,
        "node-revenue-short-source".to_string()
    );
    assert_eq!(
        report.hygiene_views.by_node[5].node_id,
        "node-revenue-owner".to_string()
    );
    assert_eq!(report.hygiene_views.by_node[5].diagnostic_count, 2);

    assert_eq!(report.hygiene_reason_views.weak_locator_by_reason.len(), 6);
    assert_eq!(
        report.hygiene_reason_views.weak_locator_by_reason[0].diagnostic_count,
        1
    );
    assert_eq!(
        report.hygiene_reason_views.weak_locator_by_reason[0].affected_node_count,
        1
    );
    assert_eq!(
        report.hygiene_reason_views.weak_locator_by_reason[0].node_ids,
        vec!["node-revenue-ambiguous-locator".to_string()]
    );
    assert_eq!(
        report.hygiene_reason_views.weak_locator_by_reason[1].node_ids,
        vec!["node-revenue-blank-locator".to_string()]
    );
    assert_eq!(
        report.hygiene_reason_views.weak_locator_by_reason[2].node_ids,
        vec!["node-revenue-invalid-range-locator".to_string()]
    );
    assert_eq!(
        report.hygiene_reason_views.weak_locator_by_reason[3].node_ids,
        vec!["node-revenue-sheet-only-locator".to_string()]
    );
    assert_eq!(
        report.hygiene_reason_views.weak_locator_by_reason[4].node_ids,
        vec!["node-revenue-single-cell-locator".to_string()]
    );
    assert_eq!(
        report.hygiene_reason_views.weak_locator_by_reason[5].node_ids,
        vec!["node-revenue-short-locator".to_string()]
    );

    assert_eq!(
        report.hygiene_reason_views.weak_source_ref_by_reason.len(),
        7
    );
    assert_eq!(
        report.hygiene_reason_views.weak_source_ref_by_reason[0].severity,
        excel_skill::ops::foundation::repository_metadata_audit::RepositoryHygieneSeverity::Critical
    );
    assert_eq!(
        report.hygiene_reason_views.weak_source_ref_by_reason[0].node_ids,
        vec!["node-revenue-governance".to_string()]
    );
    assert_eq!(
        report.hygiene_reason_views.weak_source_ref_by_reason[1].node_ids,
        vec!["node-revenue-missing-entity-source".to_string()]
    );
    assert_eq!(
        report.hygiene_reason_views.weak_source_ref_by_reason[2].node_ids,
        vec!["node-revenue-missing-namespace".to_string()]
    );
    assert_eq!(
        report.hygiene_reason_views.weak_source_ref_by_reason[3].node_ids,
        vec!["node-revenue-short-source".to_string()]
    );
    assert_eq!(
        report.hygiene_reason_views.weak_source_ref_by_reason[4].severity,
        excel_skill::ops::foundation::repository_metadata_audit::RepositoryHygieneSeverity::Warning
    );
    assert_eq!(
        report.hygiene_reason_views.weak_source_ref_by_reason[4].node_ids,
        vec!["node-revenue-whitespace-source".to_string()]
    );
    assert_eq!(
        report.hygiene_reason_views.weak_source_ref_by_reason[5].node_ids,
        vec!["node-revenue-invalid-char-source".to_string()]
    );
    assert_eq!(
        report.hygiene_reason_views.weak_source_ref_by_reason[6].node_ids,
        vec!["node-revenue-unknown-namespace-source".to_string()]
    );
}

// 2026-04-12 CST: Added a large-sample stability regression because the
// foundation roaming audit now has four governance layers and we need to lock
// ordering/count behavior before continuing to extend the model.
#[test]
fn repository_metadata_audit_keeps_stable_grouping_on_large_repository_fixture() {
    let schema = sample_schema();
    let repository = large_stability_repository();

    let report = RepositoryMetadataAudit::new(&schema).audit(&repository);

    assert_eq!(report.total_nodes, 16);
    assert_eq!(report.hygiene_summary.total_diagnostics, 20);
    assert_eq!(
        report.hygiene_summary.severity_counts.get("Critical"),
        Some(&4usize)
    );
    assert_eq!(
        report.hygiene_summary.severity_counts.get("Warning"),
        Some(&16usize)
    );
    assert_eq!(report.hygiene_summary.affected_node_count, 16);

    assert_eq!(report.hygiene_views.by_severity[0].diagnostic_count, 4);
    assert_eq!(report.hygiene_views.by_severity[0].affected_node_count, 4);
    assert_eq!(report.hygiene_views.by_severity[1].diagnostic_count, 16);
    assert_eq!(report.hygiene_views.by_severity[1].affected_node_count, 13);

    assert_eq!(
        report.hygiene_views.by_node[0].node_id,
        "node-revenue-critical-combo".to_string()
    );
    assert_eq!(report.hygiene_views.by_node[0].diagnostic_count, 3);
    assert_eq!(
        report.hygiene_views.by_node[1].node_id,
        "node-revenue-blank-source-a".to_string()
    );
    assert_eq!(report.hygiene_views.by_node[1].diagnostic_count, 1);

    assert_eq!(report.hygiene_reason_views.weak_locator_by_reason.len(), 2);
    assert_eq!(
        report.hygiene_reason_views.weak_locator_by_reason[0].node_ids,
        vec![
            "node-revenue-ambiguous-a".to_string(),
            "node-revenue-ambiguous-b".to_string(),
            "node-revenue-ambiguous-c".to_string(),
        ]
    );
    assert_eq!(
        report.hygiene_reason_views.weak_locator_by_reason[1].node_ids,
        vec![
            "node-revenue-blank-a".to_string(),
            "node-revenue-blank-b".to_string(),
        ]
    );

    assert_eq!(
        report.hygiene_reason_views.weak_source_ref_by_reason.len(),
        4
    );
    assert_eq!(
        report.hygiene_reason_views.weak_source_ref_by_reason[0].node_ids,
        vec![
            "node-revenue-blank-source-a".to_string(),
            "node-revenue-critical-combo".to_string(),
        ]
    );
    assert_eq!(
        report.hygiene_reason_views.weak_source_ref_by_reason[1].node_ids,
        vec![
            "node-revenue-critical-combo-2".to_string(),
            "node-revenue-whitespace-source-a".to_string(),
            "node-revenue-whitespace-source-b".to_string(),
        ]
    );
}

// 2026-04-12 CST: Added a boundary-semantics regression because reason groups
// currently expose both diagnostic_count and affected_node_count, so we need to
// lock how repeated same-reason evidence behaves on one node.
#[test]
fn repository_metadata_audit_reason_views_keep_node_dedup_semantics_for_repeated_reasons() {
    let schema = sample_schema();
    let repository = repeated_reason_boundary_repository();

    let report = RepositoryMetadataAudit::new(&schema).audit(&repository);

    assert_eq!(report.hygiene_summary.total_diagnostics, 9);
    assert_eq!(
        report
            .hygiene_summary
            .weak_locator_reason_counts
            .get("Blank"),
        Some(&2usize)
    );
    assert_eq!(
        report
            .hygiene_summary
            .weak_source_ref_reason_counts
            .get("ContainsWhitespace"),
        Some(&3usize)
    );
    assert_eq!(
        report
            .hygiene_summary
            .weak_source_ref_reason_counts
            .get("Blank"),
        Some(&3usize)
    );

    assert_eq!(report.hygiene_reason_views.weak_locator_by_reason.len(), 1);
    assert_eq!(
        report.hygiene_reason_views.weak_locator_by_reason[0].diagnostic_count,
        1
    );
    assert_eq!(
        report.hygiene_reason_views.weak_locator_by_reason[0].affected_node_count,
        1
    );
    assert_eq!(
        report.hygiene_reason_views.weak_locator_by_reason[0].node_ids,
        vec!["node-revenue-repeat-locator".to_string()]
    );

    assert_eq!(
        report.hygiene_reason_views.weak_source_ref_by_reason.len(),
        2
    );
    assert_eq!(
        report.hygiene_reason_views.weak_source_ref_by_reason[0].diagnostic_count,
        1
    );
    assert_eq!(
        report.hygiene_reason_views.weak_source_ref_by_reason[0].affected_node_count,
        1
    );
    assert_eq!(
        report.hygiene_reason_views.weak_source_ref_by_reason[0].node_ids,
        vec!["node-revenue-repeat-critical-source".to_string()]
    );
    assert_eq!(
        report.hygiene_reason_views.weak_source_ref_by_reason[1].diagnostic_count,
        1
    );
    assert_eq!(
        report.hygiene_reason_views.weak_source_ref_by_reason[1].affected_node_count,
        1
    );
    assert_eq!(
        report.hygiene_reason_views.weak_source_ref_by_reason[1].node_ids,
        vec!["node-revenue-repeat-warning-source".to_string()]
    );
}

// 2026-04-12 CST: Added DTO export red test because downstream AI and upper
// layers now need one stable contract instead of reading the internal report
// shape directly. Purpose: lock the first v1 export surface before implementation.
#[test]
fn dto_export_v1_mirrors_foundation_hygiene_contract() {
    let schema = sample_schema();
    let repository = sample_repository();

    let report = RepositoryMetadataAudit::new(&schema).audit(&repository);
    let dto = RepositoryMetadataAuditExportDtoV1::from_report(&report);

    assert_eq!(dto.total_nodes, 15);
    assert_eq!(dto.audited_nodes, 15);
    assert_eq!(dto.issue_count, 4);
    assert_eq!(dto.hygiene_summary.total_diagnostics, 16);
    assert_eq!(
        dto.hygiene_summary.severity_counts.get("Critical"),
        Some(&5usize)
    );
    assert_eq!(
        dto.hygiene_summary.severity_counts.get("Warning"),
        Some(&11usize)
    );
    assert_eq!(
        dto.hygiene_views.by_severity[0].severity.as_str(),
        "Critical"
    );
    assert_eq!(dto.hygiene_views.by_severity[0].diagnostic_count, 5);
    assert_eq!(
        dto.hygiene_views.by_node[0].node_id,
        "node-revenue-governance".to_string()
    );
    assert_eq!(
        dto.hygiene_reason_views.weak_source_ref_by_reason[0]
            .reason
            .as_str(),
        "Blank"
    );
    assert_eq!(
        dto.hygiene_reason_views.weak_source_ref_by_reason[0]
            .severity
            .as_str(),
        "Critical"
    );
    assert_eq!(
        dto.hygiene_reason_views.weak_source_ref_by_reason[4]
            .reason
            .as_str(),
        "ContainsWhitespace"
    );
    assert_eq!(
        dto.hygiene_reason_views.weak_source_ref_by_reason[4]
            .severity
            .as_str(),
        "Warning"
    );
}

// 2026-04-12 CST: Added DTO export stability coverage because the new contract
// must preserve already-locked ordering semantics on large repository fixtures.
// Purpose: prevent future callers from seeing DTO ordering drift even if the
// internal report evolves.
#[test]
fn dto_export_v1_keeps_large_repository_ordering_stable() {
    let schema = sample_schema();
    let repository = large_stability_repository();

    let report = RepositoryMetadataAudit::new(&schema).audit(&repository);
    let dto = RepositoryMetadataAuditExportDtoV1::from_report(&report);

    assert_eq!(dto.hygiene_summary.total_diagnostics, 20);
    assert_eq!(
        dto.hygiene_views.by_severity[0].severity.as_str(),
        "Critical"
    );
    assert_eq!(dto.hygiene_views.by_severity[0].diagnostic_count, 4);
    assert_eq!(
        dto.hygiene_views.by_node[0].node_id,
        "node-revenue-critical-combo".to_string()
    );
    assert_eq!(
        dto.hygiene_reason_views.weak_locator_by_reason[0].node_ids,
        vec![
            "node-revenue-ambiguous-a".to_string(),
            "node-revenue-ambiguous-b".to_string(),
            "node-revenue-ambiguous-c".to_string(),
        ]
    );
    assert_eq!(
        dto.hygiene_reason_views.weak_source_ref_by_reason[1].node_ids,
        vec![
            "node-revenue-critical-combo-2".to_string(),
            "node-revenue-whitespace-source-a".to_string(),
            "node-revenue-whitespace-source-b".to_string(),
        ]
    );
}

// 2026-04-12 CST: Added DTO boundary red test because export must preserve the
// current repeated-reason counting contract instead of inventing a new semantic
// at the contract layer. Purpose: keep DTO and report semantics aligned.
#[test]
fn dto_export_v1_preserves_reason_count_semantics_on_repeated_reasons() {
    let schema = sample_schema();
    let repository = repeated_reason_boundary_repository();

    let report = RepositoryMetadataAudit::new(&schema).audit(&repository);
    let dto = RepositoryMetadataAuditExportDtoV1::from_report(&report);

    assert_eq!(dto.hygiene_summary.total_diagnostics, 9);
    assert_eq!(
        dto.hygiene_summary
            .weak_source_ref_reason_counts
            .get("ContainsWhitespace"),
        Some(&3usize)
    );
    assert_eq!(
        dto.hygiene_reason_views.weak_locator_by_reason[0].diagnostic_count,
        1
    );
    assert_eq!(
        dto.hygiene_reason_views.weak_locator_by_reason[0].affected_node_count,
        1
    );
    assert_eq!(
        dto.hygiene_reason_views.weak_source_ref_by_reason[0].diagnostic_count,
        1
    );
    assert_eq!(
        dto.hygiene_reason_views.weak_source_ref_by_reason[1].diagnostic_count,
        1
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
            KnowledgeNode::new("node-revenue-missing", "Revenue Missing Domain", "Body")
                .with_concept_id("revenue"),
            KnowledgeNode::new("node-revenue-owner", "Revenue Owner Field", "Body")
                .with_concept_id("revenue")
                .with_metadata_entry("domain", "finance")
                .with_metadata_entry("owner", "ops-team")
                .with_evidence_ref(EvidenceRef::new("sheet:sales", "A1:B12"))
                .with_evidence_ref(EvidenceRef::new("sheet:sales", "A1:B12")),
            KnowledgeNode::new("node-revenue-governance", "Revenue Governance", "Body")
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
            KnowledgeNode::new("node-revenue-short-source", "Revenue Short Source", "Body")
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
            KnowledgeNode::new(
                "node-revenue-sheet-only-locator",
                "Revenue Sheet Only Locator",
                "Body",
            )
            .with_concept_id("revenue")
            .with_metadata_entry("domain", "finance")
            .with_evidence_ref(EvidenceRef::new("sheet:sales", "Sheet1")),
            KnowledgeNode::new(
                "node-revenue-single-cell-locator",
                "Revenue Single Cell Locator",
                "Body",
            )
            .with_concept_id("revenue")
            .with_metadata_entry("domain", "finance")
            .with_evidence_ref(EvidenceRef::new("sheet:sales", "B20")),
            KnowledgeNode::new(
                "node-revenue-ambiguous-locator",
                "Revenue Ambiguous Locator",
                "Body",
            )
            .with_concept_id("revenue")
            .with_metadata_entry("domain", "finance")
            .with_evidence_ref(EvidenceRef::new("sheet:ops", "row 3")),
            KnowledgeNode::new(
                "node-revenue-invalid-range-locator",
                "Revenue Invalid Range Locator",
                "Body",
            )
            .with_concept_id("revenue")
            .with_metadata_entry("domain", "finance")
            .with_evidence_ref(EvidenceRef::new("sheet:ops", "A1-B10")),
            KnowledgeNode::new(
                "node-revenue-missing-entity-source",
                "Revenue Missing Entity Source",
                "Body",
            )
            .with_concept_id("revenue")
            .with_metadata_entry("domain", "finance")
            .with_evidence_ref(EvidenceRef::new("sheet:", "D10:D20")),
            KnowledgeNode::new(
                "node-revenue-whitespace-source",
                "Revenue Whitespace Source",
                "Body",
            )
            .with_concept_id("revenue")
            .with_metadata_entry("domain", "finance")
            .with_evidence_ref(EvidenceRef::new("sheet: sales q1", "E10:E20")),
            KnowledgeNode::new(
                "node-revenue-invalid-char-source",
                "Revenue Invalid Character Source",
                "Body",
            )
            .with_concept_id("revenue")
            .with_metadata_entry("domain", "finance")
            .with_evidence_ref(EvidenceRef::new("sheet:sales?2024", "F10:F20")),
            KnowledgeNode::new(
                "node-revenue-unknown-namespace-source",
                "Revenue Unknown Namespace Source",
                "Body",
            )
            .with_concept_id("revenue")
            .with_metadata_entry("domain", "finance")
            .with_evidence_ref(EvidenceRef::new("blob:sales", "G10:G20")),
        ],
        vec![KnowledgeEdge::new(
            "node-revenue-owner",
            "node-revenue-governance",
            OntologyRelationType::References,
        )],
    ))
    .expect("sample repository should be valid")
}

// 2026-04-12 CST: Added a larger repository fixture so governance sorting and
// aggregation can be tested under more crowded foundation conditions without
// touching business-layer modules.
fn large_stability_repository() -> KnowledgeRepository {
    KnowledgeRepository::new(KnowledgeBundle::new(
        "foundation.large.v1",
        vec![OntologyConcept::new("revenue", "Revenue")],
        vec![],
        vec![
            KnowledgeNode::new("node-revenue-missing-a", "Missing A", "Body")
                .with_concept_id("revenue"),
            KnowledgeNode::new("node-revenue-missing-b", "Missing B", "Body")
                .with_concept_id("revenue"),
            KnowledgeNode::new("node-revenue-owner-a", "Owner A", "Body")
                .with_concept_id("revenue")
                .with_metadata_entry("domain", "finance")
                .with_metadata_entry("owner", "ops-team")
                .with_evidence_ref(EvidenceRef::new("sheet:sales", "A1:B12"))
                .with_evidence_ref(EvidenceRef::new("sheet:sales", "A1:B12")),
            KnowledgeNode::new("node-revenue-owner-b", "Owner B", "Body")
                .with_concept_id("revenue")
                .with_metadata_entry("domain", "finance")
                .with_metadata_entry("owner", "ops-team")
                .with_evidence_ref(EvidenceRef::new("sheet:sales", "A1:B12"))
                .with_evidence_ref(EvidenceRef::new("sheet:sales", "A1:B12")),
            KnowledgeNode::new("node-revenue-critical-combo", "Critical Combo", "Body")
                .with_concept_id("revenue")
                .with_metadata_entry("domain", "finance")
                .with_evidence_ref(EvidenceRef::new("", "A1:B12"))
                .with_evidence_ref(EvidenceRef::new("sheet:sales", "A1:B12"))
                .with_evidence_ref(EvidenceRef::new("sheet:sales", "A1:B12")),
            KnowledgeNode::new("node-revenue-critical-combo-2", "Critical Combo 2", "Body")
                .with_concept_id("revenue")
                .with_metadata_entry("domain", "finance")
                .with_evidence_ref(EvidenceRef::new("sheet: sales q1", "A1:B12"))
                .with_evidence_ref(EvidenceRef::new("sheet:sales", "A1:B12"))
                .with_evidence_ref(EvidenceRef::new("sheet:sales", "A1:B12")),
            KnowledgeNode::new("node-revenue-blank-a", "Blank A", "Body")
                .with_concept_id("revenue")
                .with_metadata_entry("domain", "finance")
                .with_evidence_ref(EvidenceRef::new("sheet:finance", "   ")),
            KnowledgeNode::new("node-revenue-blank-b", "Blank B", "Body")
                .with_concept_id("revenue")
                .with_metadata_entry("domain", "finance")
                .with_evidence_ref(EvidenceRef::new("sheet:finance", "   ")),
            KnowledgeNode::new("node-revenue-ambiguous-a", "Ambiguous A", "Body")
                .with_concept_id("revenue")
                .with_metadata_entry("domain", "finance")
                .with_evidence_ref(EvidenceRef::new("sheet:ops", "row 3")),
            KnowledgeNode::new("node-revenue-ambiguous-b", "Ambiguous B", "Body")
                .with_concept_id("revenue")
                .with_metadata_entry("domain", "finance")
                .with_evidence_ref(EvidenceRef::new("sheet:ops", "row 7")),
            KnowledgeNode::new("node-revenue-ambiguous-c", "Ambiguous C", "Body")
                .with_concept_id("revenue")
                .with_metadata_entry("domain", "finance")
                .with_evidence_ref(EvidenceRef::new("sheet:ops", "table summary")),
            KnowledgeNode::new("node-revenue-blank-source-a", "Blank Source A", "Body")
                .with_concept_id("revenue")
                .with_metadata_entry("domain", "finance")
                .with_evidence_ref(EvidenceRef::new("", "C1:C3")),
            KnowledgeNode::new(
                "node-revenue-whitespace-source-a",
                "Whitespace Source A",
                "Body",
            )
            .with_concept_id("revenue")
            .with_metadata_entry("domain", "finance")
            .with_evidence_ref(EvidenceRef::new("sheet: sales q1", "D1:D3")),
            KnowledgeNode::new(
                "node-revenue-whitespace-source-b",
                "Whitespace Source B",
                "Body",
            )
            .with_concept_id("revenue")
            .with_metadata_entry("domain", "finance")
            .with_evidence_ref(EvidenceRef::new("sheet: sales q2", "E1:E3")),
            KnowledgeNode::new(
                "node-revenue-invalid-char-source-a",
                "Invalid Source A",
                "Body",
            )
            .with_concept_id("revenue")
            .with_metadata_entry("domain", "finance")
            .with_evidence_ref(EvidenceRef::new("sheet:sales?2024", "F1:F3")),
            KnowledgeNode::new(
                "node-revenue-unknown-namespace-a",
                "Unknown Namespace A",
                "Body",
            )
            .with_concept_id("revenue")
            .with_metadata_entry("domain", "finance")
            .with_evidence_ref(EvidenceRef::new("blob:sales", "G1:G3")),
        ],
        vec![KnowledgeEdge::new(
            "node-revenue-owner-a",
            "node-revenue-critical-combo",
            OntologyRelationType::References,
        )],
    ))
    .expect("large stability repository should be valid")
}

// 2026-04-12 CST: Added a repeated-reason boundary fixture so the project can
// explicitly lock whether reason views count diagnostics or deduplicated nodes.
fn repeated_reason_boundary_repository() -> KnowledgeRepository {
    KnowledgeRepository::new(KnowledgeBundle::new(
        "foundation.boundary.v1",
        vec![OntologyConcept::new("revenue", "Revenue")],
        vec![],
        vec![
            KnowledgeNode::new(
                "node-revenue-repeat-locator",
                "Repeated Locator Reason",
                "Body",
            )
            .with_concept_id("revenue")
            .with_metadata_entry("domain", "finance")
            .with_evidence_ref(EvidenceRef::new("sheet:finance", "   "))
            .with_evidence_ref(EvidenceRef::new("sheet:finance", "   ")),
            KnowledgeNode::new(
                "node-revenue-repeat-warning-source",
                "Repeated Warning Source",
                "Body",
            )
            .with_concept_id("revenue")
            .with_metadata_entry("domain", "finance")
            .with_evidence_ref(EvidenceRef::new("sheet: sales q1", "A1:A3"))
            .with_evidence_ref(EvidenceRef::new("sheet: sales q2", "B1:B3"))
            .with_evidence_ref(EvidenceRef::new("sheet: sales q3", "C1:C3")),
            KnowledgeNode::new(
                "node-revenue-repeat-critical-source",
                "Repeated Critical Source",
                "Body",
            )
            .with_concept_id("revenue")
            .with_metadata_entry("domain", "finance")
            .with_evidence_ref(EvidenceRef::new("", "D1:D3"))
            .with_evidence_ref(EvidenceRef::new("", "E1:E3"))
            .with_evidence_ref(EvidenceRef::new("", "F1:F3")),
        ],
        vec![],
    ))
    .expect("repeated boundary repository should be valid")
}
