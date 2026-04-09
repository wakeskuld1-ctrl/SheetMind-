use excel_skill::ops::foundation::knowledge_graph_store::KnowledgeGraphStore;
use excel_skill::ops::foundation::knowledge_record::{EvidenceRef, KnowledgeEdge, KnowledgeNode};
use excel_skill::ops::foundation::metadata_constraint::{MetadataConstraint, MetadataScope};
use excel_skill::ops::foundation::metadata_registry::{
    MetadataConstraintOperator, MetadataFieldTarget, MetadataRegistry, MetadataRegistryError,
};
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
                metadata_scope: MetadataScope::new(),
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
                metadata_scope: MetadataScope::new(),
            },
            &sample_graph_store(),
        )
        .expect("hits should be ranked");

    // 2026-04-09 CST: 这里把断言从“固定两条命中”收敛为“验证前两名稳定排序”，原因是 metadata/range 边界样本引入后，
    // 同一 scope 内允许出现额外合法低分命中；旧断言若继续钉死长度，会把夹具扩充误报成排序回归。
    // 目的：继续覆盖 retrieval 的核心契约，即高分结果稳定排在前面，而不是把测试绑定到样本数量细节。
    assert!(hits.len() >= 2);
    assert_eq!(hits[0].node_id, "node-trend-1");
    // 2026-04-09 CST: 这里不再钉死第二名必须是 `node-revenue-1`，原因是 trend 概念下现在有第二个合法高分样本，
    // 它会自然排在 revenue 样本之前；继续强绑第二名会把夹具丰富度误判成检索回归。
    // 目的：只验证排序契约本身，以及 revenue 节点仍能在混合 scope 中被稳定召回。
    assert!(hits.windows(2).all(|pair| pair[0].score >= pair[1].score));
    assert!(hits.iter().any(|hit| hit.node_id == "node-revenue-1"));
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
                metadata_scope: MetadataScope::new(),
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

// 2026-04-09 CST: 这里补 retrieval 层的单值元数据过滤测试，原因是通用 MetadataConstraint 第一阶段先落到
// NavigationRequest -> Retrieval 主链，如果 retrieval 本身不能稳定按 metadata 收窄节点，主线就只是“带配置但不生效”。
// 目的：先钉死 `Equals` 约束会在评分前过滤候选节点，而不是在命中后再做结果裁剪。
#[test]
fn retrieval_engine_filters_hits_by_scope_metadata_constraint() {
    let hits = sample_retrieval_engine()
        .retrieve(
            "sales trend",
            &CandidateScope {
                concept_ids: vec!["revenue".to_string()],
                path: Vec::new(),
                metadata_scope: MetadataScope::from_constraints(vec![MetadataConstraint::equals(
                    "source",
                    "sheet:sales",
                )]),
            },
            &sample_graph_store(),
        )
        .expect("metadata-filtered hits should exist");

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].node_id, "node-revenue-1");
}

// 2026-04-09 CST: 这里补 retrieval 层的范围元数据过滤测试，原因是 metadata 不能只停留在 source/type 等离散字段，
// 否则后续时间窗、版本窗等标准能力还得重新造一套接口。
// 目的：先用 ISO 日期字符串验证 `Range` 约束在当前标准能力里可用，给后续 time-range 扩展留下稳定入口。
#[test]
fn retrieval_engine_filters_hits_by_range_metadata_constraint() {
    let hits = sample_retrieval_engine()
        .retrieve(
            "trend month",
            &CandidateScope {
                concept_ids: vec!["trend".to_string()],
                path: Vec::new(),
                metadata_scope: MetadataScope::from_constraints(vec![MetadataConstraint::range(
                    "observed_at",
                    Some("2026-02-01"),
                    Some("2026-12-31"),
                )]),
            },
            &sample_graph_store(),
        )
        .expect("range-filtered hits should exist");

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].node_id, "node-trend-2");
}

