# Security Decision Committee V3 Seven-Seat Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在不重复新增提交 Tool 的前提下，把现有证券投决会升级为“6 名审议委员 + 1 名风控委员”的七席合议制，并保持 approval/package/audit 主链可用。

**Architecture:** 继续沿用 `security_decision_committee -> security_decision_submit_approval -> approval/package/verify/revision` 主线。第一阶段先把 committee 内核升级为七席委员会并保留必要兼容字段；第二阶段再让 approval brief、approval bridge、position plan 原生消费七席数据。

**Tech Stack:** Rust、Cargo test、现有 Tool dispatcher、现有 approval bridge / package 主线

---

### Task 1: 冻结 V3 兼容边界

**Files:**
- Modify: `D:\Rust\Excel_Skill\docs\plans\2026-04-07-security-decision-committee-v3-seven-seat-design.md`
- Modify: `D:\Rust\Excel_Skill\docs\plans\2026-04-07-security-decision-committee-v3-seven-seat.md`

**Step 1: 复核“不重复新增提交 Tool”边界**

检查：
- `src/ops/security_decision_submit_approval.rs`
- `src/tools/dispatcher/stock_ops.rs`

确认：
- 继续复用 `security_decision_submit_approval`
- 仅升级 `security_decision_committee` 内部结构

**Step 2: 记录兼容阶段**

在设计/计划文档里明确：
- 阶段 1 保留双 Agent 兼容字段
- 阶段 2 再清理旧字段

**Step 3: 提交文档变更**

Run: `git diff -- docs/plans/2026-04-07-security-decision-committee-v3-seven-seat-design.md docs/plans/2026-04-07-security-decision-committee-v3-seven-seat.md`
Expected: 仅出现本轮 V3 七席设计与计划内容

### Task 2: 先写七席委员会红测

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\security_decision_committee_cli.rs`

**Step 1: 写失败测试，锁定七席基础合同**

新增测试覆盖：
- 七席总数为 7
- `member_opinions` 数组长度为 7
- 每位委员都有 `vote` 与 `reasoning`
- `vote_tally` 能统计 6 名审议委员与 1 名风控委员

**Step 2: 运行红测**

Run: `cargo test --test security_decision_committee_cli seven_seat_committee_exposes_member_opinions -- --nocapture`
Expected: FAIL，提示字段缺失或合同不匹配

**Step 3: 提交红测草案**

Run: `git diff -- tests/security_decision_committee_cli.rs`
Expected: 只出现七席 committee 新增测试

### Task 3: 新增席位注册表与委员意见合同

**Files:**
- Create: `D:\Rust\Excel_Skill\src\ops\security_committee_member_registry.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_card.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\mod.rs`

**Step 1: 写最小席位注册表**

定义：
- 6 名审议席位
- 1 名风控席位
- 市场轻微微调配置入口

**Step 2: 新增委员意见合同**

在 `security_decision_card.rs` 或独立模块中补：
- `SecurityCommitteeMemberOpinion`
- `SecurityCommitteeVoteTally`
- `SecurityCommitteeRiskVeto`

**Step 3: 运行编译检查**

Run: `cargo test --test security_decision_committee_cli seven_seat_committee_exposes_member_opinions -- --nocapture`
Expected: 仍 FAIL，但从“类型不存在”推进到“业务逻辑未实现”

### Task 4: 实现七席独立运行器

**Files:**
- Create: `D:\Rust\Excel_Skill\src\ops\security_committee_member_runner.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_committee.rs`

**Step 1: 写最小实现**

实现：
- 基于统一 `evidence_bundle` 构造七席请求
- 每席独立生成 opinion
- 保留 `execution_mode`

**Step 2: 跑第一轮绿测**

Run: `cargo test --test security_decision_committee_cli seven_seat_committee_exposes_member_opinions -- --nocapture`
Expected: PASS

**Step 3: 提交阶段性变更**

Run: `git diff -- src/ops/security_decision_committee.rs src/ops/security_committee_member_runner.rs src/ops/security_committee_member_registry.rs src/ops/security_decision_card.rs tests/security_decision_committee_cli.rs`
Expected: 仅出现七席委员会最小闭环

### Task 5: 写票数统计与多数意见红测

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\security_decision_committee_cli.rs`

**Step 1: 新增失败测试**

覆盖：
- 6 名审议委员简单多数
- `majority_opinion` 正确生成
- `minority_opinions` 正确保留

