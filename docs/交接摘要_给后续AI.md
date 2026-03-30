# 项目交接摘要（给后续 AI）
更新日期：2026-03-30

## 1. 项目目标

- 这个仓库当前主线是 `SheetMind` 的 Rust / EXE / SQLite / Skill 体系，不再走 Python 作为面向非 IT 用户的运行时依赖。
- 当前重点是把 Excel 处理、统计诊断、容量评估、股票历史导入与技术面咨询这些能力统一收口到同一条可交付主线上。
- 用户已经多次确认：以后按现有架构继续做，非必要不重构。

## 2. 当前已确认的核心结论

- 主入口仍然是 `src/main.rs`，空输入返回工具目录，正常请求走 `ToolRequest -> dispatch`。
- 工具暴露统一收口在 `src/tools/catalog.rs`，新能力已经注册到 catalog，而不是停留在内部模块。
- 股票技术面主线已经成形：
  - 数据入口：`import_stock_price_history` 与 `sync_stock_price_history`
  - 存储底座：`src/runtime/stock_history_store.rs`
  - 咨询主模块：`src/ops/technical_consultation_basic.rs`
- `technical_consultation_basic` 当前已经接入并对外暴露这些字段或语义：
  - 趋势方向：`trend_bias`
  - 趋势强度：`trend_strength`
  - 量能确认：`volume_confirmation`
  - 背离：`divergence_signal`
  - 择时：`timing_signal`
  - RSRS：`rsrs_signal`
  - 资金流：`money_flow_signal`
  - CCI：`mean_reversion_signal`
  - Williams %R：`range_position_signal`
  - 布林带：`bollinger_position_signal`、`bollinger_midline_signal`、`bollinger_bandwidth_signal`
- 当前工作树不是单一股票切片，还包含 GUI、License、统计诊断、容量评估、报表交付等并行改动。

## 3. 当前主入口与关键文件

### 3.1 主入口

- `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\main.rs`
- `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\dispatcher.rs`
- `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\catalog.rs`

### 3.2 股票链路关键文件

- `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\runtime\stock_history_store.rs`
- `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\import_stock_price_history.rs`
- `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\sync_stock_price_history.rs`
- `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\technical_consultation_basic.rs`
- `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\stock_price_history_import_cli.rs`
- `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\technical_consultation_basic_cli.rs`

### 3.3 统计诊断/交付关键文件

- `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\correlation_analysis.rs`
- `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\distribution_analysis.rs`
- `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\outlier_detection.rs`
- `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\trend_analysis.rs`
- `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\diagnostics_report.rs`
- `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\diagnostics_report_excel_report.rs`
- `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\capacity_assessment.rs`
- `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\capacity_assessment_excel_report.rs`

## 4. 当前数据源与配置来源

- 股票历史持久化默认写入 workspace 下的 SQLite，走 `StockHistoryStore::workspace_default()`。
- 股票数据来源现在有两条：
  - 手工稳定路径：CSV -> `import_stock_price_history` -> SQLite
  - 联机补数路径：Tencent 优先、Sina 兜底 -> `sync_stock_price_history` -> SQLite
- 技术指标全部在 Rust 内部基于统一历史窗口计算，不依赖外部现成技术指标 API。
- License 能力已经进入 EXE 入口层，相关逻辑在 `src/license/` 与 `src/runtime/license_store.rs`。

## 5. 已处理过的问题与结论

### 5.1 架构反复重构的问题

- 结论：用户已反复确认，后续按现有架构继续做，非必要不重构。
- 处理方式：最近多轮技术面增强都落在 `technical_consultation_basic` 内，通过薄包装函数追加语义，不新开分析链。

### 5.2 技术面数据来源问题

- 结论：外部 HTTP 只拿原始 OHLCV，指标统一在 Rust 算。
- 处理方式：保留 `CSV -> SQLite` 为稳定主线，`sync_stock_price_history` 作为联机补数能力。

### 5.3 运行时禁用外部脚本依赖

- 结论：二进制运行时守门测试会检查禁用词，连注释里的相关词汇也可能触发失败。
- 处理方式：改注释措辞，不为了注释问题去动业务逻辑。

