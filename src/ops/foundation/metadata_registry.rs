use std::collections::{BTreeMap, BTreeSet};

// 2026-04-09 CST: 这里定义 metadata 约束操作符，原因是 registry 需要显式声明字段支持哪些标准约束。
// 目的：让 concept/node 两侧都围绕同一套 operator 合同工作，而不是继续依赖隐式约定。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MetadataConstraintOperator {
    Equals,
    In,
    HasAny,
    Range,
}

// 2026-04-09 CST: 这里定义 metadata 字段适用层级，原因是同一份 MetadataScope 会同时服务 concept 和 node。
// 目的：把“字段到底作用于哪一层”提升成显式目录信息，避免主链各阶段各自猜测。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MetadataFieldTarget {
    Concept,
    Node,
}

// 2026-04-09 CST: 这里定义字段治理分组，原因是字段目录已经进入 foundation 通用能力主线。
// 目的：给 handoff、审计、后续导出提供稳定的分类维度。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MetadataFieldGroup {
    Identity,
    Classification,
    Source,
    Time,
    Governance,
}

// 2026-04-09 CST: 这里定义 registry 错误合同，原因是 registry 模式下不应再把非法字段和非法 operator 静默吃掉。
// 目的：为 route / roam / retrieve 共用同一套 fail-fast 错误边界。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataRegistryError {
    UnregisteredField { field: String },
    UnsupportedOperator {
        field: String,
        operator: MetadataConstraintOperator,
        target: MetadataFieldTarget,
    },
}

// 2026-04-09 CST: 这里定义字段值形态，原因是 registry 需要区分单值字段和多值字段。
// 目的：为后续 operator 适配、目录治理和字段说明保留标准化形状。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataFieldValueShape {
    Text,
    TextList,
}

// 2026-04-09 CST: 这里收敛字段治理属性，原因是字段目录不能只知道名字和 operator。
// 目的：把 group、说明、适用原因、兼容关系与废弃状态都纳入标准字段定义。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MetadataFieldGovernance {
    pub group: Option<MetadataFieldGroup>,
    pub description: Option<String>,
    pub applies_to_reason: Option<String>,
    pub deprecated_reason: Option<String>,
    pub aliases: Vec<String>,
    pub replaced_by: Option<String>,
    pub compatibility_note: Option<String>,
}

impl MetadataFieldGovernance {
    // 2026-04-09 CST: 这里保留空治理入口，原因是当前仍需兼容未补齐治理属性的增量阶段。
    // 目的：允许 registry 先接入，再通过 validation summary 暴露治理缺口。
    pub fn new() -> Self {
        Self::default()
    }

    // 2026-04-09 CST: 这里补字段分组声明，原因是目录治理需要结构化分类维度。
    // 目的：让字段分组成为标准元数据，而不是文档里的自由文本。
    pub fn with_group(mut self, group: MetadataFieldGroup) -> Self {
        self.group = Some(group);
        self
    }

    // 2026-04-09 CST: 这里补字段说明，原因是 handoff 和审计需要稳定的人类可读语义。
    // 目的：减少后续反复追问字段含义的成本。
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    // 2026-04-09 CST: 这里补适用原因，原因是 target 本身只能表达作用层级，不能解释为什么落在该层级。
    // 目的：让 concept/node 适用决策具备最小治理解释能力。
    pub fn with_applies_to_reason(mut self, applies_to_reason: impl Into<String>) -> Self {
        self.applies_to_reason = Some(applies_to_reason.into());
        self
    }

    // 2026-04-09 CST: 这里补废弃声明，原因是 registry 需要表达“字段仍存在但不再推荐扩展”的状态。
    // 目的：为兼容治理、替代链和目录审计提供显式状态。
    pub fn deprecated(mut self, deprecated_reason: impl Into<String>) -> Self {
        self.deprecated_reason = Some(deprecated_reason.into());
        self
    }

