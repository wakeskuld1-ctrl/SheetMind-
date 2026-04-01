# Stock History HTTP Sync Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a new Rust Tool that syncs A-share daily OHLCV history from Tencent first and Sina second into the existing SQLite `stock_price_history` table, while keeping the current CSV import path unchanged.

**Architecture:** Keep `import_stock_price_history` as the file-ingestion seam and add a new sibling Tool `sync_stock_price_history`. The new Tool owns provider routing and HTTP parsing, then reuses `StockHistoryStore` for SQLite persistence so `technical_consultation_basic` stays on the same `SQLite -> indicators -> consultation` mainline.

**Tech Stack:** Rust, `ureq`, `serde`, `serde_json`, `rusqlite`, existing CLI JSON dispatcher tests.

---

### Task 1: Lock external contract with failing CLI tests

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\stock_price_history_import_cli.rs`

**Step 1: Write the failing test**

- Add catalog discovery test for `sync_stock_price_history`.
- Add success test for Tencent daily sync into SQLite.
- Add fallback test where Tencent fails and Sina succeeds.
- Add failure test where all providers fail and Chinese error is returned.

**Step 2: Run test to verify it fails**

Run: `cargo test --test stock_price_history_import_cli sync_stock_price_history -- --nocapture --test-threads=1`

Expected: FAIL because tool and dispatcher path do not exist yet.

**Step 3: Commit**

Skip commit in this session; continue directly to minimal implementation after red is confirmed.

### Task 2: Add provider-facing HTTP sync operation

**Files:**
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\sync_stock_price_history.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\mod.rs`

**Step 1: Write minimal implementation**

- Define request / result / error types.
- Implement provider order loop.
- Implement Tencent JSON parsing for `fqkline`.
- Implement Sina KLine JSON parsing.
- Normalize provider output to `Vec<StockHistoryRow>`.
- Reuse `StockHistoryStore::import_rows()`.

**Step 2: Run targeted tests**

Run: `cargo test --test stock_price_history_import_cli sync_stock_price_history -- --nocapture --test-threads=1`

Expected: parser/dispatcher-related failures may remain, but core operation should compile.

### Task 3: Expose the Tool through catalog and dispatcher

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\catalog.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\dispatcher.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\dispatcher\analysis_ops.rs`

**Step 1: Add strong-typed dispatcher entry**

- Register `sync_stock_price_history` in catalog.
- Route it in dispatcher.
- Parse request and return JSON result in `analysis_ops`.

**Step 2: Run targeted tests**

Run: `cargo test --test stock_price_history_import_cli sync_stock_price_history -- --nocapture --test-threads=1`

Expected: tests should move from “tool missing” to provider behavior / parsing results.

### Task 4: Finish fallback and edge handling

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\sync_stock_price_history.rs`

**Step 1: Make tests pass**

- Aggregate provider errors.
- Validate symbol/date/provider inputs.
- Ensure empty result sets do not silently persist.

**Step 2: Run targeted test to green**

Run: `cargo test --test stock_price_history_import_cli sync_stock_price_history -- --nocapture --test-threads=1`

Expected: PASS

### Task 5: Run regression suite

**Files:**
- No code change expected

**Step 1: Run focused regressions**

Run: `cargo test --test stock_price_history_import_cli -- --nocapture --test-threads=1`

Expected: PASS

**Step 2: Run dependent stock path regression**

Run: `cargo test --test technical_consultation_basic_cli -- --nocapture --test-threads=1`

Expected: PASS

**Step 3: Run full regression**

Run: `cargo test -- --nocapture --test-threads=1`

Expected: PASS, with only pre-existing warnings allowed.

### Task 6: Update handoff records

**Files:**
- Modify: `D:\Rust\Excel_Skill\task_plan.md`
- Modify: `D:\Rust\Excel_Skill\findings.md`
- Modify: `D:\Rust\Excel_Skill\progress.md`
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Record what changed**

- Note that CSV remains the stable manual path.
- Note that Tencent is first provider and Sina is fallback.
- Note any residual risk about unofficial HTTP endpoints.

**Step 2: Final verification note**

- Record the exact commands and pass counts.
