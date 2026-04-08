# Security Decision Object Graph Design

<!-- 2026-04-08 CST: 新增 Task 1 设计文档，原因是用户已批准按方案 B 冻结正式决策对象图；目的是在开始改代码前先把对象边界、兼容策略和校验范围写清楚，避免后续实现再次发散。 -->

## 目标

把当前证券审批包里“靠 `artifact_manifest` 和文件路径间接表达”的对象关系，升级为显式、可校验、可扩展的正式对象图绑定块。

本轮只覆盖当前已经存在的核心对象：

- `decision_card`
- `approval_request`
- `position_plan`
- `approval_brief`

本轮不引入执行层和复盘层对象，但设计上要为后续 `execution_strategy / execution_log / review_record` 预留统一挂接位置。

## 当前问题

当前实现已经具备以下能力：

- `SecurityDecisionPackageDocument` 顶层已有 `decision_ref / approval_ref`
- `artifact_manifest` 已能记录 `decision_card / approval_request / position_plan / approval_brief` 的路径和 hash
- `SecurityPositionPlan` 与 `SecurityDecisionApprovalBrief` 内部都已经带有 `decision_ref / approval_ref`
- `SecurityDecisionApprovalBrief` 已经有稳定的 `brief_id`

但仍存在 3 个问题：

1. `position_plan` 和 `approval_brief` 还没有在 package 合同里成为显式对象引用，只是通过 artifact role 被间接找到。
2. `verify_package` 目前主要验证文件存在、hash、签名和基础治理绑定，还没有把“对象图内部引用是否一致”做成正式校验。
3. 后续要补执行层和复盘层时，如果继续沿用“只靠 manifest 推断对象关系”，对象边界会越来越散。

## 方案比较

### 方案 A：只补顶层字段

- 做法：
  - 在 `SecurityDecisionPackageDocument` 顶层增加 `position_plan_ref / approval_brief_ref`
- 优点：
  - 改动最小
  - 对当前 CLI 影响最小
- 缺点：
  - 对象图依然分散
  - 后续接更多对象时顶层字段会继续膨胀

### 方案 B：增加显式对象图绑定块

- 做法：
  - 在 `SecurityDecisionPackageDocument` 中增加统一的对象图块
  - 集中记录 `decision_ref / approval_ref / position_plan_ref / approval_brief_ref`
  - 同时记录与这些对象一一对应的落盘路径
- 优点：
  - 结构清晰，便于后续扩展
  - 可以在 `verify_package` 中做正式一致性校验
  - 不需要推翻现有顶层 `decision_ref / approval_ref`
- 缺点：
  - 需要同步修改 `submit_approval / verify_package / tests`

### 方案 C：直接引入图节点与关系边

- 做法：
  - package 内部记录节点列表与关系边列表
- 优点：
  - 结构最完整
- 缺点：
  - 明显超出 Task 1 最小范围
  - 会拖慢 Task 2/3

## 选型

采用方案 B。

理由：

- 它能把当前隐式关系提升为正式合同，又不会把 Task 1 升级成大规模重构。
- 它和现有 `decision_ref / approval_ref + artifact_manifest + governance_binding` 结构兼容最好。
- 后续新增对象时，只需要在同一个绑定块里扩展，而不需要继续在 package 顶层散着加字段。

## 设计稿

### 1. 新增 package 对象图绑定块

在 `SecurityDecisionPackageDocument` 中新增一个显式块，暂定命名为 `object_graph`。

建议包含两类字段：

- 稳定引用字段
  - `decision_ref`
  - `approval_ref`
  - `position_plan_ref`
  - `approval_brief_ref`
- 与引用一一对应的正式路径
  - `decision_card_path`
  - `approval_request_path`
  - `position_plan_path`
  - `approval_brief_path`

说明：

- `position_plan_ref` 直接复用 `SecurityPositionPlan.plan_id`
- `approval_brief_ref` 直接复用 `SecurityDecisionApprovalBrief.brief_id`
- 顶层原有 `decision_ref / approval_ref` 暂时保留，作为向后兼容层

### 2. package builder 负责一次性写入

在 `security_decision_submit_approval` 构建 package 时：

- 从 bridge 结果中直接取得：
  - `decision_ref`
  - `approval_ref`
  - `position_plan.plan_id`
  - `approval_brief.brief_id`
- 从落盘阶段直接取得：
  - `decision_path`
  - `approval_path`
  - `position_plan_path`
  - `approval_brief_path`

所有这些值只在 package builder 处收口一次，避免之后每个调用点各自拼装。

### 3. verify_package 增加对象图一致性校验

新增校验维度：

- package `object_graph.decision_ref` 是否与：
  - package 顶层 `decision_ref`
  - `decision_card.decision_ref`
  - `approval_request.decision_ref`
  - `approval_brief.decision_ref`
  一致
- package `object_graph.approval_ref` 是否与：
  - package 顶层 `approval_ref`
  - `approval_request.approval_ref`
  - `position_plan.approval_ref`
  - `approval_brief.approval_ref`
  一致
- `position_plan_ref` 是否等于 `position_plan.plan_id`
- `approval_brief_ref` 是否等于 `approval_brief.brief_id`
- `object_graph` 中记录的路径是否与 `artifact_manifest` 里相同角色的路径一致

### 4. 兼容策略

- 顶层 `decision_ref / approval_ref` 保留
- `object_graph` 作为新增必填结构，由新生成 package 一律写入
- 如果未来需要兼容旧 package，可在 `verify_package` 中区分：
  - 新 package：必须完整校验 `object_graph`
  - 旧 package：允许缺失 `object_graph`，但给出 warning

本轮优先保证新生成 package 的正式契约，不主动回填历史 fixture 文件。

## 测试策略

先按 TDD 锁定 4 类行为：

1. `submit_approval` 生成的 package 必须包含完整 `object_graph`
2. `verify_package` 对完整一致的 `object_graph` 给出通过结果
3. 若 `position_plan_ref` 与落盘 `position_plan.plan_id` 不一致，`verify_package` 必须失败
4. 若 `approval_brief_path` 与 manifest 中 `approval_brief` 角色路径不一致，`verify_package` 必须失败

## 风险

- `package_revision` 在重建 package 时也需要补上 `object_graph`，否则新旧 package 结构会漂移
- 若现有测试对 package JSON 结构做了严格断言，需要同步更新
- 终端显示有部分历史中文注释编码异常，本轮只做 UTF-8 追加，不主动清洗旧注释
