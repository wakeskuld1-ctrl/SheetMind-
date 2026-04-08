# Master Balance Scorecard System Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 将证券评分体系升级为“可训练、可回算、可发布、三线隔离”的大平衡计分卡系统，并把综合计分卡、投委会与主席裁决正式拆开。

**Architecture:** 先冻结三线对象边界，再补特征快照与未来标签回填底座；随后落离线重估链路、artifact 注册表与线上消费层；最后补主席裁决、晋级治理与复盘闭环。训练逻辑属于正式主链的一部分，但执行在线下回算流水线，不在单次线上分析中临场重训。

**Tech Stack:** Rust、Serde JSON、SQLite、本地 CLI、现有证券治理主链、离线回算任务与正式 artifact 合同。

---

### Task 1: 冻结三线隔离对象边界

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_scorecard.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_committee.rs`
- Create: `D:\Rust\Excel_Skill\src\ops\security_chair_resolution.rs`
- Test: `D:\Rust\Excel_Skill\tests\security_chair_resolution_cli.rs`

**Step 1: Write the failing test**

- 先写失败测试，断言：
  - `security_scorecard` 只表示量化输出
  - `committee_session` 只表示委员结论
  - `chair_resolution` 才能产出最终正式动作

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test security_chair_resolution_cli -- --nocapture
```

Expected:

- 因对象或 Tool 尚不存在而失败

**Step 3: Write minimal implementation**

- 新增 `SecurityChairResolution`
- 明确 `scorecard / committee / chair` 三线对象引用关系

**Step 4: Run test to verify it passes**

Run:

```powershell
cargo test --test security_chair_resolution_cli -- --nocapture
```

Expected:

- 新增测试转绿

### Task 2: 落 feature_snapshot 正式底座

**Files:**
- Create: `D:\Rust\Excel_Skill\src\ops\security_feature_snapshot.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_evidence_bundle.rs`
- Test: `D:\Rust\Excel_Skill\tests\security_feature_snapshot_cli.rs`

**Step 1: Write the failing test**

- 断言：
  - 能生成 `snapshot_id`
  - 能冻结原子特征与因子组特征
  - 能生成 `snapshot_hash`

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test security_feature_snapshot_cli -- --nocapture
```

Expected:

- 因对象与生成逻辑不存在而失败

**Step 3: Write minimal implementation**

- 新增 `SecurityFeatureSnapshot`
- 从现有 evidence/committee 输入中收口“当时可见”的特征快照

**Step 4: Run test to verify it passes**

Run:

```powershell
cargo test --test security_feature_snapshot_cli -- --nocapture
```

Expected:

- 测试通过

### Task 3: 落 forward_outcome 多期限标签回填

**Files:**
- Create: `D:\Rust\Excel_Skill\src\ops\security_forward_outcome.rs`
- Test: `D:\Rust\Excel_Skill\tests\security_forward_outcome_cli.rs`

**Step 1: Write the failing test**

- 覆盖：
  - `5/10/20/30/60/180` 天标签回填
  - `forward_return / max_drawdown / max_runup`
  - `positive_return / hit_upside_first / hit_stop_first`

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test security_forward_outcome_cli -- --nocapture
```

Expected:

- 因标签回填逻辑尚不存在而失败

**Step 3: Write minimal implementation**

- 新增 `SecurityForwardOutcome`
- 建立多期限回填逻辑

**Step 4: Run test to verify it passes**

Run:

```powershell
cargo test --test security_forward_outcome_cli -- --nocapture
```

Expected:

- 测试通过

### Task 4: 落离线重估主对象与注册表

**Files:**
- Create: `D:\Rust\Excel_Skill\src\ops\security_scorecard_refit_run.rs`
- Create: `D:\Rust\Excel_Skill\src\ops\security_scorecard_model_registry.rs`
- Test: `D:\Rust\Excel_Skill\tests\security_scorecard_refit_cli.rs`

**Step 1: Write the failing test**

- 断言：
  - 能记录一次 `refit_run`
  - 能注册 candidate artifact
  - 能保存 `train / valid / test` 窗口

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test security_scorecard_refit_cli -- --nocapture
```

Expected:

- 因 refit 对象与注册表不存在而失败

**Step 3: Write minimal implementation**

- 新增 `SecurityScorecardRefitRun`
- 新增 `SecurityScorecardModelRegistry`

**Step 4: Run test to verify it passes**

Run:

```powershell
cargo test --test security_scorecard_refit_cli -- --nocapture
```

Expected:

- 测试通过

### Task 5: 落训练流水线正式入口

**Files:**
- Create: `D:\Rust\Excel_Skill\src\ops\security_scorecard_training.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\dispatcher\stock_ops.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\catalog.rs`
- Test: `D:\Rust\Excel_Skill\tests\security_scorecard_training_cli.rs`

**Step 1: Write the failing test**

- 断言训练入口支持：
  - 指定市场范围
  - 指定期限
  - 指定目标头
  - 生成 artifact
  - 写入 refit_run 与 model_registry

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test security_scorecard_training_cli -- --nocapture
```

