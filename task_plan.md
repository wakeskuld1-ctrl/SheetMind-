# Task Plan
## 2026-03-31（多根回踩补充）
### 本轮完成
- [x] 在 `technical_consultation_basic` 的既有关键位主线上补齐“多根回踩再站稳 / 多根反抽再受压”的识别能力，不新增对外模块。
- [x] 复用统一的旧关键位 + `buffer = max(snapshot.atr_14 * 0.25, 0.15)` 灰区口径，让 `watch / confirmed / failed` 三类判断都能覆盖 2~4 根磨位结构。
- [x] 补齐 CLI 回归测试：`technical_consultation_basic_marks_multi_bar_resistance_retest_hold_signal`。
- [x] 补齐 CLI 回归测试：`technical_consultation_basic_marks_multi_bar_support_retest_reject_signal`。

### 后续待办
- [ ] 评估是否继续补“多根回踩观察态 / 多根反抽观察态”的专门样本，避免未来只测确认态而漏掉灰区。
- [ ] 评估是否把 `MULTI_BAR_RETEST_LOOKBACK_BARS` 参数化，或继续保持当前固定 `4` 根窗口。
- [ ] 如继续按方案 A 推进，优先做关键位文案和样本矩阵整理，不新开证券分析模块。


## 2026-03-31（补充）
### 本轮完成
- [x] 把 `breakout_signal` 继续扩展到 `resistance_retest_watch / support_retest_watch` 观察态，补齐“正在回踩/反抽途中”和“已确认完成”之间的中间层。
- [x] 为旧关键位附近引入灰区 buffer，避免价格刚贴近旧关键位时就被过早判成 `confirmed_*` 或 `failed_*`。
- [x] 补齐 CLI 回归测试：`technical_consultation_basic_marks_resistance_retest_watch_signal`。
- [x] 补齐 CLI 回归测试：`technical_consultation_basic_marks_support_retest_watch_signal`。

### 后续待办
- [ ] 评估是否继续扩展为“多根回踩 / 多根反抽”结构，而不只依赖最近两根 K 线。
- [ ] 评估是否把 `KEY_LEVEL_LOOKBACK_DAYS` 与 retest buffer 参数化，避免不同价位区间共用同一固定口径。
- [ ] 如继续按方案 1 推进，优先补“多根回踩 / 多根反抽”的样本、文案和专项测试，不新开证券分析模块。

## 2026-03-31
### 当前任务
- [x] 方案 1：为 `technical_consultation_basic` 增加“支撑/阻力 + 突破有效性”最小能力切片。

### 本轮完成
- [x] 新增 `breakout_signal` 对外合同字段。
- [x] 新增 `indicator_snapshot.support_level_20 / resistance_level_20` 快照字段。
- [x] 在 `summary / recommended_actions / watch_points` 中接入关键位突破语义。
- [x] 补齐 CLI 回归测试：向上有效突破、向下有效跌破、默认成功合同可见性。
- [x] 把 `breakout_signal` 扩展到二阶段确认，补齐“假突破回落 / 假跌破拉回”语义。
- [x] 补齐 CLI 回归测试：`failed_resistance_breakout / failed_support_breakdown`。
- [x] 把 `breakout_signal` 扩展到三阶段确认，补齐“阻力转支撑回踩确认 / 支撑转阻力反抽受压”语义。
- [x] 补齐 CLI 回归测试：`confirmed_resistance_retest_hold / confirmed_support_retest_reject`。

### 后续待办
- [ ] 评估是否继续补“retest_watch”观察态，区分“正在回踩/反抽途中”和“已经确认完成”。
- [ ] 评估是否要把关键位窗口参数化，或继续保持固定 `20` 日窗口。
- [ ] 如要继续扩展证券分析，优先考虑“支撑转阻力/阻力转支撑的回踩确认”而不是新开模块。
