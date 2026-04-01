mod common;

use chrono::{Duration, NaiveDate};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;

use crate::common::{create_test_runtime_db, run_cli_with_json, run_cli_with_json_and_runtime};

// 2026-03-28 CST: 这里新增技术面基础 Tool 的 CSV 测试夹具生成助手，原因是本轮要先用真实 CLI 链路锁住
// `CSV -> SQLite -> technical_consultation_basic` 的端到端合同；目的：避免只测内部函数而漏掉 Rust / exe 主线接入问题。
fn create_stock_history_csv(prefix: &str, file_name: &str, rows: &[String]) -> PathBuf {
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let fixture_dir = PathBuf::from("tests")
        .join("runtime_fixtures")
        .join("technical_consultation_basic")
        .join(format!("{prefix}_{unique_suffix}"));
    fs::create_dir_all(&fixture_dir).expect("technical consultation fixture dir should be created");

    let csv_path = fixture_dir.join(file_name);
    fs::write(&csv_path, rows.join("\n")).expect("technical consultation csv should be written");
    csv_path
}

#[test]
fn technical_consultation_basic_marks_mfi_overbought_distribution() {
    let runtime_db_path = create_test_runtime_db("technical_consultation_basic_mfi_overbought");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_mfi_overbought",
        "mfi_overbought.csv",
        &build_mfi_overbought_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "603259.SH");

    let request = json!({
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "603259.SH"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);

    // 2026-03-29 09:35 CST: 这里先锁 MFI“高位资金过热”红测，原因是资金流能力第一版要先把顶部极值场景钉成正式合同；
    // 目的：确保 technical_consultation_basic 会同时暴露 `money_flow_signal` 与 `indicator_snapshot.mfi_14`，而不是只补一个内部数值。
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["money_flow_signal"],
        "overbought_distribution"
    );
    assert!(
        output["data"]["indicator_snapshot"]["mfi_14"]
            .as_f64()
            .expect("mfi_14 should be a number")
            >= 80.0
    );
}

#[test]
fn technical_consultation_basic_marks_mfi_oversold_accumulation() {
    let runtime_db_path = create_test_runtime_db("technical_consultation_basic_mfi_oversold");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_mfi_oversold",
        "mfi_oversold.csv",
        &build_mfi_oversold_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "300274.SZ");

    let request = json!({
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "300274.SZ"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);

    // 2026-03-29 09:35 CST: 这里再锁 MFI“低位资金超卖”红测，原因是方案 A 的中级指标必须成对覆盖高低两端；
    // 目的：确保 oversold 场景能在现有 Rust / SQLite 主链里稳定落到 `oversold_accumulation`，避免后续只剩单边信号。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["money_flow_signal"], "oversold_accumulation");
    assert!(
        output["data"]["indicator_snapshot"]["mfi_14"]
            .as_f64()
            .expect("mfi_14 should be a number")
            <= 20.0
    );
}

#[test]
fn technical_consultation_basic_keeps_mfi_neutral_in_balanced_swings() {
    let runtime_db_path = create_test_runtime_db("technical_consultation_basic_mfi_neutral");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_mfi_neutral",
        "mfi_neutral.csv",
        &build_mfi_neutral_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "600036.SH");

    let request = json!({
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "600036.SH"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);
    let mfi_14 = output["data"]["indicator_snapshot"]["mfi_14"]
        .as_f64()
        .expect("mfi_14 should be a number");

    // 2026-03-29 09:35 CST: 这里补 MFI 中性边界红测，原因是只有极端夹具会让阈值调整后更难发现误报问题；
    // 目的：把“震荡均衡 -> neutral”明确钉住，避免后续实现把所有资金流样本都粗暴挤压成 80/20 两端。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["money_flow_signal"], "neutral");
    assert!(mfi_14 > 20.0 && mfi_14 < 80.0);
}

#[test]
fn technical_consultation_basic_keeps_mfi_neutral_in_mixed_volume_swings() {
    let runtime_db_path =
        create_test_runtime_db("technical_consultation_basic_mfi_mixed_volume_neutral");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_mfi_mixed_volume_neutral",
        "mfi_mixed_volume_neutral.csv",
        &build_mfi_mixed_volume_neutral_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "600887.SH");

    let request = json!({
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "600887.SH"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);
    let mfi_14 = output["data"]["indicator_snapshot"]["mfi_14"]
        .as_f64()
        .expect("mfi_14 should be a number");
    let summary = output["data"]["summary"]
        .as_str()
        .expect("summary should exist");
    let watch_points = output["data"]["watch_points"]
        .as_array()
        .expect("watch_points should exist");

    // 2026-03-29 10:20 CST: 这里新增 MFI“混合量能但不应误判极端”的 CLI 回归，原因是上一轮只锁了恒定量能 neutral，
    // 目的：补上真实成交量忽大忽小的历史样本，确保 `money_flow_signal`、`summary` 与 `watch_points` 在 mixed-volume 场景下仍保持中性语义。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["money_flow_signal"], "neutral");
    assert!(mfi_14 > 20.0 && mfi_14 < 80.0);
    assert!(summary.contains("MFI"));
    assert!(summary.contains("中性"));
    assert!(
        watch_points
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("MFI") && text.contains("中性区间"))
    );
}

#[test]
fn technical_consultation_basic_keeps_mfi_finite_when_volume_is_zero() {
    let runtime_db_path = create_test_runtime_db("technical_consultation_basic_mfi_zero_volume");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_mfi_zero_volume",
        "mfi_zero_volume.csv",
        &build_mfi_zero_volume_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "601398.SH");

    let request = json!({
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "601398.SH"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);
    let mfi_14 = output["data"]["indicator_snapshot"]["mfi_14"]
        .as_f64()
        .expect("mfi_14 should be a number");

    // 2026-03-29 09:35 CST: 这里补零量边界红测，原因是 EXE 主链必须在客户常见脏数据边界下保持稳定输出；
    // 目的：确保 `volume = 0` 与平盘窗口不会把 MFI 算成 NaN / inf，也不会让咨询结果缺失资金流字段。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["money_flow_signal"], "neutral");
    assert!(mfi_14.is_finite());
}

// 2026-03-28 CST: 这里生成足量上涨行情样本，原因是第一条红测要稳定触发多头趋势、正向动量和非空指标快照；
// 目的：先把 `technical_consultation_basic` 的最小成功合同钉住，后续实现时只允许往这份合同对齐。
#[test]
fn technical_consultation_basic_marks_cci_overbought_reversal_risk() {
    let runtime_db_path = create_test_runtime_db("technical_consultation_basic_cci_overbought");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_cci_overbought",
        "cci_overbought.csv",
        &build_cci_overbought_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "300124.SZ");

    let request = json!({
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "300124.SZ"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);
    let summary = output["data"]["summary"]
        .as_str()
        .expect("summary should exist");
    let recommended_actions = output["data"]["recommended_actions"]
        .as_array()
        .expect("recommended_actions should exist");
    let watch_points = output["data"]["watch_points"]
        .as_array()
        .expect("watch_points should exist");

    // 2026-03-30 09:20 CST: 这里新增 CCI“高位偏离过大”红测，原因是用户已批准先做 CCI(20) 第一版，并要求沿现有正式合同接入；
    // 目的：确保 CLI 结果会同时暴露 `mean_reversion_signal` 与 `indicator_snapshot.cci_20`，并把均值回归风险写进 summary / actions / watch_points。
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["mean_reversion_signal"],
        "overbought_reversal_risk"
    );
    assert!(
        output["data"]["indicator_snapshot"]["cci_20"]
            .as_f64()
            .expect("cci_20 should be a number")
            >= 100.0
    );
    assert!(summary.contains("CCI"));
    assert!(summary.contains("均值回归"));
    assert!(
        recommended_actions
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("CCI"))
    );
    assert!(
        watch_points
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("CCI"))
    );
}

#[test]
fn technical_consultation_basic_marks_cci_oversold_rebound_candidate() {
    let runtime_db_path = create_test_runtime_db("technical_consultation_basic_cci_oversold");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_cci_oversold",
        "cci_oversold.csv",
        &build_cci_oversold_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "002475.SZ");

    let request = json!({
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "002475.SZ"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);
    let summary = output["data"]["summary"]
        .as_str()
        .expect("summary should exist");
    let recommended_actions = output["data"]["recommended_actions"]
        .as_array()
        .expect("recommended_actions should exist");
    let watch_points = output["data"]["watch_points"]
        .as_array()
        .expect("watch_points should exist");

    // 2026-03-30 09:20 CST: 这里新增 CCI“低位偏离过大”红测，原因是 CCI 第一版不能只锁上沿，不锁下沿会让均值回归信号失衡；
    // 目的：确保 `cci_20 <= -100` 时正式进入 `oversold_rebound_candidate`，并把低位反抽候选语义传到完整咨询输出。
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["mean_reversion_signal"],
        "oversold_rebound_candidate"
    );
    assert!(
        output["data"]["indicator_snapshot"]["cci_20"]
            .as_f64()
            .expect("cci_20 should be a number")
            <= -100.0
    );
    assert!(summary.contains("CCI"));
    assert!(summary.contains("均值回归"));
    assert!(
        recommended_actions
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("CCI"))
    );
    assert!(
        watch_points
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("CCI"))
    );
}

#[test]
fn technical_consultation_basic_keeps_cci_neutral_in_balanced_range() {
    let runtime_db_path = create_test_runtime_db("technical_consultation_basic_cci_neutral");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_cci_neutral",
        "cci_neutral.csv",
        &build_cci_neutral_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "600703.SH");

    let request = json!({
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "600703.SH"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);
    let cci_20 = output["data"]["indicator_snapshot"]["cci_20"]
        .as_f64()
        .expect("cci_20 should be a number");
    let summary = output["data"]["summary"]
        .as_str()
        .expect("summary should exist");
    let watch_points = output["data"]["watch_points"]
        .as_array()
        .expect("watch_points should exist");

    // 2026-03-30 09:20 CST: 这里新增 CCI 中性区间红测，原因是均值回归类指标如果只测两端极值，很容易把大部分震荡样本误推向阈值边缘；
    // 目的：把 `-100 < cci_20 < 100 -> neutral` 锁进 CLI 合同，并确认中性说明会写进摘要与观察点而不是只剩一个数值。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["mean_reversion_signal"], "neutral");
    assert!(cci_20 > -100.0 && cci_20 < 100.0);
    assert!(summary.contains("CCI"));
    assert!(summary.contains("中性"));
    assert!(
        watch_points
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("CCI") && text.contains("中性区间"))
    );
}

#[test]
fn technical_consultation_basic_marks_williams_r_overbought_pullback_risk() {
    let runtime_db_path =
        create_test_runtime_db("technical_consultation_basic_williams_r_overbought");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_williams_r_overbought",
        "williams_r_overbought.csv",
        &build_williams_r_overbought_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "300433.SZ");

    let request = json!({
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "300433.SZ"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);
    let summary = output["data"]["summary"]
        .as_str()
        .expect("summary should exist");
    let recommended_actions = output["data"]["recommended_actions"]
        .as_array()
        .expect("recommended_actions should exist");
    let watch_points = output["data"]["watch_points"]
        .as_array()
        .expect("watch_points should exist");

    // 2026-03-30 10:20 CST: 这里新增 Williams %R“高位区间超买”红测，原因是用户已批准先做 Williams %R(14) 第一版；
    // 目的：确保 CLI 结果会同时暴露 `range_position_signal` 与 `indicator_snapshot.williams_r_14`，并把区间高位回落风险写进 summary / actions / watch_points。
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["range_position_signal"],
        "overbought_pullback_risk"
    );
    assert!(
        output["data"]["indicator_snapshot"]["williams_r_14"]
            .as_f64()
            .expect("williams_r_14 should be a number")
            >= -20.0
    );
    assert!(summary.contains("Williams %R"));
    assert!(summary.contains("区间"));
    assert!(
        recommended_actions
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("Williams %R"))
    );
    assert!(
        watch_points
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("Williams %R"))
    );
}

#[test]
fn technical_consultation_basic_marks_williams_r_oversold_rebound_candidate() {
    let runtime_db_path =
        create_test_runtime_db("technical_consultation_basic_williams_r_oversold");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_williams_r_oversold",
        "williams_r_oversold.csv",
        &build_williams_r_oversold_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "002273.SZ");

    let request = json!({
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "002273.SZ"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);
    let summary = output["data"]["summary"]
        .as_str()
        .expect("summary should exist");
    let recommended_actions = output["data"]["recommended_actions"]
        .as_array()
        .expect("recommended_actions should exist");
    let watch_points = output["data"]["watch_points"]
        .as_array()
        .expect("watch_points should exist");

    // 2026-03-30 10:20 CST: 这里新增 Williams %R“低位区间超卖”红测，原因是 Williams %R 第一版不能只锁高位，不锁低位会让区间信号失衡；
    // 目的：确保 `williams_r_14 <= -80` 时正式进入 `oversold_rebound_candidate`，并把低位反抽候选语义传到完整咨询输出。
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["range_position_signal"],
        "oversold_rebound_candidate"
    );
    assert!(
        output["data"]["indicator_snapshot"]["williams_r_14"]
            .as_f64()
            .expect("williams_r_14 should be a number")
            <= -80.0
    );
    assert!(summary.contains("Williams %R"));
    assert!(summary.contains("区间"));
    assert!(
        recommended_actions
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("Williams %R"))
    );
    assert!(
        watch_points
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("Williams %R"))
    );
}

#[test]
fn technical_consultation_basic_keeps_williams_r_neutral_in_balanced_range() {
    let runtime_db_path = create_test_runtime_db("technical_consultation_basic_williams_r_neutral");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_williams_r_neutral",
        "williams_r_neutral.csv",
        &build_williams_r_neutral_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "600188.SH");

    let request = json!({
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "600188.SH"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);
    let williams_r_14 = output["data"]["indicator_snapshot"]["williams_r_14"]
        .as_f64()
        .expect("williams_r_14 should be a number");
    let summary = output["data"]["summary"]
        .as_str()
        .expect("summary should exist");
    let watch_points = output["data"]["watch_points"]
        .as_array()
        .expect("watch_points should exist");

    // 2026-03-30 10:20 CST: 这里新增 Williams %R 中性区间红测，原因是区间位置类指标如果只测两端极值，很容易把震荡样本误判到上下沿；
    // 目的：把 `-80 < williams_r_14 < -20 -> neutral` 锁进 CLI 合同，并确认中性说明会写进摘要与观察点而不是只剩一个数值。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["range_position_signal"], "neutral");
    assert!(williams_r_14 > -80.0 && williams_r_14 < -20.0);
    assert!(summary.contains("Williams %R"));
    assert!(summary.contains("中性"));
    assert!(
        watch_points
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("Williams %R") && text.contains("中性区间"))
    );
}

#[test]
fn technical_consultation_basic_marks_bollinger_upper_band_breakout_risk() {
    let runtime_db_path =
        create_test_runtime_db("technical_consultation_basic_bollinger_upper_breakout");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_bollinger_upper_breakout",
        "bollinger_upper_breakout.csv",
        &build_bollinger_upper_breakout_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "688256.SH");

    let request = json!({
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "688256.SH"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);
    let summary = output["data"]["summary"]
        .as_str()
        .expect("summary should exist");
    let watch_points = output["data"]["watch_points"]
        .as_array()
        .expect("watch_points should exist");

    // 2026-03-29 23:10 CST: 这里新增布林带上轨突破红测，原因是布林带第一版不能只停留在快照数值层；
    // 目的：确保 CLI 结果会同时暴露 `bollinger_position_signal`、`bollinger_bandwidth_signal` 与 `indicator_snapshot.boll_width_ratio_20`，并把布林带上轨风险写进摘要与观察点。
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["bollinger_position_signal"],
        "upper_band_breakout_risk"
    );
    assert_eq!(output["data"]["bollinger_bandwidth_signal"], "expanding");
    assert!(
        output["data"]["indicator_snapshot"]["boll_width_ratio_20"]
            .as_f64()
            .expect("boll_width_ratio_20 should be a number")
            >= 0.12
    );
    assert!(summary.contains("布林带"));
    assert!(
        watch_points
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("布林带"))
    );
}

