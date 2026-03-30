# SheetMind Desktop GUI Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在不破坏现有 CLI/Tool 协议主链的前提下，为 `SheetMind` 增加首发可售卖的本地桌面 GUI 壳，并先打通“授权 -> 工作台 -> 文件与表 -> 数据处理骨架 -> 分析骨架 -> 导出骨架”的主路径。

**Architecture:** 继续保留 `src/main.rs` 作为现有 CLI 引擎入口，新增独立 GUI 二进制入口承载桌面界面。GUI 通过本地桥接层调用现有 Rust Tool Contract，不直接侵入业务计算层；状态、授权、结果引用优先复用当前 runtime 目录与授权服务。

**Tech Stack:** Rust 2024、`eframe/egui`、`rfd`、现有 `serde/serde_json`、现有 `rusqlite`、现有 `LicenseService`、现有 `ToolRequest/ToolResponse`

---

## 实施前提

本计划默认以下技术决策成立：

- GUI 框架选用 `eframe/egui`
- 继续保留当前 CLI 二进制入口
- 新增 GUI 二进制而不是把 GUI 混入 `src/main.rs`
- GUI 首版优先实现页面骨架、导航、状态联通与关键流程，不在第一批追求完整图表细节

如果后续明确改用别的 Rust GUI 框架，需要整体重写此计划。

---

### Task 1: 为 GUI 增加独立二进制入口与依赖骨架

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\Cargo.toml`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\lib.rs`
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\bin\sheetmind_app.rs`
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\mod.rs`
- Test: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\gui_bootstrap_cli.rs`

**Step 1: 写失败测试，锁定 GUI 二进制可启动的最小契约**

```rust
use assert_cmd::Command;

#[test]
fn sheetmind_app_help_or_bootstrap_runs() {
    let mut cmd = Command::cargo_bin("sheetmind_app").unwrap();
    cmd.arg("--help");
    cmd.assert().success();
}
```

**Step 2: 运行测试确认失败**

Run: `cargo test --test gui_bootstrap_cli sheetmind_app_help_or_bootstrap_runs -- --nocapture`  
Expected: FAIL，提示找不到 `sheetmind_app` 二进制。

**Step 3: 添加 GUI 依赖与独立入口**

- 在 `Cargo.toml` 中增加 `eframe`、`egui_extras`、`rfd`。
- 新建 `src/bin/sheetmind_app.rs`。
- 在 `src/lib.rs` 暴露 `pub mod gui;`。
- 在 `src/gui/mod.rs` 仅保留最小导出。

**Step 4: 为 GUI 入口实现最小可启动逻辑**

```rust
fn main() -> eframe::Result<()> {
    Ok(())
}
```

后续再替换为真正窗口启动。

**Step 5: 再跑测试确认转绿**

Run: `cargo test --test gui_bootstrap_cli sheetmind_app_help_or_bootstrap_runs -- --nocapture`  
Expected: PASS

**Step 6: 小步提交**

```bash
git add Cargo.toml src/lib.rs src/bin/sheetmind_app.rs src/gui/mod.rs tests/gui_bootstrap_cli.rs
git commit -m "feat: add desktop gui bootstrap entry"
```

---

### Task 2: 建立 GUI 应用状态模型与页面枚举

**Files:**
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\app.rs`
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\state.rs`
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\theme.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\mod.rs`
- Test: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\gui_state_navigation.rs`

**Step 1: 写失败测试，锁定主导航状态切换**

```rust
use excel_skill::gui::state::{AppPage, AppState};

