## 2026-03-25
### 修改内容
- 将 dispatch 主路由切到模块分发：workbook_io、single_table、multi_table、analysis_ops，并保留 compose_workbook/report_delivery/build_chart/export_chart_image 继续走旧实现以避免能力回退。
- 修复模块化切流后的文案回归：list_sheets 与 load_table_region 参数缺失报错恢复为历史 UTF-8 中文文案。
- 保持 analysis_ops 中四个统计诊断入口（correlation/outlier/distribution/trend）可被主路由调用。
### 修改原因
- 推进 CLI 模块化分批合入，降低单文件 dispatcher 维护复杂度。
- 避免切流后出现回归（测试断言与上层 Skill 对错误文案的依赖）。
### 方案还差什么
- [ ] 旧 dispatcher.rs 内历史函数尚未清退，当前仍有大量 dead_code 警告。
- [ ] compose_workbook 模块版能力与旧实现尚未完全对齐，暂未切流。
### 潜在问题
- [ ] 后续若继续切 compose_workbook，需先补齐交付参数覆盖测试，避免报表样式回退。
- [ ] 目前仍存在历史乱码注释文件，后续应单独批次收口，避免与行为改动混在一起。
### 关闭项
- 集成回归已通过：cargo build -q、cargo test -q --test integration_cli_json、cargo test -q --test stat_diagnostics_cli、cargo test -q。

## 2026-03-29
### 修改内容
- 新增 Lemon Squeezy 授权设计文档与实施计划，收敛为“直连激活 + 本地 SQLite 缓存 + 定期校验”的单机 EXE 方案。
- 新增 `src/license` 授权模块与 `src/runtime/license_store.rs`，支持激活、状态查询、反激活、本地授权缓存和过期校验。
- 在 `src/main.rs` 增加 EXE 级授权门禁，并在 `src/tools/catalog.rs` 暴露 `license_activate / license_status / license_deactivate`。
- 新增 `tests/license_cli.rs`，覆盖目录发现、未授权拦截、激活落库、过期 validate、反激活回收五条主链测试。
### 修改原因
- 用户明确要求不要自建云收费体系，直接接 Lemon Squeezy 做授权校验，同时继续保留 Rust / exe / SQLite 主线。
- 需要先限制普通层面的“一份授权多人共用”，并避免把授权逻辑塞进现有 Excel / 分析业务层。
### 方案还差什么
- [ ] 还没有补真实售卖环境的配置说明与打包约定，后续需要把 `EXCEL_SKILL_LICENSE_ENFORCED` 等变量写进交付说明。
- [ ] 目前默认门禁只在显式开启时生效，后续正式售卖版还需要确认发布时如何固定启用。
### 潜在问题
- [ ] 如果 Lemon Squeezy 将来调整 License API 返回字段，当前强类型解析需要同步更新。
- [ ] 当前只做了普通传播层面的限制，面对主动逆向和破解并不构成强 DRM。
### 关闭项
- 授权专测已通过：`cargo test --test license_cli -- --nocapture`。
- 回归验证已通过：`cargo test --test integration_tool_contract -- --nocapture`、`cargo test --test integration_cli_json cli_without_args_returns_json_help -- --nocapture`。

## 2026-03-29
### 修改内容
- 新增桌面 GUI 设计文档 `docs/plans/2026-03-29-sheetmind-desktop-gui-design.md`，明确 `SheetMind App + SheetMind Engine` 的双层结构、七个一级页面、四条核心任务流、视觉系统和交互原则。
- 新增桌面 GUI 实施计划 `docs/plans/2026-03-29-sheetmind-desktop-gui-implementation.md`，明确以 `eframe/egui` 为首发 GUI 壳、保留现有 CLI 入口、通过桥接层复用 `ToolRequest / ToolResponse / dispatch()` 主链。
- 在实施计划中拆出 GUI 引导、授权桥接、工具桥接、工作台、文件与表、数据处理、分析建模、报告导出、AI 预留页和回归验证的逐步任务。
### 修改原因
- 用户要求先把桌面 GUI 完整设计固化成正式文档，并继续产出可直接承接开发的实施计划。
- 需要把“新增 GUI 壳，不再回头大重构 CLI/Tool 主链”固化为书面约束，方便后续 AI 和开发批次持续沿同一架构推进。
### 方案还差什么
- [ ] 还没有开始实际 GUI 代码开发，后续需按实施计划逐步新增 GUI 二进制入口、状态层、桥接层和页面骨架。
- [ ] GUI 具体中文字体、图表组件和 Windows 打包细节还未定稿，后续开发阶段需要结合真实构建效果再收口。
### 潜在问题
- [ ] `eframe/egui` 首次接入会增加编译时间与依赖体积，首轮开发需要关注 Windows 构建稳定性。
- [ ] 如果 GUI 直接同步执行耗时 Tool，请求期间可能出现界面卡顿，后续大概率需要任务队列或异步封装。
### 关闭项
- 已完成桌面 GUI 设计文档落盘：`docs/plans/2026-03-29-sheetmind-desktop-gui-design.md`。
- 已完成桌面 GUI 实施计划落盘：`docs/plans/2026-03-29-sheetmind-desktop-gui-implementation.md`。

## 2026-03-29
### 修改内容
- 新增 GUI 独立二进制入口 `src/bin/sheetmind_app.rs`，通过 `--help` 提供无图形环境下可验证的最小启动契约，并保留独立于现有 CLI 的桌面入口。
- 新增 `src/gui/app.rs`、`src/gui/state.rs`、`src/gui/theme.rs`、`src/gui/bridge/license_bridge.rs` 和 `src/gui/bridge/mod.rs`，建立桌面应用壳、一级页面状态、主题常量和授权摘要桥接层。
- 修改 `Cargo.toml`、`src/lib.rs`、`src/gui/mod.rs`，引入 `eframe / egui_extras / rfd` 依赖并暴露 GUI 模块。
- 新增 `tests/gui_bootstrap_cli.rs`、`tests/gui_state_navigation.rs`、`tests/gui_license_bridge.rs`，按 TDD 先后覆盖 GUI 入口、页面切换状态和授权摘要默认值。
### 修改原因
- 用户已批准桌面 GUI 方案并要求直接进入开发，因此需要先把“可启动 GUI 壳 + 基础导航状态 + 授权摘要桥接”作为第一批最小可运行骨架落地。
- 需要确保 GUI 首批改动不侵入现有 Rust CLI 主链，只通过新的 GUI 模块和桥接层逐步承接产品界面。
### 方案还差什么
- [ ] 还没有开始 `Task 4` 的 Tool 执行桥接层，当前 GUI 还不能直接调用 workbook / table / analysis 工具。
- [ ] 七个一级页面目前只有骨架占位，尚未接入文件打开、表识别、数据处理和分析结果真实内容。
- [ ] GUI 新增代码还没有做统一样式收口，后续需要把 `app.rs` 中的占位布局逐步拆到 `pages/` 子模块。
### 潜在问题
- [ ] `eframe/egui` 0.34 当前面板 API 在本地编译中有 deprecation warning，后续需要升级到推荐写法，避免 GUI 新代码长期积累 warning。
- [ ] 当前 `SheetMindApp::new()` 会在启动时同步读取授权摘要，若未来接入更多耗时桥接，需要避免在 UI 主线程阻塞。
- [ ] 现有工程本身仍有大量历史 `dead_code` warning，这会在 GUI 批次验证时放大输出噪音，后续需单独收口而不是混进功能改动。
### 关闭项
- GUI 入口测试已通过：`cargo test --test gui_bootstrap_cli sheetmind_app_help_or_bootstrap_runs -- --nocapture`。
- GUI 导航状态测试已通过：`cargo test --test gui_state_navigation app_state_can_switch_pages -- --nocapture`。
- GUI 授权摘要测试已通过：`cargo test --test gui_license_bridge license_summary_defaults_to_unlicensed -- --nocapture`。

## 2026-03-29
### 修改内容
- 新增 `src/gui/bridge/tool_runner.rs` 与 `src/gui/bridge/view_models.rs`，把 GUI 到现有 Tool Contract 的调用收口成 `ToolRunner`，并提供 `catalog / open_workbook / list_sheets / preview_table / license_status` 最小桥接。
- 新增 `src/gui/pages/mod.rs`、`src/gui/pages/dashboard.rs`、`src/gui/pages/files.rs`，把工作台页和“文件与表”页从 `app.rs` 中拆成独立页面模块。
- 扩展 `src/gui/state.rs`，增加 `DashboardState`、`DashboardQuickAction`、`FilesPageState`，让首页和文件页拥有独立状态，而不是继续停留在纯文本占位。
- 修改 `src/gui/app.rs`，把工作台和文件页切换到真实页面模块，并让文件页通过 `rfd` 原生文件选择器承接 Excel 文件选择入口。
- 新增 `tests/gui_tool_runner.rs`、`tests/gui_dashboard_state.rs`、`tests/gui_files_flow.rs`，按 TDD 覆盖 Tool 桥接、首页快捷动作和文件页状态。
### 修改原因
- 需要完成 GUI 第二批计划任务，把“桌面外壳”推进到“可连接 Engine、可承载首页、可承载文件导入起点”的阶段。
- 需要在不改现有 Rust Tool 主链的前提下，把 GUI 和底层能力之间的桥先正式搭起来，避免页面层后续直接耦合 `dispatch` 细节。
### 方案还差什么
- [ ] 文件页当前只接了文件选择与 Sheet 列表读取，还没有接表区域识别、表头确认和数据集建立。
- [ ] 还没有开始 `Task 7+`，数据处理页、分析建模页、导出页、AI 页和授权页仍需逐步拆到独立页面模块。
- [ ] `ToolRunner` 目前只覆盖最小入口，后续需要继续补齐单表处理、多表、分析和导出调用。
### 潜在问题
- [ ] 文件页当前对 `list_sheets` 结果的字段提取做了兼容解析，但还没有用契约测试锁死真实返回结构，后续建议补结果解析测试。
- [ ] `rfd` 文件选择器在无交互或某些远程桌面环境下可能表现不一致，后续需要在真实 Windows 打包环境中验证。
- [ ] GUI 代码目前仍有 `egui` 面板 API 的 deprecation warning，后续应单独收口，避免 GUI 模块 warning 继续扩大。
### 关闭项
- ToolRunner 目录桥接测试已通过：`cargo test --test gui_tool_runner tool_runner_can_request_catalog -- --nocapture`。
- 工作台状态测试已通过：`cargo test --test gui_dashboard_state dashboard_state_exposes_quick_actions -- --nocapture`。
- 文件页状态测试已通过：`cargo test --test gui_files_flow files_page_state_can_store_selected_sheet -- --nocapture`。

## 2026-03-29
### 修改内容
- 新增 `tests/gui_data_processing_state.rs`，按 TDD 先补“默认存在预设操作模板”和“操作历史可累计”两个失败测试，再转绿。
- 重写 `src/gui/state.rs` 的 GUI 状态定义，补充 `DataProcessingPreset`、`DataProcessingOperationGroup`、`DataProcessingState`，并把数据处理页状态接入 `AppState`。
- 新增 `src/gui/pages/data_processing.rs`，把“数据处理”页从中心区占位文案升级为三栏骨架，包含预设操作分组、预览区、参数区和操作历史区。
- 修改 `src/gui/pages/mod.rs` 与 `src/gui/app.rs`，把数据处理页正式接入 GUI 路由，并保留其他未开发页面的占位提示。
### 修改原因
- 用户已批准方案 B，希望“数据处理”页不仅有骨架，还要先有一批常用处理模板可展示，避免 GUI 页面继续停留在空壳状态。
- 需要沿既定 GUI 架构继续推进，而不是回头重构 CLI/Tool 主链，因此本次只新增状态层和页面层承接点，不侵入底层处理逻辑。
### 方案还差什么
- [ ] 数据处理页当前仍是模板与说明骨架，还没有接入真实 `ToolRunner` 执行动作。
- [ ] 预设操作点击后目前只会更新参数提示和历史记录，后续需要继续补参数表单、执行按钮和结果刷新。
- [ ] `Task 8` 的分析建模页、`Task 9` 的报告导出与 AI 页还未开始。
### 潜在问题
- [ ] `src/gui/app.rs` 当前仍有 `egui` 面板 API 的 deprecation warning，后续应单独一批收口，避免 GUI 新代码持续带 warning。
- [ ] 数据处理页的预设模板名称和分组是首版信息架构，后续若底层 Tool 命名变化，需要同步维护模板映射关系。
### 关闭项
- 数据处理页新测已通过：`cargo test --test gui_data_processing_state -- --nocapture`。
- 相关 GUI 回归已通过：`cargo test --test gui_state_navigation --test gui_dashboard_state --test gui_files_flow --test gui_data_processing_state -- --nocapture`。

## 2026-03-29
### 修改内容
- 新增 `tests/gui_analysis_state.rs`，按 TDD 先补“包含 Modeling 任务类型”和“默认存在 6 张分析任务卡片”的失败测试，再转绿。
- 扩展 `src/gui/state.rs`，新增 `AnalysisTaskKind`、`AnalysisTaskCard`、`AnalysisState`，并把分析页状态接入 `AppState`。
- 新增 `src/gui/pages/analysis.rs`，将“分析建模”页升级为任务卡片化入口，包含顶部任务卡片、参数区、结果区、图表占位区和风险解释区。
- 修改 `src/gui/pages/mod.rs` 与 `src/gui/app.rs`，把分析建模页正式接入 GUI 页面模块与应用路由。
### 修改原因
- 用户批准方案 B，希望分析建模页不是简单骨架，而是具备“任务导向入口”的产品语义，便于后续逐步接入统计分析与建模算法。
- 需要继续沿既定 GUI 壳架构推进，不回头重构 CLI/Tool 主链，因此本次只新增页面状态与展示骨架，不侵入底层分析逻辑。
### 方案还差什么
- [ ] 分析页当前还是 GUI 骨架，任务卡片点击后只会切换状态说明，尚未接入真实分析 Tool。
- [ ] 还没有补参数表单、结果表格和图表真实渲染，后续需要逐步把卡片任务映射到底层分析能力。
- [ ] `Task 9` 的报告导出页与 AI 页骨架还未开始。
### 潜在问题
- [ ] `src/gui/app.rs` 仍有 `egui` 面板 API 的 deprecation warning，后续应单独批次收口，避免和业务开发混做。
- [ ] 当前分析任务卡片名称与后续 Tool 能力名称需要保持同步，否则后续接线时容易出现概念漂移。
### 关闭项
- 分析页新测已通过：`cargo test --test gui_analysis_state -- --nocapture`。
- 相关 GUI 回归已通过：`cargo test --test gui_state_navigation --test gui_dashboard_state --test gui_files_flow --test gui_data_processing_state --test gui_analysis_state -- --nocapture`。

## 2026-03-29
### 修改内容
- 新增 `tests/gui_reports_ai_state.rs`，按 TDD 先补“报告导出默认存在模板卡片”和“AI 页默认存在上下文摘要与建议容器”的失败测试，再转绿。
- 扩展 `src/gui/state.rs`，新增 `ReportTemplateCard`、`ReportsState`、`AiSuggestionCard`、`AiState`，并把报告导出页与 AI 页状态接入 `AppState`。
- 新增 `src/gui/pages/reports.rs`，把“报告导出”页升级为模板卡片 + 输出配置 + 导出预览 + 最近导出记录的产品化骨架。
- 新增 `src/gui/pages/ai.rs`，把“AI 助手”页升级为当前上下文摘要 + 推荐动作容器 + 拟执行动作区的可扩展骨架。
- 修改 `src/gui/pages/mod.rs` 与 `src/gui/app.rs`，把报告导出页和 AI 页正式接入 GUI 页面模块与应用路由。
### 修改原因
- 用户批准方案 B，希望报告导出页和 AI 助手页都具备清晰产品语义，而不是停留在简单占位文案。
- 需要继续沿既定 GUI 壳架构推进，把七个一级页面尽快补齐，同时坚持不回头重构 CLI / Tool 主链。
### 方案还差什么
- [ ] 报告导出页当前还是骨架，模板切换只更新界面状态，尚未接入真实导出执行。
- [ ] AI 页当前只有建议容器，尚未接入本地规则引擎或大模型，也没有真实建议生成逻辑。
- [ ] `Task 10` 的授权与设置页骨架还未开始。
### 潜在问题
- [ ] `src/gui/app.rs` 仍有 `egui` 面板 API 的 deprecation warning，后续应单独批次收口，避免继续混在功能开发里。
- [ ] 报告模板名称和 AI 建议卡片结构后续需要和真实导出/AI 能力对齐，否则可能出现页面语义和底层能力漂移。
### 关闭项
- 报告导出页与 AI 页新测已通过：`cargo test --test gui_reports_ai_state -- --nocapture`。
- 相关 GUI 回归已通过：`cargo test --test gui_state_navigation --test gui_dashboard_state --test gui_files_flow --test gui_data_processing_state --test gui_analysis_state --test gui_reports_ai_state -- --nocapture`。

## 2026-03-29
### 修改内容
- 新增 `tests/gui_license_page_state.rs` 的回归断言，按 TDD 补上 “`SheetMindApp` 持有统一授权摘要，且与顶部授权状态文本保持同步” 的失败测试，再转绿。
- 重写 `src/gui/app.rs` 为 UTF-8 中文内容，保留原有应用壳职责，同时新增 `license_summary` 持有字段与 `license_summary()` 只读访问器。
- 修改 `src/gui/app.rs` 的页面路由，把 `AppPage::LicenseSettings` 从占位文案正式接到 `src/gui/pages/license.rs` 的授权中心骨架。
### 修改原因
- 用户已批准 `Task 10 / 方案B`，当前目标是把“授权与设置”页真正挂到 GUI 主线，而不是继续停留在占位提示。
- 顶部栏授权状态和授权页必须共用同一份状态来源，这样后续接入刷新授权、激活、解绑时才不会出现双源状态漂移。
- 当前 `app.rs` 存在明显中文编码噪音，这会直接影响后续 AI 交接和持续开发，因此本次一并按 UTF-8 重写，但不改变既定架构。
### 方案还差什么
- [ ] 授权页当前仍是产品化骨架，激活、刷新、解绑动作还没有接入真实授权执行链路。
- [ ] 顶部栏与授权页目前共用的是启动时加载的一份摘要，后续还需要补“刷新后同步回写”的动作闭环。
- [ ] `Task 11` 的 GUI smoke / CLI 主链联合回归还没有开始。
### 潜在问题
- [ ] `src/gui/app.rs` 仍保留 `egui` 面板 API 的 deprecation warning，本次按主线要求没有顺手重构，后续需要单独收口。
- [ ] 授权页当前展示的数据来自启动时快照，如果授权状态在运行过程中变化，页面不会自动刷新，后续要补明确的刷新动作和状态回写。
### 关闭项
- 授权页新增红绿测试已通过：`cargo test --test gui_license_page_state sheetmind_app_keeps_license_summary_and_status_text_in_sync -- --nocapture`。
- 相关 GUI 回归已通过：`cargo test --test gui_state_navigation --test gui_dashboard_state --test gui_files_flow --test gui_data_processing_state --test gui_analysis_state --test gui_reports_ai_state --test gui_license_page_state -- --nocapture`。

## 2026-03-29
### 修改内容
- 新增 `tests/gui_smoke.rs`，按 TDD 先补“GUI 二进制可启动、应用壳可初始化、导航契约可读取、ToolRunner 目录调用可用”的失败测试，再转绿。
- 重写 `src/gui/app.rs`，补充 `SheetMindApp::navigation_items()` 与 `SheetMindApp::page_title(...)` 两个只读契约，并让页面标题与左侧导航复用同一份映射。
- 更新 `docs/plans/2026-03-29-sheetmind-desktop-gui-design.md` 与 `docs/plans/2026-03-29-sheetmind-desktop-gui-implementation.md`，补充 Task 11 的实际 smoke 落地方式与回归结果。
### 修改原因
- 进入 `Task 11` 后，需要先证明桌面 GUI 壳已经成型，而且不会破坏既有 Rust CLI / Tool 主链。
- 直接去测 `egui` 内部窗口对象会让 smoke 测试过重，因此本次把 GUI 壳里真正稳定的“导航结构”和“页面标题映射”抽成可复用只读契约，既便于测试，也方便后续 AI 交接。
### 方案还差什么
- [ ] 当前 `gui_smoke` 仍是应用壳级 smoke，没有覆盖真实窗口渲染、交互点击和页面动作执行。
- [ ] 授权页刷新后同步回写应用壳状态的闭环还没补，后续应继续做“刷新授权 -> 顶部栏与授权页同步更新”的回归测试。
- [ ] `eframe/egui` 的 deprecated API warning 还没有单独收口，本轮按主线要求未顺手重构。
### 潜在问题
- [ ] `SheetMindApp::navigation_items()` 与 `SheetMindApp::page_title(...)` 后续如果新增页面，需要同步维护，否则 smoke 测试会先报错。
- [ ] 现在 smoke 只验证 GUI 壳和底层桥接共存，不代表每个页面内部都已接通真实 Tool 行为。
### 关闭项
- GUI smoke 已通过：`cargo test --test gui_smoke -- --nocapture`。
- Tool Contract 回归已通过：`cargo test --test integration_tool_contract -- --nocapture`。
- CLI JSON 帮助回归已通过：`cargo test --test integration_cli_json cli_without_args_returns_json_help -- --nocapture`。
- License CLI 回归已通过：`cargo test --test license_cli -- --nocapture`。

## 2026-03-29
### 修改内容
- 新增 `tests/gui_license_page_state.rs` 的回归断言，按 TDD 补上“授权摘要刷新后，`SheetMindApp` 内部摘要和顶部授权状态文本同步更新”的失败测试，再转绿。
- 重写 `src/gui/app.rs`，补充 `refresh_license_summary()`、`refresh_license_summary_with(...)` 和内部 `store_license_summary(...)`，把启动初始化与刷新回写统一收口。
- 保持授权页、顶部栏、导航与页面标题仍沿既定 GUI 架构工作，不引入新的运行时依赖，也不修改 CLI / Tool 主链。
### 修改原因
- 当前授权中心已经能展示启动时摘要，但还缺少“刷新后同步回写”的闭环，这会让后续按钮接线时容易出现顶部栏和页面状态不一致。
- 需要先把授权状态同步逻辑稳定在应用壳层，后续无论是按钮触发、定时刷新还是授权异常恢复，都可以复用同一条路径。
### 方案还差什么
- [ ] 当前只补了应用壳层的刷新闭环，还没有把授权页里的“刷新状态”动作按钮真正接到这个方法。
- [ ] 还没有补“真实刷新失败时，页面如何提示用户”的界面反馈模型。
- [ ] `eframe/egui` 的 deprecated API warning 仍未单独收口。
### 潜在问题
- [ ] 如果后续授权刷新不仅更新状态文本，还需要联动邮箱、校验时间、设备状态文案，目前要继续把这些细节展示层一起对齐。
- [ ] 现在测试使用的是可注入摘要来验证同步逻辑，还没有覆盖真实授权服务返回异常时的 GUI 状态表现。
### 关闭项
- 刷新同步红绿测试已通过：`cargo test --test gui_license_page_state sheetmind_app_refresh_updates_license_summary_and_status_text -- --nocapture`。
- 相关 GUI 回归已通过：`cargo test --test gui_state_navigation --test gui_dashboard_state --test gui_files_flow --test gui_data_processing_state --test gui_analysis_state --test gui_reports_ai_state --test gui_license_page_state --test gui_smoke -- --nocapture`。

## 2026-03-29
### 修改内容
- 新增 `tests/gui_license_page_state.rs` 的回归断言，按 TDD 补上“授权页触发 `RefreshStatus` 动作后，`SheetMindApp` 会同步刷新授权摘要与顶部状态文本”的失败测试，再转绿。
- 修改 `src/gui/pages/license.rs` 与 `src/gui/app.rs`，把授权页“刷新状态”按钮从纯展示动作升级为页面事件，并由应用壳统一处理 `LicensePageAction::RefreshStatus`。
- 修复 `src/ops/technical_consultation_basic.rs` 中布林带观察点构造函数的参数顺序回归，解除本轮 GUI 测试前的编译阻塞。
### 修改原因
- 当前授权同步逻辑已经具备刷新能力，但页面层还没有把“刷新状态”按钮真正接入应用壳，导致授权页与顶部栏的同步闭环还差最后一跳。
- 本轮测试先被同仓的技术咨询模块编译错误拦住，必须先做最小修复，才能继续验证授权页动作回归。
### 方案还差什么
- [ ] 当前“刷新状态”只完成了动作分发与状态回写，还没有补真实授权服务失败时的 GUI 提示文案和交互反馈。
- [ ] 激活、解绑等其余授权动作目前仍是占位按钮，后续需要继续沿同一页面事件模式逐步接线。
- [ ] `eframe/egui` 的 deprecated API warning 仍未单独收口，本轮按既定主线没有顺手重构。
### 潜在问题
- [ ] 如果真实授权刷新返回慢或失败，当前页面还没有加载中/失败态，后续需要补“处理中”和“刷新失败”的用户可见反馈。
- [ ] 本轮顺手修复的是阻塞测试的编译回归，但还没有补针对技术咨询观察点参数顺序的专项回归测试，后续可以补一条更稳。
### 关闭项
- 授权页动作红绿测试已通过：`cargo test --test gui_license_page_state sheetmind_app_handles_refresh_license_page_action -- --nocapture`。
- GUI 相关回归已通过：`cargo test --test gui_state_navigation --test gui_dashboard_state --test gui_files_flow --test gui_data_processing_state --test gui_analysis_state --test gui_reports_ai_state --test gui_license_page_state --test gui_smoke -- --nocapture`。

## 2026-03-30
### 修改内容
- 新增 `tests/gui_license_page_state.rs` 的刷新态回归断言，按 TDD 补上“默认 idle / 开始刷新进入 loading / warning 更新摘要 / error 保留旧摘要”的失败测试，再转绿。
- 修改 `src/gui/state.rs`、`src/gui/bridge/license_bridge.rs`、`src/gui/app.rs`、`src/gui/pages/license.rs`，把授权页刷新升级为“后台线程 + 应用壳轮询 + loading/warning/error 反馈”的完整闭环。
- 更新 `docs/交接摘要_给后续AI.md` 与 `docs/execution-notes-2026-03-30.md`，把这轮授权页刷新口径、验证命令、环境注意事项和下一步建议写入 AI 交接手册与上传说明。
### 修改原因
- 用户已明确要求补“刷新失败态 + 加载态”，而同步调用看不到 loading，因此必须把刷新动作切到后台执行，并补页面反馈模型。
- 用户在上传前再次提醒要写 AI 交接手册，因此需要把本轮真实改动、风险和验证证据同步进现有交接文档，避免后续 AI 重走回头路。
### 方案还差什么
- [ ] 授权页当前只完成了“刷新状态”闭环，激活授权与解绑设备仍是占位按钮，后续要沿同一页面事件模式接线。
- [ ] 真实刷新失败后的“重试”体验还比较基础，后续可以补更明确的重试提示和成功后的时间戳展示。
- [ ] 整仓级全量验证这轮没有重跑，当前仍以切片级验证和历史记录为主。
### 潜在问题
- [ ] Windows GNU 环境下部分 Cargo 测试需要在沙箱外执行才能拿到完整系统链接库，后续如果复验失败，先排查环境再判断业务回归。
- [ ] `src/gui/app.rs` 仍有 `egui` deprecated warning，当前按主线要求没有顺手重构，后续需要单独收口。
### 关闭项
- 授权页刷新态回归已通过：`cargo test --test gui_license_page_state -- --nocapture`。
- GUI 相关回归已通过：`cargo test --test gui_state_navigation --test gui_dashboard_state --test gui_files_flow --test gui_data_processing_state --test gui_analysis_state --test gui_reports_ai_state --test gui_license_page_state --test gui_smoke -- --nocapture`。

## 2026-03-31
### 修改内容
- 补充 `docs/交接摘要_给后续AI.md`，把 `foundation / stock` 的正式模块范围、依赖方向、禁止串台规则、后续归属判断和“下次先读哪些文件”写清楚。
- 补充 `docs/execution-notes-2026-03-30.md`，把这轮模块隔离的交付口径、兼容策略、对外契约变化和最小验证口径写入执行说明。
- 对齐本轮 Git 交付要求，确保“模块隔离收口”这件事本身也留下可供后续 AI 直接承接的上下文，而不只停在代码提交里。
### 修改原因
- 用户明确要求先把底座能力和股票分析能力隔离开，并在 AI 交接手册中写清楚模块范围，避免后续 AI 再把两条主线串台。
- 当前最新代码已经完成 `foundation / stock` 分域，但文档还需要补到“下一位 AI 看完就能继续干”的粒度，才能真正算交付闭环。
### 方案还差什么
- [ ] 当前只是把模块边界和交付口径写清楚，还没有继续把更多旧引用从 `crate::ops::...` 迁到 `crate::ops::foundation::...` / `crate::ops::stock::...`。
- [ ] 股票域后续新增指标时，仍需要持续遵守“不反向污染 foundation”的边界，而不是靠这次文档一次性解决。
### 潜在问题
- [ ] 如果后续 AI 只看旧调用方式而不看交接手册，仍可能继续沿兼容层写新代码，导致边界再次变模糊。
- [ ] `tool_catalog` 目前是“平铺目录 + 分组元数据”双输出，后续如果只改其中一边，可能出现契约漂移，需要继续靠集成测试锁住。
### 关闭项
- 已补齐模块范围与禁止串台规则文档。
- 已补齐本轮任务日志，满足交付前留痕要求。

## 2026-03-31
### 修改内容
- 修改 `src/ops/technical_consultation_basic.rs`，在既有关键位三层结构上补 `resistance_retest_watch / support_retest_watch`，并引入旧关键位附近的灰区 buffer，避免回踩/反抽样本被过早判成确认或失败。
- 修改 `tests/technical_consultation_basic_cli.rs`，按 TDD 新增“阻力转支撑回踩观察态”“支撑转阻力反抽观察态”两组 CSV 夹具和两条红绿测试，再推动实现转绿。
- 更新 `task_plan.md`、`progress.md`、`findings.md`，把这次 `retest_watch` 的完成状态、buffer 决策与后续待办写入记录入口。

