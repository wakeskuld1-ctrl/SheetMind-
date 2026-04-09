use std::collections::BTreeMap;

use crate::ops::foundation::knowledge_record::{KnowledgeNode, MetadataFieldValue};
use crate::ops::foundation::metadata_registry::{
    MetadataConstraintOperator, MetadataFieldTarget, MetadataRegistry, MetadataRegistryError,
};

// 2026-04-09 CST: 这里定义 foundation 通用元数据约束模型，原因是当前标签约束只覆盖 concept 级别，
// 还不足以支撑 source / kind / time_range 这类节点级标准元数据过滤。
// 目的：先把 MetadataConstraint 固定成 business-agnostic 的一等输入，为后续 metadata 知识漫游与检索收敛提供统一协议。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataConstraint {
    Equals { field: String, value: String },
    In { field: String, values: Vec<String> },
    HasAny { field: String, values: Vec<String> },
    Range {
        field: String,
        min: Option<String>,
        max: Option<String>,
    },
}

// 2026-04-09 CST: 这里补 MetadataScope 标准模型，原因是方案B第二阶段要把 metadata 从“retrieval 的额外参数”
// 提升成 route 之后主线共享的正式 scope 合同。
// 目的：让 RoamingPlan、CandidateScope 与 RetrievalEngine 都围绕同一份 metadata 约束输入协作，而不是继续各自传裸数组。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MetadataScope {
    pub constraints: Vec<MetadataConstraint>,
}

impl MetadataScope {
    // 2026-04-09 CST: 这里提供空 scope 构造器，原因是大多数 foundation 用例当前还不带 metadata 约束，
    // 需要一个明确且低成本的零值表示。
    // 目的：让 plan/scope 结构在不带 metadata 时也能保持统一形状，而不是回退到 Option 或临时参数。
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
        }
    }

    // 2026-04-09 CST: 这里补批量构造器，原因是 NavigationRequest 当前对外 API 仍然是 `with_metadata_constraints(vec![...])`，
    // 需要一个最小桥接点把旧输入形态提升为标准 scope。
    // 目的：在不破坏现有调用方式的前提下，让 metadata 正式进入通用 scope 合同。
    pub fn from_constraints(constraints: Vec<MetadataConstraint>) -> Self {
        Self { constraints }
    }

    // 2026-04-09 CST: 这里补只读切片入口，原因是下游匹配逻辑仍以切片消费约束，
    // 没必要为第二阶段重写所有匹配函数。
    // 目的：先以最小改动复用既有匹配实现，同时把调用入口统一到 MetadataScope。
    pub fn as_slice(&self) -> &[MetadataConstraint] {
        self.constraints.as_slice()
    }

    // 2026-04-09 CST: 这里补节点级匹配入口，原因是 RetrievalEngine 作为 scope 消费者不应再自行拆开 metadata scope 内部结构。
    // 目的：让 retrieval 只依赖“scope 是否匹配节点”的统一布尔语义。
    pub fn matches(&self, node: &KnowledgeNode) -> bool {
        self.matches_metadata(&node.metadata)
    }

    // 2026-04-09 CST: 这里补通用 metadata map 匹配入口，原因是方案B这一阶段不再只让 metadata 服务 retrieval，
    // 还要让 concept-level 收敛复用同一套匹配语义。
    // 目的：把 “scope 是否匹配某份 metadata” 固化成标准能力，避免 node/concept 各自复制 field/operator 分支。
    pub fn matches_metadata(&self, metadata: &BTreeMap<String, MetadataFieldValue>) -> bool {
        matches_all_metadata(self.as_slice(), metadata)
    }

    // 2026-04-09 CST: 这里补按注册表目标层级筛选约束的入口，原因是同一份 MetadataScope 现在会同时服务 concept 收敛和 node 检索，
    // 若不显式按字段目标层级裁剪，concept-only 字段会错误影响 retrieval，node-only 字段也会错误影响 route。
    // 目的：让 metadata 约束在不同主线阶段只消费自己应该消费的字段子集。
    pub fn constraints_for_registered_target<'a>(
        &'a self,
        registry: &MetadataRegistry,
        target: MetadataFieldTarget,
    ) -> Result<Vec<&'a MetadataConstraint>, MetadataRegistryError> {
        let mut applicable_constraints = Vec::new();

        for constraint in self.as_slice() {
            let field = constraint.field();
            if !registry.has_field(field) {
                return Err(MetadataRegistryError::UnregisteredField {
                    field: field.to_string(),
                });
            }

            if !registry.supports_target(field, target) {
                continue;
            }

            if !registry.supports_operator(field, constraint.operator()) {
                return Err(MetadataRegistryError::UnsupportedOperator {
                    field: field.to_string(),
                    operator: constraint.operator(),
                    target,
                });
            }

            applicable_constraints.push(constraint);
        }

        Ok(applicable_constraints)
    }

    // 2026-04-09 CST: 这里补按注册表目标层级做 metadata 匹配，原因是 registry 接入后 concept/node 两侧需要共享同一份 scope，
    // 但只应用与当前目标层级兼容的约束。
    // 目的：把“字段适用层级”纳入正式匹配合同，而不是继续靠上层手工拆分。
    pub fn matches_metadata_for_registered_target(
        &self,
        metadata: &BTreeMap<String, MetadataFieldValue>,
        registry: &MetadataRegistry,
        target: MetadataFieldTarget,
    ) -> Result<bool, MetadataRegistryError> {
        self.constraints_for_registered_target(registry, target)
            .map(|constraints| {
                constraints
                    .into_iter()
                    .all(|constraint| constraint.matches_metadata(metadata))
            })
    }
}

