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

// 2026-04-02 CST: 这里新增独立的证券 package 校验测试夹具，原因是 P0-5 需要验证“先提交审批包，再回头校验”的完整往返路径；
// 目的：把 package verify 的 happy path 和篡改路径都锁在独立测试里，避免和 submit_approval 测试耦得过重。
fn create_stock_history_csv(prefix: &str, file_name: &str, rows: &[String]) -> PathBuf {
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let fixture_dir = PathBuf::from("tests")
        .join("runtime_fixtures")
        .join("security_decision_verify_package")
        .join(format!("{prefix}_{unique_suffix}"));
    fs::create_dir_all(&fixture_dir).expect("security decision verify fixture dir should exist");

    let csv_path = fixture_dir.join(file_name);
    fs::write(&csv_path, rows.join("\n"))
        .expect("security decision verify csv should be written");
    csv_path
}

// 2026-04-02 CST: 这里复用本地 HTTP 假服务，原因是 verify 测试仍然需要先跑 submit_approval 生成真实 package；
// 目的：把财报和公告依赖继续限制在本地可控夹具里，保证 package 验证测试稳定可重放。
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
fn tool_catalog_includes_security_decision_verify_package() {
    let output = run_cli_with_json("");

    // 2026-04-02 CST: 这里先锁住 verify Tool 的可发现性，原因是 package 校验如果不进 catalog，就无法进入正式产品主链；
    // 目的：确保 CLI / Skill / 后续自动化都能稳定发现“审批包校验”入口。
    assert!(output["data"]["tool_catalog"]
        .as_array()
        .expect("tool catalog should be an array")
        .iter()
        .any(|tool| tool == "security_decision_verify_package"));
}

