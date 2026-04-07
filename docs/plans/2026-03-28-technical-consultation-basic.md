# Technical Consultation Basic Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build the first Rust `technical_consultation_basic` Tool that reads stock history from SQLite, computes base technical indicators, and returns a stable consultation JSON contract.

**Architecture:** Reuse the existing Rust binary-first Tool chain and the newly landed stock history SQLite seam. Keep indicator calculation on-demand inside Rust, avoid cache layers for now, and expose one stable business-level Tool contract before any Skill mounting.

**Tech Stack:** Rust, rusqlite, serde, chrono, cargo test, existing SheetMind dispatcher/catalog

---

### Task 1: Lock the external Tool contract with failing CLI tests

**Files:**
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\technical_consultation_basic_cli.rs`
- Reference: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\common\mod.rs`

**Step 1: Write the failing test**

- Add one catalog test that asserts `technical_consultation_basic` appears in the Rust tool catalog.
- Add one bullish scenario test that seeds history into SQLite and expects:
  - `status == "ok"`
  - non-empty `summary`
  - `trend_bias` indicates bullish state
  - `indicator_snapshot` contains all required V1 fields
- Add one bearish scenario test.
- Add one sideways high-volatility scenario test.
- Add one insufficient-history test that expects a clear error when the available history is not enough for the longest indicator window.

**Step 2: Run test to verify it fails**

Run: `cargo test --test technical_consultation_basic_cli -- --nocapture`

Expected: FAIL because the Tool does not exist yet.

### Task 2: Add stock-history read support for consultation

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\runtime\stock_history_store.rs`

**Step 1: Write the failing test**

- If needed, add a focused store-level test or drive the failure from the CLI test first.
- Lock one query path that loads one symbol up to one `as_of_date` with ascending trade dates.

**Step 2: Run the test to verify it fails**

Run: `cargo test --test technical_consultation_basic_cli -- --nocapture`

Expected: FAIL because the read/query path is missing.

**Step 3: Write minimal implementation**

- Add a read method on `StockHistoryStore` for:
  - `symbol`
  - `as_of_date`
  - `lookback_days`
- Return normalized history rows in chronological order.

**Step 4: Run the test to verify it passes**

Run: `cargo test --test technical_consultation_basic_cli -- --nocapture`

Expected: still failing later at indicator or Tool wiring stages, but the read path should now exist.

### Task 3: Implement base indicator calculations

**Files:**
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\technical_consultation_basic.rs`

**Step 1: Write the failing test**

- Lock deterministic expectations for:
  - `ema_10`
  - `sma_50`
  - `sma_200`
  - `macd_line`
  - `macd_signal`
  - `macd_histogram`
  - `rsi_14`
  - `boll_middle`
  - `boll_upper`
  - `boll_lower`
  - `atr_14`

**Step 2: Run the test to verify it fails**

Run: `cargo test --test technical_consultation_basic_cli -- --nocapture`

Expected: FAIL because indicator calculations do not exist yet.

**Step 3: Write minimal implementation**

- Implement only the indicator math needed for this slice.
- Keep helper functions local to the new module unless reuse is clearly necessary.

**Step 4: Run the test to verify it passes**

Run: `cargo test --test technical_consultation_basic_cli -- --nocapture`

Expected: consultation wording tests may still fail, but indicator snapshot values should now be available.

### Task 4: Implement consultation classification and output contract

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\technical_consultation_basic.rs`

**Step 1: Write the failing test**

- Lock:
  - bullish consultation wording and actions
  - bearish consultation wording and actions
  - sideways high-volatility consultation wording and watch points

**Step 2: Run the test to verify it fails**

Run: `cargo test --test technical_consultation_basic_cli -- --nocapture`

Expected: FAIL because the consultation decision logic is not complete yet.

**Step 3: Write minimal implementation**

- Build:
  - `trend_bias`
  - `momentum_signal`
  - `volatility_state`
  - `summary`
  - `recommended_actions`
  - `watch_points`
  - `indicator_snapshot`
  - `data_window_summary`

**Step 4: Run the test to verify it passes**

Run: `cargo test --test technical_consultation_basic_cli -- --nocapture`

Expected: PASS

### Task 5: Wire the Tool into the Rust catalog and dispatcher

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\mod.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\catalog.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\dispatcher.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\dispatcher\analysis_ops.rs`

**Step 1: Add catalog entry**

- Register `technical_consultation_basic`.

**Step 2: Add dispatcher handler**

- Parse the new request type.
- Call the new op.
- Return the stable JSON payload.

**Step 3: Run test to verify it passes**

Run: `cargo test --test technical_consultation_basic_cli -- --nocapture`

Expected: PASS

### Task 6: Run focused regression and full verification

**Files:**
- No new files required

**Step 1: Run related suites**

Run:

- `cargo test --test stock_price_history_import_cli -- --nocapture`
- `cargo test --test technical_consultation_basic_cli -- --nocapture`

Expected: PASS

**Step 2: Run full suite**

Run: `cargo test`

Expected: PASS with only unchanged pre-existing warnings if any.

### Task 7: Update records and handoff notes

**Files:**
- Modify: `D:\Rust\Excel_Skill\task_plan.md`
- Modify: `D:\Rust\Excel_Skill\findings.md`
- Modify: `D:\Rust\Excel_Skill\progress.md`
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Record what changed**

- Log the new `technical_consultation_basic` Tool.
- Record the stable indicator snapshot contract.
- Record the remaining out-of-scope items:
  - Skill mounting
  - indicator cache
  - RSRS / ADX / OBV

**Step 2: Invoke task journal workflow**

- Append the required task completion note after verification.
