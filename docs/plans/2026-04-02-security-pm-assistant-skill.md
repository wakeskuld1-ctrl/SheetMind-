# 证券 PM 助手 Skill 实施计划

## 实施目标

创建一个新的上层问答 Skill：

- `security-pm-assistant-v1`

让用户在自然语言里不必记住底层 Tool 名称，也能被正确路由到：

- 证券分析
- 证券投决会
- 审批提交
- package 校验
- package 修订

<!-- 2026-04-02 CST: 新增本实施计划，原因是用户已经批准“证券 PM 助手 Skill”方案；目的是把 Skill 落地步骤、覆盖范围和验证方式写成可执行清单，减少后续实现偏差。 -->

## 实施范围

本轮做：

1. 新增 `skills/security-pm-assistant-v1/SKILL.md`
2. 明确五类问答的路由规则
3. 明确上下文复用规则
4. 补充场景化验证清单

本轮不做：

1. 不新增 Rust Tool
2. 不修改审批合同
3. 不扩展投中 / 投后治理
4. 不引入自动状态持久化

## 步骤

### 步骤 1：创建 Skill 文档

内容包括：

- Overview
- When to Use
- Stage Routing
- Context Reuse Rules
- Output Rules
- Common Mistakes
- Quick Reference

### 步骤 2：写清五类路由规则

分别覆盖：

1. 研究分析
2. 投决会判断
3. 审批提交
4. package 校验
5. package 修订

### 步骤 3：补充验证样例

至少给出 5 条自然语言场景：

- 分析类
- 投决类
- 提交审批类
- 校验类
- 修订类

### 步骤 4：检查命名与风格一致性

要求：

- 命名与现有 `security-*` Skill 风格一致
- frontmatter 描述只写触发条件，不提前概括实现流程
- 中文说明面向后续 AI 与问答使用

## 验证方式

本轮以场景化验证为主：

1. 检查 Skill 是否明确区分研究 / 投决 / 治理
2. 检查是否优先复用 `decision_ref / approval_ref / package_path`
3. 检查是否避免把研究输出伪装成投决输出
4. 检查是否避免在已有 package 的情况下重复提交审批

## 完成标准

达到以下条件即可视为完成：

1. `security-pm-assistant-v1` Skill 已创建
2. 五类问答入口已覆盖
3. 上下文复用规则已写清
4. 场景化验证清单已齐备
5. 任务日志已追加
