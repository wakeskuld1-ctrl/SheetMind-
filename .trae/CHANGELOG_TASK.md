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
