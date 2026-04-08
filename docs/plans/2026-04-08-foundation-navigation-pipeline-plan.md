# Foundation Navigation Pipeline Implementation Plan

> 对当前会话的执行约束：默认继续 foundation 主线；不回头重构 Task 1-7；不把业务线逻辑混入 pipeline。

**Goal:** 在 foundation 内补上正式 `NavigationPipeline`，把 route、roam、retrieve、assemble 四段收成最小导航闭环。  
**Architecture:** 保持纯内存、业务无关、阶段边界清晰；输出 `NavigationEvidence`；错误按 `Route / Roam / Retrieve` 抬升。  
**Tech Stack:** Rust、Cargo tests、纯内存 ontology + graph fixture。

---

## Task 1：先写 happy path 红灯测试

**Files:**

- Add: `tests/navigation_pipeline_integration.rs`

### Step 1：补测试

新增一个从问题到结构化证据的集成测试，验证：

- `route.matched_concept_ids == ["revenue"]`
- `roaming_path.len() == 1`
- `hits.len() == 2`
- `citations.len() == 2`

### Step 2：验证红灯

运行：

```powershell
cargo test --test navigation_pipeline_integration -- --nocapture
```

预期：

- 编译失败，因为 `NavigationPipeline` 尚不存在

## Task 2：实现 pipeline 最小闭环

**Files:**

- Add: `src/ops/foundation/navigation_pipeline.rs`
- Modify: `src/ops/foundation.rs`

### Step 1：最小实现

新增：

- `NavigationPipeline`
- `NavigationPipelineError`
- `new(ontology_store, graph_store)`
- `run(question)`

实现顺序：

1. route
2. roam
3. retrieve
4. assemble

### Step 2：验证绿灯

运行：

```powershell
cargo test --test navigation_pipeline_integration -- --nocapture
```

预期：

- happy path 测试通过

## Task 3：补阶段错误测试

**Files:**

- Modify: `tests/navigation_pipeline_integration.rs`

### Step 1：补 route 失败测试

验证：

- 未命中概念时返回 `NavigationPipelineError::Route`

### Step 2：补 retrieve 失败测试

验证：

- 命中概念但没有匹配证据时返回 `NavigationPipelineError::Retrieve`

### Step 3：验证通过

运行：

```powershell
cargo test --test navigation_pipeline_integration -- --nocapture
```

预期：

- pipeline 集成测试全部通过

## Task 4：跑 foundation 最小回归集

**Files:**

- Test: `tests/ontology_schema_unit.rs`
- Test: `tests/ontology_store_unit.rs`
- Test: `tests/knowledge_record_unit.rs`
- Test: `tests/knowledge_graph_store_unit.rs`
- Test: `tests/capability_router_unit.rs`
- Test: `tests/roaming_engine_unit.rs`
- Test: `tests/retrieval_engine_unit.rs`
- Test: `tests/evidence_assembler_unit.rs`
- Test: `tests/navigation_pipeline_integration.rs`

### Step 1：回归验证

运行：

```powershell
cargo test --test ontology_schema_unit --test ontology_store_unit --test knowledge_record_unit --test knowledge_graph_store_unit --test capability_router_unit --test roaming_engine_unit --test retrieval_engine_unit --test evidence_assembler_unit --test navigation_pipeline_integration -- --nocapture
```

预期：

- foundation 最小回归集全部通过

## Task 5：补交接与执行记录

**Files:**

- Modify: `docs/ai-handoff/AI_HANDOFF_MANUAL.md`
- Add or Modify: `docs/execution-notes-2026-04-08-foundation-navigation-pipeline.md`
- Modify: `.trae/CHANGELOG_TASK.md`

### Step 1：追加事实记录

记录：

- `EvidenceAssembler` 已从占位壳补成正式装配器
- `NavigationPipeline` 已成为 foundation 内部正式承接点
- 默认继续 foundation 主线，不默认跳去业务线

### Step 2：最终检查

运行：

```powershell
git status --short --branch
```

预期：

- 只包含本轮 foundation 闭环相关改动
