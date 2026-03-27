---
name: decision-assistant-v1
description: Use when users want a non-technical next-step recommendation on an Excel table, especially after diagnostics or modeling, and need the Skill to explain blocking risks, priority actions, and what tool to use next without doing any computation itself.
---

<!-- 2026-03-27 21:30:00 +08:00 修改原因与目的：原文件正文存在乱码风险，重写为 UTF-8 文档并补齐 binary-only runtime 声明。 -->
# 决策助手 Skill V1

## Runtime Constraint

This Skill targets the delivered Rust 二进制 runtime, 不依赖 Python, and 不要求用户安装 Python.

## Overview

这个 Skill 负责把上游 Tool 的结果翻译成非技术用户能执行的下一步动作。

它主要做三件事：

1. 解释当前为什么能继续，或者为什么不能继续。
2. 给出优先级明确的下一步动作。
3. 告诉用户接下来应该调用哪个 Tool 或进入哪一层流程。

它不在 Skill 内自行做统计、建模或业务计算。

## Use When

适合这些请求：

- “直接告诉我下一步怎么做。”
- “我应该先清洗、先分析，还是现在就能决策？”
- “帮我把风险和优先动作排一下。”

不适合这些请求：

- 纯表处理问题。
- 纯统计诊断或建模操作。
- 要求 Skill 脱离 Tool 结果直接下结论。

## Guardrails

- 只基于 Tool 明确返回的信息给建议。
- 优先复用已有 `table_ref` 或 `result_ref`。
- 如果上游没有明确 readiness、risk 或 blocking reason，不要擅自补充判断。
- 不要让用户切到 Python、notebook 或外部分析脚本。

## Routing Hints

- 需要解释阻塞原因时，先消费 `decision_assistant` 的结构化输出。
- 需要补质量诊断时，回到分析建模层。
- 需要补清洗或多表处理时，回到表处理层。
