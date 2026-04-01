# Security Decision Workbench V1 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在当前证券分析主链上新增证券投决会 v1 最小闭环，让系统能够输出“证据冻结 -> 正反博弈 -> 风险闸门 -> 投决卡 -> 最终建议”的结构化结果。

**Architecture:** 保持现有 `technical_consultation_basic -> security_analysis_contextual -> security_analysis_fullstack` 研究链不变，在 `src/ops/stock` 下新增 `security_decision_evidence_bundle / security_risk_gates / security_decision_card` 三个模块，并通过新的 `security_decision_committee` Tool 对外暴露。Skill 层新增一个顶层工作台 Skill 与两个立场 Skill，但实现上先以结构化合同和顶层 Tool 为主，避免一开始就把复杂多轮审批与私有工作区全量并回主仓。

**Tech Stack:** Rust、Cargo、serde、现有 CLI JSON 分发、Markdown、现有 Skill 体系

---

### Task 1: 锁定证券投决证据包合同

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\stock\mod.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\mod.rs`
- Create: `D:\Rust\Excel_Skill\src\ops\stock\security_decision_evidence_bundle.rs`
- Test: `D:\Rust\Excel_Skill\tests\security_decision_evidence_bundle_cli.rs`

**Step 1: 写失败测试**

在 `tests\security_decision_evidence_bundle_cli.rs` 新增：

- 工具目录能发现 `security_decision_evidence_bundle`
- 请求成功时返回：
  - `analysis_date`
  - `symbol`
  - `technical_context`
  - `contextual_analysis`
  - `data_gaps`
  - `evidence_hash`
- 当 fullstack 不可用时，仍能返回降级证据包

**Step 2: 运行失败测试确认缺口**

Run:

```powershell
cargo test --test security_decision_evidence_bundle_cli -- --nocapture --test-threads=1
```

Expected:

- 失败，提示工具不存在或模块未实现

**Step 3: 写最小实现**

在 `security_decision_evidence_bundle.rs` 中：

- 定义请求/响应结构
- 复用现有：
  - `technical_consultation_basic`
  - `security_analysis_contextual`
  - `security_analysis_fullstack`
- 统一整理为稳定证据包
- 生成稳定 `evidence_hash`
- 标记 `data_gaps`

**Step 4: 接入模块导出**

- 在 `src\ops\stock\mod.rs` 暴露 `security_decision_evidence_bundle`
- 在 `src\ops\mod.rs` 做兼容 re-export

**Step 5: 跑测试直到通过**

Run:

```powershell
cargo test --test security_decision_evidence_bundle_cli -- --nocapture --test-threads=1
```

Expected:

- PASS

### Task 2: 锁定证券风险闸门合同

**Files:**
- Create: `D:\Rust\Excel_Skill\src\ops\stock\security_risk_gates.rs`
- Test: `D:\Rust\Excel_Skill\tests\security_decision_evidence_bundle_cli.rs`

**Step 1: 写失败测试**

在现有测试文件追加 source-level 或 CLI-level 测试，覆盖：

- `analysis_date_gate`
- `data_completeness_gate`
- `market_alignment_gate`
- `risk_reward_gate`

至少验证三种结果：

- `Pass`
- `Warn`
- `Fail`

**Step 2: 运行失败测试**

Run:

```powershell
cargo test --test security_decision_evidence_bundle_cli security_risk_gates_ -- --nocapture --test-threads=1
```

Expected:

- 失败，提示闸门模块或字段不存在

**Step 3: 写最小实现**

在 `security_risk_gates.rs` 中：

- 定义 `SecurityRiskGateResult`
- 定义 `SecurityRiskGateStatus`
- 提供一个统一评估函数
- 先做最小 explainable 规则，不做复杂评分模型

**Step 4: 跑测试直到通过**

Run:

```powershell
cargo test --test security_decision_evidence_bundle_cli security_risk_gates_ -- --nocapture --test-threads=1
```

Expected:

- PASS

### Task 3: 锁定证券投决卡合同

**Files:**
- Create: `D:\Rust\Excel_Skill\src\ops\stock\security_decision_card.rs`
- Test: `D:\Rust\Excel_Skill\tests\security_decision_committee_cli.rs`

**Step 1: 写失败测试**

新增 `tests\security_decision_committee_cli.rs`，覆盖：

- 证据包 + 闸门结果能收敛成稳定投决卡
- 投决卡必须包含：
  - `decision_id`
  - `symbol`
  - `analysis_date`
  - `status`
  - `direction`
  - `confidence_score`
  - `bull_case`
  - `bear_case`
  - `gate_results`
  - `final_recommendation`

**Step 2: 运行失败测试**

Run:

```powershell
cargo test --test security_decision_committee_cli -- --nocapture --test-threads=1
```

Expected:

- 失败，提示投决卡或 committee 工具不存在

**Step 3: 写最小实现**

在 `security_decision_card.rs` 中：

- 定义投决卡结构
- 提供一个 builder，接收：
  - 证据包
  - 多头观点
  - 空头观点
  - 闸门结果
- 统一产出状态：
  - `ReadyForReview`
  - `Blocked`
  - `NeedsMoreEvidence`

**Step 4: 跑测试直到通过**

Run:

```powershell
cargo test --test security_decision_committee_cli -- --nocapture --test-threads=1
```

Expected:

- 仍有部分失败，但投决卡结构已存在

### Task 4: 接入证券投决会顶层 Tool

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\tools\catalog.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\dispatcher\stock_ops.rs`
- Create or Modify: `D:\Rust\Excel_Skill\src\ops\stock\security_decision_committee.rs`
- Test: `D:\Rust\Excel_Skill\tests\security_decision_committee_cli.rs`