### 修改原因
- 用户已批准继续按方案 A 推进关键位主线，第三刀完成后仍缺“正在回踩/反抽途中”的观察态，上层 AI 还不能区分“接近完成”和“已经完成”。
- 如果没有 `retest_watch`，价格刚回到旧关键位附近时就会在 `confirmed_*` 和 `failed_*` 之间来回跳，日线咨询文案不稳定。

### 方案还差什么
- [ ] 当前仍是最近两根 K 线的最小结构口径，后续需要评估是否补“多根回踩 / 多根反抽”。
- [ ] 当前关键位窗口和 retest buffer 都还没有参数化，如需支持更多标的与周期，还要继续补合同与样本。
- [ ] 整仓 `cargo test` 仍未复验通过，当前继续按 Windows 环境级资源阻塞记录。

### 潜在问题
- [ ] `max(snapshot.atr_14 * 0.25, 0.15)` 是首版灰区公式，若样本价格尺度差异过大，后续可能出现 buffer 偏宽或偏窄。
- [ ] `retest_watch` 目前只覆盖“靠近旧关键位的一根观察态”，若出现长时间横向磨位，现有语义可能仍偏粗。
- [ ] 仓内仍有与本轮无关的用户改动和未跟踪目录，提交或后续继续开发时不能误回滚。

### 关闭项
- 已通过：`cargo fmt --all`
- 已通过：`cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_resistance_retest_watch_signal -- --nocapture --test-threads=1`
- 已通过：`cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_support_retest_watch_signal -- --nocapture --test-threads=1`
- 已通过：`cargo test --test technical_consultation_basic_cli -- --nocapture --test-threads=1`
- 已通过：`cargo test --test stock_price_history_import_cli -- --nocapture --test-threads=1`
- [ ] 未执行整仓验证：`cargo test -- --nocapture --test-threads=1`，原因是当前 Windows 环境存在页文件/内存阻塞。

### 记忆点
- 这条主线继续只在 `technical_consultation_basic` 内增量推进，不串到 disclosure / Python 线。
- 每次代码任务结束后，都要同步维护 `task_plan.md`、`findings.md`、`progress.md` 和 `.trae/CHANGELOG_TASK.md`。

## 2026-03-31
### 修改内容
- 修改 `src/ops/technical_consultation_basic.rs`，新增 `breakout_signal` 顶层字段，以及 `indicator_snapshot.support_level_20 / resistance_level_20` 关键位快照字段，并把关键位语义接入 `summary / recommended_actions / watch_points`。
- 修改 `tests/technical_consultation_basic_cli.rs`，按 TDD 新增“有效突破近 20 日阻力”“有效跌破近 20 日支撑”两条专项回归，并在默认成功样本中补齐关键位字段可见性断言。
- 新增 `task_plan.md`、`progress.md`、`findings.md`，把这轮证券分析主线的任务规划、进展和关键结论沉淀成后续 AI 可直接接续的记录入口。

### 修改原因
- 用户已批准继续按证券分析主线推进“方案 1：支撑/阻力 + 突破有效性”。
- 当前 `technical_consultation_basic` 已有趋势、量能、背离、择时与波动语义，但还缺“关键位是否已被有效打破”的结构化判断。
- 用户明确要求保留任务记录入口，避免后续 AI 只能靠 `git diff` 反推上下文。

### 方案还差什么
- [ ] 当前 `breakout_signal` 仍是第一版价格结构语义，还没有细分“假突破回落”“反抽失败”“支撑转阻力”等二阶段场景。
- [ ] 当前关键位窗口固定为 `20` 日，后续如果要做不同周期或参数化，需要再补专项样本与合同设计。

### 潜在问题
- [ ] 当前 `breakout_signal` 刻意不与 `volume_confirmation` 强绑定，后续如果有人误把两者重新并成一个字段，容易造成语义重叠和回归误判。
- [ ] 整仓 `cargo test` 在当前 Windows 环境下因页文件/内存不足触发编译与链接失败，本轮无法据此声明整仓全绿。

### 关闭项
- 关键位主回归已通过：`cargo test --test technical_consultation_basic_cli -- --nocapture --test-threads=1`。
- 股票导入/同步回归已通过：`cargo test --test stock_price_history_import_cli -- --nocapture --test-threads=1`。
- 新增专项测试已通过：
- `cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_confirmed_resistance_breakout_signal -- --nocapture --test-threads=1`
- `cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_confirmed_support_breakdown_signal -- --nocapture --test-threads=1`

### 记忆点
- 用户这条主线是证券分析，不要把工作误切到 disclosure / Python 那条线。
- 记录入口优先维护 `task_plan.md`、`findings.md`、`progress.md`，并同步补 `.trae/CHANGELOG_TASK.md`。

## 2026-03-31
### 修改内容
- 修改 `src/ops/technical_consultation_basic.rs`，把 `breakout_signal` 从第一阶段确认扩展到二阶段确认，新增 `failed_resistance_breakout / failed_support_breakdown`，并同步更新 `summary / recommended_actions / watch_points` 的中文输出。
- 修改 `tests/technical_consultation_basic_cli.rs`，按 TDD 新增“假突破回落”“假跌破拉回”两条失败测试与对应 CSV 夹具样本，再推动实现转绿。
- 更新 `task_plan.md`、`progress.md`、`findings.md`，把这次关键位二阶段确认的完成项、验证证据与后续建议写入记录入口。

### 修改原因
- 用户已批准继续沿证券分析主线推进方案 1，当前第一版只能判断“是否已突破/跌破关键位”，还无法识别失效突破。
- 日线咨询如果不能区分“有效突破”和“单根越位后迅速收回”，上层 AI 很容易把假动作继续误读成趋势确认。

### 方案还差什么
- [ ] 当前关键位语义仍未覆盖“阻力转支撑回踩确认 / 支撑转阻力反抽失败”。
- [ ] 当前关键位窗口仍固定为 `20` 日，后续如需参数化，还要补合同和样本设计。
- [ ] 本轮只完成 `technical_consultation_basic` 切片回归，尚未再次挑战整仓 `cargo test` 的环境级阻塞。

### 潜在问题
- [ ] 假突破/假跌破目前基于日线“前一根越位、当前一根收回”的最小口径，若后续有人改成只看当前快照，会重新丢失二阶段语义。
- [ ] 关键位仍用收盘价而不是影线高低点，若后续切换口径，现有测试样本和文案都要一起调整。
- [ ] `cargo test --test technical_consultation_basic_cli` 通过时仍伴随仓内既有 dead_code warning，这不是本轮回归失败，但后续做整洁度治理时要单独处理。

### 关闭项
- 新增红绿测试已通过：`cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_failed_resistance_breakout_signal -- --nocapture --test-threads=1`。
- 新增红绿测试已通过：`cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_failed_support_breakdown_signal -- --nocapture --test-threads=1`。
- 关键位整组回归已通过：`cargo test --test technical_consultation_basic_cli -- --nocapture --test-threads=1`。
- 代码格式化已执行：`cargo fmt --all`。

### 记忆点
- 这条主线继续在 `technical_consultation_basic` 内增量推进，不新开证券分析模块。
- 用户要求改代码前先给多个方案并等批准；本轮是沿已批准的“方案 1”继续切第二刀。

## 2026-03-31
### 修改内容
- 修改 `src/ops/technical_consultation_basic.rs`，在原有突破/失效结构上继续补第三刀，新增 `confirmed_resistance_retest_hold / confirmed_support_retest_reject`，并同步收紧 `failed_*` 的判定口径为“重新回到旧关键位内侧”。
- 修改 `tests/technical_consultation_basic_cli.rs`，按 TDD 新增“阻力转支撑回踩确认”“支撑转阻力反抽受压”两组 CSV 夹具和两条红绿测试。
- 更新 `task_plan.md`、`progress.md`、`findings.md`，把这次关键位三阶段确认的合同、验证结果和后续待办写入记录入口。

### 修改原因
- 用户已批准按方案 A 继续推进关键位主线，目标是把突破后的第一次回踩承接、跌破后的第一次反抽受压也结构化收口。
- 当前如果没有这层语义，上层 AI 只能区分“突破成功/失败”，却无法区分“已进入回踩确认阶段”和“已进入反抽受压阶段”。

### 方案还差什么
- [ ] 当前还没有 `retest_watch` 观察态，无法区分“正在回踩途中”和“回踩已完成承接”。
- [ ] 当前仍固定 `20` 日关键位窗口，若后续要支持不同周期，还要补参数化合同和样本。
- [ ] 整仓 `cargo test` 这轮仍未复验环境级阻塞。

### 潜在问题
- [ ] 回踩确认当前基于最近两根 K 线与前序关键位关系的最小口径，若后续样本进入更长时间的多根回踩结构，可能需要再加 `watch` 中间态。
- [ ] `failed_*` 这轮已收紧到“重新回到旧关键位内侧”，如果后续有人改回看当前窗口极值，第三刀会再次被第二刀吞掉。
- [ ] 仓内既有 `dead_code` warning 仍存在，当前不影响测试结论，但后续如果做整洁度治理需要单独处理。

### 关闭项
- 新增红绿测试已通过：`cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_confirmed_resistance_retest_hold_signal -- --nocapture --test-threads=1`。
- 新增红绿测试已通过：`cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_confirmed_support_retest_reject_signal -- --nocapture --test-threads=1`。
- 技术咨询整组回归已通过：`cargo test --test technical_consultation_basic_cli -- --nocapture --test-threads=1`。
- 股票导入/同步回归已通过：`cargo test --test stock_price_history_import_cli -- --nocapture --test-threads=1`。
- 代码格式化已执行：`cargo fmt --all`。

### 记忆点
- 关键位主线现在已经有三层：突破/跌破确认、失效突破/失效跌破、回踩确认/反抽受压。
- 下一刀若继续方案 1，优先做 `retest_watch`，不要急着新开别的证券分析模块。

## 2026-03-31
### 修改内容
- 修改 `src/ops/technical_consultation_basic.rs`，新增 `MULTI_BAR_RETEST_LOOKBACK_BARS`、`find_multi_bar_resistance_retest_anchor`、`find_multi_bar_support_retest_anchor`，把多根回踩/多根反抽结构接入既有 `breakout_signal` 判断链路。
- 修改 `tests/technical_consultation_basic_cli.rs`，按 TDD 新增“多根回踩再站稳”“多根反抽再受压”两组 CSV 夹具与两条红绿测试，先看见 `range_bound` 红灯，再推动实现转绿。
- 更新 `task_plan.md`、`progress.md`、`findings.md`，补齐这次多根结构扩展的完成项、验证证据、风险与下一步建议。

### 修改原因
- 用户已批准继续按方案 A 推进关键位主线，当前实现只能稳定识别前一根突破/跌破后的确认，仍会漏掉 2~4 根磨位后的真实回踩/反抽样本。
- 如果不补这层结构，日线咨询会把更贴近实盘节奏的样本重新打回 `range_bound`，上层 AI 也无法稳定区分“旧关键位已完成承接/受压”。

### 方案还差什么
- [ ] 当前多根结构主要补在确认链路，后续可继续评估是否单独加“多根回踩观察态 / 多根反抽观察态”样本。
- [ ] `MULTI_BAR_RETEST_LOOKBACK_BARS = 4` 仍是固定窗口，若后续覆盖更多节奏样本，可能还要参数化。
- [ ] 整仓 `cargo test -- --nocapture --test-threads=1` 本轮未再执行，仍沿用既有环境阻塞记录。

### 潜在问题
- [ ] 若样本磨位长度超过 `4` 根，当前锚点扫描仍可能遗漏真实的多根回踩/反抽结构。
- [ ] 当前多根结构继续依赖 ATR 灰区和收盘价关键位，若后续切换到影线高低点口径，现有样本与文案需要同步调整。
- [ ] 仓内仍存在与本轮无关的用户改动和未跟踪目录，提交时必须继续精确暂存，不能误带。

### 关闭项
- 已通过：`cargo fmt --all`
- 已通过：`cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_multi_bar_resistance_retest_hold_signal -- --nocapture --test-threads=1`
- 已通过：`cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_multi_bar_support_retest_reject_signal -- --nocapture --test-threads=1`
- 已通过：`cargo test --test technical_consultation_basic_cli -- --nocapture --test-threads=1`
- 已通过：`cargo test --test stock_price_history_import_cli -- --nocapture --test-threads=1`
- [ ] 未执行整仓验证：`cargo test -- --nocapture --test-threads=1`，原因是当前仍沿用 Windows 环境级页文件/内存阻塞记录。

### 记忆点
- 这条主线继续只在 `technical_consultation_basic` 内增量推进，不新开证券分析模块。
- 用户要求改代码前先给多个方案并等待批准；这轮是沿已批准的方案 A 继续补第四刀。

## 2026-04-01
### 修改内容
- 修改 `src/ops/technical_consultation_basic.rs`，新增 `is_within_retest_buffer(...)` 浮点边界辅助，并把 `retest_watch` 与多根回踩/反抽锚点中的缓冲比较统一切到该辅助函数。
- 修改 `src/ops/technical_consultation_basic.rs` 内部测试，新增 `breakout_test_snapshot(...)`、`history_rows_from_closes(...)` 两个最小夹具，以及 6 条 `breakout_signal` / `confirmed_retest` / `retest_watch` 边界单测。
- 本轮没有继续扩展外部合同字段，只对既有关键位能力做边界加固与最小修补。

### 修改原因
- 用户已批准按“方案 1”继续推进，目标是先把 `technical_consultation_basic` 的关键位能力补到更稳，而不是再开新架构。
- 这轮排查里确认有一个真实逻辑问题：当 ATR 很小、缓冲退化到最小值 `0.15` 时，`abs() <= 0.15` 会被浮点误差打穿，导致本该命中的 `retest_watch` / `confirmed_retest` 边界样本偶发落空。

### 方案还差什么
- [ ] 当前补的是 `breakout_signal` 这一层的源码级规则边界，CLI 端和更长链路的组合样本仍可继续补一轮。
- [ ] 关键位窗口与 `MULTI_BAR_RETEST_LOOKBACK_BARS` 目前仍是固定值，后续若要参数化，还要一起补合同与回归样本。
- [ ] 这轮只确认了关键位切片回归，没有做整仓 `cargo test -- --nocapture --test-threads=1`。

### 潜在问题
- [ ] 当前浮点容差写成固定 `1e-9`，对现有价格尺度够用；若后续接入更大数量级或更细精度资产，可能需要改成相对容差。
- [ ] `history_rows_from_closes(...)` 刻意把高低点噪音压平，只适合当前“按收盘价判定关键位”的规则；若后续切到影线口径，相关测试要同步重写。
- [ ] 仓内既有 `dead_code` warning 仍存在，本轮测试通过但不代表仓库已经完成整洁度治理。

### 关闭项
- 已通过：`cargo test breakout_signal_ -- --nocapture`
- 已通过：`cargo test confirmed_retest_hold_accepts_exact_anchor_plus_buffer_boundary -- --nocapture`
- 已通过：`cargo test retest_watch_uses_minimum_buffer_floor_when_atr_is_too_small -- --nocapture`
- 已确认：`is_within_retest_buffer`、`breakout_test_snapshot`、`history_rows_from_closes` 及 6 条新增边界单测在源码中均只存在一份。

### 记忆点
- 用户明确要求“以后按照架构来干，非必要不重构”，所以这条主线继续采用增量补强，不为了边界修补再重开模块。
- 这轮发现的“0.15 边界掉判”属于真实浮点 bug；而“超过 4 根失效”那次最初红灯属于测试样本误造，不是生产逻辑本身出错。

## 2026-04-01
### 修改内容
- 修改 `tests/technical_consultation_basic_cli.rs`，新增 `build_breakout_boundary_rows_from_tail(...)` 共享夹具生成器，把“长历史底座 + 精确尾部 close 序列”接入 CLI 级关键位样本生成。
- 修改 `tests/technical_consultation_basic_cli.rs`，新增 3 条关键位边界真链路回归：
  - `technical_consultation_basic_accepts_just_above_buffer_boundary_in_cli`
  - `technical_consultation_basic_keeps_minimum_buffer_floor_in_cli_retest_watch`
  - `technical_consultation_basic_ignores_stale_multi_bar_anchor_in_cli`
- 本轮没有新增或修改生产逻辑，只把上一轮源码级边界继续沉到底层 `CSV -> SQLite -> CLI` 主链路。

### 修改原因
- 用户已批准继续走方案 1，当前最优先的是把关键位能力从内部规则测试进一步补到外层回归，而不是再开新模块或做架构调整。
- 上一轮已确认真实浮点边界 bug 并在源码修好，这一轮需要把它钉进 CLI 真链路，避免未来只过内部单测却放掉导入后的行为。

### 方案还差什么
- [ ] 当前新增的是 3 条关键位边界 CLI 回归，后续还可以继续补“假突破/假跌破”与“多根观察态”的更细边界外层样本。
- [ ] 这轮仍未处理仓内既有 `dead_code` warning，后续若做整洁度治理要单独开一刀。
- [ ] 新闻、技术面更上层组合咨询还没继续往前接，这轮只是在补关键位底座。

### 潜在问题
- [ ] `build_breakout_boundary_rows_from_tail(...)` 目前按收盘价关键位口径造样本；若后续关键位切到影线口径，这批 CLI 样本需要同步改造。
- [ ] 这批样本是“低波动 + 精确尾部 close 序列”的最小合同，能很好锁边界，但不等于覆盖了所有真实市场噪音。
- [ ] 仓内仍有上一轮未提交的 [src/ops/technical_consultation_basic.rs](/D:/Rust/Excel_Skill/.worktrees/SheetMind-/src/ops/technical_consultation_basic.rs) 修改，本轮汇报时要区分“上轮生产修补”和“本轮仅补 CLI 测试”。

### 关闭项
- 已按 TDD 先看到红灯：`cargo test stale_multi_bar_anchor_in_cli -- --nocapture`，初始失败原因是 3 个新样本构造函数未定义。
- 已通过：`cargo test above_buffer_boundary_in_cli -- --nocapture`
- 已通过：`cargo test minimum_buffer_floor_in_cli_retest_watch -- --nocapture`
- 已通过：`cargo test stale_multi_bar_anchor_in_cli -- --nocapture`
- 已通过整组验证：`cargo test --test technical_consultation_basic_cli -- --nocapture --test-threads=1`

### 记忆点
- 这轮继续遵守“非必要不重构”，只补回归，不动关键位主逻辑。
- 外层 `breakout_signal` 的合同里，“等于 anchor + buffer” 会先落到 `watch`；因此 CLI 层确认态边界应测“刚好高于边界”，不要误把源码级 `classify_confirmed_retest_signal` 的内层断言直接照搬到外层合同。

## 2026-04-01
### 修改内容
- 修改 `tests/technical_consultation_basic_cli.rs`，继续补齐 4 条 CLI 关键位边界真链路回归：
  - `technical_consultation_basic_marks_failed_resistance_breakout_just_below_boundary_in_cli`
  - `technical_consultation_basic_marks_failed_support_breakdown_just_above_boundary_in_cli`
  - `technical_consultation_basic_marks_multi_bar_resistance_retest_watch_in_cli`
  - `technical_consultation_basic_marks_multi_bar_support_retest_watch_in_cli`
- 复用并扩展已有共享夹具 `build_breakout_boundary_rows_from_tail(...)` 及对应样本生成函数，把“失败边界”和“多根观察态”一并沉到底层 `CSV -> SQLite -> CLI` 主链路。
- 更新 `docs/execution-notes-2026-03-30.md`、`docs/交接摘要_给后续AI.md`，补齐本轮上传前的执行说明与 AI 接手入口。

### 修改原因
- 用户已明确要求“做完以后推送到 GitHub，并补充一份 AI 交接手册”，所以这轮除了补回归，还要把交付证据和接手说明同步收口。
- 上一轮只补了 3 条 CLI 边界回归，当前还缺“假突破/假跌破边界”和“多根观察态”两组外层样本；如果不补，上层 AI 仍然缺少对关键位失败态和观察态的稳定保护。

### 方案还差什么
- [ ] 这轮完成后，关键位底座的 CLI 边界已比较完整；下一步更适合接上层组合咨询，而不是继续横向重构。
- [ ] 整仓 `cargo test -- --nocapture --test-threads=1` 仍未在本轮重新执行，整仓级健康度仍需单独复验。
- [ ] GitHub 推送完成后，若要继续扩展技术面，优先考虑如何消费现有 `breakout_signal` 语义，不要重复造关键位引擎。

### 潜在问题
- [ ] 多根结构仍受 `MULTI_BAR_RETEST_LOOKBACK_BARS = 4` 约束，若后续样本超过当前窗口长度，还要继续补更长节奏回归。
- [ ] 这批边界样本继续按收盘价关键位口径构造；若将来切换到影线口径，CLI 夹具和断言都要同步调整。
- [ ] 仓内既有 `dead_code` warning 依旧存在，本轮测试通过只说明关键位切片通过，不等于整仓整洁度问题已清空。

### 关闭项
- 已通过：`cargo test failed_resistance_breakout_just_below_boundary_in_cli -- --nocapture`
- 已通过：`cargo test failed_support_breakdown_just_above_boundary_in_cli -- --nocapture`
- 已通过：`cargo test multi_bar_resistance_retest_watch_in_cli -- --nocapture`
- 已通过：`cargo test multi_bar_support_retest_watch_in_cli -- --nocapture`
- 已通过：`cargo test --test technical_consultation_basic_cli -- --nocapture --test-threads=1`
- 已补齐上传前文档：`docs/execution-notes-2026-03-30.md`、`docs/交接摘要_给后续AI.md`

### 记忆点
- 这轮真实生产修补只有 `0.15` 最小缓冲浮点边界；本轮新增的 4 条回归只是把失败态和观察态继续补到 CLI 真链路。
- 后续继续做股票能力时，优先消费现有关键位信号，不要再为了“看起来一次到位”而重开架构；用户已经明确要求以后按当前架构推进，非必要不重构。
## 2026-04-08
### 修改内容
- 将 `security_committee_vote` 从旧 5 席实现升级为 `seven_seat_committee_v3`，固定为 6 名审议委员加 1 名风控委员。
- 在 `src/tools/contracts.rs`、`src/tools/dispatcher.rs`、`src/tools/dispatcher/stock_ops.rs` 补齐内部 `security_committee_member_agent` 子进程分发所需协议与入口。
- 更新 `tests/security_committee_vote_cli.rs` 与 `tests/security_analysis_resonance_cli.rs`，让正式测试合同同时覆盖七席结构与独立执行证明。
- 补充本轮执行说明与证券分析交接摘要，明确“为什么现在能证明独立执行”以及后续接手入口。
### 修改原因
- 用户明确要求沿方案 B 继续推进，不恢复旧投决入口，而是在现有 briefing -> vote 主链上落七席委员会与独立执行证明。
- 需要让投决会的正式结果能够证明每个委员是独立执行的，而不是父进程直接拼接结论。
### 方案还差什么
- [ ] 七席委员会的更细人格倾向、议事规则和后续更强审计材料仍可继续增强，但不属于本轮 P0 实装阻塞项。
- [ ] 当前没有顺手治理仓库已有的 `dead_code` warning，后续如需收口应单开批次。
### 潜在问题
- [ ] `process_id / execution_instance_id` 属于动态审计字段，任何重新执行都会变化；后续测试不要再拿它们做跨调用全等。
- [ ] 如果未来修改内部 child-process 路径解析，必须保住 integration test 场景下“从测试 harness 回推正式二进制”的兼容逻辑。
### 关闭项
- 红绿验证已通过：`cargo test --test security_committee_vote_cli security_committee_vote_exposes_seven_seat_independent_execution -- --nocapture`
- 回归验证已通过：`cargo test --test security_committee_vote_cli -- --nocapture`
- 联动验证已通过：`cargo test --test security_analysis_resonance_cli security_committee_vote_consumes_briefing_payload_with_historical_digest -- --nocapture`
- 联动验证已通过：`cargo test --test security_analysis_resonance_cli security_decision_briefing_includes_default_committee_recommendations_for_all_modes -- --nocapture`

## 2026-04-08
### 修改内容
- 新增 `docs/plans/2026-04-08-security-post-trade-review-position-management-design.md`，正式收口证券主链“单标的 + 多次调仓复盘”的对象模型、主链挂接方式与边界。
- 新增 `docs/plans/2026-04-08-security-post-trade-review-position-management.md`，把后续实现拆成仓位计划记录、调仓事件、投后复盘三个正式 Tool 的开发计划。
- 明确本轮聚焦证券主链，不继续扩展 foundation，优先把“投前决策 -> 投中执行 -> 投后纠偏”闭环写清楚。

### 修改原因
- 用户已明确批准按方案 B 推进“投后复盘 + 仓位管理”，并要求直接覆盖到“单标的 + 多次调仓复盘”。
- 在动代码前，必须先把对象边界、数据流和实施顺序文档化，避免后续再次出现平行链路或范围漂移。

### 方案还差什么
- [ ] 设计与计划已落盘，但正式 Tool 合同、dispatcher 接线和测试红绿还没有开始实现。
- [ ] `decision_ref / approval_ref / evidence_version` 在新对象里的具体存取方式，还需要在实现阶段结合现有 runtime 结构收口。
- [ ] 复盘的阶段性结果字段与实际持久化格式仍需在 P0-3 开发时最终钉死。

### 潜在问题
- [ ] 如果实现时把仓位计划记录、调仓事件和复盘对象拆成彼此独立但不共享引用，容易重新走回“多套事实源”的老问题。
- [ ] 当前整仓测试基线仍有 1 条既有失败，后续开发验收时要区分“新能力回归”与“仓库既有失败”。
- [ ] Windows 下长跑 `cargo test` 仍可能受磁盘空间与残留进程影响，后续执行计划时要继续用 `D:` 盘目标目录并留意 `os error 5`。

### 关闭项
- 已完成：证券主链“投后复盘 + 仓位管理”设计收口并落盘。
- 已完成：对应实现计划拆解并保存到 `docs/plans/2026-04-08-security-post-trade-review-position-management.md`。

## 2026-04-08
### 修改内容
- 在 `tests/integration_tool_contract.rs` 新增 `security_position_plan_record_contract_exposes_required_fields` 红测，锁定仓位计划记录正式合同至少要暴露 `position_plan_ref / decision_ref / approval_ref / evidence_version / symbol / analysis_date / position_action / starter_position_pct / max_position_pct`。
- 在 `src/tools/contracts.rs` 新增 `SecurityPositionPlanRecordRequest` 与 `SecurityPositionPlanRecordResult`，并补最小 `from_position_plan(...)` 构造辅助函数。
- 在 `src/ops/security_decision_briefing.rs` 为 `PositionPlan` 新增 `record_projection()`，统一提取 record 合同所需的动作和仓位边界字段。

### 修改原因
- 用户已批准进入投后复盘与仓位管理开发，Task 1 的目标是先把“仓位计划记录”这层正式合同锁定下来，再进入 Tool 落地。
- 需要先证明仓位计划对象能从 briefing 的 `position_plan` 稳定投影出来，避免后续 record Tool 与测试夹具各自手工拼字段。

### 方案还差什么
- [ ] Task 2 的正式 `security_position_plan_record` Tool 还没有开始实现，当前只有合同层。
- [ ] `position_plan_ref` 的正式生成规则与 runtime 落盘位置还未确定，要在 Task 2 收口。
- [ ] `decision_ref / approval_ref` 当前只在合同里固定，实际来源校验仍待后续 Tool 实装。

### 潜在问题
- [ ] 当前 `from_position_plan(...)` 只是最小构造辅助函数，后续如果 record 对象新增更多派生字段，必须继续保持由单点辅助函数统一提取，避免字段漂移。
- [ ] 仓库整体现有的既有失败测试仍未处理，后续验收要区分“Task 1 已绿”和“整仓全绿”。
- [ ] Windows 下长跑测试仍建议继续使用 `D:\\cargo-targets\\excel-skill-cli-mod-review`，否则可能再次被系统盘空间影响。

### 关闭项
- 红测已按预期失败：`cargo test --test integration_tool_contract security_position_plan_record_contract -- --nocapture`
- 转绿验证已通过：`cargo test --test integration_tool_contract security_position_plan_record_contract -- --nocapture`

## 2026-04-08
### 修改内容
- 在 `tests/security_analysis_resonance_cli.rs` 新增 `security_position_plan_record_persists_briefing_plan` 红测，要求正式 CLI Tool 能消费 briefing 派生的 `position_plan` 并返回结构化 `position_plan_ref`。
- 新增 `src/ops/security_position_plan_record.rs`，补最小 `security_position_plan_record` Tool 与请求边界校验。
- 更新 `src/ops/stock.rs`、`src/ops/mod.rs`，把 `security_position_plan_record` 纳入证券主链模块导出。
- 更新 `src/tools/catalog.rs`、`src/tools/dispatcher.rs`、`src/tools/dispatcher/stock_ops.rs`，完成 Tool 注册与 stock dispatcher 接线。

### 修改原因
- 用户已批准进入 Task 2，这一步的目标是把 Task 1 的合同类型推进成正式 Tool 主链入口，而不是继续停留在类型定义层。
- 需要先打通“briefing 派生仓位计划 -> position_plan_ref”的最小闭环，给后续调仓事件和投后复盘提供正式计划锚点。

### 方案还差什么
- [ ] 当前 `security_position_plan_record` 还是最小回声式 Tool，尚未引入真实 runtime 持久化。
- [ ] `position_plan_ref` 当前使用确定性规则生成，后续若引入多计划版本或重复日期计划，需要再扩展版本/去重策略。
- [ ] `decision_ref / approval_ref` 当前只做非空校验，尚未与真实审批记录交叉验证。

### 潜在问题
- [ ] 由于当前还没落真实存储，后续 `position_adjustment_event` 若需要通过 `position_plan_ref` 回查完整计划对象，还需要在 Task 4 前补持久化或对象回读机制。
- [ ] Tool 已加入 catalog 与 dispatcher，但当前没有单独补 catalog 可见性回归，若后续目录测试变动，需要在集成回归里顺带覆盖。
- [ ] 仓库整体仍存在既有测试失败与较多 `dead_code` warning，本轮转绿只说明 Task 2 目标达成，不代表整仓基线已收口。

### 关闭项
- 红测已按预期失败：`cargo test --test security_analysis_resonance_cli security_position_plan_record_persists_briefing_plan -- --nocapture`
- 转绿验证已通过：`cargo test --test security_analysis_resonance_cli security_position_plan_record_persists_briefing_plan -- --nocapture`

