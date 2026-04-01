# V2 多表关系建议设计

> 时间：2026-03-21
> 范围：V2.1 多表工作流层的第一块能力——显性关联候选建议 Tool

## 目标

在现有 `join_tables` 只能“已知键名后执行”的基础上，新增一个面向 Skill 的关系建议 Tool，让系统先识别两张表之间最明显、最保守的显性关联候选，再由上层用业务语言去问用户是否采用。

## 为什么先做这块

- 真实 Excel 用户多数先卡在“这两张表怎么连”，而不是“Join 已经决定好了怎么执行”。
- 这块能力直接复用并增强 V1 的显性 Join，不改变既有执行语义，但能把多表工作流推进到“可引导”的阶段。
- 它是后续多表流水线、批量工作簿、结果血缘追踪的入口能力。

## 方案对比

### 方案 A：新增独立 `suggest_table_links` Tool（推荐）
- 做法：给两张表，返回明显关联候选、置信度、覆盖率、业务话术问题和 keep_mode 的中文选项提示。
- 优点：边界清晰；不破坏现有 `join_tables`；Skill 最容易编排。
- 缺点：需要维护一套候选关系评分规则。

### 方案 B：把候选建议塞进 `join_tables`
- 做法：如果没给 `left_on/right_on` 就自动猜。
- 优点：用户表面上少一个 Tool。
- 缺点：职责混乱；执行 Tool 混入建议逻辑；一旦猜错风险更大。

### 方案 C：直接做多步工作流 DSL
- 做法：一次请求里描述多表分析全链路。
- 优点：长远形态更完整。
- 缺点：当前 V2.1 过重；没有建议层会导致 DSL 仍然难写。

## 采用设计

采用方案 A。

## Tool 设计

### 名称
- `suggest_table_links`

### 输入
- `left.path`
- `left.sheet`
- `right.path`
- `right.sheet`
- `left_casts` 可选
- `right_casts` 可选
- `max_candidates` 可选，默认 3

### 输出
- `candidates`
  - `left_column`
  - `right_column`
  - `confidence`
  - `match_row_count`
  - `left_match_rate`
  - `right_match_rate`
  - `reason`
  - `question`
  - `keep_mode_options`
- `human_summary`
- `recommended_next_step`

## 候选规则

只识别“明显特征”的显性关联：
- 列名规范化后相同，如 `user_id` / `userId` / `userid`
- 两边都像标识列，如 `id`、`code`、`no`、`uid`、`编号`、`编码`
- 去掉空白值后有实际交集
- 交集覆盖率达到保守阈值

V2.1 明确不做：
- 跨语言语义映射推理
- 模糊字符串匹配 Join
- 复合键自动拼接
- 自动直接执行 Join

## 与现有能力关系

- `suggest_table_links` 负责“先建议”
- `join_tables` 负责“再执行”
- Skill 负责“把建议翻译成用户问题，再决定是否调用 `join_tables`”

## 测试策略

严格 TDD：
1. 先写内存层失败测试，覆盖高置信度候选与无明显候选
2. 再写 CLI 失败测试，覆盖目录暴露和真实 Excel 夹具
3. 再实现最小规则集使测试通过
4. 最后跑局部测试 + 全量测试
