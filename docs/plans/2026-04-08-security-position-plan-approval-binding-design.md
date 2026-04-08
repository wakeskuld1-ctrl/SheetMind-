# Security Position Plan Approval Binding Design

<!-- 2026-04-08 CST: 新增 Task 2 设计文档，原因是用户已批准按方案 B 把仓位计划彻底挂入审批链；目的是在改代码前先把仓位计划的正式审批绑定、字段边界和校验口径冻结下来，避免后续执行层与复盘层继续建立在隐式关系上。 -->

## 目标

把 `security_position_plan` 从“提交审批时顺带落盘的分析附属文件”升级为“正式可审批对象”。

本轮完成后，仓位计划需要同时满足 3 件事：

1. 自身是正式合同对象，而不是只有业务字段的裸 JSON。
2. `approval_request` 能明确引用它，而不是只能通过 package manifest 间接找到。
3. `verify_package` 能验证它是否完整、是否和决议方向一致、是否和审批对象绑定一致。

## 当前问题

当前已有能力：

- `position_plan` 已有 `plan_id / decision_ref / approval_ref`
- `submit_approval` 已经会落盘 `position_plan`
- `package.object_graph` 已经会记录 `position_plan_ref / position_plan_path`
- `verify_package` 已经会检查 `package.object_graph <-> position_plan` 是否一致

但缺口仍然很明显：

1. `approval_request` 本身并没有正式声明“本次审批审的是哪一个仓位计划”。
2. `position_plan` 缺少合同头和正式审批绑定块，看起来更像中间结果而不是正式审批对象。
3. 当前 `position_plan` 结构里缺少 `reduce_rule`，与路线图中“position_size / add_rule / reduce_rule / stop_rule / take_profit_rule”并不完全一致。
4. `verify_package` 还没有把“仓位计划是否齐备、是否和决议方向一致”纳入正式校验。

## 方案比较

### 方案 A：只给 approval_request 增加 position_plan_ref

- 优点：
  - 改动最小
  - 风险最低
- 缺点：
  - 只能证明“审批对象知道 plan_id”
  - 不能证明 plan 内容、路径、版本和摘要是否被正式绑定
  - 后续 revision / execution / review 仍然要回到 package 里找细节

### 方案 B：正式审批绑定块

- 做法：
  - 给 `SecurityPositionPlan` 增加合同头与审批绑定块
  - 给 `PersistedApprovalRequest` 增加 `position_plan_binding`
  - 在 `verify_package` 中校验 `approval_request <-> position_plan <-> package.object_graph`
- 优点：
  - 仓位计划正式进入审批链
  - 后续最容易接 Task 3/4/6/7
  - 审批对象层和 package 层职责都更清晰
- 缺点：
  - 需要同时修改 bridge / submit / verify / tests

### 方案 C：把仓位计划拆成独立审批子单

- 优点：
  - 最完整
- 缺点：
  - 超出 Task 2 范围
  - 会把当前主链复杂化

## 选型

采用方案 B。

原因：

- Task 1 刚完成 package 对象图冻结，Task 2 最自然的推进就是让 `approval_request` 也正式绑定仓位计划。
- 这样做既能保持当前单主链，又能把仓位计划从“文件存在”升级成“审批合同中被确认的对象”。

## 设计稿

### 1. 给 position_plan 增加正式合同头

在 `SecurityPositionPlan` 中新增：

- `contract_version`
- `document_type`
- `decision_id`
- `plan_direction`

说明：

- `decision_id` 便于审批链和 revision 链直接定位
- `plan_direction` 用于后续校验与 `decision_card.direction` 是否一致
- 保留现有 `plan_id / decision_ref / approval_ref`

### 2. 给 position_plan 增加正式审批绑定块

新增 `approval_binding`，建议包含：

- `decision_ref`
- `approval_ref`
- `approval_request_ref`
- `package_scope`
- `binding_status`

说明：

- `approval_request_ref` 当前可与 `approval_ref` 复用同一值，但从命名上明确这是“审批链绑定”
- `package_scope` 统一写明来自 `security_decision_submit_approval`

### 3. 给 approval_request 增加 position_plan_binding

在 `PersistedApprovalRequest` 中新增 `position_plan_binding`，建议包含：

- `position_plan_ref`
- `position_plan_path`
- `position_plan_contract_version`
- `position_plan_sha256`
- `plan_status`
- `plan_direction`
- `gross_limit_summary`

说明：

- 这块不是 plan 的全文快照，而是审批对象对 plan 的正式引用与摘要
- `gross_limit_summary` 用于后续审批查看和最小校验，不需要把完整规则重复拷贝进去

### 4. 补齐 reduce_rule

给 `SecurityPositionPlan` 新增 `reduce_plan`，使结构与路线图对齐：

- `allow_reduce`
- `trigger_condition`
- `target_gross_pct`
- `notes`

这样当前正式规则集合就完整覆盖：

- `position_size`
- `add_rule`
- `reduce_rule`
- `stop_rule`
- `take_profit_rule`

### 5. verify_package 增加 3 类新校验

新增正式校验项：

1. `position_plan_binding_consistent`
   - `approval_request.position_plan_binding` 是否与真实 `position_plan` 一致
   - 是否与 `package.object_graph.position_plan_ref / position_plan_path` 一致

2. `position_plan_complete`
   - 必填字段是否齐备
   - `contract_version / document_type / plan_direction / approval_binding / reduce_plan` 是否存在

3. `position_plan_direction_aligned`
   - `decision_card.direction` 与 `position_plan.plan_direction` 是否一致
   - `NoTrade / blocked` 等特殊方向也要有明确口径

## 测试策略

先锁 4 类行为：

1. `submit_approval` 生成的 `approval_request` 必须包含 `position_plan_binding`
2. 持久化的 `position_plan` 必须包含正式合同头和 `approval_binding`
3. `verify_package` 在完整绑定时通过
4. 篡改 `approval_request.position_plan_binding` 或篡改 `position_plan.plan_direction` 时，`verify_package` 必须失败

## 风险

- `position_plan` 结构变化后，现有依赖它的 `approval_brief` 摘要生成逻辑需要保持兼容
- revision 流程更新 approval_request 后，新的 binding hash 也必须跟着变化
- 老测试 fixture 中如果直接读取 `position_plan` 字段，需要同步补断言或兼容处理