## 2026-04-08
### 修改内容
- 在 `tests/integration_tool_contract.rs` 新增 `security_record_position_adjustment_contract_exposes_required_fields` 红测，锁定调仓事件正式合同至少要暴露 `adjustment_event_ref / position_plan_ref / event_type / event_date / before_position_pct / after_position_pct / trigger_reason / plan_alignment`，并补充枚举值序列化断言。
- 在 `src/tools/contracts.rs` 新增 `PositionAdjustmentEventType`、`PositionPlanAlignment`、`SecurityRecordPositionAdjustmentRequest`、`SecurityRecordPositionAdjustmentResult`。
- 在 `src/tools/contracts.rs` 为 `SecurityRecordPositionAdjustmentResult` 新增最小 `from_request(...)` 构造辅助函数，统一后续 Task 4 的事件记录投影入口。

### 修改原因
- 用户已明确继续推进 `Task 3`，当前目标是先把“调仓事件”这层正式合同钉死，再进入正式 Tool 落地。
- 需要先统一 `build / add / reduce / exit / risk_update` 与 `on_plan / justified_deviation / off_plan` 的口径，避免后续调仓记录、审批简报和复盘对象各自使用不同字符串。

### 方案还差什么
- [ ] `security_record_position_adjustment` 正式 Tool 还没有开始实现，当前完成的是合同层，不包含 dispatcher 接线。
- [ ] `adjustment_event_ref` 的正式生成规则和多次事件的 runtime 存取方式，还需要在 `Task 4` 收口。
- [ ] 当前合同已预留 `decision_ref / approval_ref / evidence_version`，但真实来源校验仍待正式 Tool 实装时补齐。

### 潜在问题
- [ ] 当前 `before_position_pct / after_position_pct` 还是裸 `f64`，后续若要防止负值、超 100% 或精度漂移，需要在 Tool 实装时增加边界校验与归一化测试。
- [ ] 当前只锁了合同字段与枚举值，尚未验证同一 `position_plan_ref` 下多次调仓事件的顺序性与引用一致性，这会留到 `Task 4` 的 CLI 回归里处理。
- [ ] 仓库整体既有失败仍未处理，当前转绿只代表 `Task 3` 的定向合同测试通过，不代表整仓全绿。

### 关闭项
- 红测已按预期失败：`cargo test --test integration_tool_contract security_record_position_adjustment_contract -- --nocapture`
- 转绿验证已通过：`cargo test --test integration_tool_contract security_record_position_adjustment_contract -- --nocapture`
- 窄回归验证已通过：`cargo test --test integration_tool_contract security_position_plan_record_contract -- --nocapture`

## 2026-04-08
### 修改内容
- 重新执行整仓基线验证：`cargo test -- --nocapture`，确认当前仓库仍非整仓全绿，明确失败仍包含 `tests/integration_tool_contract.rs` 里的 `technical_consultation_basic_contract_exposes_bullish_continuation_conclusion`，失败点是 `analysis_date` 取到 `Null`。
- 在 `tests/security_committee_vote_cli.rs` 新增 `security_record_position_adjustment_supports_multiple_events` 红测，锁定同一 `position_plan_ref` 下连续两次调仓事件都能返回正式结构化结果，并保留 `decision_ref / approval_ref / evidence_version / plan_alignment`。
- 新增 `src/ops/security_record_position_adjustment.rs`，补最小 `security_record_position_adjustment` Tool，实现正式请求校验、确定性 `adjustment_event_ref` 生成与结构化事件回包。
- 更新 `src/ops/stock.rs`、`src/ops/mod.rs`、`src/tools/catalog.rs`、`src/tools/dispatcher.rs`、`src/tools/dispatcher/stock_ops.rs`，完成调仓事件 Tool 的 stock 主链导出、catalog 注册与 dispatcher 接线。

### 修改原因
- 用户要求先跑整仓，再直接进入 `Task 4`；因此需要先把当前真实基线重新确认清楚，再在不处理既有失败的前提下继续推进证券主链。
- `Task 4` 的目标是把“正式仓位计划”进一步推进成“正式调仓事件”，给后续 `Task 6` 的投后复盘提供可引用事件对象，而不是继续依赖对话里的临时文本记录。

### 方案还差什么
- [ ] 当前 `security_record_position_adjustment` 还是最小回声式 Tool，尚未接入真实 runtime 持久化或事件回查能力。
- [ ] `adjustment_event_ref` 目前采用确定性拼接规则，后续若同一标的在同一日期出现同类多次动作，需要在持久化阶段补版本/序号策略。
- [ ] `Task 5 / Task 6` 的正式复盘对象与聚合逻辑还未开始，当前只补齐了“计划 -> 事件”的执行层闭环。

### 潜在问题
- [ ] 当前没有对 `before_position_pct / after_position_pct` 做数值边界校验；若后续外部调用传入负值、超过 100% 或异常精度，仍可能污染复盘输入。
- [ ] 当前同一 `position_plan_ref` 的多次事件只是“可连续记录”，还没有落真实存储，因此后续复盘聚合仍需要在 `Task 6` 明确事件回读入口。
- [ ] 补完 `Task 4` 后，整份 `security_committee_vote_cli` 仍有 3 条非本轮目标失败：`security_committee_vote_rejects_key_risks_not_matching_risk_breakdown`、`security_committee_vote_rejects_risk_breakdown_category_mismatch`、`security_committee_vote_etf_fundamental_reviewer_uses_fund_review_semantics`；这些失败位于投决会逻辑，不是本轮新增调仓事件 Tool 直接引入的问题。

### 关闭项
- 整仓基线已复核：`cargo test -- --nocapture`
- 红测已按预期失败：`cargo test --test security_committee_vote_cli security_record_position_adjustment_supports_multiple_events -- --nocapture`
- 转绿验证已通过：`cargo test --test security_committee_vote_cli security_record_position_adjustment_supports_multiple_events -- --nocapture`
- 合同回归已通过：`cargo test --test integration_tool_contract security_record_position_adjustment_contract -- --nocapture`

## 2026-04-08
### 修改内容
- 在 `tests/integration_tool_contract.rs` 收口 `technical_consultation_basic` 合同红测：不再允许对外暴露 `as_of_date`，统一要求输出 `analysis_date`，并补充 `evidence_version` 断言。
- 在 `src/ops/technical_consultation_basic.rs` 将 `TechnicalConsultationBasicResult` 正式改为输出 `analysis_date / evidence_version`，删除对外 `as_of_date` 字段，并统一以行情窗口终点生成分析日期与证据版本号。
- 在 `src/ops/security_decision_briefing.rs` 同步把技术分析日期读取口径切换到 `analysis_date`，避免 briefing 继续依赖旧字段。
- 在 `tests/integration_tool_contract.rs` 新增 `security_post_trade_review_contract_exposes_required_fields` 红测，锁定投后复盘对象至少要暴露 `post_trade_review_ref / position_plan_ref / decision_ref / approval_ref / review_outcome / decision_accuracy / execution_quality / risk_control_quality / correction_actions / next_cycle_guidance`。
- 在 `src/tools/contracts.rs` 新增 `PostTradeReviewOutcome`、`PostTradeReviewDimension`、`SecurityPostTradeReviewRequest`、`SecurityPostTradeReviewResult`，并补最小 `from_request(...)` 构造辅助函数。

### 修改原因
- 用户明确要求按方案 C 处理既有失败，不留尾巴，因此需要把 `technical_consultation_basic` 的对外日期合同一次收口到证券主链统一口径，而不是继续保留 `as_of_date` 兼容尾巴。
- 用户随后要求直接继续 Task 5，因此需要趁这次合同收口把投后复盘的正式对象也先钉死，确保后续 Tool 开发不会再漂字段。

### 方案还差什么
- [ ] `security_post_trade_review` 目前完成的是合同层，正式 Tool、dispatcher 接线与聚合逻辑还未开始，需要在 Task 6 落地。
- [ ] `technical_consultation_basic` 虽已完成对外合同收口，但若仓库里还有外部文档或 Skill 说明引用旧字段 `as_of_date`，后续需要补文档同步。
- [ ] 当前没有重新跑整仓；本轮验证范围是 `integration_tool_contract` 全文件与相关定向回归，不代表整个仓库已全绿。

### 潜在问题
- [ ] `security_decision_briefing` 已切到 `analysis_date`，如果仓库中还有其它更深层路径直接读取 `TechnicalConsultationBasicResult.as_of_date`，后续在更大范围回归时才可能暴露，需要继续关注编译/运行面残留引用。
- [ ] `SecurityPostTradeReviewRequest` 当前把复盘结论维度作为正式输入字段保留下来，后续 Task 6 若改成纯聚合生成，需要确认是否保留手工覆写能力还是进一步拆成输入/输出两层对象。
- [ ] 当前 `integration_tool_contract` 全绿，但仓库仍存在既有 `dead_code` warning，且其它测试文件曾有非本轮目标失败，后续推进 Task 6 前最好继续做定向回归。

### 关闭项
- 红测已按预期失败：`cargo test --test integration_tool_contract technical_consultation_basic_contract_exposes_bullish_continuation_conclusion -- --nocapture`
- 修复后定向验证已通过：`cargo test --test integration_tool_contract technical_consultation_basic_contract_exposes_bullish_continuation_conclusion -- --nocapture`
- 窄回归已通过：`cargo test --test integration_tool_contract technical_consultation_basic_contract_exposes_bearish_continuation_conclusion -- --nocapture`
- Task 5 红测已按预期失败：`cargo test --test integration_tool_contract security_post_trade_review_contract -- --nocapture`
- Task 5 转绿验证已通过：`cargo test --test integration_tool_contract security_post_trade_review_contract -- --nocapture`
- `integration_tool_contract` 全文件回归已通过：`cargo test --test integration_tool_contract -- --nocapture`

## 2026-04-08
### 修改内容
- 在 `tests/security_analysis_resonance_cli.rs` 新增 `security_post_trade_review_aggregates_multiple_adjustments` 红测，锁定 `security_post_trade_review` 必须能围绕同一 `position_plan_ref` 聚合多条调仓事件并输出正式复盘结论。
- 在 `tests/security_analysis_resonance_cli.rs` 新增 `security_post_trade_review_rejects_broken_position_continuity` 红测，锁定方案 C 下事件链仓位衔接断裂必须直接报错。
- 在 `src/runtime/security_execution_store.rs` 新增执行层 SQLite store，把 `security_position_plan_record` 与 `security_record_position_adjustment` 的正式对象落盘，并支持按 ref 回读。
- 在 `src/ops/security_position_plan_record.rs`、`src/ops/security_record_position_adjustment.rs` 接入执行层落盘，使计划对象和调仓事件真正成为可回读的正式事实源。
- 新增 `src/ops/security_post_trade_review.rs`，实现正式复盘 Tool：请求边界校验、计划/事件回读、一致性检查、仓位衔接校验、复盘维度聚合与结构化输出。
- 更新 `src/tools/contracts.rs`，把 `SecurityPostTradeReviewRequest` 收口为真正的聚合输入，并新增 `SecurityPostTradeReviewResult::assemble(...)`。
- 更新 `src/ops/stock.rs`、`src/ops/mod.rs`、`src/runtime/mod.rs`、`src/tools/catalog.rs`、`src/tools/dispatcher.rs`、`src/tools/dispatcher/stock_ops.rs`，完成 Tool 导出、catalog 注册与 dispatcher 接线。

### 修改原因
- 用户已确认按方案 C 推进 Task 6，因此不能只做 happy path 聚合，必须同时落实事件链一致性校验与真实 ref 回读能力。
- 之前 `position_plan_record` 与 `position_adjustment` 虽然已有正式 ref，但仍然缺少真实落盘，导致 `post_trade_review` 无法只凭 ref 完成真正闭环；这轮需要把这个尾巴一起收掉。

### 方案还差什么
- [ ] 当前 `security_post_trade_review` 已能聚合单一计划下的多次调仓，但还没有把结果再落成正式审批简报或 decision package 资产，若后续要进入更强审计闭环，还需要继续接文档/审批出口。
- [ ] 当前执行层 store 只覆盖 plan / adjustment 两类对象；若后续要做更细的盘中执行日志、成交滑点或多版本计划并存，还需要扩表。
- [ ] 这轮没有重跑整仓，只完成了 Task 6 相关定向测试与 `integration_tool_contract` 全文件回归，不代表仓库其它既有失败已收口。

### 潜在问题
- [ ] `security_post_trade_review` 目前使用的是轻规则聚合，结论维度主要基于 `plan_alignment / event_type / 仓位衔接`，后续如果要纳入收益结果、赔率或复盘表现，需要再把信号结果研究层接进来。
- [ ] 执行层 store 当前使用独立 SQLite 文件 `security_execution.db`；如果未来用户希望全部统一进单一 runtime.db，需要额外迁移方案。
- [ ] 当前同日同类型调仓事件仍沿用确定性 ref 规则；若未来出现“同一日多次 reduce”场景，仍需要补版本/序号策略，否则会发生 upsert 覆盖。

### 关闭项
- 红测已按预期失败：`cargo test --test security_analysis_resonance_cli security_post_trade_review -- --nocapture`
- Task 6 定向验证已通过：`cargo test --test security_analysis_resonance_cli security_post_trade_review -- --nocapture`
- 仓位计划回归已通过：`cargo test --test security_analysis_resonance_cli security_position_plan_record_persists_briefing_plan -- --nocapture`
- 调仓事件回归已通过：`cargo test --test security_committee_vote_cli security_record_position_adjustment_supports_multiple_events -- --nocapture`
- 复盘合同回归已通过：`cargo test --test integration_tool_contract security_post_trade_review_contract -- --nocapture`
- `integration_tool_contract` 全文件回归已通过：`cargo test --test integration_tool_contract -- --nocapture`

## 2026-04-08
### 修改内容
- 更新 `README.md`，把证券主链当前状态正式收口为“仓位计划正式化 -> 多次调仓记录 -> 投后复盘”的最小闭环，并补充对应定向验证命令。
- 更新 `docs/AI_HANDOFF.md`，新增执行与复盘层交接说明，明确 `security_position_plan_record / security_record_position_adjustment / security_post_trade_review / security_execution_store` 的职责、闭环边界与未完成项。
- 在 `docs/AI_HANDOFF.md` 同步补充新的禁区、入口文件、已知风险和下一阶段优先级，避免后续 AI 把正式仓位/调仓/复盘重新退回对话文本层。

### 修改原因
- 用户已批准按方案 C 执行 Task 7，因此需要回到证券主链真实 worktree，把 Task 6 已落地的执行闭环写成正式文档状态，而不是继续沿用只覆盖 briefing/vote 的旧口径。
- Task 6 虽然已经完成代码与定向回归，但如果 README 和 AI handoff 不更新，后续接手很容易误判“仓位管理和投后复盘仍未开始”，从而重复开发或走回头路。

### 方案还差什么
- [ ] 当前只完成了文档收口，尚未把 `security_post_trade_review` 结果继续装订成正式审批简报对象或 decision package 资产。
- [ ] 当前闭环仍是单标的、多次调仓级别，组合层仓位治理、盘中执行日志、滑点与成交质量还没有进入正式对象。
- [ ] `security_execution_store` 目前仍是独立 SQLite 文件，若后续要并入统一 runtime 存储，需要单独设计迁移方案。

### 潜在问题
- [ ] 当前同日同类型调仓事件仍存在 ref 冲突风险；若未来出现“同一天多次 reduce/add”，需要补版本号或序号策略，否则可能覆盖旧事件。
- [ ] 当前 `security_post_trade_review` 仍是轻规则聚合，没有接入真实收益表现、赔率兑现或更细粒度执行质量指标，结论解释力仍有限。
- [ ] 本轮是文档任务，不会自动证明整仓全绿；需要靠本轮重新执行的定向回归来支撑“文档与当前实现一致”的结论。

### 关闭项
- 文档已同步更新：`README.md`
- 文档已同步更新：`docs/AI_HANDOFF.md`
- 任务日志已追加：`.trae/CHANGELOG_TASK.md`

## 2026-04-08
### 修改内容
- 新增 `docs/execution-notes-2026-04-08-security-post-trade-review-closeout.md`，集中记录本轮证券主链“仓位计划正式化 -> 多次调仓记录 -> 投后复盘”闭环的落地范围、定向验证命令、当前边界和接手建议。
- 为本次 GitHub 上传准备补齐交接材料，明确这一批待推送文件属于同一条证券主链任务链，而不是零散临时修补。

### 修改原因
- 用户要求直接把当前 worktree 的脏状态处理后推回 GitHub，因此在提交前需要把“做了什么、验证了什么、还有什么没做”写成可持久化文档，避免上传后只剩代码没有上下文。
- 这轮变更横跨正式 Tool、执行层存储、合同测试和交接文档，若没有单独 execution note，后续 AI 很容易只看到代码文件而误判完成边界。

### 方案还差什么
- [ ] 当前仅完成上传前交接材料补齐，尚未实际提交与推送；提交哈希和远端状态需要在 Git 操作完成后再确认。
- [ ] 本轮 execution note 记录的是定向回归与交接范围，不代表整仓 `cargo test -- --nocapture` 已重新全量转绿。
- [ ] 后续仍需要继续推进复盘结果资产化、收益结果接入和同日多次调仓版本化。

### 潜在问题
- [ ] 如果后续继续在当前分支叠加其它无关任务，而不及时拆提交，本轮这批“执行闭环”改动的历史边界会再次变模糊。
- [ ] 当前 execution note 依赖本轮已执行的定向验证命令；若后续代码继续变化但不补验证，文档结论会过时。
- [ ] Windows 下 Git 下一次触碰这些文件时仍可能把 LF 正规化为 CRLF，这不是本轮功能问题，但需要留意后续 diff 噪音。

### 关闭项
- 上传前执行说明已补齐：`docs/execution-notes-2026-04-08-security-post-trade-review-closeout.md`
- 本次上传准备已补任务日志：`.trae/CHANGELOG_TASK.md`

## 2026-04-09
### 修改内容
- 更新 `src/ops/security_committee_vote.rs`，补齐 `risk_breakdown` 分类一致性校验与 `key_risks` 派生一致性校验，避免脏 payload 进入正式投决会表决。
- 更新 `src/ops/security_committee_vote.rs`，恢复 ETF 标的在 `fundamental_reviewer` 席位上的 `fund_review` 语义，不再因为缺少个股财报式 `fundamental_ready` 就直接 `defer`。
- 更新 `docs/AI_HANDOFF.md`，新增“其他 worktree 吸收当前证券主链的默认方案”，明确默认吸收入口、推荐命令、最小验证与冲突热点。
- 更新 `docs/交接摘要_证券分析_给后续AI.md`，同步固化证券主链跨 worktree 吸收口径，避免后续在别的开发位置重复等待 AI 判断合并路径。

### 修改原因
- `security_committee_vote_cli` 里已有 3 条稳定失败用例，分别暴露了 `risk_breakdown/category` 退化、`key_risks` 与结构化风险漂移，以及 ETF `fund_review` 语义回退问题；这轮必须按正式合同把它们收口。
- 用户明确要求把“别的 worktree 应该怎么吸收这次收口结果”写进交接文档，避免后续换地方继续合并时再重复等待判断。

### 方案还差什么
- [ ] 当前跨 worktree 吸收方案已经写入文档，但若要被别的物理仓库直接消费，仍需要后续提交/推送当前分支结果。
- [ ] 当前验证聚焦于 `security_committee_vote` 定向回归与测试编译通过，尚未顺带清理仓库里既有的大量 `dead_code` warning。
- [ ] 本轮没有顺手处理 `tests/security_scorecard_cli.rs` 的 `unused import: Path` 警告，因为它不是这次合并收口的 blocker。

### 潜在问题
- [ ] `key_risks` 目前按四个 bucket 的首条 `headline` 派生；如果未来主链决定改成“多条摘要”或“权重摘要”，需要同步更新 vote/briefing 两侧合同。
- [ ] ETF `fund_review` 现在恢复为不直接 `defer` 的保守语义；如果后续要细分境内 ETF、跨境 ETF、商品 ETF，仍需要继续拆更细的 fund-review 条件。
- [ ] 文档里的跨 worktree 吸收方案假设目标位置能够访问当前分支或其远端；如果目标环境看不到该分支引用，仍需要先同步 Git 引用。

### 关闭项
- RED 已复现：`cargo test --test security_committee_vote_cli -- --nocapture`
- 定向回归已通过：`cargo test --test security_committee_vote_cli -- --nocapture`
- 测试编译验证已通过：`cargo test --tests --no-run`
- 交接手册已补跨 worktree 吸收方案：`docs/AI_HANDOFF.md`
- 证券专项交接摘要已补跨 worktree 吸收方案：`docs/交接摘要_证券分析_给后续AI.md`

## 2026-04-10
### 修改内容
- 更新 `.gitignore`，新增本地运行时与结果存储目录忽略规则，覆盖 `.excel_skill_runtime/`、`tests/runtime_fixtures/local_memory/`、`tests/runtime_fixtures/thread_local_memory/`、`tests/runtime_fixtures/integration_tool_contract/` 以及 `chart_ref_store / exports / generated_workbooks / local_memory_registry / result_ref_store* / table_ref_store`。
- 更新 `docs/AI_HANDOFF.md`，新增“证券主链最小验证清单”“运行时产物与测试夹具规则”“当前已知非 blocker”三段，固定跨 worktree 吸收后的默认验证入口与噪音识别口径。
- 更新 `docs/交接摘要_证券分析_给后续AI.md`，同步补入证券主链 6 条最小验证命令、运行时产物规则和非 blocker 尾项说明。

### 修改原因
- 用户希望后续换到别的开发位置时不要再等 AI 重新判断，因此除了写清合并方案，还需要把“合并后怎么验证”和“哪些脏目录只是本地产物”一起固化。
- 当前仓库存在大量运行时落盘目录，若不收口规则，后续很容易把本地噪音误判成证券主链真实变更。

### 方案还差什么
- [ ] 当前已把最常见的本地产物目录加入忽略规则，但 `tests/runtime_fixtures/security_scorecard_training/security_scorecard_training_ready_1775734358375766100/` 这类新业务样本仍需人工判断是否要纳入版本库。
- [ ] 当前还没有形成新的稳定提交锚点；如果要让其他机器/仓库直接吸收这轮结果，后续仍需要提交并推送。
- [ ] 当前只收口了“最小验证包”和“产物规则”，尚未把这些命令再整理成单独执行脚本或自动化命令集合。

### 潜在问题
- [ ] `.gitignore` 现在只忽略确认过“仓库中无已跟踪文件”的目录；如果后续测试又引入新的本地输出目录，仍需要继续补规则。
- [ ] `title-gap-header-stale.xlsx` 和 `tests/runtime_fixtures/中文路径/` 当前仍是未跟踪状态，它们更像候选 fixture 而不是纯产物，不能自动忽略，需要后续人工确认。
- [ ] 当前既有 `dead_code` warning 与 `tests/security_scorecard_cli.rs` 的 `unused import` warning 仍然存在，后续 AI 若做质量治理应单独处理，不要混入本轮证券主链收口。

### 关闭项
- 交接手册已补最小验证清单：`docs/AI_HANDOFF.md`
- 证券专项交接摘要已补最小验证清单：`docs/交接摘要_证券分析_给后续AI.md`
- 运行时产物忽略规则已落地：`.gitignore`
- 忽略规则已验证生效：`git check-ignore -v .excel_skill_runtime tests/runtime_fixtures/local_memory tests/runtime_fixtures/thread_local_memory tests/runtime_fixtures/exports tests/runtime_fixtures/generated_workbooks tests/runtime_fixtures/result_ref_store`

## 2026-04-10
### 修改内容
- 新增设计稿 `docs/plans/2026-04-10-security-structure-gap-and-condition-review-design.md`，正式收口“证券体系结构缺口图 + 下一阶段实施顺序”，并明确把原本不适配现状的“投中实时监控中枢”改写为“条件复核中枢”。
- 新增实施计划 `docs/plans/2026-04-10-security-condition-review-hub.md`，把条件复核中枢拆成可执行任务，覆盖合同、触发分流、CLI 接线、package 挂接、execution/review 挂接和文档收口。

### 修改原因
- 用户要求先拿到“证券体系结构缺口图 + 下一阶段实施顺序”，并明确指出当前没有实时数据，因此原先“实时投中监控中枢”的方向需要重写为可落地版本。
- 现有证券主链已经具备投前、执行、投后和评分卡最小闭环，但仍缺少一个正式说明“中间层该怎么补”的设计入口；若不先固化设计和顺序，后续实现容易再次碎片化。

### 方案还差什么
- [ ] 当前只完成了设计稿与实施计划，还没有开始实现 `security_condition_review`。
- [ ] 设计里提到的 `security_signal_outcome_backfill`、`security_committee_calibration`、ETF 专项适配层和组合级风险引擎仍是后续阶段任务。
- [ ] 这轮没有为新文档单独补 Git 提交锚点；如果要作为别处吸收依据，后续仍需要提交并推送。

### 潜在问题
- [ ] 路线图文档里旧的“投中监控”表述仍可能与新设计稿并存，后续实现时需要统一命名口径，避免再次混用“监控中枢”和“条件复核中枢”。
- [ ] 当前实施计划默认 `security_execution_journal` 已经存在且继续复用；如果后续实现中发现 journal 合同不足，计划需要同步修订。
- [ ] 这轮是纯文档任务，没有运行测试，因为没有改动 Rust 实现代码。

### 关闭项
- 结构设计稿已落盘：`docs/plans/2026-04-10-security-structure-gap-and-condition-review-design.md`
- 条件复核中枢实施计划已落盘：`docs/plans/2026-04-10-security-condition-review-hub.md`

## 2026-04-10
### 修改内容
- 新增 `src/ops/security_condition_review.rs`，落地条件复核最小正式合同与构造逻辑，包括：
  - `SecurityConditionReviewRequest`
  - `SecurityConditionReviewDocument`
  - `SecurityConditionReviewResult`
  - `SecurityConditionReviewError`
  - `security_condition_review(...)`
- 更新 `src/tools/contracts.rs`，新增条件复核共用枚举：
  - `SecurityConditionReviewTriggerType`
  - `SecurityConditionReviewFollowUpAction`
- 更新 `src/ops/stock.rs`，导出 `security_condition_review` 模块。
- 新增 `tests/security_condition_review_cli.rs`，补齐 Task 1-2 定向测试，覆盖：
  - `manual_review -> keep_plan`
  - `end_of_day_review -> update_position_plan`
  - `event_review -> reopen_committee`
  - `event_review + 冻结关键词 -> freeze_execution`
  - `data_staleness_review -> reopen_research`

### 修改原因
- 用户已批准“条件复核中枢”路线，并要求直接进入 Task 1 后继续到 Task 2，因此需要先把最小正式合同落地，再把四类触发模式与分流动作锁成测试。
- 当前证券主链缺少正式投中对象层；如果不先补条件复核，投前决策、执行事实和投后复盘之间仍然会缺一层中间判断。

### 方案还差什么
- [ ] 当前只完成了 Task 1-2，还没有把 `security_condition_review` 接进 `catalog / dispatcher / stock_ops`，因此它还不是正式 CLI Tool。
- [ ] 当前 `condition_review_ref` 还没有挂入 `security_decision_package`、`security_execution_record`、`security_post_trade_review`。
- [ ] 当前分流规则仍是最小关键词版本，后续需要接入更正式的 thesis / execution / evidence freshness 判断。

### 潜在问题
- [ ] 当前 `derive_follow_up_action` 主要依赖触发类型和关键词匹配，后续如果事件摘要口径变化，动作分流可能不够稳，需要升级为结构化条件。
- [ ] 当前 `condition_review_id` 还只按 `symbol + analysis_date + trigger_type` 生成，后续若同一类型多次复核，可能需要版本号或时间戳策略。
- [ ] 目前还没有对 `package_path` 的路径合法性或对象存在性做校验，这部分应放在后续 package 挂接阶段处理。

### 关闭项
- Task 1 红测已复现：`cargo test --test security_condition_review_cli security_condition_review_manual_review_contract -- --nocapture`
- Task 1 已转绿：`cargo test --test security_condition_review_cli security_condition_review_manual_review_contract -- --nocapture`
- Task 2 定向回归已通过：`cargo test --test security_condition_review_cli -- --nocapture`

## 2026-04-10
### 修改内容
- 追加本次代码扫描记录，整理当前工作区的主要代码问题，覆盖集成测试失败、结果集时间语义恢复缺陷，以及 dispatcher 遗留死代码导致的严格静态检查失败。

### 修改原因
- 用户要求扫描整个工程并指出代码问题，需要把本轮审查的关键结论沉淀到任务日志，便于后续继续修复时直接复用。

### 方案还差什么
- [ ] 当前只完成问题定位和风险分级，还没有进入修复方案评审与代码修改阶段。
- [ ] `tests/integration_registry.rs` 的 region table_ref 回归为什么在 `save()` 之后仍然读不到文件，还需要单独做最小复现实验收口根因。
- [ ] `src/tools/dispatcher.rs` 的遗留分发函数与拆分后的子模块之间，还需要做一次统一收口，避免继续双轨漂移。

### 潜在问题
- [ ] `src/frame/result_ref_store.rs` 当前会丢弃持久化的 `time_zone`，如果上层开始保存带时区的 `Datetime` 列，回放结果会发生静默语义漂移。
- [ ] `cargo test --quiet` 当前在 `integration_registry` 仍有失败项，因此不能把当前工作区视为稳定绿态基线。
- [ ] `cargo clippy --all-targets -- -D warnings` 当前无法通过，说明仓库还不具备“warnings as errors”质量门禁能力。

### 关闭项
- 已完成仓库级问题扫描：`cargo test --quiet`
- 已确认失败用例：`cargo test --test integration_registry stored_region_table_ref_round_trips_and_reloads_same_region -- --nocapture`
- 已确认严格静态检查失败：`cargo clippy --quiet --all-targets -- -D warnings`
## 2026-04-10
### 修改内容
- 更新 `docs/AI_HANDOFF.md`，补入 foundation metadata `Validator Linkage` 的正式交接说明，明确当前已消费 `deprecated / replaced_by / aliases`，并固定“下一步是 Repository-Level Audit，不是 Migration Executor”。
- 更新 `task_plan.md`，把方案 A 已完成、方案 B 成为默认下一步的状态写实，避免计划文件继续停留在候选阶段。
- 更新 `progress.md`，补录本轮 validator linkage 的测试、实现点和阶段结论。
- 更新 `findings.md`，补充 alias / deprecated / replaced_by 在 validator 层的当前语义边界与非目标范围。

