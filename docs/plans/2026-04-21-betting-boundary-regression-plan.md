# Betting Boundary Regression Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add approved boundary regression coverage for the betting optimizer and workbook delivery path, then fix any defects that the new tests expose.

**Architecture:** Keep the work split by layer. First lock business-critical edge cases in `betting_optimizer_unit.rs`, then add a small number of CLI/workbook end-to-end regressions, and only after RED evidence appears touch production code. Prefer assertions on business invariants and optimality signals over brittle row-by-row snapshots unless a specific row value is the point of the test.

**Tech Stack:** Rust, Cargo tests, existing betting optimizer domain types, betting solver CLI, workbook bridge.

---

### Task 1: Lock solver-side boundary regression cases

**Files:**
- Modify: `tests/betting_optimizer_unit.rs`
- Test: `tests/betting_optimizer_unit.rs`

**Step 1: Write the failing boundary tests**

Add tests covering:
- one-number concentration
- already-safe zero-refund case
- sparse many-zero case
- loss-count-below-target case that should return best-gap result instead of false infeasible
- small brute-force optimality comparison case

**Step 2: Run focused tests to verify RED**

Run: `cargo test --test betting_optimizer_unit <new_test_name> -- --nocapture`

Expected: at least one new test fails for the intended reason if a defect exists, or all pass if behavior is already correct.

**Step 3: Write minimal implementation only if RED appears**

Touch:
- `src/ops/betting_optimizer.rs`

Keep changes minimal and local to the failing behavior.

**Step 4: Run focused tests to verify GREEN**

Run: `cargo test --test betting_optimizer_unit -- --nocapture`

Expected: all optimizer unit tests pass.

### Task 2: Lock CLI / workbook edge regressions

**Files:**
- Modify: `tests/betting_solver_cli.rs`
- Modify: `tests/betting_workbook_bridge_cli.rs`
- Test: `tests/betting_solver_cli.rs`
- Test: `tests/betting_workbook_bridge_cli.rs`

**Step 1: Write the failing end-to-end tests**

Add 2-3 targeted tests covering:
- concentrated stake workbook solve
- already-safe workbook solve with zero refund
- sparse/zero-heavy workbook solve and result-sheet writeback

**Step 2: Run the focused CLI / workbook tests to verify RED**

Run:
- `cargo test --test betting_solver_cli -- --nocapture`
- `cargo test --test betting_workbook_bridge_cli -- --nocapture`

Expected: any real defect appears as a failing test with reproducible output.

**Step 3: Write minimal bridge / CLI fix only if RED appears**

Touch only if needed:
- `src/bin/betting_solver.rs`
- `src/ops/betting_workbook_bridge.rs`
- `src/ops/betting_optimizer.rs`

**Step 4: Re-run CLI / workbook tests to verify GREEN**

Run:
- `cargo test --test betting_solver_cli -- --nocapture`
- `cargo test --test betting_workbook_bridge_cli -- --nocapture`

Expected: all non-ignored tests pass.

### Task 3: Delivery verification and artifact refresh

**Files:**
- Modify: `.trae/CHANGELOG_TASK.md`
- Output: `outputs/客户交付包_2026-04-21/betting_solver.exe`
- Output: `outputs/betting_optimizer_delivery/betting_solver.exe`

**Step 1: Run fresh verification commands**

Run:
- `cargo test --test betting_optimizer_unit -- --nocapture`
- `cargo test --test betting_solver_cli -- --nocapture`
- `cargo test --test betting_workbook_bridge_cli -- --nocapture`

Expected: pass with current known ignored tests unchanged.

**Step 2: Rebuild release delivery binary**

Run: `cargo build --release --bin betting_solver`

Expected: build succeeds.

**Step 3: Refresh delivery binaries**

Copy release output into:
- `outputs/客户交付包_2026-04-21/betting_solver.exe`
- `outputs/betting_optimizer_delivery/betting_solver.exe`

**Step 4: Append task journal**

Update:
- `.trae/CHANGELOG_TASK.md`

Record:
- what edge cases were added
- what failed
- what was fixed
- what remains risky
