mod common;

use chrono::{Duration, NaiveDate};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::common::{create_test_runtime_db, run_cli_with_json, run_cli_with_json_runtime_and_envs};

// 2026-04-02 CST: 这里新增 package revision 测试夹具，原因是 P0-6 的核心是“审批包跟着审批动作生成新版本”； 
// 目的：把 v1 package -> 更新审批工件 -> 生成 v2 package 的最小闭环锁进独立测试。
fn create_stock_history_csv(prefix: &str, file_name: &str, rows: &[String]) -> PathBuf {
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let fixture_dir = PathBuf::from("tests")
        .join("runtime_fixtures")
        .join("security_decision_package_revision")
        .join(format!("{prefix}_{unique_suffix}"));
    fs::create_dir_all(&fixture_dir).expect("security decision package revision fixture dir should exist");

    let csv_path = fixture_dir.join(file_name);
    fs::write(&csv_path, rows.join("\n"))
        .expect("security decision package revision csv should be written");
    csv_path
}

// 2026-04-02 CST: 这里复用本地 HTTP 假服务，原因是 revision 测试仍然需要先生成真实审批包；
// 目的：保证财报和公告证据稳定可重放，不让外部接口波动干扰 package 版本化测试。
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
        for _ in 0..route_map.len() {
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
fn tool_catalog_includes_security_decision_package_revision() {
    let output = run_cli_with_json("");

    // 2026-04-02 CST: 这里先锁住 revision Tool 的可发现性，原因是审批包版本化如果不进 catalog，就无法成为正式主链能力；
    // 目的：确保 CLI / Skill / 后续自动化都能稳定发现“生成下一个 package 版本”的入口。
    assert!(output["data"]["tool_catalog"]
        .as_array()
        .expect("tool catalog should be an array")
        .iter()
        .any(|tool| tool == "security_decision_package_revision"));
}

#[test]
fn security_decision_package_revision_builds_v2_package_after_approval_update() {
    let runtime_db_path = create_test_runtime_db("security_decision_package_revision_v2");
    let approval_root = runtime_db_path
        .parent()
        .expect("runtime db should have parent")
        .join("scenes_runtime");

    let stock_csv = create_stock_history_csv(
        "security_decision_package_revision_v2",
        "stock.csv",
        &build_confirmed_breakout_rows(220, 88.0),
    );
    let market_csv = create_stock_history_csv(
        "security_decision_package_revision_v2",
        "market.csv",
        &build_confirmed_breakout_rows(220, 3200.0),
    );
    let sector_csv = create_stock_history_csv(
        "security_decision_package_revision_v2",
        "sector.csv",
        &build_confirmed_breakout_rows(220, 950.0),
    );
    import_history_csv(&runtime_db_path, &stock_csv, "601916.SH");
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
                        {"notice_date":"2026-03-28","title":"2025年年度报告","art_code":"AN202603281234567890","columns":[{"column_name":"定期报告"}]},
                        {"notice_date":"2026-03-28","title":"2025年度利润分配预案公告","art_code":"AN202603281234567891","columns":[{"column_name":"公司公告"}]}
                    ]
                }
            }"#,
            "application/json",
        ),
    ]);

    let submit_request = json!({
        "tool": "security_decision_submit_approval",
        "args": {
            "symbol": "601916.SH",
            "market_profile": "a_share_core",
            "sector_profile": "a_share_bank",
            "stop_loss_pct": 0.05,
            "target_return_pct": 0.12,
            "approval_runtime_root": approval_root.to_string_lossy(),
            "created_at": "2026-04-02T18:00:00+08:00",
            "approval_brief_signing_key_id": "brief_signing_key_20260402",
            "approval_brief_signing_key_secret": "brief-secret-for-tests"
        }
    });

    let submit_output = run_cli_with_json_runtime_and_envs(
        &submit_request.to_string(),
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

    let package_path = PathBuf::from(
        submit_output["data"]["decision_package_path"]
            .as_str()
            .expect("decision package path should exist"),
    );
    let approval_request_path = PathBuf::from(
        submit_output["data"]["approval_request_path"]
            .as_str()
            .expect("approval request path should exist"),
    );
    let approval_events_path = PathBuf::from(
        submit_output["data"]["approval_events_path"]
            .as_str()
            .expect("approval events path should exist"),
    );
    let audit_log_path = PathBuf::from(
        submit_output["data"]["audit_log_path"]
            .as_str()
            .expect("audit log path should exist"),
    );

    let mut approval_request: Value = serde_json::from_slice(
        &fs::read(&approval_request_path).expect("approval request should be readable"),
    )
    .expect("approval request should be valid json");
    approval_request["status"] = Value::String("Approved".to_string());
    approval_request["approved_reviewers"] = json!(["risk_officer", "pm_lead"]);
    approval_request["approved_signatures"] = json!([
        {
            "reviewer": "risk_officer",
            "reviewer_role": "RiskOfficer",
            "timestamp": "2026-04-02T18:20:00+08:00"
        },
        {
            "reviewer": "pm_lead",
            "reviewer_role": "PortfolioManager",
            "timestamp": "2026-04-02T18:22:00+08:00"
        }
    ]);
    fs::write(
        &approval_request_path,
        serde_json::to_vec_pretty(&approval_request).expect("approval request should serialize"),
    )
    .expect("approval request should be updated");

    let approval_events = json!([
        {
            "approval_id": approval_request["approval_ref"],
            "decision_id": approval_request["decision_id"],
            "reviewer": "risk_officer",
            "reviewer_role": "RiskOfficer",
            "action": "Approve",
            "timestamp": "2026-04-02T18:20:00+08:00",
            "notes": "risk cleared",
            "override_reason": null,
            "decision_version": 1
        },
        {
            "approval_id": approval_request["approval_ref"],
            "decision_id": approval_request["decision_id"],
            "reviewer": "pm_lead",
            "reviewer_role": "PortfolioManager",
            "action": "Approve",
            "timestamp": "2026-04-02T18:22:00+08:00",
            "notes": "pm approved",
            "override_reason": null,
            "decision_version": 2
        }
    ]);
    fs::write(
        &approval_events_path,
        serde_json::to_vec_pretty(&approval_events).expect("approval events should serialize"),
    )
    .expect("approval events should be updated");

    let mut audit_lines = fs::read_to_string(&audit_log_path).expect("audit log should be readable");
    audit_lines.push_str(
        "{\"event_type\":\"approval_action_applied\",\"timestamp\":\"2026-04-02T18:22:00+08:00\",\"decision_id\":\"");
    audit_lines.push_str(
        approval_request["decision_id"]
            .as_str()
            .expect("decision id should exist"),
    );
    audit_lines.push_str("\",\"decision_ref\":");
    audit_lines.push_str(&approval_request["decision_ref"].to_string());
    audit_lines.push_str(",\"approval_ref\":\"");
    audit_lines.push_str(
        approval_request["approval_ref"]
            .as_str()
            .expect("approval ref should exist"),
    );
    audit_lines.push_str("\",\"evidence_hash\":");
    audit_lines.push_str(&approval_request["evidence_hash"].to_string());
    audit_lines.push_str(",\"governance_hash\":");
    audit_lines.push_str(&approval_request["governance_hash"].to_string());
    audit_lines.push_str(",\"decision_status\":\"Approved\",\"approval_status\":\"Approved\",\"reviewer\":\"pm_lead\",\"reviewer_role\":\"PortfolioManager\",\"approval_action\":\"Approve\",\"notes\":\"pm approved\",\"override_reason\":null,\"decision_version\":2,\"signature_key_id\":null,\"signature_algorithm\":null,\"signature_path\":null,\"signed_payload_sha256\":null,\"signed_contract_version\":null,\"prev_hash\":null,\"record_hash\":null}\n");
    fs::write(&audit_log_path, audit_lines).expect("audit log should be updated");

    let revision_request = json!({
        "tool": "security_decision_package_revision",
        "args": {
            "package_path": package_path.to_string_lossy(),
            "revision_reason": "approval_event_applied",
            "reverify_after_revision": true,
            "approval_brief_signing_key_secret": "brief-secret-for-tests"
        }
    });
    let revision_output = run_cli_with_json_runtime_and_envs(
        &revision_request.to_string(),
        &runtime_db_path,
        &[],
    );

    // 2026-04-02 CST: 这里锁住审批动作后的 v2 package 主路径，原因是 P0-6 的目标就是让审批包开始具备正式版本史；
    // 目的：确保更新后的审批工件会驱动新 package 版本生成，并且能带上前版本引用、触发摘要和新的 verification report。
    assert_eq!(revision_output["status"], "ok");
    assert_eq!(revision_output["data"]["package_version"], 2);
    assert_eq!(
        revision_output["data"]["revision_reason"],
        "approval_event_applied"
    );
    assert_eq!(
        revision_output["data"]["previous_package_path"],
        Value::String(package_path.to_string_lossy().to_string())
    );
    assert_eq!(
        revision_output["data"]["decision_package"]["package_status"],
        "approved_bundle_ready"
    );
    assert!(revision_output["data"]["trigger_event_summary"]
        .as_str()
        .expect("trigger event summary should exist")
        .contains("pm_lead"));
    assert!(revision_output["data"]["verification_report_path"]
        .as_str()
        .expect("verification report path should exist")
        .contains("decision_packages_verification"));

    let revised_package_path = PathBuf::from(
        revision_output["data"]["decision_package_path"]
            .as_str()
            .expect("decision package path should exist"),
    );
    assert!(revised_package_path.exists());

    let revised_package: Value = serde_json::from_slice(
        &fs::read(&revised_package_path).expect("revised package should be readable"),
    )
    .expect("revised package should be valid json");
    assert_eq!(revised_package["package_version"], 2);
    assert_eq!(revised_package["revision_reason"], "approval_event_applied");
    assert_eq!(
        revised_package["previous_package_path"],
        package_path.to_string_lossy().to_string()
    );
    assert_eq!(revised_package["package_status"], "approved_bundle_ready");
    assert!(revised_package["artifact_manifest"]
        .as_array()
        .expect("artifact manifest should be array")
        .iter()
        .any(|artifact| artifact["artifact_role"] == "approval_events"));
}

fn import_history_csv(runtime_db_path: &Path, csv_path: &Path, symbol: &str) {
    let request = json!({
        "tool": "import_stock_price_history",
        "args": {
            "csv_path": csv_path.to_string_lossy(),
            "symbol": symbol,
            "source": "security_decision_package_revision_fixture"
        }
    });

    let output = run_cli_with_json_runtime_and_envs(
        &request.to_string(),
        &runtime_db_path.to_path_buf(),
        &[],
    );
    assert_eq!(output["status"], "ok");
}

fn build_confirmed_breakout_rows(day_count: usize, start_close: f64) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close = start_close;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let (next_close, volume): (f64, i64) = if offset < day_count - 20 {
            (close + 0.78, 880_000 + offset as i64 * 8_000)
        } else {
            let phase = offset - (day_count - 20);
            match phase % 4 {
                0 => (close + 1.35, 1_700_000 + phase as i64 * 26_000),
                1 => (close - 0.18, 420_000),
                2 => (close + 1.08, 1_540_000 + phase as i64 * 22_000),
                _ => (close + 0.42, 1_240_000),
            }
        };

        let open = close;
        let high = next_close.max(open) + 1.0;
        let low = next_close.min(open) - 0.86;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}
