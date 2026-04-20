# Betting Manual Adjustment Constraints Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add per-number manual constraint inputs to result sheets so operators can set either a hard next-round stake or a refund cap, then generate the next round sheet under those constraints.

**Architecture:** Keep VBA as the workbook shell and keep Rust as the only calculation engine. Extend the result-sheet contract, parser, solver request model, and output writer together so that the current round acts as input evidence and the next round acts as the recomputed result.

**Tech Stack:** Rust, `calamine`, `rust_xlsxwriter`, VBA shell module, `assert_cmd`, workbook XML inspection via `zip`, PowerShell verification commands.

---

### Task 1: Lock the new result-sheet contract with failing tests

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\betting_workbook_bridge_cli.rs`
- Modify: `D:\Rust\Excel_Skill\tests\betting_solver_cli.rs`

**Step 1: Write the failing workbook-layout tests**

```rust
#[test]
fn result_sheet_contains_manual_constraint_columns() {
    let output_path = run_solver_against_template();
    let header = read_result_sheet_header(&output_path, "优化建议_第1轮");
    assert!(header.contains(&"手工锁定下轮下注额（需要填写，可留空）".to_string()));
    assert!(header.contains(&"本轮最多可退款金额（需要填写，可留空）".to_string()));
    assert!(header.contains(&"对应最低保留下注额".to_string()));
    assert!(header.contains(&"人工约束状态".to_string()));
}

