# 证券持仓台账

## 文档目的
- 2026-04-11 CST：新增统一证券持仓台账，原因是用户要求把后续所有持仓都固定记录到同一文档中，避免对话结束后遗忘正式持仓锚点。
- 2026-04-11 CST：本台账只记录正式持仓计划、实际建仓事件、后续调仓事件与复盘引用，目的：让 `position_plan_ref -> adjustment_event_ref -> post_trade_review_ref` 形成可追溯主链。

## 记录规则
- 每次新增持仓，先记录一条“持仓计划”。
- 实际成交后，再补“建仓事件”。
- 后续每次加仓、减仓、退出，都追加新的“调仓事件”。
- 完成阶段性复盘后，补“复盘结果”。
- 不伪造成交：未真实成交前，`build/add/reduce/exit` 事件一律留空。

## 字段说明
- `持仓编码`：正式 `position_plan_ref`，是后续所有复盘与调仓的主锚点。
- `决策编码`：正式 `decision_ref`。
- `审批编码`：正式 `approval_ref`。
- `当前状态`：`计划中` / `已建仓` / `持有中` / `已减仓` / `已退出`。
- `建仓事件编码`：真实成交后补 `position-adjustment:*:build:v1`。
- `后续调仓事件`：真实成交后按时间顺序追加。
- `复盘编码`：完成复盘后补 `post-trade-review:*`。

## 当前持仓计划

### 2026-04-11 保本优先版组合
- 组合目标：`10000 元` 资金的保本优先版场内债券 ETF 组合。
- 组合结构：`511360.SH = 60%`，`511010.SH = 25%`，`511060.SH = 15%`。
- 组合说明：本轮只登记正式持仓计划，尚未登记真实成交事件。
- 证据版本：`defensive-portfolio.2026-04-11.v1`

| 标的 | 建议仓位 | 持仓编码 | 决策编码 | 审批编码 | 当前状态 | 建仓事件编码 | 后续调仓事件 | 复盘编码 | 备注 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `511360.SH` | `60%` | `position-plan:511360.SH:2026-04-11:v1` | `decision:511360.SH:2026-04-11:defensive-v1` | `approval:511360.SH:2026-04-11:defensive-v1` | `计划中` | `待真实成交后补录` | `暂无` | `暂无` | 核心防守仓，短融 ETF |
| `511010.SH` | `25%` | `position-plan:511010.SH:2026-04-11:v1` | `decision:511010.SH:2026-04-11:defensive-v1` | `approval:511010.SH:2026-04-11:defensive-v1` | `计划中` | `待真实成交后补录` | `暂无` | `暂无` | 国债 ETF，利率防守仓 |
| `511060.SH` | `15%` | `position-plan:511060.SH:2026-04-11:v1` | `decision:511060.SH:2026-04-11:defensive-v1` | `approval:511060.SH:2026-04-11:defensive-v1` | `计划中` | `待真实成交后补录` | `暂无` | `暂无` | 地方债 ETF，卫星收益仓 |

## 2026-04-11 多期限训练概率回填

- 2026-04-11 CST：本节补记基于 `2026-04-10` 收盘快照、并在同一本地历史库上完成回补后的正式训练概率，原因是用户要求把 `3 / 6 / 10 / 30 / 60 / 180` 天的涨跌概率写回持仓台账，便于后续复盘与对照；目的：让后续能同时看到 `position_plan_ref -> scorecard_model_path -> success_probability` 的量化锚点，而不是只剩对话里的临时口径。
- 历史回补结果：
  - 已通过正式 `sync_stock_price_history` 主链，把 `511360.SH / 511010.SH / 511060.SH / 510300.SH` 回补到 `2024-01-02 -> 2026-04-10`。
  - 当前训练库：`D:\Rust\Excel_Skill\.excel_skill_runtime\holding_review_2026-04-11_defensive_rr15\stock_history.db`
  - 当前 4 个标的历史条数均为 `548`。
