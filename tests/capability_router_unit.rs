use excel_skill::ops::foundation::capability_router::{
    CapabilityRouter, CapabilityRouterError, NavigationRequest,
};
use excel_skill::ops::foundation::ontology_schema::{OntologyConcept, OntologySchema};
use excel_skill::ops::foundation::ontology_store::OntologyStore;

// 2026-04-08 CST: 这里先补 alias 命中测试，原因是 capability router 作为导航主链入口，
// 第一职责就是把问题文本映射到种子 concept，而不是直接进入全图检索。
// 目的：先固定单词 alias 到 concept id 的最小路由契约。
#[test]
fn router_matches_question_to_seed_concepts_by_alias() {
    let route = sample_router()
        .route(&NavigationRequest::new("show sales trend"))
        .expect("route should be created");

    assert_eq!(route.matched_concept_ids, vec!["revenue"]);
}

// 2026-04-08 CST: 这里补多词短语优先测试，原因是如果先按单 token 匹配，
// 像 `billing document` 这样的 alias 会被拆散而丢失原始语义。
// 目的：先钉死 phrase-first 行为，避免后续实现只做粗糙单词匹配。
#[test]
fn router_prefers_phrase_alias_before_single_tokens() {
    let route = sample_router()
        .route(&NavigationRequest::new("open billing document summary"))
        .expect("route should be created");

    assert_eq!(route.matched_concept_ids, vec!["invoice"]);
}

// 2026-04-08 CST: 这里补无命中错误测试，原因是主链起点如果没找到任何 concept 线索，
// 应该在 router 层明确失败，而不是把空结果继续丢给后面几层。
// 目的：固定 router 的最小失败边界。
#[test]
fn router_returns_error_when_question_has_no_known_concepts() {
    let error = sample_router()
        .route(&NavigationRequest::new("forecast unknown metric"))
        .expect_err("route should fail without known concepts");

    assert_eq!(
        error,
        CapabilityRouterError::NoConceptMatched {
            question: "forecast unknown metric".to_string(),
        }
    );
}

// 2026-04-08 CST: 这里集中构造 router 样本，原因是当前测试只验证路由层职责，
// 不应耦合图谱层、检索层或任何证券业务上下文。
// 目的：用最小 ontology store 支撑 alias 和 phrase alias 的红绿循环。
fn sample_router() -> CapabilityRouter {
    let schema = OntologySchema::new(
        vec![
            OntologyConcept::new("revenue", "Revenue").with_alias("sales"),
            OntologyConcept::new("invoice", "Invoice").with_alias("billing document"),
        ],
        vec![],
    )
    .expect("sample schema should be valid");

    CapabilityRouter::new(OntologyStore::new(schema))
}
