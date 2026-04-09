use excel_skill::ops::foundation::metadata_registry::{
    MetadataConstraintOperator, MetadataFieldGovernance, MetadataFieldGroup,
    MetadataRegistryAuditReport,
    MetadataFieldTarget, MetadataRegistry, MetadataRegistryAliasMapping,
    MetadataRegistryCanonicalFieldAggregationEntry,
    MetadataRegistryCompatibilityEntry, MetadataRegistryCompatibilitySummary,
    MetadataRegistryFieldResolution, MetadataRegistryFieldResolutionBatch,
    MetadataRegistryFieldResolutionBatchSummary,
    MetadataRegistryUnknownFieldEntry,
    MetadataRegistryValidationIssue, MetadataRegistryValidationIssueKind,
};

// 2026-04-09 CST: 这里先补 metadata registry 的首条红测，原因是方案B下一阶段要把 metadata 字段管理从
// “靠 concept/node 上实际有没有这个 key 猜”升级成“显式注册的标准能力”。
// 目的：先钉死字段目标层级和支持操作符都能被 registry 稳定表达，后续 resolver / retrieval 才能复用同一套目录能力。
#[test]
fn metadata_registry_tracks_field_targets_and_supported_operators() {
    let registry = MetadataRegistry::new()
        .register_text_field(
            "namespace",
            vec![MetadataFieldTarget::Concept],
            vec![
                MetadataConstraintOperator::Equals,
                MetadataConstraintOperator::In,
            ],
        )
        .register_text_list_field(
            "channels",
            vec![MetadataFieldTarget::Concept, MetadataFieldTarget::Node],
            vec![
                MetadataConstraintOperator::In,
                MetadataConstraintOperator::HasAny,
            ],
        );

    assert!(registry.supports_target("namespace", MetadataFieldTarget::Concept));
    assert!(!registry.supports_target("namespace", MetadataFieldTarget::Node));
    assert!(registry.supports_operator("namespace", MetadataConstraintOperator::Equals));
    assert!(!registry.supports_operator("namespace", MetadataConstraintOperator::HasAny));
    assert!(registry.supports_target("channels", MetadataFieldTarget::Node));
    assert!(registry.supports_operator("channels", MetadataConstraintOperator::HasAny));
}

// 2026-04-09 CST: 这里补字段治理属性读写红测，原因是方案B2的第一步不是再扩执行链路，
// 而是把 metadata 字段本体从“可注册”推进到“可治理、可解释”。
// 目的：先钉死 registry 能稳定保存字段分组、字段说明和作用原因，给后续本体元数据管理提供标准载体。
#[test]
fn metadata_registry_exposes_field_governance_metadata() {
    let registry = MetadataRegistry::new().register_text_field_with_governance(
        "source",
        vec![MetadataFieldTarget::Node],
        vec![MetadataConstraintOperator::Equals],
        MetadataFieldGovernance::new()
            .with_group(MetadataFieldGroup::Source)
            .with_description("标准来源标识，用于区分证据出处。")
            .with_applies_to_reason("该字段只存在于节点证据侧，不参与 concept 收敛。"),
    );

    let definition = registry
        .definition("source")
        .expect("field definition should exist");

    assert_eq!(definition.field, "source");
    assert_eq!(
        definition.governance.group,
        Some(MetadataFieldGroup::Source)
    );
    assert_eq!(
        definition.governance.description.as_deref(),
        Some("标准来源标识，用于区分证据出处。")
    );
    assert_eq!(
        definition.governance.applies_to_reason.as_deref(),
        Some("该字段只存在于节点证据侧，不参与 concept 收敛。")
    );
    assert_eq!(definition.governance.deprecated_reason, None);
}

// 2026-04-09 CST: 这里补按字段分组读取的红测，原因是如果 registry 只能按名字逐个查，
// 后续字段审计、手册导出和治理分层仍然会退回 ad-hoc 遍历。
// 目的：先固定“字段分组是 registry 的一等读取维度”这条标准能力。
#[test]
fn metadata_registry_lists_fields_by_governance_group() {
    let registry = MetadataRegistry::new()
        .register_text_field_with_governance(
            "source",
            vec![MetadataFieldTarget::Node],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Source)
                .with_description("来源")
                .with_applies_to_reason("节点来源字段。"),
        )
        .register_text_field_with_governance(
            "observed_at",
            vec![MetadataFieldTarget::Node],
            vec![MetadataConstraintOperator::Range],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Time)
                .with_description("观测时间")
                .with_applies_to_reason("节点时间窗口字段。"),
        )
        .register_text_field_with_governance(
            "namespace",
            vec![MetadataFieldTarget::Concept],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Classification)
                .with_description("命名空间")
                .with_applies_to_reason("concept 分类字段。"),
        );

    let grouped_fields = registry
        .fields_in_group(MetadataFieldGroup::Source)
        .into_iter()
        .map(|definition| definition.field.as_str())
        .collect::<Vec<_>>();

    assert_eq!(grouped_fields, vec!["source"]);
}

