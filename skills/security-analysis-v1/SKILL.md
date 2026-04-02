---
name: security-analysis-v1
description: 当用户要求股票、ETF、指数、行业或综合证券分析时使用。默认先走项目内的 `security_decision_briefing` 统一入口，按当前交易日期锚定分析，并用中文输出可执行结论。
---

# 证券分析 Skill V1

## Overview

这个 Skill 负责把证券分析请求统一路由到项目内已经实现的 Rust Tool 主链，并把结构化结果翻译成可执行的中文结论。

从 2026-04-02 起，默认统一入口是：
- `security_decision_briefing`

适用原则：
- 普通咨询场景：先拿 `security_decision_briefing`，再做解释。
- 投决会 / 委员会场景：先拿 `security_decision_briefing`，只基于其中的 `committee_payload` 调用 `security_committee_vote`。
- `technical_consultation_basic`、`security_analysis_contextual`、`security_analysis_fullstack`、`security_analysis_resonance` 仍然存在，但默认只作为 briefing 的底层事实源，而不是面向最终用户的优先入口。

## 核心门禁

1. 默认先走 `security_decision_briefing`，不要绕过统一入口手工拼技术面、财报面、公告面或共振面。
2. 只要任务进入“投决、委员会、表决、是否通过、是否批准、委员意见”这类语义，必须升级为：
   `security_decision_briefing -> security_committee_vote`
3. 禁止把 `technical_consultation_basic`、`security_analysis_contextual`、`security_analysis_fullstack`、`security_analysis_resonance` 的原始输出直接当成最终投决依据。
4. 禁止手工再拼一份与 `committee_payload` 不一致的第二套事实包。
5. 必须区分“Tool 已提供的事实”和“基于事实做出的判断”，不能把推断伪装成事实。
6. 分析日期必须显式写明；如果当前日期没有有效收盘数据，只能回退到最近一个有效交易日，并明确说明。
7. 如公开信息源临时不可用，可以降级，但必须说明降级范围，不能虚构缺失数据。

## 路由优先级

- 默认综合证券分析：`security_decision_briefing`
- 纯技术面调试 / 指标排查：`technical_consultation_basic`
- 环境层调试：`security_analysis_contextual`
- 信息面调试：`security_analysis_fullstack`
- 共振层调试：`security_analysis_resonance`
- 正式投决会表决：`security_committee_vote`

## 标准流程

### 普通咨询

1. 调用 `security_decision_briefing`
2. 明确写出 `analysis_date`
3. 先引用 `summary`
4. 再按 `technical_brief / fundamental_brief / resonance_brief` 展开证据
5. 最后引用 `execution_plan` 给出执行建议与风险边界

### 投决会 / 委员会

1. 调用 `security_decision_briefing`
2. 读取 `committee_payload`
3. 调用 `security_committee_vote`
4. 只基于 vote 结果输出最终表决结论、条件、分歧和 warnings
5. 如需解释票面原因，只能回溯到同一份 briefing / payload，不能手工补第二套事实

## 输出要求

每次尽量按以下顺序输出：

1. 实际分析日期
2. 直接结论
3. 关键证据
4. 风险与情景路径

如果走的是 `security_decision_briefing`：
- 优先引用 `summary`
- 技术证据来自 `technical_brief`
- 基本面证据来自 `fundamental_brief`
- 共振证据来自 `resonance_brief`
- 执行建议来自 `execution_plan`

如果走的是 `security_committee_vote`：
- 优先引用 `final_decision / final_action / final_confidence`
- 风险边界来自 `conditions / warnings`
- 分歧来自 `key_disagreements`
- 票面结构来自 `votes`

## 禁止事项

- 不要把 `security_analysis_fullstack` 或 `security_analysis_resonance` 的原始输出直接当成最终用户默认答复。
- 不要手工再拼一套与 `committee_payload` 不一致的投决事实底稿。
- 不要混用不同分析日期的数据。
- 不要在数据缺失时自行脑补价格、财报或公告内容。
