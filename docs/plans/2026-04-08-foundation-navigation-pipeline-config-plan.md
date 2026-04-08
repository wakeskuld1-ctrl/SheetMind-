# Foundation Navigation Pipeline Config Implementation Plan

> 对当前会话的执行约束：只承接 foundation 主线；只完成 Task 10 第一节；不提前拆 retrieval config，不接 CLI / Tool。

**Goal:** 为 foundation navigation pipeline 增加最小显式配置对象，让漫游关系、深度和候选预算从硬编码值变成可调合同。  
**Architecture:** 保持 `NavigationPipeline::new()` 默认行为不变，在 `navigation_pipeline.rs` 中新增 `NavigationPipelineConfig` 与 `new_with_config()`。配置只覆盖漫游侧，不拆 retrieval 配置，不引入 CLI/Tool 透传。  
**Tech Stack:** Rust、纯内存 foundation pipeline、Cargo integration tests、builder 风格 API。

---

## Task 1：为自定义配置补红灯测试

**Files:**

- Modify: `tests/navigation_pipeline_integration.rs`

### Step 1：先写失败测试

补一条自定义配置测试：

```rust
#[test]
fn navigation_pipeline_uses_custom_config_to_limit_roaming_scope() {
    let result = sample_navigation_pipeline_with_config()
        .run("show sales month")
        .unwrap();

    assert_eq!(result.roaming_path.len(), 0);
    assert_eq!(result.hits.len(), 1);
}
```

### Step 2：验证红灯

运行：

```powershell
cargo test --test navigation_pipeline_integration -- --nocapture
```

预期：

- 编译失败或测试失败，因为 `NavigationPipelineConfig` / `new_with_config()` 尚不存在

## Task 2：实现最小配置对象

**Files:**

- Modify: `src/ops/foundation/navigation_pipeline.rs`

### Step 1：最小实现

新增：

- `NavigationPipelineConfig`
- `Default`
- builder 风格方法：
  - `with_allowed_relation_types`
  - `with_max_depth`
  - `with_max_concepts`
- `NavigationPipeline` 持有 `config`
- `new_with_config()`

### Step 2：验证绿灯

运行：

```powershell
cargo test --test navigation_pipeline_integration -- --nocapture
```

预期：

- 集成测试全部通过

## Task 3：跑 foundation 最小回归集

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

- foundation 相关测试全部通过

## Task 4：更新交接与任务日志

**Files:**

- Modify: `docs/ai-handoff/AI_HANDOFF_MANUAL.md`
- Modify: `.trae/CHANGELOG_TASK.md`
- Optional Modify: `docs/execution-notes-2026-04-08-foundation-navigation-pipeline.md`

### Step 1：追加事实记录

记录：

- pipeline 已从硬编码策略变成最小可调配置
- 默认行为保持不变
- 自定义配置验证命令与结果

### Step 2：最终检查

运行：

```powershell
git status --short --branch
```

预期：

- 只包含本轮 foundation 配置化相关改动
