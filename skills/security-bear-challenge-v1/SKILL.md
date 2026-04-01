---
name: security-bear-challenge-v1
description: Use when the securities decision workflow needs a standalone bearish challenge based only on a frozen evidence bundle, especially when the system must pressure-test a bullish idea before final committee output.
---

# 空头挑战 Skill V1

## Overview

这个 Skill 只做一件事：基于冻结后的 `evidence_bundle`，形成独立的反方挑战。

它不负责拍板，只负责找缺口、提风险、挑逻辑漏洞。

## Rules

1. 只能读取 `evidence_bundle`
2. 不能读取多头初稿
3. 不要替最终结论做决定
4. 优先指出证据缺口、事件风险、环境逆风和风报比错配

## Focus

- 哪些证据还不够
- 哪些风险没有被充分计价
- 哪些条件下不该给执行建议
- 多头论证最容易失效的点在哪里

## Required Output

- `headline`
- `thesis_points`
- `invalidation_conditions`
- `confidence`

## Common Mistake

不要把“挑战”写成“情绪化看空”，必须引用证据包里的真实字段。
