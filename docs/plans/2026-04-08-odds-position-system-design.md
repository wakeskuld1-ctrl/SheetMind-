# 基于现有 signal_outcome_research 的赔率系统与仓位管理设计

<!-- 2026-04-08 CST: 新增赔率系统与仓位管理设计文档。原因：用户已明确要求基于既有 signal_outcome_research 继续做，不允许重复开发平行复盘模块。目的：把“研究事实层 + 决策装配层”的边界、字段与最小改动面正式写清楚，供本轮实现与后续 AI 接续统一遵循。 -->

## 1. 背景

当前仓库已经具备正式的历史研究主线：

- `record_security_signal_snapshot`
- `backfill_security_signal_outcomes`
- `study_security_signal_analogs`
- `signal_outcome_research_summary`

这条链路已经能提供：

- 相似样本数量
- 10 日胜率
- 10 日平均/中位收益
- 预期收益区间
- 预期回撤区间
- 历史研究限制说明

因此，本轮不新建第二套“历史复盘系统”或“赔率研究模块”，而是在既有研究层之上新增轻量决策装配层。

## 2. 设计原则

### 2.1 研究层不重复

- `signal_outcome_research` 继续作为唯一研究事实层。
- 赔率系统只能消费研究层输出，不直接自己扫库重算。
- 仓位管理只能消费研究层、共振层、执行层的正式结果，不手工重拼第二套事实。

### 2.2 决策层轻量装配

- 在 `security_decision_briefing` 内新增：
  - `odds_brief`
  - `position_plan`
- 在 `committee_payload` 内同步新增：
  - `odds_digest`
  - `position_digest`

这样做的目的不是让投决会马上重写全部规则，而是先确保 briefing 与 committee 看到的是同一份赔率/仓位事实。

### 2.3 最小闭环优先

本轮只做 V1：

- 赔率：回答“值不值得做”
- 仓位：回答“怎么下、先下多少、什么时候加减”

不在本轮引入：

- 复杂风险预算
- Kelly 全公式仓位
- 组合级相关性管理
- 新的独立存储层

## 3. 字段方案

### 3.1 赔率层 `odds_brief`

来源：

- `CommitteeHistoricalDigest`
- `signal_outcome_research_summary`

V1 字段：

- `status`
- `historical_confidence`
- `sample_count`
- `win_rate_10d`
- `loss_rate_10d`
- `flat_rate_10d`
- `avg_return_10d`
- `median_return_10d`
- `avg_win_return_10d`
- `avg_loss_return_10d`
- `payoff_ratio_10d`
- `expectancy_10d`
- `expected_return_window`
- `expected_drawdown_window`
- `odds_grade`
- `confidence_grade`
- `rationale`
- `research_limitations`

说明：

- `win/loss/flat` 直接基于已持久化的 `matched_analogs.forward_return_10d` 计算。
- `payoff_ratio_10d = avg_win_return_10d / abs(avg_loss_return_10d)`。
- `expectancy_10d = win_rate * avg_win + loss_rate * avg_loss`。
- `odds_grade` 是规则分档，不做统计学习模型。

### 3.2 仓位层 `position_plan`

来源：

- `odds_brief`
- `ExecutionPlan`
- `SecurityAnalysisResonanceResult`

V1 字段：

- `position_action`
- `entry_mode`
- `starter_position_pct`
- `max_position_pct`
- `add_on_trigger`
- `reduce_on_trigger`
- `hard_stop_trigger`
- `liquidity_cap`
- `position_risk_grade`
- `regime_adjustment`
- `execution_notes`
- `rationale`

说明：

- 不单独再造“交易执行层”，而是复用已有 `execution_plan` 的价位与动作阈值。
- 仓位建议采用规则分档：
  - 赔率等级
  - 历史置信度
  - 共振分数
  - 当前动作偏向
  - 流动性代理

## 4. 规则概念

### 4.1 赔率等级

- `A`：胜率、赔率比、期望值同时较优
- `B`：胜率与期望值较优，但强度略弱
- `C`：边际为正，但不适合重仓
- `D`：统计边际弱，只能观察或极小试仓
- `E`：赔率劣化，不支持新开仓
- `pending_research`：历史研究未就绪

### 4.2 仓位动作

- `build_on_strength`
- `starter_then_confirm`
- `pilot_only`
- `wait`
- `defensive_reduce`

### 4.3 仓位上限原则

- 仓位不是预测准确率的奖励，而是对“赔率质量 + 研究可信度 + 执行可控性”的共同约束。
- 历史研究 unavailable 时，只允许观察仓或等待，不允许直接给高上限。

## 5. 最小改动文件

- `src/ops/signal_outcome_research.rs`
  - 扩展研究摘要，补赔率计算所需数值字段
- `src/ops/security_decision_briefing.rs`
  - 新增 `odds_brief / position_plan`
  - 新增 `committee_payload.odds_digest / position_digest`
  - 新增装配与分档规则
- `tests/security_analysis_resonance_cli.rs`
  - 新增 briefing 顶层赔率层/仓位层红测
- `tests/integration_tool_contract.rs`
  - 更新 briefing 对外合同样例
- `tests/security_committee_vote_cli.rs`
  - 更新 `CommitteePayload` 测试夹具
- `docs/AI_HANDOFF.md`
  - 补当前已接入的赔率/仓位事实层位置与边界
- `docs/DOCUMENTATION_INDEX.md`
  - 收录本设计文档

## 6. 本轮不做什么

- 不新增独立 odds SQLite
- 不新增独立 position SQLite
- 不在 committee vote 中立即重写全部席位逻辑
- 不把 ETF/海外品种专用赔率逻辑混进本轮

## 7. 预期结果

本轮完成后，正式主链应变为：

- `signal_outcome_research`：研究事实层
- `security_decision_briefing.odds_brief`：赔率解释层
- `security_decision_briefing.position_plan`：仓位建议层
- `committee_payload.odds_digest / position_digest`：投决会可消费的同源摘要

这意味着平台第一次具备“投前研究 -> 赔率判断 -> 仓位建议 -> 投决承载”的最小闭环基础。
