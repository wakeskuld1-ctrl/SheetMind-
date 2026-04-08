# Security Position Plan Approval Binding Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 把仓位计划升级为正式可审批对象，并让审批请求与审批包都能稳定绑定、校验该对象。

**Architecture:** 在 `SecurityPositionPlan` 中新增合同头、审批绑定块与 `reduce_plan`，在 `PersistedApprovalRequest` 中新增 `position_plan_binding`，由 bridge / submit 负责写入，由 verify 负责校验 `approval_request <-> position_plan <-> package.object_graph` 的一致性。

**Tech Stack:** Rust、Serde JSON、CLI 集成测试、现有 `security_decision_submit_approval / security_decision_verify_package / security_decision_package_revision` 主链。

---

### Task 1: 锁定仓位计划正式审批绑定合同

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\security_decision_submit_approval_cli.rs`
- Modify: `D:\Rust\Excel_Skill\tests\security_decision_verify_package_cli.rs`

**Step 1: Write the failing test**

- 在 `security_decision_submit_approval_writes_runtime_files_for_ready_case` 中新增断言：
  - `approval_request.position_plan_binding.position_plan_ref == position_plan.plan_id`
  - `position_plan.contract_version == "security_position_plan.v2"`
  - `position_plan.approval_binding.approval_ref == approval_ref`
  - `position_plan.reduce_plan.allow_reduce == true`
- 在 `security_decision_verify_package_accepts_signed_package_and_writes_report` 中新增断言：
  - `governance_checks.position_plan_binding_consistent == true`
  - `governance_checks.position_plan_complete == true`
  - `governance_checks.position_plan_direction_aligned == true`

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test security_decision_submit_approval_cli security_decision_submit_approval_writes_runtime_files_for_ready_case -- --nocapture
cargo test --test security_decision_verify_package_cli security_decision_verify_package_accepts_signed_package_and_writes_report -- --nocapture
```

Expected:

- 因为 `position_plan_binding`、`contract_version`、`reduce_plan` 或新校验结果不存在而失败

### Task 2: 最小实现 position_plan 正式合同与审批绑定

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_position_plan.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_approval_bridge.rs`

**Step 1: Write minimal implementation**

- 给 `SecurityPositionPlan` 增加：
  - `contract_version`
  - `document_type`
  - `decision_id`
  - `plan_direction`
  - `approval_binding`
  - `reduce_plan`
- 给 `PersistedApprovalRequest` 增加 `position_plan_binding`
- 在 bridge 中构建 `position_plan_binding`

**Step 2: Run targeted tests**

Run:

```powershell
cargo test --test security_decision_submit_approval_cli security_decision_submit_approval_writes_runtime_files_for_ready_case -- --nocapture
```

Expected:

- submit approval 相关新增断言转绿

### Task 3: 为 verify 增加仓位计划审批链校验

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_verify_package.rs`
- Modify: `D:\Rust\Excel_Skill\tests\security_decision_verify_package_cli.rs`

**Step 1: Write the failing test**

- 新增一个测试，篡改 `approval_request.position_plan_binding.position_plan_ref`
- 新增一个测试，篡改 `position_plan.plan_direction`
- 预期 `package_valid == false`

**Step 2: Verify RED**

Run:

```powershell
cargo test --test security_decision_verify_package_cli -- --nocapture
```

Expected:

- 新增“position plan binding / direction”测试失败

**Step 3: Write minimal implementation**

- 在 `SecurityDecisionPackageGovernanceCheck` 中新增：
  - `position_plan_binding_consistent`
  - `position_plan_complete`
  - `position_plan_direction_aligned`
- 校验审批请求、仓位计划和 package 对象图三方一致

**Step 4: Verify GREEN**

Run:

```powershell
cargo test --test security_decision_verify_package_cli -- --nocapture
```

Expected:

- verify package 测试全部通过

### Task 4: 回归 revision 与主链

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\security_decision_package_revision_cli.rs`

**Step 1: Add regression assertions**

- 确认 revision 后的 approval_request 仍保留 `position_plan_binding`
- 确认 v2 package 继续绑定同一仓位计划对象

**Step 2: Run regression**

Run:

```powershell
cargo test --test security_decision_submit_approval_cli -- --nocapture
cargo test --test security_decision_verify_package_cli -- --nocapture
cargo test --test security_decision_package_revision_cli -- --nocapture
```

Expected:

- 三条链路全绿

### Task 5: 完成任务日志

**Files:**
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Append task journal entry**

- 记录 position plan 正式合同、审批绑定、verify 新校验项与测试命令

**Step 2: Final verification**

Run:

```powershell
git status --short --branch
```

Expected:

- 仅包含本轮 Task 2 相关代码、测试与文档变更
