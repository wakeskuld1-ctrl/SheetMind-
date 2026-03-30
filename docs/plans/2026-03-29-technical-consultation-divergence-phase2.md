# Technical Consultation Divergence Phase 2 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在 `technical_consultation_basic` 内补齐 `bullish_divergence` 和背离误判边界回归，不改变现有 Rust / exe / SQLite 架构。

**Architecture:** 继续沿现有 `technical_consultation_basic -> classify_divergence_signal()` 主线增量推进。所有新增行为先由 CLI 级红测锁定，再只做最小实现让测试转绿，不新增第二套技术分析链路。

**Tech Stack:** Rust、cargo test、SQLite、现有 CLI 测试夹具

---

### Task 1: Lock Bullish Divergence Contract

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\technical_consultation_basic_cli.rs`
- Test: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\technical_consultation_basic_cli.rs`

**Step 1: Write the failing test**

- 新增 `build_bullish_divergence_rows(day_count: usize)` 样本。
- 新增测试，断言 `divergence_signal == "bullish_divergence"`。

**Step 2: Run test to verify it fails**

Run: `cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_price_obv_bullish_divergence -- --nocapture`

Expected: FAIL，当前实现无法稳定识别该样本。

**Step 3: Write minimal implementation**

- 只修改 `classify_divergence_signal()` 所需最小逻辑。

**Step 4: Run test to verify it passes**

Run: `cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_price_obv_bullish_divergence -- --nocapture`

Expected: PASS

### Task 2: Lock None Boundaries

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\technical_consultation_basic_cli.rs`
- Test: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\technical_consultation_basic_cli.rs`

**Step 1: Write the failing tests**

- 新增“价格创新高且 OBV 同步创新高 -> `none`”样本。
- 新增“价格未创新高但 OBV 回落 -> `none`”样本。

**Step 2: Run tests to verify they fail**

Run: `cargo test --test technical_consultation_basic_cli none_divergence -- --nocapture`

Expected: FAIL，至少一条边界测试先失败。

**Step 3: Write minimal implementation**

- 继续只在 `classify_divergence_signal()` 做最小修正。

**Step 4: Run tests to verify they pass**

Run: `cargo test --test technical_consultation_basic_cli none_divergence -- --nocapture`

Expected: PASS

### Task 3: Full Verification And Records

**Files:**
- Modify: `D:\Rust\Excel_Skill\task_plan.md`
- Modify: `D:\Rust\Excel_Skill\findings.md`
- Modify: `D:\Rust\Excel_Skill\progress.md`
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Run focused regression**

Run: `cargo test --test technical_consultation_basic_cli -- --nocapture`

Expected: PASS

**Step 2: Run import regression**

Run: `cargo test --test stock_price_history_import_cli -- --nocapture`

Expected: PASS

**Step 3: Run full verification**

Run: `cargo test`

Expected: PASS

**Step 4: Update records**

- 追加记录本轮 `bullish_divergence` 与边界合同收口结果。

