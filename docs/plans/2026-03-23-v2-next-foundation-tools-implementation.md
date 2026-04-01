# V2 Next Foundation Tools Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 新增 `parse_datetime_columns`、`lookup_values`、`window_calculation` 三个基础 Tool，继续补齐表处理层到分析建模层的公共能力。

**Architecture:** 复用现有 `LoadedTable -> ops -> dispatcher -> result_ref` 范式，先实现最小可用的日期时间标准化，再实现轻量查值与窗口计算；每个 Tool 都先在 `integration_frame` / `integration_cli_json` 写失败测试，再做最小实现。

**Tech Stack:** Rust 2024、Polars 0.51、Serde、Serde JSON、现有 CLI JSON Tool 调度架构

---

### Task 1: `parse_datetime_columns`

**Files:**
- Create: `D:/Rust/Excel_Skill/src/ops/parse_datetime.rs`
- Modify: `D:/Rust/Excel_Skill/src/ops/mod.rs`
- Modify: `D:/Rust/Excel_Skill/src/tools/contracts.rs`
- Modify: `D:/Rust/Excel_Skill/src/tools/dispatcher.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_frame.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`

**Step 1: Write the failing test**
- 增加 frame 测试：`date` 规范化、`datetime` 规范化、非法值报错。
- 增加 CLI 测试：Tool catalog 暴露、CLI 调用返回 `result_ref`。

**Step 2: Run test to verify it fails**
Run: `cargo test parse_datetime_columns --test integration_frame --test integration_cli_json -v`
Expected: FAIL

**Step 3: Write minimal implementation**
- 复用 `semantic` 里的日期/时间解析能力。
- 第一版仅支持原列原位标准化。
- 输出 preview + `result_ref`。

**Step 4: Run test to verify it passes**
Run: `cargo test parse_datetime_columns --test integration_frame --test integration_cli_json -v`
Expected: PASS

### Task 2: `lookup_values`

**Files:**
- Create: `D:/Rust/Excel_Skill/src/ops/lookup_values.rs`
- Modify: `D:/Rust/Excel_Skill/src/ops/mod.rs`
- Modify: `D:/Rust/Excel_Skill/src/tools/contracts.rs`
- Modify: `D:/Rust/Excel_Skill/src/tools/dispatcher.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_frame.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`

**Step 1: Write the failing test**
- 增加 frame 测试：带回列、未命中为空、重复 key 报错、输出列冲突报错。
- 增加 CLI mixed source 测试。

**Step 2: Run test to verify it fails**
Run: `cargo test lookup_values --test integration_frame --test integration_cli_json -v`
Expected: FAIL

**Step 3: Write minimal implementation**
- 复用 `fill_missing_from_lookup` 的双来源装载与唯一键索引思路。
- 明确 output column 冲突策略。

**Step 4: Run test to verify it passes**
Run: `cargo test lookup_values --test integration_frame --test integration_cli_json -v`
Expected: PASS

### Task 3: `window_calculation`

**Files:**
- Create: `D:/Rust/Excel_Skill/src/ops/window.rs`
- Modify: `D:/Rust/Excel_Skill/src/ops/mod.rs`
- Modify: `D:/Rust/Excel_Skill/src/tools/contracts.rs`
- Modify: `D:/Rust/Excel_Skill/src/tools/dispatcher.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_frame.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`

**Step 1: Write the failing test**
- 增加 frame 测试：`row_number`、`rank`、`cumulative_sum`、分组窗口、非数值累计报错。
- 增加 CLI 测试。

**Step 2: Run test to verify it fails**
Run: `cargo test window_calculation --test integration_frame --test integration_cli_json -v`
Expected: FAIL

**Step 3: Write minimal implementation**
- 第一版只支持最小窗口函数集合。
- 复用现有排序逻辑与显式参数解析。

**Step 4: Run test to verify it passes**
Run: `cargo test window_calculation --test integration_frame --test integration_cli_json -v`
Expected: PASS

### Task 4: 全量验证与任务日志

**Files:**
- Modify: `D:/Rust/Excel_Skill/.trae/CHANGELOG_TASK.md`

**Step 1: Run full verification**
Run: `cargo test -v`
Run: `cargo build --release -v`
Expected: PASS

**Step 2: Append task journal**
- 只追加本轮任务日志，不改写历史条目。
