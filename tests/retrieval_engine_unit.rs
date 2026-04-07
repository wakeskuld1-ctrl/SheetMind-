use excel_skill::ops::foundation::knowledge_graph_store::KnowledgeGraphStore;
use excel_skill::ops::foundation::knowledge_record::{EvidenceRef, KnowledgeEdge, KnowledgeNode};
use excel_skill::ops::foundation::ontology_schema::OntologyRelationType;
use excel_skill::ops::foundation::retrieval_engine::{RetrievalEngine, RetrievalEngineError};
use excel_skill::ops::foundation::roaming_engine::CandidateScope;

// 2026-04-07 CST: 这里先补 scoped retrieval 的首条失败测试，原因是 Task 7 的第一条契约
// 就是 retrieval 只能在 roaming 给出的 CandidateScope 内工作，不能越过主链边界去扫全图。
// 目的：先把“只检索候选域”的约束钉死，避免后续实现为了追求命中率把 foundation 架构顺序做乱。
#[test]
fn retrieval_engine_only_scores_nodes_inside_candidate_scope() {
    let hits = sample_retrieval_engine()
        .retrieve(
            "sales trend",
            &CandidateScope {
                concept_ids: vec!["revenue".to_string()],
                path: Vec::new(),
            },
            &sample_graph_store(),
        )
        .expect("hits should exist inside scope");

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].node_id, "node-revenue-1");
}

// 2026-04-07 CST: 这里补命中结果排序测试，原因是 retrieval 作为候选域内执行器，
// 至少要先提供稳定的“高分在前”输出，后续 evidence assembly 才能消费可预测的 hit 顺序。
// 目的：先把最小排序行为固化下来，避免以后因为遍历顺序不同导致上层结果抖动。
#[test]
fn retrieval_engine_returns_hits_in_descending_score_order() {
    let hits = sample_retrieval_engine()
        .retrieve(
            "revenue trend month",
            &CandidateScope {
                concept_ids: vec!["revenue".to_string(), "trend".to_string()],
                path: Vec::new(),
            },
            &sample_graph_store(),
        )
        .expect("hits should be ranked");

    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].node_id, "node-trend-1");
    assert_eq!(hits[1].node_id, "node-revenue-1");
    assert!(hits[0].score > hits[1].score);
}

// 2026-04-07 CST: 这里补无命中错误测试，原因是 retrieval 如果在候选域里完全找不到证据，
// 应该在这一层显式失败，而不是返回空数组把含义丢给下游 assembler 或更上层去猜。
// 目的：先把 Task 7 的最小失败边界稳定下来，便于后续主链明确区分“有候选域”和“有证据命中”。
#[test]
fn retrieval_engine_returns_error_when_scope_has_no_matching_evidence() {
    let error = sample_retrieval_engine()
        .retrieve(
            "cash forecast",
            &CandidateScope {
                concept_ids: vec!["revenue".to_string()],
                path: Vec::new(),
            },
            &sample_graph_store(),
        )
        .expect_err("retrieve should fail when no scoped evidence matches");

    assert_eq!(
        error,
        RetrievalEngineError::NoEvidenceFound {
            question: "cash forecast".to_string(),
        }
    );
}

// 2026-04-07 CST: 这里集中构造 retrieval 测试所需的最小引擎，原因是当前 Task 7
// 只验证 scoped retrieval 行为，不应该把测试耦合到 dispatcher、CLI 或任何业务层。
// 目的：让红绿循环只围绕 foundation 主线进行，保持测试输入足够小、故障定位足够直接。
fn sample_retrieval_engine() -> RetrievalEngine {
    RetrievalEngine::new()
}

// 2026-04-07 CST: 这里集中构造纯内存 graph store，原因是 retrieval 的评分输入
// 来自 KnowledgeNode 的标题与正文文本，而不是任何外部文件或运行时上下文。
// 目的：用最小图谱样本同时覆盖“范围过滤”“排序”“无命中”三条契约，降低测试维护成本。
fn sample_graph_store() -> KnowledgeGraphStore {
    let nodes = vec![
        KnowledgeNode::new(
            "node-revenue-1",
            "Revenue Summary",
            "Sales trend for revenue is derived from invoices.",
        )
        .with_concept_id("revenue")
        .with_evidence_ref(EvidenceRef::new("sheet:sales", "A1:B12")),
        KnowledgeNode::new(
            "node-trend-1",
            "Trend Month Review",
            "Revenue trend by month highlights trend changes.",
        )
        .with_concept_id("trend")
        .with_evidence_ref(EvidenceRef::new("sheet:trend", "C1:D20")),
        KnowledgeNode::new(
            "node-outside-scope-1",
            "Sales Trend Forecast",
            "Sales trend forecast is strong but belongs to planning only.",
        )
        .with_concept_id("forecast")
        .with_evidence_ref(EvidenceRef::new("sheet:plan", "E1:F8")),
    ];
    let edges = vec![KnowledgeEdge::new(
        "node-revenue-1",
        "node-trend-1",
        OntologyRelationType::References,
    )];

    KnowledgeGraphStore::new(nodes, edges)
}
