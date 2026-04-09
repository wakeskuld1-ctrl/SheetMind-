use std::collections::BTreeMap;

use crate::ops::foundation::knowledge_record::MetadataFieldValue;

// 2026-04-07 CST: 这里定义 ontology concept，原因是 foundation 导航内核后续所有语义定位都要围绕概念来做。
// 目的：先提供最小稳定数据结构，让 concept id、展示名和别名能在 schema 构建时统一收口。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OntologyConcept {
    pub id: String,
    pub name: String,
    pub aliases: Vec<String>,
    // 2026-04-09 CST: 这里增加 concept tags，原因是 foundation 路由开始需要最小 scope 约束能力，
    // 不能再假设同一短语在全局只会对应一个概念。
    // 目的：先用轻量标签承接“同词不同域”的约束输入，而不提前引入更重的元数据模型。
    pub tags: Vec<String>,
    // 2026-04-09 CST: 这里补 concept 级通用 metadata 容器，原因是方案B下一阶段要让 metadata 真正参与 concept-level 收敛，
    // 但又不能把 domain、channels 这类字段业务化硬编码进 roaming 或 router。
    // 目的：让 ontology concept 和 knowledge node 共享同一套 metadata value 模型，为通用 MetadataScopeResolver 提供标准输入。
    pub metadata: BTreeMap<String, MetadataFieldValue>,
}

impl OntologyConcept {
    // 2026-04-07 CST: 这里提供最小构造函数，原因是当前 TDD 测试需要直接声明概念并参与 schema 构建。
    // 目的：先用最少字段完成概念建模，避免在 Task 2 过早引入 description、capability_ids 等额外复杂度。
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            aliases: Vec::new(),
            tags: Vec::new(),
            metadata: BTreeMap::new(),
        }
    }

    // 2026-04-07 CST: 这里提供链式 alias 追加，原因是当前最关键的行为就是“名称或别名都能命中 concept id”。
    // 目的：先把别名声明方式固定成最简 API，后续 schema 索引和 router 都可以复用这套输入形态。
    pub fn with_alias(mut self, alias: impl Into<String>) -> Self {
        self.aliases.push(alias.into());
        self
    }

    // 2026-04-09 CST: 这里提供标签声明入口，原因是同词多概念场景下，foundation 路由需要最小约束维度
    // 来决定当前问题更应该落到哪个概念。
    // 目的：先把标签能力固定成最小 API，后续再决定是否扩成更完整的元数据结构。
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    // 2026-04-09 CST: 这里补 concept 单值 metadata 写入口，原因是当前红测已经要求 concept 能承载 domain 这类通用字段，
    // 如果继续只支持 tags，就无法把 MetadataConstraint 提升为跨 route / roam / retrieve 共用的标准能力。
    // 目的：用与 KnowledgeNode 相同的元数据载体承接 concept-level 白名单过滤，避免重复造一套 concept 专用 DSL。
    pub fn with_metadata_text(
        mut self,
        field: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.metadata
            .insert(field.into(), MetadataFieldValue::Text(value.into()));
        self
    }

    // 2026-04-09 CST: 这里补 concept 多值 metadata 写入口，原因是 channels 这类字段天然是多值集合，
    // 只补单值接口会让 `HasAny` 约束在 concept 收敛阶段失真。
    // 目的：让 concept metadata 与 node metadata 在字段形态上保持一致，供同一份 MetadataScope 复用。
    pub fn with_metadata_values(mut self, field: impl Into<String>, values: Vec<&str>) -> Self {
        self.metadata.insert(
            field.into(),
            MetadataFieldValue::TextList(values.into_iter().map(str::to_string).collect()),
        );
        self
    }
}

// 2026-04-07 CST: 这里先定义关系类型占位枚举，原因是本轮 schema 构造函数已经要求接收 relations 集合。
// 目的：即使当前测试暂时不使用 relation，也先把 schema 的外部形状固定下来，避免后续再改构造契约。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OntologyRelationType {
    ParentOf,
    ChildOf,
    DependsOn,
    Supports,
    References,
    AdjacentTo,
}

// 2026-04-07 CST: 这里定义 concept 之间的关系记录，原因是后续 roaming 需要基于 relation 做受控扩展。
// 目的：先稳定 relation 的最小载体，Task 2 里不做复杂校验，只保留字段骨架。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OntologyRelation {
    pub from_concept_id: String,
    pub to_concept_id: String,
    pub relation_type: OntologyRelationType,
}

// 2026-04-07 CST: 这里定义 schema 构建错误，原因是 concept id/alias 冲突必须显式失败，不能静默覆盖。
// 目的：让测试和后续调用方能明确区分“构造成功”和“索引冲突”两类结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OntologySchemaError {
    DuplicateConceptId {
        concept_id: String,
    },
}

