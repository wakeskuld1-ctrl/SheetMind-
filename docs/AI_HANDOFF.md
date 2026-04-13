# AI 交接手册

<!-- 2026-04-09 CST: 重写统一 AI 交接手册。原因：当前本地主文档已混入过期主线与乱码内容，而远端 foundation-navigation-kernel 又补进了新的证券治理与评分卡进展。目的：把“当前该看哪条主线、做到哪了、接下来该从哪里继续”统一收口到一份主文档里。 -->

## 0. 依赖图开发门禁

<!-- 2026-04-13 CST: 新增依赖图开发门禁。原因：用户明确要求把“无图禁止开发、开发完成后必须回写图谱”的约束写入正式交接手册，作为所有后续开发的统一标准。目的：让后续 AI / 开发者先按图确认主线与上下游，再进入源码实现，并在开发后保持账实一致。 -->

- 从 2026-04-13 这轮开始，依赖图与图谱说明是正式开发前置条件，不再只是辅助材料。
- 没有最新可用的图谱产物时，不应直接进入开发实现；至少要先确认现有图谱是否覆盖当前改动范围。
- 依赖图主要用于判断三件事：
  1. 当前改动位于哪条主线。
  2. 当前改动位于上游入口层、中游编排层还是下游基础治理层。
  3. 当前改动是否误入业务化支线。
- 图谱不能替代源码、合同、编译和测试验证。原因是图谱里仍可能包含 inferred 关系，它更适合看结构，不适合直接当精确调用链。
- 每次开发完成后，必须同步更新图谱产物或图谱说明；如果主线入口、核心路由、模块归属或上下游关系发生变化，必须优先补图再继续下一轮开发。
- 如果图谱与代码现实不一致，默认先修正图谱或在交接文档中明确差异，再继续后续开发。
- 当前仓库的正式图谱门禁命令是：
  - 开发前：`powershell -ExecutionPolicy Bypass -File .\scripts\check_graph_maps.ps1`
  - 开发后：`powershell -ExecutionPolicy Bypass -File .\scripts\update_graph_maps.ps1`

## 1. 当前仓库怎么理解

这个仓库当前有两条并行能力线：

- 证券治理与证券分析主线
- foundation 通用导航内核线

但从当前实际开发连续性、用户需求和后续接续价值看，默认优先级应是：

1. 证券治理与证券分析主线
2. foundation 通用导航内核线

除非用户明确要求继续 foundation，否则后续默认先沿证券主线推进。

## 1.1 2026-04-13 设计骨架前置共识

<!-- 2026-04-13 CST: 新增设计骨架前置共识。原因：用户要求新功能和重大重构先产出类似 graphify 的设计骨架，再顺着骨架开发，并在开发后做设计与成品的差距对比。目的：减少后续 AI 每次重新澄清“先设计还是先写代码”的成本。 -->

- 从 2026-04-13 这轮开始，新功能、重大重构、跨层路由改动默认先做设计骨架，不再允许直接跳进实现。
- 当前正式 Tool 入口已经扩成两条：
  - `foundation_design_skeleton`
  - `foundation_design_gap_audit`
- 两者和 `graphify` 的分工固定为：
  - `graphify`：现状实现 JSON 图谱，`graph.json` 是事实源
  - `foundation_design_skeleton`：目标设计 JSON 骨架
  - `foundation_design_gap_audit`：设计 vs 成品差距 JSON 审计
- 2026-04-13 本轮收敛补充：
  - foundation design tool 的主边界是 JSON，不是 HTML，也不是 Mermaid。
  - `graphify` 联动时默认直接读 `graph.json` 做对账，不把图渲染当主入口。
  - Mermaid 只作为 `foundation_design_skeleton` 的 `visuals` 辅助块保留，方便人工扫边界。
- 设计骨架的最小必备字段固定为：
  - `feature_name`
  - `objective`
  - `success_criteria`
  - `layers`
  - `modules`
  - `interfaces`
  - `methods`
  - `test_scenarios`
- 实现完成后，不仅要更新 `src-map-*` 图谱，还要补一次差距审计或等价对比，确认账实一致。

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

从 2026-04-13 这轮开始，foundation 已经新增一个正式 Tool 入口：

- `foundation_navigation`

当前它的边界是：

- 输入：`question + knowledge_bundle + allowed_relation_types + max_depth + max_concepts`
- 主链：`capability_router -> roaming_engine -> retrieval_engine -> evidence_assembler`
- 输出：`matched_concept_ids / roaming_path / hits / citations / summary`

当前它仍然是“通用内存知识包上的标准导航入口”，还没有继续扩成：

