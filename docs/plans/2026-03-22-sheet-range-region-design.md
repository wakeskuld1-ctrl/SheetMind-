# Sheet Range / Region Tool 设计

> 时间：2026-03-22
> 范围：`inspect_sheet_range`、`load_table_region`

## 目标

为 Excel 表处理层补齐两块更保守的显式区域能力：
- `inspect_sheet_range`：先探查一个 Sheet 里真正有内容的区域，并返回样本，帮助上层确认表大概在哪里。
- `load_table_region`：在用户明确给出区域后，把该区域显式装载成可继续分析的表结果，而不是自动猜测整张表。

## 背景

当前系统已经能：
- 打开工作簿并列出 Sheet
- 对整张 Sheet 做表头推断
- 把已确认结构加载成 `DataFrame`

但还缺两块关键基础能力：
- 当表不从 `A1` 开始时，缺少“先探查再选择”的稳定入口
- 当用户只想加载某个明确区域时，缺少“按区域读取”的显式 Tool

## 方案对比

### 方案 A：先探查已用区域，再显式按区域加载（推荐）
- 做法：
  - `inspect_sheet_range` 只返回已用边界、行列估计与样本行
  - `load_table_region` 接受显式 Excel 区域，如 `B3:D10`
  - `load_table_region` 支持显式 `header_row_count`，默认 `1`
- 优点：
  - 保守、可解释，符合当前 V1 “宁可多约束，不要误判”的产品取向
  - 能覆盖“表不在左上角”和“多层表头需显式指定 header 行数”两类真实问题
  - 不需要把“自动猜表头/数据起点”塞进 Tool 层
- 缺点：
  - 需要用户或 Skill 先 inspect，再决定 range / header 行数
  - 第一版交互步骤比全自动方案多一步

### 方案 B：直接自动猜区域并自动装载
- 做法：`load_table_region` 只给 `path + sheet`，底层自动猜 used range、header 与数据区。
- 优点：
  - 对最终用户最省心
- 缺点：
  - 容易在说明行、标题行、多层表头、空列混杂时误判
  - 一旦猜错，后续 DataFrame / 建模 / 决策链路都会被污染

### 方案 C：把区域逻辑直接揉进 `normalize_table`
- 做法：继续只保留 `normalize_table`，但让它同时承担区域识别与局部加载逻辑。
- 优点：
  - 表面上少一个 Tool
- 缺点：
  - 职责边界变糊，不利于 Skill 分层编排
  - 后续调试时不容易区分“探查问题”还是“装载问题”

## 采用设计

采用方案 A。

## Tool 设计

### 1）`inspect_sheet_range`

#### 输入
- `path`
- `sheet`
- `sample_rows` 可选，默认 `5`

#### 输出
- `path`
- `sheet`
- `used_range`，如 `B3:D8`
- `start_row`
- `start_col`
- `end_row`
- `end_col`
- `row_count_estimate`
- `column_count_estimate`
- `sample_rows`

#### 第一版边界
- 只扫描非空单元格边界，不做表头推断
- 空 Sheet 明确报错
- 行列编号按 Excel 习惯对外返回为 1-based
- `sample_rows` 返回 used range 内前 N 行原始值，供上层确认

### 2）`load_table_region`

#### 输入
- `path`
- `sheet`
- `range`，如 `B3:D10`
- `header_row_count` 可选，默认 `1`

#### 输出
- `path`
- `sheet`
- `range`
- `header_row_count`
- `columns`
- `rows`
- `row_count`
- `result_ref`

#### 第一版边界
- 只支持显式矩形区域
- 第一版不自动猜 header 行数，但允许显式指定 `header_row_count`
- 多层表头通过“把前 N 行表头路径合并成 canonical name”处理
- 第一版先返回 `result_ref`，不扩展 `table_ref` 持久化结构

## 架构落点

- `src/excel/sheet_range.rs`
  - Excel used range 扫描
  - A1 区域字符串解析 / 生成
  - `inspect_sheet_range`
- `src/frame/region_loader.rs`
  - 显式区域读取
  - `header_row_count` 解释
  - 组装 `LoadedTable`
- `src/tools/contracts.rs`
  - 暴露两个新 Tool 到 catalog
- `src/tools/dispatcher.rs`
  - 新增两个 dispatcher 入口

## 测试策略

### `inspect_sheet_range`
- 表不从 `A1` 开始时，返回正确 `used_range`
- 样本行与边界估计正确
- CLI tool catalog 包含 `inspect_sheet_range`
- CLI JSON 返回结构稳定

### `load_table_region`
- 显式区域能正确装载 DataFrame
- `header_row_count=2` 时，多层表头能被合并为稳定列名
- 非法区域语法报错
- CLI tool catalog 包含 `load_table_region`
- CLI 调用返回 preview + `result_ref`