**Step 1: 写失败测试**

在 `security_decision_committee_cli.rs` 中锁定：

- 工具目录能发现 `security_decision_committee`
- 请求后会返回：
  - `evidence_bundle`
  - `bull_case`
  - `bear_case`
  - `gate_results`
  - `decision_card`

**Step 2: 运行失败测试**

Run:

```powershell
cargo test --test security_decision_committee_cli tool_catalog_includes_security_decision_committee -- --nocapture --test-threads=1
```

Expected:

- FAIL

**Step 3: 写最小实现**

实现 `security_decision_committee`：

- 调用 `security_decision_evidence_bundle`
- 先用规则型/模板型最小实现生成 `bull_case`
- 再用规则型/模板型最小实现生成 `bear_case`
- 调用 `security_risk_gates`
- 调用 `security_decision_card`

说明：

- V1 先把“正反方结构独立”落地到合同层
- 不在第一轮就做复杂多轮辩论引擎

**Step 4: 接入目录与分发**

- `catalog.rs` 注册新工具名
- `stock_ops.rs` 新增 dispatcher 入口

**Step 5: 跑测试直到通过**

Run:

```powershell
cargo test --test security_decision_committee_cli -- --nocapture --test-threads=1
```

Expected:

- PASS

### Task 5: 新增投决会 Skill 文档

**Files:**
- Create: `D:\Rust\Excel_Skill\skills\security-decision-workbench-v1\SKILL.md`
- Create: `D:\Rust\Excel_Skill\skills\security-bull-thesis-v1\SKILL.md`
- Create: `D:\Rust\Excel_Skill\skills\security-bear-challenge-v1\SKILL.md`

**Step 1: 写 Skill 文档**

分别定义：

- 顶层工作台 Skill 的流程与输出约束
- 多头 Skill 的支持论证边界
- 空头 Skill 的反方挑战边界

**Step 2: 手工检查与仓库对齐**

检查：

- 不绕开现有证券分析 Tool 主链
- 明确要求基于统一 `evidence_bundle`
- 明确区分事实、推断、风险

### Task 6: 做最小集成验证

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\integration_tool_contract.rs`
- Test: `D:\Rust\Excel_Skill\tests\security_decision_evidence_bundle_cli.rs`
- Test: `D:\Rust\Excel_Skill\tests\security_decision_committee_cli.rs`

**Step 1: 补工具合同验证**

确认：

- 新工具在目录中可见
- JSON 输出字段稳定

**Step 2: 运行定向测试**

Run:

```powershell
cargo test --test security_decision_evidence_bundle_cli -- --nocapture --test-threads=1
cargo test --test security_decision_committee_cli -- --nocapture --test-threads=1
cargo test --test integration_tool_contract -- --nocapture --test-threads=1
```

Expected:

- 全部 PASS

**Step 3: 视情况运行格式化**

Run:

```powershell
cargo fmt --all
```

### Task 7: 更新交接与任务记录

**Files:**
- Modify: `D:\Rust\Excel_Skill\progress.md`
- Modify: `D:\Rust\Excel_Skill\findings.md`
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: 更新 progress**

记录：

- 新增证券投决会 v1 最小闭环
- 当前仍是主仓内最小实现，不等于全量审批系统

**Step 2: 更新 findings**

记录：

- 证券研究链与投决链已分层
- 当前双立场仍为主仓内最小结构，不等于私有多签审批全量并入

**Step 3: 追加任务日志**

在 `.trae\CHANGELOG_TASK.md` 记录本次任务完成情况

## 完成条件

完成以下内容即可收尾：

1. 设计文档已保存
2. 实现计划已保存
3. `security_decision_evidence_bundle` 已落地并测试通过
4. `security_risk_gates` 已落地并测试通过
5. `security_decision_card` 已落地
6. `security_decision_committee` 已接入目录与分发
7. 3 个 Skill 文档已创建
8. 定向验证通过
9. 任务日志已更新
