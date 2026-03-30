mod common;

use assert_cmd::Command;
use excel_skill::frame::result_ref_store::{PersistedResultDataset, ResultRefStore};
use polars::prelude::{DataFrame, NamedFrom, Series};
use serde_json::json;

use crate::common::{run_cli_with_json, thread_result_ref_store};

fn create_correlation_result_ref_for_cli() -> String {
    let store = thread_result_ref_store();
    let result_ref = ResultRefStore::create_result_ref();
    // 2026-03-25: 这里构造稳定的数值相关性样本，原因是 correlation_analysis 第一版要先锁住正相关和负相关排序；目的是让统计诊断型 Tool 的最小输出协议稳定下来。
    let dataframe = DataFrame::new(vec![
        Series::new("target".into(), [1.0_f64, 2.0_f64, 3.0_f64, 4.0_f64]).into(),
        Series::new(
            "positive_signal".into(),
            [2.0_f64, 4.0_f64, 6.0_f64, 8.0_f64],
        )
        .into(),
        Series::new(
            "negative_signal".into(),
            [8.0_f64, 6.0_f64, 4.0_f64, 2.0_f64],
        )
        .into(),
    ])
    .unwrap();
    let record = PersistedResultDataset::from_dataframe(
        &result_ref,
        "seed_correlation_analysis",
        vec!["seed_correlation_analysis".to_string()],
        &dataframe,
    )
    .unwrap();
    store.save(&record).unwrap();
    result_ref
}

fn create_outlier_result_ref_for_cli() -> String {
    let store = thread_result_ref_store();
    let result_ref = ResultRefStore::create_result_ref();
    // 2026-03-25: 这里构造稳定的异常值样本，原因是 outlier_detection 第一版要先锁住 IQR 明显极端值识别；目的是让统计诊断链路稳定发现可疑记录。
    let dataframe = DataFrame::new(vec![
        Series::new("customer".into(), ["A", "B", "C", "D", "E"]).into(),
        Series::new(
            "amount".into(),
            [10.0_f64, 12.0_f64, 11.0_f64, 13.0_f64, 100.0_f64],
        )
        .into(),
    ])
    .unwrap();
    let record = PersistedResultDataset::from_dataframe(
        &result_ref,
        "seed_outlier_detection",
        vec!["seed_outlier_detection".to_string()],
        &dataframe,
    )
    .unwrap();
    store.save(&record).unwrap();
    result_ref
}

fn create_distribution_result_ref_for_cli() -> String {
    let store = thread_result_ref_store();
    let result_ref = ResultRefStore::create_result_ref();
    // 2026-03-25: 这里构造偏态分布样本，原因是 distribution_analysis 第一版要先锁住 min/max/median 与分箱输出；目的是让分布观察层形成稳定协议。
    let dataframe = DataFrame::new(vec![
        Series::new(
            "amount".into(),
            [1.0_f64, 2.0_f64, 2.0_f64, 3.0_f64, 100.0_f64],
        )
        .into(),
    ])
    .unwrap();
    let record = PersistedResultDataset::from_dataframe(
        &result_ref,
        "seed_distribution_analysis",
        vec!["seed_distribution_analysis".to_string()],
        &dataframe,
    )
    .unwrap();
    store.save(&record).unwrap();
    result_ref
}

fn create_trend_result_ref_for_cli() -> String {
    let store = thread_result_ref_store();
    let result_ref = ResultRefStore::create_result_ref();
    // 2026-03-25: 这里构造稳定的趋势样本，原因是 trend_analysis 第一版要先锁住时间排序、起止值和变化率；目的是让统计诊断层先形成最小趋势观察协议。
    let dataframe = DataFrame::new(vec![
        Series::new("month".into(), ["2026-01", "2026-03", "2026-02", "2026-04"]).into(),
        Series::new(
            "revenue".into(),
            [100.0_f64, 140.0_f64, 120.0_f64, 180.0_f64],
        )
        .into(),
    ])
    .unwrap();
    let record = PersistedResultDataset::from_dataframe(
        &result_ref,
        "seed_trend_analysis",
        vec!["seed_trend_analysis".to_string()],
        &dataframe,
    )
    .unwrap();
    store.save(&record).unwrap();
    result_ref
}

