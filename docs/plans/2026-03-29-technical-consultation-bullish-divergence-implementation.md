# Technical Consultation Bullish Divergence Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在 `technical_consultation_basic` 中补齐 `bullish_divergence` 的专项红测与最小实现。

**Architecture:** 继续沿当前 `CSV -> SQLite -> technical_consultation_basic -> JSON` 主链推进，不新增 Tool，不改 dispatcher。实现只落在 `tests/technical_consultation_basic_cli.rs` 与 `src/ops/technical_consultation_basic.rs`，并同步补手册记录。

**Tech Stack:** Rust、serde_json、现有 CLI 集成测试、SQLite 历史行情存储。

---

### Task 1: 锁定底背离红测合同

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\technical_consultation_basic_cli.rs`
- Test: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\technical_consultation_basic_cli.rs`

**Step 1: 先写失败测试**

- 新增 `build_bullish_divergence_rows(day_count: usize) -> Vec<String>`
- 新增 `technical_consultation_basic_marks_price_obv_bullish_divergence()`
- 断言至少包含：
  - `output["status"] == "ok"`
  - `output["data"]["divergence_signal"] == "bullish_divergence"`

**Step 2: 跑专项测试确认先失败**

Run:
- `cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_price_obv_bullish_divergence -- --nocapture`

Expected:
- FAIL，原因应是现有规则尚未覆盖该底背离样本

### Task 2: 最小修改底背离规则

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\technical_consultation_basic.rs`

**Step 1: 在现有规则上做最小修复**

- 保持现有 `divergence_signal` 顶层合同不变
- 仅在 `classify_divergence_signal()` 内调整底背离识别逻辑
- 不新增第二条分析链
- 不新增新字段

**Step 2: 跑专项测试确认转绿**

Run:
- `cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_price_obv_bullish_divergence -- --nocapture`

Expected:
- PASS

### Task 3: 跑相关回归

**Files:**
- No code change expected

**Step 1: 运行相关回归**

Run:
- `cargo test --test technical_consultation_basic_cli -- --nocapture`
- `cargo test --test stock_price_history_import_cli -- --nocapture`
- `cargo test -- --nocapture`

Expected:
- PASS，允许保留既有 `dead_code` warnings

### Task 4: 交接与记录

**Files:**
- Modify: `D:\Rust\Excel_Skill\task_plan.md`
- Modify: `D:\Rust\Excel_Skill\progress.md`
- Modify: `D:\Rust\Excel_Skill\findings.md`
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: 追加 handoff 记录**

- 记录本轮 `bullish_divergence` 的范围、验证命令、遗留边界

**Step 2: 完成任务日志追加**

- 追加 `.trae/CHANGELOG_TASK.md`
- 明确下一步仍在 `technical_consultation_basic` 内继续补边界测试，不重开架构
