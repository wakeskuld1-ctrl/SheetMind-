# V2 Foundation Tools Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 新增 `normalize_text_columns`、`rename_columns`、`fill_missing_from_lookup`、`pivot_table` 四个基础 Tool，补齐 V2 第一批通用表处理能力。

**Architecture:** 在 `ops` 层分别新增 4 个独立模块，统一复用 `LoadedTable` 作为输入输出载体；在 `dispatcher` 层接入单表与双表来源解析、参数校验与 `result_ref` 持久化；通过 `integration_frame` 与 `integration_cli_json` 先写失败测试，再逐步最小实现。

**Tech Stack:** Rust 2024、Polars 0.51、Serde、Serde JSON、现有 CLI JSON Tool 调度架构

---

### Task 1: `normalize_text_columns`

**Files:**
- Create: `D:/Rust/Excel_Skill/src/ops/normalize_text.rs`
- Modify: `D:/Rust/Excel_Skill/src/ops/mod.rs`
- Modify: `D:/Rust/Excel_Skill/src/tools/contracts.rs`
- Modify: `D:/Rust/Excel_Skill/src/tools/dispatcher.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_frame.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`

**Step 1: Write the failing test**
- 在 `integration_frame` 增加文本标准化测试：`trim`、`collapse_whitespace`、`lowercase`、`remove_chars`、`replace_pairs`。
- 在 `integration_cli_json` 增加 Tool catalog 与 CLI 调用测试，锁定 `result_ref` 返回。

**Step 2: Run test to verify it fails**
Run: `cargo test normalize_text_columns --test integration_frame --test integration_cli_json -v`
Expected: FAIL，提示 Tool 或函数不存在。

**Step 3: Write minimal implementation**
- 定义规则结构。
- 逐列按固定顺序执行字符串清洗。
- 复用统一 preview + result_ref 返回。

**Step 4: Run test to verify it passes**
Run: `cargo test normalize_text_columns --test integration_frame --test integration_cli_json -v`
Expected: PASS

### Task 2: `rename_columns`

**Files:**
- Create: `D:/Rust/Excel_Skill/src/ops/rename.rs`
- Modify: `D:/Rust/Excel_Skill/src/ops/mod.rs`
- Modify: `D:/Rust/Excel_Skill/src/tools/contracts.rs`
- Modify: `D:/Rust/Excel_Skill/src/tools/dispatcher.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_frame.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`

**Step 1: Write the failing test**
- 增加正常重命名测试。
- 增加冲突列名报错与缺失源列报错测试。
- 增加 CLI 调用测试。

**Step 2: Run test to verify it fails**
Run: `cargo test rename_columns --test integration_frame --test integration_cli_json -v`
Expected: FAIL

**Step 3: Write minimal implementation**
- 定义映射结构。
- 先校验源列存在与目标列不冲突，再批量改名。
- 保持数据与行序不变。

**Step 4: Run test to verify it passes**
Run: `cargo test rename_columns --test integration_frame --test integration_cli_json -v`
Expected: PASS

### Task 3: `fill_missing_from_lookup`

**Files:**
- Create: `D:/Rust/Excel_Skill/src/ops/fill_lookup.rs`
- Modify: `D:/Rust/Excel_Skill/src/ops/mod.rs`
- Modify: `D:/Rust/Excel_Skill/src/tools/contracts.rs`
- Modify: `D:/Rust/Excel_Skill/src/tools/dispatcher.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_frame.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`

**Step 1: Write the failing test**
- 增加空值补齐、不覆盖非空值、lookup 多命中报错测试。
- 增加 base / lookup 混合来源的 CLI 测试。

**Step 2: Run test to verify it fails**
Run: `cargo test fill_missing_from_lookup --test integration_frame --test integration_cli_json -v`
Expected: FAIL

**Step 3: Write minimal implementation**
- 新增 base / lookup 双来源解析。
- 构建 lookup 索引并校验唯一性。
- 仅在 base 列为空时按映射回填。

**Step 4: Run test to verify it passes**
Run: `cargo test fill_missing_from_lookup --test integration_frame --test integration_cli_json -v`
Expected: PASS

### Task 4: `pivot_table`

**Files:**
- Create: `D:/Rust/Excel_Skill/src/ops/pivot.rs`
- Modify: `D:/Rust/Excel_Skill/src/ops/mod.rs`
- Modify: `D:/Rust/Excel_Skill/src/tools/contracts.rs`
- Modify: `D:/Rust/Excel_Skill/src/tools/dispatcher.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_frame.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`

**Step 1: Write the failing test**
- 增加 `sum` / `count` / `mean` 的 frame 测试。
- 增加 CLI 透视测试与非数值聚合报错测试。

**Step 2: Run test to verify it fails**
Run: `cargo test pivot_table --test integration_frame --test integration_cli_json -v`
Expected: FAIL

**Step 3: Write minimal implementation**
- 先支持行维度 + 单列透视 + 单值聚合。
- 再输出稳定列顺序与宽表结果。
- 与现有 `preview_table`、`result_ref` 持久化打通。

**Step 4: Run test to verify it passes**
Run: `cargo test pivot_table --test integration_frame --test integration_cli_json -v`
Expected: PASS

### Task 5: 全量验证与任务日志

**Files:**
- Modify: `D:/Rust/Excel_Skill/.trae/CHANGELOG_TASK.md`

**Step 1: Run full verification**
Run: `cargo test -v`
Run: `cargo build --release -v`
Expected: PASS

**Step 2: Append task journal**
- 只追加本轮任务日志。
- 记录新增文件、修改原因、剩余风险与建议测试点。
