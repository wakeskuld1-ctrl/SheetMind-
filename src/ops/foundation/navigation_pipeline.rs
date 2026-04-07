use crate::ops::foundation::capability_router::{
    CapabilityRoute, CapabilityRouter, CapabilityRouterError, NavigationRequest,
};
use crate::ops::foundation::evidence_assembler::{EvidenceAssembler, NavigationEvidence};
use crate::ops::foundation::knowledge_graph_store::KnowledgeGraphStore;
use crate::ops::foundation::ontology_store::OntologyStore;
use crate::ops::foundation::retrieval_engine::{RetrievalEngine, RetrievalEngineError};
use crate::ops::foundation::roaming_engine::{RoamingEngine, RoamingEngineError, RoamingPlan};

// 2026-04-08 CST: 这里定义 foundation 最小集成入口错误，原因是 Task 9 要把 route、roam、retrieve、assemble
// 串成一条统一调用链，不能再把上游错误原样散给调用方自己逐层猜测出错位置。
// 目的：先用结构化错误把“问题在哪一环失败”表达清楚，为后续 CLI 或更上层调用保留稳定边界。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavigationPipelineError {
    RouteFailed { question: String },
    RoamFailed { question: String },
    RetrieveFailed { question: String },
}

// 2026-04-08 CST: 这里定义 foundation 轻量 pipeline，原因是 Task 9 的目标是打通最小集成闭环，
// 让当前主线从问题文本一路走到 NavigationEvidence，而不是只存在零散单测。
// 目的：在不引入 dispatcher、GUI 或业务域编排的前提下，提供一个可复用的最小导航入口。
#[derive(Debug, Clone)]
pub struct NavigationPipeline {
    router: CapabilityRouter,
    roaming_engine: RoamingEngine,
    retrieval_engine: RetrievalEngine,
    evidence_assembler: EvidenceAssembler,
    graph_store: KnowledgeGraphStore,
    roaming_plan_template: RoamingPlan,
}

impl NavigationPipeline {
    // 2026-04-08 CST: 这里新增 pipeline 构造函数，原因是当前 Task 9 需要显式收口 ontology store、
    // graph store 和 roaming 预算模板，避免测试或上层调用到处手工拼装模块链。
    // 目的：把 foundation 最小闭环的依赖边界固定下来，同时保持对象本身足够轻量。
    pub fn new(
        ontology_store: OntologyStore,
        graph_store: KnowledgeGraphStore,
        roaming_plan_template: RoamingPlan,
    ) -> Self {
        Self {
            router: CapabilityRouter::new(ontology_store.clone()),
            roaming_engine: RoamingEngine::new(ontology_store),
            retrieval_engine: RetrievalEngine::new(),
            evidence_assembler: EvidenceAssembler::new(),
            graph_store,
            roaming_plan_template,
        }
    }

    // 2026-04-08 CST: 这里实现最小导航入口，原因是 foundation 已经拥有独立的 route、roam、retrieve、assemble，
    // 当前差的只是一个按固定顺序执行它们的统一入口。
    // 目的：把“本体定位 -> 候选域收敛 -> 候选域内检索 -> 证据装配”稳定封装成单次运行链路。
    pub fn run(
        &self,
        request: &NavigationRequest,
    ) -> Result<NavigationEvidence, NavigationPipelineError> {
        let route = self
            .router
            .route(request)
            .map_err(|_| NavigationPipelineError::RouteFailed {
                question: request.question.clone(),
            })?;

        let roaming_plan = self.build_roaming_plan(&route);
        let scope = self
            .roaming_engine
            .roam(roaming_plan)
            .map_err(|_| NavigationPipelineError::RoamFailed {
                question: request.question.clone(),
            })?;

        let hits = self
            .retrieval_engine
            .retrieve(&request.question, &scope, &self.graph_store)
            .map_err(|_| NavigationPipelineError::RetrieveFailed {
                question: request.question.clone(),
            })?;

        Ok(self.evidence_assembler.assemble(route, scope, hits))
    }

    // 2026-04-08 CST: 这里把漫游计划模板和 route 结果拼起来，原因是 pipeline 运行时的 seed concept
    // 应该来自 router，而关系白名单、深度预算和规模预算则来自外部确认过的模板配置。
    // 目的：保持 ontology-first 的顺序，同时避免把 roam 的策略参数硬编码在 pipeline 里。
    fn build_roaming_plan(&self, route: &CapabilityRoute) -> RoamingPlan {
        RoamingPlan {
            seed_concept_ids: route.matched_concept_ids.clone(),
            allowed_relation_types: self.roaming_plan_template.allowed_relation_types.clone(),
            max_depth: self.roaming_plan_template.max_depth,
            max_concepts: self.roaming_plan_template.max_concepts,
        }
    }
}

// 2026-04-08 CST: 这里显式保留底层错误类型导入的存在感，原因是后续如果需要细化 pipeline 错误携带内容，
// 首先要回到这些分层错误来源，而不是绕开 foundation 原有边界。
// 目的：提醒后续扩展时继续保持“分层错误先定义，pipeline 再收口”的方向。
#[allow(dead_code)]
fn _error_type_guard(
    route: CapabilityRouterError,
    roam: RoamingEngineError,
    retrieve: RetrievalEngineError,
) -> (CapabilityRouterError, RoamingEngineError, RetrievalEngineError) {
    (route, roam, retrieve)
}
