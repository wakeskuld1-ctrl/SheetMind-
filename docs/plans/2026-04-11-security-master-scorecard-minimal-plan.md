# Security Master Scorecard Minimal Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 交付一个最小可用的 `security_master_scorecard` 正式对象与 CLI Tool，用于汇总未来多期限赚钱效益、回撤韧性和路径质量。

**Architecture:** 复用现有 `security_decision_committee`、`security_scorecard` 与 `security_forward_outcome`，新增一个“历史回放型总卡”对象，不替代主席裁决，也不假装已经是完整版线上预测总卡。先锁红测，再做对象实现、dispatcher 接线与 catalog 暴露。

**Tech Stack:** Rust、Serde JSON、现有 stock dispatcher、SQLite 历史行情、CLI 集成测试。

---

### Task 1: 锁定 master_scorecard CLI 合同

**Files:**
- Create: `D:\Rust\Excel_Skill\tests\security_master_scorecard_cli.rs`

**Step 1: Write the failing test**

- 新增 `tool_catalog_includes_security_master_scorecard`
- 新增 `security_master_scorecard_returns_formal_multi_horizon_profitability_summary`

**Step 2: Run test to verify it fails**

Run: `cargo test --test security_master_scorecard_cli -- --nocapture`

Expected:
- 因测试文件新增但 Tool 尚不存在而失败

**Step 3: Write minimal implementation**

- 暂不写实现，本任务只锁红测

**Step 4: Run test to verify it still fails for the right reason**

Run: `cargo test --test security_master_scorecard_cli -- --nocapture`

Expected:
- 明确失败在 `catalog / dispatcher / object not found`

### Task 2: 实现正式对象与聚合逻辑

**Files:**
- Create: `D:\Rust\Excel_Skill\src\ops\security_master_scorecard.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\stock.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\mod.rs`

**Step 1: Write minimal implementation**

- 新增请求合同、正式对象合同、聚合返回合同
- 新增多期限分项分数与总卡聚合函数
- 复用 `security_decision_committee` + `build_security_scorecard` + `security_forward_outcome`

**Step 2: Run focused test**

Run: `cargo test --test security_master_scorecard_cli -- --nocapture`

Expected:
- 仍失败，但失败点收敛到 dispatcher / catalog 未接线或字段不匹配

### Task 3: 接入 CLI Tool 主链

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\tools\catalog.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\dispatcher.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\dispatcher\stock_ops.rs`

**Step 1: Write minimal implementation**

- catalog 增加 `security_master_scorecard`
- stock dispatcher 增加 `dispatch_security_master_scorecard`
- 主 dispatcher 路由接线

**Step 2: Run focused test**

Run: `cargo test --test security_master_scorecard_cli -- --nocapture`

Expected:
- 新测试转绿

### Task 4: 跑相关回归

**Files:**
- Test only

**Step 1: Run verification**

Run:
- `cargo test --test security_master_scorecard_cli -- --nocapture`
- `cargo test --test security_forward_outcome_cli -- --nocapture`
- `cargo test --test security_scorecard_training_cli security_scorecard_training_generates_artifact_and_registers_refit_outputs -- --nocapture`

**Step 2: Confirm output**

Expected:
- `security_master_scorecard_cli` 全绿
- `security_forward_outcome_cli` 不回退
- 评分卡训练最小主链不因新对象接入而回退

### Task 5: 补任务日志

**Files:**
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Append journal entry**

- 记录本轮设计、实现范围、验证命令和剩余缺口

**Step 2: Final verification**

Run:
- `git diff -- D:\Rust\Excel_Skill\docs\plans\2026-04-11-security-master-scorecard-minimal-design.md D:\Rust\Excel_Skill\docs\plans\2026-04-11-security-master-scorecard-minimal-plan.md D:\Rust\Excel_Skill\src\ops\security_master_scorecard.rs D:\Rust\Excel_Skill\src\tools\catalog.rs D:\Rust\Excel_Skill\src\tools\dispatcher.rs D:\Rust\Excel_Skill\src\tools\dispatcher\stock_ops.rs D:\Rust\Excel_Skill\tests\security_master_scorecard_cli.rs D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

Expected:
- 只包含本轮 master_scorecard 相关改动
