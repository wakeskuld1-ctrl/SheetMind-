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
