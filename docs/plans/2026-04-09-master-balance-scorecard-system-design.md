# Master Balance Scorecard System Design

<!-- 2026-04-09 CST: 新增这份准实施规格版设计文档，原因是用户已明确要求把“大平衡计分卡”升级为可训练、可回算、可调系数、可审计的正式系统，并且明确要求综合计分卡、投委会、主席裁决三条线强隔离；目的：在改代码前先冻结系统边界、训练逻辑、回算口径、模型发布规则与对象关系，避免后续继续把量化分、委员建议和正式决议混在一起。 -->

## 1. 目标

本轮要设计的不是单张分卡，而是一套“可离线训练、可线上消费、可回算评估、可版本晋级”的大平衡计分卡系统。

系统必须同时满足：

1. 支持方向准确性、收益事件准确性、风险准确性、多期限表现的统一管理。
2. 支持未来 `5 / 10 / 20 / 30 / 60 / 180` 天多期限回算。
3. 支持因子、分箱、系数、校准器、模型版本的正式重估。
4. 支持把量化结果与投委会意见并列保存，但不互相污染。
5. 主席裁决是唯一正式决议出口。

## 2. 核心原则

### 2.1 三线强隔离

系统必须分成三条线：

- `量化计分卡线`
  - 输出数学/统计意义上的评分、概率、风险分层、期限子卡和总卡
  - 不输出正式买卖结论
- `投委会线`
  - 七名成员基于同一份材料独立给出 `buy / hold / reduce / avoid / abstain`
  - 保留理由、支持点、反对点和改票条件
- `主席裁决线`
  - 读取量化计分卡线与投委会线的输出
  - 形成唯一正式决议

结论：

- `master_scorecard` 不是正式决议
- `committee_session` 不是唯一正式决议
- `chair_resolution` 才是唯一正式决议

### 2.2 线上固定、线下重估

线上分析只消费“已发布版本”的模型 artifact，不允许在单次分析时临场改系数。

线下允许：

- 补样本
- 回算
- 调整分箱
- 调整特征集
- 重估系数
- 生成 challenger
- 与 champion 比较

只有样本外结果通过晋级规则，challenger 才能升级为 champion。

### 2.3 所有训练都必须可追溯

每个已发布版本都必须能追溯：

- 训练窗口
- 验证窗口
- 样本外窗口
- 标签定义
- 特征集版本
- 分箱版本
- 系数版本
- 校准器版本
- artifact 哈希

## 3. 总体架构

系统分五层。

### 3.1 L0：硬门槛层

用途：负责否决、降权或标记不可比样本，不参与正向加分。

典型规则：

- 停牌
- 极低流动性
- 数据缺失过多
- 重大未消化负面事件
- ETF 极端溢价折价
- 跨市场价格时间错位严重

输出：

- `hard_gate_status`
- `hard_gate_reasons`
- `data_penalty`
- `risk_penalty`

### 3.2 L1：原子特征层

用途：最小训练单元。每个原子特征都必须可冻结、可回放、可分箱。

硬约束：

- 只能使用分析时点可见信息
- 必须能写入 `feature_snapshot`
- 必须定义类型、空值口径、分箱口径

### 3.3 L2：因子组层

固定 8 组：

- `M` 市场环境
- `F` 基本面
- `V` 估值
- `T` 技术趋势
- `Q` 量价资金
- `E` 事件披露
- `R` 风险脆弱性
- `X` ETF/交易结构

因子组名称应尽量固定，后续调整优先发生在组内特征与系数层，而不是频繁改组。

### 3.4 L3：多期限子卡层

每个期限一张子卡：

- `Score_5`
- `Score_10`
- `Score_20`
- `Score_30`
- `Score_60`
- `Score_180`

每张子卡单独训练、单独评估、单独发布版本。

### 3.5 L4：总平衡计分卡层

总卡不直接消费原子特征，而是汇总多期限子卡与风险惩罚项。

建议公式：

`MasterScore = Σ(alpha_h * Score_h) - RiskPenalty - DataPenalty`

其中：

- `Score_h` 为第 `h` 个期限子卡的综合输出
- `alpha_h` 由样本外结果重估，不允许手工拍定

## 4. 因子层拆分

### 4.1 因子组设计

#### M：市场环境

建议特征：

- 指数趋势状态
- 市场波动率分位
- 行业相对强弱
- 风险偏好状态
- 市场广度

#### F：基本面

建议特征：

- ROE
- 营收增速
- 净利润增速
- 现金流质量
- 银行类的资本充足率、不良率、拨备覆盖率