impl MetadataConstraint {
    // 2026-04-09 CST: 这里暴露约束字段读取入口，原因是 metadata-aware concept 收敛需要先判断某条约束是否适用于 concept metadata，
    // 否则像 `source` 这种只存在于 node 的字段会把 route 误过滤空。
    // 目的：让 resolver 能在不解构枚举细节的前提下判断约束适用范围，保持责任边界清晰。
    pub fn field(&self) -> &str {
        match self {
            MetadataConstraint::Equals { field, .. }
            | MetadataConstraint::In { field, .. }
            | MetadataConstraint::HasAny { field, .. }
            | MetadataConstraint::Range { field, .. } => field.as_str(),
        }
    }

    // 2026-04-09 CST: 这里补约束操作符读取入口，原因是字段注册表需要按 operator 校验字段是否支持该类标准约束，
    // 不能让 resolver/retrieval 再去手拆 MetadataConstraint 分支。
    // 目的：把 operator 语义也沉到标准模型层，供 registry 查询复用。
    pub fn operator(&self) -> MetadataConstraintOperator {
        match self {
            MetadataConstraint::Equals { .. } => MetadataConstraintOperator::Equals,
            MetadataConstraint::In { .. } => MetadataConstraintOperator::In,
            MetadataConstraint::HasAny { .. } => MetadataConstraintOperator::HasAny,
            MetadataConstraint::Range { .. } => MetadataConstraintOperator::Range,
        }
    }

    // 2026-04-09 CST: 这里补单值相等约束构造器，原因是 source / owner / status 这类元数据过滤最常见，
    // 需要一个最轻量的标准输入，避免调用方各自传 ad-hoc 字段。
    // 目的：先让 request / retrieval 主链有稳定的元数据等值过滤入口。
    pub fn equals(field: impl Into<String>, value: impl Into<String>) -> Self {
        Self::Equals {
            field: field.into(),
            value: value.into(),
        }
    }

    // 2026-04-09 CST: 这里补多候选单值约束构造器，原因是同一字段经常需要“在多个允许值中任选其一”，
    // 如果没有标准化入口，后续会重复造 OR 过滤逻辑。
    // 目的：把元数据白名单能力收敛到统一约束类型中。
    pub fn in_values(field: impl Into<String>, values: Vec<&str>) -> Self {
        Self::In {
            field: field.into(),
            values: values.into_iter().map(str::to_string).collect(),
        }
    }

    // 2026-04-09 CST: 这里补多值交集约束构造器，原因是 kind / tags / labels 这类字段天然是多值集合，
    // 需要显式支持“有任一交集即保留”的标准匹配语义。
    // 目的：给多值 metadata 留出正式、可测试的 foundation 入口。
    pub fn has_any(field: impl Into<String>, values: Vec<&str>) -> Self {
        Self::HasAny {
            field: field.into(),
            values: values.into_iter().map(str::to_string).collect(),
        }
    }

