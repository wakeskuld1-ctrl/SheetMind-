# Foundation Retrieval Hygiene Diagnostics Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 foundation retrieval diagnostics 增加 evidence hygiene 诊断能力，识别重复证据、弱 locator、弱 source_ref，同时保持排序与命中行为不变。

**Architecture:** 继续只在 `src/ops/foundation/retrieval_engine.rs` 内增强 `RetrievalDiagnostic`。通过最小启发式规则输出 hygiene counts 和 flags，不引入配置层，不改 `RetrievalHit`，也不把 diagnostics 扩散到 pipeline、CLI、Tool、GUI。

**Tech Stack:** Rust, cargo test, foundation retrieval engine

---

### Task 1: 先锁 hygiene diagnostics 的红灯测试

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\tests\retrieval_engine_unit.rs`

**Step 1: Write the failing test**

- 新增至少 3 条测试：
  - `retrieval_engine_diagnostics_flag_duplicate_evidence_refs`
  - `retrieval_engine_diagnostics_flag_weak_locator_refs`
  - `retrieval_engine_diagnostics_flag_weak_source_refs`

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test retrieval_engine_unit retrieval_engine_diagnostics_flag_duplicate_evidence_refs -- --nocapture
```

Expected:

- 编译失败或测试失败
- 原因指向 hygiene 字段 / enum / helper 尚未存在

### Task 2: 最小实现 hygiene diagnostics

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\src\ops\foundation\retrieval_engine.rs`

**Step 1: Write minimal implementation**

- 新增：
  - `RetrievalHygieneFlag`
  - `duplicate_evidence_ref_count`
  - `weak_locator_count`
  - `weak_source_ref_count`
  - `hygiene_flags`
- 新增最小 helper：
  - 重复证据计数
  - 弱 locator 识别
  - 弱 source_ref 识别

**Step 2: Run test to verify it passes**

Run:

```powershell
cargo test --test retrieval_engine_unit -- --nocapture
```

Expected:

- 新增 hygiene diagnostics 测试通过
- 既有 retrieval diagnostics 测试继续通过

### Task 3: foundation 回归验证

**Files:**
- No code changes required unless regression appears

**Step 1: Run regression**

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

**Step 1: Update docs**

- 追加 hygiene diagnostics 合同
- 写明 hygiene 只解释质量风险，不参与排序和命中判定

**Step 2: Append task journal**

- 按追加原则记录本轮实现与验证结果

### Task 5: 最终状态检查

**Files:**
- No code changes required

**Step 1: Run status**

Run:

```powershell
git status --short --branch
```

Expected:

- 只做状态确认
- 不清理无关脏改动
