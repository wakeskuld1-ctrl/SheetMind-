<!-- 2026-04-02 CST: 新增这份执行记录，原因是本轮证券投前治理能力和 159866 正式投研会结果已经形成明确交接价值；目的是在 push 前留下可追踪的执行证据，方便后续 AI/工程师直接接手。 -->
# 2026-04-02 执行记录：证券投前治理链与 159866 正式投研会

## 本轮改动

- 把证券主线从研究链继续扩展到投前治理链：
  - `security_decision_evidence_bundle`
  - `security_decision_committee`
  - `security_risk_gates`
  - `security_decision_card`
  - `security_decision_approval_bridge`
  - `security_decision_submit_approval`
  - `security_position_plan`
  - `security_decision_approval_brief`
  - `security_approval_brief_signature`
  - `security_decision_package`
  - `security_decision_verify_package`
  - `security_decision_package_revision`
- 新增问答总入口 Skill：
  - `skills/security-pm-assistant-v1/SKILL.md`
- 补齐本轮设计/实施文档：
  - `docs/plans/2026-04-01-security-decision-workbench-v1-design.md`
  - `docs/plans/2026-04-01-security-decision-workbench-v1.md`
  - `docs/plans/2026-04-02-security-decision-approval-bridge-design.md`
  - `docs/plans/2026-04-02-security-decision-approval-bridge.md`
  - `docs/plans/2026-04-02-security-position-plan-design.md`
  - `docs/plans/2026-04-02-security-position-plan.md`
  - `docs/plans/2026-04-02-security-approval-brief-document-design.md`
  - `docs/plans/2026-04-02-security-approval-brief-document.md`
  - `docs/plans/2026-04-02-security-decision-package-design.md`
  - `docs/plans/2026-04-02-security-decision-package.md`
  - `docs/plans/2026-04-02-security-decision-package-verification-design.md`
  - `docs/plans/2026-04-02-security-decision-package-verification.md`
  - `docs/plans/2026-04-02-security-decision-package-revision-design.md`
  - `docs/plans/2026-04-02-security-decision-package-revision.md`
  - `docs/plans/2026-04-02-security-pm-assistant-skill-design.md`
  - `docs/plans/2026-04-02-security-pm-assistant-skill.md`

## 改动原因

- 用户不再满足于“分析一下股票/ETF”，而是明确要求形成：
  - 双立场投研会/投决会
  - 可审批对象
  - 仓位计划
  - 正式审批简报
  - decision package
  - 包校验和版本化
- 用户还明确要求后续能复盘、纠错，因此本轮除了交付治理链本身，也保留了真实过会案例。

## 已验证

- `cargo test --test security_decision_evidence_bundle_cli`
  - 结果：通过
- `cargo test --test security_decision_committee_cli`
  - 结果：通过
- `cargo test --test security_decision_submit_approval_cli`
  - 结果：通过
- `cargo test --test security_decision_verify_package_cli`
  - 结果：通过
- `cargo test --test security_decision_package_revision_cli`
  - 结果：通过
- `cargo test --test security_decision_evidence_bundle_cli --test security_decision_committee_cli --test security_decision_submit_approval_cli --test security_decision_verify_package_cli --test security_decision_package_revision_cli`
  - 结果：通过

## 159866 正式投研会案例

- 运行工件：
  - `tests/runtime_fixtures/live_committee_159866_runtime/committee_result.json`
  - `tests/runtime_fixtures/live_committee_159866_runtime/runtime.db`
  - `tests/runtime_fixtures/live_committee_159866_runtime/stock_history.db`
- 输入背景：
  - 标的：`159866.SZ`
  - 市场代理：`^N225`
  - 板块代理：`^N225`
  - 分析日：`2026-04-01`
  - 用户持仓：`40%`
  - 用户成本：`1.466`
- 关键结果：
  - `decision_card.status = needs_more_evidence`
  - `direction = long`
  - `position_size_suggestion = pilot`
  - `confidence_score = 0.12`
- 结论翻译：
  - 当前不否决日本方向。
  - 当前否决“继续把 40% 重仓视作合理执行仓位”。
  - 目前更接近观察仓/试探仓结论。

## ETF 信息面缺口说明

- 当前证券信息面仍偏股票口径。
- 对 ETF 场景真正关键但尚未补齐的包括：
  - 溢价/折价
  - IOPV/NAV 偏离
  - 跟踪误差
  - ETF 专用公告
  - 份额变化/申赎
  - 汇率与海外指数映射
- 因此像 `159866.SZ` 这种跨境 ETF 当前容易出现：
  - `fundamental_context = unavailable`
  - `disclosure_context = unavailable`
  - `integrated_conclusion = technical_only`

## 当前已知风险

- 当前证券投前治理切片已验证，但未跑整仓测试。
- ETF 场景的信息面仍明显不足，继续做 ETF 决策时要优先补 ETF 专用数据，而不是继续强套股票财报/股票公告逻辑。
- `decision package` 已有 verify 与 revision，但还没有 package 自身签名，也没有 revision 自动触发。

## 下一位 AI 建议先做什么

1. 先读：
   - `docs/交接摘要_证券分析_给后续AI.md`
   - `docs/交接摘要_给后续AI.md`
   - `docs/execution-notes-2026-04-02-security-governance.md`
2. 再看：
   - `skills/security-pm-assistant-v1/SKILL.md`
   - `src/ops/security_decision_committee.rs`
   - `src/ops/security_decision_submit_approval.rs`
   - `src/ops/security_decision_verify_package.rs`
   - `src/ops/security_decision_package_revision.rs`
3. 如果继续 ETF 方向，优先补 ETF 专用信息面。
4. 如果继续纠错/复盘，优先补轻量 `review record`，绑定 `decision_ref / approval_ref / package_path`。
