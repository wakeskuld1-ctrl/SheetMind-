use excel_skill::ops::foundation::knowledge_graph_store::KnowledgeGraphStore;
use excel_skill::ops::foundation::knowledge_record::{EvidenceRef, KnowledgeEdge, KnowledgeNode};
use excel_skill::ops::foundation::navigation_pipeline::{
    NavigationPipeline, NavigationPipelineConfig, NavigationPipelineError,
};
use excel_skill::ops::foundation::ontology_schema::{
    OntologyConcept, OntologyRelation, OntologyRelationType, OntologySchema,
};
use excel_skill::ops::foundation::ontology_store::OntologyStore;

// 2026-04-08 CST: 这里先补 pipeline 集成红灯测试，原因是 foundation 当前虽然已经有 route / roam /
// 2026-04-08 CST: retrieve 模块，但还没有一个正式入口把它们串成闭环。
// 2026-04-08 CST: 目的：先锁定“问题 -> 结构化证据”的最小 happy path，防止后续实现只停留在测试里手工串模块。
#[test]
fn navigation_pipeline_resolves_question_into_structured_evidence() {
    let result = sample_navigation_pipeline()
        .run("show sales month")
        .expect("pipeline should resolve structured evidence");

    assert_eq!(result.route.matched_concept_ids, vec!["revenue"]);
    assert_eq!(result.roaming_path.len(), 1);
    assert_eq!(result.hits.len(), 2);
    assert_eq!(result.citations.len(), 2);
    assert!(result.summary.contains("1 route concept"));
    assert!(result.summary.contains("2 retrieval hit"));
}

// 2026-04-08 CST: 这里补 route 阶段失败测试，原因是 pipeline 不能把“连概念都没命中”的问题
// 2026-04-08 CST: 混成后面的 retrieval 空命中，否则后续调试会失去阶段边界。
// 2026-04-08 CST: 目的：先钉住 pipeline 对 route 失败的显式抬升行为。
#[test]
fn navigation_pipeline_returns_route_error_for_unknown_question() {
    let error = sample_navigation_pipeline()
        .run("forecast unknown metric")
        .expect_err("pipeline should fail at route stage");

    assert!(matches!(error, NavigationPipelineError::Route(_)));
}

// 2026-04-08 CST: 这里补 retrieval 阶段失败测试，原因是 foundation 主链要区分
// 2026-04-08 CST: “概念命中了，但候选节点里没有证据”和“根本没命中概念”。
// 2026-04-08 CST: 目的：先钉住 pipeline 对 retrieval 失败的显式抬升行为，避免以后把空命中静默吞掉。
#[test]
fn navigation_pipeline_returns_retrieve_error_when_scope_has_no_matching_evidence() {
    let error = sample_navigation_pipeline()
        .run("show bookings backlog")
        .expect_err("pipeline should fail at retrieve stage");

    assert!(matches!(error, NavigationPipelineError::Retrieve(_)));
}

// 2026-04-08 CST: 这里补自定义配置测试，原因是 Task 10 的目标就是把 pipeline 从硬编码策略
// 2026-04-08 CST: 提升成“默认值 + 可覆盖”的可调底座入口，而不是继续把漫游范围埋在实现内部。
// 2026-04-08 CST: 目的：先钉住 custom config 能限制漫游范围，确保配置对象不是摆设。
#[test]
fn navigation_pipeline_uses_custom_config_to_limit_roaming_scope() {
    let result = sample_navigation_pipeline_with_config()
        .run("show sales month")
        .expect("pipeline should still resolve evidence under custom config");

    assert_eq!(result.route.matched_concept_ids, vec!["revenue"]);
    assert_eq!(result.roaming_path.len(), 0);
    assert_eq!(result.hits.len(), 1);
    assert_eq!(result.citations.len(), 1);
    assert!(result.summary.contains("0 roaming step"));
    assert!(result.summary.contains("1 retrieval hit"));
}

// 2026-04-08 CST: 这里补 max_depth = 0 的边界测试，原因是当前 A1 已把深度预算提升为显式配置，
// 2026-04-08 CST: 但还没有一条集成测试钉住“零深度只保留种子概念、不发生漫游扩展”的合同。
// 2026-04-08 CST: 目的：确保后续继续增强 retrieval 或 profile 时，不会悄悄破坏保守深度策略。
#[test]
fn navigation_pipeline_stops_roaming_when_max_depth_is_zero() {
    let result = sample_navigation_pipeline_with_zero_depth()
        .run("show sales month")
        .expect("pipeline should still resolve seed evidence under zero depth");

    assert_eq!(result.route.matched_concept_ids, vec!["revenue"]);
    assert_eq!(result.roaming_path.len(), 0);
    assert_eq!(result.hits.len(), 1);
    assert_eq!(result.citations.len(), 1);
    assert!(result.summary.contains("0 roaming step"));
    assert!(result.summary.contains("1 retrieval hit"));
}

