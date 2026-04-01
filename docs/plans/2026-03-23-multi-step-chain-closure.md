# Multi-Step Chain Closure Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 补齐 `table_ref / result_ref / workbook_ref / session_state` 的多步链式闭环，让最新产出句柄能自动回写到本地会话状态。

**Architecture:** 保留现有 `active_table_ref` 兼容字段，在本地记忆层新增显式 `active_handle_ref / active_handle_kind`；dispatcher 统一通过“输出句柄状态同步”辅助函数，把新产生的 `result_ref` 与 `workbook_ref` 回写到 `session_state`，并补 CLI / runtime 集成测试锁定行为。

**Tech Stack:** Rust 2024、Serde、Serde JSON、Rusqlite、Polars、现有 CLI JSON Tool 调度架构

---

### Task 1: 扩展 session state 结构与 SQLite 迁移

**Files:**
- Modify: `D:/Rust/Excel_Skill/src/runtime/local_memory.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_registry.rs`

**Step 1: Write the failing test**

- 在 `integration_registry` 增加 session round-trip 测试：
  - `active_handle_ref`
  - `active_handle_kind`
  - `active_table_ref` 兼容保留

**Step 2: Run test to verify it fails**

Run: `cargo test runtime_persists_session_state_round_trip --test integration_registry -v`
Expected: FAIL，提示字段不存在或断言不成立

**Step 3: Write minimal implementation**

- 扩展 `SessionState`
- 扩展 `SessionStatePatch`
- 扩展 SQLite `session_state` 表结构与兼容迁移
- `get_session_state` / `update_session_state` 补新字段读写

**Step 4: Run test to verify it passes**

Run: `cargo test runtime_persists_session_state_round_trip --test integration_registry -v`
Expected: PASS

### Task 2: 调整 `update_session_state` 与会话响应兼容语义

**Files:**
- Modify: `D:/Rust/Excel_Skill/src/tools/dispatcher.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`

**Step 1: Write the failing test**

- 增加 CLI 测试：
  - `update_session_state` 可以直接写入 `active_handle_ref / active_handle_kind`
  - `get_session_state` 同时返回 `active_table_ref`、`active_handle_ref`、`active_handle.kind`

**Step 2: Run test to verify it fails**

Run: `cargo test get_session_state_exposes_active_handle_summary --test integration_cli_json -v`
Expected: FAIL

**Step 3: Write minimal implementation**

- 扩展 `UpdateSessionStateInput`
- 调整 `build_session_state_response`
- 兼容旧字段推断逻辑

**Step 4: Run test to verify it passes**

Run: `cargo test get_session_state_exposes_active_handle_summary --test integration_cli_json -v`
Expected: PASS

### Task 3: 为产出新 `result_ref` 的 Tool 接入统一输出句柄回写

**Files:**
- Modify: `D:/Rust/Excel_Skill/src/tools/dispatcher.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`

**Step 1: Write the failing test**

- 增加 CLI 测试：
  - `group_and_aggregate` 后 `get_session_state.active_handle_ref == 新 result_ref`
  - `append_tables` 后状态切到新 `result_ref`
  - `join_tables` 后状态切到新 `result_ref`

**Step 2: Run test to verify it fails**

Run: `cargo test active_handle_ref --test integration_cli_json -v`
Expected: FAIL，状态仍停留在输入句柄

**Step 3: Write minimal implementation**

- 新增统一辅助函数：
  - 识别句柄类型
  - 回写 `active_handle_ref / active_handle_kind`
  - 保留最近确认态 `active_table_ref`
- 在 `respond_with_result_dataset` / `respond_with_preview_and_result_ref` 路径接入

**Step 4: Run test to verify it passes**

Run: `cargo test active_handle_ref --test integration_cli_json -v`
Expected: PASS

### Task 4: 为 `compose_workbook` 接入 `workbook_ref` 状态同步

**Files:**
- Modify: `D:/Rust/Excel_Skill/src/tools/dispatcher.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`

**Step 1: Write the failing test**

- 增加 CLI 测试：
  - `compose_workbook` 成功后 `get_session_state.active_handle_ref == workbook_ref`
  - `active_handle.kind == workbook_ref`

**Step 2: Run test to verify it fails**

Run: `cargo test compose_workbook_updates_session_state --test integration_cli_json -v`
Expected: FAIL

**Step 3: Write minimal implementation**

- 在 `dispatch_compose_workbook` 成功路径回写会话状态
- 保持 `active_table_ref` 不被 workbook 句柄覆盖

**Step 4: Run test to verify it passes**

Run: `cargo test compose_workbook_updates_session_state --test integration_cli_json -v`
Expected: PASS

### Task 5: 统一收口读取类 Tool 的输入句柄同步

**Files:**
- Modify: `D:/Rust/Excel_Skill/src/tools/dispatcher.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`

**Step 1: Write the failing test**

- 增加 CLI 测试：
  - 对 `stat_summary` 这种纯读取类 Tool，状态应保留输入句柄而不是误造新句柄
  - 若输入是 `result_ref`，`active_handle_ref` 仍为该 `result_ref`

**Step 2: Run test to verify it fails**

Run: `cargo test stat_summary_preserves_input_handle_state --test integration_cli_json -v`
Expected: FAIL

**Step 3: Write minimal implementation**

- 把 `sync_loaded_table_state` 调整为显式输入句柄同步语义
- 避免与输出句柄同步逻辑互相覆盖

**Step 4: Run test to verify it passes**

Run: `cargo test stat_summary_preserves_input_handle_state --test integration_cli_json -v`
Expected: PASS

### Task 6: 全量验证与任务日志

**Files:**
- Modify: `D:/Rust/Excel_Skill/.trae/CHANGELOG_TASK.md`

**Step 1: Run targeted verification**

Run: `cargo test runtime_persists_session_state_round_trip --test integration_registry -v`
Run: `cargo test get_session_state_exposes_active_handle_summary --test integration_cli_json -v`
Run: `cargo test active_handle_ref --test integration_cli_json -v`
Run: `cargo test compose_workbook_updates_session_state --test integration_cli_json -v`
Run: `cargo test stat_summary_preserves_input_handle_state --test integration_cli_json -v`
Expected: PASS

**Step 2: Run full verification**

Run: `cargo test -v`
Run: `cargo build --release -v`
Expected: PASS

**Step 3: Append task journal**

- 只追加本轮任务日志，不改写历史条目
