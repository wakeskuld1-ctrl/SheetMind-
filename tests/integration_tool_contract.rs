mod common;

use chrono::{Duration, NaiveDate};
use excel_skill::tools::contracts::ToolResponse;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::common::{create_test_runtime_db, run_cli_with_json, run_cli_with_json_and_runtime};

// 2026-04-01 CST: 这里补外层合同用的股票 CSV 夹具助手，原因是 `integration_tool_contract` 之前只锁了 catalog 可见性，
// 目的：让本文件也能走真实 `CSV -> SQLite -> technical_consultation_basic` 链路，而不是继续把合同验证留在 CLI 专项文件里。
fn create_stock_history_csv(prefix: &str, file_name: &str, rows: &[String]) -> PathBuf {
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let fixture_dir = PathBuf::from("tests")
        .join("runtime_fixtures")
        .join("integration_tool_contract")
        .join(format!("{prefix}_{unique_suffix}"));
    fs::create_dir_all(&fixture_dir).expect("integration tool contract fixture dir should exist");

    let csv_path = fixture_dir.join(file_name);
    fs::write(&csv_path, rows.join("\n")).expect("integration tool contract csv should be written");
    csv_path
}

// 2026-04-01 CST: 这里补一个导入历史数据的测试助手，原因是外层合同测试也需要把技术咨询 Tool 建立在真实导入后的 SQLite 上，
// 目的：确保本文件验证的是正式工具输入输出合同，而不是手工伪造的 JSON 结构。
fn import_history_csv(runtime_db_path: &Path, csv_path: &Path, symbol: &str) {
    let request = json!({
        "tool": "import_stock_price_history",
        "args": {
            "csv_path": csv_path.to_string_lossy(),
            "symbol": symbol,
            "source": "integration_tool_contract_fixture"
        }
    });

    let output =
        run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path.to_path_buf());
    assert_eq!(output["status"], "ok");
}

// 2026-04-01 CST: 这里复用“有效向上突破”的最小样本，原因是外层合同需要至少锁住一条最强方向性输出，
// 目的：验证 `technical_consultation_basic` 对外会稳定暴露 `bullish_continuation` 的正式 `consultation_conclusion` 字段。
fn build_confirmed_breakout_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close: f64 = 88.0;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let (next_close, volume): (f64, i64) = if offset < day_count - 20 {
            (close + 0.72, 900_000 + offset as i64 * 7_000)
        } else {
            let phase = offset - (day_count - 20);
            match phase % 4 {
                0 => (close + 1.4, 1_800_000 + phase as i64 * 30_000),
                1 => (close - 0.25, 420_000),
                2 => (close + 1.1, 1_650_000 + phase as i64 * 25_000),
                _ => (close + 0.45, 1_300_000),
            }
        };

        let open = close;
        let high = next_close.max(open) + 1.05;
        let low = next_close.min(open) - 0.9;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-04-01 CST: 这里补一个横盘弱趋势样本，原因是外层合同不能只锁方向性延续，还要锁“当前只能等待”的正式输出，
// 目的：验证 `range_wait` 在工具层返回里同样有稳定的 `confidence / rationale / risk_flags` 结构。
fn build_choppy_history_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let wave = match offset % 6 {
            0 => -0.8,
            1 => 0.7,
            2 => -0.6,
            3 => 0.6,
            4 => -0.7,
            _ => 0.8,
        };
        let base = 100.0 + wave;
        let open = base - 0.15;
        let high = base + 0.85;
        let low = base - 0.85;
        let close = base + 0.1;
        let adj_close = close;
        let volume = 900_000 + (offset % 5) as i64 * 80_000;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
    }

    rows
}

