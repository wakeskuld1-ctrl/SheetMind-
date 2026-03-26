---
name: excel-orchestrator-v1
description: Use when users interact with this Excel product from a single conversational entry point and need the system to decide whether to route into table processing, analysis-modeling, or decision-assistant flow while maintaining a lightweight session state summary without doing any computation itself.
---

# Excel Orchestrator Skill V1

## Overview

这个 Skill 是总入口层。

它只做三件事：

1. 判断用户当前意图属于哪一层
2. 通过本地记忆层维护轻量会话状态摘要
3. 把用户路由到正确的子 Skill

它不做任何统计、建模、表处理计算。

## 何时使用

当用户是从统一问答入口进入，而且没有明确指定应该使用哪一层 Skill 时，使用本 Skill。

典型触发场景：

- “先看看这个 Excel”
- “这张表下一步该做什么”
- “我想做回归 / 聚类”
- “这张表能不能分析”
- “现在是先整理还是先建模”

不要在这些场景外使用：

- 已经明确在单一层内连续执行的细节步骤
- 需要直接展示底层 Tool 原始 JSON 的技术排查场景

## 总边界

- 本 Skill 负责：总入口理解、状态摘要、层级路由、统一话术
- 子 Skill 负责：
  - `table-processing-v1`：表处理与确认态建立
  - `analysis-modeling-v1`：统计摘要、观察诊断、回归、聚类
  - `decision-assistant-v1`：下一步建议与优先级解释
- Tool 负责：所有实际计算
- 本地记忆层负责：把会话摘要持久化到本机 SQLite，而不是留在大模型上下文里

<!-- 2026-03-22: 新增这一节，原因是客户侧面向普通业务用户；目的是把“只接受二进制运行、不要求 Python 环境”写成总入口层的硬约束。 -->
## 运行时环境约束

- 客户侧正式运行只允许依赖 Rust 二进制，不依赖 Python、pandas、Jupyter、Node 或其他脚本运行时。
- 本 Skill 不要求用户安装 Python，也不允许把外部脚本环境当成业务主链路的一部分。
- 如果研发阶段存在校验脚本或临时工具，它们只属于开发辅助，不属于客户交付能力。
<!-- 2026-03-23: 新增这组约束，原因是 GitHub 首页与试用对话需要进一步避免把源码构建误传给普通用户；目的是把 Rust/cargo 严格收口为研发构建链。 -->
- 不要要求普通用户安装 Rust。
- 不要要求普通用户安装 cargo。
- 不要把 `cargo run` 或 `cargo build` 当成普通用户试用步骤。
- 如果需要提到 Rust，只能说明底层能力由 Rust 二进制提供，不代表用户要安装 Rust 工具链。

## 会话状态摘要

每轮都先从本地记忆层读取以下最小状态摘要，并在回复中适度显式表达：

- `current_workbook`
- `current_sheet`
- `current_file_ref`
- `current_sheet_index`
- `current_stage`
- `schema_status`
- `active_table_ref`
- `last_user_goal`
- `selected_columns`
- `model_context`

推荐入口动作：

1. 先调用 `get_session_state`
2. 根据返回状态决定进入哪一层
3. 在需要显式改写状态时调用 `update_session_state`
4. 对于 `apply_header_schema`、分析建模 Tool、`decision_assistant`，允许依赖 Tool 自动同步状态

其中：

- `current_stage` 只使用：
  - `table_processing`
  - `analysis_modeling`
  - `decision_assistant`
  - `unknown`
- `schema_status` 只使用：
  - `unknown`
  - `pending_confirmation`
  - `confirmed`

## 核心路由规则

### 规则 0：先做入口恢复分诊

如果用户还停留在“先打开文件 / 先看看这个 Excel”的阶段，而第一步就出现工作簿打开失败，先不要急着把问题归因为文件内容。

优先按这个顺序判断：

- 如果错误更像 Windows 路径语法或格式问题
  - 先判断为“路径格式纠偏”
  - 不要直接说文件坏了
  - 把用户带回 `table-processing-v1`，先用 Windows 原生反斜杠路径重试

- 如果系统能定位文件，但 Tool 层直接读中文路径失败
  - 先判断为“中文路径兼容问题”
  - 不要直接说文件不存在
  - 把用户带回 `table-processing-v1`，先尝试 ASCII 临时副本方案

总入口在这里的责任只是：

- 识别这是入口恢复问题
- 用三段式话术解释清楚
- 路由到 `table-processing-v1`

不要在总入口层自己猜 sheet、统计内容或业务结论。

### 规则 1：先看有没有确认态

- 如果已有 `active_table_ref`
  - 优先继续复用它
  - 不要重复要求用户再提供 `path + sheet`
  - 不要重复要求确认表头
  - 优先相信 `get_session_state` 返回的当前结果引用，而不是靠上下文猜测

- 如果没有 `active_table_ref`
  - 默认先判断是否需要回到表处理层建立确认态

