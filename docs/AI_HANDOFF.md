# AI 交接手册

<!-- 2026-04-09 CST: 重写统一 AI 交接手册。原因：当前本地主文档已混入过期主线与乱码内容，而远端 foundation-navigation-kernel 又补进了新的证券治理与评分卡进展。目的：把“当前该看哪条主线、做到哪了、接下来该从哪里继续”统一收口到一份主文档里。 -->

## 1. 当前仓库怎么理解

这个仓库当前有两条并行能力线：

- 证券治理与证券分析主线
- foundation 通用导航内核线

但从当前实际开发连续性、用户需求和后续接续价值看，默认优先级应是：

1. 证券治理与证券分析主线
2. foundation 通用导航内核线

除非用户明确要求继续 foundation，否则后续默认先沿证券主线推进。

## 2. 当前正式主线

### 2.1 证券主线

当前证券主线已经不再只是单纯的技术分析，而是一个持续扩展中的证券治理链。

现阶段主线模块包括：

- 研究与分析层
  - `technical_consultation_basic`
  - `security_analysis_contextual`
  - `security_analysis_fullstack`
- 治理与审批层
  - `security_decision_evidence_bundle`
  - `security_decision_committee`
  - `security_position_plan`
  - `security_decision_submit_approval`
  - `security_decision_verify_package`
  - `security_decision_package_revision`
  - `security_record_post_meeting_conclusion`
- 评分卡与量化治理层
  - `security_feature_snapshot`
  - `security_forward_outcome`
  - `security_scorecard`
  - `security_scorecard_refit`
  - `security_scorecard_training`
  - `security_chair_resolution`

### 2.2 foundation 主线

foundation 当前是独立的通用能力内核，不应误判成证券主链的一部分。

当前主要包括：

- ontology schema / store
- capability router
- roaming engine
- retrieval engine
- evidence assembler
- navigation pipeline

它的职责仍然是通用知识导航，不是证券业务编排。

## 3. 当前证券主线做到哪了

### 3.1 已稳定落地的部分

- `analysis -> evidence_bundle -> committee -> submit_approval -> verify_package -> package_revision` 已形成正式链路
- `security_record_post_meeting_conclusion` 已形成 Task 3 最小闭环
- `scorecard` 已进入正式治理链，不再只是临时评分输出
- `verify` 已补 `scorecard_binding_consistent / scorecard_complete / scorecard_action_aligned` 护栏
- `chair_resolution`、`feature_snapshot`、`forward_outcome`、`scorecard_refit`、`scorecard_training` 已进入 catalog / dispatcher

### 3.2 当前明确未完全收口的部分

- `post_meeting_conclusion` 还没有完整挂入 `decision_package.object_graph`
- `post_meeting_conclusion` 还没有完整挂入 `artifact_manifest`
- `security_decision_verify_package` 的会后结论绑定校验还没有完全补齐
- `security_scorecard_training` 已完成最小正式训练主链收口，当前不再停留在端到端红测未通过状态

### 3.3 当前已知尾项

`security_scorecard_training` 的最小正式训练闭环已经打通，并且已经做到：

- 训练请求合同
- 样本采集
- `security_forward_outcome` 标签回填
- artifact 落盘
- `security_scorecard_refit` 接线
- `model_registry` 注册
- `security_scorecard_training_generates_artifact_and_registers_refit_outputs` 已转绿

当前状态已经从“主链已接通、仍有一处端到端红测待收敛”推进到“最小训练主链可用，后续可以继续做更完整的 walk-forward / champion-challenger / package 绑定治理”。

## 4. 日期与数据口径硬规则

- 证券分析默认只允许使用当前日期
- 如果当前日期没有有效收盘数据，才允许退到前一个交易日
- 输出中必须显式写明实际锚定日期
- 不允许混用多个交易日的数据拼一个结论
- 免费公开数据源失败时允许降级，但必须明确写出 unavailable 范围
- 不使用大模型抓行情
- 不使用 Token 依赖型证券数据入口作为默认主链

## 5. 后续接手最容易犯的错

- 把 foundation 线误当成当前默认主线
- 把 Python `tradingagents/` 架构误当成 Rust 证券主链
- 放着正式 Tool 不用，回退成手工拼接 JSON 或泛化股评
- 把 `scorecard` 当成最终主席决议，而不是量化线对象
- 把 `chair_resolution`、`committee`、`scorecard` 三条线重新混成一个输出
- 误以为 `security_scorecard_training` 还没做入口；实际当前工作区已具备正式入口，并已通过最小端到端训练回归

## 6. 推荐阅读顺序

### 6.1 继续证券主线时