    // 2026-04-09 CST: 这里补 alias 声明，原因是历史字段名和导入侧字段名需要收口到 canonical field。
    // 目的：把兼容映射从文档约定提升成 registry 可读元数据。
    pub fn with_aliases<I, S>(mut self, aliases: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.aliases = aliases.into_iter().map(Into::into).collect();
        self.aliases.sort();
        self.aliases.dedup();
        self
    }

    // 2026-04-09 CST: 这里补 replaced_by 声明，原因是废弃字段若没有明确替代目标，就无法形成稳定迁移链。
    // 目的：把“将来迁移到哪里”沉淀成结构化兼容信息。
    pub fn with_replaced_by(mut self, replaced_by: impl Into<String>) -> Self {
        self.replaced_by = Some(replaced_by.into());
        self
    }

    // 2026-04-09 CST: 这里补兼容说明，原因是仅有 replaced_by 仍不足以解释过渡期策略。
    // 目的：让字段兼容关系具备最小文字说明能力。
    pub fn with_compatibility_note(mut self, compatibility_note: impl Into<String>) -> Self {
        self.compatibility_note = Some(compatibility_note.into());
        self
    }
}

// 2026-04-09 CST: 这里定义 registry 治理问题类型，原因是 validation summary 需要稳定问题枚举。
// 目的：把治理缺口、兼容缺口和兼容冲突都沉淀成结构化问题类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MetadataRegistryValidationIssueKind {
    MissingGroup,
    MissingDescription,
    MissingAppliesToReason,
    MissingTargets,
    MissingOperators,
    Deprecated,
    DeprecatedWithoutReplacement,
    DeprecatedWithoutCompatibilityNote,
    ReplacedByMissingField,
    AliasConflict,
    ReplacementCycle,
    AmbiguousReplacementTarget,
}

// 2026-04-09 CST: 这里定义单条治理问题，原因是审计结果不能只有问题计数。
// 目的：保留“哪个字段出了什么问题”的结构化明细。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataRegistryValidationIssue {
    pub field: String,
    pub kind: MetadataRegistryValidationIssueKind,
}

// 2026-04-09 CST: 这里定义治理摘要，原因是上层需要稳定消费整份 registry 的治理结果。
// 目的：让校验输出具备统一载体，而不是散落成 ad-hoc 判断。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MetadataRegistryValidationSummary {
    pub issues: Vec<MetadataRegistryValidationIssue>,
}

impl MetadataRegistryValidationSummary {
    // 2026-04-09 CST: 这里补快捷判定入口，原因是大多数调用方只需要先判断是否存在治理问题。
    // 目的：降低消费 validation summary 时的样板代码。
    pub fn has_issues(&self) -> bool {
        !self.issues.is_empty()
    }
}

// 2026-04-09 CST: 这里定义单字段兼容视图，原因是字段兼容信息需要统一读取而不是调用方自己拼装。
// 目的：为手册、审计和后续导出提供稳定对象。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataRegistryCompatibilityEntry {
    pub field: String,
    pub aliases: Vec<String>,
    pub deprecated_reason: Option<String>,
    pub replaced_by: Option<String>,
    pub replacement_chain: Vec<String>,
    pub compatibility_note: Option<String>,
}

// 2026-04-09 CST: 这里定义 alias 映射条目，原因是全量兼容摘要不仅要看字段，还要看 alias -> canonical 映射。
// 目的：给审计和导出提供稳定映射表。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataRegistryAliasMapping {
    pub alias: String,
    pub canonical_field: String,
}

// 2026-04-09 CST: 这里定义全量兼容摘要，原因是 registry 需要输出整份目录的兼容视图。
// 目的：让兼容关系成为标准目录能力，而不是零散查询。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MetadataRegistryCompatibilitySummary {
    pub fields: Vec<MetadataRegistryCompatibilityEntry>,
    pub alias_mappings: Vec<MetadataRegistryAliasMapping>,
}

