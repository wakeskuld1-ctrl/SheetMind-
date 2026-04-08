# Foundation Metadata Filter Phase 2 Stage 3

> **For Claude/Codex:** 本文档只描述 foundation 的通用过滤扩展能力，不包含业务 DSL、不包含模糊查询、不包含范围过滤。

<!-- 2026-04-08 CST: 新增 foundation metadata filter 阶段文档。原因：当前过滤能力已从单字段 exact-match 扩展到多字段 AND 与 concept scope，需要单独记录契约和边界。目的：为后续继续扩展过滤与持久化标准提供交接入口。 -->

## 1. 目标

- 在 foundation 中扩展 `MetadataFilter`。
- 保持过滤能力仍然是通用标准能力，不变成查询 DSL。
- 让上层可以先按 `concept scope` 收窄，再做 metadata 多字段 AND 匹配。

## 2. 本阶段范围

- `MetadataFilter::with_exact_match()`
  - 继续支持 exact-match。
- `MetadataFilter::with_concept_id()`
  - 新增 concept scope 收窄能力。
- `KnowledgeRepository::filtered_node_ids()`
  - 新增“多字段全部命中 + concept scope 命中”组合过滤。

## 3. 非目标

- 不做 OR / NOT。
- 不做范围过滤。
- 不做模糊匹配、前缀匹配、正则匹配。
- 不做独立 DSL 或表达式树。

## 4. 已完成测试

- `tests/knowledge_repository_unit.rs`
  - `knowledge_repository_filters_nodes_by_exact_metadata_match`
  - `knowledge_repository_requires_all_exact_matches`
  - `knowledge_repository_limits_matches_to_concept_scope`
- 已通过回归：
  - `cargo test --test knowledge_repository_unit -- --nocapture`

## 5. 当前边界

- 现在已经支持多字段 AND。
- 现在已经支持可选 concept scope。
- 现在还不支持更复杂布尔表达式。

## 6. 下一阶段建议

1. 文件型持久化布局标准
   - 定义 bundle/repository 文件命名、目录结构与版本规则。
2. 原子写与更强校验
   - 给 repository 写入补原子落盘，并补更细边界测试。
3. 更强过滤能力
   - 如果用户再次批准，再考虑 OR/NOT 或更复杂 DSL。