// 2026-04-09 CST: 这里补 retrieval 层“metadata 把全部候选过滤掉”的失败测试，原因是 foundation 需要明确区分
// “scope 内有概念”与“scope 内最终有证据命中”这两个阶段，metadata 过滤不能把失败语义搞模糊。
// 目的：钉死 MetadataConstraint 过滤后若无命中，仍然走统一 `NoEvidenceFound` 契约。
#[test]
fn retrieval_engine_returns_error_when_metadata_constraints_filter_all_hits() {
    let error = sample_retrieval_engine()
        .retrieve(
            "sales trend",
            &CandidateScope {
                concept_ids: vec!["revenue".to_string()],
                path: Vec::new(),
                metadata_scope: MetadataScope::from_constraints(vec![MetadataConstraint::equals(
                    "source",
                    "doc:notes",
                )]),
            },
            &sample_graph_store(),
        )
        .expect_err("retrieve should fail when metadata constraints filter all hits");

    assert_eq!(
        error,
        RetrievalEngineError::NoEvidenceFound {
            question: "sales trend".to_string(),
        }
    );
}

// 2026-04-09 CST: 这里补 registry 模式下 concept-only 字段不应污染 retrieval 的执行器单测，原因是上一轮只在 pipeline 集成层钉住了该契约，
// 但 retrieval 自身还没有一个更近的失败/成功边界来证明 `retrieve_with_metadata_registry(...)` 只消费 node-target 字段。
// 目的：把 concept-only metadata 在 retrieval 侧被忽略的语义直接钉死在执行器层，降低后续 pipeline 夹具变化带来的误判成本。
#[test]
fn retrieval_engine_with_metadata_registry_ignores_concept_only_constraints() {
    let metadata_registry = MetadataRegistry::new().register_text_field(
        "namespace",
        vec![MetadataFieldTarget::Concept],
        vec![MetadataConstraintOperator::Equals],
    );
    let hits = sample_retrieval_engine()
        .retrieve_with_metadata_registry(
            "sales trend",
            &CandidateScope {
                concept_ids: vec!["revenue".to_string()],
                path: Vec::new(),
                metadata_scope: MetadataScope::from_constraints(vec![MetadataConstraint::equals(
                    "namespace",
                    "finance",
                )]),
            },
            &sample_graph_store(),
            &metadata_registry,
        )
        .expect("concept-only metadata should be ignored by retrieval");

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].node_id, "node-revenue-1");
}

// 2026-04-09 CST: 这里补 registry 模式下未注册字段的红测，原因是 metadata 字段目录已经是 foundation 标准能力，
// 如果 registry 已启用却仍允许未知字段静默滑过，就会让“字段治理缺失”伪装成“没有命中证据”。
// 目的：把“未注册字段必须显式报错”固定成 retrieval 的正式错误合同，而不是继续走静默忽略路径。
#[test]
fn retrieval_engine_with_metadata_registry_returns_error_for_unregistered_field() {
    let metadata_registry = MetadataRegistry::new().register_text_field(
        "source",
        vec![MetadataFieldTarget::Node],
        vec![MetadataConstraintOperator::Equals],
    );
    let error = sample_retrieval_engine()
        .retrieve_with_metadata_registry(
            "sales trend",
            &CandidateScope {
                concept_ids: vec!["revenue".to_string()],
                path: Vec::new(),
                metadata_scope: MetadataScope::from_constraints(vec![MetadataConstraint::equals(
                    "namespace",
                    "finance",
                )]),
            },
            &sample_graph_store(),
            &metadata_registry,
        )
        .expect_err("unregistered metadata field should be rejected in registry mode");

    assert_eq!(
        error,
        RetrievalEngineError::InvalidMetadataConstraint(
            MetadataRegistryError::UnregisteredField {
                field: "namespace".to_string(),
            }
        )
    );
}