#[test]
fn app_state_can_switch_pages() {
    let mut state = AppState::default();
    state.set_page(AppPage::AnalysisModeling);
    assert_eq!(state.current_page(), AppPage::AnalysisModeling);
}
```

**Step 2: 运行测试确认失败**

Run: `cargo test --test gui_state_navigation app_state_can_switch_pages -- --nocapture`  
Expected: FAIL，提示 `gui::state` 或相关类型不存在。

**Step 3: 建立最小状态结构**

需要包含：

- `AppPage`
- `AppState`
- 当前页面
- 当前项目名
- 当前文件名
- 授权状态摘要
- 当前数据集摘要
- 右侧消息区内容

**Step 4: 在 `app.rs` 建立 GUI 应用壳**

至少包含：

- 顶栏渲染函数
- 左侧导航渲染函数
- 中央区域占位渲染函数
- 右侧上下文区渲染函数

**Step 5: 运行测试确认转绿**

Run: `cargo test --test gui_state_navigation app_state_can_switch_pages -- --nocapture`  
Expected: PASS

**Step 6: 小步提交**

```bash
git add src/gui/mod.rs src/gui/app.rs src/gui/state.rs src/gui/theme.rs tests/gui_state_navigation.rs
git commit -m "feat: add gui app state and navigation skeleton"
```

---

### Task 3: 打通授权状态桥接，优先复用现有 LicenseService

**Files:**
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\bridge\mod.rs`
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\bridge\license_bridge.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\mod.rs`
- Test: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\gui_license_bridge.rs`

**Step 1: 写失败测试，锁定 GUI 可读取授权摘要**

```rust
use excel_skill::gui::bridge::license_bridge::LicenseSummary;

#[test]
fn license_summary_defaults_to_unlicensed() {
    let summary = LicenseSummary::default();
    assert!(!summary.licensed);
}
```

**Step 2: 运行测试确认失败**

Run: `cargo test --test gui_license_bridge license_summary_defaults_to_unlicensed -- --nocapture`  
Expected: FAIL

**Step 3: 实现授权桥接层**

桥接层职责：

- 从现有 `LicenseService` 获取状态
- 将技术字段转换成 GUI 视图可用摘要
- 不在 GUI 中复制授权业务逻辑

`LicenseSummary` 至少包含：

- `licensed`
- `status_text`
- `license_email`
- `last_validated_at`
- `device_status`

**Step 4: 把授权摘要接入 `AppState`**

- GUI 启动时尝试读取当前授权状态
- 顶栏与“授权与设置”页共享同一份摘要

**Step 5: 运行测试确认转绿**

Run: `cargo test --test gui_license_bridge license_summary_defaults_to_unlicensed -- --nocapture`  
Expected: PASS

**Step 6: 小步提交**

```bash
git add src/gui/mod.rs src/gui/bridge/mod.rs src/gui/bridge/license_bridge.rs tests/gui_license_bridge.rs
git commit -m "feat: add gui license bridge"
```

---

### Task 4: 建立通用 Tool 执行桥接层，复用 ToolRequest/ToolResponse

**Files:**
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\bridge\tool_runner.rs`
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\bridge\view_models.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\bridge\mod.rs`
- Test: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\gui_tool_runner.rs`

**Step 1: 写失败测试，锁定 GUI 能构造并执行最小 Tool 请求**

```rust
use excel_skill::gui::bridge::tool_runner::ToolRunner;

#[test]
fn tool_runner_can_request_catalog() {
    let runner = ToolRunner::new();
    let response = runner.catalog().unwrap();
    assert!(response.success);
}
```

**Step 2: 运行测试确认失败**

Run: `cargo test --test gui_tool_runner tool_runner_can_request_catalog -- --nocapture`  
Expected: FAIL

**Step 3: 实现桥接层**

桥接层职责：

- 用现有 `ToolRequest` 组织请求
- 复用现有 `dispatch()` 或明确封装一层本地调用函数
- 将 `ToolResponse` 转成 GUI 视图层可用结构

首批方法至少包含：

- `catalog()`
- `open_workbook(...)`
- `list_sheets(...)`
- `preview_table(...)`
- `license_status(...)`

**Step 4: 运行测试确认转绿**

Run: `cargo test --test gui_tool_runner tool_runner_can_request_catalog -- --nocapture`  
Expected: PASS

**Step 5: 小步提交**

```bash
git add src/gui/bridge/mod.rs src/gui/bridge/tool_runner.rs src/gui/bridge/view_models.rs tests/gui_tool_runner.rs
git commit -m "feat: add gui tool runner bridge"
```

---

### Task 5: 先完成工作台页骨架

**Files:**
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\pages\mod.rs`
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\pages\dashboard.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\app.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\mod.rs`
- Test: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\gui_dashboard_state.rs`

**Step 1: 写失败测试，锁定工作台默认可渲染状态对象**

