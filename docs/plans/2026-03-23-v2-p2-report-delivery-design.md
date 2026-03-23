# V2-P2 Report Delivery Design

> Date: 2026-03-23
> Scope: 第一轮结果交付层，独立 `report_delivery` 模块

## Goal

在不介入底层原子化拆分的前提下，先独立拉出一层上层结果交付模块，为 V2-P2 建立稳定壳层。第一轮目标不是一次做满图表能力，而是先把“汇报模板 workbook 草稿 -> workbook_ref -> Excel 导出”闭环正式建立起来。

## Constraints

- 上层 CLI / dispatcher / Skill 路由语义尽量保持稳定。
- 第一轮不扩成底层导出重构。
- 继续复用现有 `format_table_for_export`、`compose_workbook`、`export_excel_workbook` 基础能力。
- 新模块单独建文件，降低后续底层原子化对交付层的冲击。

## Why a Separate File

当前仓库已经有：
- `format_table_for_export.rs`：做导出前列布局整理
- `export.rs`：做 CSV / Excel 写出
- `workbook_ref_store.rs`：做 workbook 草稿落盘
- `dispatcher.rs`：接 `compose_workbook` / `export_excel_workbook`

但还缺少一个“结果交付层语义入口”。

也就是说，现在能导出，但还没有一个稳定的上层模块把这些能力组织成“面向汇报的交付模板”。这正是 `report_delivery.rs` 的职责。

## First-Round Scope

第一轮只做三件事：

1. 新增 `report_delivery` Tool
2. 让它产出固定模板的 `workbook_ref`
3. 让这个 `workbook_ref` 可以继续被 `export_excel_workbook` 导出

在第一轮主链路闭环以后，本日继续补一层“图表第一版增强”，但仍然保持在同一个壳层内，不扩成独立 `chart_ref` 体系。

## Template in Round 1

第一轮模板固定为三页：

1. `摘要页`
   - 来自用户提供的 `summary_source`
2. `分析结果页`
   - 来自用户提供的 `analysis_source`
3. `图表页`
   - 第一轮先允许带图表元数据的占位页
   - 第二轮增强为真实 Excel 图表写入页

这样做的原因：
- 先把“汇报模板”这条主链路建立起来
- 第一轮先不扩到复杂图表系统
- 给后续图表写入 workbook 留稳定插槽

## Input Contract

第一轮 `report_delivery` 输入建议：

```json
{
  "tool": "report_delivery",
  "args": {
    "report_name": "经营分析汇报",
    "summary": {
      "source": { "result_ref": "..." },
      "sheet_name": "摘要页"
    },
    "analysis": {
      "source": { "path": "...", "sheet": "Sales" },
      "sheet_name": "分析结果页"
    },
    "include_chart_sheet": true,
    "chart_sheet_name": "图表页"
  }
}
```

### Notes
- `summary.source` / `analysis.source` 复用已有 `NestedTableSource`
- 第一轮不在 `report_delivery` 内部再做复杂格式编排
- 如果用户需要列顺序与别名，先调用 `format_table_for_export` 再把结果 `result_ref` 传进来

图表增强后，`report_delivery` 额外支持：

```json
{
  "charts": [
    {
      "chart_type": "column",
      "title": "月度收入对比",
      "category_column": "月份",
      "series": [
        { "value_column": "收入", "name": "收入" },
        { "value_column": "成本", "name": "成本" }
      ],
      "anchor_row": 1,
      "anchor_col": 0
    }
  ]
}
```

兼容策略：
- 新接口优先使用 `series`
- 老接口仍兼容单个 `value_column`
- 未显式传锚点时，默认走两列网格自动布局

## Output Contract

```json
{
  "status": "ok",
  "data": {
    "workbook_ref": "workbook_...",
    "report_name": "经营分析汇报",
    "template": "standard_report_v1",
    "sheet_count": 3,
    "sheet_names": ["摘要页", "分析结果页", "图表页"]
  }
}
```

并且：
- 成功后同步最新激活句柄为 `workbook_ref`

## Internal Design

### New file
- `D:\Rust\Excel_Skill\src\ops\report_delivery.rs`

### Responsibilities
- 定义 `ReportDeliveryRequest` / `ReportDeliverySection`
- 定义 `ReportDeliveryChart` / `ReportDeliveryChartSeries`
- 接收 dispatcher 准备好的 `LoadedTable`
- 组装 `WorkbookSheetInput`
- 自动插入图表页 DataFrame
- 下沉图表元数据为 workbook chart spec
- 产出 `PersistedWorkbookDraft`

### Dispatcher responsibilities
- 解析 `summary` / `analysis`
- 解析 `charts`
- 使用现有 nested source 装载输入表
- 调用 `build_report_delivery_draft(...)`
- 保存 workbook 草稿
- 调用 `sync_output_handle_state`

### Workbook / Export responsibilities
- `workbook_ref_store` 持久化图表元数据，兼容单系列与多系列
- `export_excel_workbook` 负责把图表元数据写成真实 Excel 图表对象
- 多图情况下按两列网格自动布局，减少首版导出后的人工排版成本

## Current Non-Goals

当前仍明确不做：
- 独立 `chart_ref`
- 组合图 / 双轴图
- 图表图片导出
- 复杂模板系统
- 自动格式规则 DSL
- 深度样式模板与品牌化图表主题

这些仍属于 V2-P2 后续轮次。

## Testing Strategy

### CLI red-green tests
1. `tool_catalog_includes_report_delivery`
2. `report_delivery_returns_workbook_ref_for_standard_template`
3. `report_delivery_workbook_can_be_exported_to_excel`
4. `report_delivery_with_column_chart_can_be_exported`
5. `report_delivery_with_line_chart_can_be_exported`
6. `report_delivery_with_multi_series_column_chart_can_be_exported`
7. `report_delivery_exports_multiple_charts_when_requested`

### Frame-level red-green test
8. `report_delivery_builds_standard_template_draft`
9. `report_delivery_builds_chart_specs_for_analysis_sheet`
10. `report_delivery_builds_multi_series_chart_specs`
11. `report_delivery_auto_layouts_multiple_charts_into_grid`

## Risks

1. 如果第一轮把格式整理也塞进 `report_delivery`，职责会过重。
2. 如果图表页直接承诺真实图表，会让当前实现范围失控。
3. 如果不把 `workbook_ref` 句柄同步接进去，交付链就不算真正闭环。

## Recommended Next Step After Round 1

下一轮再补：
- `build_chart`
- `export_chart_image`
- 图表写入 workbook
- `report_delivery` 内部接图表对象列表

在本轮图表第一版增强完成后，下一步建议聚焦：
- 更多图表类型：饼图、散点图、柱线之外的常见汇报图
- 图表样式：标题、副标题、图例显隐、颜色主题
- 交付模板：摘要页 / 图表页 / 分析页的更稳定排版
