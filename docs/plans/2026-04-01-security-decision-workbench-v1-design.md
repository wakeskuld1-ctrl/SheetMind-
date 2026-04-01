# Security Decision Workbench V1 Design

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在当前证券分析主链之上补齐“证券投决会”最小闭环，让系统能够先冻结同源证据，再让正反两方独立博弈，最后通过确定性的风险闸门与投决卡给出结构化决策结论，而不是直接输出单边股评建议。

**Architecture:** 延续当前 Rust / EXE / SQLite / Skill 主线，不重开第二套证券分析架构。`technical_consultation_basic -> security_analysis_contextual -> security_analysis_fullstack` 继续只负责研究证据；新增 `security_decision_*` 适配层把研究输出整理成投决证据包，再由双立场 Skill 独立消费，并通过确定性风险闸门和投决卡收口。决策合同尽量向私有 `Decision Layer` 靠拢，但首版实现只在主仓内落最小闭环。

**Tech Stack:** Rust、Markdown、Cargo、现有 Skill 体系、现有 CLI JSON 工具分发

---

## 1. 背景与问题

当前主仓已经具备可交付的证券研究链：

- `technical_consultation_basic`
- `security_analysis_contextual`
- `security_analysis_fullstack`

这条链已经能输出技术面、环境面、财报与公告面的综合结论，但它本质上仍然是“研究员链”，不是“投决会链”。

当前缺口主要有四个：

1. 没有统一的证券投决证据包，导致后续任何正反方判断都可能各读各的字段。
2. 没有真正独立的正方/反方立场运行单元，推荐结论容易退化成单边建议。
3. 没有证券场景专用的风险闸门，导致“分析成立”和“可进入决策”被混为一谈。
4. 没有标准化证券投决卡，无法沉淀成后续审批、复核、复盘可复用的对象。

因此，当前最正确的补法不是继续往 `security_analysis_fullstack` 里塞更多结论，而是把它上推为“研究证据层”，再在其上新增证券投决适配层。

## 2. 设计目标

V1 只解决以下问题：

1. 把当前证券研究输出冻结成单一 `evidence_bundle`
2. 在同一轮对话中支持正反双方基于同源证据独立判断
3. 用确定性风险闸门把“可分析”与“可决策”区分开
4. 输出结构化证券投决卡
5. 给顶层 Skill 一个稳定的最终答复入口

V1 明确不做：

1. 不接真实交易执行
2. 不引入复杂多轮记忆型会议系统
3. 不直接并入私有 `SheetMind-Scenes` 全量审批/签名/审计链
4. 不把信息面重新灌回 `technical_consultation_basic`
5. 不为了一次产品落地重构现有证券分析主链

## 3. 总体结构

V1 采用四层结构：

### 3.1 研究证据层

复用当前已有主链：

- `technical_consultation_basic`
- `security_analysis_contextual`
- `security_analysis_fullstack`

职责：

- 只负责采集和计算研究证据
- 只负责给出研究结论与结构化字段
- 不直接做最终投决通过/否决判断

### 3.2 投决适配层

新增 Rust 模块：

- `security_decision_evidence_bundle`
- `security_risk_gates`
- `security_decision_card`

职责：

- 统一整理研究证据
- 提供可审计的结构化输入
- 提供确定性风控闸门
- 生成结构化投决卡

### 3.3 双立场博弈层

新增 Skill：

- `security-decision-workbench-v1`
- `security-bull-thesis-v1`
- `security-bear-challenge-v1`

职责：

- 顶层 Skill 负责协调流程
- 多头 Skill 只做支持论证
- 空头 Skill 只做反方挑战
- 双方都只读同一份 `evidence_bundle`

### 3.4 决策输出层

输出统一的 `security_decision_card`，并由顶层 Skill 把结果翻译成用户可执行结论，结论必须区分：

- 研究层事实
- 正方观点
- 反方观点
- 风险闸门结果
- 最终投决状态
- 可执行建议或待补证据项

## 4. 同一对话内实现“双立场独立”的方法

V1 不依赖“同一上下文里左右互写”的伪双角色模式，而采用“单对话、多阶段、证据冻结”的方式。

### 4.1 阶段 1：证据冻结

顶层 Skill 先调用 `security_decision_evidence_bundle`，生成固定的 `evidence_bundle`。

证据包必须携带：

- `analysis_date`
- `symbol`
- `technical_context`
- `contextual_alignment`
- `fullstack_context`
- `data_availability`
- `risk_notes`
- `evidence_hash`

这样后续所有立场都必须基于同一份证据工作，避免一边看技术面、一边自行补信息面。

### 4.2 阶段 2：双边独立初判

多头与空头两侧在逻辑上必须满足：

1. 只读取同一份 `evidence_bundle`
2. 不读取彼此初稿
3. 不共享可变记忆
4. 初轮输出优先用结构化字段，而不是长篇自由文本

V1 虽然仍在单对话内编排，但要通过“先证据冻结、后独立子任务”的协议来保持独立性。

### 4.3 阶段 3：交叉反驳

初判冻结后，双方各自只读取对方摘要，不读取全部推理过程，再生成一轮简短反驳：

- 多头回应反方的最强否决点
- 空头回应多头的最强乐观前提

这样可以最大限度减少单边输出的确认偏差。

### 4.4 阶段 4：闸门与裁决

顶层 Skill 不直接“拍板”，而是把结果送入：

- `security_risk_gates`
- `security_decision_card`

由确定性规则和统一合同给出：

- `ReadyForReview`
- `Blocked`
- `NeedsMoreEvidence`

