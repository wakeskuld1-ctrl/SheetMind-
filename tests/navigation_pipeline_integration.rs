use excel_skill::ops::foundation::capability_router::NavigationRequest;
use excel_skill::ops::foundation::evidence_assembler::NavigationEvidence;
use excel_skill::ops::foundation::knowledge_graph_store::KnowledgeGraphStore;
use excel_skill::ops::foundation::knowledge_record::{EvidenceRef, KnowledgeEdge, KnowledgeNode};
use excel_skill::ops::foundation::metadata_constraint::MetadataConstraint;
use excel_skill::ops::foundation::metadata_registry::{
    MetadataConstraintOperator, MetadataFieldTarget, MetadataRegistry, MetadataRegistryError,
};
use excel_skill::ops::foundation::navigation_pipeline::{
    NavigationPipeline, NavigationPipelineError,
};
use excel_skill::ops::foundation::ontology_schema::{
    OntologyConcept, OntologyRelation, OntologyRelationType, OntologySchema,
};
use excel_skill::ops::foundation::ontology_store::OntologyStore;
use excel_skill::ops::foundation::roaming_engine::RoamingPlan;

// 2026-04-08 CST: 这里先补 Task 9 的首条失败集成测试，原因是 foundation 主线已经完成到 Task 8，
// 当前最重要的是证明“问题文本 -> NavigationEvidence”这条最小闭环真的能从头跑通，而不是只停留在分段单测。
// 目的：先把 router、roaming、retrieval、assembler 串联后的整体结果钉死，避免后续集成入口漂移。
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

// 2026-04-09 CST: 这里先补 “同词不同域 + required_concept_tags” 的 pipeline 级红测，原因是当前标签约束只在
// router 单测里验证过，还没有证明它真的穿过 foundation 主链，最终命中正确的概念、节点和证据。
// 目的：把“通用标签约束不是停留在局部匹配层，而是能稳定影响整条导航链路”的契约钉死。
#[test]
fn navigation_pipeline_prefers_tag_constrained_domain_evidence() {
    let result = sample_tag_constrained_navigation_pipeline()
        .run(
            &NavigationRequest::new("review margin trend")
                .with_required_concept_tags(vec!["finance"]),
        )
        .expect("pipeline should honor tag-constrained domain routing");

    assert_eq!(result.route.matched_concept_ids, vec!["gross_margin"]);
    assert_eq!(result.route.matched_terms, vec!["margin"]);
    assert_eq!(result.hits.len(), 1);
    assert_eq!(result.hits[0].node_id, "node-gross-margin-1");
    assert_eq!(
        result.citations,
        vec![EvidenceRef::new("sheet:finance", "B2:C20")]
    );
}

// 2026-04-09 CST: 这里补“模板种子与标签约束冲突”边界测试，原因是 foundation pipeline 现在同时有
// request 级标签约束与 plan 模板种子收敛两层控制，必须明确二者冲突时的优先级，避免后续改动把显式 scope 吞掉。
// 目的：钉死“调用方显式给出的 required_concept_tags 优先于模板默认种子，模板只在不冲突时做收敛”的通用契约。
#[test]
fn navigation_pipeline_prefers_request_tags_over_conflicting_template_seeds() {
    let result = sample_conflicting_seed_navigation_pipeline()
        .run(
            &NavigationRequest::new("review margin trend")
                .with_required_concept_tags(vec!["finance"]),
        )
        .expect("pipeline should preserve tag-constrained route under seed conflict");

    assert_eq!(result.route.matched_concept_ids, vec!["gross_margin"]);
    assert_eq!(result.route.matched_terms, vec!["margin"]);
    assert_eq!(result.hits.len(), 1);
    assert_eq!(result.hits[0].node_id, "node-gross-margin-1");
    assert_eq!(
        result.citations,
        vec![EvidenceRef::new("sheet:finance", "B2:C20")]
    );
}

// 2026-04-09 CST: 这里补“显式标签把原本可匹配候选全部过滤掉”的 pipeline 级失败测试，原因是当前 foundation 主线
// 已验证标签约束的成功路径，但还没有钉死“标签约束本身也会触发统一 RouteFailed 边界”这一失败语义。
// 目的：确认 required_concept_tags 是真正参与主链收敛的通用输入，而不是只影响成功场景的可选提示。
#[test]
fn navigation_pipeline_surfaces_router_error_when_required_tags_filter_all_candidates() {
    let error = sample_tag_constrained_navigation_pipeline()
        .run(
            &NavigationRequest::new("review margin trend")
                .with_required_concept_tags(vec!["governance"]),
        )
        .expect_err("pipeline should fail routing when required tags filter all candidates");

    assert_eq!(
        error,
        NavigationPipelineError::RouteFailed {
            question: "review margin trend".to_string(),
        }
    );
}

// 2026-04-09 CST: 这里补 pipeline 级 roam failed 测试，原因是 foundation 主线现在已经明确把 route、roam、
// retrieve 的失败都统一映射到 `NavigationPipelineError`，需要在集成层证明“路由成功但种子不可漫游”时不会误报为其他阶段错误。
// 目的：钉死 `RoamFailed` 的阶段边界，确保后续重构不会把空种子漫游场景吞成 route failed 或 retrieval failed。
#[test]
fn navigation_pipeline_surfaces_roaming_error_for_missing_seed_concepts() {
    let error = sample_missing_seed_navigation_pipeline()
        .run(&NavigationRequest::new("review margin trend"))
        .expect_err("pipeline should surface roaming failure");

    assert_eq!(
        error,
        NavigationPipelineError::RoamFailed {
            concept_ids: vec!["ghost_seed".to_string()],
        }
    );
}

// 2026-04-09 CST: 这里补 pipeline 级 retrieval failed 测试，原因是 foundation 主线还需要证明“路由成功、漫游成功，
// 但图谱中没有匹配证据”时会保留 retrieval 阶段错误，而不是静默返回空结果。
// 目的：钉死 evidence miss 的统一失败语义，方便后续上层按阶段给出可解释反馈。
#[test]
fn navigation_pipeline_surfaces_retrieval_error_when_scope_has_no_matching_evidence() {
    let error = sample_retrieval_miss_navigation_pipeline()
        .run(&NavigationRequest::new("review margin trend"))
        .expect_err("pipeline should surface retrieval failure");

    assert_eq!(
        error,
        NavigationPipelineError::RetrievalFailed {
            question: "review margin trend".to_string(),
        }
    );
}

