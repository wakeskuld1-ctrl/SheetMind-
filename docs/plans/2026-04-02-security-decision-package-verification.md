# Security Decision Package Verification Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为证券审批包新增正式校验入口，并输出可落盘的 verification report。

**Architecture:** 新增 `security_decision_verify_package` 模块负责读取 package、遍历 artifact manifest、重算哈希、校验 detached signature 与治理绑定。通过 stock dispatcher 和 catalog 暴露正式 Tool，并将校验结果落盘到独立 verification 目录。

**Tech Stack:** Rust、serde/serde_json、sha2、hmac、文件型 runtime 持久化、CLI 合同测试

---

### Task 1: 写入 verification 失败测试

**Files:**
- Create: `D:\Rust\Excel_Skill\tests\security_decision_verify_package_cli.rs`

**Step 1: Write the failing test**

- 先走一遍 `security_decision_submit_approval` 生成 package
- 再调用 `security_decision_verify_package`
- 断言：
  - tool catalog 可发现
  - `package_valid = true`
  - `verification_report_path` 落盘
- 再补一个篡改 `approval_brief` 后失败的测试

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test security_decision_verify_package_cli
```

Expected:

- 因为 `security_decision_verify_package` 尚未存在而失败

### Task 2: 新增 verification 模块

**Files:**
- Create: `D:\Rust\Excel_Skill\src\ops\security_decision_verify_package.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_approval_brief_signature.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\stock.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\mod.rs`

**Step 1: Write minimal implementation**

- 定义 request / result / report 合同
- 新增：
  - artifact check
  - hash check
  - signature check
  - governance check
- 在签名模块里补 reusable verify helper

**Step 2: Run focused test**

Run:

```powershell
cargo test --test security_decision_verify_package_cli
```

Expected:

- 仍失败，但失败点前进到 dispatcher / catalog 尚未接入或细节断言

### Task 3: 接入 Tool 分发与落盘

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\tools\catalog.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\dispatcher.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\dispatcher\stock_ops.rs`

**Step 1: Write minimal implementation**

- 将 `security_decision_verify_package` 加入 stock tool catalog
- 加入 dispatcher 主路由和 stock dispatcher
- 支持 verification report 落盘

**Step 2: Run focused test**

Run:

```powershell
cargo test --test security_decision_verify_package_cli
```

Expected:

- happy path 转绿

### Task 4: 覆盖篡改与 warning 场景

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\security_decision_verify_package_cli.rs`

**Step 1: 补齐细化断言**

- 篡改 `approval_brief` 后：
  - `package_valid = false`
  - 哈希与签名检查失败
- 未签名 package：
  - optional signature 工件 `present=false`
  - report 仍可通过

**Step 2: Run targeted suite**

Run:

```powershell
cargo test --test security_decision_evidence_bundle_cli --test security_decision_committee_cli --test security_decision_submit_approval_cli --test security_decision_verify_package_cli
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
git diff -- D:\Rust\Excel_Skill\src\ops\security_decision_verify_package.rs D:\Rust\Excel_Skill\src\ops\security_approval_brief_signature.rs D:\Rust\Excel_Skill\src\tools\catalog.rs D:\Rust\Excel_Skill\src\tools\dispatcher.rs D:\Rust\Excel_Skill\src\tools\dispatcher\stock_ops.rs D:\Rust\Excel_Skill\tests\security_decision_verify_package_cli.rs D:\Rust\Excel_Skill\docs\plans\2026-04-02-security-decision-package-verification-design.md D:\Rust\Excel_Skill\docs\plans\2026-04-02-security-decision-package-verification.md
```

Expected:

- 只包含本轮 `P0-5` 相关改动