#[test]
fn technical_consultation_basic_marks_bollinger_lower_band_rebound_candidate() {
    let runtime_db_path =
        create_test_runtime_db("technical_consultation_basic_bollinger_lower_breakout");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_bollinger_lower_breakout",
        "bollinger_lower_breakout.csv",
        &build_bollinger_lower_breakout_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "002460.SZ");

    let request = json!({
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "002460.SZ"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);
    let summary = output["data"]["summary"]
        .as_str()
        .expect("summary should exist");
    let watch_points = output["data"]["watch_points"]
        .as_array()
        .expect("watch_points should exist");

    // 2026-03-29 23:10 CST: 这里新增布林带下轨触发红测，原因是布林带位置语义必须成对覆盖上轨与下轨；
    // 目的：确保 `close <= boll_lower` 时正式进入 `lower_band_rebound_candidate`，并把下轨反抽候选写入完整咨询输出。
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["bollinger_position_signal"],
        "lower_band_rebound_candidate"
    );
    assert_eq!(output["data"]["bollinger_bandwidth_signal"], "expanding");
    assert!(
        output["data"]["indicator_snapshot"]["boll_width_ratio_20"]
            .as_f64()
            .expect("boll_width_ratio_20 should be a number")
            >= 0.12
    );
    assert!(summary.contains("布林带"));
    assert!(
        watch_points
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("布林带"))
    );
}

#[test]
fn technical_consultation_basic_keeps_bollinger_neutral_in_tight_range() {
    let runtime_db_path = create_test_runtime_db("technical_consultation_basic_bollinger_neutral");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_bollinger_neutral",
        "bollinger_neutral.csv",
        &build_bollinger_tight_range_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "600928.SH");

    let request = json!({
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "600928.SH"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);
    let boll_width_ratio_20 = output["data"]["indicator_snapshot"]["boll_width_ratio_20"]
        .as_f64()
        .expect("boll_width_ratio_20 should be a number");
    let summary = output["data"]["summary"]
        .as_str()
        .expect("summary should exist");
    let watch_points = output["data"]["watch_points"]
        .as_array()
        .expect("watch_points should exist");

    // 2026-03-29 23:10 CST: 这里新增布林带窄幅中性红测，原因是只测上下轨极端会让带宽阈值漂移难以及时暴露；
    // 目的：把 `boll_width_ratio_20 <= 0.05 -> contracting` 与区间内 `neutral` 一起锁进 CLI 合同，并确认中性布林带文案会进入摘要与观察点。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["bollinger_position_signal"], "neutral");
    assert_eq!(output["data"]["bollinger_bandwidth_signal"], "contracting");
    assert!(boll_width_ratio_20 <= 0.05);
    assert!(summary.contains("布林带"));
    assert!(summary.contains("收敛"));
    assert!(
        watch_points
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("布林带") && text.contains("收敛"))
    );
}

#[test]
fn technical_consultation_basic_marks_bollinger_midline_support_bias() {
    let runtime_db_path =
        create_test_runtime_db("technical_consultation_basic_bollinger_midline_support");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_bollinger_midline_support",
        "bollinger_midline_support.csv",
        &build_bollinger_midline_support_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "300750.SZ");

    let request = json!({
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "300750.SZ"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);
    let summary = output["data"]["summary"]
        .as_str()
        .expect("summary should exist");
    let recommended_actions = output["data"]["recommended_actions"]
        .as_array()
        .expect("recommended_actions should exist");
    let watch_points = output["data"]["watch_points"]
        .as_array()
        .expect("watch_points should exist");

    // 2026-03-29 10:30 CST: 这里新增布林带中轨支撑红测，原因是布林带第一版目前只有上下轨极端语义，
    // 目的：先把“价格位于中轨上方但未触发上轨突破”正式锁进 CLI 合同，确保后续实现会同时落到 summary / actions / watch_points。
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["bollinger_midline_signal"],
        "midline_support_bias"
    );
    assert_eq!(output["data"]["bollinger_position_signal"], "neutral");
    assert!(!summary.is_empty());
    assert!(!recommended_actions.is_empty());
    assert!(!watch_points.is_empty());
}

#[test]
fn technical_consultation_basic_marks_bollinger_midline_resistance_bias() {
    let runtime_db_path =
        create_test_runtime_db("technical_consultation_basic_bollinger_midline_resistance");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_bollinger_midline_resistance",
        "bollinger_midline_resistance.csv",
        &build_bollinger_midline_resistance_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "000858.SZ");

    let request = json!({
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "000858.SZ"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);
    let summary = output["data"]["summary"]
        .as_str()
        .expect("summary should exist");
    let recommended_actions = output["data"]["recommended_actions"]
        .as_array()
        .expect("recommended_actions should exist");
    let watch_points = output["data"]["watch_points"]
        .as_array()
        .expect("watch_points should exist");

    // 2026-03-29 10:30 CST: 这里新增布林带中轨压制红测，原因是中轨下方运行与下轨极端回补不是同一层语义，
    // 目的：把“价格位于中轨下方但未触发下轨反弹候选”锁成独立合同，避免后续只能在 neutral 里混写。
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["bollinger_midline_signal"],
        "midline_resistance_bias"
    );
    assert_eq!(output["data"]["bollinger_position_signal"], "neutral");
    assert!(!summary.is_empty());
    assert!(!recommended_actions.is_empty());
    assert!(!watch_points.is_empty());
}

fn build_bullish_history_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let base = 100.0 + offset as f64 * 1.35;
        let open = base;
        let high = base + 2.4;
        let low = base - 1.8;
        let close = base + 1.6;
        let adj_close = close;
        let volume = 1_000_000 + offset as i64 * 12_000;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
    }

    rows
}

// 2026-03-29 CST: 这里补充“价格上涨但量能衰减”样本，原因是本轮要先锁住量价确认不充分的场景；
// 目的：避免后续只因为价格趋势向上，就把量能也误判成同步确认。
fn build_bullish_fading_volume_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let base = 100.0 + offset as f64 * 1.25;
        let open = base;
        let high = base + 2.2;
        let low = base - 1.6;
        let close = base + 1.4;
        let adj_close = close;
        let volume = 3_000_000 - offset as i64 * 9_000;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
    }

    rows
}

// 2026-03-29 CST: 这里补充“价格创新高但 OBV 未同步确认”的样本，原因是下一刀要先锁住背离识别的最小合同；
// 目的：让 technical_consultation_basic 能先稳定识别典型顶部背离风险，而不是一上来做太多复杂背离分类。
fn build_bearish_divergence_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close: f64 = 100.0;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let (next_close, volume): (f64, i64) = if offset < day_count - 20 {
            (close + 0.75, 1_300_000 + offset as i64 * 5_000)
        } else {
            let phase = (offset - (day_count - 20)) % 4;
            match phase {
                0 => (close + 2.6, 420_000),
                1 => (close - 2.1, 3_200_000),
                2 => (close + 2.4, 410_000),
                _ => (close + 0.35, 380_000),
            }
        };

        let open = close;
        let high = next_close.max(open) + 1.2;
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

// 2026-03-29 CST: 这里补“价格创新低但 OBV 未同步创新低”的样本，原因是方案 A 下一步要先把 bullish_divergence
// 单独锁进回归测试；目的：避免当前背离能力只覆盖顶部风险，而底部背离仍停留在理论支持、没有真实样本约束。
fn build_bullish_divergence_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close: f64 = 140.0;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let (next_close, volume): (f64, i64) = if offset < day_count - 20 {
            (close - 0.55, 1_150_000 + offset as i64 * 4_000)
        } else {
            let phase = offset - (day_count - 20);
            match phase {
                0 => (close - 2.2, 3_600_000),
                1 => (close + 1.6, 2_900_000),
                2 => (close - 0.7, 320_000),
                3 => (close + 1.3, 2_400_000),
                4 => (close - 0.5, 300_000),
                5 => (close + 1.1, 2_200_000),
                6 => (close - 0.45, 280_000),
                7 => (close + 0.95, 2_000_000),
                8 => (close - 0.35, 260_000),
                9 => (close + 0.8, 1_900_000),
                10 => (close - 0.3, 240_000),
                11 => (close + 0.65, 1_700_000),
                12 => (close - 0.25, 220_000),
                13 => (close + 0.55, 1_500_000),
                14 => (close - 0.2, 210_000),
                15 => (close + 0.45, 1_300_000),
                16 => (close - 0.18, 200_000),
                17 => (close + 0.35, 1_100_000),
                18 => (close - 0.12, 180_000),
                // 2026-03-29 19:20 CST: 这里把最后一笔低量下探压深，原因是要让最近 10 日价格明确跌破前 20 日低点；
                // 目的：在不把 OBV 拖出新低的前提下，构造出真正满足 `bullish_divergence` 的价格新低样本。
                _ => (close - 5.4, 160_000),
            }
        };

        let open = close;
        let high = next_close.max(open) + 1.15;
        let low = next_close.min(open) - 1.2;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-03-29 CST: 这里补“价格创新高且 OBV 同步创新高”的边界样本，原因是背离能力上线后必须证明正常强突破不会被误判；
// 目的：把 should-stay-none 的正向边界明确锁进回归测试，而不是只靠通用上涨样本间接覆盖。
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

// 2026-03-29 CST: 这里补“价格创新低且 OBV 同步创新低”的边界样本，原因是方案 A / 1 只剩最后一个
// should-stay-none 收口没有被显式锁进回归；目的：确认健康下破不会被误判成 bullish_divergence。
fn build_confirmed_breakdown_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close: f64 = 168.0;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let (next_close, volume): (f64, i64) = if offset < day_count - 20 {
            (close - 0.74, 950_000 + offset as i64 * 7_500)
        } else {
            let phase = offset - (day_count - 20);
            match phase % 4 {
                0 => (close - 1.55, 1_900_000 + phase as i64 * 28_000),
                1 => (close + 0.28, 430_000),
                2 => (close - 1.18, 1_700_000 + phase as i64 * 24_000),
                _ => (close - 0.52, 1_350_000),
            }
        };

        let open = close;
        let high = next_close.max(open) + 0.9;
        let low = next_close.min(open) - 1.05;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-03-31 CST: 这里补“先突破阻力、随后收回区间内”的二阶段确认样本，原因是方案 1 下一刀要补假突破回落；
// 目的：确保 breakout_signal 不会只看到历史上破就持续维持看多确认，而是能识别最新一根已经跌回阻力位下方。
fn build_failed_breakout_reversal_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close: f64 = 90.0;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let phase_start = day_count.saturating_sub(20);
        let (next_close, volume): (f64, i64) = if offset < phase_start {
            (close + 0.62, 920_000 + offset as i64 * 6_500)
        } else {
            let phase = offset - phase_start;
            match phase {
                0..=15 => (close + 0.82, 1_180_000 + phase as i64 * 18_000),
                16 => (close + 0.68, 1_260_000),
                17 => (close + 0.54, 1_220_000),
                18 => (close + 2.35, 1_980_000),
                _ => (close - 3.10, 1_080_000),
            }
        };

        let open = close;
        let high = next_close.max(open) + 1.05;
        let low = next_close.min(open) - 0.92;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-03-31 CST: 这里补“先跌破支撑、随后拉回支撑上方”的二阶段确认样本，原因是方案 1 下一刀要补假跌破拉回；
// 目的：确保 breakout_signal 能识别最新一根已经回到关键位上方，而不是继续把失效下破当成已确认弱势。
fn build_failed_breakdown_recovery_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close: f64 = 172.0;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let phase_start = day_count.saturating_sub(20);
        let (next_close, volume): (f64, i64) = if offset < phase_start {
            (close - 0.66, 940_000 + offset as i64 * 6_800)
        } else {
            let phase = offset - phase_start;
            match phase {
                0..=15 => (close - 0.84, 1_200_000 + phase as i64 * 18_000),
                16 => (close - 0.70, 1_260_000),
                17 => (close - 0.56, 1_220_000),
                18 => (close - 2.40, 2_020_000),
                _ => (close + 3.18, 1_120_000),
            }
        };

        let open = close;
        let high = next_close.max(open) + 0.96;
        let low = next_close.min(open) - 1.08;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-03-31 20:10 CST: 这里补“突破后回踩旧阻力但仍守住”的三阶段样本，原因是方案 A 下一刀要识别阻力转支撑；
// 目的：确保 breakout_signal 能把“不是继续创新高，但已经完成第一次有效回踩承接”的场景单独结构化输出。
fn build_resistance_retest_hold_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close: f64 = 91.0;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let phase_start = day_count.saturating_sub(20);
        let (next_close, volume): (f64, i64) = if offset < phase_start {
            (close + 0.64, 930_000 + offset as i64 * 6_400)
        } else {
            let phase = offset - phase_start;
            match phase {
                0..=15 => (close + 0.86, 1_220_000 + phase as i64 * 16_000),
                16 => (close + 0.74, 1_260_000),
                17 => (close + 0.58, 1_280_000),
                18 => (close + 2.28, 2_020_000),
                _ => (close - 1.22, 1_180_000),
            }
        };

        let open = close;
        let high = next_close.max(open) + 1.02;
        let low = next_close.min(open) - 0.88;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-03-31 20:10 CST: 这里补“跌破后反抽旧支撑但仍受压”的三阶段样本，原因是方案 A 下一刀要识别支撑转阻力；
// 目的：确保 breakout_signal 能把“不是继续创新低，但第一次反抽已经被旧支撑压回”的场景单独结构化输出。
fn build_support_retest_reject_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close: f64 = 171.0;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let phase_start = day_count.saturating_sub(20);
        let (next_close, volume): (f64, i64) = if offset < phase_start {
            (close - 0.67, 945_000 + offset as i64 * 6_700)
        } else {
            let phase = offset - phase_start;
            match phase {
                0..=15 => (close - 0.88, 1_240_000 + phase as i64 * 16_000),
                16 => (close - 0.76, 1_280_000),
                17 => (close - 0.60, 1_300_000),
                18 => (close - 2.34, 2_040_000),
                _ => (close + 1.26, 1_190_000),
            }
        };

        let open = close;
        let high = next_close.max(open) + 0.94;
        let low = next_close.min(open) - 1.04;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-03-31 20:35 CST: 这里补“回踩到旧阻力上方但仍贴近关键位”的观察态样本，原因是下一刀要区分回踩途中和回踩确认完成；
// 目的：确保 breakout_signal 可以表达“已经回到正确一侧，但承接安全垫还不够”的 retest_watch 语义。
fn build_resistance_retest_watch_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close: f64 = 91.0;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let phase_start = day_count.saturating_sub(20);
        let (next_close, volume): (f64, i64) = if offset < phase_start {
            (close + 0.64, 930_000 + offset as i64 * 6_400)
        } else {
            let phase = offset - phase_start;
            match phase {
                0..=15 => (close + 0.86, 1_220_000 + phase as i64 * 16_000),
                16 => (close + 0.74, 1_260_000),
                17 => (close + 0.58, 1_280_000),
                18 => (close + 2.28, 2_020_000),
                _ => (close - 2.36, 1_180_000),
            }
        };

        let open = close;
        let high = next_close.max(open) + 1.02;
        let low = next_close.min(open) - 0.88;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-03-31 20:35 CST: 这里补“反抽到旧支撑下方但仍贴近关键位”的观察态样本，原因是下一刀要区分反抽途中和反抽受压确认完成；
