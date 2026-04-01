# Excel Orchestrator V1 Design

## 背景

当前系统已经形成三层 Skill：

- `table-processing-v1`
- `analysis-modeling-v1`
- `decision-assistant-v1`

但用户仍然需要自己判断“现在该进哪一层”。这对低 IT 用户不友好，也会导致同一个工作簿在表处理、分析建模、决策助手之间反复丢上下文。

因此需要新增一个总入口 Skill：`excel-orchestrator-v1`。

它不做计算，不替代三层 Skill，只做：

1. 入口意图识别
2. 会话状态摘要维护
3. 三层 Skill 路由切换
4. 对外统一话术

## 设计目标

- 用户只面对一个入口，不需要自己判断该用哪层 Skill
- 上游表处理确认态要能自然承接到分析建模与决策助手层
- `table_ref` 成为跨层复用的第一优先句柄
- 会话状态不依赖大模型上下文，而是定义成可持久化的独立状态摘要协议
- Skill 继续保持“只做路由、追问、解释，不做计算”

## 非目标

- 不把三层 Skill 合并成一个大 Skill
- 不让总入口 Skill 直接做表处理、建模、决策计算
- 不在 V1 做复杂状态机、自动规划执行器或多步结果血缘树
- 不在总入口 Skill 中引入业务规则推断或模型自动选择

## 核心定位

`excel-orchestrator-v1` 是统一入口层，不是万能 Skill。

它的责任边界：

- 读取当前会话状态摘要
- 根据用户意图判断当前阶段
- 决定下一步应切到哪个子 Skill
- 用统一话术把“当前状态 + 下一步动作”说明白

它不负责：

- 统计摘要
- 回归 / 聚类 / 分类
- 显性关联 / 追加 / 筛选
- 经营决策拍板

## 会话状态摘要协议

V1 建议维护如下最小状态字段：

### 1. 会话主状态

- `session_id`
- `current_workbook`
- `current_sheet`
- `current_stage`
- `schema_status`
- `active_table_ref`
- `last_user_goal`
- `updated_at`

### 2. 当前关注字段

- `selected_columns`
- `last_preview_columns`

### 3. 建模上下文

- `features`
- `target`
- `positive_label`
- `cluster_count`
- `missing_strategy`
- `last_model_kind`

### 4. 决策上下文

- `last_decision_summary`
- `last_priority_actions`
- `last_next_tool_suggestions`

## 状态字段说明

### `current_stage`

只能取以下值：

- `table_processing`
- `analysis_modeling`
- `decision_assistant`
- `unknown`

### `schema_status`

只能取以下值：

- `unknown`
- `pending_confirmation`
- `confirmed`

### `active_table_ref`

- 如果已经通过 `apply_header_schema` 拿到确认态，则写入
- 后续分析建模层与决策助手层优先使用它
- 没有它时，才回退到 `path + sheet`

## 路由规则

### 规则 1：先看确认态

- 如果已有 `active_table_ref`
  - 优先走分析建模层或决策助手层
- 如果没有 `active_table_ref`
  - 默认先回表处理层

### 规则 2：再看用户意图

#### 进入表处理层

当用户说这些话时，优先路由到 `table-processing-v1`：

- “先看看表”
- “帮我整理一下”
- “这两张表怎么合”
- “先确认表头”
- “筛一下、汇总一下、追加一下、关联一下”

#### 进入分析建模层

当用户说这些话时，优先路由到 `analysis-modeling-v1`：

- “先看统计摘要”
- “适不适合建模”
- “做线性回归 / 逻辑回归 / 聚类”
- “看看这些字段能不能分析”

但如果此时没有 `active_table_ref`，应先回表处理层建立确认态。

#### 进入决策助手层

当用户说这些话时，优先路由到 `decision-assistant-v1`：

- “告诉我下一步该做什么”
- “现在最该先处理什么”
- “按优先级给我建议”
- “我下一步怎么办”

但如果此时没有 `active_table_ref`，也应先回表处理层建立确认态。

### 规则 3：优先复用 `table_ref`

跨层交接遵循固定协议：

- `table-processing-v1` 产出 `table_ref`
- `analysis-modeling-v1` 优先消费 `table_ref`
- `decision-assistant-v1` 优先消费 `table_ref`

不要在已经有 `table_ref` 时，再重复要求用户提供 `path + sheet` 并重新确认表头。

## 对外话术规范

总入口 Skill 每次回复尽量保持三段：

1. 当前理解
2. 当前状态摘要
3. 下一步动作

例如：

- 当前理解：你现在是想知道这张表下一步应该先清洗、先分析，还是已经可以继续建模。
- 当前状态：当前文件已经锁定在 `2026文旅体台账.xlsx`，正在处理 `旅责险`，表头已确认，已有可复用的 `table_ref`。
- 下一步动作：我会直接进入决策助手层，不再重复确认表头。

## 与三层 Skill 的关系

### `table-processing-v1`

- 负责建立确认态
- 负责把 `table_ref` 交接给总入口层
- 负责在 schema 未确认时挡住后续层

### `analysis-modeling-v1`

- 负责统计摘要、观察诊断、回归、聚类路由
- 负责在已有 `table_ref` 时直接进入分析建模
- 负责把关键建模上下文回写到会话状态摘要

### `decision-assistant-v1`

- 负责把诊断结果翻译成“下一步先做什么”
- 负责优先解释阻塞风险、优先动作和下一步 Tool
- 不负责直接启动模型执行

## 总入口 Skill V1 的文件结构建议

建议新增目录：

- `D:/Rust/Excel_Skill/skills/excel-orchestrator-v1/SKILL.md`
- `D:/Rust/Excel_Skill/skills/excel-orchestrator-v1/requests.md`
- `D:/Rust/Excel_Skill/skills/excel-orchestrator-v1/cases.md`
- `D:/Rust/Excel_Skill/skills/excel-orchestrator-v1/acceptance-dialogues.md`

其中：

- `SKILL.md`：写总入口规则、状态摘要规则、路由规则
- `requests.md`：写跨层交接模板，不直接重复子 Skill 的细节模板
- `cases.md`：写用户一句话应该进入哪层
- `acceptance-dialogues.md`：写总入口级人工验收对话稿

## 风险与控制

### 风险 1：总入口 Skill 变成大杂烩

控制方式：

- 只保留路由、状态摘要、统一话术
- 不复制三层 Skill 的细节规则

### 风险 2：状态字段太多，Skill 变脆

控制方式：

- V1 只保留最小状态字段
- 先不要引入复杂结果血缘和自动恢复逻辑

### 风险 3：和后续本地记忆层重复

控制方式：

- 先在 Skill 中只定义状态协议，不把它当最终存储
- 真实持久化交给后续 `local-memory-runtime-v1`

## 推荐实施顺序

1. 先落地 `excel-orchestrator-v1` Skill 文档与验收稿
2. 再落地 `local-memory-runtime-v1` 的本地独立记忆层
3. 最后让总入口 Skill 改成“读取独立记忆层”而不是依赖临时上下文

## 最终判断

`excel-orchestrator-v1` 值得做，而且应该作为统一入口层存在。

但它必须是：

- 总入口路由层
- 会话状态摘要维护层
- 非计算层

而不是把三层 Skill 粘成一个大 Skill。
