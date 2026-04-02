# Security Decision Briefing Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 构建统一的 `security_decision_briefing` Tool，并为 `security_committee_vote` 预留稳定输入合同，同时新增 Skill 门禁，确保咨询场景与投决会场景读取完全一致的证券分析底稿。

**Architecture:** 以现有 `technical_consultation_basic`、`security_analysis_contextual`、`security_analysis_fullstack`、`security_analysis_resonance` 为底层事实源，在 `ops` 层新增一个统一 assembler，将基本面、技术面、共振面、交易执行面拼成单一 briefing 结构。`security_committee_vote` 第一阶段只定义合同和数据入口，不先实现表决引擎；Skill 层作为基础门禁，强制所有证券咨询与投决会流程先走 briefing Tool，再允许上层 Agent 做判断。

**Tech Stack:** Rust、serde、现有 CLI Tool dispatcher、SQLite runtime、项目内 Skill 目录、cargo test

---

### Task 1: 明确 briefing 与 committee 的输出合同

**Files:**
- Modify: `E:\TradingAgents\TradingAgents\src\ops\security_analysis_resonance.rs`
- Modify: `E:\TradingAgents\TradingAgents\src\tools\contracts.rs`
- Test: `E:\TradingAgents\TradingAgents\tests\integration_tool_contract.rs`

**Step 1: Write the failing test**

在 `tests/integration_tool_contract.rs` 增加合同红测，断言未来的 `security_decision_briefing` 至少暴露以下顶层字段：
- `summary`
- `fundamental_brief`
- `technical_brief`
- `resonance_brief`
- `execution_plan`
- `committee_payload`

并断言 `committee_payload` 中至少有：
- `recommended_action`
- `confidence`
- `key_risks`
- `evidence_version`

**Step 2: Run test to verify it fails**

Run: `cargo test --test integration_tool_contract security_decision_briefing_contract -- --nocapture`
Expected: FAIL with “tool missing” or “field missing”

**Step 3: Write minimal implementation**

在 `src/ops` 和 `src/tools/contracts.rs` 里先定义 request/response struct，不接业务逻辑，只保证合同成型。

**Step 4: Run test to verify it passes**

Run: `cargo test --test integration_tool_contract security_decision_briefing_contract -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ops/security_analysis_resonance.rs src/tools/contracts.rs tests/integration_tool_contract.rs
git commit -m "feat: add security decision briefing contract"
```

### Task 2: 把现有证券分析事实源收敛成统一 briefing assembler

**Files:**
- Modify: `E:\TradingAgents\TradingAgents\src\ops\security_analysis_resonance.rs`
- Create: `E:\TradingAgents\TradingAgents\src\ops\security_decision_briefing.rs`
- Modify: `E:\TradingAgents\TradingAgents\src\ops\mod.rs`
- Test: `E:\TradingAgents\TradingAgents\tests\security_analysis_resonance_cli.rs`

**Step 1: Write the failing test**

在 `tests/security_analysis_resonance_cli.rs` 新增红测：
- 调用 `security_decision_briefing`
- 断言会复用现有 fullstack/resonance 输出
- 断言技术层必须带全量核心指标字段：
  - `adx_14`
  - `macd`
  - `rsi_14`
  - `mfi_14`
  - `obv`
  - `cci_20`
  - `williams_r_14`
  - `rsrs_beta_18`
  - `rsrs_zscore_18_60`
- 断言共振层至少带：
  - `resonance_score`
  - `action_bias`
  - `top_positive_resonances`
  - `top_negative_resonances`

**Step 2: Run test to verify it fails**

Run: `cargo test --test security_analysis_resonance_cli security_decision_briefing_exposes_full_indicator_and_resonance_layers -- --nocapture`
Expected: FAIL because tool/fields do not exist

**Step 3: Write minimal implementation**

创建 `src/ops/security_decision_briefing.rs`：
- 调用现有 `security_analysis_resonance`
- 从其返回值中提取：
  - 基本面层
  - 公告层
  - 技术面层
  - 共振层
- 增加统一 summary 字段
- 增加 `evidence_version`，用于后续投决会多 Agent 识别同一份底稿

