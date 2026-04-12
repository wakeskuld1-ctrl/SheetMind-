# Balanced Scorecard Data Thickening Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Thicken governed stock and ETF data until the formal securities stack can emit auditable conclusions and balanced scorecards on real data rather than structure-only candidate outputs.

**Architecture:** Add ETF-native information synthesis to the fullstack/evidence path, extend representative validation slices to longer replay windows, then re-run the governed multi-head batch and verify the formal scorecard/chair outputs on thicker data.

**Tech Stack:** Rust, SQLite runtime stores, CLI integration tests, governed validation slices, multi-head scorecard artifacts

---

### Task 1: Lock ETF information synthesis behavior with failing tests

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\security_analysis_fullstack_cli.rs`
- Modify: `D:\Rust\Excel_Skill\tests\security_chair_resolution_cli.rs`

**Steps:**
- Write RED tests that require ETF fullstack output to expose governed ETF-native information context instead of collapsing into stock-only `unavailable`.
- Write RED tests that require ETF final chair output to stop downgrading solely because stock-style information is missing when ETF proxy history is complete.
- Run focused tests and confirm failure.

### Task 2: Implement ETF-native information synthesis in the fullstack path

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_analysis_fullstack.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_evidence_bundle.rs`

**Steps:**
- Add ETF-specific governed information synthesis based on external proxy history and ETF sub-pool identity.
- Keep stock behavior unchanged.
- Ensure integrated conclusion and risk flags reflect ETF-native evidence instead of treating ETFs as missing stock-only layers.
- Run focused tests and turn Task 1 green.

### Task 3: Lock longer-horizon validation slice requirements with failing tests

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\security_real_data_validation_backfill_cli.rs`

**Steps:**
- Add RED tests that require validation slice refresh to preserve ETF-native market/sector semantics while supporting later end dates.
- Add assertions for manifest/runtime paths that will be used by the longer-horizon live batch.
- Run focused tests and confirm failure.

### Task 4: Extend validation slice refresh for thicker replay windows

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_real_data_validation_backfill.rs`

**Steps:**
- Implement the minimal refresh behavior needed for longer governed replay windows without breaking existing manifests.
- Preserve ETF-native `market_profile` and `sector_profile` semantics.
- Run focused tests and turn Task 3 green.

### Task 5: Re-run governed multi-head live batch on thicker data and verify balanced-scorecard outputs

**Files:**
- Modify: `D:\Rust\Excel_Skill\docs\execution-notes-2026-04-13-multi-head-live-validation.md`
- Create or update runtime outputs under: `D:\Rust\Excel_Skill\.excel_skill_runtime`

**Steps:**
- Refresh representative stock + ETF validation slices with thicker windows.
- Re-run governed multi-head training for representative assets.
- Re-run `security_master_scorecard` and `security_chair_resolution`.
- Persist updated batch summaries for audit.

### Task 6: Verify, document, and journal

**Files:**
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`
- Modify: `D:\Rust\Excel_Skill\docs\execution-notes-2026-04-13-multi-head-live-validation.md`

**Steps:**
- Run focused regressions for:
  - `security_analysis_fullstack_cli`
  - `security_real_data_validation_backfill_cli`
  - `security_master_scorecard_cli`
  - `security_chair_resolution_cli`
- Summarize:
  - final conclusion status
  - balanced scorecard availability
  - remaining blockers
- Append task journal entry.