// 2026-04-09 CST: 这里补 registry 校验摘要红测，原因是方案B2不仅要保存治理属性，
// 还要能总结“哪些字段治理信息缺失、哪些字段已经废弃”，否则 registry 仍只是被动目录。
// 目的：把 metadata 字段治理质量输出成稳定摘要，后续 handoff 和审计才能直接消费。
#[test]
fn metadata_registry_validation_summary_reports_missing_governance_and_deprecated_fields() {
    let registry = MetadataRegistry::new()
        .register_text_field("namespace", vec![MetadataFieldTarget::Concept], vec![])
        .register_text_field_with_governance(
            "source_ref",
            vec![MetadataFieldTarget::Node],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Source)
                .with_description("标准来源引用字段。")
                .with_applies_to_reason("节点来源引用字段。"),
        )
        .register_text_field_with_governance(
            "source",
            vec![MetadataFieldTarget::Node],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Source)
                .with_description("来源")
                .with_applies_to_reason("节点来源字段。")
                .with_replaced_by("source_ref")
                .with_compatibility_note("过渡阶段保留兼容读取。")
                .deprecated("后续统一迁移到 source_ref。"),
        )
        .register_text_field_with_governance(
            "kind",
            vec![MetadataFieldTarget::Node],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Classification)
                .with_applies_to_reason("节点类别字段。"),
        )
        .register_text_field_with_governance(
            "owner",
            vec![],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Governance)
                .with_description("归属人")
                .with_applies_to_reason("治理责任归属字段。"),
        );

    let summary = registry.validate_governance();

    assert!(summary.has_issues());
    assert_eq!(
        summary.issues,
        vec![
            MetadataRegistryValidationIssue {
                field: "kind".to_string(),
                kind: MetadataRegistryValidationIssueKind::MissingDescription,
            },
            MetadataRegistryValidationIssue {
                field: "namespace".to_string(),
                kind: MetadataRegistryValidationIssueKind::MissingGroup,
            },
            MetadataRegistryValidationIssue {
                field: "namespace".to_string(),
                kind: MetadataRegistryValidationIssueKind::MissingDescription,
            },
            MetadataRegistryValidationIssue {
                field: "namespace".to_string(),
                kind: MetadataRegistryValidationIssueKind::MissingAppliesToReason,
            },
            MetadataRegistryValidationIssue {
                field: "namespace".to_string(),
                kind: MetadataRegistryValidationIssueKind::MissingOperators,
            },
            MetadataRegistryValidationIssue {
                field: "owner".to_string(),
                kind: MetadataRegistryValidationIssueKind::MissingTargets,
            },
            MetadataRegistryValidationIssue {
                field: "source".to_string(),
                kind: MetadataRegistryValidationIssueKind::Deprecated,
            },
        ]
    );
}