#### V：估值

建议特征：

- PE 分位
- PB 分位
- 股息率分位
- 行业相对估值
- 历史估值回归空间

#### T：技术趋势

建议特征：

- 均线多空结构
- MACD 状态
- RSI 区间
- 突破/跌破状态
- 趋势斜率

#### Q：量价资金

建议特征：

- 换手率分位
- 量比
- 量价配合程度
- 异常放量
- 资金流代理特征

#### E：事件披露

建议特征：

- 财报是否超预期
- 分红方案质量
- 业绩预告偏正/偏负
- 诉讼/监管事件
- 公告情绪标签

#### R：风险脆弱性

建议特征：

- 历史波动率
- 近期最大回撤
- 跳空频率
- 尾部风险事件密度
- 财务脆弱性

#### X：ETF/交易结构

建议特征：

- ETF 溢价折价
- 跟踪误差
- 日内流动性
- 跨市场时差错位
- 折溢价均值回归压力

### 4.2 因子设计规则

每个特征必须满足：

1. 可在分析时点唯一确定
2. 可映射到稳定分箱
3. 可回算未来多期限表现
4. 能解释其经济意义或市场微观结构意义
5. 在不同年份与窗口内符号稳定

## 5. 目标头设计

大平衡计分卡不建议只有一个目标，而建议使用多目标头。

### 5.1 Head-A：方向头

定义：

- 未来 `N` 天收益是否为正

用途：

- 对应方向准确性

### 5.2 Head-B：收益事件头

定义：

- 未来 `N` 天是否先达到目标收益阈值

用途：

- 对应收益事件准确性

### 5.3 Head-C：风险事件头

定义：

- 未来 `N` 天是否先触发止损/大回撤阈值

用途：

- 对应风险准确性

### 5.4 Head-D：风险调整收益头

定义：

- 未来 `N` 天风险调整后收益是否进入高质量区间

用途：

- 对应中长期持有质量

### 5.5 Head-E：路径质量头

定义：

- 未来 `N` 天路径是否平滑、是否高噪声、是否容易出现“先大亏后反弹”

用途：

- 对应复盘与执行友好度

### 5.6 单期限子卡综合公式

建议：

`Score_h = aA*HeadA + aB*HeadB - aC*HeadC + aD*HeadD + aE*HeadE`

说明：

- `aA ~ aE` 不是手工给定
- 必须由样本外结果重估

## 6. 标签与回算口径

### 6.1 回算期限

固定支持：

- `5`
- `10`
- `20`
- `30`
- `60`
- `180`

### 6.2 回算对象

建议第一阶段分市场处理：

- `A股个股`
- `A股 ETF`
- 以后扩到：
  - `港股`
  - `美股/日股相关 ETF`
  - `股指/指数期货`

原则：

- 不同市场先分池建模
- 不同市场后续只允许在总卡层做汇总
- 不允许直接混训一套全市场通用系数

### 6.3 标签生成规则

每个 `snapshot_id` 都要回填：

- `forward_return`
- `max_drawdown`
- `max_runup`
- `positive_return`
- `hit_upside_first`
- `hit_stop_first`
- `risk_adjusted_bucket`
- `path_quality_bucket`

标签定义必须版本化，避免未来改口径后无法比较新旧版本。

## 7. 正式对象 / 表结构

### 7.1 feature_snapshot：冻结特征快照

对象名建议：

- `SecurityFeatureSnapshot`

关键字段：

- `snapshot_id`
- `symbol`
- `market`
- `instrument_type`
- `as_of_date`
- `data_cutoff_at`
- `feature_set_version`
- `raw_features_json`
- `group_features_json`
- `data_quality_flags`
- `snapshot_hash`

### 7.2 forward_outcome：未来表现标签

对象名建议：

- `SecurityForwardOutcome`

关键字段：

- `snapshot_id`
- `horizon_days`
- `forward_return`
- `max_drawdown`
- `max_runup`
- `positive_return`
- `hit_upside_first`
- `hit_stop_first`
- `risk_adjusted_bucket`
- `path_quality_bucket`
- `label_definition_version`

### 7.3 model_registry：模型注册表

对象名建议：

- `SecurityScorecardModelRegistry`

关键字段：

- `model_id`
- `market_scope`
- `instrument_scope`
- `horizon_days`
- `target_head`
- `model_version`
- `status`
- `training_window`
- `validation_window`
- `oot_window`
- `artifact_path`
- `artifact_sha256`
- `metrics_summary_json`
- `published_at`

