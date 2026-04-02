# Security Committee Vote Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 构建正式的 `security_committee_vote` Tool，让投决会只能基于统一 `committee_payload` 做结构化表决、聚合结论和保留异议，同时与现有 `security_decision_briefing`、研究层和 Skill 门禁保持单一事实口径。

**Architecture:** 推荐走“先扩 `committee_payload` 成投票就绪事实包，再实现确定性投票引擎，最后接研究增强与 Skill 门禁”的路线。`security_committee_vote` 只消费 `committee_payload`，不允许重新抓取 fullstack / resonance / research，也不允许上层 Agent 手工拼第二套事实；投票结果通过固定角色、固定规则、固定 veto / quorum / conditions 产出，后续再允许 Agent 在结果之上做解释，而不是在事实层自由发挥。

**Tech Stack:** Rust、serde、现有 CLI Tool dispatcher、SQLite runtime、项目内 Skill 目录、cargo test

---

## Approach Options

### 方案 A：基于当前精简 `committee_payload` 直接做轻量 vote Tool
- 优点：改动少，最快能出一个可调用的表决入口。
- 缺点：当前 payload 信息太薄，角色投票会退化成“对同一句摘要重复表态”，无法支撑真正的技术、风险、执行分工。
- 结论：不推荐，只适合临时 demo，不适合正式投决链路。

### 方案 B：先把 `committee_payload` 扩成投票就绪事实包，再做确定性 vote Tool
- 优点：最符合当前仓库“统一 briefing -> committee payload -> 上层 Agent 解释”的路线；同一份 payload 可以同时服务人工投决、自动 vote、后续 GUI。
- 缺点：前置工作比方案 A 多，需要先补 payload 结构和测试。
- 结论：**推荐方案**。这是本计划采用的实现路径。

### 方案 C：vote Tool 直接吃完整 `security_decision_briefing`
- 优点：信息最多，短期内最容易写复杂规则。
- 缺点：破坏“投决会只能在 `committee_payload` 上表决”的既定门禁，也容易让 vote Tool 重新依赖 briefing 内部细节，造成二次耦合。
- 结论：不推荐，除非用户明确放弃 `committee_payload` 作为投决统一入口。

---

## Core Design

### 1. Tool Boundary
- 新增正式 Tool：`security_committee_vote`
- 输入：`committee_payload` + `committee_mode` + 可选 `meeting_id`
- 输出：固定结构的投票结果，不重新抓取任何外部数据
- 约束：如果 payload 缺少关键字段、`evidence_version` 非法、或 schema 版本不兼容，直接报错，不做表决

### 2. Committee Roles
- `chair`
- `fundamental_reviewer`
- `technical_reviewer`
- `risk_officer`
- `execution_reviewer`

### 3. Vote Options
- `approve`
- `conditional_approve`
- `defer`
- `reject`

### 4. Committee Modes
- `standard`
  - 默认模式
  - 允许 `conditional_approve`
  - `risk_officer` 可触发 veto
- `strict`
  - 用于高金额、高争议或样本不足场景
  - 需要更高通过门槛
  - `risk_officer` 与 `execution_reviewer` 都可形成阻断
- `advisory`
  - 只输出委员会倾向
  - 不触发 veto
  - 更适合投前讨论或投后复盘

### 5. Final Decisions
- `approved`
- `approved_with_conditions`
- `deferred`
- `rejected`

### 6. Aggregation Rules
- `standard`
  - 无 veto 且 `approve + conditional_approve` 至少 3/5 时，进入通过或条件通过
  - 若 `approve` 至少 3 票且 `reject` 不超过 1 票，输出 `approved`
  - 若条件票较多或 evidence 边界明显，输出 `approved_with_conditions`
  - 若赞成不足但未明显否决，输出 `deferred`
  - 若 `risk_officer` 触发 veto，直接 `rejected`
- `strict`
  - 无 veto
  - 至少 4/5 支持，且不能有 `risk_officer` / `execution_reviewer` 的 `reject`
  - 否则至少 `deferred`
- `advisory`
  - 不做 veto
  - 按多数票输出倾向，并保留全部反对意见

