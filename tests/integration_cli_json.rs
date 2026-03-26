mod common;

use assert_cmd::Command;
use calamine::{Data, Reader, open_workbook_auto};
use excel_skill::frame::result_ref_store::{PersistedResultDataset, ResultRefStore};
use polars::prelude::{DataFrame, NamedFrom, Series};
use predicates::str::contains;
use rusqlite::Connection;
use serde_json::json;

use crate::common::{
    create_chinese_path_fixture, create_positioned_workbook, create_test_output_path,
    create_test_runtime_db, create_test_workbook, run_cli_with_bytes, run_cli_with_json,
    run_cli_with_json_and_runtime, runtime_result_ref_store, thread_result_ref_store,
    thread_runtime_root,
};

fn create_datetime_result_ref_for_cli() -> String {
    let store = thread_result_ref_store();
    let result_ref = ResultRefStore::create_result_ref();
    // 2026-03-23: 这里手工构造日期时间文本结果集，目的是让 CLI 测试直接覆盖 result_ref 输入链路。
    let dataframe = DataFrame::new(vec![
        Series::new("biz_date".into(), ["2026/03/01", "2026-03-02"]).into(),
        Series::new(
            "created_at".into(),
            ["2026-03-01 8:30", "2026-03-02T09:15:20"],
        )
        .into(),
    ])
    .unwrap();
    let record = PersistedResultDataset::from_dataframe(
        &result_ref,
        "seed_parse_datetime",
        vec!["seed_datetime".to_string()],
        &dataframe,
    )
    .unwrap();
    store.save(&record).unwrap();
    result_ref
}

fn create_summary_result_ref_for_cli() -> String {
    let store = thread_result_ref_store();
    let result_ref = ResultRefStore::create_result_ref();
    // 2026-03-23: 这里手工构造摘要结果集，原因是 report_delivery 第一轮需要稳定的摘要页输入；目的是先锁定标准汇报模板链路。
    let dataframe = DataFrame::new(vec![
        Series::new("指标".into(), ["总客户数", "总收入"]).into(),
        Series::new("值".into(), ["2", "215"]).into(),
    ])
    .unwrap();
    let record = PersistedResultDataset::from_dataframe(
        &result_ref,
        "seed_report_delivery_summary",
        vec!["seed_summary".to_string()],
        &dataframe,
    )
    .unwrap();
    store.save(&record).unwrap();
    result_ref
}

fn create_chart_analysis_result_ref_for_cli() -> String {
    let store = thread_result_ref_store();
    let result_ref = ResultRefStore::create_result_ref();
    // 2026-03-23: 这里手工构造可画图的数值分析结果集，原因是图表导出测试需要稳定的分类轴和数值轴；目的是避免直接依赖 Excel 读取后的文本类型让图表测试发散。
    let dataframe = DataFrame::new(vec![
        Series::new("region".into(), ["North", "South", "West"]).into(),
        Series::new("sales".into(), [120_i64, 95_i64, 88_i64]).into(),
    ])
    .unwrap();
    let record = PersistedResultDataset::from_dataframe(
        &result_ref,
        "seed_report_delivery_chart_analysis",
        vec!["seed_chart_analysis".to_string()],
        &dataframe,
    )
    .unwrap();
    store.save(&record).unwrap();
    result_ref
}

fn create_multi_series_analysis_result_ref_for_cli() -> String {
    let store = thread_result_ref_store();
    let result_ref = ResultRefStore::create_result_ref();
    // 2026-03-23: 这里手工构造多系列图表分析结果集，原因是多系列导出测试需要稳定的公共分类轴和两个数值系列；目的是避免把图表测试绑定到上游聚合流程。
    let dataframe = DataFrame::new(vec![
        Series::new("month".into(), ["1月", "2月", "3月"]).into(),
        Series::new("revenue".into(), [120_i64, 150_i64, 132_i64]).into(),
        Series::new("profit".into(), [35_i64, 40_i64, 38_i64]).into(),
    ])
    .unwrap();
    let record = PersistedResultDataset::from_dataframe(
        &result_ref,
        "seed_report_delivery_multi_series_analysis",
        vec!["seed_multi_series_analysis".to_string()],
        &dataframe,
    )
    .unwrap();
    store.save(&record).unwrap();
    result_ref
}

fn create_ascii_summary_result_ref_for_cli() -> String {
    let store = thread_result_ref_store();
    let result_ref = ResultRefStore::create_result_ref();
    // 2026-03-23: 这里补一份 ASCII 摘要结果集，原因是图表与模板增强测试更关注导出结构；目的是避免历史中文乱码文本干扰断言稳定性。
    let dataframe = DataFrame::new(vec![
        Series::new("metric".into(), ["customer_count", "revenue_total"]).into(),
        Series::new("value".into(), ["2", "215"]).into(),
    ])
    .unwrap();
    let record = PersistedResultDataset::from_dataframe(
        &result_ref,
        "seed_ascii_summary",
        vec!["seed_ascii_summary".to_string()],
        &dataframe,
    )
    .unwrap();
    store.save(&record).unwrap();
    result_ref
}

fn create_pie_analysis_result_ref_for_cli() -> String {
    let store = thread_result_ref_store();
    let result_ref = ResultRefStore::create_result_ref();
    // 2026-03-23: 这里构造饼图分析结果集，原因是要先锁定 pie 图的最小导出闭环；目的是让图表类型扩展测试不依赖上游聚合链路。
    let dataframe = DataFrame::new(vec![
        Series::new("segment".into(), ["travel", "hotel", "ticket"]).into(),
        Series::new("share".into(), [40_i64, 35_i64, 25_i64]).into(),
    ])
    .unwrap();
    let record = PersistedResultDataset::from_dataframe(
        &result_ref,
        "seed_pie_analysis",
        vec!["seed_pie_analysis".to_string()],
        &dataframe,
    )
    .unwrap();
    store.save(&record).unwrap();
    result_ref
}

fn create_scatter_analysis_result_ref_for_cli() -> String {
    let store = thread_result_ref_store();
    let result_ref = ResultRefStore::create_result_ref();
    // 2026-03-23: 这里构造散点图分析结果集，原因是 scatter 需要明确的 X/Y 数值列；目的是让散点图测试专注于导出协议而不是数据准备。
    let dataframe = DataFrame::new(vec![
        Series::new("x_value".into(), [1_i64, 2_i64, 3_i64, 4_i64]).into(),
        Series::new("y_value".into(), [10_i64, 13_i64, 17_i64, 20_i64]).into(),
    ])
    .unwrap();
    let record = PersistedResultDataset::from_dataframe(
        &result_ref,
        "seed_scatter_analysis",
        vec!["seed_scatter_analysis".to_string()],
        &dataframe,
    )
    .unwrap();
    store.save(&record).unwrap();
    result_ref
}

fn create_chart_ref_from_result_ref_for_cli(
    result_ref: &str,
    chart_type: &str,
    category_column: &str,
    title: &str,
    series: serde_json::Value,
) -> String {
    let request = json!({
        "tool": "build_chart",
        "args": {
            "source": {
                "result_ref": result_ref
            },
            "chart_type": chart_type,
            "title": title,
            "category_column": category_column,
            "series": series
        }
    });
    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    output["data"]["chart_ref"].as_str().unwrap().to_string()
}

fn create_delivery_format_result_ref_for_cli() -> String {
    let store = thread_result_ref_store();
    let result_ref = ResultRefStore::create_result_ref();
    // 2026-03-24: 这里构造同时包含浮点数和长文本的结果集，原因是要锁定交付层默认数字格式与自动换行；目的是让样式增强测试不依赖上游分析流程偶然产出的数据形态。
    let dataframe = DataFrame::new(vec![
        Series::new("customer".into(), ["A客户", "B客户"]).into(),
        Series::new("amount".into(), [12345.678_f64, 987.2_f64]).into(),
        Series::new(
            "note".into(),
            [
                "这是一个特别长的说明字段，用来验证导出后的 Excel 会自动开启换行样式，避免业务用户打开时一整列被截断。",
                "短说明",
            ],
        )
        .into(),
    ])
    .unwrap();
    let record = PersistedResultDataset::from_dataframe(
        &result_ref,
        "seed_delivery_format",
        vec!["seed_delivery_format".to_string()],
        &dataframe,
    )
    .unwrap();
    store.save(&record).unwrap();
    result_ref
}

fn workbook_ref_json_path(workbook_ref: &str) -> std::path::PathBuf {
    thread_runtime_root()
        .join("workbook_refs")
        .join(format!("{workbook_ref}.json"))
}