### 7.4 refit_run：重估任务记录

对象名建议：

- `SecurityScorecardRefitRun`

关键字段：

- `refit_run_id`
- `market_scope`
- `instrument_scope`
- `feature_set_version`
- `label_definition_version`
- `train_range`
- `valid_range`
- `test_range`
- `candidate_artifact_path`
- `comparison_to_champion_json`
- `promotion_decision`
- `created_at`

### 7.5 horizon_score_run：单期限子卡打分结果

对象名建议：

- `SecurityHorizonScorecardRun`

关键字段：

- `score_run_id`
- `snapshot_id`
- `symbol`
- `horizon_days`
- `model_id`
- `model_version`
- `target_head_scores_json`
- `group_scores_json`
- `raw_total_score`
- `calibrated_probability`
- `quant_signal`
- `confidence_band`
- `risk_penalty`
- `data_penalty`

### 7.6 master_score_run：总卡结果

对象名建议：

- `SecurityMasterScorecardRun`

关键字段：

- `master_score_run_id`
- `snapshot_id`
- `symbol`
- `master_score`
- `horizon_mix_weights_json`
- `horizon_scores_json`
- `group_rollup_json`
- `quant_stance`
- `quant_risk_level`
- `quant_summary`
- `limitations`

### 7.7 committee_member_opinion：委员意见

沿用并扩展现有委员对象。

关键字段：

- `opinion_id`
- `decision_id`
- `member_id`
- `seat_name`
- `vote`
- `confidence`
- `reasoning`
- `supporting_points`
- `counter_points`
- `what_changes_my_mind`

### 7.8 committee_session：投委会会话

对象名建议：

- `SecurityCommitteeSession`

关键字段：

- `committee_session_id`
- `decision_id`
- `member_opinion_refs`
- `vote_tally`
- `risk_veto`
- `committee_summary`
- `committee_action`

### 7.9 chair_resolution：主席裁决

对象名建议：

- `SecurityChairResolution`

关键字段：

- `chair_resolution_id`
- `decision_id`
- `master_score_run_ref`
- `committee_session_ref`
- `selected_action`
- `selected_exposure_side`
- `chair_reasoning`
- `why_followed_quant`
- `why_followed_committee`
- `override_reason`
- `execution_constraints`
- `final_confidence`
- `signed_off_at`

### 7.10 decision_package：正式归档

package 中必须显式挂接三条线：

- `master_scorecard_ref`
- `committee_session_ref`
- `chair_resolution_ref`

## 8. 训练逻辑

这是本设计最关键的部分。训练逻辑必须进入正式系统，而不是只在文档口头描述。

### 8.1 训练总原则

训练是正式主链的一部分，但发生在线下回算环境，不发生在单次线上分析中。

线下训练流水线至少包含：

1. 快照冻结
2. 标签回填
3. 训练样本切分
4. 分箱
5. WOE 编码
6. 模型拟合
7. 概率校准
8. 汇总卡重估
9. 生成 artifact
10. champion/challenger 比较

### 8.2 训练输入

训练输入必须来自两张表：

- `feature_snapshot`
- `forward_outcome`

禁止：

- 训练时直接读取“当前最新指标”
- 训练时用未来修订后的财报字段覆盖历史快照
- 训练时用任意非冻结字段替代当时快照

### 8.3 分市场、分期限、分目标头训练

训练粒度建议为：

- `market_scope × instrument_scope × horizon_days × target_head`

例子：

- `A_SHARE_EQUITY × 10D × DIRECTION_HEAD`
- `A_SHARE_EQUITY × 20D × RISK_HEAD`
- `A_SHARE_ETF × 60D × UPSIDE_EVENT_HEAD`

原因：

- 不同市场结构不同
- 不同期限的信号作用不同
- 不同目标头的最优系数不同

### 8.4 分箱与编码

推荐做法：

1. 对连续特征做单调分箱
2. 生成 bin 标签
3. 计算 WOE
4. 映射 points

优点：

- 易解释
- 易落正式积分卡 artifact
- 易做版本对比
- 易接审计与回算

### 8.5 模型拟合

推荐第一阶段主模型：

- `WOE + Logistic + Elastic Net`

原因：

- 可解释性强
- 系数稳定性比纯黑盒更高
- 更适合落积分卡

可选增强：

- `LightGBM` 作为特征筛选器或 challenger
- 但正式 champion 仍优先积分卡友好的可解释模型

### 8.6 概率校准

