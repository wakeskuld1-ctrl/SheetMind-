---
name: security-bull-thesis-v1
description: Use when the securities decision workflow needs a standalone bullish thesis based only on a frozen evidence bundle, especially when the system must argue for opportunity without reading the bearish draft first.
---

# 多头论证 Skill V1

## Overview

这个 Skill 只做一件事：基于冻结后的 `evidence_bundle`，形成独立的多头论证。

它不负责最终裁决，也不负责替空头观点辩护。

## Rules

1. 只能读取 `evidence_bundle`
2. 不能读取空头初稿
3. 不能直接输出最终买卖建议
4. 必须写出失效条件

## Focus

- 技术面支持点
- 环境共振支持点
- 基本面/公告加分项
- 为什么当前不是纯观察状态

## Required Output

- `headline`
- `thesis_points`
- `invalidation_conditions`
- `confidence`

## Common Mistake

不要把“有机会”写成“已经必然会涨”。
