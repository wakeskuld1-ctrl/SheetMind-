# V2 基础 Tool 设计

> 时间：2026-03-23
> 范围：V2 第一批基础 Tool——`normalize_text_columns`、`rename_columns`、`fill_missing_from_lookup`、`pivot_table`

## 目标

在现有表处理与分析建模底座之上，补齐 4 个高频、通用、可组合的基础 Tool：
- `normalize_text_columns`：把 join / lookup 前最常见的文本清洗收口成稳定原子能力。
- `rename_columns`：把字段口径统一前置成独立步骤，降低 Skill 编排复杂度。
- `fill_missing_from_lookup`：补齐“主表缺失值回填”的基础查值能力。
- `pivot_table`：补齐 Excel 用户最熟悉的透视能力，作为表处理层通往分析层的桥接。

## 为什么现在做这 4 个

- `normalize_text_columns` 能明显提升显性关联、lookup 与规则判断的稳定性。
- `rename_columns` 能先统一字段命名，再复用已有 `group_and_aggregate`、`join_tables` 等能力。
- `fill_missing_from_lookup` 能承接主数据补全、标签补齐、代码映射等真实表处理场景。
- `pivot_table` 是 Excel 用户最强的心智模型之一，缺失它会让“表处理层”不完整。

## 方案对比

### 方案 A：新增 4 个独立 Tool 并复用现有 `LoadedTable -> result_ref` 范式（推荐）
- 做法：每个 Tool 在 `ops` 层独立实现，在 `dispatcher` 层统一接入 `path + sheet` / `table_ref` / `result_ref`。
- 优点：边界清晰；遵守 SRP；与现有单表 Tool 的调用体验一致；后续最容易给 Skill 复用。
- 缺点：`dispatcher` 需要继续扩展分发逻辑与参数解析。

### 方案 B：把能力塞进现有 Tool 的可选参数里
- 做法：把文本清洗、列改名、透视、lookup 回填作为现有 Tool 的附加参数。
- 优点：短期看起来文件改动更少。
- 缺点：Tool 语义会变混乱；错误处理难收口；后续 Skill 难以组合，不符合用户“Skill 只调用能力”的边界要求。

### 方案 C：直接做上层业务化专题 Tool
- 做法：把这 4 类能力揉进“客户分析”“经营专题”等更高层 Tool。
- 优点：更接近最终产品表象。
- 缺点：会把业务逻辑过早压入 Tool 层；基础能力还未补齐，不利于长期扩展。

## 采用设计

采用方案 A。

## Tool 设计

### 1）`normalize_text_columns`

#### 输入
- 单表来源：`path + sheet` / `table_ref` / `result_ref`
- `rules`
  - `column`
  - `trim: bool` 可选
  - `collapse_whitespace: bool` 可选
  - `lowercase: bool` 可选
  - `uppercase: bool` 可选
  - `remove_chars: string[]` 可选
  - `replace_pairs: [{ from, to }]` 可选

#### 输出
- `columns`
- `rows`
- `row_count`
- `result_ref`

#### 边界
- 第一版只处理文本标准化，不做拼音、分词、模糊去噪。
- 空值保持空值，不自动填补。
- 若列不存在，直接报错。

### 2）`rename_columns`

#### 输入
- 单表来源：`path + sheet` / `table_ref` / `result_ref`
- `mappings`
  - `from`
  - `to`

#### 输出
- `columns`
- `rows`
- `row_count`
- `result_ref`

#### 边界
- 只改列名，不改数据。
- 目标列名若冲突，直接报错。
- 源列不存在时报错。

### 3）`fill_missing_from_lookup`

#### 输入
- `base`：支持 `path + sheet` / `table_ref` / `result_ref`
- `lookup`：支持 `path + sheet` / `table_ref` / `result_ref`
- `base_on`
- `lookup_on`
- `fills`
  - `base_column`
  - `lookup_column`

#### 输出
- `columns`
- `rows`
- `row_count`
- `result_ref`

#### 边界
- 只在 `base_column` 为空时回填，不覆盖非空值。
- lookup key 第一版要求唯一；若一个 key 对应多条记录则报错。
- key 找不到时保持原值，不报错。

### 4）`pivot_table`

#### 输入
- 单表来源：`path + sheet` / `table_ref` / `result_ref`
- `rows: string[]`
- `columns: string[]`
- `values: string[]`
- `aggregation: sum | count | mean`

#### 输出
- `columns`
- `rows`
- `row_count`
- `result_ref`

#### 边界
- 第一版只做最小透视：单个或多个行维度、单个列维度、单值列聚合。
- `sum` / `mean` 仅接受数值列；非数值时报错。
- 空组合保持稳定列结构，不追求 Excel 完整透视样式。

## 错误处理原则

- 参数缺失：在 `dispatcher` 层返回明确中文报错。
- 列不存在：在 `ops` 层返回明确列名。
- 类型不满足：例如 `pivot_table` 的 `sum/mean` 对非数值列，直接报错。
- lookup 多命中：在 `fill_missing_from_lookup` 返回唯一性错误，要求上层先去重。
- 所有新 Tool 都继续复用统一 `result_ref` 持久化出口。

## 与现有能力关系

- `normalize_text_columns` 是 `join_tables` / `suggest_table_links` / `fill_missing_from_lookup` 的前置清洗底座。
- `rename_columns` 用于统一字段口径，再复用 `group_and_aggregate`、`sort_rows`、`stat_summary`。
- `fill_missing_from_lookup` 介于 `join_tables` 与业务专题 Tool 之间，是更保守的“只补值不扩表”能力。
- `pivot_table` 连接表处理层与分析层，可直接把宽表输出给 `export_excel` 或上层 Skill。

## 测试策略

严格按 TDD 执行：
1. 先在 `tests/integration_frame.rs` / `tests/integration_cli_json.rs` 写失败测试。
2. 先实现最小通过版本，再逐个扩展边界。
3. 定向跑新增测试，再跑 `cargo test -v` 与 `cargo build --release -v`。

### 重点测试点
- `normalize_text_columns`：去首尾空格、折叠空白、大小写统一、删除字符、替换对子、规则顺序稳定。
- `rename_columns`：正常重命名、部分重命名、目标列冲突报错、源列不存在报错。
- `fill_missing_from_lookup`：空值回填、不覆盖非空值、lookup 多命中报错、混合来源输入可用。
- `pivot_table`：`sum` / `count` / `mean`、行列交叉透视、非数值聚合报错、输出结构稳定。