V1 默认仍由系统输出最终建议，但语义上已经从“单一研究员结论”切换成“经过正反方和闸门后的投决结论”。

## 5. 关键新增模块

### 5.1 `security_decision_evidence_bundle`

建议文件：

- `src/ops/stock/security_decision_evidence_bundle.rs`

职责：

- 接受证券投决请求
- 内部调用当前研究主链
- 返回统一证据包

建议输入：

- `symbol`
- `market_symbol` / `market_profile`
- `sector_symbol` / `sector_profile`
- `as_of_date`
- `include_fullstack`

建议输出核心字段：

- `analysis_date`
- `symbol`
- `technical_context`
- `contextual_analysis`
- `fullstack_analysis`
- `risk_notes`
- `data_gaps`
- `evidence_hash`

### 5.2 `security_risk_gates`

建议文件：

- `src/ops/stock/security_risk_gates.rs`

V1 先落以下确定性闸门：

1. `analysis_date_gate`
2. `data_completeness_gate`
3. `market_alignment_gate`
4. `event_risk_gate`
5. `risk_reward_gate`

其中：

- `analysis_date_gate`：日期必须明确且符合当前交易日规则
- `data_completeness_gate`：关键研究字段不能为空
- `market_alignment_gate`：大盘/板块逆风时不能轻易输出高置信通过
- `event_risk_gate`：财报、公告、重大事件冲突时降级
- `risk_reward_gate`：建议收益/风险比不足时不得直接给执行建议

### 5.3 `security_decision_card`

建议文件：

- `src/ops/stock/security_decision_card.rs`

该合同向私有 `DecisionCard` 对齐，但先做证券版最小子集。

建议核心字段：

- `decision_id`
- `symbol`
- `analysis_date`
- `status`
- `direction`
- `confidence_score`
- `expected_return_range`
- `downside_risk`
- `position_size_suggestion`
- `bull_case`
- `bear_case`
- `gate_results`
- `required_next_actions`
- `final_recommendation`

### 5.4 顶层证券投决入口

建议新增一个对外 Tool：

- `security_decision_committee`

职责：

- 先生成证据包
- 再组织双立场输出
- 再跑风险闸门
- 最后生成投决卡

这样用户在一个请求里就能拿到“研究 + 博弈 + 闸门 + 结论”。

## 6. Skill 设计

### 6.1 `security-decision-workbench-v1`

职责：

- 判断是否进入投决会流程
- 驱动证据冻结
- 驱动双立场输出
- 驱动投决卡生成
- 对外翻译成面向用户的中文结论

### 6.2 `security-bull-thesis-v1`

职责：

- 只论证为什么可以做
- 必须引用证据包已有字段
- 必须列出失效条件
- 不负责仓位拍板

### 6.3 `security-bear-challenge-v1`

职责：

- 只论证为什么不该做
- 重点找证据缺口、风险错配、环境逆风、事件冲突
- 不负责最终否决动作，只提供挑战意见

## 7. 文件落点

V1 建议全部落在当前主仓证券域，不污染 foundation：

- `src/ops/stock/security_decision_evidence_bundle.rs`
- `src/ops/stock/security_risk_gates.rs`
- `src/ops/stock/security_decision_card.rs`
- `src/tools/dispatcher/stock_ops.rs`
- `src/tools/catalog.rs`
- `skills/security-decision-workbench-v1/SKILL.md`
- `skills/security-bull-thesis-v1/SKILL.md`
- `skills/security-bear-challenge-v1/SKILL.md`
- `tests/security_decision_evidence_bundle_cli.rs`
- `tests/security_decision_committee_cli.rs`

必要时在 `src/ops/stock/mod.rs` 与 `src/ops/mod.rs` 做兼容 re-export。

## 8. 风险与边界

### 8.1 最大风险

如果把辩论和结论都继续塞在 `security_analysis_fullstack` 里，会重新把研究链和投决链混成一层，后续扩展会再次混乱。

### 8.2 独立性风险

如果双立场只是同一上下文里顺序生成文本，会退化成“一个 AI 自问自答”。因此 V1 必须在合同上明确：

- 先证据冻结
- 再独立初判
- 再摘要反驳
- 再统一裁决

### 8.3 产品边界风险

V1 不接真实下单，不承诺真实执行审批，只完成“证券投决会最小闭环”。

## 9. 测试策略

V1 必须坚持 TDD，优先新增 CLI/合同测试：

1. `security_decision_evidence_bundle` 能返回稳定证据包
2. 证据包必须显式带 `analysis_date`
3. 数据缺口存在时必须显式降级
4. 风险闸门能把结果分成 `Pass / Warn / Fail`
5. `security_decision_card` 能收口正反观点与闸门结果
6. `security_decision_committee` 在同一请求里返回完整投决结果

## 10. V1 完成定义

满足以下条件即可认为 V1 完成：

1. 主仓新增证券投决会设计与实现计划文档
2. 新增 `security_decision_evidence_bundle`
3. 新增 `security_risk_gates`
4. 新增 `security_decision_card`
5. 新增顶层 `security_decision_committee` Tool
6. 新增 3 个 Skill 文档
7. 关键 CLI/合同测试通过
8. 用户问“是否可以买、为什么、反方怎么看、风险闸门是否通过、建议怎么配仓”时，系统不再直接输出单边推荐，而是输出结构化投决结论

## 11. 一句话总结

证券分析链回答“研究上怎么看”，证券投决会链回答“在正反双方博弈和风险闸门之后，是否应该形成决策建议”。V1 的目标不是替换当前研究主链，而是把它提升为投决会的证据底座。
