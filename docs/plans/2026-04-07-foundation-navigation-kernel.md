# Foundation Navigation Kernel Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 构建一条业务无关、纯内存、可测试的 foundation 最小导航闭环，打通 ontology-lite、knowledge roaming、retrieval 和 evidence assembly。

**Architecture:** 保留现有 [foundation.rs](D:/Rust/Excel_Skill/src/ops/foundation.rs) 作为模块入口，在 `src/ops/foundation/` 目录下新增独立子模块。实现顺序遵循 ontology first、roaming second、retrieval third，避免让检索提前变成系统入口。

**Tech Stack:** Rust、现有 `src/ops` 模块体系、标准库集合类型、Cargo tests

---

### Task 1: 挂接 foundation 新子模块入口

**Files:**
- Modify: [foundation.rs](D:/Rust/Excel_Skill/src/ops/foundation.rs)
- Create: [ontology_schema.rs](D:/Rust/Excel_Skill/src/ops/foundation/ontology_schema.rs)
- Create: [ontology_store.rs](D:/Rust/Excel_Skill/src/ops/foundation/ontology_store.rs)
- Create: [knowledge_record.rs](D:/Rust/Excel_Skill/src/ops/foundation/knowledge_record.rs)
- Create: [knowledge_graph_store.rs](D:/Rust/Excel_Skill/src/ops/foundation/knowledge_graph_store.rs)
- Create: [capability_router.rs](D:/Rust/Excel_Skill/src/ops/foundation/capability_router.rs)
- Create: [roaming_engine.rs](D:/Rust/Excel_Skill/src/ops/foundation/roaming_engine.rs)
- Create: [retrieval_engine.rs](D:/Rust/Excel_Skill/src/ops/foundation/retrieval_engine.rs)
- Create: [evidence_assembler.rs](D:/Rust/Excel_Skill/src/ops/foundation/evidence_assembler.rs)

**Step 1: 先写最小编译失败测试**

```rust
use excel_skill::ops::foundation;

#[test]
fn foundation_navigation_modules_are_exported() {
    let _ = std::any::type_name::<foundation::ontology_schema::OntologySchema>();
}
```

**Step 2: 运行测试确认失败**

Run: `cargo test --test ontology_schema_unit foundation_navigation_modules_are_exported -- --nocapture`
Expected: 编译失败，提示 `ontology_schema` 尚未导出

**Step 3: 在 foundation 入口加路径模块声明**

```rust
#[path = "foundation/ontology_schema.rs"]
pub mod ontology_schema;
```

其余新模块同理先全部挂接，文件内容先放最小占位结构体，保证模块边界建立成功。

**Step 4: 运行测试确认通过**

Run: `cargo test --test ontology_schema_unit foundation_navigation_modules_are_exported -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ops/foundation.rs src/ops/foundation
git commit -m "feat: add foundation navigation module entry points"
```

### Task 2: 实现 ontology schema 数据结构与校验

**Files:**
- Modify: [ontology_schema.rs](D:/Rust/Excel_Skill/src/ops/foundation/ontology_schema.rs)
- Test: [ontology_schema_unit.rs](D:/Rust/Excel_Skill/tests/ontology_schema_unit.rs)

**Step 1: 写失败测试**

```rust
#[test]
fn ontology_schema_indexes_concepts_and_aliases() {
    let schema = OntologySchema::new(
        vec![OntologyConcept::new("revenue", "Revenue").with_alias("sales")],
        vec![],
    ).expect("schema should be valid");

    assert_eq!(schema.find_concept_id("Revenue"), Some("revenue"));
    assert_eq!(schema.find_concept_id("sales"), Some("revenue"));
}
```

**Step 2: 跑失败测试**

Run: `cargo test --test ontology_schema_unit -- --nocapture`
Expected: FAIL，缺少 `OntologySchema::new` 或 `find_concept_id`

**Step 3: 写最小实现**