### 修改原因
- foundation validator 联动代码和测试已经完成，但交接与计划文档尚未同步，后续 AI 很容易误判当前阶段仍停留在 migration contract。
- 用户已经反复要求固定架构与阶段结论，避免每次接手都重开架构或重复判断边界，因此需要把本轮真实状态写进 AI 交接面与任务日志。

### 方案还差什么
- [ ] 下一步默认应进入 `Repository-Level Audit`，但仍需先重新出方案并等待用户批准后再开发。
- [ ] 当前没有进入 `Migration Executor`，后续若要做 dry-run / rewrite，需要单独获批并补新的设计与测试。
- [ ] 本轮只补了文档承接，没有处理仓库中其他无关脏改动与既有 warning。

### 潜在问题
- [ ] `docs/AI_HANDOFF.md` 当前仍以证券主链为默认入口，后续 AI 需要注意“只有在用户明确要求继续 foundation 时，才进入本轮 metadata 治理口径”。
- [ ] 仓库工作树本来就较脏，如果后续直接看 `git status` 而不看交接文档，仍可能把无关改动误判成这轮 foundation 结果。
- [ ] `task-journal` skill 在当前 PowerShell 编码环境下仍不稳定，本轮继续采用手工追加日志，后续若修脚本需单独处理。

### 关闭项
- foundation validator linkage 文档承接已补齐：`docs/AI_HANDOFF.md`
- 计划与进度文档已补齐：`task_plan.md`、`progress.md`、`findings.md`
- 本轮任务日志已追加：`.trae/CHANGELOG_TASK.md`

## 2026-04-10
### 修改内容
- 更新 `tests/security_condition_review_cli.rs`，按 TDD 为 Task 3 追加两条 CLI 红绿测试，覆盖：
  - `tool_catalog` 能发现 `security_condition_review`
  - CLI JSON 请求能路由到 `security_condition_review` 并返回结构化结果
- 更新 `src/tools/catalog.rs`，把 `security_condition_review` 注册进 `STOCK_TOOL_NAMES` 与 `TOOL_NAMES`。
- 更新 `src/tools/dispatcher.rs`，为 `security_condition_review` 新增正式主路由分支。
- 更新 `src/tools/dispatcher/stock_ops.rs`，新增 `dispatch_security_condition_review(args)`，完成 JSON 解析、规则执行和 `ToolResponse` 序列化返回。

### 修改原因
- 用户要求继续执行 Task 3，而 Task 3 的明确目标就是把此前只完成内部合同和最小规则的 `security_condition_review` 升级为正式 CLI Tool。
- 如果只保留内部 `ops` 层实现，不把它注册进 `catalog / dispatcher / stock_ops`，后续 Skill、CLI 和证券主链编排都无法正式消费这条条件复核能力。

### 方案还差什么
- [ ] 当前只完成了 Task 3 的 CLI 正式接线，还没有把 `condition_review_ref` 装订进 `security_decision_package`、`security_execution_record` 与 `security_post_trade_review`。
- [ ] 当前 `security_condition_review` 仍是“触发类型 + 关键词”最小规则，后续需要在 Task 4/5 再把 thesis、执行状态和证据新鲜度挂进更完整的判断链。
- [ ] 这轮没有顺手处理仓库里既有的 `dead_code` warning，因为它们不是 Task 3 的阻塞项。

### 潜在问题
- [ ] 如果未来同一 `symbol + analysis_date + trigger_type` 下需要多次条件复核，当前 `condition_review_id` 规则仍可能发生 ref 覆盖，需要后续补版本号或序号。
- [ ] CLI 路由当前只验证了 `manual_review` 的主链 happy path，后续若要防止 JSON 枚举值或字段命名漂移，建议继续补 `event_review / data_staleness_review` 的 CLI 级回归。
- [ ] 目前 `package_path` 仍只作为透传字段，没有做文件存在性或对象合法性校验，这部分应在后续 package 挂接阶段收口。

### 关闭项
- Task 3 红测已复现：`cargo test --test security_condition_review_cli security_condition_review_is_cataloged -- --nocapture`
- Task 3 路由红测已复现：`cargo test --test security_condition_review_cli security_condition_review_cli_returns_structured_result -- --nocapture`
- Task 3 定向回归已通过：`cargo test --test security_condition_review_cli -- --nocapture`
- 本轮任务日志已追加：`.trae/CHANGELOG_TASK.md`

## 2026-04-10
### 修改内容
- 新增 `docs/plans/2026-04-10-foundation-repository-metadata-audit-design.md`，正式固化 foundation `Repository-Level Audit` 的方案边界、报告模型与最小 hygiene diagnostics 规则。
- 新增 `docs/plans/2026-04-10-foundation-repository-metadata-audit-plan.md`，把 repository audit 的 TDD 执行步骤拆成可直接落地的实施计划。
- 新增 `tests/repository_metadata_audit_unit.rs`，按 TDD 先补 repository 级红测，覆盖 issue 聚合、类型计数、concept 计数和 `DuplicateEvidenceRef / WeakLocator / WeakSourceRef`。
- 新增 `src/ops/foundation/repository_metadata_audit.rs`，实现 foundation 仓库级 metadata audit，并在 `src/ops/foundation.rs` 暴露正式模块出口。
- 更新 `docs/AI_HANDOFF.md`、`task_plan.md`、`progress.md`、`findings.md`，同步把 foundation metadata 治理最新阶段推进到 `Repository-Level Audit`。

### 修改原因
- 用户已批准方案 B，要求继续 foundation 主线，因此需要把 metadata 治理从节点级 validator 提升到仓库级可观察审计，而不是停留在候选方案。
- evidence hygiene diagnostics 已被用户明确点名，需要先以最小只读规则落地，避免后续 AI 再把它误判成未开始或直接越界做自动迁移。

### 方案还差什么
- [ ] 当前 hygiene diagnostics 仍是最小版，后续还可以继续扩细弱 locator、弱 source_ref 和重复证据的判定规则。
- [ ] 当前没有进入 `Migration Executor`，若后续要做 dry-run / rewrite，仍需单独获批并补新的设计与测试。
- [ ] 本轮没有处理仓库里既有的大量 `dead_code` warning，因为它们不是 foundation repository audit 的 blocker。

### 潜在问题
- [ ] 当前 `issue_type_counts` 直接复用 `MetadataValidationIssue` 变体名；如果未来要对外暴露稳定审计协议，可能需要单独的报告层 kind 枚举。
- [ ] 当前 `concept_issue_counts` 故意不统计 `AliasFieldUsed / DeprecatedFieldUsed / InvalidValueType / InvalidAllowedValue`，后续若要更细治理看板，需要明确是否扩口径。
- [ ] 当前重复证据诊断按“同一 `source_ref + locator` 跨节点出现多次”定义，这是一条保守提示规则，不代表这些证据一定错误。

### 关闭项
- repository audit 红测已复现：`cargo test --test repository_metadata_audit_unit -- --nocapture`
- repository audit 定向验证已通过：`cargo test --test repository_metadata_audit_unit -- --nocapture`
- foundation metadata 治理回归已通过：`cargo test --test repository_metadata_audit_unit --test metadata_validator_unit --test metadata_schema_registry_unit --test metadata_schema_versioning_unit --test metadata_migration_contract_unit --test knowledge_repository_unit --test knowledge_ingestion_unit --test knowledge_bundle_unit -- --nocapture`
- 本轮任务日志已追加：`.trae/CHANGELOG_TASK.md`

## 2026-04-10 Task Journal
- 任务：foundation 主线 `方案A`，扩展 evidence hygiene diagnostics。
- 变更：新增 `MissingEvidenceRef`、`DuplicateEvidenceRefWithinNode`、`RepositoryWeakLocatorReason`、`RepositoryWeakSourceRefReason`；扩展 `WeakLocator` 与 `WeakSourceRef` 诊断载荷。
- 测试：`cargo test --test repository_metadata_audit_unit -- --nocapture`；`cargo test --test repository_metadata_audit_unit --test metadata_validator_unit --test metadata_schema_registry_unit --test metadata_schema_versioning_unit --test metadata_migration_contract_unit --test knowledge_repository_unit --test knowledge_ingestion_unit --test knowledge_bundle_unit -- --nocapture`。
- 结论：repository-level evidence hygiene 子能力累计 7 项，当前下一步仍是 foundation 治理深化，不进入 migration executor。

## 2026-04-10
### 修改内容
- 更新 `src/ops/security_decision_package.rs`、`src/ops/security_decision_verify_package.rs`、`src/ops/security_decision_package_revision.rs` 与相关测试，完成 Task 4：
  - `decision package` 正式承载 `condition_review_ref`
  - 新增最小 `condition_review_digest`
  - `verify_package` 能校验 `condition_review` 绑定是否存在、是否一致
  - `package revision` 会继承既有 `condition_review` 锚点与摘要
- 更新 `src/ops/security_execution_record.rs`、`src/ops/security_post_trade_review.rs`、`src/ops/stock.rs`、`src/tools/catalog.rs`、`src/tools/dispatcher.rs`、`src/tools/dispatcher/stock_ops.rs` 与相关测试，完成 Task 5：
  - `security_execution_record` 与 `security_post_trade_review` 正式注册为 CLI Tool
  - `execution_record` 可挂接 `condition_review_ref`，并落盘 `trigger_type / follow_up_action / summary`
  - `post_trade_review` 可读取 execution 层挂接的 `condition_review`，生成正式解释字段
  - 补齐 execution/review 在当前分支缺失的最小正式合同，使执行与复盘链在本文件内自洽可编译

### 修改原因
- 用户已批准继续推进 Task 4 和 Task 5，要求采用稳妥方案，不做最小糊口实现，也不要把边界蔓延到新的仓储或实时监控层。
- 当前证券主链已经有 `condition_review` 中枢，但如果不继续接到 package / execution / review，投中复核就无法真正进入审批、执行、复盘的正式闭环。
- 这轮还暴露出 `execution_record / post_trade_review` 在当前分支长期处于“文件存在但未真正接线编译”的状态，因此必须顺手把 CLI 注册与最小合同补齐，否则后续 Skill 无法稳定调用。

### 方案还差什么
- [ ] 当前 `execution_record / post_trade_review` 的执行成交与仓位计划合同，采用的是“当前分支内自洽的最小正式实现”；如果后续要接更完整的成交流水、账户预算仓储或审批落盘，需要单独立项。
- [ ] 当前 `condition_review` 在 execution/review 层优先支持“显式 ref + 可选 document”，还没有做按 ref 反查历史 condition review 文档的仓储读取。
- [ ] 这轮没有进入 Task 6 文档总收口，因此 `AI_HANDOFF` 和证券交接手册里的 execution/review 新能力说明还需下一轮补齐。

### 潜在问题
- [ ] `security_execution_record.rs` 里新增了本地自洽的 journal / position-plan / portfolio-plan 合同，后续如果别的分支也补了同名正式模块，合并时需要优先统一类型来源，避免再次分叉。
- [ ] 当前 `position_plan_id` 使用 `position-plan:{symbol}:{analysis_date}:v1` 的稳定规则；如果未来要支持同日多版本计划，需要再引入版本号或审批锚点避免 ref 冲突。
- [ ] 目前整仓与投后层仍有大量既有 `dead_code` warning，这轮没有处理，因为它们不是 Task 4/5 的阻塞项。

### 关闭项
- Task 4 定向验证已通过：
  - `cargo test --test security_decision_verify_package_cli security_decision_verify_package_accepts_signed_package_and_writes_report -- --nocapture`
  - `cargo test --test security_decision_verify_package_cli security_decision_verify_package_fails_after_condition_review_binding_is_tampered -- --nocapture`
  - `cargo test --test security_decision_package_revision_cli security_decision_package_revision_builds_v2_package_after_approval_update -- --nocapture`
- Task 5 执行层整文件回归已通过：
  - `cargo test --test security_execution_record_cli -- --nocapture`
- Task 5 投后层整文件回归已通过：
  - `cargo test --test security_post_trade_review_cli -- --nocapture`
- 本轮任务日志已追加：`.trae/CHANGELOG_TASK.md`

## 2026-04-10
### 修改内容
- 更新 `docs/execution-notes-2026-04-07-foundation-navigation-kernel.md`，补入 foundation metadata repository audit 与 evidence hygiene expansion 的执行说明、验证命令和下一步承接点。
- 整理本轮 foundation 上传范围，确认只上传：
  - `repository_metadata_audit`
  - `repository_metadata_audit_unit`
  - 两组 foundation 设计/计划文档
  - handoff / task_plan / progress / findings / task journal 的承接补充

### 修改原因
- 用户要求先推到 GitHub；按既定交接规范，上传前必须把执行记录与 handoff 证据补齐，避免“代码已推但下一个 AI 不知道这轮做了什么”。
- 当前工作区存在大量与 foundation 无关的脏改动，因此需要明确本次上传只覆盖 foundation 元数据治理线的实际产物。

### 方案还差什么
- [ ] 本轮上传后，foundation 后续仍需继续补 locator/source_ref 结构规则与报告分级。
- [ ] 本轮不会顺手处理证券主线或其他既有脏改动，它们需要单独任务继续收口。

### 潜在问题
- [ ] `docs/AI_HANDOFF.md`、`task_plan.md`、`progress.md`、`findings.md`、`.trae/CHANGELOG_TASK.md` 在这轮前已存在历史修改，本次提交会把当前文件状态一并带上；后续如需细分来源，要以任务日志为准。
- [ ] 仓库仍有既有 `dead_code` warnings，本轮验证通过不代表 warnings 已清零。

### 关闭项
- foundation 上传前执行说明已补齐：`docs/execution-notes-2026-04-07-foundation-navigation-kernel.md`
- foundation 上传准备已补任务日志：`.trae/CHANGELOG_TASK.md`
## 2026-04-10
### 修改内容
- 更新 `docs/AI_HANDOFF.md`，追加“CLI 历史分支承接关系”章节，明确 `codex/merge-cli-mod-batches` 与当前 `codex/foundation-navigation-kernel` 的真实关系。
- 在交接手册中补入模块映射表，覆盖：
  - 旧 CLI 分支的证券决策链模块
  - foundation metadata / navigation 相关模块
  - Python `cli/*` 与 `tradingagents/*` 历史资产的当前定位

### 修改原因
- 用户要求先把 CLI 分支和当前主线做一次承接整理，避免后续 AI 再把 `merge-cli-mod-batches` 误判成“更近的更新来源”。
- 当前仓库长期并存多条历史证券/CLI 分支，如果不把“哪些已保留、哪些已替换、哪些不再建议继续”写进 handoff，后续很容易再次走回旧形态。

### 方案还差什么
- [ ] 当前只补了 handoff 层的映射结论，还没有单独拆成更细的“旧模块 -> 新模块”专项文档。
- [ ] 如果后续要做更严格的历史追踪，还可以继续把每个旧模块的最后提交锚点和替代模块提交锚点补成独立清单。

### 潜在问题
- [ ] 模块映射是“当前主线视角下的工程判断”，不是逐行代码一一等价替换关系；后续如果做更细审计，需要继续按具体模块深入 diff。
- [ ] `docs/AI_HANDOFF.md` 是持续追加型文档，后续 AI 应以最新日期章节为准，不要只看前面更早的 CLI 或证券历史章节。

### 关闭项
- CLI 历史分支承接关系已写入：`docs/AI_HANDOFF.md`
- 本轮整理任务日志已追加：`.trae/CHANGELOG_TASK.md`

## 2026-04-11
### 修改内容
- 更新 `docs/AI_HANDOFF.md`，在“CLI 历史分支承接关系”章节下追加“文件级映射表”。
- 追加的映射范围覆盖：
  - `security_committee_vote / security_decision_briefing / security_execution_record / security_post_trade_review / signal_outcome_research / security_analysis_resonance`
  - `security_decision_package / security_decision_verify_package / security_decision_package_revision`
  - foundation `metadata_schema / metadata_validator / repository_metadata_audit / navigation_pipeline`
  - `catalog / dispatcher / stock_ops`
  - Python `cli/*` 与 `tradingagents/*` 历史资产的当前定位

### 修改原因
- 用户批准继续按方案A整理，希望把“模块级承接”进一步细化到“文件级承接”，避免后续 AI 在具体文件层面再次迷路。
- 当前仓库历史分支和当前主线长期并存，如果没有文件级映射，后续很容易重新打开旧 CLI 文件并误判为当前应继续开发的主入口。

### 方案还差什么
- [ ] 当前文件级映射表只覆盖了最容易误判的核心文件，还没有把所有历史测试夹具和运行时样本都做逐项映射。
- [ ] 如果后续要做更严谨的历史治理，还可以继续补“旧文件最后有效提交 -> 新文件承接提交”的锚点清单。

### 潜在问题
- [ ] 文件级映射表是“当前主线的工程判断”，不是逐函数一一替换关系；后续做深度追溯时，仍需要结合具体 diff。
- [ ] `docs/AI_HANDOFF.md` 为持续追加型文档，后续接手默认应读最新日期章节，不要只看早期证券/CLI 历史章节。

### 关闭项
- 文件级映射表已写入：`docs/AI_HANDOFF.md`
- 本轮任务日志已追加：`.trae/CHANGELOG_TASK.md`
## 2026-04-10
### 修改内容
- 更新 `docs/AI_HANDOFF.md`，追加“2026-04-10 条件复核中枢正式收口”章节，统一投中层最新事实、默认验证清单和后续边界。
- 更新 `docs/交接摘要_证券分析_给后续AI.md`，追加证券专项条件复核中枢收口摘要，明确 `condition_review` 已进入 package、execution、review 主链。
- 新增 `docs/execution-notes-2026-04-10-security-condition-review-closeout.md`，记录本轮文档收口范围、验证命令和后续风险。

### 修改原因
- 用户要求进入 Task 6 并推到 GitHub，因此需要先把条件复核中枢这轮主链进展写回正式交接文档，而不是只留在代码和测试里。
- 如果 handoff 仍停留在旧口径，后续 AI 很容易误判“投中层还没正式落地”或“默认验证清单不含 security_condition_review_cli”。

### 方案还差什么
- [ ] 当前 handoff 已写明 `condition_review` 的正式边界，但还没有新增“按 ref 回仓储自动查 condition review 文档”的实现。
- [ ] 当前同日同类型多次条件复核的版本策略仍未正式落地，后续若扩展复核频次，需要单独补设计和测试。

### 潜在问题
- [ ] `docs/AI_HANDOFF.md` 与证券专项交接摘要是持续追加型文档，旧章节里仍保留历史语境；后续接手时应以最新日期章节为准。
- [ ] 当前默认验证清单虽然已经升级到 7 条，但它仍是“最小回归包”，不等于整仓全量绿。

### 关闭项
- Task 6 文档收口已补齐：`docs/AI_HANDOFF.md`
- Task 6 证券专项交接已补齐：`docs/交接摘要_证券分析_给后续AI.md`
- Task 6 执行记录已补齐：`docs/execution-notes-2026-04-10-security-condition-review-closeout.md`
## 2026-04-10
### 修改内容
- 修正文档里的验证命令口径，把不存在的 `security_decision_package_cli` 替换为真实可跑的 `security_decision_verify_package_cli + security_decision_package_revision_cli`。
- 更新 `docs/AI_HANDOFF.md`、`docs/交接摘要_证券分析_给后续AI.md` 与 `docs/execution-notes-2026-04-10-security-condition-review-closeout.md`，补入本轮 fresh 验证的真实结果。

### 修改原因
- Task 6 fresh 验证时确认 `security_decision_package_cli` 在当前仓库并不存在，如果不纠正，后续 AI 会继续拿错误命令当默认验证入口。
- 用户要求本轮直接推到 GitHub，因此上传前必须把“哪些已通过、哪些还没绿”写实，而不是只保留理想清单。

### 方案还差什么
- [ ] `security_scorecard_training_cli` 当前仍失败，若后续要恢复“默认最小验证清单全绿”，需要单独处理训练链回归。
- [ ] 本轮没有顺手修复评分卡训练失败，因为它不属于条件复核中枢文档收口的直接边界。

### 潜在问题
- [ ] 即使本轮可以推送，分支状态也应视为“条件复核链已收口，但默认清单未全绿”，不要对外误报为整仓全绿。

### 关闭项
- 文档验证命令口径已纠正：`docs/AI_HANDOFF.md`
- 证券专项验证命令口径已纠正：`docs/交接摘要_证券分析_给后续AI.md`
- 本轮 fresh 验证结果已记入执行说明：`docs/execution-notes-2026-04-10-security-condition-review-closeout.md`
## 2026-04-10
### 修改内容
- 在 `tests/security_scorecard_training_cli.rs` 新增 `build_trend_rows_keeps_low_series_variable_in_downtrend_fixture` 回归测试，锁定下跌夹具 low 序列不能塌成固定 `0.10` 楼板价。
- 调整 `build_trend_rows(...)` 的 low 下限生成逻辑，把固定下限改成随样本推进而变化的动态正数底，只吸收 `snapshot` 中对证券训练夹具稳定性有价值的最小修复。
- 在隔离 worktree 里先排查过 `snapshot` 与本地代码关系，确认真正应该回落到主工作区的是评分卡训练夹具修复，而不是整段 foundation 子树回退。

### 修改原因
- 用户要求继续处理 `snapshot` 分支并和本地代码合并，但实际排查发现 `snapshot` 的核心有效增量是 `security_scorecard_training` 夹具稳定化，而当前主工作区正好也复现了同一失败。
- 如果继续把 `snapshot` 的 foundation 子树整段对齐到主工作区，会覆盖你本地尚未提交的 foundation 演进接口，反而扩大合并面并制造拆分成本。

### 方案还差什么
- [ ] 这次只把证券训练夹具修复吸收到本地代码，还没有把隔离 worktree 里的探索性 foundation 对齐结果落回主工作区，也不建议直接落回。
- [ ] 当前没有顺手处理整仓全量测试，只验证了 `security_scorecard_training_cli` 这一条与本次合并目标直接相关的证券链路。

### 潜在问题
- [ ] `tests/security_scorecard_training_cli.rs` 当前仍带有历史乱码注释，这次没有顺手清理；如果后续继续维护这个测试文件，建议单独做一次 UTF-8 注释清整。
- [ ] 主工作区本身仍有大量既有脏改动和未跟踪文件，本次只在目标测试文件上做了最小吸收，未处理其他历史现场。

### 关闭项
- `security_scorecard_training` 下跌夹具退化红测已补齐：`tests/security_scorecard_training_cli.rs`
- `security_scorecard_training_generates_artifact_and_registers_refit_outputs` 已恢复通过：`tests/security_scorecard_training_cli.rs`
- `snapshot` 的证券侧最小有效修复已吸收到主工作区：`tests/security_scorecard_training_cli.rs`
## 2026-04-11
### 修改内容
- 新建 `docs/security-holding-ledger.md`，作为后续统一证券持仓台账，固定记录持仓计划、建仓事件、调仓事件与复盘编码。
- 新建 `docs/execution-notes-2026-04-11-security-holding-ledger.md`，记录本轮台账初始化范围、已登记持仓编码和未完成项。
- 新建 `docs/ai-handoff/AI_HANDOFF_SECURITY_HOLDING_LEDGER_2026-04-11.md`，给后续 AI 说明台账入口、落库位置和后续补录规则。

### 修改原因
- 用户要求新建一份固定文档，避免忘记当前保本优先版组合的持仓编码，并要求后续所有持仓继续写到同一文档里。
- 当前总交接手册和主文档已有未提交修改，如果直接继续往里追加，容易把不属于本轮的现场一起带上 GitHub；因此改为新建独立台账和独立交接说明。

### 方案还差什么
- [ ] 当前台账只登记了 `position_plan_ref`，真实下单后的 `build` 事件还需要等成交信息补录。
- [ ] 当前还没有把后续复盘模板扩成“组合级收益/回撤/执行偏差”汇总表，后面可以继续补。

### 潜在问题
- [ ] 如果后续真实成交日与计划日期不同，必须以真实成交日写 `position-adjustment:*:build:v1`，不能沿用计划日期。
- [ ] 当前台账是人工交接入口，不会自动从运行库回填事件状态；后续每次真实操作后仍需要补记。

### 关闭项
- 统一证券持仓台账已建立：`docs/security-holding-ledger.md`
- 本轮执行说明已建立：`docs/execution-notes-2026-04-11-security-holding-ledger.md`
- 本轮 AI 交接说明已建立：`docs/ai-handoff/AI_HANDOFF_SECURITY_HOLDING_LEDGER_2026-04-11.md`

## 2026-04-11 Task Journal
- 任务：foundation 主线 `方案A`，扩展 locator 结构诊断。
- 变更：新增 `RepositoryWeakLocatorReason::{SheetOnly, SingleCellOnly, AmbiguousKeyword, InvalidRangeFormat}`，并在 repository audit 中补齐最小结构判定逻辑。
- 测试：`cargo test --test repository_metadata_audit_unit -- --nocapture`；`cargo test --test repository_metadata_audit_unit --test metadata_validator_unit --test metadata_schema_registry_unit --test metadata_schema_versioning_unit --test metadata_migration_contract_unit --test knowledge_repository_unit --test knowledge_ingestion_unit --test knowledge_bundle_unit -- --nocapture`。
- 结论：foundation locator hygiene 已从基础长度检查推进到基础结构检查，当前下一步仍是继续扩细 evidence hygiene 诊断，不进入 migration executor。
## 2026-04-11
### 修改内容
- 更新 `skills/security-pm-assistant-v1/SKILL.md`，补入正式证券决策主链约束，明确正式口径只能沿 `security_decision_committee -> security_scorecard -> security_chair_resolution` 解释。
- 更新 `skills/security-decision-workbench-v1/SKILL.md`，补入正式评分卡与主席裁决规则，禁止再用手工平衡计分卡替代正式 scorecard。
- 更新 `skills/security-analysis-v1/SKILL.md`，补入研究链与正式决策链边界，明确研究链不能冒充正式投委会、正式评分卡或主席裁决。

### 修改原因
- 用户明确指出最近一次答复里把“手工平衡计分卡”和“正式评分卡”混在了一起，要求把正确口径直接写进 Skill，而不是停留在临时对话说明。
- 当前仓库内确实同时存在正式投委会、正式评分卡和主席裁决三条对象链，如果不把边界固化进 Skill，后续 AI 很容易再次绕开正式链直接口头下结论。

### 方案还差什么
- [ ] 当前只是把“正式口径”写进 Skill，还没有补一份适用于债券 ETF 的正式生产 scorecard model artifact。
- [ ] 如果后续要让债券 ETF 也能真正输出正式评分卡分数，还需要补训练 / 注册 / 发布对应资产类别的模型链。

### 潜在问题
- [ ] 当前仓库虽然有正式 `security_scorecard`，但对部分资产类别仍可能只能返回 `model_unavailable`；后续答复时必须继续如实披露，不能因为 Skill 已更新就默认认为模型已经就绪。
- [ ] 这轮是 Skill 文档改动，没有新增自动化测试；后续若继续扩规则，建议补一组 Skill 行为验收记录。

### 关闭项
- 统一入口已写入正式证券决策主链约束：`skills/security-pm-assistant-v1/SKILL.md`
- 投决会 Skill 已写入评分卡与主席裁决边界：`skills/security-decision-workbench-v1/SKILL.md`
- 分析 Skill 已写入研究链与正式决策链边界：`skills/security-analysis-v1/SKILL.md`

## 2026-04-11
### 修改内容
- 运行 `511360.SH / 511010.SH / 511060.SH` 的正式主席裁决链，先排除“低风报比参数导致风险闸门误伤”的干扰，再使用 `止损 1.0% / 目标 1.5% / 最小风报比 1.2` 重跑保本优先版正式复核。
- 追加 `docs/security-holding-ledger.md`，把 3 只债券 ETF 的委员会编码、评分卡编码、主席裁决编码、正式动作、风控状态和原始 JSON 路径落入统一持仓台账。

### 修改原因
- 用户要求把保本优先版组合真正跑一版，如果结果没问题就写进持仓报告；这次需要把“正式链实际跑出的结果”沉淀成可复盘文档，而不是只留在聊天记录里。
- 第一版运行使用了过低的目标收益参数，会被风报比闸门直接拦下，不能代表组合本体，因此需要先纠正参数口径再记录正式结论。

### 方案还差什么
- [ ] 当前债券 ETF 复核仍没有可用的正式评分卡模型 artifact，因此 `security_scorecard` 只能返回 `model_unavailable`。
- [ ] 当前财报/公告外部源对这 3 只 ETF 仍不可用，若后续要把债券 ETF 纳入正式可执行主策略，还需要补 ETF 适配的信息源或本地缓存链路。

### 潜在问题
- [ ] 当前正式复核虽然已落盘，但它基于 `market_symbol = 510300.SH` 与 `sector_symbol = 511010.SH` 的临时代理口径；后续若补出债券 ETF 专用 market/sector profile，结论可能变化。
- [ ] 当前主工作区 `cargo run` 重新编译会被无关的 foundation 缺文件阻断，这次运行依赖的是现有 `target\\debug\\excel_skill.exe`，后续若要重复运行，最好先收口那条无关构建问题。

### 关闭项
- 保本优先版 3 只债券 ETF 的正式主席裁决已运行并落盘：`D:\Rust\Excel_Skill\.excel_skill_runtime\holding_review_2026-04-11_defensive_rr15`
- 正式复核结果已写入统一持仓台账：`docs/security-holding-ledger.md`

## 2026-04-11
### 修改内容
- 新增正式总卡模块 `src/ops/security_master_scorecard.rs`，把 `security_decision_committee -> security_scorecard -> security_forward_outcome` 聚合成最小可用的 `security_master_scorecard` Tool。
- 新增 `tests/security_master_scorecard_cli.rs`，先锁住 catalog 发现能力，再锁住 CLI 返回 `committee_result + scorecard + master_scorecard` 三层正式对象与 6 个 horizon 的总卡合同。
- 更新 `src/ops/stock.rs`、`src/ops/mod.rs`、`src/tools/catalog.rs`、`src/tools/dispatcher.rs`、`src/tools/dispatcher/stock_ops.rs`，把 `security_master_scorecard` 接入 stock 主链的模块导出、tool catalog 与 CLI 路由。
- 最小恢复 `src/ops/foundation/knowledge_repository.rs` 与 `src/ops/foundation/knowledge_ingestion.rs`，并补上旧 JSONL metadata 到现行 `MetadataFieldValue` 的兼容映射，让本轮红测不再被无关 foundation 缺文件阻断。