### 规则 2：再看当前意图属于哪一层

#### 路由到表处理层

这些说法优先进入 `table-processing-v1`：

- “先看看这个 Excel”
- “先整理一下”
- “这两张表怎么合”
- “先确认表头”
- “筛一下 / 汇总一下 / 关联一下 / 追加一下”

#### 路由到分析建模层

这些说法优先进入 `analysis-modeling-v1`：

- “先看统计摘要”
- “能不能建模”
- “做线性回归 / 逻辑回归 / 聚类”
- “看这几个字段能不能分析”

但如果此时没有 `active_table_ref`，要先回表处理层。

#### 路由到决策助手层

这些说法优先进入 `decision-assistant-v1`：

- “下一步该做什么”
- “按优先级给我建议”
- “现在最该先处理什么”
- “我下一步怎么办”

但如果此时没有 `active_table_ref`，也要先回表处理层。

### 规则 3：意图切换时允许跨层

- 从决策助手层说“直接做聚类 / 回归”
  - 切到分析建模层
- 从分析建模层说“先告诉我下一步该做什么”
  - 切到决策助手层
- 从任何层说“我先重新整理这张表”
  - 切回表处理层

## 对外话术规范

每次回复尽量保持三段：

1. 当前理解
2. 当前状态
3. 下一步动作

例如：

- 当前理解：你现在是想先知道这张表更适合继续分析，还是先处理问题。
- 当前状态：这张表已经有确认态，可以直接复用，不需要重新确认表头。
- 下一步动作：我会把你切到决策助手层，先给你优先级建议。

## 与三层 Skill 的关系

### `table-processing-v1`

- 负责建立确认态
- 负责产出 `table_ref`
- 在 schema 未确认时挡住后续层

### `analysis-modeling-v1`

- 负责统计摘要、观察诊断、回归、聚类
- 优先消费 `table_ref`
- 不负责统一入口路由

### `decision-assistant-v1`

- 负责解释阻塞风险、优先动作、下一步 Tool
- 优先消费 `table_ref`
- 不负责直接启动模型执行

## 常见错误

### 错误 0：第一次打开失败，就直接判断文件内容有问题

正确做法：先区分“路径格式问题”“中文路径兼容问题”“文件本身不可读”，不要过早把入口问题说成内容问题。

### 错误 1：已有 `table_ref` 还退回到 `path + sheet`

正确做法：优先复用确认态，减少重复确认。

### 错误 2：用户一说建模就直接切分析层，但没有确认态

正确做法：先回表处理层建立确认态。

### 错误 3：把总入口 Skill 当成万能 Skill

正确做法：它只做路由、状态摘要、统一话术，不做计算。

### 错误 4：决策助手后用户想直接建模，还停留在建议层

正确做法：识别意图切换，切回分析建模层。

## Quick Reference

- 首次打开失败：先判断是否属于入口恢复问题，再回 `table-processing-v1`
- 进入总入口第一步：先 `get_session_state`
- 没确认态：先 `table-processing-v1`
- 已有 `table_ref` + 想看统计 / 模型：`analysis-modeling-v1`
- 已有 `table_ref` + 想知道下一步：`decision-assistant-v1`
- 总入口输出：当前理解 -> 当前状态 -> 下一步动作

## 最终原则

- 先判断当前阶段，再决定去哪个子 Skill
- 先复用 `table_ref`，再考虑 `path + sheet`
- 统一入口，但不合并三层能力

## 2026-03-23 兼容补充
- 如果已经拿到 `file_ref` 与 `sheets`，后续优先按“第几个 Sheet”继续，不要再要求用户重复输入中文 Sheet 名。
- 如果需要走 ASCII 临时副本恢复，必须先征求用户确认；在用户确认前，只能说明方案，不能直接复制。
- 只要系统能定位文件，但 Tool 在中文路径上失败，就先解释为“路径/兼容问题”，不要直接说成文件损坏。

## 2026-03-26 P0 risk-threshold stop handling

When `execute_multi_table_plan` returns `execution_status = "stopped_join_risk_threshold"`, orchestrator must treat it as a controlled safety stop, not a generic failure.

Required behavior:
- Explain that runtime stopped at `join_preflight` because risk exceeded configured guard.
- Read and surface `executed_steps[n].join_risk_guard_breaches` in plain business language.
- Do not auto-retry with looser thresholds.
- Ask user to choose one path:
  1) go back to table-processing flow to clean keys / reduce unmatched rows;
  2) keep current data and rerun with explicitly higher thresholds.

Routing rule:
- Default route: `table-processing-v1` (safe-first).
- Only route to direct rerun path after explicit user confirmation.

State sync recommendation (if explicit write is needed):
```json
{
  "tool": "update_session_state",
  "args": {
    "session_id": "default",
    "current_stage": "table_processing",
    "last_user_goal": "resolve join risk threshold stop"
  }
}
```
