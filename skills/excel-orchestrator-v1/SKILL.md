---
name: excel-orchestrator-v1
description: Use when users interact with this Excel product from a single conversational entry point and need the system to decide whether to route into table processing, analysis-modeling, or decision-assistant flow while maintaining a lightweight session state summary without doing any computation itself.
---

<!-- 2026-03-27 21:30:00 +08:00 修改原因与目的：原文件正文存在乱码风险，重写为 UTF-8 文档并显式声明统一运行时边界。 -->
# Excel Orchestrator Skill V1

## Runtime Constraint

This Skill targets the delivered Rust 二进制 runtime, 不依赖 Python, and 不要求用户安装 Python.

## Overview

这是总入口 Skill，用来判断用户当前处在哪一层流程：

1. 表处理层
2. 分析建模层
3. 决策建议层

它的职责是：

- 识别当前意图。
- 维持轻量会话上下文。
- 把用户路由到正确子 Skill。

它不做任何实际计算，也不直接替代底层 Tool。

## Use When

适合这些情况：

- 用户从统一入口进入，还没明确说要用哪个 Skill。
- 用户只说“先看看这个 Excel”。
- 用户问“下一步该做什么”，但还没明确是在清洗、分析还是决策。

不适合这些情况：

- 已经明确在单一子流程里连续执行具体步骤。
- 用户直接点名要某个 Skill 或某个 Tool。

## Routing Rules

- 涉及读取、清洗、拼表、补列、确认结构时，路由到 `table-processing-v1`。
- 涉及统计诊断、回归、分类、聚类时，路由到 `analysis-modeling-v1`。
- 涉及“告诉我下一步做什么”“怎么决策更稳妥”时，路由到 `decision-assistant-v1`。

## Guardrails

- 不要在这里做业务结论。
- 不要在这里做统计计算。
- 不要要求用户切换到 Python、notebook 或外部脚本环境。
