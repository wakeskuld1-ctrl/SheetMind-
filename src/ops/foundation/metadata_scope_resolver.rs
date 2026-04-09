use crate::ops::foundation::metadata_constraint::MetadataScope;
use crate::ops::foundation::metadata_registry::{
    MetadataFieldTarget, MetadataRegistry, MetadataRegistryError,
};
use crate::ops::foundation::ontology_store::OntologyStore;

// 2026-04-09 CST: 这里新增 metadata-aware concept scope resolver，原因是方案B要求把 concept-level 收敛沉到 foundation 通用层，
// 而不是继续把 metadata 过滤散落在 pipeline 或 roaming 的局部分支里。
// 目的：统一承接 “哪些 metadata 约束适用于 concept、如何过滤 concept ids” 这条标准能力主线。
#[derive(Debug, Clone, Default)]
pub struct MetadataScopeResolver;

impl MetadataScopeResolver {
    // 2026-04-09 CST: 这里提供 concept id 批量收敛入口，原因是 route 命中结果、模板 seeds 和 roaming 邻居扩展都需要同一套过滤规则，
    // 如果每层各自判断会很快出现语义漂移。
    // 目的：把 concept-level metadata 收敛集中在单一模块，供 pipeline 与 roaming 复用。
    pub fn constrain_concept_ids(
        ontology_store: &OntologyStore,
        concept_ids: &[String],
        metadata_scope: &MetadataScope,
    ) -> Vec<String> {
        let applicable_fields = Self::applicable_fields(ontology_store, metadata_scope);
        if applicable_fields.is_empty() {
            return concept_ids.to_vec();
        }

        concept_ids
            .iter()
            .filter(|concept_id| {
                ontology_store
                    .concept(concept_id)
                    .map(|concept| {
                        metadata_scope
                            .as_slice()
                            .iter()
                            .filter(|constraint| applicable_fields.iter().any(|field| *field == constraint.field()))
                            .all(|constraint| constraint.matches_metadata(&concept.metadata))
                    })
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    // 2026-04-09 CST: 这里补基于显式 registry 的 concept 收敛入口，原因是字段目录阶段不能再依赖“concept 上是否碰巧存在某个 key”的猜测，
    // 而要按注册表声明判断哪些约束适用于 concept。
    // 目的：让 concept-level metadata 收敛升级成显式目录驱动的标准能力。
    pub fn constrain_concept_ids_with_registry(
        ontology_store: &OntologyStore,
        concept_ids: &[String],
        metadata_scope: &MetadataScope,
        metadata_registry: &MetadataRegistry,
    ) -> Result<Vec<String>, MetadataRegistryError> {
        let applicable_constraints = metadata_scope
            .constraints_for_registered_target(metadata_registry, MetadataFieldTarget::Concept)?;
        if applicable_constraints.is_empty() {
            return Ok(concept_ids.to_vec());
        }

        Ok(concept_ids
            .iter()
            .filter(|concept_id| {
                ontology_store
                    .concept(concept_id)
                    .map(|concept| {
                        applicable_constraints
                            .iter()
                            .all(|constraint| constraint.matches_metadata(&concept.metadata))
                    })
                    .unwrap_or(false)
            })
            .cloned()
            .collect())
    }

    // 2026-04-09 CST: 这里保留单 concept 判断入口，原因是 roaming 扩展邻居时有按单个 concept 做准入判断的需求，
    // 但它仍然必须遵守与批量收敛一致的适用字段判定。
    // 目的：让 concept-level metadata 判断在不同调用点共享同一套语义。
    pub fn concept_matches(
        ontology_store: &OntologyStore,
        concept_id: &str,
        metadata_scope: &MetadataScope,
    ) -> bool {
        let constrained = Self::constrain_concept_ids(
            ontology_store,
            &[concept_id.to_string()],
            metadata_scope,
        );
        constrained.iter().any(|candidate| candidate == concept_id)
    }

    // 2026-04-09 CST: 这里先判断哪些约束字段在 concept 层真实存在，原因是同一份 MetadataScope 还会被 retrieval 用于 node metadata，
    // 像 `source` 这类只存在于 node 的字段如果直接拿来过滤 concept，会把主线错误收窄为空。
    // 目的：让 concept-level 收敛只消费 concept 侧真正有语义的字段，其余约束继续留给 retrieval 生效。
    fn applicable_fields<'a>(
        ontology_store: &'a OntologyStore,
        metadata_scope: &'a MetadataScope,
    ) -> Vec<&'a str> {
        metadata_scope
            .as_slice()
            .iter()
            .filter_map(|constraint| {
                let field = constraint.field();
                ontology_store
                    .concepts()
                    .iter()
                    .any(|concept| concept.metadata.contains_key(field))
                    .then_some(field)
            })
            .collect()
    }
}
