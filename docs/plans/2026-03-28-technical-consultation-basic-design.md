# Technical Consultation Basic Design

## 背景

当前 Rust 主线已经补齐了股票历史数据的第一刀底座：

- `import_stock_price_history` 已能把单个股票 CSV 导入 `stock_history.db`
- 历史数据已经和现有 runtime 根目录对齐，后续 Tool 可以复用同一份 SQLite
- 现阶段还没有一个 Rust Tool 能直接读取这些历史行情并输出基础技术面结论

这意味着主线现在停在了“数据已入库，但分析能力还没接上”的阶段。  
如果继续只补导入或只补文档，Rust / exe 主线就还不能真正回答“这只股票当前偏多还是偏空、波动是否偏高、下一步应该观察什么”。

因此下一刀最合理的增量，不是继续打磨交付层，也不是先补更高级指标，而是新增一个稳定、保守、可解释的 Rust 技术面基础 Tool。

## 目标

新增 Rust Tool `technical_consultation_basic`，直接从 `stock_history.db` 读取指定 `symbol` 的日线历史，计算第一批基础技术指标，并返回可供后续 Skill / AI / CLI 直接消费的稳定 JSON 合同。

本轮至少覆盖：

- 历史数据读取
- 日期升序排序
- `EMA(10)`
- `SMA(50)`
- `SMA(200)`
- `MACD(12,26,9)`
- `RSI(14)`
- `BOLL(20,2)`
- `ATR(14)`
- 基于这些指标生成
  - `trend_bias`
  - `momentum_signal`
  - `volatility_state`
  - `indicator_snapshot`
  - `recommended_actions`
  - `watch_points`

## 非目标

- 不引入 Python 运行时
- 不新增第二套股票架构
- 不在本轮接 Skill
- 不直接补 `RSRS / ADX / OBV / LSTM`
- 不做在线抓数
- 不做多股票批量分析
- 不做指标缓存表

## 方案对比

### 方案 A：直接做 Rust `technical_consultation_basic` Tool

做法：

- 新增独立 Rust Tool
- 直接读取 SQLite 历史行情
- 在 Tool 内部完成基础指标计算与轻规则结论生成

优点：

- 最符合当前主线：`CSV -> SQLite -> Rust Tool`
- 最快形成“可调用能力”
- 后续 Skill、报表、AI 承接都能复用

缺点：

- 这刀的测试和边界处理比单纯交付层增强更多
- 需要先钉死指标口径和返回合同

### 方案 B：先增强 SQLite 层，再推迟技术面 Tool

做法：

- 继续补批量导入、更多 CSV 兼容和数据校验
- 技术面 Tool 延后

优点：

- 数据底座更厚
- 输入质量更稳

缺点：

- 分析能力仍然缺位
- 容易陷入“底座一直在补，能力迟迟不交付”

### 方案 C：先补更高级指标，再一起做技术面 Tool

做法：

- 先设计 `RSRS / ADX / OBV` 甚至更复杂模型
- 一次性做成更完整的技术面 Tool

优点：

- 一次性交付的算法看起来更多

缺点：

- 风险更高
- 合同更难稳定
- 不符合“先基础、后扩展”的节奏

本轮采用 **方案 A**。

## 设计原则

### 1. 先把基础能力打通，再考虑高级指标

本轮只做第一批经典基础指标，把历史数据读取、窗口处理、结果合同、调度接入先跑通。  
后续再在同一个 Tool 或相邻 Tool 上补 `RSRS / ADX / OBV`。

### 2. 规则轻量、输出可解释

`trend_bias / momentum_signal / volatility_state` 必须能回溯到明确指标，不做黑盒评分。

例如：

- 趋势优先看 `EMA(10)`、`SMA(50)`、`SMA(200)` 的相对位置
- 动量优先看 `MACD` 与 `RSI`
- 波动优先看 `BOLL` 与 `ATR`

### 3. 沿现有 Rust Tool 主链接入

保持当前接入方式不变：

- `src/ops`
- `src/runtime`
- `src/tools/catalog.rs`
- `src/tools/dispatcher.rs`
- `src/tools/dispatcher/analysis_ops.rs`

不重开新的股票编排层。

### 4. 历史数据不足时要保守降级

如果数据长度不足以支撑长窗口指标，Tool 不能崩溃，也不能假装算出了完整结论。  
应返回明确中文错误或保守提示，让上层知道当前是“数据不足”，而不是“趋势未知但可继续决策”。

## 建议合同

建议请求合同：

- `symbol`: 必填
- `lookback_days`: 可选，默认读取最近 260 个交易日

建议返回合同：

- `symbol`
- `as_of_date`
- `history_row_count`
- `trend_bias`
- `momentum_signal`
- `volatility_state`
- `summary`
- `recommended_actions`
- `watch_points`
- `indicator_snapshot`

其中 `indicator_snapshot` 至少包含：

- `close`
- `ema_10`
- `sma_50`
- `sma_200`
- `macd`
- `macd_signal`
- `macd_histogram`
- `rsi_14`
- `boll_upper`
- `boll_middle`
- `boll_lower`
- `atr_14`

## 指标口径

### 趋势

优先规则：

- `close > ema_10 > sma_50 > sma_200` 倾向 `bullish`
- `close < ema_10 < sma_50 < sma_200` 倾向 `bearish`
- 其余保守落为 `sideways`

### 动量

优先规则：

- `macd > macd_signal` 且 `rsi_14 >= 55` 倾向 `positive`
- `macd < macd_signal` 且 `rsi_14 <= 45` 倾向 `negative`
- 否则 `neutral`

### 波动

优先规则：

- `close` 明显贴近或突破布林上下轨，或 `atr_14 / close` 偏高，给 `high`
- 区间较窄时给 `normal`

本轮不追求复杂市场状态识别，只追求稳定、可解释、可测试。

## 错误处理

- `symbol` 无历史数据：返回明确中文错误
- 历史数据条数不足：返回明确中文错误，并指出至少需要多少条
- SQLite 打开失败：沿存储层中文错误向上返回
- 指标计算过程中出现空窗口：统一按“数据不足”处理

## 测试策略

本轮至少覆盖以下测试：

1. `tool_catalog` 可发现 `technical_consultation_basic`
2. 从 SQLite 成功读取历史数据并输出稳定合同
3. 历史数据按日期乱序导入后，内部仍能按时间正确计算
4. 数据不足 200 条时，返回明确中文错误
5. 趋势多头场景能落到 `bullish`
6. 趋势空头场景能落到 `bearish`
7. 动量和波动字段至少有一组稳定断言

## 下一步

实施时继续坚持：

- 先写红测
- 先验证红测确实失败
- 再做最小实现
- 再跑专项测试
- 最后跑全量 `cargo test`
