use excel_skill::ops::foundation::capability_router::CapabilityRoute;
use excel_skill::ops::foundation::evidence_assembler::{EvidenceAssembler, NavigationEvidence};
use excel_skill::ops::foundation::knowledge_record::EvidenceRef;
use excel_skill::ops::foundation::ontology_schema::OntologyRelationType;
use excel_skill::ops::foundation::retrieval_engine::RetrievalHit;
use excel_skill::ops::foundation::roaming_engine::{CandidateScope, RoamingStep};

// 2026-04-08 CST: 这里先补主链聚合测试，原因是 Task 8 的职责不是简单拼字段，
// 而是要把 route、roaming path 和 retrieval hits 统一收口成最终证据对象。
// 目的：先固定 foundation 主链在 assemble 后的最小输出形状。
#[test]
fn evidence_assembler_preserves_route_path_and_hits() {
    let evidence = sample_assembler().assemble(sample_route(), sample_scope(), sample_hits());

    assert_eq!(evidence.route.matched_concept_ids, vec!["revenue"]);
    assert_eq!(evidence.roaming_path.len(), 1);
    assert_eq!(evidence.hits.len(), 1);
    assert_eq!(evidence.hits[0].node_id, "node-revenue-1");
}

// 2026-04-08 CST: 这里补 citation 与 summary 测试，原因是 assembler 不只是“搬运 hits”，
// 还要为上层生成统一 citations 与最小可读摘要。
// 目的：确保 Task 9 集成时能直接消费一个结构完整的 NavigationEvidence。
#[test]
fn evidence_assembler_builds_citations_and_summary() {
    let evidence = sample_assembler().assemble(sample_route(), sample_scope(), sample_hits());

    assert_eq!(
        evidence.citations,
        vec![EvidenceRef::new("sheet:sales", "A1:B12")]
    );
    assert!(evidence.summary.contains("1"));
    assert!(evidence.summary.contains("revenue"));
}

// 2026-04-08 CST: 这里集中构造 assembler，原因是当前只验证 foundation 侧装配逻辑，
// 不应把 CLI、dispatcher 或运行时环境耦合进来。
// 目的：让测试聚焦在 assemble 本身。
fn sample_assembler() -> EvidenceAssembler {
    EvidenceAssembler::new()
}

// 2026-04-08 CST: 这里构造最小 route，原因是 assembler 需要承接 router 已确认的 seed concepts，
// 但不应重新参与 route 阶段职责。
// 目的：验证 route 信息会被原样带入最终证据对象。
fn sample_route() -> CapabilityRoute {
    CapabilityRoute {
        matched_concept_ids: vec!["revenue".to_string()],
    }
}

// 2026-04-08 CST: 这里构造最小 candidate scope，原因是 assembler 只消费 roaming path，
// 不应重新扩展候选域。
// 目的：确认 path 能稳定透传到最终结果。
fn sample_scope() -> CandidateScope {
    CandidateScope {
        concept_ids: vec!["revenue".to_string(), "invoice".to_string()],
        path: vec![RoamingStep {
            from_concept_id: "revenue".to_string(),
            to_concept_id: "invoice".to_string(),
            relation_type: OntologyRelationType::DependsOn,
            depth: 1,
        }],
    }
}

// 2026-04-08 CST: 这里构造最小 retrieval hit，原因是 citations 需要直接来自检索层稳定输出，
// 不应回头依赖 graph store 重新查。
// 目的：固定 assembler 的输入边界。
fn sample_hits() -> Vec<RetrievalHit> {
    vec![RetrievalHit {
        node_id: "node-revenue-1".to_string(),
        score: 2,
        evidence_refs: vec![EvidenceRef::new("sheet:sales", "A1:B12")],
    }]
}

// 2026-04-08 CST: 这里保留显式类型守卫，原因是 NavigationEvidence 是 foundation 主链的正式输出，
// 不应只在测试过程里临时存在。
// 目的：把对外形状显式固定下来。
#[allow(dead_code)]
fn _evidence_type_guard(evidence: NavigationEvidence) -> NavigationEvidence {
    evidence
}