// 2026-04-09 CST: 这里定义单字段解析结果，原因是 canonical resolver 不应只返回一个字段名。
// 目的：同时携带 direct/alias 命中、废弃信息、替代链和兼容说明。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataRegistryFieldResolution {
    pub input: String,
    pub canonical_field: Option<String>,
    pub matched_alias: Option<String>,
    pub deprecated_reason: Option<String>,
    pub replacement_chain: Vec<String>,
    pub compatibility_note: Option<String>,
}

// 2026-04-09 CST: 这里定义批量规范化结果，原因是调用方经常需要成批规范化字段。
// 目的：保留输入顺序和明细结果，为摘要工具提供统一基底。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MetadataRegistryFieldResolutionBatch {
    pub entries: Vec<MetadataRegistryFieldResolution>,
}

// 2026-04-09 CST: 这里定义未知字段条目，原因是 unknown-field list 必须保留输入值和输入位置。
// 目的：支撑目录审计和后续工具消费。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataRegistryUnknownFieldEntry {
    pub input: String,
    pub index: usize,
}

// 2026-04-09 CST: 这里定义批量规范化摘要，原因是 batch 明细之外还需要轻量统计视图。
// 目的：给上层一个稳定的 count summary，而不替代明细本身。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MetadataRegistryFieldResolutionBatchSummary {
    pub total_inputs: usize,
    pub direct_hits: usize,
    pub alias_hits: usize,
    pub unresolved_hits: usize,
    pub deprecated_hits: usize,
}

// 2026-04-09 CST: 这里定义 canonical 聚合条目，原因是方案B3要求按 canonical field 收敛批量解析结果。
// 目的：提供只读聚合视图，但不改变主链执行语义。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataRegistryCanonicalFieldAggregationEntry {
    pub canonical_field: String,
    pub inputs: Vec<String>,
    pub direct_hits: usize,
    pub alias_hits: usize,
    pub deprecated_hits: usize,
}

impl MetadataRegistryFieldResolutionBatch {
    // 2026-04-09 CST: 这里补 batch summary，原因是上层常常只关心“这一批大概解析成什么样”。
    // 目的：提供轻量计数，不替代明细。
    pub fn summary(&self) -> MetadataRegistryFieldResolutionBatchSummary {
        let mut summary = MetadataRegistryFieldResolutionBatchSummary {
            total_inputs: self.entries.len(),
            ..MetadataRegistryFieldResolutionBatchSummary::default()
        };

        for entry in &self.entries {
            if entry.canonical_field.is_none() {
                summary.unresolved_hits += 1;
            } else if entry.matched_alias.is_some() {
                summary.alias_hits += 1;
            } else {
                summary.direct_hits += 1;
            }

            if entry.deprecated_reason.is_some() {
                summary.deprecated_hits += 1;
            }
        }

        summary
    }