- repository layout 自动装载导航
- 跨仓库批量漫游
- 上层业务化编排
- 自动建体系或自动补知识

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

### 4.1 当前已经落地的标准化门禁

从 2026-04-09 这轮开始，日期口径不再只是文档约定，已经正式下沉到 Tool 主链：

1. 先查本地 SQLite 历史数据
2. 本地不够时，再走项目内已接入的免费同步 Tool 补数
3. 请求日若仍无有效收盘，只允许回退到最近一个有效交易日
4. 输出必须显式披露“请求日期 / 实际分析日期 / 是否同步 / 为什么回退”

当前已明确接入 `analysis_date_guard` 或等价日期门禁信息的主链合同包括：

- `technical_consultation_basic`
- `security_analysis_contextual`
- `security_analysis_fullstack`
- `security_decision_briefing`
- `security_position_plan`

标准字段口径如下：

- `requested_as_of_date`：用户请求分析的日期
- `effective_analysis_date`：本次真正用于分析的日期
- `effective_trade_date`：本次真正命中的有效交易日
- `local_data_last_date`：同步前本地库里最后一条日期
- `data_freshness_status`：本次命中本地、同步补足还是回退后的状态说明
- `sync_attempted`：是否做过同步尝试
- `sync_result`：同步结果摘要
- `date_fallback_reason`：为什么没有沿用请求日期

后续所有 Skill / Tool / 报告解释，都应优先使用这些字段，不要再靠模型自己“记得应该回退到上一个交易日”。

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
## 9.1 本轮新增记忆点

- 用户明确纠正过：不能只在对话里“记住”日期回退规则，必须写进项目 Skill、Tool 和上层合同。
- 证券分析的标准顺序是“先本地，再同步，再回退，再显式披露”，不是直接拿过期缓存或手工临时解释。

## 9.2 2026-04-13 模型升级路线共识

<!-- 2026-04-13 CST: 这里补模型升级路线共识。原因：用户明确要求把“暂不上神经网络、优先补数据/标签/验证、下一阶段优先树模型”写进 AI 手册，避免后续 AI 再从黑盒模型起步。目的：统一证券主线后续的预测可靠性提升顺序。 -->

- 当前不要把“神经网络”当成证券主线的下一默认动作。
- 当前训练主链仍以“feature_snapshot -> forward_outcome -> 分箱 -> WOE -> logistic -> artifact/refit/registry”为正式基线。
- 如果目标是提高未来预测可靠性，默认优先级应是：
  1. 先补数据与验证体系
  2. 再做股票/ETF 分模
  3. 再补特征与标签定义
  4. 再引入树模型 challenger
  5. 最后才重新评估是否需要神经网络

当前这里说的“先补数据与验证体系”，最低限度包括：

- ETF 专项特征补齐
- ETF 专项标签定义补齐
- ETF 历史研究与外部代理数据补齐
- walk-forward / out-of-time 验证补齐
- champion-challenger 比较链补齐

当前这里说的“树模型 challenger”，优先考虑：

- GBDT / XGBoost / LightGBM / CatBoost 这类结构化表格模型

不要默认直接跳到：

- MLP
- LSTM
- Transformer
- 其他黑盒时序模型

原因统一口径如下：

- 当前瓶颈主要不是模型容量不够，而是特征、标签、分资产建模和验证体系还不够完整。
- 证券主链当前需要可解释、可治理、可复盘的正式对象链，黑盒模型过早接入会抬高投决会解释成本和训练治理成本。
- 在当前样本厚度与数据准备度下，树模型通常比神经网络更稳、更容易形成 challenger 体系。

如果后续 AI 要继续推进“让 ETF 达到股票这一级”，默认应先补以下 4 类硬能力，而不是直接换更复杂模型：

1. ETF 专项特征
2. ETF 专项标签
3. ETF 专项模型
4. ETF 专项验证

## 9.3 2026-04-13 主席裁决协议共识

<!-- 2026-04-13 CST: 这里补主席裁决协议共识。原因：用户确认当前体系是“议会制投决会 + 大模型独立建议 + 数据建议”的二元并行结构，并要求主席层具备正式仲裁与退回补件能力。目的：避免后续 AI 把主席层做成重复议会或自由发挥的大模型总结。 -->

- 当前推荐结构不是“主席团”，而是“议会制投决会之后的单主席仲裁层”。
- 主席层默认不重做投决会，不重新创造事实，而是消费：
  - 议会制结论
  - 数据建议
  - 大模型独立建议
  - 风险/日期/证据质量门禁