**Step 4: Run test to verify it passes**

Run: `cargo test --test security_analysis_resonance_cli security_decision_briefing_exposes_full_indicator_and_resonance_layers -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ops/security_decision_briefing.rs src/ops/mod.rs src/ops/security_analysis_resonance.rs tests/security_analysis_resonance_cli.rs
git commit -m "feat: add unified security decision briefing assembler"
```

### Task 3: 为交易执行层补硬阈值输出

**Files:**
- Modify: `E:\TradingAgents\TradingAgents\src\ops\security_decision_briefing.rs`
- Modify: `E:\TradingAgents\TradingAgents\src\ops\security_analysis_resonance.rs`
- Test: `E:\TradingAgents\TradingAgents\tests\security_analysis_resonance_cli.rs`

**Step 1: Write the failing test**

新增红测，断言 `execution_plan` 明确输出：
- `add_trigger_price`
- `add_trigger_volume_ratio`
- `add_position_pct`
- `reduce_trigger_price`
- `rejection_zone`
- `reduce_position_pct`
- `stop_loss_price`
- `invalidation_price`
- `watch_points`

并断言这些值来自真实指标，不允许为空字符串。

**Step 2: Run test to verify it fails**

Run: `cargo test --test security_analysis_resonance_cli security_decision_briefing_exposes_execution_thresholds -- --nocapture`
Expected: FAIL

**Step 3: Write minimal implementation**

在 assembler 中根据现有技术指标生成交易阈值：
- 阻力位优先取 `resistance_level_20`
- 短承接位优先取 `ema_10`
- 强弱分界优先取 `boll_middle`
- 趋势失效位优先取 `sma_50`
- 放量标准优先基于 `volume_ratio_20`

同时明确把“这些阈值来自哪些指标”写入 explanation 字段，避免结论悬空。

**Step 4: Run test to verify it passes**

Run: `cargo test --test security_analysis_resonance_cli security_decision_briefing_exposes_execution_thresholds -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ops/security_decision_briefing.rs src/ops/security_analysis_resonance.rs tests/security_analysis_resonance_cli.rs
git commit -m "feat: add execution thresholds to security decision briefing"
```

### Task 4: 把新 Tool 接入 catalog / dispatcher

**Files:**
- Modify: `E:\TradingAgents\TradingAgents\src\tools\catalog.rs`
- Modify: `E:\TradingAgents\TradingAgents\src\tools\dispatcher.rs`
- Modify: `E:\TradingAgents\TradingAgents\src\tools\dispatcher\stock_ops.rs`
- Test: `E:\TradingAgents\TradingAgents\tests\integration_tool_contract.rs`

**Step 1: Write the failing test**

新增红测，断言：
- `tool_catalog` 包含 `security_decision_briefing`
- `stock` tool group 包含 `security_decision_briefing`

**Step 2: Run test to verify it fails**

Run: `cargo test --test integration_tool_contract security_decision_briefing_is_cataloged -- --nocapture`
Expected: FAIL

**Step 3: Write minimal implementation**

把新 Tool 加入：
- `catalog.rs`
- `dispatcher.rs`
- `dispatcher/stock_ops.rs`

**Step 4: Run test to verify it passes**

Run: `cargo test --test integration_tool_contract security_decision_briefing_is_cataloged -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/tools/catalog.rs src/tools/dispatcher.rs src/tools/dispatcher/stock_ops.rs tests/integration_tool_contract.rs
git commit -m "feat: register security decision briefing tool"
```

### Task 5: 为投决会预留 committee payload 合同

**Files:**
- Modify: `E:\TradingAgents\TradingAgents\src\ops\security_decision_briefing.rs`
- Modify: `E:\TradingAgents\TradingAgents\src\tools\contracts.rs`
- Test: `E:\TradingAgents\TradingAgents\tests\security_analysis_resonance_cli.rs`

**Step 1: Write the failing test**

新增红测，断言 `committee_payload` 至少带：
- `symbol`
- `analysis_date`
- `recommended_action`
- `confidence`
- `key_risks`
- `minority_objection_points`
- `evidence_version`
- `briefing_digest`

