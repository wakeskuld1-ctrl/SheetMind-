# 2026-03-30 执行说明

## 本次目标

- 按用户确认的方案 A，将 `codex/merge-cli-mod-batches` 分支当前最新工作整理后上传到 GitHub。
- 在上传前补齐执行说明与 AI 交接材料，避免后续 AI 只能靠 `git diff` 反推上下文。
- 顺手把 GUI 授权页“刷新中 / warning / error”闭环和交接文档一起补完整，再做上传。

## 当前仓库状态

- 工作目录：`D:\Rust\Excel_Skill\.worktrees\SheetMind-`
- 当前分支：`codex/merge-cli-mod-batches`
- 远端仓库：`origin https://github.com/wakeskuld1-ctrl/SheetMind-.git`
- 最近基础提交：`4f216b5 refactor: switch dispatcher routes to modular handlers`
- 当前工作树为脏状态，`git status --short` 共统计到 `90` 条改动/新增记录，属于本分支累计最新工作，不是单一功能小补丁。

## 本次准备上传时确认到的能力范围

- 工具目录与主入口已扩展到新的 SheetMind 能力面，核心入口仍是 `src/main.rs` 与 `src/tools/catalog.rs`。
- 统计诊断链路已包含 `correlation_analysis`、`distribution_analysis`、`outlier_detection`、`trend_analysis`，并继续向 `diagnostics_report` 与 Excel 报表交付延展。
- 容量评估链路已包含 `capacity_assessment`、`capacity_assessment_from_inventory`、`capacity_assessment_excel_report`。
- 股票链路已形成 `CSV/HTTP -> SQLite -> Rust 技术咨询` 主线，核心文件包括：
  - `src/runtime/stock_history_store.rs`
  - `src/ops/import_stock_price_history.rs`
  - `src/ops/sync_stock_price_history.rs`
  - `src/ops/technical_consultation_basic.rs`
- GUI 授权页在当前最新工作里新增了“后台刷新授权状态”的异步反馈闭环，相关文件包括 `src/gui/app.rs`、`src/gui/bridge/license_bridge.rs`、`src/gui/pages/license.rs`、`src/gui/state.rs` 与 `tests/gui_license_page_state.rs`。
- 本轮进一步明确的授权页行为：
  - 按钮点击后先进入 `refresh_in_progress`
  - 刷新通过后台线程执行
  - 应用壳在每帧轮询结果并统一回写摘要
  - warning 会更新摘要并给页面提示
  - error 不覆盖旧摘要，只保留失败提示
- `technical_consultation_basic` 当前已累计接入方向、强度、量能、背离、KDJ、RSRS、MFI、CCI、Williams %R、布林带位置/带宽/中轨等语义，继续沿既定模块渐进扩展，不新开第二条技术面架构。
- 工作树还包含 GUI、License、运行时和计划文档的并行改动，因此这次上传是“当前分支最新状态同步”，不是单一股票功能提交。

## 本次会话重新验证的命令

- `cargo test --test integration_binary_only_runtime -- --nocapture --test-threads=1`
  - 结果：通过，`2 passed`
- `cargo test --test technical_consultation_basic_cli technical_consultation_basic_returns_snapshot_and_guidance_from_sqlite_history -- --nocapture --test-threads=1`
  - 结果：通过，`1 passed`
- `cargo test --test gui_license_page_state -- --nocapture --test-threads=1`
  - 结果：通过，`9 passed`
- `cargo test --test gui_state_navigation --test gui_dashboard_state --test gui_files_flow --test gui_data_processing_state --test gui_analysis_state --test gui_reports_ai_state --test gui_license_page_state --test gui_smoke -- --nocapture`
  - 结果：通过，GUI 回归全部通过
- `cargo test --test stock_price_history_import_cli sync_stock_price_history -- --nocapture --test-threads=1`
  - 结果：未通过，但不是业务断言失败；当前环境在 Windows GNU 链接阶段报错 `ld: cannot find -lshlwapi`

## 历史验证证据来源

