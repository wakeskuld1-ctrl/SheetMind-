# Technical Consultation OBV Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在 `technical_consultation_basic` 中补上 `OBV` 与第一层量价确认输出。

**Architecture:** 保持当前 `CSV -> SQLite -> technical_consultation_basic -> JSON` 主链不变，不新增 Tool，不改 dispatcher。实现只落在 `src/ops/technical_consultation_basic.rs`，并沿现有 CLI 红测转绿。

**Tech Stack:** Rust、serde、现有 CLI 集成测试、SQLite 历史行情。

---

### Task 1: 锁定量价确认合同

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\technical_consultation_basic_cli.rs`
- Test: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\technical_consultation_basic_cli.rs`

**Step 1: Reuse the failing tests**

- 复用已经写好的红测：
  - `technical_consultation_basic_returns_snapshot_and_guidance_from_sqlite_history`
  - `technical_consultation_basic_marks_fading_volume_as_weakening_confirmation`

**Step 2: Run test to verify it fails**

Run:
- `cargo test --test technical_consultation_basic_cli technical_consultation_basic_returns_snapshot_and_guidance_from_sqlite_history -- --nocapture`
- `cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_fading_volume_as_weakening_confirmation -- --nocapture`

Expected: FAIL，因为当前还没有 `volume_confirmation / obv / volume_sma_20 / volume_ratio_20`。

### Task 2: 最小实现 OBV 与量价确认

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\technical_consultation_basic.rs`

**Step 1: Write minimal implementation**

- 在 `TechnicalIndicatorSnapshot` 中新增：
  - `obv`
  - `volume_sma_20`
  - `volume_ratio_20`
- 在顶层结果中新增：
  - `volume_confirmation`
- 新增：
  - `obv_last()`
  - `volume_sma_last()`
  - `volume_ratio_last()`
  - `classify_volume_confirmation()`
- 在摘要、建议、观察点里补轻量量价提示。

**Step 2: Run targeted tests**

Run: `cargo test --test technical_consultation_basic_cli -- --nocapture`

Expected: PASS

### Task 3: 回归验证

**Files:**
- No code change expected

**Step 1: Run related regressions**

Run:
- `cargo test --test technical_consultation_basic_cli -- --nocapture`
- `cargo test --test stock_price_history_import_cli -- --nocapture`
- `cargo test -- --nocapture`

Expected: PASS，允许保留既有 `dead_code` warnings。

### Task 4: 交接收尾

**Files:**
- Modify: `D:\Rust\Excel_Skill\progress.md`
- Modify: `D:\Rust\Excel_Skill\findings.md`
- Modify: `D:\Rust\Excel_Skill\task_plan.md`
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Append handoff notes**

- 记录本轮 `OBV / volume_confirmation` 范围、验证命令与后续建议。

**Step 2: Finish with task journal discipline**

- 确认任务日志追加完成，方便下一位 AI 继续承接。
