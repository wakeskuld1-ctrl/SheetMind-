use crate::ops::foundation::capability_router::CapabilityRoute;
use crate::ops::foundation::knowledge_record::EvidenceRef;
use crate::ops::foundation::retrieval_engine::RetrievalHit;
use crate::ops::foundation::roaming_engine::{CandidateScope, RoamingStep};

// 2026-04-08 CST: 这里定义 foundation 主链的统一证据输出，原因是 Task 8 需要把 route、roaming path、
// retrieval hits、citations 与摘要收口成一个稳定结果，而不是让上层继续持有多个中间对象。
// 目的：固定最小可消费的导航证据形状。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavigationEvidence {
    pub route: CapabilityRoute,
    pub roaming_path: Vec<RoamingStep>,
    pub hits: Vec<RetrievalHit>,
    pub citations: Vec<EvidenceRef>,
    pub summary: String,
}

// 2026-04-08 CST: 这里把 assembler 保持为无状态装配器，原因是当前阶段只需要统一收口已有中间结果，
// 不应提前引入模板配置、provider 依赖或复杂摘要策略。
// 目的：维持 foundation 主链最小实现。
#[derive(Debug, Clone, Default)]
pub struct EvidenceAssembler;

impl EvidenceAssembler {
    // 2026-04-08 CST: 这里提供最小构造函数，原因是测试与后续 pipeline 都需要稳定入口，
    // 当前又没有显式配置项。
    // 目的：保持调用方式简单直接。
    pub fn new() -> Self {
        Self
    }

    // 2026-04-08 CST: 这里实现最小装配入口，原因是 foundation 已完成 route、roam、retrieve 三段，
    // 当前必须有一层把它们收束成最终 NavigationEvidence。
    // 目的：为 Task 9 集成闭环提供稳定输出对象。
    pub fn assemble(
        &self,
        route: CapabilityRoute,
        scope: CandidateScope,
        hits: Vec<RetrievalHit>,
    ) -> NavigationEvidence {
        let citations = collect_citations(&hits);
        let summary = build_summary(&route, &scope, &hits);

        NavigationEvidence {
            route,
            roaming_path: scope.path,
            hits,
            citations,
            summary,
        }
    }
}

// 2026-04-08 CST: 这里抽取 citation 收集逻辑，原因是 retrieval hit 可能携带多个 evidence refs，
// assembler 需要统一拉平并去重，而不是把这份责任留给上层。
// 目的：保证 citations 输出稳定且不重复。
fn collect_citations(hits: &[RetrievalHit]) -> Vec<EvidenceRef> {
    let mut citations = Vec::new();

    for hit in hits {
        for evidence_ref in &hit.evidence_refs {
            if citations.iter().any(|existing: &EvidenceRef| {
                existing.source_ref == evidence_ref.source_ref
                    && existing.locator == evidence_ref.locator
            }) {
                continue;
            }

            citations.push(evidence_ref.clone());
        }
    }

    citations
}

// 2026-04-08 CST: 这里生成最小摘要，原因是当前阶段只需要一个稳定、可测、零依赖的说明文本，
// 不应过早引入更复杂的摘要生成器。
// 目的：提供可直接展示的最小可读说明。
fn build_summary(route: &CapabilityRoute, scope: &CandidateScope, hits: &[RetrievalHit]) -> String {
    let concept_text = if route.matched_concept_ids.is_empty() {
        "unknown".to_string()
    } else {
        route.matched_concept_ids.join(", ")
    };

    format!(
        "Assembled {} hit(s) for concept(s) {} across {} roaming step(s).",
        hits.len(),
        concept_text,
        scope.path.len()
    )
}