- 主席层的硬约束必须放 Tool，不要只写在 Skill。
- Skill 只负责编排与解释，Tool 负责动作枚举、退回补件、冲突等级、禁止行为和越权门禁。
- 主席层必须拥有一项正式权力：
  - `return_for_revision`
  - 即禁止当前审批，退回指定环节补充资料，再次提交
- 主席层默认动作至少包括：
  - `approve`
  - `conditional_approve`
  - `reject`
  - `return_for_revision`
  - `defer`
- 当前避免主席幻觉的统一口径不是“换更强模型”，而是：
  - 限定输入
  - 禁止创造新事实
  - 强制证据映射
  - 允许拒判/退回补件
  - 不得越过风险与日期等硬门禁
- 详细协议见：
  - `docs/plans/2026-04-13-chair-arbitration-protocol.md`

## 9.4 2026-04-13 主席链测试收口规则

<!-- 2026-04-13 CST: 这里补主席链测试收口规则。原因：当前“大模型独立建议线”正式 Tool 合同已经落地，但旧 security_chair_resolution_cli 重链路在 Windows 上仍可能挂住并残留测试进程。目的：给后续 AI 一个稳定、低风险的主席链验证入口，避免继续把旧重链路当成唯一门禁。 -->

- 已正式落地的对象与入口包括：
  - `src/ops/security_independent_advice.rs`
  - `src/ops/security_chair_resolution.rs`
  - `src/ops/security_record_post_meeting_conclusion.rs`
  - `tests/security_independent_advice_cli.rs`
- 当前不要把“独立建议线未落地”和“旧主席 CLI 测试会挂住”混为一谈。
- 当前真正未完全收口的点，是旧 `tests/security_chair_resolution_cli.rs` 仍属于高风险重链路：
  - 会走 runtime
  - 会走 HTTP 假服务
  - 会触发 Windows 下测试进程残留
  - 会进一步引发链接占用与 Cargo 锁竞争
- 因此，当前主席链默认验证规则应改成：
  - 先跑新的纯 builder 测试文件：`tests/security_chair_resolution_builder_unit.rs`
  - 不把旧 `security_chair_resolution_cli.rs` 当成当前轮必须先打通的唯一门禁
  - 旧 CLI 链后续如要继续修，只能单独做风险隔离，不要顺手混进主开发任务

当前 builder 测试已锁住的最小主席分支：

- `scorecard_not_ready -> return_for_revision`
- `committee_abstain + no_independent_advice -> defer`
- `independent_advice_conflict -> return_for_revision`
- `post_meeting_conclusion` 与主席流程字段一致性

当前推荐的主席链验证命令：

- `cargo test --test security_independent_advice_cli -- --nocapture --test-threads=1`
- `cargo test --test security_chair_resolution_builder_unit -- --nocapture --test-threads=1`

当前不建议默认执行的高风险验证命令：

- `cargo test --test security_chair_resolution_cli -- --nocapture`

如果下一位 AI 继续沿主席链推进，默认规则应是：

- 先看 `security_independent_advice`、`security_chair_resolution`、`security_record_post_meeting_conclusion` 这三份正式对象
- 先改 builder 层，再决定是否回头拆旧 CLI 链
- 先定向单测，再决定是否扩大回归范围

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

## 15. 2026-04-10 账户层连续状态最小闭环补齐

- 上一轮已完成“未平仓正式快照”，这一轮继续把它正式接进账户层，而不是停留在手工补 `holdings`。
- 当前新增的最小正式合同包括：
  - `security_execution_record.current_position_pct`
  - `security_portfolio_position_plan.open_position_snapshots`
- 当前统一口径已经明确：
  - `actual_position_pct` 表示本批执行期间达到的峰值仓位
  - `current_position_pct` 表示当前仍在持有的剩余仓位
  - `portfolio_position_plan` 只消费 `position_state = open` 且 `current_position_pct > 0` 的快照
  - 账户层按 `total_equity * current_position_pct` 折算成 holdings 暴露
- 这意味着当前最小连续链路已经变成：
  - `execution_journal(open)` -> `execution_record(current_position_pct)` -> `portfolio_position_plan(open_position_snapshots)` -> 下一轮账户建议
- 当前仍然没有做的事：
  - 完整持仓台账
  - FIFO/LIFO/税务口径
  - runtime 自动持仓回写
  - 跨周期真实账本
- 本轮验证：
  - `cargo fmt --all`
  - `cargo test --test security_execution_record_cli -- --nocapture`
  - `cargo test --test security_portfolio_position_plan_cli -- --nocapture`
  - `cargo test --test security_post_trade_review_cli security_post_trade_review_marks_open_position_as_pending_closeout -- --nocapture`

## 16. 2026-04-10 方案B收口：runtime 自动回接上一轮 open execution_record

