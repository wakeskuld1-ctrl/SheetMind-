# 2026-04-09 Task 9 Post Trade Review Package Governance 收口

## 背景

Task 9 采用已批准的方案 B：不重做 `security_post_trade_review` 本身，而是把它正式挂进 `security_decision_package -> security_decision_verify_package -> security_decision_package_revision` 治理闭环。

## 本轮实现

- `security_decision_package`
  - 正式装配 `post_trade_review_result`
  - 正式挂载 `post_trade_review`
  - 把 `post_trade_review` 写入 `object_graph`
  - 把 `security_post_trade_review` 写入 `artifact_manifest`
- `security_decision_verify_package`
  - 新增 `object_graph_post_trade_review_bound`
  - 新增 `artifact_manifest_post_trade_review_bound`
  - 新增 `post_trade_review_refs_consistent`
  - 新增问题码：
    - `missing_post_trade_review`
    - `post_trade_review_ref_misaligned`
- `security_decision_package_revision`
  - 对 `missing_post_trade_review` 输出补挂建议
  - 对 `post_trade_review_ref_misaligned` 输出重新绑定建议
- `tests/security_decision_package_cli.rs`
  - 锁定 `post_trade_review` 必须进入 `object_graph`
  - 锁定 `post_trade_review` 必须进入 `artifact_manifest`
  - 锁定 verify 必须识别 review 引用错绑
  - 锁定 revision 必须输出明确修补动作

## 调试结论

- 本轮真实根因不是治理代码缺失，而是测试夹具日期窗口冲突
- 首次失败原因：
  - `as_of_date = 2025-07-15`
  - 技术分析至少需要 200 条历史样本
  - 截止该日期有效历史只有 196 条
- 最终修法：
  - 保留更长历史夹具 `420` 条
  - 把 `package_request().as_of_date` 调整到 `2025-10-15`
- 这样同时满足：
  - 技术分析最少 200 条历史窗口
  - `review_horizon_days = 20` 的未来观察窗口

## 关键设计结论

- `Task 9` 的目标是“投后复盘治理挂接”，不是“再造一个投后复盘 Tool”
- `post_trade_review` 现在已经是 package 的正式对象，而不是外层手工补件
- verify 现在不只检查会后结论，也会检查投后复盘是否真实挂载、引用是否同源
- revision 现在能把 review 缺失和 review 错绑转成统一修补动作

## 验证

- `cargo test --test security_decision_package_cli -- --nocapture`
- `cargo test --test security_post_trade_review_cli -- --nocapture`
- `cargo test --test security_position_plan_cli -- --nocapture`

## 当前边界

- 本轮没有接入真实成交执行记录
- 本轮没有补签名、审计哈希、自动修补模板
- 本轮没有执行全量 `cargo test`

## 后续建议

- 若继续做完整投后闭环，可补：
  - review 与真实成交执行对象的绑定
  - package 内更强的哈希/签名/版本一致性校验
  - revision 的结构化 remediation payload
