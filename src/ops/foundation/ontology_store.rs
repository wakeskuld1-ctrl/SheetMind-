use crate::ops::foundation::ontology_schema::{
    OntologyConcept, OntologyRelationType, OntologySchema,
};

// 2026-04-07 CST: 这里把 ontology store 收敛为只读查询壳层，原因是 schema 负责定义与建索引，
// store 负责稳定暴露查询入口，避免后续 roaming / router 直接耦合 schema 的内部结构。
// 目的：先把 foundation 主链上的“概念读取 + 关系邻接读取”落到独立模块，后面再渐进扩展索引策略。
#[derive(Debug, Clone)]
pub struct OntologyStore {
    schema: OntologySchema,
}

impl OntologyStore {
    // 2026-04-07 CST: 这里增加 store 构造函数，原因是 Task 3 需要一个明确的只读存储对象来持有 schema，
    // 不能让调用方到处直接传裸 schema，否则后续很难在不改调用面的前提下替换查询策略。
    // 目的：先稳定 foundation 内核里 store 的对象边界，让后续 roaming / router 都围绕统一入口接入。
    pub fn new(schema: OntologySchema) -> Self {
        Self { schema }
    }

    // 2026-04-07 CST: 这里把 concept id 查询委托给 schema，原因是 lookup 归一化和别名索引
    // 已经在 schema 内聚，store 不应该复制同一套规则。
    // 目的：让上层只依赖 store 接口，同时继续复用 schema 已验证通过的 name / alias 索引行为。
    pub fn find_concept_id(&self, raw: &str) -> Option<&str> {
        self.schema.find_concept_id(raw)
    }

    // 2026-04-07 CST: 这里提供按 concept id 读取概念详情的只读入口，原因是后续 router、
    // roaming 和 evidence 组装都需要安全读取概念元信息，但不该知道 schema 的索引细节。
    // 目的：把概念实体读取统一收口到 store，降低上层对 schema 内部存储方式的依赖。
    pub fn concept(&self, concept_id: &str) -> Option<&OntologyConcept> {
        self.schema.concept(concept_id)
    }

    // 2026-04-07 CST: 这里先按内存遍历实现邻接概念查询，原因是 Task 3 明确要求先做最小可用实现，
    // 不提前引入复杂关系索引，避免 foundation 底座在这一层过早复杂化。
    // 目的：给下一步 roaming 提供稳定的 relation type 过滤入口，同时保持实现简单、可测、可替换。
    pub fn related_concepts<'a>(
        &'a self,
        concept_id: &str,
        allowed_relation_types: &[OntologyRelationType],
    ) -> Vec<&'a str> {
        self.schema
            .relations
            .iter()
            .filter(|relation| relation.from_concept_id == concept_id)
            .filter(|relation| allowed_relation_types.contains(&relation.relation_type))
            .map(|relation| relation.to_concept_id.as_str())
            .collect()
    }
}