// 2026-04-01 CST: 这里补“有效向下跌破”的最小样本，原因是外层合同当前只锁了多头延续与等待态，还缺空头对称样本；
// 目的：验证 `technical_consultation_basic` 在工具层同样会稳定暴露 `bearish_continuation` 的正式 `consultation_conclusion` 合同。
fn build_confirmed_breakdown_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close: f64 = 156.0;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let (next_close, volume): (f64, i64) = if offset < day_count - 20 {
            (close - 0.68, 920_000 + offset as i64 * 6_500)
        } else {
            let phase = offset - (day_count - 20);
            match phase % 4 {
                0 => (close - 1.45, 1_760_000 + phase as i64 * 28_000),
                1 => (close + 0.22, 430_000),
                2 => (close - 1.05, 1_620_000 + phase as i64 * 24_000),
                _ => (close - 0.42, 1_280_000),
            }
        };

        let open = close;
        let high = next_close.max(open) + 0.95;
        let low = next_close.min(open) - 1.1;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

#[test]
fn cli_tool_catalog_matches_registered_tool_names() {
    let output = run_cli_with_json("");
    let actual = output["data"]["tool_catalog"]
        .as_array()
        .expect("tool catalog should be an array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("tool name should be a string")
                .to_string()
        })
        .collect::<Vec<_>>();

    let expected = excel_skill::tools::catalog::tool_names()
        .iter()
        .map(|name| (*name).to_string())
        .collect::<Vec<_>>();

    assert_eq!(actual, expected);
}

#[test]
fn tool_catalog_response_uses_registered_tool_names() {
    let response = ToolResponse::tool_catalog();
    let actual = response.data["tool_catalog"]
        .as_array()
        .expect("tool catalog should be an array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("tool name should be a string")
                .to_string()
        })
        .collect::<Vec<_>>();

    let expected = excel_skill::tools::catalog::tool_names()
        .iter()
        .map(|name| (*name).to_string())
        .collect::<Vec<_>>();

    assert_eq!(actual, expected);
}

#[test]
fn tool_catalog_response_exposes_foundation_and_stock_groups() {
    let response = ToolResponse::tool_catalog();
    let foundation = response.data["tool_catalog_modules"]["foundation"]
        .as_array()
        .expect("foundation tool catalog should be an array")
        .iter()
        .map(|value| value.as_str().expect("tool name should be a string"))
        .collect::<Vec<_>>();
    let stock = response.data["tool_catalog_modules"]["stock"]
        .as_array()
        .expect("stock tool catalog should be an array")
        .iter()
        .map(|value| value.as_str().expect("tool name should be a string"))
        .collect::<Vec<_>>();

    assert_eq!(
        foundation,
        excel_skill::tools::catalog::foundation_tool_names()
    );
    assert_eq!(stock, excel_skill::tools::catalog::stock_tool_names());
    assert!(stock.contains(&"technical_consultation_basic"));
    assert!(!foundation.contains(&"technical_consultation_basic"));
}

#[test]
fn grouped_tool_catalog_matches_flat_catalog_without_overlap() {
    let foundation = excel_skill::tools::catalog::foundation_tool_names();
    let stock = excel_skill::tools::catalog::stock_tool_names();
    let mut combined = foundation
        .iter()
        .chain(stock.iter())
        .copied()
        .collect::<Vec<_>>();
    combined.sort_unstable();

    let mut flat = excel_skill::tools::catalog::tool_names().to_vec();
    flat.sort_unstable();

    assert_eq!(combined, flat);
    for tool_name in stock {
        assert!(
            !foundation.contains(tool_name),
            "tool `{tool_name}` should not overlap between foundation and stock groups"
        );
    }
}

