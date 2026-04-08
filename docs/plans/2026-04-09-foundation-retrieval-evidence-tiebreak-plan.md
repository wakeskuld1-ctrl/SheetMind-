# Foundation Retrieval Evidence Tie-Break Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 foundation retrieval engine 增加证据数量与 locator 精度的双层 tie-break，让同分同来源层级下的节点排序更稳定、更可解释。

**Architecture:** 保持文本分数与 `source_ref` 优先级的既有主轴不变，只在后续 tie-break 阶段增加证据数量与 locator 精度排序。实现继续限制在 `retrieval_engine.rs` 内部，不修改 `RetrievalHit`，不引入 `RetrievalConfig`。

**Tech Stack:** Rust、Cargo unit tests、纯内存 fixture、最小 locator 启发式解析。

---

### Task 1: 补证据数量优先的红灯测试

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\tests\retrieval_engine_unit.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn retrieval_engine_prefers_more_evidence_refs_when_scores_and_source_priority_tie() {
    let hits = sample_retrieval_engine()
        .retrieve(
            "sales review",
            &sample_candidate_scope_for_revenue_only(),
            &sample_graph_store_for_evidence_count_priority(),
        )
        .unwrap();

    assert_eq!(hits[0].node_id, "node-z-more-evidence");
}
```

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test retrieval_engine_unit retrieval_engine_prefers_more_evidence_refs_when_scores_and_source_priority_tie -- --nocapture
```

Expected:

- 测试失败，因为当前实现还没有证据数量 tie-break

### Task 2: 补 locator 精度优先的红灯测试

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\tests\retrieval_engine_unit.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn retrieval_engine_prefers_more_specific_locator_when_scores_source_and_counts_tie() {
    let hits = sample_retrieval_engine()
        .retrieve(
            "sales review",
            &sample_candidate_scope_for_revenue_only(),
            &sample_graph_store_for_locator_precision_priority(),
        )
        .unwrap();

    assert_eq!(hits[0].node_id, "node-z-specific-locator");
}
```

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test retrieval_engine_unit retrieval_engine_prefers_more_specific_locator_when_scores_source_and_counts_tie -- --nocapture
```

Expected:

- 测试失败，因为当前实现还没有 locator 精度 tie-break

### Task 3: 补“来源优先级高于证据数量”的保护测试

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\tests\retrieval_engine_unit.rs`

**Step 1: Write the guardrail test**

```rust
#[test]
fn retrieval_engine_keeps_better_source_priority_ahead_of_more_evidence_refs() {
    let hits = sample_retrieval_engine()
        .retrieve(
            "sales review",
            &sample_candidate_scope_for_revenue_only(),
            &sample_graph_store_for_source_priority_over_evidence_count(),
        )
        .unwrap();

    assert_eq!(hits[0].node_id, "node-z-better-source");
}
```

**Step 2: Run test to verify the expected behavior**

Run:

```powershell
cargo test --test retrieval_engine_unit retrieval_engine_keeps_better_source_priority_ahead_of_more_evidence_refs -- --nocapture
```

Expected:

- 测试应保持通过，因为证据层只能排在 `source_ref` 之后

### Task 4: 最小实现证据侧双层 tie-break

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\src\ops\foundation\retrieval_engine.rs`

**Step 1: Write minimal implementation**

新增最小辅助逻辑：

- `evidence_ref_count_priority()`
- `best_locator_precision_priority()`
- `locator_precision_priority()`
- 如有需要，补最小 locator 解析辅助函数

排序顺序固定为：

1. 文本分数
2. `source_ref` 优先级
3. `evidence_refs` 数量
4. locator 精度
5. `node_id`

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

- retrieval 已新增证据数量与 locator 精度 tie-break
- 当前证据侧 tie-break 只在 `source_ref` 之后生效
- 仍不引入 `RetrievalConfig`

**Step 2: Final verification**

Run:

```powershell
git status --short --branch
```

Expected:

- 只包含本轮 foundation retrieval 增强与交接相关改动
