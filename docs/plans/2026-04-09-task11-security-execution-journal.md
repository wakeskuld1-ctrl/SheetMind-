# 2026-04-09 Task 11 Security Execution Journal 收口

## 背景

Task 11 按已批准的方案 A-1 推进：不直接打碎既有 `security_execution_record` 合同，而是先新增独立正式对象 `security_execution_journal`，再让 `execution_record` 复用 journal 聚合结果。

## 本轮实现

- 新增正式 Tool：`security_execution_journal`
- 新增正式成交输入：`execution_trades`
  - `trade_date`
  - `side`
  - `price`
  - `position_pct_delta`
  - `reason`
  - `notes`
- 新增正式文档：`SecurityExecutionJournalDocument`
  - `trades`
  - `trade_count`
  - `entry_trade_count`
  - `exit_trade_count`
  - `holding_start_date`
  - `holding_end_date`
  - `peak_position_pct`
  - `final_position_pct`
  - `weighted_entry_price`
  - `weighted_exit_price`
  - `realized_return`
- `security_execution_record`
  - 兼容旧单次进出字段
  - 若传 `execution_trades`，则先聚合 `security_execution_journal`
  - 新增 `execution_journal_ref`
- `security_post_trade_review`
  - 新增 `execution_journal_ref`
  - 结果中正式暴露 `execution_journal_result`
  - 结果中正式暴露 `execution_journal`
- `security_decision_package`
  - 新增 `execution_journal_result`
  - 新增 `execution_journal`
  - `object_graph` 新增 `execution_journal`
  - `artifact_manifest` 新增 `security_execution_journal`
- `security_decision_verify_package`
  - 新增：
    - `object_graph_execution_journal_bound`
    - `artifact_manifest_execution_journal_bound`
    - `execution_journal_refs_consistent`
  - 新增问题码：
    - `missing_execution_journal`
    - `execution_journal_ref_misaligned`
- `security_decision_package_revision`
  - 新增 execution journal 缺失和错绑的修补建议

## 关键设计结论

- 当前 `journal v1` 明确支持：
  - 分批买入
  - 分批卖出
  - 加仓
  - 减仓
- 当前 `journal v1` 明确边界：
  - 要求同一条 journal 最终回到空仓
  - 要求买卖仓位完全匹配
  - 不处理跨周期持仓延续
  - 不处理账户级现金流
  - 不处理 FIFO/LIFO 税务核算
- 当前 `execution_record` 没有被删除，而是升级为：
  - `journal` 明细事实层
  - `record` 聚合摘要层

## 验证

- `cargo fmt --all`
- `cargo test --test security_execution_journal_cli -- --nocapture`
- `cargo test --test security_execution_record_cli -- --nocapture`
- `cargo test --test security_post_trade_review_cli -- --nocapture`
- `cargo test --test security_decision_package_cli -- --nocapture`

## 当前边界

- 当前只支持“闭合型 journal”，即最终回到空仓
- 当前还没有账户级仓位管理
- 当前还没有券商回单 / 外部流水接入
- 当前还没有更细的多笔成交归因拆解
- 当前还没有执行全量 `cargo test`

## 后续建议

- 如果继续方案 A，下一个优先级应转到：
  - 账户级仓位管理
  - 更细收益归因
- 如果继续沿 journal 深挖，优先方向是：
  - 非空仓结束的持仓型 journal
  - 多周期串联 journal
  - 更细执行归因模板
