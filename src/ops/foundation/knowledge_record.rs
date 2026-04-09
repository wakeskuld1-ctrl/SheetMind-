use std::collections::BTreeMap;

use crate::ops::foundation::ontology_schema::OntologyRelationType;

// 2026-04-07 CST: 这里定义证据引用模型，原因是 foundation 图谱节点后续需要把命中的原始出处
// 以统一结构挂在节点上，而不是把 source/locator 这种字段散落在各层调用中。
// 目的：先固定最小证据载体，让 retrieval 和 evidence assembly 后面都能复用同一份基础结构。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceRef {
    pub source_ref: String,
    pub locator: String,
}

impl EvidenceRef {
    // 2026-04-07 CST: 这里提供最小证据构造函数，原因是当前 TDD 测试需要直接声明节点证据来源，
    // 没必要为了 Task 4 过早引入更多元数据或复杂 builder。
    // 目的：先让 graph fixture 能稳定表达“证据来自哪里、定位到哪里”这两个基础事实。
    pub fn new(source_ref: impl Into<String>, locator: impl Into<String>) -> Self {
        Self {
            source_ref: source_ref.into(),
            locator: locator.into(),
        }
    }
}

// 2026-04-09 CST: 这里新增节点级元数据值模型，原因是 foundation 现在要支持 business-agnostic 的 metadata 过滤，
// 但还不适合把 source / kind / time 等字段硬编码成多个专用字段。
// 目的：先用通用单值 / 多值模型承载节点元数据，为 MetadataConstraint 提供统一匹配输入。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataFieldValue {
    Text(String),
    TextList(Vec<String>),
}

impl MetadataFieldValue {
    // 2026-04-09 CST: 这里补单值只读入口，原因是范围匹配只对可比较的单值字段有意义，
    // 需要一个稳定的内部访问方式，避免匹配逻辑直接拆 enum。
    // 目的：让 Range 约束在不暴露存储细节的前提下复用单值 metadata。
    pub fn as_text(&self) -> Option<&str> {
        match self {
            MetadataFieldValue::Text(value) => Some(value.as_str()),
            MetadataFieldValue::TextList(_) => None,
        }
    }

    // 2026-04-09 CST: 这里补包含判断，原因是 Equals 约束既可能命中单值字段，也可能命中多值字段中的某一项，
    // 如果不统一成标准方法，约束匹配会散落在 retrieval 侧。
    // 目的：把元数据值级别的最小匹配语义沉到 record 层。
    pub fn contains(&self, candidate: &str) -> bool {
        match self {
            MetadataFieldValue::Text(value) => value == candidate,
            MetadataFieldValue::TextList(values) => values.iter().any(|value| value == candidate),
        }
    }

    // 2026-04-09 CST: 这里补交集判断，原因是 kind / tags / labels 这类多值字段需要和请求白名单做交集匹配，
    // 单靠调用方循环会导致同一语义在多处重复实现。
    // 目的：把多值 metadata 的通用交集语义稳定收口。
    pub fn intersects(&self, candidates: &[String]) -> bool {
        match self {
            MetadataFieldValue::Text(value) => candidates.iter().any(|candidate| candidate == value),
            MetadataFieldValue::TextList(values) => values
                .iter()
                .any(|value| candidates.iter().any(|candidate| candidate == value)),
        }
    }
}

// 2026-04-07 CST: 这里定义知识节点模型，原因是 retrieval 后续真正命中的对象应该是节点，
// 不是 ontology concept 本身，因此需要独立记录节点文本、关联概念和证据引用。
// 目的：把“节点是图谱里的信息载体”这层语义稳定下来，避免后续把 concept 和 node 混成一层。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeNode {
    pub id: String,
    pub title: String,
    pub body: String,
    pub concept_ids: Vec<String>,
    pub evidence_refs: Vec<EvidenceRef>,
    // 2026-04-09 CST: 这里新增节点级 metadata 容器，原因是通用 MetadataConstraint 第一阶段先作用于 retrieval，
    // 需要节点能够承载 source / kind / observed_at 等非语义主字段。
    // 目的：把元数据过滤建立成 foundation 正式能力，而不是让不同上层各自拼字段。
    pub metadata: BTreeMap<String, MetadataFieldValue>,
}

impl KnowledgeNode {
    // 2026-04-07 CST: 这里提供节点最小构造函数，原因是 Task 4 只需要能创建可读节点，
    // 后续 retrieval 再围绕 title/body 做简单评分即可，当前不需要更多复杂字段。
    // 目的：先把节点主键、标题和正文固定成最小可用骨架，给 graph store 查询层提供稳定载体。
    pub fn new(id: impl Into<String>, title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            body: body.into(),
            concept_ids: Vec::new(),
            evidence_refs: Vec::new(),
            metadata: BTreeMap::new(),
        }
    }

    // 2026-04-07 CST: 这里补链式 concept 绑定，原因是一个知识节点可能同时服务多个概念，
    // graph store 后续需要按 concept ids 聚合节点，因此节点自身必须保存这份关联。
    // 目的：先用最简单的链式 API 固定测试和样本构造方式，避免 Task 4 过早引入专门装配器。
    pub fn with_concept_id(mut self, concept_id: impl Into<String>) -> Self {
        self.concept_ids.push(concept_id.into());
        self
    }

    // 2026-04-07 CST: 这里补链式证据追加，原因是节点命中之后最终要回溯到原始证据，
    // 这份引用应该跟随节点走，而不是让 graph store 或 evidence assembler 重新拼装。
    // 目的：把节点与证据的一对多关系收口在 record 层，减少后续层间重复传参。
    pub fn with_evidence_ref(mut self, evidence_ref: EvidenceRef) -> Self {
        self.evidence_refs.push(evidence_ref);
        self
    }

    // 2026-04-09 CST: 这里补单值元数据写入入口，原因是 source / owner / observed_at 这类字段在当前阶段最适合
    // 先用通用元数据容器承载，而不是继续扩专用结构。
    // 目的：给测试与主链提供最小、统一、可扩展的节点 metadata 写入方式。
    pub fn with_metadata_text(
        mut self,
        field: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.metadata
            .insert(field.into(), MetadataFieldValue::Text(value.into()));
        self
    }

    // 2026-04-09 CST: 这里补多值元数据写入入口，原因是 tags / kind / labels 等字段天然可能有多个值，
    // 需要一个不额外引入业务命名的通用承载方式。
    // 目的：让多值 metadata 与单值 metadata 使用同一套节点容器和约束框架。
    pub fn with_metadata_values(mut self, field: impl Into<String>, values: Vec<&str>) -> Self {
        self.metadata.insert(
            field.into(),
            MetadataFieldValue::TextList(values.into_iter().map(str::to_string).collect()),
        );
        self
    }
}

// 2026-04-07 CST: 这里定义知识边模型，原因是图谱关系发生在 node 与 node 之间，
// 它和 ontology 层的 concept relation 不是同一层职责，因此要单独留出图谱关系载体。
// 目的：先为后续 graph store 出边读取和 retrieval/evidence 路径还原准备稳定结构。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeEdge {
    pub from_node_id: String,
    pub to_node_id: String,
    pub relation_type: OntologyRelationType,
}

impl KnowledgeEdge {
    // 2026-04-07 CST: 这里提供知识边最小构造函数，原因是当前测试只需要验证节点间关系
    // 能被稳定表达和读取，不需要附加权重、置信度等更重元数据。
    // 目的：先用简洁 API 固定 Task 4 图谱边的创建方式，为后续 roaming/assembly 保留扩展口子。
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
