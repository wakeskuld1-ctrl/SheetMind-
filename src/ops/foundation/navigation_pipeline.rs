use crate::ops::foundation::capability_router::{
    CapabilityRoute, CapabilityRouter, CapabilityRouterError, NavigationRequest,
};
use crate::ops::foundation::evidence_assembler::{EvidenceAssembler, NavigationEvidence};
use crate::ops::foundation::knowledge_graph_store::KnowledgeGraphStore;
use crate::ops::foundation::metadata_registry::{MetadataRegistry, MetadataRegistryError};
use crate::ops::foundation::metadata_scope_resolver::MetadataScopeResolver;
use crate::ops::foundation::ontology_store::OntologyStore;
use crate::ops::foundation::retrieval_engine::{RetrievalEngine, RetrievalEngineError};
use crate::ops::foundation::roaming_engine::{RoamingEngine, RoamingEngineError, RoamingPlan};

// 2026-04-09 CST: 这里补上 foundation 导航主线的正式 pipeline 入口，原因是当前集成测试已经把
// “question -> NavigationEvidence” 视为稳定契约，但源码里还没有对应模块，导致 foundation 主线无法形成闭环。
// 目的：用最小编排层把 route、roam、retrieve、assemble 串起来，先补齐通用导航内核，而不引入任何业务化逻辑。
#[derive(Debug, Clone)]
pub struct NavigationPipeline {
    router: CapabilityRouter,
    roaming_engine: RoamingEngine,
    retrieval_engine: RetrievalEngine,
    assembler: EvidenceAssembler,
    graph_store: KnowledgeGraphStore,
    roaming_plan_template: RoamingPlan,
    metadata_registry: MetadataRegistry,
}

// 2026-04-09 CST: 这里定义 pipeline 级错误边界，原因是 foundation 需要对上游暴露统一失败语义，
// 不能把 route、roam、retrieve 的内部错误类型直接泄漏给更高层调用方。
// 目的：先稳定“失败发生在哪一段”的契约，后续再按需要扩展观测字段或诊断信息。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavigationPipelineError {
    RouteFailed { question: String },
    RoamFailed { concept_ids: Vec<String> },
    RetrievalFailed { question: String },
    InvalidMetadataConstraint(MetadataRegistryError),
}

impl NavigationPipeline {
    // 2026-04-09 CST: 这里提供最小构造函数，原因是当前 foundation 集成测试只需要注入本体、图谱和漫游计划模板，
    // 不需要提前引入更重的 orchestrator、配置对象或外部依赖。
    // 目的：先固定 pipeline 的依赖边界，保持它只服务于通用导航主线。
    pub fn new(
        ontology_store: OntologyStore,
        graph_store: KnowledgeGraphStore,
        roaming_plan_template: RoamingPlan,
    ) -> Self {
        Self::new_with_metadata_registry(
            ontology_store,
            graph_store,
            roaming_plan_template,
            MetadataRegistry::new(),
        )
    }

    // 2026-04-09 CST: 这里补带 metadata registry 的 pipeline 构造器，原因是字段目录阶段需要让 route / roam / retrieve
    // 共用同一份显式字段注册表，而不是继续各自猜字段作用层级。
    // 目的：为通用 metadata 字段管理提供统一接入点，同时保持旧构造器可继续工作。
    pub fn new_with_metadata_registry(
        ontology_store: OntologyStore,
        graph_store: KnowledgeGraphStore,
        roaming_plan_template: RoamingPlan,
        metadata_registry: MetadataRegistry,
    ) -> Self {
        Self {
            router: CapabilityRouter::new(ontology_store.clone()),
            roaming_engine: RoamingEngine::new_with_metadata_registry(
                ontology_store,
                metadata_registry.clone(),
            ),
            retrieval_engine: RetrievalEngine::new(),
            assembler: EvidenceAssembler::new(),
            graph_store,
            roaming_plan_template,
            metadata_registry,
        }
    }

