# ETF Regression Head A2 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add ETF regression-head end-to-end contract coverage for the A2 slice so we can verify treasury and gold ETF return-head training before moving to heavier precision work.

**Architecture:** Keep this round incremental. First add failing CLI contract tests for `treasury_etf` and `gold_etf` `return_head` runs, then patch `security_scorecard_training` only if the new tests expose a real ETF-specific gap. Finish with narrow regression verification on the training and runtime slices that consume the training artifact.

**Tech Stack:** Rust, Cargo CLI integration tests, JSON tool dispatcher, SQLite runtime fixtures

---

### Task 1: Add treasury ETF regression-head contract test

**Files:**
- Modify: `tests/security_scorecard_training_cli.rs`

**Step 1: Write the failing test**

Add one end-to-end CLI test that trains:
- `instrument_scope = ETF`
- `instrument_subscope = treasury_etf` (resolved from request context)
- `target_head = return_head`

Assert:
- `model_registry.instrument_subscope == "treasury_etf"`
- `metrics_summary_json.target_mode == "regression"`
- `valid/test baseline_rmse` exist
- `readiness_assessment.regression_quality_status` exists
- artifact `model_id == "a_share_etf_treasury_etf_10d_return_head"`
- treasury proxy features remain in group `X`

**Step 2: Run test to verify it fails**

Run:
```powershell
cargo test --test security_scorecard_training_cli security_scorecard_training_supports_treasury_etf_return_head_contract -- --nocapture --test-threads=1
```

Expected:
- FAIL if ETF regression contract is not fully wired

**Step 3: Write minimal implementation**

Only if the test fails, patch:
- `src/ops/security_scorecard_training.rs`

Expected implementation scope:
- ETF regression artifact identity
- ETF proxy feature routing
- ETF regression summary fields

**Step 4: Run test to verify it passes**

Run the same targeted command and require PASS.

### Task 2: Add gold ETF regression-head contract test

**Files:**
- Modify: `tests/security_scorecard_training_cli.rs`
- Modify: `src/ops/security_scorecard_training.rs` only if required by RED result

**Step 1: Write the failing test**

Add one end-to-end CLI test that trains:
- `instrument_scope = ETF`
- `instrument_subscope = gold_etf`
- `target_head = return_head`

Assert:
- `model_registry.instrument_subscope == "gold_etf"`
- artifact `model_id == "a_share_etf_gold_etf_10d_return_head"`
- gold proxy features remain in group `X`
- regression baseline metrics and readiness quality status are present

**Step 2: Run test to verify it fails**

Run:
```powershell
cargo test --test security_scorecard_training_cli security_scorecard_training_supports_gold_etf_return_head_contract -- --nocapture --test-threads=1
```

Expected:
- FAIL if the treasury fix did not generalize to gold ETF

**Step 3: Write minimal implementation**

Only patch shared ETF regression logic if the new test proves a real gold-specific gap.

**Step 4: Run test to verify it passes**

Run the same targeted command and require PASS.

### Task 3: Run targeted regression verification

**Files:**
- Verify: `tests/security_scorecard_training_cli.rs`
- Verify: `tests/security_master_scorecard_cli.rs`
- Verify: `tests/security_chair_resolution_cli.rs`

**Step 1: Run the training CLI suite**

Run:
```powershell
cargo test --test security_scorecard_training_cli -- --nocapture --test-threads=1
```

Expected:
- PASS for the full training CLI slice

**Step 2: Run downstream runtime slices**

Run:
```powershell
cargo test --test security_master_scorecard_cli -- --nocapture --test-threads=1
cargo test --test security_chair_resolution_cli -- --nocapture --test-threads=1
```

Expected:
- PASS for both downstream slices

### Task 4: Update execution memory

**Files:**
- Modify: `task_plan.md`
- Modify: `progress.md`
- Modify: `.trae/CHANGELOG_TASK.md`

**Step 1: Record what landed**

Capture:
- added ETF regression contract coverage
- whether product code changed or tests alone were sufficient
- exact verification commands that passed

**Step 2: Record remaining gaps**

Capture:
- the next precision bottleneck after A2
- any residual risk around ETF sample richness or runtime promotion quality
