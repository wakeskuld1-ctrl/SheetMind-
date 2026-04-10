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

// 2026-04-09 CST: 这里新增 package 治理链 CLI 测试夹具，原因是 Task 6 要先把“会后结论 -> package -> verify -> revision”
// 锁成正式对外合同；目的：先用红测定义最小闭环，再补实现，避免后续只写文档不落代码。
fn create_stock_history_csv(prefix: &str, file_name: &str, rows: &[String]) -> PathBuf {
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let fixture_dir = PathBuf::from("tests")
        .join("runtime_fixtures")
        .join("security_decision_package")
        .join(format!("{prefix}_{unique_suffix}"));
    fs::create_dir_all(&fixture_dir).expect("security decision package fixture dir should exist");

    let csv_path = fixture_dir.join(file_name);
    fs::write(&csv_path, rows.join("\n")).expect("security decision package csv should be written");
    csv_path
}

// 2026-04-09 CST: 这里复用本地 HTTP 假服务，原因是 package 测试仍然需要稳定的信息面输入，
// 目的：隔离外部接口波动，让失败点落在治理合同而不是联网抓数。
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
fn tool_catalog_includes_security_decision_package_chain() {
    let output = run_cli_with_json("");

    // 2026-04-09 CST: 这里先锁 package 治理链工具的可发现性，原因是如果 catalog 不暴露这些正式 Tool，
    // 那么后续 Skill 和 CLI 仍然会退回手工拼对象；目的：确保治理链是正式入口而不是内部 helper。
    for tool_name in [
        "security_record_post_meeting_conclusion",
        "security_decision_package",
        "security_decision_verify_package",
        "security_decision_package_revision",
    ] {
        assert!(
            output["data"]["tool_catalog"]
                .as_array()
                .expect("tool catalog should be an array")
                .iter()
                .any(|tool| tool == tool_name),
            "tool catalog should include {tool_name}"
        );
    }
}

#[test]
fn security_decision_package_mounts_post_meeting_conclusion_into_graph_and_manifest() {
    let runtime_db_path = create_test_runtime_db("security_decision_package_ready");
    let server = prepare_security_environment(&runtime_db_path, "security_decision_package_ready");
    let request = package_request();

    let output = run_cli_with_json_runtime_and_envs(
        &request.to_string(),
        &runtime_db_path,
        &security_envs(&server),
    );

    // 2026-04-09 CST: 这里先锁最关键的 package 合同，原因是用户明确要求会后结论不能只存在于文档口径，
    // 必须真正进入 object_graph 和 artifact_manifest；目的：确保 package 已成为后续 verify 的正式输入。
    assert_eq!(
        output["status"], "ok",
        "security_decision_package should succeed, output={output}"
    );
    assert_eq!(
        output["data"]["post_meeting_conclusion"]["document_type"],
        "security_post_meeting_conclusion"
    );
    assert!(contains_node(
        &output["data"]["object_graph"],
        "post_meeting_conclusion"
    ));
    assert!(contains_artifact(
        &output["data"]["artifact_manifest"],
        "security_post_meeting_conclusion"
    ));
    assert_eq!(
        output["data"]["post_trade_review"]["document_type"],
        "security_post_trade_review"
    );
    assert_eq!(
        output["data"]["execution_record"]["document_type"],
        "security_execution_record"
    );
    assert_eq!(
        output["data"]["execution_journal"]["document_type"],
        "security_execution_journal"
    );
    assert!(contains_node(
        &output["data"]["object_graph"],
        "post_trade_review"
    ));
    assert!(contains_node(
        &output["data"]["object_graph"],
        "execution_journal"
    ));
    assert!(contains_node(
        &output["data"]["object_graph"],
        "execution_record"
    ));
    assert!(contains_artifact(
        &output["data"]["artifact_manifest"],
        "security_post_trade_review"
    ));
    assert!(contains_artifact(
        &output["data"]["artifact_manifest"],
        "security_execution_journal"
    ));
    assert!(contains_artifact(
        &output["data"]["artifact_manifest"],
        "security_execution_record"
    ));

    let verify_request = json!({
        "tool": "security_decision_verify_package",
        "args": {
            "package": output["data"].clone()
        }
    });
    let verify_output = run_cli_with_json_runtime_and_envs(
        &verify_request.to_string(),
        &runtime_db_path,
        &security_envs(&server),
    );
    assert_eq!(verify_output["status"], "ok");
    assert_eq!(verify_output["data"]["verification_status"], "passed");
}

