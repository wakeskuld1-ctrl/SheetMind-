# 2026-04-08 证券主链仓位管理与投后复盘收口说明

## 本次目标

- 把证券主链上已经完成的“仓位计划正式化 -> 多次调仓记录 -> 投后复盘”最小闭环整理为可交接、可验证、可上传的完整批次。
- 在上传前补齐 README、AI handoff、任务日志与执行记录，避免后续 AI 只能通过 `git diff` 反推上下文。
- 在推送前重新执行与本批任务直接相关的定向验证，确保文档口径和当前实现一致。

## 本次实际落地范围

- 正式仓位计划对象：
  - `src/ops/security_position_plan_record.rs`
  - 负责把 `security_decision_briefing.position_plan` 落成正式 `position_plan_ref`
- 正式调仓事件对象：
  - `src/ops/security_record_position_adjustment.rs`
  - 支持围绕同一 `position_plan_ref` 连续记录多次调仓事件
- 正式投后复盘对象：
  - `src/ops/security_post_trade_review.rs`
  - 只消费 `position_plan_ref + adjustment_event_refs`，通过 ref 回读计划与事件并执行一致性校验
- 执行层存储：
  - `src/runtime/security_execution_store.rs`
  - 当前负责 `position_plan` 与 `adjustment_event` 的最小 SQLite 落盘与回读
- 主链接线与合同：
  - `src/tools/contracts.rs`
  - `src/tools/catalog.rs`
  - `src/tools/dispatcher.rs`
  - `src/tools/dispatcher/stock_ops.rs`
  - `src/ops/stock.rs`
  - `src/ops/mod.rs`
- 文档收口：
  - `README.md`
  - `docs/AI_HANDOFF.md`
  - `.trae/CHANGELOG_TASK.md`
  - `docs/plans/2026-04-08-security-post-trade-review-position-management-design.md`
  - `docs/plans/2026-04-08-security-post-trade-review-position-management.md`

## 本次重新验证的命令

- `cargo test --test integration_tool_contract -- --nocapture`
  - 结果：通过，`14 passed; 0 failed`
- `cargo test --test security_analysis_resonance_cli security_position_plan_record_persists_briefing_plan -- --nocapture`
  - 结果：通过
- `cargo test --test security_analysis_resonance_cli security_post_trade_review -- --nocapture`
  - 结果：通过，覆盖聚合与仓位衔接断裂报错
- `cargo test --test security_committee_vote_cli security_record_position_adjustment_supports_multiple_events -- --nocapture`
  - 结果：通过

## 当前已确认状态

- 证券主链已经不再只是“给出仓位建议”，而是具备单标的执行与复盘最小闭环。
- `security_post_trade_review` 当前会执行：
  - `position_plan_ref` 一致性校验
  - `symbol / decision_ref / approval_ref / evidence_version` 一致性校验
  - `event_date` 顺序校验
  - 相邻事件仓位衔接校验
  - 轻规则复盘维度聚合
- README 与 AI handoff 已同步更新到上述口径。

## 当前未完成项

- 复盘结果尚未正式装订为审批简报对象或 decision package 资产。
- 复盘结论尚未接入真实收益结果、赔率兑现与信号结果研究层。
- 同一日同类型多次调仓事件仍缺版本号或序号策略。
- 组合层仓位治理、盘中执行日志、滑点与成交质量仍未进入正式对象。

## 接手建议

1. 先读 `docs/AI_HANDOFF.md`
2. 再读 `docs/plans/2026-04-08-security-post-trade-review-position-management.md`
3. 再看：
   - `src/ops/security_position_plan_record.rs`
   - `src/ops/security_record_position_adjustment.rs`
   - `src/ops/security_post_trade_review.rs`
   - `src/runtime/security_execution_store.rs`
4. 如果继续推进，默认优先顺序为：
   - 复盘结果资产化/审批绑定
   - 信号结果研究层接入复盘
   - 多次同日调仓的 ref 版本化
   - 更细粒度执行日志
