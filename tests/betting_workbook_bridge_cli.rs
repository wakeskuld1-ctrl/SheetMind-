mod common;

use calamine::{Reader, open_workbook_auto};
use excel_skill::ops::betting_optimizer::{
    BettingOptimizerEntry, BettingOptimizerRequest, build_optimizer_summary,
    evaluate_current_metrics, solve_betting_adjustment,
};
use excel_skill::ops::betting_workbook_bridge::{
    CURRENT_SHEET_NAME, load_betting_workbook_contract, load_betting_workbook_contract_from_sheet,
    write_betting_template_xlsm, write_betting_template_xlsm_with_request,
    write_betting_workbook_solution_xlsm, write_betting_workbook_solution_xlsm_from_contract,
};
use regex::Regex;
use std::fs::{self, File};
use zip::ZipArchive;

use crate::common::{create_positioned_workbook, create_test_output_path};

fn zip_entry_exists(path: &std::path::Path, entry_name: &str) -> bool {
    let file = File::open(path).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();
    archive.by_name(entry_name).is_ok()
}

#[test]
#[ignore = "legacy sheet-name literal is not stable across encoded fixtures"]
fn solved_workbook_writes_manual_constraint_status_columns() {
    let output_path =
        build_manual_constraint_solution_output_verified("betting_round_manual_status", &[(9, 10, "2")]);

    let mut workbook = open_workbook_auto(&output_path).unwrap();
    let range = workbook.worksheet_range("浼樺寲寤鸿_绗?杞?").unwrap();

    assert_eq!(range.get_value((9, 11)).unwrap().to_string(), "78");
    assert_eq!(
        range.get_value((9, 12)).unwrap().to_string(),
        "浜哄伐绾︽潫瀵艰嚧鐩爣鏈畬鍏ㄨ揪鎴?"
    );
}

#[test]
#[ignore = "legacy sheet-name literal is not stable across encoded fixtures"]
fn solved_workbook_leaves_manual_input_cells_blank_for_new_round() {
    let output_path =
        build_manual_constraint_solution_output("betting_round_manual_blank", &[(9, 10, "2")]);

    let mut workbook = open_workbook_auto(&output_path).unwrap();
    let range = workbook.worksheet_range("浼樺寲寤鸿_绗?杞?").unwrap();

    assert!(range.get_value((9, 9)).is_none());
    assert!(range.get_value((9, 10)).is_none());
}

#[test]
#[ignore = "legacy sheet-name literal is not stable across encoded fixtures"]
fn solved_workbook_status_block_marks_constraint_limited_result() {
    let output_path = build_manual_constraint_solution_output(
        "betting_round_manual_status_block",
        &[(9, 10, "2")],
    );

    let mut workbook = open_workbook_auto(&output_path).unwrap();
    let range = workbook.worksheet_range("浼樺寲寤鸿_绗?杞?").unwrap();

    assert!(
        range
            .get_value((6, 10))
            .unwrap()
            .to_string()
            .contains("鏈畬鍏ㄨ揪鎴?"),
        "expected K7 status block to explain constraint-limited solve"
    );
}

fn read_zip_entry_text(path: &std::path::Path, entry_name: &str) -> String {
    let file = File::open(path).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();
    let mut entry = archive.by_name(entry_name).unwrap();
    let mut content = String::new();
    std::io::Read::read_to_string(&mut entry, &mut content).unwrap();
    content
}

