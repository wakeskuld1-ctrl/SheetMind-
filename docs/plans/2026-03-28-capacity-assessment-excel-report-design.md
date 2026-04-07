# 容量评估 Excel 报表设计

## 背景

用户当前要的不是单独的 JSON 分析结果，而是“给一些业务和资源数据后，直接交付 Excel 容量评估结论”的能力。
现有工程已经具备三段底座：

- `capacity_assessment`：负责弹性、非线性、证据降级的容量结论计算
- `capacity_assessment_from_inventory`：负责把受限 SSH 盘点映射成容量证据
- `report_delivery` / `export_excel_workbook`：负责 workbook 草稿与最终 `.xlsx` 导出

因此本轮不再扩一层新的 JSON-only 分析链，而是把已有能力收口成一个面向交付的 Excel 报表 Tool。

## 目标

新增一个正式 Tool：`capacity_assessment_excel_report`

它需要支持：

- 有 Excel 业务/指标表时，直接基于表数据分析并导出容量评估 Excel 报表
- 用户提供不了完整数据时，仍然可以结合 `scenario_profile`、`deployment_profile`、`inventory_result` / `inventory_request` 输出 guidance-only 或 partial 报表
- 可选直接写出 `.xlsx`，同时保留 `workbook_ref` 供链式编排复用

## 方案对比

### 方案A：新增专用容量评估 Excel 报表 Tool

做法：

- 新增 `capacity_assessment_excel_report`
- 内部复用 `capacity_assessment` 或 `capacity_assessment_from_inventory`
- 把结果组装成固定结构的 workbook draft
- 若传入 `output_path`，则直接导出 `.xlsx`

优点：

- 最贴近用户心智，别人“给数据 -> 直接拿 Excel”
- 高层入口稳定，不需要调用方手工串 `capacity_assessment -> report_delivery -> export_excel_workbook`
- 能把弱证据、补数优先级、风险说明一并固化进交付模板

缺点：

- 需要新增一层结果装配逻辑
- 需要维护报表页结构和字段落位

### 方案B：继续让调用方手工串多个已有 Tool

做法：

- 先调用 `capacity_assessment` / `capacity_assessment_from_inventory`
- 再调用 `compose_workbook` / `report_delivery`
- 最后调用 `export_excel_workbook`

优点：

- 代码新增最少
- 保持底层 Tool 纯粹

缺点：

- 违背用户“直接交付 Excel”的目标
- 上层编排成本高，容易接错
- 弱证据场景下，调用方还得自己设计表格结构

本轮按用户已批准的 `方案A` 落地。

## 交付结构

报表默认输出四个工作表：

### 1. 结论页

用于给业务方或运维负责人直接看结论，字段包含：

- 服务名
- 证据等级
- 当前容量状态
- 当前实例数
- 建议实例数
- 瓶颈资源
- 推荐下一步

### 2. 资源测算页

用于展示具体资源维度推导，字段包含：

- 资源类型
- 当前利用率
- 目标利用率
- 预测需求放大系数
- 饱和惩罚系数
- 建议实例数
- 状态

### 3. 证据与风险页

用于让交付结果可解释、可复核，字段包含：

- 模型族
- 证据等级
- 服务等级
- SLA
- 峰值模式
- 故障影响
- 冗余模式
- 扩容步长
- 证据来源
- 盘点来源
- 主机 CPU 核数
- 主机内存 MB
- 历史趋势指标

### 4. 补数与行动页

用于在数据不全时仍然给决策思路，字段包含：

- 缺失输入项
- 补数优先级
- 手工决策路径
- 置信度说明
- 推荐下一步

## 输入设计

Tool 输入沿用现有容量分析输入，额外补充交付字段：

- `report_name`
- `report_subtitle`
- `output_path`：可选，传入则直接导出 `.xlsx`
- `summary_sheet_name` / `analysis_sheet_name` 等页签名：先保持可选，默认用固定中文名

数据来源支持三类：

- Excel 表：`path + sheet`、`table_ref`、`result_ref`
- 部署画像：`scenario_profile`、`deployment_profile`
- SSH 盘点：`inventory_request` 或 `inventory_result`

## 关键设计决策

### 决策1：允许“无 Excel 源”也能出报表

如果用户给不了工作簿，但给了 SSH 盘点或场景画像，Tool 仍然生成报表。
此时报表中的“资源测算页”可能为空或很弱，但“证据与风险页”和“补数与行动页”必须仍然可用。

### 决策2：高层 Tool 默认直接面向交付

`output_path` 存在时，Tool 直接导出 `.xlsx`。
同时仍返回 `workbook_ref`，这样兼容现有链路与调试场景。

### 决策3：不重复实现 Excel 模板引擎

报表装配优先复用：

- `WorkbookDraftStore`
- `PersistedWorkbookDraft`
- `export_excel_workbook`

必要时直接构造 workbook draft，而不是为了复用而硬套 `report_delivery` 的双段模板。
原因是容量评估报表天然是多页异构结构，不必强行压成“summary + analysis + chart”。

## 错误处理

- 既无表数据、又无盘点输入、也无画像输入时：返回稳定中文错误
- SSH 失败时：保持桥接层已有的稳定 `ssh` 错误传递
- `output_path` 不可写时：返回导出失败错误
- 缺失关键信息但仍可退化时：不报错，生成 guidance-only 报表

## 测试策略

先写失败测试，再实现：

- Tool 目录暴露测试
- 直接基于 Excel 输入导出 `.xlsx`
- 基于 `inventory_result` 的 partial 报表导出
- 无 Excel、只有弱证据时仍能输出 guidance-only 报表
- 导出文件包含目标 sheet 名称与关键单元格文本