### 5.4 当前环境级阻塞

- 本次会话里 `sync_stock_price_history` 相关测试复验失败不是断言失败，而是 Windows GNU 链接阶段缺少 `shlwapi`。
- 这更像环境/工具链问题，不应直接当作 `sync_stock_price_history` 回归。

## 6. 当前最新产物

- 新增上传说明：`D:\Rust\Excel_Skill\.worktrees\SheetMind-\docs\execution-notes-2026-03-30.md`
- 新增 AI 交接：`D:\Rust\Excel_Skill\.worktrees\SheetMind-\docs\交接摘要_给后续AI.md`
- 根目录持续维护：
  - `D:\Rust\Excel_Skill\progress.md`
  - `D:\Rust\Excel_Skill\task_plan.md`
  - `D:\Rust\Excel_Skill\findings.md`
  - `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

## 7. 已跑过的验证

### 7.1 本次会话现验

- `cargo test --test integration_binary_only_runtime -- --nocapture --test-threads=1`
  - 结果：`2 passed`
- `cargo test --test technical_consultation_basic_cli technical_consultation_basic_returns_snapshot_and_guidance_from_sqlite_history -- --nocapture --test-threads=1`
  - 结果：`1 passed`
- `cargo test --test stock_price_history_import_cli sync_stock_price_history -- --nocapture --test-threads=1`
  - 结果：失败，报 `ld: cannot find -lshlwapi`

### 7.2 历史真实记录

- 见 `D:\Rust\Excel_Skill\progress.md`
- 见 `D:\Rust\Excel_Skill\task_plan.md`
- 见 `D:\Rust\Excel_Skill\findings.md`
- 见 `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

这些记录显示 2026-03-28 到 2026-03-30 期间，多轮股票指标、统计诊断、报表交付、容量评估切片都做过对应阶段验证，但请不要把这些历史记录误读成“当前脏工作树已整仓重新全绿”。

## 8. 当前仍需注意的点

- 当前 worktree 很脏，上传是同步当前累计最新状态，不是精细化挑文件的小提交。
- `technical_consultation_basic.rs` 里仍有历史注释编码噪声，后续改这个文件时继续用小补丁策略，不要顺手做大清洗。
- GUI 相关 warning 和 dispatcher `dead_code` warning 目前仍存在，但暂不作为本轮阻塞。
- 若要重新声明整仓绿色，必须单独做更大范围验证，不要复用旧结论。

## 9. 后续 AI 建议从这里开始

1. 先读本文。
2. 再看这几个文件：
   - `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\technical_consultation_basic.rs`
   - `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\sync_stock_price_history.rs`
   - `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\catalog.rs`
   - `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\technical_consultation_basic_cli.rs`
3. 再看这些动态记录：
   - `D:\Rust\Excel_Skill\progress.md`
   - `D:\Rust\Excel_Skill\task_plan.md`
   - `D:\Rust\Excel_Skill\findings.md`
   - `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`
4. 再决定下一步：
   - 如果继续股票能力，优先继续在 `technical_consultation_basic` 内按单指标家族渐进扩展。
   - 如果优先补数据同步，先处理 `sync_stock_price_history` 的当前链接环境问题，再补 provider/date 边界。
   - 如果优先做整仓验证，先把环境与并行模块编译问题单独切片，不要把股票能力和仓库级清障混做一刀。

## 10. 对后续 AI 的明确提醒

- 不要把这次架构再推回“重新设计一遍”。
- 不要把历史记录里的“切片级通过”误写成“整仓全绿”。
- 不要顺手清理整份 `technical_consultation_basic.rs` 的旧注释编码噪声，除非单独立项。
- 继续遵守用户已经反复确认的原则：以后按照架构来干，非必要不重构。

## 11. 一句话总结

- 当前项目已经把 SheetMind 的 Rust 主线推进到“Excel 工具 + 统计诊断 + 交付报表 + 股票历史 + 技术面咨询”并行可交付阶段；下一位 AI 最应该先做的，是沿现有模块继续增量推进或单独解决验证环境问题，而不是重新开架构讨论。
