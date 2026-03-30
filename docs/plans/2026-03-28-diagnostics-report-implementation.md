# Diagnostics Report Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Rust high-level `diagnostics_report` Tool that composes correlation, outlier, distribution, and trend diagnostics into one stable JSON delivery contract.

**Architecture:** Keep the existing four statistical operators unchanged as the calculation core, and add one new high-level op that reuses their outputs, catches per-section failures, and emits a unified summary packet. The dispatcher and catalog should expose the new Tool through the existing Rust binary-first path without reopening the current architecture.

**Tech Stack:** Rust, serde, Polars, existing SheetMind tool dispatcher, cargo test

---

### Task 1: Lock the combined Tool contract with failing CLI tests

**Files:**
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\diagnostics_report_cli.rs`
- Reference: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\common\mod.rs`

**Step 1: Write the failing test**

- Add one catalog test that asserts `diagnostics_report` appears in the Rust tool catalog.
- Add one end-to-end test that passes a seeded `result_ref` plus all four section configs and expects:
  - `status == "ok"`
  - `data.report_status == "ok"`
  - `data.section_count == 4`
  - `data.available_section_count == 4`
  - `data.sections` contains `correlation / outlier / distribution / trend`
  - `data.key_findings` and `data.recommended_actions` are both non-empty
- Add one degradation test that passes an invalid trend column while keeping other sections valid and expects:
  - `status == "ok"`
  - `data.report_status == "degraded"`
  - one warning mentioning `trend_analysis`
  - other sections still return data

**Step 2: Run test to verify it fails**

Run: `cargo test --test diagnostics_report_cli -- --nocapture`

Expected: FAIL because the Tool does not exist yet.

### Task 2: Implement the high-level diagnostics_report op

**Files:**
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\diagnostics_report.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\mod.rs`

**Step 1: Write minimal request/response structs**

- Define:
  - `DiagnosticsReportRequest`
  - `DiagnosticsCorrelationRequest`
  - `DiagnosticsOutlierRequest`
  - `DiagnosticsDistributionRequest`
  - `DiagnosticsTrendRequest`
  - `DiagnosticsSectionStatus`
  - `DiagnosticsReportResult`

**Step 2: Implement section orchestration**

- Reuse:
  - `correlation_analysis()`
  - `outlier_detection()`
  - `distribution_analysis()`
  - `trend_analysis()`
- Catch each section error independently.
- Convert section errors into report warnings instead of failing the whole Tool.
- Return an error only when the request contains no section configuration at all.

**Step 3: Implement unified summary assembly**

- Build:
  - `report_status`
  - `overall_summary`
  - `key_findings`
  - `recommended_actions`
  - `warnings`
- Keep the first version intentionally small and deterministic.

**Step 4: Run targeted test**

Run: `cargo test --test diagnostics_report_cli -- --nocapture`

Expected: still failing until dispatcher/catalog wiring is complete.

### Task 3: Wire the Tool into the Rust catalog and dispatcher

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\catalog.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\dispatcher.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\dispatcher\analysis_ops.rs`

**Step 1: Add catalog entry**

- Register `diagnostics_report` in the tool catalog.

**Step 2: Add dispatcher handler**

- Parse the request into the new strong type.
- Reuse `load_table_for_analysis()` so the Tool can consume `result_ref`.
- Keep the session sync path inside the existing analysis stage.

**Step 3: Run targeted test**

Run: `cargo test --test diagnostics_report_cli -- --nocapture`

Expected: PASS

### Task 4: Run focused regression

**Files:**
- No new files required

**Step 1: Run related suites**

Run:

- `cargo test --test diagnostics_report_cli -- --nocapture`
- `cargo test --test stat_diagnostics_cli -- --nocapture`
- `cargo test --test capacity_assessment_excel_report_cli -- --nocapture`

Expected: PASS

**Step 2: Run full suite**

Run: `cargo test`

Expected: PASS with only unchanged pre-existing warnings if any.

### Task 5: Update records and handoff notes

**Files:**
- Modify: `D:\Rust\Excel_Skill\progress.md`
- Modify: `D:\Rust\Excel_Skill\findings.md`
- Modify: `D:\Rust\Excel_Skill\task_plan.md`
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Record what changed**

- Log the new `diagnostics_report` Tool.
- Record the “section failure degrades, whole report survives” rule.
- Record any remaining scope limits.

**Step 2: Invoke task journal workflow**

- Append the required task completion note after verification.