- 口径说明：
  - 概率定义为 `P(forward_return > 0)`。
  - 下跌概率 = `1 - 上涨概率`。
  - 这些结果当前是 `candidate` 候选模型口径，不是已晋级 champion。

| 标的 | 3天上涨概率 | 6天上涨概率 | 10天上涨概率 | 30天上涨概率 | 60天上涨概率 | 180天上涨概率 | 当前解释 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `511360.SH` | `99.96%` | `48.15%` | `99.95%` | `99.99%` | `78.85%` | `98.09%` | 偏短久期、短融属性更强，短周期与长周期训练都更容易被模型识别为正收益资产。 |
| `511010.SH` | `87.12%` | `99.75%` | `59.98%` | `99.95%` | `99.92%` | `99.97%` | 中久期国债 ETF，当前模型把它识别为高胜率防守资产，但短期和中期分歧仍明显。 |
| `511060.SH` | `87.12%` | `99.75%` | `59.98%` | `99.95%` | `99.92%` | `99.97%` | 当前概率与 `511010.SH` 完全一致，原因不是数据串库，而是当前正式评分卡特征过粗、两只 ETF 在 `2026-04-10` 命中的特征分箱完全相同。 |

- 当前拟合摘要：
  - `3d`: `sample_count = 24`, `train_auc = 0.7222`, `test_auc = 0.6000`
  - `6d`: `sample_count = 24`, `train_auc = 0.7571`, `valid/test AUC` 暂不完整
  - `10d`: `sample_count = 24`, `train_auc = 0.6250`, `valid/test AUC` 暂不完整
  - `30d`: `sample_count = 24`, `train_auc = 0.9500`, 但 `valid_auc = 0.3333`, `test_auc = 0.0000`，说明长期窗口虽然已经可训，但这版仍不稳定，不能把高概率误读成高可信度
  - `60d`: `sample_count = 24`, `train_auc = 0.7727`, `valid_auc = 0.3750`, `test_auc = 0.5000`
  - `180d`: `sample_count = 24`, `train_auc = 0.8636`, `valid/test AUC` 暂不完整

- 为什么 `511010.SH` 和 `511060.SH` 会出现同分：
  - 已复核当前 `2026-04-10` 的 `raw_feature_snapshot`，这两只 ETF 在正式评分卡消费的 4 个核心特征上命中完全一致：
  - `integrated_stance = technical_only`
  - `technical_alignment = mixed`
  - `data_gap_count = 2`
  - `risk_note_count = 8`
  - 由于当前正式模型主要消费这 4 个特征，所以这两只 ETF 即使历史价格序列不同，只要当前日快照命中同一组分箱，输出概率就会相同。
  - 这说明当前问题的根因是“特征区分度不足”，不是“历史库串数据”。

- 当前模型边界：
  - 本轮已补齐长期 horizon 的历史长度，所以 `30 / 60 / 180` 从“不可训练”推进到了“可训练候选模型”。
  - 但正式评分卡当前仍只有 4 个较粗特征，尚未纳入 ETF 区分度更强的输入，如久期代理、折溢价、流动性分层、波动率分层、资金流分层等。
  - 因此，本节概率更适合当作“已正式训练的候选量化视角”，暂不宜直接替代主席裁决与仓位执行口径。

## 后续追加模板

### YYYY-MM-DD 组合名称
- 组合目标：
- 组合结构：
- 证据版本：

| 标的 | 建议仓位 | 持仓编码 | 决策编码 | 审批编码 | 当前状态 | 建仓事件编码 | 后续调仓事件 | 复盘编码 | 备注 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `示例.SH` | `0%` | `position-plan:示例.SH:YYYY-MM-DD:v1` | `decision:示例.SH:YYYY-MM-DD:v1` | `approval:示例.SH:YYYY-MM-DD:v1` | `计划中` | `待补录` | `暂无` | `暂无` | 说明 |

## 2026-04-11 正式会后复核

