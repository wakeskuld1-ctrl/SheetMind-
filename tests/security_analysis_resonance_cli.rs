mod common;

use chrono::{Duration, NaiveDate};
use excel_skill::ops::stock::security_committee_vote::{
    SecurityCommitteeVoteRequest, SecurityCommitteeVoteResult, security_committee_vote,
};
use excel_skill::ops::stock::security_decision_briefing::{
    CommitteePayload, SecurityDecisionBriefingRequest,
};
use rusqlite::Connection;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::common::{
    create_test_runtime_db, run_cli_with_json, run_cli_with_json_runtime_and_envs,
};

// 2026-04-02 CST: 这里为直接调用 Rust 层证券分析函数补一个进程级环境锁，原因是这类函数会直接读取 `EXCEL_SKILL_RUNTIME_DB`
// 和多组 HTTP base 环境变量；目的：避免测试并发时互相覆盖运行时目录或本地假服务地址，造成偶发串扰。
fn security_briefing_env_lock() -> &'static Mutex<()> {
    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    ENV_LOCK.get_or_init(|| Mutex::new(()))
}

// 2026-04-02 CST: 这里补一个环境变量恢复守卫，原因是 briefing assembler 直调测试需要临时覆盖 runtime 与财报/公告 URL，
// 目的：让测试结束后无论成功还是失败都恢复进程环境，避免影响同文件后续其他用例。
struct EnvVarGuard {
    snapshots: Vec<(String, Option<String>)>,
}

impl EnvVarGuard {
    fn set_all(envs: &[(&str, String)]) -> Self {
        let mut snapshots = Vec::new();
        for (key, value) in envs {
            snapshots.push(((*key).to_string(), std::env::var(key).ok()));
            // 2026-04-02 CST: 这里显式用 unsafe 包住进程级环境变量写入，原因是当前 Rust 版本把跨线程环境修改标为 unsafe；
            // 目的：结合上面的全局互斥锁，把风险收敛在受控测试辅助函数里，而不是散落到每个测试用例主体中。
            unsafe {
                std::env::set_var(key, value);
            }
        }
        Self { snapshots }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        for (key, previous_value) in self.snapshots.iter().rev() {
            match previous_value {
                Some(value) => unsafe {
                    std::env::set_var(key, value);
                },
                None => unsafe {
                    std::env::remove_var(key);
                },
            }
        }
    }
}

// 2026-04-02 CST: 这里补一个直调 briefing 函数的测试助手，原因是当前阶段要先做 assembler，再做 dispatcher，
// 目的：让我们可以在不提前接 catalog/dispatcher 的前提下，先验证 briefing 是否完整复用了 fullstack/resonance 的事实层输出。
fn run_security_decision_briefing_direct(
    runtime_db_path: &Path,
    request: &SecurityDecisionBriefingRequest,
    envs: &[(&str, String)],
) -> Value {
    let _lock = security_briefing_env_lock()
        .lock()
        .expect("security briefing env lock should be acquirable");
    let mut merged_envs = vec![(
        "EXCEL_SKILL_RUNTIME_DB",
        runtime_db_path.to_string_lossy().to_string(),
    )];
    merged_envs.extend(envs.iter().map(|(key, value)| (*key, value.clone())));
    let _guard = EnvVarGuard::set_all(&merged_envs);

    let output =
        excel_skill::ops::stock::security_decision_briefing::security_decision_briefing(request)
            .expect("security_decision_briefing should succeed under fixture envs");
    serde_json::to_value(output).expect("security_decision_briefing result should serialize")
}

// 2026-04-02 CST：这里新增共振平台 CLI 测试夹具目录助手，原因是方案 3 已经确定先做“平台底层 + Tool 主链”；
// 目的：先把共振平台的真实 `CSV -> SQLite -> register/append -> resonance analysis` 主链固定下来，避免后续只测内部函数而漏掉外层合同。
fn create_stock_history_csv(prefix: &str, file_name: &str, rows: &[String]) -> PathBuf {
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let fixture_dir = PathBuf::from("tests")
        .join("runtime_fixtures")
        .join("security_analysis_resonance")
        .join(format!("{prefix}_{unique_suffix}"));
    fs::create_dir_all(&fixture_dir).expect("security resonance fixture dir should exist");

    let csv_path = fixture_dir.join(file_name);
    fs::write(&csv_path, rows.join("\n")).expect("security resonance csv should be written");
    csv_path
}

// 2026-04-02 CST：这里复用股票历史导入测试助手，原因是共振平台第一版仍然要建立在现有股票 SQLite 主链之上；
// 目的：确保我们验证的是正式工具链条，而不是手工拼接的内存数据。
fn import_history_csv(runtime_db_path: &Path, csv_path: &Path, symbol: &str) {
    let request = json!({
        "tool": "import_stock_price_history",
        "args": {
            "csv_path": csv_path.to_string_lossy(),
            "symbol": symbol,
            "source": "security_analysis_resonance_fixture"
        }
    });

    let output = crate::common::run_cli_with_json_and_runtime(
        &request.to_string(),
        &runtime_db_path.to_path_buf(),
    );
    assert_eq!(output["status"], "ok");
}