#[test]
fn security_decision_verify_package_flags_execution_journal_reference_misalignment_and_revision_suggests_repair()
 {
    let runtime_db_path =
        create_test_runtime_db("security_decision_package_execution_journal_misaligned");
    let server = prepare_security_environment(
        &runtime_db_path,
        "security_decision_package_execution_journal_misaligned",
    );
    let request = package_request();

    let output = run_cli_with_json_runtime_and_envs(
        &request.to_string(),
        &runtime_db_path,
        &security_envs(&server),
    );
    assert_eq!(output["status"], "ok");

    // 2026-04-09 CST: 这里新增 journal 错绑红测，原因是 journal 进入 package 后不能只校验“存在”；
    // 目的：确保 verify / revision 能识别 execution_journal 与 review / record 的引用漂移。
    let mut broken_package = output["data"].clone();
    broken_package["post_trade_review"]["execution_journal_ref"] =
        json!("execution-journal-broken");
    broken_package["execution_record"]["execution_journal_ref"] = json!("execution-journal-broken");
    broken_package["execution_journal"]["position_plan_ref"] = json!("position-plan-broken");
    broken_package["execution_journal"]["snapshot_ref"] = json!("snapshot-broken");
    broken_package["execution_journal"]["outcome_ref"] = json!("outcome-broken");

    let verify_request = json!({
        "tool": "security_decision_verify_package",
        "args": {
            "package": broken_package
        }
    });
    let verify_output = run_cli_with_json_runtime_and_envs(
        &verify_request.to_string(),
        &runtime_db_path,
        &security_envs(&server),
    );
    assert_eq!(verify_output["status"], "ok");
    assert_eq!(verify_output["data"]["verification_status"], "failed");
    assert!(
        verify_output["data"]["issues"]
            .as_array()
            .expect("issues should be an array")
            .iter()
            .any(|item| item["code"] == "execution_journal_ref_misaligned")
    );

    let revision_request = json!({
        "tool": "security_decision_package_revision",
        "args": {
            "package": output["data"].clone(),
            "verification": verify_output["data"].clone()
        }
    });
    let revision_output = run_cli_with_json_runtime_and_envs(
        &revision_request.to_string(),
        &runtime_db_path,
        &security_envs(&server),
    );
    assert_eq!(revision_output["status"], "ok");
    assert_eq!(
        revision_output["data"]["revision_status"],
        "repair_required"
    );
    assert!(
        revision_output["data"]["manifest_repairs"]
            .as_array()
            .expect("manifest repairs should be an array")
            .iter()
            .any(|item| item
                .as_str()
                .unwrap_or_default()
                .contains("execution_journal"))
    );
}