// 2026-04-09 CST: 这里补字段兼容摘要红测，原因是方案B-2不只要求把 alias / replaced_by 写进治理模型，
// 还要求 registry 能输出“单字段兼容视图 + 全量兼容映射”的标准摘要。
// 目的：先钉死字段别名、废弃状态和替代字段链路能够被稳定读取，而不是停留在静态属性上。
#[test]
fn metadata_registry_builds_compatibility_summary_and_replacement_chain() {
    let registry = MetadataRegistry::new()
        .register_text_field_with_governance(
            "source_ref",
            vec![MetadataFieldTarget::Node],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Source)
                .with_description("标准来源引用字段。")
                .with_applies_to_reason("节点来源引用字段。"),
        )
        .register_text_field_with_governance(
            "source",
            vec![MetadataFieldTarget::Node],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Source)
                .with_description("旧来源字段。")
                .with_applies_to_reason("节点来源字段。")
                .with_aliases(vec!["evidence_source"])
                .with_replaced_by("source_ref")
                .with_compatibility_note("过渡阶段保留兼容读取。")
                .deprecated("后续统一收敛到 source_ref。"),
        )
        .register_text_field_with_governance(
            "old_source",
            vec![MetadataFieldTarget::Node],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Source)
                .with_description("更早版本来源字段。")
                .with_applies_to_reason("历史节点来源字段。")
                .with_aliases(vec!["legacy_source"])
                .with_replaced_by("source")
                .with_compatibility_note("保留到历史数据迁移完成。")
                .deprecated("历史字段，逐步迁移中。"),
        );

    let compatibility = registry
        .compatibility_for("old_source")
        .expect("compatibility view should exist");
    let summary = registry.compatibility_summary();

    assert_eq!(
        compatibility,
        MetadataRegistryCompatibilityEntry {
            field: "old_source".to_string(),
            aliases: vec!["legacy_source".to_string()],
            deprecated_reason: Some("历史字段，逐步迁移中。".to_string()),
            replaced_by: Some("source".to_string()),
            replacement_chain: vec!["source".to_string(), "source_ref".to_string()],
            compatibility_note: Some("保留到历史数据迁移完成。".to_string()),
        }
    );
    assert_eq!(
        summary,
        MetadataRegistryCompatibilitySummary {
            fields: vec![
                MetadataRegistryCompatibilityEntry {
                    field: "old_source".to_string(),
                    aliases: vec!["legacy_source".to_string()],
                    deprecated_reason: Some("历史字段，逐步迁移中。".to_string()),
                    replaced_by: Some("source".to_string()),
                    replacement_chain: vec!["source".to_string(), "source_ref".to_string()],
                    compatibility_note: Some("保留到历史数据迁移完成。".to_string()),
                },
                MetadataRegistryCompatibilityEntry {
                    field: "source".to_string(),
                    aliases: vec!["evidence_source".to_string()],
                    deprecated_reason: Some("后续统一收敛到 source_ref。".to_string()),
                    replaced_by: Some("source_ref".to_string()),
                    replacement_chain: vec!["source_ref".to_string()],
                    compatibility_note: Some("过渡阶段保留兼容读取。".to_string()),
                },
                MetadataRegistryCompatibilityEntry {
                    field: "source_ref".to_string(),
                    aliases: Vec::new(),
                    deprecated_reason: None,
                    replaced_by: None,
                    replacement_chain: Vec::new(),
                    compatibility_note: None,
                },
            ],
            alias_mappings: vec![
                MetadataRegistryAliasMapping {
                    alias: "evidence_source".to_string(),
                    canonical_field: "source".to_string(),
                },
                MetadataRegistryAliasMapping {
                    alias: "legacy_source".to_string(),
                    canonical_field: "old_source".to_string(),
                },
            ],
        }
    );
}

// 2026-04-09 CST: 这里补兼容关系治理缺口红测，原因是方案B-2不仅要能描述兼容关系，
// 还要能把“废弃字段没有替代项/兼容说明”“替代字段指向不存在字段”这类问题稳定报出来。
// 目的：把兼容骨架正式纳入 registry 治理校验，而不是仅靠人工阅读字段定义发现问题。
#[test]
fn metadata_registry_validation_summary_reports_compatibility_gaps() {
    let registry = MetadataRegistry::new()
        .register_text_field_with_governance(
            "source_ref",
            vec![MetadataFieldTarget::Node],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Source)
                .with_description("标准来源引用字段。")
                .with_applies_to_reason("节点来源引用字段。"),
        )
        .register_text_field_with_governance(
            "legacy_source",
            vec![MetadataFieldTarget::Node],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Source)
                .with_description("历史来源字段。")
                .with_applies_to_reason("历史节点来源字段。")
                .deprecated("准备下线。"),
        )
        .register_text_field_with_governance(
            "source_alias",
            vec![MetadataFieldTarget::Node],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Source)
                .with_description("来源别名字段。")
                .with_applies_to_reason("兼容旧字段名。")
                .with_replaced_by("missing_target")
                .with_compatibility_note("等待统一迁移。"),
        );

    let summary = registry.validate_governance();

    assert_eq!(
        summary.issues,
        vec![
            MetadataRegistryValidationIssue {
                field: "legacy_source".to_string(),
                kind: MetadataRegistryValidationIssueKind::Deprecated,
            },
            MetadataRegistryValidationIssue {
                field: "legacy_source".to_string(),
                kind: MetadataRegistryValidationIssueKind::DeprecatedWithoutReplacement,
            },
            MetadataRegistryValidationIssue {
                field: "legacy_source".to_string(),
                kind: MetadataRegistryValidationIssueKind::DeprecatedWithoutCompatibilityNote,
            },
            MetadataRegistryValidationIssue {
                field: "source_alias".to_string(),
                kind: MetadataRegistryValidationIssueKind::ReplacedByMissingField,
            },
        ]
    );
}