// 2026-04-09 CST: 这里补“显式标签约束压住冲突模板后仍因无证据失败”的 pipeline 级失败测试，原因是 foundation 主线
// 还缺少一条能证明“模板冲突不会抢回 scope，但 retrieval miss 仍会保留为 RetrievalFailed”的复合失败边界。
// 目的：确认调用方给出的 required_concept_tags 优先级在失败路径中同样成立，而不是只在 happy path 生效。
#[test]
fn navigation_pipeline_surfaces_retrieval_error_when_tagged_route_overrides_conflicting_template() {
    let error = sample_conflicting_seed_retrieval_miss_navigation_pipeline()
        .run(
            &NavigationRequest::new("review margin trend")
                .with_required_concept_tags(vec!["finance"]),
        )
        .expect_err(
            "pipeline should preserve tagged route under conflicting template before retrieval miss",
        );

    assert_eq!(
        error,
        NavigationPipelineError::RetrievalFailed {
            question: "review margin trend".to_string(),
        }
    );
}

// 2026-04-09 CST: 这里补“MetadataConstraint 在主链里真实过滤 evidence”的集成测试，原因是通用元数据能力如果只停留在
// retrieval 单测层，还不能证明 `NavigationRequest -> Retrieval` 这条正式主链已经可用。
// 目的：钉死 metadata 约束会穿过 pipeline，最终稳定影响 hits 与 citations，而不是只存在于底层接口。
#[test]
fn navigation_pipeline_filters_evidence_by_metadata_constraints() {
    let result = sample_metadata_constrained_navigation_pipeline()
        .run(
            &NavigationRequest::new("review sales trend").with_metadata_constraints(vec![
                MetadataConstraint::equals("source", "sheet:sales"),
            ]),
        )
        .expect("pipeline should filter evidence by metadata constraints");

    assert_eq!(result.route.matched_concept_ids, vec!["revenue"]);
    assert_eq!(result.hits.len(), 1);
    assert_eq!(result.hits[0].node_id, "node-revenue-sheet");
    assert_eq!(
        result.citations,
        vec![EvidenceRef::new("sheet:sales", "A1:B12")]
    );
}

// 2026-04-09 CST: 这里补“MetadataConstraint 把主链 evidence 全部过滤掉”的集成失败测试，原因是 foundation 必须明确
// metadata 过滤失败仍然归属于 retrieval 阶段，而不是把失败语义模糊化为 route 失败或空结果。
// 目的：确认 metadata 成为标准能力后，失败边界仍然稳定可解释。
#[test]
fn navigation_pipeline_surfaces_retrieval_error_when_metadata_constraints_filter_all_evidence() {
    let error = sample_metadata_constrained_navigation_pipeline()
        .run(
            &NavigationRequest::new("review sales trend").with_metadata_constraints(vec![
                MetadataConstraint::equals("source", "doc:notes"),
            ]),
        )
        .expect_err("pipeline should fail when metadata constraints filter all evidence");

    assert_eq!(
        error,
        NavigationPipelineError::RetrievalFailed {
            question: "review sales trend".to_string(),
        }
    );
}

// 2026-04-09 CST: 这里补 “route + concept metadata(In) + roam/retrieve” 的集成红测，原因是 metadata 第二阶段不能只停留在
// CandidateScope 合同层，还要证明它能在 route 之后真实收窄进入 roaming 的概念集合。
// 目的：先钉死 `In` 约束会在不依赖业务特判的前提下，稳定收敛到 concept metadata 匹配的主线种子。
#[test]
fn navigation_pipeline_filters_route_by_concept_metadata_in_scope() {
    let result = sample_concept_metadata_navigation_pipeline()
        .run(
            &NavigationRequest::new("review margin revenue trend").with_metadata_constraints(vec![
                MetadataConstraint::in_values("domain", vec!["finance"]),
            ]),
        )
        .expect("pipeline should keep finance concepts under metadata in-scope");

    assert_eq!(
        result.route.matched_concept_ids,
        vec!["gross_margin", "revenue"]
    );
    assert_eq!(result.route.matched_terms, vec!["margin", "revenue"]);
    assert_eq!(result.roaming_path.len(), 2);
    assert_eq!(result.roaming_path[0].from_concept_id, "gross_margin");
    assert_eq!(result.roaming_path[1].from_concept_id, "revenue");
    assert_eq!(result.hits.len(), 2);
    assert_eq!(result.hits[0].node_id, "node-gross-margin-1");
    assert_eq!(result.hits[1].node_id, "node-revenue-1");
}

// 2026-04-09 CST: 这里补 “route + concept metadata(HasAny)” 的集成红测，原因是方案B明确要把多值 metadata 约束也纳入
// 通用收敛主线，不能只验证单值白名单。
// 目的：先钉死 `HasAny` 约束会在 route 命中后筛掉不满足多值 concept metadata 的种子，并让 roaming path 跟着一起收窄。
#[test]
fn navigation_pipeline_filters_route_and_roaming_by_concept_metadata_has_any_scope() {
    let result = sample_concept_metadata_navigation_pipeline()
        .run(
            &NavigationRequest::new("review margin revenue trend").with_metadata_constraints(vec![
                MetadataConstraint::has_any("channels", vec!["core"]),
            ]),
        )
        .expect("pipeline should keep only core-channel concepts under metadata scope");

    assert_eq!(result.route.matched_concept_ids, vec!["revenue"]);
    assert_eq!(result.route.matched_terms, vec!["revenue"]);
    assert_eq!(result.roaming_path.len(), 1);
    assert_eq!(result.roaming_path[0].from_concept_id, "revenue");
    assert_eq!(result.roaming_path[0].to_concept_id, "invoice");
    assert_eq!(result.hits.len(), 1);
    assert_eq!(result.hits[0].node_id, "node-revenue-1");
    assert_eq!(
        result.citations,
        vec![EvidenceRef::new("sheet:revenue", "A1:B12")]
    );
}