#[test]
fn security_decision_verify_package_accepts_signed_package_and_writes_report() {
    let runtime_db_path = create_test_runtime_db("security_decision_verify_package_signed");
    let approval_root = runtime_db_path
        .parent()
        .expect("runtime db should have parent")
        .join("scenes_runtime");

    let stock_csv = create_stock_history_csv(
        "security_decision_verify_package_signed",
        "stock.csv",
        &build_confirmed_breakout_rows(220, 88.0),
    );
    let market_csv = create_stock_history_csv(
        "security_decision_verify_package_signed",
        "market.csv",
        &build_confirmed_breakout_rows(220, 3200.0),
    );
    let sector_csv = create_stock_history_csv(
        "security_decision_verify_package_signed",
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
            "created_at": "2026-04-02T16:30:00+08:00",
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
    let package_path = submit_output["data"]["decision_package_path"]
        .as_str()
        .expect("decision package path should exist")
        .to_string();

    let verify_request = json!({
        "tool": "security_decision_verify_package",
        "args": {
            "package_path": package_path,
            "approval_brief_signing_key_secret": "brief-secret-for-tests"
        }
    });
    let verify_output = run_cli_with_json_runtime_and_envs(
        &verify_request.to_string(),
        &runtime_db_path,
        &[],
    );

    // 2026-04-02 CST: 这里锁住 signed package 的 happy path，原因是 P0-5 的核心就是证明正式审批包已可系统校验；
    // 目的：确保 manifest、detached signature 和治理绑定同时通过，verification report 也能落盘。
    assert_eq!(verify_output["status"], "ok");
    assert_eq!(verify_output["data"]["package_valid"], true);
    assert_eq!(
        verify_output["data"]["recommended_action"],
        "proceed_with_review"
    );
    assert!(verify_output["data"]["verification_report_path"]
        .as_str()
        .expect("verification report path should exist")
        .contains("decision_packages_verification"));
    assert!(verify_output["data"]["artifact_checks"]
        .as_array()
        .expect("artifact checks should be array")
        .iter()
        .all(|item| item["exists_on_disk"] == true));
    assert!(verify_output["data"]["hash_checks"]
        .as_array()
        .expect("hash checks should be array")
        .iter()
        .all(|item| item["matched"] == true));
    assert!(verify_output["data"]["signature_checks"]
        .as_array()
        .expect("signature checks should be array")
        .iter()
        .any(|item| item["signature_valid"] == true));

    let report_path = PathBuf::from(
        verify_output["data"]["verification_report_path"]
            .as_str()
            .expect("verification report path should exist"),
    );
    assert!(report_path.exists());
}

#[test]
fn security_decision_verify_package_accepts_recorded_post_meeting_conclusion() {
    let (runtime_db_path, package_path, _) =
        create_recorded_post_meeting_fixture("security_decision_verify_package_post_meeting_ok");

    let verify_request = json!({
        "tool": "security_decision_verify_package",
        "args": {
            "package_path": package_path,
            "approval_brief_signing_key_secret": "brief-secret-for-tests"
        }
    });
    let verify_output = run_cli_with_json_runtime_and_envs(
        &verify_request.to_string(),
        &runtime_db_path,
        &[],
    );

    // 2026-04-08 CST: 这里补 Task 11 的会后结论 happy path 红灯，原因是 verify 目前还不知道 package 已经正式挂上 post_meeting_conclusion；
    // 目的：锁定后续实现必须显式返回绑定一致、brief 配对一致、关键字段完整这三类校验结果。
    assert_eq!(verify_output["status"], "ok");
    assert_eq!(verify_output["data"]["package_valid"], true);
    assert_eq!(
        verify_output["data"]["governance_checks"]["post_meeting_conclusion_binding_consistent"],
        true
    );
    assert_eq!(
        verify_output["data"]["governance_checks"]["post_meeting_conclusion_brief_paired"],
        true
    );
    assert_eq!(
        verify_output["data"]["governance_checks"]["post_meeting_conclusion_complete"],
        true
    );
}

#[test]
fn security_decision_verify_package_fails_after_approval_brief_is_tampered() {
    let runtime_db_path = create_test_runtime_db("security_decision_verify_package_tampered");
    let approval_root = runtime_db_path
        .parent()
        .expect("runtime db should have parent")
        .join("scenes_runtime");

    let stock_csv = create_stock_history_csv(
        "security_decision_verify_package_tampered",
        "stock.csv",
        &build_confirmed_breakout_rows(220, 88.0),
    );
    let market_csv = create_stock_history_csv(
        "security_decision_verify_package_tampered",
        "market.csv",
        &build_confirmed_breakout_rows(220, 3200.0),
    );
    let sector_csv = create_stock_history_csv(
        "security_decision_verify_package_tampered",
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
            "created_at": "2026-04-02T16:45:00+08:00",
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
    let package_path = submit_output["data"]["decision_package_path"]
        .as_str()
        .expect("decision package path should exist")
        .to_string();
    let approval_brief_path = submit_output["data"]["approval_brief_path"]
        .as_str()
        .expect("approval brief path should exist");

    let mut approval_brief: Value = serde_json::from_slice(
        &fs::read(approval_brief_path).expect("approval brief should be readable"),
    )
    .expect("approval brief should be valid json");
    approval_brief["executive_summary"] = Value::String("tampered-summary".to_string());
    fs::write(
        approval_brief_path,
        serde_json::to_vec_pretty(&approval_brief).expect("tampered brief should serialize"),
    )
    .expect("tampered approval brief should be written");

    let verify_request = json!({
        "tool": "security_decision_verify_package",
        "args": {
            "package_path": package_path,
            "approval_brief_signing_key_secret": "brief-secret-for-tests"
        }
    });
    let verify_output = run_cli_with_json_runtime_and_envs(
        &verify_request.to_string(),
        &runtime_db_path,
        &[],
    );

    // 2026-04-02 CST: 这里锁住篡改失败路径，原因是 package 校验不能只会处理“正常工件”，还必须能识别审批简报被改写；
    // 目的：确保 manifest 哈希和 detached signature 至少有一条会报警，从而阻断带毒审批包继续流转。
    assert_eq!(verify_output["status"], "ok");
    assert_eq!(verify_output["data"]["package_valid"], false);
    assert_eq!(
        verify_output["data"]["recommended_action"],
        "quarantine_and_rebuild"
    );
    assert!(verify_output["data"]["issues"]
        .as_array()
        .expect("issues should be array")
        .len()
        >= 1);
    assert!(verify_output["data"]["hash_checks"]
        .as_array()
        .expect("hash checks should be array")
        .iter()
        .any(|item| item["artifact_role"] == "approval_brief" && item["matched"] == false));
}

#[test]
fn security_decision_verify_package_fails_when_post_meeting_conclusion_is_tampered() {
    let tamper_cases = [
        (
            "security_decision_verify_package_tampered_post_meeting_brief_ref",
            "source_brief_ref",
            Value::String("brief-tampered".to_string()),
        ),
        (
            "security_decision_verify_package_tampered_post_meeting_source_package",
            "source_package_path",
            Value::String("tampered/package/path.json".to_string()),
        ),
        (
            "security_decision_verify_package_tampered_post_meeting_disposition",
            "final_disposition",
            Value::String("unknown_disposition".to_string()),
        ),
    ];

    for (fixture_name, field_name, tampered_value) in tamper_cases {
        let (runtime_db_path, package_path, conclusion_path) =
            create_recorded_post_meeting_fixture(fixture_name);
        let mut conclusion: Value = serde_json::from_slice(
            &fs::read(&conclusion_path).expect("post meeting conclusion should be readable"),
        )
        .expect("post meeting conclusion should be valid json");
        conclusion[field_name] = tampered_value;
        fs::write(
            &conclusion_path,
            serde_json::to_vec_pretty(&conclusion)
                .expect("tampered post meeting conclusion should serialize"),
        )
        .expect("tampered post meeting conclusion should be written");

        let verify_request = json!({
            "tool": "security_decision_verify_package",
            "args": {
                "package_path": package_path,
                "approval_brief_signing_key_secret": "brief-secret-for-tests"
            }
        });
        let verify_output = run_cli_with_json_runtime_and_envs(
            &verify_request.to_string(),
            &runtime_db_path,
            &[],
        );

        // 2026-04-08 CST: 这里补 Task 11 的篡改红灯，原因是会后结论进入 package 后，verify 必须能识别绑定漂移与字段失真；
        // 目的：分别锁住 brief_ref、source_package_path、final_disposition 三条高风险篡改路径，防止“文件在但关系已坏”仍被放行。
        assert_eq!(verify_output["status"], "ok");
        assert_eq!(verify_output["data"]["package_valid"], false);
        assert_eq!(
            verify_output["data"]["recommended_action"],
            "quarantine_and_rebuild"
        );
        assert!(
            verify_output["data"]["issues"]
                .as_array()
                .expect("issues should be array")
                .len()
                >= 1
        );
    }
}

fn import_history_csv(runtime_db_path: &Path, csv_path: &Path, symbol: &str) {
    let request = json!({
        "tool": "import_stock_price_history",
        "args": {
            "csv_path": csv_path.to_string_lossy(),
            "symbol": symbol,
            "source": "security_decision_verify_package_fixture"
        }
    });

    let output = run_cli_with_json_runtime_and_envs(
        &request.to_string(),
        &runtime_db_path.to_path_buf(),
        &[],
    );
    assert_eq!(output["status"], "ok");
}

fn create_recorded_post_meeting_fixture(prefix: &str) -> (PathBuf, String, String) {
    let runtime_db_path = create_test_runtime_db(prefix);
    let approval_root = runtime_db_path
        .parent()
        .expect("runtime db should have parent")
        .join("scenes_runtime");

    let stock_csv = create_stock_history_csv(prefix, "stock.csv", &build_confirmed_breakout_rows(220, 88.0));
    let market_csv = create_stock_history_csv(prefix, "market.csv", &build_confirmed_breakout_rows(220, 3200.0));
    let sector_csv = create_stock_history_csv(prefix, "sector.csv", &build_confirmed_breakout_rows(220, 950.0));
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
                        {"notice_date":"2026-03-28","title":"2025年度报告","art_code":"AN202603281234567890","columns":[{"column_name":"定期报告"}]},
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
            "created_at": "2026-04-08T18:00:00+08:00",
            "approval_brief_signing_key_id": "brief_signing_key_20260408",
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

    let package_path = submit_output["data"]["decision_package_path"]
        .as_str()
        .expect("decision package path should exist")
        .to_string();
    let record_request = json!({
        "tool": "security_record_post_meeting_conclusion",
        "args": {
            "package_path": package_path,
            "final_disposition": "approve",
            "disposition_reason": "committee_adopted_majority",
            "key_reasons": ["risk_cleared", "thesis_accepted"],
            "required_follow_ups": ["track_post_approval_execution"],
            "reviewer_notes": "committee accepted the majority view",
            "reviewer": "pm_lead",
            "reviewer_role": "PortfolioManager",
            "revision_reason": "post_meeting_conclusion_recorded",
            "reverify_after_revision": true,
            "approval_brief_signing_key_secret": "brief-secret-for-tests"
        }
    });
    let record_output = run_cli_with_json_runtime_and_envs(
        &record_request.to_string(),
        &runtime_db_path,
        &[],
    );
    assert_eq!(record_output["status"], "ok");

    (
        runtime_db_path,
        record_output["data"]["decision_package_path"]
            .as_str()
            .expect("revised package path should exist")
            .to_string(),
        record_output["data"]["post_meeting_conclusion_path"]
            .as_str()
            .expect("post meeting conclusion path should exist")
            .to_string(),
    )
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