- 当前 `security_execution_record` 已新增最小账户绑定字段：
  - `account_id`
  - `sector_tag`
- 当前 `security_execution_record` 已不再只是响应对象，也会正式落盘到执行层 runtime：
  - `security_execution_records`
- 当前已新增独立账户状态对象：
  - `security_account_open_position_snapshot`
- 当前这条对象链已经变成：
  - `execution_record(account_id/current_position_pct)` -> runtime `security_execution_records`
  - `security_account_open_position_snapshot(account_id)` 自动读取 runtime open 记录
  - `security_portfolio_position_plan.account_open_position_snapshot_document` 直接消费标准 snapshot 文档
- 当前 runtime 自动读取规则已经明确：
  - 只读取同一 `account_id`
  - 只读取 `position_state = open`
  - 同一 `symbol` 只取最新一条 execution_record
- 这意味着当前账户层连续状态已经从“手工传裸 snapshot”推进到“运行时自动回接上一轮 open 持仓”。
- 仍未完成的边界：
  - 还没有完整持仓账本
  - 还没有逐笔成本层/FIFO/LIFO
  - 还没有按行情重估实时市值
  - 还没有自动账户对账
- 本轮验证：
  - `cargo fmt --all`
  - `cargo test --test security_execution_record_cli -- --nocapture`
  - `cargo test --test security_account_open_position_snapshot_cli -- --nocapture`
  - `cargo test --test security_portfolio_position_plan_cli -- --nocapture`
  - `cargo test --test security_post_trade_review_cli -- --nocapture`

## 14. 2026-04-10 账户层连续状态最小补齐

- `security_execution_journal` 已从“只支持已平仓闭环”补到“支持未平仓阶段快照”
- 当前新增正式状态字段：
  - `security_execution_journal.position_state`
  - `security_execution_record.position_state`
  - `security_post_trade_review.review_status`
- 当前最小口径如下：
  - `final_position_pct > 0` 时视为 `open`
  - 未平仓阶段允许 `execution_journal` 正式落对象
  - 未平仓阶段 `execution_record.actual_exit_date = ""`
  - 未平仓阶段 `execution_record.actual_exit_price = 0.0`
  - 未平仓阶段 `execution_record.exit_reason = position_still_open`
  - 未平仓阶段 `execution_quality = open_position_pending`
  - 未平仓阶段 `post_trade_review.review_status = open_position_pending`
- 这意味着当前证券主链已经不再只适合“完整买入卖出后再复盘”的场景，至少可以先承接：
  - 分批建仓后仍在持仓
  - 尚未触发退出条件
  - 需要进入下一轮账户计划前先记录阶段状态
- 仍未完成的边界：
  - 还没有完整持仓台账或跨周期账户账本
  - 还没有把未平仓快照自动回写到 runtime 持仓状态
  - 还没有做 FIFO/LIFO、税务口径和跨周期真实成本层归集
- 已验证：
  - `cargo test --test security_execution_journal_cli -- --nocapture`
  - `cargo test --test security_execution_record_cli -- --nocapture`
  - `cargo test --test security_post_trade_review_cli -- --nocapture`
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
## 20. 2026-04-10 Task 12 方案B验证收口
- 当前应把 `Task 12 / 方案B` 视为“已实现并已验证”，而不是继续当成待开发项。
- 当前已经跑通的正式链路是：
  - `security_execution_record`
  - `runtime.security_execution_records`
  - `security_account_open_position_snapshot`
  - `security_portfolio_position_plan`
- 当前自动回接规则已经明确：
  - 先由 `execution_record` 把账户维度与 `position_state / current_position_pct` 写入 runtime
  - 再由 `security_account_open_position_snapshot` 按 `account_id` 读取 `open` 状态记录
  - 同一 `symbol` 只保留最新一条 `open execution_record`
  - 再把 snapshot 文档喂给 `security_portfolio_position_plan`
- 本轮复核确认：
  - 不再需要把上一轮 open 持仓手工改写成裸 `holdings`
  - 也不应再把 `方案B` 理解为“还没做完的设计稿”
  - 后续若继续推进，应从 `Task 12` 之后的账户账本/市值重估/更细粒度子账户维度往下做，而不是重复重做 runtime 回接
- 2026-04-10 本轮再次验证通过：
  - `cargo test --test security_account_open_position_snapshot_cli -- --nocapture`
  - `cargo test --test security_portfolio_position_plan_cli -- --nocapture`
  - `cargo test --test security_execution_record_cli -- --nocapture`

这一条的意义是：后续 AI 接手时，默认前提应改成“Task 12 方案B已收口，可直接在此之上继续”，不要再回退到手工 `open_position_snapshots` 传参方案。
## 2026-04-10 foundation metadata audit 收口补充