// 2026-04-02 CST：这里补本地 HTTP 假服务，原因是 `security_analysis_resonance` 会复用 fullstack 主链；
// 目的：把财报和公告响应固定成本地夹具，避免测试结果受真实外网波动影响。
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
        for _ in 0..route_map.len() + 8 {
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

// 2026-04-02 CST：这里构造偏多但带少量回撤的石油股样本，原因是共振平台第一版需要一个能稳定观察“个股-因子同向”的真实风格数据；
// 目的：让后续断言既能覆盖趋势延续，也不会因为过于单边而把所有指标都推到极端值。
fn build_oil_stock_rows(day_count: usize, start_price: f64) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close = start_price;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let drift = if offset < day_count - 25 {
            0.32 + (offset % 5) as f64 * 0.03
        } else if offset % 4 == 0 {
            -0.18
        } else {
            0.46
        };
        let next_close = (close + drift).max(3.0);
        let open = close;
        let high = next_close.max(open) + 0.55;
        let low = next_close.min(open) - 0.42;
        let volume = 1_200_000 + offset as i64 * 4_500 + (offset % 7) as i64 * 30_000;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{next_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-04-02 CST：这里构造温和顺风的大盘样本，原因是共振平台第一版需要一个不会喧宾夺主、但能给 fullstack 提供环境背景的大盘代理；
// 目的：让测试重点停留在“行业/商品/事件共振”，而不是被大盘极端波动盖住。
fn build_supportive_market_rows(day_count: usize, start_price: f64) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close = start_price;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let drift = if offset % 6 == 0 { -0.05 } else { 0.11 };
        let next_close = close + drift;
        let open = close;
        let high = next_close.max(open) + 0.18;
        let low = next_close.min(open) - 0.16;
        let volume = 2_100_000 + offset as i64 * 2_500;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{next_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-04-02 CST：这里构造石油板块顺风样本，原因是用户明确要求以后分析像中国石油时要显式带出板块共振；
// 目的：先把“个股和板块同向”的正向共振合同钉住，避免后续只剩商品因子没有行业基本盘。
fn build_oil_sector_rows(day_count: usize, start_price: f64) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close = start_price;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let drift = if offset < day_count - 20 {
            0.22 + (offset % 4) as f64 * 0.02
        } else if offset % 5 == 0 {
            -0.12
        } else {
            0.34
        };
        let next_close = close + drift;
        let open = close;
        let high = next_close.max(open) + 0.27;
        let low = next_close.min(open) - 0.21;
        let volume = 980_000 + offset as i64 * 3_800;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{next_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-04-02 CST：这里生成与石油股高度同向的商品因子序列，原因是平台第一版必须能把“布伦特原油”这类可计算因子拉出来；
// 目的：验证数据库落库后的滚动评估能把真正强相关的因子识别到 `top_positive_resonances`。
fn build_positive_factor_points(day_count: usize, start_value: f64) -> Vec<Value> {
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut value = start_value;

    (0..day_count)
        .map(|offset| {
            let trade_date = start_date + Duration::days(offset as i64);
            let drift = if offset < day_count - 18 {
                0.45 + (offset % 3) as f64 * 0.04
            } else if offset % 4 == 0 {
                -0.28
            } else {
                0.62
            };
            value += drift;
            json!({
                "trade_date": trade_date.format("%Y-%m-%d").to_string(),
                "value": value
            })
        })
        .collect()
}

// 2026-04-02 CST：这里生成近期与石油股明显背离的伪正向因子，原因是平台不能只会找利好共振，还要能把背离风险暴露出来；
// 目的：锁住 `top_negative_resonances` 的外层合同，避免后续只输出正向驱动而看不到风险侧。
fn build_negative_factor_points(day_count: usize, start_value: f64) -> Vec<Value> {
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut value = start_value;

    (0..day_count)
        .map(|offset| {
            let trade_date = start_date + Duration::days(offset as i64);
            let drift = if offset < day_count - 20 {
                0.18
            } else {
                -0.55 - (offset % 3) as f64 * 0.06
            };
            value += drift;
            json!({
                "trade_date": trade_date.format("%Y-%m-%d").to_string(),
                "value": value
            })
        })
        .collect()
}

// 2026-04-02 CST：这里补银行股样本，原因是方案B已明确要求把银行行业完整宏观共振作为正式回归对象；
// 目的：让 601998.SH 的测试夹具拥有“温和上行但伴随阶段扰动”的收益结构，便于观察利率、地产、消费、贷款、PMI 的差异化共振。
fn build_bank_stock_rows(day_count: usize, start_price: f64) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close = start_price;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let drift = if offset < day_count - 40 {
            0.025 + (offset % 5) as f64 * 0.006
        } else if offset % 6 == 0 {
            -0.030
        } else {
            0.042
        };
        let next_close = (close + drift).max(3.0);
        let open = close;
        let high = next_close.max(open) + 0.08;
        let low = next_close.min(open) - 0.07;
        let volume = 6_200_000 + offset as i64 * 3_000 + (offset % 9) as i64 * 28_000;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{next_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-04-02 CST: 这里补一个通用代理行情夹具生成器，原因是银行宏观共振底座红测需要同时造多条代理序列，
// 目的：让银行板块、国债、信用债、地产、消费和大盘代理都能沿统一格式快速导入 stock history 主链。
fn build_proxy_rows(
    day_count: usize,
    start_price: f64,
    up_drift: f64,
    down_drift: f64,
    down_every: usize,
    volume_base: i64,
) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close = start_price;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let drift = if down_every > 0 && offset % down_every == 0 {
            down_drift
        } else {
            up_drift
        };
        let next_close = (close + drift).max(0.6);
        let open = close;
        let high = next_close.max(open) + 0.08;
        let low = next_close.min(open) - 0.07;
        let volume = volume_base + offset as i64 * 1_800 + (offset % 5) as i64 * 9_000;
        rows.push(format!(
            "{},{open:.3},{high:.3},{low:.3},{next_close:.3},{next_close:.3},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

#[test]
fn tool_catalog_includes_sync_template_resonance_factors() {
    let output = run_cli_with_json("");

    // 2026-04-02 CST: 这里先锁模板级共振因子同步 Tool 的目录可发现性，原因是平台底座若只落函数而不进 catalog，
    // 目的：后续 Agent/Skill 仍无法沿正式工具主链发现并使用这条银行宏观共振补数入口。
    assert!(
        output["data"]["tool_catalog"]
            .as_array()
            .expect("tool catalog should be an array")
            .iter()
            .any(|tool| tool == "sync_template_resonance_factors")
    );
}

#[test]
fn sync_template_resonance_factors_bank_writes_macro_proxy_series() {
    let runtime_db_path = create_test_runtime_db("sync_template_resonance_factors_bank");

    let bank_sector_csv = create_stock_history_csv(
        "sync_template_resonance_factors_bank",
        "bank_sector.csv",
        &build_proxy_rows(180, 1.020, 0.006, -0.002, 6, 620_000),
    );
    let gov_bond_csv = create_stock_history_csv(
        "sync_template_resonance_factors_bank",
        "gov_bond.csv",
        &build_proxy_rows(180, 1.000, 0.003, -0.001, 4, 540_000),
    );
    let credit_bond_csv = create_stock_history_csv(
        "sync_template_resonance_factors_bank",
        "credit_bond.csv",
        &build_proxy_rows(180, 1.050, 0.002, -0.002, 5, 500_000),
    );
    let real_estate_csv = create_stock_history_csv(
        "sync_template_resonance_factors_bank",
        "real_estate.csv",
        &build_proxy_rows(180, 0.980, 0.007, -0.003, 7, 710_000),
    );
    let consumption_csv = create_stock_history_csv(
        "sync_template_resonance_factors_bank",
        "consumption.csv",
        &build_proxy_rows(180, 1.120, 0.005, -0.002, 8, 690_000),
    );
    let market_csv = create_stock_history_csv(
        "sync_template_resonance_factors_bank",
        "market.csv",
        &build_proxy_rows(180, 4.100, 0.018, -0.006, 6, 1_600_000),
    );

    import_history_csv(&runtime_db_path, &bank_sector_csv, "512800.SH");
    import_history_csv(&runtime_db_path, &gov_bond_csv, "511260.SH");
    import_history_csv(&runtime_db_path, &credit_bond_csv, "511190.SH");
    import_history_csv(&runtime_db_path, &real_estate_csv, "512200.SH");
    import_history_csv(&runtime_db_path, &consumption_csv, "159928.SZ");
    import_history_csv(&runtime_db_path, &market_csv, "510300.SH");

    let bootstrap_request = json!({
        "tool": "bootstrap_resonance_template_factors",
        "args": {
            "market_regime": "a_share"
        }
    });
    let bootstrap_output = crate::common::run_cli_with_json_and_runtime(
        &bootstrap_request.to_string(),
        &runtime_db_path,
    );
    assert_eq!(bootstrap_output["status"], "ok");

    let request = json!({
        "tool": "sync_template_resonance_factors",
        "args": {
            "market_regime": "a_share",
            "template_key": "bank",
            "start_date": "2025-01-01",
            "end_date": "2025-06-29",
            "skip_price_sync": true
        }
    });
    let output =
        crate::common::run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);

    // 2026-04-02 CST: 这里先锁银行模板宏观代理序列写库合同，原因是方案C的第一步不是临时分析，
    // 目的：而是要把银行宏观因子统一变成可复用、可回放、可评估的正式平台资产。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["template_key"], "bank");
    assert!(
        output["data"]["synced_factor_count"]
            .as_u64()
            .expect("synced_factor_count should be numeric")
            >= 6
    );

    let resonance_db_path = runtime_db_path
        .parent()
        .expect("runtime db should have parent")
        .join("security_resonance.db");
    let connection = Connection::open(resonance_db_path).expect("resonance db should open");

    for factor_key in [
        "bank_sector_proxy",
        "cn_bond_10y_yield",
        "credit_risk_spread",
        "real_estate_proxy",
        "resident_consumption_proxy",
        "loan_growth_proxy",
        "pmi_proxy",
    ] {
        let row_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM resonance_factor_series WHERE factor_key = ?1",
                [factor_key],
                |row| row.get(0),
            )
            .expect("factor series count query should succeed");
        assert!(
            row_count >= 120,
            "factor `{factor_key}` should have persisted enough daily points"
        );
    }
}

// 2026-04-02 CST：这里为银行共振红测补“顺周期正向宏观代理”序列，原因是地产、消费、贷款、PMI 对银行更常表现为同向景气共振；
// 目的：让新接入的宏观候选因子在 `top_positive_resonances` 中有机会浮出来，而不是永远只剩板块代理。
fn build_bank_positive_macro_points(day_count: usize, start_value: f64) -> Vec<Value> {
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut value = start_value;

    (0..day_count)
        .map(|offset| {
            let trade_date = start_date + Duration::days(offset as i64);
            let drift = if offset < day_count - 35 {
                0.18 + (offset % 4) as f64 * 0.03
            } else if offset % 6 == 0 {
                -0.10
            } else {
                0.26
            };
            value += drift;
            json!({
                "trade_date": trade_date.format("%Y-%m-%d").to_string(),
                "value": value
            })
        })
        .collect()
}

// 2026-04-02 CST：这里为银行共振红测补“逆向利率/信用”序列，原因是国债收益率和信用利差对银行股不应一律当成同向因子；
// 目的：让负相关但符合预期关系的候选因子也能进入正向共振结果，覆盖“预期关系 != 正相关”的评估路径。
fn build_bank_inverse_macro_points(day_count: usize, start_value: f64) -> Vec<Value> {
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut value = start_value;

    (0..day_count)
        .map(|offset| {
            let trade_date = start_date + Duration::days(offset as i64);
            let drift = if offset < day_count - 40 {
                -0.14 - (offset % 3) as f64 * 0.02
            } else if offset % 6 == 0 {
                0.09
            } else {
                -0.20
            };
            value += drift;
            json!({
                "trade_date": trade_date.format("%Y-%m-%d").to_string(),
                "value": value
            })
        })
        .collect()
}

// 2026-04-02 CST: 这里抽出 security_decision_briefing 的真实夹具构造助手，原因是后续交易执行层与 committee payload 红绿测试
// 都需要复用同一套 stock + market + sector + financials + resonance 的完整底稿；目的：避免测试重复手抄长夹具，同时保证各轮断言都基于同一事实样本。
fn build_security_decision_briefing_fixture_output(prefix: &str) -> Value {
    let runtime_db_path = create_test_runtime_db(prefix);

    let stock_csv = create_stock_history_csv(prefix, "stock.csv", &build_oil_stock_rows(220, 8.2));
    let market_csv = create_stock_history_csv(
        prefix,
        "market.csv",
        &build_supportive_market_rows(220, 4.1),
    );
    let sector_csv =
        create_stock_history_csv(prefix, "sector.csv", &build_oil_sector_rows(220, 1.7));
    import_history_csv(&runtime_db_path, &stock_csv, "601857.SH");
    import_history_csv(&runtime_db_path, &market_csv, "510300.SH");
    import_history_csv(&runtime_db_path, &sector_csv, "516570.SH");

    let server = spawn_http_route_server(vec![
        (
            "/financials",
            "HTTP/1.1 200 OK",
            r#"[
                {
                    "REPORT_DATE":"2025-12-31",
                    "NOTICE_DATE":"2026-03-28",
                    "TOTAL_OPERATE_INCOME":298000000000.0,
                    "YSTZ":6.82,
                    "PARENT_NETPROFIT":162000000000.0,
                    "SJLTZ":7.21,
                    "ROEJQ":11.4
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
                        {"notice_date":"2026-03-28","title":"2025年度利润分配预案公告","art_code":"AN202603281010101010","columns":[{"column_name":"公司公告"}]},
                        {"notice_date":"2026-03-15","title":"关于回购进展的公告","art_code":"AN202603151010101011","columns":[{"column_name":"公司公告"}]}
                    ]
                }
            }"#,
            "application/json",
        ),
    ]);

    let bootstrap_request = json!({
        "tool": "bootstrap_resonance_template_factors",
        "args": {
            "market_regime": "a_share"
        }
    });
    let bootstrap_output =
        run_cli_with_json_runtime_and_envs(&bootstrap_request.to_string(), &runtime_db_path, &[]);
    assert_eq!(bootstrap_output["status"], "ok");

    let register_positive_factor_request = json!({
        "tool": "register_resonance_factor",
        "args": {
            "factor_key": "brent_crude",
            "display_name": "布伦特原油",
            "market_regime": "a_share",
            "template_key": "oil_petrochemical",
            "factor_type": "price_series",
            "source_kind": "manual_fixture",
            "expected_relation": "positive",
            "notes": "2026-04-02 CST 测试夹具：用于 security_decision_briefing 技术层、执行层与 committee payload 整体验证。"
        }
    });
    let register_negative_factor_request = json!({
        "tool": "register_resonance_factor",
        "args": {
            "factor_key": "false_positive_proxy",
            "display_name": "伪正向成本因子",
            "market_regime": "a_share",
            "template_key": "oil_petrochemical",
            "factor_type": "price_series",
            "source_kind": "manual_fixture",
            "expected_relation": "positive",
            "notes": "2026-04-02 CST 测试夹具：用于 security_decision_briefing 暴露负向共振风险。"
        }
    });
    assert_eq!(
        run_cli_with_json_runtime_and_envs(
            &register_positive_factor_request.to_string(),
            &runtime_db_path,
            &[]
        )["status"],
        "ok"
    );
    assert_eq!(
        run_cli_with_json_runtime_and_envs(
            &register_negative_factor_request.to_string(),
            &runtime_db_path,
            &[]
        )["status"],
        "ok"
    );

    let append_positive_series_request = json!({
        "tool": "append_resonance_factor_series",
        "args": {
            "factor_key": "brent_crude",
            "source": "manual_fixture",
            "points": build_positive_factor_points(220, 68.0)
        }
    });
    let append_negative_series_request = json!({
        "tool": "append_resonance_factor_series",
        "args": {
            "factor_key": "false_positive_proxy",
            "source": "manual_fixture",
            "points": build_negative_factor_points(220, 41.0)
        }
    });
    assert_eq!(
        run_cli_with_json_runtime_and_envs(
            &append_positive_series_request.to_string(),
            &runtime_db_path,
            &[]
        )["status"],
        "ok"
    );
    assert_eq!(
        run_cli_with_json_runtime_and_envs(
            &append_negative_series_request.to_string(),
            &runtime_db_path,
            &[]
        )["status"],
        "ok"
    );

    let request = SecurityDecisionBriefingRequest {
        symbol: "601857.SH".to_string(),
        market_symbol: Some("510300.SH".to_string()),
        sector_symbol: Some("516570.SH".to_string()),
        market_regime: "a_share".to_string(),
        sector_template: "oil_petrochemical".to_string(),
        as_of_date: None,
        lookback_days: 180,
        factor_lookback_days: 120,
        disclosure_limit: 3,
    };
    run_security_decision_briefing_direct(
        &runtime_db_path,
        &request,
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
    )
}

// 2026-04-02 CST: 这里抽出银行历史研究夹具，原因是“银行板块共振 + MACD/RSRS 等核心技术相似”的红测
// 需要反复复用同一套 bank stock / bank sector / macro proxy / snapshot / outcome 底座；
// 目的：确保 analog study 与 briefing historical_digest 都基于正式 Tool 主链，而不是测试里手工伪造研究结果。
fn build_bank_signal_outcome_fixture(prefix: &str) -> (PathBuf, String) {
    let runtime_db_path = create_test_runtime_db(prefix);

    for (symbol, file_name, rows) in [
        (
            "601998.SH",
            "citic_bank.csv",
            build_bank_stock_rows(420, 7.10),
        ),
        (
            "600000.SH",
            "spd_bank.csv",
            build_bank_stock_rows(420, 8.35),
        ),
        ("601398.SH", "icbc.csv", build_bank_stock_rows(420, 6.45)),
        (
            "512800.SH",
            "bank_sector.csv",
            build_bank_stock_rows(420, 1.05),
        ),
        (
            "511260.SH",
            "gov_bond.csv",
            build_proxy_rows(420, 1.000, 0.003, -0.001, 4, 540_000),
        ),
        (
            "511190.SH",
            "credit_bond.csv",
            build_proxy_rows(420, 1.050, 0.002, -0.002, 5, 500_000),
        ),
        (
            "512200.SH",
            "real_estate.csv",
            build_proxy_rows(420, 0.980, 0.007, -0.003, 7, 710_000),
        ),
        (
            "159928.SZ",
            "consumption.csv",
            build_proxy_rows(420, 1.120, 0.005, -0.002, 8, 690_000),
        ),
        (
            "510300.SH",
            "market.csv",
            build_proxy_rows(420, 4.100, 0.018, -0.006, 6, 1_600_000),
        ),
    ] {
        let csv_path = create_stock_history_csv(prefix, file_name, &rows);
        import_history_csv(&runtime_db_path, &csv_path, symbol);
    }

    let server = spawn_http_route_server(vec![
        (
            "/financials",
            "HTTP/1.1 200 OK",
            r#"[
                {
                    "REPORT_DATE":"2025-12-31",
                    "NOTICE_DATE":"2026-03-28",
                    "TOTAL_OPERATE_INCOME":214500000000.0,
                    "YSTZ":1.25,
                    "PARENT_NETPROFIT":68500000000.0,
                    "SJLTZ":2.91,
                    "ROEJQ":8.52
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
                        {"notice_date":"2026-03-28","title":"2025年度利润分配预案公告","art_code":"AN202603281010101010","columns":[{"column_name":"公司公告"}]},
                        {"notice_date":"2026-03-20","title":"关于资本工具发行进展的公告","art_code":"AN202603201010101011","columns":[{"column_name":"公司公告"}]}
                    ]
                }
            }"#,
            "application/json",
        ),
    ]);

    let bootstrap_request = json!({
        "tool": "bootstrap_resonance_template_factors",
        "args": {
            "market_regime": "a_share"
        }
    });
    assert_eq!(
        run_cli_with_json_runtime_and_envs(&bootstrap_request.to_string(), &runtime_db_path, &[])["status"],
        "ok"
    );

    for (factor_key, points) in [
        (
            "bank_sector_proxy",
            build_bank_positive_macro_points(420, 100.0),
        ),
        (
            "cn_bond_10y_yield",
            build_bank_inverse_macro_points(420, 2.25),
        ),
        (
            "credit_risk_spread",
            build_bank_inverse_macro_points(420, 1.35),
        ),
        (
            "real_estate_proxy",
            build_bank_positive_macro_points(420, 88.0),
        ),
        (
            "resident_consumption_proxy",
            build_bank_positive_macro_points(420, 96.0),
        ),
        (
            "loan_growth_proxy",
            build_bank_positive_macro_points(420, 104.0),
        ),
        ("pmi_proxy", build_bank_positive_macro_points(420, 49.5)),
    ] {
        let append_series_request = json!({
            "tool": "append_resonance_factor_series",
            "args": {
                "factor_key": factor_key,
                "source": "manual_fixture",
                "points": points
            }
        });
        let append_series_output = run_cli_with_json_runtime_and_envs(
            &append_series_request.to_string(),
            &runtime_db_path,
            &[],
        );
        assert!(
            append_series_output["status"] == "ok",
            "append_series_output={append_series_output:#?}"
        );
    }

    let envs = vec![
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
    ];

    for symbol in ["601998.SH", "600000.SH", "601398.SH"] {
        for snapshot_date in ["2025-09-17", "2025-11-14", "2025-12-12"] {
            let snapshot_request = json!({
                "tool": "record_security_signal_snapshot",
                "args": {
                    "symbol": symbol,
                    "market_symbol": "510300.SH",
                    "sector_symbol": "512800.SH",
                    "market_regime": "a_share",
                    "sector_template": "bank",
                    "as_of_date": snapshot_date,
                    "lookback_days": 180,
                    "factor_lookback_days": 120,
                    "disclosure_limit": 3
                }
            });
            let snapshot_output = run_cli_with_json_runtime_and_envs(
                &snapshot_request.to_string(),
                &runtime_db_path,
                &envs,
            );
            assert!(
                snapshot_output["status"] == "ok",
                "snapshot_output={snapshot_output:#?}"
            );

            let backfill_request = json!({
                "tool": "backfill_security_signal_outcomes",
                "args": {
                    "symbol": symbol,
                    "snapshot_date": snapshot_output["data"]["snapshot_date"]
                }
            });
            let backfill_output = run_cli_with_json_runtime_and_envs(
                &backfill_request.to_string(),
                &runtime_db_path,
                &[],
            );
            assert!(
                backfill_output["status"] == "ok",
                "backfill_output={backfill_output:#?}"
            );
        }
    }

    (runtime_db_path, server)
}

#[test]
fn tool_catalog_includes_security_analysis_resonance_platform_tools() {
    let output = run_cli_with_json("");
    let tools = output["data"]["tool_catalog"]
        .as_array()
        .expect("tool catalog should be an array")
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>();
    let stock_tools = output["data"]["tool_catalog_modules"]["stock"]
        .as_array()
        .expect("stock tool catalog should be an array")
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>();

    // 2026-04-02 CST：这里先锁共振平台 Tool 家族的可发现性，原因是方案 3 不是单个分析函数，而是一组可研究、可落库的正式入口；
    // 目的：防止后续只实现内部模块却漏掉 catalog/dispatcher 暴露，导致平台存在但外层不可用。
    for tool_name in [
        "register_resonance_factor",
        "append_resonance_factor_series",
        "append_resonance_event_tags",
        "bootstrap_resonance_template_factors",
        "evaluate_security_resonance",
        "security_analysis_resonance",
    ] {
        assert!(
            tools.contains(&tool_name),
            "tool catalog should include `{tool_name}`"
        );
        assert!(
            stock_tools.contains(&tool_name),
            "stock tool group should include `{tool_name}`"
        );
    }
}

#[test]
fn bootstrap_resonance_template_factors_registers_default_candidates() {
    let runtime_db_path = create_test_runtime_db("security_analysis_resonance_bootstrap");
    let request = json!({
        "tool": "bootstrap_resonance_template_factors",
        "args": {
            "market_regime": "a_share"
        }
    });

    let output = run_cli_with_json_runtime_and_envs(&request.to_string(), &runtime_db_path, &[]);

    // 2026-04-02 CST：这里先锁模板池初始化入口，原因是第二阶段要把“传统行业候选因子池”正式沉到平台底层；
    // 目的：确保后续分析和研究不必每次都手工注册石油、航运、煤炭、有色、银行的基本盘因子。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["market_regime"], "a_share");
    assert!(
        output["data"]["inserted_factor_count"]
            .as_u64()
            .expect("inserted_factor_count should be numeric")
            >= 12
    );
    assert!(
        output["data"]["templates"]
            .as_array()
            .expect("templates should be an array")
            .iter()
            .filter_map(|value| value.as_str())
            .any(|value| value == "oil_petrochemical")
    );

    let resonance_db_path = runtime_db_path
        .parent()
        .expect("runtime db path should have a parent")
        .join("security_resonance.db");
    let connection =
        Connection::open(resonance_db_path).expect("security resonance db should be created");
    let template_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM resonance_factor_registry WHERE market_regime = 'a_share'",
            [],
            |row| row.get(0),
        )
        .expect("template count query should succeed");
    let oil_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM resonance_factor_registry WHERE template_key = 'oil_petrochemical'",
            [],
            |row| row.get(0),
        )
        .expect("oil template count query should succeed");
    let bank_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM resonance_factor_registry WHERE template_key = 'bank'",
            [],
            |row| row.get(0),
        )
        .expect("bank template count query should succeed");

    let bank_factor_keys = {
        let mut statement = connection
            .prepare(
                "SELECT factor_key FROM resonance_factor_registry WHERE template_key = 'bank' ORDER BY factor_key",
            )
            .expect("bank factor query should prepare");
        statement
            .query_map([], |row| row.get::<_, String>(0))
            .expect("bank factor query should run")
            .collect::<Result<Vec<_>, _>>()
            .expect("bank factor rows should collect")
    };

    assert!(template_count >= 12);
    assert!(oil_count >= 3);
    assert!(bank_count >= 7);
    for factor_key in [
        "bank_sector_proxy",
        "cn_bond_10y_yield",
        "credit_risk_spread",
        "real_estate_proxy",
        "resident_consumption_proxy",
        "loan_growth_proxy",
        "pmi_proxy",
    ] {
        assert!(
            bank_factor_keys.iter().any(|value| value == factor_key),
            "bank template should register `{factor_key}`"
        );
    }
}