    // 2026-04-09 CST: 这里补 unknown-field list，原因是 summary 只有计数还不够做目录审计。
    // 目的：保留未知字段的原始输入顺序、重复项和 0-based 位置。
    pub fn unknown_fields(&self) -> Vec<MetadataRegistryUnknownFieldEntry> {
        self.entries
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                if entry.canonical_field.is_none() {
                    Some(MetadataRegistryUnknownFieldEntry {
                        input: entry.input.clone(),
                        index,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    // 2026-04-09 CST: 这里补 canonical 聚合视图，原因是方案B3需要按 canonical field 汇总解析结果。
    // 目的：只聚合已解析字段，保留 canonical 首次出现顺序，并保留同 canonical 下原始输入顺序。
    pub fn canonical_aggregations(&self) -> Vec<MetadataRegistryCanonicalFieldAggregationEntry> {
        let mut aggregation_indexes: BTreeMap<String, usize> = BTreeMap::new();
        let mut aggregations = Vec::new();

        for entry in &self.entries {
            let Some(canonical_field) = &entry.canonical_field else {
                continue;
            };

            let index = if let Some(index) = aggregation_indexes.get(canonical_field) {
                *index
            } else {
                let index = aggregations.len();
                aggregations.push(MetadataRegistryCanonicalFieldAggregationEntry {
                    canonical_field: canonical_field.clone(),
                    inputs: Vec::new(),
                    direct_hits: 0,
                    alias_hits: 0,
                    deprecated_hits: 0,
                });
                aggregation_indexes.insert(canonical_field.clone(), index);
                index
            };

            let aggregation = &mut aggregations[index];
            aggregation.inputs.push(entry.input.clone());

            if entry.matched_alias.is_some() {
                aggregation.alias_hits += 1;
            } else {
                aggregation.direct_hits += 1;
            }

            if entry.deprecated_reason.is_some() {
                aggregation.deprecated_hits += 1;
            }
        }

        aggregations
    }
}

// 2026-04-09 CST: 这里定义统一 registry 审计结果，原因是方案A要求把 registry 级治理和批次级目录输出收敛成标准对象。
// 目的：让 audit/export 层一次性消费标准结果，而不是重复手工拼装。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataRegistryAuditReport {
    pub governance: MetadataRegistryValidationSummary,
    pub compatibility: MetadataRegistryCompatibilitySummary,
    pub field_resolution_batch: MetadataRegistryFieldResolutionBatch,
    pub batch_summary: MetadataRegistryFieldResolutionBatchSummary,
    pub unknown_fields: Vec<MetadataRegistryUnknownFieldEntry>,
    pub canonical_aggregations: Vec<MetadataRegistryCanonicalFieldAggregationEntry>,
}

// 2026-04-09 CST: 这里定义字段目录单元，原因是 foundation 需要一个标准字段注册对象。
// 目的：把字段名、值形态、作用层级、operator 和治理属性都收口到同一载体。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataFieldDefinition {
    pub field: String,
    pub value_shape: MetadataFieldValueShape,
    pub targets: Vec<MetadataFieldTarget>,
    pub operators: Vec<MetadataConstraintOperator>,
    pub governance: MetadataFieldGovernance,
}

// 2026-04-09 CST: 这里定义 metadata registry，原因是字段管理已经进入 foundation 通用能力主线。
// 目的：为 resolver、retrieval 和目录治理提供统一字段注册表。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MetadataRegistry {
    fields: BTreeMap<String, MetadataFieldDefinition>,
}

impl MetadataRegistry {
    // 2026-04-09 CST: 这里提供空 registry 构造器，原因是主线仍需兼容尚未接入 registry 的阶段。
    // 目的：允许按增量方式接入显式字段目录。
    pub fn new() -> Self {
        Self {
            fields: BTreeMap::new(),
        }
    }

    // 2026-04-09 CST: 这里补单值字段注册入口，原因是 namespace/source/observed_at 一类字段天然是单值。
    // 目的：给最常见字段形态提供稳定 API。
    pub fn register_text_field(
        mut self,
        field: impl Into<String>,
        targets: Vec<MetadataFieldTarget>,
        operators: Vec<MetadataConstraintOperator>,
    ) -> Self {
        self.insert_definition(
            field.into(),
            MetadataFieldValueShape::Text,
            targets,
            operators,
            MetadataFieldGovernance::new(),
        );
        self
    }

    // 2026-04-09 CST: 这里补带治理属性的单值字段注册入口，原因是新字段应能一次性写入治理属性。
    // 目的：让字段定义和治理属性在同一处声明完成。
    pub fn register_text_field_with_governance(
        mut self,
        field: impl Into<String>,
        targets: Vec<MetadataFieldTarget>,
        operators: Vec<MetadataConstraintOperator>,
        governance: MetadataFieldGovernance,
    ) -> Self {
        self.insert_definition(
            field.into(),
            MetadataFieldValueShape::Text,
            targets,
            operators,
            governance,
        );
        self
    }

