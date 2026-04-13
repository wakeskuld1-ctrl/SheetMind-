# Regression Head Precision Scheme C Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Tighten regression-head governance and add one lightweight robustness improvement without changing the surrounding training architecture.

**Architecture:** Keep the existing `security_scorecard_training` pipeline intact. Add regression-specific quality metrics and readiness gating, then improve regression bin predictions by shrinking thin-bin means toward the global baseline. Verify the slice through TDD with focused CLI and unit regressions.

**Tech Stack:** Rust, Cargo CLI tests, JSON metrics summaries, SQLite runtime fixtures

---

### Task 1: Lock the new regression governance contract with failing tests

**Files:**
- Modify: `tests/security_scorecard_training_cli.rs`
- Modify: `src/ops/security_scorecard_training.rs` (test module only in this task)

**Step 1: Write a failing CLI regression contract test**

- Extend the regression-head CLI assertions to require:
  - `baseline_rmse`
  - `rmse_improvement_vs_baseline`
  - `regression_quality_status`

**Step 2: Run the focused test to verify it fails**

Run:
```powershell
cargo test --test security_scorecard_training_cli security_scorecard_training_supports_return_head_with_regression_metrics -- --nocapture --test-threads=1
```

Expected:
- FAIL because the new regression metrics / readiness field do not exist yet

**Step 3: Write a failing unit test for regression shrinkage**

- Add a unit test in `src/ops/security_scorecard_training.rs` that proves a thin-support regression bin does not keep the raw extreme mean unchanged.

**Step 4: Run the focused unit test to verify it fails**

Run:
```powershell
cargo test regression_prediction_bins_shrink_thin_support_toward_baseline -- --nocapture
```

Expected:
- FAIL because shrinkage logic is not implemented yet

### Task 2: Implement the minimal regression precision upgrade

**Files:**
- Modify: `src/ops/security_scorecard_training.rs`

**Step 1: Add regression bin shrinkage**

- Compute a baseline-aware shrinkage helper for regression bin predictions.
- Apply it to categorical and numeric regression bins using local support counts only.

**Step 2: Add regression baseline comparison metrics**

- Extend regression split evaluation to emit:
  - `baseline_mae`
  - `baseline_rmse`
  - `rmse_improvement_vs_baseline`

**Step 3: Tighten regression readiness**

- Add `regression_quality_status` to readiness output.
- Require positive baseline improvement plus directional usefulness before regression heads become `shadow_candidate_ready`.

**Step 4: Keep comments explicit**

- Add English comments with timestamp, reason, and purpose beside each new governed behavior block.

### Task 3: Verify the slice and update working memory

**Files:**
- Modify: `task_plan.md`
- Modify: `findings.md`
- Modify: `progress.md`
- Modify: `.trae/CHANGELOG_TASK.md`

**Step 1: Run focused regression tests**

Run:
```powershell
cargo test --test security_scorecard_training_cli security_scorecard_training_supports_return_head_with_regression_metrics -- --nocapture --test-threads=1
cargo test regression_prediction_bins_shrink_thin_support_toward_baseline -- --nocapture
```

Expected:
- PASS

**Step 2: Run broader regression suites**

Run:
```powershell
cargo test --test security_scorecard_training_cli -- --nocapture --test-threads=1
cargo test --test security_master_scorecard_cli -- --nocapture --test-threads=1
cargo test --test security_chair_resolution_cli -- --nocapture --test-threads=1
```

Expected:
- PASS

**Step 3: Update planning memory and task journal**

- Record what changed, why it changed, and what still remains for the next precision slice.
