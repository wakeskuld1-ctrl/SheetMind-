# Security Scorecard And Direction Semantics Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 修复证券投决 `direction` 旧语义漂移，并把正式 `security_scorecard` 独立对象挂入 package / revision / verify 主链。

**Architecture:** 先用失败测试锁住“多数票 avoid 但仍输出 long”的 bug，再新增独立评分卡对象和模型 artifact 消费层。评分卡本轮只消费版本化模型，不在运行时现场训练；无模型时返回正式 `model_unavailable` 状态，不伪造主观分数。

**Tech Stack:** Rust、Serde JSON、现有 CLI 集成测试、decision package / revision / verify 主链。

---

### Task 1: 锁定 `direction` 漂移 bug

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\security_decision_committee_cli.rs`

**Step 1: Write the failing test**

- 新增一个测试，断言：
  - `vote_tally.majority_vote = "avoid"` 时
  - `decision_card.recommendation_action = "avoid"`
  - `decision_card.exposure_side = "neutral"`
  - `decision_card.direction != "long"`

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test security_decision_committee_cli direction -- --nocapture
```

Expected:

- 失败点显示当前 `direction` 仍为 `long`

### Task 2: 锁定评分卡正式对象合同

**Files:**
- Create: `D:\Rust\Excel_Skill\tests\security_scorecard_cli.rs`

**Step 1: Write the failing test**

- 新增 happy path，断言：
  - 输出包含 `scorecard_id`
  - 输出包含 `score_status`
  - 输出包含 `raw_feature_snapshot`
  - 输出包含 `feature_contributions`
  - 无模型时 `score_status = "model_unavailable"`

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test security_scorecard_cli -- --nocapture
```

Expected:

- 因工具或对象不存在而失败

### Task 3: 实现最小评分卡对象与模型消费层

**Files:**
- Create: `D:\Rust\Excel_Skill\src\ops\security_scorecard.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\stock.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\mod.rs`

**Step 1: Write minimal implementation**

- 新增：
  - `SecurityScorecardDocument`
  - `SecurityScorecardModelArtifact`
  - `SecurityScoreFeatureContribution`
  - `SecurityScoreGroupBreakdown`
- 提供 builder：
  - 从 `committee` 提取特征
  - 若无模型则返回 `model_unavailable`
  - 若有模型则执行分箱映射和点数累加

**Step 2: Run targeted tests**

Run:

```powershell
cargo test --test security_scorecard_cli -- --nocapture
```

Expected:

- 从“对象不存在”推进到“尚未接入 package”

### Task 4: 修复 `decision_card` 语义并吸收委员会结论

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_card.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_committee.rs`
- Modify: `D:\Rust\Excel_Skill\tests\security_decision_committee_cli.rs`

**Step 1: Write minimal implementation**

- 为 `SecurityDecisionCard` 新增：
  - `recommendation_action`
  - `exposure_side`
- 让最终动作优先来自：
  - `vote_tally.majority_vote`
  - `risk_veto.status`
- 让旧字段 `direction` 从 `exposure_side` 派生

**Step 2: Run targeted tests**

Run:

```powershell
cargo test --test security_decision_committee_cli direction -- --nocapture
```

Expected:

- `direction` 漂移测试通过

### Task 5: 把评分卡挂入 package 主链

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_package.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_approval_bridge.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_submit_approval.rs`
- Modify: `D:\Rust\Excel_Skill\tests\security_decision_package_revision_cli.rs`

**Step 1: Write the failing test**

- 断言：
  - `artifact_manifest` 包含 `security_scorecard`
  - `object_graph.scorecard_ref`
  - `object_graph.scorecard_path`

**Step 2: Verify RED**

Run:

```powershell
cargo test --test security_decision_package_revision_cli scorecard -- --nocapture
```

Expected:

- 因 package 尚未挂接 scorecard 而失败

**Step 3: Write minimal implementation**

- submit 阶段落盘评分卡
- package object graph 增加评分卡引用
- manifest 增加评分卡 artifact

**Step 4: Verify GREEN**

Run:

```powershell
cargo test --test security_decision_package_revision_cli scorecard -- --nocapture
```

Expected:

- package 能稳定挂接评分卡

### Task 6: 扩展 verify 校验评分卡治理链

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_verify_package.rs`
- Modify: `D:\Rust\Excel_Skill\tests\security_decision_verify_package_cli.rs`

**Step 1: Write the failing test**

- 新增：
  - `scorecard_binding_consistent == true`
  - `scorecard_complete == true`
  - `scorecard_action_aligned == true`
- 再新增篡改失败用例：
  - 篡改 `scorecard_path`
  - 篡改 `recommendation_action`

**Step 2: Verify RED**

Run:

```powershell
cargo test --test security_decision_verify_package_cli scorecard -- --nocapture
```

Expected:

- 因 verify 尚未识别 scorecard 而失败

**Step 3: Write minimal implementation**

- `verify` 增加评分卡绑定、一致性、动作对齐校验

**Step 4: Verify GREEN**

Run:

```powershell
cargo test --test security_decision_verify_package_cli scorecard -- --nocapture
```

Expected:

- happy path 通过，篡改路径失败

### Task 7: 跑定向回归

**Files:**
- Test: `D:\Rust\Excel_Skill\tests\security_decision_committee_cli.rs`
- Test: `D:\Rust\Excel_Skill\tests\security_scorecard_cli.rs`
- Test: `D:\Rust\Excel_Skill\tests\security_decision_package_revision_cli.rs`
- Test: `D:\Rust\Excel_Skill\tests\security_decision_verify_package_cli.rs`

**Step 1: Run regression**

Run:

```powershell
cargo test --test security_decision_committee_cli -- --nocapture
cargo test --test security_scorecard_cli -- --nocapture
cargo test --test security_decision_package_revision_cli -- --nocapture
cargo test --test security_decision_verify_package_cli -- --nocapture
```

Expected:

- 本轮改动涉及的证券治理链全部通过

### Task 8: 记录任务日志

**Files:**
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Append task journal entry**

- 记录：
  - `direction` 语义修复
  - `security_scorecard` 正式对象
  - package / revision / verify 接线
  - 无模型时 `model_unavailable` 边界

**Step 2: Final verification**

Run:

```powershell
git status --short
```

Expected:

- 仅出现本轮相关代码、测试与文档改动
