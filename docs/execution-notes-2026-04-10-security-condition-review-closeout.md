# 2026-04-10 条件复核中枢收口执行说明

## 本次目标

- 把证券主链里的 `security_condition_review` 从最小合同推进到正式交接状态。
- 把投中层最新事实写回主 handoff 和证券专项交接摘要。
- 在推送前重新跑一轮与条件复核相关的定向验证，避免“代码已推但口径还是旧的”。

## 本次实际收口范围

- 文档收口：
  - `docs/AI_HANDOFF.md`
  - `docs/交接摘要_证券分析_给后续AI.md`
  - `.trae/CHANGELOG_TASK.md`
- 事实口径：
  - “投中监控中枢”正式统一为“条件复核中枢”
  - 当前不依赖实时数据
  - 当前正式支持四类触发：`manual_review / end_of_day_review / event_review / data_staleness_review`
  - 当前主链已经把 `condition_review_ref` 挂进 package、execution、review
- 验证清单：
  - 默认最小回归已修正为 8 条
  - 旧文档里残留的 `security_decision_package_cli` 在当前仓库并不存在，实际以 `security_decision_verify_package_cli + security_decision_package_revision_cli` 代替

## 需要后续继续注意的边界

- 当前 `condition_review_id` 还没有同日同类型多次复核的版本策略。
- 当前 execution/review 还不支持“只给 ref 就自动回仓储查 condition review 文档”。
- 当前这层是“条件复核中枢”，不是实时监控系统。

## 本轮验证命令

- `cargo test --test security_condition_review_cli -- --nocapture`
- `cargo test --test security_decision_verify_package_cli -- --nocapture`
- `cargo test --test security_decision_package_revision_cli -- --nocapture`
- `cargo test --test security_execution_record_cli -- --nocapture`
- `cargo test --test security_post_trade_review_cli -- --nocapture`
- `cargo test --test security_committee_vote_cli -- --nocapture`
- `cargo test --test security_scorecard_training_cli -- --nocapture`
- `cargo test --tests --no-run`

## 本轮实际验证结果

- 已通过：
  - `security_condition_review_cli`
  - `security_committee_vote_cli`
  - `security_decision_verify_package_cli`
  - `security_decision_package_revision_cli`
  - `security_execution_record_cli`
  - `security_post_trade_review_cli`
  - `cargo test --tests --no-run`
- 已确认旧命令问题：
  - `security_decision_package_cli` 在当前仓库中不存在，属于历史文档残留口径。
- 当前失败项：
  - `security_scorecard_training_cli`
  - 失败测试：`security_scorecard_training_generates_artifact_and_registers_refit_outputs`

## 推送前建议

- 优先只 stage 本轮条件复核收口相关文档与主链改动。
- 如果运行时目录或测试夹具新生成了本地产物，先按 handoff 里的“运行时产物规则”判断是否应忽略。
