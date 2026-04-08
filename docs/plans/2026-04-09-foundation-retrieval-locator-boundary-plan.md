# Foundation Retrieval Locator Boundary Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 让 foundation retrieval 的 locator hygiene 接受带 Windows 绝对路径前缀的外部工作簿 A1 风格范围 locator，同时保持其余合同不变。

**Architecture:** 继续只在 `src/ops/foundation/retrieval_engine.rs` 内补最小解析边界，复用现有 sheet-qualified A1 解析链，并把“先剥路径/工作簿前缀、再切范围”收口到局部 helper。通过 `tests/retrieval_engine_unit.rs` 先补红灯、再最小实现、最后跑 foundation 回归，不扩散到业务层或上层入口。

**Tech Stack:** Rust, cargo test, foundation retrieval engine

---

### Task 1: 先锁 Windows 绝对路径外部工作簿范围 locator 的红灯测试

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\tests\retrieval_engine_unit.rs`

**Step 1: Write the failing test**

- 新增至少 2 条测试：
  - `retrieval_engine_diagnostics_do_not_flag_windows_path_external_workbook_range_locator_as_weak`
  - `retrieval_engine_diagnostics_do_not_flag_windows_path_external_workbook_absolute_range_locator_as_weak`

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test retrieval_engine_unit windows_path_external_workbook -- --nocapture
```

Expected:

- 测试失败
- 失败原因指向当前 drive letter `:` 破坏了范围 locator 解析

### Task 2: 最小实现范围 locator 的前缀剥离

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\src\ops\foundation\retrieval_engine.rs`

**Step 1: Write minimal implementation**

- 最小修改 `parse_locator_range()` 与其邻近 helper
- 仅支持 `C:\Reports\[Workbook]Sheet!A1:B3` 这一类前缀剥离
- 不扩 3D 引用
- 不改命中和排序合同

**Step 2: Run test to verify it passes**

Run:

```powershell
cargo test --test retrieval_engine_unit windows_path_external_workbook -- --nocapture
```

Expected:

- 新增 Windows 路径外部工作簿 locator 测试通过

### Task 3: 回归验证

**Files:**
- No code changes required unless regression appears

**Step 1: Run retrieval engine unit**

Run:

```powershell
cargo test --test retrieval_engine_unit -- --nocapture
```

Expected:

- `retrieval_engine_unit` 全部通过

**Step 2: Run foundation regression**

Run:

```powershell
cargo test --test ontology_schema_unit --test ontology_store_unit --test knowledge_record_unit --test knowledge_graph_store_unit --test capability_router_unit --test roaming_engine_unit --test retrieval_engine_unit --test evidence_assembler_unit --test navigation_pipeline_integration -- --nocapture
```

Expected:

- foundation 最小回归集全部通过

### Task 4: 更新 durable memory

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\docs\execution-notes-2026-04-08-foundation-navigation-pipeline.md`
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\docs\ai-handoff\AI_HANDOFF_MANUAL.md`
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\.trae\CHANGELOG_TASK.md`

**Step 1: Update docs**

- 追加 locator hygiene 新边界
- 写清楚 Windows 绝对路径外部工作簿范围 locator 现在被接受
- 写清楚当前仍不支持 named range / 3D 引用

**Step 2: Append task journal**

- 按追加原则记录本轮实现与验证结果
---

## Post-Implementation Extension (2026-04-09 CST)

### Task 5: Lock Windows-path large-range weak-locator boundary

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\tests\retrieval_engine_unit.rs`
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\docs\execution-notes-2026-04-08-foundation-navigation-pipeline.md`
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\docs\ai-handoff\AI_HANDOFF_MANUAL.md`
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\.trae\CHANGELOG_TASK.md`

**Step 1: Add protection tests**

- 新增 2 条回归测试：
  - `retrieval_engine_diagnostics_still_flag_windows_path_external_workbook_large_range_locator_as_weak`
  - `retrieval_engine_diagnostics_still_flag_windows_path_external_workbook_absolute_large_range_locator_as_weak`

**Step 2: Verify current implementation**

Run:

```powershell
cargo test --test retrieval_engine_unit windows_path_external_workbook_large_range -- --nocapture
cargo test --test retrieval_engine_unit absolute_large_range -- --nocapture
```

Expected:

- 两条新测试直接通过
- 说明当前实现无需继续修改生产代码
- 需要把“Windows 路径前缀不影响 large range weak 语义”写入 durable memory
