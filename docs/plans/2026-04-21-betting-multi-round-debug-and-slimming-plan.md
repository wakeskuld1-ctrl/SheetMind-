# Betting Multi-Round Debug And Slimming Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add append-only multi-round result-sheet solving to the betting workbook and reduce `betting_solver.exe` size with the approved release-profile slimming route.

**Architecture:** Keep the current hybrid delivery shape: VBA remains a thin workbook shell and Rust remains the calculation core. Extend the workbook bridge so Rust can read either sheet 1 or a round sheet, then append a new round sheet on each solve. Apply compile-profile-only slimming in `Cargo.toml` without changing crate boundaries in this task.

**Tech Stack:** Rust, `calamine`, `rust_xlsxwriter`, VBA shell module, `assert_cmd`, workbook XML checks via `zip`.

---

### Task 1: Lock the approved multi-round workbook contract with failing tests

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\betting_workbook_bridge_cli.rs`
- Modify: `D:\Rust\Excel_Skill\tests\betting_solver_cli.rs`

**Step 1: Write the failing round-sheet generation tests**

```rust
#[test]
fn solver_appends_first_round_sheet_instead_of_fixed_suggestion_sheet() {
    let output_path = run_solver_against_template();
    let workbook_xml = read_zip_entry_text(&output_path, "xl/workbook.xml");
    assert!(workbook_xml.contains("优化建议_第1轮"));
}

