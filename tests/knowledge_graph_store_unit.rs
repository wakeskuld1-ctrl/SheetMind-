use excel_skill::ops::foundation::knowledge_graph_store::KnowledgeGraphStore;
use excel_skill::ops::foundation::knowledge_record::{EvidenceRef, KnowledgeEdge, KnowledgeNode};
use excel_skill::ops::foundation::ontology_schema::OntologyRelationType;

// 2026-04-07 CST: 这里先补 graph store 的按概念聚合节点测试，原因是 Task 4 的主目标之一
// 就是让 foundation 主链能从概念候选集稳定取回相关知识节点，而不是把聚合逻辑散落到后续模块。
// 目的：先固定“concept ids -> node ids”的最小查询契约，给后续 roaming / retrieval 提供稳定入口。
#[test]
fn graph_store_collects_nodes_for_candidate_concepts() {
    let store = sample_graph_store();
    let node_ids = store.node_ids_for_concepts(&["revenue", "invoice"]);

    assert_eq!(node_ids, vec!["node-revenue-1", "node-invoice-1"]);
}

// 2026-04-07 CST: 这里补节点读取与出边读取测试，原因是 retrieval 和 evidence 组装后面都需要
// 从 graph store 安全读取节点详情与节点间关系，不能直接窥探底层存储结构。
// 目的：先保证 store 具备最小只读能力，并明确当前契约是“返回节点全部出边”而不是做额外过滤。
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

// 2026-04-07 CST: 这里抽出纯内存图谱样本，原因是 Task 4 只允许先做 foundation 底座，
// 不能提前引入任何外部存储、业务层实体或运行时依赖。
// 目的：统一当前 graph store 测试数据来源，让红绿闭环聚焦在最小图谱读模型本身。
fn sample_graph_store() -> KnowledgeGraphStore {
    let nodes = vec![
        KnowledgeNode::new("node-revenue-1", "Revenue Summary", "Revenue is derived from invoices.")
            .with_concept_id("revenue")
            .with_evidence_ref(EvidenceRef::new("sheet:sales", "A1:B12")),
        KnowledgeNode::new("node-invoice-1", "Invoice Facts", "Invoices support revenue calculation.")
            .with_concept_id("invoice")
            .with_evidence_ref(EvidenceRef::new("sheet:invoice", "C1:D20")),
        KnowledgeNode::new("node-trend-1", "Trend Notes", "Trend analysis compares revenue by month.")
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
