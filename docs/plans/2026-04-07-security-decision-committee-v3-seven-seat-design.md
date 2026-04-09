# Security Decision Committee V3 Seven-Seat Design

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 将当前“bull/bear 双 Agent 投决会”升级为“6 名审议委员 + 1 名风控委员”的七席合议制投决会，同时复用现有 `security_decision_submit_approval` 主入口，不重复新增提交 Tool。
**Architecture:** 延续当前 Rust / CLI / Tool dispatcher / approval bridge 主线，不重做证券治理链。`security_decision_committee` 继续作为投决入口，对内升级为“统一证据包 -> 七席独立审议 -> 票数统计 -> 风控有限否决 -> 决议卡”的委员会编排；`security_decision_submit_approval` 继续复用该结果进入审批、签名、验签、package 与审计链。
**Tech Stack:** Rust、Cargo、现有 Tool dispatcher、approval bridge、decision package、Markdown 设计文档

---

## 1. 设计目标

本次 V3 的目标不是继续扩展双 Agent，而是把投决会升级成更接近“联邦大法官式合议庭”的制度模型：

- 固定 7 席
- 6 名审议委员 + 1 名风控委员
- 所有委员读取同一份完整证据包
- 所有委员都必须对完整案件给出判断，而不是只看单一因子
- 委员可以有稳定倾向与专业敏感度，但不能退化成“单因子裁判机”
- 每位委员都必须对 `buy / hold / reduce / avoid / abstain` 给出完整理由
- 最终决议由多数票形成，但风控委员拥有有限否决权

本次 V3 明确保留当前审批主链：

- 不重复新增“提交投决会”的 Tool
- 继续复用 `security_decision_submit_approval`
- 继续复用当前 approval bridge / approval brief / decision package / verify / revision 主线

## 2. 核心制度

### 2.1 七席结构

委员会固定为七席：

1. 席位 1：偏基本面稳健
2. 席位 2：偏趋势确认
3. 席位 3：偏事件与信息敏感
4. 席位 4：偏估值与赔率
5. 席位 5：偏宏观审慎
6. 席位 6：偏进攻与弹性
7. 席位 7：风控委员

这里的“偏”代表倾向与敏感度，不代表只能看这一类信息。所有委员都必须审阅完整证据并对全案作出完整判断。

### 2.2 市场轻微微调

七席是长期固定席位，但允许按市场做轻微参数微调，例如：

- `a_share_core`
- `a_share_bank`
- `etf_japan`
- `nikkei_future`

微调内容只能影响：

- 某类证据的优先级
- 风险阈值敏感度
- 改票条件的轻微偏移
- 置信区间上下限

微调不能改变：

- 席位身份
- 投票枚举
- 风控否决机制
- “所有委员审全案”的制度

### 2.3 投票与裁决

每位委员必须输出以下投票之一：

- `buy`
- `hold`
- `reduce`
- `avoid`
- `abstain`

每票必须附带：

- 完整理由
- 支持点
- 反对点
- 核心风险
- 什么条件下会改票
- 置信度

六名审议委员形成多数票结论。风控委员也独立投票，但其制度职责不同：

- 风控委员不负责代替多数票决定方向
- 风控委员只在明确风险条件触发时使用有限否决
- 有限否决只允许把结果降级成 `needs_more_evidence` 或 `blocked`
- 风控委员不能把多数 `avoid` 反向改成 `buy`

### 2.4 有限否决规则

建议首版有限否决触发条件：

- 关键证据缺失
- 分红口径不是分析日可得的最新正式方案
- 风险收益比低于委员会下限
- 重大事件风险未澄清
- 多席意见高度分裂且缺少足够证据压住分歧

有限否决输出：

- `none`
- `needs_more_evidence`
- `blocked`

## 3. 数据模型升级

### 3.1 新增委员意见合同

当前 `SecurityDecisionThesis` 无法承载七席合议信息，需要升级为新的委员意见对象。建议新增：

`SecurityCommitteeMemberOpinion`

核心字段：

- `member_id`
- `seat_name`
- `seat_kind`
- `market_tilt_profile`
- `vote`
- `confidence`
- `reasoning`
- `supporting_points`
- `counter_points`
- `key_risks`
- `what_changes_my_mind`
- `execution_mode`
- `evidence_hash`

### 3.2 升级委员会结果合同

当前 `SecurityDecisionCommitteeResult` 以 `bull_case` / `bear_case` 为核心，不足以表达七席制度。建议升级为：

- `committee_engine`
- `symbol`
- `analysis_date`
- `evidence_bundle`
- `committee_roster`
- `member_opinions`
- `vote_tally`
- `majority_opinion`
- `minority_opinions`
- `risk_veto`
- `risk_gates`
- `decision_card`

为兼容现有桥接链，V3 初期可以临时保留旧字段映射，直到审批对象和仓位计划全部切换完成。

### 3.3 决议卡升级

`SecurityDecisionCard` 建议新增：

