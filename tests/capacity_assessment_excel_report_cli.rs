mod common;

use std::fs;
use std::path::Path;

use calamine::{Reader, open_workbook_auto};
use serde_json::json;

use crate::common::{create_test_output_path, create_test_workbook, run_cli_with_json};

fn read_zip_entry_text(path: &Path, entry_name: &str) -> String {
    let file = fs::File::open(path).expect("xlsx file should exist");
    let mut archive = zip::ZipArchive::new(file).expect("xlsx should be a zip archive");
    let mut entry = archive
        .by_name(entry_name)
        .expect("zip entry should exist in xlsx");
    let mut content = String::new();
    std::io::Read::read_to_string(&mut entry, &mut content)
        .expect("zip entry should be readable as text");
    content
}

#[test]
fn tool_catalog_includes_capacity_assessment_excel_report() {
    let output = run_cli_with_json("");

    // 2026-03-28 22:05 CST: 修改原因和目的：先锁定新 Excel 交付 Tool 的可发现性，避免这轮再次只做底层实现却忘记暴露正式入口。
    assert!(
        output["data"]["tool_catalog"]
            .as_array()
            .expect("tool catalog should be an array")
            .iter()
            .any(|tool| tool == "capacity_assessment_excel_report")
    );
}

#[test]
fn capacity_assessment_excel_report_exports_quantified_workbook_from_excel_metrics() {
    let workbook_path = create_test_workbook(
        "capacity_excel_report_quantified",
        "capacity_excel_report_quantified.xlsx",
        &[(
            "Capacity",
            vec![
                vec![
                    "ts",
                    "service",
                    "instances",
                    "cpu_usage",
                    "memory_usage",
                    "workload_qps",
                ],
                vec!["2026-03-28 09:00", "checkout", "4", "0.62", "0.58", "1800"],
                vec!["2026-03-28 10:00", "checkout", "4", "0.74", "0.63", "2200"],
                vec!["2026-03-28 11:00", "checkout", "4", "0.86", "0.72", "2800"],
            ],
        )],
    );
    let output_path = create_test_output_path("capacity_excel_report_quantified", "xlsx");
    let request = json!({
        "tool": "capacity_assessment_excel_report",
        "args": {
            "report_name": "结算链路容量评估",
            "report_subtitle": "高峰前压测快照",
            "path": workbook_path.to_string_lossy(),
            "sheet": "Capacity",
            "time_column": "ts",
            "instance_count_column": "instances",
            "cpu_column": "cpu_usage",
            "memory_column": "memory_usage",
            "workload_column": "workload_qps",
            "scenario_profile": {
                "service_name": "checkout",
                "service_tier": "core",
                "sla_level": "99.95",
                "peak_pattern": "burst",
                "failure_impact": "high"
            },
            "deployment_profile": {
                "current_instance_count": 4,
                "redundancy_mode": "n_plus_one",
                "scaling_step": 2,
                "deployment_kind": "kubernetes"
            },
            "target_cpu_utilization": 0.70,
            "target_memory_utilization": 0.75,
            "peak_multiplier": 1.10,
            "growth_rate": 0.20,
            "require_n_plus_one": true,
            "output_path": output_path.to_string_lossy()
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-28 22:05 CST: 修改原因和目的：锁定“有 Excel 指标时可以一步分析并直接导出 xlsx”的主交付路径，防止功能再次退化成只返 JSON。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["format"], "xlsx");
    assert_eq!(
        output["data"]["capacity_result"]["evidence_level"],
        "quantified"
    );
    assert!(
        output["data"]["workbook_ref"]
            .as_str()
            .expect("workbook_ref should exist")
            .starts_with("workbook_")
    );
    assert!(output_path.exists(), "xlsx output should exist");

    let workbook_xml = read_zip_entry_text(&output_path, "xl/workbook.xml");
    assert!(workbook_xml.contains("结论页"));
    assert!(workbook_xml.contains("资源测算页"));
    assert!(workbook_xml.contains("证据与风险页"));
    assert!(workbook_xml.contains("补数与行动页"));

    let mut workbook = open_workbook_auto(&output_path).expect("xlsx should be readable");
    let conclusion_sheet = workbook
        .worksheet_range("结论页")
        .expect("conclusion sheet should exist");
    let evidence_sheet = workbook
        .worksheet_range("证据与风险页")
        .expect("evidence sheet should exist");
    assert!(
        conclusion_sheet
            .rows()
            .flat_map(|row| row.iter())
            .any(|cell| cell.to_string().contains("checkout")),
        "conclusion sheet should contain service name"
    );
    assert!(
        evidence_sheet
            .rows()
            .flat_map(|row| row.iter())
            .any(|cell| cell.to_string().contains("core")),
        "evidence sheet should contain service tier"
    );
}

#[test]
fn capacity_assessment_excel_report_exports_partial_workbook_from_inventory_mapping() {
    let workbook_path = create_test_workbook(
        "capacity_excel_report_partial",
        "capacity_excel_report_partial.xlsx",
        &[(
            "Capacity",
            vec![vec!["service", "cpu_usage"], vec!["order", "0.83"]],
        )],
    );
    let output_path = create_test_output_path("capacity_excel_report_partial", "xlsx");
    let request = json!({
        "tool": "capacity_assessment_excel_report",
        "args": {
            "report_name": "订单服务容量评估",
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
            "growth_rate": 0.15,
            "output_path": output_path.to_string_lossy()
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-28 22:05 CST: 修改原因和目的：锁定“Excel 指标 + SSH 映射证据”可以直接产出 partial 报表，确保桥接能力真正进入 Excel 交付层。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["format"], "xlsx");
    assert_eq!(
        output["data"]["capacity_result"]["evidence_level"],
        "partial"
    );
    assert_eq!(
        output["data"]["capacity_result"]["current_instance_count"],
        2
    );
    assert!(output_path.exists(), "xlsx output should exist");

    let shared_strings = read_zip_entry_text(&output_path, "xl/sharedStrings.xml");
    assert!(shared_strings.contains("订单服务容量评估"));
    assert!(shared_strings.contains("ssh_inventory"));
}

#[test]
fn capacity_assessment_excel_report_still_exports_guidance_workbook_without_excel_source() {
    let output_path = create_test_output_path("capacity_excel_report_guidance_only", "xlsx");
    let request = json!({
        "tool": "capacity_assessment_excel_report",
        "args": {
            "report_name": "活动前容量预审",
            "inventory_result": {
                "validation_mode": "executed",
                "allowed_commands": ["hostname"],
                "host": "10.0.0.9",
                "username": "readonly",
                "command_outputs": {
                    "hostname": "node-b\n"
                },
                "inventory": {
                    "hostname": "node-b",
                    "cpu_core_count": null,
                    "memory_total_mb": null,
                    "process_snapshot_excerpt": null
                }
            },
            "scenario_profile": {
                "service_name": "promotion",
                "service_tier": "important",
                "peak_pattern": "burst",
                "failure_impact": "high"
            },
            "output_path": output_path.to_string_lossy()
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-28 22:05 CST: 修改原因和目的：锁定“没有 Excel 也能给出指导型 Excel 报表”的退化路径，满足用户要求的弹性和决策思路交付。
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["capacity_result"]["evidence_level"],
        "guidance_only"
    );
    assert_eq!(output["data"]["format"], "xlsx");
    assert!(output_path.exists(), "xlsx output should exist");

    let workbook_xml = read_zip_entry_text(&output_path, "xl/workbook.xml");
    assert!(workbook_xml.contains("结论页"));
    assert!(workbook_xml.contains("补数与行动页"));
}
