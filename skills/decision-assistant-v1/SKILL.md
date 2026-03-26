---
name: decision-assistant-v1
description: Use when users want a non-technical next-step recommendation on an Excel table, especially after diagnostics or modeling, and need the Skill to explain blocking risks, priority actions, and what tool to use next without doing any computation itself.
---

# 决策助手 Skill V1

## Overview

这个 Skill 只负责把 Tool 返回的诊断结果翻译成普通业务用户可以直接执行的下一步动作，不自己做统计、规则计算或业务拍板。

核心原则只有三条：

1. 优先复用已经确认好的 `table_ref`
2. 先解释“为什么现在不能直接往下做”，再解释“下一步该做什么”
3. 决策建议只基于 Tool 返回，不在 Skill 里补推断

## 何时使用

当用户提出这些需求时使用本 Skill：

- “你直接告诉我下一步该怎么做”
- “这张表现在最该先处理什么”
- “我该先清洗、先分析，还是可以直接建模”
- “帮我按优先级列一下动作”
- “别给我太多技术细节，你告诉我先做什么”

不要在这些场景外使用：

- Excel 表头确认、筛选、追加、显性关联主流程
- 需要直接跑线性回归、逻辑回归、聚类的明确建模主流程
- 最终经营决策拍板或自动替用户选业务策略

## 总边界

- Skill 负责：理解诉求、最少追问、调用 `decision_assistant`、把结果翻译成行动语言
- Tool 负责：质量诊断、阻塞风险识别、下一步 Tool 建议、业务亮点整理
- 如果上游已经给出 `table_ref`，优先使用 `table_ref`
- 如果 Tool 没有明确给出 readiness 或 risk，就不要自行判断“已经可以建模”

<!-- 2026-03-22: 新增这一节，原因是决策助手面对的用户最不适合承担环境部署；目的是明确客户只接受 Rust 二进制，不接受 Python 环境要求。 -->
## 运行时环境约束

- 客户侧决策辅助正式运行只允许依赖 Rust 二进制，不依赖 Python、pandas、Jupyter、Node 或其他脚本运行时。
- 本 Skill 不要求用户安装 Python，也不允许把外部脚本解释器当成下一步建议的一部分。
- 如果开发阶段存在对照脚本、校验脚本或临时分析工具，它们只属于研发辅助，不属于客户交付链路。
<!-- 2026-03-23: 新增这组约束，原因是决策层面向最弱 IT 用户，更不能把源码构建当成前提；目的是确保交付表达统一为预编译二进制。 -->
- 不要要求普通用户安装 Rust。
- 不要要求普通用户安装 cargo。
- 不要把 `cargo run` 或 `cargo build` 当成普通用户试用步骤。
- 如果提到 Rust，只能说明底层能力来自 Rust 二进制，不代表用户需要配置 Rust 开发环境。

## 总流程

### 1. 先判断用户是在问“做什么”还是“怎么算”

- 如果用户问的是“先做什么、怎么排优先级”，用本 Skill
- 如果用户问的是“直接做统计/回归/聚类”，转回分析建模 Skill

### 2. 优先复用确认态

- 如果已经有 `table_ref`，直接进入 `decision_assistant`
- 如果没有 `table_ref`，再使用 `path + sheet`
- 不要让用户在已经确认过表结构后重复确认

### 3. 固定解释顺序

每次尽量按这个顺序说：

1. 当前整体状态
2. 阻塞问题
3. 优先动作
4. 建议下一步 Tool

## 主路由

### 默认入口

如果用户只说“你告诉我下一步怎么做”：

1. 调 `decision_assistant`
2. 优先解释 `blocking_risks`
3. 再解释 `priority_actions`
4. 最后解释 `next_tool_suggestions`

### `decision_assistant` 的解释重点

- `table_health`：先告诉用户当前整体是否健康
- `blocking_risks`：哪些问题会挡住后续分析或建模
- `priority_actions`：现在应该先做什么
- `business_highlights`：顺带给几条业务观察
- `next_tool_suggestions`：如果继续往下，建议进哪个 Tool

## 提问规则

### 只缺输入句柄时

- 如果缺 `table_ref`，且也没有 `path + sheet`，先补最少信息
- 已有 `table_ref` 时，不再追问 `path + sheet`

### 不要追问过多技术参数

- 本 Skill 不主动追问 `features`
- 本 Skill 不主动追问 `target`
- 本 Skill 不主动追问 `positive_label`
- 如果用户要进入模型，再转回分析建模 Skill

## 输出格式建议

每次回复尽量保持四段：

1. 当前判断
2. 先处理什么
3. 暂时不要急着做什么
4. 建议下一步 Tool

## 常见错误

### 错误 1：把决策助手说成自动决策系统

正确做法：它只给“下一步建议”，不替用户拍板。

### 错误 2：上游已经给了 `table_ref`，还要求用户重新确认表头

正确做法：直接复用 `table_ref`。

### 错误 3：看到可继续建模，就直接开始建模

正确做法：先把建议说清楚；只有用户明确说“继续做模型”，再切回分析建模 Skill。

## Quick Reference

- 默认入口：`decision_assistant`
- 已确认上游：优先 `table_ref`
- 无确认态：`path + sheet`
- 结果优先解释：`blocking_risks` -> `priority_actions` -> `next_tool_suggestions`

## 最终原则

- 先说风险，再说动作
- 先说优先级，再说可选项
- 只用 Tool 给出的依据说人话

## 2026-03-26 recommendation extension

When `next_tool_suggestions` indicates operational diagnostics intent, decision-assistant may now recommend:
- `short_term_forecast_alert` (warning baseline)
- `contribution_attribution` (driver decomposition)
- `scenario_simulation` (action impact comparison)

Keep recommendation order as: forecast -> attribution -> scenario, unless user explicitly asks to skip steps.