// 目的：确保 breakout_signal 可以表达“已经回到正确一侧，但受压安全垫还不够”的 retest_watch 语义。
fn build_support_retest_watch_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close: f64 = 171.0;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let phase_start = day_count.saturating_sub(20);
        let (next_close, volume): (f64, i64) = if offset < phase_start {
            (close - 0.67, 945_000 + offset as i64 * 6_700)
        } else {
            let phase = offset - phase_start;
            match phase {
                0..=15 => (close - 0.88, 1_240_000 + phase as i64 * 16_000),
                16 => (close - 0.76, 1_280_000),
                17 => (close - 0.60, 1_300_000),
                18 => (close - 2.34, 2_040_000),
                _ => (close + 2.42, 1_190_000),
            }
        };

        let open = close;
        let high = next_close.max(open) + 0.94;
        let low = next_close.min(open) - 1.04;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-04-01 CST: 这里新增“偏空主趋势但仍处区间等待下破”的样本，原因是方案 A 当前还没有把 bearish_range_watch 真链路锁进回归；
// 目的：确保价格尚未有效跌破关键位时，也能把“偏空但未完成下破”的语义稳定上提成组合结论，而不是退回完全中性的 range_wait。
fn build_bearish_range_watch_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close: f64 = 188.0;
    let tail_closes = [
        44.20, 44.05, 43.92, 43.80, 43.72, 43.66, 43.61, 43.58, 43.60, 43.67, 43.74, 43.70, 43.79,
        43.73, 43.82, 43.76, 43.86, 43.80, 43.89, 43.84,
    ];

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let phase_start = day_count.saturating_sub(20);
        let (next_close, volume): (f64, i64) = if offset < phase_start {
            (close - 0.72, 1_020_000 + offset as i64 * 5_200)
        } else {
            let phase = offset - phase_start;
            // 2026-04-01 CST: 这里直接用一组低位弱势箱体尾盘，原因是递减尾巴太容易在最近 4 根里继续形成新的 breakdown anchor；
            // 目的：把最近 20 根稳定固定成“长期偏空 + 近期低位箱体等待下破”的真实区间样本。
            (tail_closes[phase], 1_000_000 + phase as i64 * 8_000)
        };

        let open = close;
        let high = next_close.max(open) + 0.92;
        let low = next_close.min(open) - 1.02;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-04-01 CST: 这里新增“偏多主趋势但仍处区间等待上破”的样本，原因是方案 A 当前还没有把 bullish_range_watch 真链路锁进回归；
// 目的：确保价格尚未有效突破关键位时，也能把“偏多但未完成上破”的语义稳定上提成组合结论，而不是退回完全中性的 range_wait。
fn build_bullish_range_watch_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close: f64 = 42.0;
    let tail_closes = [
        187.62, 187.28, 187.12, 186.98, 186.90, 186.84, 186.88, 186.93, 186.87, 186.95, 186.89,
        186.97, 186.92, 186.99, 186.94, 187.01, 186.96, 187.03, 186.98, 187.00,
    ];

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let phase_start = day_count.saturating_sub(20);
        let (next_close, volume): (f64, i64) = if offset < phase_start {
            (close + 0.72, 1_000_000 + offset as i64 * 5_400)
        } else {
            let phase = offset - phase_start;
            // 2026-04-01 CST: 这里把高位箱体改成“先冲高、后横住”的尾盘，原因是缓慢抬升尾巴会在最近 4 根里反复形成 breakout anchor；
            // 目的：确保样本稳定表达“长期偏多 + 近期高位整理待上破”，而不是误落到 resistance_retest_watch。
            (tail_closes[phase], 1_020_000 + phase as i64 * 8_500)
        };

        let open = close;
        let high = next_close.max(open) + 0.98;
        let low = next_close.min(open) - 0.88;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-03-31 22:20 CST: 这里补“突破后经过多根回踩再重新站稳”的红测夹具，原因是当前实现只认前一根
// 越位样本，识别不到多根回踩后的再次承接；目的：先用真实 CLI 样本锁住“多根回踩确认”缺口，再推动主逻辑扩展。
fn build_multi_bar_resistance_retest_hold_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close: f64 = 88.0;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let phase_start = day_count.saturating_sub(22);
        let (next_close, volume): (f64, i64) = if offset < phase_start {
            (close + 0.58, 920_000 + offset as i64 * 6_200)
        } else {
            let phase = offset - phase_start;
            match phase {
                0..=15 => (close + 0.84, 1_210_000 + phase as i64 * 15_500),
                16 => (close + 0.66, 1_280_000),
                17 => (close + 2.32, 2_060_000),
                18 => (close - 2.08, 1_420_000),
                19 => (close - 0.18, 1_180_000),
                20 => (close + 0.12, 1_120_000),
                _ => (close + 1.34, 1_360_000),
            }
        };

        let open = close;
        let high = next_close.max(open) + 1.03;
        let low = next_close.min(open) - 0.92;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-03-31 22:20 CST: 这里补“跌破后经过多根反抽再重新受压”的红测夹具，原因是当前实现同样只认前一根
// 失守样本，无法覆盖多根反抽后的再次转弱；目的：让支撑转阻力的多根结构也有对称回归约束。
fn build_multi_bar_support_retest_reject_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close: f64 = 176.0;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let phase_start = day_count.saturating_sub(22);
        let (next_close, volume): (f64, i64) = if offset < phase_start {
            (close - 0.61, 935_000 + offset as i64 * 6_400)
        } else {
            let phase = offset - phase_start;
            match phase {
                0..=15 => (close - 0.86, 1_230_000 + phase as i64 * 15_800),
                16 => (close - 0.68, 1_300_000),
                17 => (close - 2.36, 2_090_000),
                18 => (close + 2.12, 1_450_000),
                19 => (close + 0.20, 1_210_000),
                20 => (close - 0.10, 1_150_000),
                _ => (close - 1.38, 1_390_000),
            }
        };

        let open = close;
        let high = next_close.max(open) + 0.96;
        let low = next_close.min(open) - 1.05;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-03-29 CST: 这里补“低位假跌破 / 低位震荡但 OBV 没有形成有效改善”的边界样本，原因是方案 A1
// 需要把“不是所有低位新低都等于 bullish_divergence”正式锁进回归；目的：防止后续继续补底背离时，
// 把仅仅处于低位拉扯、尚未形成清晰量价背离的样本误判成反转信号。
fn build_breakout_boundary_rows_from_tail(
    tail_closes: &[f64],
    intraday_padding: f64,
) -> Vec<String> {
    // 2026-04-01 CST: 这里新增关键位边界 CLI 夹具生成器，原因是本轮要把源码级 close 序列边界搬到 `CSV -> SQLite -> CLI` 真链路回归；
    // 目的：统一用“长历史底座 + 精确尾部 close 序列”的方式造样本，避免每条边界测试重复手写 220 行 CSV。
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let prefix_count = 220usize.saturating_sub(tail_closes.len());
    let mut close = 72.0;

    for offset in 0..prefix_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let step = match offset % 4 {
            0 => 0.18,
            1 => 0.14,
            2 => 0.16,
            _ => 0.12,
        };
        let next_close: f64 = close + step;
        let open = close;
        let high = next_close.max(open) + 0.08;
        let low = next_close.min(open) - 0.08;
        let adj_close = next_close;
        let volume = 880_000 + offset as i64 * 2_000;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    for (index, next_close) in tail_closes.iter().enumerate() {
        let trade_date = start_date + Duration::days((prefix_count + index) as i64);
        let open = close;
        let high = next_close.max(open) + intraday_padding;
        let low = next_close.min(open) - intraday_padding;
        let adj_close = *next_close;
        let volume = 1_120_000 + index as i64 * 12_000;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = *next_close;
    }

    rows
}

fn build_just_above_buffer_boundary_resistance_retest_hold_rows() -> Vec<String> {
    // 2026-04-01 CST: 这里新增“刚好高于 anchor + 0.15” 的 CLI 样本，原因是外层 breakout 合同里等于边界会先落到 watch；
    // 目的：把 confirmed_resistance_retest_hold 在真链路下的最小有效越界样本固定住，防止后续边界比较被悄悄改松或改严。
    build_breakout_boundary_rows_from_tail(
        &[
            99.00, 99.10, 99.20, 99.30, 99.40, 99.50, 99.60, 99.70, 99.80, 99.90, 100.00, 99.95,
            99.92, 99.90, 99.88, 99.86, 99.84, 99.82, 99.80, 99.78, 100.30, 100.16,
        ],
        0.02,
    )
}

fn build_min_buffer_floor_resistance_retest_watch_rows() -> Vec<String> {
    // 2026-04-01 CST: 这里新增“ATR 很小、最小缓冲 0.15 接管”的 CLI 样本，原因是上一轮真实 bug 就发生在这个兜底边界；
    // 目的：确保最新一根恰好落在旧阻力 + 0.15 时，对外仍稳定输出 resistance_retest_watch。
    build_breakout_boundary_rows_from_tail(
        &[
            99.00, 99.10, 99.20, 99.30, 99.40, 99.50, 99.60, 99.70, 99.80, 99.90, 100.00, 99.95,
            99.92, 99.90, 99.88, 99.86, 99.84, 99.82, 99.80, 99.78, 100.30, 100.15,
        ],
        0.02,
    )
}

fn build_stale_multi_bar_resistance_anchor_rows() -> Vec<String> {
    // 2026-04-01 CST: 这里新增“旧突破锚点已经超过多根 lookback”的 CLI 样本，原因是多根回踩逻辑最容易只在内部函数层被锁住；
    // 目的：确保真链路读取历史后，不会把已经过期的旧突破误判成仍可确认的多根回踩结构。
    build_breakout_boundary_rows_from_tail(
        &[
            99.00, 99.10, 99.20, 99.30, 99.40, 99.50, 99.60, 99.70, 99.80, 99.90, 100.00, 99.95,
            99.92, 99.90, 99.88, 99.86, 99.84, 99.82, 99.80, 99.78, 100.60, 99.95, 99.94, 99.93,
            99.92, 99.91, 100.04,
        ],
        0.02,
    )
}

fn build_failed_resistance_breakout_just_below_boundary_rows() -> Vec<String> {
    // 2026-04-01 CST: 这里新增“刚好跌破旧阻力-buffer 一点点”的 CLI 样本，原因是 failed 与 watch 之间只差极窄边界；
    // 目的：确保假突破回落在真链路里会稳定进入 failed_resistance_breakout，而不是被边界比较吞回观察态。
    build_breakout_boundary_rows_from_tail(
        &[
            99.00, 99.10, 99.20, 99.30, 99.40, 99.50, 99.60, 99.70, 99.80, 99.90, 100.00, 99.95,
            99.92, 99.90, 99.88, 99.86, 99.84, 99.82, 99.80, 99.78, 100.30, 99.84,
        ],
        0.02,
    )
}

fn build_failed_support_breakdown_just_above_boundary_rows() -> Vec<String> {
    // 2026-04-01 CST: 这里新增“刚好拉回旧支撑+buffer 一点点”的 CLI 样本，原因是空头侧失效边界也要被明确锁住；
    // 目的：确保假跌破拉回在真链路里会稳定进入 failed_support_breakdown，而不是被误留在观察态。
    build_breakout_boundary_rows_from_tail(
        &[
            100.90, 100.80, 100.70, 100.60, 100.50, 100.40, 100.30, 100.20, 100.10, 100.05, 100.00,
            100.04, 100.06, 100.08, 100.10, 100.12, 100.14, 100.16, 100.18, 100.20, 99.70, 100.16,
        ],
        0.02,
    )
}

fn build_multi_bar_resistance_retest_watch_rows() -> Vec<String> {
    // 2026-04-01 CST: 这里新增“多根回踩后仍贴着旧阻力附近磨位”的 CLI 样本，原因是多根 confirmed 已覆盖，但多根 watch 还没沉到底层；
    // 目的：确保突破后 2~4 根磨位期间，真链路能稳定保留 resistance_retest_watch 中间态。
    build_breakout_boundary_rows_from_tail(
        &[
            99.00, 99.10, 99.20, 99.30, 99.40, 99.50, 99.60, 99.70, 99.80, 99.90, 100.00, 99.95,
            99.92, 99.90, 99.88, 99.86, 99.84, 99.82, 99.80, 99.79, 99.78, 100.60, 99.95, 99.96,
            100.02,
        ],
        0.02,
    )
}

fn build_multi_bar_support_retest_watch_rows() -> Vec<String> {
    // 2026-04-01 CST: 这里新增“多根反抽后仍贴着旧支撑附近磨位”的 CLI 样本，原因是空头侧多根观察态还没被真链路回归覆盖；
    // 目的：确保跌破后 2~4 根反抽磨位期间，真链路能稳定保留 support_retest_watch 中间态。
    build_breakout_boundary_rows_from_tail(
        &[
            100.90, 100.80, 100.70, 100.60, 100.50, 100.40, 100.30, 100.20, 100.10, 100.05, 100.00,
            100.04, 100.06, 100.08, 100.10, 100.12, 100.14, 100.16, 100.18, 100.20, 99.40, 100.05,
            100.06, 100.07, 99.98,
        ],
        0.02,
    )
}

fn build_false_breakdown_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close: f64 = 126.0;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let (next_close, volume): (f64, i64) = if offset < day_count - 20 {
            (close - 0.46, 1_050_000 + offset as i64 * 2_500)
        } else {
            let phase = offset - (day_count - 20);
            match phase {
                0 => (close - 1.80, 1_650_000),
                1 => (close + 0.95, 1_100_000),
                2 => (close - 1.35, 1_700_000),
                3 => (close + 0.88, 1_060_000),
                4 => (close - 0.92, 1_420_000),
                5 => (close + 0.72, 980_000),
                6 => (close - 0.76, 1_360_000),
                7 => (close + 0.69, 940_000),
                8 => (close - 0.58, 1_320_000),
                9 => (close + 0.54, 920_000),
                10 => (close - 0.42, 1_280_000),
                11 => (close + 0.36, 900_000),
                12 => (close - 0.31, 1_240_000),
                13 => (close + 0.28, 880_000),
                14 => (close - 0.26, 1_220_000),
                15 => (close + 0.22, 860_000),
                16 => (close - 0.18, 1_180_000),
                17 => (close + 0.16, 840_000),
                18 => (close - 0.12, 1_140_000),
                _ => (close - 0.08, 1_100_000),
            }
        };

        let open = close;
        let high = next_close.max(open) + 0.95;
        let low = next_close.min(open) - 1.00;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-03-29 CST: 这里补“价格没有创新高但 OBV 明显回落”的边界样本，原因是方案 A 还要求锁住最常见的误判路径；
// 目的：确保仅仅出现量能回落时，不会把所有高位回落样本都误判成 bearish_divergence。
// 2026-03-29 CST: 这里先补 KDJ “超卖后回抽”的样本，原因是方案 A 下一步要把第一版择时信号接进
// technical_consultation_basic；目的：先用真实价格路径锁住 `timing_signal = oversold_rebound` 的最小合同。
fn build_kdj_oversold_rebound_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close: f64 = 162.0;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let (next_close, volume): (f64, i64) = if offset < day_count - 20 {
            (close - 0.58, 1_180_000 + offset as i64 * 2_000)
        } else {
            let phase = offset - (day_count - 20);
            match phase {
                0 => (close - 2.8, 2_000_000),
                1 => (close - 2.4, 2_100_000),
                2 => (close - 2.0, 2_050_000),
                3 => (close - 1.7, 1_950_000),
                4 => (close - 1.3, 1_850_000),
                5 => (close - 0.8, 1_700_000),
                // 2026-03-29 21:50 CST: 这里把尾部修成“低位止跌后的小幅修复”，原因是原夹具反而把最后 9 天推到了区间高位；
                // 目的：让 KDJ 当前值保持在低位区，同时出现 K 上穿 D 的第一版超卖修复形态。
                6 => (close - 0.55, 1_380_000),
                7 => (close - 0.35, 1_260_000),
                8 => (close - 0.18, 1_180_000),
                9 => (close + 0.10, 1_050_000),
                10 => (close + 0.18, 980_000),
                11 => (close + 0.24, 950_000),
                12 => (close + 0.22, 930_000),
                13 => (close + 0.18, 915_000),
                14 => (close + 0.15, 900_000),
                15 => (close + 0.12, 890_000),
                16 => (close + 0.10, 880_000),
                17 => (close + 0.08, 870_000),
                18 => (close + 0.07, 860_000),
                _ => (close + 0.06, 850_000),
            }
        };

        let open = close;
        let high = next_close.max(open) + 0.9;
        let low = next_close.min(open) - 1.3;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-03-29 CST: 这里再补 KDJ “高位钝化后回落”的样本，原因是第一版择时信号除了低位回抽，
