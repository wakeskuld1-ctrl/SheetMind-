mod common;

use assert_cmd::Command;
use calamine::{Reader, open_workbook_auto};
use excel_skill::ops::betting_optimizer::{BettingOptimizerEntry, BettingOptimizerRequest};
use excel_skill::ops::betting_workbook_bridge::{
    CURRENT_SHEET_NAME, SUGGESTION_ROUND_SHEET_PREFIX, write_betting_template_xlsm_with_request,
};
use std::fs;
use std::fs::File;
use zip::ZipArchive;

use crate::common::create_test_output_path;

fn numbered_entries(stakes: &[i64]) -> Vec<BettingOptimizerEntry> {
    stakes
        .iter()
        .enumerate()
        .map(|(index, stake)| BettingOptimizerEntry::new(format!("{:02}", index + 1), *stake))
        .collect::<Vec<_>>()
}

fn concentrated_request() -> BettingOptimizerRequest {
    let mut stakes = vec![0; 49];
    stakes[0] = 1000;
    BettingOptimizerRequest::new(numbered_entries(&stakes), 47.0, 0.02, 1500.0, 19)
}

fn already_safe_uniform_request() -> BettingOptimizerRequest {
    BettingOptimizerRequest::new(numbered_entries(&vec![30; 49]), 47.0, 0.02, 1500.0, 0)
}

// 2026-04-21 CST: Read the actual generated round sheet from the workbook
// instead of hard-coding a localized name. This keeps the regression stable
// across encoding differences while still proving next-round chaining works.
fn latest_round_sheet_name(path: &std::path::Path) -> String {
    let workbook = open_workbook_auto(path).unwrap();
    workbook
        .sheet_names()
        .iter()
        .filter(|name| name.starts_with(SUGGESTION_ROUND_SHEET_PREFIX))
        .last()
        .cloned()
        .expect("workbook should contain at least one round result sheet")
}

// 2026-04-21 CST: Keep a regression for the new copy-friendly summary block
// so every generated round sheet exposes the same operator-facing refund text.
fn assert_round_sheet_has_copy_block(path: &std::path::Path, sheet_name: &str) {
    let mut workbook = open_workbook_auto(path).unwrap();
    let range = workbook.worksheet_range(sheet_name).unwrap();
    let copy_label = range.get_value((7, 13)).unwrap().to_string();
    let copy_text = range.get_value((8, 13)).unwrap().to_string();

    assert_eq!(copy_label, "可复制描述");
    assert!(!copy_text.is_empty());
    assert!(copy_text.contains("建议") || copy_text.contains("无需下调"));
}

// 2026-04-22 CST: Add a new multi-box verifier because the result sheet now
// keeps four separate copy-friendly descriptions in rows only, without adding
// extra columns or changing the manual-adjustment table layout.
fn assert_round_sheet_has_all_copy_blocks(path: &std::path::Path, sheet_name: &str) {
    let mut workbook = open_workbook_auto(path).unwrap();
    let range = workbook.worksheet_range(sheet_name).unwrap();
    let original_label = range.get_value((7, 13)).unwrap().to_string();
    let original_text = range.get_value((8, 13)).unwrap().to_string();
    let overage_label = range.get_value((10, 13)).unwrap().to_string();
    let overage_text = range.get_value((11, 13)).unwrap().to_string();
    let small_group_label = range.get_value((13, 13)).unwrap().to_string();
    let small_group_text = range.get_value((14, 13)).unwrap().to_string();
    let large_group_label = range.get_value((16, 13)).unwrap().to_string();
    let large_group_text = range.get_value((17, 13)).unwrap().to_string();

    assert_eq!(original_label, "可复制描述（原始）");
    assert_eq!(overage_label, "可复制描述（大于30净额）");
    assert_eq!(small_group_label, "可复制描述（小于等于30归类）");
    assert_eq!(large_group_label, "可复制描述（大于30归类）");
    assert!(!original_text.is_empty());
    assert!(!overage_text.is_empty());
    assert!(!small_group_text.is_empty());
    assert!(!large_group_text.is_empty());
    assert!(original_text.contains("建议") || original_text.contains("无需下调"));
    assert!(overage_text.contains("净额建议"));
    assert!(small_group_text.contains("归类"));
    assert!(large_group_text.contains("归类"));
}

