# V2 Multi Table Plan Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 新增 `suggest_multi_table_plan` Tool，为 V2 多表工作流层提供“先追加、再关联、保留未决表”的保守顺序建议能力。

**Architecture:** 在 `ops` 层新增多表计划模块，复用 `append_tables` 与 `suggest_table_links`，先做同结构表追加链，再做显性关联链，并输出带 `result_ref` 的建议执行骨架；在 `dispatcher` 和 `contracts` 中接入新 Tool；用内存层与 CLI 测试锁定计划步骤。

**Tech Stack:** Rust 2024、Polars 0.51、Serde、Serde JSON、现有 CLI JSON 调度架构

---

### Task 1: 写失败测试

**Files:**
- Modify: `D:/Rust/Excel_Skill/tests/integration_frame.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`

**Step 1: Write the failing test**
- 增加内存层 / 真实 Excel 测试：
  - 三张同结构表会生成两步 `append_tables` 计划
  - 两张可关联表会生成一步 `join_tables` 计划
  - 没有明显关系的表会进入 `unresolved_refs`
- 增加 CLI 测试：
  - `tool_catalog` 包含 `suggest_multi_table_plan`
  - 多表追加计划与双表关联计划可通过 CLI 返回

**Step 2: Run test to verify it fails**
Run: `cargo test suggest_multi_table_plan --test integration_frame --test integration_cli_json -v`
Expected: FAIL，提示模块或 Tool 不存在

### Task 2: 实现多表计划模块

**Files:**
- Create: `D:/Rust/Excel_Skill/src/ops/multi_table_plan.rs`
- Modify: `D:/Rust/Excel_Skill/src/ops/mod.rs`

**Step 1: Write minimal implementation**
- 定义步骤结构、计划结果结构与 `result_ref`
- 先做同结构追加链
- 再做显性关联链
- 最后保留未决表

**Step 2: Run test to verify frame path improves**
Run: `cargo test suggest_multi_table_plan --test integration_frame --test integration_cli_json -v`
Expected: frame 测试通过或前进，CLI 因未接线继续失败

### Task 3: 接入目录与调度层

**Files:**
- Modify: `D:/Rust/Excel_Skill/src/tools/contracts.rs`
- Modify: `D:/Rust/Excel_Skill/src/tools/dispatcher.rs`

**Step 1: Write minimal implementation**
- 更新工具目录
- 在 `dispatcher` 中接入 `suggest_multi_table_plan`
- 支持 `tables` 与 `max_link_candidates`

**Step 2: Run test to verify it passes**
Run: `cargo test suggest_multi_table_plan --test integration_frame --test integration_cli_json -v`
Expected: PASS

### Task 4: 全量验证与日志收口

**Files:**
- Modify: `D:/Rust/Excel_Skill/.trae/CHANGELOG_TASK.md`

**Step 1: Run full verification**
Run: `cargo test -v`
Run: `cargo build --release -v`
Expected: PASS

**Step 2: Append task journal**
- 只追加本轮任务日志，不修改历史条目