    // 2026-04-09 CST: 这里补范围约束构造器，原因是 observed_at / effective_at / version 这类元数据不能只做离散过滤，
    // 需要一个可扩展到时间窗和版本窗的通用范围语义。
    // 目的：先用字符串区间承载标准范围过滤，为 ISO 日期等可比较字段提供最小可用能力。
    pub fn range(
        field: impl Into<String>,
        min: Option<&str>,
        max: Option<&str>,
    ) -> Self {
        Self::Range {
            field: field.into(),
            min: min.map(str::to_string),
            max: max.map(str::to_string),
        }
    }

    // 2026-04-09 CST: 这里把单条约束匹配逻辑集中收口，原因是 retrieval 只应消费统一布尔匹配结果，
    // 不应在执行阶段再散落不同 field/operator 的分支判断。
    // 目的：确保 MetadataConstraint 真正成为标准能力，而不是只是一组数据结构。
    pub fn matches(&self, node: &KnowledgeNode) -> bool {
        self.matches_metadata(&node.metadata)
    }

    // 2026-04-09 CST: 这里补单条约束对 metadata map 的通用匹配，原因是 concept/node 现在都要消费同一份 MetadataConstraint，
    // 如果继续把匹配逻辑绑死在 KnowledgeNode 上，concept-level 收敛只能重复实现一遍。
    // 目的：把 operator 语义沉到 MetadataConstraint 本身，真正形成通用 foundation 能力。
    pub fn matches_metadata(&self, metadata: &BTreeMap<String, MetadataFieldValue>) -> bool {
        match self {
            MetadataConstraint::Equals { field, value } => metadata
                .get(field)
                .map(|field_value| field_value.contains(value))
                .unwrap_or(false),
            MetadataConstraint::In { field, values } | MetadataConstraint::HasAny { field, values } => {
                metadata
                    .get(field)
                    .map(|field_value| field_value.intersects(values))
                    .unwrap_or(false)
            }
            MetadataConstraint::Range { field, min, max } => metadata
                .get(field)
                .and_then(MetadataFieldValue::as_text)
                .map(|current| {
                    // 2026-04-09 CST: 这里把范围边界统一投影为 `&str`，原因是 `current` 来自节点 metadata 的只读文本视图，
                    // 当前方案B第一阶段只需要稳定支持字符串区间比较，不需要把基础模型升级成日期/数值专用类型。
                    // 目的：消除 `&str` 与 `String` 的比较类型不一致，同时保持 ISO 日期等可比较字符串的最小可用范围过滤语义。
                    let lower_ok = min
                        .as_ref()
                        .map(|lower| current >= lower.as_str())
                        .unwrap_or(true);
                    let upper_ok = max
                        .as_ref()
                        .map(|upper| current <= upper.as_str())
                        .unwrap_or(true);
                    lower_ok && upper_ok
                })
                .unwrap_or(false),
        }
    }
}

// 2026-04-09 CST: 这里补多约束统一匹配入口，原因是 request 侧后续会自然积累多条元数据条件，
// retrieval 不应自己决定是 AND 还是 OR。
// 目的：先明确 foundation 当前标准语义为“所有 MetadataConstraint 同时成立”。
pub fn matches_all(constraints: &[MetadataConstraint], node: &KnowledgeNode) -> bool {
    matches_all_metadata(constraints, &node.metadata)
}

// 2026-04-09 CST: 这里补通用 metadata map 批量匹配，原因是 route / roam / retrieve 现在都需要消费 “多条 metadata 约束同时成立” 的标准语义，
// 不能再把这个 AND 逻辑写死在节点侧。
// 目的：为 MetadataScopeResolver 和 RetrievalEngine 提供统一、可复用的布尔合同。
pub fn matches_all_metadata(
    constraints: &[MetadataConstraint],
    metadata: &BTreeMap<String, MetadataFieldValue>,
) -> bool {
    constraints
        .iter()
        .all(|constraint| constraint.matches_metadata(metadata))
}
