# Foundation Navigation Kernel Phase 1 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在当前 `TradingAgents` 仓库中补齐第一阶段可复用的 foundation 知识导航内核，形成最小可运行的 `ontology -> roaming -> retrieval -> evidence assembly` 闭环。

**Architecture:** 本阶段只建设独立 foundation 内核，不直接改动证券分析主链入口。实现方式保持最小可用和纯内存化，先通过测试把数据结构、路由、漫游、检索和装配契约钉死，再补最小集成入口，后续再由业务适配层接入证券分析结果。

**Tech Stack:** Rust 2024, 现有 `cargo test` 测试体系, 纯内存 store, `thiserror`, 标准库集合类型。

---

### Task 1: 建立 foundation 子模块入口

**Files:**
- Modify: `E:/TradingAgents/TradingAgents/src/ops/foundation.rs`
- Create: `E:/TradingAgents/TradingAgents/src/ops/foundation/ontology_schema.rs`
- Create: `E:/TradingAgents/TradingAgents/src/ops/foundation/ontology_store.rs`
- Create: `E:/TradingAgents/TradingAgents/src/ops/foundation/knowledge_record.rs`
- Create: `E:/TradingAgents/TradingAgents/src/ops/foundation/knowledge_graph_store.rs`
- Create: `E:/TradingAgents/TradingAgents/src/ops/foundation/capability_router.rs`
- Create: `E:/TradingAgents/TradingAgents/src/ops/foundation/roaming_engine.rs`
- Create: `E:/TradingAgents/TradingAgents/src/ops/foundation/retrieval_engine.rs`
- Create: `E:/TradingAgents/TradingAgents/src/ops/foundation/evidence_assembler.rs`
- Create: `E:/TradingAgents/TradingAgents/src/ops/foundation/navigation_pipeline.rs`
- Test: `E:/TradingAgents/TradingAgents/tests/ontology_schema_unit.rs`

**Step 1: 写失败测试，确认 foundation 新子模块可被导出**

```rust
use excel_skill::ops::foundation;

#[test]
fn foundation_navigation_modules_are_exported() {
    let _ = std::any::type_name::<foundation::ontology_schema::OntologySchema>();
}
```

**Step 2: 运行测试确认红灯**

Run: `cargo test --test ontology_schema_unit foundation_navigation_modules_are_exported -- --nocapture`
Expected: FAIL，因为 `foundation::ontology_schema` 还不存在。

**Step 3: 只写最小模块占位与导出**

```rust
#[path = "foundation/ontology_schema.rs"]
pub mod ontology_schema;
```

```rust
pub struct OntologySchema;
```

**Step 4: 运行测试确认绿灯**

Run: `cargo test --test ontology_schema_unit foundation_navigation_modules_are_exported -- --nocapture`
Expected: PASS。

### Task 2: 实现 ontology schema

**Files:**
- Modify: `E:/TradingAgents/TradingAgents/src/ops/foundation/ontology_schema.rs`
- Test: `E:/TradingAgents/TradingAgents/tests/ontology_schema_unit.rs`

**Step 1: 写失败测试，固定 concept 与 alias 索引行为**

```rust
use excel_skill::ops::foundation::ontology_schema::{OntologyConcept, OntologySchema};

#[test]
fn ontology_schema_indexes_concepts_and_aliases() {
    let schema = OntologySchema::new(
        vec![OntologyConcept::new("revenue", "Revenue").with_alias("sales")],
        vec![],
    )
    .expect("schema should be valid");

    assert_eq!(schema.find_concept_id("Revenue"), Some("revenue"));
    assert_eq!(schema.find_concept_id("sales"), Some("revenue"));
}
```

**Step 2: 运行测试确认红灯**

Run: `cargo test --test ontology_schema_unit ontology_schema_indexes_concepts_and_aliases -- --nocapture`
Expected: FAIL，因为构造器和查询接口尚未实现。

**Step 3: 写最小实现**

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OntologyConcept {
    pub id: String,
    pub name: String,
    pub aliases: Vec<String>,
}

impl OntologyConcept {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self { ... }
    pub fn with_alias(mut self, alias: impl Into<String>) -> Self { ... }
}

pub struct OntologySchema { ... }

impl OntologySchema {
    pub fn new(...) -> Result<Self, OntologySchemaError> { ... }
    pub fn find_concept_id(&self, raw: &str) -> Option<&str> { ... }
}
```

**Step 4: 运行测试确认绿灯**

Run: `cargo test --test ontology_schema_unit -- --nocapture`
Expected: PASS。

### Task 3: 实现 ontology store

**Files:**
- Modify: `E:/TradingAgents/TradingAgents/src/ops/foundation/ontology_store.rs`
- Test: `E:/TradingAgents/TradingAgents/tests/ontology_store_unit.rs`

**Step 1: 写失败测试，固定 concept 读取与关系邻接读取**

```rust
#[test]
fn ontology_store_reads_concepts_from_schema() { ... }

