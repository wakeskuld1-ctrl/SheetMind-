# Statistical Summary Tool Design

## 背景

当前项目已经具备两类相邻能力：

- `summarize_table`：负责列级基础画像，适合回答“这列是什么类型、缺失多少、常见值有哪些”
- `analyze_table`：负责质量诊断和轻量业务观察，适合回答“这张表现在适不适合继续分析、有哪些风险”

但在“表处理”进入“分析建模”之前，还缺一层专门面向统计桥接的 Tool。后续线性回归、逻辑回归、聚类和决策助手都需要一个更稳定的统计视图，而不是直接复用基础画像或质量诊断结果。

## 目标

新增独立 `stat_summary` Tool，作为表处理层与分析建模层之间的桥接能力，满足以下目标：

- 输出机器可消费的结构化统计摘要
- 输出非 IT 用户可读的中文摘要
- 复用现有 `LoadedTable`、`DataFrame`、`summarize_table` 能力，不把计算放到 Skill
- 保持单二进制交付，不引入额外部署要求

## 非目标

V1 明确不做以下能力：

- 相关性矩阵、显著性检验、假设检验
- 自动建模建议、自动目标列推荐
- 多表联合统计摘要
- 时间序列专用统计
- 可视化图表输出

## 方案对比

### 方案 A：新增独立 `stat_summary` Tool

这是本次已确认采用的方案。

优点：

- 职责边界清晰：基础画像、质量诊断、统计桥接三层分离
- 不破坏现有 `summarize_table` / `analyze_table` 协议
- 更利于后续分析建模层按需消费

缺点：

- Tool 数量增加一个
- 和 `summarize_table` 存在少量字段重叠

### 方案 B：扩展 `summarize_table`

优点：

- 用户感知入口更少

缺点：

- `summarize_table` 语义会变重
- 不利于后续演化和职责稳定

### 方案 C：并入 `analyze_table`

优点：

- 一次调用拿更多结果

缺点：

- 统计与诊断耦合
- 后续建模 Tool 依赖链会变脆

## 设计原则

### 1. 统计桥接优先，不抢分析建模职责

`stat_summary` 只提供“能否进入建模前的统计视图”，不直接做模型训练或业务推理。

### 2. 双层输出

- 结构化输出：给 Tool、Skill、后续建模层消费
- 人类摘要：给终端问答界面直接展示

### 3. 保守而稳定

V1 优先保证：

- 输出口径稳定
- 空列、脏列、不规则列不 panic
- 数值/类别/布尔列的统计摘要都可解释

## Tool 定位

### Tool 名称

`stat_summary`

### 与现有 Tool 的关系

- `summarize_table`：产出列级基础画像
- `analyze_table`：产出质量诊断和业务观察
- `stat_summary`：产出建模前的结构化统计桥接结果

后续建模层会优先消费 `stat_summary`，必要时再参考 `analyze_table` 的风险结论。

## 输入设计

V1 输入参数：

- `path`：Excel 路径，必填
- `sheet`：工作表名，必填
- `columns`：可选；不传则统计整张表
- `casts`：可选；显式列类型转换
- `top_k`：可选；控制类别列高频值数量，默认 `5`

V1 不开放复杂统计参数，先保证默认行为稳定。

## 输出设计

```json
{
  "row_count": 1000,
  "column_count": 8,
  "table_overview": {
    "numeric_columns": 3,
    "categorical_columns": 4,
    "boolean_columns": 1
  },
  "numeric_summaries": [
    {
      "column": "sales",
      "count": 1000,
      "null_count": 10,
      "missing_rate": 0.01,
      "min": 10.0,
      "q1": 50.0,
      "median": 90.0,
      "mean": 105.0,
      "q3": 140.0,
      "max": 980.0,
      "sum": 103950.0,
      "zero_ratio": 0.0
    }
  ],
  "categorical_summaries": [
    {
      "column": "region",
      "count": 1000,
      "null_count": 0,
      "missing_rate": 0.0,
      "distinct_count": 4,
      "top_values": [
        { "value": "华东", "count": 420 }
      ],
      "top_share": 0.42
    }
  ],
  "boolean_summaries": [
    {
      "column": "is_active",
      "count": 1000,
      "null_count": 0,
      "missing_rate": 0.0,
      "true_count": 820,
      "false_count": 180,
      "true_ratio": 0.82
    }
  ],
  "human_summary": {
    "overall": "这张表已经具备基础统计摘要，可进入下一步分析。",
    "key_points": [
      "sales 列存在明显长尾，高值记录较少但拉高均值",
      "region 主要集中在华东"
    ],
    "recommended_next_step": "建议先选择目标列和特征列，再进入建模前检查。"
  }
}
```

## 统计口径

### 数值列

V1 输出：

- `count`
- `null_count`
- `missing_rate`
- `min`
- `q1`
- `median`
- `mean`
- `q3`
- `max`
- `sum`
- `zero_ratio`

V1 暂不强制输出 `std`，原因是当前先把四分位数与中心统计打稳；若实现成本低且测试稳定，可一并纳入首版。

### 类别列

V1 输出：

- `count`
- `null_count`
- `missing_rate`
- `distinct_count`
- `top_values`
- `top_share`

### 布尔列

V1 输出：

- `count`
- `null_count`
- `missing_rate`
- `true_count`
- `false_count`
- `true_ratio`

## Human Summary 规则

人类摘要遵循“质量优先、业务少量”的原则：

- 先给总体判断
- 再给 2-3 条关键统计观察
- 最后给下一步建议

中文文案必须直白，不出现过多专业术语。

## 错误处理

- 缺少 `path` / `sheet`：返回结构化错误
- `columns` 中存在缺失列：沿用现有 `MissingColumn` 风格
- `casts` 失败：返回直白中文错误
- 全空列、单值列、空表：返回稳定结构，不 panic

## 测试策略

坚持 TDD：

1. 先写内存层失败测试，锁定数值/类别/布尔统计结构
2. 再写 CLI 层失败测试，锁定 Tool JSON 协议
3. 再做最小实现
4. 跑定向测试、全量测试、release 构建、二进制冒烟

建议首批测试覆盖：

- 标准数值列
- 偏态数值列
- 高零值数值列
- 类别列 top share
- 布尔列 true ratio
- 全空列
- 真实 Excel 输入 + cast
