use std::time::{SystemTime, UNIX_EPOCH};

use excel_skill::ops::foundation::knowledge_ingestion::{
    KnowledgeIngestionError, load_bundle_from_json_path, load_repository_from_jsonl_path,
};

// 2026-04-08 CST: 这里先补标准 bundle JSON 导入红测，原因是 knowledge_ingestion 的第一条正式能力
// 应该是“直接吃标准包文件”，而不是一上来就把所有原始格式耦合进 foundation。
// 目的：先钉住 bundle JSON -> KnowledgeBundle 的最小导入契约。
#[test]
fn knowledge_ingestion_loads_bundle_from_json_file() {
    let path = temp_file_path("knowledge-ingestion-bundle", "json");

    std::fs::write(
        &path,
        r#"{
  "schema_version": "foundation.v1",
  "concepts": [
    {
      "id": "revenue",
      "name": "Revenue",
      "aliases": ["sales"]
    }
  ],
  "relations": [],
  "nodes": [
    {
      "id": "node-revenue-1",
      "title": "Revenue Summary",
      "body": "Revenue comes from invoices.",
      "concept_ids": ["revenue"],
      "metadata": {
        "domain": "finance"
      },
      "evidence_refs": [
        {
          "source_ref": "sheet:sales",
          "locator": "A1:B12"
        }
      ]
    }
  ],
  "edges": []
}"#,
    )
    .expect("bundle json fixture should be written");

    let bundle = load_bundle_from_json_path(&path).expect("bundle should load from json path");

    assert_eq!(bundle.schema_version, "foundation.v1");
    assert_eq!(bundle.concepts.len(), 1);
    assert_eq!(bundle.nodes.len(), 1);
    assert_eq!(
        bundle.nodes[0].metadata.get("domain"),
        Some(&"finance".to_string())
    );

    let _ = std::fs::remove_file(path);
}

// 2026-04-08 CST: 这里补标准记录 JSONL 导入红测，原因是方案 B 的价值就在于除了 bundle JSON，
// 还能接收逐行标准记录并组装成仓储。
// 目的：钉住 JSONL -> KnowledgeRepository 的最小导入闭环。
#[test]
fn knowledge_ingestion_loads_repository_from_jsonl_records() {
    let path = temp_file_path("knowledge-ingestion-records", "jsonl");

    std::fs::write(
        &path,
        r#"{"record_type":"bundle_header","schema_version":"foundation.v1"}
{"record_type":"concept","id":"revenue","name":"Revenue","aliases":["sales"]}
{"record_type":"concept","id":"invoice","name":"Invoice","aliases":[]}
{"record_type":"relation","from_concept_id":"revenue","to_concept_id":"invoice","relation_type":"DependsOn"}
{"record_type":"node","id":"node-revenue-1","title":"Revenue Summary","body":"Revenue comes from invoices.","concept_ids":["revenue"],"metadata":{"domain":"finance","source_type":"table"},"evidence_refs":[{"source_ref":"sheet:sales","locator":"A1:B12"}]}
{"record_type":"edge","from_node_id":"node-revenue-1","to_node_id":"node-revenue-1","relation_type":"References"}"#,
    )
    .expect("jsonl fixture should be written");

    let repository = load_repository_from_jsonl_path(&path)
        .expect("repository should load from standard jsonl records");

    assert_eq!(repository.bundle().schema_version, "foundation.v1");
    assert_eq!(
        repository.filtered_node_ids(
            &excel_skill::ops::foundation::knowledge_repository::MetadataFilter::new()
                .with_exact_match("domain", "finance"),
        ),
        vec!["node-revenue-1"]
    );

    let _ = std::fs::remove_file(path);
}

// 2026-04-08 CST: 这里补 JSONL 行号错误红测，原因是标准记录导入一旦失败，最重要的是告诉调用方
// “哪一行坏了”，否则排查成本会非常高。
// 目的：钉住 knowledge_ingestion 的最小可诊断错误边界。
#[test]
fn knowledge_ingestion_reports_jsonl_line_number_for_invalid_record() {
    let path = temp_file_path("knowledge-ingestion-invalid-record", "jsonl");

    std::fs::write(
        &path,
        r#"{"record_type":"bundle_header","schema_version":"foundation.v1"}
{"record_type":"edge","from_node_id":"node-revenue-1","to_node_id":"node-revenue-1","relation_type":"invalid_relation"}"#,
    )
    .expect("invalid jsonl fixture should be written");

    let error = load_repository_from_jsonl_path(&path)
        .expect_err("invalid jsonl line should fail with line number");

    assert!(matches!(
        error,
        KnowledgeIngestionError::JsonlRecordDeserializeFailed { line_number: 2, .. }
    ));

    let _ = std::fs::remove_file(path);
}

// 2026-04-08 CST: 这里抽最小临时文件路径生成器，原因是 ingestion 红测需要同时写 json/jsonl 夹具，
// 但还不值得引入额外测试依赖。
// 目的：用标准库稳定生成唯一临时文件路径。
fn temp_file_path(prefix: &str, extension: &str) -> std::path::PathBuf {
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();

    std::env::temp_dir().join(format!("{prefix}-{unique_suffix}.{extension}"))
}