**Step 2: 运行红测**

Run: `cargo test --test security_decision_committee_cli seven_seat_committee_builds_majority_and_minority_opinions -- --nocapture`
Expected: FAIL

### Task 6: 实现票数统计器与多数/少数意见生成

**Files:**
- Create: `D:\Rust\Excel_Skill\src\ops\security_committee_vote_tally.rs`
- Create: `D:\Rust\Excel_Skill\src\ops\security_committee_majority_opinion.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_committee.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_card.rs`

**Step 1: 实现计票**

要求：
- 区分审议席与风控席
- 统计多数票
- 提取少数意见

**Step 2: 运行绿测**

Run: `cargo test --test security_decision_committee_cli seven_seat_committee_builds_majority_and_minority_opinions -- --nocapture`
Expected: PASS

### Task 7: 写风控有限否决红测

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\security_decision_committee_cli.rs`

**Step 1: 新增失败测试**

覆盖：
- 多数支持但风控要求降级为 `needs_more_evidence`
- 多数支持但风控直接 `blocked`

**Step 2: 运行红测**

Run: `cargo test --test security_decision_committee_cli seven_seat_committee_applies_risk_veto -- --nocapture`
Expected: FAIL

### Task 8: 实现风控有限否决

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_committee.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_card.rs`

**Step 1: 写最小实现**

规则：
- 风控席不反向改方向
- 只能降级到 `needs_more_evidence` 或 `blocked`
- 记录 `risk_veto_status` 与理由

**Step 2: 运行绿测**

Run: `cargo test --test security_decision_committee_cli seven_seat_committee_applies_risk_veto -- --nocapture`
Expected: PASS

### Task 9: 保持 submit Tool 不重复且仍可送审

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\security_decision_submit_approval_cli.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_submit_approval.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_approval_bridge.rs`

**Step 1: 写失败测试**

覆盖：
- `security_decision_submit_approval` 继续工作
- 不新增新的 submit Tool
- 新 committee 字段能被桥接

**Step 2: 运行红测**

Run: `cargo test --test security_decision_submit_approval_cli -- --nocapture`
Expected: FAIL，集中在桥接字段不兼容

**Step 3: 写最小兼容实现**

要求：
- submit 入口保持不变
- bridge 优先消费新字段
- 阶段 1 必要时兼容旧字段

**Step 4: 运行绿测**

Run: `cargo test --test security_decision_submit_approval_cli -- --nocapture`
Expected: PASS

### Task 10: 升级 approval brief 与 position plan

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_approval_brief.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_position_plan.rs`
- Modify: `D:\Rust\Excel_Skill\tests\security_decision_verify_package_cli.rs`
- Modify: `D:\Rust\Excel_Skill\tests\security_decision_package_revision_cli.rs`

**Step 1: 写失败测试**

覆盖：
- approval brief 展示 majority/minority/veto
- position plan 读取委员会分歧与风控降级

**Step 2: 运行红测**

Run: `cargo test --test security_decision_verify_package_cli --test security_decision_package_revision_cli -- --nocapture`
Expected: FAIL

**Step 3: 写最小实现**

要求：
- brief 不再只依赖 bull/bear
- plan 可感知 consensus 与 veto

**Step 4: 运行绿测**

Run: `cargo test --test security_decision_verify_package_cli --test security_decision_package_revision_cli -- --nocapture`
Expected: PASS

### Task 11: 跑委员会与治理链完整回归

**Files:**
- No file changes required

**Step 1: 运行 committee 全量回归**

Run: `cargo test --test security_decision_committee_cli -- --nocapture`
Expected: PASS

**Step 2: 运行治理链回归**

Run: `cargo test --test security_decision_submit_approval_cli --test security_decision_verify_package_cli --test security_decision_package_revision_cli`
Expected: PASS

**Step 3: 记录回归结果**

把通过的命令与结果补进任务日志与实现说明

### Task 12: 收尾与任务日志

**Files:**
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: 追加任务日志**

记录：
- 七席委员会设计与实现范围
- 保留 submit Tool 的兼容约束
- 已完成测试
- 潜在风险与后续建议

**Step 2: 最终检查**

Run: `git diff -- docs/plans/2026-04-07-security-decision-committee-v3-seven-seat-design.md docs/plans/2026-04-07-security-decision-committee-v3-seven-seat.md .trae/CHANGELOG_TASK.md`
Expected: 只出现本轮文档与任务日志变更
