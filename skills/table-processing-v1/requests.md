# 表处理 Skill V1 固定请求模板

这些模板只覆盖当前 Tool 层已经稳定支持的请求。

使用原则：

- 先用建议型 Tool，再用执行型 Tool
- 先补参数，再发请求
- 不在 Skill 内自行发明新字段

## 路径恢复前置动作

这部分不是 Rust 计算 Tool，而是进入 Tool 之前的入口恢复动作。

### 1. Windows 路径格式纠偏

适用场景：

- 第一次打开工作簿失败
- 错误更像 Windows 路径语法不正确

动作顺序：

1. 明确告诉用户这更像路径格式问题，不是文件内容问题
2. 改用 Windows 原生反斜杠路径重新打开
3. 再继续 `open_workbook`、`normalize_table` 或后续表处理

### 2. 中文路径 ASCII 临时副本降级

适用场景：

- PowerShell 能定位到文件
- Tool 层直接读取中文路径失败

动作顺序：

1. 明确告诉用户文件存在，失败点更像中文路径兼容问题
2. 如果宿主环境支持复制动作，先征求用户确认；只有用户同意后，才复制到 ASCII 临时路径
3. 用 ASCII 临时副本继续 `open_workbook`
4. 继续读取 sheet 列表、判断表头、后续表处理

注意：

- 临时副本只用于分析，不代表修改原文件
- 如果宿主环境不支持复制能力，不要伪造“已经复制成功”

## 单表模板

### 0. 先列出工作簿 sheet

```json
{
  "tool": "open_workbook",
  "args": {
    "path": "tests/fixtures/basic-sales.xlsx"
  }
}
```

### 1. 表头判断

```json
{
  "tool": "normalize_table",
  "args": {
    "path": "tests/fixtures/basic-sales.xlsx",
    "sheet": "Sales"
  }
}
```

### 2. 预览前几行

```json
{
  "tool": "preview_table",
  "args": {
    "path": "tests/fixtures/basic-sales.xlsx",
    "sheet": "Sales",
    "limit": 5
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
    "columns": ["region", "sales"],
    "top_k": 5
  }
}
```

### 4. 选择列

```json
{
  "tool": "select_columns",
  "args": {
    "path": "tests/fixtures/basic-sales.xlsx",
    "sheet": "Sales",
    "columns": ["user_id", "region", "sales"]
  }
}
```

### 5. 条件筛选

```json
{
  "tool": "filter_rows",
  "args": {
    "path": "tests/fixtures/basic-sales.xlsx",
    "sheet": "Sales",
    "conditions": [
      {
        "column": "region",
        "operator": "equals",
        "value": "East"
      }
    ]
  }
}
```

### 6. 分组汇总

```json
{
  "tool": "group_and_aggregate",
  "args": {
    "path": "tests/fixtures/group-sales.xlsx",
    "sheet": "Sales",
    "casts": [
      {
        "column": "sales",
        "target_type": "int64"
      }
    ],
    "group_by": ["region"],
    "aggregations": [
      {
        "column": "sales",
        "operator": "sum"
      }
    ]
  }
}
```

### 7. 排序

```json
{
  "tool": "sort_rows",
  "args": {
    "path": "tests/fixtures/group-sales.xlsx",
    "sheet": "Sales",
    "casts": [
      {
        "column": "sales",
        "target_type": "int64"
      }
    ],
    "sorts": [
      {
        "column": "sales",
        "descending": true
      }
    ],
    "limit": 10
  }
}
```

### 8. 前 N 行

```json
{
  "tool": "top_n",
  "args": {
    "path": "tests/fixtures/group-sales.xlsx",
    "sheet": "Sales",
    "casts": [
      {
        "column": "sales",
        "target_type": "int64"
      }
    ],
    "sorts": [
      {
        "column": "sales",
        "descending": true
      }
    ],
    "n": 5
  }
}
```

## 双表模板

### 1. 双表工作流判断