```rust
use excel_skill::gui::state::AppState;

#[test]
fn dashboard_state_exposes_quick_actions() {
    let state = AppState::default();
    assert!(!state.quick_actions().is_empty());
}
```

**Step 2: 运行测试确认失败**

Run: `cargo test --test gui_dashboard_state dashboard_state_exposes_quick_actions -- --nocapture`  
Expected: FAIL

**Step 3: 实现工作台页**

最小展示块：

- 最近项目
- 快速开始
- 推荐任务模板
- 最近导出占位
- 当前授权状态摘要

**Step 4: 运行测试确认转绿**

Run: `cargo test --test gui_dashboard_state dashboard_state_exposes_quick_actions -- --nocapture`  
Expected: PASS

**Step 5: 小步提交**

```bash
git add src/gui/mod.rs src/gui/app.rs src/gui/pages/mod.rs src/gui/pages/dashboard.rs tests/gui_dashboard_state.rs
git commit -m "feat: add dashboard page skeleton"
```

---

### Task 6: 完成“文件与表”页主链骨架

**Files:**
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\pages\files.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\bridge\tool_runner.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\state.rs`
- Test: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\gui_files_flow.rs`

**Step 1: 写失败测试，锁定文件页状态能保存 workbook 与 sheet 列表**

```rust
use excel_skill::gui::state::FilesPageState;

#[test]
fn files_page_state_can_store_selected_sheet() {
    let mut state = FilesPageState::default();
    state.selected_sheet = Some("Sheet1".to_string());
    assert_eq!(state.selected_sheet.as_deref(), Some("Sheet1"));
}
```

**Step 2: 运行测试确认失败**

Run: `cargo test --test gui_files_flow files_page_state_can_store_selected_sheet -- --nocapture`  
Expected: FAIL

**Step 3: 实现文件页状态与桥接**

需要打通：

- 文件选择
- workbook 打开
- sheet 列表读取
- 表区域预览占位
- 表头确认动作入口占位

**Step 4: 在文件页中呈现三栏**

- 左：文件 / sheet / 表列表
- 中：预览区
- 右：字段结构与确认操作

**Step 5: 运行测试确认转绿**

Run: `cargo test --test gui_files_flow files_page_state_can_store_selected_sheet -- --nocapture`  
Expected: PASS

**Step 6: 小步提交**

```bash
git add src/gui/pages/files.rs src/gui/bridge/tool_runner.rs src/gui/state.rs tests/gui_files_flow.rs
git commit -m "feat: add files and table confirmation page skeleton"
```

---

### Task 7: 完成“数据处理”页骨架与操作历史模型

**Files:**
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\pages\data_processing.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\state.rs`
- Test: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\gui_data_processing_state.rs`

**Step 1: 写失败测试，锁定处理页操作历史可累计**

```rust
use excel_skill::gui::state::DataProcessingState;

#[test]
fn data_processing_state_tracks_history() {
    let mut state = DataProcessingState::default();
    state.push_history("筛选: 地区=华东");
    assert_eq!(state.history.len(), 1);
}
```

**Step 2: 运行测试确认失败**

Run: `cargo test --test gui_data_processing_state data_processing_state_tracks_history -- --nocapture`  
Expected: FAIL

**Step 3: 实现处理页状态**

至少包含：

- 当前分组
- 当前选中操作
- 参数面板状态
- 预览占位
- 操作历史

**Step 4: 实现处理页骨架**

必须包含：

- 六组功能入口
- 中部预览区
- 右侧参数区
- 底部或侧边历史区

**Step 5: 运行测试确认转绿**

Run: `cargo test --test gui_data_processing_state data_processing_state_tracks_history -- --nocapture`  
Expected: PASS

**Step 6: 小步提交**

```bash
git add src/gui/pages/data_processing.rs src/gui/state.rs tests/gui_data_processing_state.rs
git commit -m "feat: add data processing page skeleton"
```

---

### Task 8: 完成“分析建模”页骨架与任务导向入口

**Files:**
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\pages\analysis.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\state.rs`
- Test: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\gui_analysis_state.rs`

**Step 1: 写失败测试，锁定分析页存在任务导向分组**