// 2026-04-09 CST: 这里补单字段 canonical resolver 红测，原因是方案B3要求把前面沉淀的兼容信息转成可直接消费的目录级解析能力，
// 而不只是停留在 governance/compatibility summary 的静态视图。
// 目的：钉死字段名精确命中和 alias 命中时的结构化解析结果。
#[test]
fn metadata_registry_resolves_canonical_field_with_resolution_details() {
    let registry = MetadataRegistry::new()
        .register_text_field_with_governance(
            "source_ref",
            vec![MetadataFieldTarget::Node],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Source)
                .with_description("标准来源引用字段。")
                .with_applies_to_reason("节点来源引用字段。"),
        )
        .register_text_field_with_governance(
            "source",
            vec![MetadataFieldTarget::Node],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Source)
                .with_description("旧来源字段。")
                .with_applies_to_reason("节点来源字段。")
                .with_aliases(vec!["evidence_source"])
                .with_replaced_by("source_ref")
                .with_compatibility_note("过渡阶段保留兼容读取。")
                .deprecated("后续统一收敛到 source_ref。"),
        );

    assert_eq!(
        registry.resolve_field("source"),
        MetadataRegistryFieldResolution {
            input: "source".to_string(),
            canonical_field: Some("source".to_string()),
            matched_alias: None,
            deprecated_reason: Some("后续统一收敛到 source_ref。".to_string()),
            replacement_chain: vec!["source_ref".to_string()],
            compatibility_note: Some("过渡阶段保留兼容读取。".to_string()),
        }
    );
    assert_eq!(
        registry.resolve_field("evidence_source"),
        MetadataRegistryFieldResolution {
            input: "evidence_source".to_string(),
            canonical_field: Some("source".to_string()),
            matched_alias: Some("evidence_source".to_string()),
            deprecated_reason: Some("后续统一收敛到 source_ref。".to_string()),
            replacement_chain: vec!["source_ref".to_string()],
            compatibility_note: Some("过渡阶段保留兼容读取。".to_string()),
        }
    );
}

// 2026-04-09 CST: 这里补批量规范化红测，原因是方案B3比单字段 resolver 多一步，
// 要能把一组输入字段按原顺序转换成稳定的 canonical 解析结果。
// 目的：固定 batch normalization 是目录级能力，并且不会吞掉未识别字段。
#[test]
fn metadata_registry_normalizes_field_batch_in_input_order() {
    let registry = MetadataRegistry::new()
        .register_text_field_with_governance(
            "source_ref",
            vec![MetadataFieldTarget::Node],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Source)
                .with_description("标准来源引用字段。")
                .with_applies_to_reason("节点来源引用字段。"),
        )
        .register_text_field_with_governance(
            "source",
            vec![MetadataFieldTarget::Node],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Source)
                .with_description("旧来源字段。")
                .with_applies_to_reason("节点来源字段。")
                .with_aliases(vec!["evidence_source"])
                .with_replaced_by("source_ref")
                .with_compatibility_note("过渡阶段保留兼容读取。")
                .deprecated("后续统一收敛到 source_ref。"),
        )
        .register_text_field_with_governance(
            "namespace",
            vec![MetadataFieldTarget::Concept],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Classification)
                .with_description("命名空间字段。")
                .with_applies_to_reason("concept 分类字段。"),
        );

    assert_eq!(
        registry.normalize_fields(vec!["evidence_source", "namespace", "unknown_field"]),
        MetadataRegistryFieldResolutionBatch {
            entries: vec![
                MetadataRegistryFieldResolution {
                    input: "evidence_source".to_string(),
                    canonical_field: Some("source".to_string()),
                    matched_alias: Some("evidence_source".to_string()),
                    deprecated_reason: Some("后续统一收敛到 source_ref。".to_string()),
                    replacement_chain: vec!["source_ref".to_string()],
                    compatibility_note: Some("过渡阶段保留兼容读取。".to_string()),
                },
                MetadataRegistryFieldResolution {
                    input: "namespace".to_string(),
                    canonical_field: Some("namespace".to_string()),
                    matched_alias: None,
                    deprecated_reason: None,
                    replacement_chain: Vec::new(),
                    compatibility_note: None,
                },
                MetadataRegistryFieldResolution {
                    input: "unknown_field".to_string(),
                    canonical_field: None,
                    matched_alias: None,
                    deprecated_reason: None,
                    replacement_chain: Vec::new(),
                    compatibility_note: None,
                },
            ],
        }
    );
}