#[test]
fn security_decision_verify_package_flags_post_trade_review_reference_misalignment_and_revision_suggests_repair()
 {
    let runtime_db_path =
        create_test_runtime_db("security_decision_package_post_trade_review_misaligned");
    let server = prepare_security_environment(
        &runtime_db_path,
        "security_decision_package_post_trade_review_misaligned",
    );
    let request = package_request();

    let output = run_cli_with_json_runtime_and_envs(
        &request.to_string(),
        &runtime_db_path,
        &security_envs(&server),
    );
    assert_eq!(
        output["status"], "ok",
        "security_decision_package should succeed, output={output}"
    );

    // 2026-04-09 CST: 这里先锁投后复盘进入治理链后的引用一致性，原因是 Task 9 不能只做到“包里看起来有 review 文档”，
    // 目的：确保 verify 能识别 post_trade_review 和其底层 position_plan / snapshot / outcome 绑定被破坏的情况，revision 也要给出明确修补动作。
    let mut broken_package = output["data"].clone();
    broken_package["post_trade_review"]["position_plan_ref"] = json!("position-plan-broken");
    broken_package["post_trade_review"]["snapshot_ref"] = json!("snapshot-broken");
    broken_package["post_trade_review"]["outcome_ref"] = json!("outcome-broken");

    let verify_request = json!({
        "tool": "security_decision_verify_package",
        "args": {
            "package": broken_package
        }
    });
    let verify_output = run_cli_with_json_runtime_and_envs(
        &verify_request.to_string(),
        &runtime_db_path,
        &security_envs(&server),
    );
    assert_eq!(verify_output["status"], "ok");
    assert_eq!(verify_output["data"]["verification_status"], "failed");
    assert!(
        verify_output["data"]["issues"]
            .as_array()
            .expect("issues should be an array")
            .iter()
            .any(|item| item["code"] == "post_trade_review_ref_misaligned")
    );

    let revision_request = json!({
        "tool": "security_decision_package_revision",
        "args": {
            "package": output["data"].clone(),
            "verification": verify_output["data"].clone()
        }
    });
    let revision_output = run_cli_with_json_runtime_and_envs(
        &revision_request.to_string(),
        &runtime_db_path,
        &security_envs(&server),
    );
    assert_eq!(revision_output["status"], "ok");
    assert_eq!(
        revision_output["data"]["revision_status"],
        "repair_required"
    );
    assert!(
        revision_output["data"]["manifest_repairs"]
            .as_array()
            .expect("manifest repairs should be an array")
            .iter()
            .any(|item| item.as_str()
                == Some(
                    "重新绑定 post_trade_review 的 position_plan_ref / snapshot_ref / outcome_ref"
                ))
    );
}

#[test]
fn security_decision_verify_package_flags_execution_record_reference_misalignment_and_revision_suggests_repair()
 {
    let runtime_db_path =
        create_test_runtime_db("security_decision_package_execution_record_misaligned");
    let server = prepare_security_environment(
        &runtime_db_path,
        "security_decision_package_execution_record_misaligned",
    );
    let request = package_request();

    let output = run_cli_with_json_runtime_and_envs(
        &request.to_string(),
        &runtime_db_path,
        &security_envs(&server),
    );
    assert_eq!(
        output["status"], "ok",
        "security_decision_package should succeed, output={output}"
    );

    // 2026-04-09 CST: 这里锁 execution record 进入治理链后的引用一致性，原因是 Task 10 不能只做到“包里有 execution 对象”，
    // 目的：确保 verify 能识别 execution_record 与 review / position_plan / snapshot / outcome 的错绑，revision 也能给出修补动作。
    let mut broken_package = output["data"].clone();
    broken_package["execution_record"]["position_plan_ref"] = json!("position-plan-broken");
    broken_package["execution_record"]["snapshot_ref"] = json!("snapshot-broken");
    broken_package["execution_record"]["outcome_ref"] = json!("outcome-broken");
    broken_package["post_trade_review"]["execution_record_ref"] = json!("execution-record-broken");

    let verify_request = json!({
        "tool": "security_decision_verify_package",
        "args": {
            "package": broken_package
        }
    });
    let verify_output = run_cli_with_json_runtime_and_envs(
        &verify_request.to_string(),
        &runtime_db_path,
        &security_envs(&server),
    );
    assert_eq!(verify_output["status"], "ok");
    assert_eq!(verify_output["data"]["verification_status"], "failed");
    assert!(
        verify_output["data"]["issues"]
            .as_array()
            .expect("issues should be an array")
            .iter()
            .any(|item| item["code"] == "execution_record_ref_misaligned")
    );

    let revision_request = json!({
        "tool": "security_decision_package_revision",
        "args": {
            "package": output["data"].clone(),
            "verification": verify_output["data"].clone()
        }
    });
    let revision_output = run_cli_with_json_runtime_and_envs(
        &revision_request.to_string(),
        &runtime_db_path,
        &security_envs(&server),
    );
    assert_eq!(revision_output["status"], "ok");
    assert_eq!(
        revision_output["data"]["revision_status"],
        "repair_required"
    );
    assert!(
        revision_output["data"]["manifest_repairs"]
            .as_array()
            .expect("manifest repairs should be an array")
            .iter()
            .any(|item| item.as_str()
                == Some("重新绑定 execution_record 与 post_trade_review / position_plan / snapshot / outcome 的引用"))
    );
}

