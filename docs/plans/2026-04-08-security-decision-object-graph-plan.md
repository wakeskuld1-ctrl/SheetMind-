# Security Decision Object Graph Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为证券审批包补齐显式对象图绑定块，并让校验工具能够验证对象引用与落盘路径一致性。

**Architecture:** 在 `SecurityDecisionPackageDocument` 中引入统一 `object_graph` 结构，集中表达 `decision_ref / approval_ref / position_plan_ref / approval_brief_ref` 及其路径；由 `submit_approval` 负责写入，由 `verify_package` 负责校验，并保持现有顶层 `decision_ref / approval_ref` 的兼容输出。

**Tech Stack:** Rust、Serde JSON、CLI 集成测试、现有 `security_decision_submit_approval / security_decision_verify_package / security_decision_package_revision` 主链。

---

### Task 1: 锁定 package 显式对象图合同

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\security_decision_submit_approval_cli.rs`
- Modify: `D:\Rust\Excel_Skill\tests\security_decision_verify_package_cli.rs`

**Step 1: Write the failing test**

- 在 `security_decision_submit_approval_writes_runtime_files_for_ready_case` 中新增断言：
  - `decision_package.object_graph.decision_ref == data.decision_ref`
  - `decision_package.object_graph.approval_ref == data.approval_ref`
  - `decision_package.object_graph.position_plan_ref == data.position_plan.plan_id`
  - `decision_package.object_graph.approval_brief_ref == data.approval_brief.brief_id`
- 在 `security_decision_verify_package_accepts_signed_package_and_writes_report` 中新增断言：
  - `governance_checks.object_graph_consistent == true`

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test security_decision_submit_approval_cli security_decision_submit_approval_writes_runtime_files_for_ready_case -- --nocapture
cargo test --test security_decision_verify_package_cli security_decision_verify_package_accepts_signed_package_and_writes_report -- --nocapture
```

Expected:

- 因为 `object_graph` 字段不存在或校验结果不存在而失败

### Task 2: 最小实现 package 对象图结构

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_package.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_submit_approval.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_package_revision.rs`

**Step 1: Write minimal implementation**

- 在 `security_decision_package.rs` 新增 `SecurityDecisionPackageObjectGraph`
- 在 `SecurityDecisionPackageDocument` 中增加 `object_graph`
- 在 `SecurityDecisionPackageBuildInput` 中增加对象图所需字段
- 在 `build_decision_package_document` 中把 `plan_id / brief_id / 各对象路径` 写入 `object_graph`
- 在 `package_revision` 重建 package 时沿用上一版 package 的 `object_graph`

**Step 2: Run targeted tests**

Run:

```powershell
cargo test --test security_decision_submit_approval_cli security_decision_submit_approval_writes_runtime_files_for_ready_case -- --nocapture
```

Expected:

- submit approval 相关新增断言转绿

### Task 3: 为 verify 增加对象图一致性校验

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_verify_package.rs`
- Modify: `D:\Rust\Excel_Skill\tests\security_decision_verify_package_cli.rs`

**Step 1: Write the failing test**

- 新增一个篡改 package 中 `object_graph.position_plan_ref` 或 `approval_brief_path` 的测试
- 预期 `package_valid == false`
- 预期 issues 中有对象图不一致信息

**Step 2: Verify RED**

Run:

```powershell
cargo test --test security_decision_verify_package_cli -- --nocapture
```

Expected:

- 新增“篡改对象图”测试失败

**Step 3: Write minimal implementation**

- 在 `SecurityDecisionPackageGovernanceCheck` 增加对象图一致性字段
- 读取 `decision_card / approval_request / position_plan / approval_brief`
- 校验 ref 与 path 是否一致
- 失败时写入 `issues`

**Step 4: Verify GREEN**

Run:

```powershell
cargo test --test security_decision_verify_package_cli -- --nocapture
```

Expected:

- verify package 测试全部通过

### Task 4: 回归 package revision 与主链

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\security_decision_package_revision_cli.rs`

**Step 1: Add regression assertions**

- 确认 revision 后的新 package 仍带 `object_graph`
- 确认版本升级不丢失 `position_plan_ref / approval_brief_ref`

**Step 2: Run regression**

Run:

```powershell
cargo test --test security_decision_package_revision_cli -- --nocapture
cargo test --test security_decision_submit_approval_cli -- --nocapture
cargo test --test security_decision_verify_package_cli -- --nocapture
```

Expected:

- 三条链路全绿

### Task 5: 完成文档与任务日志

**Files:**
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Append task journal entry**

- 记录对象图字段、校验逻辑、回归测试

**Step 2: Final verification**

Run:

```powershell
git status --short --branch
```

Expected:

- 仅包含本轮 Task 1 相关文档、代码与测试变更
