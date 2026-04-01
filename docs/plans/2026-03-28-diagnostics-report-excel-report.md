# Diagnostics Report Excel Report Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Rust high-level `diagnostics_report_excel_report` Tool that turns the composed `diagnostics_report` result into a stable Excel workbook delivery contract.

**Architecture:** Reuse the existing `diagnostics_report` JSON contract as the business core, then map that result into a fixed four-sheet workbook draft and optionally export `.xlsx` through the existing workbook export pipeline. Keep the current Rust binary-first dispatcher and catalog architecture unchanged.

**Tech Stack:** Rust, serde, Polars, workbook draft store, existing Excel export path, cargo test

---

### Task 1: Lock the Excel delivery contract with failing CLI tests

**Files:**
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\diagnostics_report_excel_report_cli.rs`
- Reference: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\diagnostics_report_cli.rs`

**Step 1: Write the failing test for tool discovery**

- Add one catalog test asserting `diagnostics_report_excel_report` appears in the Rust tool catalog.

**Step 2: Write the failing test for workbook_ref delivery**

- Add one end-to-end test that passes a seeded `result_ref` plus all four diagnostics sections and expects:
  - `status == "ok"`
  - `data.format == "workbook_ref"`
  - `data.workbook_ref` is non-empty
  - `data.sheet_names.len() == 4`
  - `data.diagnostics_result.report_status == "ok"`

**Step 3: Write the failing test for degraded export**

- Add one end-to-end test that intentionally breaks the `trend` section while keeping other sections valid and expects:
  - `status == "ok"`
  - `data.diagnostics_result.report_status == "degraded"`
  - workbook delivery still succeeds
  - warnings contain `trend_analysis`

**Step 4: Write the failing test for file export**

- Add one test that passes a temp `output_path` and expects:
  - `status == "ok"`
  - `data.format == "xlsx"`
  - target file exists after execution

**Step 5: Run test to verify failure**

Run: `cargo test --test diagnostics_report_excel_report_cli -- --nocapture`

Expected: FAIL because the Tool does not exist yet.

### Task 2: Implement the high-level workbook delivery op

**Files:**
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\diagnostics_report_excel_report.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\mod.rs`

**Step 1: Add request/response structs**

- Define:
  - `DiagnosticsReportExcelReportRequest`
  - `DiagnosticsReportExcelReportResult`
- Flatten the existing `DiagnosticsReportRequest` into the new request so callers keep the same section inputs.

**Step 2: Reuse the composed diagnostics core**

- Call `diagnostics_report()` internally.
- Reject empty `report_name`.
- Reject blank `output_path`.

**Step 3: Build fixed workbook sheets**

- Create four sheet builders:
  - summary sheet
  - section overview sheet
  - correlation-and-outlier sheet
  - distribution-and-trend sheet
- Keep the first version table-first, not chart-first.

**Step 4: Persist workbook draft and optional file export**

- Reuse `WorkbookDraftStore::create_workbook_ref()`
- Reuse `PersistedWorkbookDraft::from_sheet_inputs(...)`
- Reuse `export_excel_workbook(...)`

**Step 5: Run targeted test**

Run: `cargo test --test diagnostics_report_excel_report_cli -- --nocapture`

Expected: still failing until dispatcher and catalog wiring is complete.

### Task 3: Wire the Tool into the Rust catalog and dispatcher

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\catalog.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\dispatcher.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\dispatcher\analysis_ops.rs`

**Step 1: Add catalog entry**

- Register `diagnostics_report_excel_report` in the tool catalog.

**Step 2: Add dispatcher handler**

- Parse the request into `DiagnosticsReportExcelReportRequest`.
- Reuse `load_table_for_analysis()` and `apply_optional_casts()`.
- Keep session sync behavior aligned with other analysis-delivery Tools.

**Step 3: Return the unified response**

- Ensure the Tool returns:
  - `diagnostics_result`
  - `workbook_ref`
  - `sheet_names`
  - `format`
  - optional `output_path`

**Step 4: Run targeted test**

Run: `cargo test --test diagnostics_report_excel_report_cli -- --nocapture`

Expected: PASS

### Task 4: Run focused regression

**Files:**
- No new files required

**Step 1: Run the new target suite**

Run: `cargo test --test diagnostics_report_excel_report_cli -- --nocapture`

Expected: PASS

**Step 2: Re-run adjacent diagnostics suites**

Run:

- `cargo test --test diagnostics_report_cli -- --nocapture`
- `cargo test --test stat_diagnostics_cli -- --nocapture`
- `cargo test --test capacity_assessment_excel_report_cli -- --nocapture`

Expected: PASS

**Step 3: Run full suite**

Run: `cargo test`

Expected: PASS with only unchanged pre-existing warnings if any.

### Task 5: Update records and handoff notes

**Files:**
- Modify: `D:\Rust\Excel_Skill\progress.md`
- Modify: `D:\Rust\Excel_Skill\findings.md`
- Modify: `D:\Rust\Excel_Skill\task_plan.md`
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Record what changed**

- Log the new `diagnostics_report_excel_report` Tool.
- Record the workbook-first delivery direction.
- Record the rule that section failures degrade the workbook rather than blocking export.

**Step 2: Record remaining scope limits**

- First version is table-first.
- Charts remain a future slice.

**Step 3: Invoke task journal workflow**

- Append the required task completion note after verification.
