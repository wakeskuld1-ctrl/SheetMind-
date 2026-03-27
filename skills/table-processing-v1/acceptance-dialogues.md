# 表处理 Skill V1 模拟对话验收稿

这个文档用于人工验收 `table-processing-v1`。

验收目标不是看模型“会不会自由发挥”，而是看它是否符合这 4 条：

1. 先建议，后执行
2. 先确认表头，再做处理
3. 全程使用非技术表达
4. 不伪造当前 Tool 层还没有的链式执行能力

---

## 场景 0：工作簿首次打开失败，更像路径格式问题

### 用户

帮我继续看这份台账，刚才打开失败了。

### 期望 Skill 回复

当前理解：工具层里已经存在旧会话状态，但它指向测试文件，不适合直接复用到你这份台账。  
当前状态：第一次打开工作簿失败，错误更像是 Windows 路径语法不正确，问题更像是路径格式而不是文件内容。  
下一步动作：我会先改用 Windows 原生反斜杠路径重新打开，并把当前目标切回这份台账。  
对应动作：先做入口恢复，再继续 `open_workbook`

### 验收关注点

- 不要直接说文件内容有问题
- 不要跳过入口恢复

---

## 场景 0B：文件存在，但中文路径兼容失败

### 用户

文件是有的，你继续处理。

### 期望 Skill 回复

当前理解：文件本身存在，但当前工具链把中文路径传给底层 Excel 读取库时出了兼容问题，不是你这份文件找不到。  
当前状态：直接读中文路径会失败，不过 PowerShell 能正常定位到文件，所以我可以先复制一份只用于分析的 ASCII 路径副本，再继续表处理。  
下一步动作：我会先用 ASCII 临时副本读取工作簿的 Sheet 列表，再选出最像目标台账的 Sheet。  
对应动作：先做入口恢复，再继续 `open_workbook`

### 验收关注点

- 明确文件存在
- 明确这是中文路径兼容问题
- 不把 ASCII 副本说成修改原文件

---

## 场景 1：用户只想先看表

### 用户

帮我看看这个 Excel 里的 Sales 表。

### 期望 Skill 回复

当前理解：你想先快速看一下这张表的结构和前几行内容。  
下一步动作：我会先读取这份工作簿的 Sheet 列表，再判断目标表的表头是否清楚；如果清楚，再给你看前几行。  
对应请求：

```json
{
  "tool": "open_workbook",
  "args": {
    "path": "tests/fixtures/basic-sales.xlsx"
  }
}
```

### 若已确认目标 Sheet，下一轮期望请求

```json
{
  "tool": "normalize_table",
  "args": {
    "path": "tests/fixtures/basic-sales.xlsx",
    "sheet": "Sales"
  }
}
```

### 若表头明确，后续期望 Skill 回复

当前理解：这张表的表头已经明确。  
下一步动作：我会展示前 5 行给你确认内容。  
对应请求：

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

---

## 场景 2：用户要做单表汇总

### 用户

按地区统计销售额。

### 期望 Skill 回复

当前理解：你想看不同地区的销售额汇总。  
下一步动作：我会按地区分组，再汇总销售额；如果这列还不是数值，会先按模板转成数值再统计。  
对应请求：

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

### 验收关注点

- 不要说“聚合”“cast”“group by”
- 不要自己先口算一个结果再补请求

---

## 场景 3：两张表是否该追加

### 用户

这两张销售表帮我合一下。

### 期望 Skill 第一轮回复

当前理解：你想把两张销售表整理成一个统一结果，但目前还需要先判断更像“上下拼接”还是“按字段关联”。  
下一步问题：我先判断这两张表更适合哪种合并方式。  
对应请求：

```json
{
  "tool": "suggest_table_workflow",
  "args": {
    "left": {
      "path": "tests/fixtures/append-sales-a.xlsx",
      "sheet": "Sales"
    },
    "right": {
      "path": "tests/fixtures/append-sales-b.xlsx",
      "sheet": "Sales"
    },
    "max_link_candidates": 3
  }
}
```

### 若 Tool 推荐 append，期望 Skill 第二轮回复

当前理解：这两张表结构一致，更适合先上下合并。  
下一步问题：是否先把 B 表追加到 A 表下方，形成一张统一结果表？  
确认后动作：调用 `append_tables`。