#[test]
fn evaluate_bank_resonance_uses_macro_and_sector_candidates() {
    let runtime_db_path = create_test_runtime_db("security_analysis_resonance_bank_macro");

    let stock_csv = create_stock_history_csv(
        "security_analysis_resonance_bank_macro",
        "bank_stock.csv",
        &build_bank_stock_rows(220, 7.10),
    );
    let sector_csv = create_stock_history_csv(
        "security_analysis_resonance_bank_macro",
        "bank_sector.csv",
        &build_bank_stock_rows(220, 1.05),
    );
    import_history_csv(&runtime_db_path, &stock_csv, "601998.SH");
    import_history_csv(&runtime_db_path, &sector_csv, "512800.SH");

    let bootstrap_request = json!({
        "tool": "bootstrap_resonance_template_factors",
        "args": {
            "market_regime": "a_share"
        }
    });
    let bootstrap_output =
        run_cli_with_json_runtime_and_envs(&bootstrap_request.to_string(), &runtime_db_path, &[]);
    assert_eq!(bootstrap_output["status"], "ok");

    // 2026-04-02 CST：这里一次性把银行关键宏观代理写入夹具，原因是用户明确要求共振不能只剩板块和利率；
    // 目的：验证评估结果会同时吸收地产、消费、贷款、PMI 以及利率/信用这些正式候选因子。
    for (factor_key, points) in [
        (
            "cn_bond_10y_yield",
            build_bank_inverse_macro_points(220, 2.25),
        ),
        (
            "credit_risk_spread",
            build_bank_inverse_macro_points(220, 1.35),
        ),
        (
            "real_estate_proxy",
            build_bank_positive_macro_points(220, 88.0),
        ),
        (
            "resident_consumption_proxy",
            build_bank_positive_macro_points(220, 96.0),
        ),
        (
            "loan_growth_proxy",
            build_bank_positive_macro_points(220, 104.0),
        ),
        ("pmi_proxy", build_bank_positive_macro_points(220, 49.5)),
    ] {
        let append_series_request = json!({
            "tool": "append_resonance_factor_series",
            "args": {
                "factor_key": factor_key,
                "source": "manual_fixture",
                "points": points
            }
        });
        assert_eq!(
            run_cli_with_json_runtime_and_envs(
                &append_series_request.to_string(),
                &runtime_db_path,
                &[]
            )["status"],
            "ok"
        );
    }

    let evaluate_request = json!({
        "tool": "evaluate_security_resonance",
        "args": {
            "symbol": "601998.SH",
            "market_regime": "a_share",
            "sector_template": "bank",
            "sector_symbol": "512800.SH",
            "factor_lookback_days": 120
        }
    });
    let evaluate_output =
        run_cli_with_json_runtime_and_envs(&evaluate_request.to_string(), &runtime_db_path, &[]);

    assert_eq!(evaluate_output["status"], "ok");
    assert_eq!(evaluate_output["data"]["symbol"], "601998.SH");
    assert_eq!(
        evaluate_output["data"]["resonance_context"]["sector_template"],
        "bank"
    );

    let positive_factor_keys =
        evaluate_output["data"]["resonance_context"]["top_positive_resonances"]
            .as_array()
            .expect("top_positive_resonances should be an array")
            .iter()
            .filter_map(|value| value["factor_key"].as_str())
            .collect::<Vec<_>>();
    assert!(
        positive_factor_keys
            .iter()
            .any(|value| value != &"bank_sector_proxy"),
        "bank resonance should expose at least one macro driver beyond sector proxy"
    );
    assert!(
        positive_factor_keys.iter().any(|value| {
            matches!(
                *value,
                "cn_bond_10y_yield"
                    | "credit_risk_spread"
                    | "real_estate_proxy"
                    | "resident_consumption_proxy"
                    | "loan_growth_proxy"
                    | "pmi_proxy"
            )
        }),
        "bank resonance should include a macro candidate in top positives"
    );

    let resonance_db_path = runtime_db_path
        .parent()
        .expect("runtime db path should have a parent")
        .join("security_resonance.db");
    let connection =
        Connection::open(resonance_db_path).expect("security resonance db should be created");
    let snapshot_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM security_resonance_snapshots WHERE symbol = '601998.SH'",
            [],
            |row| row.get(0),
        )
        .expect("snapshot count query should succeed");
    assert!(
        snapshot_count >= 7,
        "bank resonance should persist sector plus macro candidate snapshots"
    );
}