如果用户明确要求继续 foundation 通用能力，不要直接跳回证券业务链，也不要从 `metadata migration contract` 直接跨到自动迁移执行器。

当前 foundation 元数据治理主线已经收口为：

1. `metadata schema registry`
2. `metadata validator`
3. `metadata schema versioning`
4. `metadata migration contract`
5. `repository metadata audit`

其中第 5 步已在当前工作区正式落地，入口位于：

- `src/ops/foundation/knowledge_repository.rs`
- `tests/knowledge_repository_unit.rs`

当前 `repository metadata audit` 第一刀只做“扫描 + 报告”，明确覆盖：

- `repository schema_version` 与 `metadata schema_version` 的兼容性审计
- 节点 metadata 中 `deprecated` 字段使用痕迹审计
- 节点 metadata 中 `alias` 字段使用痕迹审计
- 节点级 `MetadataValidator` 问题的整库汇总

当前还没有做的事：

- 自动重写 alias / deprecated 字段
- 批量 migration planner
- repository 级审计结果持久化或外部报告导出

后续如果继续 foundation，优先顺序应是：

1. 先补 repository audit 的消费层或批量审计入口
2. 再做最小 migration executor
3. 最后再扩更复杂的自动修复编排
## 2026-04-10 foundation repository metadata audit Tool 鍖栨敹鍙?

- 褰撳墠 `repository metadata audit` 宸蹭笉鍐嶅彧鏄?Rust 鍐呴儴 API锛屽凡姝ｅ紡鏀跺彛涓?foundation Tool锛?
  - `foundation_repository_metadata_audit`
- Tool 鍏ュ彛浣嶇疆锛?
  - `src/tools/catalog.rs`
  - `src/tools/contracts.rs`
  - `src/tools/dispatcher.rs`
  - `src/tools/dispatcher/foundation_ops.rs`
- 褰撳墠 Tool 鐨勬渶灏忔爣鍑嗚緭鍏ュ彧鏈変袱涓細
  - `repository_layout_dir`
  - `metadata_schema`
- 鍏朵腑 `metadata_schema` 褰撳墠鏄０鏄庡紡 contract锛屽寘鍚細
  - `schema_version`
  - `fields`
  - `concept_policies`
- 褰撳墠 Tool 鐨勬爣鍑嗚緭鍑哄凡鍥哄畾涓猴細
  - `repository_layout_dir`
  - `repository_schema_version`
  - `metadata_schema_version`
  - `issue_count`
  - `is_clean`
  - `issues`
- 褰撳墠 `issues[*].kind` 宸叉敹鍙ｅ埌鍙洿鎺ユ秷璐圭殑鎵佸钩缁撴瀯锛屽凡瑕嗙洊锛?
  - `incompatible_metadata_schema_version`
  - `unknown_metadata_field`
  - `deprecated_field_usage`
  - `alias_field_usage`
  - `missing_concept_policy`
  - `missing_required_field`
  - `disallowed_field`
  - `invalid_allowed_value`
  - `invalid_value_type`
- 褰撳墠 Tool 鏄庣‘涓嶅仛鐨勪簨锛?
  - 涓嶅仛 audit 鎶ュ憡鎸佷箙鍖栨垨澶栭儴瀵煎嚭
  - 涓嶅仛 alias / deprecated 瀛楁鑷姩閲嶅啓
  - 涓嶅仛 migration executor
  - 涓嶅仛涓氬姟鍖栬瘉鍒搁摼鐗规畩閫昏緫
- 褰撳墠宸查獙璇佺殑瀹氬悜鍥炲綊锛?
  - `cargo fmt --all`
  - `cargo test --test foundation_repository_metadata_audit_cli -- --nocapture`
  - `cargo test --test integration_tool_contract foundation_repository_metadata_audit -- --nocapture`
- 褰撳墠鎺ㄨ崘鐨勪笅涓€姝ラ『搴忔洿鏂颁负锛?
  1. 鍏堝仛 repository audit 鐨勬秷璐瑰眰锛堟壒閲忓叆鍙ｃ€乧heck gate 鎴栧鍏ユ祦绋嬫帴绾匡級
  2. 鍐嶅仛鏈€灏?migration executor
  3. 鏈€鍚庡啀璇勪及鏄惁闇€瑕?report export / planner / auto-fix