1. `README.md`
2. `docs/AI_HANDOFF.md`
3. `docs/交接摘要_证券分析_给后续AI.md`
4. `docs/plans/2026-04-08-security-investment-lifecycle-roadmap.md`
5. `CHANGELOG_TASK.MD`

然后再按任务需要看：

- `src/ops/security_decision_evidence_bundle.rs`
- `src/ops/security_decision_committee.rs`
- `src/ops/security_position_plan.rs`
- `src/ops/security_decision_submit_approval.rs`
- `src/ops/security_decision_verify_package.rs`
- `src/ops/security_decision_package_revision.rs`
- `src/ops/security_record_post_meeting_conclusion.rs`
- `src/ops/security_scorecard.rs`
- `src/ops/security_scorecard_refit_run.rs`
- `src/ops/security_scorecard_training.rs`
- `src/ops/security_chair_resolution.rs`

### 6.2 继续 foundation 时

1. `README.md`
2. `docs/AI_HANDOFF.md`
3. `docs/architecture/`
4. `CHANGELOG_TASK.MD`

## 7. 当前建议优先级

如果继续做证券主线，建议优先顺序是：

1. 补齐会后结论进入 package / verify 的完整治理链
2. 继续做 scorecard 的 walk-forward / champion-challenger / 更完整训练验证
3. 再继续投中/投后闭环对象层

不要先重新开分支做新的平行实验线。

## 8. 当前工作区约定

从 2026-04-09 这轮开始，文档口径统一回收到当前主工作区。

- worktree 只允许临时用于观察或隔离验证
- 不再把长期交接信息分散写在 worktree 对应分支里
- 后续 AI 默认以当前主工作区文档为准

## 8.1 其他 worktree 吸收当前证券主链的默认方案

<!-- 2026-04-09 CST: 补充跨 worktree 吸收方案。原因：用户不希望切到别的开发位置时还要重新等待 AI 判断合并路径。目的：把“吸收哪条分支、需不需要重复解决冲突、推荐验证命令”写成可直接执行的默认手册。 -->

当前默认把 `codex/foundation-navigation-kernel` 视为这轮证券主链收口的主吸收入口；已经吸收过的 `codex/merge-cli-mod-batches` 不应再在别的地方重复手工打一遍同样的冲突。

### 默认结论

- 如果别的 worktree 还没吸收这轮证券主链，默认直接吸收 `codex/foundation-navigation-kernel`
- 不要再优先吸收旧的 `codex/merge-cli-mod-batches`，因为当前收口已经把它的结果继续往前推进了
- 其他 worktree 通常不需要“重复解决一次同样的冲突”；只需要在自己的分支上吸收当前收口结果

### 推荐操作顺序

1. 先在目标 worktree 提交或临时保存本地改动，避免脏工作区把合并判断搅乱
2. `git fetch origin`
3. 切到你要继续开发的目标分支
4. 如果当前仓库内已经能看到本地分支 `codex/foundation-navigation-kernel`，默认执行：
   - `git merge --no-ff codex/foundation-navigation-kernel`
5. 如果目标环境只能看到远端分支，默认执行：
   - `git merge --no-ff origin/codex/foundation-navigation-kernel`
6. 合并后优先跑最小验证：
   - `cargo test --test security_committee_vote_cli -- --nocapture`
   - `cargo test --tests --no-run`

### 什么时候才需要重新人工判断

只有下面三类情况，才需要重新评估而不是直接按上面方案吸收：

- 目标 worktree 上有你尚未提交、且与 `src/ops/security_*` 或 `src/tools/dispatcher*` 同时改动的内容
- 目标分支是明显偏 foundation 的实验分支，而不是证券主链延续
- 当前收口分支尚未提交/推送，但你又想在另一个物理仓库里吸收它

### 这轮已知冲突热点

如果目标 worktree 合并时出现冲突，优先关注这些文件，而不是从头怀疑整体方案：

- `src/ops/stock.rs`
- `src/ops/mod.rs`
- `src/tools/catalog.rs`
- `src/tools/dispatcher.rs`
- `src/tools/dispatcher/stock_ops.rs`
- `docs/AI_HANDOFF.md`
- `docs/交接摘要_证券分析_给后续AI.md`

### 后续统一口径

- 当前主工作区负责“主收口”
- 其他 worktree 负责“吸收主收口结果”
- 除非用户明确要求，不再在每个 worktree 里各自设计一套新的合并路径

## 8.2 证券主链最小验证清单

<!-- 2026-04-10 CST: 补充最小验证清单。原因：后续 AI 即使知道怎么吸收分支，如果不知道合并后先跑哪几条回归，仍然容易把“已吸收”和“已验证”混为一谈。目的：把证券主链当前最关键的定向回归固定成默认检查入口。 -->

