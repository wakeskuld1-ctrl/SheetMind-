mod common;

use chrono::{Duration, NaiveDate};
use serde_json::json;
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

// 2026-04-01 CST: 这里补新总 Tool 的专属 CSV 夹具助手，原因是 fullstack Tool 仍然要沿用 `CSV -> SQLite -> 技术面 -> 上层聚合` 主链；
// 目的：先锁定真实主链上的对外合同，而不是用手工拼接的技术面 JSON 伪造成功路径。
fn create_stock_history_csv(prefix: &str, file_name: &str, rows: &[String]) -> PathBuf {
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let fixture_dir = PathBuf::from("tests")
        .join("runtime_fixtures")
        .join("security_analysis_fullstack")
        .join(format!("{prefix}_{unique_suffix}"));
    fs::create_dir_all(&fixture_dir).expect("security fullstack fixture dir should exist");

    let csv_path = fixture_dir.join(file_name);
    fs::write(&csv_path, rows.join("\n")).expect("security fullstack csv should be written");
    csv_path
}

// 2026-04-01 CST: 这里补多路由本地 HTTP 假服务，原因是 fullstack Tool 会并行访问财报和公告两个外部免费源；
// 目的：让回归测试稳定重放“按路径返回不同响应”的信息面合同，而不是依赖真实第三方接口当天状态。
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
fn tool_catalog_includes_security_analysis_fullstack() {
    let output = run_cli_with_json("");

    // 2026-04-01 CST: 这里先锁目录可发现性，原因是新总 Tool 如果没进 catalog，Skill 和外部 EXE 根本无法稳定发现；
    // 目的：防止只实现业务逻辑却漏掉 dispatcher/catalog 暴露，导致产品主链名义存在、实际不可用。
    assert!(
        output["data"]["tool_catalog"]
            .as_array()
            .expect("tool catalog should be an array")
            .iter()
            .any(|tool| tool == "security_analysis_fullstack")
    );
}

#[test]
fn security_analysis_fullstack_aggregates_technical_fundamental_and_disclosures() {
    let runtime_db_path = create_test_runtime_db("security_analysis_fullstack_ok");

    let stock_csv = create_stock_history_csv(
        "security_analysis_fullstack_ok",
        "stock.csv",
        &build_range_bound_rows(220, 35.8, 38.0),
    );
    let market_csv = create_stock_history_csv(
        "security_analysis_fullstack_ok",
        "market.csv",
        &build_confirmed_breakdown_rows(220, 4.9),
    );
    let sector_csv = create_stock_history_csv(
        "security_analysis_fullstack_ok",
        "sector.csv",
        &build_choppy_history_rows(220, 1.02),
    );
    import_history_csv(&runtime_db_path, &stock_csv, "002352.SZ");
    import_history_csv(&runtime_db_path, &market_csv, "510300.SH");
    import_history_csv(&runtime_db_path, &sector_csv, "516530.SH");

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
                        {"notice_date":"2026-03-28","title":"2025年度利润分配预案公告","art_code":"AN202603281234567891","columns":[{"column_name":"公司公告"}]},
                        {"notice_date":"2026-03-10","title":"关于股份回购进展的公告","art_code":"AN202603101234567892","columns":[{"column_name":"公司公告"}]}
                    ]
                }
            }"#,
            "application/json",
        ),
    ]);

    let request = json!({
        "tool": "security_analysis_fullstack",
        "args": {
            "symbol": "002352.SZ",
            "market_symbol": "510300.SH",
            "sector_symbol": "516530.SH",
            "disclosure_limit": 3
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
            (
                "EXCEL_SKILL_OFFICIAL_FINANCIAL_URL_BASE",
                format!("{server}/official-financials"),
            ),
            (
                "EXCEL_SKILL_OFFICIAL_ANNOUNCEMENT_URL_BASE",
                format!("{server}/official-announcements"),
            ),
            (
                "EXCEL_SKILL_SINA_FINANCIAL_URL_BASE",
                format!("{server}/sina-financials"),
            ),
            (
                "EXCEL_SKILL_SINA_ANNOUNCEMENT_URL_BASE",
                format!("{server}/sina-announcements"),
            ),
        ],
    );

    // 2026-04-01 CST: 这里先锁“技术面 + 财报 + 公告”聚合成功主路径，原因是这是新总 Tool 首版对产品最关键的交付合同；
    // 目的：确保上层调用方只打一枪就能拿到完整证券分析骨架，而不是继续手工拼多个 Tool 结果。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["symbol"], "002352.SZ");
    assert_eq!(
        output["data"]["technical_context"]["contextual_conclusion"]["alignment"],
        "mixed"
    );
    assert_eq!(output["data"]["fundamental_context"]["status"], "available");
    assert_eq!(
        output["data"]["fundamental_context"]["latest_report_period"],
        "2025-12-31"
    );
    assert_eq!(
        output["data"]["fundamental_context"]["profit_signal"],
        "positive"
    );
    assert_eq!(output["data"]["disclosure_context"]["status"], "available");
    assert_eq!(
        output["data"]["disclosure_context"]["announcement_count"],
        3
    );
    assert_eq!(
        output["data"]["industry_context"]["sector_symbol"],
        "516530.SH"
    );
    assert_eq!(
        output["data"]["integrated_conclusion"]["stance"],
        "watchful_positive"
    );
}

