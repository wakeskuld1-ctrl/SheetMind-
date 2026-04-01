# Security Decision Package Revision Design

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为证券审批包增加正式版本化能力，使 `decision package` 不再只代表“初始提交态”，而能随着审批动作生成后续版本。

**Architecture:** 保持现有 `security_decision_submit_approval -> decision_package -> verify_package` 主线不变，本次新增 `security_decision_package_revision` Tool。该 Tool 负责读取现有 package、重读更新后的 `approval_request / approval_events / audit_log` 等工件、生成新的 package 版本，并可选重跑 verification report。为了让 lineage 清晰，本次会为 `SecurityDecisionPackageDocument` 增加最小版本字段与触发来源信息。

**Tech Stack:** Rust、Cargo、serde/serde_json、sha2、文件型 runtime 持久化、CLI 合同测试

---

## 1. 背景

当前主仓已经具备：

- `security_decision_submit_approval`
- `security_decision_package`
- `security_decision_verify_package`

这意味着系统已经能：

- 生成初始审批包
- 校验当前审批包是否可信

但还缺一层非常关键的时间维度：

- package 只代表“初始提交时刻”
- 如果后续 `approval_request` 状态变化、`approval_events` 追加、`audit_log` 增长
- 当前 package 不会产生新版本

所以现状是：

**有审批包，但没有审批包版本史。**

## 2. 设计目标

本次 `P0-6` 只解决以下问题：

1. 为 `SecurityDecisionPackageDocument` 增加正式版本字段
2. 新增 `security_decision_package_revision` Tool
3. 读取已有 package 并基于更新后的审批工件生成新 package 版本
4. 把版本 lineage 写进新 package
5. 可选重跑 verification report

本次明确不做：

1. 不实现完整人工审批 Tool
2. 不把私有 worktree 审批动作直接搬进主仓
3. 不自动监听文件变化生成版本
4. 不做投中持仓版本链

## 3. 三种做法对比

### 方案 A：复制式版本化

优点：

- 开发最快
- 风险最低

缺点：

- lineage 表达偏弱
- 很难说明“哪次动作触发了哪个版本”

### 方案 B：正式 package revision

优点：

- 版本号、父版本、触发来源都清楚
- 最适合后续审批史、投中史、投后归档继续扩展
- 和当前正式 package 合同路线一致

缺点：

- 比复制式版本化多一层字段与构造逻辑

### 方案 C：直接做审批动作入口并自动版本化

优点：

- 一步更完整

缺点：

- 范围过大
- 会把本轮拖进人工审批系统

本次采用：**方案 B**

## 4. 合同扩展

建议为 `SecurityDecisionPackageDocument` 新增以下字段：

- `package_version`
- `previous_package_path`
- `revision_reason`
- `trigger_event_summary`

说明：

- `package_version`
  - 初始包为 `1`
  - 每次版本化递增
- `previous_package_path`
  - 指向上一个 package 文件
- `revision_reason`
  - 例如 `approval_event_applied`
  - `approval_status_updated`
  - `manual_refresh`
- `trigger_event_summary`
  - 对最近一次审批动作做简短摘要

## 5. 新增 Tool

建议新增：

- `security_decision_package_revision`

请求字段建议：

- `package_path`
- `revision_reason`
- `reverify_after_revision`
- `approval_brief_signing_key_secret`
- `approval_brief_signing_key_secret_env`

说明：

- `package_path` 必填
- `revision_reason` 可选，默认 `approval_state_transition`
- `reverify_after_revision` 默认 `true`

## 6. 版本化行为

建议流程：

1. 读取旧 package
2. 从旧 package 的 manifest 中定位：
   - `decision_card`
   - `approval_request`
   - `approval_events`
   - `position_plan`
   - `approval_brief`
   - `approval_brief_signature`
   - `audit_log`
3. 直接读取这些最新文件内容
4. 重新生成 artifact manifest 和治理绑定
5. 生成新 package：
   - `package_version = old + 1`
   - `previous_package_path = old package path`
   - `revision_reason = request or inferred`
   - `trigger_event_summary = latest approval event summary`
6. 落盘到新版本路径
7. 可选重跑 `verify_package`

## 7. 路径策略

为了让 lineage 清晰，本次建议使用目录式版本路径：

- `decision_packages/<decision_id>/v1.json`
- `decision_packages/<decision_id>/v2.json`

兼容策略：

- 初始 `submit_approval` 可以继续沿用当前 `decision_packages/<decision_id>.json`
- revision 后的新版本先采用：
  - `decision_packages/<decision_id>/v2.json`

后续如果要统一，再考虑把初始包也迁到目录式。

## 8. 状态策略

新 package 的 `package_status` 仍沿用现有 derive 逻辑，但依赖最新审批工件：

- `Pending + ready_for_review`
  - `review_bundle_ready`
- `Approved`
  - `approved_bundle_ready`
- `Rejected`
  - `rejected_bundle_ready`
- `NeedsMoreEvidence`
  - `needs_follow_up`

本次建议顺手扩展现有 `package_status` 推导逻辑，支持审批后的状态。

## 9. 触发摘要策略

`trigger_event_summary` 建议优先来自 `approval_events` 数组最后一项：

- `reviewer`
- `reviewer_role`
- `action`
- `timestamp`

例如：

- `risk_officer approve at 2026-04-02T18:00:00+08:00`
- `ic_chair approve_with_override at ...`

如果没有 event，则退回：

- `approval_request status changed to Approved`

## 10. 测试策略

必须坚持 TDD。

优先新增：

1. tool catalog 能发现 `security_decision_package_revision`
2. 基于 v1 package 和更新后的审批工件，能生成 `v2 package`
3. `v2 package` 会写入：
   - `package_version = 2`
   - `previous_package_path`
   - `revision_reason`
   - `trigger_event_summary`
4. `reverify_after_revision = true` 时返回新的 verification report
5. `v2 package` 的 manifest 会反映更新后的 `approval_request / approval_events / audit_log`

## 11. 风险

### 11.1 初始路径与版本路径并存风险

当前初始 package 是平铺路径，revision 包建议采用目录式版本路径。  
这是可控的，但调用方需要接受“v1 和 v2 路径风格不同”的短期过渡。

### 11.2 事件来源不完整风险

如果 `approval_events` 没有按预期追加，revision 仍可生成，但 `trigger_event_summary` 可能退化。  
因此本次要允许 event 缺失但要写清楚 fallback。

### 11.3 package 状态扩展风险

如果审批后状态语义扩展太快，会和私有审批主线过早耦合。  
因此本次只补最小必要状态，不做复杂状态机。

## 12. 完成定义

满足以下条件即可认为 `P0-6` 完成：

1. `SecurityDecisionPackageDocument` 具备版本字段
2. `security_decision_package_revision` Tool 可调用
3. 可基于更新后的审批工件生成 v2 package
4. 可选重跑 verification report
5. CLI 合同测试通过

## 13. 一句话总结

`P0-6` 的目标不是复制一个新 JSON，而是让 `decision package` 开始拥有正式的版本史，能跟着审批动作演进。