如果后续 AI 在别的 worktree 吸收当前证券主链，默认先跑下面这组最小验证，而不是一上来就整仓长跑：

1. `cargo test --test security_committee_vote_cli -- --nocapture`
2. `cargo test --test security_decision_package_cli -- --nocapture`
3. `cargo test --test security_execution_record_cli -- --nocapture`
4. `cargo test --test security_post_trade_review_cli -- --nocapture`
5. `cargo test --test security_scorecard_training_cli -- --nocapture`
6. `cargo test --tests --no-run`

当前理解是：

- 前 5 条用于确认证券治理主链、投后复盘链和评分卡训练链没有在吸收时被打断
- 第 6 条用于确认至少所有测试目标仍然能成功编译
- 如果这 6 条都通过，再决定是否需要继续跑更重的全量回归

## 8.3 运行时产物与测试夹具规则

<!-- 2026-04-10 CST: 补充运行时产物策略。原因：当前仓库容易因为本地 runtime/db/thread memory 目录产生低价值脏状态。目的：明确哪些目录是本地产物、哪些才是应该评估是否纳入版本库的正式 fixture。 -->

下面这些目录按默认规则视为“本地产物”，不应作为正式代码改动提交：

- `.excel_skill_runtime/`
- `tests/runtime_fixtures/local_memory/`
- `tests/runtime_fixtures/thread_local_memory/`
- `tests/runtime_fixtures/integration_tool_contract/`
- `tests/runtime_fixtures/chart_ref_store/`
- `tests/runtime_fixtures/exports/`
- `tests/runtime_fixtures/generated_workbooks/`
- `tests/runtime_fixtures/local_memory_registry/`
- `tests/runtime_fixtures/result_ref_store*/`
- `tests/runtime_fixtures/table_ref_store/`

处理口径：

- 这些目录用于本地运行、线程态或集成测试落盘
- 它们可以帮助排查问题，但默认不是正式 fixture
- 如果 `git status` 里只出现这些目录，不要误判成主链功能变更

下面这类目录则需要人工判断，不能一刀切忽略：

- `tests/runtime_fixtures/security_scorecard_training/...`
- 其他已经存在已跟踪样本的业务夹具目录

原因是：这类目录里同时存在“正式样本夹具”和“新生成样本”，需要根据测试意图决定是否纳入版本库。

## 8.4 当前已知非 blocker

<!-- 2026-04-10 CST: 补充非 blocker 清单。原因：后续 AI 容易把既有 warning 误判成这轮主收口失败。目的：明确哪些问题当前已知存在，但不阻塞证券主链继续吸收和开发。 -->

当前已知但不阻塞本轮证券主链继续推进的问题：

- 仓库内仍有较多既有 `dead_code` warning
- `tests/security_scorecard_cli.rs` 仍有一个 `unused import: Path` warning

统一口径：

- 这些 warning 需要后续单独治理
- 它们不影响当前证券主链的核心闭环与最小验证
- 后续 AI 不要因为这些 warning 就回退或重做本轮证券主链收口判断

## 9. 一句话结论

当前仓库最有价值的正式主线，是“证券治理链 + 评分卡治理链”的持续收口；foundation 仍保留，但不是默认继续开发入口。
## 10. 2026-04-09 RSRS 正式收口补充

- `tests/security_scorecard_training_cli.rs` 已修正训练夹具的下跌样本生成逻辑，不再把 `low` 长时间压成固定 `0.10`
- `src/ops/technical_consultation_basic.rs` 已把 RSRS 从“分母为 0 直接回填中性 beta”改成“显式 `degraded` 状态”
- 当前合同新增 `rsrs_status`
- 当 `rsrs_status != "available"` 时：
  - `rsrs_signal = "degraded"`
  - `summary / recommended_actions / watch_points` 会明确写出“本次不作为方向判断依据”
  - 不再把这类退化 RSRS 伪装成正常 neutral 或共振未形成
- 已验证：
  - `cargo test --test security_scorecard_training_cli -- --nocapture`
  - `cargo test rsrs_degraded_guidance_explicitly_excludes_directional_use -- --nocapture`
  - `cargo test rsrs_snapshot_stays_computable_when_low_window_collapses_to_same_floor -- --nocapture`

这一条很重要：后续如果再遇到技术指标数学退化，默认原则不是“补一个中性值继续走”，而是“先修正；修不了就显式标记不可用，并退出投决输入”。
## 11. 2026-04-09 Task 6 package 治理链补齐

- `security_record_post_meeting_conclusion` 已正式落地，当前不再只是文档口径。
- `security_decision_package` 已正式落地，当前会把 `post_meeting_conclusion` 挂进：
  - `object_graph`
  - `artifact_manifest`
