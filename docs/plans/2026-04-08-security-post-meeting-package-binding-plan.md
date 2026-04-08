# Security Post Meeting Package Binding Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 把 `post_meeting_conclusion` 正式接入证券审批 package 合同，使 revision 能写入它、verify 能校验它。

**Architecture:** 继续复用现有 `security_record_post_meeting_conclusion -> security_decision_package_revision -> security_decision_verify_package` 主链，只在 package object graph、artifact manifest 和 verify checks 上做增量扩展。所有正式存储值继续保持英文合同值，展示翻译层不进入本轮范围。

**Tech Stack:** Rust、Serde JSON、CLI 集成测试、现有证券治理 runtime fixtures、文件型工件哈希校验。

---

### Task 1: 为 package revision 锁定会后结论挂接红测

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\tests\security_decision_package_revision_cli.rs`

**Step 1: Write the failing test**

- 在现有 revision happy path 基础上新增断言：
  - `artifact_manifest` 中存在 `post_meeting_conclusion`
  - `object_graph.post_meeting_conclusion_ref`
  - `object_graph.post_meeting_conclusion_path`

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test security_decision_package_revision_cli -- --nocapture
```

Expected:

- 因 package 还没有挂接会后结论而失败

### Task 2: 为 record conclusion 主链锁定 package 引用红测

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\tests\security_post_meeting_conclusion_cli.rs`

**Step 1: Write the failing test**

- 在现有 happy path 基础上新增断言：
  - 新 package `artifact_manifest` 包含 `post_meeting_conclusion`
  - 新 package `object_graph` 带 `post_meeting_conclusion_ref/path`
  - 返回的 `post_meeting_conclusion` 与 package 引用一致

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test security_post_meeting_conclusion_cli -- --nocapture
```

Expected:

- 因 revision 产物尚未正式挂接会后结论而失败

### Task 3: 扩展 package 合同，加入会后结论引用位

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\src\ops\security_decision_package.rs`

**Step 1: Write minimal implementation**

- 为 package object graph 增加：
  - `post_meeting_conclusion_ref: Option<String>`
  - `post_meeting_conclusion_path: Option<String>`
- 保持初始 package 兼容可空

**Step 2: Run targeted tests**

Run:

```powershell
cargo test --test security_post_meeting_conclusion_cli -- --nocapture
```

Expected:

- 从“字段缺失”推进到“revision 未赋值”阶段

### Task 4: 扩展 package revision，把会后结论写进新 package

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\src\ops\security_decision_package_revision.rs`
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\src\ops\security_record_post_meeting_conclusion.rs`

**Step 1: Write minimal implementation**

- revision 支持接收会后结论路径/引用
- 重建 manifest 时追加或更新 `post_meeting_conclusion`
- 构建新 package 时写入 `object_graph.post_meeting_conclusion_ref/path`

**Step 2: Run tests to verify green**

Run:

```powershell
cargo test --test security_decision_package_revision_cli -- --nocapture
cargo test --test security_post_meeting_conclusion_cli -- --nocapture
```

Expected:

- 两组测试通过，且新 package 正式带有会后结论引用

### Task 5: 为 verify 锁定绑定与完整性红测

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\tests\security_decision_verify_package_cli.rs`

**Step 1: Write the failing tests**

- 新增 happy path：
  - `post_meeting_conclusion_binding_consistent == true`
  - `post_meeting_conclusion_brief_paired == true`
  - `post_meeting_conclusion_complete == true`
- 新增 tamper path：
  - 篡改 `source_brief_ref`
  - 篡改 `source_package_path`
  - 篡改 `final_disposition`

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test security_decision_verify_package_cli -- --nocapture
```

Expected:

- 因 verify 尚未识别会后结论检查而失败

### Task 6: 扩展 verify 逻辑

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\src\ops\security_decision_verify_package.rs`

**Step 1: Write minimal implementation**

- 增加三项检查：
  - `post_meeting_conclusion_binding_consistent`
  - `post_meeting_conclusion_brief_paired`
  - `post_meeting_conclusion_complete`
- 保持无会后结论的旧 package 兼容

**Step 2: Run tests to verify green**

Run:

```powershell
cargo test --test security_decision_verify_package_cli -- --nocapture
```

Expected:

- happy path 和 tamper path 都按预期通过

### Task 7: 跑主链回归

**Files:**
- Test: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\tests\security_post_meeting_conclusion_cli.rs`
- Test: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\tests\security_decision_package_revision_cli.rs`
- Test: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\tests\security_decision_verify_package_cli.rs`
- Test: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\tests\security_decision_submit_approval_cli.rs`

**Step 1: Run regression**

Run:

```powershell
cargo test --test security_post_meeting_conclusion_cli -- --nocapture
cargo test --test security_decision_package_revision_cli -- --nocapture
cargo test --test security_decision_verify_package_cli -- --nocapture
cargo test --test security_decision_submit_approval_cli -- --nocapture
```

Expected:

- 四条证券治理相关 CLI 主链通过

### Task 8: 更新交接记录

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\docs\execution-notes-2026-04-08-security-post-meeting-conclusion.md`
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\docs\ai-handoff\AI_HANDOFF_MANUAL.md`
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-security-post-meeting-package-binding\.trae\CHANGELOG_TASK.md`

**Step 1: Append factual notes**

- 记录 package 合同如何挂接会后结论
- 记录 verify 新增了哪些检查
- 记录本轮验证命令和结果

**Step 2: Final verification**

Run:

```powershell
git status --short --branch
```

Expected:

- 只包含本轮 Task 11 相关改动