#[test]
fn security_analysis_fullstack_degrades_gracefully_when_info_sources_fail() {
    let runtime_db_path = create_test_runtime_db("security_analysis_fullstack_degraded");

    let stock_csv = create_stock_history_csv(
        "security_analysis_fullstack_degraded",
        "stock.csv",
        &build_range_bound_rows(220, 35.8, 38.0),
    );
    let market_csv = create_stock_history_csv(
        "security_analysis_fullstack_degraded",
        "market.csv",
        &build_confirmed_breakdown_rows(220, 4.9),
    );
    let sector_csv = create_stock_history_csv(
        "security_analysis_fullstack_degraded",
        "sector.csv",
        &build_choppy_history_rows(220, 1.02),
    );
    import_history_csv(&runtime_db_path, &stock_csv, "002352.SZ");
    import_history_csv(&runtime_db_path, &market_csv, "510300.SH");
    import_history_csv(&runtime_db_path, &sector_csv, "516530.SH");

    let server = spawn_http_route_server(vec![
        (
            "/financials",
            "HTTP/1.1 500 Internal Server Error",
            r#"{"error":"financial upstream failed"}"#,
            "application/json",
        ),
        (
            "/announcements",
            "HTTP/1.1 500 Internal Server Error",
            r#"{"error":"announcement upstream failed"}"#,
            "application/json",
        ),
        (
            "/official-financials",
            "HTTP/1.1 500 Internal Server Error",
            r#"{"error":"official financial upstream failed"}"#,
            "application/json",
        ),
        (
            "/official-announcements",
            "HTTP/1.1 500 Internal Server Error",
            r#"{"error":"official announcement upstream failed"}"#,
            "application/json",
        ),
        (
            "/sina-financials",
            "HTTP/1.1 500 Internal Server Error",
            r#"{"error":"sina financial upstream failed"}"#,
            "application/json",
        ),
        (
            "/sina-announcements",
            "HTTP/1.1 500 Internal Server Error",
            r#"{"error":"sina announcement upstream failed"}"#,
            "application/json",
        ),
    ]);

    let request = json!({
        "tool": "security_analysis_fullstack",
        "args": {
            "symbol": "002352.SZ",
            "market_symbol": "510300.SH",
            "sector_symbol": "516530.SH"
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
            (
                "EXCEL_SKILL_OFFICIAL_FINANCIAL_URL_BASE",
                format!("{server}/official-financials"),
            ),
            (
                "EXCEL_SKILL_OFFICIAL_ANNOUNCEMENT_URL_BASE",
                format!("{server}/official-announcements"),
            ),
            (
                "EXCEL_SKILL_SINA_FINANCIAL_URL_BASE",
                format!("{server}/sina-financials"),
            ),
            (
                "EXCEL_SKILL_SINA_ANNOUNCEMENT_URL_BASE",
                format!("{server}/sina-announcements"),
            ),
        ],
    );

    // 2026-04-01 CST: 这里补“信息面源失败但主 Tool 不崩”的降级合同，原因是第三方免费源天然存在限流和抖动；
    // 目的：让产品在外部源异常时仍能返回技术主结论，并显式告诉上层哪些信息面维度缺失。
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["fundamental_context"]["status"],
        "unavailable"
    );
    assert_eq!(
        output["data"]["disclosure_context"]["status"],
        "unavailable"
    );
    assert_eq!(
        output["data"]["integrated_conclusion"]["stance"],
        "technical_only"
    );
    assert!(
        output["data"]["integrated_conclusion"]["risk_flags"]
            .as_array()
            .expect("risk flags should exist")
            .len()
            >= 1
    );
}