#[test]
fn ontology_store_returns_neighbors_by_relation_type() { ... }
```

**Step 2: 运行测试确认红灯**

Run: `cargo test --test ontology_store_unit -- --nocapture`
Expected: FAIL，因为 `OntologyStore` 尚未实现。

**Step 3: 写最小实现**

```rust
#[derive(Debug, Clone)]
pub struct OntologyStore {
    schema: OntologySchema,
}

impl OntologyStore {
    pub fn new(schema: OntologySchema) -> Self { ... }
    pub fn find_concept_id(&self, raw: &str) -> Option<&str> { ... }
    pub fn concept(&self, concept_id: &str) -> Option<&OntologyConcept> { ... }
    pub fn related_concepts<'a>(&'a self, concept_id: &str, allowed: &[OntologyRelationType]) -> Vec<&'a str> { ... }
}
```

**Step 4: 运行测试确认绿灯**

Run: `cargo test --test ontology_store_unit -- --nocapture`
Expected: PASS。

### Task 4: 实现知识记录与知识图谱 store

**Files:**
- Modify: `E:/TradingAgents/TradingAgents/src/ops/foundation/knowledge_record.rs`
- Modify: `E:/TradingAgents/TradingAgents/src/ops/foundation/knowledge_graph_store.rs`
- Test: `E:/TradingAgents/TradingAgents/tests/knowledge_record_unit.rs`
- Test: `E:/TradingAgents/TradingAgents/tests/knowledge_graph_store_unit.rs`

**Step 1: 写失败测试，固定 node / edge / evidence 与 concept->node 聚合行为**

```rust
#[test]
fn knowledge_record_keeps_concepts_evidence_and_edges() { ... }

#[test]
fn graph_store_collects_nodes_for_candidate_concepts() { ... }

#[test]
fn graph_store_reads_nodes_and_outgoing_edges() { ... }
```

**Step 2: 运行测试确认红灯**

Run: `cargo test --test knowledge_record_unit --test knowledge_graph_store_unit -- --nocapture`
Expected: FAIL，因为知识模型与 store 还未实现。

**Step 3: 写最小实现**

```rust
pub struct EvidenceRef { ... }
pub struct KnowledgeNode { ... }
pub struct KnowledgeEdge { ... }
pub struct KnowledgeGraphStore { ... }
```

**Step 4: 运行测试确认绿灯**

Run: `cargo test --test knowledge_record_unit --test knowledge_graph_store_unit -- --nocapture`
Expected: PASS。

### Task 5: 实现 capability router

**Files:**
- Modify: `E:/TradingAgents/TradingAgents/src/ops/foundation/capability_router.rs`
- Test: `E:/TradingAgents/TradingAgents/tests/capability_router_unit.rs`

**Step 1: 写失败测试，固定 alias 命中、短语优先和无命中错误**

```rust
#[test]
fn router_matches_question_to_seed_concepts_by_alias() { ... }

#[test]
fn router_prefers_phrase_alias_before_single_tokens() { ... }

#[test]
fn router_returns_error_when_question_has_no_known_concepts() { ... }
```

**Step 2: 运行测试确认红灯**

Run: `cargo test --test capability_router_unit -- --nocapture`
Expected: FAIL，因为 router 尚未实现。

**Step 3: 写最小实现**

```rust
pub struct NavigationRequest { pub question: String }
pub struct CapabilityRoute { pub matched_concept_ids: Vec<String> }
pub enum CapabilityRouterError { NoConceptMatched { question: String } }
pub struct CapabilityRouter { ... }
```

**Step 4: 运行测试确认绿灯**

Run: `cargo test --test capability_router_unit -- --nocapture`
Expected: PASS。

### Task 6: 实现 roaming engine

**Files:**
- Modify: `E:/TradingAgents/TradingAgents/src/ops/foundation/roaming_engine.rs`
- Test: `E:/TradingAgents/TradingAgents/tests/roaming_engine_unit.rs`

**Step 1: 写失败测试，固定允许关系、深度预算与概念预算**

```rust
#[test]
fn roaming_engine_stops_at_allowed_relations_and_depth() { ... }

#[test]
fn roaming_engine_respects_max_concepts_budget() { ... }
```

**Step 2: 运行测试确认红灯**

Run: `cargo test --test roaming_engine_unit -- --nocapture`
Expected: FAIL，因为 roaming engine 尚未实现。

**Step 3: 写最小实现**

```rust
pub struct RoamingPlan { ... }
pub struct RoamingStep { ... }
pub struct CandidateScope { ... }
pub struct RoamingEngine { ... }
```

**Step 4: 运行测试确认绿灯**

Run: `cargo test --test roaming_engine_unit -- --nocapture`
Expected: PASS。

### Task 7: 实现 scoped retrieval

**Files:**
- Modify: `E:/TradingAgents/TradingAgents/src/ops/foundation/retrieval_engine.rs`
- Test: `E:/TradingAgents/TradingAgents/tests/retrieval_engine_unit.rs`

**Step 1: 写失败测试，固定候选域内检索、排序和无命中错误**

```rust
#[test]
fn retrieval_engine_only_scores_nodes_inside_candidate_scope() { ... }