// 2026-04-09 CST: 这里补 “concept-only metadata 字段不应在 retrieval 侧误过滤节点” 的集成红测，原因是方案B进入元数据字段目录阶段后，
// 同一份 MetadataScope 会同时流经 concept 收敛与 node 检索，必须显式区分字段适用层级。
// 目的：钉死 registry 标记为 concept-only 的字段只影响 route / roam，不会把本该命中的 node evidence 错误过滤空。
#[test]
fn navigation_pipeline_keeps_hits_when_concept_only_metadata_is_registered() {
    let result = sample_concept_only_metadata_registry_navigation_pipeline()
        .run(
            &NavigationRequest::new("review margin trend").with_metadata_constraints(vec![
                MetadataConstraint::equals("namespace", "finance"),
            ]),
        )
        .expect("pipeline should ignore concept-only metadata during retrieval");

    assert_eq!(result.route.matched_concept_ids, vec!["gross_margin"]);
    assert_eq!(result.route.matched_terms, vec!["margin"]);
    assert_eq!(result.hits.len(), 1);
    assert_eq!(result.hits[0].node_id, "node-gross-margin-1");
    assert_eq!(
        result.citations,
        vec![EvidenceRef::new("sheet:finance", "B2:C20")]
    );
}

// 2026-04-09 CST: 这里补 pipeline 对 registry 未注册字段的集成红测，原因是 registry 模式如果只在底层报错而主线继续吞掉，
// 上层调用方仍然无法区分“字段治理缺失”和“正常无结果”。
// 目的：把字段目录错误正式透传到 foundation 主线入口，形成 question -> pipeline 级别的可解释失败语义。
#[test]
fn navigation_pipeline_surfaces_invalid_metadata_constraint_for_unregistered_field() {
    let error = sample_concept_only_metadata_registry_navigation_pipeline()
        .run(
            &NavigationRequest::new("review margin trend").with_metadata_constraints(vec![
                MetadataConstraint::equals("unknown_scope", "finance"),
            ]),
        )
        .expect_err("pipeline should reject unregistered metadata fields in registry mode");

    assert_eq!(
        error,
        NavigationPipelineError::InvalidMetadataConstraint(
            MetadataRegistryError::UnregisteredField {
                field: "unknown_scope".to_string(),
            }
        )
    );
}

// 2026-04-09 CST: 这里补 pipeline 对 node-target 不支持 operator 的集成红测，原因是同一份 MetadataScope 会穿过 route / roam / retrieve，
// 需要验证 concept 阶段忽略 node-only 字段后，真正的检索阶段仍会按 node target 合同拒绝非法 operator。
// 目的：钉死 registry 模式的 target-aware operator 校验会在主线中正确落到 retrieval 阶段，而不是被前序阶段误吞。
#[test]
fn navigation_pipeline_surfaces_invalid_metadata_constraint_for_unsupported_node_operator() {
    let error = sample_node_only_metadata_registry_navigation_pipeline()
        .run(
            &NavigationRequest::new("review revenue trend").with_metadata_constraints(vec![
                MetadataConstraint::has_any("source", vec!["sheet:revenue"]),
            ]),
        )
        .expect_err("pipeline should reject unsupported node-target operator in registry mode");

    assert_eq!(
        error,
        NavigationPipelineError::InvalidMetadataConstraint(
            MetadataRegistryError::UnsupportedOperator {
                field: "source".to_string(),
                operator: MetadataConstraintOperator::HasAny,
                target: MetadataFieldTarget::Node,
            }
        )
    );
}

// 2026-04-09 CST: 这里补“模板默认种子与 route 部分重叠时取交集”的边界测试，原因是 foundation pipeline 现在
// 已经明确了“有显式标签时标签优先、无显式标签时模板可参与收敛”，但还没有钉死部分重叠时究竟是取交集还是模板全覆盖。
// 目的：把方案 C 的分层优先级固定在集成层，避免模板默认种子把问题本身已经命中的主概念误覆盖掉。
#[test]
fn navigation_pipeline_prefers_intersection_when_template_and_route_partially_overlap() {
    let result = sample_partial_overlap_navigation_pipeline()
        .run(&NavigationRequest::new("review revenue margin trend"))
        .expect("pipeline should keep only overlapping seeds");

    assert_eq!(result.route.matched_concept_ids, vec!["gross_margin"]);
    assert_eq!(result.route.matched_terms, vec!["margin"]);
    assert_eq!(result.hits.len(), 1);
    assert_eq!(result.hits[0].node_id, "node-gross-margin-1");
    assert_eq!(
        result.citations,
        vec![EvidenceRef::new("sheet:finance", "B2:C20")]
    );
}

// 2026-04-09 CST: 这里补“多个交集 seed 的保留顺序”测试，原因是当前 foundation pipeline 已经确定
// 部分重叠时取交集，但还没有说明多个交集概念时应按模板顺序保留还是按 route 命中顺序保留。
// 目的：固定通用导航主线中多个保留 seeds 的稳定顺序，避免后续扩展 path 或排序逻辑时产生抖动。
#[test]
fn navigation_pipeline_keeps_multiple_intersection_seeds_in_route_order() {
    let result = sample_multi_overlap_navigation_pipeline()
        .run(&NavigationRequest::new("review margin revenue trend"))
        .expect("pipeline should preserve overlapping seeds in route order");

    assert_eq!(
        result.route.matched_concept_ids,
        vec!["gross_margin", "revenue"]
    );
    assert_eq!(result.route.matched_terms, vec!["margin", "revenue"]);
}

// 2026-04-09 CST: 这里补“多个交集 seed 下 roaming path 只围绕保留 seeds 扩展”的测试，原因是如果 path
// 仍被模板中未保留的 ghost seeds 污染，foundation 主线虽然 route 看起来正确，实际 roaming scope 仍会偏移。
// 目的：确认 candidate scope 的图谱扩展只从最终保留的交集 seeds 出发，而不是把模板中其余默认种子偷偷带入。
#[test]
fn navigation_pipeline_roams_only_from_kept_intersection_seeds() {
    let result = sample_multi_overlap_navigation_pipeline()
        .run(&NavigationRequest::new("review margin revenue trend"))
        .expect("pipeline should roam from kept intersection seeds only");

    assert_eq!(result.roaming_path.len(), 2);
    assert_eq!(result.roaming_path[0].from_concept_id, "gross_margin");
    assert_eq!(result.roaming_path[0].to_concept_id, "finance_report");
    assert_eq!(result.roaming_path[1].from_concept_id, "revenue");
    assert_eq!(result.roaming_path[1].to_concept_id, "invoice");
    assert!(result
        .roaming_path
        .iter()
        .all(|step| step.from_concept_id != "ghost_seed"));
}

