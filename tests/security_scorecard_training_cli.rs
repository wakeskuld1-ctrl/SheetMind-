mod common;

use chrono::{Duration, NaiveDate};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::common::{
    create_test_runtime_db, run_cli_with_json, run_cli_with_json_runtime_and_envs,
};

// 2026-04-09 CST: 这里新增 scorecard training CLI 测试夹具，原因是 Task 5 需要先把正式训练入口的契约锁进红测；
// 目的：先验证“训练产物 + refit_run + model_registry”一体化输出，再做最小实现，避免后续把训练入口做成临时脚本。
fn create_training_fixture_dir(prefix: &str) -> PathBuf {
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let fixture_dir = PathBuf::from("tests")
        .join("runtime_fixtures")
        .join("security_scorecard_training")
        .join(format!("{prefix}_{unique_suffix}"));
    fs::create_dir_all(&fixture_dir).expect("security scorecard training fixture dir should exist");
    fixture_dir
}

// 2026-04-09 CST: 这里复用本地 HTTP 假服务，原因是训练入口会沿用 feature_snapshot/forward_outcome，而上游仍依赖财报和公告上下文；
// 目的：让训练测试只关注训练主链本身，不被外部网络或线上接口波动干扰。
fn spawn_http_route_server(routes: Vec<(&str, &str, &str, &str)>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("test http server should bind");
    let address = format!(
        "http://{}",
        listener
            .local_addr()
            .expect("test http server should have local addr")
    );
    let route_map: HashMap<String, (String, String, String)> = routes
        .into_iter()
        .map(|(path, status_line, body, content_type)| {
            (
                path.to_string(),
                (
                    status_line.to_string(),
                    body.to_string(),
                    content_type.to_string(),
                ),
            )
        })
        .collect();

    thread::spawn(move || {
        // 2026-04-09 CST: 这里放宽测试 HTTP 服务的接入次数，原因是训练入口会对多个样本重复拉取财报和公告上下文；
        // 目的：确保测试夹具覆盖多样本训练场景时不会因为本地假服务提早关闭而误报失败。
        for _ in 0..256 {
            let Ok((mut stream, _)) = listener.accept() else {
                break;
            };
            let mut buffer = [0_u8; 4096];
            let _ = stream.read(&mut buffer);
            let request_text = String::from_utf8_lossy(&buffer);
            let request_line = request_text.lines().next().unwrap_or_default();
            let request_path = request_line
                .split_whitespace()
                .nth(1)
                .unwrap_or("/")
                .split('?')
                .next()
                .unwrap_or("/");
            let (status_line, body, content_type) =
                route_map.get(request_path).cloned().unwrap_or_else(|| {
                    (
                        "HTTP/1.1 404 Not Found".to_string(),
                        "{\"error\":\"not found\"}".to_string(),
                        "application/json".to_string(),
                    )
                });
            let response = format!(
                "{status_line}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        }
    });

    address
}

#[test]
fn tool_catalog_includes_security_scorecard_training() {
    let output = run_cli_with_json("");

    // 2026-04-09 CST: 这里先锁 training Tool 的可发现性，原因是如果 catalog 不正式暴露它，后续 Skill 与训练编排就没有一等入口；
    // 目的：确保证券评分卡训练入口能和 snapshot/forward_outcome/refit 一样被统一发现与路由。
    assert!(
        output["data"]["tool_catalog"]
            .as_array()
            .expect("tool catalog should be an array")
            .iter()
            .any(|tool| tool == "security_scorecard_training")
    );
}

#[test]
fn security_scorecard_training_generates_artifact_and_registers_refit_outputs() {
    let runtime_db_path = create_test_runtime_db("security_scorecard_training_ready");
    let runtime_root = runtime_db_path
        .parent()
        .expect("runtime db should have parent")
        .join("scorecard_training_runtime");
    let fixture_dir = create_training_fixture_dir("security_scorecard_training_ready");

    let stock_up_csv = fixture_dir.join("stock_up.csv");
    let stock_down_csv = fixture_dir.join("stock_down.csv");
    let market_csv = fixture_dir.join("market.csv");
    let sector_csv = fixture_dir.join("sector.csv");

    fs::write(
        &stock_up_csv,
        build_trend_rows(420, 100.0, 0.9, 1.0).join("\n"),
    )
    .expect("upward symbol csv should be written");
    fs::write(
        &stock_down_csv,
        build_trend_rows(420, 120.0, -0.7, 1.0).join("\n"),
    )
    .expect("downward symbol csv should be written");
    fs::write(
        &market_csv,
        build_trend_rows(420, 3200.0, 2.5, 5.0).join("\n"),
    )
    .expect("market csv should be written");
    fs::write(
        &sector_csv,
        build_trend_rows(420, 980.0, 1.4, 2.0).join("\n"),
    )
    .expect("sector csv should be written");

    import_history_csv(&runtime_db_path, &stock_up_csv, "601916.SH");
    import_history_csv(&runtime_db_path, &stock_down_csv, "600000.SH");
    import_history_csv(&runtime_db_path, &market_csv, "510300.SH");
    import_history_csv(&runtime_db_path, &sector_csv, "512800.SH");

    let server = spawn_http_route_server(vec![
        (
            "/financials",
            "HTTP/1.1 200 OK",
            r#"[
                {
                    "REPORT_DATE":"2025-12-31",
                    "NOTICE_DATE":"2026-03-28",
                    "TOTAL_OPERATE_INCOME":308227000000.0,
                    "YSTZ":8.37,
                    "PARENT_NETPROFIT":11117000000.0,
                    "SJLTZ":9.31,
                    "ROEJQ":14.8
                }
            ]"#,
            "application/json",
        ),
        (
            "/announcements",
            "HTTP/1.1 200 OK",
            r#"{
                "data":{
                    "list":[
                        {"notice_date":"2026-03-28","title":"2025年度报告","art_code":"AN202603281234567890","columns":[{"column_name":"定期报告"}]},
                        {"notice_date":"2026-03-28","title":"2025年度利润分配预案公告","art_code":"AN202603281234567891","columns":[{"column_name":"公司公告"}]}
                    ]
                }
            }"#,
            "application/json",
        ),
    ]);

    let request = json!({
        "tool": "security_scorecard_training",
        "args": {
            "created_at": "2026-04-09T17:30:00+08:00",
            "training_runtime_root": runtime_root.to_string_lossy(),
            "market_scope": "A_SHARE",
            "instrument_scope": "EQUITY",
            "symbol_list": ["601916.SH", "600000.SH"],
            "market_symbol": "510300.SH",
            "sector_symbol": "512800.SH",
            "market_profile": "a_share_core",
            "sector_profile": "a_share_bank",
            "horizon_days": 10,
            "target_head": "direction_head",
            "train_range": "2025-03-01..2025-08-31",
            "valid_range": "2025-09-01..2025-11-30",
            "test_range": "2025-12-01..2026-01-31",
            // 2026-04-11 CST: Expand the per-symbol sample targets in the training
            // contract, because Scheme B requires the training pool to become a
            // materially larger governed dataset instead of staying at the old
            // toy-sized 2/1/1 split.
            "train_samples_per_symbol": 6,
            "valid_samples_per_symbol": 3,
            "test_samples_per_symbol": 3,
            "feature_set_version": "security_feature_snapshot.v1",
            "label_definition_version": "security_forward_outcome.v1"
        }
    });

    let output = run_cli_with_json_runtime_and_envs(
        &request.to_string(),
        &runtime_db_path,
        &[
            (
                "EXCEL_SKILL_EASTMONEY_FINANCIAL_URL_BASE",
                format!("{server}/financials"),
            ),
            (
                "EXCEL_SKILL_EASTMONEY_ANNOUNCEMENT_URL_BASE",
                format!("{server}/announcements"),
            ),
        ],
    );

    // 2026-04-09 CST: 这里锁定 Task 5 方案 B 的最小正式合同，原因是训练入口必须一次返回产物、治理对象和落盘路径；
    // 目的：确保后续回算、重估和 package 挂接都能直接消费统一输出，而不是再去拼接中间状态。
    assert!(
        output["status"] == "ok",
        "unexpected path-head output: {output}"
    );
    assert_eq!(
        output["data"]["refit_run"]["document_type"],
        "security_scorecard_refit_run"
    );
    assert_eq!(
        output["data"]["model_registry"]["document_type"],
        "security_scorecard_model_registry"
    );
    assert_eq!(
        output["data"]["model_registry"]["target_head"],
        "direction_head"
    );
    assert_eq!(output["data"]["model_registry"]["horizon_days"], 10);
    // 2026-04-11 CST: Lock the richer fit panel and larger governed sample pool,
    // because the user explicitly required training-backed conclusions to disclose
    // enough fit evidence instead of relying on a tiny opaque fixture.
    assert_eq!(output["data"]["metrics_summary_json"]["sample_count"], 24);
    assert_eq!(
        output["data"]["metrics_summary_json"]["sample_breakdown"]["train"]["sample_count"],
        12
    );
    assert_eq!(
        output["data"]["metrics_summary_json"]["sample_breakdown"]["valid"]["sample_count"],
        6
    );
    assert_eq!(
        output["data"]["metrics_summary_json"]["sample_breakdown"]["test"]["sample_count"],
        6
    );
    assert_eq!(
        output["data"]["metrics_summary_json"]["sample_breakdown"]["train"]["unique_symbol_count"],
        2
    );
    assert!(
        output["data"]["metrics_summary_json"]["train"]["auc"].is_number(),
        "train auc should be present in fit panel"
    );
    assert!(
        output["data"]["metrics_summary_json"]["train"]["ks"].is_number(),
        "train ks should be present in fit panel"
    );
    assert!(
        output["data"]["metrics_summary_json"]["train"]["confusion_matrix"]["tp"].is_number(),
        "train confusion matrix should include tp"
    );
    assert!(
        output["data"]["metrics_summary_json"]["train"]["horizon_event_summary"]["avg_forward_return"]
            .is_number(),
        "train horizon event summary should include avg forward return"
    );
    assert!(
        output["data"]["metrics_summary_json"]["train"]["horizon_event_summary"]["hit_upside_first_rate"]
            .is_number(),
        "train horizon event summary should include upside-first rate"
    );
    // 2026-04-11 CST: Lock the governed training-readiness panel in a red test,
    // because Scheme B now needs the runtime to explain whether the current
    // sample pool is only research-grade or closer to production-grade.
    // Purpose: force training outputs to expose sample sufficiency, class balance,
    // and path-event coverage before we rely on them for downstream decisions.
    assert_eq!(
        output["data"]["metrics_summary_json"]["readiness_assessment"]["minimum_sample_status"],
        "sample_ready"
    );
    assert_eq!(
        output["data"]["metrics_summary_json"]["readiness_assessment"]["class_balance_status"],
        "class_balance_ready"
    );
    assert_eq!(
        output["data"]["metrics_summary_json"]["readiness_assessment"]["path_event_coverage_status"],
        "path_event_sparse"
    );
    assert_eq!(
        output["data"]["metrics_summary_json"]["readiness_assessment"]["production_readiness"],
        "research_candidate_only"
    );
    assert!(
        output["data"]["metrics_summary_json"]["readiness_assessment"]["notes"]
            .as_array()
            .expect("readiness notes should be an array")
            .iter()
            .any(|item| item
                .as_str()
                .expect("readiness note should be string")
                .contains("path events")),
        "readiness notes should explain path-event sparsity"
    );

    let artifact_path = PathBuf::from(
        output["data"]["artifact_path"]
            .as_str()
            .expect("artifact path should exist"),
    );
    let refit_run_path = PathBuf::from(
        output["data"]["refit_run_path"]
            .as_str()
            .expect("refit run path should exist"),
    );
    let model_registry_path = PathBuf::from(
        output["data"]["model_registry_path"]
            .as_str()
            .expect("model registry path should exist"),
    );

    assert!(artifact_path.exists());
    assert!(refit_run_path.exists());
    assert!(model_registry_path.exists());

    let artifact_json: Value =
        serde_json::from_slice(&fs::read(&artifact_path).expect("artifact should be readable"))
            .expect("artifact should be valid json");
    assert_eq!(
        artifact_json["model_id"],
        "a_share_equity_10d_direction_head"
    );
    assert_eq!(
        artifact_json["label_definition"],
        "security_forward_outcome.v1"
    );
    assert_eq!(artifact_json["training_window"], "2025-03-01..2025-08-31");
    assert_eq!(artifact_json["oot_window"], "2025-12-01..2026-01-31");
    assert!(artifact_json["features"].is_array());
    assert!(
        artifact_json["features"]
            .as_array()
            .expect("features should be an array")
            .len()
            >= 1
    );

    let persisted_refit_run: Value = serde_json::from_slice(
        &fs::read(&refit_run_path).expect("persisted refit run should be readable"),
    )
    .expect("persisted refit run should be valid json");
    assert_eq!(
        persisted_refit_run["candidate_artifact_path"],
        Value::String(artifact_path.to_string_lossy().to_string())
    );

    let persisted_model_registry: Value = serde_json::from_slice(
        &fs::read(&model_registry_path).expect("persisted model registry should be readable"),
    )
    .expect("persisted model registry should be valid json");
    assert_eq!(
        persisted_model_registry["artifact_path"],
        Value::String(artifact_path.to_string_lossy().to_string())
    );
    assert_eq!(persisted_model_registry["target_head"], "direction_head");
}

