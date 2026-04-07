# Foundation Navigation Kernel Design

## 1. 背景

当前项目在产品层、股票层和工具层已经积累了较多能力，但用户已经明确要求，当前阶段不要继续扩 GUI 主线，也不要把 foundation 简化成 metadata 加检索。当前最重要的工作，是在 `foundation` 侧建立一条业务无关、可验证、可复用的最小导航内核。

本轮设计只处理 foundation 内核，不接入 GUI，不引入外部模型 provider，不做正式 Tool 契约，不绑定任何单一业务场景。目标是先把 `ontology-lite -> knowledge roaming -> retrieval -> evidence assembly` 这条主链路稳定下来。

## 2. 设计目标

- 建立一套最小可跑的 foundation 导航闭环。
- 保持能力业务无关，不把合同、发票、财报、公告、证券等语义固化为底层前提。
- 让检索成为“候选域内的执行器”，而不是全局系统入口。
- 通过纯内存数据结构和测试用例先把链路跑通，再考虑 CLI、SQLite、provider 等扩展。

## 3. 本轮范围

### 3.1 包含

- ontology-lite 概念结构与关系约束。
- 知识节点、知识边、证据引用的数据模型。
- 问题到能力和种子概念的最小路由。
- 控制式知识漫游。
- 候选域内的关键词检索。
- 导航结果与证据装配。
- 单元测试与一条最小集成测试链路。

### 3.2 不包含

- GUI 页面、交互流、聊天展示。
- 正式 CLI / Tool 注册与分发。
- SQLite 持久化。
- 向量检索和 rerank。
- 云端或本地模型 provider。
- 面向证券、财报、公告等单一领域的专用语义。

## 4. 模块落点

由于当前仓库已经存在 [foundation.rs](D:/Rust/Excel_Skill/src/ops/foundation.rs)，本轮不把它改造成目录模块，而是保留它作为 foundation 入口，并通过显式路径把新内核模块挂到 `src/ops/foundation/` 目录下。

建议新增：

- [foundation/mod.rs](D:/Rust/Excel_Skill/src/ops/foundation/mod.rs) 不新增，避免与现有 [foundation.rs](D:/Rust/Excel_Skill/src/ops/foundation.rs) 冲突。
- [ontology_schema.rs](D:/Rust/Excel_Skill/src/ops/foundation/ontology_schema.rs)
- [ontology_store.rs](D:/Rust/Excel_Skill/src/ops/foundation/ontology_store.rs)
- [knowledge_record.rs](D:/Rust/Excel_Skill/src/ops/foundation/knowledge_record.rs)
- [knowledge_graph_store.rs](D:/Rust/Excel_Skill/src/ops/foundation/knowledge_graph_store.rs)
- [capability_router.rs](D:/Rust/Excel_Skill/src/ops/foundation/capability_router.rs)
- [roaming_engine.rs](D:/Rust/Excel_Skill/src/ops/foundation/roaming_engine.rs)
- [retrieval_engine.rs](D:/Rust/Excel_Skill/src/ops/foundation/retrieval_engine.rs)
- [evidence_assembler.rs](D:/Rust/Excel_Skill/src/ops/foundation/evidence_assembler.rs)

同时修改 [foundation.rs](D:/Rust/Excel_Skill/src/ops/foundation.rs)，新增这些子模块的导出声明。

## 5. 核心数据模型

### 5.1 Ontology

- `OntologyConcept`
  - `id`
  - `name`
  - `aliases`
  - `description`
  - `capability_ids`
- `OntologyRelationType`
  - `ParentOf`
  - `ChildOf`
  - `DependsOn`
  - `Supports`
  - `References`
  - `AdjacentTo`
- `OntologyRelation`
  - `from_concept_id`
  - `to_concept_id`
  - `relation_type`
- `OntologySchema`
  - `concepts`
  - `relations`

### 5.2 Knowledge Graph

- `KnowledgeNode`
  - `id`
  - `title`
  - `content`
  - `concept_ids`
  - `keywords`
  - `metadata`
- `KnowledgeEdge`
  - `from_node_id`
  - `to_node_id`
  - `relation_type`
  - `weight`
- `EvidenceRef`
  - `node_id`
  - `snippet`
  - `score`
  - `reason`

### 5.3 Navigation

- `NavigationRequest`
  - `question`
  - `preferred_capability`
  - `metadata_filters`
- `CapabilityRoute`
  - `capability_id`
  - `matched_concept_ids`
  - `route_reason`
- `RoamingPlan`
  - `seed_concept_ids`
  - `allowed_relation_types`
  - `max_depth`
  - `max_nodes`
- `RoamingStep`
  - `depth`
  - `from_concept_id`
  - `to_concept_id`
  - `relation_type`
- `CandidateScope`
  - `concept_ids`
  - `node_ids`
  - `path`
