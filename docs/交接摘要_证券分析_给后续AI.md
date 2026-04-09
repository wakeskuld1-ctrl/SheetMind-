# 证券分析交接摘要（给后续 AI）

<!-- 2026-04-09 CST: 重写证券分析专项交接摘要。原因：当前本地主文档仍停留在旧的 briefing / committee 阶段，而远端 foundation-navigation-kernel 已补入 scorecard、chair、training 等后续进展。目的：把证券主线当前真实状态、已知尾项和推荐接续顺序写成单页入口。 -->

## 1. 当前证券主线真实状态

当前证券主线已经从“单证券技术分析”推进到“研究治理 + 审批治理 + 评分卡治理”的组合主链。

### 1.1 研究与分析层

- `technical_consultation_basic`
- `security_analysis_contextual`
- `security_analysis_fullstack`

### 1.2 治理与审批层

- `security_decision_evidence_bundle`
- `security_decision_committee`
- `security_position_plan`
- `security_decision_submit_approval`
- `security_decision_verify_package`
- `security_decision_package_revision`
- `security_record_post_meeting_conclusion`

### 1.3 评分卡与量化治理层

- `security_feature_snapshot`
- `security_forward_outcome`
- `security_scorecard`
- `security_scorecard_refit`
- `security_scorecard_training`
- `security_chair_resolution`

## 2. 当前已经完成的关键收口

- 证券治理主链已经形成正式对象链
- `security_record_post_meeting_conclusion` 已完成最小闭环
- `security_scorecard` 已进入正式治理链，不再只是临时分析输出
- `security_decision_verify_package` 已补评分卡一致性护栏
- `security_scorecard_refit` 与 `security_scorecard_training` 已进入 catalog / dispatcher

## 3. 当前仍未完全收口的点

### 3.1 会后结论链

当前仍缺：

- `post_meeting_conclusion` 挂入 `decision_package.object_graph`
- `post_meeting_conclusion` 挂入 `artifact_manifest`
- `security_decision_verify_package` 的会后结论绑定 / 配对 / 完整性校验

结论口径应保持为：

- Task 3 最小闭环已通
- Task 3 还没有完整收口

### 3.2 评分卡训练链

当前最明确的尾项是：

- `security_scorecard_training_generates_artifact_and_registers_refit_outputs`

这意味着：

- 训练入口已经存在
- artifact 落盘已经存在
- refit_run / model_registry 接线已经存在
- 当前不是“没做”，而是“还差最后一处端到端收敛”

## 4. 当前最关键的新增理解

### 4.1 `scorecard` 不是最终主席决议

当前必须区分三条线：

- `committee`：投委会线
- `scorecard`：量化评分卡线
- `chair_resolution`：主席正式决议线

不要再把三者重新混成一个字段集合。

### 4.2 `training` 已经进入主链

`security_scorecard_training` 当前已经不是草案状态，而是正式 Tool：

- 已进入 `catalog`
- 已进入 `dispatcher`
- 已有 CLI 测试
- 已有训练请求合同、artifact 合同、refit 合同、registry 合同

### 4.3 现在默认应以当前主工作区文档为准

本轮已经把远端观察分支中的交接要点回收到当前主文档。

后续接手时：

- 不要再优先从 worktree 分支文档读主线
- 默认先看当前主工作区的 `docs/AI_HANDOFF.md`
- 再看本页

## 5. 数据与日期硬规则

- 证券分析默认只允许使用当前日期
- 当前日期无有效收盘时，才允许退到前一个交易日
- 输出必须写明分析日期
- 不允许混用多个交易日的数据拼结论
- 不用大模型抓行情
- 不用 Token 作为默认数据接入方式
- 免费源 unavailable 时允许降级，但必须显式说明范围

## 6. 当前推荐续做顺序

如果继续证券主线，建议顺序是：

1. 先确认 `M3` 收口状态，并以当前主工作区文档作为唯一主入口
2. 再决定是否进入 `M4` 增强层，优先方向是多笔成交 journal / 更强审计链 / 更细执行归因
3. 评分卡训练链若仍有尾测，再单独按训练链处理，不要回退证券治理主链已完成状态

不建议继续开新的平行实验分支再写另一套链。

## 7. 接手时优先阅读

### 7.1 主文档

- `README.md`
- `docs/AI_HANDOFF.md`
- `CHANGELOG_TASK.MD`

### 7.2 证券核心实现

- `src/ops/security_decision_evidence_bundle.rs`
- `src/ops/security_decision_committee.rs`
- `src/ops/security_position_plan.rs`
- `src/ops/security_decision_submit_approval.rs`
- `src/ops/security_decision_verify_package.rs`
- `src/ops/security_decision_package_revision.rs`
- `src/ops/security_record_post_meeting_conclusion.rs`
- `src/ops/security_scorecard.rs`
- `src/ops/security_scorecard_model_registry.rs`
- `src/ops/security_scorecard_refit_run.rs`
- `src/ops/security_scorecard_training.rs`
- `src/ops/security_chair_resolution.rs`

