use excel_skill::ops::foundation::knowledge_graph_store::KnowledgeGraphStore;
use excel_skill::ops::foundation::knowledge_record::{EvidenceRef, KnowledgeEdge, KnowledgeNode};
use excel_skill::ops::foundation::metadata_constraint::MetadataScope;
use excel_skill::ops::foundation::ontology_schema::OntologyRelationType;
use excel_skill::ops::foundation::retrieval_engine::{
    RetrievalDiagnostic, RetrievalEngine, RetrievalEngineError, RetrievalHygieneFlag,
};
use excel_skill::ops::foundation::roaming_engine::{CandidateScope, RoamingStep};

// 2026-04-07 CST: 这里先补 scoped retrieval 的首条失败测试，原因是 Task 7 的第一条约束
// 2026-04-07 CST: 就是 retrieval 只能在 roaming 给出的 CandidateScope 内工作，不能越过主链边界去扫全图。
// 2026-04-07 CST: 目的：先把“只检索候选域”的约束钉住，避免后续实现为了追求命中率把 foundation 架构顺序做乱。
#[test]
fn retrieval_engine_only_scores_nodes_inside_candidate_scope() {
    let hits = sample_retrieval_engine()
        .retrieve(
            "sales trend",
            &CandidateScope {
                concept_ids: vec!["revenue".to_string()],
                path: Vec::new(),
                // 2026-04-09 CST: 这里补空 metadata scope，原因是 CandidateScope 已升级为统一承载
                // roaming/retrieval 合同；本用例不关心 metadata 过滤，只需要显式给出零值。
                // 目的：让旧测试继续覆盖“候选域内检索”语义，而不是因结构升级误报失败。
                metadata_scope: MetadataScope::new(),
            },
            &sample_graph_store(),
        )
        .expect("hits should exist inside scope");

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].node_id, "node-revenue-1");
}

// 2026-04-07 CST: 这里补命中结果排序测试，原因是 retrieval 作为候选域内执行器，
// 2026-04-07 CST: 至少要先提供稳定的“高分在前”输出，后续 evidence assembly 才能消费可预测的 hit 顺序。
// 2026-04-07 CST: 目的：先把最小排序行为固化下来，避免以后因为遍历顺序不同导致上层结果抖动。
#[test]
fn retrieval_engine_returns_hits_in_descending_score_order() {
    let hits = sample_retrieval_engine()
        .retrieve(
            "revenue trend month",
            &CandidateScope {
                concept_ids: vec!["revenue".to_string(), "trend".to_string()],
                path: Vec::new(),
                // 2026-04-09 CST: 这里补空 metadata scope，原因是当前排序测试只验证文本相关性，
                // 不希望把 metadata 约束混入断言语义。
                // 目的：维持测试关注点单一，避免 ranking 回归被结构性变更掩盖。
                metadata_scope: MetadataScope::new(),
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
// 2026-04-07 CST: 应该在这一层显式失败，而不是返回空数组把含义丢给下游 assembler 或更上层去猜。
// 2026-04-07 CST: 目的：先把 Task 7 的最小失败边界稳定下来，便于后续主链明确区分“有候选域”和“有证据命中”。
#[test]
fn retrieval_engine_returns_error_when_scope_has_no_matching_evidence() {
    let error = sample_retrieval_engine()
        .retrieve(
            "cash forecast",
            &CandidateScope {
                concept_ids: vec!["revenue".to_string()],
                path: Vec::new(),
                // 2026-04-09 CST: 这里补空 metadata scope，原因是无命中错误测试依赖的是 scoped
                // evidence 为空，而不是 metadata 拦截。
                // 目的：把失败边界继续固定在 retrieval 本身。
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

// 2026-04-08 CST: 这里补标题优先红灯测试，原因是 Task 11 第一层增强要先证明 retrieval
// 2026-04-08 CST: 不能再把标题命中和正文命中一视同仁，否则高信号标题节点会被同分字典序误排。
// 2026-04-08 CST: 目的：钉住“标题强命中优先于仅正文命中”的最小排序合同。
#[test]
fn retrieval_engine_prefers_title_match_over_body_only_match() {
    let hits = sample_retrieval_engine()
        .retrieve(
            "sales review",
            &sample_candidate_scope_for_revenue_only(),
            &sample_graph_store_for_title_body_ranking(),
        )
        .expect("title/body ranking sample should return hits");

    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].node_id, "node-z-title-match");
    assert_eq!(hits[1].node_id, "node-a-body-match");
}

// 2026-04-08 CST: 这里补短语优先红灯测试，原因是当前 retrieval 只统计 token 交集，
// 2026-04-08 CST: 还无法表达“完整问题短语命中”的更强相关性。
// 2026-04-08 CST: 目的：钉住“完整短语命中优先于分散 token 命中”的最小排序合同。
#[test]
fn retrieval_engine_prefers_exact_phrase_match_over_scattered_tokens() {
    let hits = sample_retrieval_engine()
        .retrieve(
            "sales trend",
            &sample_candidate_scope_for_revenue_only(),
            &sample_graph_store_for_phrase_ranking(),
        )
        .expect("phrase ranking sample should return hits");

    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].node_id, "node-z-phrase-match");
    assert_eq!(hits[1].node_id, "node-a-scattered-match");
}

// 2026-04-08 CST: 这里补 seed concept 优先红灯测试，原因是 foundation 主链的 retrieval
// 2026-04-08 CST: 应该保持“先核心种子概念、后漫游补全概念”的排序倾向，而不是只看纯文本交集。
// 2026-04-08 CST: 目的：钉住“seed concept 节点优先于 roamed concept 节点”的最小排序合同。
#[test]
fn retrieval_engine_prefers_seed_concept_nodes_over_roamed_nodes_when_scores_tie() {
    let hits = sample_retrieval_engine()
        .retrieve(
            "sales trend",
            &sample_candidate_scope_with_roaming_path(),
            &sample_graph_store_for_seed_priority(),
        )
        .expect("seed priority sample should return hits");

    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].node_id, "node-z-seed-revenue");
    assert_eq!(hits[1].node_id, "node-a-roamed-trend");
}

// 2026-04-09 CST: 这里补 primary source 次级优先级红灯测试，原因是 Task 11 第二层要把来源偏好收敛到 source_ref tie-break。
// 2026-04-09 CST: 当前实现只看 score 和 node_id，因此同分时还不能稳定把原始来源排到派生来源前面。
// 2026-04-09 CST: 目的：先钉住“文本同分时 primary source 优先”的最小回归合同，但不让来源优先级参与命中判定。
#[test]
fn retrieval_engine_prefers_primary_source_refs_when_scores_tie() {
    let hits = sample_retrieval_engine()
        .retrieve(
            "sales review",
            &sample_candidate_scope_for_revenue_only(),
            &sample_graph_store_for_source_priority(),
        )
        .expect("source priority sample should return hits");

    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].node_id, "node-z-primary-source");
    assert_eq!(hits[1].node_id, "node-a-derived-source");
}

