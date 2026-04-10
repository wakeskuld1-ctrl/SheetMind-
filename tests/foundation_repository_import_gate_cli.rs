mod common;

use excel_skill::ops::foundation::knowledge_bundle::KnowledgeBundle;
use excel_skill::ops::foundation::knowledge_record::KnowledgeNode;
use excel_skill::ops::foundation::knowledge_repository::KnowledgeRepository;
use excel_skill::ops::foundation::ontology_schema::OntologyConcept;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::common::run_cli_with_json;

// 2026-04-10 CST: 这里先补 foundation repository import gate 的 catalog 红测，原因是方案B1不是只做内部批量消费 helper，
// 目的：先锁住“导入接入 gate”必须成为正式 foundation Tool，避免实现落成后仍无法被 CLI / Skill 发现。
#[test]
fn foundation_repository_import_gate_cli_is_cataloged() {
    let output = run_cli_with_json(r#"{"tool":"tool_catalog","args":{}}"#);
    let tool_catalog = output["data"]["tool_catalog"]
        .as_array()
        .expect("tool catalog should be an array");
    let foundation_catalog = output["data"]["tool_catalog_modules"]["foundation"]
        .as_array()
        .expect("foundation tool group should be an array");

    assert!(
        tool_catalog
            .iter()
            .filter_map(|item| item.as_str())
            .any(|item| item == "foundation_repository_import_gate")
    );
    assert!(
        foundation_catalog
            .iter()
            .filter_map(|item| item.as_str())
            .any(|item| item == "foundation_repository_import_gate")
    );
}

// 2026-04-10 CST: 这里先补方案B1的主红测，原因是导入接入层最重要的不是再重复 batch 报告，
// 目的：而是锁住“accepted / rejected 列表 + 阻塞原因汇总 + 下一阶段是否允许继续”这组最小消费层合同。
#[test]
fn foundation_repository_import_gate_cli_returns_acceptance_and_rejection_lists() {
    let passing_repository_layout_dir = create_repository_layout_dir(
        "foundation_repository_import_gate_cli_passing",
        legacy_metadata_bundle(),
    );
    let failing_repository_layout_dir = create_repository_layout_dir(
        "foundation_repository_import_gate_cli_failing",
        missing_required_field_bundle(),
    );
    let request = json!({
        "tool": "foundation_repository_import_gate",
        "args": {
            "repository_layout_dirs": [
                passing_repository_layout_dir.to_string_lossy(),
                failing_repository_layout_dir.to_string_lossy()
            ],
            "metadata_schema": sample_metadata_schema_payload("foundation.v1")
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["next_stage_allowed"], true);
    assert_eq!(output["data"]["all_repositories_accepted"], false);
    assert_eq!(output["data"]["accepted_repository_count"], 1);
    assert_eq!(output["data"]["rejected_repository_count"], 1);
    assert_eq!(output["data"]["blocking_issue_count_total"], 1);
    assert_eq!(output["data"]["non_blocking_issue_count_total"], 2);
    assert_eq!(
        output["data"]["blocking_issue_kind_summary"][0],
        "missing_required_field"
    );
    assert_eq!(
        output["data"]["accepted_repositories"][0]["repository_layout_dir"],
        passing_repository_layout_dir.to_string_lossy().to_string()
    );
    assert_eq!(
        output["data"]["accepted_repositories"][0]["gate_passed"],
        true
    );
    assert_eq!(
        output["data"]["rejected_repositories"][0]["repository_layout_dir"],
        failing_repository_layout_dir.to_string_lossy().to_string()
    );
    assert_eq!(
        output["data"]["rejected_repositories"][0]["gate_passed"],
        false
    );
    assert_eq!(
        output["data"]["rejected_repositories"][0]["blocking_issues"][0]["kind"],
        "missing_required_field"
    );
}

// 2026-04-10 CST: 这里补“全阻塞时禁止进入下一阶段”的红测，原因是方案B1明确要给上层一个可直接消费的继续执行判定，
// 目的：锁住 next_stage_allowed 的语义是“至少有一个仓库可继续”，而不是“批次接口成功返回就继续”。
#[test]
fn foundation_repository_import_gate_cli_blocks_next_stage_when_all_repositories_are_rejected() {
    let failing_repository_layout_dir = create_repository_layout_dir(
        "foundation_repository_import_gate_cli_all_rejected",
        missing_required_field_bundle(),
    );
    let request = json!({
        "tool": "foundation_repository_import_gate",
        "args": {
            "repository_layout_dirs": [
                failing_repository_layout_dir.to_string_lossy()
            ],
            "metadata_schema": sample_metadata_schema_payload("foundation.v1")
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["next_stage_allowed"], false);
    assert_eq!(output["data"]["all_repositories_accepted"], false);
    assert_eq!(output["data"]["accepted_repository_count"], 0);
    assert_eq!(output["data"]["rejected_repository_count"], 1);
}

// 2026-04-10 CST: 这里复用标准 layout 持久化夹具，原因是方案B1消费的是 repository layout dir 批次而不是内存对象，
// 目的：让导入接入 gate 的红测继续覆盖真实仓库输入边界，而不是退回伪造批量结果。
fn create_repository_layout_dir(prefix: &str, bundle: KnowledgeBundle) -> PathBuf {
    let repository =
        KnowledgeRepository::new(bundle).expect("repository fixture should build successfully");
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let layout_dir = PathBuf::from("tests")
        .join("runtime_fixtures")
        .join("foundation_repository_import_gate")
        .join(format!("{prefix}_{unique_suffix}"));
    fs::create_dir_all(&layout_dir).expect("foundation repository layout dir should exist");
    repository
        .save_to_layout_dir(&layout_dir)
        .expect("repository fixture should persist standard layout dir");
    layout_dir
}

// 2026-04-10 CST: 这里复用仅含非阻塞问题的 repository 样例，原因是方案B1要证明“可接入”不等于“完全 clean”，
// 目的：锁住 accepted 列表沿用 gate 语义，而不是把 alias / deprecated 仓库误判成 rejected。
fn legacy_metadata_bundle() -> KnowledgeBundle {
    KnowledgeBundle::new(
        "foundation.v1",
        vec![OntologyConcept::new("revenue", "Revenue")],
        vec![],
        vec![
            KnowledgeNode::new(
                "node-legacy-1",
                "Legacy Revenue Summary",
                "Legacy metadata still exists.",
            )
            .with_metadata_entry("legacy_domain", "finance")
            .with_metadata_entry("biz_domain", "finance"),
        ],
        vec![],
    )
}

// 2026-04-10 CST: 这里复用阻塞样例，原因是方案B1必须给出“拒绝接入”的明确 blocking case，
// 目的：让 rejected 列表与 blocking_issue_kind_summary 有稳定、可验证的来源。
fn missing_required_field_bundle() -> KnowledgeBundle {
    KnowledgeBundle::new(
        "foundation.v1",
        vec![OntologyConcept::new("revenue", "Revenue")],
        vec![],
        vec![
            KnowledgeNode::new(
                "node-invalid-1",
                "Revenue Missing Domain",
                "This node misses the required domain metadata.",
            )
            .with_concept_id("revenue")
            .with_metadata_entry("source_type", "table"),
        ],
        vec![],
    )
}

// 2026-04-10 CST: 这里继续沿用单批共用 schema 的输入边界，原因是方案B1是 A1 的上层消费层，
// 目的：避免在导入接入层提前扩成“每仓库独立 schema”，把当前主线做散。
fn sample_metadata_schema_payload(schema_version: &str) -> serde_json::Value {
    json!({
        "schema_version": schema_version,
        "fields": [
            {
                "key": "domain",
                "value_type": "String",
                "description": null,
                "allowed_values": ["finance", "operations"],
                "deprecated": false,
                "replaced_by": null,
                "aliases": []
            },
            {
                "key": "source_type",
                "value_type": "String",
                "description": null,
                "allowed_values": ["table", "memo"],
                "deprecated": false,
                "replaced_by": null,
                "aliases": []
            },
            {
                "key": "legacy_domain",
                "value_type": "String",
                "description": null,
                "allowed_values": [],
                "deprecated": true,
                "replaced_by": "domain",
                "aliases": ["biz_domain"]
            }
        ],
        "concept_policies": [
            {
                "concept_id": "revenue",
                "allowed_field_keys": ["domain", "source_type"],
                "required_field_keys": ["domain"]
            }
        ]
    })
}
