# Future 180D Prediction Mode Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a governed `prediction_mode` so the stack can predict the next 180 days from `2026-04-12` using regression, risk, and clustering lines without requiring future replay rows.

**Architecture:** Extend `security_master_scorecard` and `security_chair_resolution` with a first-class prediction path that consumes existing multi-head artifacts plus governed clustering / analog outputs, while preserving the current replay path for backward compatibility.

**Tech Stack:** Rust, governed scorecard artifacts, master scorecard aggregation, chair resolution, CLI integration tests

---

### Task 1: Lock prediction-mode contracts with failing tests

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\security_master_scorecard_cli.rs`
- Modify: `D:\Rust\Excel_Skill\tests\security_chair_resolution_cli.rs`

**Step 1: Write the failing tests**
- Add a RED test that requires `security_master_scorecard` to accept `prediction_mode = prediction` and emit a 180-day prediction summary without future replay rows.
- Add a RED test that requires `security_chair_resolution` to mention prediction-mode regression / risk / clustering evidence.

**Step 2: Run tests to verify they fail**

Run:
```powershell
cargo test --test security_master_scorecard_cli security_master_scorecard_supports_prediction_mode_180d -- --nocapture
cargo test --test security_chair_resolution_cli security_chair_resolution_reads_prediction_mode_180d_context -- --nocapture
```

Expected:
- missing fields or contract assertion failures

### Task 2: Add prediction summary objects to master scorecard

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_master_scorecard.rs`

**Step 1: Write minimal implementation**
- Add request support for a governed `prediction_mode`
- Add prediction summary structures for:
  - regression line
  - risk line
  - clustering / analog line
- Keep replay path unchanged

**Step 2: Run focused tests**

Run:
```powershell
cargo test --test security_master_scorecard_cli security_master_scorecard_supports_prediction_mode_180d -- --nocapture
```

Expected:
- PASS

### Task 3: Add governed clustering / analog prediction synthesis

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_master_scorecard.rs`
- Reference: existing analog-study contracts and cluster utilities already in the repo

**Step 1: Write the failing test**
- Require prediction-mode master scorecard output to include governed cluster / analog summary.

**Step 2: Implement minimal synthesis**
- Derive a deterministic cluster label from current governed feature vectors
- Surface analog summary fields using the repo's existing signal / analog concepts

**Step 3: Re-run tests**

Run:
```powershell
cargo test --test security_master_scorecard_cli security_master_scorecard_supports_prediction_mode_180d -- --nocapture
```

Expected:
- PASS with cluster / analog fields present

### Task 4: Make chair resolution consume prediction-mode context

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_chair_resolution.rs`

**Step 1: Implement minimal consumption**
- Update chair reasoning and execution constraints so they describe:
  - expected return
  - expected drawdown
  - cluster / analog context
- Keep current replay reasoning intact when `prediction_mode` is not requested

**Step 2: Run focused tests**

Run:
```powershell
cargo test --test security_chair_resolution_cli security_chair_resolution_reads_prediction_mode_180d_context -- --nocapture
```

Expected:
- PASS

### Task 5: Run adjacent regressions

**Files:**
- No new files required unless fixtures need updates

**Steps:**
- Run:
```powershell
cargo test --test security_master_scorecard_cli -- --nocapture
cargo test --test security_chair_resolution_cli -- --nocapture
```

Expected:
- all relevant tests green

### Task 6: Document and journal

**Files:**
- Modify: `D:\Rust\Excel_Skill\docs\execution-notes-2026-04-13-multi-head-live-validation.md`
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Steps:**
- Add the new `prediction_mode` explanation and current 180-day future-looking contract
- Append a task-journal entry with:
  - what changed
  - why it changed
  - remaining gaps
  - focused verification
