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
