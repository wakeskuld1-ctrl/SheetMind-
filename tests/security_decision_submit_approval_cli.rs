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

// 2026-04-02 CST: 这里新增证券审批提交 CLI 测试夹具，原因是 P0-1 的核心不是再给一个分析结果，而是把投决对象正式送入审批主线；
// 目的：先锁住“证券投决会 -> 审批对象落盘”的正式合同，避免实现过程中把边界做散。
fn create_stock_history_csv(prefix: &str, file_name: &str, rows: &[String]) -> PathBuf {
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let fixture_dir = PathBuf::from("tests")
        .join("runtime_fixtures")
        .join("security_decision_submit_approval")
        .join(format!("{prefix}_{unique_suffix}"));
    fs::create_dir_all(&fixture_dir).expect("security decision submit fixture dir should exist");

    let csv_path = fixture_dir.join(file_name);
    fs::write(&csv_path, rows.join("\n")).expect("security decision submit csv should be written");
    csv_path
}

// 2026-04-02 CST: 这里复用本地 HTTP 假服务，原因是审批桥接测试仍然要经过真实证券研究与投决主链；
// 目的：把外部基本面与公告依赖稳定收进本地可控夹具，避免提交审批测试被外部接口波动打断。
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
fn tool_catalog_includes_security_decision_submit_approval() {
    let output = run_cli_with_json("");

    // 2026-04-02 CST: 这里先锁住新审批提交 Tool 的可发现性，原因是没进 catalog 就等于产品主入口不存在；
    // 目的：确保后续 Skill 与 CLI 能稳定找到“提交到审批主线”的正式入口。
    assert!(
        output["data"]["tool_catalog"]
            .as_array()
            .expect("tool catalog should be an array")
            .iter()
            .any(|tool| tool == "security_decision_submit_approval")
    );
}