### 若用户确认，期望请求

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

---

## 场景 4：两张表是否该关联

### 用户

把客户表和订单表合到一起看。

### 期望 Skill 第一轮回复

当前理解：你想把客户信息和订单信息放到一起看。  
下一步动作：我先判断这两张表更适合上下合并，还是通过某个字段关联。  
对应请求：

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

### 若 Tool 推荐 join，期望 Skill 第二轮回复

当前理解：这两张表更适合通过显性字段关联。  
下一步问题：是否用客户表的 `user_id` 关联订单表的 `user_id`？如果两边不完全一致，你希望只保留两边都有的数据，还是优先保留客户表或订单表？  
确认后动作：调用 `join_tables`。

### 若用户确认“只保留两边都有的数据”，期望请求

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

### 验收关注点

- 不要说 `left join` / `inner join`
- 必须先问保留方式

---

## 场景 5：多张表先追加再关联

### 用户

先把两张销售明细合并，再和客户表对上。

### 期望 Skill 第一轮回复

当前理解：这是一个多表场景，合理顺序应该先判断哪些表先合并，再决定和哪张主表去关联。  
下一步动作：我先给你一个保守处理计划，再逐步确认。  
对应请求：

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

### 若计划返回“先追加，再关联”，期望 Skill 第二轮回复

当前理解：当前计划建议分两步走：第一步先把两张销售明细表上下合并，第二步再把合并后的结果和客户表关联。  
下一步问题：是否先把 `sales_b` 追加到 `sales_a` 下方？  
确认后动作：先只执行第一步追加。

### 若用户确认，期望第一步执行请求

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

### 验收关注点

- 必须承认第二步里的 `step_1_result` 当前主要是计划引用
- 不要伪造“我已经自动把第二步也做完了”

---

## 场景 6：表头不明确

### 用户

先帮我清一下这张表。

### 期望 Skill 第一轮回复

当前理解：你想先把这张表整理成可处理状态。  
下一步动作：我先判断表头是否清楚；如果表头还不够明确，我会先请你确认。  
对应请求：

```json
{
  "tool": "normalize_table",
  "args": {
    "path": "tests/fixtures/title-gap-header.xlsx",
    "sheet": "Sheet1"
  }
}
```

### 若返回 `needs_confirmation`，期望 Skill 第二轮回复

当前理解：这张表的表头我还不能完全确认。  
下一步问题：你看一下这些列名是否符合你的意思；确认后我再继续处理。  
确认前动作：停止后续表处理请求。

### 验收关注点

- 返回 `needs_confirmation` 后必须停住
- 不能继续直接 `preview_table`、`group_and_aggregate`、`filter_rows`

---

## 场景 7：用户问“先怎么处理”

### 用户

我也不知道先看什么，你帮我判断一下。

### 期望 Skill 回复

当前理解：你现在更需要一个“先看哪里”的起步建议，而不是立刻做具体操作。  
下一步动作：如果是单表，我先给统计摘要；如果是两张表，我先判断该追加还是关联；如果是多张表，我先给处理计划。  
对应策略：

- 单表：`stat_summary`
- 双表：`suggest_table_workflow`
- 多表：`suggest_multi_table_plan`

### 验收关注点

- 不能把“判断过程”放到 Skill 自己自由发挥
- 必须优先调用建议型 Tool

---

## 场景 8：用户要求一步到位全自动多表执行

### 用户

你直接全部做完，不要一步步问我。

### 期望 Skill 回复

当前理解：你希望尽量减少中间确认，直接得到多表处理结果。  
下一步说明：当前 V1 可以先给出完整计划，也可以执行第一步最明确的动作；但涉及中间结果连续复用的后续步骤，我需要先诚实说明当前还主要是“计划 + 分步确认”模式。  
对应请求：

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

### 验收关注点

- 必须诚实暴露当前边界
- 不能假装 `step_n_result` 已可跨请求自动执行

## 2026-03-23 兼容补充
- 验收对话里要明确区分“路径/兼容问题”和“文件损坏”。
- 如果要走 ASCII 临时副本方案，回复里必须先询问用户是否同意。
- 一旦文件已打开，后续对话优先按“第几个 Sheet”继续。