### 修改原因
- 用户已确认方案 C，要先把“未来几日赚钱效益大总卡”做成正式 Tool，而不是继续停留在设计稿或手工分析说明。
- 当前工作区在真正进入 master_scorecard 红测前，会先被 foundation 缺失历史文件卡住；如果不先最小恢复这两个公共模块，本轮 TDD 无法推进到证券主链本身。

### 测试
- `cargo test --test security_master_scorecard_cli -- --nocapture`
- `cargo test --test security_forward_outcome_cli -- --nocapture`
- `cargo test --test security_scorecard_training_cli security_scorecard_training_generates_artifact_and_registers_refit_outputs -- --nocapture`

### 方案还差什么
- [ ] 当前 `security_master_scorecard` 仍是“历史回放型总卡”，不是完整训练版 master balance scorecard；还没有把多目标训练头、贡献回归重估和在线 artifact 消费全部接进去。
- [ ] 当前 `scorecard_status = model_unavailable` 时，总卡只能落为 `historical_replay_only`；如果后续要把总卡正式用于更多资产，还需要补对应资产的 scorecard model artifact。

### 潜在问题
- [ ] 当前总卡权重与子分数公式是透明固定规则，不是回归重估系数；后续如果训练版上线，需要注意不要让固定规则口径和训练版口径混淆。
- [ ] 这轮为了继续 TDD 最小恢复了 foundation 两个历史文件，但没有顺手推进 foundation 其他演进工作；后续如果 foundation 继续重构，记得把这两个恢复文件纳入统一治理。

### 关闭项
- `security_master_scorecard` 已成为正式可发现、可路由的 stock CLI Tool。
- 未来 5/10/20/30/60/180 天赚钱效益总卡已有最小正式对象，可输出 `horizon_breakdown / profitability_effectiveness_score / risk_resilience_score / path_quality_score / master_score / master_signal`。
## 2026-04-11
### 修改内容
- 更新 `src/ops/security_decision_submit_approval.rs`，把 `master_scorecard` 正式接入审批提交主链，并在 `approval_brief.master_scorecard_summary` 中持久化总卡摘要。
- 更新 `src/ops/security_master_scorecard.rs`，新增 `build_unavailable_security_master_scorecard_document(...)`，让最新实盘日缺少未来回放窗口时明确降级为 `replay_unavailable`，而不是直接中断审批流程。
- 扩展 `tests/security_decision_submit_approval_cli.rs`：
- 让 ready case 使用满足前后窗口的回放日期，锁住“正式总卡成功落盘”路径。
- 新增 `security_decision_submit_approval_degrades_master_scorecard_when_replay_window_is_unavailable`，锁住“实盘审批不被历史回放窗口阻塞”的降级路径。
- 清理 blocked case 对旧 `Long` 方向口径的陈旧断言，统一到当前正式桥接输出 `NoTrade`。
- 更新 `docs/security-holding-ledger.md`，补记审批链总卡的正式口径，明确“回放可用”和“实盘降级”两种状态及复盘关注字段。

### 修改原因
- 用户要求继续推进，并且希望正式总卡不只是单独 Tool 存在，而是进入正式审批链和持仓报告，后续可以复盘。
- 真实调试发现：`master_scorecard` 当前是历史回放型对象，直接硬接到 `submit_approval` 会在最新分析日因为缺少未来 `180` 天窗口而报错，进而误伤实盘审批流程；这必须显式收口。

### 方案还差什么
- [ ] 当前 `replay_unavailable` 仍是“显式不可回放”状态，不代表已经有了适用于实时审批的预测型总卡；后续如果要做实盘型赚钱效益总卡，还需要补单独的在线预测链。
- [ ] 当前 `decision_package` 还没有单独扩 schema 去显式挂载 `master_scorecard_ref`；本轮先收口在审批摘要与持仓报告，不扩包结构边界。

### 潜在问题
- [ ] `replay_unavailable` 分支当前使用 `master_score = 0.0` 和 `master_signal = unavailable` 作为显式哨兵值；后续如果前端直接按数值排序，必须额外读取 `aggregation_status`，不能把它误判成“最差但可比”的真实分数。
- [ ] ready case 现在依赖 `420` 天夹具和显式 `as_of_date = 2025-08-28` 来覆盖正路径；后续如果技术历史窗口或 horizon 口径调整，测试日期也要一起维护。

### 关闭项
- `security_decision_submit_approval` 已能在审批链中正式返回并落盘 `master_scorecard` 与 `approval_brief.master_scorecard_summary`。
- `submit_approval` 的回放可用路径与实盘降级路径都已有正式 CLI 回归。

## 2026-04-11
### 修改内容
- 更新 `docs/security-holding-ledger.md`，新增“2026-04-11 公开数据三线补录”章节，把 `511360.SH / 511010.SH / 511060.SH` 的公开市场数据补证、委员会补充意见、主席补充评估和对既有持仓计划的影响统一写回持仓台账。
- 在同一章节中补充了 `2026-04-10` 债市宏观环境、3 只债券 ETF 的最新公开规模/资金流/流动性信息，以及未来 `5个交易日 / 2-4周` 的公开数据推演区间，供后续复盘对照。

### 修改原因
- 用户要求把“数据线、投决会线、主席评估”三条链都落入持仓计划文档，避免后续复盘时只剩正式链 JSON，而缺少公开市场环境与管理补录判断。
- 当前正式链对这 3 只债券 ETF 的结论仍为 `avoid`，但公开数据已经能支持“观察优先级”的进一步细化，因此需要把这层补录和正式链明确并列保存，防止口径混淆。

### 测试
- 本轮为文档与研究补录更新，未新增代码，也未运行新的 Rust 自动化测试。
- 已人工核对新增文档段落的结构、ETF 顺序、日期口径和外部数据来源链接。

### 方案还差什么
- [ ] 当前三线补录仍是“公开数据补证 + 管理判断”，不是新的正式 Tool 产物；如果后续要让这层内容进入正式审批对象，还需要扩正式 `committee/brief/package` 持久化结构。
- [ ] 3 只债券 ETF 目前仍缺债券 ETF 专用正式评分卡模型，正式链仍会返回 `model_unavailable`，后续若要放行执行，需要先补模型与证据包。

### 潜在问题
- [ ] `511060.SH` 的最新可公开获取数据日期早于另外两只 ETF，数据完备度相对偏弱；后续如果能补到更近的溢价率或成交额数据，观察优先级可能需要重估。
- [ ] 本轮“未来 5 个交易日 / 2-4 周”区间属于基于公开数据和宏观利率环境的管理推演，不应被误读为正式评分卡或收益承诺。

### 关闭项
- 3 只债券 ETF 的公开数据补录、委员会补充意见和主席补充评估已写入统一持仓台账：`docs/security-holding-ledger.md`
- 后续复盘时，已可同时沿“正式链结论”和“公开数据三线补录”两条视角回看本次保本优先版组合。

## 2026-04-11
### 修改内容
- 更新 `skills/security-pm-assistant-v1/SKILL.md`，新增“训练优先治理规则”，明确所有证券结论都要先判断是否存在训练支撑、可披露拟合摘要和足够样本，禁止把少量事实直接包装成正式可执行结论。
- 更新 `skills/security-decision-workbench-v1/SKILL.md`，新增“训练优先规则”和“训练信息披露要求”，明确投决、调仓、组合建议必须先检查训练支撑，并在引用训练结论时同时披露样本与拟合摘要。
- 更新 `skills/security-analysis-v1/SKILL.md`，新增“训练优先边界”，明确研究链在没有训练支撑时只能输出研究观察、可能性推演和证据缺口，不能越权下负责建议。
- 新增 `docs/security-training-governance-rules.md`，把“训练优先、拟合可披露、训练持续回算重估、无训练不得冒充负责结论”的治理规则正式写成项目文档。

### 修改原因
- 用户明确指出：所有证券内容都应优先考虑通过训练得出结论，训练会随着样本和回算积累越来越准，不能只看一点点内容就快速给出负责建议。
- 当前项目已经具备最小训练链和正式评分卡治理对象，但技能层仍缺少“训练不足时必须降级”的硬规则；如果不把这条规则写进 Skill，后续 AI 仍可能重复出现“快结论越权”的问题。

### 测试
- 本轮为 Skill 与治理文档更新，未新增 Rust 代码，也未运行新的自动化测试。
- 已人工核对 3 个 Skill 和 1 份治理文档中的关键约束是否一致，确保“训练支撑的正式结论 / 正式链已运行但训练不可用的治理结论 / 仅研究观察”三层口径统一。

### 方案还差什么
- [ ] 当前只是把“训练优先”写成 Skill 与文档硬规则，还没有把这些规则完全编码成 CLI 层或运行时的强制校验。
- [ ] 当前训练摘要仍偏最小化，只有 `sample_count / train-valid-test accuracy / positive_rate`；后续仍需要把 `AUC / KS / OOS / 分 horizon 表现` 这些指标真正落成代码输出。

### 潜在问题
- [ ] 这轮规则更新后，如果未来答复仍然没有显式说明“训练支撑是否存在”，就属于执行问题而不是规则缺失；后续需要通过对话验收来确保 Skill 真正生效。
- [ ] 当前项目仍存在“正式链可以运行，但 scorecard = model_unavailable”的场景；后续对外表述时必须继续把它明确归类为“治理结论”，不能误叫成“训练放行结论”。

### 关闭项
- “训练优先、无训练不许冒充负责结论”的证券治理规则已写入 3 个核心证券 Skill。
- 项目内已经有一份正式书面规则文档：`docs/security-training-governance-rules.md`。

## 2026-04-11
### 修改内容
- 更新 `src/ops/security_scorecard.rs`，把 `risk_note_count` 正式补进 scorecard 原始特征快照，确保训练模型要求的风险笔记特征不会在运行时被静默漏掉。
- 更新 `tests/security_decision_submit_approval_cli.rs`，把 ready case 的正式断言升级为：`scorecard.score_status = ready` 且 `master_scorecard.aggregation_status = replay_with_quant_context`，不再沿用旧的降级口径。

### 修改原因
- 用户要求先把“无训练支撑时不得产出高确定性建议”的运行时规则彻底收口，然后再开始训练数据；这意味着 happy path 也必须真实建立在完整训练特征之上，不能只让审批闸门放宽来假通过。
- 实际排查发现 `security_decision_submit_approval_writes_runtime_files_for_ready_case` 失败的根因不是审批规则，而是 ready 模型 fixture 需要的 `risk_note_count` 没被写入 scorecard 原始特征快照，导致评分卡停在 `feature_incomplete`。

### 测试
- `cargo test --test security_decision_submit_approval_cli security_decision_submit_approval_writes_runtime_files_for_ready_case -- --nocapture`
- `cargo test --test security_decision_submit_approval_cli -- --nocapture`
- `cargo test --test security_chair_resolution_cli -- --nocapture`

### 方案还差什么
- [ ] 当前只完成了“训练支撑缺失时运行时降级”和“happy path 需要完整训练特征”的治理闭环，还没有开始扩真实证券训练数据集与训练指标面板。
- [ ] 训练链当前仍主要覆盖最小 `direction_head`，如果要把“大平衡计分卡”的未来赚钱效益做成真正可训练对象，还需要继续扩 horizon 标签、样本装配和拟合度输出。

### 潜在问题
- [ ] ready case 现在依赖现成测试 artifact 与 `risk_note_count` 一起构成完整特征命中；后续如果训练特征集合继续扩展，相关 ready fixture 和回归断言还需要同步维护。
- [ ] `security_scorecard` 现在会把训练输入缺项直接反映成 `feature_incomplete`，这是正确治理口径；但后续如果前端或 Skill 没有显式展示这个状态，仍可能把“模型存在但特征不全”误读成“模型不可用”。

### 关闭项
- `submit_approval` 的 ready path 已恢复为真实训练支撑路径，不再被遗漏特征误伤成 `request_more_evidence`。
- `chair` 与 `submit_approval` 的运行时训练闸门已完成交叉验证，当前可以进入训练数据阶段。

## 2026-04-11
### 修改内容
- 为证券分析主链新增 EastMoney 资金面最小增强路径，补入 `capital_flow_context`，并接入 `security_analysis_contextual -> security_analysis_fullstack` 返回链。
- 新增 `EastMoneyBudgetStore` 与 `EastMoneyCacheStore`，为免费额度场景提供每日预算控制与本地缓存能力。
- 新增 `providers/eastmoney` 最小 provider 层与 `eastmoney_enrichment` 聚合层，先把资金面能力从业务文件中收口成独立边界。
- 新增 `tests/eastmoney_enrichment_cli.rs`，并扩展 `security_analysis_fullstack_cli` 覆盖资金面可用与预算耗尽降级场景。
- 同步 `.env.example` 与设计/实施文档，记录本轮 EastMoney 方案 2 落地路径。
### 修改原因
- 用户确认采用“方案 2：标准增强版”，目标是只补现有证券链缺少的东财增量数据，而不是替换已有行情历史主链。
- 免费额度受限，必须优先把预算池、缓存池和降级语义立起来，否则接入后会很快失去可用性。
- 现有 `security_analysis_fullstack` 直接拼东财请求，边界过于耦合，先抽出 provider/runtime/enrichment 能降低后续继续接事件面时的改动风险。
### 方案还差什么
- [ ] 当前只完成了资金面最小闭环，尚未把公告/资讯事件面改造成同样带预算池与缓存池的 provider/enrichment 路径。
- [ ] 当前尚未把资金面补充正式纳入 `security_decision_evidence_bundle` 的证据质量与风险摘要字段。
- [ ] 当前尚未实现多 Token 池；如果后续要做，需要先确认合规边界与轮询策略。
### 潜在问题
- [ ] 资金面默认端点当前通过环境变量注入，若生产环境未配置 `EXCEL_SKILL_EASTMONEY_CAPITAL_FLOW_URL_BASE`，将返回 `unavailable` 而不是自动取到真实远端数据。
- [ ] 预算池当前按本地日期文件计数，适合单机单 runtime，但若后续引入多进程共享或多 Token 池，需要升级为更严格的并发一致性方案。
- [ ] 现有 README / task_plan / findings / progress 的历史内容存在编码噪音，本轮新增设计文档已落盘，但这些历史文件若要继续持续维护，后续最好先统一编码。
### 关闭项
- EastMoney 资金面预算池与缓存池的最小运行闭环已完成，并已有自动化测试覆盖预算耗尽与缓存命中。
- `security_analysis_fullstack` 已能稳定返回 `technical_context.capital_flow_context`，下游 `security_decision_evidence_bundle_cli` 回归通过。
## 2026-04-11 14:28 Task - bond ETF real training probe succeeded

- Scope:
  - Ran a real `security_scorecard_training` probe against the local bond ETF history DB at `D:\Rust\Excel_Skill\.excel_skill_runtime\holding_review_2026-04-11_defensive_rr15\stock_history.db`.
  - Focused symbols: `511360.SH`, `511010.SH`, `511060.SH`; market proxy `510300.SH`; sector proxy `511010.SH`.
- Key finding:
  - The earlier failed windows were mainly caused by the combined constraint of `200` history rows and a very narrow negative-label pocket on `511010.SH`.
  - A viable governed split was confirmed with:
    - `train_range = 2026-01-22..2026-02-13`
    - `valid_range = 2026-02-24..2026-03-03`
    - `test_range = 2026-03-04..2026-03-11`
    - `train/valid/test samples_per_symbol = 4/2/2`
- Training outcome:
  - Runtime root: `D:\Rust\Excel_Skill\.excel_skill_runtime\bond_etf_training_probe_20260411_b`
  - Model id: `a_share_bond_etf_10d_direction_head`
  - Model version: `candidate_2026_04_11T14_20_00_08_00`
  - Registry ref: `registry-a_share_bond_etf_10d_direction_head-candidate_2026_04_11T14_20_00_08_00-10d-direction_head`
- Fit panel summary:
  - `sample_count = 24`
  - `train accuracy = 0.9167`, `train auc = 0.7727`, `train ks = 0.8182`
  - `valid accuracy = 0.6667`, `valid auc = 0.7500`, `valid ks = 0.7500`
  - `test accuracy = 0.8333`, `test auc = 0.8000`, `test ks = 0.8000`
- Remaining risk:
  - The label mix is still highly imbalanced toward positive returns, especially because `511360.SH` and most of `511060.SH` windows stay positive in the current history slice.
  - `hit_upside_first_rate` and `hit_stop_first_rate` remain `0.0`, which means the current bond ETF history is still weak for path-event heads and should not be over-claimed as a full production-grade training set.

## 2026-04-11 15:30 Task - bond ETF history backfill and long-horizon probability refresh

- Scope:
  - Backfilled the current bond ETF training DB through the existing `sync_stock_price_history` mainline instead of creating a new history pipeline.
  - Updated the holding ledger with the refreshed multi-horizon training probabilities and the root-cause note for the identical `511010.SH / 511060.SH` outputs.
- Backfill actions:
  - Target DB: `D:\Rust\Excel_Skill\.excel_skill_runtime\holding_review_2026-04-11_defensive_rr15\stock_history.db`
  - Synced `511360.SH`, `511010.SH`, `511060.SH`, and `510300.SH` for `2024-01-01..2025-03-31`.
  - Provider used: `tencent`
  - Resulting history coverage for all four symbols: `2024-01-02..2026-04-10`, `548` rows each.
- Root-cause finding:
  - The earlier identical probabilities for `511010.SH` and `511060.SH` were not caused by mixed history rows.
  - Their `2026-04-10` `raw_feature_snapshot` values are identical on the currently consumed scorecard features:
    - `integrated_stance = technical_only`
    - `technical_alignment = mixed`
    - `data_gap_count = 2`
    - `risk_note_count = 8`
  - With the current four-feature scorecard, identical feature bins naturally produce identical probabilities.
- Training outcome after backfill:
  - Long horizons became trainable on the same governed training path.
  - Refreshed candidate probabilities for `2026-04-10`:
    - `511360.SH`: `3d=0.9996`, `6d=0.4815`, `10d=0.9995`, `30d=0.9999`, `60d=0.7885`, `180d=0.9809`
    - `511010.SH`: `3d=0.8712`, `6d=0.9975`, `10d=0.5998`, `30d=0.9995`, `60d=0.9992`, `180d=0.9997`
    - `511060.SH`: `3d=0.8712`, `6d=0.9975`, `10d=0.5998`, `30d=0.9995`, `60d=0.9992`, `180d=0.9997`
- Fit-quality caution:
  - `30d` is now trainable, but the current candidate still shows weak out-of-sample behavior (`valid_auc = 0.3333`, `test_auc = 0.0000`), so the high long-horizon probabilities must not be over-read as high-confidence signals.
  - The refreshed probabilities are suitable for review and ledgering, not yet for replacing the chair/approval execution gate.

## 2026-04-11 16:40 Task - split ETF scorecard family and add invalid binding guard

### 修改内容
- Added ETF-specific differentiating raw features to the unified evidence seed in `src/ops/security_decision_evidence_bundle.rs`.
- Switched scorecard training to choose feature configs by `instrument_scope`, keeping equity stable and expanding ETF with numeric technical differentiators in `src/ops/security_scorecard_training.rs`.
- Reused the unified evidence seed inside runtime scorecard snapshot construction and added an ETF/equity model-family guard in `src/ops/security_scorecard.rs`.
- Added focused unit tests for ETF training feature configs and ETF runtime invalid binding behavior.
- Added integration regressions proving ETF-specific fields are exposed by `security_feature_snapshot` and that `security_decision_submit_approval` downgrades an ETF symbol when bound to an equity artifact.
- Wrote design and implementation plan files:
  - `docs/plans/2026-04-11-etf-separate-scorecard-model-design.md`
  - `docs/plans/2026-04-11-etf-separate-scorecard-model-plan.md`

### 修改原因
- The previous four-feature scorecard family could collapse different ETF symbols into identical probabilities, which is mathematically explainable but operationally invalid.
- The user explicitly required ETF and equity to become separate model families, while keeping the governance chain shared.

### 方案还差什么
- [ ] Stage 2 still needs ETF external context features such as gold, overseas market, and futures linkage, but this round intentionally stayed inside the local-history pipeline.
- [ ] The current ETF invalid guard only blocks wrong model-family binding; it does not yet compare peer ETF outputs inside the same batch for deeper cross-sectional diagnostics.

### 潜在问题
- [ ] Bond ETF training samples are still relatively small, so even after splitting the feature family, long-horizon fit quality can remain unstable until more ETF history and richer external context are added.
- [ ] If a future ETF artifact includes the ETF feature names but poor bins, the current guard will treat the family as structurally valid; deeper artifact-quality checks may still be needed later.

### 关闭项
- ETF raw feature seeds now expose numeric differentiators such as `close_vs_sma50`, `volume_ratio_20`, `rsrs_zscore_18_60`, `support_gap_pct_20`, and `resistance_gap_pct_20`.
- ETF training no longer shares the same coarse-only feature config as equity.
- The approval chain now downgrades an ETF symbol to `cross_section_invalid` when it is bound to the existing equity scorecard artifact.

## 2026-04-11 18:10 Task - add governed training readiness assessment panel

### 修改内容
- Added `readiness_assessment` to `security_scorecard_training` metrics output in `src/ops/security_scorecard_training.rs`.
- Formalized four governed readiness fields:
  - `minimum_sample_status`
  - `class_balance_status`
  - `path_event_coverage_status`
  - `production_readiness`
- Added structured readiness notes so downstream governance can explain why a run stays research-only.
- Extended `tests/security_scorecard_training_cli.rs` to lock the readiness panel through the main CLI contract.

### 修改原因
- The current training chain could expose fit metrics, but it still lacked an explicit machine-readable statement of whether the sample pool is production-leaning or still only suitable for research governance.
- Scheme B needs the system to surface sample sufficiency and path-event sparsity before we keep expanding asset pools and model heads.

### 方案还差什么
- [ ] The readiness panel still uses rule-based thresholds, not learned promotion thresholds from shadow performance.
- [ ] `production_readiness` is still a governance summary and has not yet been connected to automatic `candidate -> shadow -> champion` promotion.

### 潜在问题
- [ ] The current sample threshold is fixed at `12/6/6` for `train/valid/test`, so future larger training plans may need a more adaptive readiness rule.
- [ ] Path-event coverage is currently judged from the train split only; later stages may need split-by-split and horizon-by-horizon coverage gates.

### 关闭项
- Training runs now expose an explicit governed answer when the sample pool is still only `research_candidate_only`.
- The main training CLI regression now verifies that sparse path events are surfaced as a first-class readiness limitation instead of being left implicit.

## 2026-04-11 19:05 Task - split ETF model binding into four governed subscopes

### 修改内容
- Added ETF subscope routing helpers in `src/ops/security_decision_evidence_bundle.rs`, including `is_etf_symbol(...)` and `resolve_etf_subscope(...)`, and propagated `market_profile`, `sector_profile`, and `instrument_subscope` into the raw evidence seed.
- Extended `SecurityScorecardTrainingRequest` in `src/ops/security_scorecard_training.rs` with `instrument_subscope`, enforced ETF subscope resolution during request validation, and wrote subscoped ETF model ids such as `a_share_etf_treasury_etf_10d_direction_head` into scorecard artifacts.
- Extended runtime scorecard binding in `src/ops/security_scorecard.rs` with `instrument_subscope` and downgraded ETF scorecards to `cross_section_invalid` when the runtime ETF subscope and the bound artifact subscope do not match.
- Added focused red-green coverage:
  - `etf_training_artifact_model_id_carries_treasury_subscope`
  - `etf_runtime_guard_rejects_wrong_etf_subscope_binding`
  - `security_decision_submit_approval_rejects_wrong_etf_subscope_binding_for_bond_etf`
- Re-ran adjacent regressions proving the older ETF family guard and the training CLI still pass after the subscope split.

### 修改原因
- The previous ETF-only family split was still too coarse for production-leaning governance because bond ETF, equity ETF, gold ETF, and cross-border ETF can share the same top-level `ETF` scope while depending on very different market drivers.
- Scheme C requires the system to stop accepting structurally wrong ETF artifacts before we invest in richer external factors and per-pool champion training.

### 方案还差什么
- [ ] Gold ETF and cross-border ETF still rely on symbol/profile heuristics for subscope routing; this round intentionally did not introduce a larger asset taxonomy service.
- [ ] The training chain now separates ETF subscopes structurally, but we still need true per-subscope datasets and candidate/champion artifacts before these pools can be treated as production-ready.

### 潜在问题
- [ ] `resolve_etf_subscope(...)` currently treats `511010` and `511060` as the same `treasury_etf` pool; if later governance wants local-government bond ETFs separated from treasury ETFs, we will need a fifth sub-pool and migration logic.
- [ ] A malformed artifact could still claim the correct `instrument_subscope` while containing weak bins or poor fit quality, so artifact-quality promotion gates remain necessary on top of the new structural guard.

### 关闭项
- ETF artifacts now carry an explicit governed subscope instead of only the broad `ETF` family.
- Runtime scorecards and approval submission now reject ETF artifacts that belong to the wrong ETF sub-pool.
- Adjacent regressions for ETF family guards and training CLI remained green after the four-subscope split.

## 2026-04-11 19:50 Task - add subscope-specific minimum factor families for ETF pools

### 修改内容
- Added governed ETF minimum factor families in `src/ops/security_decision_evidence_bundle.rs` for:
  - `equity_etf`
  - `treasury_etf`
  - `gold_etf`
  - `cross_border_etf`
- Added `required_etf_feature_family(...)` as the shared lookup so training and runtime consume the same ETF pool structure.
- Updated `src/ops/security_scorecard_training.rs` so ETF training configs now branch by `instrument_subscope` instead of appending one generic ETF differentiator list.
- Updated `src/ops/security_scorecard.rs` so runtime ETF guard now rejects artifacts that declare the correct ETF sub-pool but still omit that sub-pool's required minimum factor family.
- Added focused red-green coverage:
  - `etf_training_feature_config_separates_treasury_and_gold_subscopes`
  - `etf_runtime_guard_rejects_treasury_binding_without_treasury_feature_family`
  - `security_decision_submit_approval_rejects_treasury_etf_binding_without_treasury_feature_family`

### 修改原因
- The previous ETF subscope split still left all ETF pools sharing one generic differentiating feature family, which meant a treasury ETF artifact could keep passing structurally with an equity-style factor entrance.
- Scheme B needs ETF pool identity and ETF pool factor-family requirements to become the same governed contract before we add richer external market drivers.

### 方案还差什么
- [ ] Gold ETF and cross-border ETF still use local technical proxies only; this round deliberately did not connect gold price, FX, overseas index, or futures proxies yet.
- [ ] The current ETF subscope families are minimum entrances, not full production factor sets; later stages still need per-pool external factors and larger training pools.

### 潜在问题
- [ ] Some ETF pools currently share overlapping technical factors by design, so structural separation is improved but not yet sufficient for production-grade cross-sectional ranking.
- [ ] If a future artifact carries the required minimum ETF sub-pool family but has weak fit quality, runtime will treat it as structurally valid until promotion gates and fit-quality gates become stricter.

### 关闭项
- ETF training now differentiates treasury ETF and gold ETF at the minimum factor-family level instead of only by model id.
- Runtime scorecard and approval submission now reject a treasury ETF artifact that lacks the treasury ETF minimum factor family even when its `instrument_subscope` label says `treasury_etf`.
- Adjacent regressions for ETF subscope identity, ETF family mismatch, and the main training CLI remained green after the minimum factor-family split.

## 2026-04-11 20:35 Task - add placeholder external proxy contracts for governed ETF sub-pools

### 修改内容
- Added placeholder ETF external proxy contract fields in `src/ops/security_decision_evidence_bundle.rs` for:
  - `treasury_etf`
  - `gold_etf`
  - `cross_border_etf`
  - `equity_etf`
- Extended each governed ETF minimum factor family so runtime artifact validation now requires the sub-pool-specific proxy-contract fields in addition to the existing local technical entrance.
- Added raw feature seed emission for ETF proxy-contract statuses:
  - active ETF sub-pool fields emit `placeholder_unbound`
  - inactive ETF sub-pool fields emit `not_applicable`
- Updated `src/ops/security_scorecard_training.rs` so ETF `_proxy_status` features are treated as categorical inputs instead of numeric signals.
- Repaired and greened the approval regression in `tests/security_decision_submit_approval_cli.rs` so `gold_etf` artifacts that omit the gold proxy-contract family now fail with `cross_section_invalid`.
- Added supporting design/plan docs:
  - `docs/plans/2026-04-11-etf-external-proxy-contracts-design.md`
  - `docs/plans/2026-04-11-etf-external-proxy-contracts-plan.md`

### 修改原因
- The ETF pool split was still missing stable contract names for future external drivers, which meant later integration of rates, FX, gold, overseas, and ETF-flow data would have required another schema change.
- Scheme C needs us to freeze ETF proxy contract fields now so training, runtime guardrails, and approval can all reject structurally incomplete ETF artifacts before real external feeds arrive.

### 方案还差什么
- [ ] These proxy fields are placeholders only; real treasury-rate, funding, gold, FX, overseas-market, and ETF-flow data ingestion is still a later phase.
- [ ] Runtime currently validates structural presence of the proxy-contract family, not whether the external proxy values are fresh, complete, or economically useful.

### 潜在问题
- [ ] A future artifact can still carry the full ETF proxy-contract family but remain weak in fit quality, so production promotion gates are still required on top of this structural governance.
- [ ] `gold_etf` approval regression currently uses a synthetic gold-sector proxy symbol only to pass the existing contextual analysis precondition; later gold ETF knowledge should replace this with a governed peer-proxy mapping.

### 关闭项
- Four governed ETF sub-pools now expose stable placeholder external proxy contracts in the raw feature schema.
- ETF training now classifies `_proxy_status` fields as categorical features instead of misreading them as numeric indicators.
- Approval flow now rejects a `gold_etf` artifact that omits the required gold proxy-contract family.
- Focused and adjacent regressions remained green after the ETF proxy-contract rollout.