#[test]
fn security_decision_submit_approval_writes_runtime_files_for_ready_case() {
    let runtime_db_path = create_test_runtime_db("security_decision_submit_approval_ready");
    let approval_root = runtime_db_path
        .parent()
        .expect("runtime db should have parent")
        .join("scenes_runtime");

    let stock_csv = create_stock_history_csv(
        "security_decision_submit_approval_ready",
        "stock.csv",
        &build_confirmed_breakout_rows(220, 88.0),
    );
    let market_csv = create_stock_history_csv(
        "security_decision_submit_approval_ready",
        "market.csv",
        &build_confirmed_breakout_rows(220, 3200.0),
    );
    let sector_csv = create_stock_history_csv(
        "security_decision_submit_approval_ready",
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

    let request = json!({
        "tool": "security_decision_submit_approval",
        "args": {
            "symbol": "601916.SH",
            "market_profile": "a_share_core",
            "sector_profile": "a_share_bank",
            "stop_loss_pct": 0.05,
            "target_return_pct": 0.12,
            "approval_runtime_root": approval_root.to_string_lossy(),
            "created_at": "2026-04-02T10:30:00+08:00"
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

    // 2026-04-02 CST: 这里锁住 ready_for_review 提交路径，原因是 P0-1 的目标就是把可上会的证券投决对象正式落进审批主线；
    // 目的：确保 decision/approval/audit 四类工件一次写齐，后续私有多签流程可以直接接着跑。
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["committee_result"]["decision_card"]["status"],
        "ready_for_review"
    );
    assert_eq!(output["data"]["approval_request"]["status"], "Pending");
    assert_eq!(output["data"]["approval_request"]["min_approvals"], 2);
    assert_eq!(
        output["data"]["approval_request"]["require_risk_signoff"],
        true
    );
    // 2026-04-08 CST: 这里先锁定 approval_request 对仓位计划的正式绑定，原因是 Task 2 要让 position_plan 从 package 附属文件升级成正式可审批对象；
    // 目的：确保审批请求自己就明确知道“审的是哪一个仓位计划、路径在哪、合同版本是什么”，而不是只依赖 package 间接推断。
    assert_eq!(
        output["data"]["approval_request"]["position_plan_binding"]["position_plan_ref"],
        output["data"]["position_plan"]["plan_id"]
    );
    assert_eq!(
        output["data"]["approval_request"]["position_plan_binding"]["position_plan_path"],
        output["data"]["position_plan_path"]
    );
    assert!(
        output["data"]["decision_ref"]
            .as_str()
            .expect("decision ref should exist")
            .starts_with("decision_ref:")
    );
    assert!(
        output["data"]["approval_ref"]
            .as_str()
            .expect("approval ref should exist")
            .starts_with("approval_ref:")
    );
    assert!(
        output["data"]["approval_brief"]["bull_summary"]
            .as_array()
            .expect("bull summary should be array")
            .len()
            >= 1
    );
    assert!(
        output["data"]["approval_brief"]["bear_summary"]
            .as_array()
            .expect("bear summary should be array")
            .len()
            >= 1
    );
    assert!(
        output["data"]["approval_brief"]["gate_summary"]
            .as_array()
            .expect("gate summary should be array")
            .len()
            >= 1
    );
    assert_eq!(
        output["data"]["position_plan"]["decision_ref"],
        output["data"]["decision_ref"]
    );
    assert_eq!(
        output["data"]["position_plan"]["approval_ref"],
        output["data"]["approval_ref"]
    );
    assert_eq!(
        output["data"]["position_plan"]["contract_version"],
        "security_position_plan.v2"
    );
    assert_eq!(
        output["data"]["position_plan"]["document_type"],
        "security_position_plan"
    );
    assert_eq!(output["data"]["position_plan"]["plan_direction"], "Long");
    assert_eq!(
        output["data"]["position_plan"]["approval_binding"]["approval_ref"],
        output["data"]["approval_ref"]
    );
    assert_eq!(
        output["data"]["position_plan"]["approval_binding"]["approval_request_ref"],
        output["data"]["approval_ref"]
    );
    assert_eq!(
        output["data"]["position_plan"]["reduce_plan"]["allow_reduce"],
        true
    );
    assert!(
        output["data"]["approval_brief"]["brief_id"]
            .as_str()
            .expect("brief id should exist")
            .starts_with("brief-")
    );
    assert_eq!(
        output["data"]["approval_brief"]["contract_version"],
        "security_approval_brief.v1"
    );
    assert_eq!(
        output["data"]["approval_brief"]["decision_ref"],
        output["data"]["decision_ref"]
    );
    assert_eq!(
        output["data"]["approval_brief"]["approval_ref"],
        output["data"]["approval_ref"]
    );
    assert_eq!(
        output["data"]["approval_brief"]["package_binding"]["artifact_role"],
        "approval_brief"
    );
    assert!(
        output["data"]["approval_brief"]["recommended_review_action"]
            .as_str()
            .expect("recommended review action should exist")
            .contains("approve")
    );
    assert!(
        output["data"]["approval_brief_path"]
            .as_str()
            .expect("approval brief path should exist")
            .contains("approval_briefs")
    );
    assert!(
        output["data"]["decision_package_path"]
            .as_str()
            .expect("decision package path should exist")
            .contains("decision_packages")
    );
    assert_eq!(output["data"]["position_plan"]["plan_status"], "reviewable");
    // 2026-04-08 CST: 这里先锁定 package 显式对象图合同，原因是 Task 1 要把 position_plan / approval_brief 从隐式 artifact 关系升级为正式对象引用；
    // 目的：确保 submit_approval 生成的新 package 不只是“文件清单存在”，而是已经把决策对象图写成可校验的正式合同。
    assert_eq!(
        output["data"]["decision_package"]["object_graph"]["decision_ref"],
        output["data"]["decision_ref"]
    );
    assert_eq!(
        output["data"]["decision_package"]["object_graph"]["approval_ref"],
        output["data"]["approval_ref"]
    );
    assert_eq!(
        output["data"]["decision_package"]["object_graph"]["position_plan_ref"],
        output["data"]["position_plan"]["plan_id"]
    );
    assert_eq!(
        output["data"]["decision_package"]["object_graph"]["approval_brief_ref"],
        output["data"]["approval_brief"]["brief_id"]
    );
    assert!(
        output["data"]["position_plan"]["suggested_gross_pct"]
            .as_f64()
            .expect("suggested gross pct should exist")
            > 0.0
    );
    assert!(
        output["data"]["approval_brief"]["entry_summary"]
            .as_str()
            .expect("entry summary should exist")
            .contains("首仓")
    );
    assert!(
        output["data"]["approval_brief"]["stop_loss_summary"]
            .as_str()
            .expect("stop loss summary should exist")
            .contains("止损")
    );

    let decision_path = PathBuf::from(
        output["data"]["decision_card_path"]
            .as_str()
            .expect("decision card path should exist"),
    );
    let approval_path = PathBuf::from(
        output["data"]["approval_request_path"]
            .as_str()
            .expect("approval request path should exist"),
    );
    let events_path = PathBuf::from(
        output["data"]["approval_events_path"]
            .as_str()
            .expect("approval events path should exist"),
    );
    let audit_path = PathBuf::from(
        output["data"]["audit_log_path"]
            .as_str()
            .expect("audit log path should exist"),
    );
    let approval_brief_path = PathBuf::from(
        output["data"]["approval_brief_path"]
            .as_str()
            .expect("approval brief path should exist"),
    );
    let decision_package_path = PathBuf::from(
        output["data"]["decision_package_path"]
            .as_str()
            .expect("decision package path should exist"),
    );
    let position_plan_path = PathBuf::from(
        output["data"]["position_plan_path"]
            .as_str()
            .expect("position plan path should exist"),
    );

    assert!(decision_path.exists());
    assert!(approval_path.exists());
    assert!(events_path.exists());
    assert!(audit_path.exists());
    assert!(approval_brief_path.exists());
    assert!(decision_package_path.exists());
    assert!(position_plan_path.exists());

    let persisted_decision: Value = serde_json::from_slice(
        &fs::read(&decision_path).expect("persisted decision card should be readable"),
    )
    .expect("persisted decision card should be valid json");
    assert_eq!(
        persisted_decision["scene_name"],
        "security_decision_committee"
    );
    assert_eq!(persisted_decision["asset_id"], "601916.SH");
    assert_eq!(persisted_decision["status"], "ReadyForReview");
    assert_eq!(persisted_decision["direction"], "Long");
    assert_eq!(persisted_decision["approval"]["approval_state"], "Pending");

    let persisted_request: Value = serde_json::from_slice(
        &fs::read(&approval_path).expect("persisted approval request should be readable"),
    )
    .expect("persisted approval request should be valid json");
    assert_eq!(persisted_request["status"], "Pending");
    assert_eq!(
        persisted_request["decision_id"],
        persisted_decision["decision_id"]
    );
    assert_eq!(persisted_request["auto_reject_recommended"], false);
    assert_eq!(
        persisted_request["position_plan_binding"]["position_plan_ref"],
        output["data"]["position_plan"]["plan_id"]
    );
    assert_eq!(
        persisted_request["position_plan_binding"]["plan_direction"],
        output["data"]["position_plan"]["plan_direction"]
    );

    let persisted_events: Value = serde_json::from_slice(
        &fs::read(&events_path).expect("persisted approval events should be readable"),
    )
    .expect("persisted approval events should be valid json");
    assert_eq!(
        persisted_events
            .as_array()
            .expect("approval events should be array")
            .len(),
        0
    );

    let audit_lines = fs::read_to_string(&audit_path).expect("audit log should be readable");
    assert_eq!(audit_lines.lines().count(), 1);
    let audit_record: Value = serde_json::from_str(
        audit_lines
            .lines()
            .next()
            .expect("audit log should contain first line"),
    )
    .expect("audit line should be valid json");
    assert_eq!(audit_record["event_type"], "decision_persisted");
    assert_eq!(audit_record["decision_status"], "ReadyForReview");
    assert_eq!(audit_record["approval_status"], "Pending");

    let persisted_approval_brief: Value = serde_json::from_slice(
        &fs::read(&approval_brief_path).expect("persisted approval brief should be readable"),
    )
    .expect("persisted approval brief should be valid json");
    assert_eq!(
        persisted_approval_brief["contract_version"],
        "security_approval_brief.v1"
    );
    assert_eq!(
        persisted_approval_brief["package_binding"]["artifact_role"],
        "approval_brief"
    );

    let persisted_position_plan: Value = serde_json::from_slice(
        &fs::read(&position_plan_path).expect("persisted position plan should be readable"),
    )
    .expect("persisted position plan should be valid json");
    assert_eq!(
        persisted_position_plan["contract_version"],
        "security_position_plan.v2"
    );
    assert_eq!(
        persisted_position_plan["document_type"],
        "security_position_plan"
    );
    assert_eq!(persisted_position_plan["plan_status"], "reviewable");
    assert_eq!(
        persisted_position_plan["decision_ref"],
        output["data"]["decision_ref"]
    );
    assert_eq!(
        persisted_position_plan["approval_ref"],
        output["data"]["approval_ref"]
    );
    assert_eq!(
        persisted_position_plan["approval_binding"]["approval_request_ref"],
        output["data"]["approval_ref"]
    );
    assert_eq!(persisted_position_plan["reduce_plan"]["allow_reduce"], true);

    let persisted_decision_package: Value = serde_json::from_slice(
        &fs::read(&decision_package_path).expect("persisted decision package should be readable"),
    )
    .expect("persisted decision package should be valid json");
    assert_eq!(
        persisted_decision_package["contract_version"],
        "security_decision_package.v1"
    );
    assert_eq!(
        persisted_decision_package["decision_ref"],
        output["data"]["decision_ref"]
    );
    assert_eq!(
        persisted_decision_package["approval_ref"],
        output["data"]["approval_ref"]
    );
    // 2026-04-08 CST: 这里补充持久化 package 的对象图断言，原因是 Task 1 需要冻结正式对象图，而不只是保证 CLI 返回值里短暂带出；
    // 目的：确保真正落盘的 package JSON 也具备稳定的 object_graph，后续 verify / revision 都能基于磁盘对象图继续工作。
    assert_eq!(
        persisted_decision_package["object_graph"]["position_plan_ref"],
        persisted_position_plan["plan_id"]
    );
    assert_eq!(
        persisted_decision_package["object_graph"]["approval_brief_ref"],
        persisted_approval_brief["brief_id"]
    );
    assert_eq!(
        persisted_decision_package["object_graph"]["position_plan_path"],
        Value::String(position_plan_path.to_string_lossy().to_string())
    );
    assert_eq!(
        persisted_decision_package["object_graph"]["approval_brief_path"],
        Value::String(approval_brief_path.to_string_lossy().to_string())
    );
    assert_eq!(
        persisted_decision_package["package_status"],
        "review_bundle_ready"
    );
    assert!(
        persisted_decision_package["artifact_manifest"]
            .as_array()
            .expect("artifact manifest should be array")
            .iter()
            .any(|artifact| artifact["artifact_role"] == "decision_card")
    );
    assert!(
        persisted_decision_package["artifact_manifest"]
            .as_array()
            .expect("artifact manifest should be array")
            .iter()
            .any(|artifact| artifact["artifact_role"] == "approval_request")
    );
    assert!(
        persisted_decision_package["artifact_manifest"]
            .as_array()
            .expect("artifact manifest should be array")
            .iter()
            .any(|artifact| artifact["artifact_role"] == "position_plan")
    );
    assert!(
        persisted_decision_package["artifact_manifest"]
            .as_array()
            .expect("artifact manifest should be array")
            .iter()
            .any(|artifact| artifact["artifact_role"] == "approval_brief")
    );
    assert_eq!(
        persisted_decision_package["governance_binding"]["decision_ref"],
        output["data"]["decision_ref"]
    );
    assert_eq!(
        persisted_decision_package["governance_binding"]["approval_ref"],
        output["data"]["approval_ref"]
    );
}

#[test]
fn security_decision_submit_approval_maps_blocked_status_and_auto_reject_flags() {
    let runtime_db_path = create_test_runtime_db("security_decision_submit_approval_blocked");
    let approval_root = runtime_db_path
        .parent()
        .expect("runtime db should have parent")
        .join("scenes_runtime");

    let stock_csv = create_stock_history_csv(
        "security_decision_submit_approval_blocked",
        "stock.csv",
        &build_confirmed_breakout_rows(220, 88.0),
    );
    let market_csv = create_stock_history_csv(
        "security_decision_submit_approval_blocked",
        "market.csv",
        &build_confirmed_breakout_rows(220, 3200.0),
    );
    let sector_csv = create_stock_history_csv(
        "security_decision_submit_approval_blocked",
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
                        {"notice_date":"2026-03-28","title":"2025年年度报告","art_code":"AN202603281234567890","columns":[{"column_name":"定期报告"}]}
                    ]
                }
            }"#,
            "application/json",
        ),
    ]);

    let request = json!({
        "tool": "security_decision_submit_approval",
        "args": {
            "symbol": "601916.SH",
            "market_profile": "a_share_core",
            "sector_profile": "a_share_bank",
            "stop_loss_pct": 0.05,
            "target_return_pct": 0.08,
            "approval_runtime_root": approval_root.to_string_lossy(),
            "created_at": "2026-04-02T10:45:00+08:00"
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

    // 2026-04-02 CST: 这里锁住 blocked 提交路径，原因是审批桥接不能只会处理“好看”的投决对象；
    // 目的：确保被风险闸门拦下的证券决策也能形成正式审批记录，并显式带上 auto-reject 语义。
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["committee_result"]["decision_card"]["status"],
        "blocked"
    );
    assert_eq!(
        output["data"]["approval_request"]["status"],
        "NeedsMoreEvidence"
    );
    assert_eq!(
        output["data"]["approval_request"]["auto_reject_recommended"],
        true
    );
    assert_eq!(output["data"]["position_plan"]["plan_status"], "blocked");
    assert_eq!(output["data"]["position_plan"]["suggested_gross_pct"], 0.0);
    assert_eq!(output["data"]["position_plan"]["starter_gross_pct"], 0.0);
    assert_eq!(output["data"]["position_plan"]["max_gross_pct"], 0.0);
    assert!(
        output["data"]["approval_brief"]["recommended_review_action"]
            .as_str()
            .expect("recommended review action should exist")
            .contains("request_more_evidence")
    );
    assert!(
        output["data"]["approval_request"]["auto_reject_gate_names"]
            .as_array()
            .expect("auto reject gate names should be array")
            .iter()
            .any(|gate| gate == "risk_reward_gate")
    );

    let decision_path = PathBuf::from(
        output["data"]["decision_card_path"]
            .as_str()
            .expect("decision card path should exist"),
    );
    let persisted_decision: Value = serde_json::from_slice(
        &fs::read(&decision_path).expect("persisted decision card should be readable"),
    )
    .expect("persisted decision card should be valid json");
    assert_eq!(persisted_decision["status"], "Blocked");
    assert_eq!(persisted_decision["direction"], "Long");
    assert_eq!(
        output["data"]["decision_package"]["package_status"],
        "needs_follow_up"
    );
}

#[test]
fn security_decision_submit_approval_can_write_detached_signature_for_approval_brief() {
    let runtime_db_path = create_test_runtime_db("security_decision_submit_approval_brief_signed");
    let approval_root = runtime_db_path
        .parent()
        .expect("runtime db should have parent")
        .join("scenes_runtime");

    let stock_csv = create_stock_history_csv(
        "security_decision_submit_approval_brief_signed",
        "stock.csv",
        &build_confirmed_breakout_rows(220, 88.0),
    );
    let market_csv = create_stock_history_csv(
        "security_decision_submit_approval_brief_signed",
        "market.csv",
        &build_confirmed_breakout_rows(220, 3200.0),
    );
    let sector_csv = create_stock_history_csv(
        "security_decision_submit_approval_brief_signed",
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

    let request = json!({
        "tool": "security_decision_submit_approval",
        "args": {
            "symbol": "601916.SH",
            "market_profile": "a_share_core",
            "sector_profile": "a_share_bank",
            "stop_loss_pct": 0.05,
            "target_return_pct": 0.12,
            "approval_runtime_root": approval_root.to_string_lossy(),
            "created_at": "2026-04-02T12:30:00+08:00",
            "approval_brief_signing_key_id": "brief_signing_key_20260402",
            "approval_brief_signing_key_secret": "brief-secret-for-tests"
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

    // 2026-04-02 CST: 这里锁住 detached signature 路径，原因是正式审批简报对象必须支持独立签名而不是停留在内存对象；
    // 目的：确保 approval brief 后续可以作为可审计工件进入 package，而不是只有正文没有签名锚点。
    assert_eq!(output["status"], "ok");
    let signature_path = PathBuf::from(
        output["data"]["approval_brief_signature_path"]
            .as_str()
            .expect("approval brief signature path should exist"),
    );
    assert!(signature_path.exists());

    let signature_envelope: Value = serde_json::from_slice(
        &fs::read(&signature_path).expect("approval brief signature should be readable"),
    )
    .expect("approval brief signature should be valid json");
    assert_eq!(
        signature_envelope["signature_version"],
        "security_approval_brief_signature.v1"
    );
    assert_eq!(signature_envelope["algorithm"], "hmac_sha256");
    assert_eq!(
        signature_envelope["contract_version"],
        "security_approval_brief.v1"
    );
    assert_eq!(signature_envelope["key_id"], "brief_signing_key_20260402");
    assert!(
        signature_envelope["brief_id"]
            .as_str()
            .expect("brief id should exist")
            .starts_with("brief-")
    );
    assert!(
        signature_envelope["payload_sha256"]
            .as_str()
            .expect("payload sha should exist")
            .len()
            >= 32
    );
    assert!(
        output["data"]["decision_package"]["artifact_manifest"]
            .as_array()
            .expect("artifact manifest should be array")
            .iter()
            .any(|artifact| artifact["artifact_role"] == "approval_brief_signature")
    );
}

fn import_history_csv(runtime_db_path: &Path, csv_path: &Path, symbol: &str) {
    let request = json!({
        "tool": "import_stock_price_history",
        "args": {
            "csv_path": csv_path.to_string_lossy(),
            "symbol": symbol,
            "source": "security_decision_submit_approval_fixture"
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
