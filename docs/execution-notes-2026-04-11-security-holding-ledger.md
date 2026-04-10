# 2026-04-11 证券持仓台账执行说明

## 本轮完成内容
- 新建 [security-holding-ledger.md](/D:/Rust/Excel_Skill/docs/security-holding-ledger.md)，作为后续统一证券持仓台账。
- 把 `2026-04-11` 保本优先版组合的 3 条正式持仓计划写入台账。
- 明确台账口径：先记 `position_plan_ref`，真实成交后再补 `build/add/reduce/exit` 事件。

## 已记录的正式持仓编码
- `position-plan:511360.SH:2026-04-11:v1`
- `position-plan:511010.SH:2026-04-11:v1`
- `position-plan:511060.SH:2026-04-11:v1`

## 本轮验证
- 通过正式 CLI Tool 写入 3 条持仓计划记录。
- 持仓计划落库路径：`.excel_skill_runtime/security_execution.db`

## 未完成项
- 尚未补真实成交后的 `build` 事件。
- 尚未进入单标的或组合级复盘。

## 风险提醒
- 当前台账中的 3 条记录属于“计划锚点”，不是成交回报。
- 如果后续真实买入日期不是 `2026-04-11`，建仓事件日期必须按真实成交日补录，不能沿用计划日期。
