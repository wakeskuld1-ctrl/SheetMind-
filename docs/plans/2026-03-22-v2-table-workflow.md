# V2 Table Workflow Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 新增 `suggest_table_workflow` Tool，为 V2 多表工作流层提供“两张表下一步动作建议”能力。

**Architecture:** 在 `ops` 层新增多表流程建议模块，复用现有 `suggest_table_links` 和表结构判断规则，把“推荐追加 / 推荐关联 / 继续确认”统一收敛成一个稳定输出；在 `dispatcher` 和 `contracts` 中接入新 Tool；用内存层与 CLI 测试锁定推荐动作。

**Tech Stack:** Rust 2024、Polars 0.51、Serde、现有 CLI JSON 调度架构

---

### Task 1: 写失败测试

**Files:**
- Modify: `D:/Rust/Excel_Skill/tests/integration_frame.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`

**Step 1: Write the failing test**
- 增加内存层 / 真实 Excel 测试：
  - 结构一致表会优先推荐 `append_tables`
  - 显性可关联表会推荐 `join_tables`
  - 没有明显关系时会回退 `manual_confirmation`
- 增加 CLI 测试：
  - `tool_catalog` 包含 `suggest_table_workflow`
  - 真实 Excel 夹具下返回追加建议 / 关联建议

**Step 2: Run test to verify it fails**
Run: `cargo test suggest_table_workflow --test integration_frame --test integration_cli_json -v`
Expected: FAIL，提示模块或 Tool 不存在

### Task 2: 实现多表流程建议模块

**Files:**
- Create: `D:/Rust/Excel_Skill/src/ops/table_workflow.rs`
- Modify: `D:/Rust/Excel_Skill/src/ops/mod.rs`

**Step 1: Write minimal implementation**
- 复用 `suggest_table_links`
- 增加结构一致判断与追加候选输出
- 给出统一的推荐动作、原因、中文摘要与下一步建议

**Step 2: Run test to verify it still needs dispatcher/catalog wiring**
Run: `cargo test suggest_table_workflow --test integration_frame --test integration_cli_json -v`
Expected: frame 相关测试通过或前进，CLI 因未接线继续失败

### Task 3: 接入目录与调度层

**Files:**
- Modify: `D:/Rust/Excel_Skill/src/tools/contracts.rs`
- Modify: `D:/Rust/Excel_Skill/src/tools/dispatcher.rs`

**Step 1: Write minimal implementation**
- 更新工具目录
- 在 `dispatcher` 中接入 `suggest_table_workflow`
- 复用 `left/right`、`left_casts/right_casts`、`max_link_candidates` 参数模式

**Step 2: Run test to verify it passes**
Run: `cargo test suggest_table_workflow --test integration_frame --test integration_cli_json -v`
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