    // 2026-04-09 CST: 这里实现主链运行入口，原因是 foundation 现在缺的不是单点能力，而是把既有能力稳定串联的统一入口。
    // 目的：让调用方只传一个 NavigationRequest，就能得到结构化 NavigationEvidence 或清晰的阶段性错误。
    pub fn run(
        &self,
        request: &NavigationRequest,
    ) -> Result<NavigationEvidence, NavigationPipelineError> {
        let route = self
            .router
            .route(request)
            .map_err(|error| self.map_route_error(request, error))?;
        let route = self.constrain_route_to_template_seeds(request, route);
        let route = self.constrain_route_to_metadata_scope(request, route)?;
        if route.matched_concept_ids.is_empty() {
            return Err(NavigationPipelineError::RouteFailed {
                question: request.question.clone(),
            });
        }
        let scope = self
            .roaming_engine
            .roam(self.build_roaming_plan(request, &route.matched_concept_ids))
            .map_err(|error| self.map_roaming_error(&route.matched_concept_ids, error))?;
        let hits = self
            .retrieve(&request.question, &scope)
            .map_err(|error| self.map_retrieval_error(request, error))?;

        Ok(self.assembler.assemble(route, scope, hits))
    }

    // 2026-04-09 CST: 这里从模板计划派生实际漫游计划，原因是 route 产出的 matched concept ids 才是每次请求的真实种子，
    // 不能直接复用构造时写死的 seed 列表，否则 pipeline 会失去“按问题导航”的意义。
    // 目的：保留计划模板里的深度、关系类型和规模预算，只在运行时替换种子概念。
    fn build_roaming_plan(&self, request: &NavigationRequest, concept_ids: &[String]) -> RoamingPlan {
        let mut plan = self.roaming_plan_template.clone();
        plan.seed_concept_ids = concept_ids.to_vec();
        // 2026-04-09 CST: 这里把请求级 metadata scope 注入运行时 roaming plan，原因是方案B第二阶段要让
        // metadata 正式经过 RoamingPlan -> CandidateScope -> Retrieval 这条共享主线。
        // 目的：避免 metadata 继续以 retrieval 私有参数形态存在，先把 scope 合同做完整。
        plan.metadata_scope = request.metadata_scope.clone();
        plan
    }

    // 2026-04-09 CST: 这里在 pipeline 层补一次模板种子收敛，原因是 router 的职责是尽量完整识别问题中的概念线索，
    // 但 pipeline 的职责是把这些线索压缩到当前导航计划允许的主种子集合里，避免修饰性概念把主链带偏。
    // 目的：在不回退 router 多概念识别能力的前提下，让集成导航入口稳定遵循当前计划模板的主线约束。
    fn constrain_route_to_template_seeds(
        &self,
        request: &NavigationRequest,
        route: CapabilityRoute,
    ) -> CapabilityRoute {
        if self.roaming_plan_template.seed_concept_ids.is_empty() {
            return route;
        }

        let mut matched_concept_ids = Vec::new();
        let mut matched_terms = Vec::new();

        for (concept_id, matched_term) in route
            .matched_concept_ids
            .iter()
            .cloned()
            .zip(route.matched_terms.iter().cloned())
        {
            if self
                .roaming_plan_template
                .seed_concept_ids
                .iter()
                .any(|seed_concept_id| seed_concept_id == &concept_id)
            {
                matched_concept_ids.push(concept_id);
                matched_terms.push(matched_term);
            }
        }

        if matched_concept_ids.is_empty() && !request.required_concept_tags.is_empty() {
            return route;
        }

        CapabilityRoute {
            matched_concept_ids: if matched_concept_ids.is_empty() {
                self.roaming_plan_template.seed_concept_ids.clone()
            } else {
                matched_concept_ids
            },
            matched_terms,
        }
    }

    // 2026-04-09 CST: 这里在 pipeline 层补 metadata-aware concept 收敛，原因是方案B要求 metadata 不只留在 retrieval，
    // 还要在 route/template seed 合并之后真实影响进入 roaming 的 concept 集合。
    // 目的：把 concept metadata 过滤放在统一编排层，而不是交给调用方或业务模块自行拼接。
    fn constrain_route_to_metadata_scope(
        &self,
        request: &NavigationRequest,
        route: CapabilityRoute,
    ) -> Result<CapabilityRoute, NavigationPipelineError> {
        let constrained_concept_ids = if self.metadata_registry.is_empty() {
            MetadataScopeResolver::constrain_concept_ids(
                &self.router_ontology_store(),
                route.matched_concept_ids.as_slice(),
                &request.metadata_scope,
            )
        } else {
            MetadataScopeResolver::constrain_concept_ids_with_registry(
                &self.router_ontology_store(),
                route.matched_concept_ids.as_slice(),
                &request.metadata_scope,
                &self.metadata_registry,
            )
            .map_err(NavigationPipelineError::InvalidMetadataConstraint)?
        };
        let matched_term_pairs = route
            .matched_concept_ids
            .iter()
            .cloned()
            .zip(route.matched_terms.iter().cloned())
            .collect::<Vec<_>>();
        let matched_terms = constrained_concept_ids
            .iter()
            .filter_map(|concept_id| {
                matched_term_pairs
                    .iter()
                    .find(|(matched_concept_id, _)| matched_concept_id == concept_id)
                    .map(|(_, matched_term)| matched_term.clone())
            })
            .collect();

        Ok(CapabilityRoute {
            matched_concept_ids: constrained_concept_ids,
            matched_terms,
        })
    }

