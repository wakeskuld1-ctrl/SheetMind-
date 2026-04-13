mod common;

use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::common::run_cli_with_json;

// 2026-04-13 CST: 这里先补 foundation design tools 的 catalog 红测，原因是方案C要求这两个能力
// 成为正式 foundation Tool，而不是只留在规则或 Skill 里；
// 目的：锁住 CLI / Skill 对这两个入口的可发现性。
#[test]
fn foundation_design_tools_cli_are_cataloged() {
    let output = run_cli_with_json(r#"{"tool":"tool_catalog","args":{}}"#);
    let tool_catalog = output["data"]["tool_catalog"]
        .as_array()
        .expect("tool catalog should be an array");
    let foundation_catalog = output["data"]["tool_catalog_modules"]["foundation"]
        .as_array()
        .expect("foundation tool group should be an array");

    for tool_name in ["foundation_design_skeleton", "foundation_design_gap_audit"] {
        assert!(
            tool_catalog
                .iter()
                .filter_map(|item| item.as_str())
                .any(|item| item == tool_name),
            "tool catalog should include `{tool_name}`"
        );
        assert!(
            foundation_catalog
                .iter()
                .filter_map(|item| item.as_str())
                .any(|item| item == tool_name),
            "foundation tool group should include `{tool_name}`"
        );
    }
}

// 2026-04-13 CST: 这里先补 foundation_design_skeleton 主红测，原因是设计 Tool 的第一职责是稳定输出
// Mermaid 边界图、warning 与结构计数；
// 目的：锁住方案C第一阶段的标准化设计产物形态。
#[test]
fn foundation_design_skeleton_cli_returns_json_first_result_and_optional_visuals() {
    let request = json!({
        "tool": "foundation_design_skeleton",
        "args": sample_design_request()
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["feature_name"], "foundation-design-kernel");
    assert_eq!(output["data"]["layer_count"], 3);
    assert_eq!(output["data"]["module_count"], 3);
    assert_eq!(output["data"]["interface_count"], 2);
    assert_eq!(output["data"]["method_count"], 3);
    assert_eq!(output["data"]["test_scenario_count"], 2);
    assert!(
        output["data"]["visuals"]["layer_diagram_mermaid"]
            .as_str()
            .expect("layer diagram should be a string")
            .contains("flowchart TD")
    );
    assert!(
        output["data"]["visuals"]["dependency_diagram_mermaid"]
            .as_str()
            .expect("dependency diagram should be a string")
            .contains("route_module")
    );
    assert!(
        output["data"]["visuals"]["interface_diagram_mermaid"]
            .as_str()
            .expect("interface diagram should be a string")
            .contains("iface_design_service")
    );
    assert!(
        output["data"]["summary"]
            .as_str()
            .expect("summary should be a string")
            .contains("Feature `foundation-design-kernel` covers 3 layer(s)")
    );
}

// 2026-04-13 CST: 这里先补 foundation_design_gap_audit 主红测，原因是方案C要求设计骨架必须能与 graphify
// 现状图联动做“设计 vs 成品”差距收口；
// 目的：锁住 module / interface / method 三层的最小命中与缺口输出。
#[test]
fn foundation_design_gap_audit_cli_compares_design_with_graph_nodes() {
    let graph_path = create_graph_json_file(
        "foundation_design_gap_audit_cli",
        json!({
            "nodes": [
                {
                    "id": "route_module",
                    "label": "route_module.rs",
                    "source_file": "src/tools/route_module.rs"
                },
                {
                    "id": "design_service",
                    "label": "DesignService",
                    "source_file": "src/tools/design_service.rs"
                },
                {
                    "id": "design_service_build_skeleton",
                    "label": "build_skeleton()",
                    "source_file": "src/tools/design_service.rs"
                },
                {
                    "id": "design_service_audit_gap",
                    "label": "audit_gap()",
                    "source_file": "src/tools/design_service.rs"
                }
            ]
        }),
    );
    let mut request_args = sample_design_request();
    request_args["graph_path"] = json!(graph_path.to_string_lossy().to_string());
    let request = json!({
        "tool": "foundation_design_gap_audit",
        "args": request_args
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["graph_path"],
        graph_path.to_string_lossy().to_string()
    );
    assert_eq!(output["data"]["matched_module_count"], 2);
    assert_eq!(output["data"]["matched_interface_count"], 1);
    assert_eq!(output["data"]["matched_method_count"], 2);
    assert_eq!(output["data"]["missing_modules"], json!(["audit_module"]));
    assert_eq!(output["data"]["missing_interfaces"], json!(["gap_auditor"]));
    assert_eq!(output["data"]["missing_methods"], json!(["emit_report"]));
    assert!(
        output["data"]["warnings"]
            .as_array()
            .expect("warnings should be an array")
            .iter()
            .any(|item| item
                .as_str()
                .unwrap_or_default()
                .contains("missing modules"))
    );
}

// 2026-04-13 CST: 这里补 graph_path 自动发现回归测试，原因是用户明确要求 graphify 联动时优先直接读取
// `graph.json`，而不是依赖图形入口或手工传路径；
// 目的：锁住 foundation_design_gap_audit 在未显式传 graph_path 时会自动发现最新 src-map-* 图谱。
#[test]
fn foundation_design_gap_audit_cli_discovers_latest_graph_json_when_graph_path_is_missing() {
    let graph_path = create_discoverable_graph_json_file(
        "foundation_design_gap_audit_auto_discovery",
        json!({
            "nodes": [
                {
                    "id": "route_module",
                    "label": "Route Module",
                    "source_file": "src/tools/route_module.rs"
                },
                {
                    "id": "design_module",
                    "label": "Design Module",
                    "source_file": "src/tools/design_service.rs"
                },
                {
                    "id": "audit_module",
                    "label": "Audit Module",
                    "source_file": "src/tools/audit_service.rs"
                },
                {
                    "id": "design_service",
                    "label": "DesignService",
                    "source_file": "src/tools/design_service.rs"
                },
                {
                    "id": "gap_auditor",
                    "label": "GapAuditor",
                    "source_file": "src/tools/audit_service.rs"
                },
                {
                    "id": "build_skeleton",
                    "label": "build_skeleton()",
                    "source_file": "src/tools/design_service.rs"
                },
                {
                    "id": "audit_gap",
                    "label": "audit_gap()",
                    "source_file": "src/tools/audit_service.rs"
                },
                {
                    "id": "emit_report",
                    "label": "emit_report()",
                    "source_file": "src/tools/audit_service.rs"
                }
            ]
        }),
    );
    let request = json!({
        "tool": "foundation_design_gap_audit",
        "args": sample_design_request()
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["graph_path"],
        graph_path.to_string_lossy().to_string()
    );
    assert_eq!(output["data"]["missing_modules"], json!([]));
    assert_eq!(output["data"]["missing_interfaces"], json!([]));
    assert_eq!(output["data"]["missing_methods"], json!([]));
}

fn sample_design_request() -> serde_json::Value {
    json!({
        "feature_name": "foundation-design-kernel",
        "objective": "formalize design-first skeleton and implementation gap audit",
        "success_criteria": [
            "design skeleton can return stable JSON structure",
            "gap audit can compare design to graphify graph"
        ],
        "layers": [
            {
                "id": "entry_layer",
                "label": "Entry Layer",
                "depends_on": ["service_layer"]
            },
            {
                "id": "service_layer",
                "label": "Service Layer",
                "depends_on": ["audit_layer"]
            },
            {
                "id": "audit_layer",
                "label": "Audit Layer",
                "depends_on": []
            }
        ],
        "modules": [
            {
                "id": "route_module",
                "label": "Route Module",
                "layer_id": "entry_layer",
                "depends_on": ["design_module"],
                "source_files": ["src/tools/route_module.rs"]
            },
            {
                "id": "design_module",
                "label": "Design Module",
                "layer_id": "service_layer",
                "depends_on": ["audit_module"],
                "source_files": ["src/tools/design_service.rs"]
            },
            {
                "id": "audit_module",
                "label": "Audit Module",
                "layer_id": "audit_layer",
                "depends_on": [],
                "source_files": ["src/tools/audit_service.rs"]
            }
        ],
        "interfaces": [
            {
                "id": "design_service",
                "label": "DesignService",
                "module_id": "design_module",
                "kind": "service"
            },
            {
                "id": "gap_auditor",
                "label": "GapAuditor",
                "module_id": "audit_module",
                "kind": "service"
            }
        ],
        "methods": [
            {
                "id": "build_skeleton",
                "label": "build_skeleton",
                "interface_id": "design_service",
                "purpose": "render skeleton output"
            },
            {
                "id": "audit_gap",
                "label": "audit_gap",
                "interface_id": "gap_auditor",
                "purpose": "compare design with graph"
            },
            {
                "id": "emit_report",
                "label": "emit_report",
                "interface_id": "gap_auditor",
                "purpose": "assemble report payload"
            }
        ],
        "test_scenarios": [
            "skeleton tool returns JSON-first result",
            "gap audit reports missing module and method"
        ]
    })
}

fn create_graph_json_file(prefix: &str, payload: serde_json::Value) -> PathBuf {
    let fixture_dir = create_runtime_fixture_dir(prefix);
    let graph_path = fixture_dir.join("graph.json");
    fs::write(
        &graph_path,
        serde_json::to_string_pretty(&payload).expect("graph payload should serialize"),
    )
    .expect("graph fixture should be written");
    graph_path
}

fn create_discoverable_graph_json_file(prefix: &str, payload: serde_json::Value) -> PathBuf {
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let fixture_dir =
        Path::new("graphify-out").join(format!("src-map-zzzz-{prefix}-{unique_suffix}"));
    fs::create_dir_all(&fixture_dir).expect("discoverable graph fixture dir should exist");
    let graph_path = fixture_dir.join("graph.json");
    fs::write(
        &graph_path,
        serde_json::to_string_pretty(&payload).expect("graph payload should serialize"),
    )
    .expect("discoverable graph fixture should be written");
    graph_path
}

fn create_runtime_fixture_dir(prefix: &str) -> PathBuf {
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let fixture_dir = Path::new("tests")
        .join("runtime_fixtures")
        .join("foundation_design_tools")
        .join(format!("{prefix}_{unique_suffix}"));
    fs::create_dir_all(&fixture_dir).expect("fixture dir should exist");
    fixture_dir
}