#[test]
fn evaluate_security_resonance_persists_snapshots_without_fullstack_layer() {
    let runtime_db_path = create_test_runtime_db("security_analysis_resonance_evaluate");

    let stock_csv = create_stock_history_csv(
        "security_analysis_resonance_evaluate",
        "stock.csv",
        &build_oil_stock_rows(220, 8.4),
    );
    let sector_csv = create_stock_history_csv(
        "security_analysis_resonance_evaluate",
        "sector.csv",
        &build_oil_sector_rows(220, 1.8),
    );
    import_history_csv(&runtime_db_path, &stock_csv, "601857.SH");
    import_history_csv(&runtime_db_path, &sector_csv, "516570.SH");

    let bootstrap_request = json!({
        "tool": "bootstrap_resonance_template_factors",
        "args": {
            "market_regime": "a_share"
        }
    });
    let bootstrap_output =
        run_cli_with_json_runtime_and_envs(&bootstrap_request.to_string(), &runtime_db_path, &[]);
    assert_eq!(bootstrap_output["status"], "ok");

    let append_series_request = json!({
        "tool": "append_resonance_factor_series",
        "args": {
            "factor_key": "brent_crude",
            "source": "manual_fixture",
            "points": build_positive_factor_points(220, 70.0)
        }
    });
    let append_events_request = json!({
        "tool": "append_resonance_event_tags",
        "args": {
            "tags": [
                {
                    "event_key": "oil_supply_risk",
                    "event_date": "2025-09-08",
                    "title": "中东供应风险升温",
                    "market_regime": "a_share",
                    "template_key": "oil_petrochemical",
                    "symbol_scope": "601857.SH",
                    "polarity": "positive",
                    "strength": 0.76,
                    "notes": "2026-04-02 CST 测试夹具：锁独立评估入口也能带事件覆盖。"
                }
            ]
        }
    });
    assert_eq!(
        run_cli_with_json_runtime_and_envs(
            &append_series_request.to_string(),
            &runtime_db_path,
            &[]
        )["status"],
        "ok"
    );
    assert_eq!(
        run_cli_with_json_runtime_and_envs(
            &append_events_request.to_string(),
            &runtime_db_path,
            &[]
        )["status"],
        "ok"
    );

    let evaluate_request = json!({
        "tool": "evaluate_security_resonance",
        "args": {
            "symbol": "601857.SH",
            "market_regime": "a_share",
            "sector_template": "oil_petrochemical",
            "sector_symbol": "516570.SH",
            "factor_lookback_days": 120
        }
    });
    let evaluate_output =
        run_cli_with_json_runtime_and_envs(&evaluate_request.to_string(), &runtime_db_path, &[]);

    // 2026-04-02 CST：这里锁独立评估入口，原因是第二阶段要把“研究评估”和“最终分析输出”拆开；
    // 目的：确保后续 Agent/Skill 可以先跑评估，再决定要不要叠 fullstack，而不是所有逻辑都绑在一个 Tool 里。
    assert_eq!(evaluate_output["status"], "ok");
    assert_eq!(evaluate_output["data"]["symbol"], "601857.SH");
    assert_eq!(
        evaluate_output["data"]["resonance_context"]["sector_template"],
        "oil_petrochemical"
    );
    assert_eq!(
        evaluate_output["data"]["resonance_context"]["top_positive_resonances"][0]["factor_key"],
        "brent_crude"
    );
    assert_eq!(
        evaluate_output["data"]["resonance_context"]["event_overrides"][0]["event_key"],
        "oil_supply_risk"
    );

    let resonance_db_path = runtime_db_path
        .parent()
        .expect("runtime db path should have a parent")
        .join("security_resonance.db");
    let connection =
        Connection::open(resonance_db_path).expect("security resonance db should be created");
    let snapshot_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM security_resonance_snapshots WHERE symbol = '601857.SH'",
            [],
            |row| row.get(0),
        )
        .expect("snapshot count query should succeed");
    assert!(snapshot_count >= 2);
}

