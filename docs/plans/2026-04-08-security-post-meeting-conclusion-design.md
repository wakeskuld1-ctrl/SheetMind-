# Security Post Meeting Conclusion Design

<!-- 2026-04-08 CST: 新增 Task 3 设计文档，原因是用户已批准按方案 C 一步到位推进“正式会后结论对象 + 会前/会后成对治理链 + 独立 Tool”；目的：在写代码前先冻结对象边界、数据流、语言分层和校验口径，避免后续继续把会后结论塞回临时 JSON。 -->

## 目标

把当前证券审批主链里的“会后结论”从隐含状态升级为正式治理对象，并形成以下闭环：

1. 生成可单独落盘的正式会后结论对象
2. 让会前 `approval_brief` 与会后结论形成正式配对关系
3. 提供独立 Tool 来生成或更新会后结论
4. 让 `decision_package / package_revision / verify_package` 能正式引用并回读该对象
5. 保持底层合同稳定存储，Skill 再做中文展示翻译

## 当前问题

当前主链已经有：

- `security_decision_committee`
- `security_decision_submit_approval`
- `security_decision_verify_package`
- `security_decision_package_revision`
- 正式 `approval_brief`
- 显式 `object_graph`
- `position_plan` 审批绑定

但 Task 3 还存在明显缺口：

1. 审批会后的最终采纳结论没有正式对象，只能从 `approval_request.status`、`approval_events` 或 `package_revision.trigger_event_summary` 间接推断
2. `approval_brief` 虽然已经正式化，但仍然偏“会前材料”，没有与会后采纳结论形成成对治理关系
3. 缺少一个独立命令来显式生成“会后结论对象”，导致 revision 仍更像包级版本动作，而不是会后治理动作
4. 当前用户面向的 Skill 输出没有明确区分“存储合同值”和“展示语言值”

## 方案比较

### 方案 A：只补正式会后结论对象

- 优点：
  - 改动最小
  - 可以较快形成会后对象
- 缺点：
  - 只能补“对象缺失”
  - `approval_brief` 与会后结论之间仍无正式关系
  - 后续投中 / 投后层还要再次补入口

### 方案 B：补对象 + 补前后配对关系

- 优点：
  - 会前 / 会后链路更完整
  - 后续 `execution_log / review_record` 更容易挂接
- 缺点：
  - 仍然没有独立命令入口
  - 会后更新仍要借 revision 或别的流程间接驱动

### 方案 C：补对象 + 补前后配对关系 + 独立 Tool

- 优点：
  - Task 3 一次闭环
  - 会后治理动作有正式入口
  - 后续投中 / 投后对象都可以围绕该对象继续扩展
- 缺点：
  - 本轮改动面最大
  - 测试需要同时覆盖对象、引用、Tool、revision 与展示层

## 选型

采用方案 C。

原因：

- 这是最贴近 Task 3 原目标的实现，不会把“会后结论对象化”拆成半截
- 当前已经有 `approval_brief / decision_package / package_revision` 三个治理锚点，正适合一次把“会前 -> 会后 -> 包修订”的链打通
- 后续 Task 4-9 都更需要一个正式“会后结论锚点”，而不是继续围绕 `approval_request.status` 做隐式推断

## 存储层与展示层边界

这是本轮必须冻结的硬边界：

### 存储层

正式对象和 package 合同继续使用稳定、可审计、与语言无关的存储值，例如：

- `approve`
- `reject`
- `needs_more_evidence`
- `approve_with_override`

这些值进入：

- 正式 JSON 落盘
- package manifest / object graph
- verify / revision 校验逻辑
- 审计与后续回放

### 展示层

Skill 或 CLI 面向用户输出时，再把存储值翻译为用户语言。

本轮先支持中文展示映射，但接口设计按可扩展多语言预留。

示例：

- `approve` -> `通过`
- `reject` -> `驳回`
- `needs_more_evidence` -> `需要补证据`
- `approve_with_override` -> `带保留通过`

规则：

1. 翻译文本不写回正式合同
2. Skill 可以输出“合同值 + 展示值”，但底层只校验合同值
3. 后续增加英文或其他语言时，只扩展示示映射层

## 正式对象设计

建议新增模块：

- `src/ops/security_post_meeting_conclusion.rs`

建议新增正式对象：

- `SecurityPostMeetingConclusion`

建议核心字段如下：

- `conclusion_id`
- `contract_version`
- `document_type`
- `generated_at`
- `scene_name`
- `decision_id`
- `decision_ref`
- `approval_ref`
- `symbol`
- `analysis_date`
- `source_package_path`
- `source_package_version`
- `source_brief_ref`
- `committee_engine`
- `final_disposition`
- `disposition_reason`
- `adopted_thesis_summary`
- `rejected_thesis_summary`
- `key_reasons`
- `required_follow_ups`
- `dissent_summary`
- `risk_constraints`
- `reviewer_notes`
- `revision_reason`
- `governance_binding`
- `brief_pairing`

其中两块必须单独建结构：

### `governance_binding`

建议包含：

- `decision_ref`
- `approval_ref`
- `decision_id`
- `source_package_path`
- `source_package_version`
- `binding_status`

目的：

