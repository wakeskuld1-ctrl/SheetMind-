# Security Decision Package Revision Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为证券审批包增加版本化能力，使审批动作发生后可以生成新的 package 版本并可选重跑校验。

**Architecture:** 在现有 `security_decision_package` 与 `security_decision_verify_package` 基础上，新增 `security_decision_package_revision` 模块。该模块读取旧 package 与最新审批工件，生成新的 package 版本、写入新路径，并在需要时复用 verify 逻辑生成新的 verification report。

**Tech Stack:** Rust、serde/serde_json、sha2、文件型 runtime 持久化、CLI 合同测试

---

### Task 1: 写入 revision 失败测试

**Files:**
- Create: `D:\Rust\Excel_Skill\tests\security_decision_package_revision_cli.rs`

**Step 1: Write the failing test**

- 先调用 `security_decision_submit_approval` 生成 v1 package
- 手工更新：
  - `approval_request.status`
  - `approval_events`
  - `audit_log`
- 再调用 `security_decision_package_revision`
- 断言：
  - tool catalog 可发现
  - `package_version = 2`
  - `previous_package_path` 指向 v1
  - `revision_reason` 和 `trigger_event_summary` 存在
  - 新 verification report 返回

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test security_decision_package_revision_cli
```

Expected:

- 因为 `security_decision_package_revision` 尚未存在而失败

### Task 2: 扩展 package 合同与 builder

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_package.rs`

**Step 1: Write minimal implementation**

- 为 `SecurityDecisionPackageDocument` 增加：
  - `package_version`
  - `previous_package_path`
  - `revision_reason`
  - `trigger_event_summary`
- 更新 builder 输入与状态推导

**Step 2: Run focused test**

Run:

```powershell
cargo test --test security_decision_package_revision_cli
```

Expected:

- 仍失败，但失败点前进到 revision Tool 不存在

### Task 3: 新增 revision Tool

**Files:**
- Create: `D:\Rust\Excel_Skill\src\ops\security_decision_package_revision.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\stock.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\mod.rs`

**Step 1: Write minimal implementation**

- 读取旧 package
- 读取最新审批工件
- 生成 v2 package
- 可选调用 verify 生成新 report

**Step 2: Run focused test**

Run:

```powershell
cargo test --test security_decision_package_revision_cli
```

Expected:

- 主路径转绿

### Task 4: 接入 Tool 分发

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\tools\catalog.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\dispatcher.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\dispatcher\stock_ops.rs`

**Step 1: Write minimal implementation**

- 加入 tool catalog
- 加入主 dispatcher 与 stock dispatcher

**Step 2: Run targeted suite**

Run:

```powershell
cargo test --test security_decision_package_revision_cli --test security_decision_verify_package_cli
```

Expected:

- revision 和 verify 共同通过

### Task 5: 回归与收尾

**Files:**
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Run full targeted suite**

```powershell
cargo test --test security_decision_evidence_bundle_cli --test security_decision_committee_cli --test security_decision_submit_approval_cli --test security_decision_verify_package_cli --test security_decision_package_revision_cli
```

Expected:

- 全部通过

**Step 2: 追加任务记录**

- 记录设计文档、实现文件、测试命令和剩余风险
