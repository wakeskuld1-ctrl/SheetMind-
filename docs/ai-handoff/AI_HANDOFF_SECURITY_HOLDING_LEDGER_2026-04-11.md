# 证券持仓台账交接

## Start Here
- 先读 [security-holding-ledger.md](/D:/Rust/Excel_Skill/docs/security-holding-ledger.md)。
- 再看 [execution-notes-2026-04-11-security-holding-ledger.md](/D:/Rust/Excel_Skill/docs/execution-notes-2026-04-11-security-holding-ledger.md)。
- 如需核对运行时落库，检查 [security_execution.db](/D:/Rust/Excel_Skill/.excel_skill_runtime/security_execution.db)。

## 当前确认状态
- `2026-04-11` 保本优先版组合已经登记 3 条正式持仓计划。
- 当前只存在 `position_plan_ref`，还没有真实 `build` 事件。
- 后续真实下单后，应继续沿同一台账追加，不要改写历史条目。

## 本轮已登记的持仓编码
- `position-plan:511360.SH:2026-04-11:v1`
- `position-plan:511010.SH:2026-04-11:v1`
- `position-plan:511060.SH:2026-04-11:v1`

## 下一步该怎么做
- 用户给出真实成交价、成交金额、成交日期后，补 `security_record_position_adjustment` 的 `build` 事件。
- 后续每次加仓、减仓、退出都沿同一台账继续追加。
- 复盘时按 `position_plan_ref -> adjustment_event_ref -> post_trade_review_ref` 回查，不要绕过台账重新手写摘要。
