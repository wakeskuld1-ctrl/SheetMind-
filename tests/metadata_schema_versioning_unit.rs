use excel_skill::ops::foundation::metadata_schema::{
    ConceptMetadataPolicy, MetadataFieldDefinition, MetadataSchema, MetadataSchemaError,
    MetadataValueType,
};

// 2026-04-08 CST: 这里先补默认 schema version 红测，原因是 versioning 第一阶段至少要把
// “没有显式指定版本时用什么正式版本号”固定下来，而不是让调用方各自猜。
// 目的：锁住 metadata schema 的默认版本契约，为后续 migration 和兼容性判断提供统一起点。
#[test]
fn metadata_schema_uses_default_schema_version() {
    let schema = MetadataSchema::new(
        vec![MetadataFieldDefinition::new(
            "domain",
            MetadataValueType::String,
        )],
        vec![ConceptMetadataPolicy::new("revenue").with_required_field("domain")],
    )
    .expect("schema should be valid");

    assert_eq!(schema.schema_version, "metadata-schema:v1");
}

// 2026-04-08 CST: 这里补显式 schema version 红测，原因是 versioning 第一阶段不能只有默认值，
// 还必须允许上层显式声明当前 schema 的正式版本。
// 目的：为后续 schema 演进和版本迁移保留稳定构造入口。
#[test]
fn metadata_schema_accepts_explicit_schema_version() {
    let schema = MetadataSchema::new_with_version(
        "metadata-schema:v2",
        vec![MetadataFieldDefinition::new(
            "domain",
            MetadataValueType::String,
        )],
        vec![ConceptMetadataPolicy::new("revenue").with_required_field("domain")],
    )
    .expect("schema should be valid");

    assert_eq!(schema.schema_version, "metadata-schema:v2");
}

// 2026-04-08 CST: 这里补非法 schema version 红测，原因是如果允许空版本号进入 registry，
// versioning 契约会在构建期直接失效。
// 目的：把“schema version 不能为空白”这条最小治理边界钉死在构建期。
#[test]
fn metadata_schema_rejects_blank_schema_version() {
    let error = MetadataSchema::new_with_version(
        "   ",
        vec![MetadataFieldDefinition::new(
            "domain",
            MetadataValueType::String,
        )],
        vec![ConceptMetadataPolicy::new("revenue").with_required_field("domain")],
    )
    .expect_err("blank schema version should fail");

    assert_eq!(
        error,
        MetadataSchemaError::InvalidSchemaVersion {
            schema_version: "".to_string(),
        }
    );
}

// 2026-04-08 CST: 这里补最小兼容性红测，原因是 versioning 第一阶段虽然不做 migration，
// 但至少要能回答“当前 schema 是否兼容某个版本号”。
// 目的：先把兼容性契约收口为精确版本匹配，避免过早引入复杂演进规则。
#[test]
fn metadata_schema_reports_exact_version_compatibility() {
    let schema = MetadataSchema::new_with_version(
        "metadata-schema:v2",
        vec![MetadataFieldDefinition::new(
            "domain",
            MetadataValueType::String,
        )],
        vec![ConceptMetadataPolicy::new("revenue").with_required_field("domain")],
    )
    .expect("schema should be valid");

    assert!(schema.is_compatible_with("metadata-schema:v2"));
    assert!(!schema.is_compatible_with("metadata-schema:v1"));
    assert!(!schema.is_compatible_with("metadata-schema:v3"));
}
