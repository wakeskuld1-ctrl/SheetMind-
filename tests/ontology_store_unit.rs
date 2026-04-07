use excel_skill::ops::foundation::ontology_schema::{
    OntologyConcept, OntologyRelation, OntologyRelationType, OntologySchema,
};
use excel_skill::ops::foundation::ontology_store::OntologyStore;

// 2026-04-07 CST: 这里先补 ontology_store 的最小查询测试，原因是 Task 3 已明确要求 store
// 只负责承接 schema 查询与关系邻接读取，不能直接滑向业务层能力。
// 目的：先把 store 的职责边界用测试钉死，后续 roaming / retrieval 只消费这层稳定接口。
#[test]
fn ontology_store_reads_concepts_from_schema() {
    let store = sample_ontology_store();

    assert_eq!(store.find_concept_id("Revenue"), Some("revenue"));
    assert_eq!(store.find_concept_id("sales"), Some("revenue"));
    assert_eq!(store.concept("invoice").map(|concept| concept.name.as_str()), Some("Invoice"));
}

// 2026-04-07 CST: 这里补 relation 邻接查询测试，原因是 foundation 主链下一步的 roaming
// 需要先从 ontology store 读取指定关系类型的相邻概念，不能把遍历入口绑死在 schema 内部。
// 目的：先验证 store 能按 relation type 做只读过滤，后续再在 roaming 层叠加深度和路径控制。
#[test]
fn ontology_store_returns_neighbors_by_relation_type() {
    let store = sample_ontology_store();

    let neighbors = store.related_concepts("revenue", &[OntologyRelationType::DependsOn]);

    assert_eq!(neighbors, vec!["invoice"]);
}

// 2026-04-07 CST: 这里补测试样本构造，原因是 TDD 需要一个稳定且纯内存的 ontology fixture
// 来验证 store 行为，而不是把测试耦合到未来的外部存储或更高层流水线。
// 目的：统一 Task 3 当前两条测试的数据来源，减少重复样板，同时让 concept 和 relation 样本一眼可读。
fn sample_ontology_store() -> OntologyStore {
    let schema = OntologySchema::new(
        vec![
            OntologyConcept::new("revenue", "Revenue").with_alias("sales"),
            OntologyConcept::new("invoice", "Invoice").with_alias("billing document"),
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
                to_concept_id: "trend".to_string(),
                relation_type: OntologyRelationType::References,
            },
        ],
    )
    .expect("sample schema should be valid");

    OntologyStore::new(schema)
}
