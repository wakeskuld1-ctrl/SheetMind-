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
    fs::write(&csv_path, rows.join("\n")).expect("security decision verify csv should be written");
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
    assert!(
        output["data"]["tool_catalog"]
            .as_array()
            .expect("tool catalog should be an array")
            .iter()
            .any(|tool| tool == "security_decision_verify_package")
    );
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
    let verify_output =
        run_cli_with_json_runtime_and_envs(&verify_request.to_string(), &runtime_db_path, &[]);

    // 2026-04-02 CST: 这里锁住 signed package 的 happy path，原因是 P0-5 的核心就是证明正式审批包已可系统校验；
    // 目的：确保 manifest、detached signature 和治理绑定同时通过，verification report 也能落盘。
    assert_eq!(verify_output["status"], "ok");
    assert_eq!(verify_output["data"]["package_valid"], true);
    assert_eq!(
        verify_output["data"]["recommended_action"],
        "proceed_with_review"
    );
    assert!(
        verify_output["data"]["verification_report_path"]
            .as_str()
            .expect("verification report path should exist")
            .contains("decision_packages_verification")
    );
    assert!(
        verify_output["data"]["artifact_checks"]
            .as_array()
            .expect("artifact checks should be array")
            .iter()
            .all(|item| item["exists_on_disk"] == true)
    );
    assert!(
        verify_output["data"]["hash_checks"]
            .as_array()
            .expect("hash checks should be array")
            .iter()
            .all(|item| item["matched"] == true)
    );
    assert!(
        verify_output["data"]["signature_checks"]
            .as_array()
            .expect("signature checks should be array")
            .iter()
            .any(|item| item["signature_valid"] == true)
    );
    // 2026-04-08 CST: 这里先锁定 verify 输出中的对象图一致性结果，原因是 Task 1 不仅要写入 object_graph，还要把它纳入正式校验；
    // 目的：确保后续 package 就算文件还在，也不能在对象引用漂移时被误判为有效。
    assert_eq!(
        verify_output["data"]["governance_checks"]["object_graph_consistent"],
        true
    );
    // 2026-04-08 CST: 这里先锁定仓位计划正式挂入审批链后的校验输出，原因是 Task 2 不仅要落盘 binding，还要让 verify 对其进行正式约束；
    // 目的：确保后续审批链对仓位计划的引用、完整性和方向一致性都能被稳定验证，而不是只验证文件存在。
    assert_eq!(
        verify_output["data"]["governance_checks"]["position_plan_binding_consistent"],
        true
    );
    assert_eq!(
        verify_output["data"]["governance_checks"]["position_plan_complete"],
        true
    );
    assert_eq!(
        verify_output["data"]["governance_checks"]["position_plan_direction_aligned"],
        true
    );
    // 2026-04-09 CST: 这里补锁 scorecard 治理校验的 happy path，原因是本轮不只是把评分卡落盘，还要求它正式进入 package / verify 主链；
    // 目的：确保后续只要评分卡引用、完整性或动作语义漂移，verify 就能第一时间拦截，而不是只验证文件存在。
    assert_eq!(
        verify_output["data"]["governance_checks"]["scorecard_binding_consistent"],
        true
    );
    assert_eq!(
        verify_output["data"]["governance_checks"]["scorecard_complete"],
        true
    );
    assert_eq!(
        verify_output["data"]["governance_checks"]["scorecard_action_aligned"],
        true
    );
    // 2026-04-10 CST: 这里先锁住 package 内的条件复核绑定 happy path，原因是 Task 4 方案 B 要求把 condition_review_ref 与 digest 正式挂进 decision package；
    // 目的：确保 verify 不只校验 scorecard / position_plan，也会把“投中条件复核是否和当前 package 主锚点一致”纳入正式治理检查。
    let mut package_json: Value = serde_json::from_slice(
        &fs::read(&package_path).expect("decision package should be readable"),
    )
    .expect("decision package should be valid json");
    attach_condition_review_binding(
        &mut package_json,
        ConditionReviewBindingFixture {
            condition_review_ref: "condition-review:601916.SH:2026-04-02:manual_review:v1",
            generated_at: "2026-04-10T09:30:00Z",
            review_trigger_type: "manual_review",
            recommended_follow_up_action: "keep_plan",
            review_summary: "manual review confirms the package can keep the existing plan",
        },
    );
    fs::write(
        &package_path,
        serde_json::to_vec_pretty(&package_json).expect("package with condition review should serialize"),
    )
    .expect("package with condition review should be written");

    let verify_output = run_cli_with_json_runtime_and_envs(
        &verify_request.to_string(),
        &runtime_db_path,
        &[],
    );
    assert_eq!(
        verify_output["data"]["governance_checks"]["condition_review_binding_present"],
        true
    );
    assert_eq!(
        verify_output["data"]["governance_checks"]["condition_review_binding_consistent"],
        true
    );

    let report_path = PathBuf::from(
        verify_output["data"]["verification_report_path"]
            .as_str()
            .expect("verification report path should exist"),
    );
    assert!(report_path.exists());
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
    let verify_output =
        run_cli_with_json_runtime_and_envs(&verify_request.to_string(), &runtime_db_path, &[]);

    // 2026-04-02 CST: 这里锁住篡改失败路径，原因是 package 校验不能只会处理“正常工件”，还必须能识别审批简报被改写；
    // 目的：确保 manifest 哈希和 detached signature 至少有一条会报警，从而阻断带毒审批包继续流转。
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
    assert!(
        verify_output["data"]["hash_checks"]
            .as_array()
            .expect("hash checks should be array")
            .iter()
            .any(|item| item["artifact_role"] == "approval_brief" && item["matched"] == false)
    );
}

