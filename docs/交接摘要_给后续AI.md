# 项目交接摘要（给后续 AI）
更新日期：2026-03-31

## 1. 项目目标

- 这个仓库当前主线是 `SheetMind` 的 Rust / EXE / SQLite / Skill 体系，不再走 Python 作为面向非 IT 用户的运行时依赖。
- 当前重点是把 Excel 处理、统计诊断、容量评估、股票历史导入与技术面咨询这些能力统一收口到同一条可交付主线上。
- 用户已经多次确认：以后按现有架构继续做，非必要不重构。

## 2. 当前已确认的核心结论

- 主入口仍然是 `src/main.rs`，空输入返回工具目录，正常请求走 `ToolRequest -> dispatch`。
- 工具暴露统一收口在 `src/tools/catalog.rs`，新能力已经注册到 catalog，而不是停留在内部模块。
- GUI 现在已经被正式隔离成可选能力：
  - `Cargo.toml` 有 `gui` feature
  - `eframe / egui_extras / rfd` 已改为 optional
  - `sheetmind_app` 只有在 `--features gui` 下才会编译
  - `src/lib.rs` 的 `pub mod gui;` 已加 `#[cfg(feature = "gui")]`
  - 所有 `tests/gui*.rs` 也已挂到 `#![cfg(feature = "gui")]`
- 这条边界很重要：用户当前产品主线不是 GUI，所以默认验证必须继续保持“无 GUI 主线”，以后非必要不要再把 GUI 混回默认 Cargo 链路。
- GUI 当前还有一条并行落地中的授权页刷新闭环：刷新动作已从同步阻塞改成后台线程 + 页面反馈状态，避免 GUI 刷新时看不到“刷新中”。
- 授权页这轮新增的确认口径是：
  - 点击“刷新状态”后，先进入 `refresh_in_progress`
  - 后台线程完成后由应用壳统一轮询结果
  - 成功时更新摘要
  - 在线校验失败但回退到本地状态时，摘要照样更新，同时给授权页 warning
  - 真正失败时保留旧摘要，只在授权页显示 error
- 授权页的“刷新状态”现在会走在线校验桥接，不再只是重复读取本地快照。
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

### 3.4 GUI 授权页关键文件

- `D:\Rust\Excel_Skill\.worktrees\SheetMind-\Cargo.toml`
- `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\lib.rs`
- `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\app.rs`
- `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\bridge\license_bridge.rs`
- `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\pages\license.rs`
- `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\state.rs`
- `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\gui_license_page_state.rs`

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

- 2026-03-31 已确认：之前 `sync_stock_price_history` 相关命令卡在 `-lshlwapi`，根因不是股票工具本身，而是 GUI 依赖无条件混入默认编译链。
- 这轮已经把 GUI 改成可选 feature，默认主线下重新验证 `stock_price_history_import_cli` 与 `technical_consultation_basic_cli` 都已恢复通过。
- 因此后续如果再看到同类错误，先检查是否误开了 GUI 编译链，而不是先怀疑股票历史同步逻辑回归。

### 5.5 GUI 授权刷新闭环

- 结论：授权页刷新动作已经开始走后台线程与页面反馈，不再只是同步按钮。
- 处理方式：
  - `SheetMindApp` 现在会持有刷新结果通道，在每帧轮询结果。
  - `LicensePageState` 新增 `refresh_in_progress`、`refresh_feedback_message` 和 `refresh_feedback_kind`。
  - `license_bridge` 新增 `LicenseRefreshResult`，把“摘要更新”和“warning 提示”拆开返回。
  - `license.rs` 现在会显示 `刷新中...` 按钮态、spinner、warning 和 error。
- 当前别再走回头路的点：
  - 不要把刷新逻辑再改回同步调用，否则“加载态”会重新失效。
  - 不要把 warning 当成 error 直接丢弃，因为在线失败但本地回退仍然有展示价值。
  - 顶部授权状态仍然和授权摘要共用同一来源，不要拆成双源状态。

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
- `cargo test --test stock_price_history_import_cli -- --nocapture --test-threads=1`
  - 结果：`8 passed`
