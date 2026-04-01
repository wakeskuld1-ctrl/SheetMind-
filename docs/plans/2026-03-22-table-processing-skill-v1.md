# Table Processing Skill V1 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为当前 Rust Excel 二进制落一个只负责编排、不承担计算的 `表处理 Skill V1`，让单表处理和两表工作流先进入可问答使用状态，并把多表计划作为解释层先暴露出来。

**Architecture:** 在仓库内新增一个 Skill 目录，Skill 主文件负责定义触发条件、路由顺序、确认话术和边界约束；辅助文档负责沉淀验收场景与典型路由示例。Skill 只依赖现有 Tool 契约，不新增计算逻辑，也不伪装尚未实现的链式执行能力。

**Tech Stack:** Codex Skill Markdown、现有 Rust Tool JSON 契约、项目内设计/计划文档

---

### Task 1: 固化 Skill 设计边界

**Files:**
- Create: `D:/Rust/Excel_Skill/docs/plans/2026-03-22-table-processing-skill-v1-design.md`

**Step 1: 写设计文档**
- 明确 Skill 只负责编排与追问
- 明确单表 / 双表 / 多表三段式路由
- 明确 V1 对 `step_n_result` 的边界描述

**Step 2: 自查设计是否与现有 Tool 一致**

Run: 手工对照 `D:/Rust/Excel_Skill/src/tools/contracts.rs` 与 `D:/Rust/Excel_Skill/src/tools/dispatcher.rs`
Expected: 设计中引用的 Tool 都已经存在，且没有写入未实现的执行能力

### Task 2: 创建 Skill 主文件

**Files:**
- Create: `D:/Rust/Excel_Skill/skills/table-processing-v1/SKILL.md`

**Step 1: 先写验收目标**
- Skill 名称、触发条件、职责边界
- 必须强调“先建议、后执行”

**Step 2: 写主路由**
- 单表入口：`normalize_table` -> `preview_table` -> 目标 Tool
- 双表入口：`suggest_table_workflow` -> `append_tables` / `join_tables`
- 多表入口：`suggest_multi_table_plan` -> 逐步确认

**Step 3: 写常见追问模板**
- 表头确认
- 追加确认
- 关联确认
- 保留方式确认

**Step 4: 写限制与禁止项**
- 禁止 Skill 自己推算列关系
- 禁止跳过表头确认
- 禁止伪装 `step_n_result` 已可跨调用执行

### Task 3: 补辅助验收场景文档

**Files:**
- Create: `D:/Rust/Excel_Skill/skills/table-processing-v1/cases.md`

**Step 1: 写 6-8 个典型场景**
- 单表查看前几行
- 单表分组汇总
- 两表显性关联
- 两表同结构追加
- 多表先追加再关联
- 表头不明确需要确认

**Step 2: 为每个场景写出**
- 用户说法
- Skill 应调用的 Tool
- Skill 应如何提问
- 不应该做什么

### Task 4: 做一致性检查并收口任务日志

**Files:**
- Modify: `D:/Rust/Excel_Skill/.trae/CHANGELOG_TASK.md`

**Step 1: 检查 Skill 与 Tool 契约一致**

Run: 手工核对 `D:/Rust/Excel_Skill/skills/table-processing-v1/SKILL.md` 与 `D:/Rust/Excel_Skill/src/tools/contracts.rs`
Expected: Skill 中引用的 Tool 名称与当前目录完全一致

**Step 2: 检查 Skill 与 dispatcher 能力边界一致**

Run: 手工核对 `D:/Rust/Excel_Skill/skills/table-processing-v1/SKILL.md` 与 `D:/Rust/Excel_Skill/src/tools/dispatcher.rs`
Expected: Skill 不会要求当前 dispatcher 执行 `result_ref` 链式调用

**Step 3: 追加任务日志**
- 只追加本轮任务日志，不改历史内容
