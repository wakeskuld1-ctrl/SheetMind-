use excel_skill::ops::foundation::ontology_schema::{
    OntologyConcept, OntologyRelation, OntologyRelationType, OntologySchema,
};
use excel_skill::ops::foundation::metadata_constraint::{MetadataConstraint, MetadataScope};
use excel_skill::ops::foundation::ontology_store::OntologyStore;
use excel_skill::ops::foundation::roaming_engine::{RoamingEngine, RoamingPlan};

// 2026-04-07 CST: 这里先补受限漫游测试，原因是 Task 6 的主目标就是让 foundation 主链
// 从种子概念出发，在允许关系和深度预算内稳定扩展候选概念，而不是无限扩散。
// 目的：先钉死“只沿允许关系扩展，并且在最大深度处停止”的最小契约。
#[test]
fn roaming_engine_stops_at_allowed_relations_and_depth() {
    let scope = sample_roaming_engine()
        .roam(
            RoamingPlan::new(vec!["revenue"])
                .with_allowed_relation_types(vec![OntologyRelationType::DependsOn])
                .with_max_depth(1)
                .with_max_concepts(4),
        )
        .expect("scope should be created");

    assert_eq!(scope.concept_ids, vec!["revenue", "invoice"]);
    assert_eq!(scope.path.len(), 1);
    assert_eq!(scope.path[0].from_concept_id, "revenue");
    assert_eq!(scope.path[0].to_concept_id, "invoice");
    assert_eq!(scope.path[0].relation_type, OntologyRelationType::DependsOn);
    assert_eq!(scope.path[0].depth, 1);
}

// 2026-04-07 CST: 这里补最大概念数预算测试，原因是 roaming 如果不受候选规模约束，
// 后续 retrieval 输入会迅速膨胀，主链会偏离“受控候选域”的设计目标。
// 目的：先确保漫游在达到 max_concepts 后及时停下，保持候选范围可预测。
#[test]
fn roaming_engine_respects_max_concepts_budget() {
    let scope = sample_roaming_engine()
        .roam(
            RoamingPlan::new(vec!["revenue"])
                .with_allowed_relation_types(vec![
                    OntologyRelationType::DependsOn,
                    OntologyRelationType::Supports,
                ])
                .with_max_depth(1)
                .with_max_concepts(2),
        )
        .expect("scope should be created");

    assert_eq!(scope.concept_ids, vec!["revenue", "invoice"]);
    assert_eq!(scope.path.len(), 1);
}

// 2026-04-09 CST: 这里补 roaming 保留 metadata scope 的红测，原因是方案B第二阶段要把 metadata 正式提升为
// RoamingPlan -> CandidateScope -> Retrieval 共用的标准输入，而不是继续让 retrieval 单独吃外部参数。
// 目的：先钉死 roaming 至少要把 metadata scope 稳定保留到输出 scope 中，为后续真正的元数据收敛留出统一合同。
#[test]
fn roaming_engine_preserves_metadata_scope_in_candidate_scope() {
    let metadata_scope = MetadataScope::from_constraints(vec![MetadataConstraint::equals(
        "source",
        "sheet:sales",
    )]);
    let scope = sample_roaming_engine()
        .roam(
            RoamingPlan::new(vec!["revenue"])
                .with_metadata_scope(metadata_scope.clone())
                .with_max_concepts(4),
        )
        .expect("scope should preserve metadata scope");

    assert_eq!(scope.metadata_scope, metadata_scope);
}

// 2026-04-07 CST: 这里集中构造纯内存 ontology store，原因是 Task 6 当前只验证漫游规则，
// 不能让测试耦合到 graph store、retrieval 或业务层运行时依赖。
// 目的：用最小关系图样本支撑受限 BFS 的红绿闭环。
fn sample_roaming_engine() -> RoamingEngine {
    let schema = OntologySchema::new(
        vec![
            OntologyConcept::new("revenue", "Revenue").with_alias("sales"),
            OntologyConcept::new("invoice", "Invoice"),
            OntologyConcept::new("margin", "Margin"),
            OntologyConcept::new("trend", "Trend"),
        ],
        vec![
            OntologyRelation {
                from_concept_id: "revenue".to_string(),
                to_concept_id: "invoice".to_string(),
                relation_type: OntologyRelationType::DependsOn,
            },
            OntologyRelation {
                from_concept_id: "revenue".to_string(),
                to_concept_id: "margin".to_string(),
                relation_type: OntologyRelationType::Supports,
            },
            OntologyRelation {
                from_concept_id: "invoice".to_string(),
                to_concept_id: "trend".to_string(),
                relation_type: OntologyRelationType::References,
            },
        ],
    )
    .expect("sample schema should be valid");

    RoamingEngine::new(OntologyStore::new(schema))
}