// 2026-04-09 CST: 这里补 registry 模式下不支持 operator 的红测，原因是 registry 现在已经能声明 operator 合同，
// 如果执行器仍把不支持的 operator 静默忽略，就等于字段目录只管声明、不管执行。
// 目的：确保 node-target 字段一旦被当前 target 消费，就必须同时遵守 operator 支持矩阵。
#[test]
fn retrieval_engine_with_metadata_registry_returns_error_for_unsupported_operator() {
    let metadata_registry = MetadataRegistry::new().register_text_field(
        "source",
        vec![MetadataFieldTarget::Node],
        vec![MetadataConstraintOperator::Equals],
    );
    let error = sample_retrieval_engine()
        .retrieve_with_metadata_registry(
            "sales trend",
            &CandidateScope {
                concept_ids: vec!["revenue".to_string()],
                path: Vec::new(),
                metadata_scope: MetadataScope::from_constraints(vec![MetadataConstraint::has_any(
                    "source",
                    vec!["sheet:sales"],
                )]),
            },
            &sample_graph_store(),
            &metadata_registry,
        )
        .expect_err("unsupported operator should be rejected in registry mode");

    assert_eq!(
        error,
        RetrievalEngineError::InvalidMetadataConstraint(
            MetadataRegistryError::UnsupportedOperator {
                field: "source".to_string(),
                operator: MetadataConstraintOperator::HasAny,
                target: MetadataFieldTarget::Node,
            }
        )
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
        .with_evidence_ref(EvidenceRef::new("sheet:sales", "A1:B12"))
        .with_metadata_text("source", "sheet:sales")
        .with_metadata_text("kind", "table")
        .with_metadata_text("observed_at", "2026-01-15"),
        KnowledgeNode::new(
            "node-revenue-2",
            // 2026-04-09 CST: 这里把 metadata 辅助样本改成非 query 命中文本，原因是它的职责是提供 `doc:notes` 元数据分支，
            // 不是干扰既有的 scoped retrieval 排序与“metadata 全过滤”边界。
            // 目的：让新增 metadata 节点只服务约束存在性验证，不改变原有 query 的主命中集合。
            "Management Commentary",
            "Analyst notes summarize leadership guidance and profitability context.",
        )
        .with_concept_id("revenue")
        .with_evidence_ref(EvidenceRef::new("doc:notes", "section-2"))
        .with_metadata_text("source", "doc:notes")
        .with_metadata_text("kind", "memo")
        .with_metadata_text("observed_at", "2026-03-10"),
        KnowledgeNode::new(
            "node-trend-1",
            "Trend Month Review",
            "Revenue trend by month highlights trend changes.",
        )
        .with_concept_id("trend")
        .with_evidence_ref(EvidenceRef::new("sheet:trend", "C1:D20"))
        .with_metadata_text("source", "sheet:trend")
        .with_metadata_text("kind", "table")
        .with_metadata_text("observed_at", "2026-01-20"),
        KnowledgeNode::new(
            "node-trend-2",
            "Trend Month Outlook",
            "Trend month outlook compares revenue trend changes for later periods.",
        )
        .with_concept_id("trend")
        .with_evidence_ref(EvidenceRef::new("sheet:trend", "E1:F20"))
        .with_metadata_text("source", "sheet:trend")
        .with_metadata_text("kind", "table")
        .with_metadata_text("observed_at", "2026-03-15"),
        KnowledgeNode::new(
            "node-outside-scope-1",
            "Sales Trend Forecast",
            "Sales trend forecast is strong but belongs to planning only.",
        )
        .with_concept_id("forecast")
        .with_evidence_ref(EvidenceRef::new("sheet:plan", "E1:F8"))
        .with_metadata_text("source", "sheet:plan")
        .with_metadata_text("kind", "forecast")
        .with_metadata_text("observed_at", "2026-04-01"),
    ];
    let edges = vec![KnowledgeEdge::new(
        "node-revenue-1",
        "node-trend-1",
        OntologyRelationType::References,
    )];

    KnowledgeGraphStore::new(nodes, edges)
}
