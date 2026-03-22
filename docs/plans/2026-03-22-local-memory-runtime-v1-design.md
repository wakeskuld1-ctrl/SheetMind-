# Local Memory Runtime V1 Design

## 背景

当前系统已经开始出现稳定的跨层状态需求：

- 当前工作簿是谁
- 当前 sheet 是谁
- 当前是否已经确认表头
- 当前 `table_ref` 是谁
- 当前处于表处理 / 分析建模 / 决策助手哪一层
- 最近一次用户目标是什么
- 最近一次建模上下文是什么

这些状态不应该继续依赖：

- 大模型上下文
- Skill 文档
- 临时 JSON 工件

因此需要定义独立的本地记忆层：`local-memory-runtime-v1`。

## 设计目标

- 状态完全本地化，不依赖云端或外部服务
- 状态独立于 Skill 文档和大模型上下文
- 能被未来的单文件二进制直接读写
- 能支撑总入口 Skill 的会话状态摘要与路由判断
- 能支撑 `table_ref`、建模上下文、决策摘要等核心状态持久化

## 非目标

- 不做向量记忆
- 不做语义检索
- 不做长期知识库
- 不做复杂权限系统
- 不做跨设备同步

V1 只解决“当前机器上的本地运行时状态”。

## 方案比较

### 方案 A：纯 JSON 文件记忆层

#### 优点

- 开发快
- 直观
- 人工排查方便

#### 缺点

- 状态一多就乱
- 并发与一致性弱
- 查询复杂关系成本高

#### 判断

适合作为原型，不适合作为正式本地记忆层。

### 方案 B：SQLite 本地记忆层

#### 优点

- 完全本地
- 不需要额外部署服务
- 查询能力强
- 结构化状态非常适合会话、句柄、日志和上下文
- 和未来单文件二进制打包方向一致

#### 缺点

- 比 JSON 稍复杂
- 不如 JSON 直观可读

#### 判断

最适合作为 V1/V2 正式方案。

### 方案 C：SQLite + JSON 快照混合

#### 优点

- 主存储稳定
- 留痕和人工排查方便

#### 缺点

- 系统复杂度更高
- 需要定义哪些数据导出、哪些只存库

#### 判断

适合作为后续增强方向，不建议一开始就做满。

## 推荐方案

推荐先采用：

- 正式状态层：SQLite
- 留痕导出层：后续按需补 JSON 快照

也就是“先方案 B，再逐步演进到方案 C”。

## 推荐存储位置

Windows 默认建议放在：

- `%LOCALAPPDATA%/ExcelSkill/runtime.db`

如果后续需要导出可读工件，可额外放：

- `%LOCALAPPDATA%/ExcelSkill/runtime_exports/`

## 最小数据模型

### 1. `sessions`

保存会话主记录：

- `session_id`
- `created_at`
- `updated_at`
- `status`

### 2. `table_refs`

保存确认态表句柄：

- `table_ref`
- `source_path`
- `sheet_name`
- `columns_json`
- `header_row_count`
- `data_start_row_index`
- `source_fingerprint_json`
- `created_at`
- `last_used_at`

### 3. `session_state`

保存当前会话摘要：

- `session_id`
- `current_workbook`
- `current_sheet`
- `current_stage`
- `schema_status`
- `active_table_ref`
- `last_user_goal`
- `selected_columns_json`
- `updated_at`

### 4. `model_contexts`

保存建模上下文：

- `session_id`
- `features_json`
- `target`
- `positive_label`
- `cluster_count`
- `missing_strategy`
- `last_model_kind`
- `updated_at`

### 5. `event_logs`

保存运行时事件日志：

- `event_id`
- `session_id`
- `event_type`
- `stage`
- `tool_name`
- `status`
- `message`
- `created_at`

### 6. `user_preferences`

保存本地偏好：

- `user_key`
- `confirmation_style`
- `preferred_output_style`
- `default_top_k`
- `ask_before_modeling`
- `updated_at`

## 为什么这些表足够

V1 最关键的是三类问题：

1. 当前用户正处在哪个阶段
2. 当前表是否已有确认态和 `table_ref`
3. 当前是否已经积累了可复用的建模 / 决策上下文

上述 6 张表已经足够覆盖这三类问题。

## 与总入口 Skill 的关系

### `excel-orchestrator-v1`

未来应当：

- 读取 `session_state`
- 读取 `table_refs`
- 根据状态决定走哪层 Skill
- 在每轮结束后更新 `session_state` 和 `event_logs`

### `table-processing-v1`

未来应当：

- 在表头确认成功后，把 `table_ref` 写入 `table_refs`
- 更新 `session_state.active_table_ref`
- 追加 `schema_confirmed` / `table_ref_activated` 事件

### `analysis-modeling-v1`

未来应当：

- 从 `session_state` 读取 `active_table_ref`
- 把 `features` / `target` / `positive_label` 等写入 `model_contexts`
- 追加 `analysis_started` / `analysis_completed` / `modeling_blocked` 事件

### `decision-assistant-v1`

未来应当：

- 从 `session_state` 与 `model_contexts` 读取当前上下文
- 把关键摘要写回 `session_state` 或后续扩展的结果摘要表
- 追加 `decision_assistant_completed` 事件

## 推荐事件类型

V1 建议固定这些事件类型：

- `workbook_opened`
- `schema_confirmation_requested`
- `schema_confirmed`
- `table_ref_activated`
- `analysis_started`
- `analysis_completed`
- `modeling_blocked`
- `modeling_completed`
- `decision_assistant_completed`

## 风险与控制

### 风险 1：状态层和 Skill 规则混在一起

控制方式：

- Skill 只定义“读什么 / 写什么”的协议
- 不把状态本体写回 Skill 文档

### 风险 2：一开始把记忆层做太重

控制方式：

- V1 只做结构化状态，不做语义记忆
- 不做结果全文存档，只存摘要和关键句柄

### 风险 3：当前已有 `table_ref` 文件存储与未来 SQLite 重复

控制方式：

- 先允许并存
- 后续通过迁移把 `table_ref` 主存储切到统一 runtime

## 推荐实施顺序

1. 先完成 `excel-orchestrator-v1`
2. 再实现 `local-memory-runtime-v1` 最小 SQLite 版本
3. 让 `table_ref` 和 `session_state` 先接入本地 runtime
4. 再把建模上下文和事件日志补齐

## 最终判断

本地独立记忆层必须做，而且应该尽早纳入主线。

推荐方向是：

- 本地 SQLite
- 独立于 Skill
- 独立于大模型上下文
- 作为未来二进制产品的正式运行时状态层