- 根目录 `progress.md`
- 根目录 `task_plan.md`
- 根目录 `findings.md`
- `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

以上记录显示，3 月 28 日到 3 月 30 日期间，统计诊断、容量评估、股票导入/同步、技术咨询指标扩展等切片均做过对应分阶段验证。需要注意，这些是历史真实记录，不等于当前脏工作树的整仓全量重新复验。

## 当前已知风险

- 当前工作树不是干净小切片，上传会把本分支累计并行改动一起同步上去。
- `cargo test -- --nocapture --test-threads=1` 的整仓全量绿色状态，本次上传前没有重新声明；交接时只能声称“切片级验证存在，整仓级状态需单独再核”。
- `sync_stock_price_history` 本次会话复验受 Windows GNU 链接环境影响，失败点为 `-lshlwapi` 缺失，不应直接解读为 Rust 业务逻辑回归。
- 构建输出中仍存在 GUI 弃用 warning 和 dispatcher `dead_code` warning，这些目前属于已知噪声，不是本次上传阻塞项。
- GUI 授权页刷新虽然已经具备 loading / warning / error 闭环，但激活与解绑动作仍是占位，后续要沿同一页面事件模式继续接线。

## 上传后建议

- 若后续优先继续股票能力，直接从 `src/ops/technical_consultation_basic.rs` 沿当前架构增量推进。
- 若后续优先恢复更强验证，先单独处理 `sync_stock_price_history` 在当前 Windows GNU 环境下的链接问题，再决定是否做更大范围回归。
- 非必要不重构，后续继续按当前 Rust / EXE / SQLite / Skill 主线往前走。

## 2026-03-31 补充说明

- 本次会话进一步确认并落地了一个关键边界：GUI 不能继续污染默认主线。
- 已完成的边界收口包括：
  - `Cargo.toml` 新增 `gui` feature
  - `eframe / egui_extras / rfd` 改为 optional
  - `sheetmind_app` 改成 `required-features = ["gui"]`
  - `src/lib.rs` 的 `pub mod gui;` 改为 `#[cfg(feature = "gui")] pub mod gui;`
  - 所有 `tests/gui*.rs` 统一挂到 `#![cfg(feature = "gui")]`
- 本次会话同时把 `ops` / `dispatcher` 的股票能力边界继续向 `stock` 模块收口，默认方向已经更清楚：
  - 通用底座能力走 `foundation`
  - 股票导入 / 同步 / 技术咨询走 `stock`
- 这轮的最新验证应以“默认主线”和“显式 GUI feature”分开理解，不要再混成一条：
  - 默认主线：`cargo test --test stock_price_history_import_cli -- --nocapture --test-threads=1`
  - 默认主线：`cargo test --test technical_consultation_basic_cli technical_consultation_basic_returns_snapshot_and_guidance_from_sqlite_history -- --nocapture --test-threads=1`
  - GUI feature：`cargo test --features gui --test gui_bootstrap_cli -- --nocapture --test-threads=1`
  - GUI feature：`cargo test --features gui --test gui_smoke -- --nocapture --test-threads=1`
- 经过这轮边界修复后，之前 `sync_stock_price_history` 命令遇到的 `-lshlwapi`，已经可以判定为“GUI 依赖误入默认编译链”导致，而不是股票业务逻辑本身回归。

## 2026-03-31 模块隔离补充说明

- 这轮同时把 `ops` 与 `dispatcher` 正式分成两个业务域：
  - `foundation`：Excel 表处理、多表处理、统计诊断、分析建模、报告导出、容量评估、通用运行时支撑。
  - `stock`：股票历史导入、股票历史同步、股票技术面咨询，以及后续股票域指标扩展。
- 兼容策略已经落地：
  - `src/ops/mod.rs` 继续保留 `pub use` 兼容导出，避免旧的 `crate::ops::...` 调用一次性炸开。
  - 新代码默认改走 `crate::ops::foundation::...` 或 `crate::ops::stock::...`，不要继续扩大兼容层使用面。
