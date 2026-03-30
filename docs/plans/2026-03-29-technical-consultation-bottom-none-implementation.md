# Technical Consultation Bottom None Boundary Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在 `technical_consultation_basic` 中补“价格与 OBV 同步创新低时，`divergence_signal` 必须保持 `none`”的专项回归。  
**Architecture:** 继续沿当前 `CSV -> SQLite -> technical_consultation_basic -> JSON` 主线推进，不新增 Tool，不改 dispatcher / catalog / runtime。  
**Tech Stack:** Rust、serde_json、现有 CLI 集成测试、SQLite 历史行情存储。

---

### Task 1: 先锁底部 `none` 红测合同

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\technical_consultation_basic_cli.rs`

**Step 1: 新增夹具与测试**

- 新增 `build_confirmed_breakdown_rows(day_count: usize) -> Vec<String>`
- 新增 `technical_consultation_basic_keeps_none_when_price_and_obv_confirm_breakdown()`
- 断言至少包含：
  - `output["status"] == "ok"`
  - `output["data"]["divergence_signal"] == "none"`

**Step 2: 跑专项测试确认当前状态**

Run:
- `cargo test --test technical_consultation_basic_cli technical_consultation_basic_keeps_none_when_price_and_obv_confirm_breakdown -- --nocapture`

Expected:
- 先确认是红还是已经绿
- 若失败，失败原因应与 `divergence_signal` 误判直接相关

### Task 2: 若红，则最小修正 divergence 规则

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\technical_consultation_basic.rs`

**Step 1: 仅在现有逻辑内最小修复**

- 只动 `classify_divergence_signal()`
- 保持现有对外合同不变
- 不新增额外 snapshot 字段
- 不改 summary / actions / watch_points 的整体结构

**Step 2: 重新跑专项测试**

Run:
- `cargo test --test technical_consultation_basic_cli technical_consultation_basic_keeps_none_when_price_and_obv_confirm_breakdown -- --nocapture`

Expected:
- PASS

### Task 3: 跑回归

Run:
- `cargo test --test technical_consultation_basic_cli -- --nocapture`
- `cargo test -- --nocapture`

Expected:
- PASS
- 允许保留已有 `dead_code` warnings

### Task 4: 记录交接与任务日志

**Files:**
- Modify: `D:\Rust\Excel_Skill\task_plan.md`
- Modify: `D:\Rust\Excel_Skill\progress.md`
- Modify: `D:\Rust\Excel_Skill\findings.md`
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: 追加本轮 handoff**

- 记录这轮是“底部确认型 none”边界加固
- 记录专项测试与全量验证命令
- 记录本轮是否只改测试，还是测试加最小生产修复

**Step 2: 追加任务日志**

- 按固定模板只追加，不重写历史内容
- 明确下一步仍继续沿 `technical_consultation_basic` 渐进推进，不重开架构讨论