// 2026-04-09 CST: 这里补 derived source 与 planning source 的红灯测试，原因是用户已经确认第二层来源优先级要分层。
// 2026-04-09 CST: 当两个节点文本分数完全一样时，检索结果不该继续退化成字典序，而应该优先保留“摘要/派生”高于“规划/预测”。
// 2026-04-09 CST: 目的：把 source_ref 的第二层固定排序规则落成单测，后续即便继续增强 retrieval 也不能改坏这个边界。
#[test]
fn retrieval_engine_prefers_derived_sources_over_planning_sources_when_scores_tie() {
    let hits = sample_retrieval_engine()
        .retrieve(
            "sales review",
            &sample_candidate_scope_for_revenue_only(),
            &sample_graph_store_for_derived_vs_planning_source_priority(),
        )
        .expect("derived/planning source priority sample should return hits");

    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].node_id, "node-z-derived-source");
    assert_eq!(hits[1].node_id, "node-a-planning-source");
}

// 2026-04-09 CST: 这里补“来源优先级不能反压文本分数”的保护测试，原因是 source_ref 本轮只允许做 tie-break。
// 2026-04-09 CST: 这个测试主要防止后续有人把来源偏好错误并入主分数，导致文本更相关的节点反而被压到后面。
// 2026-04-09 CST: 目的：把“文本相关性仍然是一等公民”的设计意图固化下来，保持 foundation 检索主线稳定。
#[test]
fn retrieval_engine_keeps_higher_text_score_ahead_of_better_source_priority() {
    let hits = sample_retrieval_engine()
        .retrieve(
            "sales review trend",
            &sample_candidate_scope_for_revenue_only(),
            &sample_graph_store_for_source_priority_not_overriding_text_score(),
        )
        .expect("text score should stay ahead of source priority");

    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].node_id, "node-higher-text-score");
    assert_eq!(hits[1].node_id, "node-better-source-priority");
}

// 2026-04-09 CST: 这里补证据数量优先的红灯测试，原因是方案 C 已确认要把证据侧 tie-break 接在来源优先级之后。
// 2026-04-09 CST: 当前实现在同文本分数、同来源层级下仍会退回 node_id，因此还无法优先返回证据更丰富的节点。
// 2026-04-09 CST: 目的：先钉住“同分同来源时 evidence_refs 更多优先”的最小合同，给后续实现提供稳定红灯。
#[test]
fn retrieval_engine_prefers_more_evidence_refs_when_scores_and_source_priority_tie() {
    let hits = sample_retrieval_engine()
        .retrieve(
            "sales review",
            &sample_candidate_scope_for_revenue_only(),
            &sample_graph_store_for_evidence_count_priority(),
        )
        .expect("evidence count priority sample should return hits");

    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].node_id, "node-z-more-evidence");
    assert_eq!(hits[1].node_id, "node-a-less-evidence");
}

// 2026-04-09 CST: 这里补 locator 精度优先的红灯测试，原因是方案 C 不只看证据数量，还要继续表达“定位更具体更优先”。
// 2026-04-09 CST: 两个节点在文本分数、来源层级和证据数量上都保持一致，只把差异收敛到 locator 精度。
// 2026-04-09 CST: 目的：钉住“单点/小范围定位优先于宽范围定位”的基础排序边界。
#[test]
fn retrieval_engine_prefers_more_specific_locator_when_scores_source_and_counts_tie() {
    let hits = sample_retrieval_engine()
        .retrieve(
            "sales review",
            &sample_candidate_scope_for_revenue_only(),
            &sample_graph_store_for_locator_precision_priority(),
        )
        .expect("locator precision priority sample should return hits");

    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].node_id, "node-z-specific-locator");
    assert_eq!(hits[1].node_id, "node-a-broad-locator");
}

// 2026-04-09 CST: 这里补“source 优先级仍高于证据层”的保护测试，原因是证据数量和 locator 精度都只能排在来源层之后。
// 2026-04-09 CST: 这个测试防止后续有人把证据侧 tie-break 提前到 source_ref 前面，导致 derived 节点靠更多证据反压 primary 节点。
// 2026-04-09 CST: 目的：把 retrieval 当前的层级顺序固定下来，避免 tie-break 链条继续串位。
#[test]
fn retrieval_engine_keeps_better_source_priority_ahead_of_more_evidence_refs() {
    let hits = sample_retrieval_engine()
        .retrieve(
            "sales review",
            &sample_candidate_scope_for_revenue_only(),
            &sample_graph_store_for_source_priority_over_evidence_count(),
        )
        .expect("source priority should stay ahead of evidence count");

    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].node_id, "node-z-better-source");
    assert_eq!(hits[1].node_id, "node-a-more-evidence");
}

// 2026-04-09 CST: 这里先补 diagnostics 顺序对齐的红灯测试，原因是方案 A 的第一条合同不是“再发明一套新排序”，
// 2026-04-09 CST: 而是“把现有排序结果解释清楚”；目的：确保 diagnostics 和最终 hits 使用同一条排序链，不会出现命中列表和解释列表串位。
#[test]
fn retrieval_engine_returns_diagnostics_aligned_with_ranked_hits() {
    let execution = sample_retrieval_engine()
        .retrieve_with_diagnostics(
            "sales review",
            &sample_candidate_scope_for_revenue_only(),
            &sample_graph_store_for_retrieval_diagnostics(),
        )
        .expect("diagnostics sample should return hits and diagnostics");

    let hit_node_ids = execution
        .hits
        .iter()
        .map(|hit| hit.node_id.as_str())
        .collect::<Vec<_>>();
    let diagnostic_node_ids = execution
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.node_id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(hit_node_ids, diagnostic_node_ids);
    assert_eq!(
        hit_node_ids,
        vec!["node-z-diagnostic-primary", "node-a-diagnostic-derived"]
    );
}

// 2026-04-09 CST: 这里先补 diagnostics 信号明细红灯测试，原因是 retrieval 解释能力必须同时覆盖“为什么命中”
// 2026-04-09 CST: 和“为什么排在这里”两层语义；目的：把 title/body/phrase/seed/source/evidence/locator 的最小可解释合同一次钉住。
#[test]
fn retrieval_engine_diagnostics_expose_text_and_tie_break_signals() {
    let execution = sample_retrieval_engine()
        .retrieve_with_diagnostics(
            "sales review",
            &sample_candidate_scope_for_revenue_only(),
            &sample_graph_store_for_retrieval_diagnostics(),
        )
        .expect("diagnostics sample should return hits and diagnostics");

    let diagnostic = execution
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.node_id == "node-z-diagnostic-primary")
        .expect("primary diagnostic should exist");

    assert_eq!(
        diagnostic,
        &RetrievalDiagnostic {
            node_id: "node-z-diagnostic-primary".to_string(),
            matched_title_tokens: vec!["review".to_string(), "sales".to_string()],
            matched_body_tokens: vec!["review".to_string()],
            title_overlap: 2,
            body_overlap: 1,
            phrase_bonus: 4,
            seed_bonus: 2,
            text_score: 11,
            final_score: 13,
            source_priority: 0,
            evidence_ref_count: 2,
            best_locator: Some("A1".to_string()),
            locator_priority: (0, 1),
            duplicate_evidence_ref_count: 0,
            weak_locator_count: 0,
            weak_source_ref_count: 0,
            hygiene_flags: Vec::new(),
        }
    );
}