// 2026-04-08 CST: 这里补 max_concepts = 1 的边界测试，原因是当前 A1 已把候选预算提升为显式配置，
// 2026-04-08 CST: 但还没有一条集成测试钉住“候选预算只允许保留种子概念”的合同。
// 2026-04-08 CST: 目的：确保后续漫游增强时不会越过最小候选域预算，把额外概念偷偷带入检索阶段。
#[test]
fn navigation_pipeline_stops_roaming_when_max_concepts_is_one() {
    let result = sample_navigation_pipeline_with_single_concept_budget()
        .run("show sales month")
        .expect("pipeline should still resolve seed evidence under single concept budget");

    assert_eq!(result.route.matched_concept_ids, vec!["revenue"]);
    assert_eq!(result.roaming_path.len(), 0);
    assert_eq!(result.hits.len(), 1);
    assert_eq!(result.citations.len(), 1);
    assert!(result.summary.contains("0 roaming step"));
    assert!(result.summary.contains("1 retrieval hit"));
}

// 2026-04-08 CST: 这里集中构造 pipeline 样本，原因是当前集成测试只验证 foundation 内核闭环，
// 2026-04-08 CST: 不应该把 dispatcher、CLI、数据库或任何业务线依赖带进来。
// 2026-04-08 CST: 目的：用最小纯内存 ontology + graph 样本验证 pipeline 的正式承接点。
fn sample_navigation_pipeline() -> NavigationPipeline {
    let ontology_schema = OntologySchema::new(
        vec![
            OntologyConcept::new("revenue", "Revenue")
                .with_alias("sales")
                .with_alias("bookings"),
            OntologyConcept::new("trend", "Trend"),
        ],
        vec![OntologyRelation {
            from_concept_id: "revenue".to_string(),
            to_concept_id: "trend".to_string(),
            relation_type: OntologyRelationType::Supports,
        }],
    )
    .expect("sample ontology schema should be valid");

    let graph_store = KnowledgeGraphStore::new(
        vec![
            KnowledgeNode::new(
                "node-revenue-1",
                "Revenue Summary",
                "Sales review for revenue is derived from invoices.",
            )
            .with_concept_id("revenue")
            .with_evidence_ref(EvidenceRef::new("sheet:sales", "A1:B12")),
            KnowledgeNode::new(
                "node-trend-1",
                "Trend Review",
                "Month changes explain revenue movement over time.",
            )
            .with_concept_id("trend")
            .with_evidence_ref(EvidenceRef::new("sheet:trend", "C1:D20")),
        ],
        vec![KnowledgeEdge::new(
            "node-revenue-1",
            "node-trend-1",
            OntologyRelationType::References,
        )],
    );

    NavigationPipeline::new(OntologyStore::new(ontology_schema), graph_store)
}

// 2026-04-08 CST: 这里集中构造自定义配置样本，原因是当前要验证的是 pipeline 自身的配置承接，
// 2026-04-08 CST: 不是再次验证 ontology 或 graph fixture。
// 2026-04-08 CST: 目的：用“禁用 Supports 漫游”这类最小差异，把 custom config 行为钉住。
fn sample_navigation_pipeline_with_config() -> NavigationPipeline {
    let ontology_schema = OntologySchema::new(
        vec![
            OntologyConcept::new("revenue", "Revenue")
                .with_alias("sales")
                .with_alias("bookings"),
            OntologyConcept::new("trend", "Trend"),
        ],
        vec![OntologyRelation {
            from_concept_id: "revenue".to_string(),
            to_concept_id: "trend".to_string(),
            relation_type: OntologyRelationType::Supports,
        }],
    )
    .expect("sample ontology schema should be valid");

    let graph_store = KnowledgeGraphStore::new(
        vec![
            KnowledgeNode::new(
                "node-revenue-1",
                "Revenue Summary",
                "Sales review for revenue is derived from invoices.",
            )
            .with_concept_id("revenue")
            .with_evidence_ref(EvidenceRef::new("sheet:sales", "A1:B12")),
            KnowledgeNode::new(
                "node-trend-1",
                "Trend Review",
                "Month changes explain revenue movement over time.",
            )
            .with_concept_id("trend")
            .with_evidence_ref(EvidenceRef::new("sheet:trend", "C1:D20")),
        ],
        vec![KnowledgeEdge::new(
            "node-revenue-1",
            "node-trend-1",
            OntologyRelationType::References,
        )],
    );

    let config = NavigationPipelineConfig::default()
        .with_allowed_relation_types(vec![OntologyRelationType::DependsOn])
        .with_max_depth(1)
        .with_max_concepts(8);

    NavigationPipeline::new_with_config(OntologyStore::new(ontology_schema), graph_store, config)
}

