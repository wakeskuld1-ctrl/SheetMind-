mod common;

use serde_json::json;

use crate::common::{create_test_workbook, run_cli_with_json};

#[test]
fn tool_catalog_includes_capacity_assessment_from_inventory() {
    let output = run_cli_with_json("");

    // 2026-03-28 16:48 CST: 修改原因和目的：先锁住桥接 Tool 的可发现性，避免只实现内部编排逻辑却忘记把能力暴露到正式目录和分发链路。
    assert!(
        output["data"]["tool_catalog"]
            .as_array()
            .expect("tool catalog should be an array")
            .iter()
            .any(|tool| tool == "capacity_assessment_from_inventory")
    );
}

#[test]
fn capacity_assessment_from_inventory_maps_inventory_result_into_partial_assessment() {
    let workbook_path = create_test_workbook(
        "capacity_from_inventory_partial",
        "capacity_from_inventory_partial.xlsx",
        &[(
            "Capacity",
            vec![vec!["service", "cpu_usage"], vec!["order", "0.83"]],
        )],
    );
    let request = json!({
        "tool": "capacity_assessment_from_inventory",
        "args": {
            "path": workbook_path.to_string_lossy(),
            "sheet": "Capacity",
            "cpu_column": "cpu_usage",
            "inventory_result": {
                "validation_mode": "executed",
                "allowed_commands": ["ps -ef", "nproc", "free -m", "hostname"],
                "host": "10.0.0.8",
                "username": "readonly",
                "command_outputs": {
                    "ps -ef": "root 101 1 0 10:00 ? 00:00:01 java -jar order-service.jar\nroot 102 1 0 10:00 ? 00:00:01 java -jar order-service.jar\n",
                    "nproc": "8\n",
                    "free -m": "               total        used        free      shared  buff/cache   available\nMem:           32768        1024       28000         100        3744       31000\n",
                    "hostname": "node-a\n"
                },
                "inventory": {
                    "hostname": "node-a",
                    "cpu_core_count": 8,
                    "memory_total_mb": 32768,
                    "process_snapshot_excerpt": "java -jar order-service.jar\njava -jar order-service.jar"
                }
            },
            "service_matchers": {
                "command_contains": ["order-service.jar"]
            },
            "scenario_profile": {
                "service_name": "order",
                "service_tier": "important",
                "peak_pattern": "burst"
            },
            "target_cpu_utilization": 0.70,
            "peak_multiplier": 1.10,
            "growth_rate": 0.15
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-28 16:48 CST: 修改原因和目的：锁定“SSH 盘点结果可以自动转成 inventory_evidence，再与部分 Excel 指标一起做 partial 评估”的主链能力。
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["capacity_result"]["evidence_level"],
        "partial"
    );
    assert_eq!(
        output["data"]["capacity_result"]["current_instance_count"],
        2
    );
    assert_eq!(
        output["data"]["capacity_result"]["inventory_evidence"]["source"],
        "ssh_inventory"
    );
    assert_eq!(output["data"]["inventory_mapping"]["host_cpu_cores"], 8);
    assert_eq!(output["data"]["inventory_mapping"]["host_memory_mb"], 32768);
}

#[test]
fn capacity_assessment_from_inventory_does_not_guess_instances_without_matchers() {
    let workbook_path = create_test_workbook(
        "capacity_from_inventory_no_matcher",
        "capacity_from_inventory_no_matcher.xlsx",
        &[(
            "Capacity",
            vec![vec!["service", "cpu_usage"], vec!["order", "0.83"]],
        )],
    );
    let request = json!({
        "tool": "capacity_assessment_from_inventory",
        "args": {
            "path": workbook_path.to_string_lossy(),
            "sheet": "Capacity",
            "cpu_column": "cpu_usage",
            "inventory_result": {
                "validation_mode": "executed",
                "allowed_commands": ["ps -ef", "nproc", "free -m", "hostname"],
                "host": "10.0.0.8",
                "username": "readonly",
                "command_outputs": {
                    "ps -ef": "root 101 1 0 10:00 ? 00:00:01 java -jar order-service.jar\nroot 102 1 0 10:00 ? 00:00:01 java -jar order-service.jar\n"
                },
                "inventory": {
                    "hostname": "node-a",
                    "cpu_core_count": 8,
                    "memory_total_mb": 32768,
                    "process_snapshot_excerpt": "java -jar order-service.jar\njava -jar order-service.jar"
                }
            },
            "scenario_profile": {
                "service_name": "order",
                "service_tier": "important"
            },
            "target_cpu_utilization": 0.70
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-28 16:48 CST: 修改原因和目的：锁定“没有显式 matcher 时不乱猜实例数”的保守策略，避免桥接 Tool 退化成不透明的业务猜测器。
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["capacity_result"]["evidence_level"],
        "guidance_only"
    );
    assert!(output["data"]["capacity_result"]["current_instance_count"].is_null());
    assert!(output["data"]["inventory_mapping"]["discovered_instance_count"].is_null());
}

#[test]
fn capacity_assessment_from_inventory_returns_stable_error_when_ssh_fails() {
    let request = json!({
        "tool": "capacity_assessment_from_inventory",
        "args": {
            "inventory_request": {
                "host": "127.0.0.1",
                "port": 1,
                "username": "readonly",
                "commands": ["ps -ef"]
            }
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-28 16:48 CST: 修改原因和目的：锁定桥接 Tool 在底层 SSH 失败时的稳定错误出口，避免采集异常污染容量分析结构或返回伪造结论。
    assert_eq!(output["status"], "error");
    assert!(
        output["error"]
            .as_str()
            .expect("error message should exist")
            .contains("ssh")
    );
}
