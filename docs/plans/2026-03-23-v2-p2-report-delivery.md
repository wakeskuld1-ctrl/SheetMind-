# V2-P2 Report Delivery Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 新增 `report_delivery` Tool，用独立 `report_delivery.rs` 模块建立结果交付层第一轮最小闭环，并在同一壳层内完成图表第一版增强：标准汇报模板 workbook 草稿、`workbook_ref` 句柄、真实 Excel 图表写入与 Excel 导出衔接。

**Architecture:** dispatcher 负责解析 `summary` / `analysis` / `charts` 输入并装载 `LoadedTable`，新建的 `report_delivery.rs` 负责把这些输入组装成标准模板 `PersistedWorkbookDraft` 与图表元数据。导出仍复用现有 `export_excel_workbook`，从而保持上层稳定、底层低侵入。

**Tech Stack:** Rust, Polars, Serde JSON, rust_xlsxwriter, 现有 workbook/result/session runtime

---

### Task 1: 写失败测试，锁定 Tool 暴露与 workbook_ref 产出

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\integration_cli_json.rs`
- Test: `D:\Rust\Excel_Skill\tests\integration_cli_json.rs`

**Step 1: Write the failing test**

新增测试：
- `tool_catalog_includes_report_delivery`
- `report_delivery_returns_workbook_ref_for_standard_template`

第一条只断言目录暴露新 Tool。
第二条断言：
- 返回 `workbook_ref`
- `template == standard_report_v1`
- `sheet_names == ["摘要页", "分析结果页", "图表页"]`

**Step 2: Run test to verify it fails**

Run:
- `cargo test tool_catalog_includes_report_delivery --test integration_cli_json -v`
- `cargo test report_delivery_returns_workbook_ref_for_standard_template --test integration_cli_json -v`

Expected: FAIL，提示目录或分发不存在。

**Step 3: Write minimal implementation**

先不写完整实现，只保证红灯合理。

**Step 4: Run test to verify it fails correctly**

Expected: 失败原因与缺少 Tool / 缺少分发一致。

### Task 2: 写失败测试，锁定导出闭环

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\integration_cli_json.rs`
- Test: `D:\Rust\Excel_Skill\tests\integration_cli_json.rs`

**Step 1: Write the failing test**

新增：`report_delivery_workbook_can_be_exported_to_excel`

流程：
- 先调 `report_delivery`
- 再调 `export_excel_workbook`
- 最后用 calamine 验证三张 sheet 都存在

**Step 2: Run test to verify it fails**

Run:
- `cargo test report_delivery_workbook_can_be_exported_to_excel --test integration_cli_json -v`

Expected: FAIL。

### Task 3: 写 frame 层失败测试，锁定新模块职责

**Files:**
- Create: `D:\Rust\Excel_Skill\src\ops\report_delivery.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\mod.rs`
- Modify: `D:\Rust\Excel_Skill\tests\integration_frame.rs`

**Step 1: Write the failing test**

新增：`report_delivery_builds_standard_template_draft`

断言：
- 草稿里有 3 张 sheet
- `摘要页` 与 `分析结果页` 保留传入数据
- `图表页` 是结构化占位页

**Step 2: Run test to verify it fails**

Run:
- `cargo test report_delivery_builds_standard_template_draft --test integration_frame -v`

Expected: FAIL，提示模块或函数不存在。

### Task 4: 做最小实现并接入 dispatcher

**Files:**
- Create: `D:\Rust\Excel_Skill\src\ops\report_delivery.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\mod.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\dispatcher.rs`

**Step 1: Write minimal implementation**

新增：
- `ReportDeliveryError`
- `ReportDeliverySection`
- `ReportDeliveryRequest`
- `build_report_delivery_draft(...)`
- 图表占位页 DataFrame 构造 helper

dispatcher 新增：
- `dispatch_report_delivery`
- tool match 分发
- tool catalog 暴露
- `sync_output_handle_state(..., "workbook_ref", "report_delivery")`

**Step 2: Run targeted tests to verify they pass**

Run:
- `cargo test report_delivery --test integration_frame --test integration_cli_json -v`

Expected: PASS。

