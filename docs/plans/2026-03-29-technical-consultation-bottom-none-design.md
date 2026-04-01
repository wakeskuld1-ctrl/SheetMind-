# Technical Consultation Bottom None Boundary Design

## 背景

当前 `technical_consultation_basic` 已经具备第一版 `divergence_signal`，并且已经有：

- `bearish_divergence` 专项样本
- `bullish_divergence` 专项样本
- 顶部正常放量突破保持 `none`
- OBV 回落但价格未创新高保持 `none`

上一轮已经确认一个关键经验：红测不一定代表生产逻辑出错，也可能是测试夹具没有构造成真实几何形态。因此这一轮继续沿既定主线前进，但不做架构调整，也不急着扩新算法分类，而是继续补底部 should-stay-none 边界。

## 目标

在不新建 Tool、不重构 `Rust / exe / SQLite` 主链、不修改对外 JSON 合同的前提下，新增一条明确边界：

- 当价格创新低
- 且 OBV 也同步创新低
- `divergence_signal` 必须保持 `none`

这条边界的目的不是新增能力，而是防止当前底部背离识别在后续演进中把“确认性下跌”误判成 `bullish_divergence`。

## 方案对比

### 方案 A：补一条底部 `none` 专项回归

- 做法：新增一个“价格与 OBV 同步新低”的夹具和一个专门测试，先看是否已被当前实现满足；若不满足，再最小修改 `classify_divergence_signal()`
- 优点：风险最低，直接加强当前合同边界，最符合既定渐进路径
- 缺点：本轮只补一个边界，不扩更复杂的结构背离

### 方案 B：直接重写底部背离规则

- 做法：把当前窗口比较升级成更复杂的 swing 结构识别
- 优点：理论上可提升表达力
- 缺点：明显超出本轮范围，容易重新打开架构和算法复杂度

### 方案 C：暂不写新测试，只在文档里补口头约束

- 做法：只写 handoff 说明，不补代码回归
- 优点：改动最少
- 缺点：没有自动化保护，后续最容易回归

本轮采用：**方案 A**

## 边界

本轮只做“底部确认型 none”回归加固，不做以下事情：

- 不新增 `divergence_score`、`obv_signal`、`swing_points` 等新字段
- 不修改 dispatcher / catalog / runtime
- 不引入第二条股票分析链
- 不重构 `technical_consultation_basic`
- 不扩展成多级背离分类体系

## 测试策略

严格继续使用 TDD：

1. 在 `tests/technical_consultation_basic_cli.rs` 新增一个“价格与 OBV 同步创新低”的夹具
2. 新增 `technical_consultation_basic_keeps_none_when_price_and_obv_confirm_breakdown`
3. 先跑专项测试，确认当前行为到底是红还是绿
4. 如果失败，只允许最小修改 `src/ops/technical_consultation_basic.rs` 内的 `classify_divergence_signal()`
5. 追加相关回归与全量验证

## 后续承接

这条边界锁住以后，`technical_consultation_basic` 这一支在 divergence 方向上更稳的下一步顺序建议仍然是：

1. 继续补更多 should-stay-none 边界
2. 再决定是否补更细的底部 / 顶部结构样本
3. 最后才评估是否值得引入更复杂的 swing 级规则

这样可以保证我们一直沿当前架构推进，而不是每次新会话都回到“要不要重构”。
