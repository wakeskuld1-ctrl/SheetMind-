# 2026-04-09 Task 7 Security Position Plan 收口

## 背景

Task 7 采用已批准的方案 A：不重写仓位算法，只把 `security_decision_briefing` 内已经存在的 `position_plan` / `odds_brief` / `committee_payload` 正式对象化为独立 Tool。

## 本轮实现

- 新增 `security_position_plan`
  - 输入基本复用 `SecurityDecisionBriefingRequest`
  - 补充 `created_at`
  - 输出 `briefing_core` 与 `position_plan_document`
- 新增正式文档对象 `SecurityPositionPlanDocument`
  - 固化推荐动作、赔率等级、历史置信度、仓位动作、入场模式、起始仓位、最大仓位、加减仓触发、硬止损、流动性上限、风险等级、执行备注与原因
- 新增 `SecurityDecisionBriefingCore`
  - 仅承载 briefing 核心事实层
  - 不再强绑定默认 `committee_recommendations`
- 新增 `security_decision_briefing_core`
  - 复用原 briefing 的统一事实装配
  - 让仓位计划 Tool 不再被不必要的 seat agent 子链阻塞

## 关键设计结论

- `security_position_plan` 的事实源仍然只有一份：`security_decision_briefing_core`
- 这轮没有新建第二套赔率计算、仓位计算或历史研究逻辑
- 这轮解决了一个真实阻塞：
  - 若直接依赖完整 `security_decision_briefing`
  - 会因为 `committee_recommendations` 子链触发 seat agent 调用
  - 在当前夹具下报 `member_id` 缺失
  - 因此 Task 7 必须改成“复用 briefing core，而不是复用完整 briefing 返回”

## 验证

- `cargo fmt --all`
- `cargo test --test security_position_plan_cli -- --nocapture`

## 后续建议

- 若后续要把仓位计划纳入更正式的 package / committee / audit 治理，可直接复用 `position_plan_document`
- 若后续要扩展投中层，不应新增平行仓位事实源，而应继续从 `security_decision_briefing_core` 往外派生
