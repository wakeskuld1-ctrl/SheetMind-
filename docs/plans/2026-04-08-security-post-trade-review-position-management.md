# Security Post-Trade Review And Position Management Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在证券主链上补齐“单标的正式仓位计划 -> 多次调仓记录 -> 投后复盘”的最小闭环，并确保所有对象都回指既有 `decision_ref / approval_ref / evidence_version`。

**Architecture:** 复用现有 `security_decision_briefing` 输出的 `position_plan` 作为计划事实源，新增 `security_position_plan_record`、`security_record_position_adjustment`、`security_post_trade_review` 三个正式 Tool。仓位计划对象作为锚点，调仓事件只记录实际执行，投后复盘只做聚合评估，不重复生成第二套研究事实。

**Tech Stack:** Rust、serde、现有 CLI Tool dispatcher、runtime 本地存储层、项目内 Tool catalog、cargo test

---

### Task 1: 明确仓位计划记录合同

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\src\tools\contracts.rs`
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\src\ops\security_decision_briefing.rs`
- Test: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\tests\integration_tool_contract.rs`

**Step 1: Write the failing test**

在 `tests/integration_tool_contract.rs` 新增红测，断言未来的 `security_position_plan_record` 至少暴露以下字段：

- `position_plan_ref`
- `decision_ref`
- `approval_ref`
- `evidence_version`
- `symbol`
- `analysis_date`
- `position_action`
- `starter_position_pct`
- `max_position_pct`

**Step 2: Run test to verify it fails**

Run: `cargo test --test integration_tool_contract security_position_plan_record_contract -- --nocapture`
Expected: FAIL with “tool missing” or “field missing”

**Step 3: Write minimal implementation**

先在 `contracts.rs` 与相关 `ops` 文件中定义 request/response struct，只保证合同成型，不先接持久化逻辑。

**Step 4: Run test to verify it passes**

Run: `cargo test --test integration_tool_contract security_position_plan_record_contract -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/tools/contracts.rs src/ops/security_decision_briefing.rs tests/integration_tool_contract.rs
git commit -m "feat: add security position plan record contract"
```

### Task 2: 落正式仓位计划记录 Tool

**Files:**
- Create: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\src\ops\security_position_plan_record.rs`
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\src\ops\mod.rs`
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\src\tools\catalog.rs`
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\src\tools\dispatcher.rs`
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\src\tools\dispatcher\stock_ops.rs`
- Test: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\tests\security_analysis_resonance_cli.rs`

**Step 1: Write the failing test**

新增 CLI 红测，断言：

- `security_position_plan_record` 可消费 briefing 派生的仓位计划
- 返回 `position_plan_ref`
- 记录内容与 briefing 顶层 `position_plan` 同源

**Step 2: Run test to verify it fails**

Run: `cargo test --test security_analysis_resonance_cli security_position_plan_record_persists_briefing_plan -- --nocapture`
Expected: FAIL

**Step 3: Write minimal implementation**

创建 `security_position_plan_record.rs`，完成：

- request 解析
- 从 briefing/引用对象读取仓位计划
- 生成 `position_plan_ref`
- 返回稳定结构化响应

同时完成 Tool 注册与 dispatcher 接线。

**Step 4: Run test to verify it passes**

Run: `cargo test --test security_analysis_resonance_cli security_position_plan_record_persists_briefing_plan -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ops/security_position_plan_record.rs src/ops/mod.rs src/tools/catalog.rs src/tools/dispatcher.rs src/tools/dispatcher/stock_ops.rs tests/security_analysis_resonance_cli.rs
git commit -m "feat: add security position plan record tool"
```

### Task 3: 明确调仓事件合同

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\src\tools\contracts.rs`
- Test: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\tests\integration_tool_contract.rs`

**Step 1: Write the failing test**

新增红测，断言 `security_record_position_adjustment` 至少暴露：

- `adjustment_event_ref`
- `position_plan_ref`
- `event_type`
- `event_date`
- `before_position_pct`
- `after_position_pct`
- `trigger_reason`
- `plan_alignment`

**Step 2: Run test to verify it fails**

Run: `cargo test --test integration_tool_contract security_record_position_adjustment_contract -- --nocapture`
Expected: FAIL

**Step 3: Write minimal implementation**

先补 request/response 合同和枚举字段，锁死 `build/add/reduce/exit/risk_update` 与 `on_plan/justified_deviation/off_plan`。

**Step 4: Run test to verify it passes**

Run: `cargo test --test integration_tool_contract security_record_position_adjustment_contract -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/tools/contracts.rs tests/integration_tool_contract.rs
git commit -m "feat: add position adjustment contract"
```

### Task 4: 落调仓事件记录 Tool

**Files:**
- Create: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\src\ops\security_record_position_adjustment.rs`
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\src\ops\mod.rs`
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\src\tools\catalog.rs`
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\src\tools\dispatcher.rs`
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\src\tools\dispatcher\stock_ops.rs`
- Test: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\tests\security_committee_vote_cli.rs`

**Step 1: Write the failing test**

新增 CLI 红测，断言：

- 同一 `position_plan_ref` 可连续登记多条调仓事件
- 每条事件都能回指 `decision_ref / approval_ref / evidence_version`
- `plan_alignment` 被稳定保留

**Step 2: Run test to verify it fails**

Run: `cargo test --test security_committee_vote_cli security_record_position_adjustment_supports_multiple_events -- --nocapture`
Expected: FAIL

**Step 3: Write minimal implementation**

创建 `security_record_position_adjustment.rs`，完成：

- 调仓事件请求处理
- `adjustment_event_ref` 生成
- 多次事件的最小存取接口
- Tool 注册与 dispatcher 接线

**Step 4: Run test to verify it passes**

Run: `cargo test --test security_committee_vote_cli security_record_position_adjustment_supports_multiple_events -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ops/security_record_position_adjustment.rs src/ops/mod.rs src/tools/catalog.rs src/tools/dispatcher.rs src/tools/dispatcher/stock_ops.rs tests/security_committee_vote_cli.rs
git commit -m "feat: add position adjustment recording tool"
```

### Task 5: 明确投后复盘合同

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\src\tools\contracts.rs`
- Test: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\tests\integration_tool_contract.rs`

**Step 1: Write the failing test**

新增红测，断言 `security_post_trade_review` 至少暴露：

- `post_trade_review_ref`
- `position_plan_ref`
- `decision_ref`
- `approval_ref`
- `review_outcome`
- `decision_accuracy`
- `execution_quality`
- `risk_control_quality`
- `correction_actions`
- `next_cycle_guidance`

**Step 2: Run test to verify it fails**

Run: `cargo test --test integration_tool_contract security_post_trade_review_contract -- --nocapture`
Expected: FAIL

**Step 3: Write minimal implementation**

先定义复盘 request/response 合同，固定 V1 复盘维度和输出字段。

**Step 4: Run test to verify it passes**

Run: `cargo test --test integration_tool_contract security_post_trade_review_contract -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/tools/contracts.rs tests/integration_tool_contract.rs
git commit -m "feat: add post trade review contract"
```

### Task 6: 落投后复盘 Tool

**Files:**
- Create: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\src\ops\security_post_trade_review.rs`
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\src\ops\mod.rs`
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\src\tools\catalog.rs`
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\src\tools\dispatcher.rs`
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\src\tools\dispatcher\stock_ops.rs`
- Test: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\tests\security_analysis_resonance_cli.rs`

**Step 1: Write the failing test**

新增 CLI 红测，断言：

- `security_post_trade_review` 可消费单个 `position_plan_ref` 与多条 `adjustment_event_ref`
- 会输出阶段性复盘
- 会稳定返回 5 个维度的聚合判断

**Step 2: Run test to verify it fails**

Run: `cargo test --test security_analysis_resonance_cli security_post_trade_review_aggregates_multiple_adjustments -- --nocapture`
Expected: FAIL

**Step 3: Write minimal implementation**

创建 `security_post_trade_review.rs`，完成：

- 聚合同一计划的调仓事件
- 输出 V1 复盘维度
- 生成 `post_trade_review_ref`
- Tool 注册与 dispatcher 接线

**Step 4: Run test to verify it passes**

Run: `cargo test --test security_analysis_resonance_cli security_post_trade_review_aggregates_multiple_adjustments -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ops/security_post_trade_review.rs src/ops/mod.rs src/tools/catalog.rs src/tools/dispatcher.rs src/tools/dispatcher/stock_ops.rs tests/security_analysis_resonance_cli.rs
git commit -m "feat: add post trade review tool"
```

### Task 7: 打通闭环回归与交接文档

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\docs\AI_HANDOFF.md`
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\README.md`
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\.trae\CHANGELOG_TASK.md`
- Test: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\tests\integration_tool_contract.rs`
- Test: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\tests\security_analysis_resonance_cli.rs`
- Test: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\tests\security_committee_vote_cli.rs`

**Step 1: Write the failing test**

不新增独立红测，改为整理验收清单：

- 新 Tool 已进入 catalog
- 合同测试覆盖通过
- 单标的多次调仓闭环可跑通
- 文档已同步当前证券主链阶段

**Step 2: Run test to verify current status**

Run:
- `cargo test --test integration_tool_contract -- --nocapture`
- `cargo test --test security_analysis_resonance_cli -- --nocapture`
- `cargo test --test security_committee_vote_cli -- --nocapture`

Expected: 相关新能力全部 PASS

**Step 3: Write minimal implementation**

同步更新：

- `README.md`
- `docs/AI_HANDOFF.md`
- `.trae/CHANGELOG_TASK.md`

把当前证券主链状态更新为“已具备仓位计划正式化、调仓记录与投后复盘的最小闭环”。

**Step 4: Run test to verify it passes**

重复运行上述命令，确认回归仍然通过。

**Step 5: Commit**

```bash
git add README.md docs/AI_HANDOFF.md .trae/CHANGELOG_TASK.md
git commit -m "docs: record securities post trade review and position management closeout"
```

Plan complete and saved to `docs/plans/2026-04-08-security-post-trade-review-position-management.md`. Two execution options:

**1. Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

**2. Parallel Session (separate)** - Open new session with executing-plans, batch execution with checkpoints

Which approach?