- `decision_mode`
- `committee_vote_summary`
- `majority_vote`
- `minority_vote_count`
- `consensus_score`
- `risk_veto_status`
- `swing_factors`

这样审批侧和复盘侧才能看到：

- 最终方向是怎么形成的
- 是强共识还是脆弱多数
- 是否被风控降级

## 4. 运行架构

### 4.1 统一证据包

所有委员继续读取同一份 `security_decision_evidence_bundle`。这条规则必须保持不变，因为它是“同卷宗独立审议”的制度基础。

### 4.2 七席独立执行

当前 V2 已有子进程级双 Agent 经验，V3 应扩展为：

- `security_committee_member_registry`
- `security_committee_member_runner`
- `security_committee_vote_tally`
- `security_committee_majority_opinion`

运行方式：

1. `security_decision_committee` 先生成统一 `evidence_bundle`
2. 读取委员会席位注册表
3. 为 6 名审议委员 + 1 名风控委员分别生成独立请求
4. 每席独立运行，互不读取对方草稿
5. 收集 `member_opinions`
6. 统计票数并生成多数/少数意见
7. 应用风控有限否决
8. 生成最终 `decision_card`

### 4.3 不重复新增提交入口

`security_decision_submit_approval` 已经是当前“投决结果进入审批主线”的正式入口，本次不重复新增 `submit_committee_v3` 之类新 Tool。

V3 的原则是：

- 只升级 `security_decision_committee` 内部结构与结果合同
- 让 `security_decision_submit_approval` 继续消费新的 `SecurityDecisionCommitteeResult`
- 避免治理入口重复与后续分叉维护

## 5. 对现有治理链的影响

### 5.1 approval bridge

当前 bridge 直接消费：

- `committee.bull_case`
- `committee.bear_case`
- `committee.decision_card`

V3 后需要改成消费：

- `majority_opinion`
- `minority_opinions`
- `vote_tally`
- `risk_veto`
- `decision_card`

### 5.2 approval brief

当前 approval brief 主要展示 `bull_summary` / `bear_summary`。V3 应升级为：

- `majority_summary`
- `minority_summary`
- `member_vote_breakdown`
- `risk_veto_summary`
- `key_disagreements`

### 5.3 position plan

当前仓位计划主要由 `decision_card.status` 与 `confidence_score` 推导。V3 建议额外吸收：

- 多数票强度
- 委员分歧度
- 是否有风控否决
- 是否存在高权重反对理由

### 5.4 package / verify / revision

package、验签、revision 不需要新增入口，但需要确认：

- 新字段是否正确进入工件
- 新工件哈希是否稳定
- governance hash 是否覆盖关键委员会字段

## 6. 文件落点建议

重点修改：

- `src/ops/security_decision_committee.rs`
- `src/ops/security_decision_card.rs`
- `src/ops/security_decision_approval_bridge.rs`
- `src/ops/security_decision_approval_brief.rs`
- `src/ops/security_position_plan.rs`
- `src/ops/security_decision_submit_approval.rs`
- `src/tools/dispatcher/stock_ops.rs`

建议新增：

- `src/ops/security_committee_member_registry.rs`
- `src/ops/security_committee_member_runner.rs`
- `src/ops/security_committee_vote_tally.rs`
- `src/ops/security_committee_majority_opinion.rs`

测试重点：

- `tests/security_decision_committee_cli.rs`
- `tests/security_decision_submit_approval_cli.rs`
- `tests/security_decision_verify_package_cli.rs`
- `tests/security_decision_package_revision_cli.rs`

## 7. 兼容策略

V3 不应在第一步就把旧字段全部砍掉。建议分两阶段兼容：

### 阶段 1

- 增加 `member_opinions`、`vote_tally`、`majority_opinion`、`risk_veto`
- 临时保留 `bull_case` / `bear_case`
- 通过映射让现有 bridge 和 brief 继续工作

### 阶段 2

- bridge / brief / position plan 改为原生消费七席数据
- 清理只服务于双 Agent 的遗留字段

## 8. 测试策略

必须坚持 TDD。最关键的回归点：

1. 七席委员都会输出独立意见
2. 六名审议委员的票数统计正确
3. 风控委员可将多数结论降级
4. 每票都必须附理由
5. 最新分红正式方案仍是统一证据包的一部分
6. `security_decision_submit_approval` 不新增入口且仍可正常送审
7. approval brief / package / verify / revision 不回归

## 9. 非目标

本轮 V3 不做：

- 实盘下单
- 任意人数委员会
- 席位身份可自由替换
- 多套完全不同的人格模板热切换
- 重写现有审批体系

## 10. 完成定义

满足以下条件即可视为 V3 第一阶段完成：

1. `security_decision_committee` 已升级为七席委员会
2. 七席都基于统一证据包独立输出意见
3. 多数票与风控有限否决规则生效
4. 不重复新增提交 Tool，`security_decision_submit_approval` 继续可用
5. 关键治理链测试通过
