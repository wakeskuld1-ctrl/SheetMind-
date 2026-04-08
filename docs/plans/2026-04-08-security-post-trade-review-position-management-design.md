# 证券主链投后复盘与仓位管理设计

<!-- 2026-04-08 CST: 新增证券主链投后复盘与仓位管理设计文档。原因：用户已明确批准按方案 B 推进“单标的 + 多次调仓复盘”，并要求优先聚焦证券主链而不是 foundation。目的：把仓位计划正式化、调仓事件建模和投后复盘闭环一次性收口，避免后续实现时再起平行链路。 -->

## 1. 背景

当前证券主链已经具备以下正式入口：

- `security_decision_briefing`
- `security_committee_vote`
- `signal_outcome_research`

并且主链已经能正式输出：

- `odds_brief`
- `position_plan`
- `committee_payload.odds_digest`
- `committee_payload.position_digest`

这意味着平台已经具备“投前研究 -> 投决建议 -> 仓位建议”的基础，但还缺两段关键闭环：

- 投中执行如何结构化记录
- 投后复盘如何回指原始决策并输出纠偏结论

因此，本轮不新建平行复盘系统，也不脱离现有 `briefing -> committee -> approval` 主链，而是在现有证券主链上补齐正式的仓位记录与投后复盘能力。

## 2. 设计目标

### 2.1 目标

- 让 `position_plan` 从 briefing 子层建议升级为正式可引用对象
- 支持单标的下的多次调仓记录
- 让投后复盘绑定原始 `decision_ref / approval_ref / evidence_version`
- 让复盘输出不仅回答盈亏，还回答“判断是否正确、执行是否到位、错误来自哪里”

### 2.2 非目标

- 不做组合级风险预算
- 不做多标的联动复盘
- 不做账户级净值归因
- 不做自动下单或券商接入
- 不新建平行投研事实链

## 3. 设计原则

### 3.1 单一主链原则

- 仍以 `security_decision_briefing` 为投前事实入口
- 仍以 `security_committee_vote` 为正式投决入口
- 仓位计划与投后复盘都必须回指这条主链，不允许再拼第二套事实

### 3.2 对象化而不是段落化

- `position_plan` 不再只是报告里的一个 JSON 子段
- 调仓记录不再只是聊天描述
- 投后复盘不再只是自然语言总结

所有关键节点都要形成正式对象，便于审批、追踪、审计和后续纠偏。

### 3.3 计划驱动复盘

- 复盘的基准不是结果导向
- 复盘的基准是“原始计划 + 实际执行 + 市场结果”

即使亏损，也可能是正确执行；
即使盈利，也可能是计划错误但运气好。

## 4. 对象模型

### 4.1 `position_plan_record`

作用：

- 把 `security_decision_briefing.position_plan` 正式落盘为可持续追踪的仓位计划对象

绑定字段：

- `position_plan_ref`
- `decision_ref`
- `approval_ref`
- `evidence_version`
- `symbol`
- `analysis_date`

核心内容：

- `position_action`
- `entry_mode`
- `starter_position_pct`
- `max_position_pct`
- `add_on_trigger`
- `reduce_on_trigger`
- `hard_stop_trigger`
- `position_risk_grade`
- `regime_adjustment`
- `execution_notes`
- `rationale`

说明：

- 这是单标的单轮仓位管理的计划基线
- 后续所有调仓与复盘都围绕这份计划进行

### 4.2 `position_adjustment_event`

作用：

- 记录同一标的同一计划下的多次实际调仓动作

绑定字段：

- `adjustment_event_ref`
- `position_plan_ref`
- `decision_ref`
- `approval_ref`
- `evidence_version`

核心内容：

- `event_type`
- `event_date`
- `before_position_pct`
- `after_position_pct`
- `trigger_reason`
- `price_context`
- `plan_alignment`
- `decision_note`

支持动作：

- `build`
- `add`
- `reduce`
- `exit`
- `risk_update`

