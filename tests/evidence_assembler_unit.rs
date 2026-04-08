use excel_skill::ops::foundation::capability_router::CapabilityRoute;
use excel_skill::ops::foundation::evidence_assembler::EvidenceAssembler;
use excel_skill::ops::foundation::knowledge_record::EvidenceRef;
use excel_skill::ops::foundation::ontology_schema::OntologyRelationType;
use excel_skill::ops::foundation::retrieval_engine::RetrievalHit;
use excel_skill::ops::foundation::roaming_engine::{CandidateScope, RoamingStep};

// 2026-04-08 CST: 这里先补 assembler 红灯测试，原因是当前 evidence_assembler 之前还只是占位壳。
// 2026-04-08 CST: 如果不先把最终输出结构钉住，后面的 pipeline 很容易再次退回“临时拼装”。
// 2026-04-08 CST: 目的：先锁定 route、path、hits、citations、summary 这五块最小合同。
#[test]
fn evidence_assembler_preserves_route_path_hits_and_citations() {
    let evidence = EvidenceAssembler::new().assemble(sample_route(), sample_scope(), sample_hits());

    assert_eq!(evidence.route.matched_concept_ids, vec!["revenue"]);
    assert_eq!(evidence.roaming_path.len(), 1);
    assert_eq!(evidence.hits.len(), 1);
    assert_eq!(evidence.citations.len(), 1);
    assert_eq!(evidence.citations[0].node_id, "node-revenue-1");
    assert_eq!(evidence.citations[0].source_ref, "sheet:sales");
    assert_eq!(evidence.citations[0].locator, "A1:B12");
    assert!(evidence.summary.contains("1 route concept"));
    assert!(evidence.summary.contains("1 roaming step"));
    assert!(evidence.summary.contains("1 retrieval hit"));
}

// 2026-04-08 CST: 这里集中构造 route 样本，原因是当前测试目标是装配层保真，
// 2026-04-08 CST: 不应该把红灯耦合到 router 本身实现细节。
// 2026-04-08 CST: 目的：用最小稳定输入固定 evidence assembler 对 route 的保留行为。
fn sample_route() -> CapabilityRoute {
    CapabilityRoute {
        matched_concept_ids: vec!["revenue".to_string()],
    }
}

// 2026-04-08 CST: 这里集中构造漫游范围样本，原因是 assembler 需要保留 candidate scope 的路径信息，
// 2026-04-08 CST: 否则后续最小 pipeline 即使命中证据，也无法解释证据是怎样被收敛出来的。
// 2026-04-08 CST: 目的：先用单步路径钉住 roaming_path 的最小承接合同。
fn sample_scope() -> CandidateScope {
    CandidateScope {
        concept_ids: vec!["revenue".to_string(), "trend".to_string()],
        path: vec![RoamingStep {
            from_concept_id: "revenue".to_string(),
            to_concept_id: "trend".to_string(),
            relation_type: OntologyRelationType::Supports,
            depth: 1,
        }],
    }
}

// 2026-04-08 CST: 这里集中构造 retrieval hit 样本，原因是 citation 需要从 hit 的 evidence refs 展开。
// 2026-04-08 CST: 目的：用最小 hit 样本固定 citations 的铺平行为和 summary 的命中统计行为。
fn sample_hits() -> Vec<RetrievalHit> {
    vec![RetrievalHit {
        node_id: "node-revenue-1".to_string(),
        score: 2,
        evidence_refs: vec![EvidenceRef::new("sheet:sales", "A1:B12")],
    }]
}
