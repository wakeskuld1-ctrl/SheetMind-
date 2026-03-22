# Analyze Table Design

## 背景

表处理层 V1 已经完成 `open_workbook`、`normalize_table`、`apply_header_schema`、`preview_table`、`select_columns`、`filter_rows`、`cast_column_types`、`group_and_aggregate`、`sort_rows`、`top_n`、`join_tables`、`append_tables`、`summarize_table` 等能力，并且已经具备单二进制交付形态。

下一阶段需要从“表处理”平滑过渡到“分析建模”。为此，需要一个桥接 Tool，先替用户判断一张表是否适合继续分析、主要风险在哪里、下一步该做什么，而不是立刻进入回归、聚类或分类建模。

## 目标

新增 `analyze_table` Tool，完成以下目标：

- 以数据质量诊断为主，顺带给出少量业务统计观察。
- 输出双层结果：
  - 结构化 JSON，便于 Skill 和后续 Tool 编排。
  - 简洁中文结论，便于非 IT 用户直接阅读。
- 严格保持边界：
  - Skill 只做编排。
  - 所有计算都在 Rust Tool 层完成。
- 作为分析建模层 V1 的第一个桥接 Tool，为后续回归、聚类和决策助手提供稳定输入。

## 非目标

本阶段明确不做：

- 线性回归、逻辑回归、聚类等真正建模算法。
- 自动修改原始表或自动清洗数据。
- 多表联合诊断或跨表业务语义推断。
- 复杂 BI 级分组分析、动态图表或可视化。
- 将 AI 推断放入 Tool 内部代替真实统计计算。

## 设计原则

### 1. 质量诊断优先

V1 的核心职责是回答：

- 这张表能不能继续分析？
- 哪几列问题最严重？
- 哪些问题会影响后续建模或关联？
- 下一步更应该清洗，还是可以先做基础分析？

### 2. 规则引擎为主，轻量统计增强为辅

V1 采用“规则诊断 + 轻量统计增强”模式：

- 规则部分：缺失、重复、疑似键风险、类别失衡、低信息量列。
- 轻量统计部分：异常值、零值占比、离散分布、极值提醒。

这样可以在保持解释性和稳定性的前提下，给用户提供“像高级分析师一样”的第一层诊断能力。

### 3. 双层输出

输出必须同时服务两类对象：

- `structured_findings`：给 Skill、后续 Tool、未来 UI 编排。
- `human_summary`：给 Excel 用户直接看。

## Tool 定位

### Tool 名称

`analyze_table`

### Tool 层级关系

- `summarize_table`：提供列级基础画像。
- `analyze_table`：解释这些画像意味着什么，并补充更高层诊断结论。
- 后续建模 Tool：基于 `analyze_table` 的诊断结果决定是否建模、如何建模。

## 输入设计

V1 计划支持以下输入参数：

- `path`: Excel 文件路径，必填。
- `sheet`: 工作表名称，必填。
- `columns`: 可选；不传时分析整张表。
- `casts`: 可选；允许用户或 Skill 显式指定列类型。
- `top_k`: 可选；控制高频值和重点项数量，默认建议为 `5`。
- `focus`: 可选；V1 默认值为 `quality_first`。

### 输入设计说明

- 不提前开放复杂 `group_by`、`metric_columns`、`candidate_keys` 等配置，避免 V1 变成半个 BI 引擎。
- 诊断默认围绕单张已确认结构的表展开。

## 输出设计

输出采用双层结构：

```json
{
  "row_count": 1234,
  "column_count": 18,
  "table_health": {
    "level": "warning",
    "score": 0.62
  },
  "structured_findings": [
    {
      "code": "high_missing_rate",
      "severity": "high",
      "scope": "column",
      "column": "phone",
      "message": "手机号列缺失较多",
      "evidence": {
        "missing_rate": 0.42,
        "null_count": 518
      },
      "suggestion": "如果手机号不是分析目标，可以先排除该列"
    }
  ],
  "business_observations": [
    {
      "type": "top_category",
      "column": "region",
      "message": "华东是出现最多的区域"
    }
  ],
  "next_actions": [
    "建议先处理缺失值较高的列",
    "建议检查 user_id 是否可作为唯一标识"
  ],
  "human_summary": {
    "overall": "这张表可以继续分析，但有几列需要先清洗。",
    "major_issues": [
      "手机号列缺失较多",
      "客户编号疑似存在重复"
    ],
    "quick_insights": [
      "华东是出现最多的区域",
      "sales 列有少数明显偏高记录"
    ],
    "recommended_next_step": "建议先做缺失值与重复值处理，再进入分组分析或建模。"
  }
}
```

## 诊断模型

V1 分为五大类诊断：

### 1. 表级健康诊断

