# Security Committee Vote Seven-Seat Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在 CLI 分支现有 `security_decision_briefing -> security_committee_vote` 主链上，升级七席委员会与独立执行证明，并最终直接推送到 `codex/merge-cli-mod-batches`。

**Architecture:** 不回退到旧 `security_decision_committee` 架构，而是在 `security_committee_vote` 内部完成固定七席、子进程独立执行、风控有限否决和合同增强。`security_decision_briefing` 继续作为唯一事实装配层，`security_committee_vote` 继续作为唯一正式投决入口。

**Tech Stack:** Rust、Cargo test、现有 EXE stdin/stdout ToolRequest/ToolResponse 调度、Markdown 文档。

---

### Task 1: 写七席委员会红测

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\tests\security_committee_vote_cli.rs`

**Step 1: Write the failing test**

新增测试覆盖：

- `votes.len() == 7`
- `6` 个审议席 + `1` 个风控席
- 每席都有 `execution_mode / process_id / execution_instance_id / evidence_version`
- 所有席位 `evidence_version` 一致
- 所有席位 `process_id / execution_instance_id` 唯一

**Step 2: Run test to verify it fails**

Run: `cargo test --test security_committee_vote_cli security_committee_vote_exposes_seven_seat_independent_execution -- --nocapture`

Expected: FAIL，提示字段缺失或席位数量不对。

### Task 2: 扩展正式投票合同

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\src\ops\security_committee_vote.rs`

**Step 1: Write minimal contract changes**

扩展：

- `CommitteeMemberVote`
- `SecurityCommitteeVoteResult`

新增必要的独立执行元数据与七席统计字段。

**Step 2: Run test to verify progress**

Run: `cargo test --test security_committee_vote_cli security_committee_vote_exposes_seven_seat_independent_execution -- --nocapture`

Expected: 仍 FAIL，但失败点推进到运行逻辑未实现。

### Task 3: 接入子进程成员执行器

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\src\ops\security_committee_vote.rs`
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\src\tools\dispatcher\stock_ops.rs`
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\src\tools\dispatcher.rs`
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\src\tools\catalog.rs`
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\src\tools\contracts.rs`

**Step 1: Write minimal child-process path**

新增内部席位 agent：

- `security_committee_member_agent`

并让 `security_committee_vote` 通过当前 EXE 子进程调用该 agent 生成每席意见。

**Step 2: Run test to verify it passes**

Run: `cargo test --test security_committee_vote_cli security_committee_vote_exposes_seven_seat_independent_execution -- --nocapture`

Expected: PASS。

### Task 4: 写风控有限否决红测

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\tests\security_committee_vote_cli.rs`

**Step 1: Write the failing test**

新增测试覆盖：

- 多数审议席支持
- 风控席 reject
- 最终 `veto_triggered = true`
- `final_decision` 正确降级

**Step 2: Run test to verify it fails**

Run: `cargo test --test security_committee_vote_cli security_committee_vote_applies_risk_officer_veto_under_seven_seat_mode -- --nocapture`

Expected: FAIL。

### Task 5: 实现七席聚合与有限否决

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\src\ops\security_committee_vote.rs`

**Step 1: Implement minimal aggregation**

实现：

- 固定七席 roster
- 6 席多数票统计
- 风控席有限否决
- `committee_engine / majority_vote / majority_count / deliberation_seat_count / risk_seat_count`

**Step 2: Run tests**

Run: `cargo test --test security_committee_vote_cli -- --nocapture`

Expected: PASS。

### Task 6: 回归 briefing 链

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\src\ops\security_decision_briefing.rs`
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\tests\security_analysis_fullstack_cli.rs`

**Step 1: Verify compatibility**

确保 `security_decision_briefing` 继续能消费新的 `SecurityCommitteeVoteResult`。

**Step 2: Run tests**

Run: `cargo test --test security_committee_vote_cli --test security_analysis_fullstack_cli -- --nocapture`

Expected: PASS。

### Task 7: 补执行说明与交接

**Files:**
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\docs\execution-notes-2026-03-30.md`
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\docs\交接摘要_证券分析_给后续AI.md`
- Modify: `C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-cli-mod-review\.trae\CHANGELOG_TASK.md`

**Step 1: Write factual handoff updates**

记录：

- 为什么在 CLI 分支选择方案 B
- 七席委员会的正式入口
- 如何证明独立
- 已验证命令
- 仍未完成的更强隔离项

### Task 8: 最终验证与 Git 上传

**Files:**
- No file changes required

**Step 1: Run fresh verification**

Run:

- `cargo test --test security_committee_vote_cli -- --nocapture`
- `cargo test --test security_analysis_fullstack_cli -- --nocapture`

Expected: PASS。

**Step 2: Stage only task files**

只 stage 本轮七席委员会与文档相关文件。

**Step 3: Commit**

建议提交信息：

```bash
git commit -m "feat: upgrade committee vote to seven-seat isolated execution"
```

**Step 4: Push**

Run:

```bash
git push origin codex/merge-cli-mod-batches
```
