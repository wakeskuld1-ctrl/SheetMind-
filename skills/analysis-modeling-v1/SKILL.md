---
name: analysis-modeling-v1
description: Use when users ask to assess whether an Excel sheet is ready for analysis or modeling, request statistical diagnostics or summaries, or explicitly ask for regression, logistic classification, or clustering through this project's Rust tool layer, especially when the Skill must decide whether to diagnose first or go straight into model preparation without performing any computation itself.
---

<!-- 2026-03-27 22:20:00 +08:00 ????????? Git ??????? Skill ?????? UTF-8 ? binary-only runtime ??? -->

# 分析建模 Skill V1

## Overview

这个 Skill 只负责路由、追问、解释，不做任何统计计算或模型训练。

核心原则只有三条：

1. 先用 Tool 判断当前表是否适合进入分析或建模
2. Skill 只做路由、追问和解释，不自己计算、不自己下结论
3. 默认先诊断，用户明确点名模型时允许直达，但仍要做最小前置校验

## 何时使用

当用户提出这些需求时使用本 Skill：

- “这张表适不适合分析”
- “先看一下统计摘要”
- “先帮我看数据有没有问题”
- “做线性回归”
- “做逻辑回归”
- “做聚类”
- “这些字段能不能拿来建模”

不要在这些场景外使用：

- Excel 读取、表头确认、筛选、追加、显性关联主流程
- 最终业务决策拍板
- 自动调参、阈值调优、AUC 全展开、多分类 softmax

## 总边界

- Skill 负责：意图识别、路由、最少追问、结果解释、下一步建议
- Tool 负责：统计摘要、质量诊断、建模前准备、回归、分类、聚类
- 如果 Tool 没明确返回，就不要在 Skill 里补猜测
- 如果上游已经给出 `table_ref`，优先复用确认态，不重复要求 `path + sheet`

<!-- 2026-03-22: 新增这一节，原因是分析建模层最容易被误解为可改走 Python notebook；目的是锁定客户运行时只能使用 Rust 二进制。 -->
## 运行时环境约束

- 客户侧分析建模正式运行只允许依赖 Rust 二进制，不依赖 Python、pandas、Jupyter、Node 或其他脚本运行时。
- 本 Skill 不要求用户安装 Python，也不允许建议用户改走 Python notebook、pandas 脚本或临时解释器完成统计与建模。
- 如果研发阶段存在对照脚本或校验工具，它们只属于开发辅助，不属于客户交付链路。
<!-- 2026-03-23: 新增这组约束，原因是分析建模最容易被误解为需要额外开发环境；目的是明确普通用户只消费预编译二进制能力。 -->
- 不要要求普通用户安装 Rust。
- 不要要求普通用户安装 cargo。
- 不要把 `cargo run` 或 `cargo build` 当成普通用户试用步骤。
- 如果提到 Rust，只能作为底层实现说明，不能作为用户前置环境要求。

## 总流程

### 1. 先判断入口类型

- 观察诊断型：用户先想知道“数据怎么样、能不能分析、能不能建模”
- 明确建模型：用户直接说要做线性回归、逻辑回归或聚类

### 2. 默认先诊断，明确建模可直达

- 默认入口：先 `analyze_table`
- 用户明确要看统计摘要：优先 `stat_summary`
- 用户明确点名模型：可以直接进入建模准备层
- 但即使直达模型，也必须补齐最小前置校验

### 3. 最小前置校验

进入 `linear_regression`、`logistic_regression`、`cluster_kmeans` 前，Skill 必须确认：

- 表头是否已确认
- 关键列是否齐全
- 是否需要先做轻量诊断
- 缺失值处理方式是否明确
- 必要时是否需要补充列类型说明

## 观察诊断路由

### 默认观察入口

如果用户刚给出一个文件或一张 Sheet，并且只表达“先看看”：

1. 先确认表结构是否已经可分析
2. 默认调用 `analyze_table`
3. 如果用户还想看分布或统计摘要，再调用 `stat_summary`
4. 再根据结果建议进入回归、分类、聚类，或退回表处理层

如果上游已经完成表头确认并拿到 `table_ref`：

1. 直接优先使用 `table_ref`
2. 不要再让用户重复提供 `path + sheet`
3. 不要再让用户重复确认表头

### 观察诊断 Tool 路由

- 看整体健康度与风险：`analyze_table`
- 看数值、类别、布尔统计摘要：`stat_summary`
- 看更基础的列画像：`summarize_table`

## 建模前公共准备层

这一层是线性回归、逻辑回归、聚类共用的桥接层。

### 必问参数

- `table_ref` 或 `path + sheet`
- `features`
- 如果是回归或分类，还要确认 `target`
- 如果是逻辑回归，还要确认 `positive_label`
- 如果是聚类，还要确认 `cluster_count`

### 共用确认项