```rust
use excel_skill::gui::state::AnalysisTaskKind;

#[test]
fn analysis_task_kinds_include_modeling() {
    assert!(matches!(AnalysisTaskKind::Modeling, AnalysisTaskKind::Modeling));
}
```

**Step 2: 运行测试确认失败**

Run: `cargo test --test gui_analysis_state analysis_task_kinds_include_modeling -- --nocapture`  
Expected: FAIL

**Step 3: 实现分析页状态与分组**

至少包含：

- 数据概览
- 质量诊断
- 关系分析
- 趋势分析
- 模型分析
- 决策建议

**Step 4: 实现分析页骨架**

必须呈现：

- 顶部任务切换
- 参数区
- 结果区
- 图表占位区
- 风险与解释区

**Step 5: 运行测试确认转绿**

Run: `cargo test --test gui_analysis_state analysis_task_kinds_include_modeling -- --nocapture`  
Expected: PASS

**Step 6: 小步提交**

```bash
git add src/gui/pages/analysis.rs src/gui/state.rs tests/gui_analysis_state.rs
git commit -m "feat: add analysis and modeling page skeleton"
```

---

### Task 9: 完成“报告导出”与“AI 助手”页骨架

**Files:**
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\pages\reports.rs`
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\pages\ai.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\state.rs`
- Test: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\gui_reports_ai_state.rs`

**Step 1: 写失败测试，锁定导出模板与 AI 建议占位存在**

```rust
use excel_skill::gui::state::{AiState, ReportsState};

#[test]
fn reports_and_ai_states_have_defaults() {
    assert!(!ReportsState::default().templates.is_empty());
    assert!(AiState::default().suggestions.is_empty());
}
```

**Step 2: 运行测试确认失败**

Run: `cargo test --test gui_reports_ai_state reports_and_ai_states_have_defaults -- --nocapture`  
Expected: FAIL

**Step 3: 实现导出页状态**

至少包含：

- 模板列表
- 输出路径
- 导出格式
- 最近导出记录

**Step 4: 实现 AI 页状态**

至少包含：

- 当前上下文摘要
- 推荐动作列表
- 拟执行动作占位

**Step 5: 实现两个页面骨架**

**Step 6: 运行测试确认转绿**

Run: `cargo test --test gui_reports_ai_state reports_and_ai_states_have_defaults -- --nocapture`  
Expected: PASS

**Step 7: 小步提交**

```bash
git add src/gui/pages/reports.rs src/gui/pages/ai.rs src/gui/state.rs tests/gui_reports_ai_state.rs
git commit -m "feat: add reports and ai page skeletons"
```

---

### Task 10: 完成“授权与设置”页，并串起顶栏授权状态

**Files:**
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\pages\license.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\app.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\gui\state.rs`
- Test: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\gui_license_page_state.rs`

**Step 1: 写失败测试，锁定授权页可显示摘要状态**

```rust
use excel_skill::gui::state::AppState;