// 2026-04-09 CST: 这里补批量规范化摘要红测，原因是方案B1要求在不改变 normalize_fields(...) 现有明细输出的前提下，
// 再提供一层可直接消费的批量统计摘要，避免调用方重复手写计数逻辑。
// 目的：先钉死 batch summary 只做目录级统计，不扩展成未知字段清单或 canonical 聚合视图。
#[test]
fn metadata_registry_batch_normalization_summary_reports_batch_counts() {
    let registry = MetadataRegistry::new()
        .register_text_field_with_governance(
            "source_ref",
            vec![MetadataFieldTarget::Node],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Source)
                .with_description("标准来源引用字段。")
                .with_applies_to_reason("节点来源引用字段。"),
        )
        .register_text_field_with_governance(
            "source",
            vec![MetadataFieldTarget::Node],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Source)
                .with_description("旧来源字段。")
                .with_applies_to_reason("节点来源字段。")
                .with_aliases(vec!["evidence_source"])
                .with_replaced_by("source_ref")
                .with_compatibility_note("过渡阶段保留兼容读取。")
                .deprecated("后续统一收敛到 source_ref。"),
        )
        .register_text_field_with_governance(
            "namespace",
            vec![MetadataFieldTarget::Concept],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Classification)
                .with_description("命名空间字段。")
                .with_applies_to_reason("concept 分类字段。"),
        );

    let batch = registry.normalize_fields(vec![
        "source",
        "evidence_source",
        "namespace",
        "unknown_field",
    ]);

    assert_eq!(
        batch.summary(),
        MetadataRegistryFieldResolutionBatchSummary {
            total_inputs: 4,
            direct_hits: 2,
            alias_hits: 1,
            unresolved_hits: 1,
            deprecated_hits: 2,
        }
    );
}

// 2026-04-09 CST: 这里补未知字段清单视图红测，原因是方案B2要求在已有 batch summary 之外，
// 再给出可直接消费的未知字段明细视图，避免调用方自行二次过滤 unresolved entries。
// 目的：先钉死 unknown-field list 必须保留输入顺序和重复项，并显式给出输入位置，保持目录级可审计性。
#[test]
fn metadata_registry_batch_normalization_lists_unknown_fields_in_input_order() {
    let registry = MetadataRegistry::new()
        .register_text_field_with_governance(
            "source_ref",
            vec![MetadataFieldTarget::Node],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Source)
                .with_description("标准来源引用字段。")
                .with_applies_to_reason("节点来源引用字段。"),
        )
        .register_text_field_with_governance(
            "namespace",
            vec![MetadataFieldTarget::Concept],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Classification)
                .with_description("命名空间字段。")
                .with_applies_to_reason("concept 分类字段。"),
        );

    let batch = registry.normalize_fields(vec![
        "unknown_field",
        "namespace",
        "legacy_unknown",
        "unknown_field",
    ]);

    assert_eq!(
        batch.unknown_fields(),
        vec![
            MetadataRegistryUnknownFieldEntry {
                input: "unknown_field".to_string(),
                index: 0,
            },
            MetadataRegistryUnknownFieldEntry {
                input: "legacy_unknown".to_string(),
                index: 2,
            },
            MetadataRegistryUnknownFieldEntry {
                input: "unknown_field".to_string(),
                index: 3,
            },
        ]
    );
}