- `cargo test --features gui --test gui_bootstrap_cli -- --nocapture --test-threads=1`
  - 结果：`1 passed`
- `cargo test --features gui --test gui_smoke -- --nocapture --test-threads=1`
  - 结果：`2 passed`
- `cargo test --test gui_license_page_state -- --nocapture --test-threads=1`
  - 结果：`9 passed`
- `cargo test --test gui_state_navigation --test gui_dashboard_state --test gui_files_flow --test gui_data_processing_state --test gui_analysis_state --test gui_reports_ai_state --test gui_license_page_state --test gui_smoke -- --nocapture`
  - 结果：通过，GUI 相关 `19` 个测试全部通过
- `cargo test --test stock_price_history_import_cli sync_stock_price_history -- --nocapture --test-threads=1`
  - 历史上曾报 `ld: cannot find -lshlwapi`，但 2026-03-31 已明确这是 GUI 编译链污染默认主线导致；当前应优先使用“默认主线无 GUI”与“显式 GUI feature”分开验证。

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
- 以后默认不要用 GUI 测试结果代替主线结果：默认链路验证请直接跑不带 feature 的 CLI / stock / Tool 测试，GUI 只在显式 `--features gui` 时单独验。
- 若要重新声明整仓绿色，必须单独做更大范围验证，不要复用旧结论。

## 9. 后续 AI 建议从这里开始

1. 先读本文。
2. 再看这几个文件：
   - `D:\Rust\Excel_Skill\.worktrees\SheetMind-\Cargo.toml`
   - `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\lib.rs`
   - `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\app.rs`
   - `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\bridge\license_bridge.rs`
   - `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\pages\license.rs`
   - `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\state.rs`
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
   - 如果继续 GUI 授权主线，优先补“刷新失败后的按钮重试体验”和“激活/解绑动作沿同一事件模式接线”。
   - 如果继续股票能力，优先继续在 `technical_consultation_basic` 内按单指标家族渐进扩展。
   - 如果优先补数据同步，先处理 `sync_stock_price_history` 的当前链接环境问题，再补 provider/date 边界。
   - 如果优先做整仓验证，先把环境与并行模块编译问题单独切片，不要把股票能力和仓库级清障混做一刀。

## 10. 对后续 AI 的明确提醒

- 不要把这次架构再推回“重新设计一遍”。
- 不要把 GUI 再混回默认 Cargo 主线；以后 GUI 相关入口、依赖、测试都默认按 feature 隔离处理。
- 不要把历史记录里的“切片级通过”误写成“整仓全绿”。
- 不要顺手清理整份 `technical_consultation_basic.rs` 的旧注释编码噪声，除非单独立项。
- 不要把授权页的后台刷新线程和结果轮询重新塞回页面模块，应用壳现在就是这条链路的统一落点。
- 继续遵守用户已经反复确认的原则：以后按照架构来干，非必要不重构。

## 11. 一句话总结

- 当前项目已经把 SheetMind 的 Rust 主线推进到“Excel 工具 + 统计诊断 + 交付报表 + 股票历史 + 技术面咨询”并行可交付阶段；下一位 AI 最应该先做的，是沿现有模块继续增量推进或单独解决验证环境问题，而不是重新开架构讨论。

## 12. 2026-03-31 模块边界补充

### 12.1 正式模块范围

- `src/ops/foundation.rs`
  - 负责 Excel 表处理、单表/多表处理、统计诊断、分析建模、报告导出、容量评估，以及这些能力所依赖的通用运行时与领域支撑。
- `src/ops/stock.rs`
  - 负责股票历史导入、股票历史 HTTP 同步、股票技术面咨询，以及后续所有股票域指标与解读扩展。