#[test]
fn solver_can_read_round_sheet_and_append_next_round_sheet() {
    let second_output = run_solver_against_round_sheet("优化建议_第1轮");
    let workbook_xml = read_zip_entry_text(&second_output, "xl/workbook.xml");
    assert!(workbook_xml.contains("优化建议_第1轮"));
    assert!(workbook_xml.contains("优化建议_第2轮"));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --test betting_workbook_bridge_cli solver_appends_first_round_sheet_instead_of_fixed_suggestion_sheet -- --nocapture`

Run: `cargo test --test betting_solver_cli solver_can_read_round_sheet_and_append_next_round_sheet -- --nocapture`

Expected: FAIL because the current bridge still writes a fixed result sheet and the CLI cannot target a source result sheet.

### Task 2: Add source-sheet and round metadata contract to the Rust bridge

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\betting_workbook_bridge.rs`
- Modify: `D:\Rust\Excel_Skill\src\bin\betting_solver.rs`

**Step 1: Add a failing parser test for round-sheet input**

```rust
#[test]
fn workbook_bridge_reads_adjusted_baseline_and_targets_from_round_sheet() {
    let workbook = load_betting_workbook_contract_from_sheet(path, "优化建议_第1轮").unwrap();
    assert_eq!(workbook.request.loss_count_target, 19);
    assert_eq!(workbook.request.entries[1].original_stake, 71);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test betting_workbook_bridge_cli workbook_bridge_reads_adjusted_baseline_and_targets_from_round_sheet -- --nocapture`

Expected: FAIL because no round-sheet parser exists.

**Step 3: Implement minimal bridge support**
- Add workbook contract metadata:
  - `source_sheet_name`
  - `source_round_index`
- Add a public loader that accepts an explicit source sheet name.
- Parse sheet 1 with the existing coordinate contract.
- Parse round sheets from their own fixed result-sheet coordinates.

**Step 4: Run the test again**

Run: `cargo test --test betting_workbook_bridge_cli workbook_bridge_reads_adjusted_baseline_and_targets_from_round_sheet -- --nocapture`

Expected: PASS

### Task 3: Append round sheets instead of replacing one fixed suggestion sheet

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\betting_workbook_bridge.rs`
- Modify: `D:\Rust\Excel_Skill\tests\betting_workbook_bridge_cli.rs`

**Step 1: Add a failing output test**

```rust
#[test]
fn solved_workbook_names_round_sheet_and_records_source_sheet() {
    let mut workbook = open_workbook_auto(&output_path).unwrap();
    let range = workbook.worksheet_range("优化建议_第1轮").unwrap();
    assert_eq!(range.get_value((1, 0)).unwrap().to_string(), "来源页");
    assert_eq!(range.get_value((1, 1)).unwrap().to_string(), "计算器");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test betting_workbook_bridge_cli solved_workbook_names_round_sheet_and_records_source_sheet -- --nocapture`

Expected: FAIL because the current result sheet layout does not include round metadata.

**Step 3: Implement minimal write support**
- Generate next round name from the source sheet.
- Write round metadata block.
- Keep the approved summary/detail table and red highlighting.
- Make result sheets self-contained for the next round parser.

**Step 4: Run the test again**

Run: `cargo test --test betting_workbook_bridge_cli solved_workbook_names_round_sheet_and_records_source_sheet -- --nocapture`

Expected: PASS

### Task 4: Extend the CLI to solve from an explicit source sheet

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\bin\betting_solver.rs`
- Modify: `D:\Rust\Excel_Skill\tests\betting_solver_cli.rs`

**Step 1: Add a failing CLI test**

```rust
#[test]
fn betting_solver_accepts_source_sheet_option() {
    Command::cargo_bin("betting_solver")
        .unwrap()
        .args(["solve", input, output, "--source-sheet", "优化建议_第1轮"])
        .assert()
        .success();
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test betting_solver_cli betting_solver_accepts_source_sheet_option -- --nocapture`

Expected: FAIL because the current CLI only accepts positional input/output.

**Step 3: Implement minimal CLI parsing**
- Support:
  - `betting_solver solve <input.xlsm> [output.xlsm]`
  - `betting_solver solve <input.xlsm> [output.xlsm] --source-sheet <sheet_name>`
- Log the chosen source sheet.

**Step 4: Run the test again**

Run: `cargo test --test betting_solver_cli betting_solver_accepts_source_sheet_option -- --nocapture`

Expected: PASS

### Task 5: Update VBA shell for result-sheet re-solve

**Files:**
- Modify: `D:\Rust\Excel_Skill\assets\excel_templates\betting_optimizer\vba\BettingSolverRunner.bas`

**Step 1: Add a failing workbook-layout/asset expectation test**

```rust
#[test]
fn template_contains_result_round_recalc_button_macro_name() {
    let vba_text = std::fs::read_to_string(VBA_RUNNER_PATH).unwrap();
    assert!(vba_text.contains("RunBettingSolverFromActiveSheet"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test betting_workbook_bridge_cli template_contains_result_round_recalc_button_macro_name -- --nocapture`

Expected: FAIL because the current VBA only supports sheet-1 solving.

**Step 3: Implement minimal VBA changes**
- Keep `say_hello` for sheet 1.
- Add `RunBettingSolverFromActiveSheet`.
- Build solver command with `--source-sheet "<ActiveSheet.Name>"`.
- Import the newly created round sheet instead of replacing a fixed sheet.
- Log source sheet name and imported round sheet name.

**Step 4: Run the test again**

Run: `cargo test --test betting_workbook_bridge_cli template_contains_result_round_recalc_button_macro_name -- --nocapture`

Expected: PASS

### Task 6: Add release-profile slimming and verify size drops

**Files:**
- Modify: `D:\Rust\Excel_Skill\Cargo.toml`
- Modify: `D:\Rust\Excel_Skill\tests\betting_solver_cli.rs`

**Step 1: Add a failing release-build expectation note test**

```rust
#[test]
fn cargo_manifest_contains_release_slimming_profile_for_betting_delivery() {
    let manifest = std::fs::read_to_string("Cargo.toml").unwrap();
    assert!(manifest.contains("[profile.release]"));
    assert!(manifest.contains("strip"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test betting_solver_cli cargo_manifest_contains_release_slimming_profile_for_betting_delivery -- --nocapture`

Expected: FAIL because no slimming profile exists yet.

**Step 3: Implement minimal manifest changes**
- Add:
  - `lto = true`
  - `codegen-units = 1`
  - `panic = "abort"`
  - `strip = "symbols"`

**Step 4: Run the test again**

Run: `cargo test --test betting_solver_cli cargo_manifest_contains_release_slimming_profile_for_betting_delivery -- --nocapture`

Expected: PASS

### Task 7: Refresh the delivery artifacts and verify the approved flow

**Files:**
- Modify: `D:\Rust\Excel_Skill\outputs\betting_optimizer_delivery\计算器s6.1_稳交付版_页面还原.xlsm`
- Modify: `D:\Rust\Excel_Skill\outputs\betting_optimizer_delivery\计算器s6.1_稳交付版_页面还原-结果.xlsm`
- Modify: `D:\Rust\Excel_Skill\outputs\betting_optimizer_delivery\betting_solver.exe`

**Step 1: Run focused tests**

Run: `cargo test --test betting_workbook_bridge_cli -- --nocapture`

Run: `cargo test --test betting_solver_cli -- --nocapture`

Expected: PASS

**Step 2: Build release binary**

Run: `cargo build --release --bin betting_solver`

Expected: PASS

**Step 3: Record binary size**

Run: `Get-ChildItem D:\Rust\Excel_Skill\target\release\betting_solver.exe | Select-Object FullName,Length`

Expected: size is below the previous baseline of `52,562,986` bytes.

**Step 4: Rebuild delivery workbooks**

Run:

```powershell
& 'D:\Rust\Excel_Skill\target\release\betting_solver.exe' template 'D:\Rust\Excel_Skill\outputs\betting_optimizer_delivery\计算器s6.1_稳交付版_页面还原.xlsm'
& 'D:\Rust\Excel_Skill\target\release\betting_solver.exe' solve 'D:\Rust\Excel_Skill\outputs\betting_optimizer_delivery\计算器s6.1_稳交付版_页面还原.xlsm' 'D:\Rust\Excel_Skill\outputs\betting_optimizer_delivery\计算器s6.1_稳交付版_页面还原-结果.xlsm'
```

**Step 5: Verify one chained round**
- Open the generated result workbook through tests or workbook XML inspection.
- Confirm `优化建议_第1轮` exists.
- Run solve again against `--source-sheet 优化建议_第1轮`.
- Confirm `优化建议_第2轮` exists and round 1 remains present.
