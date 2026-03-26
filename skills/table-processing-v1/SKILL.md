---
name: table-processing-v1
description: Use when users ask to inspect, clean, reshape, append, or explicitly link Excel tables through this project's Rust tool layer, especially when they need non-technical confirmation questions and the Skill must not perform any computation itself.
---

# 表处理 Skill V1

## Overview

这个 Skill 只负责“理解需求 -> 选 Tool -> 追问确认 -> 给下一步建议”，不负责任何计算。

核心原则只有两条：

1. 先调用 Rust Tool 判断，再决定怎么问用户
2. Skill 绝不自己做计算、猜关系、猜类型、猜结构

## 何时使用

当用户提出这些需求时使用本 Skill：

- “帮我看这张表”
- “筛一下、排一下、汇总一下”
- “这两张表是上下拼接还是关联”
- “多张表应该先处理哪几张”
- “这个 Excel 先怎么整理”

不要在这些场景外使用：

- 线性回归、逻辑回归、聚类、决策助手主流程
- 需要真正多步中间结果自动执行到底的复杂流水线

## 总边界

- Skill 负责：路由、追问、解释、下一步建议
- Tool 负责：读取、识别、转换、分析、追加、关联、统计
- 如果 Tool 没明确返回，就不要在 Skill 里补猜测

<!-- 2026-03-22: 新增这一节，原因是表处理层最容易被误用成脚本入口；目的是明确客户运行时只能使用 Rust 二进制，不引入 Python 环境负担。 -->
## 运行时环境约束

- 客户侧表处理正式运行只允许依赖 Rust 二进制，不依赖 Python、pandas、Jupyter、Node 或其他脚本环境。
- 本 Skill 不要求用户安装 Python，也不允许建议用户改走 Python 脚本来完成读取、清洗、追加或关联。
- 如果开发阶段存在临时校验脚本，它们只用于研发验证，不属于客户交付链路。
<!-- 2026-03-23: 新增这组约束，原因是表处理层最容易在“先试一试”场景里误把 cargo 当用户入口；目的是锁定普通用户只接触预编译二进制。 -->
- 不要要求普通用户安装 Rust。
- 不要要求普通用户安装 cargo。
- 不要把 `cargo run` 或 `cargo build` 当成普通用户试用步骤。
- 如果需要说明 Rust，只能解释这是底层二进制实现方式，不是普通用户的部署动作。

## 总流程

### 0. 先处理文件入口恢复

如果用户在“先打开 / 先看看这份 Excel”阶段就失败了，先不要立刻判断文件内容有问题。

优先按下面顺序分诊：

- 如果错误更像 Windows 路径语法或格式不正确
  - 先做路径格式纠偏
  - 改用 Windows 原生反斜杠路径重试

- 如果系统能定位文件，但 Tool 层直接读中文路径失败
  - 先做中文路径兼容降级
  - 如果宿主环境支持文件复制能力，先复制一份只用于分析的 ASCII 临时副本，再继续打开工作簿

- 只有前两步都不成立或都失败，才把问题上升为“文件当前不可读”

这一段属于入口恢复，不属于 Excel 计算本身。

### 1. 先判断问题属于哪一类

- 单表问题：只涉及一张表
- 双表问题：只涉及两张表
- 多表问题：涉及三张及以上的表

### 2. 先建议，后执行

只要有对应的建议型 Tool，就先走建议型 Tool：

- 双表先用 `suggest_table_workflow`
- 多表先用 `suggest_multi_table_plan`
- 显性关联先用 `suggest_table_links`

不要一上来就直接 `join_tables` 或 `append_tables`。

### 3. 先确认表头，再做后续处理

如果 `normalize_table` 返回 `needs_confirmation`：

- 必须先让用户确认表头
- 不要继续调用后续表处理 Tool

如果用户确认表头，并且随后调用了 `apply_header_schema`：

- 要明确告诉用户这一步会产出可复用的 `table_ref`
- 后续进入分析建模层时，优先把 `table_ref` 交给上层 Skill
- 不要让用户在已经确认过表头后，再重复走一次确认流程

## 单表路由

### 单表入口

如果用户刚给了一个文件或一张 Sheet：

1. 如果文件入口失败，先做路径恢复
2. 调 `open_workbook` 或 `normalize_table`
3. 如果需要确认表头，就先问
4. 如果表头明确，可先调 `preview_table` 看内容，或调 `apply_header_schema` 固化确认态
5. 如果已经进入分析建模层，优先把 `table_ref` 交给上层 Skill
6. 再根据目标调用后续 Tool

### 单表操作路由

- 看前几行：`preview_table`
- 看列、保留部分列：`select_columns`
- 按条件筛选：`filter_rows`
- 调整列类型：`cast_column_types`
- 分组汇总：`group_and_aggregate`
- 排序：`sort_rows`
- 取前 N 行：`top_n`
- 先了解整表概况：`stat_summary`