#[test]
fn security_analysis_resonance_persists_registry_series_events_and_snapshots() {
    let runtime_db_path = create_test_runtime_db("security_analysis_resonance_platform");

    let stock_csv = create_stock_history_csv(
        "security_analysis_resonance_platform",
        "stock.csv",
        &build_oil_stock_rows(220, 8.2),
    );
    let market_csv = create_stock_history_csv(
        "security_analysis_resonance_platform",
        "market.csv",
        &build_supportive_market_rows(220, 4.1),
    );
    let sector_csv = create_stock_history_csv(
        "security_analysis_resonance_platform",
        "sector.csv",
        &build_oil_sector_rows(220, 1.7),
    );
    import_history_csv(&runtime_db_path, &stock_csv, "601857.SH");
    import_history_csv(&runtime_db_path, &market_csv, "510300.SH");
    import_history_csv(&runtime_db_path, &sector_csv, "516570.SH");

    let server = spawn_http_route_server(vec![
        (
            "/financials",
            "HTTP/1.1 200 OK",
            r#"[
                {
                    "REPORT_DATE":"2025-12-31",
                    "NOTICE_DATE":"2026-03-28",
                    "TOTAL_OPERATE_INCOME":298000000000.0,
                    "YSTZ":6.82,
                    "PARENT_NETPROFIT":162000000000.0,
                    "SJLTZ":7.21,
                    "ROEJQ":11.4
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
                        {"notice_date":"2026-03-28","title":"2025年度利润分配预案公告","art_code":"AN202603281010101010","columns":[{"column_name":"公司公告"}]},
                        {"notice_date":"2026-03-15","title":"关于回购进展的公告","art_code":"AN202603151010101011","columns":[{"column_name":"公司公告"}]}
                    ]
                }
            }"#,
            "application/json",
        ),
    ]);

    let register_positive_factor_request = json!({
        "tool": "register_resonance_factor",
        "args": {
            "factor_key": "brent_crude",
            "display_name": "布伦特原油",
            "market_regime": "a_share",
            "template_key": "oil_petrochemical",
            "factor_type": "price_series",
            "source_kind": "manual_fixture",
            "expected_relation": "positive",
            "notes": "2026-04-02 CST 测试夹具：用于锁定石油股与油价正向共振合同。"
        }
    });
    let register_negative_factor_request = json!({
        "tool": "register_resonance_factor",
        "args": {
            "factor_key": "false_positive_proxy",
            "display_name": "伪正向成本因子",
            "market_regime": "a_share",
            "template_key": "oil_petrochemical",
            "factor_type": "price_series",
            "source_kind": "manual_fixture",
            "expected_relation": "positive",
            "notes": "2026-04-02 CST 测试夹具：用于锁定背离因子应进入负向共振输出。"
        }
    });
    let register_positive_output = run_cli_with_json_runtime_and_envs(
        &register_positive_factor_request.to_string(),
        &runtime_db_path,
        &[],
    );
    let register_negative_output = run_cli_with_json_runtime_and_envs(
        &register_negative_factor_request.to_string(),
        &runtime_db_path,
        &[],
    );

    // 2026-04-02 CST：这里先锁因子注册合同，原因是平台第一层是“因子发现与注册”，不是直接跳到最终结论；
    // 目的：确保后续新增想法时，确实可以先把因子定义写入平台，而不是继续散落在代码常量里。
    assert_eq!(register_positive_output["status"], "ok");
    assert_eq!(register_negative_output["status"], "ok");

    let append_positive_series_request = json!({
        "tool": "append_resonance_factor_series",
        "args": {
            "factor_key": "brent_crude",
            "source": "manual_fixture",
            "points": build_positive_factor_points(220, 68.0)
        }
    });
    let append_negative_series_request = json!({
        "tool": "append_resonance_factor_series",
        "args": {
            "factor_key": "false_positive_proxy",
            "source": "manual_fixture",
            "points": build_negative_factor_points(220, 41.0)
        }
    });
    let append_events_request = json!({
        "tool": "append_resonance_event_tags",
        "args": {
            "tags": [
                {
                    "event_key": "hormuz_strait_tension",
                    "event_date": "2025-08-04",
                    "title": "霍尔木兹海峡运输风险升温",
                    "market_regime": "a_share",
                    "template_key": "oil_petrochemical",
                    "symbol_scope": "601857.SH",
                    "polarity": "positive",
                    "strength": 0.88,
                    "notes": "2026-04-02 CST 测试夹具：用于锁定事件标签也会进入第一版平台。"
                }
            ]
        }
    });
    let append_positive_output = run_cli_with_json_runtime_and_envs(
        &append_positive_series_request.to_string(),
        &runtime_db_path,
        &[],
    );
    let append_negative_output = run_cli_with_json_runtime_and_envs(
        &append_negative_series_request.to_string(),
        &runtime_db_path,
        &[],
    );
    let append_events_output = run_cli_with_json_runtime_and_envs(
        &append_events_request.to_string(),
        &runtime_db_path,
        &[],
    );

    // 2026-04-02 CST：这里再锁序列与事件落库合同，原因是用户明确要求“算出来以后写到数据库里，后边再评估”；
    // 目的：确保价格因子和事件标签都会成为平台正式数据资产，而不是分析时的一次性临时输入。
    assert_eq!(append_positive_output["status"], "ok");
    assert_eq!(append_negative_output["status"], "ok");
    assert_eq!(append_events_output["status"], "ok");

    let analysis_request = json!({
        "tool": "security_analysis_resonance",
        "args": {
            "symbol": "601857.SH",
            "market_symbol": "510300.SH",
            "sector_symbol": "516570.SH",
            "market_regime": "a_share",
            "sector_template": "oil_petrochemical",
            "as_of_date": "2025-08-08",
            "lookback_days": 180,
            "factor_lookback_days": 120,
            "disclosure_limit": 3
        }
    });
    let analysis_output = run_cli_with_json_runtime_and_envs(
        &analysis_request.to_string(),
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

    // 2026-04-02 CST：这里锁共振分析正式对外合同，原因是平台最终必须把“正向共振、负向共振、事件覆盖和动作倾向”一起带出来；
    // 目的：确保后续不会把平台退化成只落库不出分析，或者只出结论不暴露驱动。
    assert_eq!(analysis_output["status"], "ok");
    assert_eq!(analysis_output["data"]["symbol"], "601857.SH");
    assert_eq!(
        analysis_output["data"]["resonance_context"]["sector_template"],
        "oil_petrochemical"
    );
    assert_eq!(
        analysis_output["data"]["resonance_context"]["top_positive_resonances"][0]["factor_key"],
        "brent_crude"
    );
    assert_eq!(
        analysis_output["data"]["resonance_context"]["top_negative_resonances"][0]["factor_key"],
        "false_positive_proxy"
    );
    assert_eq!(
        analysis_output["data"]["resonance_context"]["event_overrides"][0]["event_key"],
        "hormuz_strait_tension"
    );
    assert!(
        analysis_output["data"]["resonance_context"]["resonance_score"]
            .as_f64()
            .expect("resonance score should be numeric")
            > 0.45
    );
    assert!(
        !analysis_output["data"]["resonance_context"]["action_bias"]
            .as_str()
            .expect("action bias should exist")
            .is_empty()
    );

    let resonance_db_path = runtime_db_path
        .parent()
        .expect("runtime db path should have a parent")
        .join("security_resonance.db");
    let connection =
        Connection::open(resonance_db_path).expect("security resonance db should be created");
    let registry_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM resonance_factor_registry",
            [],
            |row| row.get(0),
        )
        .expect("registry count query should succeed");
    let series_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM resonance_factor_series", [], |row| {
            row.get(0)
        })
        .expect("series count query should succeed");
    let event_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM resonance_event_tags", [], |row| {
            row.get(0)
        })
        .expect("event count query should succeed");
    let snapshot_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM security_resonance_snapshots WHERE symbol = '601857.SH'",
            [],
            |row| row.get(0),
        )
        .expect("snapshot count query should succeed");

    // 2026-04-02 CST：这里补数据库落地断言，原因是用户明确要求“算了以后写到数据库里，后边把相关性强的拉出来评估”；
    // 目的：把“平台不是一次性分析器，而是会沉淀研究资产”的要求钉成回归合同。
    assert_eq!(registry_count, 2);
    assert!(
        series_count >= 440,
        "series table should contain both factors"
    );
    assert_eq!(event_count, 1);
    assert!(
        snapshot_count >= 3,
        "snapshot table should persist evaluated drivers"
    );
}

#[test]
fn security_decision_briefing_exposes_full_indicator_and_resonance_layers() {
    let output =
        build_security_decision_briefing_fixture_output("security_decision_briefing_layers");

    for field_name in [
        "adx_14",
        "macd",
        "rsi_14",
        "mfi_14",
        "obv",
        "cci_20",
        "williams_r_14",
        "rsrs_beta_18",
        "rsrs_zscore_18_60",
    ] {
        assert!(
            output["technical_brief"]["indicator_snapshot"]
                .get(field_name)
                .is_some(),
            "technical_brief.indicator_snapshot should expose `{field_name}`"
        );
    }

    for field_name in [
        "resonance_score",
        "action_bias",
        "top_positive_resonances",
        "top_negative_resonances",
    ] {
        assert!(
            output["resonance_brief"].get(field_name).is_some(),
            "resonance_brief should expose `{field_name}`"
        );
    }
}

