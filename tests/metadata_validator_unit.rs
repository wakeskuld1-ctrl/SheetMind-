use excel_skill::ops::foundation::knowledge_record::KnowledgeNode;
use excel_skill::ops::foundation::metadata_schema::{
    ConceptMetadataPolicy, MetadataFieldDefinition, MetadataSchema, MetadataValueType,
};
use excel_skill::ops::foundation::metadata_validator::{
    MetadataValidationIssue, MetadataValidator,
};

// 2026-04-08 CST: 这里先补 required 字段缺失红测，原因是 registry 建好以后，validator 最基本的职责
// 就是把 concept policy 里的 required 约束真正落到节点 metadata 上。
// 目的：钉住“required 字段缺失必须报结构化错误”的最小执行契约。
#[test]
fn metadata_validator_reports_missing_required_field() {
    let schema = sample_schema();
    let node = KnowledgeNode::new("node-revenue-1", "Revenue Summary", "Body")
        .with_concept_id("revenue")
        .with_metadata_entry("source_type", "table");

    let issues = MetadataValidator::new(&schema).validate_node(&node);

    assert_eq!(
        issues,
        vec![MetadataValidationIssue::MissingRequiredField {
            node_id: "node-revenue-1".to_string(),
            concept_id: "revenue".to_string(),
            field_key: "domain".to_string(),
        }]
    );
}

// 2026-04-08 CST: 这里补未知 concept policy 红测，原因是节点如果挂了 concept，
// 但 schema 没有对应 policy，validator 不能静默跳过，否则 concept 级约束就形同虚设。
// 目的：钉住 “unknown concept policy” 这条执行层错误边界。
#[test]
fn metadata_validator_reports_missing_concept_policy() {
    let schema = MetadataSchema::new(
        vec![MetadataFieldDefinition::new(
            "domain",
            MetadataValueType::String,
        )],
        vec![],
    )
    .expect("schema should be valid");
    let node = KnowledgeNode::new("node-revenue-1", "Revenue Summary", "Body")
        .with_concept_id("revenue")
        .with_metadata_entry("domain", "finance");

    let issues = MetadataValidator::new(&schema).validate_node(&node);

    assert_eq!(
        issues,
        vec![MetadataValidationIssue::MissingConceptPolicy {
            node_id: "node-revenue-1".to_string(),
            concept_id: "revenue".to_string(),
        }]
    );
}

// 2026-04-08 CST: 这里补 disallowed field 红测，原因是 concept policy 除了 required，
// 还要限制“哪些字段根本不允许出现”。
// 目的：钉住 concept-field 兼容性校验契约。
#[test]
fn metadata_validator_reports_disallowed_field_for_concept() {
    let schema = sample_schema();
    let node = KnowledgeNode::new("node-revenue-1", "Revenue Summary", "Body")
        .with_concept_id("revenue")
        .with_metadata_entry("domain", "finance")
        .with_metadata_entry("owner", "ops-team");

    let issues = MetadataValidator::new(&schema).validate_node(&node);

    assert_eq!(
        issues,
        vec![MetadataValidationIssue::DisallowedField {
            node_id: "node-revenue-1".to_string(),
            concept_id: "revenue".to_string(),
            field_key: "owner".to_string(),
        }]
    );
}

// 2026-04-08 CST: 这里补 allowed values 和类型校验红测，原因是方案 B 不只是检查字段有没有出现，
// 还要真正校验值是否合法。
// 目的：钉住枚举值校验与字符串到基础类型校验的最小执行契约。
#[test]
fn metadata_validator_reports_invalid_allowed_value_and_type() {
    let schema = sample_schema();
    let node = KnowledgeNode::new("node-revenue-1", "Revenue Summary", "Body")
        .with_concept_id("revenue")
        .with_metadata_entry("domain", "marketing")
        .with_metadata_entry("priority", "high");

    let issues = MetadataValidator::new(&schema).validate_node(&node);

    assert_eq!(
        issues,
        vec![
            MetadataValidationIssue::InvalidAllowedValue {
                node_id: "node-revenue-1".to_string(),
                field_key: "domain".to_string(),
                actual_value: "marketing".to_string(),
                allowed_values: vec!["finance".to_string(), "operations".to_string()],
            },
            MetadataValidationIssue::InvalidValueType {
                node_id: "node-revenue-1".to_string(),
                field_key: "priority".to_string(),
                expected_type: MetadataValueType::Integer,
                actual_value: "high".to_string(),
            },
        ]
    );
}

// 2026-04-08 CST: 这里补多 concept 兼容性红测，原因是方案 B 明确要求 validator 能处理
// 一个节点同时挂多个 concept 的场景。
// 目的：钉住“字段必须同时被所有 concept policy 允许，required 取并集”的兼容语义。
#[test]
fn metadata_validator_requires_multi_concept_field_compatibility() {
    let schema = MetadataSchema::new(
        vec![
            MetadataFieldDefinition::new("domain", MetadataValueType::String)
                .with_allowed_value("finance")
                .with_allowed_value("operations"),
            MetadataFieldDefinition::new("source_type", MetadataValueType::String)
                .with_allowed_value("table")
                .with_allowed_value("memo"),
        ],
        vec![
            ConceptMetadataPolicy::new("revenue")
                .with_allowed_field("domain")
                .with_required_field("domain"),
            ConceptMetadataPolicy::new("invoice")
                .with_allowed_field("source_type")
                .with_required_field("source_type"),
        ],
    )
    .expect("schema should be valid");

    let node = KnowledgeNode::new("node-shared-1", "Shared Node", "Body")
        .with_concept_id("revenue")
        .with_concept_id("invoice")
        .with_metadata_entry("domain", "finance");

    let issues = MetadataValidator::new(&schema).validate_node(&node);

    assert_eq!(
        issues,
        vec![
            MetadataValidationIssue::DisallowedField {
                node_id: "node-shared-1".to_string(),
                concept_id: "invoice".to_string(),
                field_key: "domain".to_string(),
            },
            MetadataValidationIssue::MissingRequiredField {
                node_id: "node-shared-1".to_string(),
                concept_id: "invoice".to_string(),
                field_key: "source_type".to_string(),
            },
        ]
    );
}