// 2026-04-09 CST: 这里补“多个交集 seed + 单标签约束”的复合集成测试，原因是当前主线已经分别验证了多交集保留和标签约束，
// 但还没有证明这两个规则叠加时仍会先按标签筛掉错误候选，再只保留模板与 route 的交集主 seeds。
// 目的：钉死 foundation 通用导航内核在复合约束下的稳定行为，避免后续重构把模板 seed、route 顺序和标签过滤互相覆盖。
#[test]
fn navigation_pipeline_keeps_tag_filtered_intersection_seeds_in_route_order() {
    let result = sample_tag_filtered_multi_overlap_navigation_pipeline()
        .run(
            &NavigationRequest::new("review margin revenue trend")
                .with_required_concept_tags(vec!["finance"]),
        )
        .expect("pipeline should keep tag-filtered overlapping seeds in route order");

    assert_eq!(
        result.route.matched_concept_ids,
        vec!["gross_margin", "revenue"]
    );
    assert_eq!(result.route.matched_terms, vec!["margin", "revenue"]);
    assert_eq!(result.roaming_path.len(), 2);
    assert_eq!(result.roaming_path[0].from_concept_id, "gross_margin");
    assert_eq!(result.roaming_path[1].from_concept_id, "revenue");
    assert_eq!(result.hits.len(), 2);
    assert_eq!(result.hits[0].node_id, "node-gross-margin-1");
    assert_eq!(result.hits[1].node_id, "node-revenue-1");
    assert_eq!(
        result.citations,
        vec![
            EvidenceRef::new("sheet:finance", "B2:C20"),
            EvidenceRef::new("sheet:revenue", "A1:B12"),
        ]
    );
    assert!(result
        .roaming_path
        .iter()
        .all(|step| step.from_concept_id != "layout_margin"));
}

// 2026-04-09 CST: 这里补“多个交集 seed + 多标签约束”的复合集成测试，原因是当前 route 的 required_concept_tags 已支持多标签输入，
// 但还没有在 pipeline 主链里证明多标签条件下依旧会得到稳定的交集 seeds、顺序、漫游路径与证据输出。
// 目的：固定“多标签是通用 scope 约束输入的一部分”这一契约，避免后续把多标签退化成单标签或在模板收敛阶段丢失命中概念。
// 2026-04-09 CST: 这里追加修正断言顺序，原因是 retrieval 在分数相同的情况下会按 node_id 升序稳定排序，
// 不能把 route 顺序误当成 hits 顺序。
// 目的：让测试准确反映当前通用检索排序契约，只校验与主线真实行为一致的输出。
#[test]
fn navigation_pipeline_keeps_multi_tag_intersection_seeds_and_evidence() {
    let result = sample_multi_tag_filtered_overlap_navigation_pipeline()
        .run(
            &NavigationRequest::new("review revenue margin trend")
                .with_required_concept_tags(vec!["finance", "core"]),
        )
        .expect("pipeline should keep overlapping seeds under multi-tag constraints");

    assert_eq!(
        result.route.matched_concept_ids,
        vec!["revenue", "gross_margin"]
    );
    assert_eq!(result.route.matched_terms, vec!["revenue", "margin"]);
    assert_eq!(result.roaming_path.len(), 2);
    assert_eq!(result.roaming_path[0].from_concept_id, "revenue");
    assert_eq!(result.roaming_path[1].from_concept_id, "gross_margin");
    assert_eq!(result.hits.len(), 2);
    assert_eq!(result.hits[0].node_id, "node-gross-margin-1");
    assert_eq!(result.hits[1].node_id, "node-revenue-1");
    assert_eq!(
        result.citations,
        vec![
            EvidenceRef::new("sheet:finance", "B2:C20"),
            EvidenceRef::new("sheet:revenue", "A1:B12"),
        ]
    );
    assert!(result
        .roaming_path
        .iter()
        .all(|step| step.from_concept_id != "ui_revenue"));
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
                OntologyConcept::new("trend", "Trend"),
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
        RoamingPlan::new(vec!["revenue"])
            .with_allowed_relation_types(vec![OntologyRelationType::DependsOn])
            .with_max_depth(1)
            .with_max_concepts(4),
    )
}

// 2026-04-09 CST: 这里构造“同一概念下有多份不同元数据 evidence”的最小 pipeline 样本，原因是 MetadataConstraint 第一阶段
// 要先验证它能在不改动 roaming 语义的前提下，稳定收窄 retrieval 命中。
// 目的：把 metadata 约束作为 foundation 一等输入先打通到正式主链里。
fn sample_metadata_constrained_navigation_pipeline() -> NavigationPipeline {
    let ontology_store = OntologyStore::new(
        OntologySchema::new(
            vec![
                OntologyConcept::new("revenue", "Revenue").with_alias("sales"),
                OntologyConcept::new("trend", "Trend"),
            ],
            vec![],
        )
        .expect("metadata-constrained schema should be valid"),
    );

    let graph_store = KnowledgeGraphStore::new(
        vec![
            KnowledgeNode::new(
                "node-revenue-sheet",
                "Revenue Sales Trend",
                "Sales trend for revenue is recorded in the workbook.",
            )
            .with_concept_id("revenue")
            .with_evidence_ref(EvidenceRef::new("sheet:sales", "A1:B12"))
            .with_metadata_text("source", "sheet:sales")
            .with_metadata_text("kind", "table")
            .with_metadata_text("observed_at", "2026-04-01"),
            KnowledgeNode::new(
                "node-revenue-note",
                // 2026-04-09 CST: 这里把 metadata 备用 evidence 改成非 query 命中文本，原因是本轮只想验证 metadata 约束能筛选主链 evidence，
                // 不希望 `doc:notes` 样本天然命中 `review sales trend`，否则“metadata 全过滤后 RetrievalFailed”边界会被夹具本身抵消。
                // 目的：让 note 样本只承担元数据分支存在性的职责，避免污染当前导航 query 的默认命中集。
                "Management Commentary",
                "Analyst notes summarize leadership guidance and profitability context.",
            )
            .with_concept_id("revenue")
            .with_evidence_ref(EvidenceRef::new("doc:notes", "section-2"))
            .with_metadata_text("source", "doc:notes")
            .with_metadata_text("kind", "memo")
            .with_metadata_text("observed_at", "2026-04-02"),
        ],
        vec![],
    );

    NavigationPipeline::new(
        ontology_store,
        graph_store,
        RoamingPlan::new(vec!["revenue"]).with_max_concepts(4),
    )
}