- 褰撳墠濡傛灉 AI 瑕佺户缁?foundation 涓荤嚎锛岄粯璁ゅ墠鎻愬簲鏄細
  - `foundation_repository_metadata_audit` 宸插彲琚?CLI / Skill 姝ｅ紡璋冪敤
  - 涓嬩竴姝ヤ笉鏄噸澶嶅仛 tool 鍖栵紝鑰屾槸寰€鈥滄秷璐瑰眰 / 鎵归噺鍏ュ彛鈥濇帹杩涖€?
## 2026-04-10 foundation repository metadata audit gate 收口补充

如果后续 AI 继续沿 foundation 通用能力推进，不要再把这一阶段理解成“只剩 repository audit 报告工具”。当前已经进入 audit 的消费层阶段，并且范围仍然限定在通用 metadata 治理，不做任何证券业务化规则。

本轮新增的正式 foundation Tool：

- `foundation_repository_metadata_audit_gate`

当前 gate 已正式接入以下位置：

- `src/tools/catalog.rs`
- `src/tools/contracts.rs`
- `src/tools/dispatcher.rs`
- `src/tools/dispatcher/foundation_ops.rs`

当前 gate 的最小能力边界：

- 复用 `foundation_repository_metadata_audit` 的同一条 repository 装载、schema 构建和 issue 映射逻辑
- 只做 `gate_passed + blocking/non_blocking issues` 的流程判断
- 不做报告导出
- 不做自动迁移
- 不做业务化策略扩展

当前已锁定的分级规则：

- `non_blocking`
- `alias_field_usage`
- `deprecated_field_usage`
- `blocking`
- `incompatible_metadata_schema_version`
- `unknown_metadata_field`
- `missing_concept_policy`
- `missing_required_field`
- `disallowed_field`
- `invalid_allowed_value`
- `invalid_value_type`

当前 gate 输出的标准字段：

- `repository_layout_dir`
- `repository_schema_version`
- `metadata_schema_version`
- `gate_passed`
- `blocking_issue_count`
- `non_blocking_issue_count`
- `blocking_issues`
- `non_blocking_issues`

本轮已验证通过：

- `cargo fmt --all`
- `cargo test --test foundation_repository_metadata_audit_gate_cli -- --nocapture`
- `cargo test --test foundation_repository_metadata_audit_cli -- --nocapture`
- `cargo test --test integration_tool_contract foundation_repository_metadata_audit -- --nocapture`

如果下一位 AI 继续沿这条主线推进，默认前提应改成：

- `foundation_repository_metadata_audit` 已是正式报告 Tool
- `foundation_repository_metadata_audit_gate` 已是正式消费层 Tool
- 下一步不该再重复做 gate tool 化，而应继续往批量入口、导入链接入或 migration executor 推进

## 2026-04-10 foundation repository metadata audit batch 收口补充

如果后续 AI 继续沿 foundation 通用能力推进，不要把当前阶段重新理解成“还只有单仓库入口”。`方案A / A1` 已经正式落地，当前已经具备批量 repository metadata audit 入口，但范围仍然严格限定在通用标准能力，不做证券业务化口径。

本轮新增的正式 foundation Tool：

- `foundation_repository_metadata_audit_batch`

当前 batch 已正式接入以下位置：

- `src/tools/catalog.rs`
- `src/tools/contracts.rs`
- `src/tools/dispatcher.rs`
- `src/tools/dispatcher/foundation_ops.rs`
- `tests/foundation_repository_metadata_audit_batch_cli.rs`
- `tests/integration_tool_contract.rs`

当前 batch 的最小能力边界：

- 输入固定为 `repository_layout_dirs + 一份共用 metadata_schema`
- 逐仓库复用 `foundation_repository_metadata_audit_gate` 的同一套分级语义
- 输出固定为“批次摘要 + repositories[*] 逐仓库 gate 结果”
- 不做每仓库独立 schema
- 不做并发调度
- 不做复杂容错与重试策略
- 不做报告落盘

当前 batch 批次摘要字段：

- `total_repository_count`
- `passed_repository_count`
- `failed_repository_count`
- `blocking_issue_count_total`
- `non_blocking_issue_count_total`
- `repositories`

当前 A1 已锁定的批量样例口径：

- `legacy_metadata_bundle`：只触发 `alias_field_usage + deprecated_field_usage`，因此 `gate_passed = true`
- `missing_required_field_bundle`：触发 `missing_required_field`，因此 `gate_passed = false`
- 批次总预期：
  - `total_repository_count = 2`
  - `passed_repository_count = 1`
  - `failed_repository_count = 1`
  - `blocking_issue_count_total = 1`
  - `non_blocking_issue_count_total = 2`

本轮已验证通过：

