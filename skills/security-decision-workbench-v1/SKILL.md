---
name: security-decision-workbench-v1
description: Use when users want a securities recommendation to go through this project's decision-workbench flow instead of stopping at one-sided research output, especially when the answer must freeze a single evidence bundle, separate bull and bear stances, and end with risk-gated Chinese decision guidance.
---

# 证券投决会 Skill V1

## Overview

这个 Skill 负责把“证券分析”升级成“证券投决会”。

它不是直接给单边推荐，而是先冻结项目内 Tool 产出的统一证据包，再分别形成多头论证和空头挑战，最后结合风控闸门输出投决结论。

## When to Use

- 用户要“买不买、怎么配仓、值不值得进场”，而不是只要研究摘要
- 用户明确要求双立场、投决会、反方挑战、风险闸门
- 用户要求同一对话里完成“研究 -> 博弈 -> 裁决”

不要在这些场景使用：

- 只看单标的技术面
- 只做大盘/板块解读
- 只做一般聊天式股评

## Core Rules

1. 先调 `security_decision_evidence_bundle`，冻结同源证据
2. 多头和空头都只允许读取同一份 `evidence_bundle`
3. 多头初判时，不读取空头初稿
4. 空头初判时，不读取多头初稿
5. 初判完成后，才允许进入统一裁决
6. 最终结论必须引用 `security_decision_committee` 的 `risk_gates` 和 `decision_card`

## Workflow

### 1. 冻结证据

先调用：

- `security_decision_evidence_bundle`

至少确认这些字段存在：

- `analysis_date`
- `evidence_hash`
- `technical_context`
- `integrated_conclusion`
- `data_gaps`
- `risk_notes`

### 2. 独立生成多头观点

只基于 `evidence_bundle` 回答：

- 为什么值得做
- 哪些证据支持
- 主要失效条件是什么

不要读取空头草稿。

### 3. 独立生成空头观点

只基于 `evidence_bundle` 回答：

- 为什么现在不该做
- 哪些证据不足
- 哪些风险被低估

不要读取多头草稿。

### 4. 进入统一裁决

调用：

- `security_decision_committee`

输出时必须区分：

- Tool 已证实的研究事实
- 多头论证
- 空头挑战
- 风险闸门结果
- 最终投决状态

## Output Format

尽量保持五段：

1. 分析日期与证据版本
2. 多头观点
3. 空头观点
4. 风险闸门
5. 最终投决结论

## Common Mistakes

### 错误 1：先有结论，再补证据

正确做法：先冻结 `evidence_bundle`，再讨论观点。

### 错误 2：多头和空头互相引用对方初稿

正确做法：两边先独立初判，最后才合流。

### 错误 3：跳过 `decision_card`

正确做法：最终结论必须明确引用 `decision_card.status`、`risk_gates` 与 `final_recommendation`。
