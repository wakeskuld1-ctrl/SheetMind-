# 决策助手 Skill V1 固定请求模板

这些模板只覆盖当前 Tool 层已经稳定支持的请求。

使用原则：

- 已有 `table_ref` 时优先使用 `table_ref`
- 只有没有 `table_ref` 时才回退到 `path + sheet`
- 不在 Skill 内发明新的 readiness 字段或业务评分字段

## `table_ref` 入口

### 1. 默认决策助手

```json
{
  "tool": "decision_assistant",
  "args": {
    "table_ref": "table_1234567890",
    "top_k": 5
  }
}
```

### 2. 指定列进入决策助手

```json
{
  "tool": "decision_assistant",
  "args": {
    "table_ref": "table_1234567890",
    "columns": ["region", "sales", "is_member"],
    "top_k": 5
  }
}
```

### 3. 带显式类型转换的决策助手

```json
{
  "tool": "decision_assistant",
  "args": {
    "table_ref": "table_1234567890",
    "columns": ["zero_metric", "amount"],
    "casts": [
      {
        "column": "zero_metric",
        "target_type": "int64"
      },
      {
        "column": "amount",
        "target_type": "int64"
      }
    ],
    "top_k": 5
  }
}
```

## `path + sheet` 入口

### 1. 默认决策助手

```json
{
  "tool": "decision_assistant",
  "args": {
    "path": "tests/fixtures/analyze-distribution.xlsx",
    "sheet": "Metrics",
    "top_k": 5
  }
}
```

### 2. 带显式类型转换

```json
{
  "tool": "decision_assistant",
  "args": {
    "path": "tests/fixtures/analyze-distribution.xlsx",
    "sheet": "Metrics",
    "casts": [
      {
        "column": "zero_metric",
        "target_type": "int64"
      },
      {
        "column": "amount",
        "target_type": "int64"
      }
    ],
    "top_k": 5
  }
}
```

## 禁止项

- 不要在 Skill 内自己算优先级
- 不要自己构造“可直接上线”“建议立刻经营决策”之类结论
- 不要绕过 `decision_assistant` 去拼 `analyze_table + stat_summary` 的手工组合请求