- `cargo fmt --all --check`
- `cargo test --test foundation_repository_metadata_audit_batch_cli -- --nocapture`
- `cargo test --test foundation_repository_metadata_audit_gate_cli -- --nocapture`
- `cargo test --test foundation_repository_metadata_audit_cli -- --nocapture`
- `cargo test --test integration_tool_contract foundation_repository_metadata_audit -- --nocapture`

如果下一位 AI 继续沿这条主线推进，默认前提应改成：

- `foundation_repository_metadata_audit_batch` 已是正式批量入口
- A1 已经完成，下一步不该重复造 batch 壳子
- 后续优先应进入方案B的“导入链接入层/批量消费层”，而不是先扩成每仓库独立 schema 或业务化策略

## 2026-04-10 foundation repository import gate 收口补充

如果后续 AI 继续沿 foundation 通用能力推进，不要再把方案B1理解成“还停留在批量 audit 原始结果阶段”。当前已经正式具备导入接入 gate，范围仍然严格限定在通用 metadata 治理消费层，不做知识漫游业务编排，也不做自动修复。

本轮新增的正式 foundation Tool：

- `foundation_repository_import_gate`

当前 import gate 已正式接入以下位置：

- `src/tools/catalog.rs`
- `src/tools/contracts.rs`
- `src/tools/dispatcher.rs`
- `src/tools/dispatcher/foundation_ops.rs`
- `tests/foundation_repository_import_gate_cli.rs`
- `tests/integration_tool_contract.rs`

当前 import gate 的最小能力边界：

- 输入继续复用 `repository_layout_dirs + 一份共用 metadata_schema`
- 底层复用 `foundation_repository_metadata_audit_batch` 与单仓库 `gate` 的同一套分级语义
- 输出明确收口为：
  - `next_stage_allowed`
  - `all_repositories_accepted`
  - `accepted_repository_count`
  - `rejected_repository_count`
  - `blocking_issue_count_total`
  - `non_blocking_issue_count_total`
  - `blocking_issue_kind_summary`
  - `accepted_repositories`
  - `rejected_repositories`
- `next_stage_allowed` 当前语义已锁定为：至少有一个仓库可继续进入下一阶段
- 不做导入批次持久化
- 不做自动 remediation 建议
- 不做知识漫游编排
- 不做业务化优先级策略

当前 B1 已锁定的关键语义：

- 仅含 `alias / deprecated` 问题的仓库进入 `accepted_repositories`
- 含 blocking 问题的仓库进入 `rejected_repositories`
- `blocking_issue_kind_summary` 对 rejected 仓库的阻塞问题做去重汇总
- 当整批全部 rejected 时，`next_stage_allowed = false`
- 当存在至少一个 accepted 仓库时，`next_stage_allowed = true`

本轮已验证通过：

- `cargo fmt --all --check -- src/tools/contracts.rs src/tools/catalog.rs src/tools/dispatcher.rs src/tools/dispatcher/foundation_ops.rs tests/foundation_repository_import_gate_cli.rs tests/integration_tool_contract.rs`
- `cargo test --test foundation_repository_import_gate_cli -- --nocapture`
- `cargo test --test integration_tool_contract foundation_repository_import_gate -- --nocapture`
- `cargo test --test foundation_repository_metadata_audit_batch_cli -- --nocapture`
- `cargo test --test foundation_repository_metadata_audit_gate_cli -- --nocapture`
- `cargo test --test foundation_repository_metadata_audit_cli -- --nocapture`

如果下一位 AI 继续沿这条主线推进，默认前提应改成：

- `foundation_repository_import_gate` 已是正式导入接入层入口
- 方案B1已经完成，不该重复再造“accepted / rejected 列表解释层”
- 后续优先应进入方案B2或更上层的导入批次对象化，而不是回头重写 batch/gate 语义
## 2026-04-13 foundation repository metadata audit export 鏀跺彛琛ュ厖

濡傛灉鍚庣画 AI 缁х画娌?foundation 閫氱敤鑳藉姏鎺ㄨ繘锛屼笉瑕佹妸褰撳墠杩欒疆鐞嗚В鎴愨€滄妸 existing audit 鏀规垚 file mode鈥濄€傝繖涓€杞柊澧炵殑鏄竴涓?parallel public boundary锛屼笓闂ㄦ壙鎺モ€淔ile input + stable DTO output鈥濓紝鐢ㄤ簬鎺ュ叆涓嶅悓鏁版嵁婧愭垨涓婃父浜х墿钀藉嚭鐨勬爣鍑嗘枃浠躲€?

鏈疆鏂板鐨勬寮?foundation Tool锛?
- `foundation_repository_metadata_audit_export`

褰撳墠 export tool 宸叉寮忔帴鍏ヤ互涓嬩綅缃細