// 2026-04-09 CST: 这里先补重复证据 hygiene 的红灯测试，原因是 evidence diagnostics 不只要解释“量多”，
// 2026-04-09 CST: 还要告诉后续 AI “这些证据里有没有重复灌水”；目的：钉住同一节点内完全重复 `source_ref + locator` 必须被识别出来。
#[test]
fn retrieval_engine_diagnostics_flag_duplicate_evidence_refs() {
    let execution = sample_retrieval_engine()
        .retrieve_with_diagnostics(
            "sales review",
            &sample_candidate_scope_for_revenue_only(),
            &sample_graph_store_for_duplicate_evidence_hygiene(),
        )
        .expect("duplicate evidence hygiene sample should return diagnostics");

    let diagnostic = execution
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.node_id == "node-duplicate-evidence")
        .expect("duplicate evidence diagnostic should exist");

    assert_eq!(diagnostic.duplicate_evidence_ref_count, 1);
    assert_eq!(diagnostic.weak_locator_count, 0);
    assert_eq!(diagnostic.weak_source_ref_count, 0);
    assert_eq!(
        diagnostic.hygiene_flags,
        vec![RetrievalHygieneFlag::DuplicateEvidenceRefs]
    );
}

// 2026-04-09 CST: 这里先补弱 locator hygiene 的红灯测试，原因是 retrieval diagnostics 现在已经会解释 locator 精度，
// 2026-04-09 CST: 但还不会告诉后续 AI “这个 locator 本身可能太弱”；目的：把空/不可解析/过宽范围这类低质量定位先纳入风险视图。
#[test]
fn retrieval_engine_diagnostics_flag_weak_locator_refs() {
    let execution = sample_retrieval_engine()
        .retrieve_with_diagnostics(
            "sales review",
            &sample_candidate_scope_for_revenue_only(),
            &sample_graph_store_for_weak_locator_hygiene(),
        )
        .expect("weak locator hygiene sample should return diagnostics");

    let diagnostic = execution
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.node_id == "node-weak-locator")
        .expect("weak locator diagnostic should exist");

    assert_eq!(diagnostic.duplicate_evidence_ref_count, 0);
    assert_eq!(diagnostic.weak_locator_count, 1);
    assert_eq!(diagnostic.weak_source_ref_count, 0);
    assert_eq!(
        diagnostic.hygiene_flags,
        vec![RetrievalHygieneFlag::WeakLocator]
    );
}

// 2026-04-09 CST: 这里补 sheet-qualified 单元格 locator 的红灯测试，原因是当前 weak locator 规则不该把常见的 `Sheet1!A1`
// 2026-04-09 CST: 这类 Excel/WPS 定位误判成弱定位；目的：先锁定“带 sheet 前缀的单点定位仍然属于可用 locator”的基础边界。
#[test]
fn retrieval_engine_diagnostics_do_not_flag_sheet_qualified_single_cell_locator_as_weak() {
    let execution = sample_retrieval_engine()
        .retrieve_with_diagnostics(
            "sales review",
            &sample_candidate_scope_for_revenue_only(),
            &sample_graph_store_for_sheet_qualified_locator_hygiene(),
        )
        .expect("sheet-qualified locator sample should return diagnostics");

    let diagnostic = execution
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.node_id == "node-sheet-qualified-locator")
        .expect("sheet-qualified locator diagnostic should exist");

    assert_eq!(diagnostic.duplicate_evidence_ref_count, 0);
    assert_eq!(diagnostic.weak_locator_count, 0);
    assert_eq!(diagnostic.weak_source_ref_count, 0);
    assert_eq!(diagnostic.hygiene_flags, Vec::new());
}

// 2026-04-09 CST: 这里补 sheet-qualified 绝对引用范围 locator 的红灯测试，原因是 `$A$1:$D$5` 这类引用是桌面表格里高频格式，
// 2026-04-09 CST: 如果 diagnostics 继续把它们打成 weak locator，会让 explainability 和真实表格使用场景脱节；目的：锁定小范围绝对引用不应被误报。
#[test]
fn retrieval_engine_diagnostics_do_not_flag_sheet_qualified_absolute_range_locator_as_weak() {
    let execution = sample_retrieval_engine()
        .retrieve_with_diagnostics(
            "sales review",
            &sample_candidate_scope_for_revenue_only(),
            &sample_graph_store_for_sheet_qualified_absolute_range_locator_hygiene(),
        )
        .expect("sheet-qualified absolute range sample should return diagnostics");

    let diagnostic = execution
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.node_id == "node-sheet-qualified-absolute-range")
        .expect("sheet-qualified absolute range diagnostic should exist");

    assert_eq!(diagnostic.duplicate_evidence_ref_count, 0);
    assert_eq!(diagnostic.weak_locator_count, 0);
    assert_eq!(diagnostic.weak_source_ref_count, 0);
    assert_eq!(diagnostic.hygiene_flags, Vec::new());
}

// 2026-04-09 CST: 这里补 sheet-qualified 大范围 locator 的红灯测试，原因是 locator hygiene 不能因为补了 sheet 前缀解析
// 2026-04-09 CST: 就丢掉“大范围仍然算弱”的质量信号；目的：锁定“可解析”与“够不够具体”是两层判断，而不是二选一。
#[test]
fn retrieval_engine_diagnostics_flag_sheet_qualified_large_range_locator_as_weak() {
    let execution = sample_retrieval_engine()
        .retrieve_with_diagnostics(
            "sales review",
            &sample_candidate_scope_for_revenue_only(),
            &sample_graph_store_for_sheet_qualified_large_range_locator_hygiene(),
        )
        .expect("sheet-qualified large range sample should return diagnostics");

    let diagnostic = execution
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.node_id == "node-sheet-qualified-large-range")
        .expect("sheet-qualified large range diagnostic should exist");

    assert_eq!(diagnostic.duplicate_evidence_ref_count, 0);
    assert_eq!(diagnostic.weak_locator_count, 1);
    assert_eq!(diagnostic.weak_source_ref_count, 0);
    assert_eq!(
        diagnostic.hygiene_flags,
        vec![RetrievalHygieneFlag::WeakLocator]
    );
}

// 2026-04-09 CST: 这里补带 Windows 绝对路径前缀的小范围 locator 红灯测试，原因是 `C:\Reports\[Book.xlsx]Sheet!A1:B3`
// 2026-04-09 CST: 这类范围在 drive letter 的 `:` 之前就会被当前最小解析误切开；目的：锁定“绝对路径前缀 + A1 范围”不应被误判为 weak。
#[test]
fn retrieval_engine_diagnostics_do_not_flag_windows_path_external_workbook_range_locator_as_weak() {
    let execution = sample_retrieval_engine()
        .retrieve_with_diagnostics(
            "sales review",
            &sample_candidate_scope_for_revenue_only(),
            &sample_graph_store_for_windows_path_external_workbook_range_locator_hygiene(),
        )
        .expect("windows-path external workbook range sample should return diagnostics");

    let diagnostic = execution
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.node_id == "node-windows-path-external-workbook-range")
        .expect("windows-path external workbook range diagnostic should exist");

    assert_eq!(diagnostic.duplicate_evidence_ref_count, 0);
    assert_eq!(diagnostic.weak_locator_count, 0);
    assert_eq!(diagnostic.weak_source_ref_count, 0);
    assert_eq!(diagnostic.hygiene_flags, Vec::new());
}

