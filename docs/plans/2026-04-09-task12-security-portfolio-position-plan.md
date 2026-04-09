# 2026-04-09 Task 12 Security Portfolio Position Plan 收口

## 背景

Task 12 按已批准的方案 A 推进：先补“最实用规则型”的账户级仓位管理，不做马科维茨类复杂优化，也不做自动再平衡。

本轮要解决的问题不是“单票该不该买”，而是“账户里这笔可部署现金现在优先给谁、给多少、受什么约束”。

## 本轮实现

- 新增正式 Tool：`security_portfolio_position_plan`
- 新增正式请求合同：`SecurityPortfolioPositionPlanRequest`
  - `account_id`
  - `total_equity`
  - `available_cash`
  - `min_cash_reserve_pct`
  - `max_single_position_pct`
  - `max_sector_exposure_pct`
  - `holdings`
  - `candidates`
  - `created_at`
- 新增账户级正式文档：`SecurityPortfolioPositionPlanDocument`
  - `current_cash_pct`
  - `deployable_cash_amount`
  - `deployable_cash_pct`
  - `current_invested_pct`
  - `concentration_warnings`
  - `allocations`
  - `portfolio_summary`
- 新增单标的账户分配建议：`PortfolioAllocationRecommendation`
  - `action`
  - `current_position_pct`
  - `target_position_pct`
  - `incremental_position_pct`
  - `recommended_trade_amount`
  - `priority_score`
  - `constraint_flags`
  - `rationale`
- 已接入正式主链边界：
  - `src/ops/stock.rs`
  - `src/ops/mod.rs`
  - `src/tools/catalog.rs`
  - `src/tools/dispatcher.rs`
  - `src/tools/dispatcher/stock_ops.rs`

## 关键设计结论

- 当前版本明确是“规则型账户分配”，不是组合优化器。
- 当前版本的正式约束只有 4 类：
  - 现金底线 `min_cash_reserve_pct`
  - 单票上限 `max_single_position_pct`
  - 行业上限 `max_sector_exposure_pct`
  - 风险等级上限 `risk_cap_pct`
- 候选标的当前按 `confidence + odds_grade - risk_penalty` 排序，再顺序分配可部署现金。
- `risk_cap_pct` 当前口径固定为：
  - `low => 0.20`
  - `medium => 0.15`
  - `high => 0.10`
  - `default => 0.12`
- 当前实现已修正一个真实 bug：
  - 同一 `symbol` 的现有暴露不再覆盖，而是累计求和。

## 验证

- `cargo test --test security_portfolio_position_plan_cli -- --nocapture`

## 当前边界

- 当前不做马科维茨、均值方差、风险平价等复杂组合优化。
- 当前不做自动再平衡。
- 当前不直接接券商账户、订单流或真实现金流水。
- 当前优先消费正式的 `security_position_plan` 结果，不重复在账户层重建第二套单票规则。
- 当前还没有全仓级回撤预算、相关性矩阵和跨资产对冲约束。

## 后续建议

- 如果继续方案 A，下一步优先级应转到：
  - 账户级风险预算
  - 更细的仓位分层模板
  - 执行后账户偏差回写
- 如果后续要做完整版本，再补：
  - 组合相关性约束
  - 回撤预算约束
  - 分账户 / 分策略簿管理
  - 自动再平衡与调仓建议审计

## 2026-04-09 第二轮补齐：账户级风险预算门禁

- 当前 `security_portfolio_position_plan` 已不再只是现金分配器，也开始具备最小账户级风险预算门禁。
- 本轮新增正式输入：
  - `max_portfolio_risk_budget_pct`
  - `current_portfolio_risk_budget_pct`
  - `max_single_trade_risk_budget_pct`
- 本轮新增正式输出：
  - `remaining_portfolio_risk_budget_pct`
  - `estimated_new_risk_budget_pct`
  - `total_portfolio_risk_budget_pct`
  - `risk_budget_warnings`
- 本轮新增逐标的字段：
  - `estimated_risk_budget_pct`
- 当前风险预算仍是“规则型折算”，不是波动率或协方差驱动模型：
  - `low => 0.25`
  - `medium => 0.50`
  - `high => 0.75`
  - `default => 0.60`
- 当前门禁逻辑是：
  - 先算账户剩余总风险预算
  - 再算单笔风险预算上限
  - 候选标的新增仓位同时受现金、行业、单票和风险预算约束
  - 风险预算耗尽后，后续候选将命中 `portfolio_risk_budget_reached`

## 第二轮验证

- `cargo fmt --all`
- `cargo test --test security_portfolio_position_plan_cli -- --nocapture`

## 2026-04-09 第三轮补齐：仓位分层模板

- 当前 `security_position_plan` 已补入最小正式分层模板字段：
  - `entry_tranche_pct`
  - `add_tranche_pct`
  - `reduce_tranche_pct`
  - `max_tranche_count`
  - `tranche_template`
  - `tranche_trigger_rules`
  - `cooldown_rule`
- 当前分层模板直接复用 `briefing_core.execution_plan`，不再新造第三套仓位算法。
- 当前 `security_portfolio_position_plan` 已能输出账户层分层建议：
  - `suggested_tranche_action`
  - `suggested_tranche_pct`
  - `remaining_tranche_count`
- 当前账户层会根据：
  - 当前持仓占比
  - 单票 starter / max
  - add tranche 大小
  - 现金与风险预算门禁
  来决定本次属于 `entry_tranche / add_tranche / hold / reduce_tranche`

## 第三轮验证

- `cargo fmt --all`
- `cargo test --test security_position_plan_cli -- --nocapture`
- `cargo test --test security_portfolio_position_plan_cli -- --nocapture`
## 2026-04-09 第四轮补齐：账户偏差回写

- 当前 `security_execution_record` 已支持可选接入 `portfolio_position_plan_document`，并正式回写账户层计划对齐字段：
  - `portfolio_position_plan_ref`
  - `planned_tranche_action`
  - `planned_tranche_pct`
  - `planned_peak_position_pct`
  - `actual_tranche_action`
  - `actual_tranche_pct`
  - `actual_peak_position_pct`
  - `tranche_count_drift`
  - `account_budget_alignment`
- 当前 `security_post_trade_review` 已继续上卷账户层复盘字段：
  - `account_plan_alignment`
  - `tranche_discipline`
  - `budget_drift_reason`
  - `next_account_adjustment_hint`
- 当前设计口径已经明确：
  - `execution_record` 负责记录账户层“计划 vs 实际”的事实偏差
  - `post_trade_review` 负责把这份偏差翻译成治理结论和下一步动作提示
  - `decision_package` 在未传账户计划时显式走 `None`，保持旧链路兼容
- 当前 `tranche_count_drift` 已按“账户层这次明确建议的 tranche 大小”计算，而不是回退到单票默认 starter 基线。目的：避免 drift 被单票旧字段口径干扰。

## 第四轮验证
- `cargo fmt --all`
- `cargo test --test security_execution_record_cli -- --nocapture`
- `cargo test --test security_post_trade_review_cli -- --nocapture`