### 保本优先版组合（主席裁决 rr1.5 版）
- 2026-04-11 CST：补记保本优先版组合的正式投决会/评分卡/主席裁决复核结果，原因是用户要求把能复盘的正式链结果写回同一份持仓报告，而不是只留在对话里；目的：让后续能直接沿 `position_plan_ref -> committee_session_ref -> scorecard_ref -> chair_resolution_id -> 原始 JSON` 回看。
- 运行口径：`as_of_date = 2026-04-10`，`stop_loss_pct = 1.0%`，`target_return_pct = 1.5%`，`min_risk_reward_ratio = 1.2`。
- 运行目录：`D:\Rust\Excel_Skill\.excel_skill_runtime\holding_review_2026-04-11_defensive_rr15`
- 运行结论：3 只债券 ETF 在风报比合规的前提下，正式主席裁决仍全部为 `avoid`，当前不建议把原始保本优先版计划直接放行为执行计划。

| 标的 | 对应持仓编码 | 委员会编码 | 评分卡编码 | 主席裁决编码 | 正式动作 | 风控状态 | 评分卡状态 | 置信度 | 原始裁决文件 | 复核摘要 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `511360.SH` | `position-plan:511360.SH:2026-04-11:v1` | `committee-511360.SH-2026-04-10` | `scorecard-511360.SH-2026-04-10` | `chair-511360.SH-2026-04-10` | `avoid` | `needs_more_evidence` | `model_unavailable` | `0.12` | `D:\Rust\Excel_Skill\.excel_skill_runtime\holding_review_2026-04-11_defensive_rr15\511360_SH_chair_resolution.json` | 技术面为 `headwind`，同时基本面/公告面缺口未补齐，正式链要求先补证据再审。 |
| `511010.SH` | `position-plan:511010.SH:2026-04-11:v1` | `committee-511010.SH-2026-04-10` | `scorecard-511010.SH-2026-04-10` | `chair-511010.SH-2026-04-10` | `avoid` | `needs_more_evidence` | `model_unavailable` | `0.12` | `D:\Rust\Excel_Skill\.excel_skill_runtime\holding_review_2026-04-11_defensive_rr15\511010_SH_chair_resolution.json` | 技术面为 `mixed`，正式链仍要求等待环境与个股进一步共振，不放行执行。 |
| `511060.SH` | `position-plan:511060.SH:2026-04-11:v1` | `committee-511060.SH-2026-04-10` | `scorecard-511060.SH-2026-04-10` | `chair-511060.SH-2026-04-10` | `avoid` | `needs_more_evidence` | `model_unavailable` | `0.12` | `D:\Rust\Excel_Skill\.excel_skill_runtime\holding_review_2026-04-11_defensive_rr15\511060_SH_chair_resolution.json` | 技术面为 `mixed`，且信息面缺口仍在，正式链结论是继续观察而非直接建仓。 |
## 2026-04-11 审批链总卡口径补充

- 本轮已把 `master_scorecard` 接入正式 `security_decision_submit_approval` 主链，并通过 `approval_brief.master_scorecard_summary` 落入审批摘要。
- 正式口径分成两种：
- `回放可用`：当 `as_of_date` 同时满足前置技术历史窗口和后置 `5/10/20/30/60/180` 天回放窗口时，审批链会落正式多期限总卡。
- `实盘最新日`：当最新分析日还没有未来回放窗口时，审批链不再报错，而是把总卡明确降级为 `aggregation_status = replay_unavailable`、`master_signal = unavailable`，继续允许审批链持久化。
- 这条规则的目的不是放宽评分要求，而是防止“历史回放型总卡”误伤真实审批入口；没有回放窗口时，正式链必须明确披露不可回放，而不是伪造分数。
- 当前正式摘要最少会保留这些字段，供后续复盘核对：
- `master_scorecard_ref`
- `scorecard_ref`
- `scorecard_status`
- `aggregation_status`
- `master_score`
- `master_signal`
- `profitability_effectiveness_score`
- `risk_resilience_score`
- `path_quality_score`
- 相关回归已经覆盖：
- `security_decision_submit_approval_writes_runtime_files_for_ready_case`
- `security_decision_submit_approval_degrades_master_scorecard_when_replay_window_is_unavailable`