- 定义 `OntologyConcept`
- 定义 `OntologyRelationType`
- 定义 `OntologyRelation`
- 定义 `OntologySchema`
- 在 `new()` 中建立概念与别名索引
- 对重复概念 ID 和重复别名冲突做最小校验

**Step 4: 跑测试确认通过**

Run: `cargo test --test ontology_schema_unit -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add tests/ontology_schema_unit.rs src/ops/foundation/ontology_schema.rs
git commit -m "feat: add ontology schema primitives"
```

### Task 3: 实现 ontology store 查询能力

**Files:**
- Modify: [ontology_store.rs](D:/Rust/Excel_Skill/src/ops/foundation/ontology_store.rs)
- Test: [ontology_store_unit.rs](D:/Rust/Excel_Skill/tests/ontology_store_unit.rs)

**Step 1: 写失败测试**

```rust
#[test]
fn ontology_store_returns_neighbors_by_relation_type() {
    let store = sample_ontology_store();
    let neighbors = store.related_concepts("revenue", &[OntologyRelationType::DependsOn]);

    assert_eq!(neighbors, vec!["invoice"]);
}
```

**Step 2: 跑失败测试**

Run: `cargo test --test ontology_store_unit -- --nocapture`
Expected: FAIL，缺少 `related_concepts`

**Step 3: 写最小实现**

- `OntologyStore` 持有 `OntologySchema`
- 提供 `find_concept_id`
- 提供 `concept`
- 提供 `related_concepts`
- 关系查询先按内存遍历实现，不做复杂索引

**Step 4: 跑测试确认通过**

Run: `cargo test --test ontology_store_unit -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add tests/ontology_store_unit.rs src/ops/foundation/ontology_store.rs
git commit -m "feat: add ontology store queries"
```

### Task 4: 实现 knowledge graph 数据模型与存储

**Files:**
- Modify: [knowledge_record.rs](D:/Rust/Excel_Skill/src/ops/foundation/knowledge_record.rs)
- Modify: [knowledge_graph_store.rs](D:/Rust/Excel_Skill/src/ops/foundation/knowledge_graph_store.rs)
- Test: [knowledge_record_unit.rs](D:/Rust/Excel_Skill/tests/knowledge_record_unit.rs)
- Test: [knowledge_graph_store_unit.rs](D:/Rust/Excel_Skill/tests/knowledge_graph_store_unit.rs)

**Step 1: 写失败测试**

```rust
#[test]
fn graph_store_collects_nodes_for_candidate_concepts() {
    let store = sample_graph_store();
    let node_ids = store.node_ids_for_concepts(&["revenue", "invoice"]);

    assert_eq!(node_ids, vec!["node-revenue-1", "node-invoice-1"]);
}
```

**Step 2: 跑失败测试**

Run: `cargo test --test knowledge_record_unit --test knowledge_graph_store_unit -- --nocapture`
Expected: FAIL

**Step 3: 写最小实现**

- 定义 `KnowledgeNode`
- 定义 `KnowledgeEdge`
- 定义 `EvidenceRef`
- `KnowledgeGraphStore` 支持：
  - 按 node id 查询
  - 按 concept id 聚合节点
  - 读取节点边关系

**Step 4: 跑测试确认通过**

Run: `cargo test --test knowledge_record_unit --test knowledge_graph_store_unit -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add tests/knowledge_record_unit.rs tests/knowledge_graph_store_unit.rs src/ops/foundation/knowledge_record.rs src/ops/foundation/knowledge_graph_store.rs
git commit -m "feat: add knowledge graph primitives"
```

### Task 5: 实现 capability router

**Files:**
- Modify: [capability_router.rs](D:/Rust/Excel_Skill/src/ops/foundation/capability_router.rs)
- Test: [capability_router_unit.rs](D:/Rust/Excel_Skill/tests/capability_router_unit.rs)

**Step 1: 写失败测试**

