# Technical Consultation OBV Design

## 背景

当前 `technical_consultation_basic` 已经具备：

- 趋势方向：`trend_bias`
- 趋势强度：`trend_strength`
- 动量与波动：`momentum_signal / volatility_state`
- 一批价格型指标：`EMA / SMA / MACD / RSI / BOLL / ATR / ADX`

但现在仍缺少量价确认层。也就是说，系统能判断“价格偏多且趋势较强”，却还不能回答“这段上涨有没有量能配合”。

## 目标

在不新增 Tool、不重开架构的前提下，继续沿 `technical_consultation_basic` 这条 Rust 主线补上第一层量价确认能力：

- 指标快照新增 `obv`
- 指标快照新增 `volume_sma_20`
- 指标快照新增 `volume_ratio_20`
- 顶层结果新增 `volume_confirmation`

## 边界

本轮只做量价确认的第一层，不做复杂背离分析。

本轮包含：

- `OBV` 最终值计算
- `20` 日成交量均值
- 最新成交量相对 `20` 日均量比值
- 顶层 `volume_confirmation` 规则
- 对摘要、建议、观察点做轻量补充

本轮不包含：

- 不新增第二个股票分析 Tool
- 不补 `OBV` 背离识别
- 不补 `OBV EMA` 外部合同
- 不补 `RSRS`
- 不接 Skill

## 第一版规则

第一版只给出轻量量价确认标签：

- `confirmed`
  适用于当前方向明确，且最新量比高于均量，表示量能与方向基本一致。
- `weakening`
  适用于当前方向明确，但最新量比低于均量，表示方向仍在但确认力度减弱。
- `neutral`
  适用于横盘或量价没有形成清晰确认。

说明：

- 这轮虽然把 `OBV` 算进来了，但第一版外部合同先只要求暴露 `obv` 数值，不在本轮额外新增 `obv_signal`。
- `OBV` 的意义主要是为下一刀量价规则扩展做底座。

## 测试策略

继续坚持 TDD，直接复用当前已经写好的红测：

1. 上涨放量样本：
   - `volume_confirmation == "confirmed"`
   - `volume_ratio_20 > 1.0`
2. 上涨缩量样本：
   - `volume_confirmation == "weakening"`
   - `volume_ratio_20 < 1.0`
3. 指标快照必须含：
   - `obv`
   - `volume_sma_20`
   - `volume_ratio_20`

## 后续承接

这轮完成后，最自然的下一刀有两个方向：

1. 在同一模块里继续补 `OBV` 的量价背离规则
2. 继续补 `RSRS`

建议顺序仍然保持：

- 一次只补一类指标家族
- 先红测再实现
- 不要为新增指标重新开架构