#[test]
fn security_scorecard_training_supports_return_head_with_regression_metrics() {
    assert_regression_head_training_support(
        "security_scorecard_training_return_head",
        "return_head",
        "2026-04-11T23:00:00+08:00",
    );
}

#[test]
fn security_scorecard_training_supports_drawdown_head_with_regression_metrics() {
    assert_regression_head_training_support(
        "security_scorecard_training_drawdown_head",
        "drawdown_head",
        "2026-04-11T23:10:00+08:00",
    );
}

#[test]
fn security_scorecard_training_supports_path_quality_head_with_regression_metrics() {
    assert_regression_head_training_support(
        "security_scorecard_training_path_quality_head",
        "path_quality_head",
        "2026-04-11T23:20:00+08:00",
    );
}

#[test]
fn security_scorecard_training_supports_upside_first_head_with_classification_metrics() {
    assert_path_event_head_training_support(
        "security_scorecard_training_upside_first_head",
        "upside_first_head",
        "2026-04-11T23:30:00+08:00",
    );
}

#[test]
fn security_scorecard_training_supports_stop_first_head_with_classification_metrics() {
    assert_path_event_head_training_support(
        "security_scorecard_training_stop_first_head",
        "stop_first_head",
        "2026-04-11T23:40:00+08:00",
    );
}

