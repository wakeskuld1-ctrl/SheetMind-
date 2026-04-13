# 2026-04-11 Foundation Source Ref Structure Diagnostics Design

## 背景

foundation `RepositoryMetadataAudit` 当前已经具备最小 `WeakSourceRef` 规则：

- `Blank`
- `TooShort`
- `MissingNamespace`

同时，`locator` 结构诊断已经推进到：

- `Blank`
- `TooShort`
- `SheetOnly`
- `SingleCellOnly`
- `AmbiguousKeyword`
- `InvalidRangeFormat`

为了让 evidence hygiene 两侧口径更对称，需要继续扩 `source_ref` 的结构性诊断。

## 目标

在 repository audit 的只读诊断层补充更细的 `source_ref` 结构规则，让后续 AI 和人工治理能区分“太短”和“结构不稳定”。

## 非目标

- 不修改 `locator` 结构规则
- 不新增 summary / severity
- 不做自动修复、自动 rewrite 或 migration executor
- 不接证券业务层

## 方案

继续扩展 `RepositoryWeakSourceRefReason`，新增：

- `EntityMissing`
  - 例：`sheet:`
  - 含义：有 namespace，但实体部分为空
- `ContainsWhitespace`
  - 例：`sheet: sales q1`
  - 含义：包含空白，说明引用结构不稳定
- `InvalidCharacter`
  - 例：`sheet:sales?2024`
  - 含义：包含当前最小规则不接受的特殊字符
- `UnknownNamespace`
  - 例：`blob:sales`
  - 含义：namespace 结构完整，但不在当前允许集合

保留已有：

- `Blank`
- `TooShort`
- `MissingNamespace`

## 当前最小判定规则

按优先级判定：

1. `Blank`
2. `TooShort`
3. `MissingNamespace`
4. `EntityMissing`
5. `ContainsWhitespace`
6. `InvalidCharacter`
7. `UnknownNamespace`

## 当前最小 namespace 集合

本轮只引入最小允许集合：

- `sheet`
- `file`
- `workbook`
- `table`
- `range`

这只是 repository audit 启发式诊断集合，不等于未来正式标准全集。

## 设计考虑

- `EntityMissing` 优先于 `ContainsWhitespace / InvalidCharacter`，因为 `sheet:` 这类值的主问题是实体缺失。
- `ContainsWhitespace` 和 `InvalidCharacter` 都是结构弱提示，不是 hard failure。
- `UnknownNamespace` 只在结构基本完整时触发，避免和更前面的弱规则重叠。

## 测试策略

在 `tests/repository_metadata_audit_unit.rs` 新增红测覆盖：

- `EntityMissing`
- `ContainsWhitespace`
- `InvalidCharacter`
- `UnknownNamespace`

并保持已有：

- `Blank`
- `TooShort`
- `MissingNamespace`

的顺序和行为不变。

## 预期结果

这轮完成后，foundation evidence hygiene 在 `source_ref` 侧将从“基础命名检查”推进到“基础结构检查”，与 `locator` 侧形成对称能力。