- 缺失值怎么处理
- 是否需要把某些列按数值或日期理解
- 当前列是否真的表达了用户要分析的业务含义

### V1 默认策略

- `intercept` 默认 `true`
- 用户未明确时，不主动暴露过多建模参数
- 只有 Tool 明确提示类型问题，或用户明确说明列含义时，再追问 `casts`

## 模型路由

### 线性回归

适用场景：预测连续数值，例如金额、收入、成本、数量、时长。

执行顺序：

1. 确认 `target`
2. 确认 `features`
3. 确认缺失值处理方式
4. 必要时先做 `analyze_table` 或 `stat_summary`
5. 调用 `linear_regression`

### 逻辑回归

适用场景：判断是否发生某件事，例如是否成交、是否流失、是否逾期。

执行顺序：

1. 确认 `target`
2. 确认 `positive_label`
3. 确认 `features`
4. 确认缺失值处理方式
5. 必要时先做 `stat_summary`
6. 如果 Tool 提示只有一个类别，先回到目标列确认或分布检查
7. 调用 `logistic_regression`

### 聚类

适用场景：把样本自动分组，例如客户分群、门店分层、产品分组。

执行顺序：

1. 确认 `features`
2. 确认 `cluster_count`
3. 确认缺失值处理方式
4. 必要时先做 `stat_summary`
5. 调用 `cluster_kmeans`

## 结果解释规则

- `analyze_table`：优先解释 `table_health`、`structured_findings`、`human_summary`、`next_actions`
- `stat_summary`：优先解释 `table_overview`、`numeric_summaries`、`categorical_summaries`、`boolean_summaries`
- `linear_regression`：优先解释 `target`、`features`、`coefficients`、`r2`、`row_count_used`
- `logistic_regression`：优先解释 `positive_label`、`class_balance`、`training_accuracy`、`coefficients`
- `cluster_kmeans`：优先解释 `cluster_count`、`cluster_sizes`、`cluster_centers`

## 输出格式建议

每次回复尽量保持三段：

1. 当前理解
2. 下一步问题或下一步动作
3. 若确认后，将调用哪个 Tool

如果已经拿到结果，再补三段：

4. 结果概览
5. 主要提醒
6. 建议下一步

## V1 限制

- 不做模型自动优化
- 不做自动调参
- 不做阈值调优
- 不展开 AUC 与混淆矩阵全量解释
- 不做多分类 softmax
- 不自动决定目标列、特征列、正类标签、聚类分组数
- 不把训练集结果包装成最终业务决策

## 常见错误

### 错误 1：用户一说建模就直接跑模型

正确做法：先补齐关键参数，必要时先做最小前置诊断。

### 错误 2：把观察诊断结果当成最终结论

正确做法：只解释 Tool 返回，只给下一步建议，不替用户做最终业务拍板。

### 错误 3：逻辑回归不确认正类

正确做法：先问清楚用户最关心哪个结果，再调用 `logistic_regression`。

### 错误 4：上游已经拿到 `table_ref`，却又退回 `path + sheet`

正确做法：优先复用 `table_ref`，不要重复追问文件路径、Sheet 名和表头。

## Quick Reference

- 默认起手：`analyze_table`
- 看统计摘要：`stat_summary`
- 看基础列画像：`summarize_table`
- 连续值预测：`linear_regression`
- 二分类判断：`logistic_regression`
- 自动分群：`cluster_kmeans`

## 最终原则

- 默认先诊断，再决定是否建模
- 用户明确要建模时，可以直达，但不能跳过最小前置校验
- 先用 Tool 给依据，再由 Skill 说人话
- 所有统计与模型计算都留在 Rust Tool 层

<!-- 2026-03-27 22:20:00 +08:00 ?????????? 2026-03-26 ?????????????????????????? -->
## 2026-03-26 P1 operational analytics chain

This layer now supports a practical chain for business operations:
1) short-term warning forecast
2) contribution attribution
3) what-if scenario simulation

### Tool routing map

- `short_term_forecast_alert`
  - use when user asks "future few periods", "warning", "early alert", or "?????"
  - required args: `time_column`, `value_column`
  - optional args: `horizon` (default 4), `sensitivity` (default 2.5)

- `contribution_attribution`
  - use when user asks "??????", "????", "?? vs ??????"
  - required args: `period_column`, `dimension_column`, `value_column`, `baseline_period`, `current_period`
  - optional args: `top_k` (default 5)

- `scenario_simulation`
  - use when user asks "??...???", "A/B action impact", "what-if"
  - required args: `target_column`, `scenarios`
  - optional args: `driver_columns` (if missing, infer from scenario driver keys)

### Chain rule (default)

If user asks for a full diagnostic path in one thread:
1. run `short_term_forecast_alert`;
2. if warning/watch level reaches medium or high, run `contribution_attribution`;
3. run `scenario_simulation` with 2-3 action options.

If user only asks one of the three, run that one directly.