- `src/tools/contracts.rs`
- `src/tools/catalog.rs`
- `src/tools/dispatcher.rs`
- `src/tools/dispatcher/foundation_ops.rs`
- `tests/foundation_repository_metadata_audit_export_cli.rs`

褰撳墠 export tool 鐨勬渶灏忚兘鍔涜竟鐣岋細

- 杈撳叆鍥哄畾涓?`schema_path + bundle_path`
- `schema_path` 鍙敮鎸?metadata schema JSON 鏂囦欢
- `bundle_path` 鍙敮鎸?`.json` 涓?`.jsonl`
- dispatcher 灞傚厛鍋?`trim + fail-fast`
- 搴曞眰澶嶇敤 existing `knowledge_ingestion` 涓?`KnowledgeRepository::audit_metadata(...)`
- 杈撳嚭鏄ǔ瀹?DTO锛屼笉鐩存帴鏆撮湶鍐呴儴 `MetadataRepositoryAuditReport`
- 涓嶆浛鎹?`foundation_repository_metadata_audit`
- 涓嶅紩鍏?navigation evidence export`
- 涓嶅仛 remediation 寤鸿鎴?migration executor
- 涓嶅仛璇佸埜涓氬姟鍖栬兘鍔?

褰撳墠 export 杈撳嚭鐨勬爣鍑嗗瓧娈碉細

- `schema_path`
- `bundle_path`
- `bundle_format`
- `repository_schema_version`
- `metadata_schema_version`
- `issue_count`
- `is_clean`
- `issues`

鏈疆宸查獙璇侀€氳繃锛?
- `cargo fmt --all`
- `cargo test --test foundation_repository_metadata_audit_export_cli -- --nocapture`
- `cargo test --test foundation_repository_metadata_audit_cli -- --nocapture`
- `cargo test --test foundation_repository_metadata_audit_gate_cli -- --nocapture`
- `cargo test --test foundation_repository_metadata_audit_batch_cli -- --nocapture`
- `cargo test --test foundation_repository_import_gate_cli -- --nocapture`

濡傛灉涓嬩竴浣?AI 缁х画娌胯繖鏉′富绾挎帹杩涳紝榛樿鍓嶆彁搴旀敼鎴愶細

- `foundation_repository_metadata_audit_export` 宸叉槸姝ｅ紡鏂囦欢杈圭晫 Tool
- 瀹冧笌 `foundation_repository_metadata_audit` 鏄苟鍒楀叆鍙ｏ紝涓嶆槸鏇挎崲鍏ュ彛
- 涓嬩竴姝ヤ紭鍏堝簲鏄ˉ integration contract 鎴栦笂灞傛帴鍏ヨ矾鐢憋紝鑰屼笉鏄妸 export tool 鎵╂垚涓氬姟鍖栫紪鎺掋€?
## 2026-04-13 实盘可用标准补充

如果后续 AI 继续沿证券主链推进，不要再把“训练链能跑通”误判成“已经接近实盘可用”。

当前统一判断是：

- 证券主链、ETF 最小正式接入、评分卡训练入口、投决会、主席链、投中 / 投后基础对象都已经具备。
- 但当前阶段仍更接近“研究与治理底盘”，不是“可稳定辅助真金白银决策的实盘系统”。

当前离实盘可用最近的 4 个硬门槛是：

1. 样本厚度不够
- 当前真实训练已经能跑，但样本量和覆盖市场状态还不够，不能证明模型泛化能力。

2. 标签体系不够像实盘
- 不能只停留在单一方向标签。
- 需要逐步补齐 `1/5/10/20/30` 日多周期标签，以及赚钱概率、亏钱概率、预期收益、预期回撤、盈亏比等赔率化输出。

3. 验证体系不够硬
- 后续必须走 walk-forward、分阶段验证、champion/challenger，而不是只接受“单次训练成功”。

4. ETF 还没有追平股票
- ETF 当前已经正式进入 `fullstack -> evidence_bundle -> feature_snapshot`。
- 但仍缺默认免费事实源、ETF 历史研究、ETF 标签和 ETF 专项训练模型。

当前优先级共识已经锁定为：

1. 扩样本
2. 做多周期标签
3. 做滚动验证
4. 做赔率输出
5. 做仓位联动
6. 再补 ETF 专项训练
7. 最后才考虑更复杂模型

当前不建议直接跳神经网络或更复杂模型。
原因不是它们绝对没用，而是当前真正短板仍在数据、标签、验证和分资产建模。

如果后续 AI 要继续接这条线，默认优先阅读：

- `docs/plans/2026-04-13-trading-readiness-roadmap.md`
- `docs/plans/2026-04-13-security-model-upgrade-roadmap.md`
