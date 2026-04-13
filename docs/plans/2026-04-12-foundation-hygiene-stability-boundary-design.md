# 2026-04-12 Foundation Hygiene Stability Boundary Design

## 背景

foundation `RepositoryMetadataAudit` 当前已经具备四层治理输出：

- `hygiene_diagnostics`
- `hygiene_summary`
- `hygiene_views`
- `hygiene_reason_views`

到这一步，底座的治理信息已经基本齐全，但仍然缺少两类稳定性保障：

1. 大样本场景下的排序与聚合稳定性
2. 边界语义场景下的计数口径稳定性

如果这两层不锁住，后续继续补规则、补 reason、补视图时，很容易发生：

- 节点排序漂移
- reason 排序漂移
- `diagnostic_count` 与 `affected_node_count` 语义被不小心改掉
- 多条相同 reason 在同一节点上的聚合口径发生变化

## 目标

在不改动底座对外结构的前提下，为 repository hygiene audit 补一轮“稳定性 + 边界语义”保障，确保以下能力在扩展后仍然稳定：

- 大仓库下 `hygiene_summary` 计数稳定
- 大仓库下 `hygiene_views.by_severity` 排序稳定
- 大仓库下 `hygiene_views.by_node` 排序稳定
- 大仓库下 `hygiene_reason_views` 排序稳定
- 同一节点多条同 reason 时，reason group 的计数语义稳定且可测试

## 非目标

- 不新增业务能力
- 不新增自动修复逻辑
- 不重构 audit 主流程
- 不改变现有输出字段命名和数据层级

## 方案

本轮采用 `A3-标准版`：

1. 在现有 `tests/repository_metadata_audit_unit.rs` 中增加两类新测试
2. 抽最小测试辅助构造函数，避免 fixture 复制失控
3. 如果红测暴露当前聚合实现存在语义空洞，只做最小修补
4. 不对 production 结构做额外抽象和重构

## 测试设计

### 一类：大仓库稳定性测试

构造一个更大的 repository fixture，覆盖：

- 多个 `MissingEvidenceRef`
- 多个 `DuplicateEvidenceRefWithinNode`
- 多个 `DuplicateEvidenceRef`
- 多种 `WeakLocator`
- 多种 `WeakSourceRef`
- 同 severity 下多个节点的排序竞争
- 同 reason 下多个节点的聚合与排序竞争

重点断言：

- `hygiene_summary.total_diagnostics`
- `severity_counts`
- `diagnostic_type_counts`
- `affected_node_count`
- `hygiene_views.by_severity[*].node_ids`
- `hygiene_views.by_node` 前几项排序
- `hygiene_reason_views.weak_locator_by_reason`
- `hygiene_reason_views.weak_source_ref_by_reason`

### 二类：边界语义测试

构造一个小而尖的 fixture，专门覆盖：

- 同一节点同一 `WeakLocatorReason` 出现多次
- 同一节点同一 `WeakSourceRefReason` 出现多次
- 同一节点既有相同 reason 重复，又有不同 reason 混合

重点断言：

- `hygiene_summary` reason count 仍按诊断条数累计
- `hygiene_reason_views[*].diagnostic_count`
  - 当前实现口径是否按节点去重
- `hygiene_reason_views[*].affected_node_count`
  - 当前实现口径是否与 `diagnostic_count` 一致
- 节点列表是否去重且排序稳定

## 为什么是标准版

### 不选轻量版

只补测试虽然快，但如果边界红测暴露出当前语义没有被明确实现，还是得补最小实现，最后还是要回到标准版。

### 不选重型版

重型版会顺手重构 fixture builder 和 aggregation helper，这不符合“不要一来就重构”的项目纪律。

标准版只做：

- 测试先行
- 红测驱动
- 最小修补

既稳，又不会把这轮工作做成结构翻修。

## 预期结果

本轮完成后，foundation roaming 底座将获得一层真正可依赖的稳定性保障：

- 能在更大样本下维持排序与聚合一致
- 能在边界输入下保持 reason 计数语义明确
- 能让后续 AI 或工程师在继续补治理能力时，不需要每次重新猜语义