### Task 5: 图表第一版增强

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\report_delivery.rs`
- Modify: `D:\Rust\Excel_Skill\src\frame\workbook_ref_store.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\export.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\dispatcher.rs`
- Modify: `D:\Rust\Excel_Skill\tests\integration_cli_json.rs`
- Modify: `D:\Rust\Excel_Skill\tests\integration_frame.rs`

**Step 1: Write the failing tests**

新增图表增强测试：
- `report_delivery_with_column_chart_can_be_exported`
- `report_delivery_with_line_chart_can_be_exported`
- `report_delivery_with_multi_series_column_chart_can_be_exported`
- `report_delivery_exports_multiple_charts_when_requested`
- `report_delivery_builds_chart_specs_for_analysis_sheet`
- `report_delivery_builds_multi_series_chart_specs`
- `report_delivery_auto_layouts_multiple_charts_into_grid`

先红后绿，锁定：
- 单图单系列
- 单图多系列
- 同页多图自动布局

**Step 2: Implement**

实现要点：
- `report_delivery` 支持 `charts[]`
- chart spec 支持 `series[]`
- 老 `value_column` 单系列写法继续兼容
- 未显式传锚点时走两列网格自动布局
- `export_excel_workbook` 将图表元数据写成真实 Excel 图表

**Step 3: Run targeted verification**

Run:
- `cargo test report_delivery --test integration_cli_json -q`
- `cargo test report_delivery --test integration_frame -q`

Expected: PASS。

### Task 6: 全量验证与日志

**Files:**
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Run verification**

Run:
- `cargo test -v -- --test-threads=1`
- `cargo build --release -v`

Expected: PASS。

**Step 2: Update task journal**

按既有模板追加本轮记录。

**Step 3: Record deferred scope**

在日志或最终说明中明确后置项：
- 图表导出图片
- 更复杂模板体系
- 组合图 / 双轴图
- 图表样式深度定制

### Task 7: 图表类型第二批扩展

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\report_delivery.rs`
- Modify: `D:\Rust\Excel_Skill\src\frame\workbook_ref_store.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\export.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\dispatcher.rs`
- Modify: `D:\Rust\Excel_Skill\tests\integration_cli_json.rs`
- Modify: `D:\Rust\Excel_Skill\tests\integration_frame.rs`

**Step 1: Write the failing tests**

新增：
- `report_delivery_with_pie_chart_can_be_exported`
- `report_delivery_with_scatter_chart_can_be_exported`

锁定：
- `pie` 可导出为 `<c:pieChart>`
- `scatter` 可导出为 `<c:scatterChart>`

**Step 2: Run test to verify it fails**

Run:
- `cargo test report_delivery_with_pie_chart_can_be_exported --test integration_cli_json -q`
- `cargo test report_delivery_with_scatter_chart_can_be_exported --test integration_cli_json -q`

Expected: FAIL。

**Step 3: Write minimal implementation**

实现：
- 扩 `ReportDeliveryChartType`
- 扩 workbook chart spec 类型
- 导出层映射到 `rust_xlsxwriter::ChartType`
- `pie` 先限制为单系列

**Step 4: Run test to verify it passes**

Run:
- 同上两条命令

Expected: PASS。

### Task 8: 图表样式开关第一版

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\report_delivery.rs`
- Modify: `D:\Rust\Excel_Skill\src\frame\workbook_ref_store.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\export.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\dispatcher.rs`
- Modify: `D:\Rust\Excel_Skill\tests\integration_cli_json.rs`
- Modify: `D:\Rust\Excel_Skill\tests\integration_frame.rs`

**Step 1: Write the failing tests**

新增：
- `report_delivery_chart_style_controls_are_persisted_in_draft`
- `report_delivery_with_legend_position_and_style_can_be_exported`

锁定：
- `show_legend`
- `legend_position`
- `chart_style`
- `x_axis_name`
- `y_axis_name`

**Step 2: Run test to verify it fails**

Run:
- `cargo test report_delivery_chart_style_controls_are_persisted_in_draft --test integration_frame -q`
- `cargo test report_delivery_with_legend_position_and_style_can_be_exported --test integration_cli_json -q`

Expected: FAIL。

**Step 3: Write minimal implementation**

实现：
- 图表样式字段进入 chart spec
- 导出时设置 legend、style、轴名称
- 默认行为继续兼容旧请求

**Step 4: Run test to verify it passes**

Run:
- 同上两条命令

Expected: PASS。

### Task 9: 交付模板排版增强

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\frame\workbook_ref_store.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\report_delivery.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\export.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\dispatcher.rs`
- Modify: `D:\Rust\Excel_Skill\tests\integration_cli_json.rs`
- Modify: `D:\Rust\Excel_Skill\tests\integration_frame.rs`

**Step 1: Write the failing tests**

新增：
- `report_delivery_builds_standard_report_v2_layout_metadata`
- `report_delivery_export_writes_sheet_titles_before_data`

锁定：
- `summary` / `analysis` / `chart` 页都有顶部标题区
- 数据表起始行偏移后仍可被图表正确引用
- 模板版本升级到 `standard_report_v2`

**Step 2: Run test to verify it fails**

Run:
- `cargo test report_delivery_builds_standard_report_v2_layout_metadata --test integration_frame -q`
- `cargo test report_delivery_export_writes_sheet_titles_before_data --test integration_cli_json -q`

Expected: FAIL。

**Step 3: Write minimal implementation**

实现：
- workbook 草稿增加 sheet-level presentation metadata
- export 时先写标题区，再写表头和数据
- 图表锚点与数据源行号自动按偏移校正

**Step 4: Run test to verify it passes**

Run:
- 同上两条命令

Expected: PASS。