// 2026-04-09 CST: 这里补 alias 冲突治理红测，原因是方案A第一项要求 registry 能显式发现“同一个 alias 被多个字段复用”的目录冲突，
// 不能继续依赖调用方碰到歧义后再手工排查。
// 目的：先钉死 alias 冲突必须进入结构化治理摘要，而不是停留在隐式覆盖或不稳定解析行为里。
// 2026-04-09 CST: 这里补 canonical 聚合视图红测，原因是方案B3 要在 batch 明细之外，
// 再提供一层“按 canonical field 聚合”的目录工具视图，避免调用方重复手写汇总逻辑。
// 目的：钉死聚合结果只统计已解析字段、保留 canonical 首次出现顺序，并区分 direct/alias/deprecated 命中。
#[test]
fn metadata_registry_batch_normalization_groups_entries_by_canonical_field() {
    let registry = MetadataRegistry::new()
        .register_text_field_with_governance(
            "source_ref",
            vec![MetadataFieldTarget::Node],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Source)
                .with_description("标准来源引用字段。")
                .with_applies_to_reason("节点来源引用字段。"),
        )
        .register_text_field_with_governance(
            "source",
            vec![MetadataFieldTarget::Node],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Source)
                .with_description("旧来源字段。")
                .with_applies_to_reason("节点来源字段。")
                .with_aliases(vec!["evidence_source"])
                .with_replaced_by("source_ref")
                .with_compatibility_note("过渡阶段保留兼容读取。")
                .deprecated("后续统一收敛到 source_ref。"),
        )
        .register_text_field_with_governance(
            "namespace",
            vec![MetadataFieldTarget::Concept],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Classification)
                .with_description("命名空间字段。")
                .with_applies_to_reason("concept 分类字段。"),
        );

    let batch = registry.normalize_fields(vec![
        "unknown_field",
        "source",
        "evidence_source",
        "namespace",
        "source",
        "legacy_unknown",
    ]);

    assert_eq!(
        batch.canonical_aggregations(),
        vec![
            MetadataRegistryCanonicalFieldAggregationEntry {
                canonical_field: "source".to_string(),
                inputs: vec![
                    "source".to_string(),
                    "evidence_source".to_string(),
                    "source".to_string(),
                ],
                direct_hits: 2,
                alias_hits: 1,
                deprecated_hits: 3,
            },
            MetadataRegistryCanonicalFieldAggregationEntry {
                canonical_field: "namespace".to_string(),
                inputs: vec!["namespace".to_string()],
                direct_hits: 1,
                alias_hits: 0,
                deprecated_hits: 0,
            },
        ]
    );
}