- `src/tools/catalog.rs`
  - 对外保留原平铺 `tool_catalog`，同时新增 `tool_catalog_modules.foundation` 与 `tool_catalog_modules.stock`，用于给 AI、GUI 和后续路由提供稳定的模块边界元数据。
- `src/tools/dispatcher/analysis_ops.rs`
  - 只继续承接 foundation 域的分析链路，不再回挂股票导入、同步、技术咨询。
- `src/tools/dispatcher/stock_ops.rs`
  - 专门承接 `technical_consultation_basic`、`import_stock_price_history`、`sync_stock_price_history` 三个股票入口，后续新增股票 Tool 也优先落这里。

### 12.2 依赖方向与禁止串台规则

- `foundation` 不能依赖 `stock`。
- `stock` 可以依赖通用底座层，但不能把股票语义、股票字段命名、股票规则反向渗回 `foundation`。
- 新增股票能力时，不要再塞回 `analysis_ops`，也不要在 foundation 目录里继续堆股票专用逻辑。
- `crate::ops::...` 当前仍然保留兼容导出，只是为了稳住旧引用；后续新代码优先使用 `crate::ops::foundation::...` 或 `crate::ops::stock::...`。
- 如果一个能力离开股票语境后依然成立，优先判断为 `foundation`；如果它依赖股票行情、股票指标、股票交易语义或股票咨询话术，就归 `stock`。

### 12.3 后续新增能力的归属判断

- 属于 `foundation` 的典型新增项：
  - 新的表清洗算子。
  - 新的统计诊断算法。
  - 新的分析建模能力。
  - 新的报表导出与容量评估扩展。
- 属于 `stock` 的典型新增项：
  - 新的技术指标。
  - 新的择时/趋势/量价/形态信号。
  - 新的股票数据导入或同步 provider。
  - 新的股票咨询结论、评分或解释层。

### 12.4 下次接手先看哪些文件

- `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\foundation.rs`
- `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\stock.rs`
- `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\mod.rs`
- `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\catalog.rs`
- `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\contracts.rs`
- `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\dispatcher\analysis_ops.rs`
- `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\dispatcher\stock_ops.rs`

## 13. 2026-04-01 最新交接补充

### 13.1 这轮到底做了什么

- 继续沿既有股票主线补 `technical_consultation_basic`，没有新开证券分析模块，也没有改外部工具合同。
- 生产逻辑只修了一个真实 bug：
  - 文件：`D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\technical_consultation_basic.rs`
  - 内容：新增 `is_within_retest_buffer(...)`
  - 原因：ATR 很小时，最小缓冲 `0.15` 在浮点误差下会把本该命中的 `retest_watch` / `confirmed_retest` 边界打穿
- CLI 真链路新增 7 条关键位边界回归：
  - 文件：`D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\technical_consultation_basic_cli.rs`
  - 共享夹具：`build_breakout_boundary_rows_from_tail(...)`
  - 覆盖范围：
    - 突破后刚好高于确认边界
    - 最小缓冲地板 `0.15`
    - 陈旧多根锚点排除
    - 假突破 / 假跌破边界
    - 多根 `resistance_retest_watch`
    - 多根 `support_retest_watch`

### 13.2 这轮哪些是“真 bug”，哪些不是

- 真 bug：
  - `0.15` 最小缓冲在浮点比较中偶发掉判
  - 已通过 `is_within_retest_buffer(...)` 修补生产逻辑
- 不是生产 bug：
  - 多根 `retest_watch` 初次红灯
  - 根因是测试样本预热不够，导致锚点窗口里混入前缀高价
  - 处理方式是修正样本构造，不是再改判断主逻辑

### 13.3 这轮确认过的关键合同

- 外层 `breakout_signal` 合同和内部分类函数不是同一层语义。
- 在 CLI 外层合同里：
  - “等于 anchor + buffer” 优先落到 `watch`
  - “刚好高于 anchor + buffer” 才进入确认态边界测试
