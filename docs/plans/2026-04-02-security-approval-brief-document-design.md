# Security Approval Brief Document Design

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 把当前轻量 `approval_brief` 升级成正式审批简报对象，使其能够单独落盘、可选生成 detached signature，并以 package-ready 结构进入后续 decision package。

**Architecture:** 保持现有 `committee_result -> approval_request -> position_plan` 主线不变，本次将 `approval_brief` 从临时摘要升级成正式文档合同。文档对象独立于 `DecisionCard` 和 `ApprovalRequest`，由提交审批入口统一生成并落盘，同时可选生成签名工件，并暴露 package-ready 绑定信息。

**Tech Stack:** Rust、Cargo、serde/serde_json、HMAC-SHA256、现有证券审批桥接链、文件型 runtime 持久化、CLI 合同测试

---

## 1. 背景

当前主仓已经具备：

- `security_decision_committee`
- `security_decision_submit_approval`
- `position_plan`
- 轻量 `approval_brief` 摘要

也就是说，审批对象已经能回答：

- 这票能不能做
- 为什么能做 / 不能做
- 风险闸门如何
- 仓位计划怎么做

但当前 `approval_brief` 仍然只是一个“返回结果里的结构化摘要”，还不具备以下产品特性：

- 不能作为正式审批简报文档单独落盘
- 不能单独签名
- 不能作为 package-ready 对象稳定进入 decision package

因此，当前审批摘要仍然更像“接口返回的一部分”，而不是“正式审批文档”。

## 2. 设计目标

本次 `P0-3` 只解决以下问题：

1. 将 `approval_brief` 升级成正式文档合同
2. 独立落盘到 `approval_briefs/<decision_id>.json`
3. 支持可选 detached signature 文件输出
4. 提供 package-ready 绑定信息，供后续 decision package 收口

本次明确不做：

1. 不实现完整 decision package builder
2. 不实现 approval brief 的独立 verify 命令
3. 不改私有 worktree 的签名主线
4. 不做 PDF / XLSX 简报导出

## 3. 三种做法对比

### 方案 A：继续扩现有摘要 struct

优点：

- 开发最快
- 改动最少

缺点：

- 结构会越来越像“临时 JSON”
- 不利于签名与 package 集成

### 方案 B：升级为正式文档合同

优点：

- 正式对象边界清楚
- 最适合单独落盘和签名
- 最适合后续 package 集成

缺点：

- 需要增加合同设计与少量元数据字段

### 方案 C：直接升级 decision package

优点：

- 理论上最完整

缺点：

- 当前 scope 过大
- 容易把 `P0-3` 做成下一阶段

本次采用：**方案 B**

## 4. 正式文档对象

建议保留名称语义为 approval brief，但合同升级为正式文档：

- `SecurityApprovalBriefDocument`

建议字段：

- `brief_id`
- `contract_version`
- `document_type`
- `generated_at`
- `scene_name`
- `decision_id`
- `decision_ref`
- `approval_ref`
- `symbol`
- `analysis_date`
- `decision_status`
- `approval_status`
- `confidence_band`
- `executive_summary`
- `core_supporting_points`
- `core_risks`
- `gate_outcome_summary`
- `position_plan_summary`
- `recommended_review_action`
- `evidence_hash`
- `governance_hash`
- `package_binding`

## 5. 嵌套对象设计

### 5.1 `position_plan_summary`

建议包含：

- `position_plan_status`
- `risk_budget_summary`
- `entry_summary`
- `add_summary`
- `stop_loss_summary`
- `take_profit_summary`
- `cancel_summary`

### 5.2 `package_binding`

建议包含：

- `artifact_role`
- `brief_contract_version`
- `decision_ref`
- `approval_ref`
- `decision_id`

这样 approval brief 即使还没进入完整 package，也已经具备 package-ready 绑定信息。

## 6. 签名策略

本次采用 detached signature。

原因：

- 不污染正文合同
- 更接近私有 worktree 当前 decision package signing 风格
- 后续更容易接 verify 流

建议签名文件：

- `approval_briefs/<decision_id>.signature.json`

建议 envelope 字段：

- `signature_version`
- `algorithm`
- `key_id`
- `payload_sha256`
- `contract_version`
- `brief_id`
- `signature`

算法：

- `hmac_sha256`

签名行为：

- 默认不强制生成
- 当请求带签名参数时生成

## 7. 路径策略

新增目录：

- `approval_briefs`

落盘对象：

- 文档：`approval_briefs/<decision_id>.json`
- 签名：`approval_briefs/<decision_id>.signature.json`

## 8. 输出扩展

`security_decision_submit_approval` 结果新增：

- `approval_brief_path`
- `approval_brief_signature_path`

其中：

- 未签名时 `approval_brief_signature_path = null`

## 9. 推荐审批动作

文档中新增：

- `recommended_review_action`

建议规则：

- `blocked -> reject_or_request_more_evidence`
- `needs_more_evidence -> request_more_evidence`
- `ready_for_review -> approve_with_standard_review`

## 10. 测试策略

必须坚持 TDD。

优先新增：

1. ready 场景会写入正式 `approval_brief` 文档
2. 文档包含：
   - `brief_id`
   - `contract_version`
   - `decision_ref`
   - `approval_ref`
   - `package_binding`
3. 文档路径会出现在提交结果里
4. 提供签名参数时会生成 signature 文件
5. signature 文件包含：
   - `payload_sha256`
   - `contract_version`
   - `brief_id`

## 11. 风险

### 11.1 合同膨胀风险

如果把太多决策细节重新塞进 approval brief，会和 `DecisionCard` 重叠。  
因此文档要强调“审批阅读摘要”，而不是复制全部原始对象。

### 11.2 签名漂移风险

如果签名格式和私有 worktree 差得太远，后续整合会增加成本。  
因此本次尽量贴近 detached signature 风格。

### 11.3 package 假集成风险

如果只是口头说“可进入 package”，但没有绑定字段，后续仍然要重做。  
因此本次必须显式写 `package_binding`。

## 12. 完成定义

满足以下条件即可认为 `P0-3` 完成：

1. `approval_brief` 升级为正式文档合同
2. 文档单独落盘
3. 可选生成 detached signature
4. 暴露 package-ready 绑定字段
5. CLI 合同测试通过

## 13. 一句话总结

`P0-3` 的目标不是再加几句摘要，而是把审批摘要正式升级成“可落盘、可签名、可装入 package 的审批简报文档”。