## 2026-04-11 公开数据三线补录

- 2026-04-11 CST：新增“公开数据三线补录”，原因是用户要求把 `数据线 -> 投决会线 -> 主席评估` 的补充判断也写入持仓台账，避免后续只剩正式链 JSON 而缺少公开市场环境与人工补录结论；目的：让未来复盘时能同时看到“正式链结论”和“公开数据补证后的管理判断”。
- 本节不是新的正式 Tool 产物，不新增 `committee_ref / chair_resolution_ref`；正式裁决仍以“2026-04-11 正式会后复核”一节为准。

### 数据线

- 宏观背景（截至 `2026-04-10`）：
- 公开市场利率环境仍偏宽松。`2026-04-10` 债市午盘显示，央行开展 `20 亿元` `7天逆回购`，操作利率 `1.40%` 不变；10年期国债活跃券收益率下行至 `1.8120%`，30年期国债收益率下行至 `2.3290%`。这说明低波动债券资产仍有配置支撑，但并不意味着中长久期债券会单边上涨。
- 新华财经 `2026-04-07` 的债市观察同时指出，跨季资金宽松支撑短债走强，但 `3月 PMI = 50.4` 重返扩张区间，会压制长端利率继续大幅下行；对这组 ETF 来说，含义是“短端强于中长端、流动性优先于博收益”。
- 每经 `2026-04-04` 的报道也确认，一季度末国债逆回购收益率偏低的核心原因是资金面宽松，`DR001` 已降至 `1.23%` 附近。这个环境更利好短融类工具的稳定性，不利好把中久期债基当成进攻资产。

| 标的 | 最新公开数据 | 数据判断 | 未来 5 个交易日公开推演 | 未来 2-4 周公开推演 |
| --- | --- | --- | --- | --- |
| `511360.SH` 海富通中证短融ETF | `2026-04-02` 最新规模 `914.86 亿元`，较 `2025-12-31` 增长 `30.28%`；近 20 个交易日日均成交额 `543.59 亿元`；证券之星抓取到 `2026-04-03` 折溢价率约 `0.01%`。 | 三者里流动性最好、容量最大、交易摩擦最小，且受“资金宽松+短债偏强”组合支持最直接。 | `-0.03% ~ +0.12%`，偏稳。 | `0% ~ +0.35%`，更适合做防守底仓，不适合期待明显弹性。 |
| `511010.SH` 国泰上证5年期国债ETF | `2026-04-09` 最新规模 `39.05 亿元`，近 20 个交易日日均成交额 `9.65 亿元`；当日净申购 `2246.1 万元`，近 1 月净申购 `4.52 亿元`。另据 `2026-03-31` 年报解读，`2025` 年期末净资产 `30.21 亿元`，较上年末增长 `38.60%`。 | 资金承接和流动性都不差，但更容易受“权益市场偏强 + PMI 回升 + 政府债供给”压制，弹性与波动都高于短融。 | `-0.10% ~ +0.18%`，大概率窄幅震荡。 | `-0.20% ~ +0.45%`，若风险偏好回落则占优，若股强债弱则表现平淡。 |
| `511060.SH` 海富通上证5年期地方政府债ETF | `2026-03-25` 净申购 `0.54 亿元`，占前一日规模 `3.87%`，最新规模 `14.35 亿元`；`2026-03-11` 也有 `2141.11 万元` 净流入、规模升至 `13.81 亿元`。 | 有阶段性资金流入，但体量最小、公开数据完备度也最低，说明它更像“卫星仓候选”，不适合放在防守组合核心。 | `-0.12% ~ +0.20%`，受资金扰动更明显。 | `-0.30% ~ +0.50%`，上限不一定差，但确定性弱于前两只。 |

### 投决会线

