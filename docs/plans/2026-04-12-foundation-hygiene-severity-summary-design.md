# 2026-04-12 Foundation Hygiene Severity Summary Design

## 背景

foundation `RepositoryMetadataAudit` 目前已经具备：

- 节点级 metadata issue 聚合
- repository-level `hygiene_diagnostics`
- `locator` 与 `source_ref` 的基础结构诊断

但当前输出仍以“逐条明细”为主，缺少适合后续 AI 或治理流程直接消费的结构化摘要层。

## 目标

在不改变现有 `hygiene_diagnostics` 明细语义的前提下，为 `RepositoryMetadataAuditReport` 增加一层结构化 `hygiene_summary`，用于表达：

- 诊断总量
- 严重级别分布
- 诊断类型分布
- 弱 `locator` / 弱 `source_ref` 原因分布
- 受影响节点规模
- 是否存在阻塞型 hygiene 问题

## 非目标

- 不修改现有 `hygiene_diagnostics` 明细结构
- 不引入自动修复、rewrite、migration executor
- 不扩展证券业务层
- 不重构 foundation roaming 模块边界

## 方案

在 `RepositoryMetadataAuditReport` 中新增：

- `hygiene_summary: RepositoryEvidenceHygieneSummary`

新增摘要结构：

- `RepositoryHygieneSeverity`
  - `Critical`
  - `Warning`
  - `Info`

- `RepositoryEvidenceHygieneSummary`
  - `total_diagnostics: usize`
  - `severity_counts: BTreeMap<String, usize>`
  - `diagnostic_type_counts: BTreeMap<String, usize>`
  - `weak_locator_reason_counts: BTreeMap<String, usize>`
  - `weak_source_ref_reason_counts: BTreeMap<String, usize>`
  - `affected_node_count: usize`
  - `has_blocking_hygiene_issue: bool`

## 严重级别映射

本轮先采用稳定、保守的启发式映射：

### Critical

- `MissingEvidenceRef`
- `WeakSourceRef::Blank`
- `WeakSourceRef::TooShort`
- `WeakSourceRef::MissingNamespace`
- `WeakSourceRef::EntityMissing`

### Warning

- `DuplicateEvidenceRefWithinNode`
- `DuplicateEvidenceRef`
- 全部 `WeakLocator::*`
- `WeakSourceRef::ContainsWhitespace`
- `WeakSourceRef::InvalidCharacter`
- `WeakSourceRef::UnknownNamespace`

### Info

- 本轮先不主动产出，仅保留枚举口子，避免未来再改报告模型

## 阻塞口径

- `has_blocking_hygiene_issue = true`
  - 当且仅当至少存在 1 条 `Critical`

这个口径适合后续 AI 在知识库漫游主链中快速判断：

- 是否应先治理证据卫生
- 是否适合继续进入更下游的导航、检索或解释阶段

## 设计考虑

- 先保留明细与摘要双层结构，而不是把所有信息折叠进单一报告，避免后续丢失可追溯性
- `severity_counts` 与 `diagnostic_type_counts` 使用 `BTreeMap<String, usize>`，保持现有报告风格一致，方便序列化与测试稳定
- `affected_node_count` 采用去重后的节点数，而不是明细条数，便于后续做治理规模判断
- 本轮不输出“治理建议文本”，只输出结构化摘要，避免过早把策略层写死

## 测试策略

在 `tests/repository_metadata_audit_unit.rs` 新增断言覆盖：

- `hygiene_summary.total_diagnostics`
- `hygiene_summary.severity_counts`
- `hygiene_summary.diagnostic_type_counts`
- `hygiene_summary.weak_locator_reason_counts`
- `hygiene_summary.weak_source_ref_reason_counts`
- `hygiene_summary.affected_node_count`
- `hygiene_summary.has_blocking_hygiene_issue`

并保持：

- 现有 `hygiene_diagnostics` 明细断言不变
- 现有 issue aggregation 断言不变

## 预期结果

本轮完成后，foundation `RepositoryMetadataAudit` 将从“只有逐条诊断”推进到“明细 + 摘要双层治理输出”，更适合作为知识库漫游底座的治理入口能力。