- `security_decision_verify_package` 已正式落地，当前至少会校验：
  - `post_meeting_conclusion` 是否挂进 `object_graph`
  - `post_meeting_conclusion` 是否挂进 `artifact_manifest`
  - `chair_resolution` 是否仍保留正式挂接
  - `artifact_manifest` 的 `symbol / analysis_date` 是否与 package 一致
- `security_decision_package_revision` 已正式落地，当前会基于 verify 结果输出：
  - `suggested_actions`
  - `missing_objects`
  - `manifest_repairs`
- 已接入：
  - `src/ops/stock.rs`
  - `src/ops/mod.rs`
  - `src/tools/catalog.rs`
  - `src/tools/dispatcher.rs`
  - `src/tools/dispatcher/stock_ops.rs`
- 已验证：
  - `cargo fmt --all`
  - `cargo test --test security_decision_package_cli -- --nocapture`
  - `cargo test --test security_chair_resolution_cli -- --nocapture`

这意味着当前证券主线里，Task 6 不再是“文档里计划过、代码里还没有”的状态，而是已经具备最小正式 package 闭环。
## 12. 2026-04-09 Task 8 投后复盘最小正式收口

- `security_post_trade_review` 已正式落地，当前不再只是“手工把 position_plan 和 forward_outcome 拼成复盘结论”
- 当前正式输出分三层：
  - `position_plan_result`
  - `forward_outcome_result`
  - `post_trade_review`
- `post_trade_review` 当前最小正式字段已经固定：
  - `planned_position`
  - `actual_result_window`
  - `realized_return`
  - `max_drawdown_realized`
  - `max_runup_realized`
  - `thesis_status`
  - `execution_deviation`
## 13. 2026-04-09 Task 9 投后复盘治理挂接收口

- `security_post_trade_review` 已正式挂入 `security_decision_package`
- 当前 package 会同时装配：
  - `post_meeting_conclusion`
  - `post_trade_review`
- `object_graph` 当前已正式包含：
  - `post_meeting_conclusion`
  - `post_trade_review`
- `artifact_manifest` 当前已正式包含：
  - `security_post_meeting_conclusion`
  - `security_post_trade_review`
- `security_decision_verify_package` 当前至少会校验：
  - `post_trade_review` 是否挂进 `object_graph`
  - `post_trade_review` 是否挂进 `artifact_manifest`
  - `post_trade_review.position_plan_ref / snapshot_ref / outcome_ref` 是否与底层结果同源
- `security_decision_package_revision` 当前已能对以下问题输出修补建议：
  - `missing_post_trade_review`
  - `post_trade_review_ref_misaligned`
- 本轮重要排障结论：
  - 之前 `security_decision_package_cli` 卡住，不是治理链没接上
  - 真正根因是测试夹具 `as_of_date = 2025-07-15` 导致技术分析只拿到 196 条历史样本
  - 当前已改为更长夹具 + `as_of_date = 2025-10-15`
- 已验证：
  - `cargo test --test security_decision_package_cli -- --nocapture`
  - `cargo test --test security_post_trade_review_cli -- --nocapture`
  - `cargo test --test security_position_plan_cli -- --nocapture`

这一条意味着当前证券主线里，Task 9 已经不是“投后复盘 Tool 单独可用”，而是已经正式并入 package 治理闭环。
## 14. 2026-04-09 Task 10 真实执行对象最小收口

- `security_execution_record` 已正式落地
- 当前 execution record 已不再是 review 内部占位字段，而是独立正式 Tool
- 当前最小正式执行字段已经固定：
  - `actual_entry_date`
  - `actual_entry_price`
  - `actual_position_pct`
  - `actual_exit_date`
  - `actual_exit_price`
  - `exit_reason`
  - `execution_record_notes`
- 当前最小收益归因字段已经固定：
  - `planned_entry_price`
  - `planned_position_pct`
  - `planned_forward_return`
  - `actual_return`
  - `entry_slippage_pct`
  - `position_size_gap_pct`
  - `execution_return_gap`
  - `execution_quality`
- `security_post_trade_review` 当前已经正式绑定：
  - `execution_record_ref`
  - `executed_return`
  - `execution_return_gap`
- `security_decision_package` 当前已经正式挂入：
  - `execution_record_result`
  - `execution_record`
- `security_decision_verify_package` 当前至少会校验：
  - `execution_record` 是否挂进 `object_graph`
  - `execution_record` 是否挂进 `artifact_manifest`
  - `execution_record` 与 `post_trade_review / position_plan / snapshot / outcome` 是否同源
- `security_decision_package_revision` 当前已能对以下问题输出修补建议：
  - `missing_execution_record`
  - `execution_record_ref_misaligned`
