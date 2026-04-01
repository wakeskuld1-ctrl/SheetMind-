mod common;

use assert_cmd::Command;
use excel_skill::frame::result_ref_store::{PersistedResultDataset, ResultRefStore};
use polars::prelude::{DataFrame, NamedFrom, Series};
use serde_json::json;
use std::fs;
use std::path::Path;

use crate::common::{create_test_output_path, run_cli_with_json, thread_result_ref_store};

fn read_zip_entry_text(path: &Path, entry_name: &str) -> String {
    let file = fs::File::open(path).expect("xlsx file should exist");
    let mut archive = zip::ZipArchive::new(file).expect("xlsx file should be a zip archive");
    let mut entry = archive
        .by_name(entry_name)
        .expect("zip entry should exist in xlsx");
    let mut content = String::new();
    std::io::Read::read_to_string(&mut entry, &mut content)
        .expect("zip entry should be readable as text");
    content
}

#[test]
fn diagnostics_report_excel_report_handoff_guidance_prefers_review_when_degraded() {
    let result_ref = create_diagnostics_result_ref_for_excel_report_cli();
    let output_path =
        create_test_output_path("diagnostics_report_excel_report_degraded_handoff", "xlsx");
    let request = json!({
        "tool": "diagnostics_report_excel_report",
        "args": {
            "report_name": "经营诊断交付包",
            "output_path": output_path.to_string_lossy(),
            "result_ref": result_ref,
            "correlation": {
                "target_column": "target",
                "feature_columns": ["positive_signal"]
            },
            "distribution": {
                "column": "amount",
                "bins": 4
            },
            "trend": {
                "time_column": "month",
                "value_column": "missing_revenue"
            }
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-28 21:58 CST：这里先锁降级场景下的承接口径，原因是分析承接型摘要在 warning 出现时必须优先引导复核；
    // 目的：确保后续实现不会把有降级风险的数据包误导成“可直接进入建模”。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["format"], "xlsx");
    assert!(output_path.exists(), "xlsx output should exist");

    let shared_strings_xml = read_zip_entry_text(&output_path, "xl/sharedStrings.xml");
    assert!(shared_strings_xml.contains("建议立即复核"));
    assert!(shared_strings_xml.contains("暂不建议进入建模"));
    assert!(shared_strings_xml.contains("trend_analysis"));
}

fn zip_entry_exists(path: &Path, entry_name: &str) -> bool {
    let file = fs::File::open(path).expect("xlsx file should exist");
    let mut archive = zip::ZipArchive::new(file).expect("xlsx file should be a zip archive");
    archive.by_name(entry_name).is_ok()
}

fn create_diagnostics_result_ref_for_excel_report_cli() -> String {
    let store = thread_result_ref_store();
    let result_ref = ResultRefStore::create_result_ref();
    // 2026-03-28 23:40 CST；2026-03-28 23:59 CST 追加：这里复用组合诊断样本并单独保留给 Excel 交付红测，原因是 workbook 交付层必须锁定在同一份稳定统计样本之上；
    // 目的：确保后续失败与修复都围绕 diagnostics_report_excel_report 本身，而不是被输入样本漂移干扰。
    let dataframe = DataFrame::new(vec![
        Series::new(
            "month".into(),
            ["2026-01", "2026-02", "2026-03", "2026-04", "2026-05"],
        )
        .into(),
        Series::new(
            "target".into(),
            [10.0_f64, 20.0_f64, 30.0_f64, 40.0_f64, 50.0_f64],
        )
        .into(),
        Series::new(
            "positive_signal".into(),
            [12.0_f64, 22.0_f64, 32.0_f64, 42.0_f64, 52.0_f64],
        )
        .into(),
        Series::new(
            "negative_signal".into(),
            [50.0_f64, 40.0_f64, 30.0_f64, 20.0_f64, 10.0_f64],
        )
        .into(),
        Series::new(
            "amount".into(),
            [100.0_f64, 105.0_f64, 110.0_f64, 115.0_f64, 300.0_f64],
        )
        .into(),
        Series::new(
            "revenue".into(),
            [100.0_f64, 120.0_f64, 140.0_f64, 160.0_f64, 180.0_f64],
        )
        .into(),
    ])
    .unwrap();
    let record = PersistedResultDataset::from_dataframe(
        &result_ref,
        "seed_diagnostics_report_excel_report",
        vec!["seed_diagnostics_report_excel_report".to_string()],
        &dataframe,
    )
    .unwrap();
    store.save(&record).unwrap();
    result_ref
}

#[test]
fn tool_catalog_includes_diagnostics_report_excel_report() {
    let mut cmd = Command::cargo_bin("excel_skill").unwrap();
    let assert = cmd.assert().success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    // 2026-03-28 23:59 CST：这里先锁 catalog 暴露，原因是 workbook 交付 Tool 如果不进目录，后续 CLI 和编排层都无法发现；
    // 目的：先把“可发现性”作为独立行为钉住，避免只实现底层导出却忘记注册正式入口。
    assert!(
        json["data"]["tool_catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool == "diagnostics_report_excel_report")
    );
}

#[test]
fn diagnostics_report_excel_report_returns_workbook_ref_and_embedded_report() {
    let result_ref = create_diagnostics_result_ref_for_excel_report_cli();
    let request = json!({
        "tool": "diagnostics_report_excel_report",
        "args": {
            "report_name": "经营诊断交付包",
            "report_subtitle": "统计诊断首版",
            "result_ref": result_ref,
            "correlation": {
                "target_column": "target",
                "feature_columns": ["positive_signal", "negative_signal"]
            },
            "outlier": {
                "columns": ["amount"],
                "method": "iqr"
            },
            "distribution": {
                "column": "amount",
                "bins": 4
            },
            "trend": {
                "time_column": "month",
                "value_column": "revenue"
            }
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-28 23:59 CST：这里先锁“先返回组合诊断，再返回 workbook_ref”的主交付合同，原因是这次不是单纯文件导出，而是新的高层 Rust 交付入口；
    // 目的：保证上层既能拿 Excel 交付句柄，也能继续读取 diagnostics_report 的统一业务结果。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["format"], "workbook_ref");
    assert_eq!(output["data"]["diagnostics_result"]["report_status"], "ok");
    // 2026-03-29 00:34 CST：这里先把默认交付升级成 5 页合同，原因是方案3要求在现有四页之上继续补图表摘要页；
    // 目的：先从对外返回值层锁住“摘要增强 + 图表页”升级，而不是只在实现里偷偷多塞一页。
    assert_eq!(output["data"]["sheet_names"].as_array().unwrap().len(), 5);
    assert!(
        output["data"]["sheet_names"]
            .as_array()
            .unwrap()
            .iter()
            .any(|sheet| sheet == "图表摘要")
    );
    assert!(
        output["data"]["workbook_ref"]
            .as_str()
            .expect("workbook_ref should exist")
            .starts_with("workbook_")
    );
}

#[test]
fn diagnostics_report_excel_report_still_delivers_workbook_when_one_section_degrades() {
    let result_ref = create_diagnostics_result_ref_for_excel_report_cli();
    let request = json!({
        "tool": "diagnostics_report_excel_report",
        "args": {
            "report_name": "经营诊断交付包",
            "result_ref": result_ref,
            "correlation": {
                "target_column": "target",
                "feature_columns": ["positive_signal"]
            },
            "distribution": {
                "column": "amount",
                "bins": 4
            },
            "trend": {
                "time_column": "month",
                "value_column": "missing_revenue"
            }
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-28 23:59 CST：这里先锁“单 section 失败不阻断 workbook 交付”的降级规则，原因是用户已经明确要求组合诊断要有高层韧性；
    // 目的：确保后续实现即使遇到局部列错误，也仍然能交付可读工作簿，而不是整包失败。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["format"], "workbook_ref");
    assert_eq!(
        output["data"]["diagnostics_result"]["report_status"],
        "degraded"
    );
    assert!(
        output["data"]["diagnostics_result"]["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|warning| warning.as_str().unwrap().contains("trend_analysis"))
    );
    assert!(
        output["data"]["workbook_ref"]
            .as_str()
            .expect("workbook_ref should exist")
            .starts_with("workbook_")
    );
    assert!(
        output["data"]["sheet_names"]
            .as_array()
            .unwrap()
            .iter()
            .any(|sheet| sheet == "图表摘要")
    );
}

#[test]
fn diagnostics_report_excel_report_exports_xlsx_when_output_path_is_given() {
    let result_ref = create_diagnostics_result_ref_for_excel_report_cli();
    let output_path = create_test_output_path("diagnostics_report_excel_report", "xlsx");
    let request = json!({
        "tool": "diagnostics_report_excel_report",
        "args": {
            "report_name": "经营诊断交付包",
            "report_subtitle": "统计诊断首版",
            "output_path": output_path.to_string_lossy(),
            "result_ref": result_ref,
            "correlation": {
                "target_column": "target",
                "feature_columns": ["positive_signal", "negative_signal"]
            },
            "outlier": {
                "columns": ["amount"],
                "method": "iqr"
            },
            "distribution": {
                "column": "amount",
                "bins": 4
            },
            "trend": {
                "time_column": "month",
                "value_column": "revenue"
            }
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-28 23:59 CST：这里先锁真实 xlsx 落地，原因是这轮目标是 workbook-first 交付，而不是只返回内存句柄；
    // 目的：防止实现停留在 draft 层，确保传 output_path 时确实产出最终 Excel 文件。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["format"], "xlsx");
    assert!(output_path.exists(), "xlsx output should exist");

    let workbook_xml = read_zip_entry_text(&output_path, "xl/workbook.xml");
    assert!(workbook_xml.contains("执行摘要"));
    assert!(workbook_xml.contains("诊断概览"));
    assert!(workbook_xml.contains("相关性与异常"));
    assert!(workbook_xml.contains("分布与趋势"));
    // 2026-03-29 00:34 CST：这里先锁真实图表页存在，原因是 workbook-first 增强不能停留在返回 JSON 多一页；
    // 目的：确保导出的 xlsx 里真的有图表摘要页，而不是只改 sheet_names。
    assert!(workbook_xml.contains("图表摘要"));

    let shared_strings_xml = read_zip_entry_text(&output_path, "xl/sharedStrings.xml");
    // 2026-03-29 00:34 CST：这里先锁管理摘要字段落进真实 Excel，原因是方案3第一步就是让执行摘要更像管理交付；
    // 目的：防止后续实现只在内存结构里补字段，却没有真正写进最终工作簿。
    assert!(shared_strings_xml.contains("总体风险等级"));
    assert!(shared_strings_xml.contains("可直接决策"));
    assert!(shared_strings_xml.contains("优先处理方向"));
    // 2026-03-28 21:58 CST：这里先锁执行摘要的分析承接字段，原因是本轮要把诊断结果继续承接到复核、补数、建模，而不是只停留在管理摘要；
    // 目的：确保最终导出的 xlsx 真正写出“下一步怎么做”的交接字段，方便 AI 或人工直接接续。
    assert!(shared_strings_xml.contains("复核建议"));
    assert!(shared_strings_xml.contains("补数建议"));
    assert!(shared_strings_xml.contains("建模建议"));
    assert!(shared_strings_xml.contains("建议优先工具"));
    assert!(shared_strings_xml.contains("建议目标字段"));
    assert!(shared_strings_xml.contains("建议时间字段"));
    assert!(shared_strings_xml.contains("当前主要阻塞项"));
    assert!(shared_strings_xml.contains("进入下一步前需满足条件"));
    // 2026-03-28 21:58 CST：这里先锁字段承接值也要真实落进 xlsx，原因是分析承接不能只有标签，没有具体接续信息；
    // 目的：确保后续实现会把目标字段、时间字段等可直接消费的信息写入执行摘要。
    assert!(shared_strings_xml.contains("target"));
    assert!(shared_strings_xml.contains("month"));
    // 2026-03-29 01:16 CST：这里先锁图表数据区补齐，原因是方案A不只是多加图，还要把左侧数据源区整理成更完整的图表源表；
    // 目的：防止后续实现只新增 chart spec，却没有把 `分布区间 / 分布计数 / 异常数` 等字段真正写进交付页。
    assert!(shared_strings_xml.contains("分布区间"));
    assert!(shared_strings_xml.contains("分布计数"));
    assert!(shared_strings_xml.contains("异常数"));
    // 2026-03-29 00:34 CST：这里先锁至少生成一张真实图表，原因是图表页增强的交付价值不只是多一张空表；
    // 目的：确保后续实现真的走 workbook chart 导出主线。
    assert!(zip_entry_exists(&output_path, "xl/charts/chart1.xml"));
    // 2026-03-29 01:16 CST：这里先锁新增图表真正落盘，原因是方案A已经明确要继续补分布图和异常 Top 图；
    // 目的：确保这轮不是只调现有三张图的位置，而是把更高价值的统计图正式交付进 xlsx。
    assert!(zip_entry_exists(&output_path, "xl/charts/chart4.xml"));
    assert!(zip_entry_exists(&output_path, "xl/charts/chart5.xml"));
}

#[test]
fn diagnostics_report_excel_report_can_disable_chart_sheet() {
    let result_ref = create_diagnostics_result_ref_for_excel_report_cli();
    let request = json!({
        "tool": "diagnostics_report_excel_report",
        "args": {
            "report_name": "经营诊断交付包",
            "include_chart_sheet": false,
            "result_ref": result_ref,
            "correlation": {
                "target_column": "target",
                "feature_columns": ["positive_signal", "negative_signal"]
            },
            "outlier": {
                "columns": ["amount"],
                "method": "iqr"
            },
            "distribution": {
                "column": "amount",
                "bins": 4
            },
            "trend": {
                "time_column": "month",
                "value_column": "revenue"
            }
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-29 00:34 CST：这里先锁图表页开关兼容合同，原因是增强默认应更强，但仍要允许调用方回退为纯表格交付；
    // 目的：保证这次增强是向后兼容的，而不是把所有调用方都强制切到图表版 workbook。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["sheet_names"].as_array().unwrap().len(), 4);
    assert!(
        !output["data"]["sheet_names"]
            .as_array()
            .unwrap()
            .iter()
            .any(|sheet| sheet == "图表摘要")
    );
}