    // 2026-04-09 CST: 这里补多值字段注册入口，原因是 channels/tags/labels 一类字段天然是多值。
    // 目的：让多值字段与单值字段共用同一套 registry 语义。
    pub fn register_text_list_field(
        mut self,
        field: impl Into<String>,
        targets: Vec<MetadataFieldTarget>,
        operators: Vec<MetadataConstraintOperator>,
    ) -> Self {
        self.insert_definition(
            field.into(),
            MetadataFieldValueShape::TextList,
            targets,
            operators,
            MetadataFieldGovernance::new(),
        );
        self
    }

    // 2026-04-09 CST: 这里补带治理属性的多值字段注册入口，原因是多值字段也应进入同一治理主线。
    // 目的：避免值形态不同就绕开字段治理。
    pub fn register_text_list_field_with_governance(
        mut self,
        field: impl Into<String>,
        targets: Vec<MetadataFieldTarget>,
        operators: Vec<MetadataConstraintOperator>,
        governance: MetadataFieldGovernance,
    ) -> Self {
        self.insert_definition(
            field.into(),
            MetadataFieldValueShape::TextList,
            targets,
            operators,
            governance,
        );
        self
    }

    // 2026-04-09 CST: 这里补字段存在判定，原因是 registry 模式下非法字段需要 fail-fast。
    // 目的：给 scope/retrieval 提供统一字段存在校验。
    pub fn has_field(&self, field: &str) -> bool {
        self.fields.contains_key(field)
    }

    // 2026-04-09 CST: 这里补字段 target 适配查询，原因是 concept/node 需要按同一目录合同判断字段适用范围。
    // 目的：把“字段作用于哪一层”收敛成统一布尔入口。
    pub fn supports_target(&self, field: &str, target: MetadataFieldTarget) -> bool {
        self.definition(field)
            .map(|definition| definition.targets.contains(&target))
            .unwrap_or(false)
    }

    // 2026-04-09 CST: 这里补字段 operator 适配查询，原因是 registry 模式下 operator 需要显式校验。
    // 目的：让非法 operator 在主链前置暴露。
    pub fn supports_operator(&self, field: &str, operator: MetadataConstraintOperator) -> bool {
        self.definition(field)
            .map(|definition| definition.operators.contains(&operator))
            .unwrap_or(false)
    }

    // 2026-04-09 CST: 这里补字段定义读取入口，原因是测试、审计与后续导出都需要按字段名读取定义。
    // 目的：提供稳定只读接口。
    pub fn definition(&self, field: &str) -> Option<&MetadataFieldDefinition> {
        self.fields.get(field)
    }

    // 2026-04-09 CST: 这里补全量字段定义读取入口，原因是目录导出和手册生成需要遍历整个 registry。
    // 目的：按 registry 内部有序存储顺序提供稳定输出。
    pub fn definitions(&self) -> Vec<&MetadataFieldDefinition> {
        self.fields.values().collect()
    }

    // 2026-04-09 CST: 这里补按治理分组查询，原因是字段分组必须成为一等读取维度。
    // 目的：支撑按分组审计和导出。
    pub fn fields_in_group(&self, group: MetadataFieldGroup) -> Vec<&MetadataFieldDefinition> {
        self.fields
            .values()
            .filter(|definition| definition.governance.group == Some(group))
            .collect()
    }

