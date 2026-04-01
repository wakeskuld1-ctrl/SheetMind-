# Technical Consultation Basic Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 新增 Rust `technical_consultation_basic` Tool，让系统可以直接从 `stock_history.db` 读取股票历史行情，计算第一批基础技术指标并输出稳定、可解释的技术面咨询结果。

**Architecture:** 保持现有 `CSV -> SQLite -> Rust Tool -> dispatcher/catalog` 主链不变。本轮在 `stock_history_store` 上补读取能力，在 `src/ops` 新增技术面基础实现，通过新的 CLI 集成测试先锁定合同，再接入 catalog 和 dispatcher，不引入 Python，不新增并行股票架构。

**Tech Stack:** Rust, rusqlite, serde, cargo test

---

### Task 1: 先锁定 `technical_consultation_basic` 对外合同

**Files:**
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\technical_consultation_basic_cli.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\common\mod.rs`

**Step 1: Write the failing test**

新增 CLI 集成测试，至少覆盖：

- `tool_catalog` 中能发现 `technical_consultation_basic`
- 当 SQLite 中存在足够历史行情时，调用返回 `status = ok`
- 返回体中包含 `symbol / as_of_date / history_row_count / trend_bias / momentum_signal / volatility_state / indicator_snapshot / recommended_actions / watch_points`

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test technical_consultation_basic_cli -- --nocapture
```

Expected:

- 因 Tool 尚未注册或实现不存在而失败

**Step 3: Write minimal implementation**

先只写最小合同骨架需要的代码入口：

- 新建 `src/ops/technical_consultation_basic.rs`
- 定义请求、响应结构
- 先返回最小可编译实现

**Step 4: Run test to verify it passes partially or reaches next failure**

Run:

```powershell
cargo test --test technical_consultation_basic_cli -- --nocapture
```

Expected:

- 从“找不到 Tool”推进到“业务逻辑未完成”的下一层失败

### Task 2: 补 SQLite 历史数据读取层

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\runtime\stock_history_store.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\runtime\mod.rs`

**Step 1: Write the failing test**

在 `technical_consultation_basic_cli.rs` 里新增断言：

- 导入顺序打乱时，计算仍以日期升序为准
- 指定 `symbol` 不存在时，返回明确中文错误
- 数据条数不足时，返回“历史数据不足”而不是崩溃

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test technical_consultation_basic_cli -- --nocapture
```

Expected:

- 因读取接口不存在或错误处理未实现而失败

**Step 3: Write minimal implementation**

在 `stock_history_store.rs` 中新增：

- `StockHistoryQueryRow` 或等价读取结构
- 按 `symbol` 读取最近 N 条记录的方法
- 内部按 `trade_date` 排序并返回稳定 Vec

错误口径要求：

- 无数据时返回中文错误
- 数据不足时返回中文错误

**Step 4: Run test to verify it passes**

Run:

```powershell
cargo test --test technical_consultation_basic_cli -- --nocapture
```

Expected:

- 读取、排序、数据不足相关断言通过

### Task 3: 实现基础指标计算

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\technical_consultation_basic.rs`

**Step 1: Write the failing test**

补测试覆盖至少两个方向场景：

- 多头趋势场景：`trend_bias = bullish`
- 空头趋势场景：`trend_bias = bearish`

并补最小指标快照断言，例如：

- `indicator_snapshot.close`
- `indicator_snapshot.ema_10`
- `indicator_snapshot.sma_50`
- `indicator_snapshot.sma_200`

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test technical_consultation_basic_cli -- --nocapture
```

Expected:

- 因趋势判断和指标值尚未实现而失败

**Step 3: Write minimal implementation**

在 `technical_consultation_basic.rs` 中补齐：

- `SMA`
- `EMA`
- `MACD(12,26,9)`
- `RSI(14)`
- `BOLL(20,2)`
- `ATR(14)`

并基于最新一根记录生成 `indicator_snapshot`

**Step 4: Run test to verify it passes**

Run:

```powershell
cargo test --test technical_consultation_basic_cli -- --nocapture
```

Expected:

- 基础指标和趋势判断相关断言通过

### Task 4: 实现轻规则咨询输出

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\technical_consultation_basic.rs`

**Step 1: Write the failing test**

补测试锁定：

- `momentum_signal`
- `volatility_state`
- `recommended_actions`
- `watch_points`

至少覆盖：

- 动量偏正时的建议
- 动量偏负时的建议
- 波动偏高时的观察点

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test technical_consultation_basic_cli -- --nocapture
```

Expected:

- 因咨询字段仍为空或口径不稳定而失败

**Step 3: Write minimal implementation**

新增轻规则 helper，例如：

- `resolve_trend_bias()`
- `resolve_momentum_signal()`
- `resolve_volatility_state()`
- `build_summary()`
- `build_recommended_actions()`
- `build_watch_points()`

要求：

- 规则可解释
- 结果保守
- 中文输出稳定

**Step 4: Run test to verify it passes**

Run:

```powershell
cargo test --test technical_consultation_basic_cli -- --nocapture
```

Expected:

- 技术面咨询字段相关断言通过

### Task 5: 接入 Tool 主链

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\mod.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\catalog.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\dispatcher.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\dispatcher\analysis_ops.rs`

**Step 1: Write the failing test**

确认 `tool_catalog` 和真实 CLI 调用都在同一个测试文件中被覆盖。

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test technical_consultation_basic_cli -- --nocapture
```

Expected:

- 若尚未接入 catalog/dispatcher，则 catalog 发现性或真实调用失败

**Step 3: Write minimal implementation**

接入：

- `ops::technical_consultation_basic`
- `tool_catalog`
- `dispatcher` 的 match 分支
- `analysis_ops` 的请求解析和响应返回

**Step 4: Run test to verify it passes**

Run:

```powershell
cargo test --test technical_consultation_basic_cli -- --nocapture
```

Expected:

- 新 Tool 能通过 CLI 稳定调用

### Task 6: 回归验证与交接记录

**Files:**
- Modify: `D:\Rust\Excel_Skill\progress.md`
- Modify: `D:\Rust\Excel_Skill\findings.md`
- Modify: `D:\Rust\Excel_Skill\task_plan.md`
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Run targeted tests**

Run:

```powershell
cargo test --test technical_consultation_basic_cli -- --nocapture
cargo test --test stock_price_history_import_cli -- --nocapture
```

Expected:

- 全部通过

**Step 2: Run broader regression**

Run:

```powershell
cargo test --test diagnostics_report_excel_report_cli -- --nocapture
cargo test --test diagnostics_report_cli -- --nocapture
cargo test --test stat_diagnostics_cli -- --nocapture
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
git add .worktrees/SheetMind-/src/ops/technical_consultation_basic.rs .worktrees/SheetMind-/src/runtime/stock_history_store.rs .worktrees/SheetMind-/src/runtime/mod.rs .worktrees/SheetMind-/src/ops/mod.rs .worktrees/SheetMind-/src/tools/catalog.rs .worktrees/SheetMind-/src/tools/dispatcher.rs .worktrees/SheetMind-/src/tools/dispatcher/analysis_ops.rs .worktrees/SheetMind-/tests/technical_consultation_basic_cli.rs .worktrees/SheetMind-/docs/plans/2026-03-28-technical-consultation-basic-design.md .worktrees/SheetMind-/docs/plans/2026-03-28-technical-consultation-basic-implementation.md progress.md findings.md task_plan.md .trae/CHANGELOG_TASK.md
git commit -m "feat: add basic stock technical consultation tool"
```
