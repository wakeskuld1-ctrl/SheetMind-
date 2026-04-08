use excel_skill::ops::foundation::ontology_schema::{
    OntologyConcept, OntologyRelation, OntologyRelationType, OntologySchema,
};
use excel_skill::ops::foundation::ontology_store::OntologyStore;
use excel_skill::ops::foundation::roaming_engine::{RoamingEngine, RoamingPlan};

// 2026-04-08 CST: 这里先补受限漫游测试，原因是 Task 6 的核心目标不是“能遍历就行”，
// 而是要保证只沿允许关系扩展，并且严格受 max_depth 约束。
// 目的：先把 roaming 的最小搜索边界钉住，避免后续实现无意间扩散到无关 concept。
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

// 2026-04-08 CST: 这里补 concept 预算测试，原因是 foundation 主链后面要把 scope 交给 retrieval，
// 如果 roaming 不在这里截断候选规模，后续检索输入会快速失控。
// 目的：先锁定达到 max_concepts 后立即停止扩展的契约，保证候选域大小可预测。
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

// 2026-04-08 CST: 这里集中构造最小 ontology 样本，原因是当前只验证 roaming 行为，
// 不应该把 graph store、retrieval 或证券分析主链耦合进测试。
// 目的：用一个可控的小图支持红绿循环，确保失败和通过都来自 roaming 本身。
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