// 2026-04-09 CST: 这里补统一 registry audit/export 视图红测，原因是方案A要把治理摘要、
// 兼容摘要和指定输入批次的目录工具输出收敛成一个标准对象，避免调用方再手动拼装多份结果。
// 目的：钉死统一审计结果必须同时携带 governance、compatibility、batch summary、unknown-field list
// 和 canonical aggregation，且这些内容都继续保持目录治理语义，不进入主链执行。
#[test]
fn metadata_registry_audit_report_combines_registry_and_batch_views() {
    let registry = MetadataRegistry::new()
        .register_text_field_with_governance(
            "source_ref",
            vec![MetadataFieldTarget::Node],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Source)
                .with_description("标准来源引用字段。")
                .with_applies_to_reason("节点来源引用字段。"),
        )
        .register_text_field_with_governance(
            "source",
            vec![MetadataFieldTarget::Node],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Source)
                .with_description("旧来源字段。")
                .with_applies_to_reason("节点来源字段。")
                .with_aliases(vec!["evidence_source"])
                .with_replaced_by("source_ref")
                .with_compatibility_note("过渡阶段保留兼容读取。")
                .deprecated("后续统一收敛到 source_ref。"),
        )
        .register_text_field_with_governance(
            "namespace",
            vec![MetadataFieldTarget::Concept],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Classification)
                .with_description("命名空间字段。")
                .with_applies_to_reason("concept 分类字段。"),
        );

    assert_eq!(
        registry.audit_fields(vec!["source", "evidence_source", "namespace", "unknown_field"]),
        MetadataRegistryAuditReport {
            governance: excel_skill::ops::foundation::metadata_registry::MetadataRegistryValidationSummary {
                issues: vec![MetadataRegistryValidationIssue {
                    field: "source".to_string(),
                    kind: MetadataRegistryValidationIssueKind::Deprecated,
                }],
            },
            compatibility: MetadataRegistryCompatibilitySummary {
                fields: vec![
                    MetadataRegistryCompatibilityEntry {
                        field: "namespace".to_string(),
                        aliases: Vec::new(),
                        deprecated_reason: None,
                        replaced_by: None,
                        replacement_chain: Vec::new(),
                        compatibility_note: None,
                    },
                    MetadataRegistryCompatibilityEntry {
                        field: "source".to_string(),
                        aliases: vec!["evidence_source".to_string()],
                        deprecated_reason: Some("后续统一收敛到 source_ref。".to_string()),
                        replaced_by: Some("source_ref".to_string()),
                        replacement_chain: vec!["source_ref".to_string()],
                        compatibility_note: Some("过渡阶段保留兼容读取。".to_string()),
                    },
                    MetadataRegistryCompatibilityEntry {
                        field: "source_ref".to_string(),
                        aliases: Vec::new(),
                        deprecated_reason: None,
                        replaced_by: None,
                        replacement_chain: Vec::new(),
                        compatibility_note: None,
                    },
                ],
                alias_mappings: vec![MetadataRegistryAliasMapping {
                    alias: "evidence_source".to_string(),
                    canonical_field: "source".to_string(),
                }],
            },
            field_resolution_batch: MetadataRegistryFieldResolutionBatch {
                entries: vec![
                    MetadataRegistryFieldResolution {
                        input: "source".to_string(),
                        canonical_field: Some("source".to_string()),
                        matched_alias: None,
                        deprecated_reason: Some("后续统一收敛到 source_ref。".to_string()),
                        replacement_chain: vec!["source_ref".to_string()],
                        compatibility_note: Some("过渡阶段保留兼容读取。".to_string()),
                    },
                    MetadataRegistryFieldResolution {
                        input: "evidence_source".to_string(),
                        canonical_field: Some("source".to_string()),
                        matched_alias: Some("evidence_source".to_string()),
                        deprecated_reason: Some("后续统一收敛到 source_ref。".to_string()),
                        replacement_chain: vec!["source_ref".to_string()],
                        compatibility_note: Some("过渡阶段保留兼容读取。".to_string()),
                    },
                    MetadataRegistryFieldResolution {
                        input: "namespace".to_string(),
                        canonical_field: Some("namespace".to_string()),
                        matched_alias: None,
                        deprecated_reason: None,
                        replacement_chain: Vec::new(),
                        compatibility_note: None,
                    },
                    MetadataRegistryFieldResolution {
                        input: "unknown_field".to_string(),
                        canonical_field: None,
                        matched_alias: None,
                        deprecated_reason: None,
                        replacement_chain: Vec::new(),
                        compatibility_note: None,
                    },
                ],
            },
            batch_summary: MetadataRegistryFieldResolutionBatchSummary {
                total_inputs: 4,
                direct_hits: 2,
                alias_hits: 1,
                unresolved_hits: 1,
                deprecated_hits: 2,
            },
            unknown_fields: vec![MetadataRegistryUnknownFieldEntry {
                input: "unknown_field".to_string(),
                index: 3,
            }],
            canonical_aggregations: vec![
                MetadataRegistryCanonicalFieldAggregationEntry {
                    canonical_field: "source".to_string(),
                    inputs: vec!["source".to_string(), "evidence_source".to_string()],
                    direct_hits: 1,
                    alias_hits: 1,
                    deprecated_hits: 2,
                },
                MetadataRegistryCanonicalFieldAggregationEntry {
                    canonical_field: "namespace".to_string(),
                    inputs: vec!["namespace".to_string()],
                    direct_hits: 1,
                    alias_hits: 0,
                    deprecated_hits: 0,
                },
            ],
        }
    );
}

#[test]
fn metadata_registry_validation_summary_reports_alias_conflicts() {
    let registry = MetadataRegistry::new()
        .register_text_field_with_governance(
            "source_ref",
            vec![MetadataFieldTarget::Node],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Source)
                .with_description("标准来源字段。")
                .with_applies_to_reason("节点来源字段。"),
        )
        .register_text_field_with_governance(
            "legacy_source",
            vec![MetadataFieldTarget::Node],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Source)
                .with_description("历史来源字段。")
                .with_applies_to_reason("兼容旧节点来源字段。")
                .with_aliases(vec!["shared_source_alias"])
                .with_replaced_by("source_ref")
                .with_compatibility_note("等待历史数据迁移完成。")
                .deprecated("后续统一迁移到 source_ref。"),
        )
        .register_text_field_with_governance(
            "old_source",
            vec![MetadataFieldTarget::Node],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Source)
                .with_description("更早版本来源字段。")
                .with_applies_to_reason("兼容更早版本节点来源字段。")
                .with_aliases(vec!["shared_source_alias"])
                .with_replaced_by("source_ref")
                .with_compatibility_note("等待旧导入链路下线。")
                .deprecated("更早版本字段，准备下线。"),
        );

    let summary = registry.validate_governance();

    assert!(summary.issues.contains(&MetadataRegistryValidationIssue {
        field: "legacy_source".to_string(),
        kind: MetadataRegistryValidationIssueKind::AliasConflict,
    }));
    assert!(summary.issues.contains(&MetadataRegistryValidationIssue {
        field: "old_source".to_string(),
        kind: MetadataRegistryValidationIssueKind::AliasConflict,
    }));
}