- 已验证：
 - `cargo test --test security_execution_record_cli -- --nocapture`
  - `cargo test --test security_post_trade_review_cli -- --nocapture`
  - `cargo test --test security_decision_package_cli -- --nocapture`

这一条意味着当前证券主线里的 M3 闭环底座，已经从“建议层复盘”推进到“真实执行对象已正式进入 review 与 package 治理链”。

补充说明：

- `Task 8` 阶段里曾经存在的 `execution_deviation = not_tracked_v1` 旧口径，已经被 `Task 10` 的正式 `security_execution_record` 接入所取代。
- 当前 `security_post_trade_review` 里的关键字段仍然保留并继续有效：
  - `model_miss_reason`
  - `next_adjustment_hint`
- 当前归因与执行质量规则仍属于最小正式版，不是最终完整版：
  - `hit_stop_first => broken`
  - `forward_return > 0 && max_drawdown <= 0.08 => validated`
  - `forward_return > 0 => mixed`
  - 其他情况 => `broken`
- 当前阶段明确复用既有底座，不重复开发：
  - `security_position_plan`
  - `security_forward_outcome`

## 15. 2026-04-09 Task 11 多笔成交 journal 最小收口

- `security_execution_journal` 已正式落地
- 当前 journal 已不再是备注文本，而是独立正式 Tool
- 当前最小正式成交字段已经固定：
  - `trade_date`
  - `side`
  - `price`
  - `position_pct_delta`
  - `reason`
  - `notes`
- 当前最小聚合字段已经固定：
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
- `security_execution_record` 当前已经升级为复用 `execution_journal`
  - 新增 `execution_journal_ref`
- `security_post_trade_review` 当前已经正式绑定：
  - `execution_journal_ref`
- `security_decision_package` 当前已经正式挂入：
  - `execution_journal_result`
  - `execution_journal`
- `security_decision_verify_package` 当前至少会校验：
  - `execution_journal` 是否挂进 `object_graph`
  - `execution_journal` 是否挂进 `artifact_manifest`
  - `execution_journal` 与 `execution_record / post_trade_review / position_plan / snapshot / outcome` 是否同源
- `security_decision_package_revision` 当前已能对以下问题输出修补建议：
  - `missing_execution_journal`
  - `execution_journal_ref_misaligned`
- 已验证：
  - `cargo test --test security_execution_journal_cli -- --nocapture`
  - `cargo test --test security_execution_record_cli -- --nocapture`
  - `cargo test --test security_post_trade_review_cli -- --nocapture`
  - `cargo test --test security_decision_package_cli -- --nocapture`

这一条意味着当前证券主线已经从“单次执行记录”推进到“多笔成交明细 journal + 聚合 record”双层执行对象。
## 16. 2026-04-09 Task 12 账户级仓位管理最小收口
- `security_portfolio_position_plan` 已正式落地
- 当前它不是复杂组合优化器，而是“账户级增量资金分配 Tool”
- 当前正式输入会同时消费：
  - 账户总资产
  - 可用现金
  - 当前持仓
  - 候选标的及其正式 `security_position_plan`
- 当前正式输出会同时给出：
  - 现金底线占比
  - 可部署现金金额
  - 当前已投资占比
  - 集中度预警
  - 逐标的账户级分配建议
- 当前账户级约束只做最实用的 4 类：
  - `min_cash_reserve_pct`
  - `max_single_position_pct`
  - `max_sector_exposure_pct`
  - `risk_cap_pct`
- 当前候选排序规则是：
  - `confidence + odds_grade - risk_penalty`
- 当前 `risk_cap_pct` 口径固定为：
  - `low => 0.20`
  - `medium => 0.15`
  - `high => 0.10`
  - `default => 0.12`
- 本轮已经修过一个真实 bug：
  - 同一 `symbol` 的现有暴露必须累计，不能覆盖
- 已接入：
  - `src/ops/stock.rs`
  - `src/ops/mod.rs`
  - `src/tools/catalog.rs`
  - `src/tools/dispatcher.rs`
  - `src/tools/dispatcher/stock_ops.rs`
- 已验证：
  - `cargo test --test security_portfolio_position_plan_cli -- --nocapture`

这一条意味着当前 M3 后续增强线已经不再只停留在“单票 position_plan”，而是开始进入“账户层这笔钱该怎么投”的正式对象化阶段。

## 17. 2026-04-09 Task 12 第二轮补齐：账户级风险预算门禁
- `security_portfolio_position_plan` 当前已补入最小账户级风险预算门禁
- 当前新增正式输入：
  - `max_portfolio_risk_budget_pct`
  - `current_portfolio_risk_budget_pct`
  - `max_single_trade_risk_budget_pct`