这里暂不要求实现 `security_committee_vote`，只要求 committee 端拿到稳定底稿。

**Step 2: Run test to verify it fails**

Run: `cargo test --test security_analysis_resonance_cli security_decision_briefing_exposes_committee_payload -- --nocapture`
Expected: FAIL

**Step 3: Write minimal implementation**

在 briefing 响应中补 committee 专用 payload，保证：
- 咨询场景和投决会场景看到的是同一份 factual payload
- 上层 Agent 只能在这个 payload 上投票，而不是额外拼一套事实

**Step 4: Run test to verify it passes**

Run: `cargo test --test security_analysis_resonance_cli security_decision_briefing_exposes_committee_payload -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ops/security_decision_briefing.rs src/tools/contracts.rs tests/security_analysis_resonance_cli.rs
git commit -m "feat: add committee payload to security decision briefing"
```

### Task 6: 新增 Skill 门禁，强制证券分析与投决会统一入口

**Files:**
- Create: `E:\TradingAgents\TradingAgents\skills\security-decision-briefing-v1\SKILL.md`
- Modify: `E:\TradingAgents\TradingAgents\skills\security-analysis-v1\SKILL.md`
- Test: manual skill review

**Step 1: Write the failing test**

这里不写自动化测试，改为先写 Skill 行为验收标准：
- 证券咨询请求优先走 `security_decision_briefing`
- 投决会场景必须先生成 `committee_payload`
- Skill 不允许绕过 briefing Tool 手工拼事实
- Skill 要明确区分“Tool 事实”和“Agent 判断”

**Step 2: Run test to verify it fails**

Expected: 当前 Skill 规范中没有这一层门禁，无法满足统一底稿要求

**Step 3: Write minimal implementation**

新增 `security-decision-briefing-v1`：
- 把 briefing Tool 定为统一入口
- 把咨询模式与投决会模式写成固定流程
- 要求上层 Agent 只能基于 briefing 输出做解释和表决

并更新现有 `security-analysis-v1`，明确：
- 单票咨询优先走 briefing Tool
- fullstack / resonance 成为 briefing 底层事实源，而非最终用户接口

**Step 4: Run test to verify it passes**

Manual Review:
- 检查 Skill 是否明确写出统一入口
- 检查是否禁止双口径
- 检查是否明确 briefing 是“基础门禁”

**Step 5: Commit**

```bash
git add skills/security-decision-briefing-v1/SKILL.md skills/security-analysis-v1/SKILL.md
git commit -m "feat: add security decision briefing skill gate"
```

### Task 7: 全量回归、文档和任务日志

**Files:**
- Modify: `E:\TradingAgents\TradingAgents\CHANGELOG_TASK.MD`
- Optional: `E:\TradingAgents\TradingAgents\docs\plans\2026-04-02-security-decision-briefing-implementation.md`

**Step 1: Write the failing test**

无新增红测，本任务为收口。

**Step 2: Run test to verify current state**

Run:
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

**Step 4: Run final verification**

再次确认：
- briefing Tool 可调用
- catalog / dispatcher 已接入
- Skill 门禁已生效

**Step 5: Commit**

```bash
git add CHANGELOG_TASK.MD docs/plans/2026-04-02-security-decision-briefing-implementation.md
git commit -m "docs: record security decision briefing implementation plan"
```

---

## Design Notes

- 第一阶段不直接实现 `security_committee_vote`，只先把 `committee_payload` 合同稳定下来。
- Skill 是“基础门禁”，不是简单说明文档；它负责把咨询和投决会都路由到统一 briefing Tool。
- 技术层必须完整暴露当前已落地主链中的核心指标与高阶指标，尤其不能漏掉 `RSRS` 和共振层。
- 共振层要明确区分：
  - 已生效驱动
  - 未生效但已注册的候选因子
  - 因数据缺失而降级的解释

## Suggested Phase Order

1. 先做合同
2. 再做 assembler
3. 再补交易执行层
4. 再接入 catalog / dispatcher
5. 再加 Skill 门禁
6. 最后再做 vote Tool
