use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

// 2026-04-08 CST: 这里固定 metadata schema 默认版本号，原因是 versioning 第一阶段至少要先有一个正式版本锚点，
// 不能让各调用方自己拼字符串。
// 目的：为后续 validator、versioning 和 migration 提供统一的默认版本契约。
pub const DEFAULT_METADATA_SCHEMA_VERSION: &str = "metadata-schema:v1";

// 2026-04-08 CST: 这里定义 metadata 字段类型枚举，原因是 metadata schema registry 的第一步
// 必须先把“字段值是什么类型”固定成正式契约，而不是继续停留在裸字符串约定。
// 目的：为后续 validator 和迁移规则提供稳定的类型起点。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetadataValueType {
    String,
    Integer,
    Float,
    Boolean,
}

// 2026-04-08 CST: 这里定义 metadata 字段定义对象，原因是本体论元数据管理系统不能只靠
// `BTreeMap<String, String>` 存值，还必须能管理“字段本身是什么”。
// 目的：把 key、类型、说明和 allowed values 收口成正式字段定义。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetadataFieldDefinition {
    pub key: String,
    pub value_type: MetadataValueType,
    pub description: Option<String>,
    pub allowed_values: Vec<String>,
    pub deprecated: bool,
    pub replaced_by: Option<String>,
    pub aliases: Vec<String>,
}

impl MetadataFieldDefinition {
    // 2026-04-08 CST: 这里提供字段定义最小构造函数，原因是当前阶段只需要先把 key 和类型注册下来，
    // 不应过早塞入 default、deprecated、migration 等更重治理语义。
    // 目的：让 metadata registry 先具备最小、直接、可测试的字段定义入口。
    pub fn new(key: impl Into<String>, value_type: MetadataValueType) -> Self {
        Self {
            key: key.into(),
            value_type,
            description: None,
            allowed_values: Vec::new(),
            deprecated: false,
            replaced_by: None,
            aliases: Vec::new(),
        }
    }

    // 2026-04-08 CST: 这里支持补字段说明，原因是 metadata schema 除了机器可校验，
    // 还需要给后续维护者和 AI 提供最小可读语义。
    // 目的：让字段定义具备最轻量的文档化能力。
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    // 2026-04-08 CST: 这里支持补 allowed value，原因是方案 B 明确要求 metadata registry
    // 至少先支持最小枚举型约束，而不只是记录字段名。
    // 目的：为后续枚举校验预留稳定输入形态。
    pub fn with_allowed_value(mut self, value: impl Into<String>) -> Self {
        self.allowed_values.push(value.into());
        self
    }

    // 2026-04-08 CST: 这里支持把字段标记为 deprecated，原因是 migration contract 第一阶段需要先能正式声明
    // “这个字段已经进入退场状态”，而不是继续靠文档约定。
    // 目的：为后续 replaced_by、alias 审计和 migration 执行器提供最小治理锚点。
    pub fn deprecated(mut self) -> Self {
        self.deprecated = true;
        self
    }

    // 2026-04-08 CST: 这里支持声明字段替代目标，原因是 deprecated 字段如果没有正式 replacement 指向，
    // 后续版本迁移就缺少结构化落点。
    // 目的：把字段演进路径从“人读文档”提升到“schema 可读契约”。
    pub fn with_replaced_by(mut self, field_key: impl Into<String>) -> Self {
        self.replaced_by = Some(field_key.into());
        self
    }

    // 2026-04-08 CST: 这里支持声明字段 alias，原因是 migration contract 第一阶段需要正式承载旧字段名、
    // 兼容字段名或导入过渡字段名。
    // 目的：为后续 alias 审计和迁移规则提供统一注册入口。
    pub fn with_alias(mut self, alias: impl Into<String>) -> Self {
        self.aliases.push(alias.into());
        self
    }
}

// 2026-04-08 CST: 这里定义 concept metadata policy，原因是方案 B 的重点不只是注册字段，
// 还要表达“某个 concept 允许哪些字段、哪些字段必填”。
// 目的：把 ontology concept 和 metadata 字段正式绑定起来。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConceptMetadataPolicy {
    pub concept_id: String,
    pub allowed_field_keys: Vec<String>,
    pub required_field_keys: Vec<String>,
}

