use crate::ops::foundation::ontology_schema::{
    OntologyConcept, OntologyRelationType, OntologySchema,
};

// 2026-04-08 CST: 这里把 ontology store 收口成只读查询层，原因是 schema 已经负责 concept/alias
// 索引构建，而上层模块不应该继续直接耦合 schema 的内部结构和遍历方式。
// 目的：为后续 router 和 roaming 提供统一查询入口，保持 foundation 主链边界清晰。
#[derive(Debug, Clone)]
pub struct OntologyStore {
    schema: OntologySchema,
}

impl OntologyStore {
    // 2026-04-08 CST: 这里提供最小构造函数，原因是第一阶段需要明确一个 store 对象来持有 schema，
    // 否则后续模块仍会继续接收裸 schema，后面再收边界的成本会更高。
    // 目的：先把 “schema 被 store 持有” 的依赖关系固定下来。
    pub fn new(schema: OntologySchema) -> Self {
        Self { schema }
    }

    // 2026-04-08 CST: 这里委托原始文本到 concept id 的查询，原因是 schema 已经承载 alias/name
    // 归一化索引，store 不应该复制这套规则，否则后续修改会出现两套行为。
    // 目的：让上层通过 store 读取 concept id，同时复用 schema 已验证的查找能力。
    pub fn find_concept_id(&self, raw: &str) -> Option<&str> {
        self.schema.find_concept_id(raw)
    }

    // 2026-04-08 CST: 这里暴露按 concept id 读取实体的接口，原因是后续 router 和 roaming
    // 都需要安全读取 concept 元信息，但不应该知道 schema 索引的实现细节。
    // 目的：把 concept 读取统一收口到 store 层。
    pub fn concept(&self, concept_id: &str) -> Option<&OntologyConcept> {
        self.schema.concept(concept_id)
    }

    // 2026-04-08 CST: 这里先按内存遍历实现关系邻接查询，原因是第一阶段只追求最小可用，
    // 不应在没有性能证据时提前引入复杂关系索引。
    // 目的：给 roaming 提供稳定的 relation-type 过滤入口，同时保持实现简单、透明、可测。
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
