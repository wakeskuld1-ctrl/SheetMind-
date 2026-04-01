# Security Decision Approval Bridge Design

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在当前主仓证券投决会链路之上补齐“提交到私有审批主线”的桥接层，让证券投决结果可以正式进入私有 worktree 的审批存储、多签、审计链，而不是停留在分析结论。

**Architecture:** 保持主仓负责证券研究与证券投决，保持私有 worktree 负责审批治理与审计规则。本次不把私有 worktree 作为 Rust 依赖导入主仓，而是在主仓内新增桥接层，按私有 worktree 已有 JSON 合同写入兼容的 `DecisionCard`、`ApprovalRequest`、`ApprovalEvent` 与 `audit_log` 记录，使后续私有审批命令可以直接接续处理。

**Tech Stack:** Rust、Cargo、现有证券 Tool 分发链、JSON 文件持久化、CLI 合同测试

---

## 1. 背景

当前主仓已经具备：

- `security_decision_evidence_bundle`
- `security_decision_committee`
- `security_risk_gates`
- `security_decision_card`

也就是说，证券分析已经从“研究链”升级到了“投决会 v1”。  
但它仍然缺一条关键桥接：

- 还不能把投决结果正式落成审批对象
- 还不能直接进入私有 worktree 的审批请求、多签、审计链
- 还不能形成真正的 `decision_ref / approval_ref / audit_log`

因此，当前用户虽然能拿到结构化投决结论，但还没有真正进入投前治理流程。

## 2. 设计目标

本次 `P0-1` 只解决以下问题：

1. 把主仓证券投决会结果映射成私有 worktree 可消费的审批对象
2. 在主仓中直接生成并落盘兼容私有 worktree 的：
   - `DecisionCard`
   - `ApprovalRequest`
   - `ApprovalEvent`
   - `DecisionAuditRecord`
3. 让私有 worktree 后续已有的多签、风险官签核、审计链逻辑可直接接续使用
4. 给主仓提供一个正式的证券审批提交 Tool

本次明确不做：

1. 不把私有 worktree 作为主仓的编译期依赖
2. 不重写私有审批状态机
3. 不在主仓复制一整套审批治理系统
4. 不在本次接入真实下单
5. 不在本次补投中、投后管理

## 3. 总体结构

本次新增的结构为：

### 3.1 主仓证券投决层

继续复用：

- `security_decision_evidence_bundle`
- `security_decision_committee`

职责：

- 生成证券证据
- 生成多头 / 空头观点
- 生成风险闸门与投决卡

### 3.2 证券审批桥接层

新增：

- `security_decision_approval_bridge`
- `security_decision_submit_approval`
- `security_decision_approval_brief`

职责：

- 把证券投决对象映射成私有审批对象
- 生成证券版审批摘要
- 把对象持久化到私有 worktree 运行时目录

### 3.3 私有 worktree 审批治理层

复用既有结构：

- `decisions/*.json`
- `approvals/*.json`
- `approval_events/*.json`
- `audit_log/*.jsonl`

后续继续由私有 worktree 的审批命令处理：

- 多签
- 风险官签核
- override
- 审计链延续

## 4. 为什么采用“兼容 JSON 桥接”而不是直接代码依赖

主仓与私有 worktree 之间存在清晰边界。

如果直接把私有 worktree 当成 Rust 依赖引入，会带来三类问题：

1. 破坏 public kernel / private doctrine 的边界
2. 增加主仓编译耦合
3. 让后续主仓发布和私有仓演进互相绑死

因此本次采用：

- 主仓负责生成兼容对象
- 私有审批主线负责消费这些对象

也就是：

**合同对齐，代码解耦。**

## 5. 新增 Tool 设计

### 5.1 `security_decision_approval_brief`

职责：

- 输入证券投决会结果
- 输出面向人工审批的摘要对象

应包含：

- `symbol`
- `analysis_date`
- `committee_status`
- `direction`
- `confidence_score`
- `bull_summary`
- `bear_summary`
- `gate_summary`
- `position_summary`
- `required_next_actions`
- `final_recommendation`

### 5.2 `security_decision_submit_approval`

