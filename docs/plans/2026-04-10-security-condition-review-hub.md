# Security Condition Review Hub Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在没有实时数据的前提下，为证券主链补齐“条件复核中枢”，让投中阶段能够正式判断原决策是否仍成立，并把复核结果挂回 package、execution 和 review 主链。

**Architecture:** 继续沿用现有证券主链，不新建第二套执行系统。新增 `security_condition_review` 作为投中层正式对象，支持 `manual_review / end_of_day_review / event_review / data_staleness_review` 四类触发，再把 `condition_review_ref` 接入 `security_decision_package`、`security_execution_record`、`security_post_trade_review`，形成“投前决策 -> 投中复核 -> 执行事实 -> 投后复盘”的单主链。

**Tech Stack:** Rust、Serde JSON、CLI-first、现有 `src/ops/security_*` 主链、`tests/*_cli.rs` 集成测试、Markdown 交接文档。

---

## 当前前提

- 已有正式投前对象：
  - `src/ops/security_decision_briefing.rs`
  - `src/ops/security_committee_vote.rs`
  - `src/ops/security_position_plan.rs`
  - `src/ops/security_decision_package.rs`
- 已有正式执行与复盘对象：
  - `src/ops/security_execution_journal.rs`
  - `src/ops/security_execution_record.rs`
  - `src/ops/security_post_trade_review.rs`
- 当前不具备实时行情流，因此实现目标必须限定为条件复核，不做盘中秒级监控。

### Task 1: 冻结 `security_condition_review` 请求与文档合同

**Files:**
- Create: `D:\Rust\Excel_Skill\src\ops\security_condition_review.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\contracts.rs`
- Test: `D:\Rust\Excel_Skill\tests\security_condition_review_cli.rs`

**Step 1: Write the failing test**

在 `tests/security_condition_review_cli.rs` 新增最小红测，验证：

- 可以提交一次 `manual_review`
- 输出包含 `decision_ref / approval_ref / position_plan_ref`
- 输出包含 `recommended_follow_up_action`

**Step 2: Run test to verify it fails**

Run: `cargo test --test security_condition_review_cli security_condition_review_manual_review_contract -- --nocapture`
Expected: FAIL，因为新 Tool 尚不存在

**Step 3: Write minimal implementation**

在 `src/ops/security_condition_review.rs` 新增：

- `SecurityConditionReviewRequest`
- `SecurityConditionReviewDocument`
- `SecurityConditionReviewResult`
- `SecurityConditionReviewError`

在 `src/tools/contracts.rs` 补正式合同映射。

**Step 4: Run test to verify it passes**

Run: `cargo test --test security_condition_review_cli security_condition_review_manual_review_contract -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add tests/security_condition_review_cli.rs src/ops/security_condition_review.rs src/tools/contracts.rs
git commit -m "feat: add security condition review contract"
```

### Task 2: 固定四类触发模式与动作分流

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_condition_review.rs`
- Test: `D:\Rust\Excel_Skill\tests\security_condition_review_cli.rs`

**Step 1: Write the failing tests**

新增红测覆盖：

- `manual_review`
- `end_of_day_review`
- `event_review`
- `data_staleness_review`

并断言输出动作属于：

- `keep_plan`
- `update_position_plan`
- `reopen_research`
- `reopen_committee`
- `freeze_execution`

**Step 2: Run tests to verify they fail**

Run: `cargo test --test security_condition_review_cli security_condition_review_ -- --nocapture`
Expected: FAIL，因触发规则和动作映射尚未完整

**Step 3: Write minimal implementation**

在 `src/ops/security_condition_review.rs` 内：

- 固定 `review_trigger_type`
- 实现最小分流规则
- 生成 `review_findings` 与 `review_summary`

**Step 4: Run tests to verify they pass**

Run: `cargo test --test security_condition_review_cli security_condition_review_ -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add tests/security_condition_review_cli.rs src/ops/security_condition_review.rs
git commit -m "feat: add condition review trigger routing"
```

### Task 3: 注册 Tool 并接入 CLI 主链入口

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\stock.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\mod.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\catalog.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\dispatcher.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\dispatcher\stock_ops.rs`
- Test: `D:\Rust\Excel_Skill\tests\security_condition_review_cli.rs`

**Step 1: Write the failing test**

新增红测验证：

