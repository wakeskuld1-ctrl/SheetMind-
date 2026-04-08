use crate::ops::foundation::capability_router::{
    CapabilityRoute, CapabilityRouter, CapabilityRouterError, NavigationRequest,
};
use crate::ops::foundation::evidence_assembler::{EvidenceAssembler, NavigationEvidence};
use crate::ops::foundation::knowledge_graph_store::KnowledgeGraphStore;
use crate::ops::foundation::ontology_store::OntologyStore;
use crate::ops::foundation::retrieval_engine::{RetrievalEngine, RetrievalEngineError};
use crate::ops::foundation::roaming_engine::{RoamingEngine, RoamingEngineError, RoamingPlan};

// 2026-04-08 CST: 这里定义 foundation 集成入口错误，原因是 Task 9 需要把 route、roam、retrieve 的失败
// 统一映射到一个稳定边界，而不是让调用方处理多层内部错误类型。
// 目的：清楚表达问题在主链哪一环失败。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavigationPipelineError {
    RouteFailed { question: String },
    RoamFailed { question: String },
    RetrieveFailed { question: String },
}

// 2026-04-08 CST: 这里定义轻量 pipeline，原因是 Task 9 的目标是打通最小闭环，
// 而不是引入更大的编排框架。
// 目的：提供一个可复用的 foundation 级导航入口。
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
    // 2026-04-08 CST: 这里新增 pipeline 构造函数，原因是最小闭环需要显式收口 ontology store、
    // graph store 与 roaming 模板配置。
    // 目的：避免测试或上层手工重复拼接模块链。
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

    // 2026-04-08 CST: 这里实现最小导航入口，原因是 foundation 已具备 route、roam、retrieve、assemble 四段能力，
    // 当前只差一个按固定顺序执行它们的统一接口。
    // 目的：把问题文本稳定映射为 NavigationEvidence。
    pub fn run(
        &self,
        request: &NavigationRequest,
    ) -> Result<NavigationEvidence, NavigationPipelineError> {
        let route =
            self.router
                .route(request)
                .map_err(|_| NavigationPipelineError::RouteFailed {
                    question: request.question.clone(),
                })?;

        let roaming_plan = self.build_roaming_plan(&route);
        let scope = self.roaming_engine.roam(roaming_plan).map_err(|_| {
            NavigationPipelineError::RoamFailed {
                question: request.question.clone(),
            }
        })?;

        let hits = self
            .retrieval_engine
            .retrieve(&request.question, &scope, &self.graph_store)
            .map_err(|_| NavigationPipelineError::RetrieveFailed {
                question: request.question.clone(),
            })?;

        Ok(self.evidence_assembler.assemble(route, scope, hits))
    }

    // 2026-04-08 CST: 这里把 route 结果与 roaming 模板拼接起来，原因是 seed concepts 应来自 router，
    // 但允许关系、深度与规模预算应来自外部已确认的模板。
    // 目的：保持 ontology-first 的链路顺序，同时避免在 pipeline 中硬编码 roaming 策略。
    fn build_roaming_plan(&self, route: &CapabilityRoute) -> RoamingPlan {
        RoamingPlan {
            seed_concept_ids: route.matched_concept_ids.clone(),
            allowed_relation_types: self.roaming_plan_template.allowed_relation_types.clone(),
            max_depth: self.roaming_plan_template.max_depth,
            max_concepts: self.roaming_plan_template.max_concepts,
        }
    }
}

// 2026-04-08 CST: 这里保留底层错误类型守卫，原因是后续如果要细化 pipeline 错误内容，
// 仍应回到分层错误定义，而不是直接绕开 foundation 边界。
// 目的：提醒后续扩展继续遵循“底层先定义，pipeline 再收口”的方向。
#[allow(dead_code)]
fn _error_type_guard(
    route: CapabilityRouterError,
    roam: RoamingEngineError,
    retrieve: RetrievalEngineError,
) -> (
    CapabilityRouterError,
    RoamingEngineError,
    RetrievalEngineError,
) {
    (route, roam, retrieve)
}