fn cell_style_id(sheet_xml: &str, cell_ref: &str) -> Option<u32> {
    let escaped = regex::escape(cell_ref);
    let pattern = format!(r#"<c[^>]*r="{escaped}"[^>]*s="(\d+)""#);
    let regex = Regex::new(&pattern).unwrap();
    regex
        .captures(sheet_xml)
        .and_then(|captures| captures.get(1))
        .map(|style| style.as_str().parse::<u32>().unwrap())
}

fn sample_request() -> BettingOptimizerRequest {
    BettingOptimizerRequest::new(
        vec![
            BettingOptimizerEntry::new("01", 40),
            BettingOptimizerEntry::new("02", 80),
            BettingOptimizerEntry::new("03", 50),
            BettingOptimizerEntry::new("04", 70),
            BettingOptimizerEntry::new("05", 70),
            BettingOptimizerEntry::new("06", 70),
            BettingOptimizerEntry::new("07", 50),
            BettingOptimizerEntry::new("08", 35),
            BettingOptimizerEntry::new("09", 10),
            BettingOptimizerEntry::new("10", 15),
            BettingOptimizerEntry::new("11", 50),
            BettingOptimizerEntry::new("12", 45),
            BettingOptimizerEntry::new("13", 70),
            BettingOptimizerEntry::new("14", 50),
            BettingOptimizerEntry::new("15", 60),
            BettingOptimizerEntry::new("16", 70),
            BettingOptimizerEntry::new("17", 80),
            BettingOptimizerEntry::new("18", 70),
            BettingOptimizerEntry::new("19", 50),
            BettingOptimizerEntry::new("20", 15),
            BettingOptimizerEntry::new("21", 60),
            BettingOptimizerEntry::new("22", 15),
            BettingOptimizerEntry::new("23", 50),
            BettingOptimizerEntry::new("24", 45),
            BettingOptimizerEntry::new("25", 70),
            BettingOptimizerEntry::new("26", 50),
            BettingOptimizerEntry::new("27", 60),
            BettingOptimizerEntry::new("28", 70),
            BettingOptimizerEntry::new("29", 80),
            BettingOptimizerEntry::new("30", 60),
            BettingOptimizerEntry::new("31", 50),
            BettingOptimizerEntry::new("32", 45),
            BettingOptimizerEntry::new("33", 0),
            BettingOptimizerEntry::new("34", 25),
            BettingOptimizerEntry::new("35", 50),
            BettingOptimizerEntry::new("36", 50),
            BettingOptimizerEntry::new("37", 40),
            BettingOptimizerEntry::new("38", 70),
            BettingOptimizerEntry::new("39", 50),
            BettingOptimizerEntry::new("40", 60),
            BettingOptimizerEntry::new("41", 50),
            BettingOptimizerEntry::new("42", 60),
            BettingOptimizerEntry::new("43", 50),
            BettingOptimizerEntry::new("44", 55),
            BettingOptimizerEntry::new("45", 0),
            BettingOptimizerEntry::new("46", 45),
            BettingOptimizerEntry::new("47", 50),
            BettingOptimizerEntry::new("48", 40),
            BettingOptimizerEntry::new("49", 40),
        ],
        47.0,
        0.02,
        1000.0,
        19,
    )
}

fn build_solved_workbook(prefix: &str) -> std::path::PathBuf {
    let template_path = create_test_output_path(&format!("{prefix}_template"), "xlsm");
    let output_path = create_test_output_path(&format!("{prefix}_output"), "xlsm");
    let vba_project_path = "tests/fixtures/betting_optimizer/vbaProject.bin";
    let request = sample_request();

    write_betting_template_xlsm_with_request(
        template_path.to_str().unwrap(),
        vba_project_path,
        &request,
    )
    .unwrap();

    let workbook = load_betting_workbook_contract(template_path.to_str().unwrap()).unwrap();
    let current_metrics = evaluate_current_metrics(&workbook.request).unwrap();
    let solution = solve_betting_adjustment(&workbook.request).unwrap();
    let summary = build_optimizer_summary(&workbook.request, &current_metrics, &solution);

    write_betting_workbook_solution_xlsm(
        template_path.to_str().unwrap(),
        output_path.to_str().unwrap(),
        &workbook.request,
        &current_metrics,
        &solution,
        &summary,
    )
    .unwrap();

    output_path
}

fn first_sheet_name(path: &std::path::Path) -> String {
    let workbook = open_workbook_auto(path).unwrap();
    workbook.sheet_names().first().unwrap().clone()
}

fn last_sheet_name(path: &std::path::Path) -> String {
    let workbook = open_workbook_auto(path).unwrap();
    workbook.sheet_names().last().unwrap().clone()
}

fn build_manual_constraint_round_sheet(
    prefix: &str,
    row_overrides: &[(u32, u16, &str)],
) -> std::path::PathBuf {
    let request = sample_request();
    let mut cells = vec![
        (1, 9, "来源页"),
        (1, 10, CURRENT_SHEET_NAME),
        (2, 9, "本轮结果页名"),
        (2, 10, "优化建议_第1轮"),
        (3, 9, "最大计划亏损目标"),
        (3, 10, "1000"),
        (4, 9, "目标亏损号码数"),
        (4, 10, "19"),
        (7, 0, "号码"),
        (7, 1, "当前基线下注额"),
        (7, 2, "建议下注额"),
        (7, 8, "下轮基线下注额（需要填写）"),
        (7, 9, "手工锁定下轮下注额（需要填写，可留空）"),
        (7, 10, "本轮最多可退款金额（需要填写，可留空）"),
    ];
    for (index, entry) in request.entries.iter().enumerate() {
        let row = 8 + index as u32;
        let stake_text = entry.original_stake.to_string();
        cells.push((row, 0, Box::leak(entry.label.clone().into_boxed_str())));
        cells.push((row, 1, Box::leak(stake_text.clone().into_boxed_str())));
        cells.push((row, 2, Box::leak(stake_text.clone().into_boxed_str())));
        cells.push((row, 8, Box::leak(stake_text.into_boxed_str())));
    }
    cells.extend_from_slice(row_overrides);

    create_positioned_workbook(
        prefix,
        "manual-constraint-round.xlsx",
        &[("优化建议_第1轮", cells)],
    )
}

// 2026-04-21 CST: Rebuild a full next-round workbook from a constrained round
// sheet so the writer contract stays append-only and verifiable.
fn build_manual_constraint_solution_output(
    prefix: &str,
    row_overrides: &[(u32, u16, &str)],
) -> std::path::PathBuf {
    let workbook_path = build_manual_constraint_round_sheet(prefix, row_overrides);
    let _round_sheet_name = first_sheet_name(&workbook_path);
    let contract = load_betting_workbook_contract_from_sheet(
        workbook_path.to_str().unwrap(),
        Some("浼樺寲寤鸿_绗?杞?"),
    )
    .unwrap();
    let current_metrics = evaluate_current_metrics(&contract.request).unwrap();
    let solution = solve_betting_adjustment(&contract.request).unwrap();
    let summary = build_optimizer_summary(&contract.request, &current_metrics, &solution);

    let template_path = create_test_output_path(&format!("{prefix}_template"), "xlsm");
    let output_path = create_test_output_path(&format!("{prefix}_output"), "xlsm");
    let vba_project_path = "tests/fixtures/betting_optimizer/vbaProject.bin";

    write_betting_template_xlsm_with_request(
        template_path.to_str().unwrap(),
        vba_project_path,
        &sample_request(),
    )
    .unwrap();

    write_betting_workbook_solution_xlsm_from_contract(
        template_path.to_str().unwrap(),
        output_path.to_str().unwrap(),
        &contract,
        &current_metrics,
        &solution,
        &summary,
    )
    .unwrap();

    output_path
}

// 2026-04-21 CST: Keep a second helper with sheet-name discovery based on the
// generated workbook, because encoded sheet captions vary across test fixtures.
fn build_manual_constraint_solution_output_verified(
    prefix: &str,
    row_overrides: &[(u32, u16, &str)],
) -> std::path::PathBuf {
    let workbook_path = build_manual_constraint_round_sheet(prefix, row_overrides);
    let round_sheet_name = first_sheet_name(&workbook_path);
    let contract = load_betting_workbook_contract_from_sheet(
        workbook_path.to_str().unwrap(),
        Some(&round_sheet_name),
    )
    .unwrap();
    let current_metrics = evaluate_current_metrics(&contract.request).unwrap();
    let solution = solve_betting_adjustment(&contract.request).unwrap();
    let summary = build_optimizer_summary(&contract.request, &current_metrics, &solution);

    let template_path = create_test_output_path(&format!("{prefix}_template"), "xlsm");
    let output_path = create_test_output_path(&format!("{prefix}_output"), "xlsm");
    let vba_project_path = "tests/fixtures/betting_optimizer/vbaProject.bin";

    write_betting_template_xlsm_with_request(
        template_path.to_str().unwrap(),
        vba_project_path,
        &sample_request(),
    )
    .unwrap();

    write_betting_workbook_solution_xlsm_from_contract(
        template_path.to_str().unwrap(),
        output_path.to_str().unwrap(),
        &contract,
        &current_metrics,
        &solution,
        &summary,
    )
    .unwrap();

    output_path
}

#[test]
fn solved_workbook_writes_manual_constraint_status_columns_verified() {
    let output_path = build_manual_constraint_solution_output_verified(
        "betting_round_manual_status_verified",
        &[(9, 10, "2")],
    );

    let result_sheet_name = last_sheet_name(&output_path);
    let mut workbook = open_workbook_auto(&output_path).unwrap();
    let range = workbook.worksheet_range(&result_sheet_name).unwrap();

    assert_eq!(range.get_value((9, 11)).unwrap().to_string(), "78");
    assert_eq!(
        range.get_value((9, 12)).unwrap().to_string(),
        "人工约束导致目标未完全达成"
    );
}

#[test]
fn solved_workbook_leaves_manual_input_cells_blank_for_new_round_verified() {
    let output_path = build_manual_constraint_solution_output_verified(
        "betting_round_manual_blank_verified",
        &[(9, 10, "2")],
    );

    let result_sheet_name = last_sheet_name(&output_path);
    let mut workbook = open_workbook_auto(&output_path).unwrap();
    let range = workbook.worksheet_range(&result_sheet_name).unwrap();

    assert_eq!(
        range
            .get_value((9, 9))
            .map(|value| value.to_string())
            .unwrap_or_default(),
        ""
    );
    assert_eq!(
        range
            .get_value((9, 10))
            .map(|value| value.to_string())
            .unwrap_or_default(),
        ""
    );
}

#[test]
fn solved_workbook_status_block_marks_constraint_limited_result_verified() {
    let output_path = build_manual_constraint_solution_output_verified(
        "betting_round_manual_status_block_verified",
        &[(9, 10, "2")],
    );

    let result_sheet_name = last_sheet_name(&output_path);
    let mut workbook = open_workbook_auto(&output_path).unwrap();
    let range = workbook.worksheet_range(&result_sheet_name).unwrap();

    assert!(
        range
            .get_value((6, 10))
            .unwrap()
            .to_string()
            .contains("未完全达成"),
        "expected K7 status block to explain constraint-limited solve"
    );
}

#[test]
fn betting_template_writer_builds_macro_enabled_two_sheet_workbook() {
    let output_path = create_test_output_path("betting_optimizer_template", "xlsm");
    let vba_project_path = "tests/fixtures/betting_optimizer/vbaProject.bin";

    write_betting_template_xlsm(output_path.to_str().unwrap(), vba_project_path).unwrap();

    assert!(output_path.exists());
    assert!(zip_entry_exists(&output_path, "xl/vbaProject.bin"));

    let workbook_xml = read_zip_entry_text(&output_path, "xl/workbook.xml");
    assert!(workbook_xml.contains("计算器"));
    assert!(workbook_xml.contains("优化建议"));
}

#[test]
fn betting_template_writer_restores_original_style_calculator_layout() {
    let output_path = create_test_output_path("betting_optimizer_original_layout", "xlsm");
    let vba_project_path = "tests/fixtures/betting_optimizer/vbaProject.bin";

    write_betting_template_xlsm(output_path.to_str().unwrap(), vba_project_path).unwrap();

    let mut workbook = open_workbook_auto(&output_path).unwrap();
    let range = workbook.worksheet_range("计算器").unwrap();

    assert_eq!(range.get_value((0, 0)).unwrap().to_string(), "双");
    assert_eq!(range.get_value((0, 1)).unwrap().to_string(), "蛇");
    assert_eq!(
        range.get_value((0, 2)).unwrap().to_string(),
        "特下注额（需要填写）"
    );
}

#[test]
fn betting_template_writer_marks_manual_input_fields_as_required() {
    let output_path = create_test_output_path("betting_optimizer_required_fields", "xlsm");
    let vba_project_path = "tests/fixtures/betting_optimizer/vbaProject.bin";

    write_betting_template_xlsm(output_path.to_str().unwrap(), vba_project_path).unwrap();

    let mut workbook = open_workbook_auto(&output_path).unwrap();
    let range = workbook.worksheet_range("计算器").unwrap();

    assert_eq!(
        range.get_value((10, 32)).unwrap().to_string(),
        "最大亏损目标值（需要填写）"
    );
    assert_eq!(
        range.get_value((11, 32)).unwrap().to_string(),
        "目标亏损号码数（需要填写）"
    );
}

#[test]
fn workbook_bridge_reads_sheet1_targets_and_numbers_from_template() {
    let output_path = create_test_output_path("betting_optimizer_contract", "xlsm");
    let vba_project_path = "tests/fixtures/betting_optimizer/vbaProject.bin";
    let request = sample_request();

    write_betting_template_xlsm_with_request(
        output_path.to_str().unwrap(),
        vba_project_path,
        &request,
    )
    .unwrap();

    let workbook = load_betting_workbook_contract(output_path.to_str().unwrap()).unwrap();

    assert_eq!(workbook.request.entries.len(), 49);
    assert_eq!(workbook.request.entries[1].original_stake, 80);
    assert_eq!(workbook.request.max_loss_limit, 1000.0);
    assert_eq!(workbook.request.loss_count_target, 19);
}

#[test]
fn solver_writes_summary_and_adjustment_rows_into_second_sheet() {
    let template_path = create_test_output_path("betting_optimizer_template_solve", "xlsm");
    let output_path = create_test_output_path("betting_optimizer_output", "xlsm");
    let vba_project_path = "tests/fixtures/betting_optimizer/vbaProject.bin";
    let request = sample_request();

    write_betting_template_xlsm_with_request(
        template_path.to_str().unwrap(),
        vba_project_path,
        &request,
    )
    .unwrap();

    let workbook = load_betting_workbook_contract(template_path.to_str().unwrap()).unwrap();
    let current_metrics = evaluate_current_metrics(&workbook.request).unwrap();
    let solution = solve_betting_adjustment(&workbook.request).unwrap();
    let summary = build_optimizer_summary(&workbook.request, &current_metrics, &solution);

    write_betting_workbook_solution_xlsm(
        template_path.to_str().unwrap(),
        output_path.to_str().unwrap(),
        &workbook.request,
        &current_metrics,
        &solution,
        &summary,
    )
    .unwrap();

    let mut workbook = open_workbook_auto(&output_path).unwrap();
    let range = workbook.worksheet_range("优化建议_第1轮").unwrap();

    assert_eq!(range.get_value((0, 0)).unwrap().to_string(), "优化摘要");
    assert!(range.get_value((1, 0)).unwrap().to_string().contains("27"));
    assert_eq!(range.get_value((7, 0)).unwrap().to_string(), "号码");
    assert_eq!(range.get_value((8, 0)).unwrap().to_string(), "01");
    assert_eq!(range.get_value((9, 0)).unwrap().to_string(), "02");
    assert_eq!(range.get_value((9, 2)).unwrap().to_string(), "71");
}

#[test]
fn betting_template_writer_adds_red_conditional_format_to_loss_cells() {
    let output_path = create_test_output_path("betting_optimizer_loss_format", "xlsm");
    let vba_project_path = "tests/fixtures/betting_optimizer/vbaProject.bin";

    write_betting_template_xlsm(output_path.to_str().unwrap(), vba_project_path).unwrap();

    let sheet_xml = read_zip_entry_text(&output_path, "xl/worksheets/sheet1.xml");

    assert!(sheet_xml.contains("<conditionalFormatting"));
    assert!(sheet_xml.contains(r#"sqref="E2:E5""#));
    assert!(sheet_xml.contains(r#"sqref="E7:E11""#));
    assert!(sheet_xml.contains(r#"operator="greaterThan""#));
}

#[test]
fn solved_workbook_marks_adjusted_and_loss_cells_with_red_style() {
    let template_path = create_test_output_path("betting_optimizer_red_template", "xlsm");
    let output_path = create_test_output_path("betting_optimizer_red_output", "xlsm");
    let vba_project_path = "tests/fixtures/betting_optimizer/vbaProject.bin";
    let request = sample_request();

    write_betting_template_xlsm_with_request(
        template_path.to_str().unwrap(),
        vba_project_path,
        &request,
    )
    .unwrap();

    let workbook = load_betting_workbook_contract(template_path.to_str().unwrap()).unwrap();
    let current_metrics = evaluate_current_metrics(&workbook.request).unwrap();
    let solution = solve_betting_adjustment(&workbook.request).unwrap();
    let summary = build_optimizer_summary(&workbook.request, &current_metrics, &solution);

    write_betting_workbook_solution_xlsm(
        template_path.to_str().unwrap(),
        output_path.to_str().unwrap(),
        &workbook.request,
        &current_metrics,
        &solution,
        &summary,
    )
    .unwrap();

    let sheet_xml = read_zip_entry_text(&output_path, "xl/worksheets/sheet2.xml");

    let neutral_adjusted_stake_style = cell_style_id(&sheet_xml, "C9").unwrap();
    let highlighted_adjusted_stake_style = cell_style_id(&sheet_xml, "C10").unwrap();
    let neutral_refund_style = cell_style_id(&sheet_xml, "D9").unwrap();
    let highlighted_refund_style = cell_style_id(&sheet_xml, "D10").unwrap();
    let neutral_loss_style = cell_style_id(&sheet_xml, "E9").unwrap();
    let highlighted_current_loss_style = cell_style_id(&sheet_xml, "E10").unwrap();
    let highlighted_adjusted_loss_style = cell_style_id(&sheet_xml, "F10").unwrap();

    assert_ne!(highlighted_adjusted_stake_style, neutral_adjusted_stake_style);
    assert_ne!(highlighted_refund_style, neutral_refund_style);
    assert_ne!(highlighted_current_loss_style, neutral_loss_style);
    assert_eq!(highlighted_adjusted_stake_style, highlighted_refund_style);
    assert_eq!(highlighted_refund_style, highlighted_current_loss_style);
    assert_eq!(highlighted_current_loss_style, highlighted_adjusted_loss_style);
}

#[test]
fn solver_writes_first_round_sheet_name_instead_of_fixed_suggestion_sheet() {
    let template_path = create_test_output_path("betting_optimizer_round1_template", "xlsm");
    let output_path = create_test_output_path("betting_optimizer_round1_output", "xlsm");
    let vba_project_path = "tests/fixtures/betting_optimizer/vbaProject.bin";
    let request = sample_request();

    write_betting_template_xlsm_with_request(
        template_path.to_str().unwrap(),
        vba_project_path,
        &request,
    )
    .unwrap();

    let workbook = load_betting_workbook_contract(template_path.to_str().unwrap()).unwrap();
    let current_metrics = evaluate_current_metrics(&workbook.request).unwrap();
    let solution = solve_betting_adjustment(&workbook.request).unwrap();
    let summary = build_optimizer_summary(&workbook.request, &current_metrics, &solution);

    write_betting_workbook_solution_xlsm(
        template_path.to_str().unwrap(),
        output_path.to_str().unwrap(),
        &workbook.request,
        &current_metrics,
        &solution,
        &summary,
    )
    .unwrap();

    let workbook_xml = read_zip_entry_text(&output_path, "xl/workbook.xml");

    assert!(workbook_xml.contains("优化建议_第1轮"));
    assert!(!workbook_xml.contains("优化建议</sheet>"));
}

#[test]
fn solved_workbook_records_source_sheet_in_round_header() {
    let template_path = create_test_output_path("betting_optimizer_round_meta_template", "xlsm");
    let output_path = create_test_output_path("betting_optimizer_round_meta_output", "xlsm");
    let vba_project_path = "tests/fixtures/betting_optimizer/vbaProject.bin";
    let request = sample_request();

    write_betting_template_xlsm_with_request(
        template_path.to_str().unwrap(),
        vba_project_path,
        &request,
    )
    .unwrap();

    let workbook = load_betting_workbook_contract(template_path.to_str().unwrap()).unwrap();
    let current_metrics = evaluate_current_metrics(&workbook.request).unwrap();
    let solution = solve_betting_adjustment(&workbook.request).unwrap();
    let summary = build_optimizer_summary(&workbook.request, &current_metrics, &solution);

    write_betting_workbook_solution_xlsm(
        template_path.to_str().unwrap(),
        output_path.to_str().unwrap(),
        &workbook.request,
        &current_metrics,
        &solution,
        &summary,
    )
    .unwrap();

    let mut workbook = open_workbook_auto(&output_path).unwrap();
    let range = workbook.worksheet_range("优化建议_第1轮").unwrap();

    assert_eq!(range.get_value((1, 9)).unwrap().to_string(), "来源页");
    assert_eq!(
        range.get_value((1, 10)).unwrap().to_string(),
        CURRENT_SHEET_NAME
    );
}

#[test]
fn template_contains_active_sheet_recalc_macro_entrypoint() {
    let vba_text = fs::read_to_string(
        "assets/excel_templates/betting_optimizer/vba/BettingSolverRunner.bas",
    )
    .unwrap();

    assert!(vba_text.contains("RunBettingSolverFromActiveSheet"));
}

#[test]
fn template_binds_base_sheet_button_to_active_sheet_solver_macro() {
    let output_path = create_test_output_path("betting_optimizer_button_binding", "xlsm");
    let vba_project_path = "tests/fixtures/betting_optimizer/vbaProject.bin";

    write_betting_template_xlsm(output_path.to_str().unwrap(), vba_project_path).unwrap();

    let vml_text = read_zip_entry_text(&output_path, "xl/drawings/vmlDrawing1.vml");
    assert!(vml_text.contains("RunBettingSolverFromActiveSheet"));
}

#[test]
fn workbook_bridge_can_read_next_round_baseline_from_result_sheet() {
    let template_path = create_test_output_path("betting_optimizer_round_input_template", "xlsm");
    let output_path = create_test_output_path("betting_optimizer_round_input_output", "xlsm");
    let vba_project_path = "tests/fixtures/betting_optimizer/vbaProject.bin";
    let request = sample_request();

    write_betting_template_xlsm_with_request(
        template_path.to_str().unwrap(),
        vba_project_path,
        &request,
    )
    .unwrap();

    let workbook = load_betting_workbook_contract(template_path.to_str().unwrap()).unwrap();
    let current_metrics = evaluate_current_metrics(&workbook.request).unwrap();
    let solution = solve_betting_adjustment(&workbook.request).unwrap();
    let summary = build_optimizer_summary(&workbook.request, &current_metrics, &solution);

    write_betting_workbook_solution_xlsm(
        template_path.to_str().unwrap(),
        output_path.to_str().unwrap(),
        &workbook.request,
        &current_metrics,
        &solution,
        &summary,
    )
    .unwrap();

    let round_contract = load_betting_workbook_contract_from_sheet(
        output_path.to_str().unwrap(),
        Some("优化建议_第1轮"),
    )
    .unwrap();

    assert_eq!(round_contract.source_round_index, Some(1));
    assert_eq!(round_contract.request.entries[1].original_stake, 71);
    assert_eq!(round_contract.request.max_loss_limit, 1000.0);
    assert_eq!(round_contract.request.loss_count_target, 19);
}

#[test]
fn result_sheet_contains_manual_constraint_columns() {
    let output_path = build_solved_workbook("betting_manual_constraint_columns");

    let mut workbook = open_workbook_auto(&output_path).unwrap();
    let range = workbook.worksheet_range("优化建议_第1轮").unwrap();

    assert_eq!(
        range.get_value((7, 8)).unwrap().to_string(),
        "下轮基线下注额（需要填写）"
    );
    assert_eq!(
        range.get_value((7, 9)).unwrap().to_string(),
        "手工锁定下轮下注额（需要填写，可留空）"
    );
    assert_eq!(
        range.get_value((7, 10)).unwrap().to_string(),
        "本轮最多可退款金额（需要填写，可留空）"
    );
    assert_eq!(
        range.get_value((7, 11)).unwrap().to_string(),
        "对应最低保留下注额"
    );
    assert_eq!(
        range.get_value((7, 12)).unwrap().to_string(),
        "人工约束状态"
    );
}

#[test]
fn workbook_bridge_reads_manual_constraints_from_round_sheet() {
    let workbook_path = build_manual_constraint_round_sheet(
        "betting_round_manual_constraints",
        &[(8, 9, "180"), (9, 10, "20")],
    );

    let contract = load_betting_workbook_contract_from_sheet(
        workbook_path.to_str().unwrap(),
        Some("优化建议_第1轮"),
    )
    .unwrap();

    assert_eq!(contract.request.entries[0].manual_locked_stake, Some(180));
    assert_eq!(contract.request.entries[1].manual_refund_cap, Some(20));
}

#[test]
fn workbook_bridge_rejects_row_with_both_manual_inputs() {
    let workbook_path = build_manual_constraint_round_sheet(
        "betting_round_manual_constraint_conflict",
        &[(8, 9, "180"), (8, 10, "20")],
    );

    let error = match load_betting_workbook_contract_from_sheet(
        workbook_path.to_str().unwrap(),
        Some("优化建议_第1轮"),
    ) {
        Ok(_) => panic!("expected conflicting manual inputs to be rejected"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("同一行不能同时填写"));
}
