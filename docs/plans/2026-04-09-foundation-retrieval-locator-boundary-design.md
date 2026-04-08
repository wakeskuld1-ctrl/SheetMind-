# Foundation Retrieval Locator Boundary Design

## 1. 背景

当前 foundation retrieval 已经具备最小 locator hygiene 能力：

- 支持 `A1`
- 支持 `A1:B3`
- 支持带 sheet 前缀和 `$` 标记的 A1 风格引用
- 继续把 named range 视为 weak locator

但当前 locator hygiene 还缺一个常见桌面表格边界：

- 带 Windows 绝对路径前缀的外部工作簿范围 locator，例如 `C:\Reports\[Budget.xlsx]Sheet1!A1:B3`

这类 locator 在真实 Excel/WPS 证据里并不少见。
如果 foundation 继续把它们全部视为无法解析，就会把本来可接受的定位误诊成 weak locator。

## 2. 目标

本轮只补一个保守边界：

- 让 locator hygiene 接受“Windows 绝对路径前缀 + 外部工作簿前缀 + sheet 前缀 + A1 风格范围定位”

本轮不做：

- 3D 引用解析
- 命名区域解析
- 完整公式解析
- 排序改动
- hit creation 改动
- CLI / Tool / GUI 接线

## 3. 方案比较

### 方案 A：支持 Windows 绝对路径前缀的外部工作簿范围 locator

做法：

- 在现有 locator 归一化链上先剥离 `C:\Reports\[Workbook.xlsx]Sheet!` 这类前缀
- 保持后续单点 / 范围解析逻辑不变

优点：

- 风险最小
- 完全沿现有 A1 风格解析链延伸
- 最符合本轮“补保守边界”的目标

缺点：

- 只覆盖一种 locator 变体，推进粒度较小

### 方案 B：同时支持外部前缀和 3D 引用

做法：

- 一次把 `[Book]Sheet!A1` 和 `Sheet1:Sheet3!A1` 都纳入

优点：

- 覆盖面更广

缺点：

- 容易把本轮从“补边界”做成“扩解析器”
- 测试与实现复杂度上升

### 方案 C：只补 diagnostics 文案，不扩 locator 解析

做法：

- 不改解析，只补更明确的 weak locator 说明

优点：

- 完全无解析风险

缺点：

- 不能解决真实可解析 locator 被误诊的问题

### 结论

采用方案 A。

原因：

- 当前最需要的是继续把 locator hygiene 做成“最小但实用”的桌面表格边界集合
- Windows 绝对路径外部工作簿范围是一个真实缺口，而且修补面很小
- 不会把 foundation retrieval 拉向完整 Excel 语义解析器

## 4. 设计

### 4.1 新支持的 locator 形态

本轮新增接受：

- `C:\Reports\[Budget.xlsx]Sheet1!A1:B3`
- `C:\Reports\[Budget.xlsx]'Sales Detail'!$A$1:$D$5`

### 4.2 仍然保持不支持

- `Sheet1:Sheet3!A1`
- `RevenueNamedRange`
- 复杂公式引用

这些形态在当前轮次仍然应保持 weak locator。

### 4.3 实现边界

继续只在 `src/ops/foundation/retrieval_engine.rs` 内处理：

- 优先复用 `normalize_locator_cell_text()`
- 不改 `locator_precision_priority()` 的分层模型
- 不引入新配置对象
- 不让 hygiene diagnostics 参与排序

## 5. 测试策略

严格按 TDD：

1. 先补红灯测试：Windows 绝对路径外部工作簿小范围 locator 不应被标 weak
2. 再补红灯测试：Windows 绝对路径外部工作簿绝对范围 locator 不应被标 weak
3. 跑定向测试，确认当前失败
4. 做最小实现
5. 跑全量 `retrieval_engine_unit`
6. 跑 foundation 最小回归集

## 6. 风险

- 如果前缀剥离规则写得过宽，可能误伤当前仍应保持 weak 的复杂 locator
- 如果顺手把 3D 引用也做进去，会超出本轮 scope
- 如果把 locator hygiene 扩散到排序层，会破坏当前 explainability 边界

## 7. 完成标准

- Windows 绝对路径外部工作簿范围 locator 不再被误判为 weak
- named range 与其他未支持形态仍保持原合同
- `retrieval_engine_unit` 继续通过
- foundation 最小回归集继续通过
- execution notes、handoff、task journal 完成同步
## 8. 后续补强记录（2026-04-09 CST）

- 在方案 A 落地后，又追加了一轮同主题保护测试
- 新增确认：
  - `C:\Reports\[Budget.xlsx]Sheet1!A1:Z200`
  - `C:\Reports\[Budget.xlsx]'Sales Detail'!$A$1:$Z$200`
  这两类 Windows 路径前缀 locator 虽然现在可以被正确解析，但仍然因为范围过大而保留 weak locator 语义
- 这轮保护测试直接通过，说明当前设计已经满足“路径前缀支持”和“面积阈值保留”两层边界
- 因此后续 AI 不应再把这个方向误判为待修 bug，而应把它视为已确认合同