```json
{
  "tool": "suggest_table_workflow",
  "args": {
    "left": {
      "path": "tests/fixtures/join-customers.xlsx",
      "sheet": "Customers"
    },
    "right": {
      "path": "tests/fixtures/join-orders.xlsx",
      "sheet": "Orders"
    },
    "max_link_candidates": 3
  }
}
```

### 2. 显性关联候选

```json
{
  "tool": "suggest_table_links",
  "args": {
    "left": {
      "path": "tests/fixtures/join-customers.xlsx",
      "sheet": "Customers"
    },
    "right": {
      "path": "tests/fixtures/join-orders.xlsx",
      "sheet": "Orders"
    },
    "max_candidates": 3
  }
}
```

### 3. 结构相同表追加

```json
{
  "tool": "append_tables",
  "args": {
    "top": {
      "path": "tests/fixtures/append-sales-a.xlsx",
      "sheet": "Sales"
    },
    "bottom": {
      "path": "tests/fixtures/append-sales-b.xlsx",
      "sheet": "Sales"
    },
    "limit": 10
  }
}
```

### 4. 显性关联执行

```json
{
  "tool": "join_tables",
  "args": {
    "left": {
      "path": "tests/fixtures/join-customers.xlsx",
      "sheet": "Customers"
    },
    "right": {
      "path": "tests/fixtures/join-orders.xlsx",
      "sheet": "Orders"
    },
    "left_on": "user_id",
    "right_on": "user_id",
    "keep_mode": "matched_only"
  }
}
```

## 多表模板

### 1. 多表顺序计划

```json
{
  "tool": "suggest_multi_table_plan",
  "args": {
    "tables": [
      {
        "path": "tests/fixtures/join-customers.xlsx",
        "sheet": "Customers",
        "alias": "customers"
      },
      {
        "path": "tests/fixtures/append-sales-a.xlsx",
        "sheet": "Sales",
        "alias": "sales_a"
      },
      {
        "path": "tests/fixtures/append-sales-b.xlsx",
        "sheet": "Sales",
        "alias": "sales_b"
      }
    ],
    "max_link_candidates": 3
  }
}
```

### 2. 多表第一步是追加时

如果计划第一步返回的是 `append_tables`，只执行第一步：

```json
{
  "tool": "append_tables",
  "args": {
    "top": {
      "path": "tests/fixtures/append-sales-a.xlsx",
      "sheet": "Sales"
    },
    "bottom": {
      "path": "tests/fixtures/append-sales-b.xlsx",
      "sheet": "Sales"
    },
    "limit": 10
  }
}
```

### 3. 多表第一步是关联时

如果计划第一步返回的是 `join_tables`，只执行第一步：

```json
{
  "tool": "join_tables",
  "args": {
    "left": {
      "path": "tests/fixtures/join-customers.xlsx",
      "sheet": "Customers"
    },
    "right": {
      "path": "tests/fixtures/join-orders.xlsx",
      "sheet": "Orders"
    },
    "left_on": "user_id",
    "right_on": "user_id",
    "keep_mode": "matched_only"
  }
}
```

## 当前不允许伪造的模板

下面这些当前不要在 Skill 里伪造：

- `result_ref` 作为 `append_tables.top`
- `result_ref` 作为 `append_tables.bottom`
- `result_ref` 作为 `join_tables.left`
- `result_ref` 作为 `join_tables.right`

原因：

- 当前 `dispatcher` 已支持 `append_tables` 和 `join_tables` 消费真实 `result_ref`
- `step_n_result` 仍是计划占位引用；如果后续步骤返回了 `pending_result_bindings`，要把前一步真实产出的 `result_ref` 填进 `result_ref_bindings`

## 2026-03-23 兼容补充
- 如果 `open_workbook` 或 `list_sheets` 已返回 `file_ref` 与 `sheets`，后续请求优先使用 `file_ref + sheet_index`。
- 面向用户时统一问“请确认是第几个 Sheet”，不要要求用户重复输入中文 Sheet 名。
- 如果需要复制到 ASCII 临时路径，必须先征求用户确认，不能默认直接复制。

## 2026-03-26 execute_multi_table_plan risk guard

When a user confirms plan execution, you can call `execute_multi_table_plan` and set risk guard limits for join preflight.

