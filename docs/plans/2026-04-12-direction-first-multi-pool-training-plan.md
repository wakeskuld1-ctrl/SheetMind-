# Direction-First Multi-Pool Training Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build and run a governed seven-hour training workflow for bank equities and ETF sub-pools across `5d/10d/15d/30d`, with direction quality ranked first and regression quality used as a secondary optimization signal.

**Architecture:** Keep the existing `security_scorecard_training -> security_scorecard_refit -> security_shadow_evaluation -> security_model_promotion` chain intact. Add a thin orchestration layer that prepares pool-specific requests, runs staged comparisons, records metrics, and selects survivors based on direction-first ranking rules.

**Tech Stack:** Rust, Cargo CLI tool dispatcher, SQLite runtime stores, JSON artifacts, PowerShell execution, long-running local training logs

---

### Task 1: Inspect and lock the training entrypoints

**Files:**
- Inspect: `D:\Rust\Excel_Skill\src\tools\dispatcher.rs`
- Inspect: `D:\Rust\Excel_Skill\src\ops\security_scorecard_training.rs`
- Inspect: `D:\Rust\Excel_Skill\src\ops\security_scorecard_refit_run.rs`
- Inspect: `D:\Rust\Excel_Skill\src\ops\security_shadow_evaluation.rs`
- Inspect: `D:\Rust\Excel_Skill\src\ops\security_model_promotion.rs`
- Inspect: `D:\Rust\Excel_Skill\docs\plans\2026-04-12-direction-first-multi-pool-training-design.md`

**Step 1: Verify the exact training chain inputs**

Confirm:
- how `security_scorecard_training` is invoked
- what metric fields can be used for ranking
- where artifacts and registry outputs are written

**Step 2: Verify the direction-first ranking inputs exist**

Confirm:
- `accuracy`
- `auc`
- `directional_hit_rate`
- `rmse_improvement_vs_baseline`
- readiness status fields

**Step 3: Record the orchestration write targets**

Decide:
- runtime root
- staged output directory
- summary JSON or markdown path
- long-run log location

### Task 2: Add failing contract coverage for the orchestration layer

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\security_scorecard_training_cli.rs`
- Modify: `D:\Rust\Excel_Skill\tests\security_shadow_evaluation_cli.rs`
- Modify: `D:\Rust\Excel_Skill\tests\security_model_promotion_cli.rs`

**Step 1: Write the failing test for multi-horizon staged ranking**

Add a RED test that requires:
- multiple horizons can be evaluated in one governed round
- direction metrics are ranked before regression metrics
- weaker candidates are marked as non-survivors

**Step 2: Run the focused test to verify it fails**

Run:
```powershell
cargo test --test security_scorecard_training_cli <new_test_name> -- --nocapture --test-threads=1
```

Expected:
- FAIL because the orchestration or summary contract does not exist yet

**Step 3: Write the failing test for staged summary export**

Add a RED test that requires:
- the seven-hour training workflow to emit a summarized comparison output per pool and horizon

**Step 4: Run the focused test to verify it fails**

Run:
```powershell
cargo test --test security_shadow_evaluation_cli <new_test_name> -- --nocapture --test-threads=1
```

Expected:
- FAIL because the staged export path is not yet implemented

### Task 3: Implement the orchestration entrypoint

**Files:**
- Create: `D:\Rust\Excel_Skill\src\ops\direction_first_training_run.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\mod.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\dispatcher.rs`

**Step 1: Add a new governed orchestration request DTO**

Implement:
- pool list
- horizon list
- runtime root
- stage budget or execution mode
- ranking policy

**Step 2: Add stage execution logic**

Implement:
- Stage 1 baseline pass
- Stage 2 direction-focused filtering
- Stage 3 survivor continuation
- Stage 4 final export

**Step 3: Add direction-first ranking**

Implement:
- primary sort on direction accuracy
- secondary sort on direction AUC
- tertiary tie-break on regression directional hit rate
- quaternary tie-break on RMSE improvement vs baseline

**Step 4: Add explicit English comments**

Add timestamped English comments beside each governed ranking block explaining:
- reason
- purpose

### Task 4: Implement output persistence and resumability

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\direction_first_training_run.rs`
- Create: `D:\Rust\Excel_Skill\docs\execution-notes-2026-04-12-direction-first-training.md`

**Step 1: Persist staged summary outputs**

Write:
- one summary file per stage
- one final comparison file
- one best-artifact manifest

**Step 2: Persist resumable state**

Write:
- completed pool/horizon pairs
- survivor list
- best-so-far metrics

**Step 3: Record the formal execution notes**

Capture:
- command used
- runtime root
- artifact location
- follow-up interpretation notes

### Task 5: Verify the orchestration slice

**Files:**
- Verify: `D:\Rust\Excel_Skill\tests\security_scorecard_training_cli.rs`
- Verify: `D:\Rust\Excel_Skill\tests\security_shadow_evaluation_cli.rs`
- Verify: `D:\Rust\Excel_Skill\tests\security_model_promotion_cli.rs`
- Verify: `D:\Rust\Excel_Skill\tests\security_master_scorecard_cli.rs`
- Verify: `D:\Rust\Excel_Skill\tests\security_chair_resolution_cli.rs`

**Step 1: Run focused orchestration tests**

Run:
```powershell
cargo test --test security_scorecard_training_cli <new_test_name> -- --nocapture --test-threads=1
cargo test --test security_shadow_evaluation_cli <new_test_name> -- --nocapture --test-threads=1
```

Expected:
- PASS

**Step 2: Run the training chain regression suites**

Run:
```powershell
cargo test --test security_scorecard_training_cli -- --nocapture --test-threads=1
cargo test --test security_master_scorecard_cli -- --nocapture --test-threads=1
cargo test --test security_chair_resolution_cli -- --nocapture --test-threads=1
```

Expected:
- PASS

### Task 6: Execute the seven-hour run

**Files:**
- Modify: `D:\Rust\Excel_Skill\docs\execution-notes-2026-04-12-direction-first-training.md`
- Modify: `D:\Rust\Excel_Skill\.excel_skill_runtime\...`

**Step 1: Launch the staged run**

Run the new orchestration tool with:
- pools:
  - bank equities
  - treasury ETF
  - gold ETF
  - cross-border ETF
  - equity ETF
- horizons:
  - `5`
  - `10`
  - `15`
  - `30`

**Step 2: Record stage checkpoints**

At each stage record:
- survivor list
- dropped combinations
- best metrics

**Step 3: Export best artifacts**

Persist:
- best direction artifact
- best return artifact
- summary comparison sheet

### Task 7: Update planning memory and task journal

**Files:**
- Modify: `D:\Rust\Excel_Skill\task_plan.md`
- Modify: `D:\Rust\Excel_Skill\progress.md`
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Record what was built**

Capture:
- orchestration entrypoint
- ranking policy
- resumability behavior

**Step 2: Record what the seven-hour run achieved**

Capture:
- strongest pools
- weakest horizons
- whether direction-first ranking improved practical selection quality

**Step 3: Record remaining risk**

Capture:
- data thickness limits
- pool-specific instability
- runtime model landing gaps