### 7. Backward Compatibility
- 保留当前 `committee_payload` 已有字段：
  - `symbol`
  - `analysis_date`
  - `recommended_action`
  - `confidence`
  - `key_risks`
  - `minority_objection_points`
  - `evidence_version`
  - `briefing_digest`
- 在其基础上新增结构化投票块，不删除旧字段
- 这样 briefing 现有测试、Skill 文案和后续 GUI 可以平滑升级

---

### Task 1: 把 `committee_payload` 升级成投票就绪事实包

**Files:**
- Modify: `E:\TradingAgents\TradingAgents\src\ops\security_decision_briefing.rs`
- Test: `E:\TradingAgents\TradingAgents\tests\security_analysis_resonance_cli.rs`

**Step 1: Write the failing test**

在 `tests/security_analysis_resonance_cli.rs` 新增红测：
- 断言 `committee_payload` 保留现有旧字段
- 同时新增以下结构化块：
  - `recommendation_digest`
  - `execution_digest`
  - `resonance_digest`
  - `evidence_checks`
  - `historical_digest`
  - `committee_schema_version`

其中最少断言这些子字段存在：
- `recommendation_digest.final_stance`
- `recommendation_digest.action_bias`
- `execution_digest.add_trigger_price`
- `execution_digest.stop_loss_price`
- `resonance_digest.resonance_score`
- `resonance_digest.top_positive_driver_names`
- `evidence_checks.fundamental_ready`
- `evidence_checks.technical_ready`
- `historical_digest.status`

**Step 2: Run test to verify it fails**

Run: `cargo test --test security_analysis_resonance_cli security_decision_briefing_exposes_vote_ready_committee_payload -- --nocapture`

Expected: FAIL with missing committee payload fields

**Step 3: Write minimal implementation**

在 `src/ops/security_decision_briefing.rs` 中：
- 保留当前 `CommitteePayload`
- 新增嵌套结构：
  - `CommitteeRecommendationDigest`
  - `CommitteeExecutionDigest`
  - `CommitteeResonanceDigest`
  - `CommitteeEvidenceChecks`
  - `CommitteeHistoricalDigest`
- 初版 `historical_digest` 可先用占位状态：
  - `status = "unavailable"`
  - `research_limitations = ["历史研究层尚未接入 committee payload"]`
- `recommendation_digest` 从现有 `integrated_conclusion`、`action_bias`、`consultation_conclusion` 映射
- `execution_digest` 直接复用 `ExecutionPlan`
- `resonance_digest` 从 `top_positive_resonances / top_negative_resonances` 生成短摘要
- `evidence_checks` 由 `fundamental_context.status`、`technical_context`、`resonance_context` 推导

**Step 4: Run test to verify it passes**

Run: `cargo test --test security_analysis_resonance_cli security_decision_briefing_exposes_vote_ready_committee_payload -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/ops/security_decision_briefing.rs tests/security_analysis_resonance_cli.rs
git commit -m "feat: enrich committee payload for vote engine"
```

### Task 2: 为 `security_committee_vote` 定义正式合同并接入 Tool 目录

**Files:**
- Create: `E:\TradingAgents\TradingAgents\src\ops\security_committee_vote.rs`
- Modify: `E:\TradingAgents\TradingAgents\src\ops\stock.rs`
- Modify: `E:\TradingAgents\TradingAgents\src\ops\mod.rs`
- Modify: `E:\TradingAgents\TradingAgents\src\tools\catalog.rs`
- Modify: `E:\TradingAgents\TradingAgents\src\tools\dispatcher.rs`
- Modify: `E:\TradingAgents\TradingAgents\src\tools\dispatcher\stock_ops.rs`
- Test: `E:\TradingAgents\TradingAgents\tests\integration_tool_contract.rs`

**Step 1: Write the failing test**

在 `tests/integration_tool_contract.rs` 新增红测，断言：
- `tool_catalog` 包含 `security_committee_vote`
- `stock` tool group 包含 `security_committee_vote`
- 该 Tool 的返回结构至少会暴露：
  - `symbol`
  - `analysis_date`
  - `evidence_version`
  - `committee_mode`
  - `final_decision`
  - `final_action`
  - `final_confidence`
  - `approval_ratio`
  - `veto_triggered`
  - `votes`
  - `conditions`
  - `key_disagreements`
  - `warnings`

