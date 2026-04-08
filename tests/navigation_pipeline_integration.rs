use excel_skill::ops::foundation::capability_router::NavigationRequest;
use excel_skill::ops::foundation::evidence_assembler::NavigationEvidence;
use excel_skill::ops::foundation::knowledge_graph_store::KnowledgeGraphStore;
use excel_skill::ops::foundation::knowledge_record::{EvidenceRef, KnowledgeEdge, KnowledgeNode};
use excel_skill::ops::foundation::navigation_pipeline::{
    NavigationPipeline, NavigationPipelineError,
};
use excel_skill::ops::foundation::ontology_schema::{
    OntologyConcept, OntologyRelation, OntologyRelationType, OntologySchema,
};
use excel_skill::ops::foundation::ontology_store::OntologyStore;
use excel_skill::ops::foundation::roaming_engine::RoamingPlan;

// 2026-04-08 CST: 这里先补最小集成闭环测试，原因是 Task 9 的核心不是更多单测，
// 而是证明 foundation 主链能从问题文本一路走到结构化证据对象。
// 目的：钉住 route -> roam -> retrieve -> assemble 的整体行为。
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

// 2026-04-08 CST: 这里补 pipeline 错误透传测试，原因是统一入口如果吞掉上游错误，
// 后续调用方会失去定位失败环节的能力。
// 目的：固定最小集成入口的失败语义。
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

// 2026-04-08 CST: 这里集中构造最小 pipeline 样本，原因是当前集成测试只验证 foundation 内存闭环，
// 不应引入 dispatcher、运行时状态或证券分析链条。
// 目的：保证失败和通过都只来自导航内核本身。
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

// 2026-04-08 CST: 这里保留结果类型守卫，原因是 NavigationEvidence 需要成为 foundation 主链正式输出，
// 不应只在测试过程里隐式存在。
// 目的：固定闭环结果的外部形状。
#[allow(dead_code)]
fn _pipeline_result_guard(result: NavigationEvidence) -> NavigationEvidence {
    result
}