fn read_zip_entry_text(path: &std::path::Path, entry_name: &str) -> String {
    let file = std::fs::File::open(path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let mut entry = archive.by_name(entry_name).unwrap();
    let mut content = String::new();
    std::io::Read::read_to_string(&mut entry, &mut content).unwrap();
    content
}

#[test]
fn cli_without_args_returns_json_help() {
    let mut cmd = Command::cargo_bin("excel_skill").unwrap();
    cmd.assert().success().stdout(contains("tool_catalog"));
}

#[test]
fn cli_dispatches_open_workbook_json_request() {
    let request = json!({
        "tool": "open_workbook",
        "args": {
            "path": "tests/fixtures/basic-sales.xlsx"
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["sheet_names"][0], "Sales");
}

#[test]
fn open_workbook_returns_file_ref_and_indexed_sheets_for_follow_up_selection() {
    let request = json!({
        "tool": "open_workbook",
        "args": {
            "path": "tests/fixtures/basic-sales.xlsx"
        }
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "ok");
    // 2026-03-23: 这里先锁定 open_workbook 会补充 file_ref 与带序号的 Sheet 目录，目的是让后续流程可以按“第几个 Sheet”继续，而不是重复依赖中文或长字符串 Sheet 名。
    assert!(
        output["data"]["file_ref"]
            .as_str()
            .unwrap()
            .starts_with("file_")
    );
    assert_eq!(
        output["data"]["sheets"],
        json!([
            { "sheet_index": 1, "sheet_name": "Sales" },
            { "sheet_index": 2, "sheet_name": "Lookup" }
        ])
    );
}

#[test]
fn tool_catalog_includes_inspect_sheet_range() {
    let mut cmd = Command::cargo_bin("excel_skill").unwrap();
    let assert = cmd.assert().success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    // 2026-03-22: 这里先锁定目录里暴露 inspect_sheet_range，目的是让 Skill 能显式发现区域探查入口。
    assert!(
        json["data"]["tool_catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool == "inspect_sheet_range")
    );
}

#[test]
fn inspect_sheet_range_returns_used_range_and_sample() {
    let workbook_path = create_positioned_workbook(
        "inspect_sheet_range_cli",
        "offset-inspect-cli.xlsx",
        &[(
            "Report",
            vec![
                (2, 1, "user_id"),
                (2, 2, "region"),
                (2, 3, "sales"),
                (3, 1, "1001"),
                (3, 2, "North"),
                (3, 3, "88"),
                (4, 1, "1002"),
                (4, 2, "South"),
                (4, 3, "95"),
            ],
        )],
    );
    let request = json!({
        "tool": "inspect_sheet_range",
        "args": {
            "path": workbook_path.to_string_lossy(),
            "sheet": "Report",
            "sample_rows": 2
        }
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "ok");
    // 2026-03-22: 这里先锁定 CLI 层会把 used range 与样本结构稳定透出，目的是给上层问答界面明确的下一步依据。
    assert_eq!(output["data"]["used_range"], "B3:D5");
    assert_eq!(output["data"]["row_count_estimate"], 3);
    assert_eq!(output["data"]["column_count_estimate"], 3);
    assert_eq!(output["data"]["sample_rows"][0]["row_number"], 3);
    assert_eq!(
        output["data"]["sample_rows"][0]["values"],
        json!(["user_id", "region", "sales"])
    );
    assert_eq!(
        output["data"]["sample_rows"][1]["values"],
        json!(["1001", "North", "88"])
    );
}

#[test]
fn inspect_sheet_range_accepts_file_ref_and_sheet_index() {
    let workbook_path = create_positioned_workbook(
        "inspect_sheet_range_file_ref_cli",
        "offset-inspect-file-ref-cli.xlsx",
        &[(
            "Report",
            vec![
                (2, 1, "user_id"),
                (2, 2, "region"),
                (2, 3, "sales"),
                (3, 1, "1001"),
                (3, 2, "North"),
                (3, 3, "88"),
                (4, 1, "1002"),
                (4, 2, "South"),
                (4, 3, "95"),
            ],
        )],
    );
    let open_request = json!({
        "tool": "open_workbook",
        "args": {
            "path": workbook_path.to_string_lossy()
        }
    });
    let open_output = run_cli_with_json(&open_request.to_string());
    let file_ref = open_output["data"]["file_ref"].as_str().unwrap();

    let inspect_request = json!({
        "tool": "inspect_sheet_range",
        "args": {
            "file_ref": file_ref,
            "sheet_index": 1,
            "sample_rows": 2
        }
    });

    let output = run_cli_with_json(&inspect_request.to_string());

    assert_eq!(output["status"], "ok");
    // 2026-03-23: 这里先锁定 inspect_sheet_range 可以直接消费 file_ref + sheet_index，目的是避免后续再次传递中文或超长 Sheet 名。
    assert_eq!(output["data"]["sheet"], "Report");
    assert_eq!(output["data"]["used_range"], "B3:D5");
}

#[test]
fn tool_catalog_includes_load_table_region() {
    let mut cmd = Command::cargo_bin("excel_skill").unwrap();
    let assert = cmd.assert().success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    // 2026-03-22: 这里先锁定目录里暴露 load_table_region，目的是让 Skill 能从 inspect 结果继续进入显式区域加载。
    assert!(
        json["data"]["tool_catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool == "load_table_region")
    );
}

#[test]
fn load_table_region_returns_preview_and_result_ref() {
    let workbook_path = create_positioned_workbook(
        "load_table_region_cli",
        "region-cli.xlsx",
        &[(
            "Report",
            vec![
                (2, 1, "user_id"),
                (2, 2, "region"),
                (2, 3, "sales"),
                (3, 1, "1001"),
                (3, 2, "North"),
                (3, 3, "88"),
                (4, 1, "1002"),
                (4, 2, "South"),
                (4, 3, "95"),
            ],
        )],
    );
    let request = json!({
        "tool": "load_table_region",
        "args": {
            "path": workbook_path.to_string_lossy(),
            "sheet": "Report",
            "range": "B3:D5",
            "header_row_count": 1
        }
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "ok");
    // 2026-03-22: ????? CLI ??????????result_ref ? table_ref?????????????????????
    assert_eq!(
        output["data"]["columns"],
        json!(["user_id", "region", "sales"])
    );
    assert_eq!(output["data"]["row_count"], 2);
    assert_eq!(output["data"]["rows"][0]["user_id"], "1001");
    assert_eq!(output["data"]["rows"][1]["sales"], "95");
    assert!(
        output["data"]["result_ref"]
            .as_str()
            .unwrap()
            .starts_with("result_")
    );
    assert!(
        output["data"]["table_ref"]
            .as_str()
            .unwrap()
            .starts_with("table_")
    );
}

#[test]
fn preview_table_accepts_table_ref_from_load_table_region() {
    let workbook_path = create_positioned_workbook(
        "load_table_region_table_ref_cli",
        "region-table-ref-cli.xlsx",
        &[(
            "Report",
            vec![
                (2, 1, "user_id"),
                (2, 2, "region"),
                (2, 3, "sales"),
                (3, 1, "1001"),
                (3, 2, "North"),
                (3, 3, "88"),
                (4, 1, "1002"),
                (4, 2, "South"),
                (4, 3, "95"),
            ],
        )],
    );
    let load_request = json!({
        "tool": "load_table_region",
        "args": {
            "path": workbook_path.to_string_lossy(),
            "sheet": "Report",
            "range": "B3:D5",
            "header_row_count": 1
        }
    });
    let load_output = run_cli_with_json(&load_request.to_string());
    let table_ref = load_output["data"]["table_ref"].as_str().unwrap();

    let preview_request = json!({
        "tool": "preview_table",
        "args": {
            "table_ref": table_ref,
            "limit": 5
        }
    });
    let output = run_cli_with_json(&preview_request.to_string());

    assert_eq!(output["status"], "ok");
    // 2026-03-22: ????? load_table_region ??? table_ref ????? Tool ???????????????????????
    assert_eq!(output["data"]["row_count"], 2);
    assert_eq!(output["data"]["rows"][0]["user_id"], "1001");
    assert_eq!(output["data"]["rows"][1]["sales"], "95");
}

#[test]
fn load_table_region_updates_session_state_and_mirrors_region_table_ref() {
    let runtime_db = create_test_runtime_db("load_table_region_state");
    let workbook_path = create_positioned_workbook(
        "load_table_region_state_cli",
        "region-state-cli.xlsx",
        &[(
            "Report",
            vec![
                (2, 1, "user_id"),
                (2, 2, "region"),
                (2, 3, "sales"),
                (3, 1, "1001"),
                (3, 2, "North"),
                (3, 3, "88"),
                (4, 1, "1002"),
                (4, 2, "South"),
                (4, 3, "95"),
            ],
        )],
    );
    let load_request = json!({
        "tool": "load_table_region",
        "args": {
            "session_id": "session_load_table_region",
            "path": workbook_path.to_string_lossy(),
            "sheet": "Report",
            "range": "B3:D5",
            "header_row_count": 1
        }
    });
    let load_output = run_cli_with_json_and_runtime(&load_request.to_string(), &runtime_db);
    assert_eq!(load_output["status"], "ok");
    let table_ref = load_output["data"]["table_ref"]
        .as_str()
        .unwrap()
        .to_string();

    let get_request = json!({
        "tool": "get_session_state",
        "args": {
            "session_id": "session_load_table_region"
        }
    });
    let state_output = run_cli_with_json_and_runtime(&get_request.to_string(), &runtime_db);

    // 2026-03-22: 这里锁定显式区域加载后会同步会话摘要，目的是让总入口能够直接记住当前局部表上下文。
    assert_eq!(state_output["status"], "ok");
    assert_eq!(
        state_output["data"]["current_workbook"],
        workbook_path.to_string_lossy().to_string()
    );
    assert_eq!(state_output["data"]["current_sheet"], "Report");
    assert_eq!(state_output["data"]["current_stage"], "table_processing");
    assert_eq!(state_output["data"]["schema_status"], "confirmed");
    assert_eq!(state_output["data"]["active_table_ref"], table_ref);

    let connection = Connection::open(&runtime_db).unwrap();
    let mirrored_region: String = connection
        .query_row(
            "SELECT region FROM table_refs WHERE table_ref = ?1",
            [&table_ref],
            |row| row.get(0),
        )
        .unwrap();

    // 2026-03-22: 这里锁定本地记忆层会把 region table_ref 的显式范围一起镜像下来，目的是后续恢复局部上下文时不丢区域信息。
    assert_eq!(mirrored_region, "B3:D5");
}

#[test]
fn tool_catalog_includes_list_sheets() {
    let mut cmd = Command::cargo_bin("excel_skill").unwrap();
    let assert = cmd.assert().success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    // 2026-03-22: ?????????? list_sheets????????????????? I/O Tool?
    assert!(
        json["data"]["tool_catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool == "list_sheets")
    );
}

#[test]
fn list_sheets_returns_visible_sheet_names() {
    let request = json!({
        "tool": "list_sheets",
        "args": {
            "path": "tests/fixtures/basic-sales.xlsx"
        }
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "ok");
    // 2026-03-22: ????? list_sheets ?????? open_workbook ????????????????????????? sheet ???
    assert_eq!(output["data"]["sheet_names"], json!(["Sales", "Lookup"]));
}

#[test]
fn normalize_table_accepts_file_ref_and_sheet_index_without_sheet_name() {
    let open_request = json!({
        "tool": "open_workbook",
        "args": {
            "path": "tests/fixtures/basic-sales.xlsx"
        }
    });
    let open_output = run_cli_with_json(&open_request.to_string());
    let file_ref = open_output["data"]["file_ref"].as_str().unwrap();

    let request = json!({
        "tool": "normalize_table",
        "args": {
            "file_ref": file_ref,
            "sheet_index": 1
        }
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "ok");
    // 2026-03-23: 这里先锁定 normalize_table 能按“第几个 Sheet”继续，目的是让表头判断不再依赖再次传递 Sheet 名。
    assert_eq!(output["data"]["sheet"], "Sales");
    assert_eq!(output["data"]["columns"][0]["canonical_name"], "user_id");
}

#[test]
fn apply_header_schema_accepts_file_ref_and_sheet_index_and_updates_session_state() {
    let runtime_db = create_test_runtime_db("apply_header_schema_file_ref");
    let open_request = json!({
        "tool": "open_workbook",
        "args": {
            "path": "tests/fixtures/basic-sales.xlsx"
        }
    });
    let open_output = run_cli_with_json_and_runtime(&open_request.to_string(), &runtime_db);
    let file_ref = open_output["data"]["file_ref"].as_str().unwrap();

    let request = json!({
        "tool": "apply_header_schema",
        "args": {
            "session_id": "session_apply_header_schema_file_ref",
            "file_ref": file_ref,
            "sheet_index": 1
        }
    });

    let output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db);
    assert_eq!(output["status"], "ok");
    assert!(
        output["data"]["table_ref"]
            .as_str()
            .unwrap()
            .starts_with("table_")
    );

    let get_request = json!({
        "tool": "get_session_state",
        "args": {
            "session_id": "session_apply_header_schema_file_ref"
        }
    });
    let state_output = run_cli_with_json_and_runtime(&get_request.to_string(), &runtime_db);

    // 2026-03-23: 这里先锁定 file_ref + sheet_index 入口在固化 table_ref 后仍会把当前文件与当前 Sheet 正确同步回会话态，目的是保证后续编排不受新入口影响。
    assert_eq!(state_output["status"], "ok");
    assert_eq!(state_output["data"]["current_sheet"], "Sales");
    assert_eq!(
        state_output["data"]["current_workbook"],
        "tests/fixtures/basic-sales.xlsx"
    );
}

#[test]
fn tool_catalog_includes_fill_missing_values() {
    let mut cmd = Command::cargo_bin("excel_skill").unwrap();
    let assert = cmd.assert().success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    // 2026-03-22: 这里先锁定目录里暴露 fill_missing_values，目的是让上层 Skill 能显式发现通用补空入口。
    assert!(
        json["data"]["tool_catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool == "fill_missing_values")
    );
}

#[test]
fn fill_missing_values_returns_result_ref_with_filled_preview() {
    let workbook_path = create_test_workbook(
        "fill_missing_values_cli",
        "fill-missing-values.xlsx",
        &[(
            "Sales",
            vec![
                vec!["user_id", "city", "sales", "region"],
                vec!["1", "", "", "North"],
                vec!["2", "Urumqi", "15", ""],
                vec!["3", "", "", "West"],
                vec!["4", "", "", ""],
            ],
        )],
    );
    let request = json!({
        "tool": "fill_missing_values",
        "args": {
            "path": workbook_path.to_string_lossy(),
            "sheet": "Sales",
            "rules": [
                {
                    "column": "city",
                    "strategy": "constant",
                    "value": "Unknown"
                },
                {
                    "column": "sales",
                    "strategy": "zero"
                },
                {
                    "column": "region",
                    "strategy": "forward_fill"
                }
            ]
        }
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "ok");
    // 2026-03-22: 这里锁定 CLI 层补空结果与 result_ref 回传，目的是让通用补空 Tool 能直接进入后续链式分析。
    assert_eq!(output["data"]["rows"][0]["city"], "Unknown");
    assert_eq!(output["data"]["rows"][0]["sales"], "0");
    assert_eq!(output["data"]["rows"][1]["region"], "North");
    assert_eq!(output["data"]["rows"][3]["region"], "West");
    assert!(
        output["data"]["result_ref"]
            .as_str()
            .unwrap()
            .starts_with("result_")
    );
}

#[test]
fn tool_catalog_includes_distinct_rows() {
    let mut cmd = Command::cargo_bin("excel_skill").unwrap();
    let assert = cmd.assert().success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    // 2026-03-22: 这里先锁定目录里暴露 distinct_rows，目的是让上层能显式发现通用去重入口。
    assert!(
        json["data"]["tool_catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool == "distinct_rows")
    );
}

#[test]
fn distinct_rows_returns_result_ref_with_subset_deduplication() {
    let workbook_path = create_test_workbook(
        "distinct_rows_cli",
        "distinct-rows.xlsx",
        &[(
            "Sales",
            vec![
                vec!["user_id", "region", "sales"],
                vec!["1", "East", "100"],
                vec!["1", "East", "100"],
                vec!["2", "West", "90"],
                vec!["2", "West", "95"],
            ],
        )],
    );
    let request = json!({
        "tool": "distinct_rows",
        "args": {
            "path": workbook_path.to_string_lossy(),
            "sheet": "Sales",
            "subset": ["user_id", "region"],
            "keep": "last"
        }
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "ok");
    // 2026-03-22: 这里锁定 CLI 层按子集列去重并保留最后一条，目的是让 Excel 用户能稳定做主键级去重。
    assert_eq!(output["data"]["row_count"], 2);
    assert_eq!(output["data"]["rows"][0]["sales"], "100");
    assert_eq!(output["data"]["rows"][1]["sales"], "95");
    assert!(
        output["data"]["result_ref"]
            .as_str()
            .unwrap()
            .starts_with("result_")
    );
}

#[test]
fn tool_catalog_includes_deduplicate_by_key() {
    let mut cmd = Command::cargo_bin("excel_skill").unwrap();
    let assert = cmd.assert().success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    // 2026-03-22: 这里先锁定目录里暴露 deduplicate_by_key，目的是让上层能显式发现“按业务键去重”的独立入口。
    assert!(
        json["data"]["tool_catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool == "deduplicate_by_key")
    );
}

#[test]
fn deduplicate_by_key_returns_result_ref_with_kept_rows() {
    let workbook_path = create_test_workbook(
        "deduplicate_by_key_cli",
        "deduplicate-by-key.xlsx",
        &[(
            "Sales",
            vec![
                vec!["user_id", "updated_at", "sales"],
                vec!["1", "2026-03-01", "100"],
                vec!["1", "2026-03-03", "120"],
                vec!["2", "2026-03-02", "90"],
                vec!["2", "2026-03-04", "95"],
            ],
        )],
    );
    let request = json!({
        "tool": "deduplicate_by_key",
        "args": {
            "path": workbook_path.to_string_lossy(),
            "sheet": "Sales",
            "keys": ["user_id"],
            "order_by": [
                {
                    "column": "updated_at",
                    "direction": "asc"
                }
            ],
            "keep": "last"
        }
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "ok");
    // 2026-03-22: 这里锁定 CLI 层会先排序再按主键保留末条记录，目的是让 Excel 用户直接消费“保留最新记录”的去重结果。
    assert_eq!(output["data"]["row_count"], 2);
    assert_eq!(output["data"]["rows"][0]["updated_at"], "2026-03-03");
    assert_eq!(output["data"]["rows"][0]["sales"], "120");
    assert_eq!(output["data"]["rows"][1]["updated_at"], "2026-03-04");
    assert_eq!(output["data"]["rows"][1]["sales"], "95");
    assert!(
        output["data"]["result_ref"]
            .as_str()
            .unwrap()
            .starts_with("result_")
    );
}

#[test]
fn tool_catalog_includes_format_table_for_export() {
    let mut cmd = Command::cargo_bin("excel_skill").unwrap();
    let assert = cmd.assert().success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    // 2026-03-22: 这里先锁定目录里暴露 format_table_for_export，目的是让输出层桥接能力能被上层显式发现。
    assert!(
        json["data"]["tool_catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool == "format_table_for_export")
    );
}

#[test]
fn format_table_for_export_returns_result_ref_with_export_layout() {
    let workbook_path = create_test_workbook(
        "format_table_for_export_cli",
        "format-table-for-export.xlsx",
        &[(
            "Sales",
            vec![
                vec!["user_id", "region", "sales", "debug_tag"],
                vec!["1", "East", "120", "internal"],
                vec!["2", "West", "95", "internal"],
            ],
        )],
    );
    let request = json!({
        "tool": "format_table_for_export",
        "args": {
            "path": workbook_path.to_string_lossy(),
            "sheet": "Sales",
            "column_order": ["region", "sales", "user_id"],
            "rename_mappings": [
                {
                    "from": "region",
                    "to": "区域"
                },
                {
                    "from": "sales",
                    "to": "销售额"
                },
                {
                    "from": "user_id",
                    "to": "客户ID"
                }
            ],
            "drop_unspecified_columns": true
        }
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "ok");
    // 2026-03-22: 这里锁定 CLI 层导出整理结果，目的是让后续 compose_workbook 能直接消费整理好的交付表。
    assert_eq!(
        output["data"]["columns"],
        json!(["区域", "销售额", "客户ID"])
    );
    assert_eq!(output["data"]["row_count"], 2);
    assert_eq!(output["data"]["rows"][0]["区域"], "East");
    assert_eq!(output["data"]["rows"][1]["销售额"], "95");
    assert!(
        output["data"]["result_ref"]
            .as_str()
            .unwrap()
            .starts_with("result_")
    );
}

#[test]
fn cli_open_workbook_accepts_chinese_windows_path() {
    let fixture_path = create_chinese_path_fixture("\u{57fa}\u{7840}\u{9500}\u{552e}-cli.xlsx");
    let request = json!({
        "tool": "open_workbook",
        "args": {
            "path": fixture_path.to_string_lossy()
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    // 2026-03-22: ???? CLI ??????????????????????????????
    assert_eq!(output["data"]["sheet_names"], json!(["Sales", "Lookup"]));
    assert!(
        output["data"]["path"]
            .as_str()
            .unwrap()
            .contains("\u{4e2d}\u{6587}\u{8def}\u{5f84}")
    );
}

#[test]
fn cli_open_workbook_accepts_gbk_encoded_json_with_chinese_path() {
    let fixture_path = create_chinese_path_fixture("\u{57fa}\u{7840}\u{9500}\u{552e}-gbk.xlsx");
    let request = json!({
        "tool": "open_workbook",
        "args": {
            "path": fixture_path.to_string_lossy()
        }
    });

    let request_json = request.to_string();
    let (encoded, _, _) = encoding_rs::GBK.encode(&request_json);
    let output = run_cli_with_bytes(encoded.into_owned());
    assert_eq!(output["status"], "ok");
    // 2026-03-22: ???? Windows ?? GBK ??????? CLI ???????????????? IT ?????????
    assert_eq!(output["data"]["sheet_names"], json!(["Sales", "Lookup"]));
}

#[test]
fn normalize_table_returns_confirmation_payload_for_ambiguous_headers() {
    let request = json!({
        "tool": "normalize_table",
        "args": {
            "path": "tests/fixtures/title-gap-header.xlsx",
            "sheet": "Sheet1"
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "needs_confirmation");
    assert_eq!(output["data"]["confidence"], "medium");
    assert_eq!(output["data"]["columns"][0]["canonical_name"], "user_id");
}

#[test]
fn normalize_table_marks_non_ascii_headers_for_confirmation_before_dataframe_loading() {
    let request = json!({
        "tool": "normalize_table",
        "args": {
            "path": "tests/fixtures/header-non-ascii.xlsx",
            "sheet": "Sheet1"
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "needs_confirmation");
    // 2026-03-22: 这里锁定纯中文表头会先进入确认态，目的是避免空列名误判为可直接执行。
    assert_eq!(output["data"]["confidence"], "medium");
    assert_eq!(output["data"]["schema_state"], "pending");
    assert_eq!(output["data"]["columns"][0]["canonical_name"], "column_1");
    assert_eq!(output["data"]["columns"][1]["canonical_name"], "column_2");
}

#[test]
fn preview_table_stops_at_confirmation_for_non_ascii_headers() {
    let request = json!({
        "tool": "preview_table",
        "args": {
            "path": "tests/fixtures/header-non-ascii.xlsx",
            "sheet": "Sheet1",
            "limit": 2
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "needs_confirmation");
    // 2026-03-22: 这里锁定预览入口会在确认层拦住风险表头，目的是不再走到 Polars 重复空列名报错。
    assert_eq!(output["data"]["schema_state"], "pending");
    assert_eq!(output["data"]["columns"][2]["canonical_name"], "column_3");
}

#[test]
fn apply_header_schema_returns_confirmed_table_reference() {
    let request = json!({
        "tool": "apply_header_schema",
        "args": {
            "path": "tests/fixtures/multi-header-sales.xlsx",
            "sheet": "Report"
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["schema_state"], "confirmed");
    assert_eq!(
        output["data"]["columns"][0]["canonical_name"],
        "region_east_sales"
    );
    assert_eq!(output["data"]["row_count"], 2);
    assert!(
        output["data"]["table_id"]
            .as_str()
            .unwrap()
            .starts_with("table_")
    );
}

#[test]
fn apply_header_schema_returns_reusable_table_ref() {
    let request = json!({
        "tool": "apply_header_schema",
        "args": {
            "path": "tests/fixtures/title-gap-header.xlsx",
            "sheet": "Sheet1"
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    // 2026-03-22: 这里锁定确认后的返回里必须带可跨请求复用的 table_ref，目的是把表处理层确认态升级成分析建模层可直接复用的稳定句柄。
    assert!(
        output["data"]["table_ref"]
            .as_str()
            .unwrap()
            .starts_with("table_")
    );
    assert_eq!(output["data"]["schema_state"], "confirmed");
}

#[test]
fn get_session_state_returns_persisted_update_from_previous_request() {
    let runtime_db = create_test_runtime_db("session_round_trip");
    let update_request = json!({
        "tool": "update_session_state",
        "args": {
            "session_id": "session_cli_round_trip",
            "current_workbook": "tests/fixtures/title-gap-header.xlsx",
            "current_sheet": "Sheet1",
            "current_stage": "table_processing",
            "schema_status": "pending_confirmation",
            "last_user_goal": "先看看这个 Excel",
            "selected_columns": ["user_id", "sales"]
        }
    });

    let update_output = run_cli_with_json_and_runtime(&update_request.to_string(), &runtime_db);
    assert_eq!(update_output["status"], "ok");

    let get_request = json!({
        "tool": "get_session_state",
        "args": {
            "session_id": "session_cli_round_trip"
        }
    });

    let output = run_cli_with_json_and_runtime(&get_request.to_string(), &runtime_db);

    // 2026-03-22: 这里先锁定跨 CLI 请求的会话状态持久化，目的是验证 orchestrator 读取到的是上一轮已落盘的摘要，而不是同轮临时变量。
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["current_workbook"],
        "tests/fixtures/title-gap-header.xlsx"
    );
    assert_eq!(output["data"]["current_sheet"], "Sheet1");
    assert_eq!(output["data"]["current_stage"], "table_processing");
    assert_eq!(output["data"]["schema_status"], "pending_confirmation");
    assert_eq!(output["data"]["last_user_goal"], "先看看这个 Excel");
    assert_eq!(
        output["data"]["selected_columns"],
        json!(["user_id", "sales"])
    );
}

#[test]
fn get_session_state_exposes_active_handle_summary() {
    let runtime_db = create_test_runtime_db("session_active_handle_summary");
    let update_request = json!({
        "tool": "update_session_state",
        "args": {
            "session_id": "session_active_handle_summary",
            "active_handle_ref": "result_123456",
            "current_stage": "analysis_modeling"
        }
    });

    let update_output = run_cli_with_json_and_runtime(&update_request.to_string(), &runtime_db);
    assert_eq!(update_output["status"], "ok");

    let get_request = json!({
        "tool": "get_session_state",
        "args": {
            "session_id": "session_active_handle_summary"
        }
    });
    let output = run_cli_with_json_and_runtime(&get_request.to_string(), &runtime_db);

    // 2026-03-22: 这里锁定兼容式 active_handle 摘要，目的是在保留 active_table_ref 旧字段的同时，对上层 Skill 暴露更清晰的激活句柄语义。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["active_table_ref"], "result_123456");
    assert_eq!(output["data"]["active_handle"]["ref"], "result_123456");
    assert_eq!(output["data"]["active_handle"]["kind"], "result_ref");
    assert_eq!(output["data"]["active_handle_ref"], "result_123456");
}

#[test]
fn get_session_state_keeps_table_ref_and_active_handle_separately() {
    let runtime_db = create_test_runtime_db("session_active_handle_split");
    let update_request = json!({
        "tool": "update_session_state",
        "args": {
            "session_id": "session_active_handle_split",
            "active_table_ref": "table_confirmed_001",
            "active_handle_ref": "result_latest_001",
            "active_handle_kind": "result_ref",
            "current_stage": "analysis_modeling"
        }
    });

    let update_output = run_cli_with_json_and_runtime(&update_request.to_string(), &runtime_db);
    assert_eq!(update_output["status"], "ok");

    let get_request = json!({
        "tool": "get_session_state",
        "args": {
            "session_id": "session_active_handle_split"
        }
    });
    let output = run_cli_with_json_and_runtime(&get_request.to_string(), &runtime_db);

    // 2026-03-23: 这里锁定 table_ref 与最新激活句柄分离展示，原因是方案B要求保留稳定回源表句柄，同时暴露当前最新 result_ref；目的是让上层 Skill 不再把两种语义混成一个字段。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["active_table_ref"], "table_confirmed_001");
    assert_eq!(output["data"]["active_handle_ref"], "result_latest_001");
    assert_eq!(output["data"]["active_handle"]["ref"], "result_latest_001");
    assert_eq!(output["data"]["active_handle"]["kind"], "result_ref");
}

#[test]
fn apply_header_schema_updates_session_state_and_active_table_ref() {
    let runtime_db = create_test_runtime_db("apply_header_schema_state");
    let confirm_request = json!({
        "tool": "apply_header_schema",
        "args": {
            "session_id": "session_apply_header_schema",
            "path": "tests/fixtures/title-gap-header.xlsx",
            "sheet": "Sheet1"
        }
    });

    let confirm_output = run_cli_with_json_and_runtime(&confirm_request.to_string(), &runtime_db);
    assert_eq!(confirm_output["status"], "ok");
    let table_ref = confirm_output["data"]["table_ref"]
        .as_str()
        .unwrap()
        .to_string();

    let get_request = json!({
        "tool": "get_session_state",
        "args": {
            "session_id": "session_apply_header_schema"
        }
    });
    let output = run_cli_with_json_and_runtime(&get_request.to_string(), &runtime_db);

    // 2026-03-22: 这里先锁定确认态建立后会自动激活 active_table_ref，目的是让用户后续切分析或决策时不需要再次手填 path + sheet。
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["current_workbook"],
        "tests/fixtures/title-gap-header.xlsx"
    );
    assert_eq!(output["data"]["current_sheet"], "Sheet1");
    assert_eq!(output["data"]["current_stage"], "table_processing");
    assert_eq!(output["data"]["schema_status"], "confirmed");
    assert_eq!(output["data"]["active_table_ref"], table_ref);
}

#[test]
fn stat_summary_with_table_ref_advances_session_stage_to_analysis_modeling() {
    let runtime_db = create_test_runtime_db("stat_summary_stage");
    let confirm_request = json!({
        "tool": "apply_header_schema",
        "args": {
            "session_id": "session_stat_summary_stage",
            "path": "tests/fixtures/title-gap-header.xlsx",
            "sheet": "Sheet1"
        }
    });
    let confirm_output = run_cli_with_json_and_runtime(&confirm_request.to_string(), &runtime_db);
    let table_ref = confirm_output["data"]["table_ref"].as_str().unwrap();

    let stat_request = json!({
        "tool": "stat_summary",
        "args": {
            "session_id": "session_stat_summary_stage",
            "table_ref": table_ref,
            "columns": ["sales"],
            "casts": [
                {
                    "column": "sales",
                    "target_type": "int64"
                }
            ]
        }
    });
    let stat_output = run_cli_with_json_and_runtime(&stat_request.to_string(), &runtime_db);
    assert_eq!(stat_output["status"], "ok");

    let get_request = json!({
        "tool": "get_session_state",
        "args": {
            "session_id": "session_stat_summary_stage"
        }
    });
    let output = run_cli_with_json_and_runtime(&get_request.to_string(), &runtime_db);

    // 2026-03-22: 这里先锁定分析入口消费 table_ref 后会把阶段推进到 analysis_modeling，目的是让总入口能识别用户已经进入分析建模层。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["current_stage"], "analysis_modeling");
    assert_eq!(output["data"]["schema_status"], "confirmed");
    assert_eq!(output["data"]["active_table_ref"], table_ref);
}

#[test]
fn group_and_aggregate_updates_session_state_to_latest_result_ref() {
    let runtime_db = create_test_runtime_db("group_state_latest_result");
    let confirm_request = json!({
        "tool": "apply_header_schema",
        "args": {
            "session_id": "session_group_state_latest_result",
            "path": "tests/fixtures/basic-sales.xlsx",
            "sheet": "Sales"
        }
    });
    let confirm_output = run_cli_with_json_and_runtime(&confirm_request.to_string(), &runtime_db);
    assert_eq!(confirm_output["status"], "ok");
    let table_ref = confirm_output["data"]["table_ref"]
        .as_str()
        .unwrap()
        .to_string();

    let group_request = json!({
        "tool": "group_and_aggregate",
        "args": {
            "session_id": "session_group_state_latest_result",
            "table_ref": table_ref,
            "group_by": ["region"],
            "aggregations": [
                {
                    "column": "sales",
                    "operator": "sum"
                }
            ],
            "casts": [
                {
                    "column": "sales",
                    "target_type": "int64"
                }
            ]
        }
    });
    let group_output = run_cli_with_json_and_runtime(&group_request.to_string(), &runtime_db);
    assert_eq!(group_output["status"], "ok");
    let latest_result_ref = group_output["data"]["result_ref"]
        .as_str()
        .unwrap()
        .to_string();

    let get_request = json!({
        "tool": "get_session_state",
        "args": {
            "session_id": "session_group_state_latest_result"
        }
    });
    let output = run_cli_with_json_and_runtime(&get_request.to_string(), &runtime_db);

    // 2026-03-23: 这里锁定聚合后会话状态会切到最新 result_ref，原因是方案B要求产出型 Tool 自动推进当前激活对象；目的是让上层 Skill 下一步直接接到最新中间结果，而不是继续停在旧 table_ref。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["active_table_ref"], table_ref);
    assert_eq!(output["data"]["active_handle_ref"], latest_result_ref);
    assert_eq!(output["data"]["active_handle"]["kind"], "result_ref");
}

#[test]
fn append_tables_updates_session_state_to_latest_result_ref() {
    let runtime_db = create_test_runtime_db("append_state_latest_result");
    let confirm_request = json!({
        "tool": "apply_header_schema",
        "args": {
            "session_id": "session_append_state_latest_result",
            "path": "tests/fixtures/basic-sales.xlsx",
            "sheet": "Sales"
        }
    });
    let confirm_output = run_cli_with_json_and_runtime(&confirm_request.to_string(), &runtime_db);
    assert_eq!(confirm_output["status"], "ok");
    let table_ref = confirm_output["data"]["table_ref"]
        .as_str()
        .unwrap()
        .to_string();

    let append_request = json!({
        "tool": "append_tables",
        "args": {
            "session_id": "session_append_state_latest_result",
            "top": {
                "table_ref": table_ref
            },
            "bottom": {
                "path": "tests/fixtures/append-sales-b.xlsx",
                "sheet": "Sales"
            }
        }
    });
    let append_output = run_cli_with_json_and_runtime(&append_request.to_string(), &runtime_db);
    assert_eq!(append_output["status"], "ok");
    let latest_result_ref = append_output["data"]["result_ref"]
        .as_str()
        .unwrap()
        .to_string();

    let get_request = json!({
        "tool": "get_session_state",
        "args": {
            "session_id": "session_append_state_latest_result"
        }
    });
    let output = run_cli_with_json_and_runtime(&get_request.to_string(), &runtime_db);

    // 2026-03-23: 这里锁定追加后会话状态会切到最新 result_ref，原因是多表链式执行不能在成功后还停留在输入句柄；目的是让“先追加再关联”能真正按最新结果往下走。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["active_table_ref"], table_ref);
    assert_eq!(output["data"]["active_handle_ref"], latest_result_ref);
    assert_eq!(output["data"]["active_handle"]["kind"], "result_ref");
}

#[test]
fn join_tables_updates_session_state_to_latest_result_ref() {
    let runtime_db = create_test_runtime_db("join_state_latest_result");
    let confirm_request = json!({
        "tool": "apply_header_schema",
        "args": {
            "session_id": "session_join_state_latest_result",
            "path": "tests/fixtures/join-customers.xlsx",
            "sheet": "Customers"
        }
    });
    let confirm_output = run_cli_with_json_and_runtime(&confirm_request.to_string(), &runtime_db);
    assert_eq!(confirm_output["status"], "ok");
    let table_ref = confirm_output["data"]["table_ref"]
        .as_str()
        .unwrap()
        .to_string();

    let join_request = json!({
        "tool": "join_tables",
        "args": {
            "session_id": "session_join_state_latest_result",
            "left": {
                "table_ref": table_ref
            },
            "right": {
                "path": "tests/fixtures/join-orders.xlsx",
                "sheet": "Orders"
            },
            "left_on": "user_id",
            "right_on": "user_id",
            "keep_mode": "matched_only"
        }
    });
    let join_output = run_cli_with_json_and_runtime(&join_request.to_string(), &runtime_db);
    assert_eq!(join_output["status"], "ok");
    let latest_result_ref = join_output["data"]["result_ref"]
        .as_str()
        .unwrap()
        .to_string();

    let get_request = json!({
        "tool": "get_session_state",
        "args": {
            "session_id": "session_join_state_latest_result"
        }
    });
    let output = run_cli_with_json_and_runtime(&get_request.to_string(), &runtime_db);

    // 2026-03-23: 这里锁定关联后会话状态会切到最新 result_ref，原因是显性关联是多步闭环里的关键中间结果；目的是让后续分析直接消费 join 结果而不是回退到左表。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["active_table_ref"], table_ref);
    assert_eq!(output["data"]["active_handle_ref"], latest_result_ref);
    assert_eq!(output["data"]["active_handle"]["kind"], "result_ref");
}

#[test]
fn decision_assistant_with_table_ref_advances_session_stage_to_decision_assistant() {
    let runtime_db = create_test_runtime_db("decision_stage");
    let confirm_request = json!({
        "tool": "apply_header_schema",
        "args": {
            "session_id": "session_decision_stage",
            "path": "tests/fixtures/title-gap-header.xlsx",
            "sheet": "Sheet1"
        }
    });
    let confirm_output = run_cli_with_json_and_runtime(&confirm_request.to_string(), &runtime_db);
    let table_ref = confirm_output["data"]["table_ref"].as_str().unwrap();

    let decision_request = json!({
        "tool": "decision_assistant",
        "args": {
            "session_id": "session_decision_stage",
            "table_ref": table_ref,
            "columns": ["user_id", "sales"],
            "casts": [
                {
                    "column": "user_id",
                    "target_type": "int64"
                },
                {
                    "column": "sales",
                    "target_type": "int64"
                }
            ]
        }
    });
    let decision_output = run_cli_with_json_and_runtime(&decision_request.to_string(), &runtime_db);
    assert_eq!(decision_output["status"], "ok");

    let get_request = json!({
        "tool": "get_session_state",
        "args": {
            "session_id": "session_decision_stage"
        }
    });
    let output = run_cli_with_json_and_runtime(&get_request.to_string(), &runtime_db);

    // 2026-03-22: 这里先锁定决策助手入口会推进阶段，目的是让 orchestrator 下一轮能知道当前更接近建议与决策语境，而不是重新回到表处理。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["current_stage"], "decision_assistant");
    assert_eq!(output["data"]["schema_status"], "confirmed");
    assert_eq!(output["data"]["active_table_ref"], table_ref);
}

#[test]
fn stat_summary_preserves_input_result_ref_and_stable_table_ref() {
    let runtime_db = create_test_runtime_db("stat_summary_preserve_input_handle");
    let confirm_request = json!({
        "tool": "apply_header_schema",
        "args": {
            "session_id": "session_stat_summary_preserve_input_handle",
            "path": "tests/fixtures/basic-sales.xlsx",
            "sheet": "Sales"
        }
    });
    let confirm_output = run_cli_with_json_and_runtime(&confirm_request.to_string(), &runtime_db);
    assert_eq!(confirm_output["status"], "ok");
    let table_ref = confirm_output["data"]["table_ref"]
        .as_str()
        .unwrap()
        .to_string();

    let group_request = json!({
        "tool": "group_and_aggregate",
        "args": {
            "session_id": "session_stat_summary_preserve_input_handle",
            "table_ref": table_ref,
            "group_by": ["region"],
            "aggregations": [
                {
                    "column": "sales",
                    "operator": "sum"
                }
            ],
            "casts": [
                {
                    "column": "sales",
                    "target_type": "int64"
                }
            ]
        }
    });
    let group_output = run_cli_with_json_and_runtime(&group_request.to_string(), &runtime_db);
    assert_eq!(group_output["status"], "ok");
    let latest_result_ref = group_output["data"]["result_ref"]
        .as_str()
        .unwrap()
        .to_string();

    let stat_request = json!({
        "tool": "stat_summary",
        "args": {
            "session_id": "session_stat_summary_preserve_input_handle",
            "result_ref": latest_result_ref,
            "casts": [
                {
                    "column": "sales_sum",
                    "target_type": "int64"
                }
            ]
        }
    });
    let stat_output = run_cli_with_json_and_runtime(&stat_request.to_string(), &runtime_db);
    assert_eq!(stat_output["status"], "ok");

    let get_request = json!({
        "tool": "get_session_state",
        "args": {
            "session_id": "session_stat_summary_preserve_input_handle"
        }
    });
    let output = run_cli_with_json_and_runtime(&get_request.to_string(), &runtime_db);

    // 2026-03-23: 这里锁定纯读取类分析 Tool 不会覆盖稳定 table_ref，原因是方案B要求“当前输入句柄”和“稳定回源 table_ref”并存；目的是让后续继续分析时既知道当前读的是哪个 result_ref，也不会丢掉确认态表来源。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["active_table_ref"], table_ref);
    assert_eq!(output["data"]["active_handle_ref"], latest_result_ref);
    assert_eq!(output["data"]["active_handle"]["kind"], "result_ref");
}

#[test]
fn stat_summary_accepts_result_ref_from_previous_step() {
    let store = thread_result_ref_store();
    let result_ref = ResultRefStore::create_result_ref();
    // 2026-03-22: 这里先构造一个持久化中间结果，目的是锁定分析 Tool 可以直接复用上一步结果，而不必回退到 path+sheet。
    let dataframe = DataFrame::new(vec![
        Series::new("customer_id".into(), ["c001", "c002", "c003"]).into(),
        Series::new("sales".into(), [100_i64, 200_i64, 300_i64]).into(),
    ])
    .unwrap();
    let record = PersistedResultDataset::from_dataframe(
        &result_ref,
        "group_and_aggregate",
        vec!["table_sales".to_string()],
        &dataframe,
    )
    .unwrap();
    store.save(&record).unwrap();

    let request = json!({
        "tool": "stat_summary",
        "args": {
            "result_ref": result_ref,
            "columns": ["sales"]
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["numeric_summaries"][0]["column"], "sales");
    assert_eq!(output["data"]["numeric_summaries"][0]["count"], 3);
}

#[test]
fn stat_summary_accepts_table_ref_from_apply_header_schema() {
    let confirm_request = json!({
        "tool": "apply_header_schema",
        "args": {
            "path": "tests/fixtures/title-gap-header.xlsx",
            "sheet": "Sheet1"
        }
    });
    let confirm_output = run_cli_with_json(&confirm_request.to_string());
    let table_ref = confirm_output["data"]["table_ref"].as_str().unwrap();

    let request = json!({
        "tool": "stat_summary",
        "args": {
            "table_ref": table_ref,
            "columns": ["sales"],
            "casts": [
                {
                    "column": "sales",
                    "target_type": "int64"
                }
            ],
            "top_k": 5
        }
    });

    let output = run_cli_with_json(&request.to_string());
    // 2026-03-22: 这里锁定 analysis bridge 会优先复用 table_ref，而不是重新回到 needs_confirmation，目的是把表处理确认态真正接进分析摘要入口。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["row_count"], 2);
    assert_eq!(output["data"]["numeric_summaries"][0]["column"], "sales");
}

#[test]
fn stat_summary_accepts_region_table_ref_from_load_table_region() {
    let workbook_path = create_positioned_workbook(
        "region_table_ref_stat_summary",
        "region-table-ref-stat-summary.xlsx",
        &[(
            "Report",
            vec![
                (2, 1, "user_id"),
                (2, 2, "region"),
                (2, 3, "sales"),
                (3, 1, "1001"),
                (3, 2, "North"),
                (3, 3, "88"),
                (4, 1, "1002"),
                (4, 2, "South"),
                (4, 3, "95"),
            ],
        )],
    );
    let load_request = json!({
        "tool": "load_table_region",
        "args": {
            "path": workbook_path.to_string_lossy(),
            "sheet": "Report",
            "range": "B3:D5",
            "header_row_count": 1
        }
    });
    let load_output = run_cli_with_json(&load_request.to_string());
    assert_eq!(load_output["status"], "ok");
    let table_ref = load_output["data"]["table_ref"].as_str().unwrap();

    let request = json!({
        "tool": "stat_summary",
        "args": {
            "table_ref": table_ref,
            "columns": ["sales"],
            "casts": [
                {
                    "column": "sales",
                    "target_type": "int64"
                }
            ],
            "top_k": 5
        }
    });
    let output = run_cli_with_json(&request.to_string());

    // 2026-03-22: 这里锁定 region table_ref 可以直接进入统计摘要，目的是把显式区域确认态真正桥接到分析层。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["row_count"], 2);
    assert_eq!(output["data"]["numeric_summaries"][0]["column"], "sales");
    assert_eq!(output["data"]["numeric_summaries"][0]["count"], 2);
}

#[test]
fn analyze_table_accepts_region_table_ref_from_load_table_region() {
    let workbook_path = create_positioned_workbook(
        "region_table_ref_analyze",
        "region-table-ref-analyze.xlsx",
        &[(
            "Report",
            vec![
                (2, 1, "user_id"),
                (2, 2, "region"),
                (2, 3, "sales"),
                (3, 1, "1001"),
                (3, 2, "North"),
                (3, 3, "88"),
                (4, 1, "1002"),
                (4, 2, "South"),
                (4, 3, "95"),
            ],
        )],
    );
    let load_request = json!({
        "tool": "load_table_region",
        "args": {
            "path": workbook_path.to_string_lossy(),
            "sheet": "Report",
            "range": "B3:D5",
            "header_row_count": 1
        }
    });
    let load_output = run_cli_with_json(&load_request.to_string());
    assert_eq!(load_output["status"], "ok");
    let table_ref = load_output["data"]["table_ref"].as_str().unwrap();

    let request = json!({
        "tool": "analyze_table",
        "args": {
            "table_ref": table_ref,
            "casts": [
                {
                    "column": "sales",
                    "target_type": "int64"
                }
            ],
            "top_k": 3
        }
    });
    let output = run_cli_with_json(&request.to_string());

    // 2026-03-22: 这里锁定 region table_ref 也能直接进入分析诊断，目的是让局部区域分析不必退回原始 path+sheet。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["row_count"], 2);
    assert!(output["data"]["table_health"].is_object());
    assert!(output["data"]["human_summary"].is_object());
}

#[test]
fn cluster_kmeans_accepts_table_ref_from_apply_header_schema() {
    let confirm_request = json!({
        "tool": "apply_header_schema",
        "args": {
            "path": "tests/fixtures/title-gap-header.xlsx",
            "sheet": "Sheet1"
        }
    });
    let confirm_output = run_cli_with_json(&confirm_request.to_string());
    let table_ref = confirm_output["data"]["table_ref"].as_str().unwrap();

    let request = json!({
        "tool": "cluster_kmeans",
        "args": {
            "table_ref": table_ref,
            "features": ["user_id", "sales"],
            "casts": [
                {
                    "column": "user_id",
                    "target_type": "int64"
                },
                {
                    "column": "sales",
                    "target_type": "int64"
                }
            ],
            "cluster_count": 2,
            "missing_strategy": "drop_rows"
        }
    });

    let output = run_cli_with_json(&request.to_string());
    // 2026-03-22: 这里锁定建模 Tool 也能直接消费 table_ref，目的是确保桥接不是只修统计摘要而是打通到建模公共准备层。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["model_kind"], "cluster_kmeans");
    assert_eq!(output["data"]["features"], json!(["user_id", "sales"]));
    assert_eq!(output["data"]["cluster_count"], 2);
}

#[test]
fn decision_assistant_accepts_table_ref_from_apply_header_schema() {
    let confirm_request = json!({
        "tool": "apply_header_schema",
        "args": {
            "path": "tests/fixtures/title-gap-header.xlsx",
            "sheet": "Sheet1"
        }
    });
    let confirm_output = run_cli_with_json(&confirm_request.to_string());
    let table_ref = confirm_output["data"]["table_ref"].as_str().unwrap();

    let request = json!({
        "tool": "decision_assistant",
        "args": {
            "table_ref": table_ref,
            "columns": ["user_id", "sales"],
            "casts": [
                {
                    "column": "user_id",
                    "target_type": "int64"
                },
                {
                    "column": "sales",
                    "target_type": "int64"
                }
            ],
            "top_k": 5
        }
    });

    let output = run_cli_with_json(&request.to_string());
    // 2026-03-22: 这里锁定决策助手也能复用 table_ref，目的是把表处理确认态继续桥接到更上层的决策助手入口。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["assistant_kind"], "quality_diagnostic");
    assert!(output["data"]["human_summary"].is_object());
}

#[test]
fn stat_summary_rejects_stale_table_ref() {
    let runtime_dir = std::path::PathBuf::from("tests").join("runtime_fixtures");
    std::fs::create_dir_all(&runtime_dir).unwrap();
    let copied_path = runtime_dir.join("title-gap-header-stale.xlsx");
    std::fs::copy("tests/fixtures/title-gap-header.xlsx", &copied_path).unwrap();

    let confirm_request = json!({
        "tool": "apply_header_schema",
        "args": {
            "path": copied_path.to_string_lossy(),
            "sheet": "Sheet1"
        }
    });
    let confirm_output = run_cli_with_json(&confirm_request.to_string());
    let table_ref = confirm_output["data"]["table_ref"]
        .as_str()
        .unwrap()
        .to_string();

    std::fs::copy("tests/fixtures/basic-sales.xlsx", &copied_path).unwrap();

    let request = json!({
        "tool": "stat_summary",
        "args": {
            "table_ref": table_ref,
            "columns": ["sales"],
            "top_k": 5
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "error");
    // 2026-03-22: 这里锁定源文件变更后 table_ref 会被拒绝复用，目的是防止用户改过 Excel 以后系统还偷偷吃旧确认态。
    assert!(output["error"].as_str().unwrap().contains("table_ref"));
}

#[test]
fn stat_summary_rejects_stale_region_table_ref() {
    let workbook_path = create_positioned_workbook(
        "stale_region_table_ref",
        "stale-region-table-ref.xlsx",
        &[(
            "Report",
            vec![
                (2, 1, "user_id"),
                (2, 2, "region"),
                (2, 3, "sales"),
                (3, 1, "1001"),
                (3, 2, "North"),
                (3, 3, "88"),
                (4, 1, "1002"),
                (4, 2, "South"),
                (4, 3, "95"),
            ],
        )],
    );
    let load_request = json!({
        "tool": "load_table_region",
        "args": {
            "path": workbook_path.to_string_lossy(),
            "sheet": "Report",
            "range": "B3:D5",
            "header_row_count": 1
        }
    });
    let load_output = run_cli_with_json(&load_request.to_string());
    assert_eq!(load_output["status"], "ok");
    let table_ref = load_output["data"]["table_ref"]
        .as_str()
        .unwrap()
        .to_string();

    std::fs::copy("tests/fixtures/basic-sales.xlsx", &workbook_path).unwrap();

    let request = json!({
        "tool": "stat_summary",
        "args": {
            "table_ref": table_ref,
            "columns": ["sales"],
            "top_k": 5
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "error");
    // 2026-03-22: 这里锁定 region table_ref 在源文件变化后同样会被拒绝，目的是防止局部区域确认态吃到过期底表。
    assert!(output["error"].as_str().unwrap().contains("table_ref"));
}

#[test]
fn preview_table_returns_first_rows_from_target_sheet() {
    let request = json!({
        "tool": "preview_table",
        "args": {
            "path": "tests/fixtures/basic-sales.xlsx",
            "sheet": "Sales",
            "limit": 1
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["rows"][0]["region"], "East");
    assert_eq!(
        output["data"]["columns"],
        json!(["user_id", "region", "sales"])
    );
}

#[test]
fn select_columns_returns_trimmed_columns_for_target_sheet() {
    let request = json!({
        "tool": "select_columns",
        "args": {
            "path": "tests/fixtures/basic-sales.xlsx",
            "sheet": "Sales",
            "columns": ["region", "sales"]
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["columns"], json!(["region", "sales"]));
    assert_eq!(output["data"]["row_count"], 2);
}

#[test]
fn cast_column_types_returns_updated_dtype_summary() {
    let request = json!({
        "tool": "cast_column_types",
        "args": {
            "path": "tests/fixtures/basic-sales.xlsx",
            "sheet": "Sales",
            "casts": [
                {
                    "column": "sales",
                    "target_type": "int64"
                }
            ]
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["row_count"], 2);
    assert_eq!(output["data"]["column_types"][2]["column"], "sales");
    assert_eq!(output["data"]["column_types"][2]["dtype"], "int64");
}

#[test]
fn group_and_aggregate_returns_grouped_rows_for_target_sheet() {
    let request = json!({
        "tool": "group_and_aggregate",
        "args": {
            "path": "tests/fixtures/group-sales.xlsx",
            "sheet": "Sales",
            "casts": [
                {
                    "column": "sales",
                    "target_type": "int64"
                }
            ],
            "group_by": ["region"],
            "aggregations": [
                {
                    "column": "sales",
                    "operator": "sum"
                },
                {
                    "column": "sales",
                    "operator": "count"
                }
            ]
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["row_count"], 2);
    assert_eq!(
        output["data"]["columns"],
        json!(["region", "sales_sum", "sales_count"])
    );
    assert_eq!(output["data"]["rows"][0]["region"], "East");
    assert_eq!(output["data"]["rows"][0]["sales_sum"], "200");
    assert_eq!(output["data"]["rows"][1]["region"], "West");
    assert_eq!(output["data"]["rows"][1]["sales_count"], "2");
}

#[test]
fn group_and_aggregate_returns_reusable_result_ref_for_follow_up_analysis() {
    let group_request = json!({
        "tool": "group_and_aggregate",
        "args": {
            "path": "tests/fixtures/group-sales.xlsx",
            "sheet": "Sales",
            "casts": [
                {
                    "column": "sales",
                    "target_type": "int64"
                }
            ],
            "group_by": ["region"],
            "aggregations": [
                {
                    "column": "sales",
                    "operator": "sum"
                }
            ]
        }
    });

    let group_output = run_cli_with_json(&group_request.to_string());
    assert_eq!(group_output["status"], "ok");
    let result_ref = group_output["data"]["result_ref"]
        .as_str()
        .expect("group_and_aggregate should return result_ref")
        .to_string();

    let stat_request = json!({
        "tool": "stat_summary",
        "args": {
            "result_ref": result_ref,
            "casts": [
                {
                    "column": "sales_sum",
                    "target_type": "int64"
                }
            ],
            "columns": ["sales_sum"]
        }
    });

    let stat_output = run_cli_with_json(&stat_request.to_string());
    // 2026-03-22: 这里锁定表处理结果可以直接交给分析 Tool 继续消费，目的是把“先处理，再分析”的链式闭环真正落到用户可调用路径。
    assert_eq!(stat_output["status"], "ok");
    assert_eq!(
        stat_output["data"]["numeric_summaries"][0]["column"],
        "sales_sum"
    );
    assert_eq!(stat_output["data"]["numeric_summaries"][0]["count"], 2);
}

#[test]
fn sort_rows_returns_rows_in_requested_order() {
    let request = json!({
        "tool": "sort_rows",
        "args": {
            "path": "tests/fixtures/group-sales.xlsx",
            "sheet": "Sales",
            "casts": [
                {
                    "column": "sales",
                    "target_type": "int64"
                }
            ],
            "sorts": [
                {
                    "column": "region",
                    "descending": false
                },
                {
                    "column": "sales",
                    "descending": true
                }
            ],
            "limit": 4
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["row_count"], 4);
    assert_eq!(output["data"]["rows"][0]["region"], "East");
    assert_eq!(output["data"]["rows"][0]["sales"], "120");
    assert_eq!(output["data"]["rows"][1]["sales"], "80");
    assert_eq!(output["data"]["rows"][2]["region"], "West");
    assert_eq!(output["data"]["rows"][2]["sales"], "90");
}

#[test]
fn top_n_returns_highest_sales_rows_for_target_sheet() {
    let request = json!({
        "tool": "top_n",
        "args": {
            "path": "tests/fixtures/group-sales.xlsx",
            "sheet": "Sales",
            "casts": [
                {
                    "column": "sales",
                    "target_type": "int64"
                }
            ],
            "sorts": [
                {
                    "column": "sales",
                    "descending": true
                }
            ],
            "n": 2
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["row_count"], 2);
    assert_eq!(output["data"]["rows"][0]["sales"], "120");
    assert_eq!(output["data"]["rows"][1]["sales"], "90");
}

#[test]
fn derive_columns_builds_labels_buckets_and_scores() {
    let request = json!({
        "tool": "derive_columns",
        "args": {
            "path": "tests/fixtures/group-sales.xlsx",
            "sheet": "Sales",
            "casts": [
                {
                    "column": "sales",
                    "target_type": "int64"
                }
            ],
            "derivations": [
                {
                    "kind": "case_when",
                    "output_column": "priority",
                    "rules": [
                        {
                            "when": {
                                "column": "sales",
                                "operator": "gte",
                                "value": "110"
                            },
                            "value": "A"
                        }
                    ],
                    "else_value": "B"
                },
                {
                    "kind": "bucketize",
                    "source_column": "sales",
                    "output_column": "sales_band",
                    "buckets": [
                        {
                            "label": "low",
                            "max_exclusive": 110.0
                        },
                        {
                            "label": "high",
                            "min_inclusive": 110.0
                        }
                    ],
                    "else_value": "unknown"
                },
                {
                    "kind": "score_rules",
                    "output_column": "priority_score",
                    "default_score": 0,
                    "rules": [
                        {
                            "when": {
                                "column": "region",
                                "operator": "equals",
                                "value": "East"
                            },
                            "score": 10
                        },
                        {
                            "when": {
                                "column": "sales",
                                "operator": "gte",
                                "value": "110"
                            },
                            "score": 5
                        }
                    ]
                }
            ]
        }
    });

    let output = run_cli_with_json(&request.to_string());
    // 2026-03-22: 这里先锁定最小派生字段引擎，目的是让客户分层、旺淡季标签和优先级评分这些经营分析中间表能在 Tool 层直接生成。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["result_ref"].is_string(), true);
    assert_eq!(output["data"]["rows"][0]["priority"], "A");
    assert_eq!(output["data"]["rows"][0]["sales_band"], "high");
    assert_eq!(output["data"]["rows"][0]["priority_score"], "15");
}

#[test]
fn derive_columns_supports_condition_groups_date_bucket_and_template_in_cli() {
    let workbook_path = create_test_workbook(
        "derive_columns_advanced_cli",
        "derive-columns-advanced.xlsx",
        &[(
            "Sales",
            vec![
                vec!["customer_id", "sales", "visits", "biz_date", "region"],
                vec!["C001", "120", "3", "2026-01-15", "East"],
                vec!["C002", "95", "5", "2026-04-10", "West"],
                vec!["C003", "60", "1", "2026-08-01", "North"],
            ],
        )],
    );

    let request = json!({
        "tool": "derive_columns",
        "args": {
            "path": workbook_path.to_string_lossy(),
            "sheet": "Sales",
            "casts": [
                {
                    "column": "sales",
                    "target_type": "int64"
                },
                {
                    "column": "visits",
                    "target_type": "int64"
                }
            ],
            "derivations": [
                {
                    "kind": "case_when",
                    "output_column": "priority",
                    "rules": [
                        {
                            "when": {
                                "mode": "all",
                                "conditions": [
                                    {
                                        "column": "sales",
                                        "operator": "gte",
                                        "value": "100"
                                    },
                                    {
                                        "column": "visits",
                                        "operator": "gte",
                                        "value": "3"
                                    }
                                ]
                            },
                            "value": "A"
                        },
                        {
                            "when": {
                                "mode": "any",
                                "conditions": [
                                    {
                                        "column": "sales",
                                        "operator": "gte",
                                        "value": "90"
                                    },
                                    {
                                        "column": "visits",
                                        "operator": "gte",
                                        "value": "5"
                                    }
                                ]
                            },
                            "value": "B"
                        }
                    ],
                    "else_value": "C"
                },
                {
                    "kind": "date_bucketize",
                    "source_column": "biz_date",
                    "output_column": "season",
                    "buckets": [
                        {
                            "label": "Q1",
                            "start_inclusive": "2026-01-01",
                            "end_exclusive": "2026-04-01"
                        },
                        {
                            "label": "Q2",
                            "start_inclusive": "2026-04-01",
                            "end_exclusive": "2026-07-01"
                        }
                    ],
                    "else_value": "H2"
                },
                {
                    "kind": "template",
                    "output_column": "reason",
                    "template": "{customer_id}-{region}-{priority}-{season}"
                }
            ]
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-23: 这里锁定 CLI 层也能直接产出条件组标签、日期分段与说明列，目的是让 Skill 能少问一步直接生成中间分析表。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["rows"][0]["priority"], "A");
    assert_eq!(output["data"]["rows"][1]["priority"], "B");
    assert_eq!(output["data"]["rows"][2]["priority"], "C");
    assert_eq!(output["data"]["rows"][0]["season"], "Q1");
    assert_eq!(output["data"]["rows"][1]["season"], "Q2");
    assert_eq!(output["data"]["rows"][2]["season"], "H2");
    assert_eq!(output["data"]["rows"][0]["reason"], "C001-East-A-Q1");
    assert_eq!(output["data"]["rows"][1]["reason"], "C002-West-B-Q2");
    assert!(
        output["data"]["result_ref"]
            .as_str()
            .unwrap()
            .starts_with("result_")
    );
}

#[test]
fn join_tables_returns_matched_rows_for_explicit_join_request() {
    let request = json!({
        "tool": "join_tables",
        "args": {
            "left": {
                "path": "tests/fixtures/join-customers.xlsx",
                "sheet": "Customers"
            },
            "right": {
                "path": "tests/fixtures/join-orders.xlsx",
                "sheet": "Orders"
            },
            "left_on": "user_id",
            "right_on": "user_id",
            "keep_mode": "matched_only"
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["row_count"], 3);
    assert_eq!(
        output["data"]["columns"],
        json!(["user_id", "name", "region", "order_id", "amount"])
    );
    assert_eq!(output["data"]["rows"][0]["name"], "Alice");
    assert_eq!(output["data"]["rows"][1]["order_id"], "102");
    assert_eq!(output["data"]["rows"][2]["amount"], "90");
}

#[test]
fn join_tables_returns_reusable_result_ref_for_follow_up_analysis() {
    let join_request = json!({
        "tool": "join_tables",
        "args": {
            "left": {
                "path": "tests/fixtures/join-customers.xlsx",
                "sheet": "Customers"
            },
            "right": {
                "path": "tests/fixtures/join-orders.xlsx",
                "sheet": "Orders"
            },
            "left_on": "user_id",
            "right_on": "user_id",
            "keep_mode": "matched_only"
        }
    });

    let join_output = run_cli_with_json(&join_request.to_string());
    assert_eq!(join_output["status"], "ok");
    let result_ref = join_output["data"]["result_ref"]
        .as_str()
        .expect("join_tables should return result_ref")
        .to_string();

    let stat_request = json!({
        "tool": "stat_summary",
        "args": {
            "result_ref": result_ref,
            "casts": [
                {
                    "column": "amount",
                    "target_type": "int64"
                }
            ],
            "columns": ["amount"]
        }
    });

    let stat_output = run_cli_with_json(&stat_request.to_string());
    // 2026-03-22: 这里锁定多表关联结果也能无缝进入分析层，目的是把多表链式闭环补齐到客户可直接使用。
    assert_eq!(stat_output["status"], "ok");
    assert_eq!(
        stat_output["data"]["numeric_summaries"][0]["column"],
        "amount"
    );
}

#[test]
fn join_preflight_returns_estimates_risks_and_preview_rows_in_cli() {
    let request = json!({
        "tool": "join_preflight",
        "args": {
            "left": {
                "path": "tests/fixtures/join-customers.xlsx",
                "sheet": "Customers"
            },
            "right": {
                "path": "tests/fixtures/join-orders.xlsx",
                "sheet": "Orders"
            },
            "left_on": "user_id",
            "right_on": "user_id",
            "keep_mode": "matched_only",
            "limit": 5
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["left_on"], "user_id");
    assert_eq!(output["data"]["right_on"], "user_id");
    assert_eq!(output["data"]["selected_keep_mode"], "matched_only");
    assert!(output["data"]["preview_rows"].is_array());
    assert!(output["data"]["risk_summary"].is_array());
    assert_eq!(
        output["data"]["suggested_join_tool_call"]["tool"],
        "join_tables"
    );
    assert_eq!(
        output["data"]["suggested_join_tool_call"]["args"]["left_on"],
        "user_id"
    );
    assert_eq!(
        output["data"]["suggested_join_tool_call"]["args"]["right_on"],
        "user_id"
    );
}

#[test]
fn join_preflight_confirm_join_returns_confirmed_join_tool_call_in_cli() {
    let preflight_request = json!({
        "tool": "join_preflight",
        "args": {
            "left": {
                "path": "tests/fixtures/join-customers.xlsx",
                "sheet": "Customers"
            },
            "right": {
                "path": "tests/fixtures/join-orders.xlsx",
                "sheet": "Orders"
            },
            "left_on": "user_id",
            "right_on": "user_id",
            "keep_mode": "matched_only",
            "confirm_join": true
        }
    });

    let preflight_output = run_cli_with_json(&preflight_request.to_string());
    assert_eq!(preflight_output["status"], "ok");
    assert_eq!(
        preflight_output["data"]["confirmed_join_tool_call"]["tool"],
        "join_tables"
    );
    assert!(
        preflight_output["data"]["recommended_next_step"]
            .as_str()
            .unwrap()
            .contains("confirmed_join_tool_call")
    );

    let confirmed_join_request = json!({
        "tool": preflight_output["data"]["confirmed_join_tool_call"]["tool"],
        "args": preflight_output["data"]["confirmed_join_tool_call"]["args"]
    });
    let join_output = run_cli_with_json(&confirmed_join_request.to_string());
    assert_eq!(join_output["status"], "ok");
    assert_eq!(join_output["data"]["row_count"], 3);
}

#[test]
fn join_tables_accepts_casts_before_matching() {
    let request = json!({
        "tool": "join_tables",
        "args": {
            "left": {
                "path": "tests/fixtures/join-customers-padded.xlsx",
                "sheet": "Customers"
            },
            "right": {
                "path": "tests/fixtures/join-orders.xlsx",
                "sheet": "Orders"
            },
            "left_on": "user_id",
            "right_on": "user_id",
            "left_casts": [
                {
                    "column": "user_id",
                    "target_type": "int64"
                }
            ],
            "right_casts": [
                {
                    "column": "user_id",
                    "target_type": "int64"
                }
            ],
            "keep_mode": "matched_only"
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["row_count"], 3);
    // 2026-03-21: 这里校验显式类型对齐后的关联结果，目的是确保带前导零的字符串 ID 在统一转型后也能正确匹配。
    assert_eq!(output["data"]["rows"][0]["name"], "Alice");
    assert_eq!(output["data"]["rows"][1]["order_id"], "102");
    assert_eq!(output["data"]["rows"][2]["amount"], "90");
}

#[test]
fn join_tables_aligns_integer_and_float_result_refs_without_manual_casts() {
    let store = thread_result_ref_store();
    let left_result_ref = ResultRefStore::create_result_ref();
    let right_result_ref = ResultRefStore::create_result_ref();
    // 2026-03-23: 这里手工准备整数键 result_ref，目的是锁定 CLI 在多步链路里也能直接复用数值型中间结果做显性关联。
    let left_dataframe = DataFrame::new(vec![
        Series::new("user_id".into(), [1_i64, 2_i64, 3_i64]).into(),
        Series::new("name".into(), ["Alice", "Bob", "Cara"]).into(),
    ])
    .unwrap();
    let left_record = PersistedResultDataset::from_dataframe(
        &left_result_ref,
        "seed_join_left_int",
        vec!["seed_join_left_int".to_string()],
        &left_dataframe,
    )
    .unwrap();
    store.save(&left_record).unwrap();
    // 2026-03-23: 这里手工准备浮点键 result_ref，目的是复现 step_n_result 链路里“1”和“1.0”本该相等却可能错过匹配的问题。
    let right_dataframe = DataFrame::new(vec![
        Series::new("user_id".into(), [1.0_f64, 2.0_f64, 4.0_f64]).into(),
        Series::new("order_id".into(), ["A-101", "A-102", "A-104"]).into(),
    ])
    .unwrap();
    let right_record = PersistedResultDataset::from_dataframe(
        &right_result_ref,
        "seed_join_right_float",
        vec!["seed_join_right_float".to_string()],
        &right_dataframe,
    )
    .unwrap();
    store.save(&right_record).unwrap();

    let request = json!({
        "tool": "join_tables",
        "args": {
            "left": {
                "result_ref": left_result_ref
            },
            "right": {
                "result_ref": right_result_ref
            },
            "left_on": "user_id",
            "right_on": "user_id",
            "keep_mode": "matched_only"
        }
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "ok");
    // 2026-03-23: 这里锁定 CLI 层也会把整数键与浮点键按同一数值语义对齐，目的是让中间结果链式 join 更稳。
    assert_eq!(output["data"]["row_count"], 2);
    assert_eq!(output["data"]["rows"][0]["user_id"], "1");
    assert_eq!(output["data"]["rows"][0]["name"], "Alice");
    assert_eq!(output["data"]["rows"][0]["order_id"], "A-101");
    assert_eq!(output["data"]["rows"][1]["user_id"], "2");
    assert_eq!(output["data"]["rows"][1]["order_id"], "A-102");
}

#[test]
fn join_tables_ignores_blank_keys_in_cli_requests() {
    let request = json!({
        "tool": "join_tables",
        "args": {
            "left": {
                "path": "tests/fixtures/join-empty-keys.xlsx",
                "sheet": "Customers"
            },
            "right": {
                "path": "tests/fixtures/join-empty-keys.xlsx",
                "sheet": "Orders"
            },
            "left_on": "user_id",
            "right_on": "user_id",
            "keep_mode": "matched_only"
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    // 2026-03-21: 这里锁定 CLI 层也忽略空键，目的是防止问答界面把“未填 ID”数据误当作关联成功。
    assert_eq!(output["data"]["row_count"], 1);
    assert_eq!(output["data"]["rows"][0]["user_id"], "1");
    assert_eq!(output["data"]["rows"][0]["order_id"], "101");
}

#[test]
fn join_tables_expands_many_to_many_matches_and_keeps_stable_columns_in_cli() {
    let request = json!({
        "tool": "join_tables",
        "args": {
            "left": {
                "path": "tests/fixtures/join-conflict-columns.xlsx",
                "sheet": "Left"
            },
            "right": {
                "path": "tests/fixtures/join-conflict-columns.xlsx",
                "sheet": "Right"
            },
            "left_on": "user_id",
            "right_on": "user_id",
            "keep_mode": "matched_only",
            "limit": 5
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    // 2026-03-21: 这里锁定 CLI 多对多展开结果，目的是保证 Skill 看到的行数与底层 join 语义一致。
    assert_eq!(output["data"]["row_count"], 5);
    // 2026-03-21: 这里锁定连续冲突列名，目的是让上层编排能稳定引用右表补充列。
    assert_eq!(
        output["data"]["columns"],
        json!([
            "user_id",
            "region",
            "region_right",
            "tag",
            "region_right_right",
            "region_right_right_right",
            "amount"
        ])
    );
    assert_eq!(output["data"]["rows"][0]["amount"], "120");
}

#[test]
fn summarize_table_returns_column_profiles() {
    let request = json!({
        "tool": "summarize_table",
        "args": {
            "path": "tests/fixtures/basic-sales.xlsx",
            "sheet": "Sales",
            "columns": ["region", "sales"],
            "casts": [
                {
                    "column": "sales",
                    "target_type": "int64"
                }
            ],
            "top_k": 2
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["row_count"], 2);
    assert_eq!(output["data"]["summaries"][0]["column"], "region");
    // 2026-03-21: 这里校验文本列摘要会正确经过 Tool 返回，目的是保证问答界面能直接消费列画像。
    assert_eq!(output["data"]["summaries"][0]["summary_kind"], "string");
    assert_eq!(output["data"]["summaries"][0]["distinct_count"], 2);
    // 2026-03-21: 这里补 CLI 缺失率断言，目的是确保问答界面直接读取 Tool 输出即可得到质量指标。
    assert_eq!(output["data"]["summaries"][0]["missing_rate"], 0.0);
    // 2026-03-21: 这里校验数值列会先完成 cast 再做摘要，目的是让统计结果基于真实数值而不是字符串。
    assert_eq!(output["data"]["summaries"][1]["column"], "sales");
    assert_eq!(output["data"]["summaries"][1]["summary_kind"], "numeric");
    // 2026-03-21: 这里补数值列缺失率断言，目的是保证 CLI JSON 与内存摘要结构保持一致。
    assert_eq!(output["data"]["summaries"][1]["missing_rate"], 0.0);
    assert_eq!(output["data"]["summaries"][1]["mean"], 107.5);
    assert_eq!(output["data"]["summaries"][1]["sum"], 215.0);
}

#[test]
fn summarize_table_treats_blank_excel_cells_as_missing() {
    let request = json!({
        "tool": "summarize_table",
        "args": {
            "path": "tests/fixtures/summary-blanks.xlsx",
            "sheet": "Profile",
            "columns": ["notes"],
            "top_k": 3
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["row_count"], 3);
    // 2026-03-21: 这里校验 Excel 空白单元格会在 CLI 返回中进入缺失统计，目的是稳定支撑问答式质量判断。
    assert_eq!(output["data"]["summaries"][0]["count"], 1);
    assert_eq!(output["data"]["summaries"][0]["null_count"], 2);
    // 2026-03-21: 这里补空白缺失率断言，目的是把 Excel 空白单元格的质量指标稳定输出到 JSON。
    assert_eq!(output["data"]["summaries"][0]["missing_rate"], 2.0 / 3.0);
    assert_eq!(output["data"]["summaries"][0]["distinct_count"], 1);
    assert_eq!(
        output["data"]["summaries"][0]["top_values"][0]["value"],
        "done"
    );
}

#[test]
fn summarize_table_treats_placeholder_excel_values_as_missing() {
    let request = json!({
        "tool": "summarize_table",
        "args": {
            "path": "tests/fixtures/summary-placeholders.xlsx",
            "sheet": "Profile",
            "columns": ["notes"],
            "top_k": 5
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    // 2026-03-21: 这里校验 Excel 占位缺失值会在 CLI 返回中按缺失处理，目的是避免上层误把它们当有效文本。
    assert_eq!(output["data"]["summaries"][0]["count"], 1);
    assert_eq!(output["data"]["summaries"][0]["null_count"], 4);
    // 2026-03-21: 这里补占位缺失率断言，目的是让上层可以直接判断这列是否大面积缺值。
    assert_eq!(output["data"]["summaries"][0]["missing_rate"], 0.8);
    assert_eq!(output["data"]["summaries"][0]["distinct_count"], 1);
    assert_eq!(
        output["data"]["summaries"][0]["top_values"][0]["value"],
        "done"
    );
}

#[test]
fn summarize_table_handles_excel_date_and_dirty_columns_stably() {
    let request = json!({
        "tool": "summarize_table",
        "args": {
            "path": "tests/fixtures/summary-mixed-dirty.xlsx",
            "sheet": "Profile",
            "columns": ["event_date", "dirty_score"],
            "top_k": 5
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    // 2026-03-21: 这里锁定日期文本列的缺失统计，目的是保证真实 Excel 输入下也能得到稳定画像。
    assert_eq!(output["data"]["summaries"][0]["column"], "event_date");
    assert_eq!(output["data"]["summaries"][0]["count"], 3);
    assert_eq!(output["data"]["summaries"][0]["null_count"], 2);
    assert_eq!(output["data"]["summaries"][0]["missing_rate"], 0.4);
    // 2026-03-21: 这里锁定脏数据列的缺失统计与离散分布，目的是防止真实 Excel 场景下把占位值误当有效值。
    assert_eq!(output["data"]["summaries"][1]["column"], "dirty_score");
    assert_eq!(output["data"]["summaries"][1]["count"], 3);
    assert_eq!(output["data"]["summaries"][1]["null_count"], 2);
    assert_eq!(output["data"]["summaries"][1]["missing_rate"], 0.4);
    assert_eq!(output["data"]["summaries"][1]["distinct_count"], 3);
}

#[test]
fn summarize_table_summarizes_wide_sheet_without_losing_columns() {
    let request = json!({
        "tool": "summarize_table",
        "args": {
            "path": "tests/fixtures/summary-wide.xlsx",
            "sheet": "Wide",
            "top_k": 1
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    // 2026-03-21: 这里校验超宽表仍能完整输出全部列摘要，目的是避免尾部列在 JSON 编排中被遗漏。
    assert_eq!(output["data"]["row_count"], 2);
    assert_eq!(output["data"]["summaries"].as_array().unwrap().len(), 40);
    assert_eq!(output["data"]["summaries"][0]["column"], "col_01");
    assert_eq!(output["data"]["summaries"][39]["column"], "col_40");
    // 2026-03-21: 这里锁定尾列缺失率，目的是确保超宽表尾部列的缺失统计仍然正确。
    assert_eq!(output["data"]["summaries"][39]["null_count"], 1);
    assert_eq!(output["data"]["summaries"][39]["missing_rate"], 0.5);
}

#[test]
fn stat_summary_returns_typed_summary_payload_in_cli() {
    let request = json!({
        "tool": "stat_summary",
        "args": {
            "path": "tests/fixtures/basic-sales.xlsx",
            "sheet": "Sales",
            "columns": ["region", "sales"],
            "casts": [
                {
                    "column": "sales",
                    "target_type": "int64"
                }
            ],
            "top_k": 2
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    // 2026-03-21: 这里锁定统计桥接 Tool 会返回表级概览和分类输出，目的是让后续建模层直接消费稳定 JSON 契约。
    assert_eq!(output["data"]["row_count"], 2);
    assert_eq!(output["data"]["column_count"], 2);
    assert_eq!(output["data"]["table_overview"]["numeric_columns"], 1);
    assert_eq!(output["data"]["table_overview"]["categorical_columns"], 1);
    assert_eq!(output["data"]["table_overview"]["boolean_columns"], 0);
    assert!(output["data"]["numeric_summaries"].is_array());
    assert!(output["data"]["categorical_summaries"].is_array());
    assert!(output["data"]["boolean_summaries"].is_array());
    assert!(output["data"]["human_summary"].is_object());
    // 2026-03-21: 这里锁定数值列会返回中位数和四分位数，目的是让 CLI 层从第一版开始就具备建模前统计桥接能力。
    assert_eq!(output["data"]["numeric_summaries"][0]["column"], "sales");
    assert_eq!(output["data"]["numeric_summaries"][0]["median"], 107.5);
    assert_eq!(output["data"]["numeric_summaries"][0]["q1"], 101.25);
    assert_eq!(output["data"]["numeric_summaries"][0]["q3"], 113.75);
    // 2026-03-21: 这里锁定类别列会返回 top_share，目的是让问答界面直接理解主值集中度。
    assert_eq!(
        output["data"]["categorical_summaries"][0]["column"],
        "region"
    );
    assert_eq!(output["data"]["categorical_summaries"][0]["top_share"], 0.5);
}

#[test]
fn stat_summary_reports_skew_and_distribution_in_cli() {
    let request = json!({
        "tool": "stat_summary",
        "args": {
            "path": "tests/fixtures/analyze-distribution.xlsx",
            "sheet": "Metrics",
            "columns": ["region", "zero_metric", "amount"],
            "casts": [
                {
                    "column": "zero_metric",
                    "target_type": "int64"
                },
                {
                    "column": "amount",
                    "target_type": "int64"
                }
            ],
            "top_k": 3
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    // 2026-03-21: 这里锁定真实 Excel 偏态列的中位数输出，目的是保证 CLI 桥接层在极端值场景下仍然稳定。
    assert_eq!(
        output["data"]["numeric_summaries"][0]["column"],
        "zero_metric"
    );
    assert_eq!(output["data"]["numeric_summaries"][0]["zero_ratio"], 0.8);
    assert_eq!(output["data"]["numeric_summaries"][1]["column"], "amount");
    assert_eq!(output["data"]["numeric_summaries"][1]["median"], 2.0);
    assert_eq!(output["data"]["numeric_summaries"][1]["q1"], 2.0);
    assert_eq!(output["data"]["numeric_summaries"][1]["q3"], 3.0);
    // 2026-03-21: 这里锁定主类别占比和中文摘要关键点，目的是让问答界面直接展示也有解释力。
    assert_eq!(
        output["data"]["categorical_summaries"][0]["column"],
        "region"
    );
    assert_eq!(output["data"]["categorical_summaries"][0]["top_share"], 0.8);
    assert!(
        output["data"]["human_summary"]["key_points"]
            .as_array()
            .unwrap()
            .iter()
            .any(|message| message.as_str().unwrap().contains("East"))
    );
    assert!(
        output["data"]["human_summary"]["key_points"]
            .as_array()
            .unwrap()
            .iter()
            .any(|message| message.as_str().unwrap().contains("amount")
                && message.as_str().unwrap().contains("长尾"))
    );
}

#[test]
fn append_tables_returns_combined_rows_for_matching_tables() {
    let request = json!({
        "tool": "append_tables",
        "args": {
            "top": {
                "path": "tests/fixtures/append-sales-a.xlsx",
                "sheet": "Sales"
            },
            "bottom": {
                "path": "tests/fixtures/append-sales-b.xlsx",
                "sheet": "Sales"
            },
            "limit": 4
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["row_count"], 4);
    assert_eq!(
        output["data"]["columns"],
        json!(["user_id", "region", "sales"])
    );
    assert_eq!(output["data"]["rows"][0]["user_id"], "1");
    assert_eq!(output["data"]["rows"][2]["user_id"], "3");
    assert_eq!(output["data"]["rows"][3]["sales"], "60");
}

#[test]
fn append_tables_returns_reusable_result_ref_for_follow_up_analysis() {
    let append_request = json!({
        "tool": "append_tables",
        "args": {
            "top": {
                "path": "tests/fixtures/append-sales-a.xlsx",
                "sheet": "Sales"
            },
            "bottom": {
                "path": "tests/fixtures/append-sales-b.xlsx",
                "sheet": "Sales"
            },
            "limit": 4
        }
    });

    let append_output = run_cli_with_json(&append_request.to_string());
    assert_eq!(append_output["status"], "ok");
    let result_ref = append_output["data"]["result_ref"]
        .as_str()
        .expect("append_tables should return result_ref")
        .to_string();

    let sort_request = json!({
        "tool": "top_n",
        "args": {
            "result_ref": result_ref,
            "casts": [
                {
                    "column": "sales",
                    "target_type": "int64"
                }
            ],
            "sorts": [
                {
                    "column": "sales",
                    "descending": true
                }
            ],
            "n": 1
        }
    });

    let sort_output = run_cli_with_json(&sort_request.to_string());
    // 2026-03-22: 这里锁定纵向追加结果也能继续复用，目的是让“先追加再分析”的基础路径正式成立。
    assert_eq!(sort_output["status"], "ok");
    assert_eq!(sort_output["data"]["rows"][0]["sales"], "120");
}

#[test]
fn export_csv_writes_result_ref_to_file() {
    let store = thread_result_ref_store();
    let result_ref = ResultRefStore::create_result_ref();
    let output_path = create_test_output_path("export_csv", "csv");
    let dataframe = DataFrame::new(vec![
        Series::new("customer_id".into(), ["c001", "c002"]).into(),
        Series::new("sales".into(), [120_i64, 90_i64]).into(),
    ])
    .unwrap();
    let record = PersistedResultDataset::from_dataframe(
        &result_ref,
        "group_and_aggregate",
        vec!["table_sales".to_string()],
        &dataframe,
    )
    .unwrap();
    store.save(&record).unwrap();

    let request = json!({
        "tool": "export_csv",
        "args": {
            "result_ref": result_ref,
            "output_path": output_path.to_string_lossy()
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    assert!(std::fs::exists(&output_path).unwrap());
    let csv_text = std::fs::read_to_string(&output_path).unwrap();
    // 2026-03-22: 这里锁定导出 CSV 后文件真正落盘，目的是补齐“能算也能交付”的最小出口能力。
    assert!(csv_text.contains("customer_id,sales"));
    assert!(csv_text.contains("c001,120"));
}

#[test]
fn export_excel_writes_result_ref_to_workbook() {
    let store = thread_result_ref_store();
    let result_ref = ResultRefStore::create_result_ref();
    let output_path = create_test_output_path("export_excel", "xlsx");
    let dataframe = DataFrame::new(vec![
        Series::new("customer_id".into(), ["c001", "c002"]).into(),
        Series::new("priority".into(), ["A", "B"]).into(),
    ])
    .unwrap();
    let record = PersistedResultDataset::from_dataframe(
        &result_ref,
        "derive_columns",
        vec!["result_previous".to_string()],
        &dataframe,
    )
    .unwrap();
    store.save(&record).unwrap();

    let request = json!({
        "tool": "export_excel",
        "args": {
            "result_ref": result_ref,
            "output_path": output_path.to_string_lossy(),
            "sheet_name": "Report"
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    assert!(std::fs::exists(&output_path).unwrap());

    let mut workbook = open_workbook_auto(&output_path).unwrap();
    let range = workbook.worksheet_range("Report").unwrap();
    // 2026-03-22: 这里锁定导出 XLSX 后工作表名和单元格内容可被再次读取，目的是保证普通客户拿到的是标准 Excel 文件。
    assert_eq!(range.get((0, 0)).unwrap().to_string(), "customer_id");
    assert_eq!(range.get((1, 0)).unwrap().to_string(), "c001");
    assert_eq!(range.get((1, 1)).unwrap().to_string(), "A");
}

#[test]
fn tool_catalog_includes_compose_workbook_and_export_excel_workbook() {
    let mut cmd = Command::cargo_bin("excel_skill").unwrap();
    let assert = cmd.assert().success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    // 2026-03-22: 这里先锁定目录里暴露 workbook 组装与导出入口，目的是让多 Sheet 交付链路可被上层显式发现。
    assert!(
        json["data"]["tool_catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool == "compose_workbook")
    );
    assert!(
        json["data"]["tool_catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool == "export_excel_workbook")
    );
}

#[test]
fn tool_catalog_includes_build_chart_and_export_chart_image() {
    let mut cmd = Command::cargo_bin("excel_skill").unwrap();
    let assert = cmd.assert().success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert!(
        json["data"]["tool_catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool == "build_chart")
    );
    assert!(
        json["data"]["tool_catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool == "export_chart_image")
    );
}

#[test]
fn build_chart_returns_chart_ref_for_result_ref() {
    let analysis_result_ref = create_multi_series_analysis_result_ref_for_cli();
    let request = json!({
        "tool": "build_chart",
        "args": {
            "source": {
                "result_ref": analysis_result_ref
            },
            "chart_type": "column",
            "title": "Revenue vs Profit",
            "category_column": "month",
            "series": [
                { "value_column": "revenue", "name": "Revenue" },
                { "value_column": "profit", "name": "Profit" }
            ]
        }
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "ok");
    assert!(
        output["data"]["chart_ref"]
            .as_str()
            .unwrap()
            .starts_with("chart_")
    );
    assert_eq!(output["data"]["chart_type"], "column");
    assert_eq!(output["data"]["series_count"], 2);
    assert_eq!(output["data"]["row_count"], 3);
}

#[test]
fn export_chart_image_writes_svg_for_column_chart() {
    let analysis_result_ref = create_multi_series_analysis_result_ref_for_cli();
    let build_request = json!({
        "tool": "build_chart",
        "args": {
            "source": {
                "result_ref": analysis_result_ref
            },
            "chart_type": "column",
            "title": "Revenue vs Profit",
            "category_column": "month",
            "series": [
                { "value_column": "revenue", "name": "Revenue" },
                { "value_column": "profit", "name": "Profit" }
            ]
        }
    });
    let build_output = run_cli_with_json(&build_request.to_string());
    assert_eq!(build_output["status"], "ok");
    let chart_ref = build_output["data"]["chart_ref"].as_str().unwrap();
    let output_path = create_test_output_path("export_chart_column_svg", "svg");

    // 2026-03-23: 这里先锁定 column 图能从 chart_ref 落成 SVG 文件，原因是独立图表 Tool 要先形成最小导出闭环；目的是避免上层 Skill 只能停留在“构图不出图”。
    let export_request = json!({
        "tool": "export_chart_image",
        "args": {
            "chart_ref": chart_ref,
            "output_path": output_path.to_string_lossy()
        }
    });
    let export_output = run_cli_with_json(&export_request.to_string());

    assert_eq!(export_output["status"], "ok");
    assert_eq!(export_output["data"]["format"], "svg");
    assert_eq!(
        export_output["data"]["output_path"],
        output_path.to_string_lossy().to_string()
    );
    let svg = std::fs::read_to_string(&output_path).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("Revenue vs Profit"));
    assert!(svg.contains("<rect"));
}

#[test]
fn export_chart_image_writes_svg_for_pie_chart() {
    let analysis_result_ref = create_pie_analysis_result_ref_for_cli();
    let build_request = json!({
        "tool": "build_chart",
        "args": {
            "source": {
                "result_ref": analysis_result_ref
            },
            "chart_type": "pie",
            "title": "Segment Share",
            "category_column": "segment",
            "value_column": "share"
        }
    });
    let build_output = run_cli_with_json(&build_request.to_string());
    assert_eq!(build_output["status"], "ok");
    let chart_ref = build_output["data"]["chart_ref"].as_str().unwrap();
    let output_path = create_test_output_path("export_chart_pie_svg", "svg");

    // 2026-03-23: 这里先锁定 pie 图也能独立落成 SVG，原因是 V1 图表导出不能只覆盖柱线图；目的是保证客户交付层最直观的比例图也可用。
    let export_request = json!({
        "tool": "export_chart_image",
        "args": {
            "chart_ref": chart_ref,
            "output_path": output_path.to_string_lossy()
        }
    });
    let export_output = run_cli_with_json(&export_request.to_string());

    assert_eq!(export_output["status"], "ok");
    assert_eq!(export_output["data"]["format"], "svg");
    let svg = std::fs::read_to_string(&output_path).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("Segment Share"));
    assert!(svg.contains("<path"));
}

#[test]
fn export_chart_image_rejects_non_svg_output_path() {
    let analysis_result_ref = create_chart_analysis_result_ref_for_cli();
    let build_request = json!({
        "tool": "build_chart",
        "args": {
            "source": {
                "result_ref": analysis_result_ref
            },
            "chart_type": "column",
            "title": "Sales by Region",
            "category_column": "region",
            "value_column": "sales"
        }
    });
    let build_output = run_cli_with_json(&build_request.to_string());
    assert_eq!(build_output["status"], "ok");
    let chart_ref = build_output["data"]["chart_ref"].as_str().unwrap();
    let output_path = create_test_output_path("export_chart_png_reject", "png");

    // 2026-03-23: 这里先锁定独立图表导出不会悄悄接受非 svg 路径，原因是当前 V1 只承诺纯 Rust SVG；目的是防止上层误以为 PNG 等位图已经稳定可交付。
    let export_request = json!({
        "tool": "export_chart_image",
        "args": {
            "chart_ref": chart_ref,
            "output_path": output_path.to_string_lossy()
        }
    });
    let export_output = run_cli_with_json(&export_request.to_string());

    assert_eq!(export_output["status"], "error");
    assert_eq!(
        export_output["error"],
        "export_chart_image 目前只支持导出 svg"
    );
}

#[test]
fn build_chart_rejects_missing_series() {
    let analysis_result_ref = create_chart_analysis_result_ref_for_cli();
    let request = json!({
        "tool": "build_chart",
        "args": {
            "source": {
                "result_ref": analysis_result_ref
            },
            "chart_type": "column",
            "title": "Sales by Region",
            "category_column": "region"
        }
    });

    // 2026-03-23: 这里先锁定 build_chart 在缺数值列时直接报明确错误，原因是图表能力不能把“没给 Y 值”默默拖到导出阶段；目的是让 Skill 能尽早补问用户。
    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "error");
    assert_eq!(output["error"], "build_chart 至少需要一个数值系列");
}

#[test]
fn tool_catalog_includes_report_delivery() {
    let mut cmd = Command::cargo_bin("excel_skill").unwrap();
    let assert = cmd.assert().success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    // 2026-03-23: 这里先锁定目录里暴露 report_delivery，原因是结果交付层需要有独立上层入口；目的是避免继续只能手工串 format/compose/export。
    assert!(
        json["data"]["tool_catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool == "report_delivery")
    );
}

#[test]
fn report_delivery_returns_workbook_ref_for_standard_template() {
    let summary_result_ref = create_summary_result_ref_for_cli();
    let request = json!({
        "tool": "report_delivery",
        "args": {
            "report_name": "经营分析汇报",
            "summary": {
                "sheet_name": "摘要页",
                "source": {
                    "result_ref": summary_result_ref
                }
            },
            "analysis": {
                "sheet_name": "分析结果页",
                "source": {
                    "path": "tests/fixtures/basic-sales.xlsx",
                    "sheet": "Sales"
                }
            },
            "include_chart_sheet": true,
            "chart_sheet_name": "图表页"
        }
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "ok");
    // 2026-03-23: 这里先锁定 report_delivery 第一轮会产出标准三页模板 workbook_ref，原因是要先建立结果交付层独立入口；目的是给后续真实图表接入留稳定插槽。
    assert_eq!(output["data"]["template"], "standard_report_v2");
    assert_eq!(output["data"]["report_name"], "经营分析汇报");
    assert_eq!(
        output["data"]["sheet_names"],
        json!(["摘要页", "分析结果页", "图表页"])
    );
    assert_eq!(output["data"]["sheet_count"], 3);
    assert!(
        output["data"]["workbook_ref"]
            .as_str()
            .unwrap()
            .starts_with("workbook_")
    );
}

#[test]
fn report_delivery_can_skip_chart_sheet() {
    let summary_result_ref = create_summary_result_ref_for_cli();
    let request = json!({
        "tool": "report_delivery",
        "args": {
            "report_name": "\u{7ecf}\u{8425}\u{5206}\u{6790}\u{6c47}\u{62a5}",
            "summary": {
                "sheet_name": "\u{6458}\u{8981}\u{9875}",
                "source": {
                    "result_ref": summary_result_ref
                }
            },
            "analysis": {
                "sheet_name": "\u{5206}\u{6790}\u{7ed3}\u{679c}\u{9875}",
                "source": {
                    "path": "tests/fixtures/basic-sales.xlsx",
                    "sheet": "Sales"
                }
            },
            "include_chart_sheet": false
        }
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["sheet_count"], 2);
    assert_eq!(
        output["data"]["sheet_names"],
        json!([
            "\u{6458}\u{8981}\u{9875}",
            "\u{5206}\u{6790}\u{7ed3}\u{679c}\u{9875}"
        ])
    );
}

#[test]
fn report_delivery_accepts_custom_sheet_names() {
    let summary_result_ref = create_summary_result_ref_for_cli();
    let request = json!({
        "tool": "report_delivery",
        "args": {
            "report_name": "\u{7ecf}\u{8425}\u{5206}\u{6790}\u{6c47}\u{62a5}",
            "summary": {
                "sheet_name": "\u{4e1a}\u{52a1}\u{6458}\u{8981}",
                "source": {
                    "result_ref": summary_result_ref
                }
            },
            "analysis": {
                "sheet_name": "\u{5206}\u{6790}\u{660e}\u{7ec6}",
                "source": {
                    "path": "tests/fixtures/basic-sales.xlsx",
                    "sheet": "Sales"
                }
            },
            "include_chart_sheet": true,
            "chart_sheet_name": "\u{8d8b}\u{52bf}\u{56fe}"
        }
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["sheet_names"],
        json!([
            "\u{4e1a}\u{52a1}\u{6458}\u{8981}",
            "\u{5206}\u{6790}\u{660e}\u{7ec6}",
            "\u{8d8b}\u{52bf}\u{56fe}"
        ])
    );
}

#[test]
fn report_delivery_updates_session_state_to_workbook_ref() {
    let runtime_db = create_test_runtime_db("report_delivery_state_workbook_ref");
    let summary_store = runtime_result_ref_store(&runtime_db);
    let summary_result_ref = ResultRefStore::create_result_ref();
    let summary_dataframe = DataFrame::new(vec![
        Series::new("指标".into(), ["总客户数", "总收入"]).into(),
        Series::new("值".into(), ["2", "215"]).into(),
    ])
    .unwrap();
    let summary_record = PersistedResultDataset::from_dataframe(
        &summary_result_ref,
        "seed_report_delivery_summary",
        vec!["seed_summary".to_string()],
        &summary_dataframe,
    )
    .unwrap();
    summary_store.save(&summary_record).unwrap();
    let request = json!({
        "tool": "report_delivery",
        "args": {
            "session_id": "session_report_delivery_state_workbook_ref",
            "report_name": "\u{7ecf}\u{8425}\u{5206}\u{6790}\u{6c47}\u{62a5}",
            "summary": {
                "sheet_name": "\u{6458}\u{8981}\u{9875}",
                "source": {
                    "result_ref": summary_result_ref
                }
            },
            "analysis": {
                "sheet_name": "\u{5206}\u{6790}\u{7ed3}\u{679c}\u{9875}",
                "source": {
                    "path": "tests/fixtures/basic-sales.xlsx",
                    "sheet": "Sales"
                }
            }
        }
    });

    let report_output = run_cli_with_json_and_runtime(&request.to_string(), &runtime_db);
    assert_eq!(report_output["status"], "ok");
    let workbook_ref = report_output["data"]["workbook_ref"]
        .as_str()
        .unwrap()
        .to_string();

    let get_request = json!({
        "tool": "get_session_state",
        "args": {
            "session_id": "session_report_delivery_state_workbook_ref"
        }
    });
    let state_output = run_cli_with_json_and_runtime(&get_request.to_string(), &runtime_db);

    assert_eq!(state_output["status"], "ok");
    assert_eq!(state_output["data"]["active_handle_ref"], workbook_ref);
    assert_eq!(
        state_output["data"]["active_handle"]["kind"],
        "workbook_ref"
    );
}

#[test]
fn report_delivery_with_column_chart_can_be_exported() {
    let output_path = create_test_output_path("report_delivery_column_chart_cli", "xlsx");
    let summary_result_ref = create_summary_result_ref_for_cli();
    let analysis_result_ref = create_chart_analysis_result_ref_for_cli();
    let report_request = json!({
        "tool": "report_delivery",
        "args": {
            "report_name": "\u{533a}\u{57df}\u{6536}\u{5165}\u{6c47}\u{62a5}",
            "summary": {
                "sheet_name": "\u{6458}\u{8981}\u{9875}",
                "source": {
                    "result_ref": summary_result_ref
                }
            },
            "analysis": {
                "sheet_name": "\u{5206}\u{6790}\u{7ed3}\u{679c}\u{9875}",
                "source": {
                    "result_ref": analysis_result_ref
                }
            },
            "charts": [
                {
                    "chart_type": "column",
                    "title": "\u{533a}\u{57df}\u{6536}\u{5165}\u{67f1}\u{72b6}\u{56fe}",
                    "category_column": "region",
                    "value_column": "sales"
                }
            ]
        }
    });
    let report_output = run_cli_with_json(&report_request.to_string());
    assert_eq!(report_output["status"], "ok");
    let workbook_ref = report_output["data"]["workbook_ref"].as_str().unwrap();

    let export_request = json!({
        "tool": "export_excel_workbook",
        "args": {
            "workbook_ref": workbook_ref,
            "output_path": output_path.to_string_lossy()
        }
    });
    let export_output = run_cli_with_json(&export_request.to_string());

    assert_eq!(export_output["status"], "ok");
    let chart_xml = read_zip_entry_text(&output_path, "xl/charts/chart1.xml");
    assert!(chart_xml.contains("<c:barChart>"));
    assert!(chart_xml.contains("\u{533a}\u{57df}\u{6536}\u{5165}\u{67f1}\u{72b6}\u{56fe}"));
}

#[test]
fn report_delivery_with_line_chart_can_be_exported() {
    let output_path = create_test_output_path("report_delivery_line_chart_cli", "xlsx");
    let summary_result_ref = create_summary_result_ref_for_cli();
    let analysis_result_ref = create_chart_analysis_result_ref_for_cli();
    let report_request = json!({
        "tool": "report_delivery",
        "args": {
            "report_name": "\u{533a}\u{57df}\u{6536}\u{5165}\u{8d8b}\u{52bf}\u{6c47}\u{62a5}",
            "summary": {
                "sheet_name": "\u{6458}\u{8981}\u{9875}",
                "source": {
                    "result_ref": summary_result_ref
                }
            },
            "analysis": {
                "sheet_name": "\u{5206}\u{6790}\u{7ed3}\u{679c}\u{9875}",
                "source": {
                    "result_ref": analysis_result_ref
                }
            },
            "charts": [
                {
                    "chart_type": "line",
                    "title": "\u{533a}\u{57df}\u{6536}\u{5165}\u{6298}\u{7ebf}\u{56fe}",
                    "category_column": "region",
                    "value_column": "sales"
                }
            ]
        }
    });
    let report_output = run_cli_with_json(&report_request.to_string());
    assert_eq!(report_output["status"], "ok");
    let workbook_ref = report_output["data"]["workbook_ref"].as_str().unwrap();

    let export_request = json!({
        "tool": "export_excel_workbook",
        "args": {
            "workbook_ref": workbook_ref,
            "output_path": output_path.to_string_lossy()
        }
    });
    let export_output = run_cli_with_json(&export_request.to_string());

    assert_eq!(export_output["status"], "ok");
    let chart_xml = read_zip_entry_text(&output_path, "xl/charts/chart1.xml");
    assert!(chart_xml.contains("<c:lineChart>"));
    assert!(chart_xml.contains("\u{533a}\u{57df}\u{6536}\u{5165}\u{6298}\u{7ebf}\u{56fe}"));
}

#[test]
fn report_delivery_with_multi_series_column_chart_can_be_exported() {
    let output_path = create_test_output_path("report_delivery_multi_series_chart_cli", "xlsx");
    let summary_result_ref = create_summary_result_ref_for_cli();
    let analysis_result_ref = create_multi_series_analysis_result_ref_for_cli();
    let report_request = json!({
        "tool": "report_delivery",
        "args": {
            "report_name": "\u{6708}\u{5ea6}\u{7ecf}\u{8425}\u{6c47}\u{62a5}",
            "summary": {
                "sheet_name": "\u{6458}\u{8981}\u{9875}",
                "source": {
                    "result_ref": summary_result_ref
                }
            },
            "analysis": {
                "sheet_name": "\u{5206}\u{6790}\u{7ed3}\u{679c}\u{9875}",
                "source": {
                    "result_ref": analysis_result_ref
                }
            },
            "charts": [
                {
                    "chart_type": "column",
                    "title": "\u{8425}\u{6536}\u{4e0e}\u{5229}\u{6da6}\u{67f1}\u{72b6}\u{56fe}",
                    "category_column": "month",
                    "series": [
                        {
                            "value_column": "revenue",
                            "name": "\u{8425}\u{6536}"
                        },
                        {
                            "value_column": "profit",
                            "name": "\u{5229}\u{6da6}"
                        }
                    ]
                }
            ]
        }
    });
    let report_output = run_cli_with_json(&report_request.to_string());
    assert_eq!(report_output["status"], "ok");
    let workbook_ref = report_output["data"]["workbook_ref"].as_str().unwrap();

    let export_request = json!({
        "tool": "export_excel_workbook",
        "args": {
            "workbook_ref": workbook_ref,
            "output_path": output_path.to_string_lossy()
        }
    });
    let export_output = run_cli_with_json(&export_request.to_string());

    assert_eq!(export_output["status"], "ok");
    // 2026-03-23: 这里先锁定单图多系列会真正写进 chart XML，原因是多系列是图表第一版增强的核心价值；目的是防止接口接收了 series 但导出仍只画一条线。
    let chart_xml = read_zip_entry_text(&output_path, "xl/charts/chart1.xml");
    assert!(chart_xml.contains("<c:barChart>"));
    assert!(chart_xml.matches("<c:ser>").count() >= 2);
    assert!(chart_xml.contains("\u{8425}\u{6536}\u{4e0e}\u{5229}\u{6da6}\u{67f1}\u{72b6}\u{56fe}"));
}

#[test]
fn report_delivery_exports_multiple_charts_when_requested() {
    let output_path = create_test_output_path("report_delivery_multi_chart_layout_cli", "xlsx");
    let summary_result_ref = create_summary_result_ref_for_cli();
    let analysis_result_ref = create_multi_series_analysis_result_ref_for_cli();
    let report_request = json!({
        "tool": "report_delivery",
        "args": {
            "report_name": "\u{6708}\u{5ea6}\u{7ecf}\u{8425}\u{6c47}\u{62a5}",
            "summary": {
                "sheet_name": "\u{6458}\u{8981}\u{9875}",
                "source": {
                    "result_ref": summary_result_ref
                }
            },
            "analysis": {
                "sheet_name": "\u{5206}\u{6790}\u{7ed3}\u{679c}\u{9875}",
                "source": {
                    "result_ref": analysis_result_ref
                }
            },
            "charts": [
                {
                    "chart_type": "column",
                    "title": "\u{8425}\u{6536}\u{67f1}\u{72b6}\u{56fe}",
                    "category_column": "month",
                    "value_column": "revenue"
                },
                {
                    "chart_type": "line",
                    "title": "\u{5229}\u{6da6}\u{6298}\u{7ebf}\u{56fe}",
                    "category_column": "month",
                    "value_column": "profit"
                }
            ]
        }
    });
    let report_output = run_cli_with_json(&report_request.to_string());
    assert_eq!(report_output["status"], "ok");
    let workbook_ref = report_output["data"]["workbook_ref"].as_str().unwrap();

    let export_request = json!({
        "tool": "export_excel_workbook",
        "args": {
            "workbook_ref": workbook_ref,
            "output_path": output_path.to_string_lossy()
        }
    });
    let export_output = run_cli_with_json(&export_request.to_string());

    assert_eq!(export_output["status"], "ok");
    // 2026-03-23: 这里先锁定多图请求会产出多份 chart 部件，原因是自动布局只有在“至少两张图”时才有意义；目的是先证明导出链能承载多图而不是覆盖前一张图。
    let first_chart_xml = read_zip_entry_text(&output_path, "xl/charts/chart1.xml");
    let second_chart_xml = read_zip_entry_text(&output_path, "xl/charts/chart2.xml");
    assert!(first_chart_xml.contains("\u{8425}\u{6536}\u{67f1}\u{72b6}\u{56fe}"));
    assert!(second_chart_xml.contains("\u{5229}\u{6da6}\u{6298}\u{7ebf}\u{56fe}"));
}

#[test]
fn report_delivery_with_pie_chart_can_be_exported() {
    let output_path = create_test_output_path("report_delivery_pie_chart_cli", "xlsx");
    let summary_result_ref = create_ascii_summary_result_ref_for_cli();
    let analysis_result_ref = create_pie_analysis_result_ref_for_cli();
    let report_request = json!({
        "tool": "report_delivery",
        "args": {
            "report_name": "Category Mix Report",
            "summary": {
                "sheet_name": "Summary",
                "source": {
                    "result_ref": summary_result_ref
                }
            },
            "analysis": {
                "sheet_name": "Analysis",
                "source": {
                    "result_ref": analysis_result_ref
                }
            },
            "charts": [
                {
                    "chart_type": "pie",
                    "title": "Category Mix",
                    "category_column": "segment",
                    "value_column": "share"
                }
            ]
        }
    });
    let report_output = run_cli_with_json(&report_request.to_string());
    assert_eq!(report_output["status"], "ok");
    let workbook_ref = report_output["data"]["workbook_ref"].as_str().unwrap();

    let export_request = json!({
        "tool": "export_excel_workbook",
        "args": {
            "workbook_ref": workbook_ref,
            "output_path": output_path.to_string_lossy()
        }
    });
    let export_output = run_cli_with_json(&export_request.to_string());

    assert_eq!(export_output["status"], "ok");
    // 2026-03-23: 这里先锁定 pie 图会落成真实 Excel 图表，原因是图表第二批类型要先保证客户能直接打开查看；目的是避免只扩了枚举却没有真正写出图表部件。
    let chart_xml = read_zip_entry_text(&output_path, "xl/charts/chart1.xml");
    assert!(chart_xml.contains("<c:pieChart>"));
}

#[test]
fn report_delivery_with_scatter_chart_can_be_exported() {
    let output_path = create_test_output_path("report_delivery_scatter_chart_cli", "xlsx");
    let summary_result_ref = create_ascii_summary_result_ref_for_cli();
    let analysis_result_ref = create_scatter_analysis_result_ref_for_cli();
    let report_request = json!({
        "tool": "report_delivery",
        "args": {
            "report_name": "Scatter Report",
            "summary": {
                "sheet_name": "Summary",
                "source": {
                    "result_ref": summary_result_ref
                }
            },
            "analysis": {
                "sheet_name": "Analysis",
                "source": {
                    "result_ref": analysis_result_ref
                }
            },
            "charts": [
                {
                    "chart_type": "scatter",
                    "title": "Trend Scatter",
                    "category_column": "x_value",
                    "value_column": "y_value"
                }
            ]
        }
    });
    let report_output = run_cli_with_json(&report_request.to_string());
    assert_eq!(report_output["status"], "ok");
    let workbook_ref = report_output["data"]["workbook_ref"].as_str().unwrap();

    let export_request = json!({
        "tool": "export_excel_workbook",
        "args": {
            "workbook_ref": workbook_ref,
            "output_path": output_path.to_string_lossy()
        }
    });
    let export_output = run_cli_with_json(&export_request.to_string());

    assert_eq!(export_output["status"], "ok");
    // 2026-03-23: 这里先锁定 scatter 图会落成散点图 XML，原因是 scatter 与类目轴图的底层写法不同；目的是在最外层就拦住错误复用柱线图逻辑的回归。
    let chart_xml = read_zip_entry_text(&output_path, "xl/charts/chart1.xml");
    assert!(chart_xml.contains("<c:scatterChart>"));
}

#[test]
fn report_delivery_chart_style_controls_are_persisted_in_workbook_ref_json() {
    let summary_result_ref = create_ascii_summary_result_ref_for_cli();
    let analysis_result_ref = create_multi_series_analysis_result_ref_for_cli();
    let report_request = json!({
        "tool": "report_delivery",
        "args": {
            "report_name": "Styled Report",
            "summary": {
                "sheet_name": "Summary",
                "source": {
                    "result_ref": summary_result_ref
                }
            },
            "analysis": {
                "sheet_name": "Analysis",
                "source": {
                    "result_ref": analysis_result_ref
                }
            },
            "charts": [
                {
                    "chart_type": "column",
                    "title": "Revenue And Profit",
                    "category_column": "month",
                    "series": [
                        { "value_column": "revenue", "name": "Revenue" },
                        { "value_column": "profit", "name": "Profit" }
                    ],
                    "show_legend": true,
                    "legend_position": "bottom",
                    "chart_style": 10,
                    "x_axis_name": "Month",
                    "y_axis_name": "Amount"
                }
            ]
        }
    });
    let report_output = run_cli_with_json(&report_request.to_string());
    assert_eq!(report_output["status"], "ok");
    let workbook_ref = report_output["data"]["workbook_ref"].as_str().unwrap();
    let payload = std::fs::read_to_string(workbook_ref_json_path(workbook_ref)).unwrap();
    let json: serde_json::Value = serde_json::from_str(&payload).unwrap();
    let chart = &json["charts"][0];

    // 2026-03-23: 这里先锁定样式开关会进入 workbook_ref 草稿，原因是导出层应只消费稳定元数据；目的是避免 legend 和 style 逻辑重新散落回 dispatcher。
    assert_eq!(chart["show_legend"], true);
    assert_eq!(chart["legend_position"], "bottom");
    assert_eq!(chart["chart_style"], 10);
    assert_eq!(chart["x_axis_name"], "Month");
    assert_eq!(chart["y_axis_name"], "Amount");
}

#[test]
fn report_delivery_with_legend_position_and_style_can_be_exported() {
    let output_path = create_test_output_path("report_delivery_style_chart_cli", "xlsx");
    let summary_result_ref = create_ascii_summary_result_ref_for_cli();
    let analysis_result_ref = create_multi_series_analysis_result_ref_for_cli();
    let report_request = json!({
        "tool": "report_delivery",
        "args": {
            "report_name": "Styled Export Report",
            "summary": {
                "sheet_name": "Summary",
                "source": {
                    "result_ref": summary_result_ref
                }
            },
            "analysis": {
                "sheet_name": "Analysis",
                "source": {
                    "result_ref": analysis_result_ref
                }
            },
            "charts": [
                {
                    "chart_type": "column",
                    "title": "Styled Revenue",
                    "category_column": "month",
                    "series": [
                        { "value_column": "revenue", "name": "Revenue" },
                        { "value_column": "profit", "name": "Profit" }
                    ],
                    "show_legend": true,
                    "legend_position": "bottom",
                    "chart_style": 10,
                    "x_axis_name": "Month",
                    "y_axis_name": "Amount"
                }
            ]
        }
    });
    let report_output = run_cli_with_json(&report_request.to_string());
    assert_eq!(report_output["status"], "ok");
    let workbook_ref = report_output["data"]["workbook_ref"].as_str().unwrap();

    let export_request = json!({
        "tool": "export_excel_workbook",
        "args": {
            "workbook_ref": workbook_ref,
            "output_path": output_path.to_string_lossy()
        }
    });
    let export_output = run_cli_with_json(&export_request.to_string());

    assert_eq!(export_output["status"], "ok");
    // 2026-03-23: 这里先锁定 legend 位置与样式号真的被写入图表 XML，原因是客户能感知到的不是参数解析，而是导出后的样式结果；目的是防止样式字段只保存在草稿里却没有进入 Excel。
    let chart_xml = read_zip_entry_text(&output_path, "xl/charts/chart1.xml");
    assert!(chart_xml.contains("<c:style val=\"10\"/>"));
    assert!(chart_xml.contains("<c:legendPos val=\"b\"/>"));
}

#[test]
fn report_delivery_export_writes_sheet_titles_before_data() {
    let output_path = create_test_output_path("report_delivery_layout_cli", "xlsx");
    let summary_result_ref = create_ascii_summary_result_ref_for_cli();
    let analysis_result_ref = create_chart_analysis_result_ref_for_cli();
    let report_request = json!({
        "tool": "report_delivery",
        "args": {
            "report_name": "Delivery Report",
            "report_subtitle": "Layout Smoke Test",
            "summary": {
                "sheet_name": "Summary",
                "source": {
                    "result_ref": summary_result_ref
                }
            },
            "analysis": {
                "sheet_name": "Analysis",
                "source": {
                    "result_ref": analysis_result_ref
                }
            },
            "charts": [
                {
                    "chart_type": "column",
                    "title": "Revenue By Region",
                    "category_column": "region",
                    "value_column": "sales"
                }
            ]
        }
    });
    let report_output = run_cli_with_json(&report_request.to_string());
    assert_eq!(report_output["status"], "ok");
    assert_eq!(report_output["data"]["template"], "standard_report_v2");
    let workbook_ref = report_output["data"]["workbook_ref"].as_str().unwrap();

    let export_request = json!({
        "tool": "export_excel_workbook",
        "args": {
            "workbook_ref": workbook_ref,
            "output_path": output_path.to_string_lossy()
        }
    });
    let export_output = run_cli_with_json(&export_request.to_string());
    assert_eq!(export_output["status"], "ok");

    let mut workbook = open_workbook_auto(&output_path).unwrap();
    let summary = workbook.worksheet_range("Summary").unwrap();
    let analysis = workbook.worksheet_range("Analysis").unwrap();

    // 2026-03-23: 这里先锁定模板导出会在数据表上方写标题区，原因是交付层第二阶段要让客户打开即读；目的是避免报告页仍然像纯原始表格一样缺少汇报上下文。
    assert_eq!(
        summary.get((0, 0)),
        Some(&Data::String("Delivery Report".to_string()))
    );
    assert_eq!(
        summary.get((1, 0)),
        Some(&Data::String("Layout Smoke Test".to_string()))
    );
    assert_eq!(
        summary.get((2, 0)),
        Some(&Data::String("metric".to_string()))
    );
    assert_eq!(
        analysis.get((0, 0)),
        Some(&Data::String("Delivery Report".to_string()))
    );
    assert_eq!(
        analysis.get((2, 0)),
        Some(&Data::String("region".to_string()))
    );
}

#[test]
fn report_delivery_export_merges_title_rows_across_table_width() {
    let output_path = create_test_output_path("report_delivery_merge_title_cli", "xlsx");
    let summary_result_ref = create_ascii_summary_result_ref_for_cli();
    let analysis_result_ref = create_chart_analysis_result_ref_for_cli();
    let report_request = json!({
        "tool": "report_delivery",
        "args": {
            "report_name": "Merged Delivery",
            "report_subtitle": "Merged Subtitle",
            "summary": {
                "sheet_name": "Summary",
                "source": {
                    "result_ref": summary_result_ref
                }
            },
            "analysis": {
                "sheet_name": "Analysis",
                "source": {
                    "result_ref": analysis_result_ref
                }
            },
            "include_chart_sheet": false
        }
    });
    let report_output = run_cli_with_json(&report_request.to_string());
    assert_eq!(report_output["status"], "ok");
    let workbook_ref = report_output["data"]["workbook_ref"].as_str().unwrap();

    let export_request = json!({
        "tool": "export_excel_workbook",
        "args": {
            "workbook_ref": workbook_ref,
            "output_path": output_path.to_string_lossy()
        }
    });
    let export_output = run_cli_with_json(&export_request.to_string());
    assert_eq!(export_output["status"], "ok");

    let sheet_xml = read_zip_entry_text(&output_path, "xl/worksheets/sheet1.xml");

    // 2026-03-24: 这里先锁定 report_delivery 标题区会横向合并，原因是只把标题写在 A1/A2 仍然更像普通数据表；目的是把交付页进一步推向汇报稿视觉结构。
    assert!(sheet_xml.contains("<mergeCells"));
    assert!(sheet_xml.contains("ref=\"A1:B1\""));
    assert!(sheet_xml.contains("ref=\"A2:B2\""));
}

#[test]
fn report_delivery_applies_inline_export_format_rules_to_sections() {
    let output_path = create_test_output_path("report_delivery_inline_format_cli", "xlsx");
    let report_request = json!({
        "tool": "report_delivery",
        "args": {
            "report_name": "Formatted Delivery",
            "report_subtitle": "Inline Formatting",
            "summary": {
                "sheet_name": "Summary",
                "source": {
                    "path": "tests/fixtures/basic-sales.xlsx",
                    "sheet": "Sales"
                },
                "format": {
                    "column_order": ["region", "sales"],
                    "rename_mappings": [
                        { "from": "region", "to": "区域" },
                        { "from": "sales", "to": "收入" }
                    ],
                    "drop_unspecified_columns": true
                }
            },
            "analysis": {
                "sheet_name": "Analysis",
                "source": {
                    "path": "tests/fixtures/basic-sales.xlsx",
                    "sheet": "Sales"
                },
                "format": {
                    "column_order": ["user_id", "sales", "region"],
                    "rename_mappings": [
                        { "from": "user_id", "to": "客户ID" },
                        { "from": "sales", "to": "销售额" },
                        { "from": "region", "to": "区域" }
                    ],
                    "drop_unspecified_columns": true
                }
            },
            "include_chart_sheet": false
        }
    });
    let report_output = run_cli_with_json(&report_request.to_string());
    assert_eq!(report_output["status"], "ok");
    let workbook_ref = report_output["data"]["workbook_ref"].as_str().unwrap();

    let export_request = json!({
        "tool": "export_excel_workbook",
        "args": {
            "workbook_ref": workbook_ref,
            "output_path": output_path.to_string_lossy()
        }
    });
    let export_output = run_cli_with_json(&export_request.to_string());
    assert_eq!(export_output["status"], "ok");

    let mut workbook = open_workbook_auto(&output_path).unwrap();
    let summary = workbook.worksheet_range("Summary").unwrap();
    let analysis = workbook.worksheet_range("Analysis").unwrap();

    // 2026-03-24: 这里先锁定 report_delivery 可直接承接导出整理规则，原因是结果交付层不该要求上层再手工串一次 format_table_for_export；目的是把“整理 -> 交付”收口成单个高层入口。
    assert_eq!(summary.get((2, 0)), Some(&Data::String("区域".to_string())));
    assert_eq!(summary.get((2, 1)), Some(&Data::String("收入".to_string())));
    assert_eq!(summary.get((2, 2)), None);
    assert_eq!(
        analysis.get((2, 0)),
        Some(&Data::String("客户ID".to_string()))
    );
    assert_eq!(
        analysis.get((2, 1)),
        Some(&Data::String("销售额".to_string()))
    );
    assert_eq!(
        analysis.get((2, 2)),
        Some(&Data::String("区域".to_string()))
    );
}

#[test]
fn report_delivery_accepts_chart_ref_and_exports_workbook() {
    let output_path = create_test_output_path("report_delivery_chart_ref_cli", "xlsx");
    let summary_result_ref = create_ascii_summary_result_ref_for_cli();
    let analysis_result_ref = create_multi_series_analysis_result_ref_for_cli();
    let chart_ref = create_chart_ref_from_result_ref_for_cli(
        &analysis_result_ref,
        "column",
        "month",
        "Revenue vs Profit",
        json!([
            { "value_column": "revenue", "name": "Revenue" },
            { "value_column": "profit", "name": "Profit" }
        ]),
    );
    let report_request = json!({
        "tool": "report_delivery",
        "args": {
            "report_name": "ChartRef Delivery",
            "summary": {
                "sheet_name": "Summary",
                "source": {
                    "result_ref": summary_result_ref
                }
            },
            "analysis": {
                "sheet_name": "Analysis",
                "source": {
                    "result_ref": analysis_result_ref
                }
            },
            "chart_sheet_name": "Charts",
            "charts": [
                {
                    "chart_ref": chart_ref
                }
            ]
        }
    });

    // 2026-03-24: 这里先锁定 report_delivery 能直接消费 chart_ref，原因是方案 A 要统一独立图表与 workbook 交付；目的是避免继续维护两套图表输入心智。
    let report_output = run_cli_with_json(&report_request.to_string());
    assert_eq!(report_output["status"], "ok");
    let workbook_ref = report_output["data"]["workbook_ref"].as_str().unwrap();

    let export_request = json!({
        "tool": "export_excel_workbook",
        "args": {
            "workbook_ref": workbook_ref,
            "output_path": output_path.to_string_lossy()
        }
    });
    let export_output = run_cli_with_json(&export_request.to_string());

    assert_eq!(export_output["status"], "ok");
    let chart_xml = read_zip_entry_text(&output_path, "xl/charts/chart1.xml");
    assert!(chart_xml.contains("<c:barChart>"));
}

#[test]
fn report_delivery_can_mix_chart_ref_with_inline_chart_specs() {
    let output_path = create_test_output_path("report_delivery_mixed_chart_inputs_cli", "xlsx");
    let summary_result_ref = create_ascii_summary_result_ref_for_cli();
    let analysis_result_ref = create_multi_series_analysis_result_ref_for_cli();
    let chart_ref = create_chart_ref_from_result_ref_for_cli(
        &analysis_result_ref,
        "column",
        "month",
        "Revenue vs Profit",
        json!([
            { "value_column": "revenue", "name": "Revenue" },
            { "value_column": "profit", "name": "Profit" }
        ]),
    );
    let report_request = json!({
        "tool": "report_delivery",
        "args": {
            "report_name": "Mixed Chart Inputs",
            "summary": {
                "sheet_name": "Summary",
                "source": {
                    "result_ref": summary_result_ref
                }
            },
            "analysis": {
                "sheet_name": "Analysis",
                "source": {
                    "result_ref": analysis_result_ref
                }
            },
            "chart_sheet_name": "Charts",
            "charts": [
                {
                    "chart_ref": chart_ref
                },
                {
                    "chart_type": "line",
                    "title": "Profit Trend",
                    "category_column": "month",
                    "value_column": "profit"
                }
            ]
        }
    });

    // 2026-03-24: 这里先锁定 chart_ref 和 inline chart 可混用，原因是上层 Skill 未来会同时复用旧图和临时补图；目的是避免统一规格后反而牺牲编排灵活性。
    let report_output = run_cli_with_json(&report_request.to_string());
    assert_eq!(report_output["status"], "ok");
    let workbook_ref = report_output["data"]["workbook_ref"].as_str().unwrap();

    let export_request = json!({
        "tool": "export_excel_workbook",
        "args": {
            "workbook_ref": workbook_ref,
            "output_path": output_path.to_string_lossy()
        }
    });
    let export_output = run_cli_with_json(&export_request.to_string());

    assert_eq!(export_output["status"], "ok");
    let chart1_xml = read_zip_entry_text(&output_path, "xl/charts/chart1.xml");
    let chart2_xml = read_zip_entry_text(&output_path, "xl/charts/chart2.xml");
    assert!(chart1_xml.contains("<c:barChart>"));
    assert!(chart2_xml.contains("<c:lineChart>"));
}

#[test]
fn report_delivery_chart_ref_persists_chart_source_refs_into_workbook_ref() {
    let summary_result_ref = create_ascii_summary_result_ref_for_cli();
    let analysis_result_ref = create_multi_series_analysis_result_ref_for_cli();
    let chart_ref = create_chart_ref_from_result_ref_for_cli(
        &analysis_result_ref,
        "column",
        "month",
        "Revenue vs Profit",
        json!([
            { "value_column": "revenue", "name": "Revenue" },
            { "value_column": "profit", "name": "Profit" }
        ]),
    );
    let report_request = json!({
        "tool": "report_delivery",
        "args": {
            "report_name": "ChartRef Metadata",
            "summary": {
                "sheet_name": "Summary",
                "source": {
                    "result_ref": summary_result_ref
                }
            },
            "analysis": {
                "sheet_name": "Analysis",
                "source": {
                    "result_ref": analysis_result_ref
                }
            },
            "charts": [
                {
                    "chart_ref": chart_ref
                }
            ]
        }
    });

    // 2026-03-24: 这里先锁定 chart_ref 的血缘会进入 workbook_ref，原因是统一图表规格后仍要保留可解释的来源链；目的是给上层决策助手和审计链路留下抓手。
    let report_output = run_cli_with_json(&report_request.to_string());
    assert_eq!(report_output["status"], "ok");
    let workbook_ref = report_output["data"]["workbook_ref"].as_str().unwrap();
    let payload = std::fs::read_to_string(workbook_ref_json_path(workbook_ref)).unwrap();
    let json: serde_json::Value = serde_json::from_str(&payload).unwrap();
    let chart = &json["charts"][0];

    assert_eq!(chart["chart_ref"], chart_ref);
    assert_eq!(chart["source_refs"], json!([analysis_result_ref]));
}

#[test]
fn report_delivery_chart_ref_prefers_frozen_chart_layout_fields() {
    let summary_result_ref = create_ascii_summary_result_ref_for_cli();
    let analysis_result_ref = create_multi_series_analysis_result_ref_for_cli();
    let build_request = json!({
        "tool": "build_chart",
        "args": {
            "source": {
                "result_ref": analysis_result_ref
            },
            "chart_type": "column",
            "title": "Frozen Revenue vs Profit",
            "category_column": "month",
            "series": [
                { "value_column": "revenue", "name": "Revenue" },
                { "value_column": "profit", "name": "Profit" }
            ],
            "x_axis_name": "Frozen Month",
            "y_axis_name": "Frozen Amount",
            "show_legend": true,
            "width": 720,
            "height": 420
        }
    });
    let build_output = run_cli_with_json(&build_request.to_string());
    assert_eq!(build_output["status"], "ok");
    let chart_ref = build_output["data"]["chart_ref"].as_str().unwrap();

    let report_request = json!({
        "tool": "report_delivery",
        "args": {
            "report_name": "Frozen Layout Report",
            "summary": {
                "sheet_name": "Summary",
                "source": {
                    "result_ref": summary_result_ref
                }
            },
            "analysis": {
                "sheet_name": "Analysis",
                "source": {
                    "result_ref": analysis_result_ref
                }
            },
            "charts": [
                {
                    "chart_ref": chart_ref,
                    "title": "Should Not Override",
                    "x_axis_name": "Ignore X",
                    "y_axis_name": "Ignore Y",
                    "show_legend": false
                }
            ]
        }
    });

    // 2026-03-24: 这里先锁定 chart_ref 的冻结布局字段优先生效，原因是方案 A 的核心就是复用已冻结图表规格；目的是防止 report_delivery 把 chart_ref 退化成只传列名的半复用。
    let report_output = run_cli_with_json(&report_request.to_string());
    assert_eq!(report_output["status"], "ok");
    let workbook_ref = report_output["data"]["workbook_ref"].as_str().unwrap();
    let payload = std::fs::read_to_string(workbook_ref_json_path(workbook_ref)).unwrap();
    let json: serde_json::Value = serde_json::from_str(&payload).unwrap();
    let chart = &json["charts"][0];

    assert_eq!(chart["title"], "Frozen Revenue vs Profit");
    assert_eq!(chart["x_axis_name"], "Frozen Month");
    assert_eq!(chart["y_axis_name"], "Frozen Amount");
    assert_eq!(chart["show_legend"], true);
}

#[test]
fn report_delivery_rejects_missing_chart_ref() {
    let summary_result_ref = create_ascii_summary_result_ref_for_cli();
    let analysis_result_ref = create_multi_series_analysis_result_ref_for_cli();
    let request = json!({
        "tool": "report_delivery",
        "args": {
            "report_name": "Missing ChartRef Report",
            "summary": {
                "sheet_name": "Summary",
                "source": {
                    "result_ref": summary_result_ref
                }
            },
            "analysis": {
                "sheet_name": "Analysis",
                "source": {
                    "result_ref": analysis_result_ref
                }
            },
            "charts": [
                {
                    "chart_ref": "chart_missing_demo"
                }
            ]
        }
    });

    // 2026-03-24: 这里先锁定缺失 chart_ref 时会直接给明确错误，原因是上层 Skill 需要能区分“引用坏了”而不是“图表写出坏了”；目的是缩短排错路径。
    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "error");
    assert!(
        output["error"]
            .as_str()
            .unwrap()
            .contains("chart_missing_demo")
    );
}

#[test]
fn report_delivery_rejects_chart_ref_when_analysis_source_mismatches() {
    let summary_result_ref = create_ascii_summary_result_ref_for_cli();
    let chart_source_result_ref = create_multi_series_analysis_result_ref_for_cli();
    let mismatched_analysis_result_ref = create_chart_analysis_result_ref_for_cli();
    let chart_ref = create_chart_ref_from_result_ref_for_cli(
        &chart_source_result_ref,
        "column",
        "month",
        "Revenue vs Profit",
        json!([
            { "value_column": "revenue", "name": "Revenue" },
            { "value_column": "profit", "name": "Profit" }
        ]),
    );
    let request = json!({
        "tool": "report_delivery",
        "args": {
            "report_name": "Mismatched ChartRef Report",
            "summary": {
                "sheet_name": "Summary",
                "source": {
                    "result_ref": summary_result_ref
                }
            },
            "analysis": {
                "sheet_name": "Analysis",
                "source": {
                    "result_ref": mismatched_analysis_result_ref
                }
            },
            "charts": [
                {
                    "chart_ref": chart_ref
                }
            ]
        }
    });

    // 2026-03-24: 这里先锁定 chart_ref 与当前 analysis 数据不匹配时会保守报错，原因是 V1 还没把 chart_ref 冻结数据单独挂到 workbook；目的是避免客户拿到“看起来导出成功、实际图不对”的隐性错误。
    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "error");
    assert_eq!(
        output["error"],
        "report_delivery 的 chart_ref 与 analysis 数据不一致"
    );
}

#[test]
fn compose_workbook_updates_session_state_to_workbook_ref() {
    let runtime_db = create_test_runtime_db("compose_state_workbook_ref");
    let confirm_request = json!({
        "tool": "apply_header_schema",
        "args": {
            "session_id": "session_compose_state_workbook_ref",
            "path": "tests/fixtures/basic-sales.xlsx",
            "sheet": "Sales"
        }
    });
    let confirm_output = run_cli_with_json_and_runtime(&confirm_request.to_string(), &runtime_db);
    assert_eq!(confirm_output["status"], "ok");
    let table_ref = confirm_output["data"]["table_ref"]
        .as_str()
        .unwrap()
        .to_string();

    let compose_request = json!({
        "tool": "compose_workbook",
        "args": {
            "session_id": "session_compose_state_workbook_ref",
            "worksheets": [
                {
                    "sheet_name": "交付报表",
                    "source": {
                        "table_ref": table_ref
                    }
                }
            ]
        }
    });
    let compose_output = run_cli_with_json_and_runtime(&compose_request.to_string(), &runtime_db);
    assert_eq!(compose_output["status"], "ok");
    let workbook_ref = compose_output["data"]["workbook_ref"]
        .as_str()
        .unwrap()
        .to_string();

    let get_request = json!({
        "tool": "get_session_state",
        "args": {
            "session_id": "session_compose_state_workbook_ref"
        }
    });
    let output = run_cli_with_json_and_runtime(&get_request.to_string(), &runtime_db);

    // 2026-03-23: 这里锁定 compose_workbook 成功后会把当前激活句柄切到 workbook_ref，原因是交付草稿也是链式执行里的最新结果；目的是让后续 export_excel_workbook 直接承接当前状态。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["active_table_ref"], table_ref);
    assert_eq!(output["data"]["active_handle_ref"], workbook_ref);
    assert_eq!(output["data"]["active_handle"]["kind"], "workbook_ref");
}

#[test]
fn compose_workbook_returns_workbook_ref_for_multiple_sources() {
    let summary_result_ref = create_datetime_result_ref_for_cli();
    let request = json!({
        "tool": "compose_workbook",
        "args": {
            "worksheets": [
                {
                    "sheet_name": "Summary",
                    "source": {
                        "result_ref": summary_result_ref
                    }
                },
                {
                    "sheet_name": "Sales",
                    "source": {
                        "path": "tests/fixtures/basic-sales.xlsx",
                        "sheet": "Sales"
                    }
                }
            ]
        }
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "ok");
    // 2026-03-22: 这里锁定 compose_workbook 会把多来源快照装成 workbook_ref，目的是给后续真正导出提供稳定句柄。
    assert_eq!(output["data"]["sheet_count"], 2);
    assert_eq!(output["data"]["sheet_names"], json!(["Summary", "Sales"]));
    assert!(
        output["data"]["workbook_ref"]
            .as_str()
            .unwrap()
            .starts_with("workbook_")
    );
}

#[test]
fn export_excel_workbook_writes_multiple_sheets_from_workbook_ref() {
    let output_path = create_test_output_path("export_excel_workbook_cli", "xlsx");
    let formatted_request = json!({
        "tool": "format_table_for_export",
        "args": {
            "path": "tests/fixtures/basic-sales.xlsx",
            "sheet": "Sales",
            "column_order": ["region", "sales", "user_id"],
            "rename_mappings": [
                {
                    "from": "region",
                    "to": "区域"
                },
                {
                    "from": "sales",
                    "to": "销售额"
                },
                {
                    "from": "user_id",
                    "to": "客户ID"
                }
            ],
            "drop_unspecified_columns": true
        }
    });
    let formatted_output = run_cli_with_json(&formatted_request.to_string());
    assert_eq!(formatted_output["status"], "ok");
    let formatted_result_ref = formatted_output["data"]["result_ref"]
        .as_str()
        .unwrap()
        .to_string();

    let compose_request = json!({
        "tool": "compose_workbook",
        "args": {
            "worksheets": [
                {
                    "sheet_name": "交付报表",
                    "source": {
                        "result_ref": formatted_result_ref
                    }
                },
                {
                    "sheet_name": "原始数据",
                    "source": {
                        "path": "tests/fixtures/basic-sales.xlsx",
                        "sheet": "Sales"
                    }
                }
            ]
        }
    });
    let compose_output = run_cli_with_json(&compose_request.to_string());
    assert_eq!(compose_output["status"], "ok");
    let workbook_ref = compose_output["data"]["workbook_ref"].as_str().unwrap();

    let export_request = json!({
        "tool": "export_excel_workbook",
        "args": {
            "workbook_ref": workbook_ref,
            "output_path": output_path.to_string_lossy()
        }
    });
    let export_output = run_cli_with_json(&export_request.to_string());

    assert_eq!(export_output["status"], "ok");
    assert!(std::fs::exists(&output_path).unwrap());
    let mut workbook = open_workbook_auto(&output_path).unwrap();
    let report = workbook.worksheet_range("交付报表").unwrap();
    let raw = workbook.worksheet_range("原始数据").unwrap();

    // 2026-03-22: 这里锁定 compose -> export 的完整 CLI 链路，目的是保证用户能把整理结果和原表一起交付成标准多 Sheet 工作簿。
    assert_eq!(report.get((0, 0)).unwrap().to_string(), "区域");
    assert_eq!(report.get((1, 1)).unwrap().to_string(), "120");
    assert_eq!(raw.get((0, 0)).unwrap().to_string(), "user_id");
    assert_eq!(raw.get((1, 2)).unwrap().to_string(), "120");
}

#[test]
fn export_excel_workbook_sets_explicit_column_widths_for_delivery_tables() {
    let output_path = create_test_output_path("export_excel_workbook_widths_cli", "xlsx");
    let formatted_request = json!({
        "tool": "format_table_for_export",
        "args": {
            "path": "tests/fixtures/basic-sales.xlsx",
            "sheet": "Sales",
            "column_order": ["user_id", "region"],
            "rename_mappings": [
                {
                    "from": "user_id",
                    "to": "客户唯一识别编号"
                },
                {
                    "from": "region",
                    "to": "所属区域名称"
                }
            ],
            "drop_unspecified_columns": true
        }
    });
    let formatted_output = run_cli_with_json(&formatted_request.to_string());
    assert_eq!(formatted_output["status"], "ok");
    let formatted_result_ref = formatted_output["data"]["result_ref"]
        .as_str()
        .unwrap()
        .to_string();

    let compose_request = json!({
        "tool": "compose_workbook",
        "args": {
            "worksheets": [
                {
                    "sheet_name": "交付报表",
                    "source": {
                        "result_ref": formatted_result_ref
                    }
                }
            ]
        }
    });
    let compose_output = run_cli_with_json(&compose_request.to_string());
    assert_eq!(compose_output["status"], "ok");
    let workbook_ref = compose_output["data"]["workbook_ref"].as_str().unwrap();

    let export_request = json!({
        "tool": "export_excel_workbook",
        "args": {
            "workbook_ref": workbook_ref,
            "output_path": output_path.to_string_lossy()
        }
    });
    let export_output = run_cli_with_json(&export_request.to_string());
    assert_eq!(export_output["status"], "ok");

    let sheet_xml = read_zip_entry_text(&output_path, "xl/worksheets/sheet1.xml");

    // 2026-03-24: 这里先锁定交付型 workbook 会显式写出列宽，原因是结果交付层如果一直靠 Excel 默认宽度，长列名和中文字段会明显难看；目的是把“可交付”往“可直接给客户看”推进一步。
    assert!(sheet_xml.contains("<cols>"));
    assert!(sheet_xml.contains("customWidth=\"1\""));
}

#[test]
fn report_delivery_export_freezes_title_and_header_rows() {
    let output_path = create_test_output_path("report_delivery_freeze_panes_cli", "xlsx");
    let summary_result_ref = create_ascii_summary_result_ref_for_cli();
    let analysis_result_ref = create_chart_analysis_result_ref_for_cli();
    let report_request = json!({
        "tool": "report_delivery",
        "args": {
            "report_name": "Freeze Delivery",
            "report_subtitle": "Frozen Header",
            "summary": {
                "sheet_name": "Summary",
                "source": {
                    "result_ref": summary_result_ref
                }
            },
            "analysis": {
                "sheet_name": "Analysis",
                "source": {
                    "result_ref": analysis_result_ref
                }
            },
            "include_chart_sheet": false
        }
    });
    let report_output = run_cli_with_json(&report_request.to_string());
    assert_eq!(report_output["status"], "ok");
    let workbook_ref = report_output["data"]["workbook_ref"].as_str().unwrap();

    let export_request = json!({
        "tool": "export_excel_workbook",
        "args": {
            "workbook_ref": workbook_ref,
            "output_path": output_path.to_string_lossy()
        }
    });
    let export_output = run_cli_with_json(&export_request.to_string());
    assert_eq!(export_output["status"], "ok");

    let sheet_xml = read_zip_entry_text(&output_path, "xl/worksheets/sheet1.xml");

    // 2026-03-24: 这里先锁定带标题区的 report_delivery 页会冻结到首个数据行之前，原因是客户滚动台账时不能丢掉标题和表头；目的是把交付表从“能看”推进到“更适合直接使用”。
    assert!(sheet_xml.contains("state=\"frozen\""));
    assert!(sheet_xml.contains("ySplit=\"3\""));
    assert!(sheet_xml.contains("topLeftCell=\"A4\""));
}

#[test]
fn export_excel_workbook_adds_autofilter_to_header_row() {
    let output_path = create_test_output_path("export_excel_workbook_autofilter_cli", "xlsx");
    let summary_result_ref = create_ascii_summary_result_ref_for_cli();
    let request = json!({
        "tool": "compose_workbook",
        "args": {
            "worksheets": [
                {
                    "sheet_name": "Summary",
                    "source": {
                        "result_ref": summary_result_ref
                    }
                }
            ]
        }
    });
    let compose_output = run_cli_with_json(&request.to_string());
    assert_eq!(compose_output["status"], "ok");
    let workbook_ref = compose_output["data"]["workbook_ref"].as_str().unwrap();

    let export_request = json!({
        "tool": "export_excel_workbook",
        "args": {
            "workbook_ref": workbook_ref,
            "output_path": output_path.to_string_lossy()
        }
    });
    let export_output = run_cli_with_json(&export_request.to_string());
    assert_eq!(export_output["status"], "ok");

    let sheet_xml = read_zip_entry_text(&output_path, "xl/worksheets/sheet1.xml");

    // 2026-03-24: 这里先锁定导出的数据表会带 Excel 自动筛选，原因是业务用户直接在成品表里筛选字段是高频动作；目的是进一步减少“导出后还要手工点筛选”的额外操作。
    assert!(sheet_xml.contains("<autoFilter "));
    assert!(sheet_xml.contains("ref=\"A1:B3\""));
}

#[test]
fn export_excel_workbook_writes_default_number_format_for_floats() {
    let output_path = create_test_output_path("export_excel_workbook_number_format_cli", "xlsx");
    let result_ref = create_delivery_format_result_ref_for_cli();
    let request = json!({
        "tool": "compose_workbook",
        "args": {
            "worksheets": [
                {
                    "sheet_name": "Delivery",
                    "source": {
                        "result_ref": result_ref
                    }
                }
            ]
        }
    });
    let compose_output = run_cli_with_json(&request.to_string());
    assert_eq!(compose_output["status"], "ok");
    let workbook_ref = compose_output["data"]["workbook_ref"].as_str().unwrap();

    let export_request = json!({
        "tool": "export_excel_workbook",
        "args": {
            "workbook_ref": workbook_ref,
            "output_path": output_path.to_string_lossy()
        }
    });
    let export_output = run_cli_with_json(&export_request.to_string());
    assert_eq!(export_output["status"], "ok");

    let styles_xml = read_zip_entry_text(&output_path, "xl/styles.xml");

    // 2026-03-24: 这里先锁定导出的浮点数会写默认数值格式，原因是业务用户看金额时需要更接近报表习惯的两位小数；目的是避免数值虽然是 number，但展示仍像原始技术值。
    assert!(styles_xml.contains("#,##0.00"));
}

#[test]
fn export_excel_workbook_wraps_long_text_cells() {
    let output_path = create_test_output_path("export_excel_workbook_wrap_cli", "xlsx");
    let result_ref = create_delivery_format_result_ref_for_cli();
    let request = json!({
        "tool": "compose_workbook",
        "args": {
            "worksheets": [
                {
                    "sheet_name": "Delivery",
                    "source": {
                        "result_ref": result_ref
                    }
                }
            ]
        }
    });
    let compose_output = run_cli_with_json(&request.to_string());
    assert_eq!(compose_output["status"], "ok");
    let workbook_ref = compose_output["data"]["workbook_ref"].as_str().unwrap();

    let export_request = json!({
        "tool": "export_excel_workbook",
        "args": {
            "workbook_ref": workbook_ref,
            "output_path": output_path.to_string_lossy()
        }
    });
    let export_output = run_cli_with_json(&export_request.to_string());
    assert_eq!(export_output["status"], "ok");

    let styles_xml = read_zip_entry_text(&output_path, "xl/styles.xml");

    // 2026-03-24: 这里先锁定长文本列会启用换行样式，原因是交付表里说明列经常很长；目的是减少客户打开文件后手工拖宽或逐列改格式的操作。
    assert!(styles_xml.contains("wrapText=\"1\""));
}

#[test]
fn join_tables_accepts_nested_table_ref_and_result_ref_inputs() {
    let confirm_request = json!({
        "tool": "apply_header_schema",
        "args": {
            "path": "tests/fixtures/join-customers.xlsx",
            "sheet": "Customers"
        }
    });
    let confirm_output = run_cli_with_json(&confirm_request.to_string());
    assert_eq!(confirm_output["status"], "ok");
    let table_ref = confirm_output["data"]["table_ref"]
        .as_str()
        .unwrap()
        .to_string();

    let store = thread_result_ref_store();
    let right_result_ref = ResultRefStore::create_result_ref();
    // 2026-03-22: 这里先手工准备右侧 result_ref，目的是锁定 join_tables 能消费嵌套的中间结果句柄而不是只认 path+sheet。
    let right_dataframe = DataFrame::new(vec![
        Series::new("user_id".into(), ["1", "2", "4"]).into(),
        Series::new("amount".into(), ["100", "80", "60"]).into(),
    ])
    .unwrap();
    let right_record = PersistedResultDataset::from_dataframe(
        &right_result_ref,
        "top_n",
        vec!["result_seed_orders".to_string()],
        &right_dataframe,
    )
    .unwrap();
    store.save(&right_record).unwrap();

    let join_request = json!({
        "tool": "join_tables",
        "args": {
            "left": {
                "table_ref": table_ref
            },
            "right": {
                "result_ref": right_result_ref
            },
            "left_on": "user_id",
            "right_on": "user_id",
            "keep_mode": "matched_only",
            "limit": 5
        }
    });

    let join_output = run_cli_with_json(&join_request.to_string());
    assert_eq!(join_output["status"], "ok");
    assert_eq!(join_output["data"]["row_count"], 2);
    let joined_result_ref = join_output["data"]["result_ref"].as_str().unwrap();

    let joined_record = store.load(joined_result_ref).unwrap();
    // 2026-03-22: 这里校验新结果会同时记住左右两边来源，目的是给后续链式解释和血缘展示打底。
    assert!(
        joined_record
            .source_refs
            .iter()
            .any(|item| item == &table_ref)
    );
    assert!(
        joined_record
            .source_refs
            .iter()
            .any(|item| item == &right_result_ref)
    );
}

#[test]
fn append_tables_accepts_nested_result_ref_and_path_inputs() {
    let store = thread_result_ref_store();
    let top_result_ref = ResultRefStore::create_result_ref();
    // 2026-03-22: 这里先手工准备上侧 result_ref，目的是锁定 append_tables 也能消费嵌套来源句柄。
    let top_dataframe = DataFrame::new(vec![
        Series::new("user_id".into(), ["1", "2"]).into(),
        Series::new("region".into(), ["East", "West"]).into(),
        Series::new("sales".into(), ["120", "90"]).into(),
    ])
    .unwrap();
    let top_record = PersistedResultDataset::from_dataframe(
        &top_result_ref,
        "group_and_aggregate",
        vec!["result_seed_sales".to_string()],
        &top_dataframe,
    )
    .unwrap();
    store.save(&top_record).unwrap();

    let append_request = json!({
        "tool": "append_tables",
        "args": {
            "top": {
                "result_ref": top_result_ref
            },
            "bottom": {
                "path": "tests/fixtures/append-sales-b.xlsx",
                "sheet": "Sales"
            },
            "limit": 5
        }
    });

    let append_output = run_cli_with_json(&append_request.to_string());
    assert_eq!(append_output["status"], "ok");
    assert_eq!(append_output["data"]["row_count"], 4);
    let appended_result_ref = append_output["data"]["result_ref"].as_str().unwrap();

    let appended_record = store.load(appended_result_ref).unwrap();
    // 2026-03-22: 这里校验追加结果会保留上游 result_ref 和原始工作表来源，目的是把多表血缘闭环补完整。
    assert!(
        appended_record
            .source_refs
            .iter()
            .any(|item| item == &top_result_ref)
    );
    assert!(
        appended_record
            .source_refs
            .iter()
            .any(|item| item == "tests/fixtures/append-sales-b.xlsx#Sales")
    );
}

#[test]
fn export_csv_accepts_table_ref_directly() {
    let output_path = create_test_output_path("export_csv_table_ref", "csv");
    let confirm_request = json!({
        "tool": "apply_header_schema",
        "args": {
            "path": "tests/fixtures/title-gap-header.xlsx",
            "sheet": "Sheet1"
        }
    });
    let confirm_output = run_cli_with_json(&confirm_request.to_string());
    assert_eq!(confirm_output["status"], "ok");
    let table_ref = confirm_output["data"]["table_ref"].as_str().unwrap();

    let export_request = json!({
        "tool": "export_csv",
        "args": {
            "table_ref": table_ref,
            "output_path": output_path.to_string_lossy()
        }
    });

    let export_output = run_cli_with_json(&export_request.to_string());
    assert_eq!(export_output["status"], "ok");
    let csv_text = std::fs::read_to_string(&output_path).unwrap();
    // 2026-03-22: 这里锁定 confirmed table_ref 也能直接导出，目的是去掉“必须先转 result_ref 才能交付”的多余步骤。
    assert!(csv_text.contains("user_id,sales"));
    assert!(csv_text.contains("1,120"));
}

#[test]
fn export_excel_accepts_path_and_sheet_directly() {
    let output_path = create_test_output_path("export_excel_path_sheet", "xlsx");
    let export_request = json!({
        "tool": "export_excel",
        "args": {
            "path": "tests/fixtures/basic-sales.xlsx",
            "sheet": "Sales",
            "output_path": output_path.to_string_lossy(),
            "sheet_name": "DirectExport"
        }
    });

    let export_output = run_cli_with_json(&export_request.to_string());
    assert_eq!(export_output["status"], "ok");

    let mut workbook = open_workbook_auto(&output_path).unwrap();
    let range = workbook.worksheet_range("DirectExport").unwrap();
    // 2026-03-22: 这里锁定导出入口对原始 path+sheet 也开放，目的是让用户能从最直接的来源一步导出。
    assert_eq!(range.get((0, 0)).unwrap().to_string(), "user_id");
    assert_eq!(range.get((1, 1)).unwrap().to_string(), "East");
}

#[test]
fn export_csv_escapes_quotes_commas_and_newlines() {
    let store = thread_result_ref_store();
    let result_ref = ResultRefStore::create_result_ref();
    let output_path = create_test_output_path("export_csv_escape", "csv");
    // 2026-03-22: 这里构造包含逗号、引号和换行的值，目的是先锁定 CSV 交付在真实业务文本下不会破坏格式。
    let dataframe = DataFrame::new(vec![
        Series::new("customer".into(), ["ACME, Inc.", "Line\nBreak"]).into(),
        Series::new("note".into(), ["said \"hello\"", "plain"]).into(),
    ])
    .unwrap();
    let record = PersistedResultDataset::from_dataframe(
        &result_ref,
        "derive_columns",
        vec!["result_seed_escape".to_string()],
        &dataframe,
    )
    .unwrap();
    store.save(&record).unwrap();

    let request = json!({
        "tool": "export_csv",
        "args": {
            "result_ref": result_ref,
            "output_path": output_path.to_string_lossy()
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    let csv_text = std::fs::read_to_string(&output_path).unwrap();
    assert!(csv_text.contains("\"ACME, Inc.\""));
    assert!(csv_text.contains("\"said \"\"hello\"\"\""));
    assert!(csv_text.contains("\"Line\nBreak\""));
}

#[test]
fn append_tables_accepts_reordered_columns_when_names_match() {
    let request = json!({
        "tool": "append_tables",
        "args": {
            "top": {
                "path": "tests/fixtures/append-sales-a.xlsx",
                "sheet": "Sales"
            },
            "bottom": {
                "path": "tests/fixtures/append-sales-reordered.xlsx",
                "sheet": "Sales"
            },
            "limit": 4
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["row_count"], 4);
    // 2026-03-21: 这里校验 CLI 返回列顺序稳定，目的是让上层 Skill 可以继续按首表 schema 串接后续 Tool。
    assert_eq!(
        output["data"]["columns"],
        json!(["user_id", "region", "sales"])
    );
    // 2026-03-21: 这里校验重排列后的返回行，目的是确保 Tool 层也遵循“按列名对齐追加”的新语义。
    assert_eq!(output["data"]["rows"][2]["user_id"], "3");
    assert_eq!(output["data"]["rows"][2]["region"], "North");
    assert_eq!(output["data"]["rows"][2]["sales"], "90");
}

#[test]
fn filter_rows_returns_matching_rows_for_target_sheet() {
    let request = json!({
        "tool": "filter_rows",
        "args": {
            "path": "tests/fixtures/basic-sales.xlsx",
            "sheet": "Sales",
            "conditions": [
                {
                    "column": "region",
                    "operator": "equals",
                    "value": "East"
                }
            ]
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["row_count"], 1);
    assert_eq!(output["data"]["rows"][0]["region"], "East");
}

#[test]
fn analyze_table_returns_dual_layer_payload() {
    let request = json!({
        "tool": "analyze_table",
        "args": {
            "path": "tests/fixtures/basic-sales.xlsx",
            "sheet": "Sales",
            "casts": [
                {
                    "column": "sales",
                    "target_type": "int64"
                }
            ],
            "top_k": 2
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    // 2026-03-21: 这里先锁定桥接 Tool 的最小契约，目的是保证后续诊断逻辑扩展时外部协议保持稳定。
    assert_eq!(output["data"]["row_count"], 2);
    assert_eq!(output["data"]["column_count"], 3);
    assert!(output["data"]["table_health"].is_object());
    assert!(output["data"]["structured_findings"].is_array());
    assert!(output["data"]["human_summary"].is_object());
    // 2026-03-21: 这里先要求人类摘要至少包含总评，目的是让问答界面从第一版开始就能直接展示。
    assert!(output["data"]["human_summary"]["overall"].is_string());
}

#[test]
fn analyze_table_reports_quality_risks_in_human_summary() {
    let request = json!({
        "tool": "analyze_table",
        "args": {
            "path": "tests/fixtures/analyze-quality.xlsx",
            "sheet": "Profile",
            "top_k": 3
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    // 2026-03-21: 这里锁定高缺失列和全空列会进入结构化 finding，目的是让 Skill 能据此继续编排清洗动作。
    assert!(
        output["data"]["structured_findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["code"] == "high_missing_rate" && finding["column"] == "phone")
    );
    assert!(
        output["data"]["structured_findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["code"] == "all_missing" && finding["column"] == "notes")
    );
    // 2026-03-21: 这里锁定中文摘要会提到质量风险，目的是保证非 IT 用户直接读结果也能知道先做什么。
    assert_eq!(output["data"]["table_health"]["level"], "risky");
    assert!(
        output["data"]["human_summary"]["overall"]
            .as_str()
            .unwrap()
            .contains("先清洗")
    );
}

#[test]
fn analyze_table_detects_duplicate_rows_and_key_risks_in_cli() {
    let request = json!({
        "tool": "analyze_table",
        "args": {
            "path": "tests/fixtures/analyze-keys.xlsx",
            "sheet": "Orders",
            "top_k": 3
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    // 2026-03-21: 这里锁定重复行和候选键风险会进入结构化 finding，目的是让 Skill 能继续推动去重或主键确认。
    assert!(
        output["data"]["structured_findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["code"] == "duplicate_rows")
    );
    assert!(
        output["data"]["structured_findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["code"] == "duplicate_candidate_key"
                && finding["column"] == "user_id")
    );
    assert!(
        output["data"]["structured_findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["code"] == "blank_candidate_key"
                && finding["column"] == "user_id")
    );
}

#[test]
fn analyze_table_detects_distribution_risks_in_cli() {
    let request = json!({
        "tool": "analyze_table",
        "args": {
            "path": "tests/fixtures/analyze-distribution.xlsx",
            "sheet": "Metrics",
            "casts": [
                {
                    "column": "zero_metric",
                    "target_type": "int64"
                },
                {
                    "column": "amount",
                    "target_type": "int64"
                }
            ],
            "top_k": 3
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    // 2026-03-21: 这里锁定类别失衡、零值占比和异常值 finding，目的是让上层可以直接提示用户重点核查分布问题。
    assert!(
        output["data"]["structured_findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["code"] == "high_category_imbalance"
                && finding["column"] == "region")
    );
    assert!(
        output["data"]["structured_findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["code"] == "high_zero_ratio"
                && finding["column"] == "zero_metric")
    );
    assert!(
        output["data"]["structured_findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["code"] == "outlier_suspected" && finding["column"] == "amount")
    );
}

#[test]
fn analyze_table_generates_readable_human_summary() {
    let request = json!({
        "tool": "analyze_table",
        "args": {
            "path": "tests/fixtures/analyze-distribution.xlsx",
            "sheet": "Metrics",
            "casts": [
                {
                    "column": "zero_metric",
                    "target_type": "int64"
                },
                {
                    "column": "amount",
                    "target_type": "int64"
                }
            ],
            "top_k": 3
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    // 2026-03-21: 这里锁定人类摘要的固定结构，目的是让终端问答界面可以稳定展示总结区。
    assert!(output["data"]["human_summary"]["overall"].is_string());
    assert!(output["data"]["human_summary"]["major_issues"].is_array());
    assert!(output["data"]["human_summary"]["quick_insights"].is_array());
    assert!(output["data"]["human_summary"]["recommended_next_step"].is_string());
    // 2026-03-21: 这里锁定下一步建议数组，目的是让 Skill 可以直接把诊断结果转成后续动作建议。
    assert!(output["data"]["next_actions"].is_array());
}

#[test]
fn analyze_table_returns_business_observations_in_cli() {
    let request = json!({
        "tool": "analyze_table",
        "args": {
            "path": "tests/fixtures/analyze-distribution.xlsx",
            "sheet": "Metrics",
            "casts": [
                {
                    "column": "zero_metric",
                    "target_type": "int64"
                },
                {
                    "column": "amount",
                    "target_type": "int64"
                }
            ],
            "top_k": 3
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    // 2026-03-21: 这里锁定 CLI 会返回独立 business_observations 数组，目的是让问答编排层不必从 human_summary 里反解析业务提示。
    assert!(output["data"]["business_observations"].is_array());
    assert!(
        output["data"]["business_observations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|observation| observation["type"] == "top_category"
                && observation["column"] == "region")
    );
    // 2026-03-21: 这里补数值范围观察断言，目的是确保桥接层能同时输出少量业务统计和质量诊断。
    assert!(
        output["data"]["business_observations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|observation| observation["type"] == "numeric_range"
                && observation["column"] == "amount")
    );
}

#[test]
fn analyze_table_compresses_major_issues_and_sorts_findings_in_cli() {
    let request = json!({
        "tool": "analyze_table",
        "args": {
            "path": "tests/fixtures/analyze-quality.xlsx",
            "sheet": "Profile",
            "top_k": 5
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    // 2026-03-21: 这里锁定高优先级 finding 会稳定排前，目的是让问答层优先看到真正阻塞分析的问题。
    assert_eq!(
        output["data"]["structured_findings"][0]["code"],
        "all_missing"
    );
    assert_eq!(output["data"]["structured_findings"][0]["column"], "notes");
    // 2026-03-21: 这里锁定摘要主问题会压缩同列重复提示，目的是避免 CLI 结果里 phone 连续重复轰炸用户。
    let phone_issue_count = output["data"]["human_summary"]["major_issues"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|message| message.as_str().unwrap().contains("phone"))
        .count();
    assert_eq!(phone_issue_count, 1);
    // 2026-03-21: 这里锁定 notes 不会再被误报成候选键，目的是修掉列名 contains(\"no\") 的假阳性。
    assert!(!output["data"]["structured_findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| finding["code"] == "blank_candidate_key" && finding["column"] == "notes"));
}

#[test]
fn analyze_table_returns_extended_business_observations_in_cli() {
    let request = json!({
        "tool": "analyze_table",
        "args": {
            "path": "tests/fixtures/analyze-distribution.xlsx",
            "sheet": "Metrics",
            "casts": [
                {
                    "column": "zero_metric",
                    "target_type": "int64"
                },
                {
                    "column": "amount",
                    "target_type": "int64"
                }
            ],
            "top_k": 5
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    // 2026-03-21: 这里锁定 dominant_dimension 会独立返回，目的是让上层直接拿到“主分布维度”观察。
    assert!(
        output["data"]["business_observations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|observation| observation["type"] == "dominant_dimension"
                && observation["column"] == "region")
    );
    // 2026-03-21: 这里锁定中心统计观察会独立返回，目的是给后续分析建模层一个轻量中心统计桥接。
    assert!(
        output["data"]["business_observations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|observation| {
                (observation["type"] == "numeric_center" || observation["type"] == "median_center")
                    && observation["column"] == "amount"
            })
    );
}

#[test]
fn analyze_table_uses_median_center_in_cli_for_skewed_column() {
    let request = json!({
        "tool": "analyze_table",
        "args": {
            "path": "tests/fixtures/analyze-distribution.xlsx",
            "sheet": "Metrics",
            "casts": [
                {
                    "column": "amount",
                    "target_type": "int64"
                }
            ],
            "columns": ["amount"],
            "top_k": 5
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    // 2026-03-21: 这里锁定偏态数值列在 CLI 层会输出 median_center，目的是让问答界面也拿到稳健中心统计。
    assert!(
        output["data"]["business_observations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|observation| observation["type"] == "median_center"
                && observation["column"] == "amount"
                && observation["message"].as_str().unwrap().contains("2"))
    );
    // 2026-03-21: 这里锁定同一列不会再保留 numeric_center，目的是避免两套中心统计并存。
    assert!(
        !output["data"]["business_observations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|observation| observation["type"] == "numeric_center"
                && observation["column"] == "amount")
    );
}

#[test]
fn analyze_table_returns_date_time_and_amount_observations_in_cli() {
    let request = json!({
        "tool": "analyze_table",
        "args": {
            "path": "tests/fixtures/analyze-observation-enhancement.xlsx",
            "sheet": "Orders",
            "casts": [
                {
                    "column": "amount",
                    "target_type": "int64"
                }
            ],
            "top_k": 12
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    // 2026-03-21: 这里锁定真实 Excel 日期范围观察，目的是保证问答界面面对日期列时能直接展示覆盖周期。
    assert!(
        output["data"]["business_observations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|observation| observation["type"] == "date_range"
                && observation["column"] == "order_date"
                && observation["message"]
                    .as_str()
                    .unwrap()
                    .contains("2026-03-01")
                && observation["message"]
                    .as_str()
                    .unwrap()
                    .contains("2026-04-01"))
    );
    // 2026-03-21: 这里锁定日期集中观察，目的是让用户直观看到记录主要集中在哪个时间段。
    assert!(
        output["data"]["business_observations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|observation| observation["type"] == "date_concentration"
                && observation["column"] == "order_date"
                && observation["message"].as_str().unwrap().contains("2026-03"))
    );
    // 2026-03-21: 这里锁定时间高峰观察，目的是让真实 Excel 场景也能返回“上午/下午”这种直白提示。
    assert!(
        output["data"]["business_observations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|observation| observation["type"] == "time_peak_period"
                && observation["column"] == "order_time"
                && observation["message"].as_str().unwrap().contains("下午"))
    );
    // 2026-03-21: 这里锁定金额典型区间与负金额观察，目的是让业务用户先看懂金额大致分布和异常含义。
    assert!(
        output["data"]["business_observations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|observation| observation["type"] == "amount_typical_band"
                && observation["column"] == "amount"
                && observation["message"].as_str().unwrap().contains("20")
                && observation["message"].as_str().unwrap().contains("40"))
    );
    assert!(
        output["data"]["business_observations"]
            .as_array()
            .unwrap()
            .iter()
            .any(
                |observation| observation["type"] == "amount_negative_presence"
                    && observation["column"] == "amount"
            )
    );
    // 2026-03-21: 这里锁定金额长尾观察会进入 quick_insights，目的是让终端摘要优先展示更有业务解释力的信号。
    assert!(
        output["data"]["human_summary"]["quick_insights"]
            .as_array()
            .unwrap()
            .iter()
            .any(|message| message.as_str().unwrap().contains("amount")
                && message.as_str().unwrap().contains("平均值"))
    );
}

#[test]
fn tool_catalog_includes_linear_regression() {
    let mut cmd = Command::cargo_bin("excel_skill").unwrap();
    let assert = cmd.assert().success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    // 2026-03-21: 这里锁定能力目录包含 linear_regression，目的是让问答界面能发现分析建模层的新能力。
    assert!(
        json["data"]["tool_catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool == "linear_regression")
    );
}

#[test]
fn linear_regression_returns_model_payload_in_cli() {
    let request = json!({
        "tool": "linear_regression",
        "args": {
            "path": "tests/fixtures/group-sales.xlsx",
            "sheet": "Sales",
            "features": ["user_id"],
            "target": "sales",
            "casts": [
                {
                    "column": "user_id",
                    "target_type": "int64"
                },
                {
                    "column": "sales",
                    "target_type": "int64"
                }
            ],
            "intercept": true,
            "missing_strategy": "drop_rows"
        }
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "ok");
    // 2026-03-21: 这里锁定结构化模型结果字段，目的是保证 Skill 可以稳定读取回归结果而不是解析自然语言。
    assert_eq!(output["data"]["model_kind"], "linear_regression");
    assert_eq!(output["data"]["problem_type"], "regression");
    assert_eq!(output["data"]["data_summary"]["feature_count"], 1);
    assert_eq!(
        output["data"]["quality_summary"]["primary_metric"]["name"],
        "r2"
    );
    assert_eq!(output["data"]["features"], json!(["user_id"]));
    assert_eq!(output["data"]["target"], "sales");
    assert_eq!(output["data"]["coefficients"][0]["feature"], "user_id");
    assert!(
        output["data"]["coefficients"][0]["value"]
            .as_f64()
            .is_some()
    );
    assert!(output["data"]["intercept"].as_f64().is_some());
    assert!(output["data"]["r2"].as_f64().is_some());
    assert_eq!(output["data"]["row_count_used"], 4);
    assert_eq!(output["data"]["dropped_rows"], 0);
    assert!(output["data"]["assumptions"].as_array().unwrap().len() >= 2);
    assert!(
        output["data"]["human_summary"]["overall"]
            .as_str()
            .unwrap()
            .contains("有效样本")
    );
}

#[test]
fn linear_regression_reports_validation_errors_in_cli() {
    let request = json!({
        "tool": "linear_regression",
        "args": {
            "path": "tests/fixtures/basic-sales.xlsx",
            "sheet": "Sales",
            "features": ["user_id"],
            "target": "region",
            "casts": [
                {
                    "column": "user_id",
                    "target_type": "int64"
                }
            ],
            "missing_strategy": "drop_rows"
        }
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "error");
    // 2026-03-21: 这里锁定直白错误文案，目的是让低 IT 用户在 CLI/问答入口也能立刻知道不能拿文本列做线性回归。
    assert!(
        output["error"]
            .as_str()
            .unwrap()
            .contains("目标列 `region` 不是数值列")
    );
}

#[test]
fn tool_catalog_includes_logistic_regression() {
    let mut cmd = Command::cargo_bin("excel_skill").unwrap();
    let assert = cmd.assert().success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    // 2026-03-21: 这里锁定能力目录包含 logistic_regression，目的是让问答界面能发现二分类建模能力。
    assert!(
        json["data"]["tool_catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool == "logistic_regression")
    );
}

#[test]
fn logistic_regression_returns_model_payload_in_cli() {
    let request = json!({
        "tool": "logistic_regression",
        "args": {
            "path": "tests/fixtures/group-sales.xlsx",
            "sheet": "Sales",
            "features": ["user_id"],
            "target": "region",
            "casts": [
                {
                    "column": "user_id",
                    "target_type": "int64"
                }
            ],
            "positive_label": "West",
            "intercept": true,
            "missing_strategy": "drop_rows"
        }
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "ok");
    // 2026-03-21: 这里锁定逻辑回归结构化输出字段，目的是保证 Skill 可以稳定读取分类模型结果。
    assert_eq!(output["data"]["model_kind"], "logistic_regression");
    assert_eq!(output["data"]["problem_type"], "classification");
    assert_eq!(output["data"]["data_summary"]["feature_count"], 1);
    assert_eq!(
        output["data"]["quality_summary"]["primary_metric"]["name"],
        "training_accuracy"
    );
    assert_eq!(output["data"]["features"], json!(["user_id"]));
    assert_eq!(output["data"]["target"], "region");
    assert_eq!(output["data"]["positive_label"], "West");
    assert_eq!(output["data"]["coefficients"][0]["feature"], "user_id");
    assert!(
        output["data"]["coefficients"][0]["value"]
            .as_f64()
            .is_some()
    );
    assert!(output["data"]["intercept"].as_f64().is_some());
    assert_eq!(output["data"]["row_count_used"], 4);
    assert_eq!(output["data"]["dropped_rows"], 0);
    assert_eq!(output["data"]["class_balance"]["positive_count"], 2);
    assert_eq!(output["data"]["class_balance"]["negative_count"], 2);
    assert!(output["data"]["training_accuracy"].as_f64().unwrap() >= 0.99);
    assert!(
        output["data"]["assumptions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item.as_str().unwrap().contains("不做 AUC"))
    );
    assert!(
        output["data"]["human_summary"]["overall"]
            .as_str()
            .unwrap()
            .contains("有效样本")
    );
}

#[test]
fn logistic_regression_reports_non_binary_target_in_cli() {
    let request = json!({
        "tool": "logistic_regression",
        "args": {
            "path": "tests/fixtures/group-sales.xlsx",
            "sheet": "Sales",
            "features": ["user_id"],
            "target": "sales",
            "casts": [
                {
                    "column": "user_id",
                    "target_type": "int64"
                },
                {
                    "column": "sales",
                    "target_type": "int64"
                }
            ],
            "missing_strategy": "drop_rows"
        }
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "error");
    // 2026-03-21: 这里锁定多取值目标列会被明确拦截，目的是避免用户误把连续值列拿去做逻辑回归。
    assert!(output["error"].as_str().unwrap().contains("只支持二分类"));
}

#[test]
fn logistic_regression_reports_single_class_target_with_actionable_guidance() {
    let request = json!({
        "tool": "logistic_regression",
        "args": {
            "path": "tests/fixtures/model-single-class.xlsx",
            "sheet": "Customers",
            "features": ["score"],
            "target": "is_churn",
            "casts": [
                {
                    "column": "score",
                    "target_type": "float64"
                }
            ],
            "positive_label": "yes",
            "missing_strategy": "drop_rows"
        }
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "error");
    // 2026-03-22: 这里锁定单一类别目标列会返回可执行的中文引导，目的是让低 IT 用户知道下一步该先检查目标列分布或更换目标列。
    let error = output["error"].as_str().unwrap();
    assert!(error.contains("只有一个类别"));
    assert!(error.contains("先看目标列分布") || error.contains("更换目标列"));
}

#[test]
fn tool_catalog_includes_cluster_kmeans() {
    let mut cmd = Command::cargo_bin("excel_skill").unwrap();
    let assert = cmd.assert().success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    // 2026-03-21: 这里锁定能力目录包含 cluster_kmeans，目的是让问答界面能够发现聚类能力已经进入分析建模层。
    assert!(
        json["data"]["tool_catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool == "cluster_kmeans")
    );
}

#[test]
fn cluster_kmeans_returns_model_payload_in_cli() {
    let request = json!({
        "tool": "cluster_kmeans",
        "args": {
            "path": "tests/fixtures/group-sales.xlsx",
            "sheet": "Sales",
            "features": ["user_id", "sales"],
            "casts": [
                {
                    "column": "user_id",
                    "target_type": "int64"
                },
                {
                    "column": "sales",
                    "target_type": "int64"
                }
            ],
            "cluster_count": 2,
            "max_iterations": 50,
            "missing_strategy": "drop_rows"
        }
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "ok");
    // 2026-03-21: 这里锁定聚类 Tool 的统一建模输出，目的是让 Skill 后续可以和回归/分类复用同一解析入口。
    assert_eq!(output["data"]["model_kind"], "cluster_kmeans");
    assert_eq!(output["data"]["problem_type"], "clustering");
    assert_eq!(output["data"]["features"], json!(["user_id", "sales"]));
    assert_eq!(output["data"]["cluster_count"], 2);
    assert_eq!(output["data"]["row_count_used"], 4);
    assert_eq!(output["data"]["dropped_rows"], 0);
    assert_eq!(output["data"]["data_summary"]["feature_count"], 2);
    assert_eq!(
        output["data"]["quality_summary"]["primary_metric"]["name"],
        "inertia"
    );
    assert_eq!(output["data"]["cluster_sizes"].as_array().unwrap().len(), 2);
    assert_eq!(
        output["data"]["cluster_centers"].as_array().unwrap().len(),
        2
    );
    assert_eq!(output["data"]["assignments"].as_array().unwrap().len(), 4);
}

#[test]
fn cluster_kmeans_reports_invalid_cluster_count_in_cli() {
    let request = json!({
        "tool": "cluster_kmeans",
        "args": {
            "path": "tests/fixtures/group-sales.xlsx",
            "sheet": "Sales",
            "features": ["user_id", "sales"],
            "casts": [
                {
                    "column": "user_id",
                    "target_type": "int64"
                },
                {
                    "column": "sales",
                    "target_type": "int64"
                }
            ],
            "cluster_count": 8,
            "missing_strategy": "drop_rows"
        }
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "error");
    // 2026-03-21: 这里锁定 K 值错误会透传直白中文，目的是让终端问答入口也能清楚告诉用户为什么不能聚类。
    assert!(output["error"].as_str().unwrap().contains("分组数"));
}

#[test]
fn tool_catalog_includes_decision_assistant() {
    let mut cmd = Command::cargo_bin("excel_skill").unwrap();
    let assert = cmd.assert().success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    // 2026-03-21: 这里锁定能力目录包含 decision_assistant，目的是让问答界面发现决策助手层 V1 已可用。
    assert!(
        json["data"]["tool_catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool == "decision_assistant")
    );
}

#[test]
fn decision_assistant_returns_prioritized_actions_in_cli() {
    let request = json!({
        "tool": "decision_assistant",
        "args": {
            "path": "tests/fixtures/analyze-distribution.xlsx",
            "sheet": "Metrics",
            "casts": [
                {
                    "column": "zero_metric",
                    "target_type": "int64"
                },
                {
                    "column": "amount",
                    "target_type": "int64"
                }
            ],
            "top_k": 5
        }
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "ok");
    // 2026-03-21: 这里锁定决策助手的双层输出结构，目的是让用户既能看到阻塞问题，也能看到下一步工具建议。
    assert_eq!(output["data"]["assistant_kind"], "quality_diagnostic");
    assert!(output["data"]["blocking_risks"].is_array());
    assert!(output["data"]["priority_actions"].is_array());
    assert!(output["data"]["business_highlights"].is_array());
    assert!(output["data"]["next_tool_suggestions"].is_array());
    assert!(output["data"]["human_summary"].is_object());
    // 2026-03-21: 这里锁定在有两个数值列时会建议聚类，目的是让决策助手真正桥接到新加的聚类 Tool。
    assert!(
        output["data"]["next_tool_suggestions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|suggestion| suggestion["tool"] == "cluster_kmeans")
    );
}

#[test]
fn tool_catalog_includes_suggest_table_links() {
    let mut cmd = Command::cargo_bin("excel_skill").unwrap();
    let assert = cmd.assert().success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    // 2026-03-21: 这里锁定能力目录包含 suggest_table_links，目的是让上层 Skill 能发现 V2 多表工作流入口。
    assert!(
        json["data"]["tool_catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool == "suggest_table_links")
    );
}

#[test]
fn suggest_table_links_returns_join_candidate_in_cli() {
    let request = json!({
        "tool": "suggest_table_links",
        "args": {
            "left": {
                "path": "tests/fixtures/join-customers.xlsx",
                "sheet": "Customers"
            },
            "right": {
                "path": "tests/fixtures/join-orders.xlsx",
                "sheet": "Orders"
            },
            "max_candidates": 3
        }
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "ok");
    // 2026-03-21: 这里锁定 CLI 层会返回显性的 user_id 关联候选，目的是让问答界面能直接引导用户确认关联。
    assert_eq!(output["data"]["candidates"][0]["left_column"], "user_id");
    assert_eq!(output["data"]["candidates"][0]["right_column"], "user_id");
    assert_eq!(output["data"]["candidates"][0]["confidence"], "high");
    assert!(
        output["data"]["candidates"][0]["question"]
            .as_str()
            .unwrap()
            .contains("是否用")
    );
    assert_eq!(
        output["data"]["candidates"][0]["keep_mode_options"][0]["keep_mode"],
        "matched_only"
    );
    assert!(
        output["data"]["recommended_next_step"]
            .as_str()
            .unwrap()
            .contains("join_preflight")
    );
}

#[test]
fn tool_catalog_includes_normalize_text_columns_and_rename_columns() {
    let mut cmd = Command::cargo_bin("excel_skill").unwrap();
    let assert = cmd.assert().success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    // 2026-03-23: 这里先锁定目录中暴露新 Tool，目的是保证上层 Skill 能发现文本标准化与列重命名能力。
    assert!(
        json["data"]["tool_catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool == "normalize_text_columns")
    );
    assert!(
        json["data"]["tool_catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool == "rename_columns")
    );
}

#[test]
fn normalize_text_columns_returns_result_ref_with_cleaned_preview() {
    let request = json!({
        "tool": "normalize_text_columns",
        "args": {
            "path": "tests/fixtures/basic-sales.xlsx",
            "sheet": "Sales",
            "rules": [
                {
                    "column": "region",
                    "trim": true,
                    "collapse_whitespace": true,
                    "lowercase": true,
                    "replace_pairs": [
                        {
                            "from": "east",
                            "to": "east_zone"
                        }
                    ]
                }
            ]
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-23: 这里锁定 CLI 层会返回清洗后的预览和 result_ref，目的是打通后续链式分析入口。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["rows"][0]["region"], "east_zone");
    assert_eq!(output["data"]["rows"][1]["region"], "west");
    assert!(
        output["data"]["result_ref"]
            .as_str()
            .unwrap()
            .starts_with("result_")
    );
}

#[test]
fn normalize_text_columns_rejects_duplicate_rules_in_cli() {
    let request = json!({
        "tool": "normalize_text_columns",
        "args": {
            "path": "tests/fixtures/basic-sales.xlsx",
            "sheet": "Sales",
            "rules": [
                {
                    "column": "region",
                    "trim": true
                },
                {
                    "column": "region",
                    "lowercase": true
                }
            ]
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-23: 这里锁定 CLI 层会把重复列规则显式报错，目的是避免参数二义性直接落到生产链路。
    assert_eq!(output["status"], "error");
    assert!(output["error"].as_str().unwrap().contains("region"));
}

#[test]
fn rename_columns_returns_result_ref_with_renamed_columns() {
    let request = json!({
        "tool": "rename_columns",
        "args": {
            "path": "tests/fixtures/basic-sales.xlsx",
            "sheet": "Sales",
            "mappings": [
                {
                    "from": "user_id",
                    "to": "customer_id"
                },
                {
                    "from": "sales",
                    "to": "revenue"
                }
            ]
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-23: 这里锁定 CLI 层会返回新的列结构，目的是让 Skill 可直接复用改名后的 result_ref。
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["columns"],
        json!(["customer_id", "region", "revenue"])
    );
    assert_eq!(output["data"]["rows"][0]["customer_id"], "1");
    assert_eq!(output["data"]["rows"][1]["revenue"], "95");
}

#[test]
fn rename_columns_reports_missing_source_column_in_cli() {
    let request = json!({
        "tool": "rename_columns",
        "args": {
            "path": "tests/fixtures/basic-sales.xlsx",
            "sheet": "Sales",
            "mappings": [
                {
                    "from": "missing_col",
                    "to": "target"
                }
            ]
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-23: 这里锁定 CLI 层缺列报错，目的是让上层尽早暴露字段口径问题。
    assert_eq!(output["status"], "error");
    assert!(output["error"].as_str().unwrap().contains("missing_col"));
}

#[test]
fn tool_catalog_includes_fill_missing_from_lookup_and_pivot_table() {
    let mut cmd = Command::cargo_bin("excel_skill").unwrap();
    let assert = cmd.assert().success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    // 2026-03-23: 这里锁定目录里已经暴露 lookup 回填与透视能力，目的是让上层 Skill 能发现第二批基础 Tool。
    assert!(
        json["data"]["tool_catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool == "fill_missing_from_lookup")
    );
    assert!(
        json["data"]["tool_catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool == "pivot_table")
    );
}

#[test]
fn tool_catalog_includes_parse_datetime_columns() {
    let mut cmd = Command::cargo_bin("excel_skill").unwrap();
    let assert = cmd.assert().success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    // 2026-03-23: 这里锁定目录暴露日期时间标准化入口，目的是让上层 Skill 能发现第二批基础 Tool。
    assert!(
        json["data"]["tool_catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool == "parse_datetime_columns")
    );
}

#[test]
fn tool_catalog_includes_lookup_values() {
    let mut cmd = Command::cargo_bin("excel_skill").unwrap();
    let assert = cmd.assert().success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    // 2026-03-23: 这里锁定目录暴露轻量查值入口，目的是让上层 Skill 能发现 VLOOKUP/XLOOKUP 心智对应的基础 Tool。
    assert!(
        json["data"]["tool_catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool == "lookup_values")
    );
}

#[test]
fn tool_catalog_includes_window_calculation() {
    let mut cmd = Command::cargo_bin("excel_skill").unwrap();
    let assert = cmd.assert().success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    // 2026-03-23: 这里锁定目录暴露窗口计算入口，目的是让上层 Skill 能发现分析桥接层的关键 Tool。
    assert!(
        json["data"]["tool_catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool == "window_calculation")
    );
}

#[test]
fn fill_missing_from_lookup_accepts_mixed_table_ref_and_result_ref_sources() {
    let workbook_path = create_test_workbook(
        "fill_lookup_cli",
        "fill-lookup.xlsx",
        &[
            (
                "Base",
                vec![vec!["user_id", "city"], vec!["1", ""], vec!["2", "Urumqi"]],
            ),
            (
                "Lookup",
                vec![
                    vec!["user_id", "city"],
                    vec!["1", "Beijing"],
                    vec!["2", "Shanghai"],
                ],
            ),
        ],
    );

    let confirm_request = json!({
        "tool": "apply_header_schema",
        "args": {
            "path": workbook_path.to_string_lossy(),
            "sheet": "Base"
        }
    });
    let confirm_output = run_cli_with_json(&confirm_request.to_string());
    assert_eq!(confirm_output["status"], "ok");
    let table_ref = confirm_output["data"]["table_ref"]
        .as_str()
        .unwrap()
        .to_string();

    let lookup_request = json!({
        "tool": "select_columns",
        "args": {
            "path": workbook_path.to_string_lossy(),
            "sheet": "Lookup",
            "columns": ["user_id", "city"]
        }
    });
    let lookup_output = run_cli_with_json(&lookup_request.to_string());
    assert_eq!(lookup_output["status"], "ok");
    let result_ref = lookup_output["data"]["result_ref"]
        .as_str()
        .unwrap()
        .to_string();

    let fill_request = json!({
        "tool": "fill_missing_from_lookup",
        "args": {
            "base": {
                "table_ref": table_ref
            },
            "lookup": {
                "result_ref": result_ref
            },
            "base_on": "user_id",
            "lookup_on": "user_id",
            "fills": [
                {
                    "base_column": "city",
                    "lookup_column": "city"
                }
            ]
        }
    });

    let output = run_cli_with_json(&fill_request.to_string());

    // 2026-03-23: 这里锁定 mixed source 模式可以直接回填并返回 result_ref，目的是增强多步链式场景稳定性。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["rows"][0]["city"], "Beijing");
    assert_eq!(output["data"]["rows"][1]["city"], "Urumqi");
    assert!(
        output["data"]["result_ref"]
            .as_str()
            .unwrap()
            .starts_with("result_")
    );
}

#[test]
fn fill_missing_from_lookup_accepts_composite_keys_in_cli() {
    let workbook_path = create_test_workbook(
        "fill_lookup_composite_cli",
        "fill-lookup-composite.xlsx",
        &[
            (
                "Base",
                vec![
                    vec!["customer_id", "month", "city", "tier"],
                    vec!["1", "2026-01", "", ""],
                    vec!["1", "2026-02", "", ""],
                    vec!["2", "2026-01", "Urumqi", "A"],
                ],
            ),
            (
                "Lookup",
                vec![
                    vec!["customer_id", "month", "city", "tier"],
                    vec!["1", "2026-01", "Beijing", "A"],
                    vec!["1", "2026-02", "Shanghai", "B"],
                    vec!["2", "2026-01", "Shenzhen", "C"],
                ],
            ),
        ],
    );

    let confirm_request = json!({
        "tool": "apply_header_schema",
        "args": {
            "path": workbook_path.to_string_lossy(),
            "sheet": "Base"
        }
    });
    let confirm_output = run_cli_with_json(&confirm_request.to_string());
    assert_eq!(confirm_output["status"], "ok");
    let table_ref = confirm_output["data"]["table_ref"]
        .as_str()
        .unwrap()
        .to_string();

    let lookup_request = json!({
        "tool": "select_columns",
        "args": {
            "path": workbook_path.to_string_lossy(),
            "sheet": "Lookup",
            "columns": ["customer_id", "month", "city", "tier"]
        }
    });
    let lookup_output = run_cli_with_json(&lookup_request.to_string());
    assert_eq!(lookup_output["status"], "ok");
    let result_ref = lookup_output["data"]["result_ref"]
        .as_str()
        .unwrap()
        .to_string();

    let fill_request = json!({
        "tool": "fill_missing_from_lookup",
        "args": {
            "base": {
                "table_ref": table_ref
            },
            "lookup": {
                "result_ref": result_ref
            },
            "base_keys": ["customer_id", "month"],
            "lookup_keys": ["customer_id", "month"],
            "fills": [
                {
                    "base_column": "city",
                    "lookup_column": "city"
                },
                {
                    "base_column": "tier",
                    "lookup_column": "tier"
                }
            ]
        }
    });

    let output = run_cli_with_json(&fill_request.to_string());

    // 2026-03-23: 这里锁定 CLI 层能按“客户 + 月份”复合键回填，目的是把真实业务补主数据场景接到问答链路。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["rows"][0]["city"], "Beijing");
    assert_eq!(output["data"]["rows"][0]["tier"], "A");
    assert_eq!(output["data"]["rows"][1]["city"], "Shanghai");
    assert_eq!(output["data"]["rows"][1]["tier"], "B");
    assert_eq!(output["data"]["rows"][2]["city"], "Urumqi");
}

#[test]
fn pivot_table_supports_sum_aggregation_with_casts() {
    let request = json!({
        "tool": "pivot_table",
        "args": {
            "path": "tests/fixtures/group-sales.xlsx",
            "sheet": "Sales",
            "rows": ["region"],
            "columns": ["user_id"],
            "values": ["sales"],
            "aggregation": "sum",
            "casts": [
                {
                    "column": "sales",
                    "target_type": "int64"
                }
            ]
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-23: 这里锁定 pivot 在 CLI 层可直接透视并返回宽表预览，目的是把 Excel 用户熟悉的能力正式接入 Tool 层。
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["columns"],
        json!(["region", "1", "2", "3", "4"])
    );
    assert_eq!(output["data"]["rows"][0]["region"], "East");
    assert_eq!(output["data"]["rows"][0]["1"], "120");
    assert_eq!(output["data"]["rows"][0]["2"], "80");
    assert_eq!(output["data"]["rows"][1]["region"], "West");
    assert_eq!(output["data"]["rows"][1]["3"], "90");
    assert_eq!(output["data"]["rows"][1]["4"], "60");
}

#[test]
fn pivot_table_rejects_multiple_values_columns_in_cli() {
    let request = json!({
        "tool": "pivot_table",
        "args": {
            "path": "tests/fixtures/group-sales.xlsx",
            "sheet": "Sales",
            "rows": ["region"],
            "columns": ["user_id"],
            "values": ["sales", "user_id"],
            "aggregation": "count"
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-23: 这里锁定第一版 pivot 的多 values 限制，目的是防止用户误以为已经支持复杂透视。
    assert_eq!(output["status"], "error");
    assert!(output["error"].as_str().unwrap().contains("单个 values"));
}

#[test]
fn pivot_table_export_cli_writes_blank_cells_and_numeric_values() {
    let pivot_request = json!({
        "tool": "pivot_table",
        "args": {
            "path": "tests/fixtures/group-sales.xlsx",
            "sheet": "Sales",
            "rows": ["region"],
            "columns": ["user_id"],
            "values": ["sales"],
            "aggregation": "sum",
            "casts": [
                {
                    "column": "sales",
                    "target_type": "float64"
                }
            ]
        }
    });

    let pivot_output = run_cli_with_json(&pivot_request.to_string());

    // 2026-03-23: 这里先锁定 CLI 预览里缺失透视格要回空字符串，原因是以后用户看结果时不该再看到 null 文本。
    assert_eq!(pivot_output["status"], "ok");
    assert_eq!(pivot_output["data"]["rows"][0]["3"], "");

    let result_ref = pivot_output["data"]["result_ref"].as_str().unwrap();
    let output_path = create_test_output_path("pivot_table_export_cli", "xlsx");
    let export_request = json!({
        "tool": "export_excel",
        "args": {
            "result_ref": result_ref,
            "output_path": output_path.to_string_lossy(),
            "sheet_name": "Pivot"
        }
    });

    let export_output = run_cli_with_json(&export_request.to_string());
    assert_eq!(export_output["status"], "ok");

    let mut workbook = open_workbook_auto(&output_path).unwrap();
    let range = workbook.worksheet_range("Pivot").unwrap();
    let east_user3 = range.get((1, 3));
    let east_user1 = range.get((1, 1)).unwrap();
    let west_user3 = range.get((2, 3)).unwrap();

    // 2026-03-23: 这里锁定导出后的缺失值为空白、数值为 number，原因是用户需要把导出的表继续拿去做 Excel 统计。
    assert!(east_user3.is_none() || matches!(east_user3, Some(Data::Empty)));
    assert!(matches!(east_user1, Data::Float(value) if (*value - 120.0).abs() < 1e-9));
    assert!(matches!(west_user3, Data::Float(value) if (*value - 90.0).abs() < 1e-9));
}

#[test]
fn parse_datetime_columns_returns_result_ref_with_normalized_preview() {
    let request = json!({
        "tool": "parse_datetime_columns",
        "args": {
            "result_ref": create_datetime_result_ref_for_cli(),
            "rules": [
                {
                    "column": "biz_date",
                    "target_type": "date"
                },
                {
                    "column": "created_at",
                    "target_type": "datetime"
                }
            ]
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-23: 这里锁定 CLI 层日期时间标准化输出，目的是让后续窗口和趋势分析可直接消费统一时间口径。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["rows"][0]["biz_date"], "2026-03-01");
    assert_eq!(
        output["data"]["rows"][1]["created_at"],
        "2026-03-02 09:15:20"
    );
    assert!(
        output["data"]["result_ref"]
            .as_str()
            .unwrap()
            .starts_with("result_")
    );
}

#[test]
fn lookup_values_accepts_mixed_table_ref_and_result_ref_sources() {
    let workbook_path = create_test_workbook(
        "lookup_values_cli",
        "lookup-values.xlsx",
        &[
            (
                "Base",
                vec![
                    vec!["user_id", "amount"],
                    vec!["1", "120"],
                    vec!["2", "95"],
                    vec!["9", "88"],
                ],
            ),
            (
                "Lookup",
                vec![
                    vec!["user_id", "city", "tier"],
                    vec!["1", "Beijing", "A"],
                    vec!["2", "Shanghai", "B"],
                ],
            ),
        ],
    );

    let confirm_request = json!({
        "tool": "apply_header_schema",
        "args": {
            "path": workbook_path.to_string_lossy(),
            "sheet": "Base"
        }
    });
    let confirm_output = run_cli_with_json(&confirm_request.to_string());
    assert_eq!(confirm_output["status"], "ok");
    let table_ref = confirm_output["data"]["table_ref"]
        .as_str()
        .unwrap()
        .to_string();

    let lookup_request = json!({
        "tool": "select_columns",
        "args": {
            "path": workbook_path.to_string_lossy(),
            "sheet": "Lookup",
            "columns": ["user_id", "city", "tier"]
        }
    });
    let lookup_output = run_cli_with_json(&lookup_request.to_string());
    assert_eq!(lookup_output["status"], "ok");
    let result_ref = lookup_output["data"]["result_ref"]
        .as_str()
        .unwrap()
        .to_string();

    let request = json!({
        "tool": "lookup_values",
        "args": {
            "base": {
                "table_ref": table_ref
            },
            "lookup": {
                "result_ref": result_ref
            },
            "base_on": "user_id",
            "lookup_on": "user_id",
            "selects": [
                {
                    "lookup_column": "city",
                    "output_column": "customer_city"
                },
                {
                    "lookup_column": "tier",
                    "output_column": "customer_tier"
                }
            ]
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-23: 这里锁定 mixed source 轻量查值链路，目的是验证主表不变行、查值列可直接回传并生成 result_ref。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["rows"][0]["customer_city"], "Beijing");
    assert_eq!(output["data"]["rows"][1]["customer_tier"], "B");
    assert_eq!(output["data"]["rows"][2]["customer_city"], "");
    assert!(
        output["data"]["result_ref"]
            .as_str()
            .unwrap()
            .starts_with("result_")
    );
}

#[test]
fn lookup_values_accepts_composite_keys_in_cli() {
    let workbook_path = create_test_workbook(
        "lookup_values_composite_cli",
        "lookup-values-composite.xlsx",
        &[
            (
                "Base",
                vec![
                    vec!["customer_id", "month", "amount"],
                    vec!["1", "2026-01", "120"],
                    vec!["1", "2026-02", "95"],
                    vec!["2", "2026-01", "88"],
                ],
            ),
            (
                "Lookup",
                vec![
                    vec!["customer_id", "month", "city", "tier"],
                    vec!["1", "2026-01", "Beijing", "A"],
                    vec!["1", "2026-02", "Shanghai", "B"],
                    vec!["2", "2026-01", "Shenzhen", "C"],
                ],
            ),
        ],
    );

    let confirm_request = json!({
        "tool": "apply_header_schema",
        "args": {
            "path": workbook_path.to_string_lossy(),
            "sheet": "Base"
        }
    });
    let confirm_output = run_cli_with_json(&confirm_request.to_string());
    assert_eq!(confirm_output["status"], "ok");
    let table_ref = confirm_output["data"]["table_ref"]
        .as_str()
        .unwrap()
        .to_string();

    let lookup_request = json!({
        "tool": "select_columns",
        "args": {
            "path": workbook_path.to_string_lossy(),
            "sheet": "Lookup",
            "columns": ["customer_id", "month", "city", "tier"]
        }
    });
    let lookup_output = run_cli_with_json(&lookup_request.to_string());
    assert_eq!(lookup_output["status"], "ok");
    let result_ref = lookup_output["data"]["result_ref"]
        .as_str()
        .unwrap()
        .to_string();

    let request = json!({
        "tool": "lookup_values",
        "args": {
            "base": {
                "table_ref": table_ref
            },
            "lookup": {
                "result_ref": result_ref
            },
            "base_keys": ["customer_id", "month"],
            "lookup_keys": ["customer_id", "month"],
            "selects": [
                {
                    "lookup_column": "city",
                    "output_column": "customer_city"
                },
                {
                    "lookup_column": "tier",
                    "output_column": "customer_tier"
                }
            ]
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-23: 这里锁定 CLI 层能按“客户 + 月份”复合键带列，目的是让多期经营分析不再被单键限制住。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["rows"][0]["customer_city"], "Beijing");
    assert_eq!(output["data"]["rows"][1]["customer_city"], "Shanghai");
    assert_eq!(output["data"]["rows"][2]["customer_city"], "Shenzhen");
    assert_eq!(output["data"]["rows"][1]["customer_tier"], "B");
}

#[test]
fn window_calculation_returns_result_ref_with_partitioned_metrics() {
    let request = json!({
        "tool": "window_calculation",
        "args": {
            "path": "tests/fixtures/group-sales.xlsx",
            "sheet": "Sales",
            "partition_by": ["region"],
            "order_by": [
                {
                    "column": "sales",
                    "descending": true
                }
            ],
            "calculations": [
                {
                    "kind": "row_number",
                    "output_column": "row_number"
                },
                {
                    "kind": "rank",
                    "output_column": "dense_rank"
                },
                {
                    "kind": "cumulative_sum",
                    "source_column": "sales",
                    "output_column": "running_sales"
                }
            ],
            "casts": [
                {
                    "column": "sales",
                    "target_type": "int64"
                }
            ]
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-23: 这里锁定 CLI 层窗口计算输出，目的是验证排序、分组和累计指标能一并返回并生成 result_ref。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["rows"][0]["region"], "East");
    assert_eq!(output["data"]["rows"][0]["row_number"], "1");
    assert_eq!(output["data"]["rows"][0]["dense_rank"], "1");
    assert_eq!(output["data"]["rows"][0]["running_sales"], "120");
    assert_eq!(output["data"]["rows"][1]["row_number"], "2");
    assert_eq!(output["data"]["rows"][1]["running_sales"], "200");
    assert!(
        output["data"]["result_ref"]
            .as_str()
            .unwrap()
            .starts_with("result_")
    );
}

#[test]
fn window_calculation_supports_shift_percent_rank_and_rolling_metrics_in_cli() {
    let workbook_path = create_test_workbook(
        "window_advanced_cli",
        "window-advanced.xlsx",
        &[(
            "Sales",
            vec![
                vec!["region", "biz_date", "sales", "customer"],
                vec!["East", "2026-01-03", "80", "C"],
                vec!["East", "2026-01-01", "100", "A"],
                vec!["East", "2026-01-02", "100", "B"],
                vec!["West", "2026-01-01", "60", "W"],
            ],
        )],
    );
    let request = json!({
        "tool": "window_calculation",
        "args": {
            "path": workbook_path.to_string_lossy(),
            "sheet": "Sales",
            "partition_by": ["region"],
            "order_by": [
                {
                    "column": "biz_date",
                    "descending": false
                }
            ],
            "calculations": [
                {
                    "kind": "lag",
                    "source_column": "customer",
                    "output_column": "prev_customer",
                    "offset": 1
                },
                {
                    "kind": "lead",
                    "source_column": "customer",
                    "output_column": "next_customer",
                    "offset": 1
                },
                {
                    "kind": "percent_rank",
                    "output_column": "percent_rank"
                },
                {
                    "kind": "rolling_sum",
                    "source_column": "sales",
                    "output_column": "rolling_sales_2",
                    "window_size": 2
                },
                {
                    "kind": "rolling_mean",
                    "source_column": "sales",
                    "output_column": "rolling_mean_2",
                    "window_size": 2
                }
            ],
            "casts": [
                {
                    "column": "sales",
                    "target_type": "int64"
                }
            ]
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-23: 这里锁定 CLI 能直达增强窗口能力，目的是让 Skill 在不下沉实现细节的前提下直接复用桥接层分析结果。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["rows"][0]["prev_customer"], "B");
    assert_eq!(output["data"]["rows"][0]["next_customer"], "");
    assert_eq!(output["data"]["rows"][0]["percent_rank"], "1");
    assert_eq!(output["data"]["rows"][0]["rolling_sales_2"], "180");
    assert_eq!(output["data"]["rows"][0]["rolling_mean_2"], "90");

    assert_eq!(output["data"]["rows"][1]["prev_customer"], "");
    assert_eq!(output["data"]["rows"][1]["next_customer"], "B");
    assert_eq!(output["data"]["rows"][1]["percent_rank"], "0");
    assert_eq!(output["data"]["rows"][1]["rolling_sales_2"], "100");
    assert_eq!(output["data"]["rows"][1]["rolling_mean_2"], "100");
}

#[test]
fn suggest_table_links_accepts_nested_table_ref_and_result_ref_inputs() {
    let confirm_request = json!({
        "tool": "apply_header_schema",
        "args": {
            "path": "tests/fixtures/join-customers.xlsx",
            "sheet": "Customers"
        }
    });
    let confirm_output = run_cli_with_json(&confirm_request.to_string());
    assert_eq!(confirm_output["status"], "ok");
    let table_ref = confirm_output["data"]["table_ref"]
        .as_str()
        .unwrap()
        .to_string();

    let store = thread_result_ref_store();
    let result_ref = ResultRefStore::create_result_ref();
    // 2026-03-23: 这里手工构造右侧 result_ref，目的是先锁定关系建议层也能直接消费中间结果句柄。
    let dataframe = DataFrame::new(vec![
        Series::new("user_id".into(), ["1", "2", "4"]).into(),
        Series::new("amount".into(), ["100", "80", "60"]).into(),
    ])
    .unwrap();
    let record = PersistedResultDataset::from_dataframe(
        &result_ref,
        "group_and_aggregate",
        vec!["result_seed_orders".to_string()],
        &dataframe,
    )
    .unwrap();
    store.save(&record).unwrap();

    let request = json!({
        "tool": "suggest_table_links",
        "args": {
            "left": {
                "table_ref": table_ref
            },
            "right": {
                "result_ref": result_ref
            },
            "max_candidates": 3
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["candidates"][0]["left_column"], "user_id");
    assert_eq!(output["data"]["candidates"][0]["right_column"], "user_id");
}

#[test]
fn tool_catalog_includes_suggest_table_workflow() {
    let mut cmd = Command::cargo_bin("excel_skill").unwrap();
    let assert = cmd.assert().success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    // 2026-03-22: 这里锁定能力目录包含 suggest_table_workflow，目的是让 Skill 能发现多表“先判断动作”入口。
    assert!(
        json["data"]["tool_catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool == "suggest_table_workflow")
    );
}

#[test]
fn suggest_table_workflow_recommends_append_in_cli() {
    let request = json!({
        "tool": "suggest_table_workflow",
        "args": {
            "left": {
                "path": "tests/fixtures/append-sales-a.xlsx",
                "sheet": "Sales"
            },
            "right": {
                "path": "tests/fixtures/append-sales-reordered.xlsx",
                "sheet": "Sales"
            },
            "max_link_candidates": 3
        }
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "ok");
    // 2026-03-22: 这里锁定 CLI 层会优先推荐追加动作，目的是让问答界面先问“是否上下拼接”而不是误导去关联。
    assert_eq!(output["data"]["recommended_action"], "append_tables");
    assert_eq!(output["data"]["append_candidate"]["confidence"], "high");
    assert!(
        output["data"]["append_candidate"]["question"]
            .as_str()
            .unwrap()
            .contains("追加")
    );
    assert_eq!(
        output["data"]["suggested_tool_call"]["tool"],
        "append_tables"
    );
    assert_eq!(
        output["data"]["suggested_tool_call"]["args"]["top"]["sheet"],
        "Sales"
    );
}

#[test]
fn suggest_table_workflow_recommends_join_in_cli() {
    let request = json!({
        "tool": "suggest_table_workflow",
        "args": {
            "left": {
                "path": "tests/fixtures/join-customers.xlsx",
                "sheet": "Customers"
            },
            "right": {
                "path": "tests/fixtures/join-orders.xlsx",
                "sheet": "Orders"
            },
            "max_link_candidates": 3
        }
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "ok");
    // 2026-03-22: 这里锁定 CLI 层会把显性关联作为下一步动作推荐，目的是让上层直接承接 join 确认问题。
    assert_eq!(output["data"]["recommended_action"], "join_tables");
    assert_eq!(
        output["data"]["link_candidates"][0]["left_column"],
        "user_id"
    );
    assert_eq!(
        output["data"]["link_candidates"][0]["right_column"],
        "user_id"
    );
    assert_eq!(output["data"]["suggested_tool_call"]["tool"], "join_tables");
    assert_eq!(
        output["data"]["suggested_tool_call"]["args"]["left_on"],
        "user_id"
    );
}

#[test]
fn suggest_table_workflow_preserves_nested_source_payloads_in_tool_call() {
    let confirm_request = json!({
        "tool": "apply_header_schema",
        "args": {
            "path": "tests/fixtures/join-customers.xlsx",
            "sheet": "Customers"
        }
    });
    let confirm_output = run_cli_with_json(&confirm_request.to_string());
    assert_eq!(confirm_output["status"], "ok");
    let table_ref = confirm_output["data"]["table_ref"]
        .as_str()
        .unwrap()
        .to_string();

    let store = thread_result_ref_store();
    let result_ref = ResultRefStore::create_result_ref();
    // 2026-03-23: 这里手工准备右侧 result_ref，目的是锁定工作流建议层返回的建议调用不会把句柄退化回 path+sheet。
    let dataframe = DataFrame::new(vec![
        Series::new("user_id".into(), ["1", "2", "4"]).into(),
        Series::new("amount".into(), ["100", "80", "60"]).into(),
    ])
    .unwrap();
    let record = PersistedResultDataset::from_dataframe(
        &result_ref,
        "top_n",
        vec!["result_seed_orders".to_string()],
        &dataframe,
    )
    .unwrap();
    store.save(&record).unwrap();

    let request = json!({
        "tool": "suggest_table_workflow",
        "args": {
            "left": {
                "table_ref": table_ref
            },
            "right": {
                "result_ref": result_ref
            },
            "max_link_candidates": 3
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["recommended_action"], "join_tables");
    assert_eq!(
        output["data"]["suggested_tool_call"]["args"]["left"]["table_ref"],
        table_ref
    );
    assert_eq!(
        output["data"]["suggested_tool_call"]["args"]["right"]["result_ref"],
        result_ref
    );
}

#[test]
fn tool_catalog_includes_suggest_multi_table_plan() {
    let mut cmd = Command::cargo_bin("excel_skill").unwrap();
    let assert = cmd.assert().success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    // 2026-03-22: 这里锁定能力目录包含 suggest_multi_table_plan，目的是让 Skill 能发现多表顺序建议入口。
    assert!(
        json["data"]["tool_catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool == "suggest_multi_table_plan")
    );
}

#[test]
fn tool_catalog_includes_execute_suggested_tool_call() {
    let mut cmd = Command::cargo_bin("excel_skill").unwrap();
    let assert = cmd.assert().success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert!(
        json["data"]["tool_catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool == "execute_suggested_tool_call")
    );
}

#[test]
fn suggest_multi_table_plan_builds_append_chain_in_cli() {
    let request = json!({
        "tool": "suggest_multi_table_plan",
        "args": {
            "tables": [
                {
                    "path": "tests/fixtures/append-sales-a.xlsx",
                    "sheet": "Sales",
                    "alias": "sales_a"
                },
                {
                    "path": "tests/fixtures/append-sales-b.xlsx",
                    "sheet": "Sales",
                    "alias": "sales_b"
                },
                {
                    "path": "tests/fixtures/append-sales-reordered.xlsx",
                    "sheet": "Sales",
                    "alias": "sales_c"
                }
            ],
            "max_link_candidates": 3
        }
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "ok");
    // 2026-03-22: 这里锁定 CLI 层会给出多步追加链，目的是让问答界面先按顺序合并同结构表。
    assert_eq!(output["data"]["steps"][0]["action"], "append_tables");
    assert_eq!(output["data"]["steps"][0]["input_refs"][0], "sales_a");
    assert!(
        output["data"]["steps"][0]["question"]
            .as_str()
            .unwrap()
            .contains("追加")
    );
    assert_eq!(output["data"]["steps"][1]["input_refs"][0], "step_1_result");
}

#[test]
fn suggest_multi_table_plan_builds_join_step_in_cli() {
    let request = json!({
        "tool": "suggest_multi_table_plan",
        "args": {
            "tables": [
                {
                    "path": "tests/fixtures/join-customers.xlsx",
                    "sheet": "Customers",
                    "alias": "customers"
                },
                {
                    "path": "tests/fixtures/join-orders.xlsx",
                    "sheet": "Orders",
                    "alias": "orders"
                }
            ],
            "max_link_candidates": 3
        }
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["steps"][0]["action"], "join_preflight");
    assert_eq!(output["data"]["steps"][0]["execution_status"], "ready");
    assert_eq!(
        output["data"]["steps"][0]["suggested_tool_call"]["tool"],
        "join_preflight"
    );
    assert_eq!(
        output["data"]["steps"][0]["suggested_tool_call"]["args"]["left_on"],
        "user_id"
    );
    assert_eq!(
        output["data"]["steps"][0]["suggested_tool_call"]["args"]["right_on"],
        "user_id"
    );

    assert_eq!(output["data"]["steps"][1]["action"], "join_tables");
    assert_eq!(
        output["data"]["steps"][1]["execution_status"],
        "needs_preflight_confirmation"
    );
    assert_eq!(output["data"]["steps"][1]["preflight_step_id"], "step_1");
    assert_eq!(
        output["data"]["steps"][1]["suggested_tool_call"]["tool"],
        "join_tables"
    );
    assert_eq!(
        output["data"]["steps"][1]["suggested_tool_call"]["args"]["left_on"],
        "user_id"
    );
    assert_eq!(
        output["data"]["steps"][1]["suggested_tool_call"]["args"]["right_on"],
        "user_id"
    );
    assert_eq!(output["data"]["unresolved_refs"], json!(["step_2_result"]));
}

#[test]
fn suggest_multi_table_plan_builds_append_then_join_chain_in_cli() {
    let request = json!({
        "tool": "suggest_multi_table_plan",
        "args": {
            "tables": [
                {
                    "path": "tests/fixtures/join-customers.xlsx",
                    "sheet": "Customers",
                    "alias": "customers"
                },
                {
                    "path": "tests/fixtures/append-sales-a.xlsx",
                    "sheet": "Sales",
                    "alias": "sales_a"
                },
                {
                    "path": "tests/fixtures/append-sales-b.xlsx",
                    "sheet": "Sales",
                    "alias": "sales_b"
                }
            ],
            "max_link_candidates": 3
        }
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "ok");
    // 2026-03-22: 这里锁定 CLI 混合场景也会先给追加步骤，目的是让问答界面先收口同结构批次表，再进入显性关联。
    assert_eq!(output["data"]["steps"][0]["action"], "append_tables");
    assert_eq!(
        output["data"]["steps"][0]["input_refs"],
        json!(["sales_a", "sales_b"])
    );
    assert_eq!(output["data"]["steps"][0]["result_ref"], "step_1_result");
    assert!(
        output["data"]["steps"][0]["question"]
            .as_str()
            .unwrap()
            .contains("追加")
    );
    // 2026-03-22: 这里锁定第二步会直接引用 step_1_result 做 join，目的是确保 CLI 返回的建议调用骨架可以被 Skill 原样串接执行。
    assert_eq!(output["data"]["steps"][1]["action"], "join_preflight");
    assert_eq!(
        output["data"]["steps"][1]["input_refs"],
        json!(["customers", "step_1_result"])
    );
    assert_eq!(output["data"]["steps"][1]["result_ref"], "step_2_preflight");
    assert_eq!(
        output["data"]["steps"][1]["execution_status"],
        "needs_result_bindings"
    );
    assert_eq!(
        output["data"]["steps"][1]["pending_result_bindings"][0]["alias"],
        "step_1_result"
    );
    assert_eq!(
        output["data"]["steps"][1]["pending_result_bindings"][0]["from_step_id"],
        "step_1"
    );
    assert_eq!(
        output["data"]["steps"][1]["suggested_tool_call"]["tool"],
        "join_preflight"
    );
    assert_eq!(
        output["data"]["steps"][1]["suggested_tool_call"]["args"]["left"]["path"],
        "tests/fixtures/join-customers.xlsx"
    );
    assert_eq!(
        output["data"]["steps"][1]["suggested_tool_call"]["args"]["right"]["result_ref"],
        "step_1_result"
    );
    assert_eq!(
        output["data"]["steps"][1]["suggested_tool_call"]["args"]["left_on"],
        "user_id"
    );
    assert_eq!(
        output["data"]["steps"][1]["suggested_tool_call"]["args"]["right_on"],
        "user_id"
    );

    assert_eq!(output["data"]["steps"][2]["action"], "join_tables");
    assert_eq!(
        output["data"]["steps"][2]["input_refs"],
        json!(["customers", "step_1_result"])
    );
    assert_eq!(output["data"]["steps"][2]["result_ref"], "step_3_result");
    assert_eq!(
        output["data"]["steps"][2]["execution_status"],
        "needs_preflight_confirmation_and_result_bindings"
    );
    assert_eq!(output["data"]["steps"][2]["preflight_step_id"], "step_2");
    assert_eq!(
        output["data"]["steps"][2]["suggested_tool_call"]["tool"],
        "join_tables"
    );
    assert_eq!(output["data"]["unresolved_refs"], json!(["step_3_result"]));
}

#[test]
fn suggest_multi_table_plan_join_steps_can_run_preflight_then_join_in_cli() {
    let plan_request = json!({
        "tool": "suggest_multi_table_plan",
        "args": {
            "tables": [
                {
                    "path": "tests/fixtures/join-customers.xlsx",
                    "sheet": "Customers",
                    "alias": "customers"
                },
                {
                    "path": "tests/fixtures/append-sales-a.xlsx",
                    "sheet": "Sales",
                    "alias": "sales_a"
                },
                {
                    "path": "tests/fixtures/append-sales-b.xlsx",
                    "sheet": "Sales",
                    "alias": "sales_b"
                }
            ],
            "max_link_candidates": 3
        }
    });

    let plan_output = run_cli_with_json(&plan_request.to_string());
    assert_eq!(plan_output["status"], "ok");
    assert_eq!(plan_output["data"]["steps"][0]["action"], "append_tables");
    assert_eq!(plan_output["data"]["steps"][1]["action"], "join_preflight");
    assert_eq!(plan_output["data"]["steps"][2]["action"], "join_tables");

    let append_request = json!({
        "tool": plan_output["data"]["steps"][0]["suggested_tool_call"]["tool"],
        "args": plan_output["data"]["steps"][0]["suggested_tool_call"]["args"]
    });
    let append_output = run_cli_with_json(&append_request.to_string());
    assert_eq!(append_output["status"], "ok");
    let append_result_ref = append_output["data"]["result_ref"]
        .as_str()
        .expect("append should return result_ref")
        .to_string();

    let preflight_tool = plan_output["data"]["steps"][1]["suggested_tool_call"]["tool"]
        .as_str()
        .unwrap()
        .to_string();
    let mut preflight_args = plan_output["data"]["steps"][1]["suggested_tool_call"]["args"].clone();
    preflight_args["confirm_join"] = json!(true);
    preflight_args["result_ref_bindings"] = json!({
        "step_1_result": append_result_ref
    });
    let preflight_request = json!({
        "tool": preflight_tool,
        "args": preflight_args
    });
    let preflight_output = run_cli_with_json(&preflight_request.to_string());
    assert_eq!(preflight_output["status"], "ok");
    assert_eq!(
        preflight_output["data"]["confirmed_join_tool_call"]["tool"],
        "join_tables"
    );

    let join_request = json!({
        "tool": preflight_output["data"]["confirmed_join_tool_call"]["tool"],
        "args": preflight_output["data"]["confirmed_join_tool_call"]["args"]
    });
    let join_output = run_cli_with_json(&join_request.to_string());
    assert_eq!(join_output["status"], "ok");
    assert!(join_output["data"]["row_count"].as_u64().unwrap() > 0);
}

#[test]
fn suggest_multi_table_plan_preserves_mixed_source_payloads() {
    let confirm_request = json!({
        "tool": "apply_header_schema",
        "args": {
            "path": "tests/fixtures/join-customers.xlsx",
            "sheet": "Customers"
        }
    });
    let confirm_output = run_cli_with_json(&confirm_request.to_string());
    assert_eq!(confirm_output["status"], "ok");
    let customers_table_ref = confirm_output["data"]["table_ref"]
        .as_str()
        .unwrap()
        .to_string();

    let store = thread_result_ref_store();
    let sales_result_ref = ResultRefStore::create_result_ref();
    // 2026-03-23: 这里手工准备 sales_a 的 result_ref，目的是锁定多表计划器会把原始来源类型一路保留到建议调用骨架。
    let sales_dataframe = DataFrame::new(vec![
        Series::new("region".into(), ["East", "West"]).into(),
        Series::new("user_id".into(), ["1", "2"]).into(),
        Series::new("sales".into(), ["120", "90"]).into(),
    ])
    .unwrap();
    let sales_record = PersistedResultDataset::from_dataframe(
        &sales_result_ref,
        "append_tables",
        vec!["result_seed_sales_a".to_string()],
        &sales_dataframe,
    )
    .unwrap();
    store.save(&sales_record).unwrap();

    let request = json!({
        "tool": "suggest_multi_table_plan",
        "args": {
            "tables": [
                {
                    "table_ref": customers_table_ref,
                    "alias": "customers"
                },
                {
                    "result_ref": sales_result_ref,
                    "alias": "sales_a"
                },
                {
                    "path": "tests/fixtures/append-sales-b.xlsx",
                    "sheet": "Sales",
                    "alias": "sales_b"
                }
            ],
            "max_link_candidates": 3
        }
    });

    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["steps"][0]["action"], "append_tables");
    assert_eq!(
        output["data"]["steps"][0]["suggested_tool_call"]["args"]["top"]["result_ref"],
        sales_result_ref
    );
    assert_eq!(
        output["data"]["steps"][0]["suggested_tool_call"]["args"]["bottom"]["path"],
        "tests/fixtures/append-sales-b.xlsx"
    );
    assert_eq!(output["data"]["steps"][1]["action"], "join_preflight");
    assert_eq!(
        output["data"]["steps"][1]["suggested_tool_call"]["args"]["left"]["table_ref"],
        customers_table_ref
    );
    assert_eq!(
        output["data"]["steps"][1]["suggested_tool_call"]["args"]["right"]["result_ref"],
        "step_1_result"
    );
    assert_eq!(output["data"]["steps"][2]["action"], "join_tables");
    assert_eq!(
        output["data"]["steps"][2]["suggested_tool_call"]["args"]["left"]["table_ref"],
        customers_table_ref
    );
    assert_eq!(
        output["data"]["steps"][2]["suggested_tool_call"]["args"]["right"]["result_ref"],
        "step_1_result"
    );
}

#[test]
fn execute_suggested_tool_call_can_run_plan_step_with_result_ref_bindings() {
    let plan_request = json!({
        "tool": "suggest_multi_table_plan",
        "args": {
            "tables": [
                {
                    "path": "tests/fixtures/join-customers.xlsx",
                    "sheet": "Customers",
                    "alias": "customers"
                },
                {
                    "path": "tests/fixtures/append-sales-a.xlsx",
                    "sheet": "Sales",
                    "alias": "sales_a"
                },
                {
                    "path": "tests/fixtures/append-sales-b.xlsx",
                    "sheet": "Sales",
                    "alias": "sales_b"
                }
            ],
            "max_link_candidates": 3
        }
    });
    let plan_output = run_cli_with_json(&plan_request.to_string());
    assert_eq!(plan_output["status"], "ok");

    let append_exec_request = json!({
        "tool": "execute_suggested_tool_call",
        "args": {
            "tool_call": plan_output["data"]["steps"][0]["suggested_tool_call"]
        }
    });
    let append_output = run_cli_with_json(&append_exec_request.to_string());
    assert_eq!(append_output["status"], "ok");
    let append_result_ref = append_output["data"]["result_ref"].as_str().unwrap().to_string();

    let preflight_exec_request = json!({
        "tool": "execute_suggested_tool_call",
        "args": {
            "tool_call": plan_output["data"]["steps"][1]["suggested_tool_call"],
            "result_ref_bindings": {
                "step_1_result": append_result_ref
            },
            "arg_overrides": {
                "confirm_join": true
            }
        }
    });
    let preflight_output = run_cli_with_json(&preflight_exec_request.to_string());
    assert_eq!(preflight_output["status"], "ok");
    assert_eq!(
        preflight_output["data"]["confirmed_join_tool_call"]["tool"],
        "join_tables"
    );

    let join_exec_request = json!({
        "tool": "execute_suggested_tool_call",
        "args": {
            "tool_call": preflight_output["data"]["confirmed_join_tool_call"]
        }
    });
    let join_output = run_cli_with_json(&join_exec_request.to_string());
    assert_eq!(join_output["status"], "ok");
    assert!(join_output["data"]["row_count"].as_u64().unwrap() > 0);
}

#[test]
fn open_workbook_missing_path_returns_utf8_error_message() {
    let request = json!({
        "tool": "open_workbook",
        "args": {}
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "error");
    // 2026-03-23: 这里先锁定缺少 path 时必须返回正常 UTF-8 中文，目的是防止 dispatcher 里的历史乱码继续向上层扩散。
    assert_eq!(output["error"], "open_workbook 缺少 path 参数");
}

#[test]
fn list_sheets_missing_path_returns_utf8_error_message() {
    let request = json!({
        "tool": "list_sheets",
        "args": {}
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "error");
    // 2026-03-23: ????? list_sheets ???????? UTF-8 ??????????????? I/O Tool??????????????????????
    assert_eq!(output["error"], "list_sheets 缺少 path 参数");
}

#[test]
fn load_table_region_missing_range_returns_utf8_error_message() {
    let request = json!({
        "tool": "load_table_region",
        "args": {
            "path": "tests/fixtures/basic-sales.xlsx",
            "sheet": "Sales"
        }
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "error");
    // 2026-03-23: ?????????????? range ??????????????????????????????????????????????
    assert_eq!(output["error"], "load_table_region 缺少 range 参数");
}

#[test]
fn compose_workbook_missing_worksheets_returns_utf8_error_message() {
    let request = json!({
        "tool": "compose_workbook",
        "args": {}
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "error");
    // 2026-03-23: 这里锁定 workbook 草稿入口的缺参文案，目的是保证多表导出链路的报错也保持可读中文。
    assert_eq!(output["error"], "compose_workbook 缺少 worksheets 参数");
}

#[test]
fn join_tables_missing_left_returns_utf8_error_message() {
    let request = json!({
        "tool": "join_tables",
        "args": {
            "right": {
                "path": "tests/fixtures/join-orders.xlsx",
                "sheet": "Orders"
            },
            "left_on": "user_id",
            "right_on": "user_id"
        }
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "error");
    // 2026-03-23: 这里锁定显性关联入口在缺左表来源时的中文提示，目的是让业务用户能直接补齐请求而不是面对乱码。
    assert_eq!(output["error"], "join_tables 缺少 left 参数");
}

#[test]
fn report_delivery_invalid_payload_returns_utf8_parse_error() {
    let request = json!({
        "tool": "report_delivery",
        "args": {
            "report_name": "????",
            "summary": "not-an-object"
        }
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "error");
    // 2026-03-23: ????? report_delivery ? JSON ????????? UTF-8 ??????????????? V2-P2 ???????????????????????
    assert!(
        output["error"]
            .as_str()
            .unwrap()
            .starts_with("report_delivery 参数解析失败:")
    );
}

#[test]
fn update_session_state_invalid_payload_returns_utf8_parse_error() {
    let request = json!({
        "tool": "update_session_state",
        "args": {
            "selected_columns": "not-an-array"
        }
    });

    let output = run_cli_with_json(&request.to_string());

    assert_eq!(output["status"], "error");
    // 2026-03-23: 这里锁定参数解析失败时的 UTF-8 中文前缀，目的是覆盖 from_value 失败这条常见错误路径。
    assert!(
        output["error"]
            .as_str()
            .unwrap()
            .starts_with("update_session_state 参数解析失败:")
    );
}