// 2026-04-09 CST: 这里补带 Windows 绝对路径前缀的绝对范围 locator 红灯测试，原因是 drive letter 前缀不应破坏当前已支持的
// 2026-04-09 CST: `$A$1:$D$5` 小范围解析；目的：锁定 `C:\Reports\[Book.xlsx]'Sheet'!$A$1:$D$5` 仍然属于非 weak locator。
#[test]
fn retrieval_engine_diagnostics_do_not_flag_windows_path_external_workbook_absolute_range_locator_as_weak() {
    let execution = sample_retrieval_engine()
        .retrieve_with_diagnostics(
            "sales review",
            &sample_candidate_scope_for_revenue_only(),
            &sample_graph_store_for_windows_path_external_workbook_absolute_range_locator_hygiene(
            ),
        )
        .expect("windows-path external workbook absolute range sample should return diagnostics");

    let diagnostic = execution
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.node_id == "node-windows-path-external-workbook-absolute-range")
        .expect("windows-path external workbook absolute range diagnostic should exist");

    assert_eq!(diagnostic.duplicate_evidence_ref_count, 0);
    assert_eq!(diagnostic.weak_locator_count, 0);
    assert_eq!(diagnostic.weak_source_ref_count, 0);
    assert_eq!(diagnostic.hygiene_flags, Vec::new());
}

// 2026-04-09 CST: 这里补带 Windows 绝对路径前缀的大范围 locator 保护测试，原因是上轮只修了 drive letter
// 2026-04-09 CST: 不再误伤小范围 locator，但不能因此把“大范围仍然属于 weak locator”的面积语义一并放松；目的：锁定
// 2026-04-09 CST: “可解析”与“是否足够具体”仍是两层判断，`C:\Reports\[Book.xlsx]Sheet!A1:Z200` 这类定位依然要保留 weak 标记。
#[test]
fn retrieval_engine_diagnostics_still_flag_windows_path_external_workbook_large_range_locator_as_weak() {
    let execution = sample_retrieval_engine()
        .retrieve_with_diagnostics(
            "sales review",
            &sample_candidate_scope_for_revenue_only(),
            &sample_graph_store_for_windows_path_external_workbook_large_range_locator_hygiene(),
        )
        .expect("windows-path external workbook large range sample should return diagnostics");

    let diagnostic = execution
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.node_id == "node-windows-path-external-workbook-large-range")
        .expect("windows-path external workbook large range diagnostic should exist");

    assert_eq!(diagnostic.duplicate_evidence_ref_count, 0);
    assert_eq!(diagnostic.weak_locator_count, 1);
    assert_eq!(diagnostic.weak_source_ref_count, 0);
    assert_eq!(
        diagnostic.hygiene_flags,
        vec![RetrievalHygieneFlag::WeakLocator]
    );
}

// 2026-04-09 CST: 这里补带 Windows 绝对路径前缀的大范围绝对引用 locator 保护测试，原因是 `$A$1:$Z$200`
// 2026-04-09 CST: 这类范围即使现在能被正确解析，也仍然不该被视为足够具体；目的：继续钉住“Windows 路径前缀不会抹掉
// 2026-04-09 CST: large range weak 语义”的边界，避免后续 AI 把上轮修复误读成“路径前缀 locator 一律非 weak”。
#[test]
fn retrieval_engine_diagnostics_still_flag_windows_path_external_workbook_absolute_large_range_locator_as_weak(
) {
    let execution = sample_retrieval_engine()
        .retrieve_with_diagnostics(
            "sales review",
            &sample_candidate_scope_for_revenue_only(),
            &sample_graph_store_for_windows_path_external_workbook_absolute_large_range_locator_hygiene(
            ),
        )
        .expect(
            "windows-path external workbook absolute large range sample should return diagnostics",
        );

    let diagnostic = execution
        .diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic.node_id == "node-windows-path-external-workbook-absolute-large-range"
        })
        .expect("windows-path external workbook absolute large range diagnostic should exist");

    assert_eq!(diagnostic.duplicate_evidence_ref_count, 0);
    assert_eq!(diagnostic.weak_locator_count, 1);
    assert_eq!(diagnostic.weak_source_ref_count, 0);
    assert_eq!(
        diagnostic.hygiene_flags,
        vec![RetrievalHygieneFlag::WeakLocator]
    );
}

// 2026-04-09 CST: 这里补命名区域 locator 的边界测试，原因是本轮只打算最小增强 A1/范围解析，不准备把 named range 也拉进解析器；
// 2026-04-09 CST: 目的：明确当前合同下命名区域仍视为 weak locator，避免后续 AI 误以为这里已经支持完整 Excel locator 语义。
#[test]
fn retrieval_engine_diagnostics_still_flags_named_range_locator_as_weak() {
    let execution = sample_retrieval_engine()
        .retrieve_with_diagnostics(
            "sales review",
            &sample_candidate_scope_for_revenue_only(),
            &sample_graph_store_for_named_range_locator_hygiene(),
        )
        .expect("named range locator sample should return diagnostics");

    let diagnostic = execution
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.node_id == "node-named-range-locator")
        .expect("named range locator diagnostic should exist");

    assert_eq!(diagnostic.duplicate_evidence_ref_count, 0);
    assert_eq!(diagnostic.weak_locator_count, 1);
    assert_eq!(diagnostic.weak_source_ref_count, 0);
    assert_eq!(
        diagnostic.hygiene_flags,
        vec![RetrievalHygieneFlag::WeakLocator]
    );
}

// 2026-04-09 CST: 这里先补弱 source_ref hygiene 的红灯测试，原因是即便文本分数和 locator 都正常，
// 2026-04-09 CST: 证据来源本身也可能过泛或像占位符；目的：让 diagnostics 能把“来源区分度不足”的节点提前标出来。
#[test]
fn retrieval_engine_diagnostics_flag_weak_source_refs() {
    let execution = sample_retrieval_engine()
        .retrieve_with_diagnostics(
            "sales review",
            &sample_candidate_scope_for_revenue_only(),
            &sample_graph_store_for_weak_source_ref_hygiene(),
        )
        .expect("weak source hygiene sample should return diagnostics");

    let diagnostic = execution
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.node_id == "node-weak-source-ref")
        .expect("weak source diagnostic should exist");

    assert_eq!(diagnostic.duplicate_evidence_ref_count, 0);
    assert_eq!(diagnostic.weak_locator_count, 0);
    assert_eq!(diagnostic.weak_source_ref_count, 1);
    assert_eq!(
        diagnostic.hygiene_flags,
        vec![RetrievalHygieneFlag::WeakSourceRef]
    );
}

// 2026-04-09 CST: 这里补多占位词组合 source_ref 的红灯测试，原因是当前 weak source_ref 规则只拦单 token 占位词，
// 2026-04-09 CST: 还拦不住 `source data`、`table file` 这类虽然非单词但仍几乎没有区分度的来源名；目的：把“多 token 但全是占位词”纳入弱来源诊断边界。
#[test]
fn retrieval_engine_diagnostics_flag_multi_token_placeholder_source_refs() {
    let execution = sample_retrieval_engine()
        .retrieve_with_diagnostics(
            "sales review",
            &sample_candidate_scope_for_revenue_only(),
            &sample_graph_store_for_multi_token_placeholder_source_ref_hygiene(),
        )
        .expect("multi-token placeholder source sample should return diagnostics");

    let diagnostic = execution
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.node_id == "node-multi-token-placeholder-source-ref")
        .expect("multi-token placeholder source diagnostic should exist");

    assert_eq!(diagnostic.duplicate_evidence_ref_count, 0);
    assert_eq!(diagnostic.weak_locator_count, 0);
    assert_eq!(diagnostic.weak_source_ref_count, 1);
    assert_eq!(
        diagnostic.hygiene_flags,
        vec![RetrievalHygieneFlag::WeakSourceRef]
    );
}

