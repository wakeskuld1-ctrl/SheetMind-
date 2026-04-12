# P4 Path Events And History Backfill Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Turn the current multi-head securities chain into a path-aware and backfill-ready governed system by making `upside_first_head` / `stop_first_head` trainable with real event coverage and by introducing historical external-proxy backfill that can be joined into training samples instead of relying only on live manual inputs.

**Architecture:** Extend the existing `security_forward_outcome -> security_scorecard_training -> security_master_scorecard -> security_chair_resolution` chain instead of inventing a parallel model path. Add one governed historical proxy store and one governed backfill/import tool so live manual proxy inputs and historical backfill records share the same field contract, then let training and master scorecard consume those records by `symbol + as_of_date + instrument_subscope`.

**Tech Stack:** Rust, serde/serde_json, chrono, existing CLI dispatcher, runtime SQLite stores under `.excel_skill_runtime`, current securities tool contracts and tests under `tests/runtime_fixtures`

---

### Recommended Approach

**Approach A: Train path-event heads first, postpone historical proxy backfill**

- Pros:
  - Fastest route to getting `upside_first_head` and `stop_first_head` into the governed training CLI.
  - Least code surface in the short term.
- Cons:
  - Event heads will still be sample-thin because many ETF and cross-asset scenarios only exist as current manual proxy inputs.
  - We would likely need to rework the sample collector later.

**Approach B: Build historical proxy backfill first, postpone event heads**

- Pros:
  - Creates the right long-term data foundation.
  - Lowers the risk of training another round of toy event heads.
- Cons:
  - User-visible modeling progress is slower.
  - We would still not have path-event decisions feeding the chair or scorecard chain.

**Approach C: Do both in one governed phase, but in one sequence**

- Pros:
  - Best balance of correctness and momentum.
  - Backfill lands first enough to support stronger event heads, and event heads land in the same phase so the system becomes visibly more path-aware.
- Cons:
  - Broader change set than A or B alone.
  - Needs tight TDD discipline to avoid boundary spread.

**Recommendation:** Approach C. First add governed historical proxy backfill and dated sample joins, then lock and implement `upside_first_head` / `stop_first_head`, then wire path-event summaries into the formal master scorecard and chair chain.

---

### Task 1: Lock governed path-event and backfill contracts with failing tests

**Files:**
- Create: `D:\Rust\Excel_Skill\tests\security_external_proxy_backfill_cli.rs`
- Modify: `D:\Rust\Excel_Skill\tests\security_scorecard_training_cli.rs`
- Modify: `D:\Rust\Excel_Skill\tests\security_master_scorecard_cli.rs`

**Step 1: Write the failing tests**

- Add a CLI test proving a new governed tool can persist dated historical proxy records for:
  - `treasury_etf`
  - `gold_etf`
  - `cross_border_etf`
  - `equity_etf`
- Add training CLI tests proving:
  - `upside_first_head` is accepted and returns classification metrics.
  - `stop_first_head` is accepted and returns classification metrics.
- Add a master-scorecard CLI test proving path-event summary fields can be attached once event heads become available.

**Step 2: Run tests to verify they fail**

Run:

```powershell
cargo test --test security_external_proxy_backfill_cli -- --nocapture
cargo test --test security_scorecard_training_cli security_scorecard_training_supports_upside_first_head_with_classification_metrics -- --nocapture
cargo test --test security_scorecard_training_cli security_scorecard_training_supports_stop_first_head_with_classification_metrics -- --nocapture
cargo test --test security_master_scorecard_cli security_master_scorecard_attaches_path_event_context_when_available -- --nocapture
```

Expected: FAIL because there is no governed historical proxy backfill tool yet, path-event heads are not covered by dedicated regression tests, and master scorecard does not yet expose path-event head summaries.

### Task 2: Add the governed historical external-proxy backfill tool and dated runtime store

**Files:**
- Create: `D:\Rust\Excel_Skill\src\ops\security_external_proxy_backfill.rs`
- Create: `D:\Rust\Excel_Skill\tests\security_external_proxy_backfill_cli.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\stock.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\mod.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\catalog.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\dispatcher.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\dispatcher\stock_ops.rs`
- Modify: `D:\Rust\Excel_Skill\src\runtime\mod.rs`

**Step 1: Write the minimal implementation**

- Define a governed request and result contract for historical proxy backfill batches.
- Persist dated proxy records under a stable runtime location or runtime table keyed by:
  - `symbol`
  - `as_of_date`
  - `instrument_subscope`
  - `proxy_field_name`
- Reuse existing proxy field contracts so live manual input and historical backfill do not diverge.
- Make the new tool visible to CLI and catalog.