- `RetrievalHit`
  - `node_id`
  - `score`
  - `matched_terms`
  - `reason`
- `NavigationEvidence`
  - `route`
  - `roaming_path`
  - `hits`
  - `citations`
  - `summary`

## 6. 执行链路

最小闭环遵循下面的执行顺序：

1. `capability_router` 接收问题文本。
2. 基于概念名、别名和显式 capability 偏好匹配种子概念。
3. 生成 `CapabilityRoute`。
4. `roaming_engine` 根据种子概念、允许的关系类型和深度约束扩展候选概念域。
5. `knowledge_graph_store` 根据候选概念域收敛候选节点集合。
6. `retrieval_engine` 只在候选节点集合内执行关键词匹配和简单打分。
7. `evidence_assembler` 装配最终结构化输出。

这里的关键不是“搜到什么”，而是“先正确收敛，再检索命中”。这也是本轮不做全局检索的原因。

## 7. 存储策略

本轮只做纯内存实现。

原因：

- 可以先验证语义链路本身。
- 不会被 SQLite、文件加载、迁移和缓存机制分散注意力。
- 更适合 TDD。
- 后续若接入 SQLite，可以在 store 层替换而不改上层链路。

因此：

- `OntologyStore` 先封装 `OntologySchema` 的查询能力。
- `KnowledgeGraphStore` 先封装节点和边的纯内存查询能力。
- 测试中直接构造样例数据，不引入 fixture 文件格式。

## 8. 错误模型

建议定义统一的 foundation 内核错误枚举，例如：

- `UnknownConcept`
- `AmbiguousConcept`
- `MissingCapability`
- `InvalidOntologySchema`
- `InvalidKnowledgeGraph`
- `RoamingBudgetExceeded`
- `EmptyCandidateScope`
- `NoEvidenceFound`

错误设计原则：

- 先用结构化 `enum`，不要先引入复杂错误框架。
- 保证测试可以稳定断言错误类型。
- 把“无候选域”和“无证据”分开，避免后续调试时信息不够。

## 9. 测试策略

### 9.1 单元测试

建议新增：

- [ontology_schema_unit.rs](D:/Rust/Excel_Skill/tests/ontology_schema_unit.rs)
- [ontology_store_unit.rs](D:/Rust/Excel_Skill/tests/ontology_store_unit.rs)
- [knowledge_record_unit.rs](D:/Rust/Excel_Skill/tests/knowledge_record_unit.rs)
- [knowledge_graph_store_unit.rs](D:/Rust/Excel_Skill/tests/knowledge_graph_store_unit.rs)
- [capability_router_unit.rs](D:/Rust/Excel_Skill/tests/capability_router_unit.rs)
- [roaming_engine_unit.rs](D:/Rust/Excel_Skill/tests/roaming_engine_unit.rs)
- [retrieval_engine_unit.rs](D:/Rust/Excel_Skill/tests/retrieval_engine_unit.rs)
- [evidence_assembler_unit.rs](D:/Rust/Excel_Skill/tests/evidence_assembler_unit.rs)

### 9.2 集成测试

建议新增：

- [navigation_pipeline_integration.rs](D:/Rust/Excel_Skill/tests/navigation_pipeline_integration.rs)

关键断言：

- 问题能命中正确概念或别名。
- 漫游不会越过允许关系和深度限制。
- 检索只在候选域节点中执行。
- 最终输出包含路由、路径、命中和证据摘要。
- 空候选域和无证据场景会返回明确错误。

## 10. 风险与约束

- 当前 [foundation.rs](D:/Rust/Excel_Skill/src/ops/foundation.rs) 已经存在，不能直接改成目录模块，否则会和现有模块系统冲突。
- 当前仓库存在大量并行改动，本轮只新增 foundation 内核相关文件，不顺手重构其他模块。
- 如果把业务示例直接建成证券语义，会违背当前项目级约束。
- 如果先接 Tool / CLI 契约，范围会明显膨胀，不符合“最小可跑闭环”目标。

## 11. 本轮完成标准

达到以下条件即可视为本轮 foundation 内核设计落地完成：

- foundation 新内核模块文件结构明确。
- 核心数据结构稳定。
- 执行链路被单独定义。
- 错误模型和测试口径明确。
- 后续实现可以按模块逐步推进，而不需要再重新讨论整体架构。

## 12. 下一步实现顺序

推荐实现顺序：

1. `ontology_schema`
2. `ontology_store`
3. `knowledge_record`
4. `knowledge_graph_store`
5. `capability_router`
6. `roaming_engine`
7. `retrieval_engine`
8. `evidence_assembler`
9. `navigation_pipeline_integration`

这个顺序能保证先把 semantic backbone 立住，再逐步接出候选域收敛和证据装配，不会让 retrieval 提前夺走架构中心。
