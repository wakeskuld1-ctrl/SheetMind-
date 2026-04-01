# Diagnostics Report Excel Report Design

## 背景

当前 `.worktrees/SheetMind-` 已经有 4 个可独立调用的统计诊断 Tool：

- `correlation_analysis`
- `outlier_detection`
- `distribution_analysis`
- `trend_analysis`

并且已经新增了高层组合 Tool `diagnostics_report`，能够把四类诊断收口成统一 JSON 结果，同时支持 section 级降级。下一步最自然的延伸，不是再改架构，而是把这个统一结果直接交付成 Excel 工作簿，继续沿 Rust / exe 主线往上长。

## 目标

新增一个高层 Rust Tool：`diagnostics_report_excel_report`。

这个 Tool 的职责是：

1. 复用 `diagnostics_report` 生成统一诊断结果。
2. 把统一结果翻译成稳定的 workbook draft。
3. 在有 `output_path` 时导出 `.xlsx`，没有 `output_path` 时返回 `workbook_ref` 供后续链路继续消费。

## 非目标

- 第一版不直接生成图表页。
- 第一版不重新设计统计诊断底层算法。
- 第一版不绕开现有 workbook draft / export 链。
- 第一版不引入 Python 运行时或外部服务。

## 推荐方案

采用“组合诊断 JSON 先行，Excel 交付后置”的两段式设计：

1. `diagnostics_report_excel_report` 先在内存中调用 `diagnostics_report`。
2. 再把结果映射为固定 4 页工作簿。
3. 最后复用现有 `WorkbookDraftStore + export_excel_workbook`。

这样做的优点是：

- 继续复用已经验证过的 `diagnostics_report` 合同，避免重复拼摘要。
- workbook 只是新的交付层，不会把执行链再拆成新的主干。
- 后续如果要做图表页，也可以直接建立在这个统一合同之上。

## 输入合同

`diagnostics_report_excel_report` 建议支持以下输入：

- `report_name`: 报告名称，必填。
- `report_subtitle`: 报告副标题，可选。
- `output_path`: 输出路径，可选。
- `summary_sheet_name`: 摘要页名称，可选，默认固定中文名。
- `section_sheet_name`: 诊断概览页名称，可选。
- `detail_sheet_name`: 相关性与异常页名称，可选。
- `trend_sheet_name`: 分布与趋势页名称，可选。
- `result_ref`: 继续复用当前统计诊断主线的数据来源。
- `correlation / outlier / distribution / trend`: 直接沿用 `diagnostics_report` 的 section 配置。

第一版不额外引入 chart config，也不支持自定义 sheet 数量。

## 输出合同

Tool 返回统一 JSON，核心字段建议包括：

- `diagnostics_result`: 内嵌 `diagnostics_report` 返回结果，方便上层继续读统一合同。
- `workbook_ref`
- `sheet_names`
- `format`: `workbook_ref` 或 `xlsx`
- `output_path`

这样做的原因是：上层既可能要最终 Excel，也可能还要继续读取诊断摘要；把二者放在一次响应里最稳。

## 工作簿结构

第一版固定为 4 页：

### 1. `执行摘要`

放总览性内容：

- 报告名称
- 总体状态：`ok / degraded / unavailable`
- `section_count / available_section_count`
- `overall_summary`
- `key_findings`
- `recommended_actions`
- `warnings`

### 2. `诊断概览`

放各 section 的状态表：

- `key`
- `title`
- `status`
- `summary`

目的是让业务方一眼知道哪些 section 可用，哪些 section 已降级。

### 3. `相关性与异常`

主要放：

- `correlation_section.top_positive`
- `correlation_section.top_negative`
- `outlier_section.outlier_summaries`

这页偏“问题结构”。

### 4. `分布与趋势`

主要放：

- `distribution_section.distribution_summary`
- `distribution_section.histogram`
- `trend_section.direction`
- `trend_section.absolute_change`
- `trend_section.change_rate`
- `trend_section.points`

这页偏“变化形态”。

## 降级与错误处理

### 合法降级

如果某个 section 失败：

- 工作簿仍然生成
- 摘要页写入 `warnings`
- 概览页把对应 section 标记为 `unavailable`
- 明细页只写成功 section 的内容

### 整体失败

只有以下情况才整体报错：

- `report_name` 为空
- `output_path` 是空字符串
- 一个 section 都没配置
- workbook draft 或 export 阶段发生真实构建错误

## 数据流

建议沿这条数据流实现：

`dispatcher -> diagnostics_report_excel_report request -> load_table_for_analysis -> diagnostics_report -> workbook draft builder -> WorkbookDraftStore -> optional export_excel_workbook`

这条链和 `capacity_assessment_excel_report` 保持同一风格，便于后续 AI 继续沿统一套路补交付 Tool。

## 测试策略

第一版至少补 4 类 CLI 回归：

1. `tool_catalog` 能发现 `diagnostics_report_excel_report`
2. 全 section 正常时，能返回 `workbook_ref` 与 4 页工作簿
3. 单个 section 失败时，仍能导出工作簿，且 `diagnostics_result.report_status == "degraded"`
4. 传 `output_path` 时，能实际写出 `.xlsx`

必要时再补：

5. 没有 `output_path` 时只返回 `workbook_ref`
6. 一个 section 都没配置时稳定报错

## 下一步

按这个设计落地时，应继续坚持：

- TDD：先写红测，再实现
- Rust / exe 主线
- 不重开架构
- 优先复用现有 workbook draft 与导出能力
