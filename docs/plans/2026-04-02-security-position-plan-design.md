# Security Position Plan Design

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在现有证券投决会与审批桥接链路之上新增可审批的 `position_plan`，让系统不仅能回答“能不能做”，还能够以结构化对象回答“如果做，准备怎么做”。

**Architecture:** 保持 `committee_result` 负责研究与投决判断，保持 `approval_request` 负责治理状态，本次新增独立的 `position_plan` 子对象并挂到现有 `decision_ref / approval_ref` 上。`position_plan` 与 `DecisionCard` 分层，不把执行方案继续塞回投决卡主体，以便后续投中管理沿同一对象演进。

**Tech Stack:** Rust、Cargo、serde/serde_json、现有证券投决会链、文件型 runtime 持久化、CLI 合同测试

---

## 1. 背景

当前主仓已经完成：

- 证券投决会 `security_decision_committee`
- 审批桥接 `security_decision_submit_approval`
- 审批摘要 `security_decision_approval_brief`
- 兼容私有审批主线的：
  - `DecisionCard`
  - `ApprovalRequest`
  - `audit_log`

这意味着系统已经能把“研究结论”变成“正式上会对象”。  
但当前审批对象还缺一层关键执行语义：

- 能不能做：有了
- 为什么做：有了
- 风险闸门：有了
- **怎么做：还没有正式对象**

因此，当前审批仍然更像“研究审批”，而不是“投决 + 执行方案审批”。

## 2. 设计目标

本次 `P0-2` 只解决以下问题：

1. 新增结构化 `position_plan`
2. 让 `position_plan` 正式挂到现有 `decision_ref / approval_ref`
3. 把仓位计划一起纳入审批输出与落盘工件
4. 在 approval brief 中补充仓位执行摘要

本次明确不做：

1. 不做组合级优化器
2. 不做自动调仓
3. 不做投中动态持仓管理
4. 不做多标的联合仓位约束
5. 不做真实下单

## 3. 为什么要把仓位计划做成独立对象

这次有三种可能：

1. 把仓位计划继续塞进 `DecisionCard`
2. 做成独立 `position_plan`，挂到 `decision_ref / approval_ref`
3. 做成独立审批附件

本次选择第 2 种，原因是：

- 比继续塞进 `DecisionCard` 更清楚，避免把投决卡做成“万能袋子”
- 比审批附件模式更轻，当前阶段最容易落地
- 最适合后续投中管理继续沿同一对象扩展

一句话：

**决策对象负责“该不该做”，仓位计划对象负责“准备怎么做”。**

## 4. 总体结构

本次新增：

- `security_position_plan`

并扩展：

- `security_decision_submit_approval`
- `security_decision_approval_brief`

处理后的对象关系为：

- `committee_result`
- `approval_request`
- `position_plan`

其中：

- `committee_result` 是研究与裁决基础
- `approval_request` 是审批治理入口
- `position_plan` 是执行方案对象

## 5. `position_plan` 合同

建议字段：

- `plan_id`
- `decision_ref`
- `approval_ref`
- `symbol`
- `analysis_date`
- `plan_status`
- `risk_budget_pct`
- `suggested_gross_pct`
- `starter_gross_pct`
- `max_gross_pct`
- `entry_plan`
- `add_plan`
- `stop_loss_plan`
- `take_profit_plan`
- `cancel_conditions`
- `sizing_rationale`

### 5.1 `plan_status`

建议取值：

- `blocked`
- `probe_only`
- `reviewable`

语义：

- `blocked`：不允许执行
- `probe_only`：仅允许试探仓
- `reviewable`：可随审批对象一起进入正式执行审阅

### 5.2 `entry_plan`

建议为结构化对象，至少包含：

- `entry_mode`
- `trigger_condition`
- `starter_gross_pct`
- `notes`

### 5.3 `add_plan`

建议包含：

- `allow_add`
- `trigger_condition`
- `max_gross_pct`
- `notes`

### 5.4 `stop_loss_plan`

建议包含：

- `stop_loss_pct`
- `hard_stop_condition`
- `notes`

### 5.5 `take_profit_plan`

建议包含：

- `first_target_pct`
- `second_target_pct`
- `partial_exit_rule`
- `notes`

## 6. 规则型生成逻辑

本次只做规则型 v1，不做优化器。

### 6.1 `blocked`

- `plan_status = blocked`
- `suggested_gross_pct = 0`
- `starter_gross_pct = 0`
- `max_gross_pct = 0`
- `allow_add = false`
- 必须带取消执行条件

### 6.2 `needs_more_evidence`

- `plan_status = probe_only`
- 只给试探仓
- `suggested_gross_pct` 建议区间 `3%-5%`
- `starter_gross_pct` 小于等于建议总仓
- `allow_add = false`
- 加仓必须要求“补证据并重新审批”

### 6.3 `ready_for_review`

- `plan_status = reviewable`
- 先给启动仓
- 默认首仓 `5%-8%`
- 默认总仓 `8%-12%`
- 默认最大仓位 `12%-15%`

再按以下因素微调：

- `confidence_score`
- 风报比
- `risk_gates` 中 `warn` 数量
- `position_size_suggestion`

### 6.4 `sizing_rationale`

必须显式说明：

- 是因为状态允许
- 还是因为风报比够
- 还是因为置信度较高
- 是否因闸门提醒被降档

## 7. approval brief 扩展

当前审批摘要只覆盖：

- 多头
- 空头
- 闸门
- 最终建议

本次新增：

- `risk_budget_summary`
- `entry_summary`
- `add_summary`
- `stop_loss_summary`
- `take_profit_summary`
- `cancel_summary`

这样审批时，看到的是完整执行方案，而不是一句“建议 starter”。

## 8. 路径策略

继续沿用当前审批 runtime 根目录。

新增目录：

- `position_plans`

文件命名建议：

- `position_plans/<decision_id>.json`

## 9. 输出扩展

`security_decision_submit_approval` 结果新增：

- `position_plan`
- `position_plan_path`

这样 CLI / Skill / 后续审批层都能直接消费。

## 10. 测试策略

必须坚持 TDD。

优先新增：

1. `ready_for_review` 生成完整 `position_plan`
2. `blocked` 生成零仓位方案
3. `needs_more_evidence` 生成试探仓方案
4. `position_plan` 正确挂到 `decision_ref / approval_ref`
5. `position_plan` 正确落盘
6. `approval_brief` 带仓位摘要

## 11. 风险

### 11.1 规则过硬风险

如果仓位规则写得太死，未来不同策略会不够用。  
因此本次先做单票规则型 v1，并保留后续扩展点。

### 11.2 对象边界风险

如果把执行方案继续塞回 `DecisionCard`，后面投中扩展会再次混乱。  
因此本次坚持独立 `position_plan`。

### 11.3 语义漂移风险

如果 `position_plan` 没有和 `decision_ref / approval_ref` 一起绑定，就会再次退化成普通建议文本。  
因此本次必须显式挂接。

## 12. 完成定义

满足以下条件即可认为 `P0-2` 完成：

1. 新增 `position_plan` 合同与生成逻辑
2. `security_decision_submit_approval` 输出并落盘 `position_plan`
3. `position_plan` 绑定现有 `decision_ref / approval_ref`
4. `approval_brief` 展示仓位摘要
5. CLI 合同测试通过

## 13. 一句话总结

`P0-1` 让证券投决结果正式上会，`P0-2` 让“准备怎么做”也进入正式审批对象。