    // 2026-04-09 CST: 这里补空 registry 判定，原因是主线仍需保留空目录兼容路径。
    // 目的：让上层可以清晰区分“未启用 registry”和“启用了 registry 但字段缺失”。
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    // 2026-04-09 CST: 这里补治理校验摘要，原因是字段治理不能停留在人工 review。
    // 目的：输出稳定的结构化治理问题列表。
    pub fn validate_governance(&self) -> MetadataRegistryValidationSummary {
        let mut issues = Vec::new();
        let alias_owners = self.alias_owners();
        let replacement_target_sources = self.replacement_target_sources();

        for definition in self.fields.values() {
            if definition.governance.group.is_none() {
                issues.push(MetadataRegistryValidationIssue {
                    field: definition.field.clone(),
                    kind: MetadataRegistryValidationIssueKind::MissingGroup,
                });
            }

            if definition.governance.description.is_none() {
                issues.push(MetadataRegistryValidationIssue {
                    field: definition.field.clone(),
                    kind: MetadataRegistryValidationIssueKind::MissingDescription,
                });
            }

            if definition.governance.applies_to_reason.is_none() {
                issues.push(MetadataRegistryValidationIssue {
                    field: definition.field.clone(),
                    kind: MetadataRegistryValidationIssueKind::MissingAppliesToReason,
                });
            }

            if definition.targets.is_empty() {
                issues.push(MetadataRegistryValidationIssue {
                    field: definition.field.clone(),
                    kind: MetadataRegistryValidationIssueKind::MissingTargets,
                });
            }

            if definition.operators.is_empty() {
                issues.push(MetadataRegistryValidationIssue {
                    field: definition.field.clone(),
                    kind: MetadataRegistryValidationIssueKind::MissingOperators,
                });
            }

            if definition.governance.deprecated_reason.is_some() {
                issues.push(MetadataRegistryValidationIssue {
                    field: definition.field.clone(),
                    kind: MetadataRegistryValidationIssueKind::Deprecated,
                });

                if definition.governance.replaced_by.is_none() {
                    issues.push(MetadataRegistryValidationIssue {
                        field: definition.field.clone(),
                        kind: MetadataRegistryValidationIssueKind::DeprecatedWithoutReplacement,
                    });
                }

                if definition.governance.compatibility_note.is_none() {
                    issues.push(MetadataRegistryValidationIssue {
                        field: definition.field.clone(),
                        kind: MetadataRegistryValidationIssueKind::DeprecatedWithoutCompatibilityNote,
                    });
                }
            }

            if let Some(replaced_by) = &definition.governance.replaced_by {
                if !self.has_field(replaced_by) {
                    issues.push(MetadataRegistryValidationIssue {
                        field: definition.field.clone(),
                        kind: MetadataRegistryValidationIssueKind::ReplacedByMissingField,
                    });
                }
            }

            if definition.governance.aliases.iter().any(|alias| {
                alias_owners
                    .get(alias)
                    .map(|owners| owners.len() > 1)
                    .unwrap_or(false)
            }) {
                issues.push(MetadataRegistryValidationIssue {
                    field: definition.field.clone(),
                    kind: MetadataRegistryValidationIssueKind::AliasConflict,
                });
            }

            if self.has_replacement_cycle(&definition.field) {
                issues.push(MetadataRegistryValidationIssue {
                    field: definition.field.clone(),
                    kind: MetadataRegistryValidationIssueKind::ReplacementCycle,
                });
            }

            if definition
                .governance
                .replaced_by
                .as_ref()
                .and_then(|target| replacement_target_sources.get(target))
                .map(|sources| sources.len() > 1)
                .unwrap_or(false)
            {
                issues.push(MetadataRegistryValidationIssue {
                    field: definition.field.clone(),
                    kind: MetadataRegistryValidationIssueKind::AmbiguousReplacementTarget,
                });
            }
        }

        MetadataRegistryValidationSummary { issues }
    }

    // 2026-04-09 CST: 这里补单字段兼容视图，原因是兼容信息应能稳定按字段读取。
    // 目的：为手册、审计和导出提供统一入口。
    pub fn compatibility_for(&self, field: &str) -> Option<MetadataRegistryCompatibilityEntry> {
        self.definition(field)
            .map(|definition| self.compatibility_entry_for_definition(definition))
    }