#[test]
fn retrieval_engine_returns_hits_in_descending_score_order() { ... }

#[test]
fn retrieval_engine_returns_error_when_scope_has_no_matching_evidence() { ... }
```

**Step 2: 运行测试确认红灯**

Run: `cargo test --test retrieval_engine_unit -- --nocapture`
Expected: FAIL，因为 retrieval 尚未实现。

**Step 3: 写最小实现**

```rust
pub struct RetrievalHit { ... }
pub enum RetrievalEngineError { NoEvidenceFound { question: String } }
pub struct RetrievalEngine;
```

**Step 4: 运行测试确认绿灯**

Run: `cargo test --test retrieval_engine_unit -- --nocapture`
Expected: PASS。

### Task 8: 实现 evidence assembler

**Files:**
- Modify: `E:/TradingAgents/TradingAgents/src/ops/foundation/evidence_assembler.rs`
- Test: `E:/TradingAgents/TradingAgents/tests/evidence_assembler_unit.rs`

**Step 1: 写失败测试，固定 route/path/hits 保留、citation 提炼和 summary 生成**

```rust
#[test]
fn evidence_assembler_preserves_route_path_and_hits() { ... }

#[test]
fn evidence_assembler_builds_citations_and_summary() { ... }
```

**Step 2: 运行测试确认红灯**

Run: `cargo test --test evidence_assembler_unit -- --nocapture`
Expected: FAIL，因为 evidence assembler 尚未实现。

**Step 3: 写最小实现**

```rust
pub struct NavigationEvidence { ... }
pub struct EvidenceAssembler;
```

**Step 4: 运行测试确认绿灯**

Run: `cargo test --test evidence_assembler_unit -- --nocapture`
Expected: PASS。

### Task 9: 实现最小 navigation pipeline 集成闭环

**Files:**
- Modify: `E:/TradingAgents/TradingAgents/src/ops/foundation/navigation_pipeline.rs`
- Modify: `E:/TradingAgents/TradingAgents/src/ops/foundation.rs`
- Test: `E:/TradingAgents/TradingAgents/tests/navigation_pipeline_integration.rs`

**Step 1: 写失败测试，固定统一入口 happy path 与 route 错误透传**

```rust
#[test]
fn navigation_pipeline_resolves_question_into_structured_evidence() { ... }

#[test]
fn navigation_pipeline_surfaces_router_error_for_unknown_question() { ... }
```

**Step 2: 运行测试确认红灯**

Run: `cargo test --test navigation_pipeline_integration -- --nocapture`
Expected: FAIL，因为 pipeline 尚未实现。

**Step 3: 写最小实现**

```rust
pub enum NavigationPipelineError { ... }
pub struct NavigationPipeline { ... }
impl NavigationPipeline {
    pub fn new(...) -> Self { ... }
    pub fn run(&self, request: &NavigationRequest) -> Result<NavigationEvidence, NavigationPipelineError> { ... }
}
```

**Step 4: 运行测试确认绿灯**

Run: `cargo test --test navigation_pipeline_integration -- --nocapture`
Expected: PASS。

### Task 10: 回归、文档与任务日志

**Files:**
- Modify: `E:/TradingAgents/TradingAgents/docs/AI_HANDOFF.md`
- Modify: `E:/TradingAgents/TradingAgents/README.md`
- Modify: `E:/TradingAgents/TradingAgents/progress.md`
- Modify: `E:/TradingAgents/TradingAgents/findings.md`
- Modify: `E:/TradingAgents/TradingAgents/CHANGELOG_TASK.MD`

**Step 1: 跑 foundation 最小全集回归**

Run: `cargo test --test ontology_schema_unit --test ontology_store_unit --test knowledge_record_unit --test knowledge_graph_store_unit --test capability_router_unit --test roaming_engine_unit --test retrieval_engine_unit --test evidence_assembler_unit --test navigation_pipeline_integration -- --nocapture`
Expected: PASS。

**Step 2: 更新 handoff 与 README**

- 说明当前只完成 foundation 第一阶段最小闭环。
- 明确尚未实现 metadata filter、向量检索、持久化知识库和证券分析适配层。

**Step 3: 追加任务日志**

- 按 `CHANGELOG_TASK.MD` 结构记录：
  - 修改内容
  - 修改原因
  - 方案还差什么
  - 潜在问题
  - 关闭项
  - 记忆点

**Step 4: 验证文档更新后状态**

Run: `git status --short`
Expected: 仅出现本轮 foundation 和文档变更。
