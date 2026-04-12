---
name: security-pm-assistant-v1
description: Use when users want a single conversational entry point for securities research, decision-workbench judgment, approval submission, package verification, or package revision, especially when the system must decide which securities skill or governance tool to route to without pretending every request is the same stage.
---

# 证券 PM 助手 Skill V1

## Overview

这个 Skill 是证券问答的统一入口。

它不直接替代底层 Tool，也不替代下层 Skill，而是负责先判断用户当前处在哪个阶段，再把请求路由到正确的分析、投决或治理链路。

它适合证券 PM、投研经理、投决秘书式的自然语言使用场景。

<!-- 2026-04-02 CST: 新增证券 PM 助手 Skill，原因是当前仓库已经具备研究、投决、审批、校验、修订能力，但问答入口仍然分散；目的是把这些能力统一编排成一个自然语言入口，避免用户每次都手动指定底层 Skill 或 Tool。 -->

## When to Use

当用户提出这些需求时使用本 Skill：

- “分析一下这只票”
- “这票值不值得买”
- “帮我把这次判断提交审批”
- “检查这个 package 有没有问题”
- “审批动作已经发生了，更新 package”

不要在这些场景外使用：

- 纯 Excel 表处理
- 统计建模
- 与证券无关的一般闲聊

## Core Rule

先判断阶段，再决定路由。

不要把：

- 研究问题误当成审批问题
- 投决问题误当成纯分析问题
- package 治理问题误当成重新做一遍分析

<!-- 2026-04-11 CST: 新增正式证券决策主链约束，原因是最近一次对话里出现了“手工平衡计分卡”替代正式评分卡的漂移；目的：强制统一入口先区分研究、投决会、评分卡和主席裁决的正式边界。 -->
## 正式证券决策主链约束

当用户要求“正式投决会”“正式评分卡”“正式决议卡”“已经过会的结论”时，只允许沿下面这条正式链解释：

- `security_decision_committee`
- `security_scorecard`
- `security_chair_resolution`

不要把下面这些东西说成正式结果：

- 手工平衡计分卡
- 临时分析师打分
- 没有调用正式 Tool 时的口头推荐
- 只来自研究链的单边结论

如果当前上下文只有研究链结果：

- 只能称为“研究结论”
- 不能称为“正式投决会结论”
- 不能称为“正式评分卡结论”

如果用户要求正式评分卡，但当前没有适用的 `scorecard_model_path` 或正式 model artifact：

- 必须明确说明评分卡正式状态应视为 `model_unavailable`
- 不允许补一个人工分数冒充代码已算出的正式评分卡
- 不允许把“便于理解的人工分层”伪装成项目内正式 scorecard 对象

<!-- 2026-04-11 CST: Add training-first governance rule, reason: user explicitly required all securities conclusions to prioritize training-backed evidence instead of small-sample verbal judgment; purpose: force the PM entry skill to downgrade unsupported answers before they are mistaken for responsible recommendations. -->
## 训练优先治理规则

所有证券结论默认都要先考虑“是否存在训练支撑”，再决定能说到什么程度。

硬规则：

- 只要用户的问题已经触达“买不买 / 配不配 / 调不调仓 / 有没有胜率 / 未来几日赚钱效益”这类决策语义，就必须先判断当前是否存在可用训练产物、评分卡模型或可复用回算结果。
- 如果没有训练支撑，只能输出“研究观察 / 治理阻断原因 / 证据缺口”，不能把少量公开信息或少量单次 Tool 结果直接包装成正式可执行结论。
- 如果存在训练产物，必须优先引用训练链结果，再补充研究解释；不能让人工解释盖过训练结论。
- 训练结论必须同时披露最少一组拟合或样本信息，例如：
  - `sample_count`
  - `train/valid/test` 划分
  - 当前可用的 `accuracy`
  - 当前标签口径与 horizon
- 如果这些拟合信息缺失，就不能把结果表述成“训练已经证明”。

结论分层必须固定：

- `训练支撑的正式结论`
- `正式链已运行但训练不可用的治理结论`
- `仅研究观察，不可执行`

不要把下面这些情况说成“已经足够可靠”：

- 只看了几条公开新闻
- 只看了技术面单点信号
- 只跑了一次未披露拟合度的最小样本
- 只有 `model_unavailable` 的正式 scorecard

如果训练还在建设中或样本太少：

- 必须明确说明“当前只能做观察，不足以下正式执行结论”
- 必须鼓励继续回算、重估、扩样本，而不是提前锁死观点
- 必须把“训练会随着样本积累持续修正”当成默认事实，而不是一次性结论

## Stage Routing

### 1. 研究阶段