fn assert_regression_head_training_support(
    fixture_prefix: &str,
    target_head: &str,
    created_at: &str,
) {
    let runtime_db_path = create_test_runtime_db(fixture_prefix);
    let runtime_root = runtime_db_path
        .parent()
        .expect("runtime db should have parent")
        .join("scorecard_training_runtime");
    let fixture_dir = create_training_fixture_dir(fixture_prefix);

    let stock_up_csv = fixture_dir.join("stock_up.csv");
    let stock_down_csv = fixture_dir.join("stock_down.csv");
    let market_csv = fixture_dir.join("market.csv");
    let sector_csv = fixture_dir.join("sector.csv");

    // 2026-04-11 CST: Add a dedicated return-head fixture, because P3-2 needs a
    // formal regression-style training contract instead of keeping the governed
    // training chain locked to direction-only classification.
    // Purpose: prove the training tool can emit head-specific fit outputs for
    // future return estimation rather than rejecting every non-direction head.
    fs::write(
        &stock_up_csv,
        build_trend_rows(420, 92.0, 1.1, 1.0).join("\n"),
    )
    .expect("upward symbol csv should be written");
    fs::write(
        &stock_down_csv,
        build_trend_rows(420, 118.0, -0.8, 1.0).join("\n"),
    )
    .expect("downward symbol csv should be written");
    fs::write(
        &market_csv,
        build_trend_rows(420, 3200.0, 2.4, 5.0).join("\n"),
    )
    .expect("market csv should be written");
    fs::write(
        &sector_csv,
        build_trend_rows(420, 980.0, 1.6, 2.0).join("\n"),
    )
    .expect("sector csv should be written");

    import_history_csv(&runtime_db_path, &stock_up_csv, "601916.SH");
    import_history_csv(&runtime_db_path, &stock_down_csv, "600000.SH");
    import_history_csv(&runtime_db_path, &market_csv, "510300.SH");
    import_history_csv(&runtime_db_path, &sector_csv, "512800.SH");

    let server = spawn_http_route_server(vec![
        (
            "/financials",
            "HTTP/1.1 200 OK",
            r#"[
                {
                    "REPORT_DATE":"2025-12-31",
                    "NOTICE_DATE":"2026-03-28",
                    "TOTAL_OPERATE_INCOME":308227000000.0,
                    "YSTZ":8.37,
                    "PARENT_NETPROFIT":11117000000.0,
                    "SJLTZ":9.31,
                    "ROEJQ":14.8
                }
            ]"#,
            "application/json",
        ),
        (
            "/announcements",
            "HTTP/1.1 200 OK",
            r#"{
                "data":{
                    "list":[
                        {"notice_date":"2026-03-28","title":"2025骞村害鎶ュ憡","art_code":"AN202603281234567890","columns":[{"column_name":"瀹氭湡鎶ュ憡"}]},
                        {"notice_date":"2026-03-28","title":"2025骞村害鍒╂鼎鍒嗛厤棰勬鍏憡","art_code":"AN202603281234567891","columns":[{"column_name":"鍏徃鍏憡"}]}
                    ]
                }
            }"#,
            "application/json",
        ),
    ]);

    let request = json!({
        "tool": "security_scorecard_training",
        "args": {
            "created_at": created_at,
            "training_runtime_root": runtime_root.to_string_lossy(),
            "market_scope": "A_SHARE",
            "instrument_scope": "EQUITY",
            "symbol_list": ["601916.SH", "600000.SH"],
            "market_symbol": "510300.SH",
            "sector_symbol": "512800.SH",
            "market_profile": "a_share_core",
            "sector_profile": "a_share_bank",
            "horizon_days": 10,
            "target_head": target_head,
            "train_range": "2025-03-01..2025-08-31",
            "valid_range": "2025-09-01..2025-11-30",
            "test_range": "2025-12-01..2026-01-31",
            "train_samples_per_symbol": 6,
            "valid_samples_per_symbol": 3,
            "test_samples_per_symbol": 3,
            "feature_set_version": "security_feature_snapshot.v1",
            "label_definition_version": "security_forward_outcome.v1"
        }
    });

    let output = run_cli_with_json_runtime_and_envs(
        &request.to_string(),
        &runtime_db_path,
        &[
            (
                "EXCEL_SKILL_EASTMONEY_FINANCIAL_URL_BASE",
                format!("{server}/financials"),
            ),
            (
                "EXCEL_SKILL_EASTMONEY_ANNOUNCEMENT_URL_BASE",
                format!("{server}/announcements"),
            ),
        ],
    );

    // 2026-04-11 CST: Lock the formal regression-head contract, because P3-2
    // must prove the governed training tool can expose fit evidence for expected
    // return instead of rejecting every non-direction target.
    // Purpose: keep the later master scorecard and chair upgrades anchored to a
    // real trained return head rather than a verbal estimate.
    assert!(
        output["status"] == "ok",
        "unexpected path-head output: {output}"
    );
    assert_eq!(output["data"]["model_registry"]["target_head"], target_head);
    assert_eq!(
        output["data"]["metrics_summary_json"]["target_mode"],
        "regression"
    );
    assert!(
        output["data"]["metrics_summary_json"]["train"]["mae"].is_number(),
        "return head should expose MAE"
    );
    assert!(
        output["data"]["metrics_summary_json"]["train"]["rmse"].is_number(),
        "return head should expose RMSE"
    );
}