// 2026-04-09 CST: 这里补占位词加编号 source_ref 的红灯测试，原因是 `table 1`、`sheet 2` 这种名字看起来更长，
// 2026-04-09 CST: 但语义上仍然只是弱区分度占位来源；目的：避免后续 AI 因为带了编号就把这类来源误当成有业务语义。
#[test]
fn retrieval_engine_diagnostics_flag_placeholder_source_refs_with_numeric_suffix() {
    let execution = sample_retrieval_engine()
        .retrieve_with_diagnostics(
            "sales review",
            &sample_candidate_scope_for_revenue_only(),
            &sample_graph_store_for_placeholder_numeric_source_ref_hygiene(),
        )
        .expect("placeholder numeric source sample should return diagnostics");

    let diagnostic = execution
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.node_id == "node-placeholder-numeric-source-ref")
        .expect("placeholder numeric source diagnostic should exist");

    assert_eq!(diagnostic.duplicate_evidence_ref_count, 0);
    assert_eq!(diagnostic.weak_locator_count, 0);
    assert_eq!(diagnostic.weak_source_ref_count, 1);
    assert_eq!(
        diagnostic.hygiene_flags,
        vec![RetrievalHygieneFlag::WeakSourceRef]
    );
}

// 2026-04-09 CST: 这里补“含占位词但仍有业务语义”的保护测试，原因是后续扩弱来源规则时不能把 `sales detail sheet`
// 2026-04-09 CST: 这类真实来源一起误杀；目的：锁定 weak source_ref 仍然只针对低区分度命名，而不是凡是含 `sheet`/`data` 就一律标弱。
#[test]
fn retrieval_engine_diagnostics_do_not_flag_semantic_source_refs_with_placeholder_tokens_as_weak() {
    let execution = sample_retrieval_engine()
        .retrieve_with_diagnostics(
            "sales review",
            &sample_candidate_scope_for_revenue_only(),
            &sample_graph_store_for_semantic_source_ref_with_placeholder_tokens(),
        )
        .expect("semantic source sample should return diagnostics");

    let diagnostic = execution
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.node_id == "node-semantic-source-ref")
        .expect("semantic source diagnostic should exist");

    assert_eq!(diagnostic.duplicate_evidence_ref_count, 0);
    assert_eq!(diagnostic.weak_locator_count, 0);
    assert_eq!(diagnostic.weak_source_ref_count, 0);
    assert_eq!(diagnostic.hygiene_flags, Vec::new());
}

// 2026-04-09 CST: 这里补紧凑编号占位来源的红灯测试，原因是默认命名不一定写成 `sheet 1`，也常直接写成 `sheet1`，
// 2026-04-09 CST: 这种来源虽然没有空格，但语义上仍然只是弱区分度占位名；目的：避免紧凑编号写法绕过 weak source_ref 诊断。
#[test]
fn retrieval_engine_diagnostics_flag_compact_placeholder_source_refs_with_numeric_suffix() {
    let execution = sample_retrieval_engine()
        .retrieve_with_diagnostics(
            "sales review",
            &sample_candidate_scope_for_revenue_only(),
            &sample_graph_store_for_compact_placeholder_numeric_source_ref_hygiene(),
        )
        .expect("compact placeholder numeric source sample should return diagnostics");

    let diagnostic = execution
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.node_id == "node-compact-placeholder-numeric-source-ref")
        .expect("compact placeholder numeric diagnostic should exist");

    assert_eq!(diagnostic.duplicate_evidence_ref_count, 0);
    assert_eq!(diagnostic.weak_locator_count, 0);
    assert_eq!(diagnostic.weak_source_ref_count, 1);
    assert_eq!(
        diagnostic.hygiene_flags,
        vec![RetrievalHygieneFlag::WeakSourceRef]
    );
}

// 2026-04-07 CST: 这里集中构造 retrieval 测试所需的最小引擎，原因是当前 Task 7
// 2026-04-07 CST: 只验证 scoped retrieval 行为，不应该把测试耦合到 dispatcher、CLI 或任何业务层。
// 2026-04-07 CST: 目的：让红绿循环只围绕 foundation 主线进行，保持测试输入足够小、故障定位足够直接。
fn sample_retrieval_engine() -> RetrievalEngine {
    RetrievalEngine::new()
}

// 2026-04-08 CST: 这里集中构造 revenue-only 候选域，原因是标题优先和短语优先测试都只想验证
// 2026-04-08 CST: retrieval 在单概念候选域内的排序变化，不需要额外混入漫游路径因素。
// 2026-04-08 CST: 目的：给 Task 11 第一层排序测试提供最小、可复用的候选域输入。
fn sample_candidate_scope_for_revenue_only() -> CandidateScope {
    CandidateScope {
        concept_ids: vec!["revenue".to_string()],
        path: Vec::new(),
        // 2026-04-09 CST: 这里补空 metadata scope，原因是大多数 retrieval 单测当前不需要元数据过滤。
        // 目的：给统一 scope 合同提供稳定零值，减少各测试重复散落构造逻辑。
        metadata_scope: MetadataScope::new(),
    }
}

// 2026-04-08 CST: 这里集中构造带漫游路径的候选域，原因是 seed concept 优先测试需要明确区分
// 2026-04-08 CST: 哪个概念是 route 种子、哪个概念是 roam 补入，不能再只给平面 concept_ids。
// 2026-04-08 CST: 目的：用最小路径样本钉住 retrieval 对 seed/roamed 层级差异的承接行为。
fn sample_candidate_scope_with_roaming_path() -> CandidateScope {
    CandidateScope {
        concept_ids: vec!["revenue".to_string(), "trend".to_string()],
        path: vec![RoamingStep {
            from_concept_id: "revenue".to_string(),
            to_concept_id: "trend".to_string(),
            relation_type: OntologyRelationType::Supports,
            depth: 1,
        }],
        // 2026-04-09 CST: 这里补空 metadata scope，原因是该夹具只用于 seed/roamed 优先级测试，
        // metadata 不应成为额外变量。
        // 目的：保证测试只表达漫游路径语义，不被 scope 结构升级干扰。
        metadata_scope: MetadataScope::new(),
    }
}

// 2026-04-07 CST: 这里集中构造纯内存 graph store，原因是 retrieval 的评分输入
// 2026-04-07 CST: 来自 KnowledgeNode 的标题与正文文本，而不是任何外部文件或运行时上下文。
// 2026-04-07 CST: 目的：用最小图谱样本同时覆盖“范围过滤”“排序”“无命中”三条契约，降低测试维护成本。
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

// 2026-04-08 CST: 这里集中构造标题/正文排序样本，原因是我们要刻意制造“当前实现会按字典序误排”的场景，
// 2026-04-08 CST: 这样红灯才能真正证明标题权重规则是必要的，而不是测试天然就会通过。
// 2026-04-08 CST: 目的：钉住 Task 11 的第一类排序增强合同。
fn sample_graph_store_for_title_body_ranking() -> KnowledgeGraphStore {
    let nodes = vec![
        KnowledgeNode::new(
            "node-a-body-match",
            "Overview",
            "Sales review details live in the body only.",
        )
        .with_concept_id("revenue")
        .with_evidence_ref(EvidenceRef::new("sheet:body", "A1:A5")),
        KnowledgeNode::new("node-z-title-match", "Sales Review", "Overview only.")
            .with_concept_id("revenue")
            .with_evidence_ref(EvidenceRef::new("sheet:title", "B1:B5")),
    ];

    KnowledgeGraphStore::new(nodes, Vec::new())
}

