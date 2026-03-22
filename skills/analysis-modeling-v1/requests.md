# 分析建模 Skill V1 固定请求模板

这些模板只覆盖当前 Tool 层已经稳定支持的请求。

使用原则：

- 默认先诊断，再决定是否进入建模
- 用户明确点名模型时，也要先补齐关键参数
- 不在 Skill 内自行发明新字段
- 如果上游已经给出 `table_ref`，优先用 `table_ref`，只有没有句柄时才退回 `path + sheet`
- 当前 `missing_strategy` 只写 `drop_rows`
- 当前 `casts.target_type` 只使用 `string`、`int64`、`float64`、`boolean`

## `table_ref` 优先模板

### 1. 上游确认态进入统计摘要

```json
{
  "tool": "stat_summary",
  "args": {
    "table_ref": "table_1234567890",
    "columns": ["sales", "region", "is_member"],
    "top_k": 5
  }
}
```

### 2. 上游确认态进入观察诊断

```json
{
  "tool": "analyze_table",
  "args": {
    "table_ref": "table_1234567890",
    "top_k": 5
  }
}
```

## 观察诊断模板

### 1. 默认健康诊断

```json
{
  "tool": "analyze_table",
  "args": {
    "path": "tests/fixtures/basic-sales.xlsx",
    "sheet": "Sales",
    "top_k": 5
  }
}
```

### 2. 只看指定列的健康诊断

```json
{
  "tool": "analyze_table",
  "args": {
    "path": "tests/fixtures/basic-sales.xlsx",
    "sheet": "Sales",
    "columns": ["region", "sales", "customer_id"],
    "top_k": 5
  }
}
```

### 3. 统计摘要

```json
{
  "tool": "stat_summary",
  "args": {
    "path": "tests/fixtures/basic-sales.xlsx",
    "sheet": "Sales",
    "columns": ["region", "sales", "is_member"],
    "top_k": 5
  }
}
```

### 4. 基础列画像

```json
{
  "tool": "summarize_table",
  "args": {
    "path": "tests/fixtures/basic-sales.xlsx",
    "sheet": "Sales",
    "columns": ["region", "sales", "is_member"],
    "top_k": 5
  }
}
```

### 5. 带显式类型转换的统计摘要

```json
{
  "tool": "stat_summary",
  "args": {
    "path": "tests/fixtures/basic-sales.xlsx",
    "sheet": "Sales",
    "columns": ["sales", "gross_margin", "is_member"],
    "casts": [
      {
        "column": "sales",
        "target_type": "float64"
      },
      {
        "column": "gross_margin",
        "target_type": "float64"
      },
      {
        "column": "is_member",
        "target_type": "boolean"
      }
    ],
    "top_k": 5
  }
}
```

## 线性回归模板

### 1. 标准线性回归

```json
{
  "tool": "linear_regression",
  "args": {
    "path": "tests/fixtures/model-sales.xlsx",
    "sheet": "Sales",
    "features": ["customer_count", "ad_spend", "discount_rate"],
    "target": "sales_amount",
    "intercept": true,
    "missing_strategy": "drop_rows"
  }
}
```

### 2. 带显式类型转换的线性回归

```json
{
  "tool": "linear_regression",
  "args": {
    "path": "tests/fixtures/model-sales.xlsx",
    "sheet": "Sales",
    "features": ["customer_count", "ad_spend", "discount_rate"],
    "target": "sales_amount",
    "casts": [
      {
        "column": "customer_count",
        "target_type": "float64"
      },
      {
        "column": "ad_spend",
        "target_type": "float64"
      },
      {
        "column": "discount_rate",
        "target_type": "float64"
      },
      {
        "column": "sales_amount",
        "target_type": "float64"
      }
    ],
    "intercept": true,
    "missing_strategy": "drop_rows"
  }
}
```

### 3. 只在用户明确要求时关闭截距项

```json
{
  "tool": "linear_regression",
  "args": {
    "path": "tests/fixtures/model-sales.xlsx",
    "sheet": "Sales",
    "features": ["customer_count", "ad_spend"],
    "target": "sales_amount",
    "intercept": false,
    "missing_strategy": "drop_rows"
  }
}
```

## 逻辑回归模板

### 1. 标准逻辑回归

