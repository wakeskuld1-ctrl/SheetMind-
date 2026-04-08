use std::time::{SystemTime, UNIX_EPOCH};

use excel_skill::ops::foundation::knowledge_bundle::KnowledgeBundle;
use excel_skill::ops::foundation::knowledge_record::{EvidenceRef, KnowledgeEdge, KnowledgeNode};
use excel_skill::ops::foundation::knowledge_repository::{
    KnowledgeRepository, KnowledgeRepositoryError, MetadataFilter,
};
use excel_skill::ops::foundation::ontology_schema::{
    OntologyConcept, OntologyRelation, OntologyRelationType,
};

// 2026-04-08 CST: 这里先补仓储持久化红测，原因是 foundation 现在只有内存内核，
// phase 2 第一阶段必须先证明标准知识包可以稳定落盘并读回。
// 目的：钉住“标准包 <-> 本地文件”这一条最小通用能力。
#[test]
fn knowledge_repository_persists_bundle_and_loads_it_back() {
    let repository =
        KnowledgeRepository::new(sample_bundle()).expect("repository should accept sample bundle");
    let path = temp_json_path("knowledge-repository-roundtrip");

    repository
        .save_to_path(&path)
        .expect("repository should persist bundle");

    let loaded = KnowledgeRepository::load_from_path(&path)
        .expect("repository should load persisted bundle");

    assert_eq!(loaded.bundle().schema_version, "foundation.v1");
    assert_eq!(
        loaded
            .bundle()
            .nodes
            .first()
            .and_then(|node| node.metadata.get("domain")),
        Some(&"finance".to_string())
    );

    let _ = std::fs::remove_file(path);
}

// 2026-04-08 CST: 这里补标准布局目录持久化红测，原因是当前 repository 只有单文件落盘能力，
// 还没有正式的文件布局标准。
// 目的：钉住最小标准布局契约，先要求输出 `bundle.json` 和 `repository.manifest.json`。
#[test]
fn knowledge_repository_persists_standard_layout_directory() {
    let repository =
        KnowledgeRepository::new(sample_bundle()).expect("repository should accept sample bundle");
    let layout_dir = temp_layout_dir("knowledge-repository-layout");

    repository
        .save_to_layout_dir(&layout_dir)
        .expect("repository should persist standard layout directory");

    let bundle_path = layout_dir.join("bundle.json");
    let manifest_path = layout_dir.join("repository.manifest.json");

    assert!(bundle_path.exists());
    assert!(manifest_path.exists());

    let manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&manifest_path).expect("manifest should be readable"),
    )
    .expect("manifest should be valid json");

    assert_eq!(
        manifest["layout_version"],
        "foundation.repository-layout.v1"
    );
    assert_eq!(manifest["bundle_file"], "bundle.json");
    assert_eq!(manifest["schema_version"], "foundation.v1");
    assert_eq!(manifest["node_count"], 2);

    let _ = std::fs::remove_dir_all(layout_dir);
}

// 2026-04-08 CST: 这里补标准布局目录读回红测，原因是布局标准如果只有写没有读，
// 还不能算正式可复用契约。
// 目的：钉住 layout dir -> repository 的最小回读闭环。
#[test]
fn knowledge_repository_loads_repository_from_standard_layout_directory() {
    let repository =
        KnowledgeRepository::new(sample_bundle()).expect("repository should accept sample bundle");
    let layout_dir = temp_layout_dir("knowledge-repository-layout-roundtrip");

    repository
        .save_to_layout_dir(&layout_dir)
        .expect("repository should persist standard layout directory");

    let loaded = KnowledgeRepository::load_from_layout_dir(&layout_dir)
        .expect("repository should load from standard layout directory");

    assert_eq!(loaded.bundle().schema_version, "foundation.v1");
    assert_eq!(loaded.bundle().nodes.len(), 2);

    let _ = std::fs::remove_dir_all(layout_dir);
}

// 2026-04-08 CST: 这里补 metadata 精确过滤测试，原因是当前 retrieval 还没有 metadata filter，
// 但 phase 2 第一阶段至少要先把通用过滤契约立住。
// 目的：让后续任意业务域都只需要适配标准 metadata，而不是改 foundation 检索层。
#[test]
fn knowledge_repository_filters_nodes_by_exact_metadata_match() {
    let repository =
        KnowledgeRepository::new(sample_bundle()).expect("repository should accept sample bundle");

    let node_ids =
        repository.filtered_node_ids(&MetadataFilter::new().with_exact_match("domain", "finance"));

    assert_eq!(node_ids, vec!["node-revenue-1"]);
}

