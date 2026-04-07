# Security Position Plan Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 新增可审批的证券仓位计划对象，并把它正式挂到已有 `decision_ref / approval_ref` 上，作为审批对象的一部分。

**Architecture:** 保持 `committee_result` 与 `approval_request` 的边界不变，本次新增独立 `position_plan` 对象，由规则型规划器生成，并由 `security_decision_submit_approval` 一并输出与落盘。approval brief 同步扩展仓位摘要，确保审批看到完整执行方案。

**Tech Stack:** Rust、Cargo、serde/serde_json、现有证券投决会链、文件型 runtime 持久化、CLI 合同测试

---

### Task 1: 写仓位计划守护测试

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\security_decision_submit_approval_cli.rs`

**Step 1: Write the failing test**

- 为 `ready_for_review` 场景新增断言：
  - 返回 `position_plan`
  - 返回 `position_plan_path`
  - `position_plan` 挂接 `decision_ref / approval_ref`
  - `position_plan` 已落盘
- 为 `blocked` 场景新增断言：
  - `suggested_gross_pct = 0`
  - `plan_status = blocked`
- 如有必要，补一个 `needs_more_evidence` 场景测试

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test security_decision_submit_approval_cli
```

Expected:

- FAIL
- 失败原因应为字段缺失或文件不存在

### Task 2: 新增 `position_plan` 合同与生成器

**Files:**
- Create: `D:\Rust\Excel_Skill\src\ops\security_position_plan.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\stock.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\mod.rs`

**Step 1: Write the failing test**

- 如果 Task 1 未精确锁住仓位级别，补一个失败测试：
  - `ready_for_review -> reviewable`
  - `blocked -> blocked`
  - `needs_more_evidence -> probe_only`

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test security_decision_submit_approval_cli
```

Expected:

- FAIL，提示 `position_plan` 结构或状态不匹配

**Step 3: Write minimal implementation**

- 定义 `SecurityPositionPlan`
- 定义 `entry_plan / add_plan / stop_loss_plan / take_profit_plan`
- 定义规则型生成器

**Step 4: Run test to verify it passes**

Run:

```powershell
cargo test --test security_decision_submit_approval_cli
```

Expected:

- 部分测试可能仍失败，但 `position_plan` 基本结构开始通过

### Task 3: 扩展审批桥接与审批摘要

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_approval_bridge.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_approval_brief.rs`

**Step 1: Write the failing test**

- 补充失败测试：
  - approval brief 里必须有仓位摘要
  - bridge 输出必须带 `position_plan`

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test security_decision_submit_approval_cli
```

Expected:

- FAIL，提示 approval brief 缺字段或 bridge 未返回 `position_plan`

**Step 3: Write minimal implementation**

- 在审批桥接结果中挂 `position_plan`
- 在 approval brief 中加入：
  - `risk_budget_summary`
  - `entry_summary`
  - `add_summary`
  - `stop_loss_summary`
  - `take_profit_summary`
  - `cancel_summary`

**Step 4: Run test to verify it passes**

Run:

```powershell
cargo test --test security_decision_submit_approval_cli
```

Expected:

- approval brief 与 bridge 相关断言通过

### Task 4: 扩展提交 Tool 与 runtime 落盘

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_submit_approval.rs`

**Step 1: Write the failing test**

- 补充失败测试：
  - `position_plans/<decision_id>.json` 已落盘
  - 返回结果中有 `position_plan_path`

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test security_decision_submit_approval_cli
```

Expected:

- FAIL，提示路径或文件不存在

**Step 3: Write minimal implementation**

- 提交 Tool 一并输出 `position_plan`
- 新增 `position_plans` 目录写入
- 返回 `position_plan_path`

**Step 4: Run test to verify it passes**

Run:

```powershell
cargo test --test security_decision_submit_approval_cli
```

Expected:

- 该测试文件全部通过

### Task 5: 回归验证与轻量重构

**Files:**
- Modify: 仅限本次新增/变更文件

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

- 记录仓位规则过硬风险
- 记录审批摘要扩展风险
- 记录投中扩展依赖风险

**Step 2: Append task journal**

- 按仓库要求补 `.trae/CHANGELOG_TASK.md`

Plan complete and saved to `D:\Rust\Excel_Skill\docs\plans\2026-04-02-security-position-plan.md`。
