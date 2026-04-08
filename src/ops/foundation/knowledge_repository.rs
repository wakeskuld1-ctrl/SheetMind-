use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::ops::foundation::knowledge_bundle::KnowledgeBundle;
use crate::ops::foundation::knowledge_graph_store::KnowledgeGraphStore;
use crate::ops::foundation::ontology_schema::OntologySchemaError;
use crate::ops::foundation::ontology_store::OntologyStore;
use serde::{Deserialize, Serialize};

const REPOSITORY_LAYOUT_VERSION: &str = "foundation.repository-layout.v1";
const REPOSITORY_LAYOUT_BUNDLE_FILE: &str = "bundle.json";
const REPOSITORY_LAYOUT_MANIFEST_FILE: &str = "repository.manifest.json";

// 2026-04-08 CST: 这里定义 metadata 精确过滤器，原因是 phase 2 第一阶段只做最小通用过滤契约，
// 不做模糊匹配、范围过滤或业务专用规则。
// 目的：先让任意知识节点都能通过统一 exact-match 规则被过滤。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MetadataFilter {
    exact_matches: BTreeMap<String, String>,
    concept_scope: Vec<String>,
}

impl MetadataFilter {
    // 2026-04-08 CST: 这里提供最小构造函数，原因是当前过滤器没有额外配置项，
    // 用空过滤器即可表达“不过滤”。
    // 目的：保持调用方式简单。
    pub fn new() -> Self {
        Self::default()
    }

    // 2026-04-08 CST: 这里新增精确匹配项，原因是当前阶段先锁最小通用过滤行为，
    // 不提前把复杂查询语言塞进 foundation。
    // 目的：用链式方式构建稳定的 metadata 过滤条件。
    pub fn with_exact_match(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.exact_matches.insert(key.into(), value.into());
        self
    }

    // 2026-04-08 CST: 这里新增概念域收窄能力，原因是方案 B 不是只做 metadata 多字段 AND，
    // 还要允许上层先按 concept scope 缩小候选节点集合。
    // 目的：让 foundation 在不引入复杂 DSL 的前提下获得更精确的标准过滤能力。
    pub fn with_concept_id(mut self, concept_id: impl Into<String>) -> Self {
        self.concept_scope.push(concept_id.into());
        self
    }
}

// 2026-04-08 CST: 这里定义标准仓储错误边界，原因是 phase 2 第一阶段除了落盘读回，
// 还要把 bundle 构建期的关键失败语义显式化。
// 目的：让调用方能区分是包内容非法，还是文件读写失败，还是 ontology 重建失败。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KnowledgeRepositoryError {
    DuplicateNodeId { node_id: String },
    SaveFailed { path: String, message: String },
    LoadFailed { path: String, message: String },
    DeserializeFailed { path: String, message: String },
    InvalidOntology { message: String },
}

// 2026-04-08 CST: 这里把知识仓储设计成 bundle 的标准化持有层，原因是当前阶段目标是把
// “可落盘的标准包”与“可查询的内存视图”收口在同一个通用边界内。
// 目的：为后续持久化、导入导出与过滤查询提供统一入口。
#[derive(Debug, Clone)]
pub struct KnowledgeRepository {
    bundle: KnowledgeBundle,
}

impl KnowledgeRepository {
    // 2026-04-08 CST: 这里构建标准仓储，原因是 bundle 进入仓储前至少要先做最小一致性校验，
    // 否则后续 graph/query 层会在更晚阶段暴露不稳定覆盖问题。
    // 目的：把 duplicate node id 这类最关键的仓储边界前移到构建期。
    pub fn new(bundle: KnowledgeBundle) -> Result<Self, KnowledgeRepositoryError> {
        let mut seen_node_ids = BTreeMap::new();

        for node in &bundle.nodes {
            if seen_node_ids.insert(node.id.clone(), ()).is_some() {
                return Err(KnowledgeRepositoryError::DuplicateNodeId {
                    node_id: node.id.clone(),
                });
            }
        }

        Ok(Self { bundle })
    }

    // 2026-04-08 CST: 这里暴露只读 bundle 引用，原因是 phase 2 第一阶段还不需要可变写回 API，
    // 但测试与上层需要检查标准包内容。
    // 目的：保持仓储输出透明且只读。
    pub fn bundle(&self) -> &KnowledgeBundle {
        &self.bundle
    }

