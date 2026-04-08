# Foundation Retrieval Source Priority Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 foundation retrieval engine 增加 `source_ref` 次级排序能力，让更像主数据源的证据来源在同分场景下优先。  
**Architecture:** 保持文本分数仍是第一优先级，只在排序 tie-break 阶段加入 `source_ref` 固定优先级；不修改 `RetrievalHit` 合同，不引入 `RetrievalConfig`。  
**Tech Stack:** Rust、Cargo unit tests、纯内存 fixture、固定来源分层规则。

---

### Task 1: 为 primary source 优先补红灯测试

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\tests\retrieval_engine_unit.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn retrieval_engine_prefers_primary_source_refs_when_scores_tie() {
    let hits = sample_retrieval_engine()
        .retrieve(
            "sales review",
            &sample_candidate_scope_for_revenue_only(),
            &sample_graph_store_for_source_priority(),
        )
        .unwrap();

    assert_eq!(hits[0].node_id, "node-primary-source");
}
```

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test retrieval_engine_unit retrieval_engine_prefers_primary_source_refs_when_scores_tie -- --nocapture
```

Expected:

- 测试失败，因为当前排序还不看 `source_ref`

### Task 2: 为 derived vs planning source 优先级补红灯测试

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\tests\retrieval_engine_unit.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn retrieval_engine_prefers_derived_sources_over_planning_sources_when_scores_tie() {
    let hits = sample_retrieval_engine()
        .retrieve(
            "sales review",
            &sample_candidate_scope_for_revenue_only(),
            &sample_graph_store_for_derived_vs_planning_source_priority(),
        )
        .unwrap();

    assert_eq!(hits[0].node_id, "node-derived-source");
}
```

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test retrieval_engine_unit retrieval_engine_prefers_derived_sources_over_planning_sources_when_scores_tie -- --nocapture
```

Expected:

- 测试失败，因为当前实现还没有来源分层 tie-break

### Task 3: 为“来源优先不反压文本分数”补红灯测试

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\tests\retrieval_engine_unit.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn retrieval_engine_keeps_higher_text_score_ahead_of_better_source_priority() {
    let hits = sample_retrieval_engine()
        .retrieve(
            "sales review trend",
            &sample_candidate_scope_for_revenue_only(),
            &sample_graph_store_for_source_priority_not_overriding_text_score(),
        )
        .unwrap();

    assert_eq!(hits[0].node_id, "node-higher-text-score");
}
```

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test retrieval_engine_unit retrieval_engine_keeps_higher_text_score_ahead_of_better_source_priority -- --nocapture
```

Expected:

- 如果实现错误地把 `source_ref` 拉成主信号，测试会失败

### Task 4: 实现 source_ref 次级排序

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\src\ops\foundation\retrieval_engine.rs`

**Step 1: Write minimal implementation**

新增最小辅助逻辑：

- `source_ref_priority()`
- `evidence_source_priority()`

排序顺序：

1. 文本主分数
2. `source_ref` 优先级
3. `node_id`

**Step 2: Run retrieval tests to verify they pass**

Run:

```powershell
cargo test --test retrieval_engine_unit -- --nocapture
```

Expected:

- retrieval 相关测试全部通过

### Task 5: 跑 foundation 最小回归集

**Files:**
- Test: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\tests\ontology_schema_unit.rs`
- Test: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\tests\ontology_store_unit.rs`
- Test: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\tests\knowledge_record_unit.rs`
- Test: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\tests\knowledge_graph_store_unit.rs`
- Test: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\tests\capability_router_unit.rs`
- Test: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\tests\roaming_engine_unit.rs`
- Test: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\tests\retrieval_engine_unit.rs`
- Test: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\tests\evidence_assembler_unit.rs`
- Test: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\tests\navigation_pipeline_integration.rs`

**Step 1: Run regression**

Run:

```powershell
cargo test --test ontology_schema_unit --test ontology_store_unit --test knowledge_record_unit --test knowledge_graph_store_unit --test capability_router_unit --test roaming_engine_unit --test retrieval_engine_unit --test evidence_assembler_unit --test navigation_pipeline_integration -- --nocapture
```

Expected:

- foundation 最小回归集全部通过

### Task 6: 更新交接与任务日志

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\docs\ai-handoff\AI_HANDOFF_MANUAL.md`
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\docs\execution-notes-2026-04-08-foundation-navigation-pipeline.md`
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\.trae\CHANGELOG_TASK.md`

**Step 1: Append factual notes**

记录：

- retrieval 已增加 source_ref 次级排序
- 当前 source 优先级只在 tie-break 阶段生效
- 仍不引入 `RetrievalConfig`

**Step 2: Final verification**

Run:

```powershell
git status --short --branch
```

Expected:

- 只包含本轮 foundation retrieval 增强与交接相关改动