// 2026-04-09 CST: 这里构造“同词不同域”的最小 pipeline 样本，原因是 foundation 主线要验证的不是某个业务词典，
// 而是通用本体标签约束能否在重名概念间稳定做出正确收敛。
// 目的：用纯内存 fixture 验证 finance/ui 两域共享 alias 时，pipeline 会跟随 request tags 选择正确域。
fn sample_tag_constrained_navigation_pipeline() -> NavigationPipeline {
    let ontology_store = OntologyStore::new(
        OntologySchema::new(
            vec![
                OntologyConcept::new("gross_margin", "GrossMargin")
                    .with_alias("margin")
                    .with_tag("finance"),
                OntologyConcept::new("layout_margin", "LayoutMargin")
                    .with_alias("margin")
                    .with_tag("ui"),
                OntologyConcept::new("trend", "Trend"),
            ],
            vec![],
        )
        .expect("tag-constrained schema should be valid"),
    );

    let graph_store = KnowledgeGraphStore::new(
        vec![
            KnowledgeNode::new(
                "node-gross-margin-1",
                "Gross Margin Trend",
                "Gross margin trend analysis for finance reporting.",
            )
            .with_concept_id("gross_margin")
            .with_evidence_ref(EvidenceRef::new("sheet:finance", "B2:C20")),
            KnowledgeNode::new(
                "node-layout-margin-1",
                "Layout Margin Guide",
                "Layout margin guidance for UI spacing systems.",
            )
            .with_concept_id("layout_margin")
            .with_evidence_ref(EvidenceRef::new("doc:ui", "section-4")),
        ],
        vec![],
    );

    NavigationPipeline::new(
        ontology_store,
        graph_store,
        RoamingPlan::new(vec!["gross_margin", "layout_margin"]).with_max_concepts(4),
    )
}

// 2026-04-09 CST: 这里构造“模板默认种子指向 UI，但请求标签要求 finance”的冲突样本，原因是需要确认
// pipeline 不会因为模板默认种子偏向某域，就压掉请求显式声明的更强 scope。
// 目的：把标签优先于模板默认种子的边界用最小 fixture 固定下来，避免 future refactor 改坏主链语义。
fn sample_conflicting_seed_navigation_pipeline() -> NavigationPipeline {
    let ontology_store = OntologyStore::new(
        OntologySchema::new(
            vec![
                OntologyConcept::new("gross_margin", "GrossMargin")
                    .with_alias("margin")
                    .with_tag("finance"),
                OntologyConcept::new("layout_margin", "LayoutMargin")
                    .with_alias("margin")
                    .with_tag("ui"),
                OntologyConcept::new("trend", "Trend"),
            ],
            vec![],
        )
        .expect("conflicting-seed schema should be valid"),
    );

    let graph_store = KnowledgeGraphStore::new(
        vec![
            KnowledgeNode::new(
                "node-gross-margin-1",
                "Gross Margin Trend",
                "Gross margin trend analysis for finance reporting.",
            )
            .with_concept_id("gross_margin")
            .with_evidence_ref(EvidenceRef::new("sheet:finance", "B2:C20")),
            KnowledgeNode::new(
                "node-layout-margin-1",
                "Layout Margin Guide",
                "Layout margin guidance for UI spacing systems.",
            )
            .with_concept_id("layout_margin")
            .with_evidence_ref(EvidenceRef::new("doc:ui", "section-4")),
        ],
        vec![],
    );

    NavigationPipeline::new(
        ontology_store,
        graph_store,
        RoamingPlan::new(vec!["layout_margin"]).with_max_concepts(4),
    )
}

// 2026-04-09 CST: 这里构造“路由能命中，但漫游种子在 store 中不存在”的样本，原因是要把 pipeline 对 roaming
// 阶段失败的错误映射钉在集成层，而不是只依赖单点实现推断。
// 目的：验证当 route 输出 concept 与模板默认种子不一致、且模板收敛后保留下来的种子不存在于 store 时，pipeline 返回 RoamFailed。
fn sample_missing_seed_navigation_pipeline() -> NavigationPipeline {
    let ontology_store = OntologyStore::new(
        OntologySchema::new(
            vec![OntologyConcept::new("gross_margin", "GrossMargin").with_alias("margin")],
            vec![],
        )
        .expect("missing-seed schema should be valid"),
    );

    let graph_store = KnowledgeGraphStore::new(vec![], vec![]);

    NavigationPipeline::new(
        ontology_store,
        graph_store,
        RoamingPlan::new(vec!["ghost_seed"]).with_max_concepts(4),
    )
}

// 2026-04-09 CST: 这里构造“路由与漫游都成功，但检索找不到证据”的样本，原因是 foundation 主线需要把空命中
// 明确表达为 retrieval failed，而不是让上层自己猜测是图谱空、scope 空还是路由出错。
// 目的：用最小内存 fixture 固定 retrieval miss 的集成失败语义。
fn sample_retrieval_miss_navigation_pipeline() -> NavigationPipeline {
    let ontology_store = OntologyStore::new(
        OntologySchema::new(
            vec![OntologyConcept::new("gross_margin", "GrossMargin").with_alias("margin")],
            vec![],
        )
        .expect("retrieval-miss schema should be valid"),
    );

    let graph_store = KnowledgeGraphStore::new(
        vec![KnowledgeNode::new(
            "node-gross-margin-1",
            "Archive Snapshot",
            "Legacy baseline figures for prior periods only.",
        )
        .with_concept_id("gross_margin")
        .with_evidence_ref(EvidenceRef::new("sheet:archive", "D1:E4"))],
        vec![],
    );

    NavigationPipeline::new(
        ontology_store,
        graph_store,
        RoamingPlan::new(vec!["gross_margin"]).with_max_concepts(4),
    )
}