- 明确这份会后结论到底绑定的是哪一个 package 版本
- 让 verify 或后续审计能清楚知道它是不是当前 revision 链的一部分

### `brief_pairing`

建议包含：

- `pre_meeting_brief_ref`
- `pre_meeting_brief_path`
- `pairing_status`
- `pairing_summary`

目的：

- 明确会前简报与会后结论是一对正式治理对象
- 后续 revision 时可以清楚回读“结论是基于哪一版 brief 得出的”

## Tool 设计

建议新增独立 Tool：

- `security_record_post_meeting_conclusion`

请求建议字段：

- `package_path`
- `final_disposition`
- `disposition_reason`
- `key_reasons`
- `required_follow_ups`
- `reviewer_notes`
- `reviewer`
- `reviewer_role`
- `revision_reason`
- `reverify_after_revision`
- `approval_brief_signing_key_secret`
- `approval_brief_signing_key_secret_env`

响应建议字段：

- `post_meeting_conclusion`
- `post_meeting_conclusion_path`
- `decision_package`
- `decision_package_path`
- `package_version`
- `previous_package_path`
- `revision_reason`
- `verification_report_path`

这个 Tool 的职责是：

1. 回读当前 package
2. 回读 `approval_brief`
3. 构建正式 `post_meeting_conclusion`
4. 把会后结论落盘
5. 驱动一次 package revision
6. 在 revision 后把新对象挂进 package
7. 可选重新 verify

它不是替代 `security_decision_package_revision`，而是“会后治理语义入口”。

## 数据流设计

本轮建议数据流如下：

1. `security_decision_submit_approval`
   - 继续生成会前 `approval_brief`
   - package 仍然先不带会后结论

2. `security_record_post_meeting_conclusion`
   - 读取旧 package
   - 读取 `approval_brief`
   - 生成 `post_meeting_conclusion`
   - 落盘到 `post_meeting_conclusions/<decision_id>.json`
   - 调用 revision 生成新 package

3. `security_decision_package_revision`
   - 从旧 package 继承已有对象图
   - 增加对 `post_meeting_conclusion` 的 object graph / artifact manifest 支持
   - 让 v2 或更高版本 package 能稳定引用结论对象

4. `security_decision_verify_package`
   - 回读并校验 `post_meeting_conclusion`
   - 校验它与 `approval_brief`、`package.object_graph`、`approval_ref` 是否一致

## Package 与 Object Graph 扩展

建议对 `SecurityDecisionPackageDocument.object_graph` 增加：

- `post_meeting_conclusion_ref`
- `post_meeting_conclusion_path`

建议对 artifact manifest 增加新角色：

- `post_meeting_conclusion`

规则：

- 初始提交态可以缺失该工件
- 进入会后结论 revision 后，该工件应为正式存在
- verify 需要区分“初始态允许为空”和“会后结论 revision 后必须存在”

## Approval Brief 的成对关系扩展

`approval_brief` 不需要变成会后对象，但建议补一块轻量配对字段，例如：

- `post_meeting_pairing`

建议字段：

- `has_post_meeting_conclusion`
- `latest_conclusion_ref`
- `pairing_status`

这样可以做到：

- `approval_brief` 仍然代表会前材料
- 但它可以被正式标记为“已经配对到了哪一版会后结论”

## 校验设计

本轮 verify 建议新增至少三类校验：

1. `post_meeting_conclusion_binding_consistent`
   - 会后结论的 `decision_ref / approval_ref / source_package_path`
   - 是否和 package 一致

2. `post_meeting_conclusion_brief_paired`
   - 会后结论的 `source_brief_ref`
   - 是否和 package / brief 一致

3. `post_meeting_conclusion_complete`
   - 必填字段是否齐全
   - `final_disposition` 是否属于允许枚举

## Skill 翻译层设计

本轮不把翻译写进正式对象，只增加映射层。

建议新增一个轻量翻译帮助函数或模块，例如：

- `src/ops/security_conclusion_i18n.rs`

职责：

- 把 `final_disposition` 之类的合同值翻译成中文展示值
- 给 Skill 或 CLI 返回用户可读文案

建议先覆盖：

- `final_disposition`
- `pairing_status`
- `binding_status`
- 常见 `required_follow_ups`

## 测试策略

本轮至少需要覆盖以下 6 类行为：

1. Tool catalog 能发现 `security_record_post_meeting_conclusion`
2. 会后结论对象能独立落盘，且字段完整
3. 会后结论创建后会驱动 revision 生成新 package
4. revision 后 package 能正式引用 `post_meeting_conclusion`
5. verify 能在结论对象存在且绑定正确时通过
6. 篡改 `source_brief_ref`、`source_package_path` 或 `final_disposition` 时，verify 必须失败

## 风险

1. 如果把“会后结论 Tool”做得过重，容易和通用 revision 逻辑重复
2. 如果把翻译文本写进正式对象，会污染审计和版本比较
3. 如果 package 对“是否必须存在 post_meeting_conclusion”的状态边界定义不清，verify 会出现误报

## 边界控制

本轮明确不做：

- 不做投中 `execution_strategy`
- 不做 `execution_log`
- 不做 `review_record`
- 不做多语言完整系统，只做中文展示映射预留
- 不把会后结论扩展成新的审批入口系统，只做单一 Tool
