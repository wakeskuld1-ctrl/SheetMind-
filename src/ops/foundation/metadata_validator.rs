use crate::ops::foundation::knowledge_record::KnowledgeNode;
use crate::ops::foundation::metadata_schema::{MetadataSchema, MetadataValueType};

// 2026-04-08 CST: 这里定义 metadata 节点级校验问题枚举，原因是方案 B 要求 validator 输出结构化错误，
// 不能退化成单个字符串或首错即停的 Result。
// 目的：把 required / allowed / allowed values / type / concept policy 缺失这几类最小治理结果固定成稳定契约。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataValidationIssue {
    MissingConceptPolicy {
        node_id: String,
        concept_id: String,
    },
    MissingRequiredField {
        node_id: String,
        concept_id: String,
        field_key: String,
    },
    DisallowedField {
        node_id: String,
        concept_id: String,
        field_key: String,
    },
    InvalidAllowedValue {
        node_id: String,
        field_key: String,
        actual_value: String,
        allowed_values: Vec<String>,
    },
    InvalidValueType {
        node_id: String,
        field_key: String,
        expected_type: MetadataValueType,
        actual_value: String,
    },
}

// 2026-04-08 CST: 这里定义 metadata validator 本体，原因是当前阶段只需要一个可复用的只读校验器，
// 不需要仓储级扫描、自动修复或迁移上下文。
// 目的：让 schema registry 上层先具备稳定的节点校验入口，后续 versioning 再在此基础上扩展。
pub struct MetadataValidator<'a> {
    schema: &'a MetadataSchema,
}

impl<'a> MetadataValidator<'a> {
    // 2026-04-08 CST: 这里提供最小构造函数，原因是 validator 只依赖 schema，
    // 不应该在当前阶段引入 repository、graph store 或运行态状态。
    // 目的：保持依赖单一，符合 SRP，也方便测试样本直接装配。
    pub fn new(schema: &'a MetadataSchema) -> Self {
        Self { schema }
    }

    // 2026-04-08 CST: 这里执行单节点 metadata 校验，原因是用户当前批准的范围是“节点级 validator”，
    // 不是 repository 全量审计。
    // 目的：按固定顺序产出结构化问题，保证测试可复跑、调用方可消费。
    pub fn validate_node(&self, node: &KnowledgeNode) -> Vec<MetadataValidationIssue> {
        let mut issues = Vec::new();
        let mut active_policy_ids = Vec::new();

        for concept_id in &node.concept_ids {
            if self.schema.concept_policy(concept_id).is_some() {
                active_policy_ids.push(concept_id.as_str());
            } else {
                issues.push(MetadataValidationIssue::MissingConceptPolicy {
                    node_id: node.id.clone(),
                    concept_id: concept_id.clone(),
                });
            }
        }

        for (field_key, field_value) in &node.metadata {
            for concept_id in &active_policy_ids {
                if let Some(policy) = self.schema.concept_policy(concept_id) {
                    if !policy.allows_field(field_key) {
                        issues.push(MetadataValidationIssue::DisallowedField {
                            node_id: node.id.clone(),
                            concept_id: (*concept_id).to_string(),
                            field_key: field_key.clone(),
                        });
                    }
                }
            }

            if let Some(field_definition) = self.schema.field_definition(field_key) {
                if !field_definition.allowed_values.is_empty()
                    && !field_definition
                        .allowed_values
                        .iter()
                        .any(|allowed_value| allowed_value == field_value)
                {
                    issues.push(MetadataValidationIssue::InvalidAllowedValue {
                        node_id: node.id.clone(),
                        field_key: field_key.clone(),
                        actual_value: field_value.clone(),
                        allowed_values: field_definition.allowed_values.clone(),
                    });
                }

                if !matches_value_type(field_value, &field_definition.value_type) {
                    issues.push(MetadataValidationIssue::InvalidValueType {
                        node_id: node.id.clone(),
                        field_key: field_key.clone(),
                        expected_type: field_definition.value_type.clone(),
                        actual_value: field_value.clone(),
                    });
                }
            }
        }

        for concept_id in &active_policy_ids {
            if let Some(policy) = self.schema.concept_policy(concept_id) {
                for required_field_key in &policy.required_field_keys {
                    if !node.metadata.contains_key(required_field_key) {
                        issues.push(MetadataValidationIssue::MissingRequiredField {
                            node_id: node.id.clone(),
                            concept_id: (*concept_id).to_string(),
                            field_key: required_field_key.clone(),
                        });
                    }
                }
            }
        }

        issues
    }
}

// 2026-04-08 CST: 这里收口字符串到 metadata 值类型的最小判定，原因是 KnowledgeNode 当前仍以字符串存储 metadata，
// validator 只能基于字符串解析做第一层治理。
// 目的：在不改动 record 存储模型的前提下，为 Integer / Float / Boolean 提供稳定校验语义。
fn matches_value_type(value: &str, expected_type: &MetadataValueType) -> bool {
    match expected_type {
        MetadataValueType::String => true,
        MetadataValueType::Integer => value.parse::<i64>().is_ok(),
        MetadataValueType::Float => value.parse::<f64>().is_ok(),
        MetadataValueType::Boolean => value.parse::<bool>().is_ok(),
    }
}