// 2026-04-08 CST: 这里补多字段 AND 过滤红测，原因是当前 metadata 过滤虽然已经支持 exact-match，
// 但还不能证明多个字段会被同时要求满足。
// 目的：钉住 foundation 下一阶段的最小扩展行为，避免过滤语义退化成“任一命中即可”。
#[test]
fn knowledge_repository_requires_all_exact_matches() {
    let repository =
        KnowledgeRepository::new(sample_bundle()).expect("repository should accept sample bundle");

    let node_ids = repository.filtered_node_ids(
        &MetadataFilter::new()
            .with_exact_match("domain", "finance")
            .with_exact_match("source_type", "table"),
    );

    assert_eq!(node_ids, vec!["node-revenue-1"]);
}

// 2026-04-08 CST: 这里补 concept scope 红测，原因是方案 B 要求在 metadata 过滤前先能按概念域收窄候选节点，
// 否则 foundation 过滤精度仍然不够。
// 目的：钉住 “concept scope + exact-match” 的组合过滤契约。
#[test]
fn knowledge_repository_limits_matches_to_concept_scope() {
    let repository =
        KnowledgeRepository::new(sample_bundle()).expect("repository should accept sample bundle");

    let node_ids = repository.filtered_node_ids(
        &MetadataFilter::new()
            .with_exact_match("domain", "finance")
            .with_concept_id("invoice"),
    );

    assert!(node_ids.is_empty());
}

// 2026-04-08 CST: 这里补 node id 去重失败测试，原因是持久化仓储如果接收重复 node id，
// 后续 graph store 与查询层都会出现不稳定覆盖。
// 目的：把仓储构建期的最小失败边界显式化。
#[test]
fn knowledge_repository_rejects_duplicate_node_ids() {
    let error = KnowledgeRepository::new(duplicate_node_bundle())
        .expect_err("repository should reject duplicate node ids");

    assert_eq!(
        error,
        KnowledgeRepositoryError::DuplicateNodeId {
            node_id: "node-revenue-1".to_string(),
        }
    );
}

// 2026-04-08 CST: 这里集中构造最小标准知识包，原因是当前测试只验证通用仓储能力，
// 不应掺入业务链对象或运行时状态。
// 目的：让持久化与过滤测试建立在同一份小样本上。
fn sample_bundle() -> KnowledgeBundle {
    KnowledgeBundle::new(
        "foundation.v1",
        vec![
            OntologyConcept::new("revenue", "Revenue").with_alias("sales"),
            OntologyConcept::new("invoice", "Invoice"),
        ],
        vec![OntologyRelation {
            from_concept_id: "revenue".to_string(),
            to_concept_id: "invoice".to_string(),
            relation_type: OntologyRelationType::DependsOn,
        }],
        vec![
            KnowledgeNode::new(
                "node-revenue-1",
                "Revenue Summary",
                "Revenue comes from invoices.",
            )
            .with_concept_id("revenue")
            .with_metadata_entry("domain", "finance")
            .with_metadata_entry("source_type", "table")
            .with_evidence_ref(EvidenceRef::new("sheet:sales", "A1:B12")),
            KnowledgeNode::new(
                "node-policy-1",
                "Policy Summary",
                "Policy notes are operational guidance.",
            )
            .with_concept_id("invoice")
            .with_metadata_entry("domain", "operations")
            .with_metadata_entry("source_type", "memo")
            .with_evidence_ref(EvidenceRef::new("memo:ops", "P1")),
        ],
        vec![KnowledgeEdge::new(
            "node-revenue-1",
            "node-policy-1",
            OntologyRelationType::References,
        )],
    )
}

// 2026-04-08 CST: 这里构造重复 node id 的异常样本，原因是仓储错误边界需要真实样本触发，
// 不能靠手工伪造错误值。
// 目的：验证 duplicate node id 会在 repository 构建时被及时拦住。
fn duplicate_node_bundle() -> KnowledgeBundle {
    KnowledgeBundle::new(
        "foundation.v1",
        vec![OntologyConcept::new("revenue", "Revenue")],
        vec![],
        vec![
            KnowledgeNode::new(
                "node-revenue-1",
                "Revenue Summary",
                "Revenue comes from invoices.",
            )
            .with_concept_id("revenue"),
            KnowledgeNode::new(
                "node-revenue-1",
                "Revenue Summary Copy",
                "Duplicated node id should fail.",
            )
            .with_concept_id("revenue"),
        ],
        vec![],
    )
}

// 2026-04-08 CST: 这里抽出最小临时路径生成器，原因是当前测试只需要唯一文件名，
// 不值得为了这一点额外引入新的开发依赖。
// 目的：在纯标准库前提下稳定创建临时 JSON 路径。
fn temp_json_path(prefix: &str) -> std::path::PathBuf {
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();

    std::env::temp_dir().join(format!("{prefix}-{unique_suffix}.json"))
}

// 2026-04-08 CST: 这里抽布局目录临时路径生成器，原因是本轮开始给 repository 引入标准布局目录，
// 测试需要稳定生成唯一目录名。
// 目的：在纯标准库前提下构造不互相冲突的 layout dir。
fn temp_layout_dir(prefix: &str) -> std::path::PathBuf {
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();

    std::env::temp_dir().join(format!("{prefix}-{unique_suffix}"))
}