- Tool 已出现在 `tool_catalog`
- CLI stdin/stdout 路由能返回结构化 `condition_review`

**Step 2: Run test to verify it fails**

Run: `cargo test --test security_condition_review_cli security_condition_review_is_cataloged -- --nocapture`
Expected: FAIL

**Step 3: Write minimal implementation**

把 `security_condition_review` 接入：

- `src/ops/stock.rs`
- `src/ops/mod.rs`
- `src/tools/catalog.rs`
- `src/tools/dispatcher.rs`
- `src/tools/dispatcher/stock_ops.rs`

**Step 4: Run test to verify it passes**

Run: `cargo test --test security_condition_review_cli -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ops/stock.rs src/ops/mod.rs src/tools/catalog.rs src/tools/dispatcher.rs src/tools/dispatcher/stock_ops.rs tests/security_condition_review_cli.rs
git commit -m "feat: wire security condition review into cli"
```

### Task 4: 把 `condition_review_ref` 挂入 decision package

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_package.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_verify_package.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_package_revision.rs`
- Test: `D:\Rust\Excel_Skill\tests\security_decision_package_cli.rs`

**Step 1: Write the failing test**

新增红测验证：

- package 可以引用 `condition_review_ref`
- verify 能识别缺失或漂移的 `condition_review_ref`
- revision 能给出修补建议

**Step 2: Run test to verify it fails**

Run: `cargo test --test security_decision_package_cli condition_review -- --nocapture`
Expected: FAIL

**Step 3: Write minimal implementation**

在 package / verify / revision 中：

- 增加 `condition_review` 对象挂载
- 增加完整性校验
- 增加修补建议

**Step 4: Run test to verify it passes**

Run: `cargo test --test security_decision_package_cli condition_review -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ops/security_decision_package.rs src/ops/security_decision_verify_package.rs src/ops/security_decision_package_revision.rs tests/security_decision_package_cli.rs
git commit -m "feat: bind condition review into decision package"
```

### Task 5: 把 `condition_review_ref` 挂入 execution 和 review 链

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_execution_record.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_post_trade_review.rs`
- Test: `D:\Rust\Excel_Skill\tests\security_execution_record_cli.rs`
- Test: `D:\Rust\Excel_Skill\tests\security_post_trade_review_cli.rs`

**Step 1: Write the failing tests**

新增红测验证：

- `execution_record` 可选挂接 `condition_review_ref`
- `post_trade_review` 能读取并解释最近一次条件复核结果

**Step 2: Run tests to verify they fail**

Run: `cargo test --test security_execution_record_cli condition_review -- --nocapture`
Run: `cargo test --test security_post_trade_review_cli condition_review -- --nocapture`
Expected: FAIL

**Step 3: Write minimal implementation**

在执行与复盘层中：

- 新增 `condition_review_ref`
- 新增最小解释字段
- 保持旧链路兼容

**Step 4: Run tests to verify they pass**

Run: `cargo test --test security_execution_record_cli condition_review -- --nocapture`
Run: `cargo test --test security_post_trade_review_cli condition_review -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ops/security_execution_record.rs src/ops/security_post_trade_review.rs tests/security_execution_record_cli.rs tests/security_post_trade_review_cli.rs
git commit -m "feat: connect condition review to execution and review"
```

### Task 6: 更新交接文档与验证清单

**Files:**
- Modify: `D:\Rust\Excel_Skill\docs\AI_HANDOFF.md`
- Modify: `D:\Rust\Excel_Skill\docs\交接摘要_证券分析_给后续AI.md`
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Document the new layer**

补充：

- “投中监控中枢”正式改名为“条件复核中枢”
- 当前不依赖实时数据
- 默认验证命令新增 `security_condition_review_cli`

**Step 2: Run the targeted verification**

Run: `cargo test --test security_condition_review_cli -- --nocapture`
Run: `cargo test --test security_decision_package_cli -- --nocapture`
Run: `cargo test --test security_execution_record_cli -- --nocapture`
Run: `cargo test --test security_post_trade_review_cli -- --nocapture`
Run: `cargo test --tests --no-run`
Expected: PASS

**Step 3: Commit**

```bash
git add docs/AI_HANDOFF.md "docs/交接摘要_证券分析_给后续AI.md" .trae/CHANGELOG_TASK.md
git commit -m "docs: record condition review hub handoff"
```