// 还要覆盖高位回撤；目的：先把 `timing_signal = overbought_pullback` 正式钉进回归测试。
fn build_kdj_overbought_pullback_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close: f64 = 84.0;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let (next_close, volume): (f64, i64) = if offset < day_count - 20 {
            (close + 0.66, 1_020_000 + offset as i64 * 3_000)
        } else {
            let phase = offset - (day_count - 20);
            match phase {
                0 => (close + 2.8, 1_950_000),
                1 => (close + 2.4, 2_020_000),
                2 => (close + 2.0, 1_980_000),
                3 => (close + 1.7, 1_900_000),
                4 => (close + 1.4, 1_850_000),
                5 => (close + 1.1, 1_760_000),
                6 => (close + 0.85, 1_620_000),
                7 => (close + 0.62, 1_520_000),
                8 => (close + 0.4, 1_420_000),
                9 => (close + 0.18, 1_320_000),
                // 2026-03-29 21:50 CST: 这里把尾部回落改成“高位轻回撤”，原因是原夹具把最后 9 天直接拉回到了低位区；
                // 目的：让 KDJ 保持在高位区，同时形成 K 低于 D 的第一版高位钝化回落形态。
                10 => (close + 0.12, 1_180_000),
                11 => (close + 0.08, 1_120_000),
                12 => (close + 0.05, 1_080_000),
                13 => (close - 0.05, 1_000_000),
                14 => (close - 0.08, 980_000),
                15 => (close - 0.10, 960_000),
                16 => (close - 0.12, 940_000),
                17 => (close - 0.10, 920_000),
                18 => (close - 0.08, 900_000),
                _ => (close - 0.06, 890_000),
            }
        };

        let open = close;
        let high = next_close.max(open) + 1.2;
        let low = next_close.min(open) - 0.85;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-03-29 CST: 这里补 RSRS “斜率强化”的专用样本，原因是方案 A 下一刀要把 RSRS 第一版正式接进技术咨询主线；
// 目的：构造“最近窗口高低点回归斜率显著高于历史均值”的场景，先把 `rsrs_signal = bullish_breakout` 锁进回归测试。
fn build_rsrs_bullish_breakout_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let low = 90.0 + offset as f64 * 0.58;
        let (beta, intercept, volume) = if offset < day_count - 60 {
            (1.02, 4.0, 1_020_000 + offset as i64 * 4_200)
        } else {
            let phase = offset - (day_count - 60);
            // 2026-03-29 22:50 CST: 这里把尾部改成“持续斜率强化”而不是一次性跳台，原因是原样本会把最近窗口均值整体抬上去，
            // 目的：让最新 RSRS beta 真正位于最近历史分布右侧，从而形成稳定正 zscore。
            (
                1.02 + phase as f64 * 0.0025,
                4.0,
                1_380_000 + offset as i64 * 5_500,
            )
        };
        let high = low * beta + intercept;
        let range = high - low;
        let open = low + range * 0.24;
        let close = high - range * 0.18;
        let adj_close = close;

        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
    }

    rows
}

// 2026-03-29 CST: 这里补 RSRS “压力转强”的专用样本，原因是 RSRS 第一版不能只覆盖正向强化，还要覆盖斜率走弱场景；
// 目的：构造“最近窗口高低点回归斜率显著低于历史均值”的样本，先把 `rsrs_signal = bearish_pressure` 锁进回归测试。
fn build_rsrs_bearish_pressure_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let low = 88.0 + offset as f64 * 0.54;
        let (beta, intercept, volume) = if offset < day_count - 60 {
            (1.11, 8.0, 1_080_000 + offset as i64 * 3_800)
        } else {
            let phase = offset - (day_count - 60);
            // 2026-03-29 22:50 CST: 这里把尾部改成“持续斜率走弱”而不是一步跳空，原因是原样本会把最近窗口均值一起压低后抵消信号，
            // 目的：让最新 RSRS beta 真正落在最近历史分布左侧，从而形成稳定负 zscore。
            (
                1.11 - phase as f64 * 0.0025,
                8.0,
                920_000 - (offset as i64 - (day_count as i64 - 60)) * 2_100,
            )
        };
        let high = low * beta + intercept;
        let range = high - low;
        let open = low + range * 0.28;
        let close = low + range * 0.46;
        let adj_close = close;

        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
    }

    rows
}

fn build_obv_pullback_without_breakout_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close: f64 = 96.0;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let (next_close, volume): (f64, i64) = if offset < day_count - 20 {
            (close + 0.6, 1_050_000 + offset as i64 * 6_000)
        } else {
            let phase = offset - (day_count - 20);
            match phase {
                0 => (close + 1.1, 1_700_000),
                1 => (close + 0.8, 1_500_000),
                2 => (close + 0.5, 1_300_000),
                3 => (close - 0.35, 380_000),
                4 => (close + 0.25, 460_000),
                5 => (close - 0.3, 420_000),
                6 => (close + 0.18, 430_000),
                7 => (close - 0.22, 440_000),
                8 => (close + 0.12, 450_000),
                9 => (close - 0.25, 500_000),
                10 => (close + 0.08, 520_000),
                11 => (close - 0.28, 560_000),
                12 => (close + 0.05, 580_000),
                13 => (close - 0.3, 620_000),
                14 => (close + 0.02, 700_000),
                15 => (close - 0.32, 1_300_000),
                16 => (close + 0.06, 720_000),
                17 => (close - 0.24, 1_450_000),
                18 => (close + 0.04, 760_000),
                // 2026-03-29 CST: 这里把最后一天改成“小幅回落但不创新低”，原因是上一版夹具把最后一根跌成了新低，
                // 反而真的满足了 bullish_divergence 条件；目的：把边界样本修正为“OBV 走弱，但价格并未形成新低”的真实 none 场景。
                _ => (close - 0.02, 1_650_000),
            }
        };

        let open = close;
        let high = next_close.max(open) + 0.95;
        let low = next_close.min(open) - 0.95;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-03-29 CST: 这里补充横盘震荡样本生成器，原因是本轮要先用红测锁住 ADX 弱趋势判定；
// 目的：让 technical_consultation_basic 不只验证上涨强趋势，也能验证“方向不清 + 强度偏弱”的 Rust 合同输出。
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
        let volume = 900_000 + (offset % 5) as i64 * 8_000;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
    }

    rows
}

// 2026-03-28 CST: 这里集中通过现有导入 Tool 预热 SQLite，原因是用户要求股票能力统一走 Rust / exe 主链，
// 目的：让技术面红测依赖的历史数据输入方式和真实生产入口保持一致，而不是在测试里绕开导入层直接写库。
// 2026-03-29 09:35 CST: 这里新增 MFI“高位资金持续流入”夹具，原因是技术面第一版资金流信号需要先锁住超买分支；
// 目的：确保后续实现能用真实 OHLCV 历史把 `money_flow_signal = overbought_distribution` 稳定钉进 CLI 回归合同。
fn build_mfi_overbought_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close = 72.0;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let (next_close, volume): (f64, i64) = if offset < day_count - 20 {
            (close + 0.42, 920_000 + offset as i64 * 2_800)
        } else {
            let phase = offset - (day_count - 20);
            (
                close + 1.10 + phase as f64 * 0.06,
                2_600_000 + phase as i64 * 110_000,
            )
        };

        let open = close;
        let high = next_close.max(open) + 1.20;
        let low = next_close.min(open) - 0.35;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-03-29 09:35 CST: 这里新增 MFI“低位资金持续流出”夹具，原因是资金流信号必须成对覆盖超买与超卖两端；
// 目的：确保 `money_flow_signal = oversold_accumulation` 不只是理论规则，而是能被 Rust 主链真实历史样本稳定触发。
fn build_mfi_oversold_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close = 148.0;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let (next_close, volume): (f64, i64) = if offset < day_count - 20 {
            (close - 0.38, 900_000 + offset as i64 * 2_300)
        } else {
            let phase = offset - (day_count - 20);
            (
                close - 1.15 - phase as f64 * 0.05,
                2_550_000 + phase as i64 * 105_000,
            )
        };

        let open = close;
        let high = next_close.max(open) + 0.40;
        let low = next_close.min(open) - 1.15;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-03-29 09:35 CST: 这里新增 MFI“震荡均衡”夹具，原因是第一版资金流信号不能只有两端极值，没有中性边界；
// 目的：把 `20 < MFI < 80 -> neutral` 的主路径正式锁进 CLI 回归，避免后续阈值或公式调整时全部样本都被推向极端。
fn build_mfi_neutral_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close = 96.0;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let phase = offset.saturating_sub(day_count - 20);
        let (next_close, volume): (f64, i64) = if offset < day_count - 20 {
            (close + 0.18, 880_000 + offset as i64 * 1_600)
        } else {
            let swing = match phase % 4 {
                0 => 1.10,
                1 => -1.10,
                2 => 0.88,
                _ => -0.88,
            };
            (close + swing, 1_650_000)
        };

        let open = close;
        let high_padding = if offset < day_count - 20 {
            0.85
        } else if phase % 2 == 0 {
            0.82 + phase as f64 * 0.01
        } else {
            0.74 + phase as f64 * 0.01
        };
        let low_padding = if offset < day_count - 20 {
            0.85
        } else if phase % 2 == 0 {
            0.71 + phase as f64 * 0.01
        } else {
            0.79 + phase as f64 * 0.01
        };
        let high = next_close.max(open) + high_padding;
        let low = next_close.min(open) - low_padding;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-03-29 09:35 CST: 这里新增 MFI“零量平稳”夹具，原因是用户明确希望非 IT 环境下的 EXE 能稳定输出而不是跑出 NaN；
// 目的：锁住 `volume = 0` 与平盘组合场景，确保资金流计算在边界输入下仍返回可消费的中性信号。
fn build_mfi_zero_volume_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close = 84.0;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let phase = offset.saturating_sub(day_count - 20);
        let (next_close, volume): (f64, i64) = if offset < day_count - 20 {
            (close + 0.22, 760_000 + offset as i64 * 1_800)
        } else {
            let swing = match phase % 4 {
                0 => 0.18,
                1 => -0.12,
                2 => 0.09,
                _ => -0.15,
            };
            (close + swing, 0)
        };

        let open = close;
        let high_padding = if offset < day_count - 20 {
            0.45
        } else {
            0.38 + phase as f64 * 0.005
        };
        let low_padding = if offset < day_count - 20 {
            0.45
        } else {
            0.34 + phase as f64 * 0.005
        };
        let high = next_close.max(open) + high_padding;
        let low = next_close.min(open) - low_padding;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-03-29 10:20 CST: 这里新增 MFI“混合量能中性”夹具，原因是 A1 这轮要把真实成交量起伏下的 neutral 合同也锁进 CLI；
// 目的：构造出涨跌与量能都明显交替、但正负资金流总体保持平衡的窗口，防止后续实现把 mixed-volume 样本误压成 80/20 两端。
fn build_mfi_mixed_volume_neutral_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close = 108.0;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let phase = offset.saturating_sub(day_count - 20);
        let (next_close, volume): (f64, i64) = if offset < day_count - 20 {
            (close + 0.20, 900_000 + offset as i64 * 1_900)
        } else {
            match phase % 6 {
                0 => (close + 1.42, 2_450_000),
                1 => (close - 1.28, 2_360_000),
                2 => (close + 0.94, 980_000),
                3 => (close - 0.88, 930_000),
                4 => (close + 0.76, 1_920_000),
                _ => (close - 0.74, 1_860_000),
            }
        };

        let open = close;
        let high_padding = if offset < day_count - 20 {
            0.82
        } else if phase % 2 == 0 {
            0.96 + phase as f64 * 0.008
        } else {
            0.87 + phase as f64 * 0.008
        };
        let low_padding = if offset < day_count - 20 {
            0.80
        } else if phase % 2 == 0 {
            0.83 + phase as f64 * 0.008
        } else {
            0.94 + phase as f64 * 0.008
        };
        let high = next_close.max(open) + high_padding;
        let low = next_close.min(open) - low_padding;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-03-30 09:20 CST: 这里新增 CCI“高位偏离过大”夹具，原因是 CCI 第一版要先锁住 `cci_20 >= 100` 的均值回归风险场景；
// 目的：让后续实现必须能在真实 OHLC 历史窗口上稳定产出 `overbought_reversal_risk`，而不是只在源码级硬编码阈值。
fn build_cci_overbought_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close = 102.0;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let phase = offset.saturating_sub(day_count - 20);
        let (next_close, high_padding, low_padding, volume): (f64, f64, f64, i64) =
            if offset < day_count - 20 {
                (close + 0.16, 0.55, 0.52, 920_000 + offset as i64 * 1_800)
            } else {
                (
                    close + 1.85 + phase as f64 * 0.08,
                    0.42 + phase as f64 * 0.01,
                    0.18 + phase as f64 * 0.004,
                    2_080_000 + phase as i64 * 92_000,
                )
            };

        let open = close;
        let high = next_close.max(open) + high_padding;
        let low = next_close.min(open) - low_padding;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-03-30 09:20 CST: 这里新增 CCI“低位偏离过大”夹具，原因是均值回归信号必须成对覆盖上沿与下沿，不能只做高位回落风险；
// 目的：构造 `cci_20 <= -100` 的真实历史样本，确保第一版咨询输出能稳定暴露 `oversold_rebound_candidate`。
fn build_cci_oversold_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close = 148.0;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let phase = offset.saturating_sub(day_count - 20);
        let (next_close, high_padding, low_padding, volume): (f64, f64, f64, i64) =
            if offset < day_count - 20 {
                (close - 0.15, 0.52, 0.55, 910_000 + offset as i64 * 1_700)
            } else {
                (
                    close - 1.90 - phase as f64 * 0.08,
                    0.20 + phase as f64 * 0.004,
                    0.44 + phase as f64 * 0.01,
                    2_120_000 + phase as i64 * 96_000,
                )
            };

        let open = close;
        let high = next_close.max(open) + high_padding;
        let low = next_close.min(open) - low_padding;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-03-30 09:20 CST: 这里新增 CCI“震荡中性”夹具，原因是 CCI 类指标如果缺少 `-100 ~ 100` 的稳定样本，后续阈值漂移很难第一时间暴露；
