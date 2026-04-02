---
name: security-committee-v1
description: 当用户要求投决会、委员会讨论、正式表决、批准/否决/附条件通过结论时使用。强制先调用 `security_decision_briefing`，再只基于 `committee_payload` 调用 `security_committee_vote`。
---

# Security Committee Skill V1

## Overview

这个 Skill 负责把“委员会讨论”与“正式表决”收口到统一链路：

`security_decision_briefing -> committee_payload -> security_committee_vote`

目标：
- 让委员会只能基于统一事实包表决
- 让咨询口径、投决口径和历史研究口径保持一致
- 禁止上层 Agent 手工拼接第二套投决事实

## 强制门禁

1. 先 briefing，后 vote  
任何委员会、投决会、是否通过、是否批准、委员表态、附条件通过、否决等场景，必须先调用 `security_decision_briefing`。

2. vote 只吃 `committee_payload`  
`security_committee_vote` 的唯一事实输入是 `committee_payload`。禁止额外混入自定义字段、临时推断字段、手工拼接摘要。

3. 禁止手写票面  
不要根据 `committee_payload` 自己写“主席同意、风控反对、执行附条件通过”之类结果。正式票面必须来自 `security_committee_vote`。

4. 禁止第二套事实  
如果你对 briefing 有疑问，应该指出 briefing 与判断的冲突，而不是重组另一份“更合理”的事实包。

5. 历史研究层必须沿主链消费  
如果 briefing 中 `historical_digest.status = available`，说明历史研究层已经进入统一事实包；后续投决解释必须沿这份 digest 消费，而不是脱离主链再重查一次。

## 标准流程

1. 调用 `security_decision_briefing`
2. 检查 `committee_payload`
3. 调用 `security_committee_vote`
4. 输出：
- `final_decision`
- `final_action`
- `final_confidence`
- `conditions`
- `warnings`
- `key_disagreements`
- 必要时展示 `votes`

## 输出规则

- 必须写明 `analysis_date`
- 必须写明 `committee_mode`
- 必须写明结论来自 `security_committee_vote`
- 必须把 `conditions / warnings` 当成正式约束，而不是附带说明
- 如有分歧，优先引用 `key_disagreements`

## 禁止事项

- 不要跳过 `security_decision_briefing`
- 不要直接用 `security_analysis_fullstack` 或 `security_analysis_resonance` 产出委员会结论
- 不要手工制造委员角色、票面、veto、approval ratio
- 不要混用不同 `evidence_version` 的事实
