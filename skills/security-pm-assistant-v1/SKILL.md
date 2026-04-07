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