#[test]
fn technical_consultation_basic_contract_exposes_bullish_continuation_conclusion() {
    let runtime_db_path =
        create_test_runtime_db("integration_tool_contract_technical_consultation_breakout");
    let csv_path = create_stock_history_csv(
        "integration_tool_contract_technical_consultation_breakout",
        "confirmed_breakout.csv",
        &build_confirmed_breakout_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "688169.SH");

    let request = json!({
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "688169.SH"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);

    // 2026-04-01 CST: 这里先锁外层工具合同里的多头延续输出，原因是 CLI 专项回归虽然已覆盖内部矩阵，
    // 目的：但 `integration_tool_contract` 还没有证明调用方在工具层拿到的确实是完整 `consultation_conclusion` 合同。
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["consultation_conclusion"]["bias"],
        "bullish_continuation"
    );
    assert_eq!(
        output["data"]["consultation_conclusion"]["confidence"],
        "medium"
    );
    assert!(
        output["data"]["consultation_conclusion"]["headline"]
            .as_str()
            .expect("headline should exist")
            .contains("多头延续")
    );
    assert!(
        output["data"]["consultation_conclusion"]["rationale"]
            .as_array()
            .expect("rationale should exist")
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("多头延续") && text.contains("突破后"))
    );
    assert!(
        output["data"]["consultation_conclusion"]["risk_flags"]
            .as_array()
            .expect("risk_flags should exist")
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("突破后回踩"))
    );
}

#[test]
fn technical_consultation_basic_contract_exposes_bearish_continuation_conclusion() {
    let runtime_db_path =
        create_test_runtime_db("integration_tool_contract_technical_consultation_breakdown");
    let csv_path = create_stock_history_csv(
        "integration_tool_contract_technical_consultation_breakdown",
        "confirmed_breakdown.csv",
        &build_confirmed_breakdown_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "600887.SH");

    let request = json!({
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "600887.SH"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);

    // 2026-04-01 CST: 这里补外层工具合同里的空头延续输出，原因是当前合同样本还只锁了多头延续与等待态；
    // 目的：确保调用方在工具层也能稳定拿到对称的 `bearish_continuation` 正式合同，而不是只靠 CLI 专项回归证明空头路径存在。
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["consultation_conclusion"]["bias"],
        "bearish_continuation"
    );
    assert_eq!(
        output["data"]["consultation_conclusion"]["confidence"],
        "medium"
    );
    assert!(
        output["data"]["consultation_conclusion"]["headline"]
            .as_str()
            .expect("headline should exist")
            .contains("空头延续")
    );
    assert!(
        output["data"]["consultation_conclusion"]["rationale"]
            .as_array()
            .expect("rationale should exist")
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("空头延续") && text.contains("跌破后"))
    );
    assert!(
        output["data"]["consultation_conclusion"]["risk_flags"]
            .as_array()
            .expect("risk_flags should exist")
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("跌破后反抽"))
    );
}

#[test]
fn technical_consultation_basic_contract_exposes_range_wait_conclusion() {
    let runtime_db_path =
        create_test_runtime_db("integration_tool_contract_technical_consultation_choppy");
    let csv_path = create_stock_history_csv(
        "integration_tool_contract_technical_consultation_choppy",
        "choppy_prices.csv",
        &build_choppy_history_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "510300.SH");

    let request = json!({
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "510300.SH"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);

    // 2026-04-01 CST: 这里再锁外层工具合同里的等待态输出，原因是外层合同不能只证明方向性样本，
    // 目的：还要证明 `range_wait` 这种中性等待结论同样带完整的解释层和风控层。
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["consultation_conclusion"]["bias"],
        "range_wait"
    );
    assert_eq!(
        output["data"]["consultation_conclusion"]["confidence"],
        "low"
    );
    assert!(
        output["data"]["consultation_conclusion"]["headline"]
            .as_str()
            .expect("headline should exist")
            .contains("区间震荡")
    );
    assert!(
        output["data"]["consultation_conclusion"]["rationale"]
            .as_array()
            .expect("rationale should exist")
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("趋势强度不足") && text.contains("等待区间"))
    );
    assert!(
        output["data"]["consultation_conclusion"]["risk_flags"]
            .as_array()
            .expect("risk_flags should exist")
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("方向确认不足"))
    );
}
