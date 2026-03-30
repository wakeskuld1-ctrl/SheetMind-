# Diagnostics Report Summary Handoff Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 把 `diagnostics_report_excel_report` 的 `执行摘要` 从字段摘要增强为分析承接摘要，让交付结果能直接回答“是否复核 / 是否补数 / 是否进入建模 / 下一步用什么 Tool”。

**Architecture:** 保持 `diagnostics_report -> diagnostics_report_excel_report -> workbook draft -> export_excel_workbook` 主链不变。本轮只在 `diagnostics_report_excel_report.rs` 的摘要组装层增加轻规则字段，并通过 CLI 导出测试锁定 shared strings 与承接口径，不新增 Tool，不改 dispatcher。

**Tech Stack:** Rust, Polars, serde, workbook draft store, rust_xlsxwriter export path, cargo test

---

### Task 1: 锁定执行摘要新增承接字段

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\diagnostics_report_excel_report_cli.rs`
- Test: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\diagnostics_report_excel_report_cli.rs`

**Step 1: Write the failing test**

在导出 `.xlsx` 的测试中新增断言，锁定 `sharedStrings.xml` 至少包含：

- `复核建议`
- `补数建议`
- `建模建议`
- `建议优先工具`
- `当前主要阻塞项`
- `进入下一步前需满足条件`

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test diagnostics_report_excel_report_cli -- --nocapture
```

Expected:

- 因新增承接字段尚未写入摘要页而失败

**Step 3: Write minimal implementation**

在 `src/ops/diagnostics_report_excel_report.rs` 中：

- 扩展 `build_summary_dataframe()`
- 先把承接字段固定写入摘要页

**Step 4: Run test to verify it passes**

Run:

```powershell
cargo test --test diagnostics_report_excel_report_cli -- --nocapture
```

Expected:

- 新增 shared strings 相关断言通过

### Task 2: 锁定降级场景的承接口径

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\diagnostics_report_excel_report_cli.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\diagnostics_report_excel_report.rs`

**Step 1: Write the failing test**

为降级场景补断言，锁定：

- 有 warning 时 `建模建议` 不能直接给“进入建模”
- 有 warning 时 `复核建议` 应落到保守口径
- `当前主要阻塞项` 应优先反映 warning 或缺失 section

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test diagnostics_report_excel_report_cli -- --nocapture
```

Expected:

- 因当前摘要规则尚未区分承接口径而失败

**Step 3: Write minimal implementation**

新增轻规则 helper，例如：

- `resolve_review_recommendation()`
- `resolve_data_completion_recommendation()`
- `resolve_modeling_recommendation()`
- `resolve_main_blocker()`
- `resolve_next_stage_condition()`

规则只复用现有 `diagnostics_result` 字段，不改底层逻辑。

**Step 4: Run test to verify it passes**

Run:

```powershell
cargo test --test diagnostics_report_excel_report_cli -- --nocapture
```

Expected:

- 降级承接判断相关断言通过

### Task 3: 锁定字段承接信息

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\diagnostics_report_excel_report_cli.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\diagnostics_report_excel_report.rs`

**Step 1: Write the failing test**

补断言锁定：

- 已配置 `correlation.target_column` 时写出 `建议目标字段`
- 已配置 `trend.time_column` 时写出 `建议时间字段`
- `建议优先工具` 使用稳定的现有 Tool 名称

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test diagnostics_report_excel_report_cli -- --nocapture
```

Expected:

- 因字段承接信息尚未写入摘要页而失败

**Step 3: Write minimal implementation**

在 `build_summary_dataframe()` 中：

- 从 `request.diagnostics_request` 读取目标字段和时间字段
- 生成固定承接字段值
- 保持无配置时的保守默认文案

**Step 4: Run test to verify it passes**

Run:

```powershell
cargo test --test diagnostics_report_excel_report_cli -- --nocapture
```

Expected:

- 字段承接相关断言通过

### Task 4: 回归验证与交接记录

**Files:**
- Modify: `D:\Rust\Excel_Skill\progress.md`
- Modify: `D:\Rust\Excel_Skill\findings.md`
- Modify: `D:\Rust\Excel_Skill\task_plan.md`
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Run targeted tests**

Run:

```powershell
cargo test --test diagnostics_report_excel_report_cli -- --nocapture
cargo test --test diagnostics_report_cli -- --nocapture
cargo test --test stat_diagnostics_cli -- --nocapture
```

Expected:

- 全部通过

**Step 2: Run full verification**

Run:

```powershell
cargo test
```

Expected:

- 全量通过
- 允许保留既有 `dead_code` warnings

**Step 3: Update handoff records**

补写：

- `progress.md`
- `findings.md`
- `task_plan.md`
- `.trae/CHANGELOG_TASK.md`

**Step 4: Commit**

```powershell
git add .worktrees/SheetMind-/src/ops/diagnostics_report_excel_report.rs .worktrees/SheetMind-/tests/diagnostics_report_excel_report_cli.rs .worktrees/SheetMind-/docs/plans/2026-03-28-diagnostics-report-summary-handoff-design.md .worktrees/SheetMind-/docs/plans/2026-03-28-diagnostics-report-summary-handoff-implementation.md progress.md findings.md task_plan.md .trae/CHANGELOG_TASK.md
git commit -m "feat: strengthen diagnostics summary handoff"
```