// 2026-04-10 CST: 这里补 alias 联动红测，原因是 migration contract 已经声明 alias 是正式治理信号，
// 但当前 validator 还不会把 alias 解析成 canonical 字段来完成 required / allowed 校验。
// 目的：钉死“alias 字段应被 canonical policy 接受，同时输出结构化 alias issue”这条最小联动契约。
#[test]
fn metadata_validator_reports_alias_usage_without_failing_required_field() {
    let schema = sample_schema_with_governance();
    let node = KnowledgeNode::new("node-revenue-2", "Revenue Summary", "Body")
        .with_concept_id("revenue")
        .with_metadata_entry("biz_domain", "finance");

    let issues = MetadataValidator::new(&schema).validate_node(&node);

    assert_eq!(
        issues,
        vec![MetadataValidationIssue::AliasFieldUsed {
            node_id: "node-revenue-2".to_string(),
            alias_field_key: "biz_domain".to_string(),
            canonical_field_key: "domain".to_string(),
        }]
    );
}

// 2026-04-10 CST: 这里补 deprecated 字段治理红测，原因是当前 schema 已经能承载 deprecated / replaced_by，
// 但 validator 还没有把“节点正在使用已废弃字段”显式暴露出来。
// 目的：钉死节点级校验必须返回 replacement 建议，而不是把 deprecated 留在静态 schema 文档层。
#[test]
fn metadata_validator_reports_deprecated_field_usage_with_replacement() {
    let schema = sample_schema_with_governance();
    let node = KnowledgeNode::new("node-revenue-3", "Revenue Summary", "Body")
        .with_concept_id("revenue")
        .with_metadata_entry("legacy_domain", "finance");

    let issues = MetadataValidator::new(&schema).validate_node(&node);

    assert_eq!(
        issues,
        vec![MetadataValidationIssue::DeprecatedFieldUsed {
            node_id: "node-revenue-3".to_string(),
            field_key: "legacy_domain".to_string(),
            replaced_by: Some("domain".to_string()),
        }]
    );
}

// 2026-04-10 CST: 这里补 alias 命中 deprecated 字段的组合红测，原因是 alias 和 deprecated 可能同时作用于同一治理字段，
// validator 不能只吐出其中一半信号。
// 目的：钉死 alias 命中 deprecated 字段时应同时返回 alias 与 deprecated 两类结构化 issue。
#[test]
fn metadata_validator_reports_alias_and_deprecated_signals_together() {
    let schema = sample_schema_with_governance();
    let node = KnowledgeNode::new("node-revenue-4", "Revenue Summary", "Body")
        .with_concept_id("revenue")
        .with_metadata_entry("old_domain", "finance");

    let issues = MetadataValidator::new(&schema).validate_node(&node);

    assert_eq!(
        issues,
        vec![
            MetadataValidationIssue::AliasFieldUsed {
                node_id: "node-revenue-4".to_string(),
                alias_field_key: "old_domain".to_string(),
                canonical_field_key: "legacy_domain".to_string(),
            },
            MetadataValidationIssue::DeprecatedFieldUsed {
                node_id: "node-revenue-4".to_string(),
                field_key: "legacy_domain".to_string(),
                replaced_by: Some("domain".to_string()),
            },
        ]
    );
}

// 2026-04-08 CST: 这里集中构造单 concept schema，原因是当前 validator 红测主要验证单 concept 下的
// required / allowed / allowed values / type 四类最小行为。
// 目的：让测试样本保持紧凑并减少不必要噪音。
fn sample_schema() -> MetadataSchema {
    MetadataSchema::new(
        vec![
            MetadataFieldDefinition::new("domain", MetadataValueType::String)
                .with_allowed_value("finance")
                .with_allowed_value("operations"),
            MetadataFieldDefinition::new("source_type", MetadataValueType::String)
                .with_allowed_value("table")
                .with_allowed_value("memo"),
            MetadataFieldDefinition::new("priority", MetadataValueType::Integer),
        ],
        vec![
            ConceptMetadataPolicy::new("revenue")
                .with_allowed_field("domain")
                .with_allowed_field("source_type")
                .with_allowed_field("priority")
                .with_required_field("domain"),
        ],
    )
    .expect("sample schema should be valid")
}

// 2026-04-10 CST: 这里集中构造带治理信号的 schema，原因是本轮 validator 联动只围绕 alias / deprecated / replaced_by 展开，
// 不应把原有基础 validator fixture 搅乱。
// 目的：让新增红测只关注治理联动，不混入无关字段噪声。
fn sample_schema_with_governance() -> MetadataSchema {
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
        ],
        vec![
            ConceptMetadataPolicy::new("revenue")
                .with_allowed_field("domain")
                .with_required_field("domain")
                .with_allowed_field("legacy_domain"),
        ],
    )
    .expect("governance sample schema should be valid")
}
