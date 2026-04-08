use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::ops::foundation::knowledge_bundle::KnowledgeBundle;
use crate::ops::foundation::knowledge_record::{EvidenceRef, KnowledgeEdge, KnowledgeNode};
use crate::ops::foundation::knowledge_repository::{KnowledgeRepository, KnowledgeRepositoryError};
use crate::ops::foundation::ontology_schema::{
    OntologyConcept, OntologyRelation, OntologyRelationType,
};

// 2026-04-08 CST: 这里定义导入错误边界，原因是 knowledge_ingestion 需要同时处理文件读取、
// JSON 反序列化、JSONL 行级反序列化和 repository 构建失败，不能全部混成一个通用字符串错误。
// 目的：让调用方能区分“文件坏了”“哪一行坏了”“还是构建期校验失败”。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KnowledgeIngestionError {
    ReadFailed {
        path: String,
        message: String,
    },
    JsonDeserializeFailed {
        path: String,
        message: String,
    },
    JsonlRecordDeserializeFailed {
        path: String,
        line_number: usize,
        message: String,
    },
    MissingBundleHeader {
        path: String,
    },
    DuplicateBundleHeader {
        path: String,
    },
    RepositoryBuildFailed {
        message: String,
    },
}

// 2026-04-08 CST: 这里提供标准 bundle JSON 导入入口，原因是方案 B 的第一条路径仍应优先支持
// “已经整理好的标准包文件”，这样 foundation 才能先具备最小、直接的导入能力。
// 目的：形成 json file -> KnowledgeBundle 的稳定入口。
pub fn load_bundle_from_json_path(path: &Path) -> Result<KnowledgeBundle, KnowledgeIngestionError> {
    let raw = fs::read_to_string(path).map_err(|error| KnowledgeIngestionError::ReadFailed {
        path: path.display().to_string(),
        message: error.to_string(),
    })?;

    serde_json::from_str(&raw).map_err(|error| KnowledgeIngestionError::JsonDeserializeFailed {
        path: path.display().to_string(),
        message: error.to_string(),
    })
}

// 2026-04-08 CST: 这里提供标准 bundle JSON -> repository 桥接，原因是上层大多数场景最终消费的
// 不是原始 bundle，而是带校验的标准仓储。
// 目的：让 bundle JSON 导入路径可以直接落到 repository。
pub fn load_repository_from_json_path(
    path: &Path,
) -> Result<KnowledgeRepository, KnowledgeIngestionError> {
    let bundle = load_bundle_from_json_path(path)?;
    build_repository(bundle)
}

