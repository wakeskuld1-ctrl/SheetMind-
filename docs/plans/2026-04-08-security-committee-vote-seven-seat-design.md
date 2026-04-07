# Security Committee Vote Seven-Seat Design

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在不回退 CLI 新主链的前提下，把现有 `security_committee_vote` 升级为“6 名审议委员 + 1 名风控委员”的七席委员会，并补齐子进程级独立执行证明。

**Architecture:** 保持 `security_decision_briefing -> security_committee_vote` 作为 CLI 分支唯一正式投决主链，不重新引入旧的 `security_decision_committee` 入口。委员会继续消费统一的 `CommitteePayload`，但内部从 5 席同步执行升级为 7 席子进程独立执行，再由多数制与风控席有限否决生成最终决议。

**Tech Stack:** Rust、Cargo test、现有 CLI EXE stdin/stdout ToolRequest/ToolResponse 调度链、Markdown 交接文档。

---

## 1. 设计背景

当前 CLI 分支已经形成新的证券投决主链：

- `security_decision_briefing` 负责把 fullstack、resonance、execution digest 装配成统一 `CommitteePayload`
- `security_committee_vote` 负责消费 `CommitteePayload` 并输出正式投决结果

因此，本轮不能把旧工作树中的 `security_decision_committee -> submit_approval -> package` 整套再搬回来，否则会在 CLI 分支形成双入口、双合同、双维护面。

本轮设计目标是：

- 保住 CLI 分支既有 `briefing / committee_vote` 主链
- 在 `security_committee_vote` 内部升级为七席委员会
- 明确回答“怎么证明这些席位是独立执行的”
- 不把用户已经在 CLI 分支做好的 `briefing-driven security committee flow` 打掉

## 2. 方案选择

### 方案 A：恢复旧 `security_decision_committee`

优点：

- 与旧工作树里的治理链最接近
- 旧的测试和合同思路更容易直接照搬

缺点：

- 会和 CLI 分支现有 `security_decision_briefing / security_committee_vote` 双轨并存
- 重复造正式投决入口
- 后续 GitHub 接手者更难判断哪条链才是主链

### 方案 B：在 `security_committee_vote` 上升级七席委员会

优点：

- 保留 CLI 分支现有主链
- 不重复造入口
- 能把“七席委员会 + 独立执行证明”沉淀到 CLI 分支现有合同中

缺点：

- 需要把旧工作树里的独立执行思路重新映射到 `CommitteePayload / CommitteeMemberVote / SecurityCommitteeVoteResult`
- 需要更新一轮 CLI 测试

### 方案 C：保留旧 5 席 vote，再包一层七席 wrapper

优点：

- 对现有代码侵入最小

缺点：

- 形成壳层叠壳层
- 解释链条会更混乱
- 很难说清正式投决结果到底由谁负责

**推荐：方案 B。**

## 3. 七席委员会映射

CLI 分支中的七席委员会固定为：

1. `chair`，偏整体结论与证据整合
2. `fundamental_reviewer`，偏基本面与分红口径
3. `technical_reviewer`，偏技术面与节奏确认
4. `event_reviewer`，偏信息面、公告与事件冲击
5. `valuation_reviewer`，偏赔率、风险收益比与性价比
6. `execution_reviewer`，偏执行阈值与仓位动作
7. `risk_officer`，风控席，拥有有限否决

这些席位都看同一份 `CommitteePayload`，不能退化成“只看某一类因子就给票”。席位差异体现在倾向性、敏感度、风险阈值和解释侧重点上，而不是体现在信息孤岛。

## 4. 独立执行证明

仅输出 `execution_mode = child_process` 还不足以证明独立执行。

本轮正式证明标准为：

- 每席都消费同一份 `CommitteePayload`
- 每席都暴露相同的 `evidence_version`
- 每席都暴露 `execution_mode = child_process`
- 每席都暴露独立的 `process_id`
- 每席都暴露独立的 `execution_instance_id`

这意味着我们能证明：

- 输入事实包是同源的
- 每席不是在同一个内存对象上改写出来的
- 每席有自己的运行时实例

本轮先做到“子进程级独立证明”。更强的“单席故障注入不污染其他席位”留作下一轮增强。

## 5. 合同调整

### 5.1 `CommitteeMemberVote`

新增字段：

- `member_id`
- `seat_kind`
- `execution_mode`
- `execution_instance_id`
- `process_id`
- `evidence_version`

保留现有字段：

- `role`
- `vote`
- `confidence`
- `rationale`
- `focus_points`
- `blockers`
- `conditions`

### 5.2 `SecurityCommitteeVoteResult`

新增字段：

- `committee_engine`
- `deliberation_seat_count`
- `risk_seat_count`
- `majority_vote`
- `majority_count`
- `member_votes`

兼容策略：

- 继续保留 `votes`
- `votes` 与 `member_votes` 初期可映射同一批正式委员意见，避免 CLI/GitHub 既有调用立刻断裂

## 6. 执行路径

委员会主流程改为：

1. `security_decision_briefing` 生成统一 `CommitteePayload`
2. `security_committee_vote` 根据固定 roster 生成 7 个席位请求
3. 每席通过子进程调用内部成员 agent
4. 汇总 6 名审议委员多数票
5. 应用 `risk_officer` 有限否决
6. 生成最终 `SecurityCommitteeVoteResult`

## 7. 工具与入口策略

本轮不新增新的正式“提交投决会”入口。

允许新增一个内部成员执行工具，例如：

- `security_committee_member_agent`

它的角色仅用于：

- 子进程执行单席投票
- 给独立执行证明提供可运行入口

它不是用户面向的正式投决入口，正式入口仍然只有：

- `security_decision_briefing`
- `security_committee_vote`

## 8. 测试策略

坚持 TDD，至少覆盖：

1. 七席数量正确
2. 每席都有正式投票理由
3. 七席执行模式均为 `child_process`
4. 七席 `evidence_version` 相同
5. 七席 `process_id / execution_instance_id` 唯一
6. 风控席可以将多数决议降级
7. CLI 输出合同仍兼容已有字段

## 9. 非目标

本轮不做：

- 旧 `security_decision_committee` 全套治理链回灌 CLI 分支
- approval/package 主链迁回 CLI 分支
- 可自由替换人格模板的动态席位系统
- 容器级或沙箱级更强隔离

## 10. 完成定义

满足以下条件即可视为本轮完成：

1. `security_committee_vote` 已升级为七席委员会
2. 七席为子进程级独立执行
3. 输出里具备 `execution_mode / process_id / execution_instance_id / evidence_version`
4. 相关 CLI 测试通过
5. 文档、交接说明、任务日志已补齐
