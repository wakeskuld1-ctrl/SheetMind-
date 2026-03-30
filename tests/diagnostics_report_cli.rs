mod common;

use assert_cmd::Command;
use excel_skill::frame::result_ref_store::{PersistedResultDataset, ResultRefStore};
use polars::prelude::{DataFrame, NamedFrom, Series};
use serde_json::json;

use crate::common::{run_cli_with_json, thread_result_ref_store};

fn create_diagnostics_result_ref_for_cli() -> String {
    let store = thread_result_ref_store();
    let result_ref = ResultRefStore::create_result_ref();
    // 2026-03-28 23:40 CST: 这里构造组合诊断专用样本，原因是 diagnostics_report 第一版要同时覆盖相关性、异常值、分布和趋势四类输入；
    // 目的是让高层 Tool 的对外合同先被同一份稳定 result_ref 锁住，避免测试样本分散导致诊断摘要口径飘动。
    let dataframe = DataFrame::new(vec![
        Series::new(
            "month".into(),
            ["2026-01", "2026-02", "2026-03", "2026-04", "2026-05"],
        )
        .into(),
        Series::new(
            "target".into(),
            [10.0_f64, 20.0_f64, 30.0_f64, 40.0_f64, 50.0_f64],
        )
        .into(),
        Series::new(
            "positive_signal".into(),
            [12.0_f64, 22.0_f64, 32.0_f64, 42.0_f64, 52.0_f64],
        )
        .into(),
        Series::new(
            "negative_signal".into(),
            [50.0_f64, 40.0_f64, 30.0_f64, 20.0_f64, 10.0_f64],
        )
        .into(),
        Series::new(
            "amount".into(),
            [100.0_f64, 105.0_f64, 110.0_f64, 115.0_f64, 300.0_f64],
        )
        .into(),
        Series::new(
            "revenue".into(),
            [100.0_f64, 120.0_f64, 140.0_f64, 160.0_f64, 180.0_f64],
        )
        .into(),
    ])
    .unwrap();
    let record = PersistedResultDataset::from_dataframe(
        &result_ref,
        "seed_diagnostics_report",
        vec!["seed_diagnostics_report".to_string()],
        &dataframe,
    )
    .unwrap();
    store.save(&record).unwrap();
    result_ref
}

#[test]
fn tool_catalog_includes_diagnostics_report() {
    let mut cmd = Command::cargo_bin("excel_skill").unwrap();
    let assert = cmd.assert().success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    // 2026-03-28 23:40 CST: 这里先锁目录暴露，原因是高层组合 Tool 如果不进 catalog，后续 CLI 和 Skill 都无法发现；
    // 目的是先把“能被系统发现”这件事独立钉死，避免实现完算子却忘了对外注册。
    assert!(
        json["data"]["tool_catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool == "diagnostics_report")
    );
}

#[test]
fn diagnostics_report_returns_unified_sections_and_actions() {
    let result_ref = create_diagnostics_result_ref_for_cli();
    let request = json!({
        "tool": "diagnostics_report",
        "args": {
            "result_ref": result_ref,
            "correlation": {
                "target_column": "target",
                "feature_columns": ["positive_signal", "negative_signal"]
            },
            "outlier": {
                "columns": ["amount"],
                "method": "iqr"
            },
            "distribution": {
                "column": "amount",
                "bins": 4
            },
            "trend": {
                "time_column": "month",
                "value_column": "revenue"
            }
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-28 23:40 CST: 这里先锁完整组合合同，原因是第一版高层 Tool 的核心价值就是把四类诊断收口成一个统一结果；
    // 目的是确保后续实现不能只返回零散 section，而必须同时给出总体状态、关键发现和下一步动作。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["report_status"], "ok");
    assert_eq!(output["data"]["section_count"], 4);
    assert_eq!(output["data"]["available_section_count"], 4);
    assert_eq!(output["data"]["warnings"].as_array().unwrap().len(), 0);
    assert!(
        output["data"]["sections"]
            .as_array()
            .unwrap()
            .iter()
            .any(|section| section["key"] == "correlation" && section["status"] == "ok")
    );
    assert!(
        output["data"]["sections"]
            .as_array()
            .unwrap()
            .iter()
            .any(|section| section["key"] == "outlier" && section["status"] == "ok")
    );
    assert!(
        output["data"]["sections"]
            .as_array()
            .unwrap()
            .iter()
            .any(|section| section["key"] == "distribution" && section["status"] == "ok")
    );
    assert!(
        output["data"]["sections"]
            .as_array()
            .unwrap()
            .iter()
            .any(|section| section["key"] == "trend" && section["status"] == "ok")
    );
    assert!(output["data"]["correlation_section"].is_object());
    assert!(output["data"]["outlier_section"].is_object());
    assert!(output["data"]["distribution_section"].is_object());
    assert!(output["data"]["trend_section"].is_object());
    assert!(
        !output["data"]["key_findings"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(
        !output["data"]["recommended_actions"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn diagnostics_report_degrades_when_one_section_is_unavailable() {
    let result_ref = create_diagnostics_result_ref_for_cli();
    let request = json!({
        "tool": "diagnostics_report",
        "args": {
            "result_ref": result_ref,
            "correlation": {
                "target_column": "target",
                "feature_columns": ["positive_signal"]
            },
            "distribution": {
                "column": "amount",
                "bins": 4
            },
            "trend": {
                "time_column": "month",
                "value_column": "missing_revenue"
            }
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-28 23:40 CST: 这里先锁“单 section 失败不打死整包”的降级规则，原因是用户明确要求组合 Tool 要有高层交付韧性；
    // 目的是保证以后列名不完整或局部字段缺失时，仍然能把其他可用诊断结论交付出来。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["report_status"], "degraded");
    assert_eq!(output["data"]["section_count"], 3);
    assert_eq!(output["data"]["available_section_count"], 2);
    assert!(
        output["data"]["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|warning| warning.as_str().unwrap().contains("trend_analysis"))
    );
    assert!(output["data"]["correlation_section"].is_object());
    assert!(output["data"]["distribution_section"].is_object());
    assert!(output["data"]["trend_section"].is_null());
}