    // 2026-04-09 CST: 这里补全量兼容摘要，原因是 registry 需要输出整份目录的兼容总览。
    // 目的：形成稳定的 compatibility summary。
    pub fn compatibility_summary(&self) -> MetadataRegistryCompatibilitySummary {
        let fields = self
            .fields
            .values()
            .map(|definition| self.compatibility_entry_for_definition(definition))
            .collect::<Vec<_>>();

        let mut alias_mappings = self
            .fields
            .values()
            .flat_map(|definition| {
                definition
                    .governance
                    .aliases
                    .iter()
                    .cloned()
                    .map(|alias| MetadataRegistryAliasMapping {
                        alias,
                        canonical_field: definition.field.clone(),
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        alias_mappings.sort_by(|left, right| left.alias.cmp(&right.alias));

        MetadataRegistryCompatibilitySummary {
            fields,
            alias_mappings,
        }
    }

    // 2026-04-09 CST: 这里补 canonical resolver，原因是 alias 和 direct 字段都需要收口到统一目录视图。
    // 目的：返回结构化解析结果，而不是只返回字段名。
    pub fn resolve_field(&self, input: &str) -> MetadataRegistryFieldResolution {
        if let Some(definition) = self.definition(input) {
            return MetadataRegistryFieldResolution {
                input: input.to_string(),
                canonical_field: Some(definition.field.clone()),
                matched_alias: None,
                deprecated_reason: definition.governance.deprecated_reason.clone(),
                replacement_chain: self.replacement_chain_for(input),
                compatibility_note: definition.governance.compatibility_note.clone(),
            };
        }

        if let Some(definition) = self.fields.values().find(|definition| {
            definition
                .governance
                .aliases
                .iter()
                .any(|alias| alias == input)
        }) {
            return MetadataRegistryFieldResolution {
                input: input.to_string(),
                canonical_field: Some(definition.field.clone()),
                matched_alias: Some(input.to_string()),
                deprecated_reason: definition.governance.deprecated_reason.clone(),
                replacement_chain: self.replacement_chain_for(&definition.field),
                compatibility_note: definition.governance.compatibility_note.clone(),
            };
        }

        MetadataRegistryFieldResolution {
            input: input.to_string(),
            canonical_field: None,
            matched_alias: None,
            deprecated_reason: None,
            replacement_chain: Vec::new(),
            compatibility_note: None,
        }
    }

    // 2026-04-09 CST: 这里补批量规范化入口，原因是调用方经常需要对一组字段名做统一规范化。
    // 目的：保留输入顺序，并把单字段解析语义收敛到同一条路径。
    pub fn normalize_fields<I, S>(&self, fields: I) -> MetadataRegistryFieldResolutionBatch
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        MetadataRegistryFieldResolutionBatch {
            entries: fields
                .into_iter()
                .map(|field| self.resolve_field(field.as_ref()))
                .collect(),
        }
    }

    // 2026-04-09 CST: 这里补统一 registry audit/export 入口，原因是方案A要求把多份目录工具输出收敛成标准对象。
    // 目的：让上层一次性拿到治理摘要、兼容摘要和指定输入批次的审计结果。
    pub fn audit_fields<I, S>(&self, fields: I) -> MetadataRegistryAuditReport
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let field_resolution_batch = self.normalize_fields(fields);
        let batch_summary = field_resolution_batch.summary();
        let unknown_fields = field_resolution_batch.unknown_fields();
        let canonical_aggregations = field_resolution_batch.canonical_aggregations();

        MetadataRegistryAuditReport {
            governance: self.validate_governance(),
            compatibility: self.compatibility_summary(),
            field_resolution_batch,
            batch_summary,
            unknown_fields,
            canonical_aggregations,
        }
    }

    // 2026-04-09 CST: 这里集中写入字段定义，原因是单值/多值注册共享相同的去重和覆盖逻辑。
    // 目的：让后续字段扩展只改一处。
    fn insert_definition(
        &mut self,
        field: String,
        value_shape: MetadataFieldValueShape,
        mut targets: Vec<MetadataFieldTarget>,
        mut operators: Vec<MetadataConstraintOperator>,
        governance: MetadataFieldGovernance,
    ) {
        targets.sort();
        targets.dedup();
        operators.sort();
        operators.dedup();

        self.fields.insert(
            field.clone(),
            MetadataFieldDefinition {
                field,
                value_shape,
                targets,
                operators,
                governance,
            },
        );
    }

    // 2026-04-09 CST: 这里集中构造单字段兼容视图，原因是 compatibility_for 与 summary 共享同一逻辑。
    // 目的：避免兼容摘要生成路径分叉。
    fn compatibility_entry_for_definition(
        &self,
        definition: &MetadataFieldDefinition,
    ) -> MetadataRegistryCompatibilityEntry {
        MetadataRegistryCompatibilityEntry {
            field: definition.field.clone(),
            aliases: definition.governance.aliases.clone(),
            deprecated_reason: definition.governance.deprecated_reason.clone(),
            replaced_by: definition.governance.replaced_by.clone(),
            replacement_chain: self.replacement_chain_for(&definition.field),
            compatibility_note: definition.governance.compatibility_note.clone(),
        }
    }

    // 2026-04-09 CST: 这里集中计算替代链，原因是兼容视图和 resolver 都需要同一条迁移路径。
    // 目的：在不改运行语义的前提下，输出稳定的目录级替代链。
    fn replacement_chain_for(&self, field: &str) -> Vec<String> {
        let mut chain = Vec::new();
        let mut visited = BTreeSet::new();
        let mut current = field.to_string();

        while let Some(next_field) = self
            .fields
            .get(current.as_str())
            .and_then(|definition| definition.governance.replaced_by.clone())
        {
            if !visited.insert(next_field.clone()) {
                break;
            }

            chain.push(next_field.clone());
            current = next_field;
        }

        chain
    }

    // 2026-04-09 CST: 这里集中构造 alias -> owning fields 索引，原因是 alias 冲突属于 registry 全局治理问题。
    // 目的：避免校验时重复遍历全表。
    fn alias_owners(&self) -> BTreeMap<String, Vec<String>> {
        let mut alias_owners = BTreeMap::new();

        for definition in self.fields.values() {
            for alias in &definition.governance.aliases {
                alias_owners
                    .entry(alias.clone())
                    .or_insert_with(Vec::new)
                    .push(definition.field.clone());
            }
        }

        alias_owners
    }

    // 2026-04-09 CST: 这里集中构造 replacement target -> source fields 索引，原因是共享替代目标属于全局兼容问题。
    // 目的：为 AmbiguousReplacementTarget 校验提供稳定索引。
    fn replacement_target_sources(&self) -> BTreeMap<String, Vec<String>> {
        let mut replacement_target_sources = BTreeMap::new();

        for definition in self.fields.values() {
            if let Some(replaced_by) = &definition.governance.replaced_by {
                replacement_target_sources
                    .entry(replaced_by.clone())
                    .or_insert_with(Vec::new)
                    .push(definition.field.clone());
            }
        }

        replacement_target_sources
    }

    // 2026-04-09 CST: 这里集中检测替代链循环，原因是循环不能只靠 replacement_chain_for 内部静默截断。
    // 目的：把循环显式抬升成治理问题。
    fn has_replacement_cycle(&self, field: &str) -> bool {
        let mut visited = BTreeSet::new();
        let mut current = field.to_string();
        visited.insert(current.clone());

        while let Some(next_field) = self
            .fields
            .get(current.as_str())
            .and_then(|definition| definition.governance.replaced_by.clone())
        {
            if !visited.insert(next_field.clone()) {
                return true;
            }

            current = next_field;
        }

        false
    }
}
