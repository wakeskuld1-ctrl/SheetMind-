use excel_skill::ops::foundation::knowledge_graph_store::KnowledgeGraphStore;
use excel_skill::ops::foundation::knowledge_record::{EvidenceRef, KnowledgeEdge, KnowledgeNode};
use excel_skill::ops::foundation::ontology_schema::OntologyRelationType;
use excel_skill::ops::foundation::retrieval_engine::{RetrievalEngine, RetrievalEngineError};
use excel_skill::ops::foundation::roaming_engine::CandidateScope;

// 2026-04-08 CST: 这里先补 scoped retrieval 的范围测试，原因是 Task 7 的第一条边界不是“尽量多找”，
// 而是 retrieval 只能在 roaming 给出的 CandidateScope 内工作。
// 目的：先钉住“只检索候选域”的契约，避免实现时越过 foundation 主链边界扫描全图。
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

// 2026-04-08 CST: 这里补命中排序测试，原因是 retrieval 输出如果不稳定排序，
// 上层 evidence assembly 和后续 pipeline 结果会随遍历顺序抖动。
// 目的：先固定“高分在前”的最小行为，保证结果顺序可预测。
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

// 2026-04-08 CST: 这里补无命中失败测试，原因是如果候选域里完全没有相关证据，
// retrieval 应该明确失败，而不是返回空数组把语义丢给下游猜。
// 目的：先固定 Task 7 的失败边界，便于后续 pipeline 清晰区分“有 scope”和“有 hit”。
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

// 2026-04-08 CST: 这里集中构造 retrieval 引擎样本，原因是当前阶段只验证 foundation 内核行为，
// 不应把 dispatcher、CLI 或业务对象耦合进单测。
// 目的：让红绿循环只围绕 scoped retrieval 本身展开。
fn sample_retrieval_engine() -> RetrievalEngine {
    RetrievalEngine::new()
}

// 2026-04-08 CST: 这里集中构造最小 graph store，原因是 retrieval 的评分输入只来自节点文本与证据引用，
// 不需要任何外部文件或运行时上下文。
// 目的：用一个小图同时覆盖范围过滤、排序和无命中三条契约。
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