impl ConceptMetadataPolicy {
    // 2026-04-08 CST: 这里提供 concept policy 最小构造函数，原因是当前阶段只需要围绕单个 concept
    // 注册 allowed/required 字段，不做继承或层级覆盖。
    // 目的：让 concept 绑定从最小形态开始稳定下来。
    pub fn new(concept_id: impl Into<String>) -> Self {
        Self {
            concept_id: concept_id.into(),
            allowed_field_keys: Vec::new(),
            required_field_keys: Vec::new(),
        }
    }

    // 2026-04-08 CST: 这里支持追加 allowed 字段，原因是 concept policy 的第一职责就是
    // 限定当前 concept 可接受的 metadata 字段集合。
    // 目的：让测试和后续 schema 装配都复用这套链式 API。
    pub fn with_allowed_field(mut self, field_key: impl Into<String>) -> Self {
        self.allowed_field_keys.push(field_key.into());
        self
    }

    // 2026-04-08 CST: 这里支持追加 required 字段，原因是 metadata 管理系统如果不能表达
    // concept 下的必填字段，就还只是一个静态字段目录。
    // 目的：把 concept 级 required 约束显式建模。
    pub fn with_required_field(mut self, field_key: impl Into<String>) -> Self {
        self.required_field_keys.push(field_key.into());
        self
    }

    // 2026-04-08 CST: 这里暴露 concept 是否允许某字段的判定，原因是上层后续 validator
    // 不应自己直接遍历 policy 内部向量。
    // 目的：为 concept-field 兼容性校验提供稳定只读接口。
    pub fn allows_field(&self, field_key: &str) -> bool {
        self.allowed_field_keys.iter().any(|key| key == field_key)
            || self.required_field_keys.iter().any(|key| key == field_key)
    }

    // 2026-04-08 CST: 这里暴露 concept 是否要求某字段的判定，原因是 required 语义是 concept 绑定层
    // 的正式输出，不应在 validator 里临时拼规则。
    // 目的：让 required 字段判断保持单点定义。
    pub fn requires_field(&self, field_key: &str) -> bool {
        self.required_field_keys.iter().any(|key| key == field_key)
    }
}

// 2026-04-08 CST: 这里定义 metadata schema 构建错误，原因是字段注册表和 concept policy
// 一旦允许重复或引用未知字段，后续整个元数据管理系统都会失去约束意义。
// 目的：把 registry 构建期的最小失败边界显式化。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataSchemaError {
    InvalidSchemaVersion {
        schema_version: String,
    },
    DuplicateFieldKey {
        field_key: String,
    },
    DuplicateFieldAlias {
        alias: String,
    },
    DuplicateConceptPolicy {
        concept_id: String,
    },
    UnknownFieldReference {
        concept_id: String,
        field_key: String,
    },
    UnknownReplacementTarget {
        field_key: String,
        replaced_by: String,
    },
    SelfReplacementTarget {
        field_key: String,
    },
}

// 2026-04-08 CST: 这里定义 metadata schema registry 本体，原因是 metadata 管理需要在一个对象里同时持有
// 字段定义、concept policy 和高频查询索引。
// 目的：把“字段层 + concept 绑定层”统一收口成正式 foundation 能力。
#[derive(Debug, Clone)]
pub struct MetadataSchema {
    pub schema_version: String,
    pub fields: Vec<MetadataFieldDefinition>,
    pub concept_policies: Vec<ConceptMetadataPolicy>,
    field_index: BTreeMap<String, usize>,
    concept_policy_index: BTreeMap<String, usize>,
}

impl MetadataSchema {
    // 2026-04-08 CST: 这里构造 metadata schema registry，原因是当前阶段最重要的是确保
    // 字段去重、policy 去重和 policy 引用的字段都已注册。
    // 目的：让 metadata 管理的最小约束在构建期就能被锁住。
    pub fn new(
        fields: Vec<MetadataFieldDefinition>,
        concept_policies: Vec<ConceptMetadataPolicy>,
    ) -> Result<Self, MetadataSchemaError> {
        Self::new_with_version(DEFAULT_METADATA_SCHEMA_VERSION, fields, concept_policies)
    }