```json
{
  "tool": "logistic_regression",
  "args": {
    "path": "tests/fixtures/model-churn.xlsx",
    "sheet": "Customers",
    "features": ["order_count", "refund_count", "last_active_days"],
    "target": "is_churn",
    "positive_label": "yes",
    "intercept": true,
    "missing_strategy": "drop_rows"
  }
}
```

### 2. 带显式类型转换的逻辑回归

```json
{
  "tool": "logistic_regression",
  "args": {
    "path": "tests/fixtures/model-churn.xlsx",
    "sheet": "Customers",
    "features": ["order_count", "refund_count", "last_active_days"],
    "target": "is_churn",
    "positive_label": "yes",
    "casts": [
      {
        "column": "order_count",
        "target_type": "float64"
      },
      {
        "column": "refund_count",
        "target_type": "float64"
      },
      {
        "column": "last_active_days",
        "target_type": "float64"
      }
    ],
    "intercept": true,
    "missing_strategy": "drop_rows"
  }
}
```

### 3. 当前不要省略正类标签

逻辑回归当前模板里，不要默认省略 `positive_label`。

原因：

- Skill 不应自己猜哪个值算用户最关心的结果
- 即使底层在某些场景可自动推断，Skill V1 仍应优先明确询问
- 如果目标列分布还没看过，优先先走一次 `stat_summary`，避免在单一类别上直接撞到逻辑回归错误

## 聚类模板

### 1. 标准 KMeans 聚类

```json
{
  "tool": "cluster_kmeans",
  "args": {
    "path": "tests/fixtures/model-segments.xlsx",
    "sheet": "Customers",
    "features": ["annual_spend", "visit_count", "refund_rate"],
    "cluster_count": 3,
    "max_iterations": 100,
    "missing_strategy": "drop_rows"
  }
}
```

### 2. 带显式类型转换的 KMeans 聚类

```json
{
  "tool": "cluster_kmeans",
  "args": {
    "path": "tests/fixtures/model-segments.xlsx",
    "sheet": "Customers",
    "features": ["annual_spend", "visit_count", "refund_rate"],
    "casts": [
      {
        "column": "annual_spend",
        "target_type": "float64"
      },
      {
        "column": "visit_count",
        "target_type": "float64"
      },
      {
        "column": "refund_rate",
        "target_type": "float64"
      }
    ],
    "cluster_count": 3,
    "max_iterations": 100,
    "missing_strategy": "drop_rows"
  }
}
```

### 3. 首轮试探分组数

如果用户暂时没有想法，可以建议首轮先试 `3` 组，但不能把这个建议伪装成系统已自动判定的最优值。

## 诊断后进入建模的模板串联

### 1. 先诊断，再做线性回归

第一步：

```json
{
  "tool": "analyze_table",
  "args": {
    "path": "tests/fixtures/model-sales.xlsx",
    "sheet": "Sales",
    "columns": ["customer_count", "ad_spend", "discount_rate", "sales_amount"],
    "top_k": 5
  }
}
```

第二步：用户确认继续后，再发线性回归模板。

### 2. 先看统计摘要，再做逻辑回归

第一步：

```json
{
  "tool": "stat_summary",
  "args": {
    "path": "tests/fixtures/model-churn.xlsx",
    "sheet": "Customers",
    "columns": ["is_churn", "order_count", "refund_count", "last_active_days"],
    "top_k": 5
  }
}
```

第二步：确认正类标签和特征列后，再发逻辑回归模板。

### 3. 先看字段分布，再做聚类

第一步：

```json
{
  "tool": "stat_summary",
  "args": {
    "path": "tests/fixtures/model-segments.xlsx",
    "sheet": "Customers",
    "columns": ["annual_spend", "visit_count", "refund_rate"],
    "top_k": 5
  }
}
```

第二步：确认特征列和分组数后，再发聚类模板。

## 当前不允许伪造的模板

下面这些当前不要在 Skill 里伪造：

- 未向用户确认就自动补 `target`
- 未向用户确认就自动补 `positive_label`
- 未向用户确认就自动补 `cluster_count`
- 写出当前不存在的 `missing_strategy` 选项
- 写出当前不存在的建模指标请求字段
- 把 `decision_assistant` 混进分析建模 Skill V1 的主执行模板

原因：

- 当前分析建模层 V1 的目标是“稳态路由 + 传统计算 Tool 调用”，不是自动替用户做建模决策
- Skill 只能组织调用，不能替 Tool 发明协议字段