## 2026-04-11 21:05 Task - wire manual gold ETF proxy inputs into the governed decision chain

### 修改内容
- Added `SecurityExternalProxyInputs` to the governed gold ETF path so manual proxy values can now travel through:
  - `security_feature_snapshot`
  - `security_forward_outcome`
  - `security_decision_committee`
  - `security_scorecard_training`
  - `security_master_scorecard`
  - `security_chair_resolution`
  - `security_decision_submit_approval`
- Extended `src/ops/security_decision_evidence_bundle.rs` so `gold_etf` raw features now freeze:
  - `gold_spot_proxy_status`
  - `gold_spot_proxy_return_5d`
  - `usd_index_proxy_status`
  - `usd_index_proxy_return_5d`
  - `real_rate_proxy_status`
  - `real_rate_proxy_delta_bp_5d`
- Kept the evidence hash deterministic by hashing manual floating-point proxy values via `to_bits()`.
- Refactored gold numeric proxy emission to reuse the governed field registry instead of repeating three separate inserts.
- Added supporting design/plan docs:
  - `docs/plans/2026-04-11-gold-etf-manual-proxy-input-design.md`
  - `docs/plans/2026-04-11-gold-etf-manual-proxy-input-plan.md`

### 修改原因
- Scheme B needed a formal way to feed live gold ETF proxy values into the scorecard and approval chain without bypassing the existing audited request objects.
- The placeholder contract rollout was structurally ready, but still could not preserve manually supplied gold, USD, and real-rate proxy values as governed raw features.

### 方案还差什么
- [ ] Real external proxy ingestion is still missing for gold ETF; this task only formalizes manual governed inputs.
- [ ] Historical proxy backfill and refit are still required before these inputs can support production-grade gold ETF modeling.

### 潜在问题
- [ ] Manual proxy inputs can still be economically weak or stale even when the structure is complete, so promotion gates must continue to check fit quality separately.
- [ ] Current grouped-feature exposure only proves transport and freezing; later consumers may still need stronger semantic grouping once gold-specific modeling deepens.

### 关闭项
- Manual gold ETF proxy inputs now survive the formal feature-freeze path instead of staying outside the governed decision chain.
- Gold ETF scorecard and approval flows can now consume manually supplied proxy values without falling back to `placeholder_unbound`.
- Focused and adjacent regressions remained green after the manual gold proxy-input rollout.

## 2026-04-11 21:40 Task - wire manual treasury ETF proxy inputs into the governed decision chain

### 修改内容
- Extended `src/ops/security_decision_evidence_bundle.rs` so `SecurityExternalProxyInputs` now carries treasury ETF manual proxy fields:
  - `yield_curve_proxy_status`
  - `yield_curve_slope_delta_bp_5d`
  - `funding_liquidity_proxy_status`
  - `funding_liquidity_spread_delta_bp_5d`
- Expanded the governed treasury ETF feature family to require both treasury proxy statuses and treasury numeric proxy fields.
- Added treasury ETF numeric raw-feature freezing and grouped `X` feature exposure in:
  - `src/ops/security_decision_evidence_bundle.rs`
  - `src/ops/security_feature_snapshot.rs`
- Updated proxy-status resolution so an active treasury ETF request with numeric treasury proxy inputs can resolve to `manual_bound` instead of staying `placeholder_unbound`.
- Extended evidence hashing so treasury ETF manual proxy values participate in the deterministic evidence hash.
- Added treasury ETF red-to-green regressions in:
  - `tests/security_feature_snapshot_cli.rs`
  - `tests/security_decision_submit_approval_cli.rs`
  - treasury ETF feature-family and training config assertions inside the relevant unit tests
- Added supporting design/plan docs:
  - `docs/plans/2026-04-11-treasury-etf-manual-proxy-input-design.md`
  - `docs/plans/2026-04-11-treasury-etf-manual-proxy-input-plan.md`

### 修改原因
- Scheme B required treasury ETF to stop relying on placeholder-only proxy contracts and begin accepting governed manual macro proxy inputs, just like the gold ETF path.
- Without treasury numeric proxy fields, scorecard and approval consumers could not distinguish a real manual treasury macro view from an unbound placeholder state.

### 方案还差什么
- [ ] Real treasury external proxy ingestion is still missing; this task only formalizes manually supplied treasury macro inputs.
- [ ] Historical treasury proxy backfill and refit are still required before treasury ETF modeling can move beyond research-grade governance.

### 潜在问题
- [ ] Manual treasury proxy values can be stale or economically weak even when the structure is correct, so model promotion must still depend on fit and freshness governance later.
- [ ] Grouped `X` exposure proves transport and freezing, but treasury-specific grouping semantics may need refinement once richer rate and funding proxies are added.

### 关闭项
- Treasury ETF manual proxy inputs now survive the governed feature-freeze path.
- Treasury ETF scorecard and approval flows can consume manual treasury proxy inputs without falling back to `placeholder_unbound`.
- Focused and adjacent treasury ETF regressions remained green after the manual proxy-input rollout.

## 2026-04-11 22:18 Task - wire manual cross-border ETF proxy inputs into the governed decision chain

### 修改内容
- Extended `src/ops/security_decision_evidence_bundle.rs` so `SecurityExternalProxyInputs` now carries governed manual cross-border ETF proxy fields:
  - `fx_proxy_status`
  - `fx_return_5d`
  - `overseas_market_proxy_status`
  - `overseas_market_return_5d`
  - `market_session_gap_status`
  - `market_session_gap_days`
- Expanded the governed `cross_border_etf` feature family and proxy field registries so runtime scorecard, training, and approval share the same audited contract.
- Added cross-border ETF numeric raw-feature freezing and grouped `X` feature exposure in:
  - `src/ops/security_decision_evidence_bundle.rs`
  - `src/ops/security_feature_snapshot.rs`
- Updated cross-border proxy-status resolution so active-pool numeric-only inputs can resolve to `manual_bound` instead of remaining `placeholder_unbound`.
- Extended evidence hashing so FX, overseas market, and market-session-gap inputs are part of the deterministic evidence hash.
- Narrowed ETF training config handling in `src/ops/security_scorecard_training.rs` so `market_session_gap_status` is treated as a categorical contract field instead of a numeric feature.
- Added supporting design/plan docs:
  - `docs/plans/2026-04-11-cross-border-etf-manual-proxy-input-design.md`
  - `docs/plans/2026-04-11-cross-border-etf-manual-proxy-input-plan.md`

### 修改原因
- Scheme B required `cross_border_etf` to stop relying on placeholder-only overseas proxy contracts and begin accepting governed manual FX, overseas-market, and session-gap inputs.
- Without these fields, Nikkei, QDII, and other overseas-linked ETF reviews could not freeze manual cross-border context into the same audited decision package as other ETF pools.

### 方案还差什么
- [ ] Real FX and overseas market feed ingestion is still missing; this task only formalizes manually supplied cross-border proxy inputs.
- [ ] Historical cross-border proxy backfill and model refit are still required before `cross_border_etf` can move beyond research-grade governance.

### 潜在问题
- [ ] Manual cross-border proxy values can become stale or misaligned with overseas close timing, so later freshness governance still needs to be enforced.
- [ ] Session-gap support is currently a scalar contract only; richer exchange-calendar semantics may be required when cross-border ETF pools expand beyond the current QDII/Nikkei-style assumptions.

### 关闭项
- Cross-border ETF manual proxy inputs now survive the governed feature-freeze path.
- Cross-border ETF feature snapshot and approval flows can consume FX, overseas market, and session-gap proxy inputs without falling back to `placeholder_unbound`.
- Focused and adjacent cross-border ETF regressions remained green after the manual proxy-input rollout.

## 2026-04-11 22:42 Task - wire manual equity ETF proxy inputs into the governed decision chain

### 修改内容
- Extended `src/ops/security_decision_evidence_bundle.rs` so `SecurityExternalProxyInputs` now carries governed manual equity ETF proxy fields:
  - `etf_fund_flow_proxy_status`
  - `etf_fund_flow_5d`
  - `premium_discount_proxy_status`
  - `premium_discount_pct`
  - `benchmark_relative_strength_status`
  - `benchmark_relative_return_5d`
- Expanded the governed `equity_etf` feature family and proxy field registries so runtime scorecard, training, and approval share the same audited contract.
- Added equity ETF numeric raw-feature freezing and grouped `X` feature exposure in:
  - `src/ops/security_decision_evidence_bundle.rs`
  - `src/ops/security_feature_snapshot.rs`
- Updated equity ETF proxy-status resolution so active-pool numeric-only inputs can resolve to `manual_bound` instead of remaining `placeholder_unbound`.
- Extended evidence hashing so ETF fund-flow, premium-discount, and benchmark-relative inputs are part of the deterministic evidence hash.
- Narrowed ETF training config handling in `src/ops/security_scorecard_training.rs` so `benchmark_relative_strength_status` is treated as a categorical contract field instead of a numeric feature.
- Added supporting design/plan docs:
  - `docs/plans/2026-04-11-equity-etf-manual-proxy-input-design.md`
  - `docs/plans/2026-04-11-equity-etf-manual-proxy-input-plan.md`

### 修改原因
- Scheme B required `equity_etf` to stop relying on placeholder-only ETF market-structure contracts and begin accepting governed manual fund-flow, premium-discount, and benchmark-relative inputs.
- Without these fields, sector ETF and broad-index ETF reviews could not freeze manual ETF structure context into the same audited decision package as other ETF pools.

### 方案还差什么
- [ ] Real ETF fund-flow and premium-discount ingestion is still missing; this task only formalizes manually supplied equity ETF proxy inputs.
- [ ] Historical equity ETF proxy backfill and model refit are still required before `equity_etf` can move beyond research-grade governance.

### 潜在问题
- [ ] Manual equity ETF proxy values can become stale or inconsistent with intraday ETF structure, so later freshness governance still needs to be enforced.
- [ ] `benchmark_relative_strength_status` is currently a contract-level flag only; richer benchmark classification semantics may be required when equity ETF pools are split further.

### 关闭项
- Equity ETF manual proxy inputs now survive the governed feature-freeze path.
- Equity ETF feature snapshot and approval flows can consume fund-flow, premium-discount, and benchmark-relative proxy inputs without falling back to `placeholder_unbound`.
- Focused and adjacent equity ETF regressions remained green after the manual proxy-input rollout.

## 2026-04-11 23:58 Task - land P3 multi-head scorecard flow through master scorecard and chair resolution

### 修改内容
- Extended `src/ops/security_scorecard_training.rs` so governed training now supports multi-head target modes instead of a direction-only path:
  - classification heads: `direction_head`, `upside_first_head`, `stop_first_head`
  - regression heads: `return_head`, `drawdown_head`, `path_quality_head`
- Added regression artifact support in `src/ops/security_scorecard.rs`:
  - `target_head`
  - `prediction_mode`
  - `prediction_baseline`
  - `predicted_value`
- Exposed governed regression prediction helpers so downstream scorecard consumers can load and score trained multi-head artifacts without forking the scorecard contract.
- Extended `src/ops/security_master_scorecard.rs` so master scorecard requests can consume:
  - `return_head_model_path`
  - `drawdown_head_model_path`
  - `path_quality_head_model_path`
- Added `trained_head_summary` to the formal master scorecard document and upgraded aggregation states to distinguish:
  - `historical_replay_only`
  - `replay_with_quant_context`
  - `replay_with_multi_head_quant_context`
- Changed `src/ops/security_master_scorecard.rs` to degrade gracefully when a live snapshot has no full replay window:
  - instead of returning an error, it now emits a governed `replay_unavailable` document
  - the degraded document preserves multi-head summary context when trained heads are available
- Rewired `src/ops/security_chair_resolution.rs` so chair resolution now reads the formal `security_master_scorecard` object instead of rebuilding committee and scorecard context ad hoc.
- Upgraded chair reasoning and execution constraints to reference multi-head quant context, including trained expected drawdown guidance.
- Updated `src/ops/security_decision_submit_approval.rs` so approval-stage degraded master-scorecard generation follows the new unavailable-builder signature without dropping governed context.
- Added and updated focused coverage in:
  - `tests/security_scorecard_training_cli.rs`
  - `tests/security_master_scorecard_cli.rs`
  - `tests/security_chair_resolution_cli.rs`

### 修改原因
- P3 needed to stop at neither verbal “future profitability” explanations nor single-head direction classification.
- The user required one continuous path from governed training outputs to master scorecard aggregation and final chair reasoning, without pretending replay-unavailable live dates are hard errors.

### 方案还差什么
- [ ] The three regression heads are now governed and consumable, but they are still simple bin-mean regression heads rather than a stronger production-grade modeling stack.
- [ ] `upside_first_head` and `stop_first_head` remain classification-ready in training, but they still need richer path-event sample coverage before they can be treated as strong live signals.
- [ ] Master-scorecard aggregation still uses transparent fixed rules; coefficient re-estimation and champion promotion remain future work in the later governance phase.

### 潜在问题
- [ ] Live snapshots with `replay_unavailable` can still carry multi-head context, so downstream consumers must not sort or rank solely by `master_score` without also reading `aggregation_status`.
- [ ] Regression-head predictions currently depend on matching feature bins and fall back to a baseline when bins are sparse; this is correct for the current governed candidate stage but may underfit new regimes.
- [ ] Chair reasoning now trusts trained-head presence, not full replay availability, so future UI or Skill layers must explain the difference between `multi_head_ready` and `replay_unavailable`.

### 关闭项
- Multi-head regression training now runs through the governed CLI flow for `return_head`, `drawdown_head`, and `path_quality_head`.
- Master scorecard can now attach trained multi-head quant context and keep operating when the latest snapshot lacks a full forward replay window.
- Chair resolution now reads and cites the governed multi-head master scorecard instead of stopping at committee plus single-head scorecard language.
- Focused and adjacent regressions passed:
  - `cargo test --test security_scorecard_training_cli -- --nocapture`
  - `cargo test --test security_master_scorecard_cli -- --nocapture`
  - `cargo test --test security_chair_resolution_cli -- --nocapture`
  - `cargo test --test security_decision_submit_approval_cli -- --nocapture`

## 2026-04-11 23:59 Task - write the next-phase plan for path-event heads and historical proxy backfill

### 修改内容
- Added the next formal implementation plan at:
  - `docs/plans/2026-04-11-p4-path-events-and-history-backfill.md`
- The new plan defines one governed phase that combines:
  - historical external-proxy backfill
  - dated training-sample joins
  - `upside_first_head`
  - `stop_first_head`
  - master-scorecard and chair-level path-event consumption
- Captured recommended sequencing, exact file targets, focused test commands, and done-state criteria so the next session can execute without rediscovering the work boundary.

### 修改原因
- The user approved moving directly into the next gate after P3 and asked for a concrete plan first.
- This phase needs a stable written boundary, otherwise path-event modeling and history backfill are likely to sprawl into an unfocused “do everything” branch.

### 方案还差什么
- [ ] The plan is written, but the new governed backfill tool and path-event heads are not implemented yet.
- [ ] The plan assumes one new historical proxy runtime path; exact storage shape still needs to be finalized during Task 1 red tests.

### 潜在问题
- [ ] Historical proxy backfill can drift from live proxy contracts if later code changes only one side; contract tests will be essential.
- [ ] Path-event heads may still remain sample-thin after backfill unless we deliberately backfill enough stressed windows for ETFs and cross-asset pools.

### 关闭项
- The next implementation gate is now documented with exact tasks, files, and regression commands.
- Future execution can start directly from `docs/plans/2026-04-11-p4-path-events-and-history-backfill.md` without re-planning from scratch.

## 2026-04-11 Task - close the P4 historical-backfill and path-event bundle

### 修改内容
- Finished the governed `P4` bundle across historical proxy backfill, path-event heads, master-scorecard aggregation, and chair-level consumption:
  - historical external proxy storage/tool wiring
  - dated proxy hydration into feature snapshots
  - `upside_first_head` / `stop_first_head` training-readiness reporting
  - master-scorecard path-event probability context
  - chair reasoning and execution-constraint path-event asymmetry
- Stabilized the failing `stop_first_head` synthetic fixture in [tests/security_scorecard_training_cli.rs](D:/Rust/Excel_Skill/tests/security_scorecard_training_cli.rs):
  - kept the positive symbol in a `none` regime
  - retuned the negative symbol from an over-steep collapse to a controlled stop-first path
  - preserved the strict mixed-label guard instead of weakening production validation
- Verified the focused `P4` suites end-to-end:
  - `cargo test --test security_external_proxy_backfill_cli -- --nocapture`
  - `cargo test --test security_scorecard_training_cli -- --nocapture`
  - `cargo test --test security_master_scorecard_cli -- --nocapture`
  - `cargo test --test security_chair_resolution_cli -- --nocapture`

### 修改原因
- The last blocking failure in the continuous `Task 1 -> Task 6` delivery was not production logic drift but a stop-first fixture that collapsed to floor pricing before the governed sample windows.
- The user required this whole gate to be finished in one pass, so the remaining work needed to end with a green focused bundle rather than another partial handoff.

### 方案还差什么
- [ ] Historical proxy inputs are now governable and hydratable, but most pools still rely on manual or placeholder proxy values rather than true historical external series.
- [ ] `upside_first_head` and `stop_first_head` are now trained and consumable, but richer stressed-window coverage is still needed before they can be promoted beyond candidate use.
- [ ] Downstream consumers still need later governance work to distinguish `candidate`, `shadow`, and `champion` models formally.

### 潜在问题
- [ ] The repaired `stop_first_head` fixture is deterministic for the governed sample windows, but future changes to sampling ranges or history thresholds could invalidate the geometry; a fixture-intent test would be a good next lock.
- [ ] `master_scorecard` and chair outputs now carry more path-event context, so any UI or Skill layer that reads them must not treat those probabilities as champion-grade live signals yet.
- [ ] The repo still contains many unrelated dirty files and fixture directories; this task intentionally did not clean them to avoid crossing into unrelated work.

### 关闭项
- The `stop_first_head` training regression is green again without loosening the production mixed-label guard.
- The focused `P4` bundle is now self-consistent across backfill, training, master-scorecard, and chair layers.
- The continuous `Task 1 -> Task 6` execution for this gate is functionally closed at the focused-regression level.

## 2026-04-11 Task - design the P5 shadow champion and history expansion phase

### 修改内容
- Added the formal `P5` design document at:
  - `docs/plans/2026-04-11-p5-shadow-champion-and-history-expansion-design.md`
- Added the executable implementation plan at:
  - `docs/plans/2026-04-11-p5-shadow-champion-and-history-expansion.md`
- Locked the next governed phase to one recommended direction:
  - historical proxy expansion governance
  - `candidate -> shadow -> champion` grade semantics
  - approval and decision-package grade consumption

### 修改原因
- After `P4` closed, the next structural blocker was no longer path-event support but the lack of a governed lifecycle that tells us which trained artifacts are only research candidates and which can enter release-grade approval.
- The user approved Scheme C and wanted to move directly into the next gate without re-discovering scope later.

### 方案还差什么
- [ ] The new `P5` phase is designed and planned, but no implementation code has been written yet.
- [ ] Exact grade thresholds for `shadow` and `champion` still need to be encoded during Task 1 and Task 4 red tests.
- [ ] Real historical proxy expansion remains only partially covered today and will need governed accumulation before promotion can become meaningful.

### 潜在问题
- [ ] If grade rules are too loose, `candidate` models may be overstated as approval-grade.
- [ ] If grade rules are too strict, the chain may remain permanently stuck in `candidate`.
- [ ] Approval consumers will need careful wording so `shadow` is treated as reference context rather than full quant release authority.

### 关闭项
- The next phase now has both a design boundary and a task-by-task execution plan.
- Future implementation can start directly from the saved `P5` plan without another planning round.

## 2026-04-11 Task - close the P5 shadow champion and history expansion bundle

### 修改内容
- Completed the governed `P5` bundle across history expansion governance, model-grade lifecycle, and approval/package consumption:
  - added `security_history_expansion` as a formal CLI tool and persisted governed history-expansion documents
  - added `security_shadow_evaluation` to summarize readiness, proxy coverage, and recommended `candidate/shadow/champion` grade
  - added `security_model_promotion` to emit governed promotion decisions from registry + shadow evaluation inputs
  - extended scorecard registry/refit outputs with `model_grade` and `grade_reason`
  - wired approval brief and decision package to expose `model_grade_summary`
  - applied approval-stage guardrails so `shadow` stays reference-only and `candidate` remains governance-only
- Added backward-compatible registry loading in `src/ops/security_scorecard_model_registry.rs` so legacy shadow fixtures without explicit window metadata still deserialize during approval-grade loading.
- Verified the focused `P5` suites end-to-end:
  - `cargo test --test security_history_expansion_cli -- --nocapture`
  - `cargo test --test security_scorecard_refit_cli -- --nocapture`
  - `cargo test --test security_shadow_evaluation_cli -- --nocapture`
  - `cargo test --test security_model_promotion_cli -- --nocapture`
  - `cargo test --test security_decision_submit_approval_cli -- --nocapture`
  - `cargo test --test security_master_scorecard_cli -- --nocapture`
  - `cargo test --test security_chair_resolution_cli -- --nocapture`

### 修改原因
- The system could already train and score, but it still lacked a governed answer to “is this model only research, reference-only shadow, or approval-grade champion?”
- The last blocker inside the continuous `Task 1 -> P5` run was a runtime failure when approval tried to load an older registry fixture; fixing that compatibility gap was necessary to finish the whole gate without loosening grade governance.

### 方案还差什么
- [ ] The new grade lifecycle is now encoded, but real historical proxy expansion is still thin for many pools; future promotion quality still depends on richer historical coverage.
- [ ] `shadow` and `champion` thresholds are now governable, but long-horizon walk-forward/OOT promotion rules still need to be strengthened before champion can be trusted broadly.
- [ ] Approval/package now consume grade summaries, but downstream UI/Skill layers still need explicit champion-only release behavior if they want fully automated execution authority.

### 潜在问题
- [ ] Legacy registry compatibility is now broader, but very old fixtures that omit more than window metadata may still need targeted defaults if encountered later.
- [ ] `shadow` models now surface as reference-only context; any downstream consumer that ignores `approval_consumption_mode` could still over-read quant output.
- [ ] The repo remains dirty outside the securities mainline scope, including unrelated foundation work and fixture churn; this closeout intentionally did not clean those files.

### 关闭项
- `P5` is functionally closed at the focused-regression level in the current session.
- Approval and decision-package artifacts now carry governed model-grade semantics instead of treating all quant output as equal.
- The `Task 1 -> P5` continuous execution request is complete for the securities mainline scope that was under implementation.

## 2026-04-11 Task - close the P6 shadow champion hardening bundle

### 修改内容
- Completed the governed `P6` hardening layer across history-expansion coverage, repeated shadow observations, promotion gates, and approval/package governance summaries:
  - standardized `security_history_expansion` coverage output with `coverage_tier`, shadow/champion readiness hints, and per-proxy coverage rows
  - extended `security_shadow_evaluation` to consume prior governed observations and persist `shadow_observation_count`, `shadow_consistency_status`, and `promotion_blockers`
  - tightened `security_model_promotion` so champion promotion now requires repeated, consistent shadow observations with no blockers
  - wired `security_decision_submit_approval` to merge shadow-evaluation governance into approval briefs, guardrail actions, and decision packages
  - carried `model_governance_summary` through `security_decision_package_revision` so later package versions keep blocker/shadow lineage
- Added and turned green the new P6 regression coverage:
  - `security_history_expansion_exposes_standardized_readiness_coverage`
  - `security_shadow_evaluation_tracks_repeated_shadow_observations_for_champion_readiness`
  - `security_model_promotion_rejects_champion_when_shadow_observations_are_still_thin`
  - the upgraded `security_decision_submit_approval_downgrades_shadow_grade_to_reference_only_quant_context`
- Verified the focused `P6` bundle end-to-end:
  - `cargo test --test security_history_expansion_cli -- --nocapture`
  - `cargo test --test security_shadow_evaluation_cli -- --nocapture`
  - `cargo test --test security_model_promotion_cli -- --nocapture`
  - `cargo test --test security_decision_submit_approval_cli -- --nocapture`
  - `cargo test --test security_decision_package_revision_cli -- --nocapture`

### 修改原因
- `P5` introduced grade governance, but champion-readiness was still too thin because one green shadow snapshot could masquerade as durable approval evidence.
- The approval and package layers also still lacked a single persisted governance summary that preserved blocker and shadow-depth context across submit and revision flows.

### 方案还差什么
- [ ] Champion governance is now stricter, but it still relies on the current rule-based thresholds; future walk-forward and OOT windows should become first-class promotion gates.
- [ ] History expansion is now standardized, but many asset pools still need deeper real proxy coverage before `shadow` and `champion` can become common outcomes.
- [ ] Downstream UI/Skill consumers still need explicit champion-only release behavior if they want to consume the richer governance summary safely.

### 潜在问题
- [ ] Old package/registry fixtures that are more incomplete than the current compatibility assumptions may still need extra defaults when encountered later.
- [ ] Approval briefs now surface promotion blockers directly in `required_next_actions`; downstream readers must not treat those items as optional commentary.
- [ ] The repo remains dirty outside the securities mainline scope, and this P6 closeout intentionally did not clean unrelated worktree noise.

### 关闭项
- `P6` is functionally closed at the focused-regression level in the current session.
- Champion promotion now requires repeated governed shadow observations instead of a single optimistic snapshot.
- Approval briefs, decision packages, and package revisions now preserve one consistent shadow-governance summary across the securities mainline.

## 2026-04-11 Task - close the P7 automatic proxy backfill and promotion hardening bundle

### 修改内容
- Completed the governed `P7` bundle across reusable proxy-backfill coverage, multi-window shadow evidence, harder champion promotion, and downstream approval/package disclosure:
  - extended `security_external_proxy_backfill` so each governed backfill batch now persists a reusable result document with `covered_symbol_count`, `covered_dates`, `covered_proxy_fields`, `coverage_tier`, and `backfill_result_path`
  - extended `security_history_expansion` to consume governed backfill-result documents and fold actual batch refs, covered dates, and imported-record counts into standardized coverage summaries
  - extended `security_shadow_evaluation` with `comparison_model_registry_paths`, `shadow_window_count`, `oot_stability_status`, `window_consistency_status`, and `promotion_evidence_notes`
  - hardened `security_model_promotion` so champion now requires repeated shadow observations plus stable comparison-window / OOT evidence with no remaining evidence notes
  - extended approval brief and decision-package governance summaries so submit/revision flows preserve `shadow_window_count`, `oot_stability_status`, `window_consistency_status`, and `promotion_evidence_notes`
  - surfaced multi-window promotion evidence notes into approval `required_next_actions`, so thin comparison-window evidence is no longer silently hidden behind coarse grade labels
- Kept the existing stock-tool chain intact instead of introducing a parallel lifecycle:
  - `security_external_proxy_backfill -> security_history_expansion -> security_shadow_evaluation -> security_model_promotion -> security_decision_submit_approval`
- Verified the focused `P7` bundle end-to-end:
  - `cargo test --test security_external_proxy_backfill_cli -- --nocapture`
  - `cargo test --test security_history_expansion_cli -- --nocapture`
  - `cargo test --test security_shadow_evaluation_cli -- --nocapture`
  - `cargo test --test security_model_promotion_cli -- --nocapture`
  - `cargo test --test security_decision_submit_approval_cli -- --nocapture`
  - `cargo test --test security_decision_package_revision_cli -- --nocapture`

### 修改原因
- `P6` already hardened shadow/champion governance, but historical proxy backfill still was not reusable promotion evidence; it remained a storage write instead of a governed coverage source.
- Champion promotion also still lacked explicit multi-window / OOT-style stability evidence, so a repeated shadow count could look stronger than the model’s actual out-of-window behavior.
- Approval and package consumers needed the stronger evidence trail to avoid flattening all promotion outcomes into one coarse grade summary.

### 方案还差什么
- [ ] Real proxy history is now more governable, but most pools still need broader and denser historical coverage before `shadow` and especially `champion` can become common outcomes.
- [ ] Champion promotion now checks stronger evidence, but it is still rule-based; future walk-forward / OOT promotion rules should become even more first-class and less fixture-like.
- [ ] Downstream UI/Skill layers still need explicit champion-only release semantics if they want to consume the richer `promotion_evidence_notes` safely in automated workflows.

### 潜在问题
- [ ] `covered_proxy_fields` now follows the governed result’s actual bound-field list; if later consumers expect grouped proxy families instead of raw fields, they may need an extra normalization layer.
- [ ] Comparison-window stability currently uses rule-based `test.auc` / `test.accuracy` thresholds; later changes to registry metrics or asset-pool readiness thresholds may require synchronized updates.
- [ ] Approval briefs now surface `promotion_evidence_notes` directly into follow-up actions; downstream readers must not treat those as optional commentary.

### 关闭项
- `P7` is functionally closed at the focused-regression level in the current session.
- Historical proxy backfill is now a reusable governed coverage source instead of only a runtime side effect.
- Champion promotion now depends on both repeated shadow observations and multi-window / OOT-style stability evidence.

## 2026-04-12 Task - land the P8-1 formal condition review tool

### 修改内容
- Added the first formal `security_condition_review` lifecycle object on the stock mainline:
  - created `src/ops/security_condition_review.rs`
  - introduced the formal request, binding, document, result, and error contracts
  - added deterministic follow-up actions such as `keep_plan`, `update_position_plan`, `reopen_committee`, `reopen_research`, and `freeze_execution`
- Wired the new object into the current branch instead of reviving outdated module layouts:
  - mounted the module in `src/ops/stock.rs`
  - re-exported it in `src/ops/mod.rs`
  - exposed it in `src/tools/catalog.rs`
  - added dispatcher support in `src/tools/dispatcher/stock_ops.rs`
  - routed the top-level dispatcher in `src/tools/dispatcher.rs`
- Added focused CLI coverage:
  - created `tests/security_condition_review_cli.rs`
  - locked both tool discovery and structured result output
- Verified the focused regression:
  - `cargo test --test security_condition_review_cli -- --nocapture`

### 修改原因
- The current branch had approval, scorecard, package, and promotion governance, but it still lacked a formal intraperiod review object.
- `P8` needs review triggers to become replayable lifecycle artifacts before execution records and post-trade reviews can bind to them.
- The older memory of separate review/execution files did not match the current branch, so the safest path was to add the missing object directly on today’s stock mainline.