**Step 2: Run test to verify it fails**

Run: `cargo test --test integration_tool_contract security_committee_vote_is_cataloged -- --nocapture`

Expected: FAIL with missing tool

**Step 3: Write minimal implementation**

在 `src/ops/security_committee_vote.rs` 定义：

```rust
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityCommitteeVoteRequest {
    pub committee_payload: CommitteePayload,
    #[serde(default = "default_committee_mode")]
    pub committee_mode: String,
    #[serde(default)]
    pub meeting_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SecurityCommitteeVoteResult {
    pub symbol: String,
    pub analysis_date: String,
    pub evidence_version: String,
    pub committee_mode: String,
    pub final_decision: String,
    pub final_action: String,
    pub final_confidence: String,
    pub approval_ratio: f64,
    pub quorum_met: bool,
    pub veto_triggered: bool,
    pub veto_role: Option<String>,
    pub votes: Vec<CommitteeMemberVote>,
    pub conditions: Vec<String>,
    pub key_disagreements: Vec<String>,
    pub warnings: Vec<String>,
    pub meeting_digest: String,
}
```

并完成 `catalog / dispatcher / stock re-export` 的接线，但先允许实现返回占位结构。

**Step 4: Run test to verify it passes**

Run: `cargo test --test integration_tool_contract security_committee_vote_is_cataloged -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/ops/security_committee_vote.rs src/ops/stock.rs src/ops/mod.rs src/tools/catalog.rs src/tools/dispatcher.rs src/tools/dispatcher/stock_ops.rs tests/integration_tool_contract.rs
git commit -m "feat: add security committee vote contract"
```

### Task 3: 为 vote Tool 增加 payload 校验与门禁

**Files:**
- Modify: `E:\TradingAgents\TradingAgents\src\ops\security_committee_vote.rs`
- Test: `E:\TradingAgents\TradingAgents\tests\security_committee_vote_cli.rs`

**Step 1: Write the failing test**

新建 `tests/security_committee_vote_cli.rs`，先写红测覆盖：
- `committee_payload.evidence_version` 为空时直接报错
- `committee_schema_version` 不支持时直接报错
- `committee_mode` 非 `standard/strict/advisory` 时报错
- `historical_digest.status = unavailable` 时，vote 可以继续，但结果必须带 warning
- `key_risks` 为空且 `briefing_digest` 为空时，视为 payload 不完整并报错

**Step 2: Run test to verify it fails**

Run: `cargo test --test security_committee_vote_cli security_committee_vote_rejects_invalid_payload -- --nocapture`

Expected: FAIL

**Step 3: Write minimal implementation**

在 `src/ops/security_committee_vote.rs` 中实现：
- `validate_committee_payload()`
- `parse_committee_mode()`
- `build_vote_warnings()`

错误类型建议至少包含：
- `UnsupportedCommitteeMode`
- `MissingEvidenceVersion`
- `UnsupportedCommitteeSchemaVersion`
- `IncompleteCommitteePayload`

**Step 4: Run test to verify it passes**

Run: `cargo test --test security_committee_vote_cli security_committee_vote_rejects_invalid_payload -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/ops/security_committee_vote.rs tests/security_committee_vote_cli.rs
git commit -m "feat: validate committee vote payloads"
```

### Task 4: 实现固定角色投票器

**Files:**
- Modify: `E:\TradingAgents\TradingAgents\src\ops\security_committee_vote.rs`
- Test: `E:\TradingAgents\TradingAgents\tests\security_committee_vote_cli.rs`

**Step 1: Write the failing test**

新增红测，构造一个风偏较正、证据完整的 payload，断言会产出 5 个固定角色投票：
- `chair`
- `fundamental_reviewer`
- `technical_reviewer`
- `risk_officer`
- `execution_reviewer`

并断言每个 vote 至少有：
- `role`
- `vote`
- `confidence`
- `rationale`
- `focus_points`
- `blockers`
- `conditions`

**Step 2: Run test to verify it fails**

Run: `cargo test --test security_committee_vote_cli security_committee_vote_emits_fixed_member_votes -- --nocapture`