每个头在样本外必须做概率校准。

推荐：

- `Platt Scaling`
- 或 `Isotonic Regression`

如果不校准，会出现：

- 排序可用
- 概率值不可用
- 后续无法稳定做总卡汇总与风险预算

### 8.7 总卡汇总训练

总卡不是手工加权，而要用样本外结果重估：

- 期限权重 `alpha_h`
- 目标头权重 `aA ~ aE`
- 风险惩罚强度
- 数据惩罚强度

建议方式：

- 用样本外表现做二层回归或优化
- 目标函数优先考虑：
  - 分层单调性
  - 风险调整收益
  - 回撤约束

### 8.8 artifact 产出

每次训练成功后，必须产出正式 artifact。

artifact 至少包含：

- `model_id`
- `model_version`
- `market_scope`
- `instrument_scope`
- `horizon_days`
- `target_head`
- `label_definition`
- `training_window`
- `validation_window`
- `oot_window`
- `feature_set_version`
- `binning_version`
- `coefficient_version`
- `model_sha256`
- `intercept`
- `base_score`
- `features`
- `calibration`
- `metrics_summary`

线上 `security_scorecard` 只消费该 artifact，不负责训练。

## 9. 过拟合控制

### 9.1 时间切分

必须只允许时间切分，禁止随机切分。

推荐：

- `Walk-Forward`
- `Rolling Train / Validation / Test`

### 9.2 Purge + Embargo

由于 `5/10/20/30/60/180` 天标签存在时间重叠，必须做：

- `purge`
- `embargo`

避免标签泄露。

### 9.3 特征稳定性筛选

如果某个特征在不同年份、不同窗口中频繁翻符号或贡献方向不稳定，则不能进入正式 champion。

### 9.4 特征数量控制

每个子模型的特征数必须受控。

原则：

- 宁可少特征
- 不允许为了提高回测表现而堆太多弱特征

### 9.5 单调约束

对业务确定性高的特征尽量加单调约束，例如：

- 溢价率越高，风险应越大
- 回撤越深，得分不应越高
- 数据缺口越多，置信区间应越差

### 9.6 幸存者偏差控制

训练与回算时必须保留：

- 退市股
- 暴雷股
- 历史坏样本

不允许只用当前存活标的。

### 9.7 漂移监控

上线后必须监控：

- PSI
- 评分分布漂移
- 各组贡献漂移
- 期限命中率漂移

## 10. 评估与晋级规则

### 10.1 评估指标

不能只看单一分类指标。

至少要同时记录：

- `AUC`
- `Brier Score`
- `Precision / Recall`
- `Top vs Bottom 分层收益`
- `最大回撤`
- `收益单调性`
- `不同年份稳定性`

### 10.2 champion / challenger

正式发布流程：

1. 旧版本为 `champion`
2. 新重估版本为 `challenger`
3. 在样本外窗口比较
4. 通过阈值才升级

### 10.3 晋级条件

建议必须同时满足：

- 风险调整后收益不劣于 champion
- 回撤不显著恶化
- 概率校准不明显变差
- 多个年份窗口中稳定优于或至少不输

## 11. 与现有证券主链的关系

### 11.1 scorecard 线

现有 `security_scorecard` 继续保留，但语义调整为：

- 它是量化计分卡线的正式输出对象
- 它不是正式投资建议对象

后续应避免继续把 `recommendation_action` 当成正式投决输出使用，可以逐步改名为：

- `quant_signal`
- `quant_stance`
- `quant_hint`

### 11.2 committee 线

现有 `security_decision_committee` 继续作为独立人类式审议模拟系统存在。

### 11.3 chair 线

后续必须新增 `chair_resolution`，把最终正式决议从 `decision_card` 中剥离出来。

## 12. 本轮准实施范围

本轮设计完成后，开发不应一次性全做完，而应分阶段推进：

- `P0`
  - 三线正式分离
  - feature_snapshot / forward_outcome / master_scorecard_run 落地
- `P1`
  - 多期限回算
  - refit_run / model_registry / artifact 训练发布链
- `P2`
  - chair_resolution
  - champion/challenger 晋级
  - 漂移监控与复盘比较

## 13. 完成定义

满足以下条件，才算这套系统进入可实施状态：

1. 三条线语义彻底分离
2. 线上只消费正式版本 artifact
3. 线下可以对多期限标签做回算
4. 训练结果可以生成正式 artifact
5. 新旧模型可比较、可晋级、可回退
6. 最终正式决议只出现在主席裁决线