// 目的：让 balanced-range 场景在 CLI 回归里固定落到 `neutral`，并为摘要/观察点保留中性语义覆盖。
fn build_cci_neutral_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close = 118.0;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let phase = offset.saturating_sub(day_count - 20);
        let (next_close, high_padding, low_padding, volume): (f64, f64, f64, i64) =
            if offset < day_count - 20 {
                (close + 0.10, 0.72, 0.70, 930_000 + offset as i64 * 1_300)
            } else {
                match phase % 6 {
                    0 => (close + 0.82, 0.76, 0.72, 1_520_000),
                    1 => (close - 0.78, 0.73, 0.77, 1_480_000),
                    2 => (close + 0.64, 0.74, 0.71, 1_410_000),
                    3 => (close - 0.62, 0.72, 0.75, 1_450_000),
                    4 => (close + 0.56, 0.70, 0.68, 1_390_000),
                    _ => (close - 0.54, 0.69, 0.73, 1_430_000),
                }
            };

        let open = close;
        let high = next_close.max(open) + high_padding;
        let low = next_close.min(open) - low_padding;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-03-30 10:20 CST: 这里新增 Williams %R“高位区间超买”夹具，原因是 Williams %R 第一版要先锁住 `williams_r_14 >= -20` 的高位风险场景；
// 目的：让后续实现必须能在真实 OHLC 历史窗口上稳定产出 `overbought_pullback_risk`，而不是只在源码级硬编码阈值。
fn build_williams_r_overbought_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close = 86.0;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let phase = offset.saturating_sub(day_count - 14);
        let (next_close, high_padding, low_padding, volume): (f64, f64, f64, i64) =
            if offset < day_count - 14 {
                (close + 0.10, 0.85, 0.82, 910_000 + offset as i64 * 1_500)
            } else {
                (
                    close + 1.55 + phase as f64 * 0.04,
                    0.24 + phase as f64 * 0.006,
                    0.05,
                    1_980_000 + phase as i64 * 75_000,
                )
            };

        let open = close;
        let high = next_close.max(open) + high_padding;
        let low = next_close.min(open) - low_padding;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-03-30 10:20 CST: 这里新增 Williams %R“低位区间超卖”夹具，原因是区间位置类信号必须成对覆盖上沿与下沿；
// 目的：构造 `williams_r_14 <= -80` 的真实历史样本，确保第一版咨询输出能稳定暴露 `oversold_rebound_candidate`。
fn build_williams_r_oversold_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close = 154.0;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let phase = offset.saturating_sub(day_count - 14);
        let (next_close, high_padding, low_padding, volume): (f64, f64, f64, i64) =
            if offset < day_count - 14 {
                (close - 0.11, 0.82, 0.86, 900_000 + offset as i64 * 1_400)
            } else {
                (
                    close - 1.58 - phase as f64 * 0.04,
                    0.05,
                    0.25 + phase as f64 * 0.006,
                    2_020_000 + phase as i64 * 78_000,
                )
            };

        let open = close;
        let high = next_close.max(open) + high_padding;
        let low = next_close.min(open) - low_padding;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-03-30 10:20 CST: 这里新增 Williams %R“震荡中性”夹具，原因是缺少 `-80 ~ -20` 稳定样本时，后续阈值漂移很难第一时间暴露；
// 目的：让 balanced-range 场景在 CLI 回归里固定落到 `neutral`，并为摘要/观察点保留中性语义覆盖。
fn build_williams_r_neutral_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close = 118.0;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let phase = offset.saturating_sub(day_count - 14);
        let (next_close, high_padding, low_padding, volume): (f64, f64, f64, i64) =
            if offset < day_count - 14 {
                (close + 0.08, 0.88, 0.84, 920_000 + offset as i64 * 1_200)
            } else {
                match phase % 4 {
                    0 => (close + 0.46, 0.74, 0.70, 1_420_000),
                    1 => (close - 0.42, 0.71, 0.75, 1_450_000),
                    2 => (close + 0.36, 0.69, 0.68, 1_390_000),
                    _ => (close - 0.34, 0.68, 0.72, 1_410_000),
                }
            };

        let open = close;
        let high = next_close.max(open) + high_padding;
        let low = next_close.min(open) - low_padding;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-03-29 23:10 CST: 这里新增布林带上轨突破夹具，原因是布林带第一版要先锁住 `close >= boll_upper` 且带宽扩张的高波动场景；
// 目的：让后续实现必须能在真实 OHLC 历史窗口上稳定产出 `upper_band_breakout_risk + expanding`，而不是只在源码级硬编码阈值。
fn build_bollinger_upper_breakout_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close = 92.0;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let phase = offset.saturating_sub(day_count - 20);
        let (next_close, high_padding, low_padding, volume): (f64, f64, f64, i64) =
            if offset < day_count - 20 {
                (close + 0.18, 0.95, 0.88, 980_000 + offset as i64 * 1_800)
            } else {
                match phase {
                    19 => (close + 9.80, 2.80, 0.06, 4_800_000),
                    18 => (close + 4.10, 1.90, 0.08, 3_600_000),
                    _ => (
                        close + 1.75 + phase as f64 * 0.05,
                        1.10 + phase as f64 * 0.03,
                        0.10,
                        2_050_000 + phase as i64 * 72_000,
                    ),
                }
            };

        let open = close;
        let high = next_close.max(open) + high_padding;
        let low = next_close.min(open) - low_padding;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-03-29 23:10 CST: 这里新增布林带下轨击穿夹具，原因是布林带位置能力不能只覆盖上轨一端；
// 目的：构造 `close <= boll_lower` 且带宽扩张的真实历史样本，确保第一版咨询输出能稳定暴露 `lower_band_rebound_candidate`。
fn build_bollinger_lower_breakout_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close = 168.0;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let phase = offset.saturating_sub(day_count - 20);
        let (next_close, high_padding, low_padding, volume): (f64, f64, f64, i64) =
            if offset < day_count - 20 {
                (close - 0.20, 0.90, 0.96, 1_020_000 + offset as i64 * 1_700)
            } else {
                match phase {
                    19 => (close - 10.20, 0.08, 2.95, 4_900_000),
                    18 => (close - 4.25, 0.10, 2.00, 3_700_000),
                    _ => (
                        close - 1.82 - phase as f64 * 0.05,
                        0.12,
                        1.12 + phase as f64 * 0.03,
                        2_080_000 + phase as i64 * 75_000,
                    ),
                }
            };

        let open = close;
        let high = next_close.max(open) + high_padding;
        let low = next_close.min(open) - low_padding;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-03-29 23:10 CST: 这里新增布林带窄幅震荡夹具，原因是布林带第一版还需要一个 `neutral + contracting` 的稳定中性样本；
// 目的：把 tight-range 场景固定在 CLI 回归里，避免后续带宽阈值或中轨口径漂移后只剩极端样本还能通过。
fn build_bollinger_tight_range_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close = 106.0;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let phase = offset.saturating_sub(day_count - 20);
        let (next_close, high_padding, low_padding, volume): (f64, f64, f64, i64) =
            if offset < day_count - 20 {
                match offset % 4 {
                    0 => (close + 0.05, 0.42, 0.40, 1_060_000),
                    1 => (close - 0.04, 0.41, 0.42, 1_040_000),
                    2 => (close + 0.03, 0.40, 0.41, 1_050_000),
                    _ => (close - 0.02, 0.41, 0.40, 1_055_000),
                }
            } else {
                match phase % 4 {
                    0 => (close + 0.12, 0.35, 0.33, 1_080_000),
                    1 => (close - 0.10, 0.34, 0.35, 1_070_000),
                    2 => (close + 0.08, 0.33, 0.34, 1_075_000),
                    _ => (close - 0.07, 0.34, 0.33, 1_065_000),
                }
            };

        let open = close;
        let high = next_close.max(open) + high_padding;
        let low = next_close.min(open) - low_padding;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-03-29 10:30 CST: 这里新增布林带中轨支撑夹具，原因是方案 A 要先补“中轨上方运行”的独立语义，
// 目的：构造“价格高于中轨、低于上轨、带宽不极端”的稳定样本，避免测试误落到上下轨极端分类。
fn build_bollinger_midline_support_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close = 118.0;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let phase = offset.saturating_sub(day_count - 20);
        let (next_close, high_padding, low_padding, volume): (f64, f64, f64, i64) =
            if offset < day_count - 20 {
                match offset % 5 {
                    0 => (close + 0.22, 0.95, 0.88, 1_180_000),
                    1 => (close - 0.06, 0.82, 0.76, 1_145_000),
                    2 => (close + 0.18, 0.90, 0.80, 1_165_000),
                    3 => (close - 0.04, 0.78, 0.74, 1_155_000),
                    _ => (close + 0.15, 0.86, 0.79, 1_170_000),
                }
            } else {
                match phase % 5 {
                    0 => (close + 0.28, 0.84, 0.68, 1_240_000),
                    1 => (close + 0.12, 0.74, 0.62, 1_210_000),
                    2 => (close - 0.05, 0.70, 0.60, 1_195_000),
                    3 => (close + 0.18, 0.76, 0.61, 1_225_000),
                    _ => (close + 0.09, 0.72, 0.59, 1_205_000),
                }
            };

        let open = close;
        let high = next_close.max(open) + high_padding;
        let low = next_close.min(open) - low_padding;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

// 2026-03-29 10:30 CST: 这里新增布林带中轨压制夹具，原因是方案 A 还需要补“中轨下方运行”的对称语义，
// 目的：构造“价格低于中轨、高于下轨、带宽不极端”的稳定样本，确保测试不会误触发下轨反弹或收缩极值。
fn build_bollinger_midline_resistance_rows(day_count: usize) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close = 132.0;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let phase = offset.saturating_sub(day_count - 20);
        let (next_close, high_padding, low_padding, volume): (f64, f64, f64, i64) =
            if offset < day_count - 20 {
                match offset % 5 {
                    0 => (close - 0.20, 0.88, 0.92, 1_160_000),
                    1 => (close + 0.05, 0.76, 0.81, 1_135_000),
                    2 => (close - 0.16, 0.83, 0.90, 1_150_000),
                    3 => (close + 0.03, 0.74, 0.78, 1_140_000),
                    _ => (close - 0.14, 0.81, 0.88, 1_155_000),
                }
            } else {
                match phase % 5 {
                    0 => (close - 0.26, 0.70, 0.84, 1_205_000),
                    1 => (close - 0.10, 0.66, 0.78, 1_190_000),
                    2 => (close + 0.04, 0.63, 0.73, 1_175_000),
                    3 => (close - 0.17, 0.68, 0.80, 1_200_000),
                    _ => (close - 0.08, 0.65, 0.76, 1_185_000),
                }
            };

        let open = close;
        let high = next_close.max(open) + high_padding;
        let low = next_close.min(open) - low_padding;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}

fn import_history_csv(runtime_db_path: &Path, csv_path: &Path, symbol: &str) {
    let request = json!({
        "tool": "import_stock_price_history",
        "args": {
            "csv_path": csv_path.to_string_lossy(),
            "symbol": symbol,
            "source": "technical_consultation_fixture"
        }
    });

    let output =
        run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path.to_path_buf());
    assert_eq!(output["status"], "ok");
}

#[test]
fn tool_catalog_includes_technical_consultation_basic() {
    let output = run_cli_with_json("");

    // 2026-03-28 CST: 这里先锁目录可发现性，原因是新 Tool 如果不进 catalog，后续 CLI 和 AI 就算底层实现好了也调不起来；
    // 目的：防止只写业务代码却漏接 catalog / dispatcher，导致能力实际上不可用。
    assert!(
        output["data"]["tool_catalog"]
            .as_array()
            .expect("tool catalog should be an array")
            .iter()
            .any(|tool| tool == "technical_consultation_basic")
    );
}

#[test]
fn technical_consultation_basic_returns_snapshot_and_guidance_from_sqlite_history() {
    let runtime_db_path = create_test_runtime_db("technical_consultation_basic_ok");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_ok",
        "bullish_prices.csv",
        &build_bullish_history_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "600519.SH");

    let request = json!({
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "600519.SH"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);

    // 2026-03-28 CST: 这里先锁第一版成功合同，原因是这轮不是只补一个指标函数，而是要形成后续 Skill / AI 可复用的稳定输出；
    // 目的：把 `symbol / trend_bias / momentum_signal / volatility_state / indicator_snapshot / recommended_actions / watch_points`
    // 这几个外部字段先钉进回归测试，避免后续实现阶段边写边漂。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["symbol"], "600519.SH");
    assert_eq!(output["data"]["history_row_count"], 220);
    assert_eq!(output["data"]["trend_bias"], "bullish");
    // 2026-03-29 CST: 这里先锁住 ADX 方案 A 的新增合同，原因是后续实现要把“方向 + 强度”一起稳定暴露给 AI；
    // 目的：确保上涨样本不仅能给出 bullish，还能给出 strong，并把 ADX/+DI/-DI 快照正式钉进回归测试。
    assert_eq!(output["data"]["trend_strength"], "strong");
    // 2026-03-29 CST: 这里补充量价确认红测，原因是下一刀要把技术面从“方向 + 强度”推进到“方向 + 强度 + 量能确认”；
    // 目的：确保上涨放量样本会稳定暴露 `volume_confirmation` 与对应量能快照，不让后续实现只停留在文案层。
    assert_eq!(output["data"]["volume_confirmation"], "confirmed");
    // 2026-03-29 CST: 这里先锁默认无背离场景，原因是背离能力上线后不能把普通强趋势样本误判成背离；
    // 目的：确保新增背离字段不会污染现有趋势向上样本的主判断。
    assert_eq!(output["data"]["divergence_signal"], "none");
    assert_eq!(output["data"]["timing_signal"], "neutral");
    // 2026-03-29 09:35 CST: 这里补充 MFI 合同可见性断言，原因是资金流信号这轮要正式进外部结果而不是停留在内部快照；
    // 目的：确保成功主样本至少能暴露 `money_flow_signal` 与 `mfi_14`，方便后续 Skill / AI 直接消费。
    assert!(output["data"]["money_flow_signal"].is_string());
    // 2026-03-30 09:20 CST: 这里补充 CCI 合同可见性断言，原因是均值回归能力这轮要正式进入对外结果而不是停留在内部计算；
    // 目的：确保成功主样本至少能暴露 `mean_reversion_signal` 与 `cci_20`，方便后续 Skill / AI 直接消费。
    assert!(output["data"]["mean_reversion_signal"].is_string());
    // 2026-03-30 10:20 CST: 这里补充 Williams %R 合同可见性断言，原因是区间位置能力这轮也要正式进入对外结果；
    // 目的：确保成功主样本至少能暴露 `range_position_signal` 与 `williams_r_14`，方便后续 Skill / AI 直接消费。
    assert!(output["data"]["range_position_signal"].is_string());
    // 2026-03-29 23:10 CST: 这里补充布林带合同可见性断言，原因是本轮要把布林带位置和带宽正式接进对外结果；
    // 目的：确保成功主样本至少能暴露 `bollinger_position_signal`、`bollinger_bandwidth_signal` 与 `boll_width_ratio_20`，方便后续 Skill / AI 直接消费。
    assert!(output["data"]["bollinger_position_signal"].is_string());
    // 2026-03-29 10:30 CST: 这里补充布林带中轨合同可见性断言，原因是本轮要把中轨支撑/压制正式接进对外结果，
    // 目的：确保成功主样本至少能暴露 `bollinger_midline_signal`，方便后续 Skill / AI 统一消费布林带三层语义。
    assert!(output["data"]["bollinger_midline_signal"].is_string());
    assert!(output["data"]["bollinger_bandwidth_signal"].is_string());
    assert!(output["data"]["breakout_signal"].is_string());
    assert!(output["data"]["momentum_signal"].is_string());
    assert!(output["data"]["volatility_state"].is_string());
    assert!(output["data"]["summary"].is_string());
    assert!(output["data"]["recommended_actions"].is_array());
    assert!(output["data"]["watch_points"].is_array());
    // 2026-04-01 CST: 这里先锁组合结论对象的对外可见性，原因是方案 A 要把既有技术面信号上提成更易消费的证券分析输出；
    // 目的：确保后续不是只补一段中文文案，而是正式暴露稳定的 `bias / confidence / headline / rationale / risk_flags` 合同。
    assert!(output["data"]["consultation_conclusion"]["bias"].is_string());
    assert!(output["data"]["consultation_conclusion"]["confidence"].is_string());
    assert!(output["data"]["consultation_conclusion"]["headline"].is_string());
    assert!(output["data"]["consultation_conclusion"]["rationale"].is_array());
    assert!(output["data"]["consultation_conclusion"]["risk_flags"].is_array());
    assert!(output["data"]["indicator_snapshot"]["close"].is_number());
    assert!(output["data"]["indicator_snapshot"]["ema_10"].is_number());
    assert!(output["data"]["indicator_snapshot"]["sma_50"].is_number());
    assert!(output["data"]["indicator_snapshot"]["sma_200"].is_number());
    assert!(output["data"]["indicator_snapshot"]["macd"].is_number());
    assert!(output["data"]["indicator_snapshot"]["rsi_14"].is_number());
    assert!(output["data"]["indicator_snapshot"]["mfi_14"].is_number());
    assert!(output["data"]["indicator_snapshot"]["cci_20"].is_number());
    assert!(output["data"]["indicator_snapshot"]["williams_r_14"].is_number());
    assert!(output["data"]["indicator_snapshot"]["boll_width_ratio_20"].is_number());
    assert!(output["data"]["indicator_snapshot"]["support_level_20"].is_number());
    assert!(output["data"]["indicator_snapshot"]["resistance_level_20"].is_number());
    assert!(output["data"]["indicator_snapshot"]["atr_14"].is_number());
    assert!(output["data"]["indicator_snapshot"]["adx_14"].is_number());
    assert!(output["data"]["indicator_snapshot"]["plus_di_14"].is_number());
    assert!(output["data"]["indicator_snapshot"]["minus_di_14"].is_number());
    assert!(output["data"]["indicator_snapshot"]["obv"].is_number());
    assert!(output["data"]["indicator_snapshot"]["volume_sma_20"].is_number());
    assert!(output["data"]["indicator_snapshot"]["volume_ratio_20"].is_number());
    assert!(output["data"]["indicator_snapshot"]["k_9"].is_number());
    assert!(output["data"]["indicator_snapshot"]["d_9"].is_number());
    assert!(output["data"]["indicator_snapshot"]["j_9"].is_number());
    // 2026-03-29 CST: 这里先锁 RSRS 第一版最小快照合同，原因是这轮已经明确要把 RSRS 一起接进咨询输出；
    // 目的：确保后续实现不是只在文案层硬编码，而是真的把 beta / zscore 结构化暴露给上层。
    assert!(output["data"]["rsrs_signal"].is_string());
    assert!(output["data"]["indicator_snapshot"]["rsrs_beta_18"].is_number());
    assert!(output["data"]["indicator_snapshot"]["rsrs_zscore_18_60"].is_number());
    assert!(
        output["data"]["indicator_snapshot"]["plus_di_14"]
            .as_f64()
            .expect("plus_di_14 should be a number")
            > output["data"]["indicator_snapshot"]["minus_di_14"]
                .as_f64()
                .expect("minus_di_14 should be a number")
    );
    assert!(
        output["data"]["indicator_snapshot"]["volume_ratio_20"]
            .as_f64()
            .expect("volume_ratio_20 should be a number")
            > 1.0
    );
}

