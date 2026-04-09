use excel_skill::ops::foundation::metadata_constraint::{MetadataConstraint, MetadataScope};
use excel_skill::ops::foundation::metadata_registry::{
    MetadataConstraintOperator, MetadataFieldTarget, MetadataRegistry, MetadataRegistryError,
};
use excel_skill::ops::foundation::metadata_scope_resolver::MetadataScopeResolver;
use excel_skill::ops::foundation::ontology_schema::{OntologyConcept, OntologySchema};
use excel_skill::ops::foundation::ontology_store::OntologyStore;

// 2026-04-09 CST: 这里补 metadata scope resolver 的 `In` 约束红测，原因是方案B这一阶段要把 metadata-aware scope
// 收敛逻辑单独沉淀成 foundation 通用层，而不是继续散在 pipeline 或 roaming 里临时判断。
// 目的：先钉死 resolver 能基于概念级 metadata 对 concept ids 做标准化白名单收窄。
#[test]
fn metadata_scope_resolver_filters_concept_ids_by_in_constraint() {
    let ontology_store = sample_metadata_resolver_store();
    let concept_ids = vec![
        "revenue".to_string(),
        "layout_margin".to_string(),
        "ops_margin".to_string(),
    ];
    let constrained = MetadataScopeResolver::constrain_concept_ids(
        &ontology_store,
        concept_ids.as_slice(),
        &MetadataScope::from_constraints(vec![MetadataConstraint::in_values(
            "domain",
            vec!["finance", "ops"],
        )]),
    );

    assert_eq!(constrained, vec!["revenue", "ops_margin"]);
}

// 2026-04-09 CST: 这里补 metadata scope resolver 的 `HasAny` 约束红测，原因是通用 metadata 管理后续会频繁遇到
// 多值标签/能力面集合，不能只验证单值或范围语义。
// 目的：先钉死 resolver 对多值概念 metadata 的交集匹配能力，给后续 roaming 收敛复用。
#[test]
fn metadata_scope_resolver_filters_concept_ids_by_has_any_constraint() {
    let ontology_store = sample_metadata_resolver_store();
    let concept_ids = vec![
        "revenue".to_string(),
        "layout_margin".to_string(),
        "ops_margin".to_string(),
    ];
    let constrained = MetadataScopeResolver::constrain_concept_ids(
        &ontology_store,
        concept_ids.as_slice(),
        &MetadataScope::from_constraints(vec![MetadataConstraint::has_any(
            "channels",
            vec!["core"],
        )]),
    );

    assert_eq!(constrained, vec!["revenue"]);
}

// 2026-04-09 CST: 这里补 registry 模式下 concept resolver 对未注册字段的红测，原因是 route / roam 在启用字段目录后也应受同一份治理合同约束，
// 不能让未知字段在 concept 收敛阶段静默掉过去，然后把问题推迟到下游。
// 目的：把“registry 模式下字段必须先注册”固定成 concept-level 收敛的直接错误边界。
#[test]
fn metadata_scope_resolver_returns_error_for_unregistered_field_in_registry_mode() {
    let ontology_store = sample_metadata_resolver_store();
    let metadata_registry = MetadataRegistry::new().register_text_field(
        "domain",
        vec![MetadataFieldTarget::Concept],
        vec![MetadataConstraintOperator::Equals],
    );
    let concept_ids = vec!["revenue".to_string(), "layout_margin".to_string()];
    let error = MetadataScopeResolver::constrain_concept_ids_with_registry(
        &ontology_store,
        concept_ids.as_slice(),
        &MetadataScope::from_constraints(vec![MetadataConstraint::equals(
            "namespace",
            "finance",
        )]),
        &metadata_registry,
    )
    .expect_err("unregistered field should be rejected by concept resolver");

    assert_eq!(
        error,
        MetadataRegistryError::UnregisteredField {
            field: "namespace".to_string(),
        }
    );
}

// 2026-04-09 CST: 这里补 registry 模式下 concept resolver 对不支持 operator 的红测，原因是 concept target 的字段合同如果只声明 Equals，
// resolver 就不应继续接受 HasAny 这类超出合同的约束。
// 目的：让 concept-level metadata 收敛与 retrieval 一样遵守同一套 operator 合同，而不是出现上下游语义分叉。
#[test]
fn metadata_scope_resolver_returns_error_for_unsupported_operator_in_registry_mode() {
    let ontology_store = sample_metadata_resolver_store();
    let metadata_registry = MetadataRegistry::new().register_text_field(
        "domain",
        vec![MetadataFieldTarget::Concept],
        vec![MetadataConstraintOperator::Equals],
    );
    let concept_ids = vec!["revenue".to_string(), "layout_margin".to_string()];
    let error = MetadataScopeResolver::constrain_concept_ids_with_registry(
        &ontology_store,
        concept_ids.as_slice(),
        &MetadataScope::from_constraints(vec![MetadataConstraint::has_any(
            "domain",
            vec!["finance"],
        )]),
        &metadata_registry,
    )
    .expect_err("unsupported operator should be rejected by concept resolver");

    assert_eq!(
        error,
        MetadataRegistryError::UnsupportedOperator {
            field: "domain".to_string(),
            operator: MetadataConstraintOperator::HasAny,
            target: MetadataFieldTarget::Concept,
        }
    );
}

// 2026-04-09 CST: 这里集中构造概念级 metadata resolver 样本，原因是当前阶段要验证的是“概念 metadata 收敛”，
// 不是节点检索或业务链路。
// 目的：用最小 ontology fixture 覆盖单值与多值 metadata 约束的基础行为。
fn sample_metadata_resolver_store() -> OntologyStore {
    let schema = OntologySchema::new(
        vec![
            OntologyConcept::new("revenue", "Revenue")
                .with_metadata_text("domain", "finance")
                .with_metadata_values("channels", vec!["core", "analytics"]),
            OntologyConcept::new("layout_margin", "LayoutMargin")
                .with_metadata_text("domain", "ui")
                .with_metadata_values("channels", vec!["design"]),
            OntologyConcept::new("ops_margin", "OpsMargin")
                .with_metadata_text("domain", "ops")
                .with_metadata_values("channels", vec!["operations"]),
        ],
        vec![],
    )
    .expect("metadata resolver schema should be valid");

    OntologyStore::new(schema)
}