### 7.3 证券关键测试

- `tests/security_post_meeting_conclusion_cli.rs`
- `tests/security_decision_verify_package_cli.rs`
- `tests/security_scorecard_cli.rs`
- `tests/security_scorecard_refit_cli.rs`
- `tests/security_scorecard_training_cli.rs`

## 8. 一句话结论

当前证券主线最值得继续推进的，不是重开新架构，而是把已有治理链、评分卡链和训练链做完最后的正式收口。
## 9. 2026-04-09 Task 6 收口状态

- 之前文档里提到、但代码中缺失的 4 个对象现在都已落地：
  - `security_record_post_meeting_conclusion`
  - `security_decision_package`
  - `security_decision_verify_package`
  - `security_decision_package_revision`
- 当前最小正式链路已经是：
  - `chair_resolution -> post_meeting_conclusion -> decision_package -> verify_package -> package_revision`
- `decision_package` 当前已经把 `post_meeting_conclusion` 正式挂进：
  - `object_graph`
  - `artifact_manifest`
- `verify_package` 当前已经能抓出“会后结论缺挂载”这一类假收口问题。
- `package_revision` 当前已经能把 verify 失败转换成可执行修补建议，而不是只返回失败状态。
- 对应测试已经新增并通过：
  - `tests/security_decision_package_cli.rs`
  - `cargo test --test security_decision_package_cli -- --nocapture`
  - `cargo test --test security_chair_resolution_cli -- --nocapture`

后续如果继续往下做，就不要再把 Task 6 当成未落地计划项了；下一步应转到更完整的投中 / 投后对象层，或者继续增强 package 校验颗粒度。

## 10. 2026-04-09 Task 8-10 最新真实状态

- `Task 8` 已不是路线图项，而是正式可调用的 `security_post_trade_review`
- `Task 9` 已把 `post_trade_review` 正式挂入 `security_decision_package -> verify -> revision`
- `Task 10` 已把 `security_execution_record` 作为真实执行对象正式落地，并挂入：
  - `security_post_trade_review`
  - `security_decision_package`
  - `security_decision_verify_package`
  - `security_decision_package_revision`
- 当前 `Task 10` 已固定最小正式字段：
  - `actual_entry_date`
  - `actual_entry_price`
  - `actual_position_pct`
  - `actual_exit_date`
  - `actual_exit_price`
  - `exit_reason`
  - `execution_record_notes`
- 当前最小收益归因字段：
  - `planned_entry_price`
  - `planned_position_pct`
  - `planned_forward_return`
  - `actual_return`
  - `entry_slippage_pct`
  - `position_size_gap_pct`
  - `execution_return_gap`
  - `execution_quality`
- 当前 verify / revision 已能识别并修补建议：
  - `missing_execution_record`
  - `execution_record_ref_misaligned`
- 已验证：
  - `cargo test --test security_execution_record_cli -- --nocapture`
  - `cargo test --test security_post_trade_review_cli -- --nocapture`
  - `cargo test --test security_decision_package_cli -- --nocapture`

这意味着当前证券主线里的 `M3` 可以视为核心收口完成：主链已经从“投前/建议层治理”推进到“真实执行对象正式进入 review 与 package 治理链”。

## 11. M3 之后默认怎么接

- 默认不要再回头重做 `Task 6-10`
- 如果用户继续要求“把 M3 做完整”，优先补的是：
  - 多笔成交 `execution_journal`
  - 更强 package 审计链
  - 更细 execution 归因模板
- 如果用户问“现在做到哪了”，统一口径应是：
  - `M3` 核心闭环已完成
  - 后续进入的是 `M4` 增强层，不是回补 `M3` 基础缺口
## 12. 2026-04-09 Task 12 最新真实状态
- `security_portfolio_position_plan` 已正式落地，当前不是草稿 helper，而是正式 Tool
- 当前它消费的是“账户级输入 + 单票正式 position_plan”，不是再造第二套单票建议逻辑
- 当前正式输入包括：
  - `total_equity`
  - `available_cash`
  - `holdings`
  - `candidates`
- 当前正式输出已经能回答：
  - 账户当前现金占比是多少
  - 可部署现金还有多少
  - 哪些标的优先加仓 / 开仓
  - 每只建议分配多少金额
  - 哪些约束命中了
- 当前版本明确只做规则型账户分配，不做：
  - 马科维茨类复杂优化
  - 自动再平衡
  - 券商真实账户直连
