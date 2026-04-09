---
name: security-decision-briefing-v1
description: 当用户要做证券咨询、交易决策准备或委员会投决准备时使用。强制先调用 `security_decision_briefing`，再基于同一份 briefing 输出解释、建议或 committee 表决。
---

# Security Decision Briefing Skill V1

## Overview

这个 Skill 是证券咨询与投决流程的基础门禁。

它要求所有上层 Agent 在输出证券观点之前，先调用项目内统一 Tool：
- `security_decision_briefing`

该 Tool 会把技术面、基本面、共振面、执行层和委员会载荷装配成一份单一 briefing。
本 Skill 的目标不是替代分析，而是先统一事实底稿，再允许上层做判断。

## 必须遵守的门禁

1. 普通证券咨询场景：
先调用 `security_decision_briefing`，再根据 `summary / technical_brief / fundamental_brief / resonance_brief / execution_plan` 解释结论。

2. 投决或委员会场景：
先调用 `security_decision_briefing`，再只基于 `committee_payload` 调用 `security_committee_vote`。

3. 不允许绕过 briefing：
不要直接把 `technical_consultation_basic`、`security_analysis_contextual`、`security_analysis_fullstack`、`security_analysis_resonance` 的结果手工拼成另一份最终底稿。

4. 不允许混淆层次：
- Tool 输出的是事实层
- Agent 输出的是解释层、建议层、投决层

5. 不允许制造第二份事实：
如果 briefing 和你想表达的结论不一致，应明确说明冲突点，而不是额外再拼一套“更符合直觉”的事实。

6. 不允许跳过正式 vote：
只要输出里出现“投决会结论、委员会是否通过、委员投票、批准/否决/附条件通过”等结果，就必须调用 `security_committee_vote`，不能只凭 `committee_payload` 手写票面。

## 标准流程

### 咨询模式

1. 调用 `security_decision_briefing`
2. 明确写出 `analysis_date`
3. 先给 `summary`
4. 再按 `technical_brief / fundamental_brief / resonance_brief` 展开证据
5. 最后引用 `execution_plan` 给出执行建议和风险控制

### 投决模式

1. 调用 `security_decision_briefing`
2. 读取 `committee_payload`
3. 调用 `security_committee_vote`
4. 用 `final_decision / final_action / final_confidence / votes / conditions / warnings / key_disagreements` 组织正式投决输出

## 适用场景

- “帮我看这只股票现在该不该加仓”
- “把这只股票做成一页投决简报”
- “准备委员会讨论这只股票的事实底稿”
- “基于同一份 briefing 给出正式委员会表决结果”

## 不适用场景

- 只做底层指标调试
- 只查某个 Tool 的内部字段
- 与证券分析无关的普通表格处理

## 输出提醒

- 必须标明分析日期
- 必须区分事实与判断
- 必须优先复用 briefing 输出
- 必须保持咨询场景与投决场景看到的是同一份 `evidence_version`
- 一旦进入投决场景，必须显式说明 vote 是基于同一份 `committee_payload`
