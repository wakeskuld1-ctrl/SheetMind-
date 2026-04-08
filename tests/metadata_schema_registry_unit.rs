use excel_skill::ops::foundation::metadata_schema::{
    ConceptMetadataPolicy, MetadataFieldDefinition, MetadataSchema, MetadataSchemaError,
    MetadataValueType,
};

// 2026-04-08 CST: 这里先补字段注册表红测，原因是 metadata schema registry 的第一职责
// 不是校验数据内容，而是先把“字段是什么、字段类型是什么、允许值是什么”注册成正式对象。
// 目的：钉住字段定义检索与 allowed values 的最小契约。
#[test]
fn metadata_schema_registers_field_definitions_with_allowed_values() {
    let schema = MetadataSchema::new(
        vec![
            MetadataFieldDefinition::new("domain", MetadataValueType::String)
                .with_description("知识节点所属业务域")
                .with_allowed_value("finance")
                .with_allowed_value("operations"),
            MetadataFieldDefinition::new("priority", MetadataValueType::Integer),
        ],
        vec![],
    )
    .expect("metadata schema should accept valid field definitions");

    let field = schema
        .field_definition("domain")
        .expect("domain field definition should exist");

    assert_eq!(field.key, "domain");
    assert_eq!(field.value_type, MetadataValueType::String);
    assert_eq!(
        field.allowed_values,
        vec!["finance".to_string(), "operations".to_string()]
    );
}

// 2026-04-08 CST: 这里补 concept 绑定红测，原因是方案 B 的关键不是只做字段注册，
// 而是要把“哪些 concept 允许哪些字段、哪些字段在该 concept 下必填”正式绑定起来。
// 目的：钉住 concept metadata policy 的最小管理契约。
#[test]
fn metadata_schema_binds_allowed_and_required_fields_to_concept() {
    let schema = MetadataSchema::new(
        vec![
            MetadataFieldDefinition::new("domain", MetadataValueType::String),
            MetadataFieldDefinition::new("source_type", MetadataValueType::String),
            MetadataFieldDefinition::new("priority", MetadataValueType::Integer),
        ],
        vec![
            ConceptMetadataPolicy::new("revenue")
                .with_allowed_field("domain")
                .with_allowed_field("source_type")
                .with_required_field("domain"),
        ],
    )
    .expect("metadata schema should accept valid concept policy");

    let policy = schema
        .concept_policy("revenue")
        .expect("revenue policy should exist");

    assert!(policy.allows_field("domain"));
    assert!(policy.allows_field("source_type"));
    assert!(!policy.allows_field("priority"));
    assert!(policy.requires_field("domain"));
    assert!(!policy.requires_field("source_type"));
}

// 2026-04-08 CST: 这里补未知字段红测，原因是 concept policy 如果能引用未注册字段，
// 整个 metadata 管理系统就失去约束意义。
// 目的：把“policy 引用未知字段时必须失败”这条边界钉死。
#[test]
fn metadata_schema_rejects_unknown_field_references_in_concept_policy() {
    let error = MetadataSchema::new(
        vec![MetadataFieldDefinition::new(
            "domain",
            MetadataValueType::String,
        )],
        vec![ConceptMetadataPolicy::new("revenue").with_required_field("source_type")],
    )
    .expect_err("unknown field reference should fail");

    assert_eq!(
        error,
        MetadataSchemaError::UnknownFieldReference {
            concept_id: "revenue".to_string(),
            field_key: "source_type".to_string(),
        }
    );
}
