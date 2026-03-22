# V1 Phase A Gap Closure Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 收口 V1 真实待办，补稳 `result_ref_store` 边界、日期解析边界、会话激活句柄语义，并顺手清理本轮触达文件中的 UTF-8 乱码。

**Architecture:** 先把“已解决 / V1 待补 / V2 规划”状态文档固化，避免继续混用旧遗留清单；然后严格按 TDD 先补 `integration_registry` / `integration_frame` / `integration_cli_json` 的红灯测试，再做最小实现。实现层保持兼容现有 JSON 协议与本地 runtime，新增能力优先兼容旧字段，不做破坏式切换。

**Tech Stack:** Rust 2024、Polars、Serde JSON、Rusqlite、cargo test / cargo build。

---

### Task 1: 固化真实待办状态表

**Files:**
- Create: `D:/Rust/Excel_Skill/docs/plans/2026-03-22-v1-phase-a-gap-closure.md`
- Modify: `D:/Rust/Excel_Skill/progress.md`
- Modify: `D:/Rust/Excel_Skill/task_plan.md`

**Step 1: 写文档更新需求**
- 输出“已解决 / V1 待补 / V2 规划”三段状态表。

**Step 2: 保存计划与进度**
Run: `powershell -Command "Get-Content D:/Rust/Excel_Skill/progress.md -Tail 40"`
Expected: 能看到新增阶段记录。

### Task 2: 先补 result_ref_store 红灯测试

**Files:**
- Modify: `D:/Rust/Excel_Skill/tests/integration_registry.rs`
- Modify: `D:/Rust/Excel_Skill/src/frame/result_ref_store.rs`

**Step 1: 写失败测试**
- `stored_result_dataset_round_trips_dense_nulls_and_all_null_columns`
- `stored_result_dataset_preserves_non_finite_float_values`
- `stored_result_dataset_round_trips_date_and_datetime_columns`

**Step 2: 跑定向测试确认失败**
Run: `cargo test stored_result_dataset --test integration_registry -v`
Expected: 新增测试失败，失败原因是 `result_ref_store` 还不能正确保留非有限浮点值或日期/时间类型。

**Step 3: 写最小实现**
- 扩展 `PersistedColumnType` 与持久化/恢复逻辑。
- 保持旧 JSON 结构兼容，不破坏已有 4 类类型。

**Step 4: 重跑定向测试确认通过**
Run: `cargo test stored_result_dataset --test integration_registry -v`
Expected: PASS

### Task 3: 先补 parse_datetime 与会话激活句柄红灯测试

**Files:**
- Modify: `D:/Rust/Excel_Skill/tests/integration_frame.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`
- Modify: `D:/Rust/Excel_Skill/src/ops/parse_datetime.rs`
- Modify: `D:/Rust/Excel_Skill/src/runtime/local_memory.rs`
- Modify: `D:/Rust/Excel_Skill/src/tools/dispatcher.rs`

**Step 1: 写失败测试**
- `parse_datetime_columns_rejects_invalid_calendar_dates`
- `parse_datetime_columns_accepts_excel_serial_date_numbers`
- `get_session_state_exposes_active_handle_summary`

**Step 2: 跑定向测试确认失败**
Run: `cargo test parse_datetime_columns --test integration_frame --test integration_cli_json -v`
Run: `cargo test get_session_state --test integration_cli_json -v`
Expected: 失败，说明当前还不支持真实日历校验、Excel 序列值或显式激活句柄摘要。

**Step 3: 写最小实现**
- 补真实日期合法性校验与 Excel 序列值解析。
- 在 session state 响应里增加兼容式 `active_handle` 摘要，底层仍兼容旧 `active_table_ref`。

**Step 4: 重跑定向测试确认通过**
Run: `cargo test parse_datetime_columns --test integration_frame --test integration_cli_json -v`
Run: `cargo test get_session_state --test integration_cli_json -v`
Expected: PASS

### Task 4: 清理本轮触达文件乱码并做全量验证

**Files:**
- Modify: `D:/Rust/Excel_Skill/src/tools/dispatcher.rs`
- Modify: `D:/Rust/Excel_Skill/src/ops/join.rs`
- Modify: `D:/Rust/Excel_Skill/.trae/CHANGELOG_TASK.md`

**Step 1: 清理 UTF-8 乱码**
- 只清理本轮触达区域与 `join.rs` 全文件，避免无关扩散。

**Step 2: 跑验证**
Run: `cargo test -v`
Run: `cargo build --release -v`
Expected: 全部通过。

**Step 3: 追加任务日志**
- 按 `task-journal` 模板只追加，不改历史。