#[test]
fn security_decision_briefing_exposes_execution_thresholds() {
    let output =
        build_security_decision_briefing_fixture_output("security_decision_briefing_execution");

    for field_name in [
        "add_trigger_price",
        "add_trigger_volume_ratio",
        "add_position_pct",
        "reduce_trigger_price",
        "rejection_zone",
        "reduce_position_pct",
        "stop_loss_price",
        "invalidation_price",
        "watch_points",
    ] {
        assert!(
            output["execution_plan"].get(field_name).is_some(),
            "execution_plan should expose `{field_name}`"
        );
    }

    // 2026-04-02 CST: 这里要求执行层阈值必须已经从真实指标派生，原因是 Task 3 的目标不是只补字段，而是把阈值从技术层收敛出来；
    // 目的：阻止 assembler 长期停留在 0 仓位、pending 区间这类占位状态，确保 briefing 已具备可执行交易语义。
    assert!(
        output["execution_plan"]["add_trigger_price"]
            .as_f64()
            .expect("add_trigger_price should be numeric")
            > 0.0
    );
    assert!(
        output["execution_plan"]["add_trigger_volume_ratio"]
            .as_f64()
            .expect("add_trigger_volume_ratio should be numeric")
            > 0.0
    );
    assert!(
        output["execution_plan"]["add_position_pct"]
            .as_f64()
            .expect("add_position_pct should be numeric")
            > 0.0
    );
    assert!(
        output["execution_plan"]["reduce_trigger_price"]
            .as_f64()
            .expect("reduce_trigger_price should be numeric")
            > 0.0
    );
    assert!(
        output["execution_plan"]["reduce_position_pct"]
            .as_f64()
            .expect("reduce_position_pct should be numeric")
            > 0.0
    );
    assert!(
        output["execution_plan"]["stop_loss_price"]
            .as_f64()
            .expect("stop_loss_price should be numeric")
            > 0.0
    );
    assert!(
        output["execution_plan"]["invalidation_price"]
            .as_f64()
            .expect("invalidation_price should be numeric")
            > 0.0
    );
    assert!(
        !output["execution_plan"]["rejection_zone"]
            .as_str()
            .expect("rejection_zone should be a string")
            .contains("pending")
    );
    assert!(
        output["execution_plan"]["watch_points"]
            .as_array()
            .expect("watch_points should be an array")
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| !text.trim().is_empty())
    );
}

#[test]
fn security_decision_briefing_exposes_committee_payload() {
    let output =
        build_security_decision_briefing_fixture_output("security_decision_briefing_committee");

    for field_name in [
        "symbol",
        "analysis_date",
        "recommended_action",
        "confidence",
        "key_risks",
        "minority_objection_points",
        "evidence_version",
        "briefing_digest",
    ] {
        assert!(
            output["committee_payload"].get(field_name).is_some(),
            "committee_payload should expose `{field_name}`"
        );
    }

    // 2026-04-02 CST: 这里把 committee payload 和顶层 briefing 的同源约束一起锁住，原因是 Task 5 的真正目标是让咨询与投决共享同一份 factual payload；
    // 目的：防止后续 committee 端再拼第二份摘要或版本号，导致同一只证券在两个场景看到不同底稿。
    assert_eq!(output["committee_payload"]["symbol"], output["symbol"]);
    assert_eq!(
        output["committee_payload"]["analysis_date"],
        output["analysis_date"]
    );
    assert_eq!(
        output["committee_payload"]["evidence_version"],
        output["evidence_version"]
    );
    assert_eq!(
        output["committee_payload"]["briefing_digest"],
        output["summary"]
    );
}

#[test]
fn security_decision_briefing_exposes_vote_ready_committee_payload() {
    let output =
        build_security_decision_briefing_fixture_output("security_decision_briefing_vote_ready");

    // 2026-04-02 CST: 这里先锁投决会事实包的结构化子层，原因是方案 B 的第一步不是直接做 vote engine，
    // 目的：而是先把 committee_payload 扩成“可投票、可解释、可保留风险边界”的统一事实包，避免后续角色投票继续吃扁平摘要。
    for field_name in [
        "committee_schema_version",
        "recommendation_digest",
        "execution_digest",
        "resonance_digest",
        "evidence_checks",
        "historical_digest",
    ] {
        assert!(
            output["committee_payload"].get(field_name).is_some(),
            "committee_payload should expose structured field `{field_name}`"
        );
    }

    for field_name in ["final_stance", "action_bias"] {
        assert!(
            output["committee_payload"]["recommendation_digest"]
                .get(field_name)
                .is_some(),
            "recommendation_digest should expose `{field_name}`"
        );
    }

    for field_name in ["add_trigger_price", "stop_loss_price", "watch_points"] {
        assert!(
            output["committee_payload"]["execution_digest"]
                .get(field_name)
                .is_some(),
            "execution_digest should expose `{field_name}`"
        );
    }

    for field_name in [
        "resonance_score",
        "action_bias",
        "top_positive_driver_names",
        "top_negative_driver_names",
    ] {
        assert!(
            output["committee_payload"]["resonance_digest"]
                .get(field_name)
                .is_some(),
            "resonance_digest should expose `{field_name}`"
        );
    }

    for field_name in ["fundamental_ready", "technical_ready", "resonance_ready"] {
        assert!(
            output["committee_payload"]["evidence_checks"]
                .get(field_name)
                .is_some(),
            "evidence_checks should expose `{field_name}`"
        );
    }

    for field_name in ["status", "historical_confidence", "research_limitations"] {
        assert!(
            output["committee_payload"]["historical_digest"]
                .get(field_name)
                .is_some(),
            "historical_digest should expose `{field_name}`"
        );
    }
}

