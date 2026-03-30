# Technical Consultation Divergence Phase 2 Design

## Goal

在现有 `technical_consultation_basic` 不重构的前提下，继续补齐第一版背离识别的合同完整性：新增 `bullish_divergence` 专项样本，并补两类误判边界测试。

## Scope

- 保持现有 Rust / EXE / SQLite 主线不变。
- 保持顶层合同仍为 `divergence_signal = bearish_divergence / bullish_divergence / none`。
- 本轮不新增新的快照字段，不拆新 Tool，不引入更复杂 swing 背离模型。

## Chosen Approach

采用渐进式方案 A：

1. 先补 `bullish_divergence` 红测，锁定“价格创新低但 OBV 未同步创新低”。
2. 再补两个边界测试，锁定应保持 `none` 的场景。
3. 只在现有 `classify_divergence_signal()` 内做最小实现修正。

## Tradeoffs

- 优点：最符合当前“非必要不重构”的路线，能优先压住误判风险。
- 缺点：本轮主要是合同补强，不会扩展到更高级背离结构。

## Testing Strategy

- 必须先写失败测试，再实现。
- 新增测试集中在 [tests/technical_consultation_basic_cli.rs](D:/Rust/Excel_Skill/.worktrees/SheetMind-/tests/technical_consultation_basic_cli.rs)。
- 回归顺序固定为：
  - `cargo test --test technical_consultation_basic_cli -- --nocapture`
  - `cargo test --test stock_price_history_import_cli -- --nocapture`
  - `cargo test`

## Non-Goals

- 本轮不做 `RSRS`、不做更多高级结构背离。
- 本轮不改 Tool 注册、Dispatcher、SQLite 结构。