#[test]
fn technical_consultation_basic_rejects_when_history_is_insufficient() {
    let runtime_db_path = create_test_runtime_db("technical_consultation_basic_insufficient");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_insufficient",
        "short_prices.csv",
        &build_bullish_history_rows(30),
    );
    import_history_csv(&runtime_db_path, &csv_path, "600519.SH");

    let request = json!({
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "600519.SH"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);

    // 2026-03-28 CST: 这里先锁数据不足错误口径，原因是 200 日均线和 ATR/BOLL 等窗口都对样本长度敏感；
    // 目的：要求第一版实现宁可明确报“历史数据不足”，也不要在窗口不够时静默给出误导性结论。
    assert_eq!(output["status"], "error");
    assert!(
        output["error"]
            .as_str()
            .expect("error should exist")
            .contains("历史数据不足")
    );
}

#[test]
fn technical_consultation_basic_marks_choppy_history_as_weak_trend() {
    let runtime_db_path = create_test_runtime_db("technical_consultation_basic_choppy");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_choppy",
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

    // 2026-03-29 CST: 这里先补“横盘弱趋势”红测，原因是方案 A 不应只对单边上涨样本生效；
    // 目的：确保 ADX 引入后，咨询结果能稳定区分 sideways + weak，而不是继续只给模糊方向。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["trend_bias"], "sideways");
    assert_eq!(output["data"]["trend_strength"], "weak");
    // 2026-04-01 CST: 这里补“区间等待态”组合结论断言，原因是方案 A 不能只锁突破和陷阱场景；
    // 目的：确保横盘弱趋势样本会稳定落到 `range_wait + low`，而不是被误抬成带方向性的区间观察态。
    assert_eq!(
        output["data"]["consultation_conclusion"]["bias"],
        "range_wait"
    );
    assert_eq!(
        output["data"]["consultation_conclusion"]["confidence"],
        "low"
    );
    // 2026-04-01 CST: 这里继续补 `range_wait` 的证据层断言，原因是方案 A-4 不能只锁中性 headline，
    // 目的：确保横盘弱趋势场景会把“趋势强度不足、先等区间重新选边”的解释和风控提示正式输出给上层。
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
    assert!(
        output["data"]["consultation_conclusion"]["headline"]
            .as_str()
            .expect("headline should exist")
            .contains("区间震荡")
    );
}

#[test]
fn technical_consultation_basic_marks_bearish_range_watch_conclusion() {
    let runtime_db_path =
        create_test_runtime_db("technical_consultation_basic_bearish_range_watch");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_bearish_range_watch",
        "bearish_range_watch.csv",
        &build_bearish_range_watch_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "600104.SH");

    let request = json!({
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "600104.SH"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);

    // 2026-04-01 CST: 这里补“偏空区间观察态”的 CLI 红测，原因是方案 A 目前还缺 `bearish_range_watch` 的真链路回归；
    // 目的：确保主趋势偏空但尚未有效跌破关键位时，组合结论会明确输出偏空等待，而不是被折叠成完全中性的 range_wait。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["breakout_signal"], "range_bound");
    assert_eq!(
        output["data"]["consultation_conclusion"]["bias"],
        "bearish_range_watch"
    );
    assert_eq!(
        output["data"]["consultation_conclusion"]["confidence"],
        "medium"
    );
    // 2026-04-01 CST: 这里继续补偏空区间观察态的解释证据与风险断言，原因是方案 A-2 不能只锁 `bias / confidence / headline`；
    // 目的：确保调用方能直接消费“为什么偏空等待”以及“当前主要风险是什么”，而不是再自行翻译底层字段。
    assert!(
        output["data"]["consultation_conclusion"]["rationale"]
            .as_array()
            .expect("rationale should exist")
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("偏空") && text.contains("等待支撑"))
    );
    assert!(
        output["data"]["consultation_conclusion"]["risk_flags"]
            .as_array()
            .expect("risk_flags should exist")
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("支撑跌破尚待确认"))
    );
    assert!(
        output["data"]["consultation_conclusion"]["headline"]
            .as_str()
            .expect("headline should exist")
            .contains("偏空结构仍在区间内")
    );
}

#[test]
fn technical_consultation_basic_marks_bullish_range_watch_conclusion() {
    let runtime_db_path =
        create_test_runtime_db("technical_consultation_basic_bullish_range_watch");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_bullish_range_watch",
        "bullish_range_watch.csv",
        &build_bullish_range_watch_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "300750.SZ");

    let request = json!({
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "300750.SZ"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);

    // 2026-04-01 CST: 这里补“偏多区间观察态”的 CLI 红测，原因是方案 A 当前还缺 `bullish_range_watch` 的真链路回归；
    // 目的：确保主趋势偏多但尚未有效突破关键位时，组合结论会明确输出偏多等待，而不是被折叠成完全中性的 range_wait。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["breakout_signal"], "range_bound");
    assert_eq!(
        output["data"]["consultation_conclusion"]["bias"],
        "bullish_range_watch"
    );
    assert_eq!(
        output["data"]["consultation_conclusion"]["confidence"],
        "medium"
    );
    // 2026-04-01 CST: 这里继续补偏多区间观察态的解释证据与风险断言，原因是方案 A-2 需要把区间方向态做成可直接消费的正式合同；
    // 目的：确保调用方不只拿到“偏多等待”的标签，还能读到等待阻力突破的理由与对应风险提示。
    assert!(
        output["data"]["consultation_conclusion"]["rationale"]
            .as_array()
            .expect("rationale should exist")
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("偏多") && text.contains("等待阻力"))
    );
    assert!(
        output["data"]["consultation_conclusion"]["risk_flags"]
            .as_array()
            .expect("risk_flags should exist")
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("阻力突破尚待确认"))
    );
    assert!(
        output["data"]["consultation_conclusion"]["headline"]
            .as_str()
            .expect("headline should exist")
            .contains("偏多结构仍在区间内")
    );
}

#[test]
fn technical_consultation_basic_marks_fading_volume_as_weakening_confirmation() {
    let runtime_db_path = create_test_runtime_db("technical_consultation_basic_fading_volume");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_fading_volume",
        "bullish_fading_volume.csv",
        &build_bullish_fading_volume_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "000001.SZ");

    let request = json!({
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "000001.SZ"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);

    // 2026-03-29 CST: 这里锁“上涨但缩量”红测，原因是量能确认不能只看价格方向；
    // 目的：确保后续实现能稳定把缩量上涨识别为 `weakening`，而不是继续给出默认确认。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["trend_bias"], "bullish");
    assert_eq!(output["data"]["volume_confirmation"], "weakening");
    assert!(
        output["data"]["indicator_snapshot"]["volume_ratio_20"]
            .as_f64()
            .expect("volume_ratio_20 should be a number")
            < 1.0
    );
}

#[test]
fn technical_consultation_basic_marks_kdj_oversold_rebound() {
    let runtime_db_path = create_test_runtime_db("technical_consultation_basic_kdj_oversold");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_kdj_oversold",
        "kdj_oversold.csv",
        &build_kdj_oversold_rebound_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "300122.SZ");

    let request = json!({
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "300122.SZ"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);

    // 2026-03-29 CST: 这里先锁 KDJ “超卖后回抽”的红测，原因是第一版择时层要先覆盖最常见的低位修复场景；
    // 目的：确保后续实现不是只暴露三个数值，而是真正给出 `oversold_rebound` 语义。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["timing_signal"], "oversold_rebound");
}

#[test]
fn technical_consultation_basic_marks_kdj_overbought_pullback() {
    let runtime_db_path = create_test_runtime_db("technical_consultation_basic_kdj_overbought");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_kdj_overbought",
        "kdj_overbought.csv",
        &build_kdj_overbought_pullback_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "301589.SZ");

    let request = json!({
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "301589.SZ"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);

    // 2026-03-29 CST: 这里再锁 KDJ “高位回落”的红测，原因是超买区回撤与超卖反抽需要成对覆盖；
    // 目的：确保后续实现能稳定给出 `overbought_pullback`，而不是把所有高位样本都压回 neutral。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["timing_signal"], "overbought_pullback");
}

#[test]
fn technical_consultation_basic_marks_rsrs_bullish_breakout() {
    let runtime_db_path = create_test_runtime_db("technical_consultation_basic_rsrs_bullish");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_rsrs_bullish",
        "rsrs_bullish.csv",
        &build_rsrs_bullish_breakout_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "688008.SH");

    let request = json!({
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "688008.SH"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);

    // 2026-03-29 CST: 这里先锁 RSRS “斜率强化”红测，原因是这轮不是只补两个数值字段，而是要把 RSRS 正式接进咨询语义；
    // 目的：确保后续实现能稳定给出 `bullish_breakout`，并把 RSRS 判断写进 summary / actions / watch_points。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["rsrs_signal"], "bullish_breakout");
    assert!(
        output["data"]["indicator_snapshot"]["rsrs_zscore_18_60"]
            .as_f64()
            .expect("rsrs_zscore_18_60 should be a number")
            > 0.7
    );
    assert!(
        output["data"]["summary"]
            .as_str()
            .expect("summary should exist")
            .contains("RSRS")
    );
    assert!(
        output["data"]["recommended_actions"]
            .as_array()
            .expect("recommended_actions should exist")
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("RSRS"))
    );
    assert!(
        output["data"]["watch_points"]
            .as_array()
            .expect("watch_points should exist")
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("RSRS"))
    );
}

#[test]
fn technical_consultation_basic_marks_rsrs_bearish_pressure() {
    let runtime_db_path = create_test_runtime_db("technical_consultation_basic_rsrs_bearish");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_rsrs_bearish",
        "rsrs_bearish.csv",
        &build_rsrs_bearish_pressure_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "002371.SZ");

    let request = json!({
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "002371.SZ"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);

    // 2026-03-29 CST: 这里再锁 RSRS “压力转强”红测，原因是 RSRS 第一版必须成对覆盖正负两侧信号；
    // 目的：确保后续实现能稳定给出 `bearish_pressure`，而不是把所有斜率走弱场景都压回 neutral。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["rsrs_signal"], "bearish_pressure");
    assert!(
        output["data"]["indicator_snapshot"]["rsrs_zscore_18_60"]
            .as_f64()
            .expect("rsrs_zscore_18_60 should be a number")
            < -0.7
    );
    assert!(
        output["data"]["summary"]
            .as_str()
            .expect("summary should exist")
            .contains("RSRS")
    );
    assert!(
        output["data"]["recommended_actions"]
            .as_array()
            .expect("recommended_actions should exist")
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("RSRS"))
    );
    assert!(
        output["data"]["watch_points"]
            .as_array()
            .expect("watch_points should exist")
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("RSRS"))
    );
}

#[test]
fn technical_consultation_basic_marks_price_obv_bearish_divergence() {
    let runtime_db_path = create_test_runtime_db("technical_consultation_basic_bearish_divergence");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_bearish_divergence",
        "bearish_divergence.csv",
        &build_bearish_divergence_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "300750.SZ");

    let request = json!( {
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "300750.SZ"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);

    // 2026-03-29 CST: 这里先锁价格新高但 OBV 不确认的红测，原因是背离识别第一版要先覆盖最常见的顶部风险提示；
    // 目的：确保后续实现能把“价格还强、量能已背离”的样本稳定标成 `bearish_divergence`。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["trend_bias"], "bullish");
    assert_eq!(output["data"]["divergence_signal"], "bearish_divergence");
}

#[test]
fn technical_consultation_basic_marks_price_obv_bullish_divergence() {
    let runtime_db_path = create_test_runtime_db("technical_consultation_basic_bullish_divergence");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_bullish_divergence",
        "bullish_divergence.csv",
        &build_bullish_divergence_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "002594.SZ");

    let request = json!( {
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "002594.SZ"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);

    // 2026-03-29 CST: 这里先锁价格创新低但 OBV 不再创新低的红测，原因是本轮要补齐 bullish_divergence；
    // 目的：确保后续实现不只是“理论上支持底背离”，而是能被真实夹具稳定识别出来。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["divergence_signal"], "bullish_divergence");
}

#[test]
fn technical_consultation_basic_keeps_none_when_price_and_obv_confirm_breakout() {
    let runtime_db_path = create_test_runtime_db("technical_consultation_basic_confirmed_breakout");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_confirmed_breakout",
        "confirmed_breakout.csv",
        &build_confirmed_breakout_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "688111.SH");

    let request = json!( {
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "688111.SH"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);

    // 2026-03-29 CST: 这里锁正常放量突破应保持 none，原因是背离逻辑不能污染健康突破样本；
    // 目的：把“价格和 OBV 同步确认”明确钉成 should-stay-none 的回归边界。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["trend_bias"], "bullish");
    assert_eq!(output["data"]["divergence_signal"], "none");
}