// 2026-04-08 CST: 这里提供标准记录 JSONL 导入入口，原因是方案 B 的核心就是允许逐行记录组装
// 标准知识包，而不是只支持一次性全包 JSON。
// 目的：形成 jsonl file -> KnowledgeBundle 的最小导入闭环。
pub fn load_bundle_from_jsonl_path(
    path: &Path,
) -> Result<KnowledgeBundle, KnowledgeIngestionError> {
    let raw = fs::read_to_string(path).map_err(|error| KnowledgeIngestionError::ReadFailed {
        path: path.display().to_string(),
        message: error.to_string(),
    })?;

    let mut schema_version = None;
    let mut concepts = Vec::new();
    let mut relations = Vec::new();
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    for (line_index, line) in raw.lines().enumerate() {
        let line_number = line_index + 1;
        if line.trim().is_empty() {
            continue;
        }

        let record: KnowledgeBundleJsonlRecord = serde_json::from_str(line).map_err(|error| {
            KnowledgeIngestionError::JsonlRecordDeserializeFailed {
                path: path.display().to_string(),
                line_number,
                message: error.to_string(),
            }
        })?;

        match record {
            KnowledgeBundleJsonlRecord::BundleHeader {
                schema_version: incoming_schema_version,
            } => {
                if schema_version.replace(incoming_schema_version).is_some() {
                    return Err(KnowledgeIngestionError::DuplicateBundleHeader {
                        path: path.display().to_string(),
                    });
                }
            }
            KnowledgeBundleJsonlRecord::Concept { id, name, aliases } => {
                let mut concept = OntologyConcept::new(id, name);
                for alias in aliases {
                    concept = concept.with_alias(alias);
                }
                concepts.push(concept);
            }
            KnowledgeBundleJsonlRecord::Relation {
                from_concept_id,
                to_concept_id,
                relation_type,
            } => {
                relations.push(OntologyRelation {
                    from_concept_id,
                    to_concept_id,
                    relation_type,
                });
            }
            KnowledgeBundleJsonlRecord::Node {
                id,
                title,
                body,
                concept_ids,
                metadata,
                evidence_refs,
            } => {
                nodes.push(KnowledgeNode {
                    id,
                    title,
                    body,
                    concept_ids,
                    metadata,
                    evidence_refs,
                });
            }
            KnowledgeBundleJsonlRecord::Edge {
                from_node_id,
                to_node_id,
                relation_type,
            } => {
                edges.push(KnowledgeEdge::new(from_node_id, to_node_id, relation_type));
            }
        }
    }

    let schema_version =
        schema_version.ok_or_else(|| KnowledgeIngestionError::MissingBundleHeader {
            path: path.display().to_string(),
        })?;

    Ok(KnowledgeBundle::new(
        schema_version,
        concepts,
        relations,
        nodes,
        edges,
    ))
}

// 2026-04-08 CST: 这里提供标准记录 JSONL -> repository 桥接，原因是逐行导入后仍需复用
// repository 的一致性校验，避免导入器自己复制一套校验逻辑。
// 目的：让 JSONL 路径与 JSON 路径最终收敛到同一套仓储边界。
pub fn load_repository_from_jsonl_path(
    path: &Path,
) -> Result<KnowledgeRepository, KnowledgeIngestionError> {
    let bundle = load_bundle_from_jsonl_path(path)?;
    build_repository(bundle)
}

// 2026-04-08 CST: 这里抽公共 repository 构建桥接，原因是 JSON 与 JSONL 两条路径最终都要走
// 同一套标准仓储校验，不值得复制错误映射逻辑。
// 目的：保持导入器内部职责单一且复用一致。
fn build_repository(
    bundle: KnowledgeBundle,
) -> Result<KnowledgeRepository, KnowledgeIngestionError> {
    KnowledgeRepository::new(bundle).map_err(|error: KnowledgeRepositoryError| {
        KnowledgeIngestionError::RepositoryBuildFailed {
            message: format!("{error:?}"),
        }
    })
}

// 2026-04-08 CST: 这里定义标准记录 JSONL 的最小 tagged-record 格式，原因是方案 B 需要在不引入
// 目录布局的前提下，把 concept / relation / node / edge 收进一条稳定的导入契约。
// 目的：用单文件 JSONL 表达“标准包分行记录”，为后续 ingestion 扩展保留兼容入口。
#[derive(Debug, Deserialize)]
#[serde(tag = "record_type", rename_all = "snake_case")]
enum KnowledgeBundleJsonlRecord {
    BundleHeader {
        schema_version: String,
    },
    Concept {
        id: String,
        name: String,
        #[serde(default)]
        aliases: Vec<String>,
    },
    Relation {
        from_concept_id: String,
        to_concept_id: String,
        relation_type: OntologyRelationType,
    },
    Node {
        id: String,
        title: String,
        body: String,
        #[serde(default)]
        concept_ids: Vec<String>,
        #[serde(default)]
        metadata: BTreeMap<String, String>,
        #[serde(default)]
        evidence_refs: Vec<EvidenceRef>,
    },
    Edge {
        from_node_id: String,
        to_node_id: String,
        relation_type: OntologyRelationType,
    },
}
