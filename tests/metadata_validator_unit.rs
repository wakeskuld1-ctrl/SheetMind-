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
