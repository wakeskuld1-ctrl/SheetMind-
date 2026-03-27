---
name: analysis-modeling-v1
description: Use when users ask to assess whether an Excel sheet is ready for analysis or modeling, request statistical diagnostics or summaries, or explicitly ask for regression, logistic classification, or clustering through this project's Rust tool layer, especially when the Skill must decide whether to diagnose first or go straight into model preparation without performing any computation itself.
---

<!-- 2026-03-27 21:30:00 +08:00 修改原因与目的：原文件正文出现乱码且缺少 binary-only runtime 约束短语，重写为 UTF-8 文档并保留职责边界。 -->
# 分析建模 Skill V1

## Runtime Constraint

This Skill targets the delivered Rust 二进制 runtime, 不依赖 Python, and 不要求用户安装 Python.

## Overview

这个 Skill 负责在“是否适合分析/建模”和“应该调用哪类分析 Tool”之间做路由与解释。

它只负责：

1. 判断当前表是否已经具备进入统计诊断或建模的最小条件。
2. 在合适时优先引导用户先做诊断，再进入回归、分类或聚类。
3. 把 Tool 返回的结果翻译成业务可理解的下一步建议。

它不负责：

1. 在 Skill 内做任何统计计算。
2. 跳过 Tool 结果直接给出模型结论。
3. 让用户安装额外脚本环境或 notebook 依赖。

## Use When

适合这些请求：

- “这张表能不能直接分析？”
- “先帮我看一下统计摘要/异常/分布/趋势。”
- “我想做回归、逻辑分类或聚类。”
- “这些字段适不适合建模？”

不适合这些请求：

- 纯 Excel 读取、清洗、拼接、补列、表头确认。
- 已经明确是决策建议而不是分析建模。
- 要求 Skill 自己做计算而不走 Rust Tool。

## Guardrails

默认先诊断，后建模。

- 如果用户没有明确指定模型，优先建议 `stat_summary`、`correlation_analysis`、`distribution_analysis`、`outlier_detection`、`trend_analysis`。
- 如果用户明确指定模型，也要先确认输入是否具备最小可用条件。
- 如果 Tool 没有返回明确结论，不要在 Skill 里补猜。

## Routing Hints

- 需要整体质量判断时，先走 `analyze_table` 或 `stat_summary`。
- 需要数值关系判断时，优先 `correlation_analysis`。
- 需要分布/异常判断时，优先 `distribution_analysis` 与 `outlier_detection`。
- 需要时间走势判断时，优先 `trend_analysis`。
- 进入模型阶段时，再路由到 `linear_regression`、`logistic_regression`、`cluster_kmeans`。
