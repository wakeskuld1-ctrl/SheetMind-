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

<!-- 2026-04-11 CST: 新增正式决策链硬约束，原因是最近一次对话里出现了未经过正式链就直接给“平衡计分卡”和“投决会结论”的情况；目的：把 workbench 固定为“投委会 -> 评分卡 -> 主席裁决”的正式编排入口。 -->
7. 如果用户要求“正式投决会 / 正式评分卡 / 正式决议卡”，必须显式区分三条正式对象：
   - `security_decision_committee`
   - `security_scorecard`
   - `security_chair_resolution`
8. 不允许用手工平衡计分卡替代 `security_scorecard`
9. 如果没有适用的 `scorecard_model_path` 或正式 model artifact，必须明确说明正式评分卡状态应为 `model_unavailable`
10. 不允许把未走正式 Tool 主链的推荐，说成“已经过投决会”或“评分卡已算出”

<!-- 2026-04-11 CST: Add training-backed decision rule, reason: user corrected that small evidence slices must not be turned into responsible portfolio guidance; purpose: make the decision workbench treat training readiness as a prerequisite for actionable conviction instead of a nice-to-have add-on. -->
11. 任何“买 / 持有 / 减仓 / 调仓 / 组合建议”都必须先检查是否有训练支撑、回算支撑或可披露的拟合摘要
12. 如果没有训练支撑，workbench 只能输出：
   - 治理阻断原因
   - 研究观察
   - 继续补证据 / 继续训练的下一步
   不能直接把少量信息包装成负责的可执行建议
13. 如果引用训练结论，必须同时披露至少一组当前拟合摘要或样本摘要，不能只给方向不给拟合度
14. 当前训练如果只能提供最小摘要，也必须如实披露“这是最小摘要，不是完整生产拟合报告”

## 训练优先规则

当用户的问题包含这些语义时：

- 是否值得买入
- 是否应该调仓
- 胜率高不高
- 未来几日赚钱效益如何
- 这次投决凭什么成立

必须先判断三件事：

1. 有没有正式训练 artifact 或可复用 scorecard model
2. 有没有当前可披露的拟合摘要
3. 当前样本规模是否足以避免“看一点点就下结论”

如果三者任一不成立：

- 只能输出“低置信治理意见”或“研究观察”
- 不得把这类输出包装成高确定性投决
- 不得跳过“训练不足”这个核心风险

如果三者成立：

- 先给训练支撑结论
- 再给委员会讨论和主席解释
- 不允许先写人工结论，再倒推训练去配合

## 训练信息披露要求

只要答案声称“这次结论考虑了训练”或“这次结论通过训练得出”，就必须同时说明当前至少有哪些内容：

- 使用的模型或 artifact 身份
- horizon / label 定义
- `sample_count`
- `train/valid/test` 划分
- 当前可用的拟合摘要

如果当前系统还没有 `AUC / KS / 命中率 / OOS 表现` 这类更完整指标：

- 可以明确说“当前只具备最小拟合摘要”
- 不能把“最小拟合摘要”夸大成完整生产级验证

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

### 5. 评分卡与主席裁决

如果用户要求正式评分卡或正式决议卡：

1. 在 `security_decision_committee` 之后，才允许进入 `security_scorecard`
2. 在 `security_scorecard` 之后，才允许进入 `security_chair_resolution`
3. 输出时必须明确：
   - 哪部分是投委会结果
   - 哪部分是评分卡结果
   - 哪部分是主席最终裁决

如果 `security_scorecard` 没有可用模型：

- 必须明确写出正式状态是 `model_unavailable`
- 不允许补一个人工总分冒充正式 scorecard
- 不允许把“帮助理解的人工归纳”写成代码已算出的正式分组分数

## Output Format

尽量保持五段：

1. 分析日期与证据版本
2. 多头观点
3. 空头观点
4. 风险闸门
5. 最终投决结论

如果进入正式评分卡 / 主席裁决阶段，升级成七段：

1. 分析日期与证据版本
2. 多头观点
3. 空头观点
4. 风险闸门
5. 投委会正式结论
6. 评分卡正式状态与结果
7. 主席最终裁决

说明规则：

- “投委会正式结论” 只能引用 `security_decision_committee`
- “评分卡正式结果” 只能引用 `security_scorecard`
- “主席最终裁决” 只能引用 `security_chair_resolution`
- 如果没有走到某一步，必须明确写“本轮未进入该正式阶段”

## Common Mistakes

### 错误 1：先有结论，再补证据

正确做法：先冻结 `evidence_bundle`，再讨论观点。

### 错误 2：多头和空头互相引用对方初稿

正确做法：两边先独立初判，最后才合流。

### 错误 3：跳过 `decision_card`

正确做法：最终结论必须明确引用 `decision_card.status`、`risk_gates` 与 `final_recommendation`。

### 错误 4：手工做一张平衡计分卡，就说成正式评分卡

正确做法：

- 项目内正式评分卡来自 `security_scorecard`
- 它的分数来自模型 artifact 中的分箱、`woe`、`logit_contribution` 和 `points`
- 如果当前资产类别没有适用模型，就必须明确是 `model_unavailable`

### 错误 5：把“还没调用正式 Tool”的推荐说成“已经过会”

正确做法：

- 没有调用 `security_decision_committee`，就不能说“正式投决会已通过”
- 没有调用 `security_scorecard`，就不能说“正式评分卡已算出”
- 没有调用 `security_chair_resolution`，就不能说“主席已形成最终正式决议”
