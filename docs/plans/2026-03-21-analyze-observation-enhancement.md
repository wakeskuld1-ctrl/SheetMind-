# Analyze Observation Enhancement Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 `analyze_table` 新增日期列、时间列、金额列的保守型业务观察，提升表处理层到分析建模层的桥接解释力。

**Architecture:** 在 `D:/Rust/Excel_Skill/src/ops` 下增加轻量语义识别层，用列名与样本值联合判断列语义；在 `D:/Rust/Excel_Skill/src/ops/analyze.rs` 中继续由 `build_business_observations(...)` 统一收敛日期、时间、金额三类观察结果，并复用现有去重与 `top_k` 限流逻辑。

**Tech Stack:** Rust、Polars、Serde、Serde JSON、Thiserror、assert_cmd、calamine。

---

### Task 1: 先写日期/时间/金额观察的内存层红灯测试

**Files:**
- Modify: `D:/Rust/Excel_Skill/tests/integration_frame.rs`

**Step 1: Write the failing test**

- 新增内存层测试，构造：
  - 日期列
  - 时间列
  - 金额列
- 断言会输出：
  - `date_range`
  - `date_concentration`
  - `time_peak_period`
  - `amount_typical_band`
  - `amount_negative_presence`
  - `amount_skew_hint`

**Step 2: Run test to verify it fails**

Run: `cargo test --test integration_frame analyze_table_builds_date_time_and_amount_observations -v`

Expected: FAIL，因为这些观察类型当前尚不存在。

### Task 2: 写 CLI 层红灯测试，锁定 JSON 契约

**Files:**
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`
- Create or Reuse: `D:/Rust/Excel_Skill/tests/fixtures/analyze-observation-enhancement.xlsx`

**Step 1: Write the failing test**

- 新增 CLI 测试，请求 `analyze_table`
- 断言 `business_observations` 与 `human_summary.quick_insights` 中能看到新增观察

**Step 2: Run test to verify it fails**

Run: `cargo test --test integration_cli_json analyze_table_returns_date_time_and_amount_observations_in_cli -v`

Expected: FAIL，因为 CLI 还不会返回这些观察。

### Task 3: 实现轻量语义识别层

**Files:**
- Create: `D:/Rust/Excel_Skill/src/ops/semantic.rs`
- Modify: `D:/Rust/Excel_Skill/src/ops/mod.rs`
- Modify: `D:/Rust/Excel_Skill/src/ops/analyze.rs`

**Step 1: Write minimal implementation**

- 定义列语义枚举与基础识别函数：
  - `looks_like_date_column`
  - `looks_like_time_column`
  - `looks_like_amount_column`
- 支持最小日期/时间解析

**Step 2: Run targeted test**

Run: `cargo test --test integration_frame analyze_table_builds_date_time_and_amount_observations -v`

Expected: 可能仍 FAIL，但失败点应收敛到观察生成而不是“识别层不存在”。

### Task 4: 实现日期与时间观察

**Files:**
- Modify: `D:/Rust/Excel_Skill/src/ops/analyze.rs`

**Step 1: Write minimal implementation**

- 新增：
  - `build_date_observations(...)`
  - `build_time_observations(...)`
- 接入 `build_business_observations(...)`

**Step 2: Run test**

Run: `cargo test --test integration_frame analyze_table_builds_date_time_and_amount_observations -v`

Expected: 部分观察转绿，金额相关仍可能失败。

### Task 5: 实现金额观察

**Files:**
- Modify: `D:/Rust/Excel_Skill/src/ops/analyze.rs`

**Step 1: Write minimal implementation**

- 新增：
  - `build_amount_observations(...)`
- 补：
  - 典型区间
  - 负数存在
  - 长尾提示

**Step 2: Run tests to verify they pass**

Run: `cargo test --test integration_frame analyze_table_builds_date_time_and_amount_observations -v`

Run: `cargo test --test integration_cli_json analyze_table_returns_date_time_and_amount_observations_in_cli -v`

Expected: PASS

### Task 6: 整理 quick_insights 与优先级

**Files:**
- Modify: `D:/Rust/Excel_Skill/src/ops/analyze.rs`

**Step 1: Write minimal implementation**

- 确保 `quick_insights` 优先展示更有业务解释力的新增观察
- 避免与已有数值观察重复轰炸

**Step 2: Run targeted tests**

Run: `cargo test --test integration_cli_json analyze_table_returns_date_time_and_amount_observations_in_cli -v`

Expected: PASS

### Task 7: 完整验证

**Files:**
- Verify only

**Step 1: Run targeted integration tests**

Run: `cargo test --test integration_frame --test integration_cli_json -v`

Expected: PASS

**Step 2: Run full suite**

Run: `cargo test -v`

Expected: PASS

**Step 3: Build release**

Run: `cargo build --release -v`

Expected: PASS

**Step 4: Smoke test**

Run: `D:/Rust/Excel_Skill/target/release/excel_skill.exe`

Expected: 返回稳定 `tool_catalog`

### Task 8: 任务日志

**Files:**
- Modify: `D:/Rust/Excel_Skill/.trae/CHANGELOG_TASK.md`

**Step 1: Append task journal**

- 只追加，不改历史
- 记录新增观察类型、测试、验证结果和遗留问题
