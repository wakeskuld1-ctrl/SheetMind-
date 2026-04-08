use crate::ops::foundation::knowledge_graph_store::KnowledgeGraphStore;
use crate::ops::foundation::knowledge_record::{KnowledgeEdge, KnowledgeNode};
use crate::ops::foundation::ontology_schema::{
    OntologyConcept, OntologyRelation, OntologySchema, OntologySchemaError,
};
use crate::ops::foundation::ontology_store::OntologyStore;
use serde::{Deserialize, Serialize};

// 2026-04-08 CST: 这里定义标准知识包，原因是 phase 2 第一阶段要先把“什么是可持久化知识单元”
// 固化为通用结构，而不是直接把内存 store 当落盘格式。
// 目的：为持久化、导入导出和跨域适配提供统一基础载体。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeBundle {
    pub schema_version: String,
    pub concepts: Vec<OntologyConcept>,
    pub relations: Vec<OntologyRelation>,
    pub nodes: Vec<KnowledgeNode>,
    pub edges: Vec<KnowledgeEdge>,
}

impl KnowledgeBundle {
    // 2026-04-08 CST: 这里提供最小 bundle 构造函数，原因是当前阶段只需要固定原始 ontology 与 graph 数据载体，
    // 不应过早引入额外包级元数据或复杂 builder。
    // 目的：让测试与后续仓储实现都围绕一套轻量标准包结构展开。
    pub fn new(
        schema_version: impl Into<String>,
        concepts: Vec<OntologyConcept>,
        relations: Vec<OntologyRelation>,
        nodes: Vec<KnowledgeNode>,
        edges: Vec<KnowledgeEdge>,
    ) -> Self {
        Self {
            schema_version: schema_version.into(),
            concepts,
            relations,
            nodes,
            edges,
        }
    }

    // 2026-04-08 CST: 这里提供 bundle -> ontology store 重建入口，原因是持久化包最终仍要回到
    // foundation 已有查询层，不应让调用方自己重复拼 schema/store。
    // 目的：把标准包与现有 ontology 查询层衔接起来。
    pub fn to_ontology_store(&self) -> Result<OntologyStore, OntologySchemaError> {
        Ok(OntologyStore::new(OntologySchema::new(
            self.concepts.clone(),
            self.relations.clone(),
        )?))
    }

    // 2026-04-08 CST: 这里提供 bundle -> graph store 重建入口，原因是标准包既要能落盘，
    // 也要能恢复成当前 retrieval 直接消费的图查询层。
    // 目的：把标准包与现有 graph 查询层衔接起来。
    pub fn to_graph_store(&self) -> KnowledgeGraphStore {
        KnowledgeGraphStore::new(self.nodes.clone(), self.edges.clone())
    }
}