// 2026-04-09 CST: 这里构造“route 命中 revenue + gross_margin，但模板只允许 gross_margin + ghost_seed”的部分重叠样本，
// 原因是需要验证模板默认种子与 route 部分重叠时，pipeline 会取交集而不是直接退回模板全量。
// 目的：固定方案 C 的中间分支语义，即“有交集就取交集，无交集才保留模板默认种子”。
// 2026-04-09 CST: 这里构造“模板默认种子冲突，但显式标签仍保留 finance route，随后因为缺少证据而 retrieval failed”的样本，
// 原因是需要把“标签优先于冲突模板”的规则继续压测到失败路径，而不只是成功命中路径。
// 目的：验证在模板默认种子被显式 tag 压住后，主链仍围绕正确 route 运行，并把真正的失败阶段稳定暴露为 RetrievalFailed。
fn sample_conflicting_seed_retrieval_miss_navigation_pipeline() -> NavigationPipeline {
    let ontology_store = OntologyStore::new(
        OntologySchema::new(
            vec![
                OntologyConcept::new("gross_margin", "GrossMargin")
                    .with_alias("margin")
                    .with_tag("finance"),
                OntologyConcept::new("layout_margin", "LayoutMargin")
                    .with_alias("margin")
                    .with_tag("ui"),
                OntologyConcept::new("trend", "Trend"),
            ],
            vec![],
        )
        .expect("conflicting-seed retrieval-miss schema should be valid"),
    );

    let graph_store = KnowledgeGraphStore::new(
        vec![KnowledgeNode::new(
            "node-gross-margin-1",
            "Archive Snapshot",
            "Legacy baseline figures for prior periods only.",
        )
        .with_concept_id("gross_margin")
        .with_evidence_ref(EvidenceRef::new("sheet:archive", "D1:E4"))],
        vec![],
    );

    NavigationPipeline::new(
        ontology_store,
        graph_store,
        RoamingPlan::new(vec!["layout_margin"]).with_max_concepts(4),
    )
}

fn sample_partial_overlap_navigation_pipeline() -> NavigationPipeline {
    let ontology_store = OntologyStore::new(
        OntologySchema::new(
            vec![
                OntologyConcept::new("revenue", "Revenue").with_alias("revenue"),
                OntologyConcept::new("gross_margin", "GrossMargin").with_alias("margin"),
                OntologyConcept::new("trend", "Trend"),
            ],
            vec![],
        )
        .expect("partial-overlap schema should be valid"),
    );

    let graph_store = KnowledgeGraphStore::new(
        vec![
            KnowledgeNode::new(
                "node-revenue-1",
                "Revenue Overview",
                "Revenue overview for finance reporting.",
            )
            .with_concept_id("revenue")
            .with_evidence_ref(EvidenceRef::new("sheet:revenue", "A1:B12")),
            KnowledgeNode::new(
                "node-gross-margin-1",
                "Gross Margin Trend",
                "Gross margin trend analysis for finance reporting.",
            )
            .with_concept_id("gross_margin")
            .with_evidence_ref(EvidenceRef::new("sheet:finance", "B2:C20")),
        ],
        vec![],
    );

    NavigationPipeline::new(
        ontology_store,
        graph_store,
        RoamingPlan::new(vec!["gross_margin", "ghost_seed"]).with_max_concepts(4),
    )
}

// 2026-04-09 CST: 这里构造“模板默认种子与 route 有两个交集概念”的样本，原因是需要验证 pipeline 在多交集场景下
// 的保留顺序和漫游起点都稳定可预测，而不是依赖 BTreeMap 或 fixture 偶然顺序。
// 目的：钉死多个交集 seed 时按 route 命中顺序保留，并且 roaming 只从这些最终保留的 seeds 扩展。
fn sample_multi_overlap_navigation_pipeline() -> NavigationPipeline {
    let ontology_store = OntologyStore::new(
        OntologySchema::new(
            vec![
                OntologyConcept::new("revenue", "Revenue").with_alias("revenue"),
                OntologyConcept::new("gross_margin", "GrossMargin").with_alias("margin"),
                OntologyConcept::new("finance_report", "FinanceReport"),
                OntologyConcept::new("invoice", "Invoice"),
                OntologyConcept::new("trend", "Trend"),
            ],
            vec![
                OntologyRelation {
                    from_concept_id: "gross_margin".to_string(),
                    to_concept_id: "finance_report".to_string(),
                    relation_type: OntologyRelationType::DependsOn,
                },
                OntologyRelation {
                    from_concept_id: "revenue".to_string(),
                    to_concept_id: "invoice".to_string(),
                    relation_type: OntologyRelationType::DependsOn,
                },
            ],
        )
        .expect("multi-overlap schema should be valid"),
    );

    let graph_store = KnowledgeGraphStore::new(
        vec![
            KnowledgeNode::new(
                "node-gross-margin-1",
                "Gross Margin Trend",
                "Gross margin trend analysis for finance reporting.",
            )
            .with_concept_id("gross_margin")
            .with_evidence_ref(EvidenceRef::new("sheet:finance", "B2:C20")),
            KnowledgeNode::new(
                "node-revenue-1",
                "Revenue Trend",
                "Revenue trend analysis sourced from invoices.",
            )
            .with_concept_id("revenue")
            .with_evidence_ref(EvidenceRef::new("sheet:revenue", "A1:B12")),
        ],
        vec![],
    );

    NavigationPipeline::new(
        ontology_store,
        graph_store,
        RoamingPlan::new(vec!["ghost_seed", "gross_margin", "revenue"])
            .with_allowed_relation_types(vec![OntologyRelationType::DependsOn])
            .with_max_depth(1)
            .with_max_concepts(6),
    )
}

