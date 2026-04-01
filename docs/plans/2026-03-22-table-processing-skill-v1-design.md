# Table Processing Skill V1 Design

## 背景

当前 Rust 二进制已经具备一批稳定的表处理 Tool：

- 单表入口：`normalize_table`、`apply_header_schema`、`preview_table`
- 单表处理：`select_columns`、`filter_rows`、`cast_column_types`、`group_and_aggregate`、`sort_rows`、`top_n`
- 多表判断：`suggest_table_links`、`suggest_table_workflow`、`suggest_multi_table_plan`
- 多表执行：`append_tables`、`join_tables`
- 桥接分析：`stat_summary`

用户已经明确要求：

- Skill 只能调用能力，不能承担计算
- 计算必须全部留在 Rust Tool 层
- 面向低 IT 用户，问题话术必须非技术化
- 先做“表处理层”，再扩分析建模层和决策助手层

因此当前最合适的落地方式，不是写一个“大而全”的总控 Skill，而是先落一个表处理 Skill，把已经稳定的 Tool 按用户行为顺序编排起来。

## 目标

`表处理 Skill V1` 只做三件事：

1. 把用户自然语言需求路由到正确 Tool
2. 在必要时向用户追问最少量、最容易理解的问题
3. 把 Tool 返回结果转成下一步建议，而不是在 Skill 内做任何数据运算

## 非目标

本轮 Skill 不做这些事：

- 不直接承担任何数据计算或推导
- 不自己猜测 Join 键、聚合逻辑、列转换逻辑
- 不接管分析建模层 Tool 的完整编排
- 不假装支持当前引擎还没有实现的多步中间结果执行句柄

最后一点需要明确说明：当前 `suggest_multi_table_plan` 已经能产出 `step_n_result` 形式的计划引用，但 `dispatcher` 还没有把 `result_ref` 接成真正可复用的执行输入，所以 V1 Skill 在多表链式场景里应当“能规划、能解释、能逐步确认”，但不能伪装成“全自动连续执行”。

## 方案对比

### 方案 A：表处理 Skill V1（推荐）

只覆盖表处理层，把现有稳定 Tool 串起来。

优点：

- 最贴近当前 Tool 完成度
- 风险最低
- 最适合先验证真实用户问答路径
- 不会把未完成的建模层能力强行塞进首版 Skill

缺点：

- 分析建模和决策助手需要后续接入
- 多表链式执行目前仍以“计划 + 分步确认”为主

### 方案 B：总控 Excel Skill V1

直接覆盖表处理、建模、决策助手三层。

优点：

- 用户入口最统一
- 长远看最像最终产品形态

缺点：

- 当前 Tool 和交互边界还不够稳定
- 首版追问分支会非常多
- 一旦某个层级能力变化，Skill 容易整体返工

### 方案 C：拆成多个 Skill

分别做表处理、建模、决策助手三个 Skill。

优点：

- 结构最清晰
- 后续维护和迭代最方便

缺点：

- 首版用户入口分散
- 还需要额外总控路由层

## 推荐结论

先实现方案 A：`表处理 Skill V1`。

等表处理 Skill 跑稳后，再按同样模式扩出：

- `analysis-modeling-v1`
- `decision-assistant-v1`
- 最后再做总控 `excel-skill-v1`

## Skill 的职责边界

### Skill 负责

- 判断当前是单表、双表还是多表问题
- 先调用“建议型 Tool”，后调用“执行型 Tool”
- 把技术参数翻译成业务化确认问题
- 根据 Tool 结果决定下一步继续提问、执行还是停下解释

### Skill 不负责

- 列类型判断
- Join 键识别
- 表结构是否同构
- 聚合与排序算法
- 建模算法
- 中间结果计算

这些都必须下沉到 Tool。

## 对话流程设计

### 1. 单表入口

用户第一次提到某个 Excel / Sheet 时：

1. 先判断是否已经提供 `path` / `sheet`
2. 如果缺失，先补最少必要信息
3. 调 `normalize_table`
4. 若返回 `needs_confirmation`，让用户确认表头
5. 若返回 `ok`，再继续后续处理

### 2. 单表处理

单表处理优先顺序：

1. `preview_table`
2. `stat_summary` 或 `summarize_table`
3. 根据意图调用：
   - 看列：`select_columns`
   - 筛选：`filter_rows`
   - 转类型：`cast_column_types`
   - 分组汇总：`group_and_aggregate`
   - 排序：`sort_rows`
   - 取前 N：`top_n`

### 3. 两表处理

两表场景先调用 `suggest_table_workflow`：

- 如果推荐 `append_tables`
  - Skill 用“是否把 B 表追加到 A 表下方”来问
- 如果推荐 `join_tables`
  - Skill 用“是否用 A 表某列关联 B 表某列”来问
  - 并继续问保留方式：只保留两边都有 / 优先保留 A / 优先保留 B
- 如果推荐 `manual_confirmation`
  - Skill 不乱执行，继续收集业务意图

### 4. 多表处理

三表及以上先调用 `suggest_multi_table_plan`：

- Skill 要先解释步骤顺序
- 要用 `question` 字段逐步与用户确认
- 当前 V1 只把它当“计划与解释层”
- 不把 `step_n_result` 伪装成已经可跨调用复用的执行句柄

这意味着：

- V1 可以做“多表顺序规划”
- V1 可以做“第一步执行”
- V1 不承诺“任意多步链式自动执行到底”

## 语言与话术约束

Skill 内统一用非技术表达：

- 不说 `left join`
- 不说 `inner join`
- 不说 `schema inference`
- 不说 `cast`

统一翻译成：

- “只保留两边都有的数据”
- “优先保留 A 表”
- “优先保留 B 表”
- “这张表的表头还需要你确认一下”
- “先把结构相同的表上下合并”

## 输出结构

首版 Skill 输出建议采用固定三段式：

1. 当前理解
2. 下一步动作或确认问题
3. 如果用户确认，将调用的 Tool

这样做的目的，是让用户始终知道系统“为什么这么问”和“确认后会做什么”。

## 需要显式暴露的限制

Skill 文档里必须写清楚这些限制：

- 当前多表链式计划主要用于说明顺序，不等于所有中间结果都可直接跨调用执行
- 当前表头不明确时，必须先确认，不能跳过
- 当前 Skill 只覆盖表处理层，不直接进入回归、逻辑回归、聚类与决策助手

## 验收标准

只要做到下面 5 点，就认为 `表处理 Skill V1` 可进入首轮验收：

1. 能区分单表、双表、多表
2. 能优先调用建议型 Tool，而不是直接乱执行
3. 能用非技术语言向用户提问
4. 能把 Tool 返回结果翻译成下一步动作
5. 能明确暴露当前多表自动执行的边界
