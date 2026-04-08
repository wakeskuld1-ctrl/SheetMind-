use crate::ops::foundation::capability_router::CapabilityRoute;
use crate::ops::foundation::retrieval_engine::RetrievalHit;
use crate::ops::foundation::roaming_engine::{CandidateScope, RoamingStep};

// 2026-04-08 CST: 这里新增 citation 结构，原因是 retrieval hit 里的 evidence refs 仍挂在节点层。
// 2026-04-08 CST: 如果不在 assembler 中统一摊平，后续 pipeline 或更上层消费时还要再次手工拆解。
// 2026-04-08 CST: 目的：先固定最小“节点 -> 证据来源/定位”输出结构，保持底座结果可检查、可复用。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavigationCitation {
    pub node_id: String,
    pub source_ref: String,
    pub locator: String,
}

// 2026-04-08 CST: 这里新增 navigation evidence 统一输出，原因是当前 foundation 虽然已经有
// 2026-04-08 CST: route / roam / retrieve 三段结果，但还没有一个稳定对象把它们正式收口。
// 2026-04-08 CST: 目的：先把最小闭环输出固定下来，为接下来的 pipeline 承接留出正式合同。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavigationEvidence {
    pub route: CapabilityRoute,
    pub roaming_path: Vec<RoamingStep>,
    pub hits: Vec<RetrievalHit>,
    pub citations: Vec<NavigationCitation>,
    pub summary: String,
}

// 2026-04-08 CST: 这里把 evidence assembler 从占位壳补成正式装配器，原因是 Task 8 的缺口
// 2026-04-08 CST: 已经阻塞了最小 pipeline 的落地；如果继续保留空壳，Task 9 只能回退成测试里手工拼结构。
// 2026-04-08 CST: 目的：在 foundation 内正式收口 route、path、hits、citations 和 summary。
#[derive(Debug, Clone, Default)]
pub struct EvidenceAssembler;

impl EvidenceAssembler {
    // 2026-04-08 CST: 这里提供最小构造函数，原因是 assembler 当前仍保持无状态，
    // 2026-04-08 CST: 但测试和 pipeline 需要一个稳定入口，不能继续依赖默认派生细节。
    // 2026-04-08 CST: 目的：把 Task 8 的调用方式固定成简单、清晰且可复用的对象接口。
    pub fn new() -> Self {
        Self
    }

    // 2026-04-08 CST: 这里实现统一装配入口，原因是 foundation 主链已经明确要求 retrieval 之后
    // 2026-04-08 CST: 不直接把各段结果裸露给上层，而要先收敛成一个可检查的结构化证据对象。
    // 2026-04-08 CST: 目的：把最小闭环输出稳定下来，并为后续 pipeline / CLI 接口保留单一承接点。
    pub fn assemble(
        &self,
        route: CapabilityRoute,
        scope: CandidateScope,
        hits: Vec<RetrievalHit>,
    ) -> NavigationEvidence {
        let citations = hits
            .iter()
            .flat_map(|hit| {
                hit.evidence_refs
                    .iter()
                    .map(|evidence_ref| NavigationCitation {
                        node_id: hit.node_id.clone(),
                        source_ref: evidence_ref.source_ref.clone(),
                        locator: evidence_ref.locator.clone(),
                    })
            })
            .collect::<Vec<_>>();

        let summary = format!(
            "{} route concept(s), {} roaming step(s), {} retrieval hit(s), {} citation(s)",
            route.matched_concept_ids.len(),
            scope.path.len(),
            hits.len(),
            citations.len()
        );

        NavigationEvidence {
            route,
            roaming_path: scope.path,
            hits,
            citations,
            summary,
        }
    }
}