职责：

- 在单次请求内运行证券投决会
- 映射生成审批对象
- 落盘到私有 worktree 运行时目录
- 生成初始审计记录

建议输入：

- 继承 `security_decision_committee` 的参数
- `scene_name`
- `approval_runtime_root`
- `min_approvals`
- `require_risk_signoff`
- `created_at`

建议输出：

- `decision_ref`
- `approval_ref`
- `decision_id`
- `status`
- `approval_brief`
- `decision_card_path`
- `approval_request_path`
- `approval_events_path`
- `audit_log_path`

## 6. 数据映射规则

### 6.1 证券投决卡 -> 私有 DecisionCard

- `symbol -> asset_id`
- `analysis_date -> timestamp anchor`
- `status`
  - `blocked -> Blocked`
  - `needs_more_evidence -> NeedsMoreEvidence`
  - `ready_for_review -> ReadyForReview`
- `direction`
  - `long -> Long`
  - `avoid -> NoTrade`
- `confidence_score -> confidence_score`
- `bull_case.thesis_points -> key_supporting_points`
- `bear_case.thesis_points + gate warning/fail -> key_risks`
- `bull_case.invalidation_conditions -> invalidation_conditions`
- `evidence_hash -> evidence_refs`

### 6.2 审批请求初始规则

- `approval_state = Pending`
- `required = true`
- 默认 `min_approvals = 2`
- 默认 `require_risk_signoff = true`
- `decision_ref`、`approval_ref` 由主仓生成
- `evidence_hash` 从证券证据包继承
- `governance_hash` 本版先使用稳定桥接摘要哈希

### 6.3 初始审批事件

首版不直接写入人工审批动作事件。  
`approval_events/<decision_id>.json` 初始写入空数组，等待私有审批命令后续追加正式事件。

### 6.4 初始审计记录

审计日志至少落一条 `decision_persisted` 兼容记录，锚定：

- `decision_id`
- `decision_ref`
- `approval_ref`
- `evidence_hash`
- `governance_hash`
- `decision_status`
- `approval_status`

## 7. 路径策略

默认写入私有 worktree 的运行时目录：

- `D:\Rust\Excel_Skill\.worktrees\SheetMind-Scenes-inspect\.sheetmind_scenes_runtime`

同时允许请求显式覆盖 `approval_runtime_root`，便于测试。

运行时目录结构保持与私有 worktree 一致：

- `decisions`
- `approvals`
- `approval_events`
- `audit_log`

## 8. 测试策略

本次必须坚持 TDD。

优先新增以下测试：

1. `security_decision_submit_approval` 能进入 tool catalog
2. 提交成功时能生成：
   - `decision_ref`
   - `approval_ref`
   - 兼容 DecisionCard 文件
   - 兼容 ApprovalRequest 文件
   - 空 ApprovalEvent 文件
   - 初始 audit 记录
3. `blocked` 状态映射正确
4. `ready_for_review` 状态映射正确
5. `approval brief` 包含多头、空头、风险闸门摘要

## 9. 风险

### 9.1 合同漂移风险

如果私有 worktree 后续修改审批对象结构，主仓桥接层可能失配。  
因此本次要尽量贴近当前字段，并用测试锁死。

### 9.2 审计兼容风险

如果审计记录字段不兼容，私有 verify / review 流可能读不进。  
因此初始审计记录必须严格遵守当前 JSON 结构。

### 9.3 边界风险

如果把审批判断重新写回主仓，会重新制造第二套治理逻辑。  
因此本次只负责“提交”，不负责“重写审批”。

## 10. 完成定义

满足以下条件即可认为 `P0-1` 完成：

1. 主仓新增证券审批桥接设计与实现计划文档
2. 新增证券审批提交 Tool
3. 证券投决结果可映射并落盘为私有 worktree 兼容对象
4. 生成初始审计记录
5. CLI 合同测试通过
6. 用户后续可以直接用私有审批主线继续执行多签 / 风险官签核 / override

## 11. 一句话总结

本次不是重写审批系统，而是让主仓证券投决会第一次正式“上会”，并进入私有审批治理主线。