// 2026-04-09 CST: 这里构造“多交集 seeds + 单标签约束”的最小样本，原因是需要把同词异域候选、模板默认 seeds 和 route 命中顺序
// 同时放进一个 fixture 中，才能验证复合规则在 pipeline 主链里的真实叠加结果。
// 目的：让测试直接覆盖 finance 标签筛掉 UI 候选后，仍按 route 顺序保留 gross_margin 与 revenue 两个交集 seeds。
fn sample_tag_filtered_multi_overlap_navigation_pipeline() -> NavigationPipeline {
    let ontology_store = OntologyStore::new(
        OntologySchema::new(
            vec![
                OntologyConcept::new("gross_margin", "GrossMargin")
                    .with_alias("margin")
                    .with_tag("finance"),
                OntologyConcept::new("layout_margin", "LayoutMargin")
                    .with_alias("margin")
                    .with_tag("ui"),
                OntologyConcept::new("revenue", "Revenue")
                    .with_alias("revenue")
                    .with_tag("finance"),
                OntologyConcept::new("finance_report", "FinanceReport"),
                OntologyConcept::new("invoice", "Invoice"),
                OntologyConcept::new("trend", "Trend"),
            ],
            vec![
                OntologyRelation {
                    from_concept_id: "gross_margin".to_string(),
                    to_concept_id: "finance_report".to_string(),
                    relation_type: OntologyRelationType::DependsOn,
                },
                OntologyRelation {
                    from_concept_id: "revenue".to_string(),
                    to_concept_id: "invoice".to_string(),
                    relation_type: OntologyRelationType::DependsOn,
                },
            ],
        )
        .expect("tag-filtered multi-overlap schema should be valid"),
    );

    let graph_store = KnowledgeGraphStore::new(
        vec![
            KnowledgeNode::new(
                "node-gross-margin-1",
                "Gross Margin Trend",
                "Gross margin trend analysis for finance reporting.",
            )
            .with_concept_id("gross_margin")
            .with_evidence_ref(EvidenceRef::new("sheet:finance", "B2:C20")),
            KnowledgeNode::new(
                "node-revenue-1",
                "Revenue Trend",
                "Revenue trend analysis sourced from invoices.",
            )
            .with_concept_id("revenue")
            .with_evidence_ref(EvidenceRef::new("sheet:revenue", "A1:B12")),
            KnowledgeNode::new(
                "node-layout-margin-1",
                "Layout Margin Guide",
                "Layout margin guidance for UI spacing systems.",
            )
            .with_concept_id("layout_margin")
            .with_evidence_ref(EvidenceRef::new("doc:ui", "section-4")),
        ],
        vec![],
    );

    NavigationPipeline::new(
        ontology_store,
        graph_store,
        RoamingPlan::new(vec!["ghost_seed", "gross_margin", "revenue", "layout_margin"])
            .with_allowed_relation_types(vec![OntologyRelationType::DependsOn])
            .with_max_depth(1)
            .with_max_concepts(6),
    )
}

// 2026-04-09 CST: 这里构造“多交集 seeds + 多标签约束”的最小样本，原因是要验证 required_concept_tags 传入多个标签时，
// route 与模板收敛逻辑不会丢掉分别命中不同标签的通用概念。
// 目的：用 revenue(core) 与 gross_margin(finance) 的组合，固定多标签输入下的 OR 过滤 + 交集保留行为。
fn sample_multi_tag_filtered_overlap_navigation_pipeline() -> NavigationPipeline {
    let ontology_store = OntologyStore::new(
        OntologySchema::new(
            vec![
                OntologyConcept::new("revenue", "Revenue")
                    .with_alias("revenue")
                    .with_tag("core"),
                OntologyConcept::new("ui_revenue", "UIRevenue")
                    .with_alias("revenue")
                    .with_tag("ui"),
                OntologyConcept::new("gross_margin", "GrossMargin")
                    .with_alias("margin")
                    .with_tag("finance"),
                OntologyConcept::new("layout_margin", "LayoutMargin")
                    .with_alias("margin")
                    .with_tag("ui"),
                OntologyConcept::new("invoice", "Invoice"),
                OntologyConcept::new("finance_report", "FinanceReport"),
                OntologyConcept::new("trend", "Trend"),
            ],
            vec![
                OntologyRelation {
                    from_concept_id: "revenue".to_string(),
                    to_concept_id: "invoice".to_string(),
                    relation_type: OntologyRelationType::DependsOn,
                },
                OntologyRelation {
                    from_concept_id: "gross_margin".to_string(),
                    to_concept_id: "finance_report".to_string(),
                    relation_type: OntologyRelationType::DependsOn,
                },
            ],
        )
        .expect("multi-tag filtered overlap schema should be valid"),
    );

    let graph_store = KnowledgeGraphStore::new(
        vec![
            KnowledgeNode::new(
                "node-revenue-1",
                "Revenue Trend",
                "Revenue trend analysis sourced from invoices.",
            )
            .with_concept_id("revenue")
            .with_evidence_ref(EvidenceRef::new("sheet:revenue", "A1:B12")),
            KnowledgeNode::new(
                "node-gross-margin-1",
                "Gross Margin Trend",
                "Gross margin trend analysis for finance reporting.",
            )
            .with_concept_id("gross_margin")
            .with_evidence_ref(EvidenceRef::new("sheet:finance", "B2:C20")),
            KnowledgeNode::new(
                "node-ui-revenue-1",
                "UI Revenue Copy",
                "Revenue wording guide for user interface labels.",
            )
            .with_concept_id("ui_revenue")
            .with_evidence_ref(EvidenceRef::new("doc:ui", "section-8")),
        ],
        vec![],
    );

    NavigationPipeline::new(
        ontology_store,
        graph_store,
        RoamingPlan::new(vec!["gross_margin", "revenue", "ghost_seed", "ui_revenue"])
            .with_allowed_relation_types(vec![OntologyRelationType::DependsOn])
            .with_max_depth(1)
            .with_max_concepts(6),
    )
}