输出整张表的总体健康状态：

- `good`
- `warning`
- `risky`

判断依据包括：

- 是否存在全空列。
- 是否存在高缺失列。
- 是否存在重复行。
- 是否存在疑似主键重复或空值。
- 是否存在明显异常值、低信息量列或类别失衡列。

### 2. 列级质量诊断

每列至少包含：

- `column`
- `dtype`
- `summary_kind`
- `count`
- `null_count`
- `missing_rate`

并在此基础上生成更高层 finding：

- `all_missing`
- `high_missing_rate`
- `medium_missing_rate`
- `single_value_column`
- `placeholder_heavy`
- `mixed_type_like`
- `whitespace_dirty`
- `suspicious_identifier`

### 3. 重复与键风险诊断

V1 重点覆盖：

- `duplicate_rows`
- `duplicate_candidate_key`
- `blank_candidate_key`

候选键识别采用保守规则：

- 列名包含 `id`、`code`、`no`、`编号`、`编码` 等特征。
- 或列唯一率显著偏高且命名形态接近标识列。

### 4. 分布与异常诊断

#### 数值列

- `outlier_suspected`
- `high_zero_ratio`
- `near_constant`
- `extreme_range`

#### 类别列

- `high_category_imbalance`
- `too_many_distinct_values`
- `rare_categories_present`

异常值采用 IQR 等轻量统计规则，不引入复杂建模。

### 5. 少量业务统计提示

V1 只提供少量、可解释的业务观察：

- 类别列 top category。
- 数值列极值提醒。
- 疑似金额/销量列范围提示。
- 对明显主维度列给出“适合优先分组观察”的提示。

## 阈值策略

V1 使用固定阈值，不向终端用户暴露：

- 高缺失：`missing_rate >= 0.30`
- 中缺失：`0.10 <= missing_rate < 0.30`
- 全空列：`missing_rate == 1.0`
- 类别高度集中：`top1_share >= 0.80`
- 零值占比高：`zero_ratio >= 0.80`
- 近乎常量：有效值 `distinct_count <= 1` 或数值变化极小

阈值固定的理由：

- 非 IT 用户不适合在 V1 面对过多配置。
- 先确保行为稳定，再考虑将来开放配置。

## 中文摘要风格

`human_summary` 采用固定结构：

- 一句总体判断。
- 三条最重要问题。
- 两条快速观察。
- 一条下一步建议。

要求：

- 不直接暴露过多技术术语。
- 文案要能让用户立刻明白“这张表还能不能继续用”。
- 如果因为数据量太小无法判断，也要用直白中文表达。

## 架构设计

### 新增模块

计划新增：

- `src/ops/analyze.rs`

该模块负责：

- 聚合 `summarize_table` 结果。
- 对 DataFrame 做补充扫描。
- 产出结构化 finding。
- 生成中文摘要。

### 复用模块

- `src/ops/summary.rs`
- `src/ops/cast.rs`
- `src/tools/dispatcher.rs`
- `src/tools/contracts.rs`

### 计算边界

- `analyze_table` 不通过“套调用 Tool”实现，而是在 Rust 内部直接复用底层函数。
- 这样可以避免 Tool 套 Tool 造成协议耦合和重复序列化成本。

## 错误处理设计

错误文案保持用户友好：

- 缺 `path` / `sheet`
- 指定列不存在
- 目标表还没确认表头
- 数据量太少，无法做某项判断
- 列类型不适合某类诊断

错误示例：

- “这列有效数据太少，暂时无法判断是否存在异常值”
- “这列更像文本，不适合做数值异常分析”

## 测试策略

采用 TDD，至少覆盖四组测试：

### 1. 质量诊断基础

- 高缺失列
- 全空列
- 占位缺失列

### 2. 重复与键风险

- 重复行
- 疑似 ID 重复
- 空 ID

### 3. 分布与异常

- 类别高度集中
- 数值异常值
- 零值占比过高

### 4. 中文摘要

- `human_summary` 返回结构稳定
- 结论、问题、建议都可直接展示

## 预期收益

完成 `analyze_table` 后，系统将具备以下桥接能力：

- 用户拿到 Excel 后，不需要先懂 DataFrame 就能知道表是否健康。
- Skill 可以基于结构化 finding 自动决定下一步动作。
- 后续回归、聚类、逻辑回归 Tool 可以共享这套前置诊断结果。

## 下一阶段衔接

`analyze_table` 完成后，分析建模层 V1 可以继续推进：

- `describe/analyze` 类工具增强
- 线性回归
- 逻辑回归
- 聚类

其中建模入口将优先依赖 `analyze_table` 输出，用于决定：

- 是否需要先清洗
- 是否存在明显数据质量风险
- 哪些列适合作为目标列或特征列候选
