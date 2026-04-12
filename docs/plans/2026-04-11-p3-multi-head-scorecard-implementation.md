# P3 Multi-Head Scorecard Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Upgrade the governed securities training chain from a single `direction_head` classifier into a multi-head analysis backbone that can produce return, drawdown, and path-quality signals and feed them into the formal master scorecard and chair-resolution chain.

**Architecture:** Extend the existing `security_forward_outcome -> security_scorecard_training -> security_master_scorecard -> security_chair_resolution` flow instead of creating a parallel pipeline. Keep the current scorecard artifact schema backward-compatible for `direction_head`, add explicit head semantics and fit metrics for new heads, and let the master scorecard consume model-backed quantitative context without pretending historical replay and trained heads are the same thing.

**Tech Stack:** Rust, serde/serde_json, chrono, existing CLI tool dispatcher, current test fixtures under `tests/runtime_fixtures`

---

### Task 1: Lock multi-head training contracts with failing tests

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\security_scorecard_training_cli.rs`
- Modify: `D:\Rust\Excel_Skill\tests\security_master_scorecard_cli.rs`
- Modify: `D:\Rust\Excel_Skill\tests\security_chair_resolution_cli.rs`

**Step 1: Write the failing tests**

- Add a CLI test proving `security_scorecard_training` accepts `return_head`, persists a registry/artifact with the new head, and emits regression metrics.
- Add a CLI test proving `security_master_scorecard` upgrades aggregation status when trained return/drawdown/path context is present.
- Add a CLI test proving `security_chair_resolution` references trained master-score context instead of only `score_status`.

**Step 2: Run tests to verify they fail**

Run:

```powershell
cargo test --test security_scorecard_training_cli security_scorecard_training_supports_return_head_with_regression_metrics -- --nocapture
cargo test --test security_master_scorecard_cli security_master_scorecard_uses_multi_head_quant_context_when_available -- --nocapture
cargo test --test security_chair_resolution_cli security_chair_resolution_references_multi_head_master_scorecard_context -- --nocapture
```

Expected: FAIL because current training request validation only allows `direction_head`, current master scorecard does not consume trained multi-head context, and chair reasoning does not mention multi-head evidence.

### Task 2: Implement P3-1 and P3-2 in the training layer

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_forward_outcome.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_scorecard_training.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_scorecard.rs`

**Step 1: Write the minimal code**

- Add governed target-head support for:
  - `direction_head`
  - `return_head`
  - `drawdown_head`
  - `upside_first_head`
  - `stop_first_head`
  - `path_quality_head`
- Map each head to a stable label source:
  - `positive_return`
  - `forward_return`
  - `max_drawdown`
  - `hit_upside_first`
  - `hit_stop_first`
  - derived `path_quality_score`
- Keep `direction_head` behavior backward-compatible.
- Extend artifact metadata and metrics summary to describe regression/classification head type and head-specific fit outputs.

**Step 2: Run focused tests**

Run:

```powershell
cargo test --test security_scorecard_training_cli security_scorecard_training_supports_return_head_with_regression_metrics -- --nocapture
cargo test --test security_scorecard_training_cli -- --nocapture
```

Expected: PASS with new metrics and artifact metadata.

### Task 3: Implement P3-3 and P3-4 head aggregation

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_scorecard_training.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_master_scorecard.rs`

**Step 1: Write the minimal code**

- Define a stable `path_quality_score` label in training based on `max_runup`, `max_drawdown`, `hit_upside_first`, and `hit_stop_first`.
- Add helper logic so `master_scorecard` can consume optional trained summaries for:
  - expected return
  - expected drawdown
  - path-quality signal
- Preserve the current historical replay fallback when no trained head is available.

**Step 2: Run focused tests**

Run:

```powershell
cargo test --test security_master_scorecard_cli security_master_scorecard_uses_multi_head_quant_context_when_available -- --nocapture
cargo test --test security_master_scorecard_cli -- --nocapture
```

Expected: PASS with upgraded aggregation status and non-replay-only quant context.

### Task 4: Wire P3-5 into chair resolution

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_chair_resolution.rs`
- Modify: `D:\Rust\Excel_Skill\tests\security_chair_resolution_cli.rs`

**Step 1: Write the minimal code**

- Route master-scorecard multi-head summaries into chair reasoning and execution constraints.
- Keep the existing training guard, but let a trained multi-head context raise explanation quality and confidence bounds.
- Do not let chair resolution overstate model confidence when only partial heads are ready.

**Step 2: Run focused tests**

Run:

```powershell
cargo test --test security_chair_resolution_cli security_chair_resolution_references_multi_head_master_scorecard_context -- --nocapture
cargo test --test security_chair_resolution_cli -- --nocapture
```

Expected: PASS with formal chair output referencing multi-head quant context.

### Task 5: Run adjacent regressions and document the closeout

**Files:**
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`
- Modify if needed: `D:\Rust\Excel_Skill\docs\plans\2026-04-11-p3-multi-head-scorecard-implementation.md`

**Step 1: Run adjacent regressions**

Run:

```powershell
cargo test --test security_scorecard_training_cli -- --nocapture
cargo test --test security_master_scorecard_cli -- --nocapture
cargo test --test security_chair_resolution_cli -- --nocapture
cargo test --test security_decision_submit_approval_cli -- --nocapture
```

Expected: PASS for the relevant governed chain.

**Step 2: Append task journal entry**

- Add a new dated entry to `.trae/CHANGELOG_TASK.md`.
- Record what changed, why it changed, remaining gaps, and suggested follow-up tests.

