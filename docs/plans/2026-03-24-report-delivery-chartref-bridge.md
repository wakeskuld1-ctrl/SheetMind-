# Report Delivery ChartRef Bridge Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 让 `report_delivery` 可以直接复用已有 `chart_ref`，把独立图表导出和 workbook 交付统一到一套图表规格上。

**Architecture:** 在 `report_delivery` 的请求层新增 `chart_ref` 引用输入，让 dispatcher 能从 `ChartDraftStore` 读取已冻结的图表草稿，再把它桥接成 `PersistedWorkbookChartSpec` 写入 `workbook_ref`。保留现有 `charts[]` 直传能力不变，新增的是一条“引用已存在图表草稿”的稳定复用路径。优先只复用图表元数据与数据源快照，不在这轮引入新的图表计算。

**Tech Stack:** Rust, Serde JSON, Polars, rust_xlsxwriter, 本地 `chart_ref` / `workbook_ref` 持久化

---

### Task 1: 锁定 `report_delivery` 接受 `chart_ref` 的外部行为

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\integration_cli_json.rs`
- Modify: `D:\Rust\Excel_Skill\tests\integration_registry.rs`

**Step 1: Write the failing test**

新增 CLI 测试：
- `report_delivery_accepts_chart_ref_and_exports_workbook`
- `report_delivery_can_mix_chart_ref_with_inline_chart_specs`

新增 registry / store 测试：
- `chart_draft_can_be_mapped_to_workbook_chart_spec`

锁定行为：
- `report_delivery` 的 `charts[]` 可接受 `chart_ref`
- `chart_ref` 可被桥接成 workbook chart spec
- 混合输入下，导出的 workbook 仍能写出真实图表 XML

**Step 2: Run test to verify it fails**

Run:
- `cargo test --test integration_cli_json report_delivery_accepts_chart_ref_and_exports_workbook -q`
- `cargo test --test integration_cli_json report_delivery_can_mix_chart_ref_with_inline_chart_specs -q`
- `cargo test --test integration_registry chart_draft_can_be_mapped_to_workbook_chart_spec -q`

Expected: FAIL。

### Task 2: 为 `report_delivery` 增加 `chart_ref` 桥接结构

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\report_delivery.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\dispatcher.rs`
- Possibly Modify: `D:\Rust\Excel_Skill\src\frame\chart_ref_store.rs`

**Step 1: Write minimal implementation**

新增 / 调整：
- `report_delivery` 请求模型支持 `chart_ref`
- dispatcher 解析 `chart_ref` 时从 `ChartDraftStore` 读取草稿
- 将 `chart_ref` 映射为 `ReportDeliveryChart` 或直接映射为 workbook chart spec 所需字段
- 保留现有 inline `charts[]` 路径兼容

**Step 2: Run targeted tests to verify they pass**

Run:
- 同 Task 1 三条命令

Expected: PASS。

### Task 3: 锁定桥接后的元数据与来源血缘

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\integration_cli_json.rs`
- Modify: `D:\Rust\Excel_Skill\tests\integration_registry.rs`

**Step 1: Write the failing test**

新增：
- `report_delivery_chart_ref_persists_chart_source_refs_into_workbook_ref`
- `report_delivery_chart_ref_prefers_frozen_chart_layout_fields`

锁定行为：
- `chart_ref` 的 `source_refs` 会被带入 `workbook_ref`
- `title / x_axis_name / y_axis_name / show_legend / width / height` 等冻结字段按桥接策略保留

**Step 2: Run test to verify it fails**

Run:
- `cargo test --test integration_cli_json report_delivery_chart_ref_persists_chart_source_refs_into_workbook_ref -q`
- `cargo test --test integration_cli_json report_delivery_chart_ref_prefers_frozen_chart_layout_fields -q`

Expected: FAIL。

**Step 3: Write minimal implementation**

实现：
- `chart_ref` -> workbook chart spec 的字段映射
- source_refs 合并策略
- 对 inline 参数覆盖 / 不覆盖策略做最小明确收口

**Step 4: Run tests to verify they pass**

Run:
- 同上两条命令

Expected: PASS。

### Task 4: 补桥接边界测试

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\integration_cli_json.rs`

**Step 1: Write the failing test**

新增：
- `report_delivery_rejects_missing_chart_ref`
- `report_delivery_rejects_chart_ref_when_analysis_source_mismatches`

锁定行为：
- 不存在的 `chart_ref` 会给明确中文错误
- 如果桥接策略要求 `chart_ref` 的冻结数据源与 `analysis` 页不匹配，则报明确错误（或若设计决定允许，则改成锁定允许复用）

**Step 2: Run test to verify it fails**

Run:
- `cargo test --test integration_cli_json report_delivery_rejects_missing_chart_ref -q`
- `cargo test --test integration_cli_json report_delivery_rejects_chart_ref_when_analysis_source_mismatches -q`

Expected: FAIL。

**Step 3: Write minimal implementation**

实现：
- 缺失 `chart_ref` 错误透传
- 明确桥接匹配策略并收口错误文案

**Step 4: Run tests to verify they pass**

Run:
- 同上两条命令

Expected: PASS。

### Task 5: 全量验证与文档收口

**Files:**
- Modify: `D:\Rust\Excel_Skill\task_plan.md`
- Modify: `D:\Rust\Excel_Skill\findings.md`
- Modify: `D:\Rust\Excel_Skill\progress.md`
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Run verification**

Run:
- `cargo test --test integration_cli_json report_delivery -q`
- `cargo test --test integration_cli_json export_chart_image -q`
- `cargo test --test integration_registry chart_draft -q`
- `cargo build --release -v`

Expected: PASS。

**Step 2: Update task journal**

追加记录：
- `report_delivery` 已能复用 `chart_ref`
- 独立图表导出与 workbook 交付开始共用同一套图表规格
- 本轮仍未做 PNG/JPEG，继续保持 SVG + Excel 图表双通路