### 方案还差什么
- [ ] `security_execution_record` still needs to be added as the next formal lifecycle object.
- [ ] `security_post_trade_review` still needs to be added with layered attribution and governance feedback.
- [ ] Approval/package/holding chains still need to preserve the new P8 refs beyond the first review object.

### 潜在问题
- [ ] `condition_review_id` currently uses `symbol + analysis_date + trigger_type`; repeated same-day same-type reviews will need a later versioning or sequence rule.
- [ ] Follow-up derivation is intentionally rule-based in this first pass; later execution and replay policies may require richer trigger semantics.
- [ ] The current review tool is not yet persisted to runtime files; that will be completed when the broader P8 object graph is wired.

### 关闭项
- `P8-1` is closed at the focused CLI-contract level in the current session.
- The stock mainline now has a first-class formal condition-review tool.

## 2026-04-12 Task - land the P8-2 formal execution record tool

### 修改内容
- Added the formal `security_execution_record` lifecycle object on the stock mainline:
  - created `src/ops/security_execution_record.rs`
  - introduced the request, binding, document, result, and error contracts
  - formalized execution facts such as `execution_action`, `execution_status`, `executed_gross_pct`, and optional `condition_review_ref`
- Wired the new execution object into the current branch:
  - mounted the module in `src/ops/stock.rs`
  - re-exported it in `src/ops/mod.rs`
  - exposed it in `src/tools/catalog.rs`
  - added dispatcher support in `src/tools/dispatcher/stock_ops.rs`
  - routed the top-level dispatcher in `src/tools/dispatcher.rs`
- Added focused CLI coverage:
  - created `tests/security_execution_record_cli.rs`
  - locked tool discovery and structured execution-record output
- Verified the focused regression:
  - `cargo test --test security_execution_record_cli -- --nocapture`

### 修改原因
- `P8-1` added the first formal review object, but the current branch still had no replayable execution event object.
- The lifecycle needed a stable contract for build/add/reduce/exit/freeze style events before post-trade review can attach to actual execution facts.
- The safest path remained adding the missing execution object directly onto the current stock mainline instead of reviving obsolete branch-only layouts.

### 方案还差什么
- [ ] `security_post_trade_review` still needs to be added with layered attribution and governance feedback.
- [ ] Approval/package/holding chains still need to preserve the new review and execution refs.
- [ ] Runtime persistence and replay wiring for the new P8 objects still needs to be completed in later P8 tasks.

### 潜在问题
- [ ] `execution_record_id` currently uses `symbol + analysis_date + execution_action`; repeated same-day same-action records will need a later versioning or sequence rule.
- [ ] The first-pass execution contract records facts but does not yet enforce richer action vocabularies or transition rules.
- [ ] The new execution tool is formal and routable, but not yet persisted into package/object-graph outputs.

### 关闭项
- `P8-2` is closed at the focused CLI-contract level in the current session.
- The stock mainline now has a first-class formal execution-record tool.

## 2026-04-12 Task - land the P8-3 formal post-trade review tool

### 修改内容
- Added the formal `security_post_trade_review` lifecycle object on the stock mainline:
  - created `src/ops/security_post_trade_review.rs`
  - introduced the request, layered attribution, binding, document, result, and error contracts
  - formalized replayable review facts such as `review_status`, `recommended_governance_action`, and layered `data/model/governance/execution` attribution
- Wired the new post-trade review object into the current branch:
  - mounted the module in `src/ops/stock.rs`
  - re-exported it in `src/ops/mod.rs`
  - exposed it in `src/tools/catalog.rs`
  - added dispatcher support in `src/tools/dispatcher/stock_ops.rs`
  - routed the top-level dispatcher in `src/tools/dispatcher.rs`
- Added focused CLI coverage:
  - created `tests/security_post_trade_review_cli.rs`
  - locked tool discovery and structured post-trade-review output
- Verified the focused regression:
  - `cargo test --test security_post_trade_review_cli -- --nocapture`

### 修改原因
- `P8-2` added replayable execution events, but the current branch still lacked a formal post-trade review object to close the lifecycle.
- `P8` needs review conclusions to be stored as first-class governed artifacts instead of conversational notes or ad hoc summaries.
- Layered attribution had to become machine-readable before later feedback loops can distinguish data/model/governance/execution problems.

### 方案还差什么
- [ ] Approval/package/holding chains still need to preserve the new review/execution/post-trade refs.
- [ ] Lifecycle outputs still need to feed later governance feedback and operator-facing holding views.
- [ ] Runtime persistence for the new P8 objects still needs to be wired through the broader package/object graph.

### 潜在问题
- [ ] `post_trade_review_id` currently uses `symbol + analysis_date + review_status`; repeated same-day same-status reviews will need a later sequencing rule.
- [ ] The first-pass review notes are deterministic but intentionally shallow; later replay and retraining flows may require richer typed actions.
- [ ] The new post-trade review tool is formal and routable, but not yet persisted into package/object-graph outputs.

### 关闭项
- `P8-3` is closed at the focused CLI-contract level in the current session.
- The stock mainline now has a first-class formal post-trade-review tool.

## 2026-04-12 Task - close the P8-4 to P8-6 lifecycle package and operator-view bundle

### 修改内容
- Extended the formal package contract so lifecycle artifacts can join the securities mainline after approval-time package creation:
  - added optional `condition_review_ref / execution_record_ref / post_trade_review_ref`
  - added optional lifecycle paths to `object_graph`
  - added `lifecycle_governance_summary` with `lifecycle_status`, `recommended_governance_action`, and structured attribution layers
- Hardened `security_decision_package_revision` for lifecycle attachment:
  - added optional `condition_review_path / execution_record_path / post_trade_review_path`
  - load and validate lifecycle bindings against the existing package refs
  - upsert lifecycle artifacts into `artifact_manifest`
  - preserve lifecycle refs and feedback summary across revisions
- Kept the initial approval package conservative:
  - `security_decision_submit_approval` now initializes lifecycle fields as empty and expects later revisions to attach governed lifecycle documents
- Added focused regression coverage:
  - extended `tests/security_decision_package_revision_cli.rs`
  - locked attachment of lifecycle refs plus feedback summary
- Closed the operator-view side with new docs:
  - updated `docs/security-holding-ledger.md` with lifecycle tracking fields and operator sequence
  - added `docs/execution-notes-2026-04-12-p8-lifecycle-closeout.md`
- Verified the focused regressions:
  - `cargo test --test security_decision_package_revision_cli -- --nocapture`
  - `cargo test --test security_decision_submit_approval_cli security_decision_submit_approval_writes_runtime_files_for_ready_case -- --nocapture`
  - `cargo test --test security_condition_review_cli -- --nocapture`
  - `cargo test --test security_execution_record_cli -- --nocapture`
  - `cargo test --test security_post_trade_review_cli -- --nocapture`

### 修改原因
- `P8-1` to `P8-3` created the formal lifecycle tools, but the package chain still could not preserve them as first-class governed refs.
- Operator-facing holding artifacts still had no standard place to record lifecycle refs and feedback actions.
- The current workspace contains handoff-file conflict residue, so a clean execution-note document was needed to preserve the P8 closeout without disturbing unrelated merge state.

### 方案还差什么
- [ ] P9 still needs a narrow verification-data slice so the new lifecycle can replay against realistic runtime artifacts instead of only fixture JSON.
- [ ] Automatic consumption of post-trade feedback by shadow/promotion governance still belongs to later data-validation phases.
- [ ] Existing handoff documents with conflict markers still need a separate cleanup pass when the broader workspace is ready.

### 潜在问题
- [ ] Lifecycle attachment currently happens during package revision, not initial submission; callers must remember the two-step pattern.
- [ ] The first lifecycle-feedback summary is intentionally compact; later phases may need richer typed remediation objects beyond `recommended_governance_action`.
- [ ] `security-holding-ledger.md` already had historical encoding noise in older sections, so future edits should continue to avoid broad rewrites unless the file is normalized first.

### 关闭项
- `P8` is functionally closed at the focused-regression and operator-view level in the current session.
- The securities mainline now has a governed lifecycle path from approval into condition review, execution recording, post-trade review, and package revision attachment.

## 2026-04-12 Task - close the P9-P10 verification slice and replay bundle

### 修改内容
- Added a governed end-to-end lifecycle validation slice test:
  - created `tests/security_lifecycle_validation_cli.rs`
  - generated formal lifecycle artifacts through the actual tool chain instead of hand-authored JSON
  - persisted `condition_review.json`, `execution_record.json`, `post_trade_review.json`, and `validation_slice_manifest.json`
- Added P9/P10 operator-facing documentation:
  - created `docs/plans/2026-04-12-security-verification-data-slice.md`
  - created `docs/execution-notes-2026-04-12-p9-p10-validation-closeout.md`
  - updated `docs/security-holding-ledger.md` with a stable lifecycle validation-slice entry
- Materialized a stable runtime replay slice:
  - copied the latest lifecycle validation fixture into `.excel_skill_runtime/validation_slices/601916_SH_2026-04-12_lifecycle/`
  - normalized `validation_slice_manifest.json` so the stored paths point to the stable runtime copy
  - added `.excel_skill_runtime/validation_slices/601916_SH_2026-04-12_lifecycle/README.md`
- Verified the focused lifecycle suites:
  - `cargo test --test security_lifecycle_validation_cli -- --nocapture`
  - `cargo test --test security_condition_review_cli -- --nocapture`
  - `cargo test --test security_execution_record_cli -- --nocapture`
  - `cargo test --test security_post_trade_review_cli -- --nocapture`
  - `cargo test --test security_decision_submit_approval_cli -- --nocapture`
  - `cargo test --test security_decision_package_revision_cli -- --nocapture`

### 修改原因
- `P9` needed one narrow but real replay slice before broader data backfill starts, otherwise later verification would keep depending on ad hoc fixture reconstruction.
- `P10` needed a true end-to-end lifecycle proof that uses the formal tools themselves, not only manually authored JSON attachments.
- Operators and later AIs need a stable runtime location for replay, not only a transient test-fixture directory.

### 方案还差什么
- [ ] Broader live-data backfill beyond the validation slice still belongs to the next data-expansion phase.
- [ ] The validation slice currently uses mocked approval-time disclosure endpoints, so live-data equivalence is still a separate concern.
- [ ] `docs/AI_HANDOFF.md` still has unrelated conflict residue and remains intentionally untouched in this session.

### 潜在问题
- [ ] The stable validation slice is a copied replay artifact, so it should be refreshed deliberately if the underlying lifecycle contracts change.
- [ ] Consumers must still read lifecycle refs and governance summaries together; replay files alone do not imply live trading approval.
- [ ] Re-running the validation test will create new transient fixture directories under `tests/runtime_fixtures/security_lifecycle_validation/`, so cleanup may be useful later.

### 关闭项
- `P9` is closed with one governed validation-data slice and a stable runtime replay copy.
- `P10` is closed with focused lifecycle verification across the new execution/review loop.
- `P8` to `P10` are now functionally closed in the current session.

## 2026-04-12 Task - add governed real-data validation backfill and refresh the first live-compatible slice

### 修改内容
- Added a governed stock tool for real-data validation refresh:
  - created `src/ops/security_real_data_validation_backfill.rs`
  - wired `security_real_data_validation_backfill` through `src/ops/stock.rs`, `src/ops/mod.rs`, `src/tools/catalog.rs`, `src/tools/dispatcher.rs`, and `src/tools/dispatcher/stock_ops.rs`
- Refactored stock-history sync reuse:
  - extended `src/ops/sync_stock_price_history.rs`
  - added reusable provider-fetch contract `SyncStockPriceHistoryFetchedRows`
  - kept the existing `sync_stock_price_history` behavior unchanged for current callers
- Added focused TDD coverage:
  - created `tests/security_real_data_validation_backfill_cli.rs`
  - locked tool discovery plus dedicated validation-root persistence for `stock_history.db`, `fullstack_context.json`, and `real_data_validation_manifest.json`
- Added operator-facing notes:
  - updated `docs/execution-notes-2026-04-12-p9-p10-validation-closeout.md`
  - added `docs/execution-notes-2026-04-12-real-data-validation-backfill.md`
- Refreshed the first live-compatible validation slice:
  - `.excel_skill_runtime/validation_slices/601916_SH_real_data_20260412/stock_history.db`
  - `.excel_skill_runtime/validation_slices/601916_SH_real_data_20260412/fullstack_context.json`
  - `.excel_skill_runtime/validation_slices/601916_SH_real_data_20260412/real_data_validation_manifest.json`
  - imported `388` rows each for `601916.SH`, `510300.SH`, and `512800.SH`

### 修改原因
- The securities mainline had deterministic replay slices after `P9/P10`, but there was still no governed tool to refresh a validation slice with live-compatible price history plus public disclosure context.
- Operators needed one repeatable entry point instead of re-running shell sequences for price sync and fullstack analysis.
- Real-data verification needed to remain narrow and auditable before broader production ingestion begins.

### 方案还差什么
- [ ] Broader automatic real-data ingestion for treasury/gold/cross-border proxy histories still belongs to the later data-expansion phase.
- [ ] `docs/security-holding-ledger.md` still has historical encoding noise, so the first real-data slice was documented through dedicated execution notes instead of a broad ledger rewrite.
- [ ] Live-compatible validation slices still need more symbols and wider market coverage before they become a stronger operator regression set.

### 潜在问题
- [ ] The first live-compatible attempt with `2025-01-01 -> 2025-08-08` failed because the formal technical gate only saw `89` rows; callers must ensure windows cover at least the governed technical minimum.
- [ ] The new tool temporarily redirects `EXCEL_SKILL_STOCK_DB` inside the CLI process so the reused fullstack chain reads the slice-local `stock_history.db`; future deeper refactors may replace this with an explicit path parameter.
- [ ] Public providers can change contracts, so the live-compatible slice should be treated as a validation artifact, not a production trading evidence pack.

### 关闭项
- The governed tool `security_real_data_validation_backfill` is now available on the public stock tool chain.
- One live-compatible validation slice for `601916.SH` has been refreshed and persisted under `.excel_skill_runtime/validation_slices/601916_SH_real_data_20260412/`.
- Focused regressions passed for the new tool, the stock-history sync chain, and the lifecycle validation slice.

## 2026-04-12 Task - backfill the second live-compatible validation-slice batch for ETF coverage

### 修改内容
- Refreshed four additional live-compatible validation slices with the governed tool `security_real_data_validation_backfill`:
  - `.excel_skill_runtime/validation_slices/511010_SH_real_data_20260412/`
  - `.excel_skill_runtime/validation_slices/518880_SH_real_data_20260412/`
  - `.excel_skill_runtime/validation_slices/513500_SH_real_data_20260412/`
  - `.excel_skill_runtime/validation_slices/512800_SH_real_data_20260412/`
- Standardized the current contextual proxy wiring for validation usage:
  - `511010.SH`, `518880.SH`, and `513500.SH` use `510300.SH` plus `512800.SH`
  - `512800.SH` uses `510300.SH` plus `sector_profile = a_share_bank`
- Updated operator-facing notes:
  - `docs/execution-notes-2026-04-12-real-data-validation-backfill.md`
  - `docs/execution-notes-2026-04-12-p9-p10-validation-closeout.md`

### 修改原因
- One stock-oriented live-compatible slice was not enough for later ETF-path validation, shadow checks, or promotion hardening.
- The current fullstack chain still enforces contextual proxy inputs, so the ETF validation set needed a stable reusable proxy policy before broader real-proxy ingestion begins.
- Operators need a broader but still governed validation set before moving on to treasury/gold/cross-border proxy-history expansion.

### 方案还差什么
- [ ] The ETF-oriented slices still use generic A-share contextual proxies, so category-native proxy histories remain a later data-expansion task.
- [ ] Real external proxy histories for treasury, gold, and cross-border ETF drivers still need to be backfilled before those slices can support deeper model validation.
- [ ] The current validation set is still centered on `2025-08-08` windows; later phases should widen dates and symbols further.

### 潜在问题
- [ ] The current fullstack stack does not auto-sync proxy symbols when only `market_profile` / `sector_profile` is supplied, so validation runs should continue to prefer explicit proxy symbols unless the primary symbol already covers the resolved profile.
- [ ] ETF slices that reuse `510300.SH` and `512800.SH` are fit for governed validation, but they should not be mistaken for final category-native environment packs.
- [ ] Public providers can still drift, so these slices should be refreshed deliberately before any stricter regression baselines are locked.

### 关闭项
- Four additional live-compatible ETF validation slices are now available under `.excel_skill_runtime/validation_slices/`.
- The governed real-data validation set now covers one stock sample plus treasury, gold, cross-border, and equity ETF samples.
- Operator-facing execution notes now record the second validation-slice batch and the current proxy-wiring rule.

## 2026-04-12 Task - close governed stock-information history replay for validation slices

### 修改内容
- Completed the governed stock-information replay path inside `security_real_data_validation_backfill`:
  - added `with_validation_history_overrides(...)` in `src/ops/security_real_data_validation_backfill.rs`
  - scoped `EXCEL_SKILL_STOCK_DB`, `EXCEL_SKILL_FUNDAMENTAL_HISTORY_DB`, and `EXCEL_SKILL_DISCLOSURE_HISTORY_DB` together for slice-local fullstack replay
  - added `restore_env_override(...)` to restore all runtime overrides after one validation call
- Closed the stock CSV importer gap for lean fixtures:
  - updated `src/ops/import_stock_price_history.rs`
  - made `adj_close` optional
  - added fallback parsing that reuses `close` when `adj_close` is absent
- Fixed the governed validation test harness:
  - updated `tests/security_stock_history_governance_cli.rs`
  - expanded the local HTTP fixture server to survive repeated price-sync plus disclosure/fundamental fetches in the same validation flow
- Added and verified the red-green regression for lean price CSV input:
  - `tests/stock_price_history_import_cli.rs`
  - `import_stock_price_history_defaults_adj_close_to_close_when_missing`

### 修改原因
- The Historical Data Phase 1 closure still failed on two real blockers:
  - validation slices could not point fullstack at slice-local governed fundamental/disclosure stores
  - lean CSV fixtures without `adj_close` were rejected before governed fullstack replay could even start
- The validation-slice regression also exposed a fixture bug where the local HTTP server stopped accepting requests before disclosure and financial fetches happened.
- These issues prevented governed stock-information history from becoming a reusable validation baseline for later real-data expansion.

### 方案还差什么
- [ ] Historical Data Phase 1 still needs the next broader data-expansion wave for real ETF proxy histories, stock fundamental batches, and stock disclosure batches beyond the governed replay closure.
- [ ] `security_real_data_validation_backfill` still relies on live-compatible fetches before it freezes slice-local governed history; later phases can add broader governed source ingestion to reduce live dependency.
- [ ] The broader warning set under `src/tools/dispatcher.rs` remains untouched, because this task stayed scoped to governed history replay and validation.

### 潜在问题
- [ ] The validation-slice helper still depends on environment-variable overrides, so future refactors that change runtime path resolution should keep the triple-override path covered.
- [ ] The local HTTP route server now accepts a fixed request budget (`12`); if validation flows grow materially, this fixture may need a more explicit stop condition.
- [ ] Lean CSV imports now fall back from missing `adj_close` to `close`; if a future data source requires rejecting unadjusted inputs, that rule should be enforced by source-specific validation instead of the shared importer.

### 关闭项
- Governed stock-information history replay now works inside validation slices for price, fundamental, and disclosure context together.
- `security_analysis_fullstack` now prefers governed stock-information history in the focused validation path without depending on live financial or disclosure success.
- The lean CSV importer bug is closed and covered by a dedicated regression.
- Focused regressions passed for importer fallback, governed fullstack history precedence, slice-local history persistence, and the adjacent backfill tools.

## 2026-04-12 Task - land real governed stock history backfill and live validation slice

### 修改内容
- Closed the CSV-based ETF proxy-history import path:
  - updated `src/ops/security_external_proxy_history_import.rs`
  - normalized UTF-8 BOM on the first header cell
  - padded short trailing CSV rows so optional trailing blanks no longer break governed import
- Fixed the formal CSV fixture to use one standards-shaped shared header width:
  - updated `tests/security_external_proxy_history_import_cli.rs`
  - aligned treasury, gold, cross-border, and equity ETF rows to the same 25-column contract
- Switched the default EastMoney financial endpoint to the currently live metrics route:
  - updated `src/ops/security_analysis_fullstack.rs`
  - default financial live fetch now uses `ZYZBAjaxNew` instead of the retired `MainTargetAjax`
- Executed the first real governed stock-information backfill batch for `601916.SH`:
  - `security_fundamental_history_live_backfill` imported `9` report periods into `.excel_skill_runtime\\governed_history_live\\security_fundamental_history.db`
  - `security_disclosure_history_live_backfill` imported `60` announcement rows into `.excel_skill_runtime\\governed_history_live\\security_disclosure_history.db`
- Executed one live real-data validation slice for `601916.SH`:
  - `security_real_data_validation_backfill` created `.excel_skill_runtime\\validation_slices\\601916_SH_real_data_20260412_live`
  - imported `388` governed price rows each for `601916.SH`, `510300.SH`, and `512800.SH`
  - persisted `fullstack_context.json` and `real_data_validation_manifest.json`

### 修改原因
- Historical Data Phase 1 was still blocked by one real runtime defect:
  - the formal ETF proxy-history import failed on CSV batches before any real historical expansion could continue
- The first attempt to backfill live fundamentals for `601916.SH` exposed a second real blocker:
  - the default EastMoney financial endpoint had drifted and now returned `406/HTML`, so governed stock fundamental history could not be refreshed from live data
- These two blockers prevented us from moving from “tooling is ready” to “real governed history has actually landed and can be validated”.

### 方案还差什么
- [ ] ETF external proxy history still needs broader real batches beyond the formal file-import bridge; treasury/gold/cross-border/equity histories are not yet filled with long governed series.
- [ ] Stock fundamental and disclosure live backfill has only been proven on the first real symbol slice (`601916.SH`); the next wave should expand to a broader stock basket.
- [ ] Real-data validation still needs broader governed slices for more stocks and ETF pools before stronger shadow/champion verification can be treated as representative.

### 潜在问题
- [ ] `security_external_proxy_history_import` still uses a lightweight CSV parser, so quoted commas or embedded newlines would need stronger CSV decoding if future operator batches become more complex.
- [ ] The live EastMoney financial route now works through `ZYZBAjaxNew`, but provider payload shape could still drift later; the live backfill tests only lock the override contract, not the public provider schema.
- [ ] The first live validation slice uses `sina` for governed prices; later provider drift or symbol-specific gaps should be watched when we expand the validation basket.

### 关闭项
- The governed ETF proxy-history import bug is closed and covered by a green CLI regression.
- Live governed stock fundamental history for `601916.SH` is now landing again through the default provider path.
- Live governed stock disclosure history for `601916.SH` is now persisted into the shared governed history runtime.
- A first real-data validation slice for `601916.SH` has been generated successfully and is ready for later verification-chain consumption.

## 2026-04-12 Task - batch backfill governed stock information history and ETF proxy history

### 修改内容
- Executed the first governed stock-information batch across 10 A-share symbols:
  - live fundamental history landed in `.excel_skill_runtime\\governed_history_live\\security_fundamental_history.db`
  - live disclosure history landed in `.excel_skill_runtime\\governed_history_live\\security_disclosure_history.db`
  - covered symbols:
    - `601916.SH`
    - `600519.SH`
    - `300750.SZ`
    - `000001.SZ`
    - `600036.SH`
    - `601318.SH`
    - `600030.SH`
    - `600887.SH`
    - `002594.SZ`
    - `000333.SZ`
- Generated and imported one governed ETF external proxy history batch:
  - source file: `.excel_skill_runtime\\governed_history_live\\external_proxy_history_batches\\external_proxy_history_phase1_20260412_c.csv`
  - import result: `.excel_skill_runtime\\external_proxy_backfill_results\\external_proxy_backfill_external_proxy_history_phase1_20260412_c.json`
  - storage target: `.excel_skill_runtime\\security_external_proxy.db`
  - imported `1528` dated proxy rows across:
    - `511010.SH` treasury ETF
    - `518880.SH` gold ETF
    - `513500.SH` cross-border ETF
    - `512800.SH` equity ETF
- Verified governed proxy consumption through formal snapshot calls:
  - `511010.SH` now hydrates `yield_curve_*` and `funding_liquidity_*` from historical proxy backfill on `2025-08-08`
  - `518880.SH` now hydrates `gold_spot_*`, `usd_index_*`, and `real_rate_*` on `2025-08-08`
  - `513500.SH` now hydrates `fx_*`, `overseas_market_*`, and `market_session_gap_*` on `2025-08-08`
  - `512800.SH` now hydrates `etf_fund_flow_*`, `premium_discount_*`, and `benchmark_relative_*` on `2025-08-08`

### 修改原因
- The user explicitly requested closing stock-information batch backfill and ETF proxy history in one continuous wave instead of proving the pipeline on a single symbol.
- The project had already reached the stage where structure was no longer the blocker; the blocker was lack of thicker real governed history for both stock information and ETF external proxies.
- We needed one auditable batch that could be consumed by the formal stock tool chain, not another one-off manual note beside the runtime.

### 方案还差什么
- [ ] ETF proxy history is now governed and dated, but the current batch still uses real observable market-derived proxies rather than all-official specialized provider series.
- [ ] Broader stock validation slices are still thin; only `601916.SH` has a live governed validation slice with stock-information history fully refreshed in this wave.
- [ ] Shadow / promotion still needs a later wave that consumes these thicker governed histories over broader symbol baskets.

### 潜在问题
- [ ] Historical ETF proxy hydration only happens when the formal request carries `as_of_date`; callers that omit it will still fall back to `placeholder_unbound`.
- [ ] `512800.SH` still depends on local governed price history rather than Yahoo-compatible remote history, so future large-basket replay should keep a China-market-native fallback.
- [ ] The current ETF proxy batch intentionally mixes official public macro series and real market-derived ETF proxies; later promotion rules should keep these proxy provenance notes visible.

### 关闭项
- The first governed stock-information batch for 10 A-share symbols is landed and queryable.
- The first governed ETF external proxy history batch is landed and auditable.
- Treasury, gold, cross-border, and equity ETF proxy histories are now consumable by `security_feature_snapshot` on dated replay requests.
- The securities mainline now has enough governed real data to continue with stronger validation instead of only pipeline hardening.

## 2026-04-12 Task - harden ETF final-chain replay from proxy snapshot to chair resolution

### 修改内容
- Hardened the ETF proxy deep-chain handoff so governed historical proxy inputs now survive beyond `security_feature_snapshot` and enter:
  - `security_decision_evidence_bundle`
  - `security_scorecard`
  - `security_master_scorecard`
  - `security_chair_resolution`
- Refreshed ETF-native validation slices and corrected their governed peer environments:
  - `511010_SH_real_data_20260412` now persists `sector_symbol = 511060.SH` with `sector_profile = treasury_etf`
  - `518880_SH_real_data_20260412` now persists `sector_symbol = 518800.SH` with `sector_profile = gold_etf`
  - `513500_SH_real_data_20260412` now persists `sector_symbol = 513500.SH` with `sector_profile = cross_border_etf`
  - `512800_SH_real_data_20260412` now persists `sector_symbol = 512800.SH` with `sector_profile = equity_etf_peer`
- Trained one real ETF sub-scope direction-head candidate artifact per pool under `.excel_skill_runtime\\etf_training_live_batch_20260412`:
  - `511010.SH` -> `treasury_etf`
  - `518880.SH` -> `gold_etf`
  - `513500.SH` -> `cross_border_etf`
  - `512800.SH` -> `equity_etf`
- Re-ran the formal final chair layer for all four ETFs with those governed artifacts:
  - every symbol now reaches `score_status = ready`
  - final outputs no longer degrade to `model_unavailable` or `cross_section_invalid`
  - current chair actions for all four remain `abstain` because committee/risk governance still reports `needs_more_evidence`

### 修改原因
- The user explicitly required us to fix the three ETF blockers together instead of stopping at snapshot visibility:
  - proxy history had to survive into the final conclusion layer
  - validation slices had to become ETF-native instead of reusing wrong peer environments
  - each ETF sub-pool needed a structurally correct scorecard artifact before the final chair layer could be trusted
- We had already proven that ETF proxy history was visible in snapshot output, but the final analysis stage still could not consume it reliably.
- The real debugging goal in this wave was not “more rows exist” but “governed ETF data can now reach the last formal decision object”.

### 方案还差什么
- [ ] ETF final-chain artifacts are now structurally valid and consumable, but the current direction-head models are still only `candidate` grade and should not yet be treated as champion-grade real-money signals.
- [ ] ETF final chair outputs still downgrade to `abstain` because governed stock-information layers remain unavailable for ETF symbols; later ETF research quality needs a more explicit ETF-specific information strategy instead of inheriting stock-only expectations.
- [ ] Multi-head ETF artifacts (`return_head`, `drawdown_head`, `path_quality_head`) still need a later live-data training wave if we want the final chair layer to move beyond a direction-only quantitative context.

### 潜在问题
- [ ] `513500.SH` cross-border validation currently uses the primary ETF itself as the governed peer environment, which is structurally acceptable for replay but still thinner than a fuller overseas index proxy basket.
- [ ] `512800.SH` now keeps `equity_etf_peer` semantics in the slice manifest, but downstream consumers that hard-code `a_share_bank` could still misread it if they bypass the normalized profile path.
- [ ] The current ETF candidate artifacts show uneven out-of-sample quality (`gold_etf` and especially `cross_border_etf` remain weak), so later thickening should focus on data depth and pool-specific external drivers before any promotion attempt.

### 关闭项
- ETF governed proxy history now survives from dated replay into the final chair chain.
- ETF validation slices now keep ETF-native peer environments instead of the earlier wrong generic bank-ETF fallback.
- Treasury, gold, cross-border, and equity ETF pools each have a governed direction-head candidate artifact that the formal scorecard runtime accepts.
- The four ETF validation targets now reach the last formal conclusion object with `score_status = ready`, so the current bottleneck has moved from structure to evidence depth and model strength.

## 2026-04-13 Task - complete multi-head live validation for stock and ETF representative assets

### 修改内容
- Trained a governed six-head live batch for one stock and four ETF representative assets under `.excel_skill_runtime\\multi_head_live_batch_20260413`:
  - `601916.SH`
  - `511010.SH`
  - `518880.SH`
  - `513500.SH`
  - `512800.SH`
