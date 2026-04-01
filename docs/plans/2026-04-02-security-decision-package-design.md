# Security Decision Package Design

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 把当前分散落盘的证券审批工件收成正式 `decision package`，使其能够单独落盘、绑定治理上下文，并为后续验签与归档提供统一入口。

**Architecture:** 保持现有 `security_decision_submit_approval` 主入口不变，本次新增一个正式 `SecurityDecisionPackageDocument` 合同对象。该对象不重复承载原始审批业务字段，而是负责引用并绑定已生成的 `decision_card / approval_request / position_plan / approval_brief / approval_brief_signature` 等工件，形成一个 package-ready 的总清单。

**Tech Stack:** Rust、Cargo、serde/serde_json、sha2、现有证券审批桥接链、文件型 runtime 持久化、CLI 合同测试

---

## 1. 背景

当前主仓已经具备：

- `security_decision_submit_approval`
- `decision_card`
- `approval_request`
- `position_plan`
- 正式 `approval_brief`
- 可选 `approval_brief_signature`

这意味着审批链上的核心工件已经能分别落盘，但仍然存在一个明显缺口：

- 工件彼此分散，缺少正式的包级对象
- 调用方无法一次拿到完整工件清单与绑定关系
- 后续验签、归档、审批包导出仍然没有统一锚点

因此，当前状态更像“若干独立文件”，还不是“正式审批包”。

## 2. 设计目标

本次 `P0-4` 只解决以下问题：

1. 新增正式 `SecurityDecisionPackageDocument`
2. 把核心审批工件收成统一 `artifact_manifest`
3. 单独落盘到 `decision_packages/<decision_id>.json`
4. 绑定 `decision_ref / approval_ref / evidence_hash / governance_hash`
5. 在 `security_decision_submit_approval` 结果里返回 package 对象和路径

本次明确不做：

1. 不做 package 自身签名
2. 不做 package 验签命令
3. 不做 PDF / XLSX 审批包导出
4. 不接入投中/投后归档策略

## 3. 三种做法对比

### 方案 A：轻量 manifest 清单

优点：

- 开发最快
- 风险最低

缺点：

- 合同约束偏弱
- 后续仍需升级为正式文档

### 方案 B：正式 package 合同

优点：

- 与当前 formal approval brief 路线一致
- 可直接作为后续归档与验签锚点
- 更适合长期治理

缺点：

- 需要多设计一层对象与工件描述

### 方案 C：一步做到 package 签名

优点：

- 理论上最完整

缺点：

- 当前 scope 过大
- 会把 `P0-4` 做成半个下一阶段

本次采用：**方案 B**

## 4. 正式 package 对象

建议新增：

- `SecurityDecisionPackageDocument`

建议字段：

- `package_id`
- `contract_version`
- `created_at`
- `scene_name`
- `decision_id`
- `decision_ref`
- `approval_ref`
- `symbol`
- `analysis_date`
- `package_status`
- `artifact_manifest`
- `governance_binding`

说明：

- `package_status` 负责表达当前包是否可供审批流消费，例如 `review_bundle_ready` 或 `needs_follow_up`
- `artifact_manifest` 负责收口所有工件，不复制原始对象全部字段
- `governance_binding` 负责绑定证据和治理上下文

## 5. 嵌套对象设计

### 5.1 `artifact_manifest`

建议为数组，每一项包含：

- `artifact_role`
- `path`
- `sha256`
- `contract_version`
- `required`
- `present`

本次建议至少收这些角色：

- `decision_card`
- `approval_request`
- `position_plan`
- `approval_brief`
- `approval_brief_signature`（可选）
- `approval_events`
- `audit_log`

### 5.2 `governance_binding`

建议包含：

- `evidence_hash`
- `governance_hash`
- `decision_ref`
- `approval_ref`
- `package_scope`

这样 package 本身就能成为治理与审计的统一锚点。

## 6. 路径策略

新增目录：

- `decision_packages`

落盘文件：

- `decision_packages/<decision_id>.json`

## 7. 状态策略

建议规则：

- `decision status = ready_for_review` 且审批状态为 `Pending`
  - `package_status = review_bundle_ready`
- `decision status = blocked`
  - `package_status = needs_follow_up`
- 其他中间态
  - `package_status = pending_review_materials`

## 8. 输出扩展

`security_decision_submit_approval` 结果新增：

- `decision_package`
- `decision_package_path`

这样上层调用方可以一次拿到：

- 决策卡
- 审批请求
- 仓位计划
- 正式审批简报
- 审批包

## 9. 哈希策略

本次不直接对文件做二次读取哈希，而是优先对落盘 payload 做稳定 SHA256。

原因：

- 避免额外文件读取顺序带来的复杂度
- 当前已有对象 payload，可直接在提交阶段生成哈希
- 后续如果要做文件级校验，再新增 verify 流即可

## 10. 测试策略

必须坚持 TDD。

优先新增：

1. ready 场景会生成 `decision_package`
2. `decision_package_path` 会返回并成功落盘
3. `artifact_manifest` 至少包含：
   - `decision_card`
   - `approval_request`
   - `position_plan`
   - `approval_brief`
4. 提供签名时会包含：
   - `approval_brief_signature`
5. `decision_ref / approval_ref / evidence_hash / governance_hash` 在 package 内保持一致

## 11. 风险

### 11.1 manifest 过胖风险

如果 package 又重复存一整份原始对象，会和已有工件冗余。  
因此 package 只做绑定与清单，不复制完整正文。

### 11.2 路径与哈希漂移风险

如果路径和哈希来源不统一，后续验签或归档会出现不一致。  
因此本次在提交入口统一构造 artifact manifest。

### 11.3 包级状态语义过度设计风险

如果一开始做太多 package 状态，会和审批状态机重叠。  
因此本次只保留最小状态表达。

## 12. 完成定义

满足以下条件即可认为 `P0-4` 完成：

1. `SecurityDecisionPackageDocument` 合同落地
2. `decision_package` 单独落盘
3. `security_decision_submit_approval` 返回 package 对象与路径
4. `artifact_manifest` 成功收口核心工件
5. CLI 合同测试通过

## 13. 一句话总结

`P0-4` 的目标不是再多写一个 JSON，而是让证券审批链第一次拥有正式的“审批包”锚点，后续签名、验签、归档和导出都能围绕它扩展。