#[test]
fn technical_consultation_basic_marks_confirmed_resistance_breakout_signal() {
    let runtime_db_path =
        create_test_runtime_db("technical_consultation_basic_confirmed_resistance_breakout");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_confirmed_resistance_breakout",
        "confirmed_resistance_breakout.csv",
        &build_confirmed_breakout_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "688169.SH");

    let request = json!( {
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "688169.SH"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);
    let summary = output["data"]["summary"]
        .as_str()
        .expect("summary should exist");
    let recommended_actions = output["data"]["recommended_actions"]
        .as_array()
        .expect("recommended_actions should exist");
    let watch_points = output["data"]["watch_points"]
        .as_array()
        .expect("watch_points should exist");
    let close = output["data"]["indicator_snapshot"]["close"]
        .as_f64()
        .expect("close should be a number");
    let resistance_level_20 = output["data"]["indicator_snapshot"]["resistance_level_20"]
        .as_f64()
        .expect("resistance_level_20 should be a number");

    // 2026-03-31 CST: 这里先锁“放量突破前高”的关键位红测，原因是方案 1 要把支撑/阻力与突破有效性正式接入
    // technical_consultation_basic；目的：确保后续实现不仅返回文案，还会稳定暴露 `breakout_signal` 与 20 日阻力位数值。
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["breakout_signal"],
        "confirmed_resistance_breakout"
    );
    // 2026-04-01 CST: 这里先锁“有效向上突破”场景的组合结论，原因是方案 A 需要先把最强多头样本提升成上层证券分析结论；
    // 目的：确保上层不必自己重拼 trend/breakout/volume，就能直接消费 `bullish_continuation + high`。
    assert_eq!(
        output["data"]["consultation_conclusion"]["bias"],
        "bullish_continuation"
    );
    // 2026-04-01 CST: 这里把置信度收在 medium，原因是该样本虽然价格结构已确认突破，但历史上就刻意保留了量能偏弱这一维；
    // 目的：确保上层组合结论尊重“breakout_signal 是价格结构、volume_confirmation 是独立维度”的既有合同，不把所有 confirmed_breakout 都夸大成 high。
    assert_eq!(
        output["data"]["consultation_conclusion"]["confidence"],
        "medium"
    );
    // 2026-04-01 CST: 这里补 `bullish_continuation` 的证据层断言，原因是方案 A-4 需要把“已突破后的延续剧本”收成正式合同；
    // 目的：确保上层拿到的不只是 bias 标签，还能直接消费“多头延续为何成立、当前最该防什么”的结构化信息。
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
    assert!(
        output["data"]["consultation_conclusion"]["rationale"]
            .as_array()
            .expect("rationale should exist")
            .len()
            >= 2
    );
    assert!(output["data"]["indicator_snapshot"]["support_level_20"].is_number());
    assert!(output["data"]["indicator_snapshot"]["resistance_level_20"].is_number());
    assert!(close > resistance_level_20);
    assert!(summary.contains("阻力") || summary.contains("突破"));
    assert!(
        recommended_actions
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("突破") || text.contains("阻力"))
    );
    assert!(
        watch_points
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("突破") || text.contains("阻力"))
    );
}

#[test]
fn technical_consultation_basic_keeps_none_when_confirmed_breakdown_uses_alt_symbol() {
    let runtime_db_path =
        create_test_runtime_db("technical_consultation_basic_confirmed_breakdown");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_confirmed_breakdown",
        "confirmed_breakdown.csv",
        &build_confirmed_breakdown_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "000333.SZ");

    let request = json!( {
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "000333.SZ"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);

    // 2026-03-29 CST: 这里先锁“价格与 OBV 同步创新低”必须保持 none，原因是底部背离只应在 OBV 不再确认新低时才成立；
    // 目的：把确认性下跌从 bullish_divergence 的判定边界里排除出去，避免后续继续补背离时把同跌场景误伤成反转信号。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["divergence_signal"], "none");
}

#[test]
fn technical_consultation_basic_marks_confirmed_support_breakdown_signal() {
    let runtime_db_path =
        create_test_runtime_db("technical_consultation_basic_confirmed_support_breakdown");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_confirmed_support_breakdown",
        "confirmed_support_breakdown.csv",
        &build_confirmed_breakdown_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "600887.SH");

    let request = json!( {
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "600887.SH"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);
    let summary = output["data"]["summary"]
        .as_str()
        .expect("summary should exist");
    let recommended_actions = output["data"]["recommended_actions"]
        .as_array()
        .expect("recommended_actions should exist");
    let watch_points = output["data"]["watch_points"]
        .as_array()
        .expect("watch_points should exist");
    let close = output["data"]["indicator_snapshot"]["close"]
        .as_f64()
        .expect("close should be a number");
    let support_level_20 = output["data"]["indicator_snapshot"]["support_level_20"]
        .as_f64()
        .expect("support_level_20 should be a number");

    // 2026-03-31 CST: 这里补“有效跌破近端支撑”的关键位回归，原因是方案 1 不能只覆盖向上突破；
    // 目的：确保 technical_consultation_basic 会对称输出 `confirmed_support_breakdown` 与 20 日支撑位数值。
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["breakout_signal"],
        "confirmed_support_breakdown"
    );
    // 2026-04-01 CST: 这里先锁“有效向下跌破”场景的组合结论，原因是证券分析层不能只会输出多头延续而缺空头对称口径；
    // 目的：确保上层能直接消费 `bearish_continuation`，而不是继续依赖下游自己翻译 breakout_signal。
    assert_eq!(
        output["data"]["consultation_conclusion"]["bias"],
        "bearish_continuation"
    );
    // 2026-04-01 CST: 这里补 `bearish_continuation` 的证据层断言，原因是方案 A-4 也要把空头延续剧本锁成上层正式合同；
    // 目的：确保上层能直接读到“空头延续为何成立、当前需要防守的反抽风险是什么”，而不是再自行二次推断。
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
    assert!(output["data"]["indicator_snapshot"]["support_level_20"].is_number());
    assert!(output["data"]["indicator_snapshot"]["resistance_level_20"].is_number());
    assert!(close < support_level_20);
    assert!(summary.contains("支撑") || summary.contains("跌破"));
    assert!(
        recommended_actions
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("支撑") || text.contains("跌破"))
    );
    assert!(
        watch_points
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("支撑") || text.contains("跌破"))
    );
}

#[test]
fn technical_consultation_basic_marks_failed_resistance_breakout_signal() {
    let runtime_db_path =
        create_test_runtime_db("technical_consultation_basic_failed_resistance_breakout");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_failed_resistance_breakout",
        "failed_resistance_breakout.csv",
        &build_failed_breakout_reversal_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "603501.SH");

    let request = json!( {
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "603501.SH"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);
    let summary = output["data"]["summary"]
        .as_str()
        .expect("summary should exist");
    let recommended_actions = output["data"]["recommended_actions"]
        .as_array()
        .expect("recommended_actions should exist");
    let watch_points = output["data"]["watch_points"]
        .as_array()
        .expect("watch_points should exist");
    let close = output["data"]["indicator_snapshot"]["close"]
        .as_f64()
        .expect("close should be a number");
    let resistance_level_20 = output["data"]["indicator_snapshot"]["resistance_level_20"]
        .as_f64()
        .expect("resistance_level_20 should be a number");

    // 2026-03-31 CST: 这里先锁“假突破回落”的二阶段红测，原因是方案 1 下一刀不能只覆盖成功突破；
    // 目的：确保 technical_consultation_basic 会把上根越过阻力但最新一根收回关键位下方的场景识别成失效突破。
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["breakout_signal"],
        "failed_resistance_breakout"
    );
    // 2026-04-01 CST: 这里先锁“假突破回落”场景的组合结论，原因是方案 A 的上层证券分析必须能直接表达多头陷阱风险；
    // 目的：确保调用方拿到的是稳定的 `bull_trap_risk`，而不是再从 failed_* 文案里自行二次猜测。
    assert_eq!(
        output["data"]["consultation_conclusion"]["bias"],
        "bull_trap_risk"
    );
    // 2026-04-01 CST: 这里继续补多头陷阱态的证据层断言，原因是方案 A-3 不能只锁“这是陷阱”，还要锁“原有突破延续判断已经失效”；
    // 目的：确保调用方能直接消费陷阱态的解释与风险，而不是继续把它当成普通回落噪声。
    assert!(
        output["data"]["consultation_conclusion"]["rationale"]
            .as_array()
            .expect("rationale should exist")
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("突破延续判断") && text.contains("失效"))
    );
    assert!(
        output["data"]["consultation_conclusion"]["risk_flags"]
            .as_array()
            .expect("risk_flags should exist")
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("突破延续判断失效"))
    );
    assert!(close < resistance_level_20);
    assert!(summary.contains("假突破") || summary.contains("回落"));
    assert!(
        recommended_actions
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("假突破") || text.contains("回落"))
    );
    assert!(
        watch_points
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("阻力") && (text.contains("失守") || text.contains("收回")))
    );
}

#[test]
fn technical_consultation_basic_marks_failed_support_breakdown_signal() {
    let runtime_db_path =
        create_test_runtime_db("technical_consultation_basic_failed_support_breakdown");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_failed_support_breakdown",
        "failed_support_breakdown.csv",
        &build_failed_breakdown_recovery_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "002027.SZ");

    let request = json!( {
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "002027.SZ"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);
    let summary = output["data"]["summary"]
        .as_str()
        .expect("summary should exist");
    let recommended_actions = output["data"]["recommended_actions"]
        .as_array()
        .expect("recommended_actions should exist");
    let watch_points = output["data"]["watch_points"]
        .as_array()
        .expect("watch_points should exist");
    let close = output["data"]["indicator_snapshot"]["close"]
        .as_f64()
        .expect("close should be a number");
    let support_level_20 = output["data"]["indicator_snapshot"]["support_level_20"]
        .as_f64()
        .expect("support_level_20 should be a number");

    // 2026-03-31 CST: 这里先锁“假跌破拉回”的二阶段红测，原因是方案 1 下一刀要对称补齐下方关键位的失效场景；
    // 目的：确保 technical_consultation_basic 会把上根跌破支撑但最新一根重新收回支撑上方的场景识别成失效跌破。
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["breakout_signal"],
        "failed_support_breakdown"
    );
    // 2026-04-01 CST: 这里补“假跌破拉回”场景的组合结论断言，原因是当前只锁了多头陷阱，空头陷阱口径还没被真链路钉住；
    // 目的：确保调用方能直接消费 `bear_trap_risk`，并拿到与假跌破修复一致的 headline / risk_flags。
    assert_eq!(
        output["data"]["consultation_conclusion"]["bias"],
        "bear_trap_risk"
    );
    assert_eq!(
        output["data"]["consultation_conclusion"]["confidence"],
        "medium"
    );
    // 2026-04-01 CST: 这里继续补空头陷阱态的证据层断言，原因是方案 A-3 也要把“弱势延续判断已经失效”锁成正式合同；
    // 目的：确保调用方能直接知道当前不是单纯反抽，而是原有跌破延续逻辑被破坏。
    assert!(
        output["data"]["consultation_conclusion"]["rationale"]
            .as_array()
            .expect("rationale should exist")
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("弱势延续判断") && text.contains("失效"))
    );
    assert!(
        output["data"]["consultation_conclusion"]["headline"]
            .as_str()
            .expect("headline should exist")
            .contains("空头陷阱风险")
    );
    assert!(
        output["data"]["consultation_conclusion"]["risk_flags"]
            .as_array()
            .expect("risk_flags should exist")
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("跌破延续判断失效"))
    );
    assert!(close > support_level_20);
    assert!(summary.contains("假跌破") || summary.contains("拉回"));
    assert!(
        recommended_actions
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("假跌破") || text.contains("拉回"))
    );
    assert!(
        watch_points
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("支撑") && (text.contains("收复") || text.contains("拉回")))
    );
}

#[test]
fn technical_consultation_basic_marks_confirmed_resistance_retest_hold_signal() {
    let runtime_db_path =
        create_test_runtime_db("technical_consultation_basic_confirmed_resistance_retest_hold");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_confirmed_resistance_retest_hold",
        "confirmed_resistance_retest_hold.csv",
        &build_resistance_retest_hold_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "688036.SH");

    let request = json!( {
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "688036.SH"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);
    let summary = output["data"]["summary"]
        .as_str()
        .expect("summary should exist");
    let recommended_actions = output["data"]["recommended_actions"]
        .as_array()
        .expect("recommended_actions should exist");
    let watch_points = output["data"]["watch_points"]
        .as_array()
        .expect("watch_points should exist");
    let close = output["data"]["indicator_snapshot"]["close"]
        .as_f64()
        .expect("close should be a number");
    let resistance_level_20 = output["data"]["indicator_snapshot"]["resistance_level_20"]
        .as_f64()
        .expect("resistance_level_20 should be a number");

    // 2026-03-31 20:10 CST: 这里先锁“阻力转支撑回踩确认”的红测，原因是方案 A 要把突破后的第一次承接单独结构化；
    // 目的：确保 technical_consultation_basic 不会把已经完成回踩承接的场景继续误判成普通区间震荡。
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["breakout_signal"],
        "confirmed_resistance_retest_hold"
    );
    assert!(close < resistance_level_20);
    assert!(summary.contains("回踩") || summary.contains("承接"));
    assert!(
        recommended_actions
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("回踩") || text.contains("支撑"))
    );
    assert!(
        watch_points
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("阻力") && (text.contains("支撑") || text.contains("承接")))
    );
}

#[test]
fn technical_consultation_basic_marks_confirmed_support_retest_reject_signal() {
    let runtime_db_path =
        create_test_runtime_db("technical_consultation_basic_confirmed_support_retest_reject");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_confirmed_support_retest_reject",
        "confirmed_support_retest_reject.csv",
        &build_support_retest_reject_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "600585.SH");

    let request = json!( {
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "600585.SH"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);
    let summary = output["data"]["summary"]
        .as_str()
        .expect("summary should exist");
    let recommended_actions = output["data"]["recommended_actions"]
        .as_array()
        .expect("recommended_actions should exist");
    let watch_points = output["data"]["watch_points"]
        .as_array()
        .expect("watch_points should exist");
    let close = output["data"]["indicator_snapshot"]["close"]
        .as_f64()
        .expect("close should be a number");
    let support_level_20 = output["data"]["indicator_snapshot"]["support_level_20"]
        .as_f64()
        .expect("support_level_20 should be a number");

    // 2026-03-31 20:10 CST: 这里先锁“支撑转阻力反抽受压”的红测，原因是方案 A 要把跌破后的第一次反抽失败单独结构化；
    // 目的：确保 technical_consultation_basic 不会把已经完成反抽受压的场景继续误判成普通区间震荡或假跌破修复。
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["breakout_signal"],
        "confirmed_support_retest_reject"
    );
    assert!(close > support_level_20);
    assert!(summary.contains("反抽") || summary.contains("受压"));
    assert!(
        recommended_actions
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("反抽") || text.contains("受压"))
    );
    assert!(
        watch_points
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("支撑") && (text.contains("阻力") || text.contains("受压")))
    );
}