**Step 2: Run focused tests**

Run:

```powershell
cargo test --test security_external_proxy_backfill_cli -- --nocapture
```

Expected: PASS with persisted dated proxy records and deterministic result refs.

### Task 3: Join historical proxy records into feature snapshot and training sample collection

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_evidence_bundle.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_feature_snapshot.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_scorecard_training.rs`
- Modify: `D:\Rust\Excel_Skill\tests\security_feature_snapshot_cli.rs`
- Modify: `D:\Rust\Excel_Skill\tests\security_scorecard_training_cli.rs`

**Step 1: Write the minimal implementation**

- Load historical proxy records by `symbol + as_of_date`.
- When historical data exists, prefer dated backfill records for training snapshots.
- Keep current live/manual proxy input behavior for current-date decision flows.
- Preserve `placeholder_unbound` and `manual_bound` semantics when no historical proxy exists.

**Step 2: Run focused tests**

Run:

```powershell
cargo test --test security_feature_snapshot_cli -- --nocapture
cargo test --test security_scorecard_training_cli -- --nocapture
```

Expected: PASS with training samples now able to consume dated proxy history instead of only current manual values.

### Task 4: Implement and lock path-event heads in the governed training chain

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_forward_outcome.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_scorecard_training.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_scorecard.rs`
- Modify: `D:\Rust\Excel_Skill\tests\security_scorecard_training_cli.rs`

**Step 1: Write the minimal implementation**

- Promote `hit_upside_first` and `hit_stop_first` from background labels into first-class governed training heads.
- Add explicit fit reporting for these heads:
  - `accuracy`
  - `auc`
  - `ks`
  - event positive-rate
- Extend readiness assessment so path-event coverage is judged per head instead of only by aggregate train split.
- Keep direction and regression heads backward-compatible.

**Step 2: Run focused tests**

Run:

```powershell
cargo test --test security_scorecard_training_cli security_scorecard_training_supports_upside_first_head_with_classification_metrics -- --nocapture
cargo test --test security_scorecard_training_cli security_scorecard_training_supports_stop_first_head_with_classification_metrics -- --nocapture
```

Expected: PASS with governed classification metrics for both path-event heads.

### Task 5: Extend master scorecard and chair reasoning with path-event summaries

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_master_scorecard.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_chair_resolution.rs`
- Modify: `D:\Rust\Excel_Skill\tests\security_master_scorecard_cli.rs`
- Modify: `D:\Rust\Excel_Skill\tests\security_chair_resolution_cli.rs`

**Step 1: Write the minimal implementation**

- Extend trained-head summary to optionally include:
  - `expected_upside_first_probability`
  - `expected_stop_first_probability`
- Upgrade `master_scorecard` aggregation to expose path-event availability without overstating confidence.
- Add chair reasoning and execution-constraint language that references path-event asymmetry when present.
- Keep `replay_unavailable` behavior compatible with the current degraded live flow.

**Step 2: Run focused tests**

Run:

```powershell
cargo test --test security_master_scorecard_cli security_master_scorecard_attaches_path_event_context_when_available -- --nocapture
cargo test --test security_chair_resolution_cli -- --nocapture
```

Expected: PASS with formal master-scorecard and chair objects citing path-event context.

### Task 6: Run adjacent regressions, document gaps, and close the phase

**Files:**
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`
- Modify if needed: `D:\Rust\Excel_Skill\docs\plans\2026-04-11-p4-path-events-and-history-backfill.md`

**Step 1: Run adjacent regressions**

Run:

```powershell
cargo test --test security_external_proxy_backfill_cli -- --nocapture
cargo test --test security_feature_snapshot_cli -- --nocapture
cargo test --test security_scorecard_training_cli -- --nocapture
cargo test --test security_master_scorecard_cli -- --nocapture
cargo test --test security_chair_resolution_cli -- --nocapture
cargo test --test security_decision_submit_approval_cli -- --nocapture
```

Expected: PASS for the relevant governed chain.

**Step 2: Append task journal entry**

- Add a new dated entry to `.trae/CHANGELOG_TASK.md`.
- Record:
  - what changed
  - why it changed
  - what remains before shadow/champion governance
  - suggested follow-up tests

---

### Done-State Checklist

- Historical external-proxy records can be backfilled through a governed CLI tool.
- Training samples can join dated proxy records instead of relying only on live manual inputs.
- `upside_first_head` and `stop_first_head` are trainable, testable, and reported with explicit fit metrics.
- Master scorecard and chair resolution can cite path-event context without breaking degraded live flows.
- The phase is documented in `.trae/CHANGELOG_TASK.md` and ready for execution without rediscovery.
