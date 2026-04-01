# Capacity Assessment Excel Report Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a dedicated `capacity_assessment_excel_report` Tool that turns capacity inputs into a deliverable Excel workbook and can optionally export a final `.xlsx` in one call.

**Architecture:** Reuse the existing capacity-analysis core for calculation, then map the structured result into a workbook draft with four fixed sheets. Keep export optional but first-class by letting the Tool return `workbook_ref` and write `output_path` when present.

**Tech Stack:** Rust, serde, polars, rust_xlsxwriter, existing workbook draft store / tool dispatcher stack

---

### Task 1: Add the first failing CLI tests

**Files:**
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\capacity_assessment_excel_report_cli.rs`
- Reference: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\common\mod.rs`

**Step 1: Write the failing test**

- Add a catalog exposure test for `capacity_assessment_excel_report`
- Add a test that passes Excel CPU/实例数据 plus `output_path` and expects:
  - `status == "ok"`
  - returned `workbook_ref`
  - returned `format == "xlsx"`
  - generated file exists
- Add a test that passes only `inventory_result` + `scenario_profile` and expects:
  - `evidence_level == "guidance_only"` or `partial`
  - exported file exists

**Step 2: Run test to verify it fails**

Run: `cargo test --test capacity_assessment_excel_report_cli -- --nocapture`

Expected: FAIL because tool is not registered / implemented yet

**Step 3: Commit**

Skip until implementation is green.

### Task 2: Implement workbook assembly for capacity report

**Files:**
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\capacity_assessment_excel_report.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\mod.rs`

**Step 1: Write minimal implementation skeleton**

- Define request struct:
  - report metadata
  - optional source table
  - optional inventory bridge inputs
  - inherited capacity request fields
  - optional `output_path`
- Define response struct:
  - `capacity_result`
  - `workbook_ref`
  - `sheet_names`
  - optional `output_path`
  - `format`

**Step 2: Implement calculation path**

- If `inventory_request` or `inventory_result` present, call `capacity_assessment_from_inventory`
- Else call `capacity_assessment`
- If no table source exists, use the existing empty-table fallback style so guidance-only still works

**Step 3: Implement workbook draft builder**

- Build four DataFrames:
  - conclusion
  - resource assessments
  - evidence and risks
  - missing inputs and actions
- Convert them into `WorkbookSheetInput`
- Persist via `WorkbookDraftStore`
- If `output_path` exists, call `export_excel_workbook`

**Step 4: Run targeted test**

Run: `cargo test --test capacity_assessment_excel_report_cli -- --nocapture`

Expected: partial progress, maybe still failing on dispatcher/catalog wiring

### Task 3: Wire the Tool into catalog and dispatcher

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\catalog.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\dispatcher\analysis_ops.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\dispatcher.rs`

**Step 1: Add catalog entry**

- Register `capacity_assessment_excel_report` in tool catalog

**Step 2: Add dispatcher handler**

- Parse request into the new strong typed request
- Reuse existing source loading path when a workbook source exists
- Allow no-source execution for guidance-only and inventory-only flows

**Step 3: Run targeted test**

Run: `cargo test --test capacity_assessment_excel_report_cli -- --nocapture`

Expected: PASS

### Task 4: Run broader regression

**Files:**
- No new files required

**Step 1: Run focused related suites**

Run:

- `cargo test --test capacity_assessment_cli -- --nocapture`
- `cargo test --test ssh_inventory_cli -- --nocapture`
- `cargo test --test capacity_assessment_from_inventory_cli -- --nocapture`

Expected: PASS

**Step 2: Run full test suite**

Run: `cargo test`

Expected: PASS

### Task 5: Update records and handoff notes

**Files:**
- Modify: `D:\Rust\Excel_Skill\progress.md`
- Modify: `D:\Rust\Excel_Skill\findings.md`
- Modify: `D:\Rust\Excel_Skill\task_plan.md`
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Record what changed**

- Add a brief progress entry
- Record any residual limitations

**Step 2: Invoke task journal workflow**

- Append the required task completion note after verification

