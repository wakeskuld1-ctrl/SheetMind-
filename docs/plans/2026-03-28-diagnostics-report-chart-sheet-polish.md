# Diagnostics Report Chart Sheet Polish Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 打磨 `diagnostics_report_excel_report` 的 `图表摘要` 页，补齐分布图、异常 Top 图，并整理图表数据源区。

**Architecture:** 保持现有 `diagnostics_report -> diagnostics_report_excel_report -> workbook draft -> export_excel_workbook` 主链不变。图表增强全部继续在 `diagnostics_report_excel_report` 内部完成，复用既有 workbook chart spec 机制，不新增 Tool，不改 dispatcher 架构。

**Tech Stack:** Rust, Polars, serde, workbook draft store, rust_xlsxwriter export path, cargo test

---

### Task 1: 锁定新增图表合同

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\diagnostics_report_excel_report_cli.rs`
- Test: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\diagnostics_report_excel_report_cli.rs`

**Step 1: Write the failing test**

补红测，锁定导出 `.xlsx` 后：

- 至少出现更多 chart entry，例如 `xl/charts/chart4.xml` 与 `xl/charts/chart5.xml`
- `sharedStrings.xml` 包含 `分布区间`、`分布计数`、`异常数`

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test diagnostics_report_excel_report_cli -- --nocapture
```

Expected:

- 因新增图表不存在或数据源列不存在而失败

**Step 3: Write minimal implementation**

在 `src/ops/diagnostics_report_excel_report.rs`：

- 扩展图表源 DataFrame
- 添加 `分布图`
- 添加 `异常 Top 图`

**Step 4: Run test to verify it passes**

Run:

```powershell
cargo test --test diagnostics_report_excel_report_cli -- --nocapture
```

Expected:

- 新图表相关断言通过

### Task 2: 整理图表数据区

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\diagnostics_report_excel_report.rs`
- Test: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\diagnostics_report_excel_report_cli.rs`

**Step 1: Write the failing test**

补一条更聚焦的测试，锁定图表页数据区包含：

- `异常数`
- `分布占比`

并保持已有图表页不丢。

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test diagnostics_report_excel_report_cli -- --nocapture
```

Expected:

- 因当前数据区字段不完整而失败

**Step 3: Write minimal implementation**

整理 `build_chart_sheet_dataframe()`：

- 加入 `异常数`
- 加入 `分布区间 / 分布计数 / 分布占比`
- 保持旧列向后兼容，不影响已有图表

**Step 4: Run test to verify it passes**

Run:

```powershell
cargo test --test diagnostics_report_excel_report_cli -- --nocapture
```

Expected:

- 图表数据区相关断言通过

### Task 3: 保持降级交付稳定

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\diagnostics_report_excel_report_cli.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\diagnostics_report_excel_report.rs`

**Step 1: Write the failing test**

补红测，锁定在 `trend` 降级时：

- 图表页仍然存在
- 至少保留相关性图 / 异常图 / 分布图中可用的那些图

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test diagnostics_report_excel_report_cli -- --nocapture
```

Expected:

- 若图表构建假设 section 全量存在，则会失败

**Step 3: Write minimal implementation**

在 `build_chart_specs()` 里继续按 section 可用性独立生成图表：

- 有 distribution 就生成分布图
- 有 outlier 就生成异常相关图
- 有 trend 才生成趋势图

**Step 4: Run test to verify it passes**

Run:

```powershell
cargo test --test diagnostics_report_excel_report_cli -- --nocapture
```

Expected:

- 降级场景仍通过

### Task 4: 回归验证

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
git add .worktrees/SheetMind-/src/ops/diagnostics_report_excel_report.rs .worktrees/SheetMind-/tests/diagnostics_report_excel_report_cli.rs .worktrees/SheetMind-/docs/plans/2026-03-28-diagnostics-report-chart-sheet-polish-design.md .worktrees/SheetMind-/docs/plans/2026-03-28-diagnostics-report-chart-sheet-polish.md progress.md findings.md task_plan.md .trae/CHANGELOG_TASK.md
git commit -m "feat: polish diagnostics chart sheet"
```