- 2026-04-11 CST：本节为“基于最新公开数据的委员会补充意见”，不是新的正式投决 Tool 产物；它的作用是解释为什么在正式链仍为 `avoid` 的情况下，管理层视角下这 3 只 ETF 的优先级会出现分化。
- 委员会补充共识：
- `511360.SH`：补充意见从“直接回避”放宽到“优先观察名单第 1 位”。理由是流动性、规模、资金面和宏观环境最匹配当前“保本优先”目标，但由于正式评分卡仍 `model_unavailable`，且未补充完整债券 ETF 专用证据包，所以还不能把它从“观察”直接升格为“执行”。
- `511010.SH`：补充意见为“次优观察”。它的中等久期在债市震荡期仍有防守属性，但相比短融，它对风险偏好回升、政府债供给与曲线波动更敏感，因此不宜在正式链尚未放行时提前上升为核心仓。
- `511060.SH`：补充意见维持“谨慎观察/暂不优先”。地方债 ETF 的阶段性资金流入确实存在，但体量偏小、数据补证完整度偏弱，意味着它更适合作为未来的补充选择，而不是当前防守组合的第一选择。
- 委员会补充排序：`511360.SH > 511010.SH > 511060.SH`。

### 主席评估

- 主席补充评估结论：
- 既有正式裁决 `avoid` 不改写。原因不是这 3 只 ETF 本身出现了明确利空，而是当前正式链缺少债券 ETF 专用评分卡模型与更完整证据包，不能把公开数据补录直接冒充为正式放行。
- 如果后续用户坚持在这 3 只中做“低波动、防守型、场内可交易”的候选池，主席允许保留一个“观察优先级”：
- `第一优先观察`：`511360.SH`
- `第二优先观察`：`511010.SH`
- `第三优先观察`：`511060.SH`
- 主席对未来短周期表现的补充判断：
- `511360.SH` 更像资金停泊工具，目标是稳，不是涨。
- `511010.SH` 有一定波段属性，但上涨空间和回撤风险都比 `511360.SH` 大。
- `511060.SH` 若后续资金继续集中流入，短期相对收益可能不差，但确定性最弱。

### 对持仓计划影响

- 现有 3 条 `position-plan:*:2026-04-11:v1` 持仓计划继续保留，不删除，原因是它们仍然是后续复盘的主锚点。
- 现有正式链状态保持不变：仍以“2026-04-11 正式会后复核”的 `avoid` 为执行口径。
- 新增的公开数据补录只改变“观察优先级”，不改变“正式执行状态”：
- `position-plan:511360.SH:2026-04-11:v1`：从“泛化计划中”细化为“优先观察”
- `position-plan:511010.SH:2026-04-11:v1`：从“泛化计划中”细化为“次优观察”
- `position-plan:511060.SH:2026-04-11:v1`：从“泛化计划中”细化为“谨慎观察”

### 数据来源

