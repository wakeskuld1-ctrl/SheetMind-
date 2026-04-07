use excel_skill::ops::foundation::capability_router::CapabilityRoute;
use excel_skill::ops::foundation::evidence_assembler::{EvidenceAssembler, NavigationEvidence};
use excel_skill::ops::foundation::knowledge_record::EvidenceRef;
use excel_skill::ops::foundation::ontology_schema::OntologyRelationType;
use excel_skill::ops::foundation::retrieval_engine::RetrievalHit;
use excel_skill::ops::foundation::roaming_engine::{CandidateScope, RoamingStep};

// 2026-04-08 CST: 这里先补 Task 8 的首条失败测试，原因是当前 foundation 主链已经走到
// route -> roam -> retrieve 之后，必须把最终输出结构钉死，避免 evidence assembly 再次散回上层各自拼装。
// 目的：先确认 assembler 会完整保留 route、roaming path 和 retrieval hits 这三类主链信息。
#[test]
fn evidence_assembler_preserves_route_path_and_hits() {
    let evidence = sample_assembler().assemble(sample_route(), sample_scope(), sample_hits());

    assert_eq!(evidence.route.matched_concept_ids, vec!["revenue"]);
    assert_eq!(evidence.roaming_path.len(), 1);
    assert_eq!(evidence.hits.len(), 1);
    assert_eq!(evidence.hits[0].node_id, "node-revenue-1");
}

// 2026-04-08 CST: 这里补 citation 与 summary 测试，原因是 Task 8 的职责不只是“把字段堆一起”，
// 还要把 retrieval hit 里的 evidence refs 提炼成统一 citations，并生成最小可读摘要。
// 目的：先把最小可消费输出固定下来，为 Task 9 集成闭环提供稳定结果对象。
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

// 2026-04-08 CST: 这里集中构造 assembler，原因是当前 Task 8 只验证 foundation 侧的装配逻辑，
// 不应该把测试耦合到 CLI、dispatcher 或其他运行时组件。
// 目的：让红绿循环只围绕 evidence assembly 的最小职责展开。
fn sample_assembler() -> EvidenceAssembler {
    EvidenceAssembler::new()
}

// 2026-04-08 CST: 这里集中构造最小路由结果，原因是 assembler 需要承接 router 已经确认的种子概念。
// 目的：保持测试输入足够小，同时明确 route 会被原样带入最终证据对象。
fn sample_route() -> CapabilityRoute {
    CapabilityRoute {
        matched_concept_ids: vec!["revenue".to_string()],
    }
}

// 2026-04-08 CST: 这里集中构造最小 candidate scope，原因是 assembler 需要读取 roaming path，
// 但不应该重新参与候选域扩展或概念收敛。
// 目的：验证 scope 中的 path 能被稳定透传到最终输出。
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

// 2026-04-08 CST: 这里集中构造 retrieval hit，原因是 citation 装配必须直接消费检索层稳定输出，
// 不能回头再依赖 graph store 重查。
// 目的：先固定“hit 带 evidence refs，assembler 只做收口和摘要”的职责边界。
fn sample_hits() -> Vec<RetrievalHit> {
    vec![RetrievalHit {
        node_id: "node-revenue-1".to_string(),
        score: 2,
        evidence_refs: vec![EvidenceRef::new("sheet:sales", "A1:B12")],
    }]
}

// 2026-04-08 CST: 这里保留显式类型检查，原因是 Task 8 需要让 NavigationEvidence 成为 foundation 主线的
// 正式稳定输出，而不是只在测试里临时出现。
// 目的：即使后续字段扩展，这个测试文件也能继续作为对外形状的最小守门人。
#[allow(dead_code)]
fn _evidence_type_guard(evidence: NavigationEvidence) -> NavigationEvidence {
    evidence
}