// 2026-04-08 CST: 这里集中构造短语排序样本，原因是我们要区分“完整短语命中”和“分散 token 命中”，
// 2026-04-08 CST: 不能再依赖纯 token 交集数量，否则 phrase bonus 永远无法被回归测试约束。
// 2026-04-08 CST: 目的：钉住 Task 11 的第二类排序增强合同。
fn sample_graph_store_for_phrase_ranking() -> KnowledgeGraphStore {
    let nodes = vec![
        KnowledgeNode::new(
            "node-a-scattered-match",
            "Sales Overview",
            "Trend signals are discussed separately here.",
        )
        .with_concept_id("revenue")
        .with_evidence_ref(EvidenceRef::new("sheet:scatter", "C1:C5")),
        KnowledgeNode::new(
            "node-z-phrase-match",
            "Overview",
            "The sales trend is summarized here.",
        )
        .with_concept_id("revenue")
        .with_evidence_ref(EvidenceRef::new("sheet:phrase", "D1:D5")),
    ];

    KnowledgeGraphStore::new(nodes, Vec::new())
}

// 2026-04-08 CST: 这里集中构造 seed priority 排序样本，原因是 retrieval 第一层增强要开始体现
// 2026-04-08 CST: foundation 主链“先 route 种子、后 roam 补全”的结构倾向，而不是把两类概念完全拍平。
// 2026-04-08 CST: 目的：钉住 Task 11 的第三类排序增强合同。
fn sample_graph_store_for_seed_priority() -> KnowledgeGraphStore {
    let nodes = vec![
        KnowledgeNode::new("node-a-roamed-trend", "Sales Trend", "Trend note.")
            .with_concept_id("trend")
            .with_evidence_ref(EvidenceRef::new("sheet:trend", "E1:E5")),
        KnowledgeNode::new("node-z-seed-revenue", "Sales Trend", "Revenue note.")
            .with_concept_id("revenue")
            .with_evidence_ref(EvidenceRef::new("sheet:revenue", "F1:F5")),
    ];

    KnowledgeGraphStore::new(nodes, Vec::new())
}

// 2026-04-09 CST: 这里集中构造 primary/derived 来源排序样本，原因是第二层增强需要刻意制造“文本同分但来源层级不同”的场景。
// 2026-04-09 CST: 节点 id 故意让派生来源排在字典序前面，这样红灯才能证明当前实现确实缺少 source_ref tie-break。
// 2026-04-09 CST: 目的：让测试失败原因只指向来源排序规则缺失，而不是混入其他文本分数差异。
fn sample_graph_store_for_source_priority() -> KnowledgeGraphStore {
    let nodes = vec![
        KnowledgeNode::new("node-a-derived-source", "Sales Review", "Overview only.")
            .with_concept_id("revenue")
            .with_evidence_ref(EvidenceRef::new("sheet:trend_summary", "G1:G5")),
        KnowledgeNode::new("node-z-primary-source", "Sales Review", "Overview only.")
            .with_concept_id("revenue")
            .with_evidence_ref(EvidenceRef::new("sheet:sales_raw", "H1:H5")),
    ];

    KnowledgeGraphStore::new(nodes, Vec::new())
}

// 2026-04-09 CST: 这里集中构造 derived/planning 来源排序样本，原因是来源优先级不仅要识别原始来源，还要区分中间层和规划层。
// 2026-04-09 CST: 两个节点保持完全相同的文本得分，只让 source_ref 承担分层差异，避免测试含义被其他 bonus 污染。
// 2026-04-09 CST: 目的：把“derived 高于 planning”的固定规则单独钉住，防止后续排序逻辑回退成字典序。
fn sample_graph_store_for_derived_vs_planning_source_priority() -> KnowledgeGraphStore {
    let nodes = vec![
        KnowledgeNode::new("node-a-planning-source", "Sales Review", "Overview only.")
            .with_concept_id("revenue")
            .with_evidence_ref(EvidenceRef::new("sheet:plan_forecast", "I1:I5")),
        KnowledgeNode::new("node-z-derived-source", "Sales Review", "Overview only.")
            .with_concept_id("revenue")
            .with_evidence_ref(EvidenceRef::new("sheet:summary_report", "J1:J5")),
    ];

    KnowledgeGraphStore::new(nodes, Vec::new())
}

// 2026-04-09 CST: 这里集中构造“文本分数更高但来源较弱”的保护样本，原因是 source_ref 只能在同分阶段生效。
// 2026-04-09 CST: 我们故意让原始来源节点文本略弱，防止后续有人把来源偏好直接并入主分数时悄悄改坏排序主轴。
// 2026-04-09 CST: 目的：用最小 fixture 保护“文本优先、来源次级”的 foundation 检索边界。
fn sample_graph_store_for_source_priority_not_overriding_text_score() -> KnowledgeGraphStore {
    let nodes = vec![
        KnowledgeNode::new(
            "node-higher-text-score",
            "Sales Review Trend",
            "Overview only.",
        )
        .with_concept_id("revenue")
        .with_evidence_ref(EvidenceRef::new("sheet:plan_forecast", "K1:K5")),
        KnowledgeNode::new(
            "node-better-source-priority",
            "Sales Review",
            "Trend note only.",
        )
        .with_concept_id("revenue")
        .with_evidence_ref(EvidenceRef::new("sheet:sales_raw", "L1:L5")),
    ];

    KnowledgeGraphStore::new(nodes, Vec::new())
}

// 2026-04-09 CST: 这里集中构造证据数量优先样本，原因是方案 C 的第一层证据侧信号就是 evidence_refs 数量。
// 2026-04-09 CST: 两个节点保持相同文本和相同 source 层级，只让证据条数不同，并故意把少证据节点放在字典序前面。
// 2026-04-09 CST: 目的：让失败原因明确指向“缺少 evidence count tie-break”，不混入其他评分差异。
fn sample_graph_store_for_evidence_count_priority() -> KnowledgeGraphStore {
    let nodes = vec![
        KnowledgeNode::new("node-a-less-evidence", "Sales Review", "Overview only.")
            .with_concept_id("revenue")
            .with_evidence_ref(EvidenceRef::new("sheet:sales_raw", "M1")),
        KnowledgeNode::new("node-z-more-evidence", "Sales Review", "Overview only.")
            .with_concept_id("revenue")
            .with_evidence_ref(EvidenceRef::new("sheet:invoice_detail", "N1"))
            .with_evidence_ref(EvidenceRef::new("sheet:invoice_detail", "N2")),
    ];

    KnowledgeGraphStore::new(nodes, Vec::new())
}

// 2026-04-09 CST: 这里集中构造 locator 精度优先样本，原因是方案 C 的第二层证据侧信号要表达“更具体的定位更优先”。
// 2026-04-09 CST: 两个节点证据条数一致、来源层级一致，只让 locator 从单点与大范围上产生差异。
// 2026-04-09 CST: 目的：给 locator precision helper 提供最小、可复现的回归样本。
fn sample_graph_store_for_locator_precision_priority() -> KnowledgeGraphStore {
    let nodes = vec![
        KnowledgeNode::new("node-a-broad-locator", "Sales Review", "Overview only.")
            .with_concept_id("revenue")
            .with_evidence_ref(EvidenceRef::new("sheet:sales_raw", "A1:D20")),
        KnowledgeNode::new("node-z-specific-locator", "Sales Review", "Overview only.")
            .with_concept_id("revenue")
            .with_evidence_ref(EvidenceRef::new("sheet:invoice_detail", "A1")),
    ];

    KnowledgeGraphStore::new(nodes, Vec::new())
}

