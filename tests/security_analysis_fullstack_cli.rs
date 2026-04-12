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
            "/capital-flow",
            "HTTP/1.1 200 OK",
            r#"{
                "data":{
                    "symbol":"002352.SZ",
                    "main_net_inflow":12500000.0,
                    "super_order_net_inflow":6200000.0,
                    "headline":"主力资金当日净流入，超大单同步净流入"
                }
            }"#,
            "application/json",
        ),
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
                "EXCEL_SKILL_EASTMONEY_CAPITAL_FLOW_URL_BASE",
                format!("{server}/capital-flow"),
            ),
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

    // 2026-04-01 CST: 这里先锁“技术面 + 财报 + 公告”聚合成功主路径，原因是这是新总 Tool 首版对产品最关键的交付合同；
    // 目的：确保上层调用方只打一枪就能拿到完整证券分析骨架，而不是继续手工拼多个 Tool 结果。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["symbol"], "002352.SZ");
    assert_eq!(
        output["data"]["technical_context"]["capital_flow_context"]["status"],
        "available"
    );
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
            "/capital-flow",
            "HTTP/1.1 200 OK",
            r#"{
                "data":{
                    "symbol":"002352.SZ",
                    "main_net_inflow":12500000.0,
                    "super_order_net_inflow":6200000.0,
                    "headline":"主力资金当日净流入，超大单同步净流入"
                }
            }"#,
            "application/json",
        ),
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
            ("EXCEL_SKILL_EASTMONEY_DAILY_LIMIT", "0".to_string()),
            (
                "EXCEL_SKILL_EASTMONEY_CAPITAL_FLOW_URL_BASE",
                format!("{server}/capital-flow"),
            ),
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

    // 2026-04-01 CST: 这里补“信息面源失败但主 Tool 不崩”的降级合同，原因是第三方免费源天然存在限流和抖动；
    // 目的：让产品在外部源异常时仍能返回技术主结论，并显式告诉上层哪些信息面维度缺失。
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["technical_context"]["capital_flow_context"]["status"],
        "budget_exhausted"
    );
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
fn security_analysis_fullstack_synthesizes_etf_information_from_governed_proxy_history() {
    let runtime_db_path = create_test_runtime_db("security_analysis_fullstack_etf_info");
    let external_proxy_db_path = runtime_db_path
        .parent()
        .expect("runtime db should have parent")
        .join("security_external_proxy.db");

    let etf_csv = create_stock_history_csv(
        "security_analysis_fullstack_etf_info",
        "gold_etf.csv",
        &build_confirmed_breakout_rows(260, 101.0),
    );
    let market_csv = create_stock_history_csv(
        "security_analysis_fullstack_etf_info",
        "market.csv",
        &build_confirmed_breakout_rows(260, 3200.0),
    );
    let sector_csv = create_stock_history_csv(
        "security_analysis_fullstack_etf_info",
        "sector.csv",
        &build_confirmed_breakout_rows(260, 99.0),
    );
    import_history_csv(&runtime_db_path, &etf_csv, "518880.SH");
    import_history_csv(&runtime_db_path, &market_csv, "510300.SH");
    import_history_csv(&runtime_db_path, &sector_csv, "518800.SH");

    let backfill_request = json!({
        "tool": "security_external_proxy_backfill",
        "args": {
            "batch_id": "fullstack-etf-info",
            "created_at": "2026-04-13T10:00:00+08:00",
            "records": [{
                "symbol": "518880.SH",
                "as_of_date": "2025-08-08",
                "instrument_subscope": "gold_etf",
                "external_proxy_inputs": {
                    "gold_spot_proxy_status": "manual_bound",
                    "gold_spot_proxy_return_5d": 0.021019,
                    "usd_index_proxy_status": "manual_bound",
                    "usd_index_proxy_return_5d": -0.003841,
                    "real_rate_proxy_status": "manual_bound",
                    "real_rate_proxy_delta_bp_5d": -2.0
                }
            }]
        }
    });
    let backfill_output = run_cli_with_json_runtime_and_envs(
        &backfill_request.to_string(),
        &runtime_db_path,
        &[(
            "EXCEL_SKILL_EXTERNAL_PROXY_DB",
            external_proxy_db_path.to_string_lossy().to_string(),
        )],
    );
    assert_eq!(backfill_output["status"], "ok");

    let server = spawn_http_route_server(vec![
        (
            "/financials",
            "HTTP/1.1 406 Not Acceptable",
            "<html><body>financials unavailable for ETF fixture</body></html>",
            "text/html",
        ),
        (
            "/announcements",
            "HTTP/1.1 200 OK",
            r#"{"data":{"list":[]}}"#,
            "application/json",
        ),
    ]);

    let request = json!({
        "tool": "security_analysis_fullstack",
        "args": {
            "symbol": "518880.SH",
            "market_symbol": "510300.SH",
            "sector_symbol": "518800.SH",
            "market_profile": "a_share_core",
            "sector_profile": "gold_etf_peer",
            "as_of_date": "2025-08-08"
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
                "EXCEL_SKILL_EXTERNAL_PROXY_DB",
                external_proxy_db_path.to_string_lossy().to_string(),
            ),
        ],
    );

    // 2026-04-13 UTC+08: Add an ETF information-layer red test here, because the
    // user explicitly required the stack to stop treating ETF symbols as permanently
    // missing stock-style information once governed ETF proxy history is available.
    // Purpose: force fullstack to synthesize ETF-native evidence so later committee
    // and chair layers can move beyond the automatic `technical_only` downgrade.
    assert_eq!(
        output["status"], "ok",
        "unexpected ETF fullstack output: {output}"
    );
    assert_eq!(output["data"]["fundamental_context"]["status"], "available");
    assert_eq!(
        output["data"]["fundamental_context"]["source"],
        "governed_etf_proxy_information"
    );
    assert_eq!(output["data"]["disclosure_context"]["status"], "available");
    assert_eq!(
        output["data"]["disclosure_context"]["source"],
        "governed_etf_proxy_information"
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
