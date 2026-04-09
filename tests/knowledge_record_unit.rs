use excel_skill::ops::foundation::knowledge_record::{
    EvidenceRef, KnowledgeEdge, KnowledgeNode, MetadataFieldValue,
};
use excel_skill::ops::foundation::ontology_schema::OntologyRelationType;

// 2026-04-07 CST: 这里先补 knowledge record 的最小模型测试，原因是 Task 4 需要先把节点、
// 边和证据引用的承载结构钉死，避免后续把图谱查询逻辑和数据定义再次混在一起。
// 目的：先验证 record 层能稳定表达“节点关联概念 + 节点挂载证据 + 节点间关系”三件核心事情。
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

// 2026-04-09 CST: 这里补 knowledge record 的元数据承载测试，原因是通用 MetadataConstraint 要先有稳定的节点元数据落点，
// 否则 retrieval 层会被迫回退成散落字段判断，破坏 foundation 的标准能力边界。
// 目的：先钉死 KnowledgeNode 可以稳定承载单值与多值元数据，为后续 metadata 约束匹配提供统一输入。
#[test]
fn knowledge_record_keeps_generic_metadata_fields() {
    let node = KnowledgeNode::new(
        "node-revenue-1",
        "Revenue Summary",
        "Revenue is derived from invoices.",
    )
    .with_metadata_text("source", "sheet:sales")
    .with_metadata_text("observed_at", "2026-04-01")
    .with_metadata_values("kind", vec!["table", "finance"]);

    assert_eq!(
        node.metadata.get("source"),
        Some(&MetadataFieldValue::Text("sheet:sales".to_string()))
    );
    assert_eq!(
        node.metadata.get("observed_at"),
        Some(&MetadataFieldValue::Text("2026-04-01".to_string()))
    );
    assert_eq!(
        node.metadata.get("kind"),
        Some(&MetadataFieldValue::TextList(vec![
            "table".to_string(),
            "finance".to_string(),
        ]))
    );
}
