use excel_skill::ops::foundation::knowledge_bundle::KnowledgeBundle;
use excel_skill::ops::foundation::knowledge_record::{EvidenceRef, KnowledgeEdge, KnowledgeNode};
use excel_skill::ops::foundation::ontology_schema::{
    OntologyConcept, OntologyRelation, OntologyRelationType,
};

// 2026-04-08 CST: 这里先补标准包转 store 的失败测试，原因是 phase 2 第一阶段的核心不是继续扩展导航算法，
// 而是先把“可持久化的标准知识包”钉成正式契约。
// 目的：确保 bundle 既能承载 ontology 原始数据，也能承载 graph 原始数据，并可重建查询层输入。
#[test]
fn knowledge_bundle_rebuilds_ontology_and_graph_inputs() {
    let bundle = sample_bundle();

    let ontology_store = bundle
        .to_ontology_store()
        .expect("ontology store should rebuild from bundle");
    let graph_store = bundle.to_graph_store();

    assert_eq!(bundle.schema_version, "foundation.v1");
    assert_eq!(ontology_store.find_concept_id("sales"), Some("revenue"));
    assert_eq!(
        graph_store
            .node("node-revenue-1")
            .expect("node should exist")
            .title,
        "Revenue Summary"
    );
}

// 2026-04-08 CST: 这里集中构造最小标准知识包，原因是当前测试只验证通用标准能力，
// 不应该掺入任何证券分析主链上下文。
// 目的：让 bundle 的红绿循环完全围绕通用结构展开。
fn sample_bundle() -> KnowledgeBundle {
    KnowledgeBundle::new(
        "foundation.v1",
        vec![
            OntologyConcept::new("revenue", "Revenue").with_alias("sales"),
            OntologyConcept::new("invoice", "Invoice"),
        ],
        vec![OntologyRelation {
            from_concept_id: "revenue".to_string(),
            to_concept_id: "invoice".to_string(),
            relation_type: OntologyRelationType::DependsOn,
        }],
        vec![
            KnowledgeNode::new(
                "node-revenue-1",
                "Revenue Summary",
                "Revenue comes from invoices.",
            )
            .with_concept_id("revenue")
            .with_metadata_entry("domain", "finance")
            .with_evidence_ref(EvidenceRef::new("sheet:sales", "A1:B12")),
        ],
        vec![KnowledgeEdge::new(
            "node-revenue-1",
            "node-revenue-1",
            OntologyRelationType::References,
        )],
    )
}
