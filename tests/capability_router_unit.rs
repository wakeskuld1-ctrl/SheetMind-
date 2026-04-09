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
    assert_eq!(route.matched_terms, vec!["sales"]);
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
    assert_eq!(route.matched_terms, vec!["billing document"]);
}

// 2026-04-09 CST: 这里补标点归一化测试，原因是 foundation 路由接下来会面对不同知识源里的
// `gross-margin` / `gross_margin` / `gross margin` 等写法差异，如果 lookup 规则不能把它们视为同一短语，
// 语义路由就会被纯文本格式噪声打断。
// 目的：先把“标点形式不同但语义相同”的最小通用契约钉死，避免后续业务层各自再补一遍格式兼容。
#[test]
fn router_normalizes_phrase_punctuation_variants() {
    let route = sample_router()
        .route(&NavigationRequest::new("review gross margin trend"))
        .expect("route should normalize punctuation variants");

    assert_eq!(route.matched_concept_ids, vec!["gross_margin"]);
    assert_eq!(route.matched_terms, vec!["gross margin"]);
}

// 2026-04-09 CST: 这里补标签约束路由测试，原因是 foundation 路由下一步必须支持“同词不同域”
// 的最小约束能力，不能在出现重名概念时仍然只能靠全局唯一 lookup key 硬扛。
// 目的：先把“相同短语在不同标签域下命中不同概念”的通用契约钉死，为后续 metadata/scope 约束继续扩展留出稳定入口。
#[test]
fn router_prefers_candidates_matching_requested_concept_tags() {
    let route = sample_router()
        .route(
            &NavigationRequest::new("review margin trend")
                .with_required_concept_tags(vec!["finance"]),
        )
        .expect("route should honor requested concept tags");

    assert_eq!(route.matched_concept_ids, vec!["gross_margin"]);
    assert_eq!(route.matched_terms, vec!["margin"]);
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
            OntologyConcept::new("gross_margin", "MarginSpread")
                .with_alias("gross-margin")
                .with_tag("finance")
                .with_alias("margin"),
            OntologyConcept::new("layout_margin", "LayoutMargin")
                .with_alias("margin")
                .with_tag("ui"),
        ],
        vec![],
    )
    .expect("sample schema should be valid");

    CapabilityRouter::new(OntologyStore::new(schema))
}