#[test]
fn security_decision_verify_package_fails_after_object_graph_is_tampered() {
    let runtime_db_path =
        create_test_runtime_db("security_decision_verify_package_object_graph_tampered");
    let approval_root = runtime_db_path
        .parent()
        .expect("runtime db should have parent")
        .join("scenes_runtime");

    let stock_csv = create_stock_history_csv(
        "security_decision_verify_package_object_graph_tampered",
        "stock.csv",
        &build_confirmed_breakout_rows(220, 88.0),
    );
    let market_csv = create_stock_history_csv(
        "security_decision_verify_package_object_graph_tampered",
        "market.csv",
        &build_confirmed_breakout_rows(220, 3200.0),
    );
    let sector_csv = create_stock_history_csv(
        "security_decision_verify_package_object_graph_tampered",
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
                        {"notice_date":"2026-03-28","title":"2025骞村勾搴︽姤鍛?,"art_code":"AN202603281234567890","columns":[{"column_name":"瀹氭湡鎶ュ憡"}]},
                        {"notice_date":"2026-03-28","title":"2025骞村害鍒╂鼎鍒嗛厤棰勬鍏憡","art_code":"AN202603281234567891","columns":[{"column_name":"鍏徃鍏憡"}]}
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
            "created_at": "2026-04-08T10:10:00+08:00",
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
        .expect("decision package path should exist");
    let mut package_json: Value = serde_json::from_slice(
        &fs::read(package_path).expect("decision package should be readable"),
    )
    .expect("decision package should be valid json");
    package_json["object_graph"]["approval_brief_path"] =
        Value::String("tampered/approval_brief.json".to_string());
    fs::write(
        package_path,
        serde_json::to_vec_pretty(&package_json).expect("tampered package should serialize"),
    )
    .expect("tampered package should be written");

    let verify_request = json!({
        "tool": "security_decision_verify_package",
        "args": {
            "package_path": package_path,
            "approval_brief_signing_key_secret": "brief-secret-for-tests"
        }
    });
    let verify_output =
        run_cli_with_json_runtime_and_envs(&verify_request.to_string(), &runtime_db_path, &[]);

    // 2026-04-08 CST: 这里锁定 object_graph 被篡改后的失败路径，原因是 Task 1 的核心就是“对象图本身也属于正式合同”；
    // 目的：确保 package 即便文件和 hash 仍可读，只要对象图路径与真实 artifact 清单不一致，也会被 verify 明确判为无效。
    assert_eq!(verify_output["status"], "ok");
    assert_eq!(verify_output["data"]["package_valid"], false);
    assert_eq!(
        verify_output["data"]["governance_checks"]["object_graph_consistent"],
        false
    );
    assert!(
        verify_output["data"]["issues"]
            .as_array()
            .expect("issues should be array")
            .iter()
            .any(|item| item
                .as_str()
                .expect("issue should be string")
                .contains("object_graph"))
    );
}

#[test]
fn security_decision_verify_package_fails_after_position_plan_binding_is_tampered() {
    let runtime_db_path =
        create_test_runtime_db("security_decision_verify_package_position_binding_tampered");
    let approval_root = runtime_db_path
        .parent()
        .expect("runtime db should have parent")
        .join("scenes_runtime");

    let stock_csv = create_stock_history_csv(
        "security_decision_verify_package_position_binding_tampered",
        "stock.csv",
        &build_confirmed_breakout_rows(220, 88.0),
    );
    let market_csv = create_stock_history_csv(
        "security_decision_verify_package_position_binding_tampered",
        "market.csv",
        &build_confirmed_breakout_rows(220, 3200.0),
    );
    let sector_csv = create_stock_history_csv(
        "security_decision_verify_package_position_binding_tampered",
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
            "created_at": "2026-04-08T10:20:00+08:00",
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
        .expect("decision package path should exist");
    let approval_request_path = submit_output["data"]["approval_request_path"]
        .as_str()
        .expect("approval request path should exist");
    let mut approval_request: Value = serde_json::from_slice(
        &fs::read(approval_request_path).expect("approval request should be readable"),
    )
    .expect("approval request should be valid json");
    // 2026-04-08 CST: 这里先补 binding 被篡改的红测，原因是 Task 2 的核心不是“有 position_plan 文件”而是“审批请求明确绑定哪个计划”；
    // 目的：确保只要 approval_request 对 plan 的正式引用漂移，verify 就会把整条审批链判为无效，而不是继续放行。
    approval_request["position_plan_binding"]["position_plan_ref"] =
        Value::String("tampered-plan-ref".to_string());
    fs::write(
        approval_request_path,
        serde_json::to_vec_pretty(&approval_request)
            .expect("tampered approval request should serialize"),
    )
    .expect("tampered approval request should be written");

    let verify_request = json!({
        "tool": "security_decision_verify_package",
        "args": {
            "package_path": package_path,
            "approval_brief_signing_key_secret": "brief-secret-for-tests"
        }
    });
    let verify_output =
        run_cli_with_json_runtime_and_envs(&verify_request.to_string(), &runtime_db_path, &[]);

    assert_eq!(verify_output["status"], "ok");
    assert_eq!(verify_output["data"]["package_valid"], false);
    assert_eq!(
        verify_output["data"]["governance_checks"]["position_plan_binding_consistent"],
        false
    );
    assert!(
        verify_output["data"]["issues"]
            .as_array()
            .expect("issues should be array")
            .iter()
            .any(|item| item
                .as_str()
                .expect("issue should be string")
                .contains("position_plan_binding"))
    );
}

#[test]
fn security_decision_verify_package_fails_after_position_plan_direction_is_tampered() {
    let runtime_db_path =
        create_test_runtime_db("security_decision_verify_package_position_direction_tampered");
    let approval_root = runtime_db_path
        .parent()
        .expect("runtime db should have parent")
        .join("scenes_runtime");

    let stock_csv = create_stock_history_csv(
        "security_decision_verify_package_position_direction_tampered",
        "stock.csv",
        &build_confirmed_breakout_rows(220, 88.0),
    );
    let market_csv = create_stock_history_csv(
        "security_decision_verify_package_position_direction_tampered",
        "market.csv",
        &build_confirmed_breakout_rows(220, 3200.0),
    );
    let sector_csv = create_stock_history_csv(
        "security_decision_verify_package_position_direction_tampered",
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
            "created_at": "2026-04-08T10:30:00+08:00",
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
        .expect("decision package path should exist");
    let position_plan_path = submit_output["data"]["position_plan_path"]
        .as_str()
        .expect("position plan path should exist");
    let mut position_plan: Value = serde_json::from_slice(
        &fs::read(position_plan_path).expect("position plan should be readable"),
    )
    .expect("position plan should be valid json");
    // 2026-04-08 CST: 这里补 direction 被篡改的红测，原因是 Task 2 除了绑定 plan 本体，还要求 plan 方向与投决方向显式对齐；
    // 目的：确保即便 position_plan 文件仍存在，只要方向被改写，verify 也会稳定打回而不是误判为可继续审议。
    position_plan["plan_direction"] = Value::String("Short".to_string());
    fs::write(
        position_plan_path,
        serde_json::to_vec_pretty(&position_plan).expect("tampered position plan should serialize"),
    )
    .expect("tampered position plan should be written");

    let verify_request = json!({
        "tool": "security_decision_verify_package",
        "args": {
            "package_path": package_path,
            "approval_brief_signing_key_secret": "brief-secret-for-tests"
        }
    });
    let verify_output =
        run_cli_with_json_runtime_and_envs(&verify_request.to_string(), &runtime_db_path, &[]);

    assert_eq!(verify_output["status"], "ok");
    assert_eq!(verify_output["data"]["package_valid"], false);
    assert_eq!(
        verify_output["data"]["governance_checks"]["position_plan_direction_aligned"],
        false
    );
    assert!(
        verify_output["data"]["issues"]
            .as_array()
            .expect("issues should be array")
            .iter()
            .any(|item| item
                .as_str()
                .expect("issue should be string")
                .contains("position_plan direction"))
    );
}

#[test]
fn security_decision_verify_package_fails_after_scorecard_action_is_tampered() {
    let runtime_db_path =
        create_test_runtime_db("security_decision_verify_package_scorecard_action_tampered");
    let approval_root = runtime_db_path
        .parent()
        .expect("runtime db should have parent")
        .join("scenes_runtime");

    let stock_csv = create_stock_history_csv(
        "security_decision_verify_package_scorecard_action_tampered",
        "stock.csv",
        &build_confirmed_breakout_rows(220, 88.0),
    );
    let market_csv = create_stock_history_csv(
        "security_decision_verify_package_scorecard_action_tampered",
        "market.csv",
        &build_confirmed_breakout_rows(220, 3200.0),
    );
    let sector_csv = create_stock_history_csv(
        "security_decision_verify_package_scorecard_action_tampered",
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
            "created_at": "2026-04-09T11:10:00+08:00",
            "approval_brief_signing_key_id": "brief_signing_key_20260409",
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
        .expect("decision package path should exist");
    let scorecard_path = submit_output["data"]["scorecard_path"]
        .as_str()
        .expect("scorecard path should exist");
    let mut scorecard: Value =
        serde_json::from_slice(&fs::read(scorecard_path).expect("scorecard should be readable"))
            .expect("scorecard should be valid json");
    // 2026-04-09 CST: 这里补 scorecard 动作被篡改的红测，原因是评分卡进入正式治理链后，必须和 decision_card 的动作语义保持一致；
    // 目的：确保就算篡改者不动 decision_card，只改评分卡，也会被 verify 明确识别并打回。
    scorecard["recommendation_action"] = Value::String("__tampered__".to_string());
    scorecard["exposure_side"] = Value::String("neutral".to_string());
    fs::write(
        scorecard_path,
        serde_json::to_vec_pretty(&scorecard).expect("tampered scorecard should serialize"),
    )
    .expect("tampered scorecard should be written");

    let verify_request = json!({
        "tool": "security_decision_verify_package",
        "args": {
            "package_path": package_path,
            "approval_brief_signing_key_secret": "brief-secret-for-tests"
        }
    });
    let verify_output =
        run_cli_with_json_runtime_and_envs(&verify_request.to_string(), &runtime_db_path, &[]);

    assert_eq!(verify_output["status"], "ok");
    assert_eq!(verify_output["data"]["package_valid"], false);
    assert_eq!(
        verify_output["data"]["governance_checks"]["scorecard_binding_consistent"],
        true
    );
    assert_eq!(
        verify_output["data"]["governance_checks"]["scorecard_action_aligned"],
        false
    );
    assert!(
        verify_output["data"]["issues"]
            .as_array()
            .expect("issues should be array")
            .iter()
            .any(|item| item
                .as_str()
                .expect("issue should be string")
                .contains("security_scorecard action"))
    );
}

#[test]
fn security_decision_verify_package_fails_after_condition_review_binding_is_tampered() {
    let runtime_db_path =
        create_test_runtime_db("security_decision_verify_package_condition_review_tampered");
    let approval_root = runtime_db_path
        .parent()
        .expect("runtime db should have parent")
        .join("scenes_runtime");

    let stock_csv = create_stock_history_csv(
        "security_decision_verify_package_condition_review_tampered",
        "stock.csv",
        &build_confirmed_breakout_rows(220, 88.0),
    );
    let market_csv = create_stock_history_csv(
        "security_decision_verify_package_condition_review_tampered",
        "market.csv",
        &build_confirmed_breakout_rows(220, 3200.0),
    );
    let sector_csv = create_stock_history_csv(
        "security_decision_verify_package_condition_review_tampered",
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
                        {"notice_date":"2026-03-28","title":"2025骞村害鎶ュ憡","art_code":"AN202603281234567890","columns":[{"column_name":"瀹氭湡鎶ュ憡"}]},
                        {"notice_date":"2026-03-28","title":"2025骞村害鍒╂鼎鍒嗛厤棰勬鍏憡","art_code":"AN202603281234567891","columns":[{"column_name":"鍏徃鍏憡"}]}
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
            "created_at": "2026-04-10T10:30:00+08:00",
            "approval_brief_signing_key_id": "brief_signing_key_20260410",
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
        .expect("decision package path should exist");
    let mut package_json: Value =
        serde_json::from_slice(&fs::read(package_path).expect("decision package should be readable"))
            .expect("decision package should be valid json");
    // 2026-04-10 CST: 这里补条件复核绑定漂移红测，原因是 Task 4 不只要求 package 能带上 condition_review_ref，
    // 目的：还要确保只要复核摘要与 package 主锚点不一致，verify 就会把它明确识别成治理失败，而不是静默放过。
    attach_condition_review_binding(
        &mut package_json,
        ConditionReviewBindingFixture {
            condition_review_ref: "condition-review:601916.SH:2026-04-02:manual_review:v1",
            generated_at: "2026-04-10T09:30:00Z",
            review_trigger_type: "manual_review",
            recommended_follow_up_action: "keep_plan",
            review_summary: "manual review confirms the package can keep the existing plan",
        },
    );
    package_json["condition_review_digest"]["decision_ref"] =
        Value::String("decision:tampered:v1".to_string());
    fs::write(
        package_path,
        serde_json::to_vec_pretty(&package_json).expect("tampered package should serialize"),
    )
    .expect("tampered package should be written");

    let verify_request = json!({
        "tool": "security_decision_verify_package",
        "args": {
            "package_path": package_path,
            "approval_brief_signing_key_secret": "brief-secret-for-tests"
        }
    });
    let verify_output =
        run_cli_with_json_runtime_and_envs(&verify_request.to_string(), &runtime_db_path, &[]);

    assert_eq!(verify_output["status"], "ok");
    assert_eq!(verify_output["data"]["package_valid"], false);
    assert_eq!(
        verify_output["data"]["governance_checks"]["condition_review_binding_present"],
        true
    );
    assert_eq!(
        verify_output["data"]["governance_checks"]["condition_review_binding_consistent"],
        false
    );
    assert!(
        verify_output["data"]["issues"]
            .as_array()
            .expect("issues should be array")
            .iter()
            .any(|item| item
                .as_str()
                .expect("issue should be string")
                .contains("condition_review"))
    );
}

#[derive(Clone, Copy)]
struct ConditionReviewBindingFixture<'a> {
    condition_review_ref: &'a str,
    generated_at: &'a str,
    review_trigger_type: &'a str,
    recommended_follow_up_action: &'a str,
    review_summary: &'a str,
}

// 2026-04-10 CST: 这里统一封装测试用的 condition review 绑定写入，原因是 Task 4 需要在多个 package 验证场景里重复注入同一类 binding；
// 目的：把 ref/digest 的最小测试合同集中到单点，避免各条测试分别手拼字段后再次出现命名漂移。
fn attach_condition_review_binding(
    package_json: &mut Value,
    fixture: ConditionReviewBindingFixture<'_>,
) {
    let decision_ref = package_json["decision_ref"]
        .as_str()
        .expect("decision_ref should exist")
        .to_string();
    let approval_ref = package_json["approval_ref"]
        .as_str()
        .expect("approval_ref should exist")
        .to_string();
    let symbol = package_json["symbol"]
        .as_str()
        .expect("symbol should exist")
        .to_string();
    let analysis_date = package_json["analysis_date"]
        .as_str()
        .expect("analysis_date should exist")
        .to_string();
    let position_plan_ref = package_json["object_graph"]["position_plan_ref"]
        .as_str()
        .expect("position_plan_ref should exist")
        .to_string();

    package_json["object_graph"]["condition_review_ref"] =
        Value::String(fixture.condition_review_ref.to_string());
    package_json["condition_review_digest"] = json!({
        "condition_review_ref": fixture.condition_review_ref,
        "generated_at": fixture.generated_at,
        "review_trigger_type": fixture.review_trigger_type,
        "recommended_follow_up_action": fixture.recommended_follow_up_action,
        "decision_ref": decision_ref,
        "approval_ref": approval_ref,
        "position_plan_ref": position_plan_ref,
        "symbol": symbol,
        "analysis_date": analysis_date,
        "review_summary": fixture.review_summary
    });
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
