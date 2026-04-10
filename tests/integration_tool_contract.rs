#![recursion_limit = "256"]

mod common;

use chrono::{Duration, NaiveDate};
use excel_skill::ops::stock::security_decision_briefing::PositionPlan;
use excel_skill::tools::contracts::PositionAdjustmentEventType;
use excel_skill::tools::contracts::PositionPlanAlignment;
use excel_skill::tools::contracts::PostTradeReviewDimension;
use excel_skill::tools::contracts::PostTradeReviewOutcome;
use excel_skill::tools::contracts::SecurityPositionPlanRecordResult;
use excel_skill::tools::contracts::SecurityPostTradeReviewResult;
use excel_skill::tools::contracts::SecurityRecordPositionAdjustmentResult;
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
fn explicit_tool_catalog_request_returns_registered_tool_names() {
    // 2026-04-02 CST: 这里补显式 `tool_catalog` 请求回归，原因是当前 catalog 既支持空输入兜底，也被新 stock Tool 测试当作正式 Tool 调用；
    // 目的：锁住“显式请求”和“空输入兜底”返回同一份目录合同，避免后续只保住默认入口却让 dispatcher 路由悄悄漂移。
    let output = run_cli_with_json(r#"{"tool":"tool_catalog","args":{}}"#);
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

// 2026-04-10 CST: 这里补 foundation repository metadata audit 的 catalog 红测，原因是当前 repository audit 还停留在内部 API，
// 方案A要求把它正式提升为 foundation Tool；目的：先锁住“工具可发现”这层对外契约，避免后续只接入 dispatcher 却漏掉 catalog 主线。
#[test]
fn foundation_repository_metadata_audit_is_cataloged_in_foundation_group() {
    let response = ToolResponse::tool_catalog();
    let tool_catalog = response.data["tool_catalog"]
        .as_array()
        .expect("tool catalog should be an array");
    let foundation_catalog = response.data["tool_catalog_modules"]["foundation"]
        .as_array()
        .expect("foundation tool group should be an array");
    let stock_catalog = response.data["tool_catalog_modules"]["stock"]
        .as_array()
        .expect("stock tool group should be an array");

    assert!(
        tool_catalog
            .iter()
            .filter_map(|item| item.as_str())
            .any(|item| item == "foundation_repository_metadata_audit")
    );
    assert!(
        foundation_catalog
            .iter()
            .filter_map(|item| item.as_str())
            .any(|item| item == "foundation_repository_metadata_audit")
    );
    assert!(
        !stock_catalog
            .iter()
            .filter_map(|item| item.as_str())
            .any(|item| item == "foundation_repository_metadata_audit")
    );
}

// 2026-04-10 CST: 这里补 foundation repository metadata audit gate 的 catalog 红测，原因是下一阶段要把 repository audit
// 从“报告工具”推进到“流程消费层”，因此 gate 入口也必须先进入正式 foundation 目录；目的：锁住可发现性，避免只做内部 helper。
#[test]
fn foundation_repository_metadata_audit_gate_is_cataloged_in_foundation_group() {
    let response = ToolResponse::tool_catalog();
    let tool_catalog = response.data["tool_catalog"]
        .as_array()
        .expect("tool catalog should be an array");
    let foundation_catalog = response.data["tool_catalog_modules"]["foundation"]
        .as_array()
        .expect("foundation tool group should be an array");
    let stock_catalog = response.data["tool_catalog_modules"]["stock"]
        .as_array()
        .expect("stock tool group should be an array");

    assert!(
        tool_catalog
            .iter()
            .filter_map(|item| item.as_str())
            .any(|item| item == "foundation_repository_metadata_audit_gate")
    );
    assert!(
        foundation_catalog
            .iter()
            .filter_map(|item| item.as_str())
            .any(|item| item == "foundation_repository_metadata_audit_gate")
    );
    assert!(
        !stock_catalog
            .iter()
            .filter_map(|item| item.as_str())
            .any(|item| item == "foundation_repository_metadata_audit_gate")
    );
}

// 2026-04-10 CST: 这里补 foundation repository metadata audit batch 的 catalog 红测，原因是 A1 要把单仓库 gate 向上提升为批量入口，
// 目的：先锁住 batch 入口也属于正式 foundation Tool，而不是只停留在内部批处理函数。
#[test]
fn foundation_repository_metadata_audit_batch_is_cataloged_in_foundation_group() {
    let response = ToolResponse::tool_catalog();
    let tool_catalog = response.data["tool_catalog"]
        .as_array()
        .expect("tool catalog should be an array");
    let foundation_catalog = response.data["tool_catalog_modules"]["foundation"]
        .as_array()
        .expect("foundation tool group should be an array");
    let stock_catalog = response.data["tool_catalog_modules"]["stock"]
        .as_array()
        .expect("stock tool group should be an array");

    assert!(
        tool_catalog
            .iter()
            .filter_map(|item| item.as_str())
            .any(|item| item == "foundation_repository_metadata_audit_batch")
    );
    assert!(
        foundation_catalog
            .iter()
            .filter_map(|item| item.as_str())
            .any(|item| item == "foundation_repository_metadata_audit_batch")
    );
    assert!(
        !stock_catalog
            .iter()
            .filter_map(|item| item.as_str())
            .any(|item| item == "foundation_repository_metadata_audit_batch")
    );
}

// 2026-04-10 CST: 这里补 foundation repository import gate 的 catalog 红测，原因是方案B1要把 batch 消费层正式提升为“导入接入 gate”入口，
// 目的：先锁住该入口属于 foundation 通用目录，而不是让上层继续手工解释 batch 结果。
#[test]
fn foundation_repository_import_gate_is_cataloged_in_foundation_group() {
    let response = ToolResponse::tool_catalog();
    let tool_catalog = response.data["tool_catalog"]
        .as_array()
        .expect("tool catalog should be an array");
    let foundation_catalog = response.data["tool_catalog_modules"]["foundation"]
        .as_array()
        .expect("foundation tool group should be an array");
    let stock_catalog = response.data["tool_catalog_modules"]["stock"]
        .as_array()
        .expect("stock tool group should be an array");

    assert!(
        tool_catalog
            .iter()
            .filter_map(|item| item.as_str())
            .any(|item| item == "foundation_repository_import_gate")
    );
    assert!(
        foundation_catalog
            .iter()
            .filter_map(|item| item.as_str())
            .any(|item| item == "foundation_repository_import_gate")
    );
    assert!(
        !stock_catalog
            .iter()
            .filter_map(|item| item.as_str())
            .any(|item| item == "foundation_repository_import_gate")
    );
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
    // 2026-04-08 CST: 这里先补顶层日期与证据版本红测，原因是方案 C 第一批要把证券分析链公共合同收口到统一字段；
    // 目的：锁定 technical_consultation_basic 不再只暴露 as_of_date，而是对外也提供统一 analysis_date / evidence_version。
    assert_eq!(
        output["data"]["analysis_date"],
        output["data"]["data_window_summary"]["end_date"]
    );
    assert!(
        output["data"].get("as_of_date").is_none(),
        "technical_consultation_basic should no longer expose legacy as_of_date"
    );
    assert_eq!(
        output["data"]["evidence_version"],
        format!(
            "technical-consultation-basic:688169.SH:{}:v1",
            output["data"]["analysis_date"]
                .as_str()
                .expect("analysis_date should exist")
        )
    );
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

#[test]
fn security_decision_briefing_contract_exposes_required_top_level_fields() {
    // 2026-04-02 CST: 这里先用最小夹具锁定 security_decision_briefing 的正式对外合同，原因是本轮第一步只需要先把 briefing
    // Tool 的字段边界钉住，再继续做 assembler 和 dispatcher；目的：避免后续实现过程中字段名漂移，导致咨询场景与投决场景拿到不同口径。
    let serialized = json!({
        "symbol": "600519.SH",
        "analysis_date": "2026-04-02",
        "summary": "结构化简报合同样例",
        "evidence_version": "briefing-contract-v1",
        "fundamental_brief": {
            "headline": "财报叙事样例",
            "profit_signal": "stable_growth"
        },
        "technical_brief": {
            "consultation_conclusion": {
                "bias": "bullish_continuation"
            }
        },
        "resonance_brief": {
            "resonance_score": 0.72,
            "action_bias": "add_on_strength"
        },
        "execution_plan": {
            "add_trigger_price": 1510.0,
            "add_trigger_volume_ratio": 1.35,
            "add_position_pct": 0.12,
            "reduce_trigger_price": 1460.0,
            "rejection_zone": "1495-1510",
            "reduce_position_pct": 0.08,
            "stop_loss_price": 1420.0,
            "invalidation_price": 1398.0,
            "watch_points": ["量价共振是否延续"],
            "explanation": ["execution_plan 样例仅用于锁定合同字段"]
        },
        "odds_brief": {
            "status": "available",
            "historical_confidence": "medium",
            "sample_count": 12,
            "win_rate_10d": 0.58,
            "loss_rate_10d": 0.33,
            "flat_rate_10d": 0.09,
            "avg_return_10d": 0.041,
            "median_return_10d": 0.036,
            "avg_win_return_10d": 0.086,
            "avg_loss_return_10d": -0.041,
            "payoff_ratio_10d": 2.10,
            "expectancy_10d": 0.036,
            "expected_return_window": "10日收益均值 4.10%，中位数 3.60%",
            "expected_drawdown_window": "10日回撤均值 -2.30%，中位数 -2.10%",
            "odds_grade": "B",
            "confidence_grade": "medium",
            "rationale": ["赔率层合同样例仅用于锁定字段"],
            "research_limitations": []
        },
        "position_plan": {
            "position_action": "starter_then_confirm",
            "entry_mode": "breakout_confirmation",
            "starter_position_pct": 0.10,
            "max_position_pct": 0.22,
            "add_on_trigger": "站上关键位且放量后再追加",
            "reduce_on_trigger": "跌破短承接位先减仓",
            "hard_stop_trigger": "跌破失效位退出",
            "liquidity_cap": "单次执行不超过计划仓位的 50%",
            "position_risk_grade": "medium",
            "regime_adjustment": "若市场共振转弱则整体降一级仓位",
            "execution_notes": ["position_plan 样例仅用于锁定合同字段"],
            "rationale": ["仓位层合同样例仅用于锁定字段"]
        },
        "committee_payload": {
            "symbol": "600519.SH",
            "analysis_date": "2026-04-02",
            "recommended_action": "hold_and_confirm",
            "confidence": "medium",
            "key_risks": ["高位放量回撤"],
            "risk_breakdown": {
                "technical": [
                    {
                        "category": "technical",
                        "severity": "medium",
                        "headline": "高位放量回撤",
                        "rationale": "价格仍处于高位震荡，若量价背离扩大则需要重新评估突破有效性。"
                    }
                ],
                "fundamental": [],
                "resonance": [],
                "execution": []
            },
            "minority_objection_points": ["尚未突破关键阻力"],
            "evidence_version": "briefing-contract-v1",
            "briefing_digest": "结构化简报摘要样例",
            "committee_schema_version": "committee-payload:v1",
            "recommendation_digest": {
                "final_stance": "watchful_positive",
                "action_bias": "hold_and_confirm",
                "summary": "结构化简报摘要样例",
                "confidence": "medium"
            },
            "execution_digest": {
                "add_trigger_price": 1510.0,
                "add_trigger_volume_ratio": 1.35,
                "add_position_pct": 0.12,
                "reduce_trigger_price": 1460.0,
                "reduce_position_pct": 0.08,
                "stop_loss_price": 1420.0,
                "invalidation_price": 1398.0,
                "rejection_zone": "1495-1510",
                "watch_points": ["量价共振是否延续"],
                "explanation": ["execution_plan 样例仅用于锁定合同字段"]
            },
            "resonance_digest": {
                "resonance_score": 0.72,
                "action_bias": "hold_and_confirm",
                "top_positive_driver_names": ["行业景气"],
                "top_negative_driver_names": ["高位回撤"],
                "event_override_titles": ["政策观察"]
            },
            "evidence_checks": {
                "fundamental_ready": true,
                "technical_ready": true,
                "resonance_ready": true,
                "execution_ready": true,
                "briefing_ready": true
            },
            "historical_digest": {
                "status": "available",
                "historical_confidence": "medium",
                "analog_sample_count": 12,
                "analog_win_rate_10d": 0.58,
                "analog_loss_rate_10d": 0.33,
                "analog_flat_rate_10d": 0.09,
                "analog_avg_return_10d": 0.041,
                "analog_median_return_10d": 0.036,
                "analog_avg_win_return_10d": 0.086,
                "analog_avg_loss_return_10d": -0.041,
                "analog_payoff_ratio_10d": 2.10,
                "analog_expectancy_10d": 0.036,
                "expected_return_window": "10日收益均值 4.10%，中位数 3.60%",
                "expected_drawdown_window": "10日回撤均值 -2.30%，中位数 -2.10%",
                "research_limitations": []
            },
            "odds_digest": {
                "status": "available",
                "historical_confidence": "medium",
                "sample_count": 12,
                "win_rate_10d": 0.58,
                "loss_rate_10d": 0.33,
                "flat_rate_10d": 0.09,
                "avg_return_10d": 0.041,
                "median_return_10d": 0.036,
                "avg_win_return_10d": 0.086,
                "avg_loss_return_10d": -0.041,
                "payoff_ratio_10d": 2.10,
                "expectancy_10d": 0.036,
                "expected_return_window": "10日收益均值 4.10%，中位数 3.60%",
                "expected_drawdown_window": "10日回撤均值 -2.30%，中位数 -2.10%",
                "odds_grade": "B",
                "confidence_grade": "medium",
                "rationale": ["赔率层合同样例仅用于锁定字段"],
                "research_limitations": []
            },
            "position_digest": {
                "position_action": "starter_then_confirm",
                "entry_mode": "breakout_confirmation",
                "starter_position_pct": 0.10,
                "max_position_pct": 0.22,
                "add_on_trigger": "站上关键位且放量后再追加",
                "reduce_on_trigger": "跌破短承接位先减仓",
                "hard_stop_trigger": "跌破失效位退出",
                "liquidity_cap": "单次执行不超过计划仓位的 50%",
                "position_risk_grade": "medium",
                "regime_adjustment": "若市场共振转弱则整体降一级仓位",
                "execution_notes": ["position_plan 样例仅用于锁定合同字段"],
                "rationale": ["仓位层合同样例仅用于锁定字段"]
            }
        }
    });

    for field_name in [
        "summary",
        "fundamental_brief",
        "technical_brief",
        "resonance_brief",
        "execution_plan",
        "odds_brief",
        "position_plan",
        "committee_payload",
    ] {
        assert!(
            serialized.get(field_name).is_some(),
            "security_decision_briefing should expose top-level field `{field_name}`"
        );
    }

    for field_name in [
        "recommended_action",
        "confidence",
        "key_risks",
        "risk_breakdown",
        "evidence_version",
        "odds_digest",
        "position_digest",
    ] {
        assert!(
            serialized["committee_payload"].get(field_name).is_some(),
            "committee_payload should expose field `{field_name}`"
        );
    }
}

#[test]
fn security_decision_briefing_is_cataloged() {
    let output = run_cli_with_json("");
    let tool_catalog = output["data"]["tool_catalog"]
        .as_array()
        .expect("tool_catalog should be an array")
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>();
    let stock_catalog = output["data"]["tool_catalog_modules"]["stock"]
        .as_array()
        .expect("stock tool catalog should be an array")
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>();

    // 2026-04-02 CST: 这里先锁 security_decision_briefing 的目录可见性，原因是 Task 4 的目标是把 briefing Tool 正式接入
    // catalog / dispatcher 主链；目的：防止后续只在 ops 层存在实现，但外层 CLI / Skill 仍无法发现或路由到它。
    assert!(
        tool_catalog.contains(&"security_decision_briefing"),
        "tool_catalog should include security_decision_briefing"
    );
    assert!(
        stock_catalog.contains(&"security_decision_briefing"),
        "stock tool group should include security_decision_briefing"
    );
}

#[test]
fn security_position_plan_record_contract_exposes_required_fields() {
    // 2026-04-08 CST: 这里先补仓位计划记录正式合同红测，原因是投后复盘闭环的第一步必须先把 position_plan
    // 从 briefing 子层建议升级成正式可引用对象；目的：锁定后续 position_plan_record 至少要暴露引用字段与核心仓位边界，避免实现时字段漂移。
    let result = SecurityPositionPlanRecordResult {
        position_plan_ref: "position-plan:600519.SH:2026-04-08:v1".to_string(),
        decision_ref: "decision:600519.SH:2026-04-08:v1".to_string(),
        approval_ref: "approval:600519.SH:2026-04-08:v1".to_string(),
        evidence_version: "briefing:600519.SH:2026-04-08:v1".to_string(),
        symbol: "600519.SH".to_string(),
        analysis_date: "2026-04-08".to_string(),
        position_action: "starter_then_confirm".to_string(),
        starter_position_pct: 0.10,
        max_position_pct: 0.22,
        position_plan: PositionPlan {
            position_action: "starter_then_confirm".to_string(),
            entry_mode: "breakout_confirmation".to_string(),
            starter_position_pct: 0.10,
            max_position_pct: 0.22,
            add_on_trigger: "站上关键位且放量后再追加".to_string(),
            reduce_on_trigger: "跌破短承接位先减仓".to_string(),
            hard_stop_trigger: "跌破失效位退出".to_string(),
            liquidity_cap: "单次执行不超过计划仓位的 50%".to_string(),
            position_risk_grade: "medium".to_string(),
            regime_adjustment: "若市场共振转弱则整体降一级仓位".to_string(),
            execution_notes: vec!["position_plan_record 样例仅用于锁定合同字段".to_string()],
            rationale: vec!["position_plan_record 合同样例仅用于锁定字段".to_string()],
        },
    };

    let serialized =
        serde_json::to_value(&result).expect("position plan record contract should serialize");

    for field_name in [
        "position_plan_ref",
        "decision_ref",
        "approval_ref",
        "evidence_version",
        "symbol",
        "analysis_date",
        "position_action",
        "starter_position_pct",
        "max_position_pct",
    ] {
        assert!(
            serialized.get(field_name).is_some(),
            "security_position_plan_record should expose field `{field_name}`"
        );
    }
}

#[test]
fn security_record_position_adjustment_contract_exposes_required_fields() {
    // 2026-04-08 CST: 这里先补调仓事件正式合同红测，原因是 Task 3 要先把“多次调仓记录”的最小对象边界锁死，
    // 目的：确保后续 security_record_position_adjustment Tool 实装时，事件引用、仓位变化和计划偏离口径不会在实现阶段漂移。
    let result = SecurityRecordPositionAdjustmentResult {
        adjustment_event_ref: "position-adjustment:600519.SH:2026-04-09:reduce:v1".to_string(),
        decision_ref: "decision:600519.SH:2026-04-08:v1".to_string(),
        approval_ref: "approval:600519.SH:2026-04-08:v1".to_string(),
        evidence_version: "briefing:600519.SH:2026-04-08:v1".to_string(),
        position_plan_ref: "position-plan:600519.SH:2026-04-08:v1".to_string(),
        symbol: "600519.SH".to_string(),
        event_type: PositionAdjustmentEventType::Reduce,
        event_date: "2026-04-09".to_string(),
        before_position_pct: 0.22,
        after_position_pct: 0.14,
        trigger_reason: "跌破减仓线，按计划先降回试错仓位".to_string(),
        plan_alignment: PositionPlanAlignment::JustifiedDeviation,
    };

    let serialized =
        serde_json::to_value(&result).expect("position adjustment event contract should serialize");

    for field_name in [
        "adjustment_event_ref",
        "position_plan_ref",
        "event_type",
        "event_date",
        "before_position_pct",
        "after_position_pct",
        "trigger_reason",
        "plan_alignment",
    ] {
        assert!(
            serialized.get(field_name).is_some(),
            "security_record_position_adjustment should expose field `{field_name}`"
        );
    }

    assert_eq!(serialized["event_type"], "reduce");
    assert_eq!(serialized["plan_alignment"], "justified_deviation");
}

#[test]
fn security_post_trade_review_contract_exposes_required_fields() {
    // 2026-04-08 CST: 这里先补投后复盘正式合同红测，原因是 Task 5 需要先把复盘对象的最小字段边界钉住；
    // 目的：保证后续 security_post_trade_review Tool 落地时，能够稳定回指 plan / decision / approval，并输出统一复盘维度。
    let result = SecurityPostTradeReviewResult {
        post_trade_review_ref: "post-trade-review:600519.SH:2026-04-15:v1".to_string(),
        position_plan_ref: "position-plan:600519.SH:2026-04-08:v1".to_string(),
        decision_ref: "decision:600519.SH:2026-04-08:v1".to_string(),
        approval_ref: "approval:600519.SH:2026-04-08:v1".to_string(),
        evidence_version: "briefing:600519.SH:2026-04-08:v1".to_string(),
        symbol: "600519.SH".to_string(),
        analysis_date: "2026-04-15".to_string(),
        adjustment_event_refs: vec![
            "position-adjustment:600519.SH:2026-04-09:build:v1".to_string(),
            "position-adjustment:600519.SH:2026-04-11:reduce:v1".to_string(),
        ],
        review_outcome: PostTradeReviewOutcome::Mixed,
        decision_accuracy: PostTradeReviewDimension::Acceptable,
        execution_quality: PostTradeReviewDimension::Strong,
        risk_control_quality: PostTradeReviewDimension::Weak,
        correction_actions: vec![
            "缩小首次建仓比例，避免确认前过早打满计划仓位".to_string(),
            "若再次放量冲高回落，优先执行计划内减仓".to_string(),
        ],
        next_cycle_guidance: vec![
            "下一轮只在量价共振重建后恢复加仓".to_string(),
            "继续对照 position_plan_ref 记录偏离原因".to_string(),
        ],
    };

    let serialized =
        serde_json::to_value(&result).expect("post trade review contract should serialize");

    for field_name in [
        "post_trade_review_ref",
        "position_plan_ref",
        "decision_ref",
        "approval_ref",
        "review_outcome",
        "decision_accuracy",
        "execution_quality",
        "risk_control_quality",
        "correction_actions",
        "next_cycle_guidance",
    ] {
        assert!(
            serialized.get(field_name).is_some(),
            "security_post_trade_review should expose field `{field_name}`"
        );
    }

    assert_eq!(serialized["review_outcome"], "mixed");
    assert_eq!(serialized["decision_accuracy"], "acceptable");
    assert_eq!(serialized["execution_quality"], "strong");
    assert_eq!(serialized["risk_control_quality"], "weak");
}

#[test]
fn signal_outcome_platform_contract_exposes_research_tools_and_required_fields() {
    let flat_catalog = excel_skill::tools::catalog::tool_names();
    let stock_catalog = excel_skill::tools::catalog::stock_tool_names();

    // 2026-04-02 CST：这里先锁研究平台 Tool 家族的可发现性，原因是方案C要求把“快照 -> forward returns -> analog study”
    // 做成正式平台入口，而不是只留在内部 helper；目的：保证后续咨询、briefing 和投决会都能基于统一 Tool 主链读取研究结果。
    for tool_name in [
        "record_security_signal_snapshot",
        "backfill_security_signal_outcomes",
        "study_security_signal_analogs",
        "signal_outcome_research_summary",
    ] {
        assert!(
            flat_catalog.contains(&tool_name),
            "tool catalog should include `{tool_name}`"
        );
        assert!(
            stock_catalog.contains(&tool_name),
            "stock tool catalog should include `{tool_name}`"
        );
    }

    let signal_outcome_contract = json!({
        "snapshot": {
            "symbol": "600519.SH",
            "snapshot_date": "2026-04-02",
            "indicator_digest": "adx=31.2|rsi=63.4|rsrs=0.58",
            "resonance_score": 0.73,
            "action_bias": "hold_and_confirm"
        },
        "forward_returns": [
            {
                "horizon_days": 5,
                "forward_return_pct": 0.042,
                "max_drawdown_pct": -0.018
            }
        ],
        "analog_summary": {
            "sample_count": 24,
            "win_rate": 0.625,
            "avg_return_pct": 0.031,
            "median_return_pct": 0.024
        }
    });

    for field_name in [
        "symbol",
        "snapshot_date",
        "indicator_digest",
        "resonance_score",
        "action_bias",
    ] {
        assert!(
            signal_outcome_contract["snapshot"]
                .get(field_name)
                .is_some(),
            "snapshot contract should expose `{field_name}`"
        );
    }

    for field_name in ["horizon_days", "forward_return_pct", "max_drawdown_pct"] {
        assert!(
            signal_outcome_contract["forward_returns"][0]
                .get(field_name)
                .is_some(),
            "forward return contract should expose `{field_name}`"
        );
    }

    for field_name in [
        "sample_count",
        "win_rate",
        "avg_return_pct",
        "median_return_pct",
    ] {
        assert!(
            signal_outcome_contract["analog_summary"]
                .get(field_name)
                .is_some(),
            "analog summary contract should expose `{field_name}`"
        );
    }
}
