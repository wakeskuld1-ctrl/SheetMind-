# 2026-04-09 Task 8 Security Post Trade Review 收口

## 背景

Task 8 采用已批准的方案 A：不重造快照、未来结果和仓位建议逻辑，只把现有 `security_position_plan` 与 `security_forward_outcome` 正式装配成最小可用的投后复盘 Tool。

## 本轮实现

- 新增 `security_post_trade_review`
  - 输入复用证券主链公共上下文，只补 `review_horizon_days`、止损止盈阈值与 `created_at`
  - 输出 `position_plan_result`、`forward_outcome_result` 与正式 `post_trade_review`
- 新增正式复盘对象 `SecurityPostTradeReviewDocument`
  - 固化 `planned_position`
  - 固化 `actual_result_window`
  - 固化 `realized_return`、`max_drawdown_realized`、`max_runup_realized`
  - 固化 `thesis_status`、`execution_deviation`、`model_miss_reason`、`next_adjustment_hint`
- 新增 `build_security_post_trade_review`
  - 统一组装正式复盘文档
  - 避免后续 package / audit / replay 在多个 Tool 里重复拼接复盘结论
- 新增 `tests/security_post_trade_review_cli.rs`
  - 锁定 `tool_catalog` 可发现性
  - 锁定复盘文档与 `position_plan` / `snapshot` / `forward_outcome` 的同源对齐关系

## 关键设计结论

- `security_post_trade_review` 的事实源仍然只有两份：
  - `security_position_plan`
  - `security_forward_outcome`
- 本轮不引入真实成交执行记录，因此当前复盘属于“建议层复盘”，不是“实盘成交复盘”
- 本轮归因规则保持最小正式版：
  - `hit_stop_first => broken`
  - `forward_return > 0 && max_drawdown <= 0.08 => validated`
  - `forward_return > 0 => mixed`
  - 其他情况 => `broken`
- `execution_deviation` 当前固定为 `not_tracked_v1`
  - 这是显式声明“暂未接入执行层”，不是用空值掩盖缺口

## 验证

- `cargo fmt --all`
- `cargo test --test security_post_trade_review_cli -- --nocapture`
- `cargo test --test security_forward_outcome_cli -- --nocapture`
- `cargo test --test security_position_plan_cli -- --nocapture`

## 后续建议

- 若后续进入完整投后闭环，应补真实执行记录输入
  - 例如成交价、成交时间、分批执行、实际止损止盈动作
- 若后续进入治理链，可把 `post_trade_review` 挂入 `decision_package` / `committee_payload` 的后续审阅对象
- 若后续进入策略复盘层，可继续补：
  - 执行偏差分类
  - 仓位是否给大/给小
  - 市场状态判断偏差
  - 因子权重修正建议
