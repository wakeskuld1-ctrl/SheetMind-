use std::collections::VecDeque;

use crate::ops::foundation::ontology_schema::OntologyRelationType;
use crate::ops::foundation::ontology_store::OntologyStore;

// 2026-04-08 CST: 这里定义漫游计划，原因是 roaming 不应直接接收裸参数列表，
// 否则后续 route -> roam -> retrieval 主链会持续传递松散参数，边界会越来越模糊。
// 目的：把种子概念、允许关系、深度预算与规模预算收口成稳定输入对象。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoamingPlan {
    pub seed_concept_ids: Vec<String>,
    pub allowed_relation_types: Vec<OntologyRelationType>,
    pub max_depth: usize,
    pub max_concepts: usize,
}

impl RoamingPlan {
    // 2026-04-08 CST: 这里提供最小计划构造函数，原因是当前阶段只需要稳定表达 seed 集合，
    // 其余预算采用保守默认值即可，避免测试样板代码膨胀。
    // 目的：让 Task 6 的调用方式先保持简洁，同时为链式补充配置预留出口。
    pub fn new(seed_concept_ids: Vec<&str>) -> Self {
        Self {
            seed_concept_ids: seed_concept_ids.into_iter().map(str::to_string).collect(),
            allowed_relation_types: Vec::new(),
            max_depth: 0,
            max_concepts: usize::MAX,
        }
    }

    // 2026-04-08 CST: 这里链式设置允许关系，原因是 relation-type 白名单是 roaming 的核心约束，
    // 不应埋在内部默认策略里让上层无法显式声明。
    // 目的：保证候选域扩展规则透明、可测、可控。
    pub fn with_allowed_relation_types(
        mut self,
        allowed_relation_types: Vec<OntologyRelationType>,
    ) -> Self {
        self.allowed_relation_types = allowed_relation_types;
        self
    }

    // 2026-04-08 CST: 这里链式设置最大深度，原因是受限 BFS 的第一道刹车就是 depth budget，
    // 没有这一层约束，候选域会沿关系持续蔓延。
    // 目的：把深度预算显式固化到计划对象中，便于上层统一控制。
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }

    // 2026-04-08 CST: 这里链式设置最大概念数，原因是 retrieval 的输入规模需要由 roaming 先控制，
    // 不应把无限扩张后的代价推给后续模块。
    // 目的：保证候选概念规模始终可预测。
    pub fn with_max_concepts(mut self, max_concepts: usize) -> Self {
        self.max_concepts = max_concepts;
        self
    }
}

// 2026-04-08 CST: 这里定义漫游路径步骤，原因是后续 evidence assembly 需要知道候选域是如何展开的，
// 不能只返回一组扁平 concept ids。
// 目的：把扩展路径结构化保留下来，给后续解释链路留接口。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoamingStep {
    pub from_concept_id: String,
    pub to_concept_id: String,
    pub relation_type: OntologyRelationType,
    pub depth: usize,
}

// 2026-04-08 CST: 这里定义候选范围，原因是 roaming 的输出同时要服务 retrieval 和后续路径解释，
// 只返回概念列表会丢失扩展来源。
// 目的：统一收口 concept 集合与 path 轨迹。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateScope {
    pub concept_ids: Vec<String>,
    pub path: Vec<RoamingStep>,
}

// 2026-04-08 CST: 这里定义漫游失败边界，原因是如果连一个有效 seed 都没有，
// 继续扩展没有意义，应该在这一层直接失败。
// 目的：让上层可以明确识别“起点为空”的失败语义。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoamingEngineError {
    NoSeedConcepts,
}

// 2026-04-08 CST: 这里把 roaming engine 设计成只依赖 ontology store，原因是当前阶段仍处于
// ontology -> roaming -> retrieval 的内核补齐阶段，不应提前耦合 graph store 或业务链。
// 目的：维持 foundation 子链职责清晰。
#[derive(Debug, Clone)]
pub struct RoamingEngine {
    ontology_store: OntologyStore,
}

impl RoamingEngine {
    // 2026-04-08 CST: 这里提供构造函数，原因是 roaming 需要显式持有 ontology store，
    // 避免再次退回为一组散落工具函数。
    // 目的：把依赖边界固定住，便于后续 pipeline 复用。
    pub fn new(ontology_store: OntologyStore) -> Self {
        Self { ontology_store }
    }

    // 2026-04-08 CST: 这里实现最小受限 BFS，原因是 Task 6 当前只需要把 seed 概念在白名单关系、
    // 深度预算和规模预算内稳定扩展成候选域，不需要更复杂的评分或排序策略。
    // 目的：先补齐 route 与 retrieval 之间缺失的 roaming 层主契约。
    pub fn roam(&self, plan: RoamingPlan) -> Result<CandidateScope, RoamingEngineError> {
        let mut concept_ids = Vec::new();
        let mut queue = VecDeque::new();

        for seed_concept_id in plan.seed_concept_ids {
            if self.ontology_store.concept(&seed_concept_id).is_none() {
                continue;
            }

            if concept_ids.len() >= plan.max_concepts {
                break;
            }

            if !concept_ids
                .iter()
                .any(|concept_id| concept_id == &seed_concept_id)
            {
                concept_ids.push(seed_concept_id.clone());
                queue.push_back((seed_concept_id, 0usize));
            }
        }

        if concept_ids.is_empty() {
            return Err(RoamingEngineError::NoSeedConcepts);
        }

        let mut path = Vec::new();

        while let Some((current_concept_id, current_depth)) = queue.pop_front() {
            if current_depth >= plan.max_depth {
                continue;
            }

            if concept_ids.len() >= plan.max_concepts {
                break;
            }

            for relation_type in &plan.allowed_relation_types {
                let neighbors = self
                    .ontology_store
                    .related_concepts(&current_concept_id, std::slice::from_ref(relation_type));

                for neighbor_concept_id in neighbors {
                    if concept_ids.len() >= plan.max_concepts {
                        break;
                    }

                    if concept_ids
                        .iter()
                        .any(|concept_id| concept_id == neighbor_concept_id)
                    {
                        continue;
                    }

                    let next_depth = current_depth + 1;
                    concept_ids.push(neighbor_concept_id.to_string());
                    path.push(RoamingStep {
                        from_concept_id: current_concept_id.clone(),
                        to_concept_id: neighbor_concept_id.to_string(),
                        relation_type: relation_type.clone(),
                        depth: next_depth,
                    });
                    queue.push_back((neighbor_concept_id.to_string(), next_depth));
                }
            }
        }

        Ok(CandidateScope { concept_ids, path })
    }
}
