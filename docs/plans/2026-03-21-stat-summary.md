# Statistical Summary Tool Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 新增独立 `stat_summary` Tool，输出建模前可直接消费的结构化统计摘要与中文摘要。

**Architecture:** 在 `D:/Rust/Excel_Skill/src/ops/stat_summary.rs` 中实现独立统计桥接逻辑，复用现有 `LoadedTable` 和 `summarize_table`，按列类型拆分数值/类别/布尔摘要；在 `D:/Rust/Excel_Skill/src/tools/dispatcher.rs` 中挂接新 Tool，并保持与现有 CLI JSON 协议一致。

**Tech Stack:** Rust、Polars、Serde、Serde JSON、Thiserror、assert_cmd、calamine。

---

### Task 1: 先写 `stat_summary` 的内存层红灯测试

**Files:**
- Modify: `D:/Rust/Excel_Skill/tests/integration_frame.rs`

**Step 1: Write the failing test**

- 新增一个最小内存层测试，断言：
  - 数值列能输出 `median`、`q1`、`q3`、`zero_ratio`
  - 类别列能输出 `distinct_count`、`top_share`
  - 布尔列能输出 `true_ratio`

**Step 2: Run test to verify it fails**

Run: `cargo test --test integration_frame stat_summary_builds_typed_statistical_profiles -v`

Expected: FAIL，因为 `stat_summary` 模块和结果结构尚不存在。

**Step 3: Write minimal implementation**

- 创建 `D:/Rust/Excel_Skill/src/ops/stat_summary.rs`
- 先定义最小结构体与空实现接口，使测试从“编译失败”推进到“行为失败”

**Step 4: Run test to verify it still fails for the right reason**

Run: `cargo test --test integration_frame stat_summary_builds_typed_statistical_profiles -v`

Expected: FAIL，因统计值尚未正确输出。

### Task 2: 实现最小统计核心使内存层测试转绿

**Files:**
- Create: `D:/Rust/Excel_Skill/src/ops/stat_summary.rs`
- Modify: `D:/Rust/Excel_Skill/src/ops/mod.rs`
- Modify: `D:/Rust/Excel_Skill/src/lib.rs`

**Step 1: Write minimal implementation**

- 在 `stat_summary.rs` 中定义：
  - `StatSummaryResult`
  - `TableOverview`
  - `NumericStatSummary`
  - `CategoricalStatSummary`
  - `BooleanStatSummary`
  - `StatHumanSummary`
- 实现：
  - 数值列分位数、中位数、零值占比
  - 类别列 top share
  - 布尔列 true ratio

**Step 2: Run test to verify it passes**

Run: `cargo test --test integration_frame stat_summary_builds_typed_statistical_profiles -v`

Expected: PASS

### Task 3: 写 CLI 层红灯测试，锁定 Tool JSON 契约

**Files:**
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`

**Step 1: Write the failing test**

- 新增 CLI 测试，断言：
  - `tool_catalog` 包含 `stat_summary`
  - `stat_summary` 返回：
    - `row_count`
    - `column_count`
    - `table_overview`
    - `numeric_summaries`
    - `categorical_summaries`
    - `boolean_summaries`
    - `human_summary`

**Step 2: Run test to verify it fails**

Run: `cargo test --test integration_cli_json stat_summary_returns_typed_summary_payload_in_cli -v`

Expected: FAIL，因为 dispatcher 尚未暴露 `stat_summary`。

### Task 4: 接入 Tool catalog 与 dispatcher

**Files:**
- Modify: `D:/Rust/Excel_Skill/src/tools/contracts.rs`
- Modify: `D:/Rust/Excel_Skill/src/tools/dispatcher.rs`

**Step 1: Write minimal implementation**

- 将 `stat_summary` 加入 tool catalog
- 在 dispatcher 中新增 `dispatch_stat_summary`
- 支持复用 `casts`、`columns`、`top_k`

**Step 2: Run CLI test to verify it passes**

Run: `cargo test --test integration_cli_json stat_summary_returns_typed_summary_payload_in_cli -v`

Expected: PASS

### Task 5: 补真实 Excel 场景测试

**Files:**
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_frame.rs`
- Reuse or Create: `D:/Rust/Excel_Skill/tests/fixtures/analyze-distribution.xlsx`

**Step 1: Write the failing test**

- 断言偏态数值列会输出稳定 `median`
- 断言高零值列会输出正确 `zero_ratio`
- 断言类别列会输出稳定 `top_share`

**Step 2: Run tests to verify they fail**

Run: `cargo test --test integration_frame stat_summary_handles_skew_and_category_distribution -v`

Run: `cargo test --test integration_cli_json stat_summary_reports_skew_and_distribution_in_cli -v`

Expected: FAIL，因为当前实现还不完整或文案未接好。

**Step 3: Write minimal implementation**

- 补齐真实 Excel 场景下的数值/类别摘要细节
- 补齐 `human_summary` 的关键统计观察生成

**Step 4: Run tests to verify they pass**

Run: `cargo test --test integration_frame stat_summary_handles_skew_and_category_distribution -v`

Run: `cargo test --test integration_cli_json stat_summary_reports_skew_and_distribution_in_cli -v`

Expected: PASS

### Task 6: 完整验证与单二进制冒烟

**Files:**
- Verify only

**Step 1: Run targeted tests**

Run: `cargo test --test integration_frame --test integration_cli_json -v`

Expected: PASS

**Step 2: Run full test suite**

Run: `cargo test -v`

Expected: PASS

**Step 3: Build release**

Run: `cargo build --release -v`

Expected: PASS

**Step 4: Smoke test the binary**

Run: `D:/Rust/Excel_Skill/target/release/excel_skill.exe`

Expected: 空输入返回包含 `stat_summary` 的 `tool_catalog`

### Task 7: 记录风险与任务日志

**Files:**
- Modify: `D:/Rust/Excel_Skill/.trae/CHANGELOG_TASK.md`

**Step 1: Append task journal**

- 只追加，不改历史
- 记录新增 `stat_summary` Tool、测试覆盖、验证结果、未完成项和潜在问题

**Step 2: Prepare handoff summary**

- 汇总修改文件
- 汇总验证结果
- 汇总剩余风险与建议测试
