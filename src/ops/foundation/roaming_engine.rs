use std::collections::VecDeque;

use crate::ops::foundation::ontology_schema::OntologyRelationType;
use crate::ops::foundation::ontology_store::OntologyStore;

// 2026-04-07 CST: 这里定义漫游计划，原因是 roaming 不应该直接消费裸参数列表，
// 而是要把种子概念、允许关系、深度和规模预算收敛成独立输入对象。
// 目的：先固定 Task 6 的输入边界，让后续主链可以稳定从 route 过渡到 candidate scope。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoamingPlan {
    pub seed_concept_ids: Vec<String>,
    pub allowed_relation_types: Vec<OntologyRelationType>,
    pub max_depth: usize,
    pub max_concepts: usize,
}

impl RoamingPlan {
    // 2026-04-07 CST: 这里提供最小计划构造函数，原因是当前 TDD 只需要稳定表达种子概念集合，
    // 其余预算参数先给出保守默认值，避免样板代码在测试里膨胀。
    // 目的：让 Task 6 的样本计划构造保持简洁，同时保留链式补充配置的扩展空间。
    pub fn new(seed_concept_ids: Vec<&str>) -> Self {
        Self {
            seed_concept_ids: seed_concept_ids.into_iter().map(str::to_string).collect(),
            allowed_relation_types: Vec::new(),
            max_depth: 0,
            max_concepts: usize::MAX,
        }
    }

    // 2026-04-07 CST: 这里补允许关系链式配置，原因是 roam 的核心约束之一
    // 就是只能沿指定 relation type 扩展，不能把所有邻接概念一股脑放进候选域。
    // 目的：让测试和后续调用都能显式声明关系白名单，稳定主链行为。
    pub fn with_allowed_relation_types(
        mut self,
        allowed_relation_types: Vec<OntologyRelationType>,
    ) -> Self {
        self.allowed_relation_types = allowed_relation_types;
        self
    }

    // 2026-04-07 CST: 这里补最大深度链式配置，原因是 foundation 主链必须受控扩展，
    // 不能让概念漫游无限蔓延到无关节点。
    // 目的：把 depth budget 显式放进计划对象，方便测试和后续 orchestrator 统一控制。
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }

    // 2026-04-07 CST: 这里补最大概念数链式配置，原因是候选概念规模过大时，
    // 后续 retrieval 输入会迅速失控，因此需要在 roaming 这一层先截住。
    // 目的：把 candidate budget 直接作为漫游计划的一部分固定下来。
    pub fn with_max_concepts(mut self, max_concepts: usize) -> Self {
        self.max_concepts = max_concepts;
        self
    }
}

// 2026-04-07 CST: 这里定义漫游步骤，原因是后续 evidence assembly 需要知道
// 候选概念范围是沿哪条关系路径扩展出来的，而不是只拿一组平铺 concept ids。
// 目的：先把路径信息结构化保留下来，为后续主链继续串接做准备。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoamingStep {
    pub from_concept_id: String,
    pub to_concept_id: String,
    pub relation_type: OntologyRelationType,
    pub depth: usize,
}

// 2026-04-07 CST: 这里定义候选范围，原因是 roaming 的输出应该同时包含
// 候选概念集合和生成它们的路径，而不是只返回一个扁平概念数组。
// 目的：让 retrieval 能消费范围，让 evidence assembly 能消费路径，保持主链职责清晰。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateScope {
    pub concept_ids: Vec<String>,
    pub path: Vec<RoamingStep>,
}

// 2026-04-07 CST: 这里定义漫游错误，原因是没有有效种子概念时继续漫游没有意义，
// 应该在这一层尽早失败，而不是让下游模块收到空范围后再猜测发生了什么。
// 目的：把 roaming 的最小失败边界显式化，方便后续主链清晰处理。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoamingEngineError {
    NoSeedConcepts,
}

// 2026-04-07 CST: 这里把 roaming engine 设计成只依赖 ontology store 的受控扩展器，
// 原因是 Task 6 还处于 ontology-lite 主链阶段，不应该提前接入 graph store 或 retrieval。
// 目的：继续坚持 ontology -> roaming -> retrieval 的架构顺序。
#[derive(Debug, Clone)]
pub struct RoamingEngine {
    ontology_store: OntologyStore,
}

impl RoamingEngine {
    // 2026-04-07 CST: 这里新增 roaming engine 构造函数，原因是漫游阶段需要显式持有
    // ontology store，后续即使扩展策略变化，也能保持调用方式稳定。
    // 目的：把 Task 6 的依赖边界固定在 ontology store，而不是散落的工具函数。
    pub fn new(ontology_store: OntologyStore) -> Self {
        Self { ontology_store }
    }

    // 2026-04-07 CST: 这里实现受限 BFS 漫游入口，原因是当前 foundation 主线需要
    // 在关系白名单、深度预算和概念数预算下，从种子概念稳定扩展出候选概念范围。
    // 目的：用最小可用算法把 route 与 retrieval 之间的 candidate scope 补起来。
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

            if !concept_ids.iter().any(|concept_id| concept_id == &seed_concept_id) {
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
