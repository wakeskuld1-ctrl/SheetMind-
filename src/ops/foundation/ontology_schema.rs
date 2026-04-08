use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

// 2026-04-08 CST: 这里定义 concept 实体，原因是 foundation 知识导航内核的语义起点
// 必须先有稳定的“概念 id / 展示名 / alias”载体，后续 router 和 roaming 才有统一输入。
// 目的：先把最小 concept 数据结构固定下来，避免后面把概念信息散落到 store 或 router 内部。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OntologyConcept {
    pub id: String,
    pub name: String,
    pub aliases: Vec<String>,
}

impl OntologyConcept {
    // 2026-04-08 CST: 这里提供最小构造函数，原因是第一阶段只需要 concept id 与显示名
    // 就能支撑 name/alias 到 concept id 的基础映射，不需要过早引入 description 或 capability。
    // 目的：让测试和后续样本构造保持最小、直接、可读。
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            aliases: Vec::new(),
        }
    }

    // 2026-04-08 CST: 这里提供 alias 链式追加，原因是当前 schema 最关键的行为就是
    // “原始名称和 alias 都能稳定命中同一个 concept id”，这条能力要从输入建模层开始固定。
    // 目的：让测试与后续概念装配代码都使用同一套最小 API，减少额外 builder 噪音。
    pub fn with_alias(mut self, alias: impl Into<String>) -> Self {
        self.aliases.push(alias.into());
        self
    }
}

// 2026-04-08 CST: 这里先定义关系类型枚举，原因是第一阶段虽然还没进入 roaming 测试，
// 但 ontology schema 的外部形状已经需要预留 relation 容器，避免后续重复修改构造契约。
// 目的：把 concept 间关系的最小枚举集合固定下来，为 store 和 roaming 继续扩展留出口。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OntologyRelationType {
    ParentOf,
    ChildOf,
    DependsOn,
    Supports,
    References,
    AdjacentTo,
}

// 2026-04-08 CST: 这里定义 concept 之间的关系记录，原因是知识漫游阶段需要根据 relation
// 做受限扩展，不能只保留 concept 节点本体而没有可遍历的邻接边。
// 目的：先把 relation 的最小载体稳定下来，后续再逐步补合法性校验和更细元数据。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OntologyRelation {
    pub from_concept_id: String,
    pub to_concept_id: String,
    pub relation_type: OntologyRelationType,
}

// 2026-04-08 CST: 这里定义 schema 构建错误，原因是 concept id 与 lookup key 冲突
// 不能静默覆盖，否则会让上层路由命中不稳定且难以排查。
// 目的：把 schema 构建期的失败边界显式化，让测试和调用方都能明确识别原因。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OntologySchemaError {
    DuplicateConceptId {
        concept_id: String,
    },
    DuplicateLookupKey {
        lookup_key: String,
        existing_concept_id: String,
        incoming_concept_id: String,
    },
}

// 2026-04-08 CST: 这里定义 schema 本体，原因是 foundation 第一阶段需要在一个对象里同时持有
// 原始 concept / relation 和 name/alias 查找索引，避免上层重复建索引。
// 目的：把“原始数据 + 查找索引”统一收口，后续 ontology store 只负责查询包装，不重复造轮子。
#[derive(Debug, Clone)]
pub struct OntologySchema {
    pub concepts: Vec<OntologyConcept>,
    pub relations: Vec<OntologyRelation>,
    concept_index: BTreeMap<String, usize>,
    lookup_index: BTreeMap<String, String>,
}

impl OntologySchema {
    // 2026-04-08 CST: 这里构造最小 schema，原因是第一阶段最先要跑通的是 concept id 去重
    // 与 name/alias 建索引，而不是提前做复杂 relation 校验或外部存储装配。
    // 目的：让 schema 先具备最核心的可用能力，为后续 ontology store 和 router 提供稳定底座。
    pub fn new(
        concepts: Vec<OntologyConcept>,
        relations: Vec<OntologyRelation>,
    ) -> Result<Self, OntologySchemaError> {
        let mut concept_index = BTreeMap::new();
        let mut lookup_index = BTreeMap::new();

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

    // 2026-04-08 CST: 这里提供最小查找入口，原因是 schema 当前第一条被验证的职责
    // 就是把原始文本归一化后映射到 concept id，上层路由只应依赖这个查询结果。
    // 目的：先稳定暴露只读接口，后续即使索引策略变化也不影响上层调用面。
    pub fn find_concept_id(&self, raw: &str) -> Option<&str> {
        let normalized = Self::normalize_lookup_key(raw);
        self.lookup_index.get(&normalized).map(String::as_str)
    }

    // 2026-04-08 CST: 这里提供按 concept id 读取 concept 的入口，原因是下一步 ontology store
    // 需要安全地暴露 concept 详情，而不应暴露 schema 内部索引细节。
    // 目的：先把 concept 读取契约稳定下来，给 store 和 roaming 后续实现复用。
    pub fn concept(&self, concept_id: &str) -> Option<&OntologyConcept> {
        self.concept_index
            .get(concept_id)
            .and_then(|index| self.concepts.get(*index))
    }

    // 2026-04-08 CST: 这里抽出 lookup key 插入逻辑，原因是 concept name 与 alias
    // 共用同一套归一化和冲突规则，写成独立函数能避免重复并减少后续扩展摩擦。
    // 目的：让 schema 构建期行为保持单点定义、单点修改。
    fn insert_lookup_key(
        lookup_index: &mut BTreeMap<String, String>,
        raw_key: &str,
        concept_id: &str,
    ) -> Result<(), OntologySchemaError> {
        let normalized = Self::normalize_lookup_key(raw_key);
        if normalized.is_empty() {
            return Ok(());
        }

        if let Some(existing_concept_id) = lookup_index.get(&normalized) {
            return Err(OntologySchemaError::DuplicateLookupKey {
                lookup_key: normalized,
                existing_concept_id: existing_concept_id.clone(),
                incoming_concept_id: concept_id.to_string(),
            });
        }

        lookup_index.insert(normalized, concept_id.to_string());
        Ok(())
    }

    // 2026-04-08 CST: 这里采用最小归一化策略，原因是第一阶段只需要大小写无关和去首尾空白
    // 就能稳定支撑测试，不应在还没有明确需求时过度引入更重文本标准化。
    // 目的：先建立便宜、稳定、可预测的 lookup 规则，为后续扩展留出空间。
    fn normalize_lookup_key(raw: &str) -> String {
        raw.trim().to_lowercase()
    }
}
