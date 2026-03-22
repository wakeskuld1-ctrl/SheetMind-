# V2 多表计划设计

> 时间：2026-03-22
> 范围：V2.1 多表工作流层的第三块能力——多表顺序建议 Tool

## 目标

在已经具备 `suggest_table_links` 与 `suggest_table_workflow` 的基础上，再往前推一步：
系统面对 3 张及以上表时，能先给出一个保守的多表处理计划，回答“先追加哪些表、再关联哪些表、哪些表还要人工确认”。

这样上层 Skill 不需要自己做多表排序与分组推理，只需要读取计划步骤并把每一步翻译成用户确认问题。

## 为什么现在做这块

- 两表判断已经具备，但真实业务往往是多工作表、多工作本同时进入。
- 仅有 pairwise Tool 时，Skill 仍然要自己决定先合并哪几张表，再关联哪几张表，编排负担仍偏重。
- 多表计划器能把 V2 从“两表建议”推进到“多表保守编排建议”。

## 方案对比

### 方案 A：新增独立 `suggest_multi_table_plan` Tool（推荐）
- 做法：输入多张表，输出计划步骤、未决表、建议执行骨架和中文摘要。
- 优点：边界清晰；直接复用已有 `append_tables` / `suggest_table_links` / `suggest_table_workflow`；最适合 Skill 编排。
- 缺点：需要定义一个保守的贪心顺序规则。

### 方案 B：只返回多表关系矩阵
- 做法：给出所有两两之间更像追加/关联/待确认。
- 优点：实现更简单。
- 缺点：Skill 还得自己决定步骤顺序，编排价值不够高。

### 方案 C：直接做自动执行流水线
- 做法：系统自己决定并直接追加/关联。
- 优点：终态更强。
- 缺点：当前风险过高；用户确认还没完全沉淀，不适合 V2 直接自动执行。

## 采用设计

采用方案 A。

## Tool 设计

### 名称
- `suggest_multi_table_plan`

### 输入
- `tables`
  - `path`
  - `sheet`
  - `alias` 可选
- `max_link_candidates` 可选，默认 3

### 输出
- `steps`
  - `step_id`
  - `action`
  - `input_refs`
  - `result_ref`
  - `confidence`
  - `reason`
  - `suggested_tool_call`
- `unresolved_refs`
- `human_summary`
- `recommended_next_step`

## 计划规则

### 第一步：先做追加分组
- 对列集合完全一致的表，优先视为同结构表
- 按输入顺序做保守追加链
- 生成中间 `result_ref`，供后续步骤继续引用

### 第二步：再做显性关联
- 在追加后的代表表之间，寻找最明显的显性关联候选
- 仅当 `suggest_table_links` 能识别出足够明显的候选时，才生成关联步骤
- 当前默认生成 `matched_only` 的执行骨架，具体保留策略仍由上层继续确认

### 第三步：保留未决表
- 对既不能追加、也找不到明显关联候选的表，保留到 `unresolved_refs`
- 由 Skill 继续询问用户“这张表要不要参与当前流程”

## 边界

V2.1 本轮明确不做：
- 复杂最优路径搜索
- 多种 Join 策略自动切换
- 自动执行计划步骤
- 多表复合键推理

## 与现有能力关系

- `suggest_table_links` 负责两表显性关联候选
- `suggest_table_workflow` 负责两表动作判断
- `suggest_multi_table_plan` 负责多表步骤顺序建议
- `append_tables` / `join_tables` 仍然只负责真实执行
- Skill 负责把计划步骤翻译成确认问题并决定是否执行

## 测试策略

严格 TDD：
1. 先写失败测试，覆盖多表追加链、双表关联计划、无明显关系回退
2. 再接入最小实现
3. 跑定向测试、全量测试与 release 构建