fn assert_path_event_head_training_support(
    fixture_prefix: &str,
    target_head: &str,
    created_at: &str,
) {
    let runtime_db_path = create_test_runtime_db(fixture_prefix);
    let runtime_root = runtime_db_path
        .parent()
        .expect("runtime db should have parent")
        .join("scorecard_training_runtime");
    let fixture_dir = create_training_fixture_dir(fixture_prefix);

    let stock_up_csv = fixture_dir.join("stock_up.csv");
    let stock_down_csv = fixture_dir.join("stock_down.csv");
    let market_csv = fixture_dir.join("market.csv");
    let sector_csv = fixture_dir.join("sector.csv");

    // 2026-04-11 CST: Add a dedicated path-event fixture, because P4 must prove
    // upside-first and stop-first are first-class governed classification heads
    // instead of remaining implicit background labels.
    // Purpose: lock explicit event-head metrics before implementation touches the
    // training reporting layer.
    let (stock_up_rows, stock_down_rows) = if target_head == "stop_first_head" {
        // 2026-04-11 CST: Retune the stop-first negative fixture, because the
        // earlier steep downtrend collapsed to the synthetic floor before the
        // governed sampling windows, which left every sampled horizon as `none`.
        // Purpose: keep the production mixed-label guard strict while making the
        // stop-first train/valid/test splits contain deterministic positive labels.
        (
            build_path_event_rows(420, 92.0, 0.20, 0.05, 0.01),
            build_path_event_rows(420, 420.0, -0.30, 0.05, 8.00),
        )
    } else {
        (
            build_path_event_rows(420, 92.0, 1.20, 0.55, 0.22),
            build_path_event_rows(420, 118.0, -1.40, 0.30, 1.85),
        )
    };
    fs::write(&stock_up_csv, stock_up_rows.join("\n"))
        .expect("upward symbol csv should be written");
    fs::write(&stock_down_csv, stock_down_rows.join("\n"))
        .expect("downward symbol csv should be written");
    fs::write(
        &market_csv,
        build_trend_rows(420, 3200.0, 2.4, 5.0).join("\n"),
    )
    .expect("market csv should be written");
    fs::write(
        &sector_csv,
        build_trend_rows(420, 980.0, 1.6, 2.0).join("\n"),
    )
    .expect("sector csv should be written");

    import_history_csv(&runtime_db_path, &stock_up_csv, "601916.SH");
    import_history_csv(&runtime_db_path, &stock_down_csv, "600000.SH");
    import_history_csv(&runtime_db_path, &market_csv, "510300.SH");
    import_history_csv(&runtime_db_path, &sector_csv, "512800.SH");

    let server = spawn_http_route_server(vec![
        (
            "/financials",
            "HTTP/1.1 200 OK",
            r#"[
                {
                    "REPORT_DATE":"2025-12-31",
                    "NOTICE_DATE":"2026-03-28",
                    "TOTAL_OPERATE_INCOME":308227000000.0,
                    "YSTZ":8.37,
                    "PARENT_NETPROFIT":11117000000.0,
                    "SJLTZ":9.31,
                    "ROEJQ":14.8
                }
            ]"#,
            "application/json",
        ),
        (
            "/announcements",
            "HTTP/1.1 200 OK",
            r#"{
                "data":{
                    "list":[
                        {"notice_date":"2026-03-28","title":"2025年度报告","art_code":"AN202603281234567890","columns":[{"column_name":"定期报告"}]},
                        {"notice_date":"2026-03-28","title":"2025年度利润分配预案公告","art_code":"AN202603281234567891","columns":[{"column_name":"公司公告"}]}
                    ]
                }
            }"#,
            "application/json",
        ),
    ]);

    // 2026-04-11 CST: Tune the stop-loss/target pair per path-event head, because
    // upside-first and stop-first need different trigger geometry to keep the train
    // split mixed under governed synthetic fixtures.
    // Purpose: preserve the strict positive/negative-label guard while making the
    // red tests exercise both event directions deterministically.
    let (stop_loss_pct, target_return_pct) = match target_head {
        "stop_first_head" => (0.015, 0.20),
        _ => (0.03, 0.04),
    };

    let request = json!({
        "tool": "security_scorecard_training",
        "args": {
            "created_at": created_at,
            "training_runtime_root": runtime_root.to_string_lossy(),
            "market_scope": "A_SHARE",
            "instrument_scope": "EQUITY",
            "symbol_list": ["601916.SH", "600000.SH"],
            "market_symbol": "510300.SH",
            "sector_symbol": "512800.SH",
            "market_profile": "a_share_core",
            "sector_profile": "a_share_bank",
            "horizon_days": 10,
            "target_head": target_head,
            "train_range": "2025-03-01..2025-08-31",
            "valid_range": "2025-09-01..2025-11-30",
            "test_range": "2025-12-01..2026-01-31",
            "train_samples_per_symbol": 6,
            "valid_samples_per_symbol": 3,
            "test_samples_per_symbol": 3,
            "feature_set_version": "security_feature_snapshot.v1",
            "label_definition_version": "security_forward_outcome.v1",
            "stop_loss_pct": stop_loss_pct,
            "target_return_pct": target_return_pct
        }
    });

    let output = run_cli_with_json_runtime_and_envs(
        &request.to_string(),
        &runtime_db_path,
        &[
            (
                "EXCEL_SKILL_EASTMONEY_FINANCIAL_URL_BASE",
                format!("{server}/financials"),
            ),
            (
                "EXCEL_SKILL_EASTMONEY_ANNOUNCEMENT_URL_BASE",
                format!("{server}/announcements"),
            ),
        ],
    );

    // 2026-04-11 CST: Add a path-event-head red test, because P4 requires
    // upside-first and stop-first to publish explicit governed classification
    // metrics instead of hiding behind generic direction-head reporting.
    // Purpose: make the training output state both the target head and its
    // event-positive-rate contract before master-scorecard integration begins.
    assert!(
        output["status"] == "ok",
        "unexpected path-head output: {output}"
    );
    assert_eq!(output["data"]["model_registry"]["target_head"], target_head);
    assert_eq!(
        output["data"]["metrics_summary_json"]["target_mode"],
        "classification"
    );
    assert!(
        output["data"]["metrics_summary_json"]["train"]["auc"].is_number(),
        "path-event head should expose AUC"
    );
    assert!(
        output["data"]["metrics_summary_json"]["train"]["ks"].is_number(),
        "path-event head should expose KS"
    );
    assert!(
        output["data"]["metrics_summary_json"]["train"]["event_positive_rate"].is_number(),
        "path-event head should expose event positive rate"
    );
    assert!(
        output["data"]["metrics_summary_json"]["readiness_assessment"]["head_path_event_coverage_status"]
            .is_string(),
        "path-event head should expose per-head event coverage readiness"
    );
}

