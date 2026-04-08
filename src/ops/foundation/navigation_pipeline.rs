use crate::ops::foundation::capability_router::{
    CapabilityRouter, CapabilityRouterError, NavigationRequest,
};
use crate::ops::foundation::evidence_assembler::{EvidenceAssembler, NavigationEvidence};
use crate::ops::foundation::knowledge_graph_store::KnowledgeGraphStore;
use crate::ops::foundation::ontology_schema::OntologyRelationType;
use crate::ops::foundation::ontology_store::OntologyStore;
use crate::ops::foundation::retrieval_engine::{RetrievalEngine, RetrievalEngineError};
use crate::ops::foundation::roaming_engine::{RoamingEngine, RoamingEngineError, RoamingPlan};

// 2026-04-08 CST: 这里新增 pipeline 最小配置对象，原因是当前最小闭环虽然已经可运行，
// 2026-04-08 CST: 但 relation whitelist、漫游深度和概念预算仍硬编码在实现里，不利于继续把底座从样板推进到可调内核。
// 2026-04-08 CST: 目的：先把漫游侧最关键的三个策略值提升成显式合同，同时保持默认行为完全兼容。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavigationPipelineConfig {
    pub allowed_relation_types: Vec<OntologyRelationType>,
    pub max_depth: usize,
    pub max_concepts: usize,
}

impl Default for NavigationPipelineConfig {
    // 2026-04-08 CST: 这里把现有硬编码策略收进默认配置，原因是 `new()` 的既有行为不能因为配置化而改变。
    // 2026-04-08 CST: 目的：确保本轮配置化是“增加可调能力”，而不是“悄悄改默认策略”。
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
    // 2026-04-08 CST: 这里补 relation whitelist 覆盖入口，原因是当前第一节配置化
    // 2026-04-08 CST: 只需要支持最直接的漫游范围控制，不应该过早引入 profile 或更复杂配置层。
    // 2026-04-08 CST: 目的：让测试和后续调用方能显式声明允许关系集合。
    pub fn with_allowed_relation_types(
        mut self,
        allowed_relation_types: Vec<OntologyRelationType>,
    ) -> Self {
        self.allowed_relation_types = allowed_relation_types;
        self
    }

    // 2026-04-08 CST: 这里补最大深度覆盖入口，原因是 depth 是当前漫游控制里最核心的范围阀门之一。
    // 2026-04-08 CST: 目的：让 pipeline 可以在不改内部实现的前提下切换保守或更宽的漫游深度。
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }

    // 2026-04-08 CST: 这里补最大概念数覆盖入口，原因是候选域预算同样属于底座可调策略。
    // 2026-04-08 CST: 目的：把候选规模也纳入同一份最小配置合同，便于后续排查“算法问题还是预算问题”。
    pub fn with_max_concepts(mut self, max_concepts: usize) -> Self {
        self.max_concepts = max_concepts;
        self
    }
}

// 2026-04-08 CST: 这里定义 pipeline 错误抬升枚举，原因是当前最小闭环最需要保留的就是
// 2026-04-08 CST: “到底失败在 route、roam 还是 retrieve” 这层阶段边界，而不是过早统一成模糊字符串错误。
// 2026-04-08 CST: 目的：让测试和后续交接都能稳定断言失败发生在哪一层。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavigationPipelineError {
    Route(CapabilityRouterError),
    Roam(RoamingEngineError),
    Retrieve(RetrievalEngineError),
}

// 2026-04-08 CST: 这里新增 foundation 侧最小 pipeline 入口，原因是当前模块虽然已经分段具备，
// 2026-04-08 CST: 但没有正式承接点把 route / roam / retrieve / assemble 串起来。
// 2026-04-08 CST: 目的：提供一个纯内存、业务无关、可测试的最小导航闭环入口。
#[derive(Debug, Clone)]
pub struct NavigationPipeline {
    capability_router: CapabilityRouter,
    roaming_engine: RoamingEngine,
    retrieval_engine: RetrievalEngine,
    evidence_assembler: EvidenceAssembler,
    graph_store: KnowledgeGraphStore,
    config: NavigationPipelineConfig,
}

impl NavigationPipeline {
    // 2026-04-08 CST: 这里通过 ontology store 和 graph store 构造 pipeline，原因是当前 foundation
    // 2026-04-08 CST: 主链只需要纯内存样本就能验证闭环，不应该提前把 CLI、数据库或运行时上下文拉进来。
    // 2026-04-08 CST: 目的：先把底座正式入口固定为最小依赖集合，后续扩展时不改外部调用形状。
    pub fn new(ontology_store: OntologyStore, graph_store: KnowledgeGraphStore) -> Self {
        Self::new_with_config(
            ontology_store,
            graph_store,
            NavigationPipelineConfig::default(),
        )
    }

    // 2026-04-08 CST: 这里新增带配置构造函数，原因是 Task 10 第一节的目标就是让调用方
    // 2026-04-08 CST: 能显式覆盖漫游策略，同时保持默认构造方式继续可用。
    // 2026-04-08 CST: 目的：把“默认值”和“自定义值”分成两个清晰入口，避免调用面含糊。
    pub fn new_with_config(
        ontology_store: OntologyStore,
        graph_store: KnowledgeGraphStore,
        config: NavigationPipelineConfig,
    ) -> Self {
        Self {
            capability_router: CapabilityRouter::new(ontology_store.clone()),
            roaming_engine: RoamingEngine::new(ontology_store),
            retrieval_engine: RetrievalEngine::new(),
            evidence_assembler: EvidenceAssembler::new(),
            graph_store,
            config,
        }
    }

    // 2026-04-08 CST: 这里实现最小 run 入口，原因是 Task 9 的目标就是把问题文本正式收敛成
    // 2026-04-08 CST: 结构化证据，而不是继续停留在“测试里手工串几个模块”的状态。
    // 2026-04-08 CST: 目的：用固定且保守的漫游策略先跑通 foundation 主链闭环，不扩大成通用配置系统。
    pub fn run(&self, question: &str) -> Result<NavigationEvidence, NavigationPipelineError> {
        let request = NavigationRequest::new(question);
        let route = self
            .capability_router
            .route(&request)
            .map_err(NavigationPipelineError::Route)?;

        let seed_concept_ids = route
            .matched_concept_ids
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let scope = self
            .roaming_engine
            .roam(
                RoamingPlan::new(seed_concept_ids)
                    .with_allowed_relation_types(self.config.allowed_relation_types.clone())
                    .with_max_depth(self.config.max_depth)
                    .with_max_concepts(self.config.max_concepts),
            )
            .map_err(NavigationPipelineError::Roam)?;

        let hits = self
            .retrieval_engine
            .retrieve(question, &scope, &self.graph_store)
            .map_err(NavigationPipelineError::Retrieve)?;

        Ok(self.evidence_assembler.assemble(route, scope, hits))
    }
}
