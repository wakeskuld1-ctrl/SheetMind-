# Security Master Scorecard Minimal Design

<!-- 2026-04-11 CST: 新增这份最小实现设计稿，原因是用户确认按方案 C 推进“大平衡计分卡”的第一阶段落地；目的：先把“历史回放型 master_scorecard”的正式对象边界、计算口径和非目标冻结下来，避免实现时把设计稿中的完整版多头训练误混进当前交付。 -->

## 1. 目标

本轮不是直接交付完整版“大平衡计分卡系统”，而是先交付一个可正式消费、可落盘、可复盘的最小对象：

- 正式对象名：`security_master_scorecard`
- 语义：`历史回放型总卡`
- 用途：汇总同一 `as_of_date` 下 `5 / 10 / 20 / 30 / 60 / 180` 天未来标签的赚钱效益、回撤韧性和路径质量
- 角色：研究/回算对象，不是正式主席裁决

本轮完成后，系统至少要能回答：

1. 某次历史时点在未来多期限里“赚不赚钱”
2. 赚钱过程是否伴随过大回撤
3. 盈利是否靠顺畅路径获得，还是高噪声、高波动结果
4. 这些多期限结果如何汇总成一张正式总卡对象

## 2. 非目标

本轮明确不做：

- 不做完整版多目标训练头
- 不做 `security_master_scorecard` 的线上预测版
- 不把 `master_scorecard` 接成主席裁决
- 不引入新的训练 artifact 格式
- 不改动现有 `security_chair_resolution` 的对象语义

## 3. 对象边界

### 3.1 输入对象

`security_master_scorecard` 只消费现有正式链对象：

- `security_decision_committee`
- `security_scorecard`
- `security_forward_outcome`

其中：

- `committee` 提供研究/风控上下文
- `scorecard` 提供量化线当前状态与快照锚点
- `forward_outcome` 提供未来多期限赚钱效益与路径标签

### 3.2 输出对象

新增正式对象：

- `SecurityMasterScorecardDocument`

至少包含：

- `master_scorecard_id`
- `contract_version`
- `document_type`
- `generated_at`
- `symbol`
- `analysis_date`
- `committee_session_ref`
- `scorecard_ref`
- `scorecard_status`
- `aggregation_version`
- `horizon_breakdown`
- `profitability_effectiveness_score`
- `risk_resilience_score`
- `path_quality_score`
- `master_score`
- `master_signal`
- `limitations`

### 3.3 返回对象

新增 Tool 聚合返回：

- `committee_result`
- `scorecard`
- `master_scorecard`

这样可以保持和现有 `chair_resolution` 一致的“中间线 + 正式对象”结构，但不把 `master_scorecard` 混成主席决议。

## 4. 计算口径

### 4.1 每期限子分

对每个 horizon 计算三类子分，统一映射到 `0 ~ 100`：

- `profitability_score`
  - 依据 `forward_return` 与 `max_runup`
- `risk_score`
  - 依据 `max_drawdown` 相对 `stop_loss_pct`
- `path_score`
  - 依据 `positive_return / hit_upside_first / hit_stop_first`

### 4.2 每期限总分

每个 horizon 的总分先用透明固定权重：

`horizon_total = 0.45 * profitability_score + 0.35 * risk_score + 0.20 * path_score`

说明：

- 这是“最小可用聚合公式”
- 明确标注为 `aggregation_version = security_master_scorecard.replay.v1`
- 该公式只用于回放型总卡，不宣称是训练得出的预测系数

### 4.3 多期限汇总

固定期限权重：

- `5d = 0.10`
- `10d = 0.15`
- `20d = 0.20`
- `30d = 0.20`
- `60d = 0.20`
- `180d = 0.15`

汇总得到：

- `profitability_effectiveness_score`
- `risk_resilience_score`
- `path_quality_score`
- `master_score`

### 4.4 信号分档

最小版信号分档：

- `>= 75` -> `historically_effective`
- `>= 60` -> `constructive`
- `>= 45` -> `mixed`
- `< 45` -> `weak`

## 5. 状态语义

`security_master_scorecard` 必须显式区分状态：

- 如果 `scorecard.score_status == ready`
  - 允许标注为 `replay_with_quant_context`
- 如果 `scorecard.score_status != ready`
  - 标注为 `historical_replay_only`

这能保证：

- 总卡可以在没有量化模型 artifact 时仍完成历史回放
- 但不会冒充“完整线上量化总卡”

## 6. 测试策略

最小红绿链包含：

1. `tool_catalog` 能发现 `security_master_scorecard`
2. CLI 能返回正式 `security_master_scorecard` 对象
3. 在单边上行夹具下：
   - 六个 horizon 全部生成
   - `master_score` 高于中性阈值
   - `master_signal = historically_effective`
4. 无模型 artifact 时：
   - `scorecard.score_status = model_unavailable`
   - `master_scorecard.scorecard_status = model_unavailable`
   - `master_scorecard` 仍能落正式对象

## 7. 后续升级点

本轮完成后，下一阶段可以继续做：

- 把固定聚合权重升级为离线重估权重
- 把 `master_scorecard` 的 `profitability / risk / path` 三个头升级为正式训练头
- 把债券 ETF、跨市场 ETF 的专用 profile 接入
- 再考虑是否把 `master_scorecard_ref` 挂进更高层 package