#[test]
fn app_state_exposes_license_status_text() {
    let state = AppState::default();
    assert!(!state.license_status_text().is_empty());
}
```

**Step 2: 运行测试确认失败**

Run: `cargo test --test gui_license_page_state app_state_exposes_license_status_text -- --nocapture`  
Expected: FAIL

**Step 3: 实现授权与设置页**

最小模块：

- 当前授权状态
- 激活入口占位
- 刷新状态
- 解绑入口占位
- 本地设置占位

**Step 4: 把授权状态接入顶栏**

- 顶栏显示“已授权 / 未授权 / 校验异常”
- 与页面内摘要使用同一状态源

**Step 5: 运行测试确认转绿**

Run: `cargo test --test gui_license_page_state app_state_exposes_license_status_text -- --nocapture`  
Expected: PASS

**Step 6: 小步提交**

```bash
git add src/gui/pages/license.rs src/gui/app.rs src/gui/state.rs tests/gui_license_page_state.rs
git commit -m "feat: add license and settings page skeleton"
```

---

### Task 11: 做 GUI 冒烟验证，确保不破坏现有 CLI 主链

**Files:**
- Test: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\gui_smoke.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\docs\plans\2026-03-29-sheetmind-desktop-gui-design.md`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\docs\plans\2026-03-29-sheetmind-desktop-gui-implementation.md`

**Step 1: 写 GUI 冒烟测试**

覆盖以下最小断言：

- GUI 二进制可构建
- 默认应用状态可初始化
- 顶栏 / 导航 / 中心页路由可创建
- Tool runner 可请求 catalog
- 现有 CLI 测试样例不需要改协议

**Step 2: 运行 GUI 冒烟测试**

Run: `cargo test --test gui_smoke -- --nocapture`  
Expected: PASS

**Step 3: 运行现有关键回归**

Run: `cargo test --test integration_tool_contract -- --nocapture`  
Expected: PASS

Run: `cargo test --test integration_cli_json cli_without_args_returns_json_help -- --nocapture`  
Expected: PASS

Run: `cargo test --test license_cli -- --nocapture`  
Expected: PASS

**Step 4: 根据实际结果回填文档**

- 若 GUI 框架、入口名或文件路径与计划不同，要同步更新文档。
- 若发现额外桥接层需求，要补到设计文档与计划文档。

**Step 5: 小步提交**

```bash
git add tests/gui_smoke.rs docs/plans/2026-03-29-sheetmind-desktop-gui-design.md docs/plans/2026-03-29-sheetmind-desktop-gui-implementation.md
git commit -m "test: add gui smoke coverage and sync docs"
```

---

## 测试总表

建议至少按以下顺序执行：

1. `cargo test --test gui_bootstrap_cli -- --nocapture`
2. `cargo test --test gui_state_navigation -- --nocapture`
3. `cargo test --test gui_license_bridge -- --nocapture`
4. `cargo test --test gui_tool_runner -- --nocapture`
5. `cargo test --test gui_dashboard_state -- --nocapture`
6. `cargo test --test gui_files_flow -- --nocapture`
7. `cargo test --test gui_data_processing_state -- --nocapture`
8. `cargo test --test gui_analysis_state -- --nocapture`
9. `cargo test --test gui_reports_ai_state -- --nocapture`
10. `cargo test --test gui_license_page_state -- --nocapture`
11. `cargo test --test gui_smoke -- --nocapture`
12. `cargo test --test integration_tool_contract -- --nocapture`
13. `cargo test --test integration_cli_json cli_without_args_returns_json_help -- --nocapture`
14. `cargo test --test license_cli -- --nocapture`

---

## 风险提醒

- `eframe/egui` 接入后会拉入新的构建依赖，首次编译时间会明显增加。
- GUI 若直接在主线程执行耗时 Tool，容易造成界面卡顿；后续需要补异步执行或任务队列。
- 文件选择器、窗口生命周期、字体加载、中文排版都可能在 Windows 下暴露新问题。
- GUI 首批应先重视主流程和状态一致性，不应在第一批陷入图表美化和复杂动画。

---

## 完成标准

满足以下条件才可认为 GUI 第一阶段完成：

- CLI 主链继续可用，现有协议无回归。
- GUI 二进制能稳定启动。
- 七个一级页面均有骨架与状态联通。
- 授权摘要、文件导入、基础 Tool 调用桥接已经打通。
- 关键 GUI 状态测试与现有回归测试均通过。

---

Plan complete and saved to `docs/plans/2026-03-29-sheetmind-desktop-gui-implementation.md`. Two execution options:

**1. Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

**2. Parallel Session (separate)** - Open new session with executing-plans, batch execution with checkpoints

Which approach?
## Task 11 实际结果补充

- 已新增 `tests/gui_smoke.rs`，覆盖 GUI 二进制启动、`SheetMindApp::new()` 初始化、导航契约读取和 `ToolRunner::catalog()` 调用。
- 已补充 `SheetMindApp::navigation_items()` 与 `SheetMindApp::page_title(...)`，作为 GUI 壳的最小可测契约。
- 已完成关键回归验证：
  - `cargo test --test gui_smoke -- --nocapture`
  - `cargo test --test integration_tool_contract -- --nocapture`
  - `cargo test --test integration_cli_json cli_without_args_returns_json_help -- --nocapture`
  - `cargo test --test license_cli -- --nocapture`
