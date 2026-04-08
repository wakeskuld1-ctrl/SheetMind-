# Security Post Meeting Package Binding Design

<!-- 2026-04-08 CST: 新增 Task 11 设计文档，原因是当前会后结论最小 Green 已落地，但 package/object_graph/verify 还没有正式承接它；目的：先冻结“正式挂接会后结论”的边界和数据流，避免后续继续靠临时字段串联。 -->

## 目标

把 `SecurityPostMeetingConclusion` 从“独立可落盘对象”推进为“正式进入 package 合同的治理对象”。

本轮只做以下三件事：

1. 让 revision 后的 `decision_package` 正式携带 `post_meeting_conclusion`
2. 让 package `artifact_manifest` 与 `object_graph` 都能稳定引用该对象
3. 为 `security_decision_verify_package` 补齐会后结论绑定与完整性校验

本轮不做：

- 投中执行对象
- 投后复盘对象
- GUI 展示
- 大模型增强
- 重新设计 foundation 主线

## 当前问题

当前分支已经完成：

- 正式对象 `SecurityPostMeetingConclusion`
- 独立 Tool `security_record_post_meeting_conclusion`
- Tool 能落盘会后结论
- Tool 能复用 `security_decision_package_revision` 生成新版本 package

但还存在三个结构性缺口：

1. revision 后的 package 还没有正式把 `post_meeting_conclusion` 带进 `object_graph`
2. revision 后的 `artifact_manifest` 还没有正式登记 `post_meeting_conclusion`
3. `security_decision_verify_package` 还不能验证：
   - `source_brief_ref`
   - `source_package_path`
   - `final_disposition`

这意味着当前链路仍然是：

`record conclusion -> write standalone file -> revise package`

而不是：

`record conclusion -> bind into package contract -> verify package/conclusion integrity`

## 方案比较

### 方案 A：只把会后结论挂入 artifact manifest

优点：

- 改动最小
- 先解决“文件被 package 看见”的问题

缺点：

- `object_graph` 仍然不知道它
- verify 很难做强绑定校验

### 方案 B：同时扩展 artifact manifest 和 object graph

优点：

- package 合同更完整
- verify 有正式锚点可用
- 后续 object graph / artifact manifest 都能统一消费

缺点：

- 需要同步修改 revision 和 verify

### 方案 C：直接把会后结论塞进 package 全文

优点：

- 一眼就能在 package 里看到全部内容

缺点：

- 破坏当前“package 引用工件而不是内嵌全文”的设计
- 会让后续 diff、哈希和审计成本变高

## 选型

采用方案 B。

原因：

- 它延续当前 package 的正式合同形态，不用回退成“大 JSON 装一切”
- `object_graph` 提供治理引用，`artifact_manifest` 提供工件登记，职责清晰
- verify 可以围绕正式锚点做绑定检查，而不是回落成路径猜测

## 合同扩展设计

### Package Object Graph

给 `SecurityDecisionPackageDocument.object_graph` 增加：

- `post_meeting_conclusion_ref`
- `post_meeting_conclusion_path`

规则：

- 初始 package 可以为空
- 一旦发生 `security_record_post_meeting_conclusion` 驱动的 revision，新 package 必须填写

### Artifact Manifest

新增 artifact role：

- `post_meeting_conclusion`

规则：

- 初始 package 不强制要求存在
- 会后 revision 后必须 `present = true`
- `path` 指向正式落盘 JSON
- `sha256` 按现有 JSON 哈希规则生成

### Post Meeting Conclusion Binding

继续沿用当前对象里的正式字段，不额外引入展示层字段：

- `source_package_path`
- `source_package_version`
- `source_brief_ref`
- `final_disposition`

这些值继续使用稳定合同值，不写中文翻译进落盘 JSON。

## 数据流设计

本轮数据流固定为：

1. `security_record_post_meeting_conclusion`
   - 回读旧 package
   - 回读 `approval_brief`
   - 生成并落盘 `post_meeting_conclusion`

2. `security_record_post_meeting_conclusion`
   - 调用 `security_decision_package_revision`
   - 透传新生成的会后结论路径

3. `security_decision_package_revision`
   - 重建 `artifact_manifest`
   - 追加或更新 `post_meeting_conclusion`
   - 写入新的 `object_graph.post_meeting_conclusion_ref/path`
   - 生成 v2 或更高版本 package

4. `security_decision_verify_package`
   - 读取 package object graph
   - 读取 post meeting conclusion
   - 校验 binding 和完整性

## Verify 扩展设计

本轮 verify 至少新增三项检查：

1. `post_meeting_conclusion_binding_consistent`
   - package `decision_ref / approval_ref`
   - 与会后结论 `governance_binding` 和根字段一致

2. `post_meeting_conclusion_brief_paired`
   - package `approval_brief_ref/path`
   - 与会后结论 `source_brief_ref`
   - 以及 `brief_pairing.pre_meeting_brief_ref/path` 一致

3. `post_meeting_conclusion_complete`
   - `final_disposition` 在允许枚举中
   - `source_package_path`
   - `source_brief_ref`
   - `decision_ref / approval_ref`
   - 这些关键字段不能为空

失败时原则：

- 返回 `package_valid = false`
- 推荐动作继续沿用现有隔离口径，例如 `quarantine_and_rebuild`

## 测试策略

本轮只围绕正式合同和 verify 做 TDD，不扩大战线。

### 第一组：revision 主链

在 `security_decision_package_revision_cli` 增加失败测试，断言：

- revision 后 package 的 `artifact_manifest` 包含 `post_meeting_conclusion`
- revision 后 package 的 `object_graph` 包含 `post_meeting_conclusion_ref/path`

### 第二组：record conclusion 主链

在 `security_post_meeting_conclusion_cli` 增加失败测试，断言：

- Tool 返回的新 package 中已经正式挂入会后结论
- 返回的 conclusion 与 package 引用一致

### 第三组：verify 主链

在 `security_decision_verify_package_cli` 增加失败测试，断言：

- happy path 下 binding 一致时通过
- 篡改 `source_brief_ref` 失败
- 篡改 `source_package_path` 失败
- 篡改 `final_disposition` 失败

## 风险控制

1. 不要把 `post_meeting_conclusion` 内嵌进 package 全文
2. 不要把中文展示字段写入正式 JSON
3. 不要在这轮顺手扩投中 / 投后对象，避免任务漂移
4. 不要修改原工作区脏文件，只在隔离 worktree 做本轮开发

## 完成标准

满足以下条件，才算 Task 11 完成：

- `security_record_post_meeting_conclusion` 生成的新 package 正式携带会后结论引用
- `artifact_manifest` 可见 `post_meeting_conclusion`
- `object_graph` 可见 `post_meeting_conclusion_ref/path`
- verify 能识别并校验会后结论绑定关系
- 相关 CLI 测试在合并结果上 fresh 通过