### 单表执行模板规则

单表场景优先使用固定 JSON 模板，不要自由拼装参数。

- 表头确认前：只允许使用 `normalize_table`
- 表头确认后：优先使用 `preview_table` 或 `apply_header_schema`
- 用户只是想先了解整表：优先使用 `stat_summary`
- 用户明确说“只保留几列”：使用 `select_columns`
- 用户明确说“筛出满足条件的记录”：使用 `filter_rows`
- 用户明确说“按某列汇总”：使用 `group_and_aggregate`

具体请求骨架见 `requests.md`。

### 单表提问风格

统一用业务表达，不用技术词：

- 说“只看这几列”，不说“投影”
- 说“筛出满足条件的行”，不说“谓词过滤”
- 说“把这列转成数值”，不说“cast”
- 说“路径格式不对”或“中文路径兼容有问题”，不说“库层编码异常”

## 双表路由

### 双表先判断动作类型

先调 `suggest_table_workflow`。

根据返回结果处理：

- `append_tables`
  - 问法：是否先把 B 表追加到 A 表下方，形成一张统一结果表？
- `join_tables`
  - 问法：是否用 A 表某列关联 B 表某列？
  - 继续问：如果两边不一致，你希望只保留两边都有的数据，还是优先保留 A 表或 B 表？
- `manual_confirmation`
  - 不执行
  - 继续收集用户业务意图

### 双表禁止项

- 不要自己决定 Join 键
- 不要自己决定保留方式
- 不要跳过 `suggest_table_workflow`

### 双表执行模板规则

双表场景必须先走建议型请求，再决定执行型请求：

1. 先发 `suggest_table_workflow`
2. 如果返回 `append_tables`，再发 `append_tables`
3. 如果返回 `join_tables`，先问用户保留方式，再发 `join_tables`

不要跳过第一步，直接拼 `append_tables` 或 `join_tables`。

## 多表路由

### 多表先给计划

三张及以上的表，先调 `suggest_multi_table_plan`。

Skill 需要做的事：

1. 用自然语言解释步骤顺序
2. 逐步使用返回的 `question` 与用户确认
3. 明确告诉用户当前是“先出计划，再按步骤执行”

### 多表 V1 限制

当前 V1 要明确说清楚：

- `step_n_result` 仍然是计划层占位引用，不是已经落地的真实 `result_ref`
- 但如果后续步骤返回了 `pending_result_bindings`，可以把前一步真实产出的 `result_ref` 通过 `result_ref_bindings` 回填后继续执行
- 所以 V1 多表能力已经能做“计划 + 分步串接执行”，但还不是“任意多步无确认自动执行到底”

如果用户要求一口气跑完整个多表链：

- 可以先给计划
- 可以执行第一步
- 但不要假装后续步骤已经具备完整自动串接能力

### 多表执行模板规则

多表场景当前只固定三类请求：

1. 计划请求：`suggest_multi_table_plan`
2. 第一步追加执行：如果第一个步骤是 `append_tables`，只执行第一步
3. 第一步关联执行：如果第一个步骤是 `join_tables`，只执行第一步

如果计划中的第二步依赖 `step_n_result`：

- 要先看这一步的 `pending_result_bindings`
- 如果需要绑定，就把前一步真实返回的 `result_ref` 按 `alias -> result_ref` 写进 `result_ref_bindings`
- 如果还没拿到前一步真实结果，就继续解释计划，不要伪造执行结果

## 标准追问模板

### 表头确认

- “这张表的表头我还不能 100% 确认。你看一下这几个列名是否符合你的意思？”

### 追加确认

- “这两张表结构看起来一致。是否先把 B 表追加到 A 表下方，形成一张统一结果表？”

### 关联确认

- “我识别到 A 表的 `{left_column}` 和 B 表的 `{right_column}` 很可能是对应字段。是否用这两列来关联？”

### 保留范围确认

- “如果两边数据不完全一致，你希望只保留两边都有的数据，还是优先保留 A 表，或者优先保留 B 表？”

## 输出格式建议

每次回复尽量保持三段：

1. 当前理解
2. 下一步动作或确认问题
3. 若确认后，将调用哪个 Tool

示例：

- 当前理解：你想把客户表和订单表合到一起看
- 下一步问题：是否用客户表的 `user_id` 关联订单表的 `user_id`？
- 确认后动作：调用 `join_tables`

## 固定请求模板

本 Skill 优先使用固定模板，而不是自由组织 JSON。

- 单表模板：见 `requests.md` 中的“单表模板”
- 双表模板：见 `requests.md` 中的“双表模板”
- 多表模板：见 `requests.md` 中的“多表模板”

使用规则：

- 缺 `path` / `sheet` 时，先补信息，不发请求
- 参数不完整时，先追问，不猜测
- 只在用户确认后发送执行型请求