// 2026-04-08 CST: 这里集中构造 zero depth 配置样本，原因是这一轮要验证的是 pipeline
// 2026-04-08 CST: 对深度预算边界的承接，而不是继续扩写新的 ontology/graph fixture。
// 2026-04-08 CST: 目的：用最小差异配置钉住“零深度不漫游”的集成合同。
fn sample_navigation_pipeline_with_zero_depth() -> NavigationPipeline {
    let ontology_schema = OntologySchema::new(
        vec![
            OntologyConcept::new("revenue", "Revenue")
                .with_alias("sales")
                .with_alias("bookings"),
            OntologyConcept::new("trend", "Trend"),
        ],
        vec![OntologyRelation {
            from_concept_id: "revenue".to_string(),
            to_concept_id: "trend".to_string(),
            relation_type: OntologyRelationType::Supports,
        }],
    )
    .expect("sample ontology schema should be valid");

    let graph_store = KnowledgeGraphStore::new(
        vec![
            KnowledgeNode::new(
                "node-revenue-1",
                "Revenue Summary",
                "Sales review for revenue is derived from invoices.",
            )
            .with_concept_id("revenue")
            .with_evidence_ref(EvidenceRef::new("sheet:sales", "A1:B12")),
            KnowledgeNode::new(
                "node-trend-1",
                "Trend Review",
                "Month changes explain revenue movement over time.",
            )
            .with_concept_id("trend")
            .with_evidence_ref(EvidenceRef::new("sheet:trend", "C1:D20")),
        ],
        vec![KnowledgeEdge::new(
            "node-revenue-1",
            "node-trend-1",
            OntologyRelationType::References,
        )],
    );

    let config = NavigationPipelineConfig::default()
        .with_allowed_relation_types(vec![OntologyRelationType::Supports])
        .with_max_depth(0)
        .with_max_concepts(8);

    NavigationPipeline::new_with_config(OntologyStore::new(ontology_schema), graph_store, config)
}

// 2026-04-08 CST: 这里集中构造 single concept budget 配置样本，原因是这一轮要验证的是 pipeline
// 2026-04-08 CST: 对候选预算边界的承接，而不是继续扩写新的 fixture 体系。
// 2026-04-08 CST: 目的：用最小差异配置钉住“预算为 1 时只保留种子概念”的集成合同。
fn sample_navigation_pipeline_with_single_concept_budget() -> NavigationPipeline {
    let ontology_schema = OntologySchema::new(
        vec![
            OntologyConcept::new("revenue", "Revenue")
                .with_alias("sales")
                .with_alias("bookings"),
            OntologyConcept::new("trend", "Trend"),
        ],
        vec![OntologyRelation {
            from_concept_id: "revenue".to_string(),
            to_concept_id: "trend".to_string(),
            relation_type: OntologyRelationType::Supports,
        }],
    )
    .expect("sample ontology schema should be valid");

    let graph_store = KnowledgeGraphStore::new(
        vec![
            KnowledgeNode::new(
                "node-revenue-1",
                "Revenue Summary",
                "Sales review for revenue is derived from invoices.",
            )
            .with_concept_id("revenue")
            .with_evidence_ref(EvidenceRef::new("sheet:sales", "A1:B12")),
            KnowledgeNode::new(
                "node-trend-1",
                "Trend Review",
                "Month changes explain revenue movement over time.",
            )
            .with_concept_id("trend")
            .with_evidence_ref(EvidenceRef::new("sheet:trend", "C1:D20")),
        ],
        vec![KnowledgeEdge::new(
            "node-revenue-1",
            "node-trend-1",
            OntologyRelationType::References,
        )],
    );

    let config = NavigationPipelineConfig::default()
        .with_allowed_relation_types(vec![OntologyRelationType::Supports])
        .with_max_depth(1)
        .with_max_concepts(1);

    NavigationPipeline::new_with_config(OntologyStore::new(ontology_schema), graph_store, config)
}