    // 2026-04-08 CST: 这里新增显式 schema version 构造入口，原因是 versioning 第一阶段除了默认版本，
    // 还必须允许上层正式声明当前 schema 的版本号。
    // 目的：为后续 schema 演进、兼容性判断和 migration 接口预留稳定入口。
    pub fn new_with_version(
        schema_version: impl Into<String>,
        fields: Vec<MetadataFieldDefinition>,
        concept_policies: Vec<ConceptMetadataPolicy>,
    ) -> Result<Self, MetadataSchemaError> {
        let schema_version = schema_version.into().trim().to_string();
        if schema_version.is_empty() {
            return Err(MetadataSchemaError::InvalidSchemaVersion { schema_version });
        }

        let mut field_index = BTreeMap::new();
        let mut alias_index = BTreeMap::new();
        let mut concept_policy_index = BTreeMap::new();

        for (position, field) in fields.iter().enumerate() {
            if field_index.insert(field.key.clone(), position).is_some() {
                return Err(MetadataSchemaError::DuplicateFieldKey {
                    field_key: field.key.clone(),
                });
            }
        }

        for field in &fields {
            if let Some(replaced_by) = &field.replaced_by {
                if replaced_by == &field.key {
                    return Err(MetadataSchemaError::SelfReplacementTarget {
                        field_key: field.key.clone(),
                    });
                }

                if !field_index.contains_key(replaced_by) {
                    return Err(MetadataSchemaError::UnknownReplacementTarget {
                        field_key: field.key.clone(),
                        replaced_by: replaced_by.clone(),
                    });
                }
            }

            for alias in &field.aliases {
                if field_index.contains_key(alias) || alias == &field.key {
                    return Err(MetadataSchemaError::DuplicateFieldAlias {
                        alias: alias.clone(),
                    });
                }

                if alias_index
                    .insert(alias.clone(), field.key.clone())
                    .is_some()
                {
                    return Err(MetadataSchemaError::DuplicateFieldAlias {
                        alias: alias.clone(),
                    });
                }
            }
        }

        for (position, policy) in concept_policies.iter().enumerate() {
            if concept_policy_index
                .insert(policy.concept_id.clone(), position)
                .is_some()
            {
                return Err(MetadataSchemaError::DuplicateConceptPolicy {
                    concept_id: policy.concept_id.clone(),
                });
            }

            for field_key in policy
                .allowed_field_keys
                .iter()
                .chain(policy.required_field_keys.iter())
            {
                if !field_index.contains_key(field_key) {
                    return Err(MetadataSchemaError::UnknownFieldReference {
                        concept_id: policy.concept_id.clone(),
                        field_key: field_key.clone(),
                    });
                }
            }
        }

        Ok(Self {
            schema_version,
            fields,
            concept_policies,
            field_index,
            concept_policy_index,
        })
    }

    // 2026-04-08 CST: 这里按字段 key 读取字段定义，原因是 registry 的最小查询职责
    // 就是返回某个 metadata 字段的正式定义对象。
    // 目的：让调用方不必接触内部索引结构。
    pub fn field_definition(&self, field_key: &str) -> Option<&MetadataFieldDefinition> {
        self.field_index
            .get(field_key)
            .and_then(|index| self.fields.get(*index))
    }

    // 2026-04-08 CST: 这里按 concept id 读取 metadata policy，原因是 concept 绑定是方案 B 的核心，
    // 后续 validator 和治理动作都要复用这个入口。
    // 目的：为 concept -> metadata policy 提供稳定只读接口。
    pub fn concept_policy(&self, concept_id: &str) -> Option<&ConceptMetadataPolicy> {
        self.concept_policy_index
            .get(concept_id)
            .and_then(|index| self.concept_policies.get(*index))
    }

    // 2026-04-08 CST: 这里提供最小版本兼容性判断，原因是 versioning 第一阶段虽然不做 migration，
    // 但至少要能判断某个 schema version 是否与当前 registry 相容。
    // 目的：先把 compatibility 契约收口为精确匹配，避免过早引入复杂的跨版本推断规则。
    pub fn is_compatible_with(&self, schema_version: &str) -> bool {
        self.schema_version == schema_version.trim()
    }
}
