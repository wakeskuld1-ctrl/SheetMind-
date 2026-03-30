# Technical Consultation Bullish Divergence Design

## 背景

当前 `technical_consultation_basic` 已经具备：
- 趋势方向：`trend_bias`
- 趋势强度：`trend_strength`
- 量价确认：`volume_confirmation`
- 第一版背离输出：`divergence_signal`

并且 `bearish_divergence` 已经有专项回归样本锁定，但 `bullish_divergence` 还没有独立红测样本。这样会带来两个问题：
- 现有 `bullish_divergence` 分支虽然已经存在，但没有被真实场景回归锁住
- 后续 AI 或工程继续扩展背离规则时，容易只围绕顶背离优化，底背离被误伤而不自知

## 目标

在不新增 Tool、不重开架构、不脱离 `Rust / exe / SQLite` 主线的前提下，继续沿 `technical_consultation_basic` 做一刀最小增量：

- 新增一个“价格创新低但 OBV 不再创新低”的专项样本
- 先用红测锁住 `bullish_divergence`
- 如红测失败，再最小修改现有 `classify_divergence_signal()`
- 保持现有顶层合同不变，只继续复用 `divergence_signal = bearish_divergence / bullish_divergence / none`

## 边界

本轮只做底背离的专项锁定与最小实现，不做下面这些事情：

- 不新增第二个股票分析 Tool
- 不改 dispatcher / catalog / runtime 主链
- 不新增更复杂的 swing 结构背离分类
- 不新增 `obv_signal`、`divergence_score`、`swing_points` 等新字段
- 不同时处理多窗口参数配置化

## 方案对比

### 方案 A：先补 `bullish_divergence` 红测，再最小实现

- 做法：新增底背离样本生成器和专项测试，再按测试结果最小修改现有窗口级背离规则
- 优点：最符合当前渐进节奏，风险最低，直接补齐现有能力缺口
- 缺点：本轮只补一个能力点，边界保护测试仍要下一轮继续补

### 方案 B：先补边界测试，不扩能力

- 做法：先补“同步创新低时仍为 none”等保护测试
- 优点：可进一步收紧误报
- 缺点：底背离能力仍没有专门样本锁定

### 方案 C：直接升级成更复杂的结构背离

- 做法：引入更细的 price swing / OBV swing 比对
- 优点：理论上更强
- 缺点：明显超出本轮最小增量边界，容易把当前切片做重

本轮采用：**方案 A**

## 测试策略

继续严格使用 TDD：

1. 在 `tests/technical_consultation_basic_cli.rs` 新增底背离样本生成器
2. 新增 `technical_consultation_basic_marks_price_obv_bullish_divergence`
3. 先跑专项测试，确认它先失败
4. 再最小修改 `src/ops/technical_consultation_basic.rs`
5. 回归：
   - `cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_price_obv_bullish_divergence -- --nocapture`
   - `cargo test --test technical_consultation_basic_cli -- --nocapture`
   - `cargo test --test stock_price_history_import_cli -- --nocapture`
   - `cargo test -- --nocapture`

## 后续承接

本轮完成后，下一刀仍建议继续留在 `technical_consultation_basic` 内，优先顺序如下：

1. 补“价格创新低且 OBV 同步创新低时必须保持 `none`”的边界测试
2. 补“假跌破 / 低位震荡”不应误报底背离的边界测试
3. 再考虑窗口长度稳定性，而不是提前新开独立 divergence 模块
