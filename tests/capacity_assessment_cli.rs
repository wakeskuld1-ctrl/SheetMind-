mod common;

use serde_json::json;

use crate::common::{create_test_workbook, run_cli_with_json};

#[test]
fn tool_catalog_includes_capacity_assessment() {
    let output = run_cli_with_json("");

    // 2026-03-28 08:25 CST: 修改原因和目的：先把工具目录里的可发现性锁住，避免后续只实现底层逻辑却忘了对外注册新场景 tool。
    assert!(
        output["data"]["tool_catalog"]
            .as_array()
            .expect("tool catalog should be an array")
            .iter()
            .any(|tool| tool == "capacity_assessment")
    );
}

#[test]
fn capacity_assessment_returns_quantified_snapshot_conclusion() {
    let workbook_path = create_test_workbook(
        "capacity_assessment_snapshot",
        "snapshot.xlsx",
        &[(
            "Capacity",
            vec![
                vec!["service", "instances", "cpu_usage", "memory_usage"],
                vec!["api-gateway", "4", "0.82", "0.68"],
            ],
        )],
    );
    let request = json!({
        "tool": "capacity_assessment",
        "args": {
            "path": workbook_path.to_string_lossy(),
            "sheet": "Capacity",
            "instance_count_column": "instances",
            "cpu_column": "cpu_usage",
            "memory_column": "memory_usage",
            "target_cpu_utilization": 0.70,
            "target_memory_utilization": 0.75,
            "peak_multiplier": 1.20,
            "growth_rate": 0.30,
            "require_n_plus_one": true
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-28 08:25 CST: 修改原因和目的：锁定“只有快照数据也能量化”的能力，避免场景被错误设计成必须依赖完整历史序列。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["evidence_level"], "quantified");
    assert_eq!(output["data"]["capacity_status"], "insufficient");
    assert!(
        output["data"]["recommended_instance_count"]
            .as_u64()
            .expect("recommended instance count should exist")
            >= 6
    );
}

#[test]
fn capacity_assessment_returns_guidance_when_data_is_incomplete() {
    let workbook_path = create_test_workbook(
        "capacity_assessment_guidance",
        "guidance.xlsx",
        &[(
            "Capacity",
            vec![
                vec!["service", "known_issue", "current_peak_note"],
                vec!["billing", "peak lag", "double eleven burst observed"],
            ],
        )],
    );
    let request = json!({
        "tool": "capacity_assessment",
        "args": {
            "path": workbook_path.to_string_lossy(),
            "sheet": "Capacity"
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-28 08:25 CST: 修改原因和目的：锁定“数据不全时仍给决策思路”的降级路径，避免工具直接报错把用户挡在门外。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["evidence_level"], "guidance_only");
    assert!(
        output["data"]["missing_inputs"]
            .as_array()
            .expect("missing inputs should be an array")
            .len()
            >= 3
    );
    assert!(
        output["data"]["decision_guidance"]["recommended_next_step"]
            .as_str()
            .expect("recommended next step should exist")
            .contains("补")
    );
}

#[test]
fn capacity_assessment_uses_history_when_time_series_is_available() {
    let workbook_path = create_test_workbook(
        "capacity_assessment_history",
        "history.xlsx",
        &[(
            "Capacity",
            vec![
                vec!["ts", "instances", "cpu_usage", "workload_qps"],
                vec!["2026-03-25 09:00", "4", "0.52", "1200"],
                vec!["2026-03-25 10:00", "4", "0.61", "1500"],
                vec!["2026-03-25 11:00", "4", "0.79", "2100"],
                vec!["2026-03-25 12:00", "4", "0.87", "2600"],
            ],
        )],
    );
    let request = json!({
        "tool": "capacity_assessment",
        "args": {
            "path": workbook_path.to_string_lossy(),
            "sheet": "Capacity",
            "time_column": "ts",
            "instance_count_column": "instances",
            "cpu_column": "cpu_usage",
            "workload_column": "workload_qps",
            "target_cpu_utilization": 0.70,
            "peak_multiplier": 1.00,
            "growth_rate": 0.10
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-28 08:25 CST: 修改原因和目的：锁定“有历史就向上使用趋势证据”的路径，确保容量分析不会永远退化成静态快照规则。
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["trend_observations"]["cpu"]["direction"],
        "upward"
    );
    assert!(
        output["data"]["history_signals"]["has_time_series"]
            .as_bool()
            .expect("history signal flag should exist")
    );
}

#[test]
fn capacity_assessment_uses_scenario_and_deployment_profiles_when_sheet_is_partial() {
    let workbook_path = create_test_workbook(
        "capacity_assessment_partial_profile",
        "partial_profile.xlsx",
        &[(
            "Capacity",
            vec![
                vec!["service", "cpu_usage", "latency_p95_ms"],
                vec!["checkout", "0.84", "220"],
            ],
        )],
    );
    let request = json!({
        "tool": "capacity_assessment",
        "args": {
            "path": workbook_path.to_string_lossy(),
            "sheet": "Capacity",
            "cpu_column": "cpu_usage",
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
            "target_cpu_utilization": 0.68,
            "peak_multiplier": 1.15,
            "growth_rate": 0.20
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-28 14:35 CST: 修改原因和目的：锁定“指标列不全，但场景/部署信息足够时仍能输出部分量化结论”的路径，避免新版本继续把部署上下文当成可有可无的装饰字段。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["evidence_level"], "partial");
    assert!(
        output["data"]["recommended_instance_count"]
            .as_u64()
            .expect("recommended instance count should exist in partial mode")
            >= 8
    );
    assert_eq!(
        output["data"]["service_risk_profile"]["service_tier"],
        "core"
    );
    assert_eq!(
        output["data"]["service_risk_profile"]["redundancy_mode"],
        "n_plus_one"
    );
}

#[test]
fn capacity_assessment_accepts_inventory_evidence_to_fill_instance_facts() {
    let workbook_path = create_test_workbook(
        "capacity_assessment_inventory_evidence",
        "inventory_evidence.xlsx",
        &[(
            "Capacity",
            vec![
                vec!["service", "cpu_usage", "memory_usage"],
                vec!["order", "0.78", "0.66"],
            ],
        )],
    );
    let request = json!({
        "tool": "capacity_assessment",
        "args": {
            "path": workbook_path.to_string_lossy(),
            "sheet": "Capacity",
            "cpu_column": "cpu_usage",
            "memory_column": "memory_usage",
            "inventory_evidence": {
                "source": "ssh_inventory",
                "discovered_instance_count": 6,
                "host_count": 3,
                "host_cpu_cores": 8,
                "host_memory_mb": 32768
            },
            "scenario_profile": {
                "service_name": "order",
                "service_tier": "important",
                "peak_pattern": "steady"
            },
            "target_cpu_utilization": 0.70,
            "target_memory_utilization": 0.75,
            "peak_multiplier": 1.05,
            "growth_rate": 0.10
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-28 14:35 CST: 修改原因和目的：锁定“实例事实可以来自 SSH 清单证据，而不是只能死等 Excel 列”的弹性路径，避免工具遇到缺实例列时再次退化成 guidance_only。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["evidence_level"], "partial");
    assert_eq!(output["data"]["current_instance_count"], 6);
    assert_eq!(
        output["data"]["inventory_evidence"]["source"],
        "ssh_inventory"
    );
}
