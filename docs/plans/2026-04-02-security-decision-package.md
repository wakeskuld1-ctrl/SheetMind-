# Security Decision Package Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为证券审批提交流程新增正式 `decision package` 对象，并将其与现有审批工件一并落盘和返回。

**Architecture:** 在不改变现有 `security_decision_submit_approval` 主入口的前提下，新增 `security_decision_package` 模块负责构造正式包合同。提交入口统一收集决策卡、审批请求、仓位计划、审批简报、可选签名以及事件/审计路径，组装为 `SecurityDecisionPackageDocument` 并写入 `decision_packages` 目录。

**Tech Stack:** Rust、serde/serde_json、sha2、文件型 runtime 持久化、CLI 合同测试

---

### Task 1: 写入 decision package 失败测试

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\security_decision_submit_approval_cli.rs`

**Step 1: Write the failing test**

- 在 ready case 测试里新增对 `decision_package` 与 `decision_package_path` 的断言。
- 新增签名场景下 package manifest 包含 `approval_brief_signature` 的断言。

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test security_decision_submit_approval_cli
```

Expected:

- 因为 `decision_package` 尚未存在而失败

### Task 2: 新增 package 合同模块

**Files:**
- Create: `D:\Rust\Excel_Skill\src\ops\security_decision_package.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\stock.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\mod.rs`

**Step 1: Write minimal implementation**

- 定义：
  - `SecurityDecisionPackageDocument`
  - `SecurityDecisionPackageArtifact`
  - `SecurityDecisionPackageGovernanceBinding`
  - `SecurityDecisionPackageBuildInput`
- 提供 builder：
  - `build_security_decision_package(...)`

**Step 2: Run focused test**

Run:

```powershell
cargo test --test security_decision_submit_approval_cli
```

Expected:

- 仍失败，但失败点前进到提交流程未写 package

### Task 3: 在提交入口落盘 package

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_submit_approval.rs`

**Step 1: Write minimal implementation**

- 扩展结果对象：
  - `decision_package`
  - `decision_package_path`
- 在落盘其他审批工件后生成 package
- 新增目录：
  - `decision_packages/<decision_id>.json`

**Step 2: Run focused test**

Run:

```powershell
cargo test --test security_decision_submit_approval_cli
```

Expected:

- package 相关断言转绿

### Task 4: 完整回归本切片

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\security_decision_submit_approval_cli.rs`

**Step 1: 补齐细化断言**

- 断言 `artifact_manifest` 中角色完整
- 断言 `governance_binding` 中引用一致
- 断言签名场景下 `approval_brief_signature` 为可选工件

**Step 2: Run targeted suite**

Run:

```powershell
cargo test --test security_decision_evidence_bundle_cli --test security_decision_committee_cli --test security_decision_submit_approval_cli
```

Expected:

- 全部通过

### Task 5: 收尾记录

**Files:**
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: 追加任务记录**

- 记录设计文档、实现文件、测试命令和剩余风险

**Step 2: 最终检查**

Run:

```powershell
git diff -- D:\Rust\Excel_Skill\src\ops\security_decision_package.rs D:\Rust\Excel_Skill\src\ops\security_decision_submit_approval.rs D:\Rust\Excel_Skill\tests\security_decision_submit_approval_cli.rs D:\Rust\Excel_Skill\docs\plans\2026-04-02-security-decision-package-design.md D:\Rust\Excel_Skill\docs\plans\2026-04-02-security-decision-package.md
```

Expected:

- 只包含本轮 `P0-4` 相关改动