Expected:

- 因训练 Tool/入口不存在而失败

**Step 3: Write minimal implementation**

- 新增离线训练入口
- 实现最小的：
  - 样本装载
  - 分箱
  - WOE 编码
  - Logistic 拟合
  - artifact 生成

**Step 4: Run test to verify it passes**

Run:

```powershell
cargo test --test security_scorecard_training_cli -- --nocapture
```

Expected:

- 训练入口测试通过

### Task 6: 把多期限子卡与总卡正式落盘

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_scorecard.rs`
- Create: `D:\Rust\Excel_Skill\src\ops\security_master_scorecard.rs`
- Test: `D:\Rust\Excel_Skill\tests\security_master_scorecard_cli.rs`

**Step 1: Write the failing test**

- 断言：
  - 每个期限有独立子卡结果
  - 能生成总卡
  - 总卡只输出量化立场，不输出正式主席决议

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test security_master_scorecard_cli -- --nocapture
```

Expected:

- 因总卡对象不存在或语义混淆而失败

**Step 3: Write minimal implementation**

- 新增总卡对象
- 新增多期限子卡汇总逻辑

**Step 4: Run test to verify it passes**

Run:

```powershell
cargo test --test security_master_scorecard_cli -- --nocapture
```

Expected:

- 测试通过

### Task 7: 接入 package 三线对象图

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_package.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_submit_approval.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_package_revision.rs`
- Test: `D:\Rust\Excel_Skill\tests\security_decision_package_revision_cli.rs`

**Step 1: Write the failing test**

- 断言 package object graph 同时挂：
  - `master_scorecard_ref`
  - `committee_session_ref`
  - `chair_resolution_ref`

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test security_decision_package_revision_cli scorecard_committee_chair -- --nocapture
```

Expected:

- 因 object graph 尚未完整接线而失败

**Step 3: Write minimal implementation**

- 扩展 package object graph
- 扩展 revision 透传

**Step 4: Run test to verify it passes**

Run:

```powershell
cargo test --test security_decision_package_revision_cli scorecard_committee_chair -- --nocapture
```

Expected:

- 测试通过

### Task 8: 扩展 verify 的三线治理校验

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_verify_package.rs`
- Test: `D:\Rust\Excel_Skill\tests\security_decision_verify_package_cli.rs`

**Step 1: Write the failing test**

- 覆盖：
  - `master_scorecard` 完整性
  - `committee_session` 完整性
  - `chair_resolution` 完整性
  - 主席决议只能引用前两条线，不能缺边

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test security_decision_verify_package_cli chair -- --nocapture
```

Expected:

- 因 verify 尚未识别主席线而失败

**Step 3: Write minimal implementation**

- 增加三线对象图治理校验
- 增加篡改路径失败测试

**Step 4: Run test to verify it passes**

Run:

```powershell
cargo test --test security_decision_verify_package_cli chair -- --nocapture
```

Expected:

- 测试通过

### Task 9: 落 champion / challenger 晋级规则

**Files:**
- Create: `D:\Rust\Excel_Skill\src\ops\security_scorecard_promotion.rs`
- Test: `D:\Rust\Excel_Skill\tests\security_scorecard_promotion_cli.rs`

**Step 1: Write the failing test**

- 断言：
  - challenger 必须对比 champion
  - 不达标不能晋级
  - 允许保留旧 champion

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test security_scorecard_promotion_cli -- --nocapture
```

Expected:

- 因晋级逻辑不存在而失败

**Step 3: Write minimal implementation**

- 增加晋级规则对象与执行器

**Step 4: Run test to verify it passes**

Run:

```powershell
cargo test --test security_scorecard_promotion_cli -- --nocapture
```

Expected:

- 测试通过

### Task 10: 跑阶段性回归并更新交接

**Files:**
- Modify: `D:\Rust\Excel_Skill\docs\交接摘要_证券分析_给后续AI.md`
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Run regression**

Run:

```powershell
cargo test --test security_scorecard_cli -- --nocapture
cargo test --test security_decision_committee_cli -- --nocapture
cargo test --test security_decision_verify_package_cli -- --nocapture
cargo test --test security_decision_package_revision_cli -- --nocapture
```

Expected:

- 证券主链相关套件通过

**Step 2: Append task journal entry**

- 记录：
  - 三线隔离
  - feature_snapshot / forward_outcome
  - 训练流水线
  - 多期限总卡
  - promotion 规则

**Step 3: Final verification**

Run:

```powershell
git status --short
```

Expected:

- 只出现本轮相关文档、代码与测试改动
