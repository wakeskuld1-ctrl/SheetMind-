# Foundation Standard Capabilities Phase 2 Stage 1

> **For Claude/Codex:** 本文档描述 foundation 线的通用标准能力阶段，不包含业务化接入，不默认允许把 foundation 直接耦合到证券分析主链。

<!-- 2026-04-08 CST: 新增 foundation phase 2 第一阶段计划文档。原因：本轮已经完成标准知识包、标准仓储和 metadata 精确过滤，需要把范围、非目标与验收口径沉淀成独立文档。目的：为后续继续推进通用标准能力提供统一交接入口。 -->

## 1. 目标

- 在 `src/ops/foundation/` 内补齐通用标准知识能力，而不是业务化知识能力。
- 让 foundation 在 Phase 1 导航闭环之上，新增“标准包结构 + 标准仓储 + metadata 精确过滤”三件套。
- 保持 foundation 仍为独立通用层，不接入证券分析主链，不扩展业务化知识入库。

## 2. 本阶段范围

- `KnowledgeBundle`
  - 标准知识包结构，统一承载 `schema_version / concepts / relations / nodes / edges`。
- `KnowledgeRepository`
  - 标准仓储入口，支持构建校验、JSON 落盘、JSON 读回、重建 ontology/graph store。
- `MetadataFilter`
  - 只支持 `node.metadata` 上的 exact-match 过滤，作为最小通用过滤契约。
- `KnowledgeNode.metadata`
  - 把 metadata 统一沉到标准节点模型中，避免后续业务域各自发明一套外置过滤结构。

## 3. 非目标

- 不做证券分析业务适配。
- 不做向量检索、重排序或召回融合。
- 不做高级 metadata DSL、范围过滤、模糊过滤。
- 不做知识入库流水线或外部数据源对接。
- 不做主链接入、catalog/dispatcher/skill 扩展。

## 4. 已完成实现

- `src/ops/foundation/knowledge_bundle.rs`
  - 定义标准知识包，并支持重建 `OntologyStore / KnowledgeGraphStore`。
- `src/ops/foundation/knowledge_repository.rs`
  - 提供最小仓储校验、序列化读写与 metadata exact-match 过滤。
- `src/ops/foundation/knowledge_record.rs`
  - `KnowledgeNode` 新增 `metadata` 与 `with_metadata_entry()`。
- `src/ops/foundation/ontology_schema.rs`
  - 已补 `serde` 派生，支持标准包持久化。
- `src/ops/foundation.rs`
  - 已导出新增 foundation 模块。

## 5. 测试与验收

- 新增测试：
  - `tests/knowledge_bundle_unit.rs`
  - `tests/knowledge_repository_unit.rs`
- 已通过回归：
  - `cargo test --test knowledge_bundle_unit --test knowledge_repository_unit -- --nocapture`
  - `cargo test --test ontology_schema_unit --test ontology_store_unit --test knowledge_record_unit --test knowledge_graph_store_unit --test capability_router_unit --test roaming_engine_unit --test retrieval_engine_unit --test evidence_assembler_unit --test navigation_pipeline_integration --test knowledge_bundle_unit --test knowledge_repository_unit -- --nocapture`

## 6. 当前边界

- foundation 现在是“通用标准能力 + 最小持久化闭环”。
- foundation 现在还不是“完整知识库产品”。
- foundation 现在也不是“证券分析主链的一部分”。

## 7. 下一阶段建议

1. `knowledge_ingestion`
   - 把外部标准 JSON/JSONL 包导入为 `KnowledgeBundle / KnowledgeRepository`。
2. 扩展 `MetadataFilter`
   - 在 exact-match 基础上补多字段 AND 与可选 concept scope。
3. 文件型持久化布局标准
   - 统一 bundle 文件命名、版本管理与目录约定。