// 2026-04-07 CST: 这里定义 ontology schema，原因是 schema 需要同时持有原始 concepts/relations 和查找索引。
// 目的：先把“原始数据 + 归一化索引”放到同一个对象里，后续 store 层只负责查询封装，不重复造索引。
#[derive(Debug, Clone)]
pub struct OntologySchema {
    pub concepts: Vec<OntologyConcept>,
    pub relations: Vec<OntologyRelation>,
    concept_index: BTreeMap<String, usize>,
    lookup_index: BTreeMap<String, Vec<String>>,
}

impl OntologySchema {
    // 2026-04-07 CST: 这里构建最小 schema，原因是当前测试要验证 concept name 和 alias 都能建立稳定索引。
    // 目的：先完成 concept id 去重与 lookup key 建表，后续再继续补关系合法性、约束校验和更多元数据。
    pub fn new(
        concepts: Vec<OntologyConcept>,
        relations: Vec<OntologyRelation>,
    ) -> Result<Self, OntologySchemaError> {
        let mut concept_index = BTreeMap::new();
        let mut lookup_index = BTreeMap::<String, Vec<String>>::new();

        for (position, concept) in concepts.iter().enumerate() {
            if concept_index.insert(concept.id.clone(), position).is_some() {
                return Err(OntologySchemaError::DuplicateConceptId {
                    concept_id: concept.id.clone(),
                });
            }

            Self::insert_lookup_key(&mut lookup_index, &concept.name, &concept.id)?;
            for alias in &concept.aliases {
                Self::insert_lookup_key(&mut lookup_index, alias, &concept.id)?;
            }
        }

        Ok(Self {
            concepts,
            relations,
            concept_index,
            lookup_index,
        })
    }

    // 2026-04-07 CST: 这里提供概念查询入口，原因是当前最小闭环的第一步就是从问题文本映射到种子 concept id。
    // 目的：先提供稳定只读查询接口，后续 router 和 store 都可以围绕这个返回值继续实现。
    pub fn find_concept_id(&self, raw: &str) -> Option<&str> {
        let normalized = Self::normalize_lookup_key(raw);
        self.lookup_index
            .get(&normalized)
            .and_then(|concept_ids| concept_ids.first())
            .map(String::as_str)
    }

    // 2026-04-09 CST: 这里增加多候选 lookup 读取，原因是 foundation 路由准备支持最小 scope/tag 约束，
    // 同一 lookup key 命中多个 concept 不再应被 schema 层直接判死。
    // 目的：让上层 router 能基于约束在候选 concept 之间做最小决策。
    pub fn find_concept_ids(&self, raw: &str) -> Vec<&str> {
        let normalized = Self::normalize_lookup_key(raw);
        self.lookup_index
            .get(&normalized)
            .map(|concept_ids| concept_ids.iter().map(String::as_str).collect())
            .unwrap_or_default()
    }

    // 2026-04-07 CST: 这里保留概念读取入口，原因是后续 ontology store 需要安全读取概念详情。
    // 目的：先把索引和概念向量之间的关联固定下来，避免后续再次暴露内部存储细节。
    pub fn concept(&self, concept_id: &str) -> Option<&OntologyConcept> {
        self.concept_index
            .get(concept_id)
            .and_then(|index| self.concepts.get(*index))
    }

    // 2026-04-07 CST: 这里单独封装 lookup key 插入，原因是 name 和 alias 共用同一套冲突规则。
    // 目的：减少重复逻辑，也为后续扩到 description 关键短语或 capability alias 留出口子。
    fn insert_lookup_key(
        lookup_index: &mut BTreeMap<String, Vec<String>>,
        raw_key: &str,
        concept_id: &str,
    ) -> Result<(), OntologySchemaError> {
        let normalized = Self::normalize_lookup_key(raw_key);
        if normalized.is_empty() {
            return Ok(());
        }

        let candidates = lookup_index.entry(normalized).or_default();
        if !candidates.iter().any(|existing| existing == concept_id) {
            candidates.push(concept_id.to_string());
        }
        Ok(())
    }

    // 2026-04-07 CST: 这里做最小归一化，原因是 concept name/alias 匹配至少需要大小写无关。
    // 目的：先建立稳定且便宜的 lookup 规则，后续再视需要扩展更复杂的标准化策略。
    // 2026-04-09 CST: 这里把标点、连字符、下划线统一折叠为空格，原因是 foundation 路由现在要先
    // 抵抗不同知识源常见的短语写法差异，不能把纯格式差异误当成概念差异。
    // 目的：把“gross-margin / gross_margin / gross margin 等价”固定成通用 lookup 契约。
    fn normalize_lookup_key(raw: &str) -> String {
        raw.chars()
            .map(|character| {
                if character.is_alphanumeric() {
                    character.to_ascii_lowercase()
                } else {
                    ' '
                }
            })
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }
}
