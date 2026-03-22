# V2 多表流程建议设计

> 时间：2026-03-22
> 范围：V2.1 多表工作流层的第二块能力——两表下一步动作建议 Tool

## 目标

在已经具备 `suggest_table_links` 的基础上，再往前推一步：
系统不仅判断“这两张表能不能明显关联”，还要判断“它们更像应该纵向追加、显性关联，还是需要继续人工确认”。

这样上层 Skill 在面对两张表时，不需要自己拼规则，只需要读取一个稳定的建议结果，再决定如何向用户提问。

## 为什么现在做这块

- `suggest_table_links` 解决了“怎么连”的一半问题，但真实用户还有另一半问题：这两张表到底是要合并在一起，还是要互相补信息。
- V1 已有 `append_tables` 与 `join_tables`，但它们都要求用户或 Skill 先决定动作；当前缺的是“先判断动作”的编排桥。
- 这块能力能把多表工作流推进到“先识别动作类型，再让 Skill 做确认”的阶段。

## 方案对比

### 方案 A：新增独立 `suggest_table_workflow` Tool（推荐）
- 做法：给两张表，统一返回“更像追加 / 更像关联 / 需要继续确认”的推荐动作，并附带追加候选和关联候选。
- 优点：边界清晰；不会污染 `append_tables` / `join_tables` 的执行职责；最适合 Skill 编排。
- 缺点：会多一个工作流层 Tool，需要维护一个简单的动作优先级规则。

### 方案 B：把动作判断塞进 `suggest_table_links`
- 做法：`suggest_table_links` 既判断关联，也顺带判断追加。
- 优点：用户表面上少一个 Tool。
- 缺点：职责开始混乱；“关系候选”与“动作建议”不是一个层次；后续扩展更难。

### 方案 C：直接做多表 DSL 编排器
- 做法：一次请求里把“先判断、再确认、再执行”都塞进去。
- 优点：长远形态完整。
- 缺点：当前过重；没有中间桥接层就很难稳定迭代。

## 采用设计

采用方案 A。

## Tool 设计

### 名称
- `suggest_table_workflow`

### 输入
- `left.path`
- `left.sheet`
- `right.path`
- `right.sheet`
- `left_casts` 可选
- `right_casts` 可选
- `max_link_candidates` 可选，默认 3

### 输出
- `recommended_action`
  - `append_tables`
  - `join_tables`
  - `manual_confirmation`
- `action_reason`
- `human_summary`
- `recommended_next_step`
- `append_candidate`
  - `confidence`
  - `shared_columns`
  - `left_row_count`
  - `right_row_count`
  - `reason`
  - `question`
- `link_candidates`
  - 直接复用 `suggest_table_links` 的候选结构

## 判断规则

### 追加优先级
当两张表的列集合完全一致时：
- 认为存在高置信度追加候选
- 默认推荐动作为 `append_tables`
- 原因是这更像“结构相同数据分批到达”而不是“主表 + 明细表补信息”

### 关联优先级
当结构不一致，但 `suggest_table_links` 能识别到明显关联候选时：
- 默认推荐动作为 `join_tables`
- 由 Skill 先把首个候选翻译成业务确认问题，再决定是否执行显性关联

### 人工确认回退
当既不满足追加，也没有明显关联候选时：
- 推荐 `manual_confirmation`
- 提示用户继续确认“是否是同结构表”“是否存在对应 ID 列”

## 边界

V2.1 本轮明确不做：
- 多于两张表的自动编排
- 自动直接执行追加或关联
- 复合键/模糊匹配/跨语言语义推理
- 追加与关联同时成立时的复杂概率评分

## 与现有能力关系

- `suggest_table_workflow` 负责“先判断下一步该走哪类动作”
- `suggest_table_links` 负责“如果要关联，先给出显性关联候选”
- `append_tables` / `join_tables` 负责“真正执行”
- Skill 负责“把建议翻译成用户问题，再决定是否调用执行 Tool”

## 测试策略

严格 TDD：
1. 先写内存层 / 真实 Excel 失败测试，覆盖推荐追加、推荐关联、人工确认回退
2. 再写 CLI 失败测试，覆盖工具目录暴露和 JSON 返回结构
3. 最小实现使测试通过
4. 跑定向测试、全量测试与 release 构建
