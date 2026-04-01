# Technical Consultation ADX Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在现有 `technical_consultation_basic` Rust Tool 内补上 `ADX + DI`，让技术面基础咨询从“只有方向”提升到“方向 + 强度”。

**Architecture:** 继续复用当前 `CSV -> SQLite -> technical_consultation_basic -> JSON` 主链，不新增 Tool、不重开运行时。指标计算仍放在 `src/ops/technical_consultation_basic.rs`，先扩展 `indicator_snapshot`，再用轻规则生成 `trend_strength` 和更具体的摘要/建议。

**Tech Stack:** Rust、serde、现有 CLI 集成测试、SQLite 运行时历史行情。

---

### Task 1: 锁定外部合同

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\technical_consultation_basic_cli.rs`
- Test: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\technical_consultation_basic_cli.rs`

**Step 1: Write the failing test**

- 为上涨样本补断言，要求结果里新增：
  - `trend_strength`
  - `indicator_snapshot.adx_14`
  - `indicator_snapshot.plus_di_14`
  - `indicator_snapshot.minus_di_14`
- 断言上涨样本满足：
  - `trend_strength == "strong"`
  - `plus_di_14 > minus_di_14`

**Step 2: Run test to verify it fails**

Run: `cargo test --test technical_consultation_basic_cli technical_consultation_basic_returns_snapshot_and_guidance_from_sqlite_history -- --nocapture`

Expected: FAIL，因为当前合同里还没有 `trend_strength / adx_14 / plus_di_14 / minus_di_14`。

### Task 2: 锁定弱趋势场景

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\technical_consultation_basic_cli.rs`
- Test: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\technical_consultation_basic_cli.rs`

**Step 1: Write the failing test**

- 增加横盘震荡样本。
- 断言结果满足：
  - `trend_bias == "sideways"`
  - `trend_strength == "weak"`

**Step 2: Run test to verify it fails**

Run: `cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_choppy_history_as_weak_trend -- --nocapture`

Expected: FAIL，因为当前实现还没有 ADX 趋势强度判断。

### Task 3: 最小实现 ADX + DI

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\technical_consultation_basic.rs`

**Step 1: Write minimal implementation**

- 在 `TechnicalIndicatorSnapshot` 中新增：
  - `adx_14`
  - `plus_di_14`
  - `minus_di_14`
- 在 `TechnicalConsultationBasicResult` 中新增：
  - `trend_strength`
- 新增 `adx_snapshot()` 计算 `ADX / +DI / -DI`。
- 新增轻规则：
  - `ADX >= 25` -> `strong`
  - `ADX < 20` -> `weak`
  - 否则 -> `moderate`
- 在摘要、建议、观察点里接入趋势强度语义，但不改整体架构。

**Step 2: Run targeted tests to verify they pass**

Run: `cargo test --test technical_consultation_basic_cli -- --nocapture`

Expected: PASS

### Task 4: 回归验证

**Files:**
- No code change expected

**Step 1: Run related regressions**

Run:
- `cargo test --test technical_consultation_basic_cli -- --nocapture`
- `cargo test --test stock_price_history_import_cli -- --nocapture`
- `cargo test -- --nocapture`

Expected: PASS，允许保留既有非本轮 `dead_code` warning。

### Task 5: 交接收尾

**Files:**
- Modify: `D:\Rust\Excel_Skill\progress.md`
- Modify: `D:\Rust\Excel_Skill\findings.md`
- Modify: `D:\Rust\Excel_Skill\task_plan.md`
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Append handoff notes**

- 记录本轮 `ADX + DI` 范围、验证命令、后续建议。

**Step 2: Finish with task journal discipline**

- 确认 `.trae/CHANGELOG_TASK.md` 已追加本轮条目，便于下一位 AI 直接承接。