Expected: FAIL

**Step 3: Write minimal implementation**

在 `src/ops/security_committee_vote.rs` 实现：
- `build_chair_vote()`
- `build_fundamental_vote()`
- `build_technical_vote()`
- `build_risk_vote()`
- `build_execution_vote()`

角色建议使用如下规则：
- `chair`
  - 看 `recommendation_digest.final_stance`、`confidence`、`evidence_checks`
- `fundamental_reviewer`
  - 看 `evidence_checks.fundamental_ready`、`key_risks`、`historical_digest`
- `technical_reviewer`
  - 看 `recommendation_digest.action_bias`、`execution_digest.watch_points`
- `risk_officer`
  - 看 `key_risks`、`minority_objection_points`、`historical_digest.research_limitations`
- `execution_reviewer`
  - 看 `execution_digest` 是否完整、止损/失效位是否明确、conditions 是否足够

**Step 4: Run test to verify it passes**

Run: `cargo test --test security_committee_vote_cli security_committee_vote_emits_fixed_member_votes -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/ops/security_committee_vote.rs tests/security_committee_vote_cli.rs
git commit -m "feat: add deterministic committee member votes"
```

### Task 5: 实现委员会聚合、quorum 与 veto 规则

**Files:**
- Modify: `E:\TradingAgents\TradingAgents\src\ops\security_committee_vote.rs`
- Test: `E:\TradingAgents\TradingAgents\tests\security_committee_vote_cli.rs`

**Step 1: Write the failing test**

新增三组红测：
- `standard` 模式下，多数支持且无 veto -> `approved`
- `standard` 模式下，`risk_officer = reject` -> `rejected` 且 `veto_triggered = true`
- `strict` 模式下，支持票不足 4/5 -> `deferred`

并断言：
- `approval_ratio`
- `quorum_met`
- `veto_triggered`
- `veto_role`
- `final_decision`
- `final_action`

**Step 2: Run test to verify it fails**

Run: `cargo test --test security_committee_vote_cli security_committee_vote_applies_aggregation_rules -- --nocapture`

Expected: FAIL

**Step 3: Write minimal implementation**

实现：
- `aggregate_votes_standard()`
- `aggregate_votes_strict()`
- `aggregate_votes_advisory()`
- `detect_veto_role()`
- `approval_ratio_from_votes()`
- `build_meeting_digest()`

建议的映射：
- `approve = 1.0`
- `conditional_approve = 0.6`
- `defer = 0.3`
- `reject = 0.0`

**Step 4: Run test to verify it passes**

Run: `cargo test --test security_committee_vote_cli security_committee_vote_applies_aggregation_rules -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/ops/security_committee_vote.rs tests/security_committee_vote_cli.rs
git commit -m "feat: add committee vote aggregation and veto rules"
```

### Task 6: 实现 conditions、分歧摘要和 warning 收口

**Files:**
- Modify: `E:\TradingAgents\TradingAgents\src\ops\security_committee_vote.rs`
- Test: `E:\TradingAgents\TradingAgents\tests\security_committee_vote_cli.rs`

**Step 1: Write the failing test**

新增红测，断言：
- 当存在 `conditional_approve` 时，结果必须聚合出 `conditions`
- 当存在正反两类投票理由时，结果必须产出 `key_disagreements`
- 当 `historical_digest.status = unavailable` 或 `research_limitations` 非空时，必须在 `warnings` 中保留

**Step 2: Run test to verify it fails**

Run: `cargo test --test security_committee_vote_cli security_committee_vote_surfaces_conditions_and_disagreements -- --nocapture`

Expected: FAIL

**Step 3: Write minimal implementation**

实现：
- `collect_committee_conditions()`
- `collect_key_disagreements()`
- `merge_vote_warnings()`

规则要求：
- 不去重就直接丢失信息不行，必须做去重但保留不同角色来源
- `key_disagreements` 优先保留：
  - 风险官的反对点
  - 技术/基本面与主席意见不一致的点
  - 研究层 limitations

**Step 4: Run test to verify it passes**

