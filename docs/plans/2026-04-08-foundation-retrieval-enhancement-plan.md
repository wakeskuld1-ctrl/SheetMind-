# Foundation Retrieval Enhancement Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 foundation retrieval engine 增加第一层排序增强，在不拆 RetrievalConfig 的前提下提升候选域内证据排序稳定性。  
**Architecture:** 保持 `RetrievalEngine` 无状态与 `RetrievalHit` 合同不变，只在 `retrieval_engine.rs` 内部增加标题优先、短语 bonus、seed concept bonus 三类轻量评分信号。  
**Tech Stack:** Rust、Cargo unit tests、纯内存 graph/ontology fixture、固定权重评分。

---

### Task 1: 为标题优先规则补红灯测试

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\tests\retrieval_engine_unit.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn retrieval_engine_prefers_title_match_over_body_only_match() {
    let hits = sample_retrieval_engine()
        .retrieve(
            "sales review",
            &sample_candidate_scope_for_revenue_only(),
            &sample_graph_store_for_title_body_ranking(),
        )
        .unwrap();

    assert_eq!(hits[0].node_id, "node-title-match");
}
```

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test retrieval_engine_unit retrieval_engine_prefers_title_match_over_body_only_match -- --nocapture
```

Expected:

- 测试失败，因为当前实现还没有标题权重优势

### Task 2: 为短语优先规则补红灯测试

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\tests\retrieval_engine_unit.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn retrieval_engine_prefers_exact_phrase_match_over_scattered_tokens() {
    let hits = sample_retrieval_engine()
        .retrieve(
            "sales trend",
            &sample_candidate_scope_for_revenue_and_trend(),
            &sample_graph_store_for_phrase_ranking(),
        )
        .unwrap();

    assert_eq!(hits[0].node_id, "node-phrase-match");
}
```

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test retrieval_engine_unit retrieval_engine_prefers_exact_phrase_match_over_scattered_tokens -- --nocapture
```

Expected:

- 测试失败，因为当前实现只统计 token 交集，不识别完整短语 bonus

### Task 3: 为 seed concept 优先规则补红灯测试

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\tests\retrieval_engine_unit.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn retrieval_engine_prefers_seed_concept_nodes_over_roamed_nodes_when_scores_tie() {
    let hits = sample_retrieval_engine()
        .retrieve(
            "sales trend",
            &sample_candidate_scope_with_roaming_path(),
            &sample_graph_store_for_seed_priority(),
        )
        .unwrap();

    assert_eq!(hits[0].node_id, "node-seed-revenue");
}
```

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test retrieval_engine_unit retrieval_engine_prefers_seed_concept_nodes_over_roamed_nodes_when_scores_tie -- --nocapture
```

Expected:

- 测试失败，因为当前实现还没有 seed concept bonus

### Task 4: 实现最小排序增强

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\src\ops\foundation\retrieval_engine.rs`

**Step 1: Write minimal implementation**

新增最小辅助逻辑：

- `normalized_text()`
- `seed_concept_ids()`
- `score_node()`

评分合成：

- `title` 命中 > `body` 命中
- 完整短语命中加分
- 节点 concept 命中 seed concept 加分

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

- retrieval 已进入第一层排序增强
- 当前增强只覆盖标题优先、短语 bonus、seed bonus
- 仍不引入 RetrievalConfig

**Step 2: Final verification**

Run:

```powershell
git status --short --branch
```

Expected:

- 只包含本轮 foundation retrieval 增强与交接相关改动