    // 2026-04-08 CST: 这里提供落盘能力，原因是当前阶段必须先证明标准包能稳定写入本地文件，
    // 不应继续停留在纯内存结构。
    // 目的：形成 bundle -> file 的最小持久化能力。
    pub fn save_to_path(&self, path: &Path) -> Result<(), KnowledgeRepositoryError> {
        write_json_with_staging(path, &self.bundle)
    }

    // 2026-04-08 CST: 这里新增标准布局目录保存入口，原因是 foundation 进入持久化标准化阶段后，
    // 单文件 JSON 已不足以表达“正式布局契约”。
    // 目的：统一输出 `bundle.json + repository.manifest.json` 这组最小布局标准。
    pub fn save_to_layout_dir(&self, layout_dir: &Path) -> Result<(), KnowledgeRepositoryError> {
        fs::create_dir_all(layout_dir).map_err(|error| KnowledgeRepositoryError::SaveFailed {
            path: layout_dir.display().to_string(),
            message: error.to_string(),
        })?;

        let bundle_path = layout_dir.join(REPOSITORY_LAYOUT_BUNDLE_FILE);
        let manifest_path = layout_dir.join(REPOSITORY_LAYOUT_MANIFEST_FILE);
        let manifest = KnowledgeRepositoryLayoutManifest::from_bundle(&self.bundle);

        write_json_with_staging(&bundle_path, &self.bundle)?;
        write_json_with_staging(&manifest_path, &manifest)
    }

    // 2026-04-08 CST: 这里提供读盘能力，原因是标准仓储闭环必须覆盖 file -> bundle -> repository，
    // 否则落盘格式还不能算正式可复用契约。
    // 目的：形成最小读回能力，并在读回后复用同一套仓储校验。
    pub fn load_from_path(path: &Path) -> Result<Self, KnowledgeRepositoryError> {
        let raw =
            fs::read_to_string(path).map_err(|error| KnowledgeRepositoryError::LoadFailed {
                path: path.display().to_string(),
                message: error.to_string(),
            })?;

        let bundle: KnowledgeBundle = serde_json::from_str(&raw).map_err(|error| {
            KnowledgeRepositoryError::DeserializeFailed {
                path: path.display().to_string(),
                message: error.to_string(),
            }
        })?;

        Self::new(bundle)
    }

    // 2026-04-08 CST: 这里新增标准布局目录读回入口，原因是布局标准如果只定义写法而没有回读入口，
    // 后续调用方仍会绕回单文件路径，布局标准就无法真正生效。
    // 目的：形成 layout dir -> repository 的最小正式闭环。
    pub fn load_from_layout_dir(layout_dir: &Path) -> Result<Self, KnowledgeRepositoryError> {
        let manifest_path = layout_dir.join(REPOSITORY_LAYOUT_MANIFEST_FILE);
        let manifest_raw = fs::read_to_string(&manifest_path).map_err(|error| {
            KnowledgeRepositoryError::LoadFailed {
                path: manifest_path.display().to_string(),
                message: error.to_string(),
            }
        })?;

        let manifest: KnowledgeRepositoryLayoutManifest = serde_json::from_str(&manifest_raw)
            .map_err(|error| KnowledgeRepositoryError::DeserializeFailed {
                path: manifest_path.display().to_string(),
                message: error.to_string(),
            })?;

        let bundle_path = layout_dir.join(manifest.bundle_file);
        Self::load_from_path(&bundle_path)
    }