#[test]
fn security_decision_verify_package_flags_missing_post_meeting_bindings_and_revision_suggests_repair()
 {
    let runtime_db_path = create_test_runtime_db("security_decision_package_missing_post_meeting");
    let server = prepare_security_environment(
        &runtime_db_path,
        "security_decision_package_missing_post_meeting",
    );
    let request = package_request();

    let output = run_cli_with_json_runtime_and_envs(
        &request.to_string(),
        &runtime_db_path,
        &security_envs(&server),
    );
    assert_eq!(
        output["status"], "ok",
        "security_decision_package should succeed, output={output}"
    );

    // 2026-04-09 CST: 这里故意破坏 package 挂载，原因是用户不接受“看起来像有数据、其实链路没接上”的假收口，
    // 目的：锁住 verify 必须能识别缺失绑定，revision 必须给出明确修补动作。
    let mut broken_package = output["data"].clone();
    broken_package["object_graph"] = Value::Array(
        broken_package["object_graph"]
            .as_array()
            .expect("object graph should be an array")
            .iter()
            .filter(|item| item["object_type"] != "post_meeting_conclusion")
            .cloned()
            .collect(),
    );
    broken_package["artifact_manifest"] = Value::Array(
        broken_package["artifact_manifest"]
            .as_array()
            .expect("artifact manifest should be an array")
            .iter()
            .filter(|item| item["document_type"] != "security_post_meeting_conclusion")
            .cloned()
            .collect(),
    );

    let verify_request = json!({
        "tool": "security_decision_verify_package",
        "args": {
            "package": broken_package
        }
    });
    let verify_output = run_cli_with_json_runtime_and_envs(
        &verify_request.to_string(),
        &runtime_db_path,
        &security_envs(&server),
    );
    assert_eq!(verify_output["status"], "ok");
    assert_eq!(verify_output["data"]["verification_status"], "failed");
    assert!(
        verify_output["data"]["issues"]
            .as_array()
            .expect("issues should be an array")
            .iter()
            .any(|item| item["code"] == "missing_post_meeting_conclusion")
    );

    let revision_request = json!({
        "tool": "security_decision_package_revision",
        "args": {
            "package": output["data"].clone(),
            "verification": verify_output["data"].clone()
        }
    });
    let revision_output = run_cli_with_json_runtime_and_envs(
        &revision_request.to_string(),
        &runtime_db_path,
        &security_envs(&server),
    );
    assert_eq!(revision_output["status"], "ok");
    assert_eq!(
        revision_output["data"]["revision_status"],
        "repair_required"
    );
    assert!(
        revision_output["data"]["manifest_repairs"]
            .as_array()
            .expect("manifest repairs should be an array")
            .iter()
            .any(|item| item.as_str() == Some("补挂 post_meeting_conclusion 到 artifact_manifest"))
    );
}

fn prepare_security_environment(runtime_db_path: &Path, prefix: &str) -> String {
    // 2026-04-09 CST: 这里把 package 测试夹具长度从 220 提升到 420，原因是 Task 9 新接入 post_trade_review 后，
    // 既要满足技术分析至少 200 条历史样本，又要给 review_horizon_days=20 留出未来窗口；目的：修复治理链测试被夹具窗口截断的假失败。
    let stock_csv = create_stock_history_csv(
        prefix,
        "stock.csv",
        &build_confirmed_breakout_rows(420, 88.0),
    );
    let market_csv = create_stock_history_csv(
        prefix,
        "market.csv",
        &build_confirmed_breakout_rows(420, 3200.0),
    );
    let sector_csv = create_stock_history_csv(
        prefix,
        "sector.csv",
        &build_confirmed_breakout_rows(420, 950.0),
    );
    import_history_csv(runtime_db_path, &stock_csv, "601916.SH");
    import_history_csv(runtime_db_path, &market_csv, "510300.SH");
    import_history_csv(runtime_db_path, &sector_csv, "512800.SH");

    spawn_http_route_server(vec![
        (
            "/financials",
            "HTTP/1.1 200 OK",
            r#"[{"REPORT_DATE":"2025-12-31","NOTICE_DATE":"2026-03-28","TOTAL_OPERATE_INCOME":308227000000.0,"YSTZ":8.37,"PARENT_NETPROFIT":11117000000.0,"SJLTZ":9.31,"ROEJQ":14.8}]"#,
            "application/json",
        ),
        (
            "/announcements",
            "HTTP/1.1 200 OK",
            r#"{"data":{"list":[{"notice_date":"2026-03-28","title":"2025年度报告","art_code":"AN202603281234567890","columns":[{"column_name":"定期报告"}]},{"notice_date":"2026-03-28","title":"2025年度利润分配预案公告","art_code":"AN202603281234567891","columns":[{"column_name":"公司公告"}]}]}}"#,
            "application/json",
        ),
    ])
}

