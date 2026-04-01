# V2 Table Links Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 新增 `suggest_table_links` Tool，为 V2 多表工作流层提供显性关联候选建议能力。

**Architecture:** 在 `ops` 层新增多表关系建议模块，复用现有 Excel 读取、显式 cast、候选键语义与 Join 语义文案；在 `dispatcher` 和 `contracts` 中接入新 Tool；用内存层与 CLI 夹具测试锁定保守候选规则。

**Tech Stack:** Rust 2024、Polars 0.51、Serde、现有 CLI JSON 调度架构

---

### Task 1: 写失败测试

**Files:**
- Modify: `tests/integration_frame.rs`
- Modify: `tests/integration_cli_json.rs`

**Step 1: Write the failing test**
- 增加内存层测试：
  - 明显 `user_id -> user_id` 候选会返回高置信度建议
  - 没有明显交集时不会返回候选
- 增加 CLI 测试：
  - `tool_catalog` 包含 `suggest_table_links`
  - 真实 Excel 夹具会返回 `user_id` 候选与中文问题

**Step 2: Run test to verify it fails**
Run: `cargo test suggest_table_links --test integration_frame --test integration_cli_json -v`
Expected: FAIL，提示模块或 Tool 不存在

**Step 3: Write minimal implementation**
- 暂不写生产代码

**Step 4: Run test to verify it still fails for the expected reason**
Run: `cargo test suggest_table_links --test integration_frame --test integration_cli_json -v`
Expected: FAIL，且原因是缺失实现而不是测试拼写错误

### Task 2: 实现候选键语义复用与关系建议模块

**Files:**
- Modify: `src/ops/semantic.rs`
- Modify: `src/ops/analyze.rs`
- Create: `src/ops/table_links.rs`
- Modify: `src/ops/mod.rs`

**Step 1: Write the failing test**
- 沿用 Task 1 的失败测试

**Step 2: Run test to verify it fails**
Run: `cargo test suggest_table_links --test integration_frame --test integration_cli_json -v`
Expected: FAIL

**Step 3: Write minimal implementation**
- 抽出标识列命名识别 helper
- 实现明显关联候选评分、覆盖率计算、中文问题与 keep_mode 选项输出
- 接入 `ops::mod`

**Step 4: Run test to verify it passes**
Run: `cargo test suggest_table_links --test integration_frame --test integration_cli_json -v`
Expected: PASS

### Task 3: 接入目录与调度层

**Files:**
- Modify: `src/tools/contracts.rs`
- Modify: `src/tools/dispatcher.rs`
- Modify: `tests/integration_cli_json.rs`

**Step 1: Write the failing test**
- 用 CLI 测试锁定目录与 JSON 返回结构

**Step 2: Run test to verify it fails**
Run: `cargo test suggest_table_links --test integration_cli_json -v`
Expected: FAIL

**Step 3: Write minimal implementation**
- 更新工具目录
- 在 `dispatcher` 中接入 `suggest_table_links`
- 复用 `left/right` 输入格式与 `casts`

**Step 4: Run test to verify it passes**
Run: `cargo test suggest_table_links --test integration_cli_json -v`
Expected: PASS

### Task 4: 全量验证与日志收口

**Files:**
- Modify: `.trae/CHANGELOG_TASK.md`

**Step 1: Run focused verification**
Run: `cargo test suggest_table_links --test integration_frame --test integration_cli_json -v`
Expected: PASS

**Step 2: Run full verification**
Run: `cargo test -v`
Run: `cargo build --release -v`
Expected: PASS

**Step 3: Append task journal**
- 只追加本轮任务日志，不修改历史条目