Run: `cargo test --test security_committee_vote_cli security_committee_vote_surfaces_conditions_and_disagreements -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/ops/security_committee_vote.rs tests/security_committee_vote_cli.rs
git commit -m "feat: surface committee conditions and disagreements"
```

### Task 7: 把研究层增强字段正式接进 vote 链路

**Files:**
- Modify: `E:\TradingAgents\TradingAgents\src\ops\security_decision_briefing.rs`
- Modify: `E:\TradingAgents\TradingAgents\src\ops\security_committee_vote.rs`
- Test: `E:\TradingAgents\TradingAgents\tests\security_analysis_resonance_cli.rs`
- Test: `E:\TradingAgents\TradingAgents\tests\security_committee_vote_cli.rs`

**Step 1: Write the failing test**

在 `tests/security_analysis_resonance_cli.rs` 新增红测，断言 `committee_payload` 增加：
- `historical_digest.historical_confidence`
- `historical_digest.analog_sample_count`
- `historical_digest.analog_win_rate_10d`
- `historical_digest.expected_return_window`
- `historical_digest.expected_drawdown_window`
- `historical_digest.research_limitations`

在 `tests/security_committee_vote_cli.rs` 新增红测，断言：
- 当 `historical_confidence = low` 或样本量不足时，`risk_officer` 更倾向 `defer / reject`
- 当 `analog_sample_count` 充足且 win rate 明显偏强时，可提升 `chair` / `fundamental_reviewer` 的信心等级

**Step 2: Run test to verify it fails**

Run:
- `cargo test --test security_analysis_resonance_cli committee_payload_includes_research_confidence_fields -- --nocapture`
- `cargo test --test security_committee_vote_cli security_committee_vote_uses_historical_confidence -- --nocapture`

Expected: FAIL

**Step 3: Write minimal implementation**

在 `security_decision_briefing` 中：
- 研究层已接上时，填充 `historical_digest`
- 未接上时，仍保持 `status = unavailable`

在 `security_committee_vote` 中：
- `fundamental_reviewer` 与 `chair` 可提升信心
- `risk_officer` 可把样本不足直接写成 `conditions` 或 `defer`
- `execution_reviewer` 可在历史回撤窗显著偏大时要求更严止损条件

**Step 4: Run test to verify it passes**

Run:
- `cargo test --test security_analysis_resonance_cli committee_payload_includes_research_confidence_fields -- --nocapture`
- `cargo test --test security_committee_vote_cli security_committee_vote_uses_historical_confidence -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/ops/security_decision_briefing.rs src/ops/security_committee_vote.rs tests/security_analysis_resonance_cli.rs tests/security_committee_vote_cli.rs
git commit -m "feat: connect research confidence to committee voting"
```

### Task 8: 新增投决会 Skill 门禁

**Files:**
- Create: `E:\TradingAgents\TradingAgents\skills\security-committee-v1\SKILL.md`
- Modify: `E:\TradingAgents\TradingAgents\skills\security-analysis-v1\SKILL.md`
- Modify: `E:\TradingAgents\TradingAgents\skills\security-decision-briefing-v1\SKILL.md`
- Test: manual skill review

**Step 1: Write the failing test**

先写手工验收标准：
- 证券咨询必须先走 `security_decision_briefing`
- 投决会必须先拿 `committee_payload`
- 需要表决时必须调用 `security_committee_vote`
- Skill 不允许绕过 Tool 手工编写投票结论
- Skill 不允许再拼第二份 committee 事实底稿

**Step 2: Run test to verify it fails**

Expected: 当前没有 `security-committee-v1`，且现有 Skill 只写到 `committee_payload`

**Step 3: Write minimal implementation**

新增 `skills/security-committee-v1/SKILL.md`，明确流程：
1. 调 `security_decision_briefing`
2. 读取 `committee_payload`
3. 调 `security_committee_vote`
4. 上层 Agent 只能解释 vote result，不得重写事实层

并更新现有 Skill：
- `security-analysis-v1`：强调投决模式必须经过 vote Tool
- `security-decision-briefing-v1`：强调 `committee_payload` 是投票前唯一合法事实入口

**Step 4: Run test to verify it passes**

Manual Review:
- 检查是否明确写出统一入口
- 检查是否禁止双口径
- 检查是否禁止绕过 vote Tool