// 2026-04-09 CST: 这里构造“concept metadata 与 node metadata 使用同名字段”的通用导航样本，原因是当前阶段要验证的是
// metadata-aware scope 能先收窄 concept，再让 retrieval 复用同一组标准字段做节点过滤。
// 目的：用一套纯 foundation fixture 同时覆盖 `In` 与 `HasAny` 在 route -> roam -> retrieve 主线里的叠加行为。
fn sample_concept_metadata_navigation_pipeline() -> NavigationPipeline {
    let ontology_store = OntologyStore::new(
        OntologySchema::new(
            vec![
                OntologyConcept::new("gross_margin", "GrossMargin")
                    .with_alias("margin")
                    .with_metadata_text("domain", "finance")
                    .with_metadata_values("channels", vec!["analytics"]),
                OntologyConcept::new("layout_margin", "LayoutMargin")
                    .with_alias("margin")
                    .with_metadata_text("domain", "ui")
                    .with_metadata_values("channels", vec!["design"]),
                OntologyConcept::new("revenue", "Revenue")
                    .with_alias("revenue")
                    .with_metadata_text("domain", "finance")
                    .with_metadata_values("channels", vec!["core", "analytics"]),
                OntologyConcept::new("finance_report", "FinanceReport")
                    .with_metadata_text("domain", "finance")
                    .with_metadata_values("channels", vec!["analytics"]),
                OntologyConcept::new("invoice", "Invoice")
                    .with_metadata_text("domain", "finance")
                    .with_metadata_values("channels", vec!["core"]),
                OntologyConcept::new("trend", "Trend"),
            ],
            vec![
                OntologyRelation {
                    from_concept_id: "gross_margin".to_string(),
                    to_concept_id: "finance_report".to_string(),
                    relation_type: OntologyRelationType::DependsOn,
                },
                OntologyRelation {
                    from_concept_id: "revenue".to_string(),
                    to_concept_id: "invoice".to_string(),
                    relation_type: OntologyRelationType::DependsOn,
                },
                OntologyRelation {
                    from_concept_id: "layout_margin".to_string(),
                    to_concept_id: "trend".to_string(),
                    relation_type: OntologyRelationType::DependsOn,
                },
            ],
        )
        .expect("concept metadata schema should be valid"),
    );

    let graph_store = KnowledgeGraphStore::new(
        vec![
            KnowledgeNode::new(
                "node-gross-margin-1",
                "Gross Margin Trend",
                "Gross margin trend analysis for finance reporting.",
            )
            .with_concept_id("gross_margin")
            .with_evidence_ref(EvidenceRef::new("sheet:finance", "B2:C20"))
            .with_metadata_text("domain", "finance")
            .with_metadata_values("channels", vec!["analytics"]),
            KnowledgeNode::new(
                "node-layout-margin-1",
                "Layout Margin Guide",
                "Layout margin guidance for spacing systems.",
            )
            .with_concept_id("layout_margin")
            .with_evidence_ref(EvidenceRef::new("doc:ui", "section-4"))
            .with_metadata_text("domain", "ui")
            .with_metadata_values("channels", vec!["design"]),
            KnowledgeNode::new(
                "node-revenue-1",
                "Revenue Trend",
                "Revenue trend analysis sourced from invoices.",
            )
            .with_concept_id("revenue")
            .with_evidence_ref(EvidenceRef::new("sheet:revenue", "A1:B12"))
            .with_metadata_text("domain", "finance")
            .with_metadata_values("channels", vec!["core", "analytics"]),
        ],
        vec![],
    );

    NavigationPipeline::new(
        ontology_store,
        graph_store,
        RoamingPlan::new(vec!["gross_margin", "layout_margin", "revenue"])
            .with_allowed_relation_types(vec![OntologyRelationType::DependsOn])
            .with_max_depth(1)
            .with_max_concepts(6),
    )
}

// 2026-04-09 CST: 这里构造“concept-only metadata 通过 registry 显式注册”的最小主线样本，原因是字段目录阶段需要验证
// concept 侧字段只在 concept 收敛生效，而不会污染 retrieval 对 node metadata 的过滤。
// 目的：用纯 foundation fixture 钉死 namespace 这类 concept-only 字段的目标层级语义。
fn sample_concept_only_metadata_registry_navigation_pipeline() -> NavigationPipeline {
    let ontology_store = OntologyStore::new(
        OntologySchema::new(
            vec![
                OntologyConcept::new("gross_margin", "GrossMargin")
                    .with_alias("margin")
                    .with_metadata_text("namespace", "finance"),
                OntologyConcept::new("layout_margin", "LayoutMargin")
                    .with_alias("margin")
                    .with_metadata_text("namespace", "ui"),
            ],
            vec![],
        )
        .expect("concept-only registry schema should be valid"),
    );

    let graph_store = KnowledgeGraphStore::new(
        vec![
            KnowledgeNode::new(
                "node-gross-margin-1",
                "Gross Margin Trend",
                "Gross margin trend analysis for finance reporting.",
            )
            .with_concept_id("gross_margin")
            .with_evidence_ref(EvidenceRef::new("sheet:finance", "B2:C20")),
            KnowledgeNode::new(
                "node-layout-margin-1",
                "Layout Margin Guide",
                "Layout margin guidance for spacing systems.",
            )
            .with_concept_id("layout_margin")
            .with_evidence_ref(EvidenceRef::new("doc:ui", "section-4")),
        ],
        vec![],
    );

    let metadata_registry = MetadataRegistry::new().register_text_field(
        "namespace",
        vec![MetadataFieldTarget::Concept],
        vec![
            MetadataConstraintOperator::Equals,
            MetadataConstraintOperator::In,
        ],
    );

    NavigationPipeline::new_with_metadata_registry(
        ontology_store,
        graph_store,
        RoamingPlan::new(vec!["gross_margin", "layout_margin"]).with_max_concepts(4),
        metadata_registry,
    )
}

// 2026-04-09 CST: 这里构造 node-only metadata 通过 registry 显式注册的主线样本，原因是要验证 node target 的 operator 错误会在 retrieval 阶段暴露，
// 而不是在 concept 收敛阶段被错误处理掉。
// 目的：提供一个纯 foundation 的最小样本，让 pipeline 可以验证 target-aware metadata 校验的阶段归属。
fn sample_node_only_metadata_registry_navigation_pipeline() -> NavigationPipeline {
    let ontology_store = OntologyStore::new(
        OntologySchema::new(
            vec![OntologyConcept::new("revenue", "Revenue").with_alias("revenue")],
            vec![],
        )
        .expect("node-only registry schema should be valid"),
    );

    let graph_store = KnowledgeGraphStore::new(
        vec![KnowledgeNode::new(
            "node-revenue-1",
            "Revenue Trend",
            "Revenue trend analysis sourced from invoices.",
        )
        .with_concept_id("revenue")
        .with_evidence_ref(EvidenceRef::new("sheet:revenue", "A1:B12"))
        .with_metadata_text("source", "sheet:revenue")],
        vec![],
    );

    let metadata_registry = MetadataRegistry::new().register_text_field(
        "source",
        vec![MetadataFieldTarget::Node],
        vec![MetadataConstraintOperator::Equals],
    );

    NavigationPipeline::new_with_metadata_registry(
        ontology_store,
        graph_store,
        RoamingPlan::new(vec!["revenue"]).with_max_concepts(4),
        metadata_registry,
    )
}

#[allow(dead_code)]
fn _pipeline_result_guard(result: NavigationEvidence) -> NavigationEvidence {
    result
}
