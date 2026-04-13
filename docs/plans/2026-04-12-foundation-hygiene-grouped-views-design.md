# 2026-04-12 Foundation Hygiene Grouped Views Design

## 背景

foundation `RepositoryMetadataAudit` 当前已经具备两层输出：

- `hygiene_diagnostics`
  - 保留逐条证据卫生问题
- `hygiene_summary`
  - 提供 severity、类型、原因、受影响节点数量等聚合摘要

这已经足够做“是否需要拦截”的判断，但还不够直接支持后续 AI 做“先治理哪一批节点、先看哪一类问题”的执行级路由。

## 目标

在保留现有明细层与摘要层不变的前提下，新增 grouped views，让 repository audit 输出三层治理信息：

1. 明细层
2. 摘要层
3. 分组视图层

分组视图需要同时支持：

- 按 severity 看问题
- 按 node 看问题

## 非目标

- 不修改已有 severity 映射规则
- 不引入自动修复、rewrite、治理动作建议
- 不扩展证券业务层
- 不改动 foundation roaming 模块边界

## 方案

在 `RepositoryMetadataAuditReport` 中新增：

- `hygiene_views: RepositoryEvidenceHygieneViews`

新增结构：

- `RepositoryEvidenceHygieneViews`
  - `by_severity: Vec<RepositoryEvidenceHygieneSeverityGroup>`
  - `by_node: Vec<RepositoryEvidenceHygieneNodeGroup>`

- `RepositoryEvidenceHygieneSeverityGroup`
  - `severity: RepositoryHygieneSeverity`
  - `diagnostic_count: usize`
  - `affected_node_count: usize`
  - `node_ids: Vec<String>`

- `RepositoryEvidenceHygieneNodeGroup`
  - `node_id: String`
  - `highest_severity: RepositoryHygieneSeverity`
  - `diagnostic_count: usize`
  - `diagnostic_type_counts: BTreeMap<String, usize>`

## 设计考虑

### 为什么按 severity 只放聚合信息

本轮 `by_severity` 不直接再复制完整 diagnostics 明细，而是保留：

- 严重级别
- 诊断数量
- 涉及节点数量
- 节点列表

原因是：

- 明细层已经存在，不需要再做第三份重复数据
- 这里的目标是快速路由，而不是重复展开所有字段
- 更轻量，结构更稳定

### 为什么按 node 要放最高 severity 和类型计数

按节点治理时，最重要的是：

- 这个节点要不要优先处理
- 它有几类问题
- 问题规模大概如何

因此 `by_node` 先保留：

- `highest_severity`
- `diagnostic_count`
- `diagnostic_type_counts`

而不直接复制全部 diagnostics 列表。后续如果真要“单节点治理工作台”，再扩。

### 排序规则

为了让输出稳定且方便测试，本轮固定：

- `by_severity`
  - 按 severity 顺序输出：`Critical -> Warning -> Info`

- `by_node`
  - 先按 `highest_severity` 排序：`Critical -> Warning -> Info`
  - 再按 `diagnostic_count` 降序
  - 最后按 `node_id` 升序

## 与现有层次的关系

- `hygiene_diagnostics`
  - 看原始明细
- `hygiene_summary`
  - 看全局摘要
- `hygiene_views.by_severity`
  - 看先拦哪一类
- `hygiene_views.by_node`
  - 看先治哪几个节点

## 测试策略

在 `tests/repository_metadata_audit_unit.rs` 新增断言覆盖：

- `report.hygiene_views.by_severity`
  - `Critical`
  - `Warning`
  - `Info`

- `report.hygiene_views.by_node`
  - 重点断言前几项节点排序与字段
  - 至少覆盖：
    - `node-revenue-governance`
    - `node-revenue-owner`
    - `node-revenue-missing`

并保持：

- 已有 `hygiene_diagnostics` 断言不变
- 已有 `hygiene_summary` 断言不变

## 预期结果

本轮完成后，foundation `RepositoryMetadataAudit` 将形成：

- 可追溯明细
- 可判断全局风险的摘要
- 可直接支持 AI 路由和节点治理排序的 grouped views

这会让知识库漫游底座的治理输出更接近“可直接消费”的状态。
