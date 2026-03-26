# Trend Analysis Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a minimal `trend_analysis` tool that reads a time column and value column, summarizes direction and change, and exposes it through the CLI.

**Architecture:** Keep the first version as a lightweight traditional Rust statistical tool. Reuse the existing analysis loading path so the tool accepts `path+sheet`, `table_ref`, and `result_ref`, and return a stable JSON payload without introducing a new persistence model.

**Tech Stack:** Rust, Polars, serde, existing dispatcher/contracts/integration CLI tests.

---

### Task 1: Lock the CLI contract with RED tests

**Files:**
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`

**Step 1: Write the failing test**
- Add a small seeded `result_ref` dataset for time-series values.
- Add one CLI test for `trend_analysis` returning direction and summary.
- Add one tool catalog test proving discoverability.

**Step 2: Run test to verify it fails**
- Run the two focused test commands and confirm they fail for missing tool wiring.

### Task 2: Implement the minimal trend analysis tool

**Files:**
- Create: `D:/Rust/Excel_Skill/src/ops/trend_analysis.rs`
- Modify: `D:/Rust/Excel_Skill/src/ops/mod.rs`
- Modify: `D:/Rust/Excel_Skill/src/tools/dispatcher.rs`
- Modify: `D:/Rust/Excel_Skill/src/tools/contracts.rs`

**Step 1: Write minimal implementation**
- Parse time/value columns from a loaded table.
- Sort points by time key.
- Compute start/end values, absolute change, percent change, and coarse direction.
- Return a human summary and ordered points.

**Step 2: Run tests to verify GREEN**
- Run the focused CLI tests and adjust only the minimum code needed.

### Task 3: Regression and closure

**Files:**
- Modify: `D:/Rust/Excel_Skill/.trae/CHANGELOG_TASK.md`

**Step 1: Run regression commands**
- Re-run focused trend tests plus existing statistical diagnostics tests.
- Run `cargo build -q`.

**Step 2: Journal and prepare push**
- Append the task log entry.
- Review `git status` so the next step can push safely.