- 当前新增正式输出：
  - `remaining_portfolio_risk_budget_pct`
  - `estimated_new_risk_budget_pct`
  - `total_portfolio_risk_budget_pct`
  - `risk_budget_warnings`
- 当前逐标的建议已新增：
  - `estimated_risk_budget_pct`
- 当前风险预算口径仍是规则型折算，不是波动率 / 协方差模型
- 当前风险预算折算系数是：
  - `low => 0.25`
  - `medium => 0.50`
  - `high => 0.75`
  - `default => 0.60`
- 当前门禁逻辑是：
  - 候选标的先过现金底线
  - 再过单票上限与行业上限
  - 再过账户总风险预算与单笔风险预算
- 已验证：
  - `cargo fmt --all`
  - `cargo test --test security_portfolio_position_plan_cli -- --nocapture`

这一条意味着账户层现在已经不只是“给建议仓位”，而是开始正式回答“账户风险预算还够不够继续加”。

## 18. 2026-04-09 Task 12 第三轮补齐：仓位分层模板
- `security_position_plan` 当前已正式暴露分层模板，不再只有 `starter_position_pct / max_position_pct`
- 当前新增单票层字段：
  - `entry_tranche_pct`
  - `add_tranche_pct`
  - `reduce_tranche_pct`
  - `max_tranche_count`
  - `tranche_template`
  - `tranche_trigger_rules`
  - `cooldown_rule`
- 当前这些字段直接复用：
  - `briefing_core.position_plan`
  - `briefing_core.execution_plan`
- `security_portfolio_position_plan` 当前已新增账户层分层输出：
  - `suggested_tranche_action`
  - `suggested_tranche_pct`
  - `remaining_tranche_count`
- 当前账户层不仅能说“要不要买”，还能说“当前该走试仓、加仓还是等待”
- 已验证：
  - `cargo fmt --all`
  - `cargo test --test security_position_plan_cli -- --nocapture`
  - `cargo test --test security_portfolio_position_plan_cli -- --nocapture`

这一条意味着方案 A 已经从“账户级风险预算”继续推进到“账户级动作分层”，开始具备更接近实盘执行的话术和对象层。
## 19. 2026-04-09 Task 12 第四轮补齐：账户偏差回写
- `security_execution_record` 当前已可选消费 `portfolio_position_plan_document`
- 当前新增正式账户偏差字段：
  - `portfolio_position_plan_ref`
  - `planned_tranche_action`
  - `planned_tranche_pct`
  - `planned_peak_position_pct`
  - `actual_tranche_action`
  - `actual_tranche_pct`
  - `actual_peak_position_pct`
  - `tranche_count_drift`
  - `account_budget_alignment`
- `security_post_trade_review` 当前已继续输出：
  - `account_plan_alignment`
  - `tranche_discipline`
  - `budget_drift_reason`
  - `next_account_adjustment_hint`
- 当前明确分层职责：
  - `execution_record` 记事实偏差
  - `post_trade_review` 记治理解释
  - `decision_package` 旧链路默认不传账户计划，兼容不破
- 当前已修一个真实口径问题：
  - `tranche_count_drift` 不再按单票 starter 基线反推
  - 改为按“账户层本次建议的 tranche 大小”折算，避免 drift 被单票模板误导
- 已验证：
  - `cargo fmt --all`
  - `cargo test --test security_execution_record_cli -- --nocapture`
  - `cargo test --test security_post_trade_review_cli -- --nocapture`

这一轮意味着 `方案A-2` 已经从“账户层建议能不能给出来”推进到“执行完以后能不能对上账户计划并正式复盘”。
## 20. 2026-04-10 Foundation Metadata Validator 联动收口

<!-- 2026-04-10 CST：补充 foundation metadata validator 联动交接说明。原因：本轮代码已经把 deprecated / aliases / replaced_by 正式接进 validator，但 AI_HANDOFF 仍停留在 migration contract 阶段，容易让后续 AI 误判“还没做”或直接跳去重构。目的：固定当前 foundation 元数据治理做到哪、下一步该做什么、什么暂时不要做。 -->

这一段只在“用户明确要求继续 foundation 主线”时作为最新口径使用；不要把它误读成证券主线已经切换。

- foundation metadata 治理当前最新已完成阶段不是 `Migration Contract`，而是 `Validator Linkage`
- 本轮已经在节点级校验里正式消费：
  - `deprecated`
  - `replaced_by`
  - `aliases`
- 当前 validator 已具备的正式语义：
  - alias 命中 canonical 字段时，会输出 `AliasFieldUsed`
  - 但 required / disallowed / allowed-values / value-type 校验会继续按 canonical 字段语义执行
  - deprecated 字段被使用时，会输出 `DeprecatedFieldUsed`
  - 若 deprecated 字段配置了 `replaced_by`，issue 中会显式带出替代字段
  - 若节点使用的是 deprecated 字段，且其 `replaced_by` 正好是 required 字段，则当前 validator 视为“已满足 required”
