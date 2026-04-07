use excel_skill::ops::foundation::capability_router::NavigationRequest;
use excel_skill::ops::foundation::evidence_assembler::NavigationEvidence;
use excel_skill::ops::foundation::knowledge_graph_store::KnowledgeGraphStore;
use excel_skill::ops::foundation::knowledge_record::{EvidenceRef, KnowledgeEdge, KnowledgeNode};
use excel_skill::ops::foundation::navigation_pipeline::{NavigationPipeline, NavigationPipelineError};
use excel_skill::ops::foundation::ontology_schema::{
    OntologyConcept, OntologyRelation, OntologyRelationType, OntologySchema,
};
use excel_skill::ops::foundation::ontology_store::OntologyStore;
use excel_skill::ops::foundation::roaming_engine::RoamingPlan;

// 2026-04-08 CST: 这里先补 Task 9 的首条集成测试，原因是 foundation 主线已经完成到 Task 8，
// 当前最重要的是证明“问题文本 -> NavigationEvidence”这条最小闭环真的能从头跑通，而不是只停留在分段单测。
// 目的：先把 router、roaming、retrieval、assembler 串联后的整体结果钉死，避免集成入口漂移。
#[test]
fn navigation_pipeline_resolves_question_into_structured_evidence() {
    let result = sample_navigation_pipeline()
        .run(&NavigationRequest::new("show sales trend"))
        .expect("pipeline should produce structured evidence");

    assert_eq!(result.route.matched_concept_ids, vec!["revenue"]);
    assert_eq!(result.roaming_path.len(), 1);
    assert_eq!(result.hits.len(), 1);
    assert_eq!(result.hits[0].node_id, "node-revenue-1");
    assert_eq!(
        result.citations,
        vec![EvidenceRef::new("sheet:sales", "A1:B12")]
    );
}

// 2026-04-08 CST: 这里补 pipeline 错误透传测试，原因是 Task 9 不只是要跑通 happy path，
// 还要确保上游没有命中概念时，错误会在统一入口被清晰保留，而不是在集成层被吞掉。
// 目的：固定本轮最小集成入口的失败语义，为后续上层调用提供稳定错误边界。
#[test]
fn navigation_pipeline_surfaces_router_error_for_unknown_question() {
    let error = sample_navigation_pipeline()
        .run(&NavigationRequest::new("show cash forecast"))
        .expect_err("pipeline should surface router failure");

    assert_eq!(
        error,
        NavigationPipelineError::RouteFailed {
            question: "show cash forecast".to_string(),
        }
    );
}

// 2026-04-08 CST: 这里集中构造最小 pipeline 样本，原因是 Task 9 只验证 foundation 侧的纯内存闭环，
// 不应该引入 dispatcher、运行时状态或任何业务域专用对象。
// 目的：把本体、图谱、漫游计划和检索问题一起收口成可重复运行的最小集成夹具。
fn sample_navigation_pipeline() -> NavigationPipeline {
    let ontology_store = OntologyStore::new(
        OntologySchema::new(
            vec![
                OntologyConcept::new("revenue", "Revenue").with_alias("sales"),
                OntologyConcept::new("invoice", "Invoice"),
            ],
            vec![OntologyRelation {
                from_concept_id: "revenue".to_string(),
                to_concept_id: "invoice".to_string(),
                relation_type: OntologyRelationType::DependsOn,
            }],
        )
        .expect("sample schema should be valid"),
    );

    let graph_store = KnowledgeGraphStore::new(
        vec![
            KnowledgeNode::new(
                "node-revenue-1",
                "Revenue Summary",
                "Sales trend for revenue is derived from invoices.",
            )
            .with_concept_id("revenue")
            .with_evidence_ref(EvidenceRef::new("sheet:sales", "A1:B12")),
            KnowledgeNode::new(
                "node-invoice-1",
                "Invoice Support",
                "Invoice details support revenue analysis.",
            )
            .with_concept_id("invoice")
            .with_evidence_ref(EvidenceRef::new("sheet:invoice", "C1:D12")),
        ],
        vec![KnowledgeEdge::new(
            "node-revenue-1",
            "node-invoice-1",
            OntologyRelationType::DependsOn,
        )],
    );

    NavigationPipeline::new(
        ontology_store,
        graph_store,
        RoamingPlan::new(vec![])
            .with_allowed_relation_types(vec![OntologyRelationType::DependsOn])
            .with_max_depth(1)
            .with_max_concepts(4),
    )
}

#[allow(dead_code)]
fn _pipeline_result_guard(result: NavigationEvidence) -> NavigationEvidence {
    result
}