    // 2026-04-08 CST: 这里提供最小 metadata 精确过滤，原因是 phase 2 第一阶段只需要把通用过滤
    // 边界钉住，不需要提前实现更复杂的检索 DSL。
    // 目的：让节点过滤统一围绕 node.metadata 进行。
    pub fn filtered_node_ids<'a>(&'a self, filter: &MetadataFilter) -> Vec<&'a str> {
        self.bundle
            .nodes
            .iter()
            .filter(|node| {
                let exact_matches_all_satisfied =
                    filter.exact_matches.iter().all(|(key, value)| {
                        node.metadata
                            .get(key)
                            .is_some_and(|candidate| candidate == value)
                    });

                let concept_scope_satisfied = filter.concept_scope.is_empty()
                    || node
                        .concept_ids
                        .iter()
                        .any(|concept_id| filter.concept_scope.contains(concept_id));

                exact_matches_all_satisfied && concept_scope_satisfied
            })
            .map(|node| node.id.as_str())
            .collect()
    }

    // 2026-04-08 CST: 这里提供仓储到 ontology store 的标准入口，原因是 bundle 的直接消费者
    // 仍应优先走 foundation 既有查询层，而不是自行重建 schema。
    // 目的：让仓储层与导航内核平滑衔接。
    pub fn to_ontology_store(&self) -> Result<OntologyStore, KnowledgeRepositoryError> {
        self.bundle
            .to_ontology_store()
            .map_err(
                |error: OntologySchemaError| KnowledgeRepositoryError::InvalidOntology {
                    message: format!("{error:?}"),
                },
            )
    }

    // 2026-04-08 CST: 这里提供仓储到 graph store 的标准入口，原因是 retrieval 仍然消费 graph store，
    // 仓储层应提供稳定桥接方法。
    // 目的：让持久化后的知识包可以重新进入现有导航链。
    pub fn to_graph_store(&self) -> KnowledgeGraphStore {
        self.bundle.to_graph_store()
    }
}

// 2026-04-08 CST: 这里定义最小布局 manifest，原因是目录布局标准除了实际数据文件，
// 还需要一个轻量描述文件来固定版本、入口文件名和基础计数信息。
// 目的：让布局目录具备可读、可检查、可演进的最小元数据。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct KnowledgeRepositoryLayoutManifest {
    layout_version: String,
    bundle_file: String,
    schema_version: String,
    concept_count: usize,
    relation_count: usize,
    node_count: usize,
    edge_count: usize,
}

impl KnowledgeRepositoryLayoutManifest {
    // 2026-04-08 CST: 这里从 bundle 构造布局 manifest，原因是当前布局规范只需要暴露
    // 最小的版本、入口文件与计数信息，不需要再引入更重的仓储摘要对象。
    // 目的：让布局写入与后续检查围绕同一份稳定元数据展开。
    fn from_bundle(bundle: &KnowledgeBundle) -> Self {
        Self {
            layout_version: REPOSITORY_LAYOUT_VERSION.to_string(),
            bundle_file: REPOSITORY_LAYOUT_BUNDLE_FILE.to_string(),
            schema_version: bundle.schema_version.clone(),
            concept_count: bundle.concepts.len(),
            relation_count: bundle.relations.len(),
            node_count: bundle.nodes.len(),
            edge_count: bundle.edges.len(),
        }
    }
}

// 2026-04-08 CST: 这里抽 staging 写入辅助函数，原因是 repository 进入布局标准化阶段后，
// 单次直接写目标文件会继续暴露半成品文件风险。
// 目的：统一走“同目录 staging 写入 -> 再替换正式文件”的最小安全路径。
fn write_json_with_staging<T: Serialize>(
    path: &Path,
    value: &T,
) -> Result<(), KnowledgeRepositoryError> {
    let serialized = serde_json::to_string_pretty(value).map_err(|error| {
        KnowledgeRepositoryError::SaveFailed {
            path: path.display().to_string(),
            message: error.to_string(),
        }
    })?;

    let staging_path = staging_path_for(path);

    fs::write(&staging_path, serialized).map_err(|error| KnowledgeRepositoryError::SaveFailed {
        path: staging_path.display().to_string(),
        message: error.to_string(),
    })?;

    if path.exists() {
        fs::remove_file(path).map_err(|error| KnowledgeRepositoryError::SaveFailed {
            path: path.display().to_string(),
            message: error.to_string(),
        })?;
    }

    fs::rename(&staging_path, path).map_err(|error| KnowledgeRepositoryError::SaveFailed {
        path: path.display().to_string(),
        message: error.to_string(),
    })?;

    Ok(())
}

// 2026-04-08 CST: 这里生成同目录 staging 文件路径，原因是替换式写入必须保证临时文件和正式文件
// 位于同一目录，才能尽量降低跨目录移动带来的额外风险。
// 目的：为 repository 持久化统一 staging 命名策略。
fn staging_path_for(path: &Path) -> std::path::PathBuf {
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("repository");

    path.with_file_name(format!("{file_name}.staging-{unique_suffix}.tmp"))
}