#[test]
fn record_security_signal_snapshot_persists_full_indicator_state() {
    let runtime_db_path = create_test_runtime_db("signal_outcome_snapshot_contract");

    // 2026-04-02 CST: 这里复用油气顺风样本构造 snapshot 红测，原因是 Task 2 需要证明 research 平台不是手写假数据入库，
    // 目的：而是沿真实 `CSV -> SQLite -> resonance/fullstack -> signal_outcome_store` 主链把技术面与共振面快照一起沉淀下来。
    let stock_csv = create_stock_history_csv(
        "signal_outcome_snapshot_contract",
        "stock.csv",
        &build_oil_stock_rows(220, 8.2),
    );
    let market_csv = create_stock_history_csv(
        "signal_outcome_snapshot_contract",
        "market.csv",
        &build_supportive_market_rows(220, 4.1),
    );
    let sector_csv = create_stock_history_csv(
        "signal_outcome_snapshot_contract",
        "sector.csv",
        &build_oil_sector_rows(220, 1.7),
    );
    import_history_csv(&runtime_db_path, &stock_csv, "601857.SH");
    import_history_csv(&runtime_db_path, &market_csv, "510300.SH");
    import_history_csv(&runtime_db_path, &sector_csv, "516570.SH");

    let server = spawn_http_route_server(vec![
        (
            "/financials",
            "HTTP/1.1 200 OK",
            r#"[
                {
                    "REPORT_DATE":"2025-12-31",
                    "NOTICE_DATE":"2026-03-28",
                    "TOTAL_OPERATE_INCOME":298000000000.0,
                    "YSTZ":6.82,
                    "PARENT_NETPROFIT":162000000000.0,
                    "SJLTZ":7.21,
                    "ROEJQ":11.4
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
                        {"notice_date":"2026-03-28","title":"2025年度利润分配预案公告","art_code":"AN202603281010101010","columns":[{"column_name":"公司公告"}]}
                    ]
                }
            }"#,
            "application/json",
        ),
    ]);

    let bootstrap_request = json!({
        "tool": "bootstrap_resonance_template_factors",
        "args": {
            "market_regime": "a_share"
        }
    });
    assert_eq!(
        run_cli_with_json_runtime_and_envs(&bootstrap_request.to_string(), &runtime_db_path, &[])["status"],
        "ok"
    );

    let snapshot_request = json!({
        "tool": "record_security_signal_snapshot",
        "args": {
            "symbol": "601857.SH",
            "market_symbol": "510300.SH",
            "sector_symbol": "516570.SH",
            "market_regime": "a_share",
            "sector_template": "oil_petrochemical",
            "as_of_date": "2025-08-08",
            "lookback_days": 180,
            "factor_lookback_days": 120,
            "disclosure_limit": 3
        }
    });
    let snapshot_output = run_cli_with_json_runtime_and_envs(
        &snapshot_request.to_string(),
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

    // 2026-04-02 CST: 这里先锁研究快照的对外合同，原因是用户明确要求“很多指标不能丢，而且要真正落数据库”，
    // 目的：确保研究层至少把趋势/动量/波动/资金/RSRS/共振与动作偏向沉淀成统一 indicator_digest，而不是只存几项摘要。
    assert_eq!(snapshot_output["status"], "ok");
    for field_name in [
        "sma_50",
        "sma_200",
        "ema_10",
        "adx_14",
        "plus_di_14",
        "minus_di_14",
        "macd",
        "macd_histogram",
        "rsi_14",
        "k_9",
        "d_9",
        "j_9",
        "atr_14",
        "boll_middle",
        "boll_width_ratio_20",
        "mfi_14",
        "obv",
        "volume_ratio_20",
        "cci_20",
        "williams_r_14",
        "rsrs_beta_18",
        "rsrs_zscore_18_60",
        "resonance_score",
        "action_bias",
    ] {
        assert!(
            snapshot_output["data"]["indicator_snapshot"]
                .get(field_name)
                .is_some(),
            "indicator_snapshot should expose `{field_name}`"
        );
    }

    let signal_outcome_db_path = runtime_db_path
        .parent()
        .expect("runtime db path should have a parent")
        .join("signal_outcome_research.db");
    let connection =
        Connection::open(signal_outcome_db_path).expect("signal outcome db should be created");
    let row: (String, f64, String, String) = connection
        .query_row(
            "SELECT indicator_digest, resonance_score, action_bias, snapshot_payload
             FROM security_signal_snapshots
             WHERE symbol = '601857.SH'
             ORDER BY snapshot_date DESC
             LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .expect("signal outcome snapshot row should exist");

    for field_name in [
        "sma_50",
        "sma_200",
        "ema_10",
        "adx_14",
        "plus_di_14",
        "minus_di_14",
        "macd",
        "macd_histogram",
        "rsi_14",
        "k_9",
        "d_9",
        "j_9",
        "atr_14",
        "boll_middle",
        "boll_width_ratio_20",
        "mfi_14",
        "obv",
        "volume_ratio_20",
        "cci_20",
        "williams_r_14",
        "rsrs_beta_18",
        "rsrs_zscore_18_60",
        "resonance_score",
        "action_bias",
    ] {
        assert!(
            row.0.contains(field_name) || row.3.contains(field_name),
            "persisted snapshot should retain `{field_name}` in digest or payload"
        );
    }
    assert!(row.1 > 0.0, "persisted resonance_score should be numeric");
    assert!(
        !row.2.is_empty(),
        "persisted action_bias should keep an executable bias string"
    );
}

#[test]
fn backfill_security_signal_outcomes_records_forward_metrics() {
    let runtime_db_path = create_test_runtime_db("signal_outcome_forward_returns");

    // 2026-04-02 CST: 这里继续沿用同一套研究夹具，原因是 Task 3 要验证的是 snapshot -> outcomes 的链路衔接，
    // 目的：确保 forward returns 来自已经落库的同一份信号快照，而不是单独伪造一批未来收益样本。
    let stock_csv = create_stock_history_csv(
        "signal_outcome_forward_returns",
        "stock.csv",
        &build_oil_stock_rows(240, 8.2),
    );
    let market_csv = create_stock_history_csv(
        "signal_outcome_forward_returns",
        "market.csv",
        &build_supportive_market_rows(240, 4.1),
    );
    let sector_csv = create_stock_history_csv(
        "signal_outcome_forward_returns",
        "sector.csv",
        &build_oil_sector_rows(240, 1.7),
    );
    import_history_csv(&runtime_db_path, &stock_csv, "601857.SH");
    import_history_csv(&runtime_db_path, &market_csv, "510300.SH");
    import_history_csv(&runtime_db_path, &sector_csv, "516570.SH");

    let server = spawn_http_route_server(vec![
        (
            "/financials",
            "HTTP/1.1 200 OK",
            r#"[
                {
                    "REPORT_DATE":"2025-12-31",
                    "NOTICE_DATE":"2026-03-28",
                    "TOTAL_OPERATE_INCOME":298000000000.0,
                    "YSTZ":6.82,
                    "PARENT_NETPROFIT":162000000000.0,
                    "SJLTZ":7.21,
                    "ROEJQ":11.4
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
                        {"notice_date":"2026-03-28","title":"2025年度利润分配预案公告","art_code":"AN202603281010101010","columns":[{"column_name":"公司公告"}]}
                    ]
                }
            }"#,
            "application/json",
        ),
    ]);

    let bootstrap_request = json!({
        "tool": "bootstrap_resonance_template_factors",
        "args": {
            "market_regime": "a_share"
        }
    });
    assert_eq!(
        run_cli_with_json_runtime_and_envs(&bootstrap_request.to_string(), &runtime_db_path, &[])["status"],
        "ok"
    );

    let snapshot_request = json!({
        "tool": "record_security_signal_snapshot",
        "args": {
            "symbol": "601857.SH",
            "market_symbol": "510300.SH",
            "sector_symbol": "516570.SH",
            "market_regime": "a_share",
            "sector_template": "oil_petrochemical",
            "as_of_date": "2025-08-08",
            "lookback_days": 180,
            "factor_lookback_days": 120,
            "disclosure_limit": 3
        }
    });
    let snapshot_output = run_cli_with_json_runtime_and_envs(
        &snapshot_request.to_string(),
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
    assert_eq!(snapshot_output["status"], "ok");

    let backfill_request = json!({
        "tool": "backfill_security_signal_outcomes",
        "args": {
            "symbol": "601857.SH",
            "snapshot_date": snapshot_output["data"]["snapshot_date"]
        }
    });
    let backfill_output =
        run_cli_with_json_runtime_and_envs(&backfill_request.to_string(), &runtime_db_path, &[]);

    // 2026-04-02 CST: 这里先锁 forward returns 的最小交付面，原因是用户要求后续投研判断必须知道“历史上类似状态后面通常怎么走”，
    // 目的：先保证 1/3/5/10/20 日的收益、最大回撤、最大上冲全部能稳定回填，再在下一批扩 analog study。
    assert_eq!(backfill_output["status"], "ok", "{backfill_output:#?}");
    for horizon in [1_i64, 3, 5, 10, 20] {
        let matched = backfill_output["data"]["forward_returns"]
            .as_array()
            .expect("forward_returns should be an array")
            .iter()
            .find(|item| item["horizon_days"].as_i64() == Some(horizon))
            .expect("expected horizon should exist");
        for field_name in ["forward_return_pct", "max_drawdown_pct", "max_runup_pct"] {
            assert!(
                matched.get(field_name).is_some(),
                "forward return row should expose `{field_name}`"
            );
        }
    }

    let signal_outcome_db_path = runtime_db_path
        .parent()
        .expect("runtime db path should have a parent")
        .join("signal_outcome_research.db");
    let connection =
        Connection::open(signal_outcome_db_path).expect("signal outcome db should be created");
    let stored_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM security_signal_forward_returns
             WHERE symbol = '601857.SH' AND snapshot_date = ?1",
            [snapshot_output["data"]["snapshot_date"]
                .as_str()
                .expect("snapshot_date should be a string")],
            |row| row.get(0),
        )
        .expect("forward return rows should be persisted");
    assert_eq!(stored_count, 5);
}

#[test]
fn study_security_signal_analogs_builds_bank_core_technical_summary() {
    let (runtime_db_path, _) =
        build_bank_signal_outcome_fixture("signal_outcome_bank_analogs_contract");

    let study_request = json!({
        "tool": "study_security_signal_analogs",
        "args": {
            "symbol": "601998.SH",
            "snapshot_date": "2025-12-12",
            "comparison_symbols": ["601998.SH", "600000.SH", "601398.SH"],
            "study_key": "bank_resonance_core_technical_v1",
            "min_similarity_score": 0.58,
            "sample_limit": 12
        }
    });
    let study_output =
        run_cli_with_json_runtime_and_envs(&study_request.to_string(), &runtime_db_path, &[]);

    // 2026-04-02 CST: 这里先锁“银行板块共振 + 核心技术相似”研究的正式输出合同，原因是用户明确要求
    // 不仅看中信银行个股，还要在银行体系里比较相似状态并输出后续涨跌统计；
    // 目的：确保 analog study 真正把银行共振、MACD、RSRS 等相似性统计沉成研究资产，而不是停留在口头分析。
    assert_eq!(study_output["status"], "ok", "{study_output:#?}");
    assert_eq!(study_output["data"]["symbol"], "601998.SH");
    assert_eq!(
        study_output["data"]["study_key"],
        "bank_resonance_core_technical_v1"
    );
    assert!(
        study_output["data"]["sample_count"]
            .as_u64()
            .expect("sample_count should be numeric")
            > 0
    );
    assert!(
        study_output["data"]["win_rate_10d"]
            .as_f64()
            .expect("win_rate_10d should be numeric")
            >= 0.0
    );
    assert!(
        study_output["data"]["matched_analogs"]
            .as_array()
            .expect("matched_analogs should be an array")
            .iter()
            .all(|item| item.get("similarity_score").is_some()),
        "matched analog rows should expose similarity_score"
    );

    let signal_outcome_db_path = runtime_db_path
        .parent()
        .expect("runtime db path should have a parent")
        .join("signal_outcome_research.db");
    let connection =
        Connection::open(signal_outcome_db_path).expect("signal outcome db should be created");
    let stored_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM security_signal_analog_studies
             WHERE symbol = '601998.SH'
               AND snapshot_date = '2025-12-12'
               AND study_key = 'bank_resonance_core_technical_v1'",
            [],
            |row| row.get(0),
        )
        .expect("analog study row should be persisted");
    assert_eq!(stored_count, 1);
}

