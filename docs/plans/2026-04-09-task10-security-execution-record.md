# 2026-04-09 Task 10 Security Execution Record 收口

## 背景

Task 10 采用已批准的方案 A-2：不直接进入多笔成交 journal，而是先把真实执行对象 `security_execution_record` 正式落地，并把收益归因接入 `security_post_trade_review -> security_decision_package -> verify -> revision`。

## 本轮实现

- 新增 `security_execution_record`
  - 正式输入分析时点上下文
  - 正式输入真实执行字段：
    - `actual_entry_date`
    - `actual_entry_price`
    - `actual_position_pct`
    - `actual_exit_date`
    - `actual_exit_price`
    - `exit_reason`
    - `execution_record_notes`
  - 正式输出 `SecurityExecutionRecordDocument`
- 收益归因字段最小正式化
  - `planned_entry_price`
  - `planned_position_pct`
  - `planned_forward_return`
  - `actual_return`
  - `entry_slippage_pct`
  - `position_size_gap_pct`
  - `execution_return_gap`
  - `execution_quality`
  - `attribution_summary`
- `security_post_trade_review`
  - 改为复用 `security_execution_record`
  - 新增：
    - `execution_record_ref`
    - `executed_return`
    - `execution_return_gap`
  - `execution_deviation` 不再是 `not_tracked_v1`，而是正式引用 `execution_quality`
- `security_decision_package`
  - 正式挂入 `execution_record_result`
  - 正式挂入 `execution_record`
  - `object_graph` 新增 `execution_record`
  - `artifact_manifest` 新增 `security_execution_record`
- `security_decision_verify_package`
  - 新增：
    - `object_graph_execution_record_bound`
    - `artifact_manifest_execution_record_bound`
    - `execution_record_refs_consistent`
  - 新增 issue code：
    - `missing_execution_record`
    - `execution_record_ref_misaligned`
- `security_decision_package_revision`
  - 新增 execution record 缺失与错绑的修补建议

## 关键设计结论

- 本轮明确不做分批成交 journal
- `security_execution_record` 先锁“一次完整执行”的最小正式合同
- 收益归因先回答 3 个问题：
  - 真实入场是否比计划更差
  - 实际仓位是否偏离计划
  - 真实收益相对计划收益偏差多大
- `execution_quality` 当前最小分档：
  - `aligned`
  - `partial_drift`
  - `adverse`

## 排障记录

- 本轮业务逻辑没有出现新的主链错误
- Windows 下回归过程中出现 `target\\debug\\excel_skill.exe` 残留占用
- 处理方式：
  - 定位残留 `excel_skill` 进程
  - 结束残留进程后重新跑测试
- 这属于测试环境锁文件问题，不是 Task 10 业务失败

## 验证

- `cargo test --test security_execution_record_cli -- --nocapture`
- `cargo test --test security_post_trade_review_cli -- --nocapture`
- `cargo test --test security_decision_package_cli -- --nocapture`

## 当前边界

- 本轮只支持单次进出，不支持分批建仓 / 加仓 / 减仓 / 清仓 journal
- 本轮没有接入真实券商成交单或外部交易回执
- 本轮没有执行全量 `cargo test`

## 后续建议

- 如果 M3 收口后继续做增强，下一优先级可以是：
  - 多笔成交 `execution_journal`
  - review 对分批执行路径的归因
  - package 的签名 / 哈希 / 审计链增强
