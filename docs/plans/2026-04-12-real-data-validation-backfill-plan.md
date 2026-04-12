# Real Data Validation Backfill Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** add a governed tool that refreshes one validation slice with real-compatible price history and public disclosure context.

**Architecture:** reuse the existing stock-history sync providers and the fullstack public-disclosure analysis flow, then persist both outputs into a dedicated validation-slice runtime root plus a stable manifest. Keep the feature narrow to validation usage so it does not become a second production ingestion path.

**Tech Stack:** Rust, serde/serde_json, stock tool catalog/dispatcher, SQLite runtime storage, mocked HTTP integration tests, markdown docs.

---

### Task 1: Add failing CLI contract for real-data validation backfill

**Files:**
- Create: `tests/security_real_data_validation_backfill_cli.rs`

**Steps:**
1. Write the failing test for:
   - tool discovery
   - dedicated validation runtime root output
   - manifest and fullstack-context persistence
2. Run the focused test and verify it fails because the tool does not exist yet.

### Task 2: Reuse price-sync provider fetch without workspace-only persistence

**Files:**
- Modify: `src/ops/sync_stock_price_history.rs`

**Steps:**
1. Refactor the provider-fetch logic into a reusable helper that returns rows and provider metadata.
2. Keep `sync_stock_price_history` behavior unchanged for existing callers.
3. Run the focused adjacent price-sync tests if needed.

### Task 3: Implement `security_real_data_validation_backfill`

**Files:**
- Create: `src/ops/security_real_data_validation_backfill.rs`
- Modify: `src/ops/mod.rs`
- Modify: `src/ops/stock.rs`

**Steps:**
1. Implement the request/result contracts.
2. Persist a dedicated validation-slice runtime DB under the requested root.
3. Sync price history for the primary, market, and sector symbols.
4. Fetch and persist one `security_analysis_fullstack` result for the primary symbol.
5. Persist a stable manifest.

### Task 4: Wire tool catalog and dispatcher

**Files:**
- Modify: `src/tools/catalog.rs`
- Modify: `src/tools/dispatcher/stock_ops.rs`
- Modify: `src/tools/dispatcher.rs`

**Steps:**
1. Register the tool in the stock catalog.
2. Add dispatcher parsing and response wiring.
3. Re-run the focused CLI test until green.

### Task 5: Add operator-facing docs for the real-data slice

**Files:**
- Modify: `docs/security-holding-ledger.md`
- Modify: `docs/execution-notes-2026-04-12-p9-p10-validation-closeout.md`

**Steps:**
1. Record where the real-data manifest lives.
2. Explain how this slice differs from the deterministic replay slice.

### Task 6: Verify and journal

**Files:**
- Modify: `.trae/CHANGELOG_TASK.md`

**Steps:**
1. Run focused tests:
   - `security_real_data_validation_backfill_cli`
   - `security_lifecycle_validation_cli`
   - adjacent lifecycle suites if touched
2. Append the task-journal entry.