- 这一轮明确没有做：
  - repository-level audit
  - migration executor
  - 自动 rewrite
  - foundation 与证券业务层联动

### 20.1 本轮对应文件

- 设计文档：`docs/plans/2026-04-10-foundation-metadata-validator-linkage-design.md`
- 实施计划：`docs/plans/2026-04-10-foundation-metadata-validator-linkage-plan.md`
- 核心实现：
  - `src/ops/foundation/metadata_schema.rs`
  - `src/ops/foundation/metadata_validator.rs`
- 关键测试：`tests/metadata_validator_unit.rs`

### 20.2 已完成验证

- `cargo test --test metadata_validator_unit -- --nocapture`
- `cargo test --test metadata_schema_registry_unit --test metadata_schema_versioning_unit --test metadata_migration_contract_unit --test metadata_validator_unit --test knowledge_repository_unit --test knowledge_ingestion_unit --test knowledge_bundle_unit -- --nocapture`

### 20.3 如果继续 foundation，默认下一步

- 下一步默认进入 `Repository-Level Audit`
- 不要把当前阶段误判成已经进入 `Migration Executor`
- 继续开发前仍然要先走“出方案 -> 用户批准 -> TDD 实现”的固定流程

### 20.4 交接提醒

- foundation 这轮是“在既有 metadata 治理链上继续往前推”，不是重新做架构收口
- 后续 AI 不要因为看到既有 `dead_code` warnings 就回头重做本轮 validator 联动判断
- 只要用户没有明确要求，不要顺手把 repository audit 扩成自动迁移或批量重写

## 21. 2026-04-10 Foundation Repository Metadata Audit 收口

<!-- 2026-04-10 CST：补充 repository-level audit 交接说明。原因：foundation metadata 治理已从 validator linkage 继续推进到仓库级审计，如果不写进 handoff，后续 AI 容易重复判断“下一步还只是候选方案”。目的：固定当前已完成范围、hygiene diagnostics 边界和下一步非目标。 -->

这一段同样只在“用户明确要求继续 foundation 主线”时作为最新口径使用。

- foundation metadata 治理当前最新已完成阶段已经推进到 `Repository-Level Audit`
- 当前 repository audit 已正式具备：
  - 遍历 `KnowledgeRepository` 全节点
  - 复用 `MetadataValidator` 聚合节点级 issue
  - 输出 issue 类型聚合计数
  - 输出 concept 维度聚合计数
  - 输出最小 evidence hygiene diagnostics
- 当前最小 hygiene diagnostics 只做到：
  - `DuplicateEvidenceRef`
  - `WeakLocator`
  - `WeakSourceRef`
- 当前仍然没有做：
  - 自动 rewrite
  - migration executor
  - 更复杂的 locator/source_ref 质量评分
  - foundation 与证券业务层联动

### 21.1 本轮对应文件

- 设计文档：`docs/plans/2026-04-10-foundation-repository-metadata-audit-design.md`
- 实施计划：`docs/plans/2026-04-10-foundation-repository-metadata-audit-plan.md`
- 核心实现：
  - `src/ops/foundation/repository_metadata_audit.rs`
  - `src/ops/foundation.rs`
- 关键测试：`tests/repository_metadata_audit_unit.rs`

### 21.2 已完成验证

- `cargo test --test repository_metadata_audit_unit -- --nocapture`
- `cargo test --test repository_metadata_audit_unit --test metadata_validator_unit --test metadata_schema_registry_unit --test metadata_schema_versioning_unit --test metadata_migration_contract_unit --test knowledge_repository_unit --test knowledge_ingestion_unit --test knowledge_bundle_unit -- --nocapture`

### 21.3 如果继续 foundation，默认下一步

- 下一步优先是“扩细 evidence hygiene diagnostics”
- 或在再次获批后，才讨论 `Migration Executor`
- 不要把当前 repository audit 误判成自动迁移已经开始

