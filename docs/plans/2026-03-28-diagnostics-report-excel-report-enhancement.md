# Diagnostics Report Excel Report Enhancement Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在现有 `diagnostics_report_excel_report` 之上补管理摘要增强和图表页，形成更完整的 Rust 原生诊断交付包。

**Architecture:** 继续复用 `diagnostics_report` 作为唯一业务核心，在 `diagnostics_report_excel_report` 内做 workbook 交付增强。摘要页增强只基于现有 `diagnostics_result` 轻规则生成，图表页继续复用既有 workbook chart draft/export 主线，不新增新 Tool，也不改 dispatcher 主架构。

**Tech Stack:** Rust, Polars, serde, workbook draft store, existing Excel export pipeline, cargo test

---

### Task 1: 锁定管理摘要增强合同

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\diagnostics_report_excel_report_cli.rs`
- Test: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\diagnostics_report_excel_report_cli.rs`

**Step 1: Write the failing test**

在 `diagnostics_report_excel_report_cli.rs` 新增针对导出 `.xlsx` 的断言，检查摘要页 XML 或共享字符串中包含：

- `总体风险等级`
- `可直接决策`
- `优先处理方向`

并且 `sheet_names` 长度从 4 变成 5。

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test diagnostics_report_excel_report_cli -- --nocapture
```

Expected:

- 新增测试失败
- 失败原因应是缺少管理摘要字段或缺少图表页

**Step 3: Write minimal implementation**

在 `src/ops/diagnostics_report_excel_report.rs`：

- 重构 `build_summary_dataframe()`
- 新增管理摘要字段生成逻辑
- 预留 `chart_sheet_name` / `include_chart_sheet` 的请求字段

**Step 4: Run test to verify it passes**

Run:

```powershell
cargo test --test diagnostics_report_excel_report_cli -- --nocapture
```

Expected:

- 管理摘要相关断言通过

### Task 2: 锁定图表页存在与导出行为

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\diagnostics_report_excel_report_cli.rs`
- Test: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\diagnostics_report_excel_report_cli.rs`

**Step 1: Write the failing test**

补一条红测，锁定：

- 默认输出 `图表摘要`
- `xl/workbook.xml` 包含该 sheet
- 至少存在 chart 相关 zip entry，例如 `xl/charts/chart1.xml`

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test diagnostics_report_excel_report_cli -- --nocapture
```

Expected:

- 因当前实现没有图表页或没有 chart entry 而失败

**Step 3: Write minimal implementation**

在 `src/ops/diagnostics_report_excel_report.rs`：

- 构造 chart sheet 占位 DataFrame
- 基于 `diagnostics_result` 生成 chart spec
- 走现有 workbook-with-charts 草稿构建路径

必要时参考：

- `src/ops/report_delivery.rs`
- `src/ops/capacity_assessment_excel_report.rs`

**Step 4: Run test to verify it passes**

Run:

```powershell
cargo test --test diagnostics_report_excel_report_cli -- --nocapture
```

Expected:

- 图表页与 chart entry 相关断言通过

### Task 3: 实现可降级图表页

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\diagnostics_report_excel_report_cli.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\diagnostics_report_excel_report.rs`

**Step 1: Write the failing test**

新增一条红测，锁定：

- 当 `trend` section 失败时，仍生成 `图表摘要`
- 摘要页仍有降级提醒
- workbook 仍可导出

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test diagnostics_report_excel_report_cli -- --nocapture
```

Expected:

- 当前实现若图表页依赖完整 section，则会失败

**Step 3: Write minimal implementation**

实现“按可用 section 生成部分图表”的策略：

- correlation 可用则生成相关图
- outlier 可用则生成异常图
- trend 可用则生成趋势图
- 都不可用时保留图表页说明表，不报错

**Step 4: Run test to verify it passes**

Run:

```powershell
cargo test --test diagnostics_report_excel_report_cli -- --nocapture
```

Expected:

- 降级场景仍保持通过

### Task 4: 增加显式关闭图表页的兼容入口

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\diagnostics_report_excel_report_cli.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\diagnostics_report_excel_report.rs`

**Step 1: Write the failing test**

新增一条红测，锁定：

- `include_chart_sheet = false` 时不生成 `图表摘要`
- `sheet_names` 回退为 4 页
- 其他逻辑保持不变

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test diagnostics_report_excel_report_cli -- --nocapture
```

Expected:

- 当前实现没有该字段，测试失败

**Step 3: Write minimal implementation**

在请求结构体中添加：

- `chart_sheet_name`
- `include_chart_sheet`

并在 workbook 组装分支里接入。

**Step 4: Run test to verify it passes**

Run:

```powershell
cargo test --test diagnostics_report_excel_report_cli -- --nocapture
```

Expected:

- 兼容开关通过

### Task 5: 回归验证

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
cargo test --test capacity_assessment_excel_report_cli -- --nocapture
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
git add .worktrees/SheetMind-/src/ops/diagnostics_report_excel_report.rs .worktrees/SheetMind-/tests/diagnostics_report_excel_report_cli.rs .worktrees/SheetMind-/docs/plans/2026-03-28-diagnostics-report-excel-report-enhancement-design.md .worktrees/SheetMind-/docs/plans/2026-03-28-diagnostics-report-excel-report-enhancement.md progress.md findings.md task_plan.md .trae/CHANGELOG_TASK.md
git commit -m "feat: enhance diagnostics excel delivery"
```