- 对外契约已经补了分组元数据：
  - `ToolResponse::tool_catalog()` 仍保留原来的平铺 `tool_catalog`。
  - 同时新增 `tool_catalog_modules.foundation` 与 `tool_catalog_modules.stock`，方便 AI、GUI、路由层按业务域识别能力。
- 禁止串台规则本次明确固定：
  - 不要再把股票 Tool 挂回 `analysis_ops`。
  - 不要在 foundation 域继续新增股票语义逻辑。
  - `foundation` 不依赖 `stock`；`stock` 只依赖通用底座，不反向污染 foundation。
- 这轮最小相关验证口径：
  - `cargo test --test integration_tool_contract -- --nocapture`
  - `cargo test --test stock_price_history_import_cli --test technical_consultation_basic_cli -- --nocapture`
  - 如果只是继续维护模块边界文档，不需要再重跑整仓全量测试。

## 2026-04-01 关键位边界回归补充说明

- 本轮收口聚焦在 `technical_consultation_basic` 的关键位边界稳定性，不新增技术面外部合同字段，也不改动既定模块边界。
- 真实生产修补点只有一个：
  - `src/ops/technical_consultation_basic.rs` 新增 `is_within_retest_buffer(...)`
  - 目的：统一 `retest_watch`、`confirmed_retest` 与多根回踩/反抽锚点的最小缓冲比较，避免 `0.15` 最小缓冲在浮点误差下偶发掉判
- 本轮继续把上一轮源码级修补沉到 CLI 真链路：
  - `tests/technical_consultation_basic_cli.rs` 新增 `build_breakout_boundary_rows_from_tail(...)` 共享夹具
  - 新增 7 条 CLI 边界回归，覆盖：
    - 刚好高于确认边界时仍能判定有效回踩承接
    - ATR 很小时最小缓冲地板 `0.15` 在 CLI 层仍然生效
    - 多根回踩/反抽样本不会误吃到陈旧锚点
    - 假突破/假跌破在刚好越过失败边界时能稳定落到 `failed_*`
    - 多根 `resistance_retest_watch` / `support_retest_watch` 在真链路可稳定识别
- 这轮还确认了一个重要口径：
  - 外层 `breakout_signal` 合同与内部分类函数合同不同
  - 在外层 CLI 语义里，“等于 anchor + buffer” 先落 `watch`
  - 因此 CLI 确认态边界测试必须测“刚好高于边界”，不能直接照搬源码内部“等于边界也算确认”的断言

## 2026-04-01 本轮重新验证的命令

- `cargo test failed_resistance_breakout_just_below_boundary_in_cli -- --nocapture`
  - 结果：通过
- `cargo test failed_support_breakdown_just_above_boundary_in_cli -- --nocapture`
  - 结果：通过
- `cargo test multi_bar_resistance_retest_watch_in_cli -- --nocapture`
  - 结果：通过
- `cargo test multi_bar_support_retest_watch_in_cli -- --nocapture`
  - 结果：通过
- `cargo test --test technical_consultation_basic_cli -- --nocapture --test-threads=1`
  - 结果：通过，`49 passed; 0 failed`

## 2026-04-01 当前结论与上传前提醒

- 当前这轮可以确认的是：`technical_consultation_basic` 关键位主链路的源码边界与 CLI 真链路边界都已经补齐到最新状态。
- 当前不能额外声称的是：整仓全量 `cargo test -- --nocapture --test-threads=1` 已重新复验；这轮没有做该声明。
- 这轮红灯里需要区分两类原因：
  - `0.15` 最小缓冲掉判：真实浮点 bug，已经在生产逻辑修补
  - 多根 `retest_watch` 初始红灯：测试样本预热不够导致锚点窗口混入前缀高价，已通过修正样本构造解决，不属于生产逻辑 bug
- 按用户明确要求，后续仍然遵守“以后按照架构来干，非必要不重构”；关键位主线继续在 `technical_consultation_basic` 内增量补强。