说明：

- 每一条事件只代表一次动作
- 事件记录的是“实际执行”，不是重新生成仓位建议

### 4.3 `post_trade_review`

作用：

- 对同一计划周期做阶段性或结束性投后复盘

绑定字段：

- `post_trade_review_ref`
- `position_plan_ref`
- `decision_ref`
- `approval_ref`
- `evidence_version`

输入依赖：

- 原始 `position_plan_record`
- 多条 `position_adjustment_event`
- 当前阶段结果或最终结果

核心输出：

- `review_outcome`
- `decision_accuracy`
- `execution_quality`
- `risk_control_quality`
- `biggest_mistake`
- `what_worked`
- `what_failed`
- `correction_actions`
- `next_cycle_guidance`

说明：

- 这是聚合性对象，不替代事件流水
- 可以支持阶段复盘，不要求必须清仓后才能生成

## 5. 数据流

正式主链调整后应为：

`analysis -> briefing -> committee -> approval -> position_plan_record -> position_adjustment_event* -> post_trade_review`

其中：

- `briefing` 负责投前事实与初始仓位建议
- `committee` 负责正式审议结论
- `position_plan_record` 负责把计划固定下来
- `position_adjustment_event` 负责记录实际执行
- `post_trade_review` 负责阶段性归因与纠偏

## 6. 复盘维度

V1 固定 5 个复盘维度：

### 6.1 方向判断

- 看多/看空/观望是否成立

### 6.2 仓位决策

- 初始仓位和最大仓位是否合理

### 6.3 调仓纪律

- 加仓、减仓、观望是否按计划阈值执行

### 6.4 风控执行

- 止损、减仓、回撤控制是否做到位

### 6.5 研究充分性

- 技术面、基本面、信息面、赔率面是否足以支撑当时决策

## 7. 计划一致性规则

### 7.1 `plan_alignment`

每条调仓事件都要明确标记与计划的一致性：

- `on_plan`
- `justified_deviation`
- `off_plan`

### 7.2 复盘不是事后诸葛亮

复盘时优先比较：

- 原始计划是否合理
- 执行是否一致
- 风控是否到位

而不是简单根据最终盈亏倒推“当初对错”。

## 8. Tool 接入方案

本轮建议新增 3 个正式 Tool：

### 8.1 `security_position_plan_record`

作用：

- 把当前 `position_plan` 正式固化为计划对象

输出：

- `position_plan_ref`

### 8.2 `security_record_position_adjustment`

作用：

- 记录一次实际调仓事件

输出：

- `adjustment_event_ref`

### 8.3 `security_post_trade_review`

作用：

- 基于计划与调仓事件生成正式复盘

输出：

- `post_trade_review_ref`
- 结构化复盘摘要

## 9. 最小改动面

- `src/ops/security_decision_briefing.rs`
  - 复用既有 `position_plan` 合同，补正式 record 装配入口所需字段
- `src/ops/`
  - 新增仓位计划记录、调仓事件记录、投后复盘三个证券主链对象/Tool
- `src/tools/catalog.rs`
  - 注册新 Tool
- `src/tools/dispatcher.rs`
  - 接入 dispatcher 分发
- `src/tools/dispatcher/stock_ops.rs`
  - 接入 stock tool 组
- `tests/`
  - 补合同测试、CLI 测试、闭环测试
- `docs/AI_HANDOFF.md`
  - 更新证券主链当前阶段

## 10. 分阶段实施建议

### 10.1 P0-1

- 先做 `position_plan_record`

### 10.2 P0-2

- 再做 `position_adjustment_event`

### 10.3 P0-3

- 最后做 `post_trade_review`

## 11. 预期结果

本轮完成后，证券主链将第一次具备：

- 投前有正式仓位计划
- 投中有多次调仓记录
- 投后有回指原始决策的正式复盘

也就是说，平台会从“能分析、能投决”进入“能执行、能纠偏”的闭环阶段。
