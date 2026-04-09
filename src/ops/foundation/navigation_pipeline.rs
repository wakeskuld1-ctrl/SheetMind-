use crate::ops::foundation::capability_router::{
    CapabilityRoute, CapabilityRouter, CapabilityRouterError, NavigationRequest,
};
use crate::ops::foundation::evidence_assembler::{EvidenceAssembler, NavigationEvidence};
use crate::ops::foundation::knowledge_graph_store::KnowledgeGraphStore;
use crate::ops::foundation::metadata_registry::{MetadataRegistry, MetadataRegistryError};
use crate::ops::foundation::metadata_scope_resolver::MetadataScopeResolver;
use crate::ops::foundation::ontology_schema::OntologyRelationType;
use crate::ops::foundation::ontology_store::OntologyStore;
use crate::ops::foundation::retrieval_engine::{RetrievalEngine, RetrievalEngineError};
use crate::ops::foundation::roaming_engine::{RoamingEngine, RoamingEngineError, RoamingPlan};

// 2026-04-09 CST: 这里补回 pipeline 配置对象，原因是本地分支已经把“关系白名单 / 深度 / 概念预算”抽成显式底座合同，
// foundation 合并时不能因为 metadata 主线接入就把这层可调入口丢回硬编码。
// 目的：让 navigation pipeline 同时保留 metadata-aware 主线和既有可调策略边界，避免回退成只能靠模板手改的状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavigationPipelineConfig {
    pub allowed_relation_types: Vec<OntologyRelationType>,
    pub max_depth: usize,
    pub max_concepts: usize,
}

impl Default for NavigationPipelineConfig {
    // 2026-04-09 CST: 这里沿用本地分支已经稳定下来的默认漫游策略，原因是这套默认值已经被 foundation 闭环测试消费过，
    // 合并阶段不应静默改变默认搜索范围。
    // 目的：确保 metadata 编排并入后，未显式传配置的调用方仍保持既有保守漫游行为。
    fn default() -> Self {
        Self {
            allowed_relation_types: vec![
                OntologyRelationType::DependsOn,
                OntologyRelationType::Supports,
                OntologyRelationType::References,
                OntologyRelationType::AdjacentTo,
            ],
            max_depth: 1,
            max_concepts: 8,
        }
    }
}

impl NavigationPipelineConfig {
    // 2026-04-09 CST: 这里保留 relation whitelist 覆盖入口，原因是 foundation 侧仍需要低成本调窄漫游边界，
    // 不应该每次都退回手工改 RoamingPlan fixture。
    // 目的：维持本地分支已经形成的最小配置化接口。
    pub fn with_allowed_relation_types(
        mut self,
        allowed_relation_types: Vec<OntologyRelationType>,
    ) -> Self {
        self.allowed_relation_types = allowed_relation_types;
        self
    }

    // 2026-04-09 CST: 这里保留 max_depth 覆盖入口，原因是深度预算仍是 foundation 漫游最核心的保守控制阀门。
    // 目的：让集成测试和后续编排层可以显式声明“零深度 / 浅漫游 / 默认漫游”。
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }

    // 2026-04-09 CST: 这里保留 max_concepts 覆盖入口，原因是概念预算仍然决定 retrieval 候选域是否可控，
    // 合并 metadata 主线时不能把这个预算入口丢掉。
    // 目的：让 foundation 主线继续具备最小候选域治理能力。
    pub fn with_max_concepts(mut self, max_concepts: usize) -> Self {
        self.max_concepts = max_concepts;
        self
    }
}

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
    config: NavigationPipelineConfig,
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

    // 2026-04-09 CST: 这里补回基于显式配置的构造器，原因是本地 foundation 主线已经形成“默认值 + 可覆盖值”的调用习惯，
    // 合并后不能只剩模板型构造器，否则原有配置化回归会直接失效。
    // 目的：继续支持无模板 seed 的通用导航入口，由 router 命中结果在运行时补齐真正的 seed concept。
    pub fn new_with_config(
        ontology_store: OntologyStore,
        graph_store: KnowledgeGraphStore,
        config: NavigationPipelineConfig,
    ) -> Self {
        Self::new_with_config_and_metadata_registry(
            ontology_store,
            graph_store,
            config,
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
        let config = NavigationPipelineConfig {
            // 2026-04-09 CST: 这里把模板里的漫游参数同步回 config，原因是 metadata 分支的主构造器以 RoamingPlan 为入口，
            // 合并后需要保证模板和显式配置视图描述的是同一份运行时策略。
            // 目的：避免同一个 pipeline 同时持有两套互相漂移的漫游预算定义。
            allowed_relation_types: roaming_plan_template.allowed_relation_types.clone(),
            max_depth: roaming_plan_template.max_depth,
            max_concepts: roaming_plan_template.max_concepts,
        };
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
            config,
        }
    }

    // 2026-04-09 CST: 这里补“配置 + registry”并存构造器，原因是合并后 foundation 需要同时支持 metadata 目录治理
    // 和原有可调漫游策略，不能逼调用方二选一。
    // 目的：把配置化入口和 registry 入口收敛到同一层，避免后续测试或上层编排重复拼装模板。
    pub fn new_with_config_and_metadata_registry(
        ontology_store: OntologyStore,
        graph_store: KnowledgeGraphStore,
        config: NavigationPipelineConfig,
        metadata_registry: MetadataRegistry,
    ) -> Self {
        let roaming_plan_template = RoamingPlan::new(Vec::new())
            .with_allowed_relation_types(config.allowed_relation_types.clone())
            .with_max_depth(config.max_depth)
            .with_max_concepts(config.max_concepts);

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
            config,
        }
    }

    // 2026-04-09 CST: 这里补回字符串问题入口包装器，原因是本地分支已有调用面直接传 question 文本，
    // 合并后不应强制所有旧调用点立刻切到 NavigationRequest。
    // 目的：在不牺牲新 request 合同的前提下保住旧入口兼容性。
    pub fn run_question(&self, question: &str) -> Result<NavigationEvidence, NavigationPipelineError> {
        self.run(&NavigationRequest::new(question))
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
        // 2026-04-09 CST: 这里把显式 config 再投影回运行时 plan，原因是合并后同一个 pipeline 既可能来自模板入口，
        // 也可能来自本地分支保留的 config 入口，运行时必须统一以 config 视图收敛。
        // 目的：确保 metadata 并入后，旧配置化调用仍能稳定影响 relation whitelist / depth / concept budget。
        plan.allowed_relation_types = self.config.allowed_relation_types.clone();
        plan.max_depth = self.config.max_depth;
        plan.max_concepts = self.config.max_concepts;
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