```rust
#[test]
fn router_matches_question_to_seed_concepts_by_alias() {
    let route = sample_router().route(&NavigationRequest::new("show sales trend"));

    assert_eq!(route.matched_concept_ids, vec!["revenue"]);
}
```

**Step 2: 跑失败测试**

Run: `cargo test --test capability_router_unit -- --nocapture`
Expected: FAIL

**Step 3: 写最小实现**

- 定义 `NavigationRequest`
- 定义 `CapabilityRoute`
- 解析问题中的 token
- 基于概念名和别名匹配种子概念
- 如果没有命中概念，返回明确错误

**Step 4: 跑测试确认通过**

Run: `cargo test --test capability_router_unit -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add tests/capability_router_unit.rs src/ops/foundation/capability_router.rs
git commit -m "feat: add foundation capability router"
```

### Task 6: 实现 roaming engine

**Files:**
- Modify: [roaming_engine.rs](D:/Rust/Excel_Skill/src/ops/foundation/roaming_engine.rs)
- Test: [roaming_engine_unit.rs](D:/Rust/Excel_Skill/tests/roaming_engine_unit.rs)

**Step 1: 写失败测试**

```rust
#[test]
fn roaming_engine_stops_at_allowed_relations_and_depth() {
    let scope = sample_roaming_engine().roam(sample_plan()).expect("scope");

    assert_eq!(scope.concept_ids, vec!["revenue", "invoice"]);
    assert_eq!(scope.path.len(), 1);
}
```

**Step 2: 跑失败测试**

Run: `cargo test --test roaming_engine_unit -- --nocapture`
Expected: FAIL

**Step 3: 写最小实现**

- 定义 `RoamingPlan`
- 定义 `RoamingStep`
- 定义 `CandidateScope`
- 实现受限 BFS 或 DFS
- 遵守：
  - 允许关系类型
  - 最大深度
  - 最大节点数

**Step 4: 跑测试确认通过**

Run: `cargo test --test roaming_engine_unit -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add tests/roaming_engine_unit.rs src/ops/foundation/roaming_engine.rs
git commit -m "feat: add controlled knowledge roaming"
```

### Task 7: 实现 retrieval engine

**Files:**
- Modify: [retrieval_engine.rs](D:/Rust/Excel_Skill/src/ops/foundation/retrieval_engine.rs)
- Test: [retrieval_engine_unit.rs](D:/Rust/Excel_Skill/tests/retrieval_engine_unit.rs)

**Step 1: 写失败测试**

```rust
#[test]
fn retrieval_engine_only_scores_nodes_inside_candidate_scope() {
    let hits = sample_retrieval_engine().retrieve(
        "sales trend",
        &sample_candidate_scope(),
        &sample_graph_store(),
    ).expect("hits");

    assert_eq!(hits[0].node_id, "node-revenue-1");
}
```

**Step 2: 跑失败测试**

Run: `cargo test --test retrieval_engine_unit -- --nocapture`
Expected: FAIL

**Step 3: 写最小实现**

- 定义 `RetrievalHit`
- 输入问题、候选域和 graph store
- 只遍历候选域 node ids
- 基于关键词交集计算简单分数
- 无命中时返回 `NoEvidenceFound`

**Step 4: 跑测试确认通过**

Run: `cargo test --test retrieval_engine_unit -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add tests/retrieval_engine_unit.rs src/ops/foundation/retrieval_engine.rs
git commit -m "feat: add scoped retrieval engine"
```

### Task 8: 实现 evidence assembler

**Files:**
- Modify: [evidence_assembler.rs](D:/Rust/Excel_Skill/src/ops/foundation/evidence_assembler.rs)
- Test: [evidence_assembler_unit.rs](D:/Rust/Excel_Skill/tests/evidence_assembler_unit.rs)

**Step 1: 写失败测试**