```json
{
  "tool": "execute_multi_table_plan",
  "args": {
    "tables": [
      {
        "path": "tests/fixtures/join-customers.xlsx",
        "sheet": "Customers",
        "alias": "customers"
      },
      {
        "path": "tests/fixtures/join-orders.xlsx",
        "sheet": "Orders",
        "alias": "orders"
      }
    ],
    "max_link_candidates": 3,
    "auto_confirm_join": true,
    "max_left_unmatched_rows": 10,
    "max_right_unmatched_rows": 10,
    "max_left_duplicate_keys": 5,
    "max_right_duplicate_keys": 5
  }
}
```

If a `join_preflight` step exceeds configured limits, execution stops with:
- `execution_status = "stopped_join_risk_threshold"`
- `stopped_at_step_id` set to that preflight step
- `executed_steps[n].join_risk_guard_breaches` listing breached metrics

Default behavior note:
- If `auto_confirm_join=true` and no thresholds are provided, runtime applies safe defaults:
  - `max_left_unmatched_rows = 10`
  - `max_right_unmatched_rows = 10`
  - `max_left_duplicate_keys = 5`
  - `max_right_duplicate_keys = 5`

## 2026-03-26 recovery templates

### A) resume after `stopped_missing_result_bindings`

If runtime says a step is missing aliases in `result_ref_bindings`, rerun with explicit bindings restored:

```json
{
  "tool": "execute_multi_table_plan",
  "args": {
    "plan": {"steps": []},
    "auto_confirm_join": true,
    "result_ref_bindings": {
      "step_1_result": "result_xxx"
    }
  }
}
```

### B) retry after `stopped_join_risk_threshold` with explicit confirmation

Only after user agrees to loosen thresholds:

```json
{
  "tool": "execute_multi_table_plan",
  "args": {
    "plan": {"steps": []},
    "auto_confirm_join": true,
    "max_left_unmatched_rows": 30,
    "max_right_unmatched_rows": 30,
    "max_left_duplicate_keys": 10,
    "max_right_duplicate_keys": 10
  }
}
```

If user does not approve threshold change, route to key cleanup first.

### C) recover from `execution_status=failed` with `failure_diagnostics`

If runtime returns unknown failure diagnostics, do diagnostics-first routing before retry:

```json
{
  "failure_diagnostics": {
    "recovery_templates": {
      "update_session_state": {
        "tool": "update_session_state",
        "args": {
          "session_id": "default",
          "current_stage": "table_processing",
          "last_user_goal": "resolve unknown multi-table failure at step_1"
        }
      },
      "resume_execution": {
        "tool": "execute_multi_table_plan",
        "args": {
          "session_id": "default",
          "plan": {"steps": [{"step_id": "step_1"}]},
          "auto_confirm_join": false,
          "stop_after_step_id": "step_1",
          "result_ref_bindings": {}
        }
      },
      "resume_full_chain": {
        "tool": "execute_multi_table_plan",
        "args": {
          "session_id": "default",
          "plan": {"steps": [{"step_id": "step_1"}]},
          "auto_confirm_join": false,
          "result_ref_bindings": {}
        }
      }
    }
  }
}
```

Interpretation rule:
- `failure_diagnostics.fallback_route = table_processing_diagnostics` means do not jump directly to analysis/modeling.
- First restore missing inputs/step args in table-processing flow, then execute:
  - `recovery_templates.resume_execution` for blocked-step replay, or
  - `recovery_templates.resume_full_chain` for full continuation.
- If replay returns `continuation_templates.default_template = resume_full_chain`, execute that template directly.

### D) one-call macro recovery

```json
{
  "tool": "recover_multi_table_failure",
  "args": {
    "failure_diagnostics": {},
    "continue_after_replay": true,
    "template_overrides": {
      "resume_execution": {
        "max_left_unmatched_rows": 30
      },
      "resume_full_chain": {
        "max_steps": 4
      }
    }
  }
}
```

This macro performs blocked-step replay first, then continues full chain when replay succeeds.
Use `template_overrides` when you only need to patch selected replay/continue args.