- 当前最关键的 4 个账户级约束是：
  - 现金底线
  - 单票上限
  - 行业上限
  - 风险等级上限
- 当前优先级排序规则是：
  - `confidence + odds_grade - risk_penalty`
- 当前已确认并修复的真实问题是：
  - 同一 `symbol` 的现有持仓暴露必须累加，不能覆盖
- 已验证：
  - `cargo test --test security_portfolio_position_plan_cli -- --nocapture`

这意味着后续如果用户再问“什么时候可以真正用起来”，答案已经不再只是“单票能不能分析”，而是开始具备“账户里新增这笔钱按什么顺序分出去”的正式能力。

## 13. 2026-04-09 Task 12 第二轮最新真实状态
- `security_portfolio_position_plan` 已补入账户级风险预算门禁，不再只有现金与行业约束
- 当前新增输入：
  - `max_portfolio_risk_budget_pct`
  - `current_portfolio_risk_budget_pct`
  - `max_single_trade_risk_budget_pct`
- 当前新增输出：
  - `remaining_portfolio_risk_budget_pct`
  - `estimated_new_risk_budget_pct`
  - `total_portfolio_risk_budget_pct`
  - `risk_budget_warnings`
- 当前逐标的输出已能告诉上层：
  - 这笔新增仓位预计占用多少风险预算
  - 是否被账户总风险预算挡住
- 当前风险预算仍是最小规则型版本：
  - 通过 `position_risk_grade` 折算预算占用
  - 还没有真实波动率、相关性和组合回撤模型
- 当前已经有一个明确可复现的门禁场景：
  - 第一只候选吃满剩余风险预算后
  - 第二只候选即使命中现金和行业条件，也会因 `portfolio_risk_budget_reached` 被挡住
- 已验证：
  - `cargo fmt --all`
  - `cargo test --test security_portfolio_position_plan_cli -- --nocapture`

这说明账户层已经从“资金分配建议”推进到“资金分配 + 风险预算门禁”的下一层，而不是继续往复杂优化跳。

## 14. 2026-04-09 Task 12 第三轮最新真实状态
- `security_position_plan` 已补入正式分层模板字段，单票层现在能直接告诉上层：
  - 首层仓位多大
  - 加仓层多大
  - 减仓层多大
  - 最多几层
  - 每层怎么触发
- `security_portfolio_position_plan` 已补入账户层分层动作建议，当前能直接回答：
  - 这次建议是 `entry_tranche` 还是 `add_tranche`
  - 这次建议加多少层
  - 还剩几层容量
- 当前设计结论是：
  - 单票层负责定义分层模板
  - 账户层负责结合当前持仓、现金预算、风险预算，决定本次走哪一层
  - 不新增第三套平行仓位算法
- 当前实现已修掉一个真实回退 bug：
  - 旧版 `SecurityPositionPlanDocument` 没有 `entry_tranche_pct / max_tranche_count` 时，账户层现在会自动回退到 `starter_position_pct` 等字段推导，不再误算成 0 层
- 已验证：
  - `cargo fmt --all`
  - `cargo test --test security_position_plan_cli -- --nocapture`
  - `cargo test --test security_portfolio_position_plan_cli -- --nocapture`

这意味着方案 A-1 这一轮已经不只是补字段，而是把“仓位模板定义”和“账户层当前该走哪一层”都收成了正式对象。
## 15. 2026-04-09 Task 12 第四轮最新真实状态
- `security_execution_record` 已支持挂入账户层 `portfolio_position_plan_document`
- 当前 execution record 已能直接回答：
  - 账户层原计划是 `entry_tranche` 还是 `add_tranche`
  - 计划 tranche 大小是多少
  - 实际执行波段做了多大
  - 实际峰值仓位是否超出账户原计划
  - 这次执行相对账户预算属于 `aligned / under_budget / over_budget / direction_mismatch`
- `security_post_trade_review` 已继续把账户偏差翻译成正式复盘语言：
  - `account_plan_alignment`
  - `tranche_discipline`
  - `budget_drift_reason`
  - `next_account_adjustment_hint`
- 当前最关键的设计结论是：
  - 账户计划对齐事实写在 `execution_record`
  - 治理解释写在 `post_trade_review`
  - 不再新拆第三个平行 Tool
- 当前已确认并修正一个真实偏差口径问题：
  - `tranche_count_drift` 不该按单票默认 starter 推导
  - 必须按“账户层这次明确建议的 tranche 大小”来算
- 已验证：
  - `cargo fmt --all`
  - `cargo test --test security_execution_record_cli -- --nocapture`
  - `cargo test --test security_post_trade_review_cli -- --nocapture`

这说明 `方案A-2` 已经收口，账户级仓位管理链路现在不仅能给计划，还能回写执行偏差并进入投后复盘。
