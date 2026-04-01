# 多步链式闭环收口设计

> 时间：2026-03-23
> 范围：`table_ref / result_ref / workbook_ref / session state` 的统一闭环

## 背景

当前仓库已经具备：

- `table_ref`：确认态表结构的持久化句柄
- `result_ref`：中间结果集的持久化句柄
- `workbook_ref`：多 Sheet 导出草稿句柄
- `session_state`：供 orchestrator Skill 读取的本地状态摘要

但这四者之间还没有真正闭环：

1. 很多产生新结果的 Tool 已经返回 `result_ref`，但 `session_state.active_table_ref` 仍停留在输入句柄，而不是本轮新产出的句柄。
2. `compose_workbook` 会返回 `workbook_ref`，但当前会话状态不会同步到这个新句柄。
3. `active_table_ref` 这个字段名已经承载了 `table_ref / result_ref / workbook_ref` 三种语义，只是对外兼容时通过 `active_handle_ref` 动态映射，底层语义仍不够清晰。
4. 上层 Skill 虽然可以继续调用下一步 Tool，但它读取到的“当前激活对象”不一定是最新结果，这会削弱连续规划、解释和留痕能力。

## 目标

本轮只做“基础链路收口”，不扩业务专题：

- 让每个会产生新句柄的 Tool，在返回 JSON 的同时，把最新激活句柄写回本地会话状态。
- 在不破坏现有 `active_table_ref` 兼容语义的前提下，新增更清晰的激活句柄字段。
- 让 `get_session_state` 能稳定暴露：
  - 当前激活句柄
  - 当前激活句柄类型
  - 当前工作簿 / Sheet / 文件句柄
  - 最近一次多步链条真正落到哪一个中间结果
- 通过 CLI 集成测试把“先追加再关联”“结果转导出草稿”“分析后继续决策”等典型链路锁死。

## 方案对比

### 方案 A：只修 `session_state` 的展示层

- 做法：不改底层存储，只在 `get_session_state` 返回时尽量从现有字段推断最新句柄。
- 优点：改动最小，风险最低。
- 缺点：治标不治本；真正的会话状态仍然落后于实际执行链，Tool 之间还是靠调用方自己记住最新句柄。

### 方案 B：统一“新产出句柄回写会话状态”（推荐）

- 做法：
  - 给 `session_state` 增加显式 `active_handle_ref / active_handle_kind`
  - 所有会产生新句柄的 Tool 在成功返回前统一回写最新激活句柄
  - `active_table_ref` 保留为兼容字段，并在底层与 `table_ref` 语义重新对齐
- 优点：
  - 真正补齐多步闭环
  - 上层 Skill 可直接读“最新结果”
  - 不需要重做现有 Tool 结构
- 缺点：
  - 需要补 SQLite 迁移、dispatcher 收口和较多集成测试

### 方案 C：直接引入“激活句柄栈 / 血缘图”

- 做法：在本地记忆层中维护完整步骤栈、父子依赖、回退指针。
- 优点：长期最强，适合未来复杂对话编排。
- 缺点：超出当前 V1/V1.5 收口范围，改动过重，验证面过大。

## 采用方案

采用方案 B。

原因：

- 它正好解决“Skill 能继续调 Tool，但并不知道当前真正最新结果是谁”的核心问题。
- 它不会把业务语义压进 Tool 层，只是在基础句柄和会话层补齐闭环。
- 它能与现有 `result_ref`、`workbook_ref`、`table_ref` 体系自然兼容，不需要大重构。

## 设计细节

### 1）会话状态语义收口

在 `LocalMemoryRuntime::SessionState` / `SessionStatePatch` 中新增：

- `active_handle_ref: Option<String>`
- `active_handle_kind: Option<String>`

兼容策略：