// 2026-04-09 CST: 这里集中构造“source 优先级高于证据数量”的保护样本，原因是证据侧 tie-break 必须排在来源层之后。
// 2026-04-09 CST: 我们故意让 derived 节点拥有更多 evidence_refs，用来防止后续排序链条被错误调整。
// 2026-04-09 CST: 目的：把当前 retrieval 分层顺序固定为 source 优先于 evidence，而不是相反。
fn sample_graph_store_for_source_priority_over_evidence_count() -> KnowledgeGraphStore {
    let nodes = vec![
        KnowledgeNode::new("node-a-more-evidence", "Sales Review", "Overview only.")
            .with_concept_id("revenue")
            .with_evidence_ref(EvidenceRef::new("sheet:summary_report", "P1"))
            .with_evidence_ref(EvidenceRef::new("sheet:summary_report", "P2")),
        KnowledgeNode::new("node-z-better-source", "Sales Review", "Overview only.")
            .with_concept_id("revenue")
            .with_evidence_ref(EvidenceRef::new("sheet:sales_raw", "Q1")),
    ];

    KnowledgeGraphStore::new(nodes, Vec::new())
}

// 2026-04-09 CST: 这里集中构造 retrieval diagnostics 样本，原因是方案 A 需要一个同时覆盖命中解释和排序解释的最小夹具。
// 2026-04-09 CST: 两个节点保持相同问题、相同概念、接近文本信号，只通过 source/evidence/locator 拉开可解释差异；目的：让 diagnostics 测试只对准 retrieval 内部合同。
fn sample_graph_store_for_retrieval_diagnostics() -> KnowledgeGraphStore {
    let nodes = vec![
        KnowledgeNode::new(
            "node-a-diagnostic-derived",
            "Sales Review",
            "Review note only.",
        )
        .with_concept_id("revenue")
        .with_evidence_ref(EvidenceRef::new("sheet:summary_report", "A1:D20")),
        KnowledgeNode::new(
            "node-z-diagnostic-primary",
            "Sales Review",
            "Review note only.",
        )
        .with_concept_id("revenue")
        .with_evidence_ref(EvidenceRef::new("sheet:sales_raw", "A1"))
        .with_evidence_ref(EvidenceRef::new("sheet:invoice_detail", "A2:B2")),
    ];

    KnowledgeGraphStore::new(nodes, Vec::new())
}

// 2026-04-09 CST: 这里集中构造重复证据 hygiene 样本，原因是我们要把“同一节点内重复引用”单独从一般 evidence_count 里拆出来观察。
// 2026-04-09 CST: 目的：让后续 diagnostics 能区分“证据真的更多”还是“同一证据重复出现”。
fn sample_graph_store_for_duplicate_evidence_hygiene() -> KnowledgeGraphStore {
    let nodes = vec![
        KnowledgeNode::new(
            "node-duplicate-evidence",
            "Sales Review",
            "Review note only.",
        )
        .with_concept_id("revenue")
        .with_evidence_ref(EvidenceRef::new("sheet:sales_raw", "A1"))
        .with_evidence_ref(EvidenceRef::new("sheet:sales_raw", "A1")),
    ];

    KnowledgeGraphStore::new(nodes, Vec::new())
}

// 2026-04-09 CST: 这里集中构造弱 locator hygiene 样本，原因是 locator 现在参与可解释 tie-break，但还没有质量风险标记。
// 2026-04-09 CST: 目的：先用一个超宽范围样本把 weak locator 最小边界钉住。
fn sample_graph_store_for_weak_locator_hygiene() -> KnowledgeGraphStore {
    let nodes = vec![
        KnowledgeNode::new("node-weak-locator", "Sales Review", "Review note only.")
            .with_concept_id("revenue")
            .with_evidence_ref(EvidenceRef::new("sheet:sales_raw", "A1:Z200")),
    ];

    KnowledgeGraphStore::new(nodes, Vec::new())
}

// 2026-04-09 CST: 这里集中构造带 sheet 前缀的单点 locator 样本，原因是我们准备把 locator hygiene 的识别边界从裸 `A1`
// 2026-04-09 CST: 扩到真实表格常见的 `Sheet!A1` 形式；目的：给后续最小解析增强提供稳定回归样本，不把业务层格式感知混进 foundation。
fn sample_graph_store_for_sheet_qualified_locator_hygiene() -> KnowledgeGraphStore {
    let nodes = vec![
        KnowledgeNode::new(
            "node-sheet-qualified-locator",
            "Sales Review",
            "Review note only.",
        )
        .with_concept_id("revenue")
        .with_evidence_ref(EvidenceRef::new("sheet:sales_raw", "Sheet1!A1")),
    ];

    KnowledgeGraphStore::new(nodes, Vec::new())
}

// 2026-04-09 CST: 这里集中构造带 sheet 前缀和绝对引用的小范围 locator 样本，原因是真实工作簿里大量存在 `$A$1:$D$5`
// 2026-04-09 CST: 这样的固定引用；目的：锁定最小增强后 diagnostics 能正确接受这类 locator，而不是把它们误判成无法解析。
fn sample_graph_store_for_sheet_qualified_absolute_range_locator_hygiene() -> KnowledgeGraphStore {
    let nodes = vec![
        KnowledgeNode::new(
            "node-sheet-qualified-absolute-range",
            "Sales Review",
            "Review note only.",
        )
        .with_concept_id("revenue")
        .with_evidence_ref(EvidenceRef::new(
            "sheet:sales_raw",
            "'Sales Detail'!$A$1:$D$5",
        )),
    ];

    KnowledgeGraphStore::new(nodes, Vec::new())
}

// 2026-04-09 CST: 这里集中构造带 Windows 绝对路径前缀的小范围 locator 样本，原因是真实桌面环境里常见
// 2026-04-09 CST: `C:\Reports\[Book.xlsx]Sheet!A1:B3` 这类引用；目的：让本轮红灯准确命中 drive letter 破坏范围解析的缺口。
fn sample_graph_store_for_windows_path_external_workbook_range_locator_hygiene() -> KnowledgeGraphStore {
    let nodes = vec![
        KnowledgeNode::new(
            "node-windows-path-external-workbook-range",
            "Sales Review",
            "Review note only.",
        )
        .with_concept_id("revenue")
        .with_evidence_ref(EvidenceRef::new(
            "sheet:sales_raw",
            "C:\\Reports\\[Budget.xlsx]Sheet1!A1:B3",
        )),
    ];

    KnowledgeGraphStore::new(nodes, Vec::new())
}

// 2026-04-09 CST: 这里集中构造带 Windows 绝对路径前缀的绝对范围 locator 样本，原因是 drive letter 前缀不应让
// 2026-04-09 CST: 现有 `$A$1:$D$5` 小范围解析失效；目的：验证本轮补丁只修正范围切分边界，不扩成完整路径语义系统。
fn sample_graph_store_for_windows_path_external_workbook_absolute_range_locator_hygiene(
) -> KnowledgeGraphStore {
    let nodes = vec![
        KnowledgeNode::new(
            "node-windows-path-external-workbook-absolute-range",
            "Sales Review",
            "Review note only.",
        )
        .with_concept_id("revenue")
        .with_evidence_ref(EvidenceRef::new(
            "sheet:sales_raw",
            "C:\\Reports\\[Budget.xlsx]'Sales Detail'!$A$1:$D$5",
        )),
    ];

    KnowledgeGraphStore::new(nodes, Vec::new())
}