#[test]
fn technical_consultation_basic_marks_resistance_retest_watch_signal() {
    let runtime_db_path =
        create_test_runtime_db("technical_consultation_basic_resistance_retest_watch");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_resistance_retest_watch",
        "resistance_retest_watch.csv",
        &build_resistance_retest_watch_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "688981.SH");

    let request = json!( {
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "688981.SH"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);
    let summary = output["data"]["summary"]
        .as_str()
        .expect("summary should exist");
    let recommended_actions = output["data"]["recommended_actions"]
        .as_array()
        .expect("recommended_actions should exist");
    let watch_points = output["data"]["watch_points"]
        .as_array()
        .expect("watch_points should exist");

    // 2026-03-31 20:35 CST: 这里先锁“阻力转支撑回踩观察态”的红测，原因是下一刀要明确区分回踩途中与回踩确认完成；
    // 目的：确保 technical_consultation_basic 能输出仍需继续观察承接质量的 retest_watch，而不是直接跳到 confirmed。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["breakout_signal"], "resistance_retest_watch");
    // 2026-04-01 CST: 这里先锁“多头回踩观察态”的组合结论，原因是方案 A 要把中间态也提升成可消费的上层证券分析信号；
    // 目的：确保等待确认的多头结构会稳定输出 `bullish_confirmation_watch`，而不是混回普通区间态。
    assert_eq!(
        output["data"]["consultation_conclusion"]["bias"],
        "bullish_confirmation_watch"
    );
    assert_eq!(
        output["data"]["consultation_conclusion"]["confidence"],
        "medium"
    );
    // 2026-04-01 CST: 这里继续补多头确认观察态的解释证据与风险断言，原因是方案 A-2 要把“观察中”也做成正式证券分析合同；
    // 目的：确保调用方能直接知道当前仍在等回踩承接确认，而不是只看到一个 watch 标签。
    assert!(
        output["data"]["consultation_conclusion"]["rationale"]
            .as_array()
            .expect("rationale should exist")
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("观察阶段"))
    );
    assert!(
        output["data"]["consultation_conclusion"]["risk_flags"]
            .as_array()
            .expect("risk_flags should exist")
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("回踩承接尚待确认"))
    );
    assert!(summary.contains("回踩") && summary.contains("观察"));
    assert!(
        recommended_actions
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("回踩") && text.contains("观察"))
    );
    assert!(
        watch_points
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("阻力") && (text.contains("观察") || text.contains("站稳")))
    );
}

#[test]
fn technical_consultation_basic_marks_support_retest_watch_signal() {
    let runtime_db_path =
        create_test_runtime_db("technical_consultation_basic_support_retest_watch");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_support_retest_watch",
        "support_retest_watch.csv",
        &build_support_retest_watch_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "601888.SH");

    let request = json!( {
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "601888.SH"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);
    let summary = output["data"]["summary"]
        .as_str()
        .expect("summary should exist");
    let recommended_actions = output["data"]["recommended_actions"]
        .as_array()
        .expect("recommended_actions should exist");
    let watch_points = output["data"]["watch_points"]
        .as_array()
        .expect("watch_points should exist");

    // 2026-03-31 20:35 CST: 这里先锁“支撑转阻力反抽观察态”的红测，原因是下一刀也要对称区分反抽途中与反抽受压确认完成；
    // 目的：确保 technical_consultation_basic 能输出仍需继续观察压制质量的 retest_watch，而不是直接跳到 confirmed。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["breakout_signal"], "support_retest_watch");
    // 2026-04-01 CST: 这里补“空头反抽观察态”的组合结论断言，原因是方案 A 当前只锁了多头观察态，空头对称口径还没被测试覆盖；
    // 目的：确保 `support_retest_watch` 会稳定上提成 `bearish_confirmation_watch`，并给出可直接展示的观察阶段 headline。
    assert_eq!(
        output["data"]["consultation_conclusion"]["bias"],
        "bearish_confirmation_watch"
    );
    assert_eq!(
        output["data"]["consultation_conclusion"]["confidence"],
        "medium"
    );
    // 2026-04-01 CST: 这里继续补空头确认观察态的解释证据与风险断言，原因是方案 A-2 需要把空头观察态也补成完整合同；
    // 目的：确保调用方能直接知道当前仍在等反抽受压确认，而不是继续自己拼 breakout 与摘要文案。
    assert!(
        output["data"]["consultation_conclusion"]["rationale"]
            .as_array()
            .expect("rationale should exist")
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("观察阶段"))
    );
    assert!(
        output["data"]["consultation_conclusion"]["risk_flags"]
            .as_array()
            .expect("risk_flags should exist")
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("反抽受压尚待确认"))
    );
    assert!(
        output["data"]["consultation_conclusion"]["headline"]
            .as_str()
            .expect("headline should exist")
            .contains("反抽观察阶段")
    );
    assert!(summary.contains("反抽") && summary.contains("观察"));
    assert!(
        recommended_actions
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("反抽") && text.contains("观察"))
    );
    assert!(
        watch_points
            .iter()
            .filter_map(|value| value.as_str())
            .any(|text| text.contains("支撑") && (text.contains("观察") || text.contains("受压")))
    );
}
#[test]
fn technical_consultation_basic_accepts_just_above_buffer_boundary_in_cli() {
    let runtime_db_path =
        create_test_runtime_db("technical_consultation_basic_just_above_buffer_boundary_cli");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_just_above_buffer_boundary_cli",
        "just_above_buffer_boundary.csv",
        &build_just_above_buffer_boundary_resistance_retest_hold_rows(),
    );
    import_history_csv(&runtime_db_path, &csv_path, "688100.SH");

    let request = json!({
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "688100.SH"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);
    let breakout_signal = output["data"]["breakout_signal"]
        .as_str()
        .expect("breakout_signal should exist");

    // 2026-04-01 CST: 这里补 CLI 层“刚好高于锚点+缓冲边界”回踩确认回归，原因是外层 breakout 合同里等于边界会优先落到 watch；
    // 目的：确保真链路下只要越过最小有效边界一点点，仍会稳定进入 confirmed_resistance_retest_hold。
    assert_eq!(output["status"], "ok");
    assert_eq!(breakout_signal, "confirmed_resistance_retest_hold");
}

#[test]
fn technical_consultation_basic_keeps_minimum_buffer_floor_in_cli_retest_watch() {
    let runtime_db_path =
        create_test_runtime_db("technical_consultation_basic_min_buffer_floor_cli");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_min_buffer_floor_cli",
        "minimum_buffer_floor_watch.csv",
        &build_min_buffer_floor_resistance_retest_watch_rows(),
    );
    import_history_csv(&runtime_db_path, &csv_path, "688101.SH");

    let request = json!({
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "688101.SH"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);
    let breakout_signal = output["data"]["breakout_signal"]
        .as_str()
        .expect("breakout_signal should exist");

    // 2026-04-01 CST: 这里先补 CLI 层“ATR 过小由最小缓冲 0.15 接管”的观察态红测，原因是浮点毛刺在真链路里更值得防回归；
    // 目的：把最小缓冲兜底也锁进外层回归，避免后续有人只顾源码级测试而放掉导入后行为。
    assert_eq!(output["status"], "ok");
    assert_eq!(breakout_signal, "resistance_retest_watch");
}

#[test]
fn technical_consultation_basic_ignores_stale_multi_bar_anchor_in_cli() {
    let runtime_db_path =
        create_test_runtime_db("technical_consultation_basic_stale_multi_bar_anchor_cli");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_stale_multi_bar_anchor_cli",
        "stale_multi_bar_anchor.csv",
        &build_stale_multi_bar_resistance_anchor_rows(),
    );
    import_history_csv(&runtime_db_path, &csv_path, "688102.SH");

    let request = json!({
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "688102.SH"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);
    let breakout_signal = output["data"]["breakout_signal"]
        .as_str()
        .expect("breakout_signal should exist");

    // 2026-04-01 CST: 这里先补 CLI 层“旧突破锚点超过多根 lookback 后应失效”的红测，原因是这类衰减规则最容易只在内部函数上被覆盖；
    // 目的：确保导入后的真实历史样本不会把过期锚点误判成仍然有效的多根回踩确认。
    assert_eq!(output["status"], "ok");
    assert_eq!(breakout_signal, "range_bound");
}

#[test]
fn technical_consultation_basic_marks_failed_resistance_breakout_just_below_boundary_in_cli() {
    let runtime_db_path =
        create_test_runtime_db("technical_consultation_basic_failed_resistance_boundary_cli");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_failed_resistance_boundary_cli",
        "failed_resistance_boundary.csv",
        &build_failed_resistance_breakout_just_below_boundary_rows(),
    );
    import_history_csv(&runtime_db_path, &csv_path, "688103.SH");

    let request = json!({
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "688103.SH"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);

    // 2026-04-01 CST: 这里先补 CLI 层“刚好跌破旧阻力-buffer 一点点”应进入 failed 的边界红测，原因是 failed 与 watch 只差一条很窄的边界；
    // 目的：确保真链路下假突破回落的失效口径不会被后续条件顺序或比较符号改坏。
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["breakout_signal"],
        "failed_resistance_breakout"
    );
}

#[test]
fn technical_consultation_basic_marks_failed_support_breakdown_just_above_boundary_in_cli() {
    let runtime_db_path =
        create_test_runtime_db("technical_consultation_basic_failed_support_boundary_cli");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_failed_support_boundary_cli",
        "failed_support_boundary.csv",
        &build_failed_support_breakdown_just_above_boundary_rows(),
    );
    import_history_csv(&runtime_db_path, &csv_path, "688104.SH");

    let request = json!({
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "688104.SH"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);

    // 2026-04-01 CST: 这里先补 CLI 层“刚好拉回旧支撑+buffer 一点点”应进入 failed 的边界红测，原因是空头侧也要和多头侧保持对称；
    // 目的：把假跌破拉回的最小有效失效边界也锁进外层回归，避免之后只修一边。
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["breakout_signal"],
        "failed_support_breakdown"
    );
}

#[test]
fn technical_consultation_basic_marks_multi_bar_resistance_retest_watch_in_cli() {
    let runtime_db_path =
        create_test_runtime_db("technical_consultation_basic_multi_bar_resistance_watch_cli");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_multi_bar_resistance_watch_cli",
        "multi_bar_resistance_watch.csv",
        &build_multi_bar_resistance_retest_watch_rows(),
    );
    import_history_csv(&runtime_db_path, &csv_path, "688105.SH");

    let request = json!({
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "688105.SH"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);

    // 2026-04-01 CST: 这里先补 CLI 层“多根回踩仍在旧阻力附近磨位”的观察态红测，原因是 confirmed 的多根样本已有，但多根 watch 还没落到外层；
    // 目的：确保 2~4 根磨位期间不会被误判成 confirmed 或 range_bound。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["breakout_signal"], "resistance_retest_watch");
}

#[test]
fn technical_consultation_basic_marks_multi_bar_support_retest_watch_in_cli() {
    let runtime_db_path =
        create_test_runtime_db("technical_consultation_basic_multi_bar_support_watch_cli");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_multi_bar_support_watch_cli",
        "multi_bar_support_watch.csv",
        &build_multi_bar_support_retest_watch_rows(),
    );
    import_history_csv(&runtime_db_path, &csv_path, "688106.SH");

    let request = json!({
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "688106.SH"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);

    // 2026-04-01 CST: 这里先补 CLI 层“多根反抽仍在旧支撑附近磨位”的观察态红测，原因是空头侧多根 watch 还没被真链路覆盖；
    // 目的：确保空头侧在多根反抽期间也能稳定停留在 support_retest_watch。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["breakout_signal"], "support_retest_watch");
}

#[test]
fn technical_consultation_basic_marks_multi_bar_resistance_retest_hold_signal() {
    let runtime_db_path =
        create_test_runtime_db("technical_consultation_basic_multi_bar_resistance_retest_hold");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_multi_bar_resistance_retest_hold",
        "multi_bar_resistance_retest_hold.csv",
        &build_multi_bar_resistance_retest_hold_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "603986.SH");

    let request = json!( {
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "603986.SH"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);
    let recommended_actions = output["data"]["recommended_actions"]
        .as_array()
        .expect("recommended_actions should exist");
    let watch_points = output["data"]["watch_points"]
        .as_array()
        .expect("watch_points should exist");

    // 2026-03-31 22:20 CST: 这里先锁“突破后经历多根回踩再重新站稳”的红测，原因是当前确认逻辑只认前一根突破，
    // 会漏掉多根回踩后的再次承接；目的：确保 breakout_signal 能识别更贴近日线实盘节奏的多根回踩确认。
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["breakout_signal"],
        "confirmed_resistance_retest_hold"
    );
    assert!(!recommended_actions.is_empty());
    assert!(!watch_points.is_empty());
}

#[test]
fn technical_consultation_basic_marks_multi_bar_support_retest_reject_signal() {
    let runtime_db_path =
        create_test_runtime_db("technical_consultation_basic_multi_bar_support_retest_reject");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_multi_bar_support_retest_reject",
        "multi_bar_support_retest_reject.csv",
        &build_multi_bar_support_retest_reject_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "000858.SZ");

    let request = json!( {
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "000858.SZ"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);
    let recommended_actions = output["data"]["recommended_actions"]
        .as_array()
        .expect("recommended_actions should exist");
    let watch_points = output["data"]["watch_points"]
        .as_array()
        .expect("watch_points should exist");

    // 2026-03-31 22:20 CST: 这里再锁“跌破后经历多根反抽再重新受压”的红测，原因是空头侧也存在同样的两根 K 线限制；
    // 目的：确保支撑转阻力的多根反抽结构能够继续落到 confirmed_support_retest_reject，而不是退化回 range_bound。
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["breakout_signal"],
        "confirmed_support_retest_reject"
    );
    assert!(!recommended_actions.is_empty());
    assert!(!watch_points.is_empty());
}

#[test]
fn technical_consultation_basic_keeps_none_when_price_and_obv_confirm_breakdown() {
    let runtime_db_path =
        create_test_runtime_db("technical_consultation_basic_confirmed_breakdown");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_confirmed_breakdown",
        "confirmed_breakdown.csv",
        &build_confirmed_breakdown_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "600519.SH");

    let request = json!( {
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "600519.SH"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);

    // 2026-03-29 CST: 这里锁“价格创新低且 OBV 同步创新低”必须保持 none，原因是方案 A / 1 需要把
    // 背离识别的最后一个确认性下破边界补齐；目的：确保只有“价格创新低但 OBV 未确认”才会触发 bullish_divergence。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["trend_bias"], "bearish");
    assert_eq!(output["data"]["divergence_signal"], "none");
}

#[test]
fn technical_consultation_basic_keeps_none_when_false_breakdown_lacks_obv_divergence() {
    let runtime_db_path = create_test_runtime_db("technical_consultation_basic_false_breakdown");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_false_breakdown",
        "false_breakdown.csv",
        &build_false_breakdown_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "002460.SZ");

    let request = json!( {
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "002460.SZ"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);

    // 2026-03-29 CST: 这里锁“低位假跌破 / 低位震荡”必须保持 none，原因是方案 A1 要把底部误报边界继续收紧；
    // 目的：确保只有“价格创新低且 OBV 明显不再确认新低”的清晰背离，才允许输出 bullish_divergence。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["divergence_signal"], "none");
}

#[test]
fn technical_consultation_basic_keeps_none_when_obv_falls_without_price_breakout() {
    let runtime_db_path =
        create_test_runtime_db("technical_consultation_basic_obv_pullback_without_breakout");
    let csv_path = create_stock_history_csv(
        "technical_consultation_basic_obv_pullback_without_breakout",
        "obv_pullback_without_breakout.csv",
        &build_obv_pullback_without_breakout_rows(220),
    );
    import_history_csv(&runtime_db_path, &csv_path, "601318.SH");

    let request = json!( {
        "tool": "technical_consultation_basic",
        "args": {
            "symbol": "601318.SH"
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db_path);

    // 2026-03-29 CST: 这里锁“价格没创新高但 OBV 回落”不得误判，原因是这类高位震荡回落是最常见的假阳性来源；
    // 目的：确保背离逻辑仍以价格创新高/创新低为前置条件，而不是只看到 OBV 走弱就直接报风险。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["divergence_signal"], "none");
}
