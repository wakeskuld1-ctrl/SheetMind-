# Diagnostics Report Excel Report Enhancement Design

## 背景

当前 `diagnostics_report_excel_report` 已经具备第一版 workbook-first 交付能力：

- 复用 `diagnostics_report` 输出统一诊断合同
- 固定生成 4 页表格型工作簿
- 支持 `workbook_ref` 与真实 `.xlsx` 双落点
- 保留 section 级降级，不因单 section 失败而整包失败

这条主线已经满足“Rust / exe 原生交付”的基本要求，但还缺两层关键能力：

1. `执行摘要` 还不够 manager-facing，更多是键值表而不是决策页。
2. workbook 还没有图表页，视觉交付力偏弱。

本轮继续沿已批准的方案 3 前进，但保持增量开发，不重开架构。

## 目标

在不新增新 Tool 的前提下，增强现有 `diagnostics_report_excel_report`：

1. 先把 `执行摘要` 升级成管理摘要页。
2. 再补一个稳定的图表页。
3. 继续复用 `diagnostics_report` 作为唯一上游业务合同。
4. 继续支持 `workbook_ref` 与 `.xlsx` 双交付。

## 非目标

- 本轮不新增新的底层统计诊断算法。
- 本轮不重构 dispatcher / catalog / workbook draft 主链。
- 本轮不把图表逻辑拆成新 Tool。
- 本轮不引入 Python 运行时或外部服务。

## 方案对比

### 方案 A：只补图表页

优点：

- 改动集中，风险最低
- 最快提升 Excel 可视化效果

缺点：

- 摘要页仍然不够像管理交付
- AI 或上层系统读取时，决策合同增强有限

### 方案 B：只补管理摘要

优点：

- 直接增强业务可读性和决策性
- 对 AI 调用更友好

缺点：

- 视觉交付提升有限
- Excel 成品感仍偏弱

### 方案 C：双阶段增强

做法：

1. 先增强管理摘要页
2. 再补图表页
3. 都继续落在 `diagnostics_report_excel_report`

优点：

- 形成“能读、能看、能决策”的完整交付包
- 继续复用同一份诊断合同，不会分叉主链

缺点：

- 实现面比单一增强更宽

本轮采用方案 C。

## 设计原则

### 1. 不重开架构

数据流继续保持：

`dispatcher -> diagnostics_report_excel_report -> diagnostics_report -> workbook draft -> optional xlsx export`

### 2. 先增强摘要，再增强视觉

摘要页是上层 AI 与业务方的共用核心合同，必须先稳住，再叠图表页。

### 3. 图表页只消费现有结果

图表页不重新跑算法，不直接读原始工作簿字段，只消费 `diagnostics_result` 中已存在的结构化结果，避免业务口径漂移。

### 4. 降级规则不变

只要 `diagnostics_report` 还能返回结果，workbook 就继续交付。图表页如某个 section 不可用，可缺图但不能整包失败。

## 管理摘要增强设计

当前 `执行摘要` 页主要是通用键值对。本轮改成“管理摘要表”风格，新增更强的管理字段：

- `决策结论`
- `总体风险等级`
- `可直接决策`
- `优先处理方向`
- `关键发现`
- `建议动作`
- `降级提醒`

建议映射规则如下：

- `决策结论`：复用 `overall_summary`
- `总体风险等级`：
  - `degraded` 时优先为 `高`
  - `ok` 且关键发现较少时为 `中` 或 `低`
- `可直接决策`：
  - `ok` 且无 warnings 为 `是`
  - 有 warnings 或 `degraded` 为 `否（建议复核）`
- `优先处理方向`：优先取 `recommended_actions` 前 1 到 3 条
- `关键发现`：继续复用 `key_findings`
- `降级提醒`：复用 `warnings`

这里保持轻规则、可解释，不引入新的复杂评分引擎。

## 图表页设计

新增固定图表页，默认名称建议为 `图表摘要`。

第一版建议只放 3 类图表，全部由现有诊断结果直接生成：

1. `相关性 Top 对比图`
   - 数据来源：`correlation_section.top_positive / top_negative`
   - 图表类型：柱状图

2. `异常占比图`
   - 数据来源：`outlier_section.outlier_summaries`
   - 图表类型：柱状图

3. `趋势变化图`
   - 数据来源：`trend_section.points`
   - 图表类型：折线图

第一版不强求分布直方图真实落图，因为现有 histogram 数据结构更适合后续单独打磨，先保证 3 张最有价值的图稳定导出。

## Workbook 结构调整

当前 4 页调整为 5 页：

1. `执行摘要`
2. `诊断概览`
3. `相关性与异常`
4. `分布与趋势`
5. `图表摘要`

如个别图表缺数据：

- 仍创建图表页
- 用占位表说明“该图未生成”
- 其余图表继续正常导出

## 输入合同调整

保留现有字段，并新增可选字段：

- `chart_sheet_name`
- `include_chart_sheet`，默认 `true`

这样可保持向后兼容：

- 不传时默认生成图表页
- 显式关闭时仍可只交付表格版 workbook

## 输出合同调整

现有字段继续保留：

- `diagnostics_result`
- `workbook_ref`
- `sheet_names`
- `format`
- `output_path`

本轮不新增复杂 chart metadata 输出，只通过 `sheet_names` 反映图表页存在。

## 错误与降级处理

### 合法降级

- 某个 section 不可用：仍生成 workbook
- 某张图因无数据无法生成：图表页仍存在，写入占位说明
- 有 warnings：摘要页显式展示“需复核”

### 整体失败

只有以下情况才整体报错：

- `report_name` 为空
- `output_path` 为空字符串
- workbook 草稿构建失败
- chart sheet 配置非法且无法恢复

## 测试策略

本轮至少补 4 类测试：

1. 默认返回 5 个 sheet 名，并包含 `图表摘要`
2. `.xlsx` 导出后，workbook 内真实存在图表页
3. 管理摘要页包含新的管理字段，如 `总体风险等级`、`可直接决策`
4. 单个 section 降级时，仍保留图表页与管理摘要降级提醒

必要时再补：

5. `include_chart_sheet = false` 时仍回退为无图表页版本

## 下一步

实施阶段按以下顺序推进：

1. 先补设计锁定文档
2. 再补实施计划
3. 先写红测
4. 再做最小实现
5. 跑针对性测试和全量 `cargo test`
6. 最后补 task journal 与交接记录
