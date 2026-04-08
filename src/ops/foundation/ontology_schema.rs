use std::collections::BTreeMap;

// 2026-04-07 CST: 这里定义 ontology concept，原因是 foundation 导航内核后续所有语义定位都要围绕概念来做。
// 目的：先提供最小稳定数据结构，让 concept id、展示名和别名能在 schema 构建时统一收口。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OntologyConcept {
    pub id: String,
    pub name: String,
    pub aliases: Vec<String>,
}

impl OntologyConcept {
    // 2026-04-07 CST: 这里提供最小构造函数，原因是当前 TDD 测试需要直接声明概念并参与 schema 构建。
    // 目的：先用最少字段完成概念建模，避免在 Task 2 过早引入 description、capability_ids 等额外复杂度。
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            aliases: Vec::new(),
        }
    }

    // 2026-04-07 CST: 这里提供链式 alias 追加，原因是当前最关键的行为就是“名称或别名都能命中 concept id”。
    // 目的：先把别名声明方式固定成最简 API，后续 schema 索引和 router 都可以复用这套输入形态。
    pub fn with_alias(mut self, alias: impl Into<String>) -> Self {
        self.aliases.push(alias.into());
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
    DuplicateLookupKey {
        lookup_key: String,
        existing_concept_id: String,
        incoming_concept_id: String,
    },
}

// 2026-04-07 CST: 这里定义 ontology schema，原因是 schema 需要同时持有原始 concepts/relations 和查找索引。
// 目的：先把“原始数据 + 归一化索引”放到同一个对象里，后续 store 层只负责查询封装，不重复造索引。
#[derive(Debug, Clone)]
pub struct OntologySchema {
    pub concepts: Vec<OntologyConcept>,
    pub relations: Vec<OntologyRelation>,
    concept_index: BTreeMap<String, usize>,
    lookup_index: BTreeMap<String, String>,
}

impl OntologySchema {
    // 2026-04-07 CST: 这里构建最小 schema，原因是当前测试要验证 concept name 和 alias 都能建立稳定索引。
    // 目的：先完成 concept id 去重与 lookup key 建表，后续再继续补关系合法性、约束校验和更多元数据。
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

    // 2026-04-07 CST: 这里提供概念查询入口，原因是当前最小闭环的第一步就是从问题文本映射到种子 concept id。
    // 目的：先提供稳定只读查询接口，后续 router 和 store 都可以围绕这个返回值继续实现。
    pub fn find_concept_id(&self, raw: &str) -> Option<&str> {
        let normalized = Self::normalize_lookup_key(raw);
        self.lookup_index.get(&normalized).map(String::as_str)
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
        lookup_index: &mut BTreeMap<String, String>,
        raw_key: &str,
        concept_id: &str,
    ) -> Result<(), OntologySchemaError> {
        let normalized = Self::normalize_lookup_key(raw_key);
        if normalized.is_empty() {
            return Ok(());
        }

        if let Some(existing) = lookup_index.get(&normalized) {
            return Err(OntologySchemaError::DuplicateLookupKey {
                lookup_key: normalized,
                existing_concept_id: existing.clone(),
                incoming_concept_id: concept_id.to_string(),
            });
        }

        lookup_index.insert(normalized, concept_id.to_string());
        Ok(())
    }

    // 2026-04-07 CST: 这里做最小归一化，原因是 concept name/alias 匹配至少需要大小写无关。
    // 目的：先建立稳定且便宜的 lookup 规则，后续再视需要扩展更复杂的标准化策略。
    fn normalize_lookup_key(raw: &str) -> String {
        raw.trim().to_lowercase()
    }
}
