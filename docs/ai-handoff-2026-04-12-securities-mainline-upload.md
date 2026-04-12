# 项目交接摘要（给后续AI）
更新时间：2026-04-12

## 1. 项目目标
- 本轮目标是把 `2026-04-10` 以来与证券/股票主链相关的工作整理后上传到 GitHub。
- 当前交付目标不是宣布“已经可以放心做实盘”，而是把证券主链、验证链、治理链和最新 ETF proxy 修复打成一个可交接分支。
- 当前上传分支是：
  - `codex/etf-proxy-import-latest-ready-20260412`

## 2. 当前已经确认的核心结论
- ETF validation / pool 流程已经能自动导入池级 proxy history。
- 当 ETF 缺少所需 proxy history 时，流程现在会显式失败，不再给假阳性结果。
- `513180.SH` 与 `515790.SH` 的 latest `2026-04-10` raw 已确认从 `feature_incomplete` 变成 `score_status = ready`。
- 这轮上传收口的是证券主链与交接上下文，不包含 `docs/AI_HANDOFF.md` 冲突文件。

## 3. 当前主入口与关键文件

### 3.1 主入口
- `src/ops/security_*`
- `src/ops/stock.rs`
- `src/tools/dispatcher/stock_ops.rs`

### 3.2 关键文件
- `src/ops/security_real_data_validation_backfill.rs`
- `tests/security_real_data_validation_backfill_cli.rs`
- `src/ops/security_master_scorecard.rs`
- `src/ops/security_chair_resolution.rs`
- `src/ops/security_decision_submit_approval.rs`
- `src/ops/security_condition_review.rs`
- `src/ops/security_execution_record.rs`
- `src/ops/security_post_trade_review.rs`

### 3.3 关键文档
- `docs/execution-notes-2026-04-12-p8-lifecycle-closeout.md`
- `docs/execution-notes-2026-04-12-p9-p10-validation-closeout.md`
- `docs/execution-notes-2026-04-12-real-data-validation-backfill.md`
- `docs/execution-notes-2026-04-13-multi-head-live-validation.md`
- `docs/execution-notes-2026-04-12-github-upload-securities-mainline.md`

## 4. 当前数据源 / 配置来源
- governed stock history:
  - `.excel_skill_runtime\\stock_history.db`
- governed ETF external proxy history:
  - `.excel_skill_runtime\\security_external_proxy.db`
- latest ETF pool rerun summaries:
  - `.excel_skill_runtime\\pool_training_fix_20260412\\latest_chair_pool_summary_after_proxy_import.json`
- current latest rerun detail:
  - `.excel_skill_runtime\\pool_training_fix_20260412\\513180_SH_latest_chair_raw_after_proxy_import.json`
  - `.excel_skill_runtime\\pool_training_fix_20260412\\515790_SH_latest_chair_raw_after_proxy_import.json`

## 5. 已处理过的问题与结论

### 5.1 ETF 池级代理历史漏导
- 现象：
  - pooled holdout 已改善，但 latest ETF rerun 仍然掉到 `feature_incomplete`
- 根因：
  - pool-generated proxy CSV 已存在，但没有自动导入 governed external proxy store
- 当前修复：
  - `security_real_data_validation_backfill` 会自动发现并导入 ETF pool proxy history
  - 若缺失，则流程显式失败
- 不要再走的弯路：
  - 不要只看 summary 文件就以为 latest 已修好，必须对照 raw payload

## 6. 当前最新输出 / 产物
- latest 汇总：
  - `.excel_skill_runtime\\pool_training_fix_20260412\\latest_chair_pool_summary_after_proxy_import.json`
- 上传执行说明：
  - `docs/execution-notes-2026-04-12-github-upload-securities-mainline.md`

当前已确认：
- `513180.SH`
  - `analysis_date = 2026-04-10`
  - `score_status = ready`
  - `selected_action = abstain`
- `515790.SH`
  - `analysis_date = 2026-04-10`
  - `score_status = ready`
  - `selected_action = abstain`

## 7. 已跑过的验证
- `cargo fmt --all`
- `cargo test --test security_real_data_validation_backfill_cli -- --nocapture`

结果摘要：
- 这条 ETF pool proxy auto-import 回归已通过。
- 更广的证券验证证据请优先查看前述 execution notes。

## 8. 当前仍需注意的点

### 8.1 风险或待确认点
- `ready` 不等于“已经可以放心做实盘”，当前 latest 最终动作仍保守。
- `docs/AI_HANDOFF.md` 仍处于冲突态，本轮上传故意不碰。

### 8.2 后续可能继续出现的问题
- 如果后续 pool CSV 命名规则变化，ETF auto-discovery 仍需同步调整。
- 如果需要隔离验证，请显式指定独立的 `EXCEL_SKILL_EXTERNAL_PROXY_DB`。

## 9. 如果后续 AI 要继续做，建议从这里开始
1. 先读本文档。
2. 再读这些关键文件：
   - `src/ops/security_real_data_validation_backfill.rs`
   - `tests/security_real_data_validation_backfill_cli.rs`
   - `docs/execution-notes-2026-04-12-github-upload-securities-mainline.md`
3. 再执行这些验证或打开这些输出：
   - `cargo test --test security_real_data_validation_backfill_cli -- --nocapture`
   - `.excel_skill_runtime\\pool_training_fix_20260412\\latest_chair_pool_summary_after_proxy_import.json`
4. 再决定下一步：
   - 继续扩证券主链验收范围
   - 或继续推进“可放心做实盘”的整包验收

## 10. 对后续 AI 的明确提醒
- 不要误把 `ready` 解释成“治理已放行”。
- 不要改动 `docs/AI_HANDOFF.md`，除非先处理掉冲突。
- 如果要继续上传，仍然要保持“只 stage 证券主链相关文件”的边界。

## 11. 一句话总结
- 当前项目已经把证券主链和 ETF latest proxy 修复整理到可上传状态；下一位 AI 最应该先验证本次上传分支的范围与 latest rerun 结果是否一致。