// 2026-04-11 CST: Add a dedicated path-event fixture builder, because the generic
// drift-only rows were still too smooth for governed stop-first training and kept
// collapsing the train split into one-sided labels.
// Purpose: make upside-first and stop-first tests exercise real mixed event labels
// without weakening the production training guard.
fn build_path_event_rows(
    day_count: usize,
    start_close: f64,
    daily_drift: f64,
    upside_padding: f64,
    stop_padding: f64,
) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close = start_close;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let next_close = (close + daily_drift).max(1.0);
        let open = close;
        let swing_bias = match offset % 4 {
            0 => 1.0,
            1 => 0.35,
            2 => 0.8,
            _ => 0.2,
        };
        let high = open.max(next_close) + upside_padding * (1.0 + swing_bias);
        let dynamic_low_floor = (start_close * 0.01).max(0.05) + offset as f64 * 0.001;
        let low = (open.min(next_close) - stop_padding * (1.0 + (1.0 - swing_bias)))
            .max(dynamic_low_floor);
        let volume = 920_000 + offset as i64 * 7_500;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{next_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

fn import_history_csv(runtime_db_path: &Path, csv_path: &Path, symbol: &str) {
    let request = json!({
        "tool": "import_stock_price_history",
        "args": {
            "csv_path": csv_path.to_string_lossy(),
            "symbol": symbol,
            "source": "security_scorecard_training_fixture"
        }
    });

    let output = run_cli_with_json_runtime_and_envs(
        &request.to_string(),
        &runtime_db_path.to_path_buf(),
        &[],
    );
    assert_eq!(output["status"], "ok");
}

// 2026-04-09 CST: 这里构造可控趋势样本，原因是训练测试需要同时覆盖正负标签，但不希望把失败点散到复杂行情生成上；
// 目的：用可手算的上升/下降路径稳定生成 direction_head 样本，便于后续训练、回归与调试。
fn build_trend_rows(
    day_count: usize,
    start_close: f64,
    daily_drift: f64,
    intraday_padding: f64,
) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close = start_close;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let next_close = (close + daily_drift).max(1.0);
        let open = close;
        let high = open.max(next_close) + intraday_padding;
        // 2026-04-10 CST: 这里把训练夹具的 low 下限改成动态正数底，原因是固定 0.10 会让长下跌样本后段低点失去波动，
        // 目的：保留下跌夹具在极端区间的 low 变化，避免 RSRS 窗口被测试数据人为压成分母退化的假信号。
        let dynamic_low_floor = (start_close * 0.01).max(0.05) + offset as f64 * 0.001;
        let low = (open.min(next_close) - intraday_padding).max(dynamic_low_floor);
        let volume = 900_000 + offset as i64 * 8_000;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{next_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

#[test]
fn build_trend_rows_keeps_low_series_variable_in_downtrend_fixture() {
    // 2026-04-10 CST: 这里先补训练夹具退化的红测，原因是当前失败更像是下跌样本的 low 被固定楼板价压平，
    // 目的：先锁住“下跌夹具必须保留 low 序列变化”这个约束，避免直接改实现却没有抓到真正退化点。
    let rows = build_trend_rows(420, 120.0, -0.7, 1.0);
    let collapsed_low_count = rows
        .iter()
        .skip(1)
        .filter(|line| line.split(',').nth(3) == Some("0.10"))
        .count();

    assert_eq!(
        collapsed_low_count, 0,
        "下跌夹具不应该把 low 压成重复的 0.10 楼板价"
    );
}