    // 2026-04-09 CST: 这里补一个只读 ontology store 访问桥，原因是 metadata-aware route 收敛仍然属于 pipeline 编排职责，
    // 但真正的 ontology 查询边界应继续通过 store 暴露，不要在此处复制 schema 访问。
    // 目的：让新增的 metadata 约束编排继续复用现有 foundation 边界。
    fn router_ontology_store(&self) -> OntologyStore {
        self.roaming_engine.ontology_store().clone()
    }

    // 2026-04-09 CST: 这里补 pipeline 内部 retrieval 桥接，原因是字段目录阶段需要在显式 registry 存在时切到
    // node-target aware 的 metadata 检索语义，但又要兼容旧阶段的无 registry 主线。
    // 目的：把两套阶段性行为收口在 pipeline 内部，不把兼容分支扩散给调用方。
    fn retrieve(
        &self,
        question: &str,
        scope: &crate::ops::foundation::roaming_engine::CandidateScope,
    ) -> Result<Vec<crate::ops::foundation::retrieval_engine::RetrievalHit>, RetrievalEngineError> {
        if self.metadata_registry.is_empty() {
            self.retrieval_engine.retrieve(question, scope, &self.graph_store)
        } else {
            self.retrieval_engine.retrieve_with_metadata_registry(
                question,
                scope,
                &self.graph_store,
                &self.metadata_registry,
            )
        }
    }

    // 2026-04-09 CST: 这里单独收口 route 错误映射，原因是当前上游只需要知道“哪个问题在路由阶段失败”，
    // 没必要耦合 router 内部错误枚举。
    // 目的：把 pipeline 的错误契约控制在通用、稳定、可断言的最小范围内。
    fn map_route_error(
        &self,
        request: &NavigationRequest,
        error: CapabilityRouterError,
    ) -> NavigationPipelineError {
        match error {
            CapabilityRouterError::NoConceptMatched { .. } => NavigationPipelineError::RouteFailed {
                question: request.question.clone(),
            },
        }
    }

    // 2026-04-09 CST: 这里单独收口 roaming 错误映射，原因是上游需要知道是哪组概念种子导致漫游失败，
    // 但不需要依赖 RoamingEngineError 的内部命名。
    // 目的：先把 candidate scope 生成失败的边界稳定下来，便于后续扩展诊断信息。
    fn map_roaming_error(
        &self,
        concept_ids: &[String],
        error: RoamingEngineError,
    ) -> NavigationPipelineError {
        match error {
            RoamingEngineError::NoSeedConcepts => NavigationPipelineError::RoamFailed {
                concept_ids: concept_ids.to_vec(),
            },
            RoamingEngineError::InvalidMetadataConstraint(error) => {
                NavigationPipelineError::InvalidMetadataConstraint(error)
            }
        }
    }

    // 2026-04-09 CST: 这里单独收口 retrieval 错误映射，原因是当前 foundation 主线只需要保留原始问题文本，
    // 作为“这次检索没找到证据”的最小诊断线索。
    // 目的：把 retrieval 的失败语义统一到 pipeline 错误上，避免上层再感知内部执行器类型。
    fn map_retrieval_error(
        &self,
        request: &NavigationRequest,
        error: RetrievalEngineError,
    ) -> NavigationPipelineError {
        match error {
            RetrievalEngineError::NoEvidenceFound { .. } => {
                NavigationPipelineError::RetrievalFailed {
                    question: request.question.clone(),
                }
            }
            RetrievalEngineError::InvalidMetadataConstraint(error) => {
                NavigationPipelineError::InvalidMetadataConstraint(error)
            }
        }
    }
}