#[test]
fn re_solve_from_round_sheet_still_generates_next_round_sheet() {
    let round2 = run_solver_against_round_sheet_with_constraints("优化建议_第1轮");
    let workbook_xml = read_zip_entry_text(&round2, "xl/workbook.xml");
    assert!(workbook_xml.contains("优化建议_第2轮"));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --test betting_workbook_bridge_cli result_sheet_contains_manual_constraint_columns -- --nocapture`

Run: `cargo test --test betting_solver_cli re_solve_from_round_sheet_still_generates_next_round_sheet -- --nocapture`

Expected: FAIL because the current result-sheet contract does not include manual constraint columns.

### Task 2: Add round-sheet parsing for manual constraints

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\betting_workbook_bridge.rs`
- Modify: `D:\Rust\Excel_Skill\tests\betting_workbook_bridge_cli.rs`

**Step 1: Write the failing parser tests**

```rust
#[test]
fn workbook_bridge_reads_hard_lock_and_refund_cap_from_round_sheet() {
    let contract = load_betting_workbook_contract_from_sheet(path, "优化建议_第1轮").unwrap();
    assert_eq!(contract.request.entries[0].manual_locked_stake, Some(180));
    assert_eq!(contract.request.entries[1].manual_refund_cap, Some(20));
}

#[test]
fn workbook_bridge_rejects_row_with_both_manual_inputs() {
    let err = load_betting_workbook_contract_from_sheet(path, "优化建议_第1轮").unwrap_err();
    assert!(err.to_string().contains("同一行不能同时填写"));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --test betting_workbook_bridge_cli workbook_bridge_reads_hard_lock_and_refund_cap_from_round_sheet -- --nocapture`

Run: `cargo test --test betting_workbook_bridge_cli workbook_bridge_rejects_row_with_both_manual_inputs -- --nocapture`

Expected: FAIL because no manual constraint parser exists yet.

**Step 3: Write minimal implementation**
- Extend the round-sheet column contract.
- Parse both manual input columns.
- Validate integer-only, one-or-the-other, and range rules.
- Store parsed values in the solver request model.

**Step 4: Run the tests again**

Run: `cargo test --test betting_workbook_bridge_cli workbook_bridge_reads_hard_lock_and_refund_cap_from_round_sheet -- --nocapture`

Run: `cargo test --test betting_workbook_bridge_cli workbook_bridge_rejects_row_with_both_manual_inputs -- --nocapture`

Expected: PASS

### Task 3: Add solver-side manual constraint support

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\bin\betting_solver.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\betting_workbook_bridge.rs`
- Modify: `D:\Rust\Excel_Skill\tests\betting_solver_cli.rs`

**Step 1: Write the failing solver behavior tests**

```rust
#[test]
fn solver_honors_manual_locked_stake_when_recomputing() {
    let solved = run_solver_with_locked_stake();
    assert_eq!(row_for(&solved, 30).suggested_stake, 180);
}

#[test]
fn solver_honors_manual_refund_cap_when_recomputing() {
    let solved = run_solver_with_refund_cap();
    assert!(row_for(&solved, 30).refund <= 20);
}

#[test]
fn solver_redistributes_other_risky_rows_after_manual_constraint() {
    let solved = run_solver_with_refund_cap();
    assert_ne!(row_for(&solved, 31).suggested_stake, row_for_baseline(&solved, 31));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --test betting_solver_cli solver_honors_manual_locked_stake_when_recomputing -- --nocapture`

Run: `cargo test --test betting_solver_cli solver_honors_manual_refund_cap_when_recomputing -- --nocapture`

Run: `cargo test --test betting_solver_cli solver_redistributes_other_risky_rows_after_manual_constraint -- --nocapture`

Expected: FAIL because the solver currently ignores manual per-row constraints.

**Step 3: Write minimal implementation**
- Extend the solver request entry model with:
  - `manual_locked_stake`
  - `manual_refund_cap`
- Apply hard lock before optimization decisions.
- Apply refund cap as an upper bound on reducible stake for that row.
- Preserve the existing objective order:
  - minimum total refund
  - risky-number count closest to target

**Step 4: Run the tests again**

Run: `cargo test --test betting_solver_cli solver_honors_manual_locked_stake_when_recomputing -- --nocapture`

Run: `cargo test --test betting_solver_cli solver_honors_manual_refund_cap_when_recomputing -- --nocapture`

Run: `cargo test --test betting_solver_cli solver_redistributes_other_risky_rows_after_manual_constraint -- --nocapture`

Expected: PASS

### Task 4: Distinguish invalid input from constraint-limited output

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\bin\betting_solver.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\betting_workbook_bridge.rs`
- Modify: `D:\Rust\Excel_Skill\tests\betting_solver_cli.rs`

**Step 1: Write the failing status tests**

```rust
#[test]
fn solver_marks_constraint_limited_result_when_target_cannot_be_fully_met() {
    let solved = run_solver_with_unreachable_constraints();
    assert!(solved.summary_text.contains("目标未完全达成"));
}

#[test]
fn solver_rejects_refund_cap_on_non_risk_row() {
    let assert = run_solver_with_non_risk_refund_cap();
    assert.failure().stderr(predicates::str::contains("非风险号码"));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --test betting_solver_cli solver_marks_constraint_limited_result_when_target_cannot_be_fully_met -- --nocapture`

Run: `cargo test --test betting_solver_cli solver_rejects_refund_cap_on_non_risk_row -- --nocapture`

Expected: FAIL because the current flow does not distinguish these two cases.

**Step 3: Write minimal implementation**
- Treat invalid workbook inputs as hard errors.
- Treat feasible-but-target-limited constraint sets as successful solve results with explicit status.
- Surface the distinction in:
  - console output
  - trace log
  - workbook status block

**Step 4: Run the tests again**

Run: `cargo test --test betting_solver_cli solver_marks_constraint_limited_result_when_target_cannot_be_fully_met -- --nocapture`

Run: `cargo test --test betting_solver_cli solver_rejects_refund_cap_on_non_risk_row -- --nocapture`

Expected: PASS

### Task 5: Write next-round workbook output with new columns and statuses

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\betting_workbook_bridge.rs`
- Modify: `D:\Rust\Excel_Skill\tests\betting_workbook_bridge_cli.rs`

**Step 1: Write the failing workbook-output tests**

```rust
#[test]
fn solved_workbook_writes_manual_constraint_status_columns() {
    let solved = run_solver_with_refund_cap();
    let row = read_result_row(&solved, "优化建议_第2轮", 30);
    assert_eq!(row.constraint_status, "已设置退款上限");
}

#[test]
fn solved_workbook_leaves_manual_input_cells_blank_for_new_round() {
    let solved = run_solver_with_refund_cap();
    let row = read_result_row(&solved, "优化建议_第2轮", 30);
    assert_eq!(row.manual_locked_stake_cell, "");
    assert_eq!(row.manual_refund_cap_cell, "");
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --test betting_workbook_bridge_cli solved_workbook_writes_manual_constraint_status_columns -- --nocapture`

Run: `cargo test --test betting_workbook_bridge_cli solved_workbook_leaves_manual_input_cells_blank_for_new_round -- --nocapture`

Expected: FAIL because the current writer does not emit the new columns.

**Step 3: Write minimal implementation**
- Add the four new columns to the result-sheet writer.
- Keep manual input cells editable and yellow.
- Keep derived/status columns read-only.
- Preserve existing red risk and adjustment highlighting.
- Ensure the new round remains self-contained for future parsing.

**Step 4: Run the tests again**

Run: `cargo test --test betting_workbook_bridge_cli solved_workbook_writes_manual_constraint_status_columns -- --nocapture`

Run: `cargo test --test betting_workbook_bridge_cli solved_workbook_leaves_manual_input_cells_blank_for_new_round -- --nocapture`

Expected: PASS

### Task 6: Update VBA flow to read current sheet and write the next round only

**Files:**
- Modify: `D:\Rust\Excel_Skill\assets\excel_templates\betting_optimizer\vba\BettingSolverRunner.bas`
- Modify: `D:\Rust\Excel_Skill\tests\betting_workbook_bridge_cli.rs`

**Step 1: Write the failing VBA expectation test**

```rust
#[test]
fn vba_runner_keeps_round_recalc_as_append_only_next_round_flow() {
    let vba_text = std::fs::read_to_string(VBA_RUNNER_PATH).unwrap();
    assert!(vba_text.contains("ActiveSheet.Name"));
    assert!(vba_text.contains("基于本页再次测算"));
}
```

**Step 2: Run test to verify it fails or is incomplete**

Run: `cargo test --test betting_workbook_bridge_cli vba_runner_keeps_round_recalc_as_append_only_next_round_flow -- --nocapture`

Expected: FAIL or insufficient assertion coverage.

**Step 3: Write minimal implementation**
- Keep reading constraints from the active result sheet.
- Keep solver invocation on the active source sheet.
- Import only the next generated result sheet.
- Do not modify the current result sheet in place.
- Update status and log text so the operator sees:
  - source round
  - generated next round

**Step 4: Run the test again**

Run: `cargo test --test betting_workbook_bridge_cli vba_runner_keeps_round_recalc_as_append_only_next_round_flow -- --nocapture`

Expected: PASS

### Task 7: Run full verification and refresh delivery artifacts

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

**Step 3: Refresh the delivery workbooks**

Run:

```powershell
& 'D:\Rust\Excel_Skill\target\release\betting_solver.exe' template 'D:\Rust\Excel_Skill\outputs\betting_optimizer_delivery\计算器s6.1_稳交付版_页面还原.xlsm'
& 'D:\Rust\Excel_Skill\target\release\betting_solver.exe' solve 'D:\Rust\Excel_Skill\outputs\betting_optimizer_delivery\计算器s6.1_稳交付版_页面还原.xlsm' 'D:\Rust\Excel_Skill\outputs\betting_optimizer_delivery\计算器s6.1_稳交付版_页面还原-结果.xlsm'
```

Expected: PASS

**Step 4: Record release evidence**

Run: `Get-Item D:\Rust\Excel_Skill\outputs\betting_optimizer_delivery\betting_solver.exe | Select-Object FullName,Length,LastWriteTime`

Expected: fresh release binary metadata is shown.

**Step 5: Commit**

```bash
git add docs/plans/2026-04-21-betting-manual-adjustment-constraints-design.md docs/plans/2026-04-21-betting-manual-adjustment-constraints-plan.md src/ops/betting_workbook_bridge.rs src/bin/betting_solver.rs assets/excel_templates/betting_optimizer/vba/BettingSolverRunner.bas tests/betting_workbook_bridge_cli.rs tests/betting_solver_cli.rs outputs/betting_optimizer_delivery
git commit -m "feat: add betting manual adjustment constraints"
```