```rust
#[test]
fn evidence_assembler_preserves_route_path_and_hits() {
    let evidence = assemble_sample_evidence();

    assert_eq!(evidence.hits.len(), 1);
    assert_eq!(evidence.roaming_path.len(), 1);
}
```

**Step 2: 跑失败测试**

Run: `cargo test --test evidence_assembler_unit -- --nocapture`
Expected: FAIL

**Step 3: 写最小实现**

- 定义 `NavigationEvidence`
- 把 route、path、hits 装配成统一结构
- 生成简短 `summary`
- 把 hit 转成 citation/evidence refs

**Step 4: 跑测试确认通过**

Run: `cargo test --test evidence_assembler_unit -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add tests/evidence_assembler_unit.rs src/ops/foundation/evidence_assembler.rs
git commit -m "feat: add navigation evidence assembler"
```

### Task 9: 打通最小集成闭环

**Files:**
- Test: [navigation_pipeline_integration.rs](D:/Rust/Excel_Skill/tests/navigation_pipeline_integration.rs)
- Modify: [capability_router.rs](D:/Rust/Excel_Skill/src/ops/foundation/capability_router.rs)
- Modify: [roaming_engine.rs](D:/Rust/Excel_Skill/src/ops/foundation/roaming_engine.rs)
- Modify: [retrieval_engine.rs](D:/Rust/Excel_Skill/src/ops/foundation/retrieval_engine.rs)
- Modify: [evidence_assembler.rs](D:/Rust/Excel_Skill/src/ops/foundation/evidence_assembler.rs)

**Step 1: 写失败测试**

```rust
#[test]
fn navigation_pipeline_resolves_question_into_structured_evidence() {
    let result = sample_navigation_pipeline().run("show sales trend").expect("result");

    assert_eq!(result.route.matched_concept_ids, vec!["revenue"]);
    assert!(!result.hits.is_empty());
}
```

**Step 2: 跑失败测试**

Run: `cargo test --test navigation_pipeline_integration -- --nocapture`
Expected: FAIL

**Step 3: 写最小集成实现**

- 增加一个轻量 pipeline 入口，或在测试中串联各模块
- 保持 pipeline 仍在 foundation 侧
- 不接 Tool 和 dispatcher

**Step 4: 跑测试确认通过**

Run: `cargo test --test navigation_pipeline_integration -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add tests/navigation_pipeline_integration.rs src/ops/foundation
git commit -m "feat: add foundation navigation pipeline"
```

### Task 10: 回归验证与文档同步

**Files:**
- Modify: [.trae/CHANGELOG_TASK.md](D:/Rust/Excel_Skill/.trae/CHANGELOG_TASK.md)
- Optional Modify: [AI_HANDOFF_MANUAL.md](D:/Rust/Excel_Skill/docs/ai-handoff/AI_HANDOFF_MANUAL.md)
- Optional Modify: [project-baseline.md](D:/Rust/Excel_Skill/docs/ai-memory/project-baseline.md)

**Step 1: 运行完整最小测试集**

Run: `cargo test --test ontology_schema_unit --test ontology_store_unit --test knowledge_record_unit --test knowledge_graph_store_unit --test capability_router_unit --test roaming_engine_unit --test retrieval_engine_unit --test evidence_assembler_unit --test navigation_pipeline_integration -- --nocapture`
Expected: 全部 PASS

**Step 2: 复核约束没有跑偏**

检查项：
- 没有引入 GUI
- 没有引入 Tool dispatcher
- 没有引入 domain-specific 语义
- retrieval 只在候选域内执行

**Step 3: 追加任务日志**

```markdown
## 2026-04-07
### 修改内容
- ...
```

**Step 4: 再跑一次核心集成测试**

Run: `cargo test --test navigation_pipeline_integration -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add .trae/CHANGELOG_TASK.md docs/ai-handoff/AI_HANDOFF_MANUAL.md docs/ai-memory/project-baseline.md
git commit -m "docs: record foundation navigation kernel delivery"
```
