---
name: table-processing-v1
description: Use when users ask to inspect, clean, reshape, append, or explicitly link Excel tables through this project's Rust tool layer, especially when they need non-technical confirmation questions and the Skill must not perform any computation itself.
---

<!-- 2026-03-27 21:30:00 +08:00 修改原因与目的：原文件正文存在乱码风险，重写为 UTF-8 文档并统一 binary-only runtime 约束。 -->
# 表处理 Skill V1

## Runtime Constraint

This Skill targets the delivered Rust 二进制 runtime, 不依赖 Python, and 不要求用户安装 Python.

## Overview

这个 Skill 负责把用户的表处理意图翻译成稳定的 Tool 调用顺序。

它主要覆盖：

- 读取工作簿与表区域
- 表头确认与结构检查
- 列清洗、重命名、类型转换
- 多表关联、追加、计划执行与失败恢复

它不直接做业务判断，也不代替分析建模层解释结果。

## Use When

适合这些请求：

- “先读一下这个 Excel。”
- “帮我确认表头/字段。”
- “把两张表 join/append。”
- “给我一个多表处理计划。”
- “这条多表执行失败了，帮我恢复。”

不适合这些请求：

- 统计诊断、建模、预测解释。
- 决策建议和业务优先级判断。

## Guardrails

- 优先复用 `table_ref` / `result_ref`，避免反复回读原始文件。
- 多表链路优先走显式计划与风险预检。
- 当 join 风险高或前置绑定缺失时，先停下来解释，不要盲目继续。
- 不要要求用户安装 Python、pandas、openpyxl 或 notebook 环境。

## Routing Hints

- 读取入口：`open_workbook`、`list_sheets`、`inspect_sheet_range`、`load_table_region`
- 清洗整理：`normalize_table`、`rename_columns`、`cast_column_types`、`fill_missing_values`
- 多表处理：`suggest_table_links`、`suggest_multi_table_plan`、`join_preflight`、`execute_multi_table_plan`
- 故障恢复：`recover_multi_table_failure`
