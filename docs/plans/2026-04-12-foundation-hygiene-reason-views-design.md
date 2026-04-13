# 2026-04-12 Foundation Hygiene Reason Views Design

## 背景

foundation `RepositoryMetadataAudit` 当前已经提供三层 evidence hygiene 输出：

- `hygiene_diagnostics`
  - 保留逐条明细，适合回放和精查
- `hygiene_summary`
  - 提供 severity、diagnostic type、reason count 等摘要
- `hygiene_views`
  - 提供按 `severity` 与按 `node` 的分组治理视图

这已经足够回答“仓库现在脏不脏”“优先修哪些节点”，但还不够直接回答“优先修哪一类 hygiene 原因”。对于 AI 路由和治理编排来说，这一层仍然需要在调用侧自己重建。

## 目标

在不修改现有三层结构语义的前提下，新增一层按 weak reason 聚合的只读治理视图，让 foundation 输出直接支持：

- 先看哪类 `weak_locator` 原因最集中
- 先看哪类 `weak_source_ref` 原因最阻塞
- 同时保留该 reason 对应的 severity、节点覆盖面和节点列表

## 非目标

- 不修改现有 `RepositoryHygieneSeverity` 的映射规则
- 不引入自动修复、建议修复、rewrite 或迁移动作
- 不扩展到业务层工作流
- 不重构 `hygiene_diagnostics`、`hygiene_summary`、`hygiene_views` 的既有结构

## 方案

在 `RepositoryMetadataAuditReport` 中新增：

- `hygiene_reason_views: RepositoryEvidenceHygieneReasonViews`

新增结构：

- `RepositoryEvidenceHygieneReasonViews`
  - `weak_locator_by_reason: Vec<RepositoryWeakLocatorReasonGroup>`
  - `weak_source_ref_by_reason: Vec<RepositoryWeakSourceRefReasonGroup>`

- `RepositoryWeakLocatorReasonGroup`
  - `reason: RepositoryWeakLocatorReason`
  - `severity: RepositoryHygieneSeverity`
  - `diagnostic_count: usize`
  - `affected_node_count: usize`
  - `node_ids: Vec<String>`

- `RepositoryWeakSourceRefReasonGroup`
  - `reason: RepositoryWeakSourceRefReason`
  - `severity: RepositoryHygieneSeverity`
  - `diagnostic_count: usize`
  - `affected_node_count: usize`
  - `node_ids: Vec<String>`

## 为什么选 A2

### 相比 A1

A1 只补 `by_reason`，虽然够轻，但 AI 仍然需要自己回推这个 reason 属于阻塞还是提醒级问题。

A2 直接把 `reason + severity` 放在一个聚合单元里，调用侧可以更快决定：

- 是否应该先拦截
- 是先修 source 还是先修 locator
- 是先清理高频 warning，还是先处理低频但 blocking 的 critical

### 相比 A3

A3 会把 reason 和 node 的展开层做得更重，和已有 `hygiene_views.by_node` 会出现明显重叠。

A2 保持轻量：

- 不复制完整 diagnostics
- 不引入 reason->node->diagnostics 的深层嵌套
- 只补 AI 路由真正缺的一层聚合口

## 排序规则

为了让输出稳定、可测，本轮固定：

- `weak_locator_by_reason`
  - 先按 `severity`：`Critical -> Warning -> Info`
  - 再按 `diagnostic_count` 降序
  - 最后按 reason 名称升序

- `weak_source_ref_by_reason`
  - 先按 `severity`：`Critical -> Warning -> Info`
  - 再按 `diagnostic_count` 降序
  - 最后按 reason 名称升序

说明：

- 当前 `weak_locator` reason 全部映射到 `Warning`
- 当前 `weak_source_ref` reason 可能落到 `Critical` 或 `Warning`
- 仍然保留 `Info` 排序位，避免未来扩展时打破结构

## 与现有层次的关系

- `hygiene_diagnostics`
  - 负责“原始明细”
- `hygiene_summary`
  - 负责“全局计数摘要”
- `hygiene_views`
  - 负责“按 severity / node 的治理入口”
- `hygiene_reason_views`
  - 负责“按 weak reason 的治理入口”

这四层职责分离后，AI 可以按不同入口选择：

- 明细追溯
- 全局阻塞判断
- 节点优先级治理
- 原因优先级治理

## 测试策略

继续在 `tests/repository_metadata_audit_unit.rs` 中扩展红测，覆盖：

- `report.hygiene_reason_views.weak_locator_by_reason`
  - 校验每个 locator reason 的计数、节点数、排序
- `report.hygiene_reason_views.weak_source_ref_by_reason`
  - 校验 critical / warning reason 的排序与计数
- 保持已有 `hygiene_diagnostics`
- 保持已有 `hygiene_summary`
- 保持已有 `hygiene_views`

## 预期结果

本轮完成后，foundation `RepositoryMetadataAudit` 将形成四层治理输出：

- 可追溯明细
- 可聚合摘要
- 可按节点和严重性排序的 grouped views
- 可按 weak reason 直接治理的 reason views

这会让知识库漫游底座更接近“可被 AI 直接消费”的治理底座，而不是只提供原始诊断材料。
