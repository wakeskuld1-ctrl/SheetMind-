# Security Decision Approval Bridge Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在主仓新增证券审批桥接层，把证券投决会结果提交到私有 worktree 的审批存储、多签、审计入口。

**Architecture:** 先在主仓新增桥接合同与持久化逻辑，再通过单一 Tool `security_decision_submit_approval` 暴露产品入口。实现坚持 TDD，先锁住 CLI 合同与落盘结构，再补最小实现，通过测试后再做必要重构。

**Tech Stack:** Rust、Cargo、serde/serde_json、现有股票 Tool 分发层、文件型 runtime 持久化

---

### Task 1: 写桥接设计守护测试

**Files:**
- Create: `D:\Rust\Excel_Skill\tests\security_decision_submit_approval_cli.rs`
- Reference: `D:\Rust\Excel_Skill\tests\security_decision_committee_cli.rs`

**Step 1: Write the failing test**

- 写 catalog 测试，要求 `security_decision_submit_approval` 出现在 tool catalog
- 写成功提交测试，要求生成兼容的 decision / approval / approval_events / audit 文件
- 写状态映射测试，要求 `ready_for_review` 和 `blocked` 分别正确落盘

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test security_decision_submit_approval_cli
```

Expected:

- FAIL
- 失败原因应为 tool 不存在或模块未实现

**Step 3: Commit**

本任务先不提交，继续实现直到该切片变绿。

### Task 2: 新增证券审批桥接合同与 brief 生成

**Files:**
- Create: `D:\Rust\Excel_Skill\src\ops\security_decision_approval_bridge.rs`
- Create: `D:\Rust\Excel_Skill\src\ops\security_decision_approval_brief.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\stock.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\mod.rs`

**Step 1: Write the failing test**

- 如果 Task 1 中未覆盖 brief 字段，则先补一个失败测试，要求输出含 `bull_summary` / `bear_summary` / `gate_summary`

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test security_decision_submit_approval_cli approval_brief
```

Expected:

- FAIL，提示字段不存在或内容不匹配

**Step 3: Write minimal implementation**

- 定义桥接输入输出对象
- 定义 approval brief 生成器
- 定义证券投决卡到私有审批卡的映射逻辑

**Step 4: Run test to verify it passes**

Run:

```powershell
cargo test --test security_decision_submit_approval_cli
```

Expected:

- 部分测试仍失败，但 brief 相关断言开始通过

### Task 3: 新增提交 Tool 与 runtime 持久化

**Files:**
- Create: `D:\Rust\Excel_Skill\src\ops\security_decision_submit_approval.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\dispatcher\stock_ops.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\dispatcher.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\catalog.rs`

**Step 1: Write the failing test**

- 若尚未精确锁住路径与文件内容，补充失败测试：
  - `decisions/*.json`
  - `approvals/*.json`
  - `approval_events/*.json`
  - `audit_log/*.jsonl`

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test security_decision_submit_approval_cli submit_writes_runtime_files -- --nocapture
```

Expected:

- FAIL，提示文件不存在或内容不兼容

**Step 3: Write minimal implementation**

- 定义请求与结果对象
- 调用 `security_decision_committee`
- 生成 `decision_ref` / `approval_ref`
- 按私有合同落盘 JSON
- 写初始空审批事件数组
- 写初始 audit 记录

**Step 4: Run test to verify it passes**

Run:

```powershell
cargo test --test security_decision_submit_approval_cli
```

Expected:

- 该测试文件整体 PASS

### Task 4: 接入工具目录和主分发层

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\tools\catalog.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\dispatcher.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\dispatcher\stock_ops.rs`

**Step 1: Write the failing test**

- 若 catalog 测试仍不够明确，补充“dispatcher 真能路由”的失败测试

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test security_decision_submit_approval_cli tool_catalog_includes_security_decision_submit_approval -- --nocapture
```

Expected:

- FAIL，提示 catalog 缺少工具名

**Step 3: Write minimal implementation**

- 在 catalog 注册工具名
- 在主 dispatcher 注册路由
- 在 stock dispatcher 解析请求并分发

**Step 4: Run test to verify it passes**

Run:

```powershell
cargo test --test security_decision_submit_approval_cli
```

Expected:

- 所有该文件测试通过

### Task 5: 做回归验证与必要重构

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

- 只做轻量重构
- 保持注释与原因说明完整
- 不扩 scope

**Step 3: Run verification again**

Run:

```powershell
cargo test --test security_decision_evidence_bundle_cli --test security_decision_committee_cli --test security_decision_submit_approval_cli
```

Expected:

- 仍然全部 PASS

### Task 6: 记录风险、测试建议与任务日志

**Files:**
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Summarize risks**

- 记录合同漂移风险
- 记录默认 runtime path 风险
- 记录审计兼容风险

**Step 2: Append task journal**

- 按仓库要求补 `.trae/CHANGELOG_TASK.md`

**Step 3: Final verification note**

- 如有未跑全仓测试，明确说明

Plan complete and saved to `D:\Rust\Excel_Skill\docs\plans\2026-04-02-security-decision-approval-bridge.md`。