#[test]
fn betting_solver_returns_nonzero_when_arguments_missing() {
    let mut cmd = Command::cargo_bin("betting_solver").unwrap();
    cmd.assert().failure();
}

#[test]
fn betting_solver_template_and_solve_commands_succeed() {
    let template_path = create_test_output_path("betting_solver_template", "xlsm");
    let output_path = create_test_output_path("betting_solver_output", "xlsm");

    Command::cargo_bin("betting_solver")
        .unwrap()
        .args(["template", template_path.to_str().unwrap()])
        .assert()
        .success();

    Command::cargo_bin("betting_solver")
        .unwrap()
        .args([
            "solve",
            template_path.to_str().unwrap(),
            output_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(template_path.exists());
    assert!(output_path.exists());
}

#[test]
fn betting_solver_solve_writes_trace_log() {
    let template_path = create_test_output_path("betting_solver_log_template", "xlsm");
    let output_path = create_test_output_path("betting_solver_log_output", "xlsm");
    let log_dir = output_path.parent().unwrap().join("logs");

    Command::cargo_bin("betting_solver")
        .unwrap()
        .env("BETTING_SOLVER_LOG_DIR", &log_dir)
        .args(["template", template_path.to_str().unwrap()])
        .assert()
        .success();

    Command::cargo_bin("betting_solver")
        .unwrap()
        .env("BETTING_SOLVER_LOG_DIR", &log_dir)
        .args([
            "solve",
            template_path.to_str().unwrap(),
            output_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    let log_paths = fs::read_dir(&log_dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();

    assert!(!log_paths.is_empty());

    let log_text = fs::read_to_string(&log_paths[0]).unwrap();
    assert!(log_text.contains("solver_start"));
    assert!(log_text.contains("contract_loaded"));
    assert!(log_text.contains("solve_success"));
}

#[test]
fn betting_solver_log_records_constraint_limited_flag() {
    let template_path = create_test_output_path("betting_solver_flag_template", "xlsm");
    let output_path = create_test_output_path("betting_solver_flag_output", "xlsm");
    let log_dir = output_path.parent().unwrap().join("logs");

    Command::cargo_bin("betting_solver")
        .unwrap()
        .env("BETTING_SOLVER_LOG_DIR", &log_dir)
        .args(["template", template_path.to_str().unwrap()])
        .assert()
        .success();

    Command::cargo_bin("betting_solver")
        .unwrap()
        .env("BETTING_SOLVER_LOG_DIR", &log_dir)
        .args([
            "solve",
            template_path.to_str().unwrap(),
            output_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    let log_paths = fs::read_dir(&log_dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();

    let log_text = fs::read_to_string(&log_paths[0]).unwrap();
    assert!(log_text.contains("constraint_limited=false"));
}

#[test]
fn betting_solver_accepts_source_sheet_option() {
    let template_path = create_test_output_path("betting_solver_source_sheet_template", "xlsm");
    let output_path = create_test_output_path("betting_solver_source_sheet_output", "xlsm");

    Command::cargo_bin("betting_solver")
        .unwrap()
        .args(["template", template_path.to_str().unwrap()])
        .assert()
        .success();

    Command::cargo_bin("betting_solver")
        .unwrap()
        .args([
            "solve",
            template_path.to_str().unwrap(),
            output_path.to_str().unwrap(),
            "--source-sheet",
            CURRENT_SHEET_NAME,
        ])
        .assert()
        .success();
}

#[test]
fn cargo_manifest_contains_release_slimming_profile_for_betting_delivery() {
    let manifest = fs::read_to_string("Cargo.toml").unwrap();

    assert!(manifest.contains("[profile.release]"));
    assert!(manifest.contains("lto"));
    assert!(manifest.contains("codegen-units"));
    assert!(manifest.contains("panic"));
    assert!(manifest.contains("strip"));
}

#[test]
fn betting_solver_can_solve_from_round_sheet_and_produce_next_round() {
    let template_path = create_test_output_path("betting_solver_round1_template", "xlsm");
    let round1_path = create_test_output_path("betting_solver_round1_output", "xlsm");
    let round2_path = create_test_output_path("betting_solver_round2_output", "xlsm");
    let round3_path = create_test_output_path("betting_solver_round3_output", "xlsm");

    Command::cargo_bin("betting_solver")
        .unwrap()
        .args(["template", template_path.to_str().unwrap()])
        .assert()
        .success();

    Command::cargo_bin("betting_solver")
        .unwrap()
        .args([
            "solve",
            template_path.to_str().unwrap(),
            round1_path.to_str().unwrap(),
            "--source-sheet",
            CURRENT_SHEET_NAME,
        ])
        .assert()
        .success();

    // 2026-04-21 CST: Feed the second solve with the real round-1 result sheet
    // name so the CLI regression checks workbook chaining behavior rather than
    // depending on a brittle encoded literal.
    let round1_sheet_name = latest_round_sheet_name(&round1_path);
    assert_round_sheet_has_all_copy_blocks(&round1_path, &round1_sheet_name);

    Command::cargo_bin("betting_solver")
        .unwrap()
        .args([
            "solve",
            round1_path.to_str().unwrap(),
            round2_path.to_str().unwrap(),
            "--source-sheet",
            round1_sheet_name.as_str(),
        ])
        .assert()
        .success();

    let round2_sheet_name = latest_round_sheet_name(&round2_path);
    let file = File::open(&round2_path).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();
    let mut workbook_xml = String::new();
    std::io::Read::read_to_string(
        &mut archive.by_name("xl/workbook.xml").unwrap(),
        &mut workbook_xml,
    )
    .unwrap();

    assert_ne!(round2_sheet_name, round1_sheet_name);
    assert!(workbook_xml.contains(&round2_sheet_name));
    assert_round_sheet_has_all_copy_blocks(&round2_path, &round2_sheet_name);

    Command::cargo_bin("betting_solver")
        .unwrap()
        .args([
            "solve",
            round2_path.to_str().unwrap(),
            round3_path.to_str().unwrap(),
            "--source-sheet",
            round2_sheet_name.as_str(),
        ])
        .assert()
        .success();

    let round3_sheet_name = latest_round_sheet_name(&round3_path);
    assert_ne!(round3_sheet_name, round2_sheet_name);
    assert_round_sheet_has_all_copy_blocks(&round3_path, &round3_sheet_name);
}

#[test]
fn betting_solver_can_rebuild_clean_template_from_existing_round_sheet() {
    let template_path = create_test_output_path("betting_solver_rebuild_template", "xlsm");
    let round1_path = create_test_output_path("betting_solver_rebuild_round1", "xlsm");
    let rebuilt_path = create_test_output_path("betting_solver_rebuild_output", "xlsm");

    Command::cargo_bin("betting_solver")
        .unwrap()
        .args(["template", template_path.to_str().unwrap()])
        .assert()
        .success();

    Command::cargo_bin("betting_solver")
        .unwrap()
        .args([
            "solve",
            template_path.to_str().unwrap(),
            round1_path.to_str().unwrap(),
            "--source-sheet",
            CURRENT_SHEET_NAME,
        ])
        .assert()
        .success();

    let round1_sheet_name = latest_round_sheet_name(&round1_path);

    Command::cargo_bin("betting_solver")
        .unwrap()
        .args([
            "rebuild",
            round1_path.to_str().unwrap(),
            rebuilt_path.to_str().unwrap(),
            "--source-sheet",
            round1_sheet_name.as_str(),
        ])
        .assert()
        .success();

    let rebuilt_workbook = open_workbook_auto(&rebuilt_path).unwrap();
    let rebuilt_sheet_names = rebuilt_workbook.sheet_names().to_vec();
    assert_eq!(
        rebuilt_sheet_names,
        vec![CURRENT_SHEET_NAME.to_string(), "优化建议".to_string()],
        "rebuild should generate a fresh two-sheet workbook without carried-over rounds"
    );

    let rebuilt_contract = excel_skill::ops::betting_workbook_bridge::load_betting_workbook_contract(
        rebuilt_path.to_str().unwrap(),
    )
    .unwrap();
    let source_contract =
        excel_skill::ops::betting_workbook_bridge::load_betting_workbook_contract_from_sheet(
            round1_path.to_str().unwrap(),
            Some(&round1_sheet_name),
        )
        .unwrap();

    assert_eq!(
        rebuilt_contract.request.max_loss_limit,
        source_contract.request.max_loss_limit
    );
    assert_eq!(
        rebuilt_contract.request.loss_count_target,
        source_contract.request.loss_count_target
    );
    assert_eq!(
        rebuilt_contract
            .request
            .entries
            .iter()
            .map(|entry| entry.original_stake)
            .collect::<Vec<_>>(),
        source_contract
            .request
            .entries
            .iter()
            .map(|entry| entry.original_stake)
            .collect::<Vec<_>>()
    );
}

#[test]
fn betting_solver_can_solve_concentrated_boundary_workbook() {
    let template_path = create_test_output_path("betting_solver_concentrated_template", "xlsm");
    let output_path = create_test_output_path("betting_solver_concentrated_output", "xlsm");
    let log_dir = output_path.parent().unwrap().join("logs");

    write_betting_template_xlsm_with_request(
        template_path.to_str().unwrap(),
        "tests/fixtures/betting_optimizer/vbaProject.bin",
        &concentrated_request(),
    )
    .unwrap();

    Command::cargo_bin("betting_solver")
        .unwrap()
        .env("BETTING_SOLVER_LOG_DIR", &log_dir)
        .args([
            "solve",
            template_path.to_str().unwrap(),
            output_path.to_str().unwrap(),
            "--source-sheet",
            CURRENT_SHEET_NAME,
        ])
        .assert()
        .success();

    let log_path = fs::read_dir(&log_dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .contains("betting_solver_solve")
        })
        .unwrap();
    let log_text = fs::read_to_string(log_path).unwrap();

    assert!(log_text.contains("solve_success"));
    assert!(log_text.contains("total_refund=968"));
    assert!(log_text.contains("loss_count=1"));
}

#[test]
fn betting_solver_can_solve_zero_refund_boundary_workbook() {
    let template_path = create_test_output_path("betting_solver_zero_refund_template", "xlsm");
    let output_path = create_test_output_path("betting_solver_zero_refund_output", "xlsm");
    let log_dir = output_path.parent().unwrap().join("logs");

    write_betting_template_xlsm_with_request(
        template_path.to_str().unwrap(),
        "tests/fixtures/betting_optimizer/vbaProject.bin",
        &already_safe_uniform_request(),
    )
    .unwrap();

    Command::cargo_bin("betting_solver")
        .unwrap()
        .env("BETTING_SOLVER_LOG_DIR", &log_dir)
        .args([
            "solve",
            template_path.to_str().unwrap(),
            output_path.to_str().unwrap(),
            "--source-sheet",
            CURRENT_SHEET_NAME,
        ])
        .assert()
        .success();

    let log_path = fs::read_dir(&log_dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .contains("betting_solver_solve")
        })
        .unwrap();
    let log_text = fs::read_to_string(log_path).unwrap();

    assert!(log_text.contains("solve_success"));
    assert!(log_text.contains("total_refund=0"));
    assert!(log_text.contains("loss_count=0"));
}