- Landed live candidate artifacts for all six heads on every representative asset:
  - `direction_head`
  - `return_head`
  - `drawdown_head`
  - `path_quality_head`
  - `upside_first_head`
  - `stop_first_head`
- Re-ran `security_chair_resolution` with the full multi-head model set and confirmed that all five assets now reach the final chair layer with:
  - `score_status = ready`
  - governed multi-head context available
  - final path-risk constraints emitted into chair reasoning / execution constraints
- Persisted the live run summaries to:
  - `.excel_skill_runtime\\multi_head_live_batch_20260413\\training_summary.json`
  - `.excel_skill_runtime\\multi_head_live_batch_20260413\\chair_resolution_summary.json`
- Added the execution note:
  - `docs\\execution-notes-2026-04-13-multi-head-live-validation.md`

### 修改原因
- The user explicitly required us to stop treating the securities stack as a trend-only system and to push risk, regression, path-event, and classification lines all the way to real-data-backed final conclusions.
- Earlier ETF work had already fixed the structural replay chain, so the next honest checkpoint was to prove that the formal chair layer could consume:
  - return context
  - drawdown context
  - path quality context
  - upside-first context
  - stop-first context
- We also needed an auditable live batch that would make it obvious whether the remaining blocker was still structure or had moved to evidence depth and model quality.

### 方案还差什么
- [ ] `180d` governed direction validation is still unavailable in the current live slices because the future-window rows are insufficient (`required 180, available 168` in the ETF batch).
- [ ] ETF information-side evidence remains thinner than the stock side, so the final chair layer still tends to preserve `needs_more_evidence` semantics even when the quantitative heads are structurally ready.
- [ ] The current multi-head artifacts are still `candidate/shadow` grade rather than champion-grade production signals, so later thickening should focus on evidence depth, live-history coverage, and promotion strength.

### 潜在问题
- [ ] `513500.SH` cross-border live artifacts remain the weakest in the representative batch, so cross-border ETF external-driver history still needs thickening before promotion should ever be considered.
- [ ] Some event heads can now train successfully but are still research-grade in stability; downstream consumers should not read raw probabilities as production-grade action signals.
- [ ] Downstream readers that only look at final action or raw score could still miss the fact that the current abstain outputs are now driven by governance/evidence quality rather than missing model heads.

### 关闭项
- One governed stock and four ETF representative assets now have a full six-head live validation batch.
- The formal final chair layer can consume direction, return, drawdown, path quality, upside-first, and stop-first model outputs on real governed data.
- The current securities bottleneck has moved beyond structural chain breaks and is now mainly about evidence thickness, ETF information semantics, and model promotion quality.

## 2026-04-13 Task - close ETF balanced-scorecard live rerun with long-window slices and formal chair outputs

### 修改内容
- Built one long-window real-data validation slice per ETF pool under `.excel_skill_runtime\\validation_real_data_slices`:
  - `511010_SH_real_data_20260413_long`
  - `518880_SH_real_data_20260413_long`
  - `513500_SH_real_data_20260413_long`
  - `512800_SH_real_data_20260413_long`
- Extended governed ETF replay history through `2026-04-10` and confirmed the latest common analysis date that still supports full `180d` replay is `2025-07-11`.
- Trained a complete six-head ETF live batch under `.excel_skill_runtime\\balanced_scorecard_live_complete_20260413` for:
  - `treasury_etf`
  - `gold_etf`
  - `cross_border_etf`
  - `equity_etf`
- Re-ran the last formal ETF decision layer at both:
  - `2025-08-08` to confirm the latest cut still downgrades to `replay_unavailable` for `180d`
  - `2025-07-11` to capture the latest full balanced-scorecard-capable ETF closeout
- Persisted the ETF final-chain summary to:
  - `.excel_skill_runtime\\balanced_scorecard_live_complete_20260413\\etf_final_chain_summary.json`
  - `.excel_skill_runtime\\balanced_scorecard_live_complete_20260413\\etf_final_chain_summary_20250711.json`
- Updated the execution note:
  - `docs\\execution-notes-2026-04-13-multi-head-live-validation.md`

### 修改原因
- The user explicitly asked us to continue thickening real data until ETF pools could reach the last formal conclusion layer with an auditable balanced scorecard instead of stopping at snapshot visibility.
- Earlier work had already removed the structural replay break, so the next honest checkpoint was to prove whether ETF pools could now:
  - carry ETF-native information into the scorecard
  - train live six-head artifacts on governed history
  - produce a formal chair output with complete balanced-scorecard components
- We also needed a precise answer to the `180d` question, which required finding the latest ETF date where the future replay window is actually complete.

### 方案还差什么
- [ ] The ETF balanced-scorecard path is now complete through the formal chair layer, but all current ETF artifacts are still `candidate/shadow` grade rather than `champion`.
- [ ] `2025-08-08` still cannot produce a complete `180d` governed output because the live slices only have `160` future rows beyond that date; later waves need even longer live windows if we want full latest-date 180-day validation.
- [ ] ETF information semantics are now structurally available from governed proxy history, but later waves still need thicker ETF-native research evidence so committee/risk governance can move beyond conservative `avoid`.

### 潜在问题
- [ ] `cross_border_etf` remains the weakest live pool even after long-window retraining, so later thickening should bias toward overseas-driver history rather than only more price rows.
- [ ] Some ETF balanced-scorecard components can look constructive while the final chair action remains `avoid`; downstream readers must keep reading governance state instead of raw scores alone.
- [ ] The long-window ETF slices are now sufficient for `2025-07-11` full replay, but future contract changes in slice generation could silently shift the latest `180d`-capable date if we do not keep a dedicated regression around it.

### 关闭项
- The ETF long-window governed replay base is landed and auditable.
- The latest common ETF date that supports full `180d` balanced-scorecard replay is now known and documented as `2025-07-11`.
- Treasury, gold, cross-border, and equity ETF pools now reach the last formal chair layer with `score_status = ready` and non-empty balanced-scorecard component scores.
- The current ETF bottleneck has moved from structure and missing data paths to model grade, evidence thickness, and committee/risk governance quality.

## 2026-04-12 Task - land future 180d prediction mode from master scorecard into chair resolution

### 修改内容
- Extended [security_master_scorecard.rs](D:/Rust/Excel_Skill/src/ops/security_master_scorecard.rs) with governed `prediction_mode` inputs and a future-looking `prediction_summary` payload for `180d` use cases.
- Added prediction-mode contract coverage in:
  - `tests\\security_master_scorecard_cli.rs`
  - `tests\\security_chair_resolution_cli.rs`
- Extended [security_chair_resolution.rs](D:/Rust/Excel_Skill/src/ops/security_chair_resolution.rs) so chair requests now forward:
  - `prediction_mode`
  - `prediction_horizon_days`
- Added chair-layer prediction-mode reasoning and execution-constraint output:
  - reasoning now explicitly references `prediction-mode quant context`
  - constraints now include the governed `regime cluster` summary when prediction mode is active
- Added prediction-mode helpers inside the chair layer:
  - `default_prediction_horizon_days`
  - `is_prediction_mode`

### 修改原因
- The user explicitly corrected the product standard from "find a replay-capable historical date" to "predict the next 180 days from 2026-04-12".
- The previous state already had a future-looking builder in `security_master_scorecard`, but the last formal decision layer still consumed replay semantics and therefore could not expose a governed future-facing conclusion.
- This round was required to make regression, risk, and clustering lines visible at the final `security_chair_resolution` layer instead of stopping at the intermediate scorecard object.

### 方案还差什么
- [ ] The current prediction-mode cluster line is still a governed deterministic analog summary rather than a dedicated learned clustering artifact.
- [ ] Prediction mode is now structurally available at the chair layer, but the downstream live-data thickening work is still needed before many assets can graduate beyond `candidate/shadow` governance.
- [ ] We still need a future-looking end-to-end consumption pass on broader live assets to decide which asset classes can already sustain production-grade 180d prediction outputs.

### 潜在问题
- [ ] Downstream consumers might over-read `prediction_summary` as a production release signal even when the model grade is still `candidate/shadow`; they must keep reading governance state together with predicted values.
- [ ] The current cluster analog count is rule-derived from available heads and horizon length, so it is stable and auditable but not yet a learned analog-study output.
- [ ] If later work adds a separate clustering tool or artifact family, the current `future_prediction_v1` summary contract will need a compatibility regression to keep chair reasoning stable.

### 关闭项
- `security_master_scorecard` now supports a governed future-looking `prediction_mode = prediction` path for `180d` requests.
- `security_chair_resolution` now consumes future regression, risk, and clustering context instead of silently falling back to replay-only wording.
- The two new red tests for future `180d` prediction mode are green, and adjacent CLI suites for master-scorecard and chair-resolution are also green.

## 2026-04-12 Task - fix non-trading ETF proxy fallback and rerun same-standard 180d prediction scorecards

### 修改内容
- Added a non-trading-date fallback loader in [security_external_proxy_store.rs](D:/Rust/Excel_Skill/src/runtime/security_external_proxy_store.rs) so governed ETF proxy history can resolve the latest record on or before the requested `as_of_date`.
- Updated [security_external_proxy_backfill.rs](D:/Rust/Excel_Skill/src/ops/security_external_proxy_backfill.rs) to try exact-date proxy lookup first and then fall back to the latest prior trading date when the request lands on weekends or holidays.
- Added a red-to-green regression in [security_chair_resolution_cli.rs](D:/Rust/Excel_Skill/tests/security_chair_resolution_cli.rs) that locks treasury ETF proxy hydration on `2026-04-12` while the governed proxy record itself is stored at `2026-04-10`.
- Extended the governed stock long slice for `601916.SH` to `2026-04-10` and reran the same-standard `prediction_mode = prediction`, `prediction_horizon_days = 180` stack for:
  - `601916.SH`
  - `511010.SH`
  - `518880.SH`
  - `513500.SH`
  - `512800.SH`
- Wrote the rerun summary to [prediction_and_chair_summary.json](D:/Rust/Excel_Skill/.excel_skill_runtime/prediction_mode_rerun_20260412_fixed/prediction_and_chair_summary.json).

### 修改原因
- The user explicitly asked us to stop using replay-capable historical anchor dates and instead analyze from the current time reference (`2026-04-12`) toward the next `180` days.
- Earlier ETF slices already had governed proxy history through `2026-04-10`, but direct future-mode runs on `2026-04-12` still downgraded to incomplete outputs because the proxy loader required an exact date match.
- We needed one honest checkpoint showing whether the repaired ETF information path and the extended stock long slice could now reach the same-standard `master_scorecard + chair_resolution` layer together.

### 方案还差什么
- [ ] ETF pools now carry governed information into the final chain on `2026-04-12`, but their scorecards still land at `feature_incomplete` because at least one feature family remains outside the trained bins, with `integrated_stance = mixed_watch` being a confirmed gap for treasury ETF.
- [ ] The stock side now has a same-standard long slice through `2026-04-10`, but its current `180d` prediction still ends at `abstain`, so later thickening still needs broader evidence and stronger model grade rather than only more price rows.
- [ ] We still need a dedicated ETF scorecard refinement wave so future governed ETF predictions can move from `feature_incomplete` into fully matched `ready` scorecards under prediction mode.

### 潜在问题
- [ ] Downstream readers could misread the rerun as “ETF final chain is fully solved” because governed ETF information is now available at the chair layer; in reality, the remaining bottleneck has shifted into scorecard feature completeness rather than data reachability.
- [ ] The non-trading-date fallback currently uses “latest on or before” semantics, which is correct for weekends and holidays, but future consumers should keep passing explicit `as_of_date` values so replay and prediction behavior remain auditable.
- [ ] The current future-mode summary now combines `master_scorecard` and `chair_resolution` outputs from two separate tool calls; if the chair contract later starts returning master-scorecard inline, this stitched summary path should be simplified to avoid drift.

### 关闭项
- ETF governed proxy information now survives the `2026-04-12 -> 2026-04-10` non-trading-date handoff and reaches the final committee / chair chain.
- The governed long stock slice for `601916.SH` is now extended to `2026-04-10`, so stock and ETF can be rerun under the same `180d` prediction-mode standard.
- A same-standard `180d` prediction summary now exists for the five tracked symbols, including:
  - governed information sources
  - predicted return / drawdown / path-quality values
  - regime cluster labels
  - final chair actions

## 2026-04-12 Task - normalize ETF integrated stance buckets, retrain live artifacts, and verify 180d replay-to-2026-04-10

### 修改内容
- Added ETF-only `integrated_stance` modeling normalization in [security_scorecard.rs](D:/Rust/Excel_Skill/src/ops/security_scorecard.rs), so richer ETF information-layer outputs such as `mixed_watch` and `watchful_positive` are collapsed into stable modeling buckets before scorecard bin matching.
- Added the matching ETF normalization path in [security_scorecard_training.rs](D:/Rust/Excel_Skill/src/ops/security_scorecard_training.rs), so training-time feature extraction now stores the same governed ETF stance buckets that runtime scorecards expect.
- Locked the normalization fix with three focused red-to-green tests:
  - `normalize_integrated_stance_for_modeling_collapses_new_etf_watch_labels`
  - `value_matches_bin_accepts_normalized_etf_watch_bucket`
  - `extract_feature_values_normalizes_etf_integrated_stance_bucket`
- Rebuilt a fresh live batch under [prediction_mode_rerun_20260412_stance_fixed_v2](D:/Rust/Excel_Skill/.excel_skill_runtime/prediction_mode_rerun_20260412_stance_fixed_v2) for:
  - `511010.SH`
  - `518880.SH`
  - `513500.SH`
  - `512800.SH`
  - `601916.SH`
- Trained six heads for every tracked asset in that batch:
  - `direction_head`
  - `return_head`
  - `drawdown_head`
  - `path_quality_head`
  - `upside_first_head`
  - `stop_first_head`
- Reran same-standard `prediction_mode = prediction`, `prediction_horizon_days = 180` outputs and wrote the governed summary to [prediction_and_chair_summary.json](D:/Rust/Excel_Skill/.excel_skill_runtime/prediction_mode_rerun_20260412_stance_fixed_v2/prediction_and_chair_summary.json).
- Added the requested 180-day lookback validation toward `2026-04-10` and wrote it to [replay_180d_to_2026_04_10_summary.json](D:/Rust/Excel_Skill/.excel_skill_runtime/prediction_mode_rerun_20260412_stance_fixed_v2/replay_180d_to_2026_04_10_summary.json).

### 修改原因
- The previous governed ETF information thickening wave had already made ETF proxy evidence visible at the final chair layer, but ETF scorecards still degraded to `feature_incomplete`.
- We isolated the concrete blocker to the ETF `integrated_stance` vocabulary drifting ahead of the older ETF artifacts: runtime was emitting `mixed_watch` / `watchful_positive`, while the trained bins still only recognized older coarse stance values.
- This round was required to make ETF scorecards consume the richer ETF information semantics without silently breaking feature coverage, and then to verify both:
  - future-looking `2026-04-12 -> next 180 days` predictions
  - historical `180 days before -> realized at 2026-04-10` replay checks

### 方案还差什么
- [ ] The ETF scorecards are now `ready`, but all four ETF pools still end at `avoid`, so the next thickening wave must target evidence depth and model-grade uplift rather than feature-plumbing repairs.
- [ ] `601916.SH` now reaches `score_status = ready` in the same future-looking standard, but its governed final action is still conservative, so stock-side champion promotion remains a separate next-step problem.
- [ ] We still need broader live-data thickening for ETF-native information semantics and longer-horizon evidence so future `180d` outputs can move from `candidate/shadow` support toward production-grade governance.

## 2026-04-12 资产池预测校准与长窗口复核

### 修改文件
- [src/ops/security_master_scorecard.rs](D:/Rust/Excel_Skill/src/ops/security_master_scorecard.rs)

### 修改内容
- Added pool-level prediction calibration profiles for:
  - `treasury_etf`
  - `gold_etf`
  - `cross_border_etf`
  - `equity_etf`
  - default equity/stock fallback
- Routed `prediction_mode = prediction` through asset-pool-specific:
  - `target_return_pct`
  - `stop_loss_pct`
  - profitability/risk/path weights
- Kept explicit non-default request thresholds overrideable while still preserving pool-level scoring weights.
- Added unit coverage for:
  - treasury calibration defaults
  - equity fallback defaults
  - calibrated master-score weighting behavior

### 修改原因
- The previous prediction-mode scoring reused the same `12% / 5% / 45-35-20` defaults across treasury, gold, cross-border, equity ETF, and stocks.
- That setup risked overfitting decision semantics to a single narrow calibration regime, especially for low-volatility treasury ETFs and high-path-variance gold/cross-border ETFs.
- This round was required to move from symbol-like one-size-fits-all thresholds toward stable pool-level calibration so future reruns evaluate economically similar assets under comparable rules rather than reusing mismatched hurdles.

### 验证
- `cargo test --lib security_master_scorecard -- --nocapture`
- `cargo test --test security_master_scorecard_cli security_master_scorecard_supports_prediction_mode_180d -- --nocapture`
- `cargo test --test security_chair_resolution_cli security_chair_resolution_reads_prediction_mode_180d_context -- --nocapture`

### 方案还差什么
- [ ] The long-slice `2026-04-12 -> 180d` rerun still exposes a new ETF stance bucket (`watchful_context`) that is not yet covered by ETF scorecard bins, so long-slice production-style reruns remain blocked on one more stance-normalization repair.
- [ ] The old pool-calibrated rerun summary under `.excel_skill_runtime/prediction_mode_rerun_20260412_pool_calibrated_v1` should not be treated as the final `2026-04-10` answer because it still used the shorter `2025-08-08` validation slices.
- [ ] We still need a full long-slice rerun for all target ETFs after the `watchful_context` repair before comparing calibrated prediction outputs against realized `2026-04-10` outcomes.

### 潜在问题
- [ ] Pool-level calibration reduces per-symbol overfitting pressure, but it does not solve missing stance-bin coverage; if runtime emits new ETF stance labels faster than artifacts are updated, prediction chains can still degrade to `feature_incomplete`.
- [ ] Prediction-mode calibration currently affects future-looking scoring only; replay-mode historical summaries will remain on their previous scoring regime unless we explicitly align them later.
- [ ] If later we promote a true ETF-specific stance artifact family, the current normalization-backed calibration layer may need compatibility coverage to avoid silent score drift.

### 潜在问题
- [ ] The ETF stance normalization currently collapses multiple richer information-layer values into a smaller governed modeling vocabulary; if later ETF product semantics need to distinguish those buckets economically, we should extend artifact families rather than keep overloading the same normalized bucket forever.
- [ ] Prediction summaries now look much cleaner because `score_status = ready`, but downstream readers still must not treat `avoid` + low-governance grades as deployable long signals.
- [ ] The 180-day replay summary is anchored at `2025-07-11` because that is the latest common date with a full 180-day realized window to `2026-04-10`; if later live slices extend further, that anchor date will shift and should stay under explicit regression coverage.

### 关闭项
- The four ETF pools no longer degrade to `feature_incomplete` under the `2026-04-12 -> 180d` future-looking standard; they now all reach `score_status = ready`.
- The new same-standard 180-day future-looking summary now reads:
  - `511010.SH`: `ready`, `master_score = 58.14`, `selected_action = avoid`
  - `518880.SH`: `ready`, `master_score = 56.08`, `selected_action = avoid`
  - `513500.SH`: `ready`, `master_score = 26.37`, `selected_action = avoid`
  - `512800.SH`: `ready`, `master_score = 24.42`, `selected_action = avoid`
  - `601916.SH`: `ready`, `master_score = 46.39`, `selected_action = avoid`
- The requested `180 days before -> realized at 2026-04-10` replay validation is now recorded for the same five assets, including realized return, max drawdown, max runup, and path-event hits.

## 2026-04-12 ETF 池级修复收口与 latest chair 对照

### 修改文件
- [`.excel_skill_runtime/pool_training_fix_20260412/513180_SH_latest_chair_summary.json`](D:/Rust/Excel_Skill/.excel_skill_runtime/pool_training_fix_20260412/513180_SH_latest_chair_summary.json)
- [`.excel_skill_runtime/pool_training_fix_20260412/515790_SH_latest_chair_summary.json`](D:/Rust/Excel_Skill/.excel_skill_runtime/pool_training_fix_20260412/515790_SH_latest_chair_summary.json)
- [`.excel_skill_runtime/pool_training_fix_20260412/latest_chair_pool_summary.json`](D:/Rust/Excel_Skill/.excel_skill_runtime/pool_training_fix_20260412/latest_chair_pool_summary.json)

### 修改内容
- Ran latest `security_chair_resolution` summaries for the pooled-training holdout symbols:
  - `513180.SH` under `cross_border_etf`
  - `515790.SH` under `equity_etf_peer`
- Bound the latest runs to:
  - slice-local `stock_history.db`
  - slice-local governed `security_fundamental_history.db`
  - slice-local governed `security_disclosure_history.db`
  - shared governed `security_external_proxy.db`
- Used the pooled `30d` artifacts produced under:
  - `cross_border_training`
  - `equity_training`
- Wrote both per-symbol summaries and a combined rollup file for easier follow-up debugging.

### 修改原因
- The earlier pooled holdout validation already proved that `ready_rate` improved materially, but the session still lacked a like-for-like `latest chair` summary for `515790.SH`.
- This round closes that gap so the next optimization wave can compare:
  - historical pooled holdout improvements
  - latest-date governance behavior
  without manually reconstructing JSON from multiple runtime artifacts.

### 验证
- Executed live `security_chair_resolution` runs for:
  - `513180.SH`
  - `515790.SH`
- Confirmed both summaries were persisted to:
  - [`513180_SH_latest_chair_summary.json`](D:/Rust/Excel_Skill/.excel_skill_runtime/pool_training_fix_20260412/513180_SH_latest_chair_summary.json)
  - [`515790_SH_latest_chair_summary.json`](D:/Rust/Excel_Skill/.excel_skill_runtime/pool_training_fix_20260412/515790_SH_latest_chair_summary.json)
  - [`latest_chair_pool_summary.json`](D:/Rust/Excel_Skill/.excel_skill_runtime/pool_training_fix_20260412/latest_chair_pool_summary.json)

### 结果摘要
- `513180.SH`
  - `analysis_date = 2026-04-10`
  - `selected_action = avoid`
  - `score_status = feature_incomplete`
  - `expected_return ≈ 1.75%`
  - `expected_drawdown ≈ 8.57%`
- `515790.SH`
  - `analysis_date = 2026-04-10`
  - `selected_action = avoid`
  - `score_status = feature_incomplete`
  - `expected_return ≈ 0.59%`
  - `expected_drawdown ≈ 7.83%`

### 方案还差什么
- [ ] The pooled holdout path fixes materially improved historical `ready_rate`, but latest-date `chair_resolution` still degrades to `feature_incomplete`, so the next debugging pass must isolate which runtime feature bins still miss on `2026-04-10`.
- [ ] `cross_border_etf` remains the weaker repair: latest governance is still conservative and its pooled holdout `group_X_points` is still `0.0`.
- [ ] `equity_etf` shows meaningful holdout improvement, but the latest-date governance still does not accept it as fully ready, so latest runtime feature coverage remains incomplete.

### 潜在问题
- [ ] The latest chair summaries are generated with live runtime calls rather than test fixtures, so if slice-local governed history or proxy bindings change later, these summaries will need regeneration.
- [ ] The latest-date degradation proves that pooled holdout readiness and latest runtime readiness are not yet identical states; downstream readers should not assume that a high holdout `ready_rate` guarantees a `ready` latest chair output.

## 2026-04-12 ETF 池级代理历史自动导入与 latest ready 修复
### 修改内容
- Added red tests in [`tests/security_real_data_validation_backfill_cli.rs`](D:/Rust/Excel_Skill/tests/security_real_data_validation_backfill_cli.rs) to lock:
  - ETF validation slices auto-import pool proxy history into governed external proxy storage.
  - ETF validation slices fail fast when required pool proxy history is missing.
- Updated [`src/ops/security_real_data_validation_backfill.rs`](D:/Rust/Excel_Skill/src/ops/security_real_data_validation_backfill.rs) so ETF validation requests now:
  - auto-discover pool proxy CSV files under the governed runtime root,
  - import them through the formal `security_external_proxy_history_import` path,
  - persist `external_proxy_db_path` and `external_proxy_import_result_paths`,
  - fail explicitly when an ETF slice still has no governed proxy history on or before `end_date`.
- Repaired planning memory files to UTF-8 and refreshed:
  - [`task_plan.md`](D:/Rust/Excel_Skill/task_plan.md)
  - [`findings.md`](D:/Rust/Excel_Skill/findings.md)
  - [`progress.md`](D:/Rust/Excel_Skill/progress.md)
- Imported the real pool proxy batches:
  - [`cross_border_pool_proxy_history.csv`](D:/Rust/Excel_Skill/.excel_skill_runtime/pool_training_fix_20260412/cross_border_pool_proxy_history.csv)
  - [`equity_pool_proxy_history.csv`](D:/Rust/Excel_Skill/.excel_skill_runtime/pool_training_fix_20260412/equity_pool_proxy_history.csv)
  into the shared governed proxy store:
  - [`security_external_proxy.db`](D:/Rust/Excel_Skill/.excel_skill_runtime/security_external_proxy.db)
- Re-ran latest `security_chair_resolution` for:
  - `513180.SH`
  - `515790.SH`
  and wrote refreshed raw/summary artifacts:
  - [`513180_SH_latest_chair_raw_after_proxy_import.json`](D:/Rust/Excel_Skill/.excel_skill_runtime/pool_training_fix_20260412/513180_SH_latest_chair_raw_after_proxy_import.json)
  - [`513180_SH_latest_chair_summary_after_proxy_import.json`](D:/Rust/Excel_Skill/.excel_skill_runtime/pool_training_fix_20260412/513180_SH_latest_chair_summary_after_proxy_import.json)
  - [`515790_SH_latest_chair_raw_after_proxy_import.json`](D:/Rust/Excel_Skill/.excel_skill_runtime/pool_training_fix_20260412/515790_SH_latest_chair_raw_after_proxy_import.json)
  - [`515790_SH_latest_chair_summary_after_proxy_import.json`](D:/Rust/Excel_Skill/.excel_skill_runtime/pool_training_fix_20260412/515790_SH_latest_chair_summary_after_proxy_import.json)
  - [`latest_chair_pool_summary_after_proxy_import.json`](D:/Rust/Excel_Skill/.excel_skill_runtime/pool_training_fix_20260412/latest_chair_pool_summary_after_proxy_import.json)

### 修改原因
- The previous fix removed the `as_of_date` hydration bug, but latest ETF reruns still degraded to `feature_incomplete` because the pool-generated proxy CSVs never entered the governed external proxy store.
- Summary files had started to drift away from the raw latest chair payloads, so this round needed to fix both the import path and the “missing data should fail loudly” guardrail.

### 方案还差什么
- [ ] Continue the larger bundle only after confirming the broader “can use for live trading” acceptance gate, because this round only closes the ETF proxy-history import gap.
- [ ] Re-run the next pooled/latest comparison wave for additional ETF pools once more latest symbols are prepared, so the same auto-import contract is exercised beyond `513180.SH` and `515790.SH`.

### 潜在问题
- [ ] Auto-discovery currently keys off deterministic pool CSV file names plus symbol presence; if future pool jobs rename those artifacts, validation slices will fail fast until the discovery map is updated.
- [ ] Latest raw is now `ready`, but both symbols still resolve to `selected_action = abstain`; this fix removes placeholder drift, not the broader governance threshold.
- [ ] The shared governed proxy store now contains imported pool batches; if later debugging needs a pristine store, operators should isolate it with `EXCEL_SKILL_EXTERNAL_PROXY_DB`.

### 关闭项
- `security_real_data_validation_backfill` no longer returns a misleading ETF success result when pool proxy history is missing.
- `513180.SH` latest raw now resolves:
  - `score_status = ready`
  - `fx_proxy_status = manual_bound`
  - `overseas_market_proxy_status = manual_bound`
  - `market_session_gap_status = manual_bound`
- `515790.SH` latest raw now resolves:
  - `score_status = ready`
  - `etf_fund_flow_proxy_status = manual_bound`
  - `premium_discount_proxy_status = manual_bound`
  - `benchmark_relative_strength_status = manual_bound`
- latest summary and raw are now aligned through [`latest_chair_pool_summary_after_proxy_import.json`](D:/Rust/Excel_Skill/.excel_skill_runtime/pool_training_fix_20260412/latest_chair_pool_summary_after_proxy_import.json).

## 2026-04-12 GitHub 上传收口（证券主链方案C）
### 修改内容
- Added upload closeout notes for the securities mainline branch:
  - [`docs/execution-notes-2026-04-12-github-upload-securities-mainline.md`](D:/Rust/Excel_Skill/docs/execution-notes-2026-04-12-github-upload-securities-mainline.md)
- Added a dedicated AI handoff note that avoids the unresolved conflict file:
  - [`docs/ai-handoff-2026-04-12-securities-mainline-upload.md`](D:/Rust/Excel_Skill/docs/ai-handoff-2026-04-12-securities-mainline-upload.md)
- Prepared the upload boundary around the securities/stock mainline on branch:
  - `codex/etf-proxy-import-latest-ready-20260412`
- Explicitly excluded:
  - `docs/AI_HANDOFF.md`
  - unrelated `foundation` dirty files
  - regenerated runtime fixture bulk directories

### 修改原因
- The repository worktree is heavily dirty, so a direct upload would mix unrelated changes and an unresolved conflict into the GitHub branch.
- This upload round needed a factual execution note plus a fresh handoff note so the branch can be reviewed and continued safely.

### 方案还差什么
- [ ] Stage the final securities-mainline file set, create the commit, and push the branch to GitHub.
- [ ] Keep verifying that no unrelated foundation/conflict files were included before the final upload.

### 潜在问题
- [ ] `docs/AI_HANDOFF.md` remains unresolved and must stay out of the upload until the conflict is handled separately.
- [ ] Because the workspace contains many unrelated changes, any broad `git add` would still risk contaminating the branch if the boundary check is skipped.

### 关闭项
- A dedicated upload execution note now exists for this branch.
- A dedicated AI handoff note now exists without touching the conflicted handoff file.