## 2026-04-10 Foundation Evidence Hygiene Diagnostics Expansion
- 状态：已完成并通过测试。
- 本轮在 repository audit 基础上新增 4 项 hygiene 子能力：`MissingEvidenceRef`、`DuplicateEvidenceRefWithinNode`、`WeakLocator` 原因分类、`WeakSourceRef` 原因分类。
- 当前 repository-level evidence hygiene 已累计 7 项子能力：`MissingEvidenceRef`、`DuplicateEvidenceRefWithinNode`、`DuplicateEvidenceRef`、`WeakLocator(Blank/TooShort)`、`WeakSourceRef(Blank/TooShort/MissingNamespace)`。
- 当前默认下一步：继续 foundation 主线，优先补更细的 locator/source_ref 结构规则与审计报告分级；不要误判为已经进入 Migration Executor。
## 2026-04-10 条件复核中枢正式收口
<!-- 2026-04-10 CST: 追加条件复核中枢 handoff。原因：Task 1-5 已把 security_condition_review 正式接入 package、execution 和 review 主链，但主 handoff 还没有收进这层最新事实。目的：统一后续 AI 对“投中层做到哪了、怎么验证、接下来先看什么”的口径。 -->

- “投中监控中枢”在当前证券主链里的正式名称改为“条件复核中枢”。
- 当前实现明确不依赖实时行情流，边界固定为四类触发：
  - `manual_review`
  - `end_of_day_review`
  - `event_review`
  - `data_staleness_review`
- 当前正式 Tool 是 `security_condition_review`，已经进入：
  - `src/tools/catalog.rs`
  - `src/tools/dispatcher.rs`
  - `src/tools/dispatcher/stock_ops.rs`
- 当前最小动作分流已经固定：
  - `manual_review -> keep_plan`
  - `end_of_day_review -> update_position_plan`
  - `event_review -> reopen_committee`
  - `data_staleness_review -> reopen_research`
  - 若摘要命中 `冻结 / 停牌 / 止损 / 重大负面`，强制升级为 `freeze_execution`

### 当前正式主链口径

- 当前证券主链已经从“投前决策 -> 执行/复盘”扩成：
  - `投前决策 -> 投中条件复核 -> 执行事实 -> 投后复盘`
- `security_decision_package` 当前已经正式承载：
  - `condition_review_ref`
  - `condition_review_digest`
- `security_decision_verify_package` 当前已经正式校验：
  - 是否存在 `condition_review` 绑定
  - `condition_review_ref` 与 digest 是否一致
  - `condition_review` 是否与 `decision_ref / approval_ref / position_plan_ref / symbol / analysis_date` 同源
- `security_decision_package_revision` 当前已经会继承既有 `condition_review` 锚点，不会在 revision 时把这层绑定丢掉。
- `security_execution_record` 当前已经支持可选挂接：
  - `condition_review_ref`
  - `condition_review_trigger_type`
  - `condition_review_follow_up_action`
  - `condition_review_summary`
- `security_post_trade_review` 当前已经能继续读取最近一次条件复核，并输出：
  - `condition_review_ref`
  - `condition_review_trigger_type`
  - `condition_review_follow_up_action`
  - `condition_review_summary`
  - `condition_review_interpretation`

### 当前验证清单

- 当前证券主链默认最小验证清单应更新为 8 条，而不是旧文档里残留的 `security_decision_package_cli`：
  1. `cargo test --test security_committee_vote_cli -- --nocapture`
  2. `cargo test --test security_condition_review_cli -- --nocapture`
  3. `cargo test --test security_decision_verify_package_cli -- --nocapture`
  4. `cargo test --test security_decision_package_revision_cli -- --nocapture`
  5. `cargo test --test security_execution_record_cli -- --nocapture`
  6. `cargo test --test security_post_trade_review_cli -- --nocapture`
  7. `cargo test --test security_scorecard_training_cli -- --nocapture`
  8. `cargo test --tests --no-run`
- 这 8 条的默认解释是：
  - 1 检查投决会与七席执行链没有断。
  - 2 检查投中条件复核层已经是正式 CLI Tool。
  - 3 和 4 检查 package 治理链、绑定校验与 revision 修补链没有断。
  - 5 和 6 检查执行/复盘层已经吸收条件复核。
  - 7 检查评分卡训练主链没有断。
  - 8 检查全量测试目标至少还能成功编译。

### 已知边界与后续默认顺序

- 当前 `condition_review_id` 仍采用 `symbol + analysis_date + trigger_type` 稳定规则；如果未来同日同类触发需要多次复核，后续要补版本号或序号策略。
- 当前 execution/review 层优先支持“显式 ref + 可选 document”输入；还没有实现“只给 ref 就自动回仓储反查复核文档”。
- 2026-04-10 这轮 fresh 验证里，`security_scorecard_training_cli` 仍然失败；这不是条件复核中枢本轮引入的文档问题，但它意味着当前“默认最小验证清单”还不是全绿状态。
- 当前不建议把这层误读成实时监控系统；它是“条件复核中枢”，不是秒级监控中台。
- 后续如果继续往下做，默认优先顺序应是：
  1. 按 ref 回读条件复核历史文档
  2. 为同日多次复核补版本策略
  3. 再考虑把条件复核结果进一步装订进更完整的 decision package 审计资产