- `511360.SH`：
- [4月2日海富通中证短融ETF(511360)获净申购5.66亿元，位居当日债券ETF净流入排名1/53](https://finance.sina.com.cn/money/fund/aiassistant/etfshzj/2026-04-03/doc-inhtetza0394062.shtml)
- [短融ETF海富通(511360)折溢价率-基金频道-证券之星](https://fund.stockstar.com/funds/f10/fundzyj_511360.html)
- `511010.SH`：
- [4月9日国泰上证5年期国债ETF(511010)获净申购2246.1万元，位居当日债券ETF净流入排名10/53](https://finance.sina.com.cn/money/fund/aiassistant/etfshzj/2026-04-10/doc-inhtyrvy1328493.shtml)
- [国泰上证5年期国债ETF年报解读：净利润骤降85% 份额增长39.9%凸显机构配置需求](https://finance.sina.com.cn/stock/aigc/fundfs/2026-03-31/doc-inhswawe2159401.shtml)
- [国泰上证5年期国债ETF(511010)拆溢价率-基金频道-证券之星](https://fund.stockstar.com/funds/f10/fundzyj_511010.html)
- `511060.SH`：
- [3月25日最受青睐债券ETF：海富通上证5年期地方政府债ETF净申购占规模比例为3.87%，国泰上证5年期国债ETF净申购占规模比例为3.84%（名单）](https://finance.sina.com.cn/money/fund/aiassistant/etfshzj/2026-03-26/doc-inhshnxf9119171.shtml)
- [ETF资金榜 | 5年地方债ETF(511060)：净流入2141.11万元，居全市场第一梯队-20260311](https://finance.sina.com.cn/jjxw/2026-03-12/doc-inhqstev8755871.shtml)
- 宏观债市：
- [2026年04月10日债市午盘](https://news.10jqka.com.cn/20260410/c675892066.shtml)
- [〖债市观察〗跨季资金宽松支撑短债走强 PMI回升制约长端利率下行](https://finance.sina.com.cn/money/bond/2026-04-07/doc-inhtrumi7337765.shtml)
- [时值一季度末国债逆回购收益率却偏低，机构解析：资金面宽松成核心主因](https://www.nbd.com.cn/articles/2026-04-04/4325494.html)
# 2026-04-12 P8 实盘生命周期补充

- 本节新增原因：`P8` 已把 `condition_review / execution_record / post_trade_review` 做成正式对象，持仓台账后续必须保留同一条生命周期引用链。
- 本节使用目的：以后每次建仓、调仓、冻结、复盘，都能沿着 `position_plan_ref -> condition_review_ref -> execution_record_ref -> post_trade_review_ref` 追溯。

## 生命周期记录规则

- `condition_review_ref`：记录最近一次正式条件复核对象编码；没有触发复核时留空。
- `execution_record_ref`：记录最近一次正式执行事件编码；未成交或仅研究观察时留空。
- `post_trade_review_ref`：记录最近一次正式投后复盘编码；未复盘时留空。
- `governance_feedback_action`：记录最近一次复盘建议的治理动作，例如 `continue_shadow / retrain / downgrade / freeze_consumption`。
- `lifecycle_status`：记录当前生命周期状态，例如 `recorded / filled / completed / frozen`。

## 生命周期台账模板

| 标的 | 持仓编码 | 条件复核编码 | 执行记录编码 | 投后复盘编码 | 生命周期状态 | 治理反馈动作 | 备注 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `示例.SH` | `position-plan:示例.SH:YYYY-MM-DD:v1` | `condition-review:示例.SH:YYYY-MM-DD:manual_review:v1` | `execution-record:示例.SH:YYYY-MM-DD:build:v1` | `post-trade-review:示例.SH:YYYY-MM-DD:completed:v1` | `completed` | `continue_shadow` | 按实际生命周期逐步补齐 |

## 实盘补录顺序

1. 先登记 `position_plan_ref`。
2. 触发盘中或日终复核时，再补 `condition_review_ref`。
3. 实际成交或冻结动作发生后，补 `execution_record_ref`。
4. 完成阶段性复盘后，补 `post_trade_review_ref` 和 `governance_feedback_action`。
5. 如果后续又发生新的复核或调仓，按时间顺序覆盖“最近一次”引用，并在备注里保留关键变化。
# 2026-04-12 生命周期验证切片

- 2026-04-12 CST：新增一条正式生命周期验证切片记录，原因是 `P9/P10` 需要把 `approval -> condition_review -> execution_record -> post_trade_review -> package_revision` 固化成可重复回放的样本，而不是只停在临时测试命令里。
- 目的：后续补数据、补治理、补复盘时，都沿同一条验证切片检查主链没有断。

| 验证切片 | 标的 | 分析日期 | 决策锚点 | 审批锚点 | 条件复核锚点 | 执行锚点 | 投后复盘锚点 | Manifest | 说明 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `601916_SH_2026-04-12_lifecycle` | `601916.SH` | `2026-04-12` | `decision_ref:27808:1775945190641680100` | `approval_ref:27808:1775945190641692200` | `condition-review:601916.SH:2026-04-12:manual_review:v1` | `execution-record:601916.SH:2026-04-12:build:v1` | `post-trade-review:601916.SH:2026-04-12:completed:v1` | `.excel_skill_runtime/validation_slices/601916_SH_2026-04-12_lifecycle/validation_slice_manifest.json` | 固定用于回放验证，不用于直接生成投资结论。 |
