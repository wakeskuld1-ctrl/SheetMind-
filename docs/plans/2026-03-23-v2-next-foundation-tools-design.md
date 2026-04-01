# V2 第二批基础 Tool 设计

> 时间：2026-03-23
> 范围：V2 第二批基础 Tool——`parse_datetime_columns`、`lookup_values`、`window_calculation`

## 目标

在第一批基础 Tool 已补齐文本标准化、列改名、lookup 回填与透视之后，继续补三块更强的通用底座：
- `parse_datetime_columns`：把日期/时间文本标准化为统一格式，支撑时间序列分析与后续窗口计算。
- `lookup_values`：提供“按 key 带回字段但不做全表 join 语义暴露”的轻量查值能力。
- `window_calculation`：提供累计值、分组排名、序号等窗口计算，连接表处理层与分析建模层。

## 为什么选择这三块

- `parse_datetime_columns` 能把 Excel 里最常见的日期文本混乱先收口，否则月度分析、趋势分析与窗口函数都不稳定。
- `lookup_values` 比 `join_tables` 更轻，适合业务用户熟悉的“VLOOKUP / XLOOKUP”心智。
- `window_calculation` 能覆盖排名、累计、组内序号、环比前置准备等高频分析动作。

## 方案对比

### 方案 A：日期标准化 + 轻量查值 + 最小窗口函数（推荐）
- 做法：第二批先只补这三块最能桥接“表处理 -> 分析”的基础能力。
- 优点：复用价值高；不依赖业务语义；能自然承接前一批 Tool。
- 缺点：窗口函数的参数设计要做得足够保守。

### 方案 B：先做清洗层剩余 Tool，如 `distinct_rows` / `replace_values`
- 优点：实现简单。
- 缺点：能力增量相对有限，对分析建模层桥接帮助没有方案 A 明显。

### 方案 C：直接上更重的导出/图表/专题 Tool
- 优点：更接近用户可见功能。
- 缺点：基础数据口径还没完全稳住，容易把业务复杂性过早压进 Tool 层。

## 采用设计

采用方案 A。

## Tool 设计

### 1）`parse_datetime_columns`

#### 输入
- 单表来源：`path + sheet` / `table_ref` / `result_ref`
- `rules`
  - `column`
  - `target_type: date | datetime`

#### 输出
- `columns`
- `rows`
- `row_count`
- `result_ref`

#### 第一版边界
- `date`：统一输出 `YYYY-MM-DD`
- `datetime`：统一输出 `YYYY-MM-DD HH:MM:SS`
- 空值保持空值
- 非空但无法解析时报错
- 第一版先支持项目现有语义层已经能识别的常见格式：`YYYY-MM-DD`、`YYYY/MM/DD`、`YYYY-MM-DD HH:MM`、`YYYY-MM-DD HH:MM:SS`、`YYYY-MM-DDTHH:MM:SS`

### 2）`lookup_values`

#### 输入
- `base`：支持 `path + sheet` / `table_ref` / `result_ref`
- `lookup`：支持 `path + sheet` / `table_ref` / `result_ref`
- `base_on`
- `lookup_on`
- `selects`
  - `lookup_column`
  - `output_column`

#### 输出
- `columns`
- `rows`
- `row_count`
- `result_ref`

#### 第一版边界
- 会把 lookup 值带回主表，等价于“轻量显性查值”
- 不暴露 join 术语给上层
- lookup key 要求唯一，多命中报错
- 未命中保持空值
- 若 `output_column` 已存在，要求显式避让或报错

### 3）`window_calculation`

#### 输入
- 单表来源：`path + sheet` / `table_ref` / `result_ref`
- `partition_by: string[]` 可选
- `order_by: [{ column, descending }]`
- `calculations`
  - `kind: row_number | rank | cumulative_sum`
  - `source_column` 可选
  - `output_column`

#### 输出
- `columns`
- `rows`
- `row_count`
- `result_ref`

#### 第一版边界
- `row_number`：组内或全表顺序编号
- `rank`：按排序列做稠密排名
- `cumulative_sum`：按排序序列累计
- 第一版不做 lag/lead、rolling、percent_rank

## 与现有能力关系

- `parse_datetime_columns` 是 `stat_summary`、后续趋势分析与窗口计算的前置标准化层。
- `lookup_values` 衔接 `normalize_text_columns`、`rename_columns` 与业务层专题 Skill，覆盖 Excel 用户熟悉的查值动作。
- `window_calculation` 可接在 `sort_rows` / `group_and_aggregate` / `parse_datetime_columns` 后面，形成更强的分析准备链路。

## 实施顺序

1. `parse_datetime_columns`
2. `lookup_values`
3. `window_calculation`

## 测试策略

- `parse_datetime_columns`：日期规范化、日期时间规范化、空值保留、非法值报错
- `lookup_values`：带回字段、未命中留空、唯一键约束、输出列冲突报错、mixed source
- `window_calculation`：row number、dense rank、累计和、分组窗口、非数值累计报错