#[test]
fn security_analysis_fullstack_falls_back_to_sina_sources_when_eastmoney_fails() {
    let runtime_db_path = create_test_runtime_db("security_analysis_fullstack_sina_fallback");

    let stock_csv = create_stock_history_csv(
        "security_analysis_fullstack_sina_fallback",
        "stock.csv",
        &build_range_bound_rows(220, 35.8, 38.0),
    );
    let market_csv = create_stock_history_csv(
        "security_analysis_fullstack_sina_fallback",
        "market.csv",
        &build_confirmed_breakdown_rows(220, 4.9),
    );
    let sector_csv = create_stock_history_csv(
        "security_analysis_fullstack_sina_fallback",
        "sector.csv",
        &build_choppy_history_rows(220, 1.02),
    );
    import_history_csv(&runtime_db_path, &stock_csv, "002352.SZ");
    import_history_csv(&runtime_db_path, &market_csv, "510300.SH");
    import_history_csv(&runtime_db_path, &sector_csv, "516530.SH");

    let server = spawn_http_route_server(vec![
        (
            "/financials",
            "HTTP/1.1 500 Internal Server Error",
            r#"{"error":"eastmoney financial upstream failed"}"#,
            "application/json",
        ),
        (
            "/announcements",
            "HTTP/1.1 500 Internal Server Error",
            r#"{"error":"eastmoney announcement upstream failed"}"#,
            "application/json",
        ),
        (
            "/sina-financials",
            "HTTP/1.1 200 OK",
            &build_sina_financial_guideline_fixture(),
            "text/html; charset=utf-8",
        ),
        (
            "/sina-announcements",
            "HTTP/1.1 200 OK",
            &build_sina_all_bulletin_fixture(),
            "text/html; charset=utf-8",
        ),
    ]);

    let request = json!({
        "tool": "security_analysis_fullstack",
        "args": {
            "symbol": "002352.SZ",
            "market_symbol": "510300.SH",
            "sector_symbol": "516530.SH",
            "disclosure_limit": 3
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
            (
                "EXCEL_SKILL_SINA_FINANCIAL_URL_BASE",
                format!("{server}/sina-financials"),
            ),
            (
                "EXCEL_SKILL_SINA_ANNOUNCEMENT_URL_BASE",
                format!("{server}/sina-announcements"),
            ),
        ],
    );

    // 2026-04-02 CST: 这里先锁“东财失败时默认新浪备源接管”的产品合同，原因是当前真实环境里东财 TLS 已经稳定复现不可用；
    // 目的：确保 fullstack Tool 在主源失联后仍能返回完整信息面，而不是一律退化成 technical_only。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["fundamental_context"]["status"], "available");
    assert_eq!(
        output["data"]["fundamental_context"]["source"],
        "sina_financial_guideline"
    );
    assert_eq!(
        output["data"]["fundamental_context"]["latest_report_period"],
        "2025-12-31"
    );
    assert_eq!(
        output["data"]["fundamental_context"]["profit_signal"],
        "positive"
    );
    assert_eq!(output["data"]["disclosure_context"]["status"], "available");
    assert_eq!(
        output["data"]["disclosure_context"]["source"],
        "sina_announcements"
    );
    assert_eq!(
        output["data"]["disclosure_context"]["announcement_count"],
        3
    );
    assert_ne!(
        output["data"]["integrated_conclusion"]["stance"],
        "technical_only"
    );
}

#[test]
fn security_analysis_fullstack_recognizes_real_chinese_sina_financial_labels() {
    let runtime_db_path =
        create_test_runtime_db("security_analysis_fullstack_sina_real_chinese_labels");

    let stock_csv = create_stock_history_csv(
        "security_analysis_fullstack_sina_real_chinese_labels",
        "stock.csv",
        &build_range_bound_rows(220, 35.8, 38.0),
    );
    let market_csv = create_stock_history_csv(
        "security_analysis_fullstack_sina_real_chinese_labels",
        "market.csv",
        &build_confirmed_breakdown_rows(220, 4.9),
    );
    let sector_csv = create_stock_history_csv(
        "security_analysis_fullstack_sina_real_chinese_labels",
        "sector.csv",
        &build_choppy_history_rows(220, 1.02),
    );
    import_history_csv(&runtime_db_path, &stock_csv, "601998.SH");
    import_history_csv(&runtime_db_path, &market_csv, "510300.SH");
    import_history_csv(&runtime_db_path, &sector_csv, "512800.SH");

    let server = spawn_http_route_server(vec![
        (
            "/financials",
            "HTTP/1.1 500 Internal Server Error",
            r#"{"error":"eastmoney financial upstream failed"}"#,
            "application/json",
        ),
        (
            "/announcements",
            "HTTP/1.1 200 OK",
            r#"{"data":{"list":[]}}"#,
            "application/json",
        ),
        (
            "/sina-financials",
            "HTTP/1.1 200 OK",
            &build_sina_financial_guideline_real_chinese_fixture(),
            "text/html; charset=utf-8",
        ),
        (
            "/sina-announcements",
            "HTTP/1.1 200 OK",
            &build_sina_all_bulletin_fixture(),
            "text/html; charset=utf-8",
        ),
    ]);

    let request = json!({
        "tool": "security_analysis_fullstack",
        "args": {
            "symbol": "601998.SH",
            "market_symbol": "510300.SH",
            "sector_symbol": "512800.SH",
            "disclosure_limit": 3
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
            (
                "EXCEL_SKILL_SINA_FINANCIAL_URL_BASE",
                format!("{server}/sina-financials"),
            ),
            (
                "EXCEL_SKILL_SINA_ANNOUNCEMENT_URL_BASE",
                format!("{server}/sina-announcements"),
            ),
        ],
    );

    // 2026-04-02 CST: 这里先锁定“新浪财报页返回真实中文标签时也必须被识别”的回归，原因是线上真实页面可访问但当前实现会把财报误判为空；
    // 目的：先用红测复现 `fundamental_context` 被错误降级成 unavailable 的 bug，再最小修复中文标签解析。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["fundamental_context"]["status"], "available");
    assert_eq!(
        output["data"]["fundamental_context"]["latest_report_period"],
        "2025-12-31"
    );
    assert_eq!(
        output["data"]["fundamental_context"]["report_metrics"]["net_profit_yoy_pct"],
        2.9107
    );
    assert_eq!(
        output["data"]["fundamental_context"]["report_metrics"]["roe_pct"],
        8.52
    );
}

#[test]
fn security_analysis_fullstack_recognizes_mojibake_sina_financial_labels_from_live_page() {
    let runtime_db_path =
        create_test_runtime_db("security_analysis_fullstack_sina_mojibake_labels");

    let stock_csv = create_stock_history_csv(
        "security_analysis_fullstack_sina_mojibake_labels",
        "stock.csv",
        &build_range_bound_rows(220, 35.8, 38.0),
    );
    let market_csv = create_stock_history_csv(
        "security_analysis_fullstack_sina_mojibake_labels",
        "market.csv",
        &build_confirmed_breakdown_rows(220, 4.9),
    );
    let sector_csv = create_stock_history_csv(
        "security_analysis_fullstack_sina_mojibake_labels",
        "sector.csv",
        &build_choppy_history_rows(220, 1.02),
    );
    import_history_csv(&runtime_db_path, &stock_csv, "601998.SH");
    import_history_csv(&runtime_db_path, &market_csv, "510300.SH");
    import_history_csv(&runtime_db_path, &sector_csv, "512800.SH");

    let server = spawn_http_route_server(vec![
        (
            "/financials",
            "HTTP/1.1 500 Internal Server Error",
            r#"{"error":"eastmoney financial upstream failed"}"#,
            "application/json",
        ),
        (
            "/announcements",
            "HTTP/1.1 200 OK",
            r#"{"data":{"list":[]}}"#,
            "application/json",
        ),
        (
            "/sina-financials",
            "HTTP/1.1 200 OK",
            &build_sina_financial_guideline_live_mojibake_fixture(),
            "text/html; charset=utf-8",
        ),
        (
            "/sina-announcements",
            "HTTP/1.1 200 OK",
            &build_sina_all_bulletin_fixture(),
            "text/html; charset=utf-8",
        ),
    ]);

    let request = json!({
        "tool": "security_analysis_fullstack",
        "args": {
            "symbol": "601998.SH",
            "market_symbol": "510300.SH",
            "sector_symbol": "512800.SH",
            "disclosure_limit": 3
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
            (
                "EXCEL_SKILL_SINA_FINANCIAL_URL_BASE",
                format!("{server}/sina-financials"),
            ),
            (
                "EXCEL_SKILL_SINA_ANNOUNCEMENT_URL_BASE",
                format!("{server}/sina-announcements"),
            ),
        ],
    );

    // 2026-04-02 CST: 这里锁定“线上新浪页经当前链路进入后出现乱码标签时仍要识别关键财报项”的回归，原因是用户现场实际跑出来的就是这类页面；
    // 目的：把修复目标从“理想中文页面”收口到“真实线上可见页面”，避免修完测试绿了但现场还是拿不到财报。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["fundamental_context"]["status"], "available");
    assert_eq!(
        output["data"]["fundamental_context"]["latest_report_period"],
        "2025-12-31"
    );
    assert_eq!(
        output["data"]["fundamental_context"]["report_metrics"]["net_profit_yoy_pct"],
        2.9107
    );
    assert_eq!(
        output["data"]["fundamental_context"]["report_metrics"]["roe_pct"],
        8.52
    );
}

#[test]
fn security_analysis_fullstack_uses_third_level_fallback_after_official_sources_fail() {
    let runtime_db_path =
        create_test_runtime_db("security_analysis_fullstack_third_level_fallback");

    let stock_csv = create_stock_history_csv(
        "security_analysis_fullstack_third_level_fallback",
        "stock.csv",
        &build_range_bound_rows(220, 35.8, 38.0),
    );
    let market_csv = create_stock_history_csv(
        "security_analysis_fullstack_third_level_fallback",
        "market.csv",
        &build_confirmed_breakdown_rows(220, 4.9),
    );
    let sector_csv = create_stock_history_csv(
        "security_analysis_fullstack_third_level_fallback",
        "sector.csv",
        &build_choppy_history_rows(220, 1.02),
    );
    import_history_csv(&runtime_db_path, &stock_csv, "002352.SZ");
    import_history_csv(&runtime_db_path, &market_csv, "510300.SH");
    import_history_csv(&runtime_db_path, &sector_csv, "516530.SH");

    let server = spawn_http_route_server(vec![
        (
            "/financials",
            "HTTP/1.1 500 Internal Server Error",
            r#"{"error":"eastmoney financial upstream failed"}"#,
            "application/json",
        ),
        (
            "/announcements",
            "HTTP/1.1 500 Internal Server Error",
            r#"{"error":"eastmoney announcement upstream failed"}"#,
            "application/json",
        ),
        (
            "/official-financials",
            "HTTP/1.1 500 Internal Server Error",
            r#"{"error":"official financial upstream failed"}"#,
            "application/json",
        ),
        (
            "/official-announcements",
            "HTTP/1.1 500 Internal Server Error",
            r#"{"error":"official announcement upstream failed"}"#,
            "application/json",
        ),
        (
            "/sina-financials",
            "HTTP/1.1 200 OK",
            &build_sina_financial_guideline_fixture(),
            "text/html; charset=utf-8",
        ),
        (
            "/sina-announcements",
            "HTTP/1.1 200 OK",
            &build_sina_all_bulletin_fixture(),
            "text/html; charset=utf-8",
        ),
    ]);

    let request = json!({
        "tool": "security_analysis_fullstack",
        "args": {
            "symbol": "002352.SZ",
            "market_symbol": "510300.SH",
            "sector_symbol": "516530.SH",
            "disclosure_limit": 3
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
            (
                "EXCEL_SKILL_OFFICIAL_FINANCIAL_URL_BASE",
                format!("{server}/official-financials"),
            ),
            (
                "EXCEL_SKILL_OFFICIAL_ANNOUNCEMENT_URL_BASE",
                format!("{server}/official-announcements"),
            ),
            (
                "EXCEL_SKILL_SINA_FINANCIAL_URL_BASE",
                format!("{server}/sina-financials"),
            ),
            (
                "EXCEL_SKILL_SINA_ANNOUNCEMENT_URL_BASE",
                format!("{server}/sina-announcements"),
            ),
        ],
    );

    // 2026-04-02 CST: 这里再锁“三层 provider 优先级”，原因是 A3 方案要求主源、官方备源、通用公开备源都能串起来；
    // 目的：避免后续实现只做到两层降级，导致官方源故障时仍然直接退化到 technical_only。
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["fundamental_context"]["source"],
        "sina_financial_guideline"
    );
    assert_eq!(
        output["data"]["disclosure_context"]["source"],
        "sina_announcements"
    );
    assert_ne!(
        output["data"]["integrated_conclusion"]["stance"],
        "technical_only"
    );
}

// 2026-04-01 CST: 这里复用股票历史导入助手，原因是新总 Tool 的技术主链仍建立在统一 stock_history_store 之上；
// 目的：确保 fullstack 回归验证的是正式入库后的真实调用链，而不是测试内存态样本。
fn import_history_csv(runtime_db_path: &Path, csv_path: &Path, symbol: &str) {
    let request = json!({
        "tool": "import_stock_price_history",
        "args": {
            "csv_path": csv_path.to_string_lossy(),
            "symbol": symbol,
            "source": "security_analysis_fullstack_fixture"
        }
    });

    let output = run_cli_with_json_runtime_and_envs(
        &request.to_string(),
        &runtime_db_path.to_path_buf(),
        &[],
    );
    assert_eq!(output["status"], "ok");
}

// 2026-04-02 CST: 这里补新浪财务指标页夹具，原因是当前真实备源首版决定先接入新浪公开页面；
// 目的：用最小 HTML 片段稳定重放“报告日期 + 关键指标首列”的解析合同，避免测试依赖真实站点结构每天波动。
fn build_sina_financial_guideline_fixture() -> String {
    r#"
    <html>
      <body>
        <table id="BalanceSheetNewTable0">
          <tbody>
            <tr><td>报告日期</td><td>2025-12-31</td><td>2025-09-30</td><td>2025-06-30</td></tr>
            <tr><td>净利润(元)</td><td>69819000000</td><td>52956000000</td><td>36335000000</td></tr>
            <tr><td>净资产收益率(%)</td><td>8.52</td><td>6.48</td><td>4.48</td></tr>
            <tr><td>净利润增长率(%)</td><td>2.9107</td><td>3.3047</td><td>3.3537</td></tr>
            <tr><td>主营业务收入增长率(%)</td><td>1.1200</td><td>1.0500</td><td>0.9800</td></tr>
          </tbody>
        </table>
      </body>
    </html>
    "#
    .to_string()
}

// 2026-04-02 CST: 这里补真实中文标签的新浪财报夹具，原因是用户现场发现真实页面可访问，但当前实现对中文标签识别失败；
// 目的：让回归测试直接覆盖线上页面的真实字段命名，避免只对历史乱码夹具通过而漏掉真实站点行为。
fn build_sina_financial_guideline_real_chinese_fixture() -> String {
    r#"
    <html>
      <body>
        <table id="BalanceSheetNewTable0">
          <tbody>
            <tr><td>报告日期</td><td>2025-12-31</td><td>2025-09-30</td><td>2025-06-30</td></tr>
            <tr><td>净利润(元)</td><td>69819000000</td><td>52956000000</td><td>36335000000</td></tr>
            <tr><td>净资产收益率(%)</td><td>8.52</td><td>6.48</td><td>4.48</td></tr>
            <tr><td>净利润增长率(%)</td><td>2.9107</td><td>3.3047</td><td>3.3537</td></tr>
            <tr><td>主营业务收入增长率(%)</td><td>1.1200</td><td>1.0500</td><td>0.9800</td></tr>
          </tbody>
        </table>
      </body>
    </html>
    "#
    .to_string()
}

// 2026-04-02 CST: 这里补线上实际抓回来的新浪财报乱码标签夹具，原因是用户现场看到的页面文本已经不是纯中文，而是混杂乱码与稳定 typecode；
// 目的：让测试覆盖“文本标签失真但结构锚点还在”的真实场景，驱动实现同时利用标签和 typecode 识别关键指标。
fn build_sina_financial_guideline_live_mojibake_fixture() -> String {
    r#"
    <html>
      <body>
        <table id="BalanceSheetNewTable0">
          <tbody>
            <tr><td width='200px'><strong>��������</strong></td><td>2025-12-31</td><td>2025-09-30</td><td>2025-06-30</td></tr>
            <tr><td width='200px' style='padding-left:30px;'><a target='_blank' href='/corp/view/vFD_FinancialGuideLineHistory.php?stockid=601998&typecode=financialratios59'>���ʲ�������(%)</a></td><td>8.52</td><td>6.48</td><td>4.48</td></tr>
            <tr><td width='200px' style='padding-left:30px;'><a target='_blank' href='/corp/view/vFD_FinancialGuideLineHistory.php?stockid=601998&typecode=financialratios65'>�۳��Ǿ����������ľ�����(Ԫ)</a></td><td>69819000000</td><td>52956000000</td><td>36335000000</td></tr>
            <tr><td width='200px' style='padding-left:30px;'><a target='_blank' href='/corp/view/vFD_FinancialGuideLineHistory.php?stockid=601998&typecode=financialratios44'>����������(%)</a></td><td>2.9107</td><td>3.3047</td><td>3.3537</td></tr>
            <tr><td width='200px' style='padding-left:30px;'><a target='_blank' href='/corp/view/vFD_FinancialGuideLineHistory.php?stockid=601998&typecode=financialratios43'>��Ӫҵ��������(%)</a></td><td>--</td><td>--</td><td>--</td></tr>
          </tbody>
        </table>
      </body>
    </html>
    "#
    .to_string()
}

// 2026-04-02 CST: 这里补新浪公司公告页夹具，原因是公告备源首版要覆盖日期、标题、详情链接三个最关键字段；
// 目的：锁住 datelist 快速通道的解析逻辑，确保后续即使页面其余导航块变化也不影响公告摘要链路。
fn build_sina_all_bulletin_fixture() -> String {
    r#"
    <html>
      <body>
        <div class="datelist">
          <ul>
            2026-03-21&nbsp;<a target='_blank' href='/corp/view/vCB_AllBulletinDetail.php?stockid=002352&id=12008304'>示例公司：2025年度利润分配方案公告</a><br>
            2026-03-21&nbsp;<a target='_blank' href='/corp/view/vCB_AllBulletinDetail.php?stockid=002352&id=12008299'>示例公司：2025年年度报告</a><br>
            2026-03-20&nbsp;<a target='_blank' href='/corp/view/vCB_AllBulletinDetail.php?stockid=002352&id=12003384'>示例公司：董事会会议决议公告</a><br>
            2026-03-18&nbsp;<a target='_blank' href='/corp/view/vCB_AllBulletinDetail.php?stockid=002352&id=11999230'>示例公司：关于回购股份进展的公告</a><br>
          </ul>
        </div>
      </body>
    </html>
    "#
    .to_string()
}

// 2026-04-01 CST: 这里补区间震荡样本，原因是顺丰当前真实场景更接近“区间上沿待选边”而不是单边突破；
// 目的：让 fullstack Tool 首版直接覆盖产品最常见的“技术偏观察、信息面再辅助决策”的使用场景。
fn build_range_bound_rows(day_count: usize, support: f64, resistance: f64) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let span = resistance - support;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let phase = offset % 12;
        let anchor = match phase {
            0 => support + span * 0.24,
            1 => support + span * 0.37,
            2 => support + span * 0.44,
            3 => support + span * 0.58,
            4 => support + span * 0.66,
            5 => support + span * 0.73,
            6 => support + span * 0.62,
            7 => support + span * 0.55,
            8 => support + span * 0.68,
            9 => support + span * 0.81,
            10 => support + span * 0.93,
            _ => support + span * 0.88,
        };
        let open = anchor - 0.18;
        let close = anchor + if phase >= 9 { 0.12 } else { 0.05 };
        let high = resistance.min(close + 0.16);
        let low = support.max(open - 0.18);
        let volume = 180_000 + (offset % 7) as i64 * 12_000;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{close:.2},{close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
    }

    rows
}

// 2026-04-01 CST: 这里复用弱震荡样本，原因是板块代理首版只需要提供一个“中性偏等待”的行业环境；
// 目的：避免测试把行业层误写成强趋势，反而掩盖 fullstack Tool 的综合判断逻辑。
fn build_choppy_history_rows(day_count: usize, base: f64) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let wave = match offset % 6 {
            0 => -0.008,
            1 => 0.007,
            2 => -0.006,
            3 => 0.006,
            4 => -0.007,
            _ => 0.008,
        };
        let close = base + wave;
        let open = close - 0.003;
        let high = close + 0.012;
        let low = close - 0.012;
        let volume = 4_000_000 + (offset % 5) as i64 * 180_000;
        rows.push(format!(
            "{},{open:.3},{high:.3},{low:.3},{close:.3},{close:.3},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
    }

    rows
}

// 2026-04-01 CST: 这里复用稳定下行样本，原因是市场代理需要给新总 Tool 一个明确偏弱的大盘背景；
// 目的：验证综合结论确实在“技术偏观察、市场偏弱”条件下输出 watchful_positive，而不是误判成顺风。
fn build_confirmed_breakdown_rows(day_count: usize, start_close: f64) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close = start_close;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let (next_close, volume): (f64, i64) = if offset < day_count - 20 {
            (close - 0.007, 8_600_000 + offset as i64 * 18_000)
        } else {
            let phase = offset - (day_count - 20);
            match phase % 4 {
                0 => (close - 0.013, 16_600_000 + phase as i64 * 40_000),
                1 => (close + 0.002, 4_100_000),
                2 => (close - 0.011, 15_200_000 + phase as i64 * 36_000),
                _ => (close - 0.004, 12_000_000),
            }
        };

        let open = close;
        let high = next_close.max(open) + 0.008;
        let low = next_close.min(open) - 0.012;
        rows.push(format!(
            "{},{open:.3},{high:.3},{low:.3},{next_close:.3},{next_close:.3},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}