// 2026-04-09 CST: 这里补 replacement chain 循环治理红测，原因是当前 replacement_chain_for(...) 只会静默截断循环，
// 但方案A要求把循环从“避免卡死的内部细节”提升为“可审计的治理问题”。
// 目的：先钉死替代链循环必须显式进入治理摘要，避免后续 AI 把循环误当成正常兼容链。
#[test]
fn metadata_registry_validation_summary_reports_replacement_cycles() {
    let registry = MetadataRegistry::new()
        .register_text_field_with_governance(
            "cycle_a",
            vec![MetadataFieldTarget::Node],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Source)
                .with_description("循环字段A。")
                .with_applies_to_reason("循环治理校验样本。")
                .with_replaced_by("cycle_b")
                .with_compatibility_note("临时兼容。")
                .deprecated("准备下线。"),
        )
        .register_text_field_with_governance(
            "cycle_b",
            vec![MetadataFieldTarget::Node],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Source)
                .with_description("循环字段B。")
                .with_applies_to_reason("循环治理校验样本。")
                .with_replaced_by("cycle_a")
                .with_compatibility_note("临时兼容。")
                .deprecated("准备下线。"),
        );

    let summary = registry.validate_governance();

    assert!(summary.issues.contains(&MetadataRegistryValidationIssue {
        field: "cycle_a".to_string(),
        kind: MetadataRegistryValidationIssueKind::ReplacementCycle,
    }));
    assert!(summary.issues.contains(&MetadataRegistryValidationIssue {
        field: "cycle_b".to_string(),
        kind: MetadataRegistryValidationIssueKind::ReplacementCycle,
    }));
}

// 2026-04-09 CST: 这里补多个字段汇聚到同一替代目标的治理红测，原因是方案A第三项要求 registry 能显式暴露
// “多个废弃字段都宣称迁移到同一个目标字段”带来的目录歧义，而不是留给调用方自行猜测。
// 目的：先钉死共享替代目标的歧义必须进入结构化治理摘要，避免兼容迁移路径变成隐式约定。
#[test]
fn metadata_registry_validation_summary_reports_ambiguous_shared_replacement_targets() {
    let registry = MetadataRegistry::new()
        .register_text_field_with_governance(
            "source_ref",
            vec![MetadataFieldTarget::Node],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Source)
                .with_description("标准来源字段。")
                .with_applies_to_reason("节点来源字段。"),
        )
        .register_text_field_with_governance(
            "legacy_source",
            vec![MetadataFieldTarget::Node],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Source)
                .with_description("历史来源字段。")
                .with_applies_to_reason("兼容旧节点来源字段。")
                .with_replaced_by("source_ref")
                .with_compatibility_note("等待旧导入链路下线。")
                .deprecated("准备下线。"),
        )
        .register_text_field_with_governance(
            "source_alias",
            vec![MetadataFieldTarget::Node],
            vec![MetadataConstraintOperator::Equals],
            MetadataFieldGovernance::new()
                .with_group(MetadataFieldGroup::Source)
                .with_description("来源别名字段。")
                .with_applies_to_reason("兼容旧字段别名。")
                .with_replaced_by("source_ref")
                .with_compatibility_note("等待统一迁移。")
                .deprecated("准备下线。"),
        );

    let summary = registry.validate_governance();

    assert!(summary.issues.contains(&MetadataRegistryValidationIssue {
        field: "legacy_source".to_string(),
        kind: MetadataRegistryValidationIssueKind::AmbiguousReplacementTarget,
    }));
    assert!(summary.issues.contains(&MetadataRegistryValidationIssue {
        field: "source_alias".to_string(),
        kind: MetadataRegistryValidationIssueKind::AmbiguousReplacementTarget,
    }));
}
