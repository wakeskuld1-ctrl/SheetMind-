# Technical Consultation False Breakdown None Boundary Design

## 背景

当前 `technical_consultation_basic` 已经具备以下已锁定合同：

- 能识别 `bearish_divergence`
- 能识别 `bullish_divergence`
- 价格与 OBV 同步确认突破时保持 `none`
- 价格与 OBV 同步确认下破时保持 `none`
- 仅有 OBV 回落、价格未形成有效突破时保持 `none`

同时，工作区里已经并行存在 `KDJ / timing_signal` 相关改动。本轮不重开架构讨论，也不混做 KDJ 收口，而是继续沿既定 `technical_consultation_basic` 主线，把 divergence 再补一个更细的 `should-stay-none` 边界。

## 目标

在不新增 Tool、不调整 `Rust / exe / SQLite` 主链、不修改对外 JSON 合同的前提下，新增并锁定一条边界：

- 价格在低位出现“假跌破”或仅是低位震荡
- OBV 没有形成足以支持底背离的改善
- `divergence_signal` 必须保持 `none`

这条边界的目的不是扩展新的分类，而是防止当前底背离识别在后续演进中，把“尚未形成有效背离的低位波动”误报成 `bullish_divergence`。

## 方案对比

### 方案 A：补一个低位假跌破 / 低位震荡 `none` 专项回归

- 做法：新增一个更贴近“价格试探新低但量能结构并未改善”的夹具与测试；先跑红测，再决定是否最小修改 `classify_divergence_signal()`
- 优点：风险最低，最符合当前“渐进补边界”的主线
- 缺点：本轮只补一条边界，不扩展更复杂的结构识别

### 方案 B：直接把底背离改成 swing 结构识别

- 做法：重写当前窗口比较逻辑，引入更复杂的局部高低点识别
- 优点：理论表达力更强
- 缺点：明显超出本轮范围，容易再次滑向架构重写

### 方案 C：只写 handoff，不加自动化测试

- 做法：只在文档里说明“低位假跌破不算底背离”
- 优点：代码改动最小
- 缺点：没有自动化约束，后续最容易回归

本轮采用：**方案 A**

## 边界定义

本轮的“假跌破 / 低位震荡”按以下意图约束：

- 最近窗口里价格可以触碰或轻微跌破前低
- 但整体表现仍接近低位反复拉扯，而不是明确形成“价格创新低、OBV 先行抬高”的经典底背离
- 换句话说，只有当价格新低与 OBV 改善形成明确反向关系时，才允许输出 `bullish_divergence`

因此本轮强调的是：

- 低位波动本身不等于底背离
- 假跌破本身不等于底背离
- 必须先锁住误报，再考虑以后是否增加更细的结构分类

## 非目标

本轮不做以下内容：

- 不新增 `divergence_score`、`swing_points`、`obv_signal`
- 不调整 `timing_signal` / `KDJ` 逻辑
- 不改 `dispatcher / catalog / runtime`
- 不新增新的技术咨询 Tool
- 不重构 `technical_consultation_basic`

## 测试策略

继续严格采用 TDD：

1. 在 `tests/technical_consultation_basic_cli.rs` 新增“低位假跌破 / 低位震荡”的夹具
2. 新增 `technical_consultation_basic_keeps_none_when_false_breakdown_lacks_obv_divergence`
3. 先跑专项测试，确认当前行为是否已经满足
4. 如果失败，只允许最小修改 `src/ops/technical_consultation_basic.rs` 内的 `classify_divergence_signal()`
5. 最后跑 `technical_consultation_basic` 回归与全量测试

## 后续承接

这条边界锁住后，divergence 主线的后续顺序仍建议保持：

1. 继续补 `should-stay-none` 边界
2. 再评估是否值得增加更细的底部 / 顶部结构样本
3. 最后才考虑是否需要更复杂的 swing 级规则

这样可以保证我们始终沿当前架构持续推进，而不是每次新会话又回到“要不要重构”。
