use excel_skill::ops::foundation::metadata_schema::{
    MetadataFieldDefinition, MetadataSchema, MetadataSchemaError, MetadataValueType,
};

// 2026-04-08 CST: 这里先补字段演进契约注册红测，原因是 migration contract 第一阶段首先要确认
// registry 能正式承载 deprecated / replaced_by / aliases 这三类治理元数据。
// 目的：锁住字段演进对象本身的最小数据契约，为后续 migration 执行器打底。
#[test]
fn metadata_schema_registers_field_migration_contract() {
    let schema = MetadataSchema::new(
        vec![
            MetadataFieldDefinition::new("domain", MetadataValueType::String),
            MetadataFieldDefinition::new("legacy_domain", MetadataValueType::String)
                .deprecated()
                .with_replaced_by("domain")
                .with_alias("biz_domain"),
        ],
        vec![],
    )
    .expect("schema should be valid");

    let field = schema
        .field_definition("legacy_domain")
        .expect("legacy field should exist");

    assert!(field.deprecated);
    assert_eq!(field.replaced_by, Some("domain".to_string()));
    assert_eq!(field.aliases, vec!["biz_domain".to_string()]);
}

// 2026-04-08 CST: 这里补 unknown replaced_by 红测，原因是如果 replacement target 不存在，
// migration contract 在构建期就已经失去治理意义。
// 目的：把“替代目标必须已注册”这条边界锁死在 schema 构建期。
#[test]
fn metadata_schema_rejects_unknown_replaced_by_target() {
    let error = MetadataSchema::new(
        vec![
            MetadataFieldDefinition::new("legacy_domain", MetadataValueType::String)
                .deprecated()
                .with_replaced_by("domain"),
        ],
        vec![],
    )
    .expect_err("unknown replacement target should fail");

    assert_eq!(
        error,
        MetadataSchemaError::UnknownReplacementTarget {
            field_key: "legacy_domain".to_string(),
            replaced_by: "domain".to_string(),
        }
    );
}

// 2026-04-08 CST: 这里补 self replaced_by 红测，原因是字段不能把自己声明为自己的替代目标，
// 否则 migration contract 会形成无意义自环。
// 目的：锁住字段演进关系最基本的非自环约束。
#[test]
fn metadata_schema_rejects_self_replaced_by_target() {
    let error = MetadataSchema::new(
        vec![
            MetadataFieldDefinition::new("domain", MetadataValueType::String)
                .deprecated()
                .with_replaced_by("domain"),
        ],
        vec![],
    )
    .expect_err("self replacement target should fail");

    assert_eq!(
        error,
        MetadataSchemaError::SelfReplacementTarget {
            field_key: "domain".to_string(),
        }
    );
}

// 2026-04-08 CST: 这里补 alias 冲突红测，原因是如果 alias 可以和正式字段 key 重名，
// 后续 migration 审计与引用解析都会出现歧义。
// 目的：把 alias 全局唯一这条最小治理约束固定下来。
#[test]
fn metadata_schema_rejects_alias_conflict_with_field_key() {
    let error = MetadataSchema::new(
        vec![
            MetadataFieldDefinition::new("domain", MetadataValueType::String),
            MetadataFieldDefinition::new("legacy_domain", MetadataValueType::String)
                .with_alias("domain"),
        ],
        vec![],
    )
    .expect_err("alias conflict should fail");

    assert_eq!(
        error,
        MetadataSchemaError::DuplicateFieldAlias {
            alias: "domain".to_string(),
        }
    );
}
