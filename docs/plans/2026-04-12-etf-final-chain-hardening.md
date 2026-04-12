# ETF Final Chain Hardening Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** make ETF governed proxy history, ETF-native validation slices, and ETF sub-scope artifacts reach the final `security_chair_resolution` layer without degrading for the wrong reasons.

**Architecture:** reuse one governed ETF proxy hydration path across snapshot, committee, and forward outcome; enrich ETF validation slices with ETF-native environment symbols; then train and bind ETF sub-scope scorecard artifacts before rerunning the formal chair path.

**Tech Stack:** Rust, serde_json, SQLite runtime stores, cargo test, governed stock tool chain

---

### Task 1: Lock deep-chain ETF proxy hydration with failing tests

**Files:**
- Modify: `tests/security_chair_resolution_cli.rs`
- Modify: `tests/security_master_scorecard_cli.rs`

**Step 1: Write the failing tests**

Add ETF-specific tests that:
- seed governed external proxy history,
- call `security_chair_resolution`,
- assert ETF proxy fields appear in `scorecard.raw_feature_snapshot`.

**Step 2: Run tests to verify they fail**

Run:

```powershell
cargo test --test security_chair_resolution_cli historical_proxy -- --nocapture
```

Expected: FAIL because the deeper chain does not hydrate ETF historical proxy inputs yet.

**Step 3: Write minimal implementation**

Modify committee / forward-outcome input preparation so they use the same governed proxy hydration path as feature snapshot.

**Step 4: Run tests to verify they pass**

Run:

```powershell
cargo test --test security_chair_resolution_cli historical_proxy -- --nocapture
```

Expected: PASS

### Task 2: Lock ETF-native validation-slice enrichment with failing tests

**Files:**
- Modify: `tests/security_real_data_validation_backfill_cli.rs`
- Modify: `src/ops/security_real_data_validation_backfill.rs`

**Step 1: Write the failing tests**

Add tests that require:
- treasury ETF validation slices to sync `511060.SH` as an ETF peer environment symbol,
- equity ETF validation slices to keep ETF-native profile semantics in the manifest/runtime setup.

**Step 2: Run tests to verify they fail**

Run:

```powershell
cargo test --test security_real_data_validation_backfill_cli etf -- --nocapture
```

Expected: FAIL because the current slice builder only syncs the explicitly listed symbols.

**Step 3: Write minimal implementation**

Extend validation-slice symbol collection with ETF-aware environment enrichment.

**Step 4: Run tests to verify they pass**

Run:

```powershell
cargo test --test security_real_data_validation_backfill_cli etf -- --nocapture
```

Expected: PASS

### Task 3: Lock ETF sub-scope artifact generation with failing tests

**Files:**
- Modify: `tests/security_scorecard_training_cli.rs`
- Modify: `src/ops/security_scorecard_training.rs`

**Step 1: Write the failing tests**

Add ETF-subscope training tests that require:
- treasury ETF artifact IDs to carry treasury semantics,
- gold / cross-border / equity ETF artifacts to carry the matching sub-scope and ETF feature family.

**Step 2: Run tests to verify they fail**

Run:

```powershell
cargo test --test security_scorecard_training_cli etf -- --nocapture
```

Expected: FAIL because the current ETF training coverage does not yet prove final-chair-ready artifact routing for all sub-pools.

**Step 3: Write minimal implementation**

Make ETF training/runtime outputs expose and preserve the right sub-scope artifact identity for final-chair binding.

**Step 4: Run tests to verify they pass**

Run:

```powershell
cargo test --test security_scorecard_training_cli etf -- --nocapture
```

Expected: PASS

### Task 4: Rerun final chair-resolution path for all ETF sub-pools

**Files:**
- Modify: `tests/security_chair_resolution_cli.rs`

**Step 1: Write the failing tests**

Add final-chain tests that bind ETF-subscope-correct artifacts and assert the chair path no longer degrades for proxy propagation reasons.

**Step 2: Run tests to verify they fail**

Run:

```powershell
cargo test --test security_chair_resolution_cli etf -- --nocapture
```

Expected: FAIL until the artifact binding and deep-chain hydration are both correct.

**Step 3: Write minimal implementation**

Finish any remaining runtime fixes needed for ETF final-chain consumption.

**Step 4: Run tests to verify they pass**

Run:

```powershell
cargo test --test security_chair_resolution_cli etf -- --nocapture
```

Expected: PASS

### Task 5: Focused regression and live ETF reruns

**Files:**
- Modify: none required unless regressions expose new gaps

**Step 1: Run focused regressions**

Run:

```powershell
cargo test --test security_feature_snapshot_cli -- --nocapture
cargo test --test security_scorecard_training_cli -- --nocapture
cargo test --test security_real_data_validation_backfill_cli -- --nocapture
cargo test --test security_chair_resolution_cli -- --nocapture
```

Expected: PASS

**Step 2: Rerun live ETF chair flows**

Rerun:
- `511010.SH`
- `518880.SH`
- `513500.SH`
- `512800.SH`

with governed ETF proxy history and ETF-subscope-correct artifact binding.

**Step 3: Record the new final conclusions**

Capture:
- chair action
- majority vote
- scorecard status
- remaining blockers

### Task 6: Task journal

**Files:**
- Modify: `.trae/CHANGELOG_TASK.md`

**Step 1: Append a task journal entry**

Record:
- what changed
- what was verified
- what still remains

**Step 2: Sanity-check the final paths**

Make sure the plan outcome references:
- ETF deep-chain hydration
- ETF validation-slice enrichment
- ETF sub-scope artifacts
