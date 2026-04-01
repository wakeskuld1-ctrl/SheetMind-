# Derive Lookup Window Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 `derive_columns`、`lookup_values` / `fill_missing_from_lookup`、`window_calculation` 补齐 V1 必需增强能力，并保持现有单表/多表链路兼容。

**Architecture:** 继续沿用当前 Rust + Polars 的 Tool 分层，不引入新的表达式引擎；优先通过扩展现有参数结构与枚举，分别把复合键、派生规则和窗口计算增强接到现有 dispatcher 与 result_ref 闭环。所有功能都按 TDD 逐条先写红灯测试，再做最小实现。

**Tech Stack:** Rust 2024, Polars, serde/serde_json, cargo test, 现有 dispatcher/result_ref/runtime 体系。

---

### Task 1: 复合键 lookup/fill 设计与红灯测试

**Files:**
- Modify: `D:/Rust/Excel_Skill/tests/integration_frame.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`
- Inspect: `D:/Rust/Excel_Skill/src/ops/lookup_values.rs`
- Inspect: `D:/Rust/Excel_Skill/src/ops/fill_lookup.rs`
- Inspect: `D:/Rust/Excel_Skill/src/tools/dispatcher.rs`

**Step 1: Write the failing tests**
- 为 `lookup_values` 增加复合键成功测试
- 为 `fill_missing_from_lookup` 增加复合键成功测试
- 为“单键/复合键混传”与“复合键数量不一致”增加明确报错测试

**Step 2: Run test to verify it fails**
Run: `cargo test --test integration_frame lookup_values_ -- --nocapture`
Expected: FAIL，提示当前只支持单键或参数结构不兼容

**Step 3: Write minimal implementation**
- 扩展 ops 参数结构，兼容 `base_on/lookup_on` 与 `base_keys/lookup_keys`
- 统一生成稳定复合键字符串，仅支持等值复合键
- dispatcher 继续兼容旧参数

**Step 4: Run test to verify it passes**
Run: `cargo test --test integration_frame lookup_values_ -- --nocapture`
Expected: PASS

### Task 2: 复合键 CLI 链路转绿

**Files:**
- Modify: `D:/Rust/Excel_Skill/src/tools/dispatcher.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`

**Step 1: Write the failing tests**
- 补 `lookup_values` 复合键 CLI 测试
- 补 `fill_missing_from_lookup` 复合键 CLI 测试

**Step 2: Run test to verify it fails**
Run: `cargo test --test integration_cli_json lookup_values_ -- --nocapture`
Expected: FAIL

**Step 3: Write minimal implementation**
- 在 dispatcher 中解析新键参数并保持旧参数兼容

**Step 4: Run test to verify it passes**
Run: `cargo test --test integration_cli_json lookup_values_ -- --nocapture`
Expected: PASS

### Task 3: derive_columns 红灯测试

**Files:**
- Modify: `D:/Rust/Excel_Skill/tests/integration_frame.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`
- Inspect: `D:/Rust/Excel_Skill/src/ops/derive.rs`

**Step 1: Write the failing tests**
- `all` / `any` 条件组测试
- 日期分段测试
- 模板拼接 / 推荐原因文本生成测试

**Step 2: Run test to verify it fails**
Run: `cargo test --test integration_frame derive_columns_ -- --nocapture`
Expected: FAIL

**Step 3: Write minimal implementation**
- 在 `DerivationSpec` 中新增最小规则类型
- 保持现有 `case_when` / `bucket` / `score` 不破坏

**Step 4: Run test to verify it passes**
Run: `cargo test --test integration_frame derive_columns_ -- --nocapture`
Expected: PASS

### Task 4: derive_columns CLI 链路转绿

**Files:**
- Modify: `D:/Rust/Excel_Skill/src/tools/dispatcher.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`

**Step 1: Write the failing tests**
- `derive_columns` CLI 走通条件组/日期分段/模板列

**Step 2: Run test to verify it fails**
Run: `cargo test --test integration_cli_json derive_columns_ -- --nocapture`
Expected: FAIL

**Step 3: Write minimal implementation**
- 如果 ops 层枚举变更需要 dispatcher 兼容，补最小解析适配

**Step 4: Run test to verify it passes**
Run: `cargo test --test integration_cli_json derive_columns_ -- --nocapture`
Expected: PASS

### Task 5: window_calculation 红灯测试

**Files:**
- Modify: `D:/Rust/Excel_Skill/tests/integration_frame.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`
- Inspect: `D:/Rust/Excel_Skill/src/ops/window.rs`

**Step 1: Write the failing tests**
- `lag` / `lead`
- `percent_rank`
- `rolling_sum` / `rolling_mean`

**Step 2: Run test to verify it fails**
Run: `cargo test --test integration_frame window_calculation_ -- --nocapture`
Expected: FAIL

**Step 3: Write minimal implementation**
- 扩展 `WindowCalculationKind`
- 只支持固定窗口大小与最小必要参数

**Step 4: Run test to verify it passes**
Run: `cargo test --test integration_frame window_calculation_ -- --nocapture`
Expected: PASS

### Task 6: 全量验证与任务日志

**Files:**
- Modify: `D:/Rust/Excel_Skill/.trae/CHANGELOG_TASK.md`

**Step 1: Run targeted regression**
Run: `cargo test --test integration_frame -- --nocapture`
Expected: PASS

**Step 2: Run full verification**
Run: `cargo test -v`
Expected: PASS

**Step 3: Build release**
Run: `cargo build --release -v`
Expected: PASS

**Step 4: Append task journal**
- 追加 UTF-8 中文记录到 `D:/Rust/Excel_Skill/.trae/CHANGELOG_TASK.md`
