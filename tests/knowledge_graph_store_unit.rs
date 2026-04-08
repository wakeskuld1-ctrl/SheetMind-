use excel_skill::ops::foundation::knowledge_graph_store::KnowledgeGraphStore;
use excel_skill::ops::foundation::knowledge_record::{EvidenceRef, KnowledgeEdge, KnowledgeNode};
use excel_skill::ops::foundation::ontology_schema::OntologyRelationType;

// 2026-04-08 CST: 这里补按 concept 聚合 node 的测试，原因是 retrieval 阶段不会直接扫全图，
// 它需要先把候选 concept 域映射成候选 node 域。
// 目的：先钉死 “concept ids -> node ids” 的最小查询契约。
#[test]
fn graph_store_collects_nodes_for_candidate_concepts() {
    let store = sample_graph_store();
    let node_ids = store.node_ids_for_concepts(&["revenue", "invoice"]);

    assert_eq!(node_ids, vec!["node-revenue-1", "node-invoice-1"]);
}

// 2026-04-08 CST: 这里补节点读取和出边读取测试，原因是后续 retrieval 与 evidence assembly
// 都需要从 graph store 安全获取 node 正文和 node 间关系。
// 目的：先把图谱 store 的最小只读职责钉死，不让后续模块反向依赖内部向量结构。
#[test]
fn graph_store_reads_nodes_and_outgoing_edges() {
    let store = sample_graph_store();

    let node = store.node("node-revenue-1").expect("node should exist");
    let edges = store.outgoing_edges("node-revenue-1");

    assert_eq!(node.title, "Revenue Summary");
    assert_eq!(node.body, "Revenue is derived from invoices.");
    assert_eq!(node.evidence_refs.len(), 1);
    assert_eq!(edges.len(), 2);
    assert_eq!(edges[0].to_node_id, "node-invoice-1");
    assert_eq!(edges[0].relation_type, OntologyRelationType::DependsOn);
    assert_eq!(edges[1].to_node_id, "node-trend-1");
    assert_eq!(edges[1].relation_type, OntologyRelationType::References);
}

// 2026-04-08 CST: 这里集中构造纯内存图谱样本，原因是第一阶段当前只验证 foundation 图谱读模型，
// 不应把测试耦合到持久化、外部知识文件或业务装配流程。
// 目的：让 concept 聚合与出边读取共用同一批小样本，保持测试足够直接。
fn sample_graph_store() -> KnowledgeGraphStore {
    let nodes = vec![
        KnowledgeNode::new(
            "node-revenue-1",
            "Revenue Summary",
            "Revenue is derived from invoices.",
        )
        .with_concept_id("revenue")
        .with_evidence_ref(EvidenceRef::new("sheet:sales", "A1:B12")),
        KnowledgeNode::new(
            "node-invoice-1",
            "Invoice Facts",
            "Invoices support revenue calculation.",
        )
        .with_concept_id("invoice")
        .with_evidence_ref(EvidenceRef::new("sheet:invoice", "C1:D20")),
        KnowledgeNode::new(
            "node-trend-1",
            "Trend Notes",
            "Trend analysis compares revenue by month.",
        )
        .with_concept_id("trend")
        .with_evidence_ref(EvidenceRef::new("sheet:trend", "E1:F8")),
    ];
    let edges = vec![
        KnowledgeEdge::new(
            "node-revenue-1",
            "node-invoice-1",
            OntologyRelationType::DependsOn,
        ),
        KnowledgeEdge::new(
            "node-revenue-1",
            "node-trend-1",
            OntologyRelationType::References,
        ),
    ];

    KnowledgeGraphStore::new(nodes, edges)
}