- `active_table_ref` 暂不删除，继续保留给旧 Skill / 旧测试。
- 当 `active_handle_kind == table_ref` 时，`active_table_ref` 与 `active_handle_ref` 保持一致。
- 当 `active_handle_kind != table_ref` 时，`active_table_ref` 继续保留最近可用的 `table_ref`，而 `active_handle_ref` 指向真正最新对象。

这样可以同时满足：

- 旧逻辑还能拿到可继续回源的确认态表句柄
- 新逻辑能拿到真正最新的链式中间句柄

### 2）统一产出句柄后的状态回写

新增统一辅助函数，负责：

- 接收 `tool_name`
- 接收最新生成的句柄 `table_ref / result_ref / workbook_ref`
- 推断句柄类型
- 回写 `session_state`
- 记录事件日志

首批接入对象：

- `apply_header_schema`
- `load_table_region`
- 所有通过 `respond_with_result_dataset` / `respond_with_preview_and_result_ref` 产出 `result_ref` 的 Tool
- `compose_workbook`

其中：

- 表处理 Tool 产出 `result_ref` 后，`active_handle_ref` 应切到新 `result_ref`
- 但 `active_table_ref` 仍保留最近确认态 `table_ref`
- `compose_workbook` 成功后，`active_handle_ref` 切到 `workbook_ref`

### 3）分析与决策层输入状态同步

当前 `sync_loaded_table_state` 是按输入句柄回写状态。

本轮将把它拆分为两类：

- `sync_input_handle_state`：只在“纯读取类 Tool”中记录当前输入对象
- `sync_output_handle_state`：在“产生新结果类 Tool”中记录最新输出对象

这样可以避免：

- `group_and_aggregate` 返回了新 `result_ref`，状态却还指向旧 `table_ref`
- `join_tables` / `append_tables` 产生了新表，但状态仍停留在左右输入任一侧

### 4）对上层返回的结构

`get_session_state` 统一返回：

- `active_table_ref`
- `active_handle_ref`
- `active_handle.kind`
- `active_handle.ref`
- `current_workbook`
- `current_sheet`
- `current_file_ref`
- `current_sheet_index`

如果当前句柄是：

- `table_ref`：三者一致
- `result_ref`：`active_handle_ref` 指向结果；`active_table_ref` 保留最近确认态表
- `workbook_ref`：`active_handle_ref` 指向导出草稿；`active_table_ref` 仍保留最近确认态表

### 5）边界与非目标

本轮不做：

- 记忆层的完整步骤栈
- 可回退历史句柄列表
- 血缘图可视化
- 业务专题 Tool
- GUI 层状态栏

## 测试策略

### 必测链路

1. `load_table_region -> get_session_state`
   - 预期：`active_table_ref` 和 `active_handle_ref` 都正确
2. `group_and_aggregate -> get_session_state`
   - 预期：`active_handle_ref` 切到新 `result_ref`，`active_table_ref` 保留确认态 `table_ref`
3. `append_tables / join_tables -> get_session_state`
   - 预期：多表结果产生后，会话状态指向最新 `result_ref`
4. `compose_workbook -> get_session_state`
   - 预期：`active_handle_ref` 切到 `workbook_ref`
5. `update_session_state` 兼容旧字段
   - 预期：旧 payload 不崩，旧字段仍可 round-trip

### 风险点

- SQLite 旧库迁移需要补新列，避免破坏已有测试库
- `active_table_ref` 与 `active_handle_ref` 的兼容关系必须稳定，否则旧 Skill 可能误解
- 统一辅助函数接入范围大，若漏掉某些 Tool，会出现“部分链路闭环、部分不闭环”的隐性问题

## 验收标准

- 连续调用表处理 Tool 后，`get_session_state` 总能指出“当前真正最新的句柄”
- `append_tables`、`join_tables`、`compose_workbook` 不再只返回结果，而是同步更新会话状态
- 旧测试和旧字段兼容
- 全量 `cargo test -v` 与 `cargo build --release -v` 通过