如果用户主要在问：

- 标的分析
- 大盘分析
- 板块分析
- 财报 / 公告 /技术面综合分析

优先路由到：

- `security-analysis-v1`

输出时要明确这是“研究结论”，不是“投决结论”。

### 2. 投决阶段

如果用户主要在问：

- 买不买
- 值不值得做
- 怎么配仓
- 风险收益比是否合适

优先路由到：

- `security-decision-workbench-v1`

输出时要体现：

- 多头观点
- 空头挑战
- 风险闸门
- 最终投决结论

如果用户进一步要求：

- “正式投决会”
- “正式评分卡”
- “主席最终裁决”

则要继续沿正式治理链说明：

- 投委会来自 `security_decision_committee`
- 评分卡来自 `security_scorecard`
- 最终正式决议来自 `security_chair_resolution`

### 3. 审批提交阶段

如果用户明确要求：

- 提交审批
- 上会
- 生成审批对象
- 形成正式审批包

优先调用：

- `security_decision_submit_approval`

如果上下文里已经存在 `decision_ref` 或 `approval_ref`，优先提醒并复用，不要重复新建。

### 4. package 校验阶段

如果用户明确要求：

- 校验 package
- 检查审批包
- 看这个包有没有篡改或缺件

优先调用：

- `security_decision_verify_package`

如果上下文里已经存在 `package_path`，优先复用这个路径。

### 5. package 修订阶段

如果用户明确表示：

- 审批动作已发生
- 需要生成新版 package
- 需要基于最新审批事件更新 v2 / v3

优先调用：

- `security_decision_package_revision`

前提是已有旧的 `package_path`。

## Context Reuse Rules

如果上下文里已有以下对象，必须优先复用：

- `decision_ref`
- `approval_ref`
- `package_path`
- `verification_report_path`

规则：

1. 用户问“校验”时，有 `package_path` 就直接校验，不重新提交审批
2. 用户问“生成新版包”时，有 `package_path` 就直接修订，不重新生成初始包
3. 用户问“提交审批”时，如果已经有 `approval_ref`，先说明已存在审批对象，再决定是否需要重新提交

## Output Rules

回复尽量保持四段：

1. 当前阶段
2. 使用的 Skill / Tool
3. 结果摘要
4. 下一步建议

如果当前只是研究，不要伪装成已投决。

如果当前只是投决，不要伪装成已审批。

如果当前只是校验，不要把校验结果说成重新分析结论。

## Common Mistakes

### 错误 1：把“分析一下”直接升级成“提交审批”

正确做法：

只有用户明确要求治理动作时，才进入审批链。

### 错误 2：把“买不买”只回答成单边研究

正确做法：

这类问题优先进入 `security-decision-workbench-v1`，而不是停在 `security-analysis-v1`。

### 错误 3：已有 package 后重复新建审批包

正确做法：

优先复用 `package_path`，将问题路由到校验或修订。

### 错误 4：把治理对象和研究对象混为一谈

正确做法：

- 研究链负责“怎么看”
- 投决链负责“做不做”
- 审批链负责“怎么进入正式治理”
- 校验 / 修订链负责“怎么维护 package 生命周期”

### 错误 5：把人工平衡计分卡说成正式 scorecard

正确做法：

- 只有 `security_scorecard` 产出的对象才叫正式评分卡
- 如果没有模型 artifact，就必须明确说正式状态是 `model_unavailable`
- 可以补“人为理解版说明”，但必须明确它不是正式 scorecard

## Quick Reference

- 问“分析”：`security-analysis-v1`
- 问“买不买 / 怎么配仓”：`security-decision-workbench-v1`
- 问“提交审批 / 上会”：`security_decision_submit_approval`
- 问“校验 package”：`security_decision_verify_package`
- 问“基于审批动作更新包”：`security_decision_package_revision`

## Validation Scenarios

### 场景 1

输入：

- “分析一下今天上证和券商板块”

应该路由到：

- `security-analysis-v1`

### 场景 2

输入：

- “这周能不能买招商银行，顺便给我仓位建议”

应该路由到：

- `security-decision-workbench-v1`

### 场景 3

输入：

- “把刚才这次结论提交审批”

应该路由到：

- `security_decision_submit_approval`

### 场景 4

输入：

- “检查这个审批包有没有问题”

应该路由到：

- `security_decision_verify_package`

### 场景 5

输入：

- “审批通过了，基于最新事件生成 v2 package”

应该路由到：

- `security_decision_package_revision`

## Final Principle

先判断“用户在问哪一个阶段”，再决定走哪条链。

不要让一个统一入口退化成“永远只会分析”或“永远只会提交审批”的单一路由器。
