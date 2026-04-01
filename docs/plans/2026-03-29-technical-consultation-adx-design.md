# Technical Consultation ADX Design

## 背景

当前 `technical_consultation_basic` 已经能基于 `stock_history.db` 输出：

- `trend_bias`
- `momentum_signal`
- `volatility_state`
- `summary`
- `recommended_actions`
- `watch_points`
- `indicator_snapshot`

但现有趋势判断主要依赖 `close / ema_10 / sma_50 / sma_200` 的相对位置，缺少“趋势是否已经成立”的强度层。这会导致横盘样本在短期均线偶尔排成多头时，也可能被误判为 `bullish`。

## 目标

在不新增 Tool、不重开架构、不中断 `Rust / exe / SQLite` 主线的前提下，为 `technical_consultation_basic` 增加第一层中级趋势强度能力：

- 新增 `ADX / +DI / -DI`
- 新增顶层字段 `trend_strength`
- 在弱趋势时把方向自动降级为 `sideways`

## 边界

本轮只做以下内容：

- 在 `src/ops/technical_consultation_basic.rs` 内实现 `adx_14 / plus_di_14 / minus_di_14`
- 扩展 `indicator_snapshot`
- 扩展顶层结果合同 `trend_strength`
- 让摘要、建议、观察点能够体现趋势强弱
- 用 CLI 红测锁住上涨强趋势和横盘弱趋势两个场景

本轮明确不做：

- 不新增第二个股票分析 Tool
- 不接 Skill
- 不补 `RSRS / OBV / LSTM`
- 不重构 `dispatcher / catalog / runtime`

## 规则

第一版趋势强度规则保持轻量、可解释：

- `ADX >= 25` -> `strong`
- `20 <= ADX < 25` -> `moderate`
- `ADX < 20` -> `weak`

第一版方向保护规则：

- 若 `trend_strength == weak`，则优先返回 `trend_bias = sideways`
- 否则再根据均线结构和 `+DI / -DI` 判断 `bullish / bearish / sideways`

## 测试策略

继续坚持 TDD：

1. 先补红测，锁住：
   - 上涨样本必须返回 `trend_strength = strong`
   - `indicator_snapshot` 必须含 `adx_14 / plus_di_14 / minus_di_14`
   - 横盘样本必须返回 `trend_bias = sideways`
   - 横盘样本必须返回 `trend_strength = weak`
2. 再补最小实现。
3. 最后跑：
   - `cargo test --test technical_consultation_basic_cli -- --nocapture`
   - `cargo test --test stock_price_history_import_cli -- --nocapture`
   - `cargo test -- --nocapture`

## 后续承接

这轮完成后，下一刀仍然应该继续留在 `technical_consultation_basic` 内做单指标家族增量，推荐顺序：

1. `OBV`
2. `RSRS`
3. 更多横盘高波动场景红测

原则保持不变：

- 一次只补一类指标
- 先红测再实现
- 不要因为增加指标就重新开架构
