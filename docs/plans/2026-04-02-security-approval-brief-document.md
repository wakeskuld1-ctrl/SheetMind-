# Security Approval Brief Document Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 把证券审批摘要升级成正式审批简报文档，并支持独立落盘、可选 detached signature 与 package-ready 输出。

**Architecture:** 保持 `security_decision_submit_approval` 作为统一提交入口，本次通过正式 `approval_brief` 文档对象替换当前轻量摘要，并在提交时同步写入 `approval_briefs` 目录。签名输出采用 detached signature，不改正文合同。package 集成先做 package-binding 元数据，不直接引入完整 package builder。

**Tech Stack:** Rust、Cargo、serde/serde_json、HMAC-SHA256、现有证券审批桥接链、CLI 合同测试

---

### Task 1: 先写 formal brief 守护测试

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\security_decision_submit_approval_cli.rs`

**Step 1: Write the failing test**

- 为 ready 场景新增断言：
  - 返回 `approval_brief_path`
  - 文档已落盘
  - 文档有 `brief_id`
  - 文档有 `contract_version`
  - 文档有 `decision_ref`
  - 文档有 `approval_ref`
  - 文档有 `package_binding`

- 新增签名场景测试：
  - 请求传入 signing args
  - 返回 `approval_brief_signature_path`
  - 签名文件存在
  - 签名 envelope 字段完整

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test security_decision_submit_approval_cli
```

Expected:

- FAIL
- 失败原因应为 `approval_brief_path` / signature 字段不存在或文档结构不匹配

### Task 2: 新增 formal brief 文档合同

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_approval_brief.rs`

**Step 1: Write the failing test**

- 如果 Task 1 未精确锁住文档内容，补充失败测试：
  - `recommended_review_action`
  - `executive_summary`
  - `position_plan_summary`

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test security_decision_submit_approval_cli
```

Expected:

- FAIL，提示 approval brief 文档字段缺失

**Step 3: Write minimal implementation**

- 升级 approval brief 为正式文档对象
- 新增 `package_binding`
- 新增 `position_plan_summary`
- 新增 `recommended_review_action`

**Step 4: Run test to verify it passes**

Run:

```powershell
cargo test --test security_decision_submit_approval_cli
```

Expected:

- formal brief 相关断言通过，签名相关仍可能失败

### Task 3: 新增 detached signature 输出

**Files:**
- Create: `D:\Rust\Excel_Skill\src\ops\security_approval_brief_signature.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\stock.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\mod.rs`
- Modify: `D:\Rust\Excel_Skill\Cargo.toml`

**Step 1: Write the failing test**

- 补充签名失败测试：
  - 提供 key_id + secret 时生成 signature 文件
  - envelope 含 `payload_sha256`
  - envelope 含 `contract_version`
  - envelope 含 `brief_id`

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test security_decision_submit_approval_cli
```

Expected:

- FAIL，提示签名路径或 envelope 字段不存在

**Step 3: Write minimal implementation**

- 新增 HMAC-SHA256 detached signature helper
- 写入 `approval_briefs/<decision_id>.signature.json`

**Step 4: Run test to verify it passes**

Run:

```powershell
cargo test --test security_decision_submit_approval_cli
```

Expected:

- 签名相关断言通过

### Task 4: 扩展提交入口与路径输出

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_submit_approval.rs`

**Step 1: Write the failing test**

- 若路径断言仍不完整，补一个失败测试：
  - `approval_brief_path`
  - `approval_brief_signature_path`

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test security_decision_submit_approval_cli
```

Expected:

- FAIL，提示路径字段缺失

**Step 3: Write minimal implementation**

- 提交时写入 `approval_briefs` 目录
- 可选写入 signature 文件
- 返回路径字段

**Step 4: Run test to verify it passes**

Run:

```powershell
cargo test --test security_decision_submit_approval_cli
```

Expected:

- 该测试文件全部通过

### Task 5: 证券链回归验证

**Files:**
- Modify: 仅限本次新增/改动文件

**Step 1: Run focused regression**

Run:

```powershell
cargo test --test security_decision_evidence_bundle_cli --test security_decision_committee_cli --test security_decision_submit_approval_cli
```

Expected:

- 三个证券链测试文件全部 PASS

**Step 2: Refactor carefully**

- 只做轻量整理
- 不扩 scope

**Step 3: Run verification again**

Run:

```powershell
cargo test --test security_decision_evidence_bundle_cli --test security_decision_committee_cli --test security_decision_submit_approval_cli
```

Expected:

- 仍然全部 PASS

### Task 6: 风险说明与任务日志

**Files:**
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Summarize risks**

- 记录 approval brief 合同膨胀风险
- 记录 detached signature 兼容风险
- 记录 package-builder 尚未实装的边界

**Step 2: Append task journal**

- 按仓库要求补 `.trae/CHANGELOG_TASK.md`

Plan complete and saved to `D:\Rust\Excel_Skill\docs\plans\2026-04-02-security-approval-brief-document.md`。
