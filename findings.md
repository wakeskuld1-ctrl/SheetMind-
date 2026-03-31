# Findings
## 2026-03-31（多根回踩补充）
### 关键发现
- 仅依赖“前一根突破/跌破、当前一根确认”的两根 K 线口径，会把真实存在的 2~4 根回踩磨位样本误判为 `range_bound`。
- 多根结构不需要新字段；只要保留旧关键位锚点，并确保中间 bars 没有明显走坏，就可以继续落到既有 `confirmed_* / failed_* / watch` 语义。

### 本轮决策
- 在最近 `4` 根内扫描更早的突破/跌破锚点，复用该锚点对应的旧关键位作为后续多根结构的判断基准。
- 中间 bars 继续使用统一灰区约束：多头侧不允许明显跌破 `level - buffer`，空头侧不允许明显上破 `level + buffer`。
- 保持对外合同不变，仍由 `breakout_signal` 表达结构结论，避免扩字段带来上层适配成本。

### 原因
- 这条主线目标是把日线关键位结构做成稳定可消费的咨询合同，而不是做高频执行模型。
- 多根回踩/反抽是日线样本里的常见节奏，如果不补这层，`summary / recommended_actions / watch_points` 会在真实样本里频繁退化成区间态。

### 风险提示
- 当前回看窗口固定为 `4` 根，若后续样本存在更长时间的横向磨位，仍可能需要继续扩窗口或参数化。
- 当前多根结构仍依赖收盘价关键位口径，若未来改成影线极值口径，现有样本与文案都要一起调整。


## 2026-03-31（补充）
### 关键发现
- 如果价格只是刚回到旧关键位附近，直接判成 `confirmed_resistance_retest_hold / confirmed_support_retest_reject` 或 `failed_*`，会把“正在测试关键位”的样本过早定性。
- `retest_watch` 需要一个和价格水平相关的灰区口径，否则低波动品种与高波动品种会被同一条硬阈值误伤。

### 本轮决策
- 为旧关键位附近引入灰区 buffer：`max(snapshot.atr_14 * 0.25, 0.15)`。
- 当价格仍停留在旧关键位灰区附近时，分别输出 `resistance_retest_watch` 或 `support_retest_watch`，表示回踩/反抽仍在观察中。
- `confirmed_resistance_retest_hold / confirmed_support_retest_reject` 只有在价格重新站稳或压回，并且相对旧关键位拉开明显距离后才触发。
- `failed_resistance_breakout / failed_support_breakdown` 也同步收紧为“反向离开旧关键位灰区并拉开距离”后才触发。

### 原因
- 当前主线是日线级 `technical_consultation_basic` 结构化咨询，不应该把单根贴边波动直接翻译成完成态。
- 灰区 buffer 能让“观察态”和“确认态/失败态”之间有稳定的合同边界，减少 summary、actions 和 watch_points 的语义跳变。

### 风险提示
- 当前 buffer 仍是固定公式，若后续覆盖极低价股或超高价股，可能还需要进一步参数化或做比例/绝对值混合校准。
- 当前模型依然是“最近两根 K 线 + 旧关键位”最小口径，多根回踩/多根反抽还没有纳入识别。

## 2026-03-31
### 关键发现
- `build_confirmed_breakout_rows(220)` 对应的真实样本输出中：
- `trend_bias = bullish`
- `trend_strength = strong`
- `volume_confirmation = weakening`
- 这说明“关键位有效突破”不应和“放量确认”强绑为同一字段，否则会与现有 `volume_confirmation` 语义重叠并互相冲突。

### 本轮决策
- `breakout_signal` 采用价格结构口径：
- `confirmed_resistance_breakout`
- `confirmed_support_breakdown`
- `resistance_breakout_watch`
- `support_breakdown_watch`
- `confirmed_resistance_retest_hold`
- `confirmed_support_retest_reject`
- `failed_resistance_breakout`
- `failed_support_breakdown`
- `range_bound`
- `support_level_20 / resistance_level_20` 采用“排除最新一根 K 线后的近 20 日收盘极值”，而不是影线高低点。
- “假突破 / 假跌破”不靠当前快照单点判断，而是使用“前一根相对前序关键位的位置 + 当前一根是否收回当前关键位”的二阶段口径。
- “回踩确认 / 反抽受压”不要求当前一根继续创新高/创新低，而是使用“前一根已越过旧关键位 + 当前一根仍站在旧关键位正确一侧，但已回到当前窗口内”的三阶段口径。

### 原因
- 当前主线是 `technical_consultation_basic` 的结构化咨询，不是高频交易执行系统。
- 用收盘关键位比用影线关键位更稳，更贴合“有效突破/有效跌破”的日线咨询口径。
- 量能已经有独立字段，拆成双维度更利于上层 AI 组合解释。

### 风险提示
- `breakout_signal` 已补到“回踩确认 / 反抽受压”，但还没有继续细分“retest_watch”观察态。
- 当前窗口固定为 `20` 日，适合这条主线的最小版本，但不一定适合所有证券或周期。
