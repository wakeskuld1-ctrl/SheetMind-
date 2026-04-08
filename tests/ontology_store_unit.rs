use excel_skill::ops::foundation::ontology_schema::{
    OntologyConcept, OntologyRelation, OntologyRelationType, OntologySchema,
};
use excel_skill::ops::foundation::ontology_store::OntologyStore;

// 2026-04-08 CST: 这里先补 ontology store 的最小查询测试，原因是第一阶段进入 store 层后，
// 需要把 schema 的直接访问收口成稳定查询接口，而不是让后续 roaming 到处碰内部索引。
// 目的：先固定 concept 读取的最小契约，为后续路由和漫游共享同一入口。
#[test]
fn ontology_store_reads_concepts_from_schema() {
    let store = sample_ontology_store();

    assert_eq!(store.find_concept_id("Revenue"), Some("revenue"));
    assert_eq!(store.find_concept_id("sales"), Some("revenue"));
    assert_eq!(
        store
            .concept("invoice")
            .map(|concept| concept.name.as_str()),
        Some("Invoice")
    );
}

// 2026-04-08 CST: 这里补 relation 邻接读取测试，原因是 roaming 阶段只应消费 store 暴露的
// 指定关系过滤结果，而不应该自行遍历 schema 原始 relation 向量。
// 目的：先把 “concept -> allowed relation neighbors” 这条最小契约钉死，避免后续职责漂移。
#[test]
fn ontology_store_returns_neighbors_by_relation_type() {
    let store = sample_ontology_store();

    let neighbors = store.related_concepts("revenue", &[OntologyRelationType::DependsOn]);

    assert_eq!(neighbors, vec!["invoice"]);
}

// 2026-04-08 CST: 这里集中构造纯内存 ontology store 样本，原因是当前阶段只验证 foundation
// 底座查询行为，不应该把测试耦合到业务运行时、持久化或外部数据源。
// 目的：统一两条 store 测试的数据来源，让红绿循环更小、更直接、更容易定位问题。
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
