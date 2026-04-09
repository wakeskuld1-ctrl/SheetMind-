use crate::ops::foundation::capability_router::CapabilityRoute;
use crate::ops::foundation::knowledge_record::EvidenceRef;
use crate::ops::foundation::retrieval_engine::RetrievalHit;
use crate::ops::foundation::roaming_engine::{CandidateScope, RoamingStep};

// 2026-04-08 CST: 这里定义 foundation 主链的统一证据输出，原因是 Task 8 需要把 route、roaming path、
// retrieval hits、citations 和简短摘要收口成单一结构，而不是让上层继续分别拿着多个中间对象自行拼接。
// 目的：固定最小可消费结果形状，为后续 Task 9 集成闭环和更上层调用提供稳定输出契约。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavigationEvidence {
    pub route: CapabilityRoute,
    pub roaming_path: Vec<RoamingStep>,
    pub hits: Vec<RetrievalHit>,
    pub citations: Vec<EvidenceRef>,
    pub summary: String,
}

// 2026-04-08 CST: 这里把 assembler 保持为无状态装配器，原因是当前 Task 8 只负责把既有中间结果统一收口，
// 不需要提前引入模板配置、provider 依赖或更复杂的摘要策略。
// 目的：继续遵守 foundation 主线“先最小可用，再逐步增强”的节奏，避免装配层过早膨胀。
#[derive(Debug, Clone, Default)]
pub struct EvidenceAssembler;

impl EvidenceAssembler {
    // 2026-04-08 CST: 这里补最小构造函数，原因是测试和后续 pipeline 都需要一个稳定入口，
    // 但当前 assembler 没有任何可配置项，不应该为了形式引入空配置对象。
    // 目的：把 Task 8 的调用面固定成简单明确的无状态装配器接口。
    pub fn new() -> Self {
        Self
    }

    // 2026-04-08 CST: 这里实现最小装配入口，原因是 foundation 主线已经完成 route、roam、retrieve，
    // 当前必须把这三段结果统一组合成最终证据对象，不能再把收口责任散给上层。
    // 目的：在不引入额外业务语义的前提下，稳定产出 route、path、hits、citations 和 summary。
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

// 2026-04-08 CST: 这里提取 citation 收集逻辑，原因是 retrieval hit 可能带多个证据引用，
// assembler 需要把它们统一拉平、去重，再作为最终输出的一部分稳定暴露。
// 目的：保证上层消费 citations 时不需要自己回头遍历所有 hits，也避免重复证据反复出现。
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

// 2026-04-08 CST: 这里生成最小摘要文本，原因是 Task 8 只需要提供一个可读的装配结果说明，
// 不应该在这个阶段引入 LLM 摘要或复杂模板系统。
// 目的：用稳定、可测试、零依赖的方式说明“命中了多少条证据、围绕哪些种子概念、经过了多少步漫游”。
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