## 常见错误

### 错误 0：一看到打开失败就说文件坏了

正确做法：

- 先判断是不是路径格式问题
- 再判断是不是中文路径兼容问题
- 最后才判断文件本身不可读

### 错误 1：Skill 自己猜 Join 键

错误做法：

- 直接说“我帮你按 user_id 关联好了”

正确做法：

- 先调 `suggest_table_workflow` 或 `suggest_table_links`
- 再把 Tool 给出的候选问给用户

### 错误 2：表头没确认就继续算

错误做法：

- `normalize_table` 返回 `needs_confirmation` 后继续做 `group_and_aggregate`

正确做法：

- 先停下确认表头

### 错误 3：把多表计划当成自动流水线

错误做法：

- 看到 `step_1_result` 后直接假设后续调用已经能跨步骤复用

正确做法：

- 把 `step_n_result` 当成计划层引用
- 明确告知当前 V1 的自动执行边界

## Quick Reference

- 首次打开失败：先做路径恢复，再继续表处理
- 先列出工作簿 sheet：`open_workbook`
- 单表起手：`normalize_table` -> `preview_table` / `apply_header_schema`
- 单表概况：`stat_summary`
- 双表起手：`suggest_table_workflow`
- 多表起手：`suggest_multi_table_plan`
- 显性关联候选：`suggest_table_links`
- 真正执行追加：`append_tables`
- 真正执行关联：`join_tables`
- 跨层交接：`apply_header_schema` -> `table_ref` -> 分析建模 / 决策助手
- 固定请求骨架：`requests.md`

## 最终原则

- 先确认结构，再做处理
- 先建议动作，再做执行
- 先用 Tool 给依据，再用 Skill 说人话
- 任何计算都留在 Rust Tool 层

## 2026-03-23 兼容补充
- 面向用户时统一说“文件”“第几个 Sheet”，不要使用偏技术化的内部术语。
- 如果 `open_workbook` 或 `list_sheets` 已返回 `file_ref` 与 `sheets`，后续内部优先走 `file_ref + sheet_index`。
- 如果要复制 ASCII 临时副本，必须先征求用户确认；复制只用于兼容读取，不代表修改原文件。
- 只要系统能定位文件，但 Tool 在中文路径上失败，就先解释为“路径/兼容问题”，不要直接判断成文件损坏。

## 2026-03-26 default execute policy (P0)

For multi-table end-to-end execution:
- Prefer `execute_multi_table_plan`.
- If user wants auto execution (`auto_confirm_join=true`), keep the runtime default join risk guard unless user provides custom thresholds.
- When runtime returns `stopped_join_risk_threshold`, explain breached metrics and ask whether to:
  1) clean keys / fix source tables, or
  2) raise thresholds and retry.

## 2026-03-26 P1 ingress recovery playbook

Before any table computation request, apply this recovery order:

1. Path format correction (Windows syntax first)
2. Chinese-path compatibility fallback (ASCII temp-copy only after explicit user confirmation)
3. Controlled stop recovery
4. Unknown failure fallback

### Controlled stop recovery rules

- `stopped_join_risk_threshold`
  - Explain exceeded risk metrics from `join_risk_guard_breaches`.
  - Default action: key cleanup or join-condition correction first.
  - Do not auto-loosen thresholds.

- `stopped_missing_result_bindings`
  - Explain missing aliases from stop reason or step metadata.
  - Default action: rebuild/replay missing prior step and then rerun blocked step.

- `stopped_needs_preflight_confirmation`
  - Ask explicit user confirmation before proceeding.

This recovery branch is host/workflow preparation, not a new Excel computation primitive.

## 2026-03-26 P2 unknown failure diagnostics handling

If `execute_multi_table_plan` returns:
- `execution_status = "failed"`
- `failure_diagnostics.failure_class = "unknown_runtime_failure"`

Then table-processing should:
1. Explain this is an unclassified runtime/tool failure, not a completed plan result.
2. Use `failure_diagnostics.failed_step_id` and `failed_action` to localize the blocked step.
3. Check `failure_diagnostics.state_synced`; if false, run `recovery_templates.update_session_state` first.
4. Prefer `failure_diagnostics.recovery_templates` to resume execution from the blocked step or full chain.
5. If blocked-step replay returns `continuation_templates`, use `continuation_templates.resume_full_chain` for direct continuation.
6. Keep default route as diagnostics-first in table-processing, then retry after inputs are fixed.
7. If you want runtime to orchestrate replay + continuation automatically, call `recover_multi_table_failure`.
8. If replay/continue thresholds need tuning, pass `template_overrides` (or legacy `template_arg_overrides`) to patch template args only.
9. Read `applied_template_overrides` / `ignored_template_overrides`; use `strict_template_overrides=true` if unknown override keys must fail fast.

Do not route this branch directly into analysis-modeling or decision-assistant.