// 2026-04-09 CST: 这里集中构造带 Windows 绝对路径前缀的大范围 locator 样本，原因是上轮已经证明这类路径前缀
// 2026-04-09 CST: 可以让小范围 A1 locator 进入可解析集合，但还需要继续保留“大范围仍 weak”的质量边界；目的：给
// 2026-04-09 CST: 本轮保护测试提供真实桌面格式样本，防止路径前缀修复把面积阈值语义冲掉。
fn sample_graph_store_for_windows_path_external_workbook_large_range_locator_hygiene(
) -> KnowledgeGraphStore {
    let nodes = vec![
        KnowledgeNode::new(
            "node-windows-path-external-workbook-large-range",
            "Sales Review",
            "Review note only.",
        )
        .with_concept_id("revenue")
        .with_evidence_ref(EvidenceRef::new(
            "sheet:sales_raw",
            "C:\\Reports\\[Budget.xlsx]Sheet1!A1:Z200",
        )),
    ];

    KnowledgeGraphStore::new(nodes, Vec::new())
}

// 2026-04-09 CST: 这里集中构造带 Windows 绝对路径前缀的大范围绝对引用 locator 样本，原因是 `$A$1:$Z$200`
// 2026-04-09 CST: 这类引用最容易在“解析成功后就被误当成高质量定位”的心智误差里漏掉；目的：确认 foundation
// 2026-04-09 CST: retrieval 仍然把它视作 weak locator，而不是因为路径前缀已支持就整体豁免面积规则。
fn sample_graph_store_for_windows_path_external_workbook_absolute_large_range_locator_hygiene(
) -> KnowledgeGraphStore {
    let nodes = vec![
        KnowledgeNode::new(
            "node-windows-path-external-workbook-absolute-large-range",
            "Sales Review",
            "Review note only.",
        )
        .with_concept_id("revenue")
        .with_evidence_ref(EvidenceRef::new(
            "sheet:sales_raw",
            "C:\\Reports\\[Budget.xlsx]'Sales Detail'!$A$1:$Z$200",
        )),
    ];

    KnowledgeGraphStore::new(nodes, Vec::new())
}

// 2026-04-09 CST: 这里集中构造带 sheet 前缀的大范围 locator 样本，原因是补了解析能力以后还要继续保留“过宽范围属于弱定位”的质量诊断；
// 2026-04-09 CST: 目的：防止实现只顾着让解析通过，却把 locator hygiene 的面积阈值语义悄悄丢掉。
fn sample_graph_store_for_sheet_qualified_large_range_locator_hygiene() -> KnowledgeGraphStore {
    let nodes = vec![
        KnowledgeNode::new(
            "node-sheet-qualified-large-range",
            "Sales Review",
            "Review note only.",
        )
        .with_concept_id("revenue")
        .with_evidence_ref(EvidenceRef::new(
            "sheet:sales_raw",
            "'Sales Detail'!$A$1:$Z$200",
        )),
    ];

    KnowledgeGraphStore::new(nodes, Vec::new())
}

// 2026-04-09 CST: 这里集中构造命名区域 locator 样本，原因是本轮故意不把 named range 解析并入 foundation retrieval，
// 2026-04-09 CST: 目的：把“当前仍视为 weak locator”的边界写成回归测试，防止后续误读范围导致合同漂移。
fn sample_graph_store_for_named_range_locator_hygiene() -> KnowledgeGraphStore {
    let nodes = vec![
        KnowledgeNode::new(
            "node-named-range-locator",
            "Sales Review",
            "Review note only.",
        )
        .with_concept_id("revenue")
        .with_evidence_ref(EvidenceRef::new("sheet:sales_raw", "RevenueNamedRange")),
    ];

    KnowledgeGraphStore::new(nodes, Vec::new())
}

// 2026-04-09 CST: 这里集中构造弱 source_ref hygiene 样本，原因是 retrieval diagnostics 需要区分“有来源”与“来源有语义”。
// 2026-04-09 CST: 目的：先用最小占位符式来源样本把 weak source_ref 的检测合同钉住。
fn sample_graph_store_for_weak_source_ref_hygiene() -> KnowledgeGraphStore {
    let nodes = vec![
        KnowledgeNode::new(
            "node-weak-source-ref",
            "Sales Review",
            "Review note only.",
        )
        .with_concept_id("revenue")
        .with_evidence_ref(EvidenceRef::new("sheet", "A1")),
    ];

    KnowledgeGraphStore::new(nodes, Vec::new())
}

// 2026-04-09 CST: 这里集中构造多占位词组合来源样本，原因是我们准备把 weak source_ref 从“单 token 占位词”
// 2026-04-09 CST: 扩到“全由占位词组成的短来源名”；目的：给后续最小规则增强提供明确红灯夹具。
fn sample_graph_store_for_multi_token_placeholder_source_ref_hygiene() -> KnowledgeGraphStore {
    let nodes = vec![
        KnowledgeNode::new(
            "node-multi-token-placeholder-source-ref",
            "Sales Review",
            "Review note only.",
        )
        .with_concept_id("revenue")
        .with_evidence_ref(EvidenceRef::new("source data", "A1")),
    ];

    KnowledgeGraphStore::new(nodes, Vec::new())
}

// 2026-04-09 CST: 这里集中构造占位词加编号来源样本，原因是现实里常见 `table 1`、`sheet 2` 这类默认命名，
// 2026-04-09 CST: 它们长度虽变长但区分度仍然很弱；目的：让编号后缀不会绕过 weak source_ref 诊断。
fn sample_graph_store_for_placeholder_numeric_source_ref_hygiene() -> KnowledgeGraphStore {
    let nodes = vec![
        KnowledgeNode::new(
            "node-placeholder-numeric-source-ref",
            "Sales Review",
            "Review note only.",
        )
        .with_concept_id("revenue")
        .with_evidence_ref(EvidenceRef::new("table 1", "A1")),
    ];

    KnowledgeGraphStore::new(nodes, Vec::new())
}

// 2026-04-09 CST: 这里集中构造“含占位词但仍具业务语义”的来源样本，原因是后续扩规则不能把真实来源一起误伤；
// 2026-04-09 CST: 目的：给 weak source_ref 保留最小保护边界，确保具体业务词仍然能让来源名保有区分度。
fn sample_graph_store_for_semantic_source_ref_with_placeholder_tokens() -> KnowledgeGraphStore {
    let nodes = vec![
        KnowledgeNode::new(
            "node-semantic-source-ref",
            "Sales Review",
            "Review note only.",
        )
        .with_concept_id("revenue")
        .with_evidence_ref(EvidenceRef::new("sales detail sheet", "A1")),
    ];

    KnowledgeGraphStore::new(nodes, Vec::new())
}

// 2026-04-09 CST: 这里集中构造紧凑编号占位来源样本，原因是默认命名常见 `sheet1`、`table01` 这类不带空格的写法，
// 2026-04-09 CST: 目的：给 weak source_ref 扩一个保守但实用的最小边界，确保紧凑编号命名不会漏诊。
fn sample_graph_store_for_compact_placeholder_numeric_source_ref_hygiene() -> KnowledgeGraphStore {
    let nodes = vec![
        KnowledgeNode::new(
            "node-compact-placeholder-numeric-source-ref",
            "Sales Review",
            "Review note only.",
        )
        .with_concept_id("revenue")
        .with_evidence_ref(EvidenceRef::new("sheet1", "A1")),
    ];

    KnowledgeGraphStore::new(nodes, Vec::new())
}
