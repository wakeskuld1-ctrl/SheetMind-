use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ops::foundation::ontology_schema::OntologyRelationType;

// 2026-04-08 CST: 这里定义证据引用模型，原因是 node 命中后最终必须能追溯到来源与定位，
// 不能让 source_ref / locator 这类字段继续散落在更高层逻辑里。
// 目的：先把证据引用收口成最小统一结构，为 retrieval 和 evidence assembly 共用。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceRef {
    pub source_ref: String,
    pub locator: String,
}

impl EvidenceRef {
    // 2026-04-08 CST: 这里提供最小构造函数，原因是当前阶段只需要表达“来源是谁、定位在哪”，
    // 不需要过早引入置信度、时间戳或其他更重元数据。
    // 目的：让测试样本与后续知识节点装配保持最小而稳定。
    pub fn new(source_ref: impl Into<String>, locator: impl Into<String>) -> Self {
        Self {
            source_ref: source_ref.into(),
            locator: locator.into(),
        }
    }
}

// 2026-04-08 CST: 这里定义知识节点模型，原因是 foundation 检索命中的应该是 node，
// 而不是直接拿 ontology concept 当证据承载体。
// 目的：先把 title / body / concept_ids / evidence_refs 这组最小字段稳定下来。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeNode {
    pub id: String,
    pub title: String,
    pub body: String,
    pub concept_ids: Vec<String>,
    pub metadata: BTreeMap<String, String>,
    pub evidence_refs: Vec<EvidenceRef>,
}

impl KnowledgeNode {
    // 2026-04-08 CST: 这里提供节点最小构造函数，原因是当前阶段 retrieval 只会消费标题与正文，
    // 不应在没有明确需求时一次性塞入更多业务字段。
    // 目的：先把节点骨架固定为足够简单、足够可测的最小形状。
    pub fn new(id: impl Into<String>, title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            body: body.into(),
            concept_ids: Vec::new(),
            metadata: BTreeMap::new(),
            evidence_refs: Vec::new(),
        }
    }

    // 2026-04-08 CST: 这里支持链式挂接 concept id，原因是一个 node 可以同时服务多个 concept，
    // graph store 需要依赖这份关联做 concept -> node 的候选聚合。
    // 目的：把 concept 关联显式保存在 node 内部，避免 graph store 额外维护重复映射输入。
    pub fn with_concept_id(mut self, concept_id: impl Into<String>) -> Self {
        self.concept_ids.push(concept_id.into());
        self
    }

    // 2026-04-08 CST: 这里新增 metadata 链式挂载能力，原因是 phase 2 第一阶段要把通用过滤契约前移到标准节点模型，
    // 不能等到后续检索层再临时发明一套节点外的 metadata 容器。
    // 目的：让任何业务域都先适配到统一 node metadata，再由 foundation 提供标准过滤能力。
    pub fn with_metadata_entry(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    // 2026-04-08 CST: 这里支持链式挂接证据引用，原因是 node 命中后上层需要直接取回原始证据，
    // 不应在 evidence assembly 阶段再反查一次别的来源。
    // 目的：把 node 与证据的一对多关系固定在 record 层。
    pub fn with_evidence_ref(mut self, evidence_ref: EvidenceRef) -> Self {
        self.evidence_refs.push(evidence_ref);
        self
    }
}

// 2026-04-08 CST: 这里定义知识边模型，原因是 node 之间的图谱关系与 ontology concept 关系
// 不是同一职责层，必须单独留出图谱关系载体。
// 目的：为 graph store 出边读取和后续证据路径保留打基础。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeEdge {
    pub from_node_id: String,
    pub to_node_id: String,
    pub relation_type: OntologyRelationType,
}

impl KnowledgeEdge {
    // 2026-04-08 CST: 这里提供最小知识边构造函数，原因是当前阶段只需要能稳定表达边关系，
    // 还不需要权重、置信度等扩展字段。
    // 目的：让测试和后续图谱装配统一使用这套轻量 API。
    pub fn new(
        from_node_id: impl Into<String>,
        to_node_id: impl Into<String>,
        relation_type: OntologyRelationType,
    ) -> Self {
        Self {
            from_node_id: from_node_id.into(),
            to_node_id: to_node_id.into(),
            relation_type,
        }
    }
}
