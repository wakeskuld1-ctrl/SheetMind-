# V2-P2 Chart Tools Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 新增独立 `chart_ref` / `build_chart` / `export_chart_image` 能力，让图表构建从 `report_delivery` 中拆出来，并先以纯 Rust 的 SVG 导出建立最小独立闭环。

**Architecture:** 在 `frame` 层新增 `chart_ref_store` 持久化图表草稿，把图表所需的数据快照与图表规格一起落盘；在 `ops` 层新增轻量图表渲染模块，先支持 `column / line / pie / scatter` 四类 SVG 输出。`dispatcher` 负责解析单表来源、构建 `chart_ref`、导出 SVG，并保持现有 CLI 路由稳定。

**Tech Stack:** Rust, Polars, Serde JSON, 本地文件持久化, SVG 文本导出

---

### Task 1: 写失败测试，锁定 Tool 暴露与 `chart_ref` 产出

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\integration_cli_json.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\contracts.rs`

**Step 1: Write the failing test**

新增：
- `tool_catalog_includes_build_chart_and_export_chart_image`
- `build_chart_returns_chart_ref_for_result_ref`

锁定：
- tool catalog 暴露 `build_chart` 与 `export_chart_image`
- `build_chart` 支持 `result_ref`
- 成功返回 `chart_ref`、`chart_type`、`row_count`、`series_count`

**Step 2: Run test to verify it fails**

Run:
- `cargo test --test integration_cli_json tool_catalog_includes_build_chart_and_export_chart_image -q`
- `cargo test --test integration_cli_json build_chart_returns_chart_ref_for_result_ref -q`

Expected: FAIL。

### Task 2: 新增 `chart_ref` 持久化层

**Files:**
- Create: `D:\Rust\Excel_Skill\src\frame\chart_ref_store.rs`
- Modify: `D:\Rust\Excel_Skill\src\frame\mod.rs`
- Test: `D:\Rust\Excel_Skill\tests\integration_registry.rs`

**Step 1: Write the failing test**

新增：
- `chart_draft_roundtrips_through_disk`

锁定：
- `chart_ref` 可保存/加载
- 数据快照和图表规格可回放
- 多系列配置可保留

**Step 2: Run test to verify it fails**

Run:
- `cargo test --test integration_registry chart_draft_roundtrips_through_disk -q`

Expected: FAIL。

### Task 3: 新增最小实现并接入 `build_chart`

**Files:**
- Create: `D:\Rust\Excel_Skill\src\ops\chart_svg.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\mod.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\dispatcher.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\contracts.rs`
- Test: `D:\Rust\Excel_Skill\tests\integration_cli_json.rs`

**Step 1: Write minimal implementation**

新增：
- `PersistedChartDraft`
- `ChartDraftStore`
- `dispatch_build_chart`
- `build_chart` 参数解析与校验
- `chart_ref` 会话句柄同步

**Step 2: Run targeted tests to verify they pass**

Run:
- `cargo test --test integration_cli_json tool_catalog_includes_build_chart_and_export_chart_image -q`
- `cargo test --test integration_cli_json build_chart_returns_chart_ref_for_result_ref -q`
- `cargo test --test integration_registry chart_draft_roundtrips_through_disk -q`

Expected: PASS。

### Task 4: 写失败测试，锁定 SVG 导出闭环

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\integration_cli_json.rs`
- Modify: `D:\Rust\Excel_Skill\tests\integration_frame.rs`

**Step 1: Write the failing tests**

新增：
- `export_chart_image_writes_svg_for_column_chart`
- `export_chart_image_writes_svg_for_pie_chart`
- `render_line_chart_svg_contains_polyline`
- `render_scatter_chart_svg_contains_points`

锁定：
- `export_chart_image` 支持 `.svg`
- `column / line / pie / scatter` 都能落成基础可视 SVG
- 标题、系列、分类/坐标轴至少保留最小可读结构

**Step 2: Run test to verify it fails**

Run:
- `cargo test --test integration_cli_json export_chart_image_writes_svg_for_column_chart -q`
- `cargo test --test integration_cli_json export_chart_image_writes_svg_for_pie_chart -q`
- `cargo test --test integration_frame render_line_chart_svg_contains_polyline -q`
- `cargo test --test integration_frame render_scatter_chart_svg_contains_points -q`

Expected: FAIL。

### Task 5: 实现 SVG 渲染与导出

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\chart_svg.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\dispatcher.rs`
- Test: `D:\Rust\Excel_Skill\tests\integration_cli_json.rs`
- Test: `D:\Rust\Excel_Skill\tests\integration_frame.rs`

**Step 1: Write minimal implementation**

实现：
- `render_chart_svg(...)`
- `export_chart_image` 先只支持 `svg`
- `column / line / pie / scatter` 四类基础 SVG 输出
- 明确对非 svg 导出返回 UTF-8 中文错误

**Step 2: Run targeted tests to verify they pass**

Run:
- `cargo test --test integration_cli_json export_chart_image_writes_svg_for_column_chart -q`
- `cargo test --test integration_cli_json export_chart_image_writes_svg_for_pie_chart -q`
- `cargo test --test integration_frame render_line_chart_svg_contains_polyline -q`
- `cargo test --test integration_frame render_scatter_chart_svg_contains_points -q`

Expected: PASS。

### Task 6: 补定向边界测试

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\integration_cli_json.rs`

**Step 1: Write the failing test**

新增：
- `export_chart_image_rejects_non_svg_output_path`
- `build_chart_rejects_missing_series`

**Step 2: Run tests to verify they fail**

Run:
- `cargo test --test integration_cli_json export_chart_image_rejects_non_svg_output_path -q`
- `cargo test --test integration_cli_json build_chart_rejects_missing_series -q`

Expected: FAIL。

**Step 3: Write minimal implementation**

实现：
- 非 `.svg` 输出时报明确中文错误
- 缺少数值系列时报明确中文错误

**Step 4: Run tests to verify they pass**

Run:
- 同上两条命令

Expected: PASS。

### Task 7: 全量验证与日志

**Files:**
- Modify: `D:\Rust\Excel_Skill\progress.md`
- Modify: `D:\Rust\Excel_Skill\findings.md`
- Modify: `D:\Rust\Excel_Skill\task_plan.md`
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Run verification**

Run:
- `cargo test --test integration_cli_json build_chart -q`
- `cargo test --test integration_cli_json export_chart_image -q`
- `cargo test --test integration_frame render_ -q`
- `cargo test --test integration_registry chart_draft_roundtrips_through_disk -q`
- `cargo build --release -v`

Expected: PASS。

**Step 2: Update task journal**

按模板追加本轮记录，明确：
- V1 独立图表导出先支持 SVG
- 不引入 Python / 浏览器依赖
- `report_delivery` 复用 `chart_ref` 留到下一轮
