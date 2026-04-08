use excel_skill::ops::foundation::knowledge_record::{EvidenceRef, KnowledgeEdge, KnowledgeNode};
use excel_skill::ops::foundation::ontology_schema::OntologyRelationType;

// 2026-04-08 CST: 这里先补 knowledge record 的最小模型测试，原因是知识图谱层真正被命中的对象
// 应该是 node 与 edge，而不是直接把 ontology concept 当成证据载体。
// 目的：先钉死节点挂概念、节点挂证据、节点间关系这三条最基础契约。
#[test]
fn knowledge_record_keeps_concepts_evidence_and_edges() {
    let node = KnowledgeNode::new(
        "node-revenue-1",
        "Revenue Summary",
        "Revenue is derived from invoices.",
    )
    .with_concept_id("revenue")
    .with_concept_id("invoice")
    .with_evidence_ref(EvidenceRef::new("sheet:sales", "A1:B12"));
    let edge = KnowledgeEdge::new(
        "node-revenue-1",
        "node-invoice-1",
        OntologyRelationType::DependsOn,
    );

    assert_eq!(node.id, "node-revenue-1");
    assert_eq!(node.concept_ids, vec!["revenue", "invoice"]);
    assert_eq!(node.evidence_refs.len(), 1);
    assert_eq!(node.evidence_refs[0].source_ref, "sheet:sales");
    assert_eq!(node.evidence_refs[0].locator, "A1:B12");
    assert_eq!(edge.from_node_id, "node-revenue-1");
    assert_eq!(edge.to_node_id, "node-invoice-1");
    assert_eq!(edge.relation_type, OntologyRelationType::DependsOn);
}
