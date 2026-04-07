use excel_skill::ops::foundation::capability_router::{
    CapabilityRouter, CapabilityRouterError, NavigationRequest,
};
use excel_skill::ops::foundation::ontology_schema::{OntologyConcept, OntologySchema};
use excel_skill::ops::foundation::ontology_store::OntologyStore;

// 2026-04-07 CST: 这里先补单词别名命中测试，原因是 Task 5 的首要目标就是把用户问题中的
// 名称或别名路由到种子概念，而不是直接进入 retrieval。
// 目的：先固定最小契约，保证 router 能把 "sales" 这类别名稳定映射到 revenue。
#[test]
fn router_matches_question_to_seed_concepts_by_alias() {
    let route = sample_router()
        .route(&NavigationRequest::new("show sales trend"))
        .expect("route should be created");

    assert_eq!(route.matched_concept_ids, vec!["revenue"]);
}

// 2026-04-07 CST: 这里补多词短语优先命中测试，原因是方案 B 已经确定要先做短语匹配，
// 再回退到单词 token，避免多词 alias 被拆碎后丢掉语义。
// 目的：先钉死 phrase-first 的行为，让后续实现不能只做单词级精确匹配。
#[test]
fn router_prefers_phrase_alias_before_single_tokens() {
    let route = sample_router()
        .route(&NavigationRequest::new("open billing document summary"))
        .expect("route should be created");

    assert_eq!(route.matched_concept_ids, vec!["invoice"]);
}

// 2026-04-07 CST: 这里补无命中错误测试，原因是 capability router 是主链起点，
// 如果问题里没有任何概念线索，就应该明确失败，而不是静默返回空路由让后续模块继续误跑。
// 目的：先确保 router 对“没有种子概念”的场景给出清晰错误边界。
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

// 2026-04-07 CST: 这里集中构造 router 测试样本，原因是 Task 5 当前只验证路由职责，
// 不应该让测试耦合到图谱层、roaming 层或任何业务层 fixture。
// 目的：用最小 ontology store 样本支撑 alias 与 phrase alias 的路由闭环。
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