**Step 5: Commit**

```bash
git add skills/security-analysis-v1/SKILL.md skills/security-decision-briefing-v1/SKILL.md skills/security-committee-v1/SKILL.md
git commit -m "feat: add security committee voting skill gate"
```

### Task 9: 回归验证、任务日志与边界文档

**Files:**
- Modify: `E:\TradingAgents\TradingAgents\CHANGELOG_TASK.MD`
- Optional Modify: `E:\TradingAgents\TradingAgents\docs\plans\2026-04-02-security-committee-vote-implementation.md`

**Step 1: Write the failing test**

无新增红测，本任务用于收口。

**Step 2: Run test to verify current state**

Run:
- `cargo test --test security_committee_vote_cli -- --nocapture`
- `cargo test --test security_analysis_resonance_cli -- --nocapture`
- `cargo test --test integration_tool_contract -- --nocapture`
- `cargo fmt`

Expected: all PASS

**Step 3: Write minimal implementation**

按 `task-journal` 要求追加：
- 修改内容
- 修改原因
- 未完成项
- 潜在风险
- 验证结果
- 记忆点

同时在计划末尾补充边界说明：
- vote Tool 是结构化表决引擎，不替代研究层
- research limitations 必须保留进投决结果
- Agent 只能解释 vote result，不能覆盖 factual payload

**Step 4: Run final verification**

再次确认：
- `security_committee_vote` 可调
- `committee_payload` 已投票就绪
- Skill 门禁已生效
- research limitations 未被吞掉

**Step 5: Commit**

```bash
git add CHANGELOG_TASK.MD docs/plans/2026-04-02-security-committee-vote-implementation.md
git commit -m "docs: record security committee vote plan"
```

---

## Testing Matrix

- 合同层：
  - `security_committee_vote_is_cataloged`
  - `security_decision_briefing_exposes_vote_ready_committee_payload`
- 输入校验层：
  - `security_committee_vote_rejects_invalid_payload`
- 角色投票层：
  - `security_committee_vote_emits_fixed_member_votes`
- 聚合规则层：
  - `security_committee_vote_applies_aggregation_rules`
- 分歧与条件层：
  - `security_committee_vote_surfaces_conditions_and_disagreements`
- 研究增强层：
  - `committee_payload_includes_research_confidence_fields`
  - `security_committee_vote_uses_historical_confidence`

## Open Decisions To Confirm Before Implementation

1. `committee_payload` 的升级是否允许新增嵌套结构但保留旧字段
2. `strict` 模式下是否允许 `conditional_approve`
3. `risk_officer` 的 veto 是否在 `advisory` 模式完全失效
4. `historical_digest` 未接入时默认是 `unavailable` 还是 `unknown`
5. 最终结果是否需要额外写入 SQLite 留痕

## Recommended Defaults

- 保留旧字段，新增嵌套结构，不破坏现有 briefing 消费方
- `strict` 模式允许 `conditional_approve`，但不能直接 `approved`
- `advisory` 模式禁用 veto，只输出建议倾向
- `historical_digest.status` 默认 `unavailable`
- 第一阶段不把 vote 结果写库，先锁合同和规则；若后续要复盘，再单独加 `committee_vote_store`

## Design Notes

- `security_committee_vote` 必须只消费 `committee_payload`，不能直接去读 `security_decision_briefing` 的其他层，更不能重新调用 fullstack / resonance / research。
- `committee_payload` 必须同时服务人工投决与自动 vote，因此结构应当是“摘要可读 + 子块可算”的混合体。
- `research_limitations` 不能只在 briefing 中出现，必须进 `committee_payload`，并最终出现在 vote result 里。
- 上层 Agent 的职责是解释、追问和展示，不是重算 vote。
- 如果后续要做多 Agent 真正并行投票，也应在这个确定性引擎之上加 orchestration，而不是绕过它。

## Suggested Phase Order

1. 先把 `committee_payload` 补成投票就绪事实包
2. 再做 `security_committee_vote` 合同与接线
3. 再做输入校验
4. 再做固定角色投票器
5. 再做聚合、veto、conditions
6. 再接研究增强字段
7. 最后补 Skill 门禁和任务日志