#[test]
fn correlation_analysis_returns_ranked_correlations() {
    let result_ref = create_correlation_result_ref_for_cli();
    let request = json!({
        "tool": "correlation_analysis",
        "args": {
            "result_ref": result_ref,
            "target_column": "target",
            "feature_columns": ["positive_signal", "negative_signal"]
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-25: 这里锁定 correlation_analysis 可以直接消费 result_ref，原因是统计诊断层要能无缝接上一步中间结果；目的是把先处理再观察的桥接链路固定下来。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["target_column"], "target");
    assert_eq!(output["data"]["row_count"], 4);
    assert_eq!(output["data"]["method"], "pearson");
    assert_eq!(
        output["data"]["correlations"][0]["feature_column"],
        "positive_signal"
    );
    assert_eq!(
        output["data"]["correlations"][1]["feature_column"],
        "negative_signal"
    );
}

#[test]
fn outlier_detection_returns_flagged_result_ref_and_summary() {
    let result_ref = create_outlier_result_ref_for_cli();
    let request = json!({
        "tool": "outlier_detection",
        "args": {
            "result_ref": result_ref,
            "columns": ["amount"],
            "method": "iqr"
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-25: 这里锁定 outlier_detection 会返回可继续复用的 result_ref 和结构化摘要，原因是异常值诊断不应停留在口头说明；目的是让用户后续还能基于标记结果继续筛选、导出或汇报。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["method"], "iqr");
    assert_eq!(output["data"]["row_count"], 5);
    assert_eq!(output["data"]["outlier_summaries"][0]["column"], "amount");
    assert_eq!(output["data"]["outlier_summaries"][0]["outlier_count"], 1);
    assert!(output["data"]["result_ref"].as_str().is_some());
    assert_eq!(output["data"]["rows"][4]["amount__is_outlier"], "true");
}

#[test]
fn distribution_analysis_returns_histogram_and_summary() {
    let result_ref = create_distribution_result_ref_for_cli();
    let request = json!({
        "tool": "distribution_analysis",
        "args": {
            "result_ref": result_ref,
            "column": "amount",
            "bins": 4
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-25: 这里锁定 distribution_analysis 会返回单列分布摘要和分箱结果，原因是建模前观察需要先看分布是否偏态；目的是让后续 Skill 可以直接把分布观察翻译给用户。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["column"], "amount");
    assert_eq!(output["data"]["row_count"], 5);
    assert_eq!(output["data"]["non_null_count"], 5);
    assert_eq!(output["data"]["bin_count"], 4);
    assert_eq!(output["data"]["distribution_summary"]["min"], 1.0);
    assert_eq!(output["data"]["distribution_summary"]["max"], 100.0);
    assert_eq!(output["data"]["distribution_summary"]["median"], 2.0);
    assert_eq!(output["data"]["bins"].as_array().unwrap().len(), 4);
}

#[test]
fn trend_analysis_returns_direction_and_ordered_points() {
    let result_ref = create_trend_result_ref_for_cli();
    let request = json!({
        "tool": "trend_analysis",
        "args": {
            "result_ref": result_ref,
            "time_column": "month",
            "value_column": "revenue"
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-25: 这里先锁定 trend_analysis 会返回排序后的趋势点、起止值和方向判断，原因是趋势观察必须先给上层一个稳定协议；目的是让 Skill 能直接解释“整体是在上升还是下降”。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["time_column"], "month");
    assert_eq!(output["data"]["value_column"], "revenue");
    assert_eq!(output["data"]["row_count"], 4);
    assert_eq!(output["data"]["direction"], "upward");
    assert_eq!(output["data"]["start_value"], 100.0);
    assert_eq!(output["data"]["end_value"], 180.0);
    assert_eq!(output["data"]["absolute_change"], 80.0);
    assert_eq!(output["data"]["change_rate"], 0.8);
    assert_eq!(output["data"]["points"][0]["time"], "2026-01");
    assert_eq!(output["data"]["points"][1]["time"], "2026-02");
    assert_eq!(output["data"]["points"][2]["time"], "2026-03");
    assert_eq!(output["data"]["points"][3]["time"], "2026-04");
    assert!(
        output["data"]["human_summary"]["overall"]
            .as_str()
            .unwrap()
            .contains("趋势")
    );
}

#[test]
fn tool_catalog_includes_stat_diagnostic_tools() {
    let mut cmd = Command::cargo_bin("excel_skill").unwrap();
    let assert = cmd.assert().success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    // 2026-03-25: 这里锁定能力目录包含统计诊断层四件套，原因是新的干净测试入口需要一起守住能力可发现性；目的是让上层 Skill 可以稳定推荐完整观察路径。
    assert!(
        json["data"]["tool_catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool == "correlation_analysis")
    );
    assert!(
        json["data"]["tool_catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool == "outlier_detection")
    );
    assert!(
        json["data"]["tool_catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool == "distribution_analysis")
    );
    assert!(
        json["data"]["tool_catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool == "trend_analysis")
    );
}