#[test]
fn security_decision_briefing_exposes_available_historical_digest_after_analog_study() {
    let (runtime_db_path, server) =
        build_bank_signal_outcome_fixture("security_decision_briefing_historical_digest");

    let study_request = json!({
        "tool": "study_security_signal_analogs",
        "args": {
            "symbol": "601998.SH",
            "snapshot_date": "2025-12-12",
            "comparison_symbols": ["601998.SH", "600000.SH", "601398.SH"],
            "study_key": "bank_resonance_core_technical_v1",
            "min_similarity_score": 0.58,
            "sample_limit": 12
        }
    });
    let study_output =
        run_cli_with_json_runtime_and_envs(&study_request.to_string(), &runtime_db_path, &[]);
    assert_eq!(study_output["status"], "ok", "{study_output:#?}");

    let request = SecurityDecisionBriefingRequest {
        symbol: "601998.SH".to_string(),
        market_symbol: Some("510300.SH".to_string()),
        sector_symbol: Some("512800.SH".to_string()),
        market_regime: "a_share".to_string(),
        sector_template: "bank".to_string(),
        as_of_date: Some("2025-12-12".to_string()),
        lookback_days: 180,
        factor_lookback_days: 120,
        disclosure_limit: 3,
    };
    let output = run_security_decision_briefing_direct(
        &runtime_db_path,
        &request,
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

    // 2026-04-02 CST: 这里锁 briefing 对历史摘要的接入，原因是用户明确要求咨询口径和投决口径看到的信息必须一致；
    // 目的：确保 committee_payload.historical_digest 不再永远 unavailable，而是能读到银行体系内相似样本的胜率与预期区间。
    assert_eq!(
        output["committee_payload"]["historical_digest"]["status"],
        "available"
    );
    assert!(
        output["committee_payload"]["historical_digest"]["analog_sample_count"]
            .as_u64()
            .expect("analog_sample_count should be numeric")
            > 0
    );
    assert!(
        output["committee_payload"]["historical_digest"]["analog_win_rate_10d"]
            .as_f64()
            .expect("analog_win_rate_10d should be numeric")
            >= 0.0
    );
    assert!(
        output["committee_payload"]["historical_digest"]["expected_return_window"]
            .as_str()
            .expect("expected_return_window should be a string")
            .contains("10日")
    );
    assert!(
        output["committee_payload"]["historical_digest"]["expected_drawdown_window"]
            .as_str()
            .expect("expected_drawdown_window should be a string")
            .contains("10日")
    );
}

#[test]
fn security_committee_vote_consumes_briefing_payload_with_historical_digest() {
    let (runtime_db_path, server) =
        build_bank_signal_outcome_fixture("security_committee_vote_end_to_end");

    let study_request = json!({
        "tool": "study_security_signal_analogs",
        "args": {
            "symbol": "601998.SH",
            "snapshot_date": "2025-12-12",
            "comparison_symbols": ["601998.SH", "600000.SH", "601398.SH"],
            "study_key": "bank_resonance_core_technical_v1",
            "min_similarity_score": 0.58,
            "sample_limit": 12
        }
    });
    let study_output =
        run_cli_with_json_runtime_and_envs(&study_request.to_string(), &runtime_db_path, &[]);
    assert_eq!(study_output["status"], "ok", "{study_output:#?}");

    let briefing_request = json!({
        "tool": "security_decision_briefing",
        "args": {
            "symbol": "601998.SH",
            "market_symbol": "510300.SH",
            "sector_symbol": "512800.SH",
            "market_regime": "a_share",
            "sector_template": "bank",
            "as_of_date": "2025-12-12",
            "lookback_days": 180,
            "factor_lookback_days": 120,
            "disclosure_limit": 3
        }
    });
    let briefing_output = run_cli_with_json_runtime_and_envs(
        &briefing_request.to_string(),
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
    assert_eq!(briefing_output["status"], "ok", "{briefing_output:#?}");

    // 2026-04-02 CST: 这里锁 briefing -> committee vote 的正式主链，原因是用户要求投决会只能消费统一 committee_payload，
    // 目的：证明历史研究层、briefing 事实包和正式 vote Tool 已经收口成同一条 CLI 可执行链路，而不是各层各跑各的。
    assert_eq!(
        briefing_output["data"]["committee_payload"]["historical_digest"]["status"],
        "available"
    );

    let vote_request = json!({
        "tool": "security_committee_vote",
        "args": {
            "committee_payload": briefing_output["data"]["committee_payload"].clone(),
            "committee_mode": "standard",
            "meeting_id": "bank-committee-e2e-001"
        }
    });
    let vote_output =
        run_cli_with_json_runtime_and_envs(&vote_request.to_string(), &runtime_db_path, &[]);
    assert_eq!(vote_output["status"], "ok", "{vote_output:#?}");

    let vote_result: SecurityCommitteeVoteResult = serde_json::from_value(vote_output["data"].clone())
        .expect("security_committee_vote should return structured result");
    assert_eq!(vote_result.symbol, "601998.SH");
    assert_eq!(vote_result.analysis_date, "2025-12-12");
    assert_eq!(
        vote_result.evidence_version,
        briefing_output["data"]["evidence_version"]
            .as_str()
            .expect("briefing evidence_version should exist")
    );
    assert_eq!(vote_result.committee_mode, "standard");
    assert_eq!(vote_result.votes.len(), 7);
    assert!(vote_result.quorum_met);
    assert!(
        ["approved", "approved_with_conditions", "deferred"]
            .contains(&vote_result.final_decision.as_str()),
        "vote final_decision should stay on the formal committee path"
    );
    assert!(
        vote_result
            .warnings
            .iter()
            .all(|item| !item.contains("historical")),
        "historical digest available after analog study should not degrade into unavailable warning"
    );
}

#[test]
fn security_decision_briefing_includes_default_committee_recommendations_for_all_modes() {
    let (runtime_db_path, server) =
        build_bank_signal_outcome_fixture("security_decision_briefing_default_committee_modes");

    let study_request = json!({
        "tool": "study_security_signal_analogs",
        "args": {
            "symbol": "601998.SH",
            "snapshot_date": "2025-12-12",
            "comparison_symbols": ["601998.SH", "600000.SH", "601398.SH"],
            "study_key": "bank_resonance_core_technical_v1",
            "min_similarity_score": 0.58,
            "sample_limit": 12
        }
    });
    let study_output =
        run_cli_with_json_runtime_and_envs(&study_request.to_string(), &runtime_db_path, &[]);
    assert_eq!(study_output["status"], "ok", "{study_output:#?}");

    let briefing_request = json!({
        "tool": "security_decision_briefing",
        "args": {
            "symbol": "601998.SH",
            "market_symbol": "510300.SH",
            "sector_symbol": "512800.SH",
            "market_regime": "a_share",
            "sector_template": "bank",
            "as_of_date": "2025-12-12",
            "lookback_days": 180,
            "factor_lookback_days": 120,
            "disclosure_limit": 3
        }
    });
    let briefing_output = run_cli_with_json_runtime_and_envs(
        &briefing_request.to_string(),
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
    assert_eq!(briefing_output["status"], "ok", "{briefing_output:#?}");

    // 2026-04-02 CST: 这里先锁 briefing 默认就要带投决会建议摘要，原因是用户明确要求普通股票分析报告也要直接给出投决会建议，
    // 目的：让上层用户不需要先知道“要不要开投决会”，而是默认在同一份报告里看到 standard / strict / advisory 三种正式建议口径。
    let recommendations = &briefing_output["data"]["committee_recommendations"];
    assert_eq!(recommendations["default_mode"], "standard");
    assert_eq!(recommendations["report_focus"], "stock_analysis_report");
    assert_eq!(recommendations["standard"]["scenario"], "个股分析报告默认投决会建议");
    assert_eq!(recommendations["strict"]["scenario"], "涉及金额与买卖动作的严格交易建议");
    assert_eq!(recommendations["advisory"]["scenario"], "已有持仓判断与持仓处置建议");

    let committee_payload: CommitteePayload =
        serde_json::from_value(briefing_output["data"]["committee_payload"].clone())
            .expect("committee_payload should deserialize");

    for mode in ["standard", "strict", "advisory"] {
        let expected_vote = security_committee_vote(&SecurityCommitteeVoteRequest {
            committee_payload: committee_payload.clone(),
            committee_mode: mode.to_string(),
            meeting_id: Some(format!("briefing-default-{mode}")),
        })
        .expect("committee vote should succeed for all default report modes");
        let embedded_vote: SecurityCommitteeVoteResult =
            serde_json::from_value(recommendations[mode]["vote"].clone())
                .expect("embedded committee recommendation should deserialize");
        assert_eq!(embedded_vote.symbol, expected_vote.symbol);
        assert_eq!(embedded_vote.analysis_date, expected_vote.analysis_date);
        assert_eq!(embedded_vote.evidence_version, expected_vote.evidence_version);
        assert_eq!(embedded_vote.committee_engine, expected_vote.committee_engine);
        assert_eq!(embedded_vote.committee_mode, expected_vote.committee_mode);
        assert_eq!(
            embedded_vote.deliberation_seat_count,
            expected_vote.deliberation_seat_count
        );
        assert_eq!(embedded_vote.risk_seat_count, expected_vote.risk_seat_count);
        assert_eq!(embedded_vote.majority_vote, expected_vote.majority_vote);
        assert_eq!(embedded_vote.majority_count, expected_vote.majority_count);
        assert_eq!(embedded_vote.final_decision, expected_vote.final_decision);
        assert_eq!(embedded_vote.final_action, expected_vote.final_action);
        assert_eq!(embedded_vote.final_confidence, expected_vote.final_confidence);
        assert_eq!(embedded_vote.approval_ratio, expected_vote.approval_ratio);
        assert_eq!(embedded_vote.quorum_met, expected_vote.quorum_met);
        assert_eq!(embedded_vote.veto_triggered, expected_vote.veto_triggered);
        assert_eq!(embedded_vote.veto_role, expected_vote.veto_role);
        assert_eq!(embedded_vote.conditions, expected_vote.conditions);
        assert_eq!(embedded_vote.key_disagreements, expected_vote.key_disagreements);
        assert_eq!(embedded_vote.warnings, expected_vote.warnings);

        let embedded_votes_by_role = embedded_vote
            .votes
            .iter()
            .map(|vote| (vote.role.as_str(), vote))
            .collect::<HashMap<_, _>>();
        let expected_votes_by_role = expected_vote
            .votes
            .iter()
            .map(|vote| (vote.role.as_str(), vote))
            .collect::<HashMap<_, _>>();
        assert_eq!(embedded_votes_by_role.len(), expected_votes_by_role.len());

        for (role, expected_member_vote) in expected_votes_by_role {
            let embedded_member_vote = embedded_votes_by_role
                .get(role)
                .expect("embedded vote should contain the same committee role");
            assert_eq!(embedded_member_vote.member_id, expected_member_vote.member_id);
            assert_eq!(embedded_member_vote.seat_kind, expected_member_vote.seat_kind);
            assert_eq!(embedded_member_vote.evidence_version, expected_member_vote.evidence_version);
            assert_eq!(embedded_member_vote.vote, expected_member_vote.vote);
            assert_eq!(embedded_member_vote.confidence, expected_member_vote.confidence);
            assert_eq!(embedded_member_vote.rationale, expected_member_vote.rationale);
            assert_eq!(embedded_member_vote.focus_points, expected_member_vote.focus_points);
            assert_eq!(embedded_member_vote.blockers, expected_member_vote.blockers);
            assert_eq!(embedded_member_vote.conditions, expected_member_vote.conditions);
            assert_eq!(embedded_member_vote.execution_mode, "child_process");
            assert_eq!(expected_member_vote.execution_mode, "child_process");
            assert_ne!(embedded_member_vote.execution_instance_id, expected_member_vote.execution_instance_id);
            assert_ne!(embedded_member_vote.process_id, expected_member_vote.process_id);
        }
    }
}
