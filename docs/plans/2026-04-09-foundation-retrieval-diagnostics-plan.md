# Foundation Retrieval Diagnostics Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 foundation retrieval 增加只在底座内部使用的排序/命中可解释 diagnostics 能力，同时保持现有 `retrieve()` 合同不变。

**Architecture:** 在 `retrieval_engine.rs` 内新增 diagnostics 结果对象与内部候选结构，让 `retrieve()` 复用同一套执行路径，但只暴露 hits；`retrieve_with_diagnostics()` 额外暴露排序解释信息。整个实现只留在 foundation retrieval 内部，不扩散到 pipeline、CLI、Tool 或 GUI。

**Tech Stack:** Rust, cargo test, foundation in-memory retrieval engine

---

### Task 1: 先锁 diagnostics 合同的红灯测试

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\tests\retrieval_engine_unit.rs`

**Step 1: Write the failing test**

- 新增 2 到 3 条测试：
  - `retrieval_engine_returns_diagnostics_aligned_with_ranked_hits`
  - `retrieval_engine_diagnostics_expose_text_and_tie_break_signals`
  - 如有必要，再补一条 `retrieve_still_preserves_original_contract`

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test retrieval_engine_unit retrieval_engine_returns_diagnostics_aligned_with_ranked_hits -- --nocapture
```

Expected:

- 编译失败或测试失败
- 原因指向 diagnostics API / struct 尚未存在

**Step 3: Commit**

暂不提交，进入最小实现。

### Task 2: 最小实现 retrieval diagnostics

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\src\ops\foundation\retrieval_engine.rs`

**Step 1: Write minimal implementation**

- 新增：
  - `RetrievalExecution`
  - `RetrievalDiagnostic`
  - 必要的内部候选结构
- 新增：
  - `retrieve_with_diagnostics()`
- 保持：
  - `retrieve()` 继续返回 `Vec<RetrievalHit>`
  - `retrieve()` 内部委托给 diagnostics 执行路径

**Step 2: Run test to verify it passes**

Run:

```powershell
cargo test --test retrieval_engine_unit -- --nocapture
```

Expected:

- 新增 diagnostics 测试通过
- 既有 retrieval 测试继续通过

### Task 3: 补最小回归验证

**Files:**
- No code changes required unless regression appears

**Step 1: Run foundation regression**

Run:

```powershell
cargo test --test ontology_schema_unit --test ontology_store_unit --test knowledge_record_unit --test knowledge_graph_store_unit --test capability_router_unit --test roaming_engine_unit --test retrieval_engine_unit --test evidence_assembler_unit --test navigation_pipeline_integration -- --nocapture
```

Expected:

- foundation 最小回归集全部通过

### Task 4: 更新 durable memory

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\docs\ai-handoff\AI_HANDOFF_MANUAL.md`
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\docs\execution-notes-2026-04-08-foundation-navigation-pipeline.md`
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\.trae\CHANGELOG_TASK.md`

**Step 1: Update handoff and notes**

- 追加 Task 11 下一层的 diagnostics 合同
- 写明 diagnostics 只解释排序，不改变排序行为
- 写明 `retrieve()` 合同仍保持不变

**Step 2: Update task journal**

- 按追加原则写入 `.trae/CHANGELOG_TASK.md`

### Task 5: 最终状态检查

**Files:**
- No code changes required

**Step 1: Run status check**

Run:

```powershell
git status --short --branch
```

Expected:

- 只确认当前工作区状态
- 不清理任何无关脏改动