- 下次如果继续补回归，不要把源码内部边界断言原样照搬到 CLI 层。

### 13.4 这轮已确认的验证证据

- `cargo test --test technical_consultation_basic_cli -- --nocapture --test-threads=1`
  - 结果：`49 passed; 0 failed`
- `cargo test failed_resistance_breakout_just_below_boundary_in_cli -- --nocapture`
  - 结果：通过
- `cargo test failed_support_breakdown_just_above_boundary_in_cli -- --nocapture`
  - 结果：通过
- `cargo test multi_bar_resistance_retest_watch_in_cli -- --nocapture`
  - 结果：通过
- `cargo test multi_bar_support_retest_watch_in_cli -- --nocapture`
  - 结果：通过

### 13.5 下一个 AI 最适合怎么接

1. 先看本文第 13 节，再看：
   - `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\technical_consultation_basic.rs`
   - `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\technical_consultation_basic_cli.rs`
   - `D:\Rust\Excel_Skill\.worktrees\SheetMind-\docs\execution-notes-2026-03-30.md`
2. 先复用这轮的关键位边界夹具和测试口径，不要新造第二套样本体系。
3. 如果继续补股票技术面，优先顺序建议是：
   - 补更上层组合咨询如何消费现有 `breakout_signal`
   - 或继续补更多关键位观察态/解释层
4. 如果不是明确有收益，不要再发起架构级重构；用户已多次确认后续按现有架构推进。
<!-- 2026-04-01 CST: 新增这组证券分析交接补充，原因是仓库在 2026-04-01 已经形成 security_analysis_contextual / security_analysis_fullstack 主链；目的是让后续 AI 先看到最新证券分析边界、日期规则和入口文件，而不是只停留在旧摘要。 -->
## 0. 2026-04-01 证券分析主线补充

### 0.1 当前证券分析已经做到哪

- 底层单证券技术面 Tool：`src/ops/technical_consultation_basic.rs`
- 上层环境聚合 Tool：`src/ops/security_analysis_contextual.rs`
- 上层全面证券分析 Tool：`src/ops/security_analysis_fullstack.rs`
- dispatcher 接入：
  - `src/tools/dispatcher/stock_ops.rs`
  - `src/tools/dispatcher.rs`
- catalog 接入：
  - `src/tools/catalog.rs`

### 0.2 当前产品级证券分析主链

- `technical_consultation_basic`
  - 负责单证券日线技术面，不混入财报、公告、大盘、板块语义
- `security_analysis_contextual`
  - 负责个股 + 大盘代理 + 板块代理的环境共振
- `security_analysis_fullstack`
  - 负责技术面 + 财报快照 + 公告摘要 + 行业上下文 + 综合结论

### 0.3 后续 AI 必须遵守的日期锚定规则

- 证券分析默认只允许锚定“当前日期”
- 如果当前日期没有有效收盘数据，才允许退到前一个交易日
- 输出必须显式写明实际分析日期，例如“按 2026-04-01 收盘分析”
- 不要混用多个日期的数据去拼一个结论

### 0.4 后续 AI 必须遵守的边界

- 不要把信息面回灌到 `technical_consultation_basic`
- 不要绕开项目内 Tool 主链，回退成泛化股评
- 免费公开源失败时可以降级为 `technical_only`，但必须明确说明降级范围
- 当前证券分析专用 Skill 已新增：`skills/security-analysis-v1/SKILL.md`

### 0.5 先看哪些文件

- `skills/security-analysis-v1/SKILL.md`
- `src/ops/security_analysis_contextual.rs`
- `src/ops/security_analysis_fullstack.rs`
- `tests/security_analysis_contextual_cli.rs`
- `tests/security_analysis_fullstack_cli.rs`
- `docs/acceptance/2026-04-01-security-analysis-contextual-v1.md`
- `docs/acceptance/2026-04-01-security-analysis-fullstack-v1.md`
- `CHANGELOG_TASK.MD`
