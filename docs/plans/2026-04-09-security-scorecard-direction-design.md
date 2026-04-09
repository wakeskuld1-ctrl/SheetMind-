# Security Scorecard And Direction Semantics Design

<!-- 2026-04-09 CST: 新增这份设计文档，原因是用户已经批准方案 B，要求把 direction 旧语义漂移问题和正式评分卡对象一起处理；目的：先冻结对象边界、版本策略、验证规则与不做事项，再进入 TDD 实现，避免继续在 decision_card 上叠加临时字段。 -->

## 目标

本轮要同时完成两件事：

1. 清理 `decision_card.direction` 的旧语义漂移，避免再出现“委员会多数票为 `avoid`，但 `direction = long`”的错误输出。
2. 新增独立的 `security_scorecard` 正式对象，把评分卡从“分析师主观打分”升级为“模型 artifact 驱动的版本化评分结果”。

本轮的核心原则不是“先给一个看起来像分数的东西”，而是：

- 没有模型 artifact，就明确返回 `unavailable` 或 `model_unavailable`
- 有模型 artifact，才允许计算分箱、WOE、点数、总分与归因
- 评分结果必须能独立落盘、进入 decision package、被 verify 校验、被后续复盘引用

## 当前问题

### 1. `direction` 语义漂移

当前 `SecurityDecisionCard.direction` 是直接从 `integrated_conclusion.stance` 推导出来的。

这会带来两个问题：

- 它没有吸收七席委员会多数票结果
- 它也没有吸收风控席降级后的最终裁决语义

因此会出现：

- `vote_tally.majority_vote = avoid`
- `risk_veto.status = needs_more_evidence`
- 但 `decision_card.direction = long`

这会进一步污染：

- `approval_bridge`
- `position_plan`
- `verify` 中的方向一致性判断

### 2. 当前“置信分”不是正式评分卡

当前 `score_confidence()` 只是规则分：

- 它没有模型版本
- 没有特征分箱
- 没有 WOE / 回归系数
- 没有原始特征快照
- 没有幂等重放基础

因此它不能作为正式平衡计分卡使用，也不能作为后续调参与复盘的可靠基线。

## 方案 B 设计

### 1. 清理 `decision_card` 语义

本轮将 `decision_card` 从“单一 direction 语义”升级为“双层语义”：

- `recommendation_action`
  - 枚举：`buy / hold / reduce / avoid / abstain`
  - 含义：委员会最终建议的执行动作
- `exposure_side`
  - 枚举：`long / short / hedge / neutral`
  - 含义：该建议对应的风险暴露方向

兼容策略：

- 暂时保留旧字段 `direction`
- 但它不再自由推导，而是从 `exposure_side` 派生
- 当前长多主链下：
  - `buy / hold / reduce -> exposure_side = long`
  - `avoid / abstain -> exposure_side = neutral`

这样可以保证：

- 多数票 `avoid` 时，不会再落成 `direction = long`
- `position_plan` 和 `approval_bridge` 可以逐步切换到新字段

### 2. 新增独立正式对象 `security_scorecard`

建议新增：

- `src/ops/security_scorecard.rs`

正式对象名：

- `SecurityScorecardDocument`

它是独立治理对象，不嵌死在 `decision_card` 内。

原因：

- 模型会重训
- 分箱会变更
- 系数会变更
- 评分卡需要单独落盘、单独复盘、单独校验

### 3. `security_scorecard` 存储层合同

建议最小字段如下：

- `scorecard_id`
- `contract_version`
- `document_type`
- `generated_at`
- `symbol`
- `analysis_date`
- `decision_id`
- `decision_ref`
- `approval_ref`
- `score_status`
  - `ready`
  - `model_unavailable`
  - `feature_incomplete`
- `label_definition`
  - 例如 `horizon_10d_stop_5pct_target_10pct`
- `model_binding`
  - `model_id`
  - `model_version`
  - `training_window`
  - `oot_window`
  - `positive_label_definition`
  - `binning_version`
  - `coefficient_version`
  - `model_sha256`
- `raw_feature_snapshot`
  - 原始特征值快照
- `feature_contributions`
  - 每个特征的分箱、WOE、points、group_name
- `group_breakdown`
  - `T / F / E / V` 分组聚合
- `base_score`
- `total_score`
- `success_probability`
- `recommendation_action`
- `exposure_side`
- `score_summary`
- `limitations`

### 4. 评分卡模型边界

本轮不在运行时“现场训练模型”，而是执行“模型消费层”。

也就是说：

- 训练产物来自外部离线训练流程
- 运行时只读取版本化模型 artifact
- 运行时负责：
  - 特征提取
  - 分箱映射
  - WOE/points 累加
  - 概率换算
  - 归因聚合

这样才能满足：

- 系数来自训练
- 线上推理幂等
- package 可审计

### 5. 无模型时的正式行为

如果没有提供评分卡模型 artifact，本轮绝不伪造主观分数。

正式行为是：

- 生成 `security_scorecard`
- `score_status = model_unavailable`
- 保留：
  - 原始特征快照
  - 限制说明
  - 需要补齐的模型绑定信息

这样做的原因是：

- 可以先把对象图、package、verify 链打通
- 同时不违反“不能手工拍权重”的原则

### 6. package / revision / verify 接线

本轮把评分卡作为正式 artifact 接入：

- `decision_package.object_graph`
  - `scorecard_ref`
  - `scorecard_path`
- `artifact_manifest`
  - 新增 `security_scorecard`
- `package_revision`
  - 保留评分卡引用
- `verify`
  - 新增评分卡治理校验：
    - `scorecard_binding_consistent`
    - `scorecard_complete`
    - `scorecard_action_aligned`

### 7. 与 `decision_card`、`approval_bridge`、`position_plan` 的关系

关系分层如下：

- `committee`
  - 生成多数票、风控结果、最终动作语义
- `scorecard`
  - 生成模型分、概率分、分组归因
- `decision_card`
  - 汇总最终执行动作和暴露方向
- `approval_bridge`
  - 将 `recommendation_action / exposure_side` 映射到私有审批语义
- `position_plan`
  - 根据 `recommendation_action / exposure_side` 生成计划方向
  - 后续可继续吸收评分卡概率做仓位细化，但本轮不强耦合

### 8. 本轮不做

本轮明确不做以下内容：

- 不做离线训练流水线
- 不做回归样本回填工具
- 不做自动重训
- 不做评分卡调参 UI
- 不把主观 `score_confidence()` 伪装成正式评分卡

## 测试策略

本轮必须先红后绿，至少补以下测试：

1. 复现 `majority_vote = avoid` 但 `decision_card.direction = long` 的失败测试
2. 评分卡对象可独立落盘的失败测试
3. package 中存在 `scorecard_ref / scorecard_path` 的失败测试
4. verify 能识别评分卡绑定和动作一致性的失败测试
5. 篡改评分卡动作或路径后 verify 失败的测试

## 完成定义

满足以下条件才算本轮完成：

1. `decision_card` 不再出现多数票 `avoid` 但方向仍为 `long`
2. 新增正式 `security_scorecard` 对象
3. `security_scorecard` 可单独落盘并进入 `decision_package`
4. `package_revision` 不丢失评分卡引用
5. `verify` 能校验评分卡绑定、一致性与动作对齐
6. 无模型时系统明确返回 `model_unavailable`，而不是手工打分
