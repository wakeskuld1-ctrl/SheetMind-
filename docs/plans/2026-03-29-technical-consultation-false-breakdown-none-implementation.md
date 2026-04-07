# Technical Consultation False Breakdown None Boundary Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 `technical_consultation_basic` 增加“低位假跌破 / 低位震荡但不构成底背离时，`divergence_signal` 必须保持 `none`”的专项回归。

**Architecture:** 继续沿 `CSV -> SQLite -> technical_consultation_basic -> JSON` 主线推进，不新增 Tool，不改 `dispatcher / catalog / runtime`。优先通过测试夹具锁住边界，只有在红测证明误报存在时，才最小修改 `classify_divergence_signal()`。

**Tech Stack:** Rust、serde_json、现有 CLI 集成测试、SQLite 历史行情存储。

---

### Task 1: 锁定 A1 红测合同

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\technical_consultation_basic_cli.rs`

**Step 1: 写失败测试夹具**

- 新增 `build_false_breakdown_rows(day_count: usize) -> Vec<String>`
- 样本目标：价格在低位试探新低或轻微跌破，但 OBV 不形成足以支撑 `bullish_divergence` 的明确改善

**Step 2: 写失败测试**

- 新增 `technical_consultation_basic_keeps_none_when_false_breakdown_lacks_obv_divergence()`
- 断言至少包含：
  - `output["status"] == "ok"`
  - `output["data"]["divergence_signal"] == "none"`

**Step 3: 跑专项测试验证 RED**

Run:
- `cargo test --test technical_consultation_basic_cli technical_consultation_basic_keeps_none_when_false_breakdown_lacks_obv_divergence -- --nocapture`

Expected:
- 先确认测试是红还是已经绿
- 如果失败，失败原因必须直接指向 `divergence_signal` 误报

### Task 2: 仅在必要时最小修正 divergence 规则

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\technical_consultation_basic.rs`

**Step 1: 只改 `classify_divergence_signal()`**

- 不修改对外 JSON 字段
- 不引入新的快照字段
- 不顺手改 `timing_signal / summary / actions / watch_points`

**Step 2: 跑专项测试验证 GREEN**

Run:
- `cargo test --test technical_consultation_basic_cli technical_consultation_basic_keeps_none_when_false_breakdown_lacks_obv_divergence -- --nocapture`

Expected:
- PASS

### Task 3: 跑回归

Run:
- `cargo test --test technical_consultation_basic_cli -- --nocapture`
- `cargo test -- --nocapture`

Expected:
- PASS
- 允许保留与本轮无关的既有 warning

### Task 4: 追加交接记录

**Files:**
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: 记录本轮内容**

- 记录本轮是 A1：“低位假跌破 / 低位震荡保持 none”的边界加固
- 记录专项测试与回归命令
- 记录本轮是否只加测试，还是包含最小生产修正

**Step 2: 明确下一步**

- 继续沿 `technical_consultation_basic` 渐进推进
- 不重开架构重构话题