fn security_envs(server: &str) -> [(&'static str, String); 6] {
    [
        (
            "EXCEL_SKILL_EASTMONEY_FINANCIAL_URL_BASE",
            format!("{server}/financials"),
        ),
        (
            "EXCEL_SKILL_EASTMONEY_ANNOUNCEMENT_URL_BASE",
            format!("{server}/announcements"),
        ),
        (
            "EXCEL_SKILL_OFFICIAL_FINANCIAL_URL_BASE",
            format!("{server}/financials"),
        ),
        (
            "EXCEL_SKILL_OFFICIAL_ANNOUNCEMENT_URL_BASE",
            format!("{server}/announcements"),
        ),
        (
            "EXCEL_SKILL_SINA_FINANCIAL_URL_BASE",
            format!("{server}/financials"),
        ),
        (
            "EXCEL_SKILL_SINA_ANNOUNCEMENT_URL_BASE",
            format!("{server}/announcements"),
        ),
    ]
}

fn package_request() -> Value {
    json!({
        "tool": "security_decision_package",
        "args": {
            "symbol": "601916.SH",
            "market_regime": "a_share",
            "sector_template": "bank",
            "market_symbol": "510300.SH",
            "sector_symbol": "512800.SH",
            "market_profile": "a_share_core",
            "sector_profile": "a_share_bank",
            // 2026-04-09 CST: 这里把快照日后移到 2025-10-15，原因是 package 治理链同时需要满足
            // “技术分析至少 200 条历史样本”和“review_horizon_days=20 的未来观察窗口”；目的：让 Task 9 的失败只反映治理逻辑，而不是测试日期窗口过早。
            "as_of_date": "2025-10-15",
            "stop_loss_pct": 0.05,
            "target_return_pct": 0.12,
            "review_horizon_days": 20,
            "actual_entry_date": "2025-10-16",
            "actual_entry_price": 68.20,
            "actual_position_pct": 0.12,
            "actual_exit_date": "2025-10-29",
            "actual_exit_price": 71.60,
            "exit_reason": "take_profit_partial",
            "execution_record_notes": [
                "突破确认后一日建仓",
                "目标位附近先兑现部分利润"
            ],
            "created_at": "2026-04-09T12:00:00+08:00",
            "execution_notes": [
                "保持观察仓位，不满足条件不追价",
                "若后续补证据失败，则退回下一轮投决"
            ],
            "follow_up_actions": [
                "核对下一次财报披露后的盈利增速",
                "跟踪银行板块与沪深300相对强弱"
            ]
        }
    })
}

fn contains_node(object_graph: &Value, object_type: &str) -> bool {
    object_graph
        .as_array()
        .expect("object graph should be an array")
        .iter()
        .any(|item| item["object_type"] == object_type)
}

fn contains_artifact(artifact_manifest: &Value, document_type: &str) -> bool {
    artifact_manifest
        .as_array()
        .expect("artifact manifest should be an array")
        .iter()
        .any(|item| item["document_type"] == document_type)
}

fn import_history_csv(runtime_db_path: &Path, csv_path: &Path, symbol: &str) {
    let request = json!({
        "tool": "import_stock_price_history",
        "args": {
            "csv_path": csv_path.to_string_lossy(),
            "symbol": symbol,
            "source": "security_decision_package_fixture"
        }
    });

    let output = run_cli_with_json_runtime_and_envs(
        &request.to_string(),
        &runtime_db_path.to_path_buf(),
        &[],
    );
    assert_eq!(output["status"], "ok");
}

// 2026-04-09 CST: 这里沿用稳定上行末端突破样本，原因是 package 测试目标不是重复验证行情因子本身，
// 目的：把噪声压低，专注验证治理对象装配和校验链。
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
