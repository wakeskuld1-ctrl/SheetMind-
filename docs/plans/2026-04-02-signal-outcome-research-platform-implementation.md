# Signal Outcome Research Platform Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 构建证券信号结果研究平台，把“当前指标快照”升级为“当前快照 + forward returns + 历史相似信号表现 + briefing 投决接入”的完整研究底座。

**Architecture:** 在现有 `technical_consultation_basic`、`security_analysis_fullstack`、`security_analysis_resonance` 之上新增研究层：先把分析时点的核心指标、共振分数和执行建议落库，再回填未来 `1/3/5/10/20` 日表现，随后基于标签化信号做历史相似样本统计，并最终把结果通过统一 `security_decision_briefing` 暴露给咨询和投决会。平台层负责事实、统计与回填；Agent/Skill 只消费统一结果，不再手工拼历史类比。

**Tech Stack:** Rust、serde、SQLite runtime、现有 Tool dispatcher、cargo test、项目内 Skill 目录

---

### Task 1: 设计研究平台数据表与合同

**Files:**
- Create: `E:\TradingAgents\TradingAgents\src\runtime\signal_outcome_store.rs`
- Modify: `E:\TradingAgents\TradingAgents\src\runtime\mod.rs`
- Modify: `E:\TradingAgents\TradingAgents\src\tools\contracts.rs`
- Test: `E:\TradingAgents\TradingAgents\tests\integration_tool_contract.rs`

**Step 1: Write the failing test**

在 `tests/integration_tool_contract.rs` 新增红测，断言未来研究平台至少会暴露这些工具或合同字段：
- `record_security_signal_snapshot`
- `backfill_security_signal_outcomes`
- `study_security_signal_analogs`
- `signal_outcome_research_summary`

并为核心数据结构定义字段断言：
- snapshot：`symbol`、`snapshot_date`、`indicator_digest`、`resonance_score`、`action_bias`
- forward return：`horizon_days`、`forward_return_pct`、`max_drawdown_pct`
- analog summary：`sample_count`、`win_rate`、`avg_return_pct`、`median_return_pct`

**Step 2: Run test to verify it fails**

Run: `cargo test --test integration_tool_contract signal_outcome_platform_contract -- --nocapture`
Expected: FAIL with missing tools or missing contract fields

**Step 3: Write minimal implementation**

先在 `contracts.rs` 和新的 store 文件里定义结构和表设计，不接业务逻辑。建议至少包含：
- `security_signal_snapshots`
- `security_signal_forward_returns`
- `security_signal_tags`
- `security_signal_analog_studies`

**Step 4: Run test to verify it passes**

Run: `cargo test --test integration_tool_contract signal_outcome_platform_contract -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/runtime/signal_outcome_store.rs src/runtime/mod.rs src/tools/contracts.rs tests/integration_tool_contract.rs
git commit -m "feat: add signal outcome platform contracts and store schema"
```

### Task 2: 补快照记录入口，固定当前指标与高阶指标全量落库

**Files:**
- Create: `E:\TradingAgents\TradingAgents\src\ops\signal_outcome_research.rs`
- Modify: `E:\TradingAgents\TradingAgents\src\ops\mod.rs`
- Modify: `E:\TradingAgents\TradingAgents\src\ops\security_analysis_resonance.rs`
- Test: `E:\TradingAgents\TradingAgents\tests\security_analysis_resonance_cli.rs`

**Step 1: Write the failing test**

新增红测，调用 `record_security_signal_snapshot` 时断言会落下这些核心字段：
- 趋势组：`sma_50`、`sma_200`、`ema_10`、`adx_14`、`plus_di_14`、`minus_di_14`
- 动量组：`macd`、`macd_histogram`、`rsi_14`、`k_9`、`d_9`、`j_9`
- 波动组：`atr_14`、`boll_middle`、`boll_width_ratio_20`
- 资金组：`mfi_14`、`obv`、`volume_ratio_20`
- 均值回归组：`cci_20`、`williams_r_14`
- 高阶组：`rsrs_beta_18`、`rsrs_zscore_18_60`
- 共振组：`resonance_score`、`action_bias`

**Step 2: Run test to verify it fails**

Run: `cargo test --test security_analysis_resonance_cli record_security_signal_snapshot_persists_full_indicator_state -- --nocapture`
Expected: FAIL

**Step 3: Write minimal implementation**

新建 `signal_outcome_research.rs`：
- 复用现有 `security_analysis_resonance` 或 `security_decision_briefing` 的事实源
- 生成标准化 snapshot row
- 调用 `signal_outcome_store` 持久化

**Step 4: Run test to verify it passes**

Run: `cargo test --test security_analysis_resonance_cli record_security_signal_snapshot_persists_full_indicator_state -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ops/signal_outcome_research.rs src/ops/mod.rs src/ops/security_analysis_resonance.rs tests/security_analysis_resonance_cli.rs
git commit -m "feat: add security signal snapshot recording"
```

### Task 3: 补 forward returns 回填链路

**Files:**
- Modify: `E:\TradingAgents\TradingAgents\src\ops\signal_outcome_research.rs`
- Modify: `E:\TradingAgents\TradingAgents\src\runtime\signal_outcome_store.rs`
- Test: `E:\TradingAgents\TradingAgents\tests\security_analysis_resonance_cli.rs`

**Step 1: Write the failing test**

新增红测，准备固定行情夹具后：
- 先记录某个 snapshot
- 再调用 `backfill_security_signal_outcomes`
- 断言会写入 `1/3/5/10/20` 日 forward returns
- 断言每个 horizon 至少有：
  - `forward_return_pct`
  - `max_drawdown_pct`
  - `max_runup_pct`

**Step 2: Run test to verify it fails**

Run: `cargo test --test security_analysis_resonance_cli backfill_security_signal_outcomes_records_forward_metrics -- --nocapture`
Expected: FAIL

**Step 3: Write minimal implementation**

在研究 op 里：
- 根据 snapshot_date 之后的历史行情计算未来窗口收益
- 计算窗口内最大回撤与最大上冲
- 写入 forward return 表

**Step 4: Run test to verify it passes**

Run: `cargo test --test security_analysis_resonance_cli backfill_security_signal_outcomes_records_forward_metrics -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ops/signal_outcome_research.rs src/runtime/signal_outcome_store.rs tests/security_analysis_resonance_cli.rs
git commit -m "feat: add forward return backfill for signal snapshots"
```

### Task 4: 建立相似信号标签体系

**Files:**
- Modify: `E:\TradingAgents\TradingAgents\src\ops\signal_outcome_research.rs`
- Modify: `E:\TradingAgents\TradingAgents\src\runtime\signal_outcome_store.rs`
- Test: `E:\TradingAgents\TradingAgents\tests\security_analysis_resonance_cli.rs`

**Step 1: Write the failing test**

新增红测，断言 snapshot 会自动被打上标签，至少包含：
- 趋势标签：`trend_strong_bullish` / `trend_weak_sideways` 等
- 动量标签：`momentum_positive` / `momentum_negative`
- RSRS 标签：`rsrs_confirmed` / `rsrs_unconfirmed` / `rsrs_pressure`
- 资金标签：`flow_overheated` / `flow_neutral` / `flow_weak`
- 共振标签：`resonance_high` / `resonance_medium` / `resonance_low`
- 风险标签：`overbought_pullback_risk` / `range_bound_risk`

**Step 2: Run test to verify it fails**

Run: `cargo test --test security_analysis_resonance_cli signal_snapshot_derives_research_tags -- --nocapture`
Expected: FAIL

**Step 3: Write minimal implementation**

实现标签规则，只用当前已落地指标，不新增外部依赖：
- ADX/DI 生成趋势标签
- MACD/RSI/KDJ 生成动量标签
- RSRS beta / zscore 生成斜率标签
- MFI/OBV/volume ratio 生成资金标签
- resonance_score / action_bias 生成共振标签

**Step 4: Run test to verify it passes**

Run: `cargo test --test security_analysis_resonance_cli signal_snapshot_derives_research_tags -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ops/signal_outcome_research.rs src/runtime/signal_outcome_store.rs tests/security_analysis_resonance_cli.rs
git commit -m "feat: derive signal research tags"
```

### Task 5: 实现历史相似信号研究摘要

**Files:**
- Modify: `E:\TradingAgents\TradingAgents\src\ops\signal_outcome_research.rs`
- Modify: `E:\TradingAgents\TradingAgents\src\runtime\signal_outcome_store.rs`
- Test: `E:\TradingAgents\TradingAgents\tests\security_analysis_resonance_cli.rs`

**Step 1: Write the failing test**

新增红测，给定一个 snapshot：
- 调用 `study_security_signal_analogs`
- 断言会按标签和结构找到历史类似样本
- 输出：
  - `sample_count`
  - `win_rate_5d`
  - `win_rate_10d`
  - `avg_return_5d`
  - `median_return_10d`
  - `avg_max_drawdown_10d`
  - `best_matching_tags`

**Step 2: Run test to verify it fails**

Run: `cargo test --test security_analysis_resonance_cli study_security_signal_analogs_summarizes_forward_stats -- --nocapture`
Expected: FAIL

**Step 3: Write minimal implementation**

按 snapshot tags 做相似检索：
- 第一版先做“标签重合度 + 同标的同市场筛选”
- 聚合 forward return 统计
- 生成 analog summary

**Step 4: Run test to verify it passes**

Run: `cargo test --test security_analysis_resonance_cli study_security_signal_analogs_summarizes_forward_stats -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ops/signal_outcome_research.rs src/runtime/signal_outcome_store.rs tests/security_analysis_resonance_cli.rs
git commit -m "feat: add signal analog outcome study"
```

### Task 6: 增加统一研究摘要 Tool，并接入 briefing

**Files:**
- Modify: `E:\TradingAgents\TradingAgents\src\ops\security_decision_briefing.rs`
- Modify: `E:\TradingAgents\TradingAgents\src\tools\catalog.rs`
- Modify: `E:\TradingAgents\TradingAgents\src\tools\dispatcher.rs`
- Modify: `E:\TradingAgents\TradingAgents\src\tools\dispatcher\stock_ops.rs`
- Test: `E:\TradingAgents\TradingAgents\tests\integration_tool_contract.rs`
- Test: `E:\TradingAgents\TradingAgents\tests\security_analysis_resonance_cli.rs`

**Step 1: Write the failing test**

新增两类红测：
- `signal_outcome_research_summary` 在 tool catalog 中可发现、可调用
- `security_decision_briefing` 新增：
  - `historical_analog_brief`
  - `forward_return_stats`
  - `evidence_strength`

**Step 2: Run test to verify it fails**

Run:
- `cargo test --test integration_tool_contract signal_outcome_research_tools_are_cataloged -- --nocapture`
- `cargo test --test security_analysis_resonance_cli security_decision_briefing_exposes_historical_analog_layer -- --nocapture`
Expected: FAIL

**Step 3: Write minimal implementation**

接入新 Tool：
- `record_security_signal_snapshot`
- `backfill_security_signal_outcomes`
- `study_security_signal_analogs`
- `signal_outcome_research_summary`

并让 `security_decision_briefing` 直接消费 analog summary 与 forward stats。

**Step 4: Run test to verify it passes**

Run:
- `cargo test --test integration_tool_contract signal_outcome_research_tools_are_cataloged -- --nocapture`
- `cargo test --test security_analysis_resonance_cli security_decision_briefing_exposes_historical_analog_layer -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ops/security_decision_briefing.rs src/tools/catalog.rs src/tools/dispatcher.rs src/tools/dispatcher/stock_ops.rs tests/integration_tool_contract.rs tests/security_analysis_resonance_cli.rs
git commit -m "feat: integrate signal outcome research into briefing"
```

### Task 7: 为投决会准备 committee 研究输入

**Files:**
- Modify: `E:\TradingAgents\TradingAgents\src\ops\security_decision_briefing.rs`
- Modify: `E:\TradingAgents\TradingAgents\src\tools\contracts.rs`
- Test: `E:\TradingAgents\TradingAgents\tests\security_analysis_resonance_cli.rs`

**Step 1: Write the failing test**

新增红测，断言 committee payload 除了原有事实字段，还必须新增：
- `historical_confidence`
- `analog_sample_count`
- `analog_win_rate_10d`
- `expected_return_window`
- `expected_drawdown_window`
- `research_limitations`

**Step 2: Run test to verify it fails**

Run: `cargo test --test security_analysis_resonance_cli committee_payload_includes_research_confidence_fields -- --nocapture`
Expected: FAIL

**Step 3: Write minimal implementation**

把历史研究层结果映射到 committee payload：
- 让投决 Agent 看到同一份研究证据
- 明确哪些结论是“历史验证强”，哪些只是“当前快照强”

**Step 4: Run test to verify it passes**

Run: `cargo test --test security_analysis_resonance_cli committee_payload_includes_research_confidence_fields -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ops/security_decision_briefing.rs src/tools/contracts.rs tests/security_analysis_resonance_cli.rs
git commit -m "feat: add research confidence to committee payload"
```

### Task 8: Skill 门禁升级，强制咨询与投决会共用研究底稿

**Files:**
- Modify: `E:\TradingAgents\TradingAgents\skills\security-analysis-v1\SKILL.md`
- Modify: `E:\TradingAgents\TradingAgents\skills\security-decision-briefing-v1\SKILL.md`
- Optional Create: `E:\TradingAgents\TradingAgents\skills\security-committee-v1\SKILL.md`
- Test: manual skill review

**Step 1: Write the failing test**

先写 Skill 行为验收标准：
- 证券咨询先走 `security_decision_briefing`
- 如果 briefing 已有 `historical_analog_brief`，上层解释必须优先复用
- 投决会 Agent 只能在 committee payload 上表决
- 不允许 Skill 绕过 Tool 手工编写“历史上大概率如何”的结论

**Step 2: Run test to verify it fails**

Expected: 当前技能链还没有研究层门禁

**Step 3: Write minimal implementation**

更新 Skill：
- 把研究平台作为 briefing 的一部分
- 明确“历史 analog 统计优先于口头经验判断”
- 强制咨询与投决会共用同一研究底稿

**Step 4: Run test to verify it passes**

Manual Review:
- 检查是否写明统一入口
- 检查是否禁止双口径
- 检查是否禁止绕过研究 Tool 自由发挥

**Step 5: Commit**

```bash
git add skills/security-analysis-v1/SKILL.md skills/security-decision-briefing-v1/SKILL.md skills/security-committee-v1/SKILL.md
git commit -m "feat: gate security research workflow through briefing skills"
```

### Task 9: 回归验证、任务日志与研究边界说明

**Files:**
- Modify: `E:\TradingAgents\TradingAgents\CHANGELOG_TASK.MD`
- Optional Modify: `E:\TradingAgents\TradingAgents\docs\plans\2026-04-02-signal-outcome-research-platform-implementation.md`

**Step 1: Write the failing test**

无新增红测，本任务为收口。

**Step 2: Run test to verify current state**

Run:
- `cargo test --test security_analysis_resonance_cli -- --nocapture`
- `cargo test --test integration_tool_contract -- --nocapture`
- `cargo fmt`

Expected: all PASS

**Step 3: Write minimal implementation**

按 `task-journal` 要求记录：
- 研究平台修改内容
- 为什么要做 forward returns 与 analog study
- 未完成项
- 风险边界
- 验证结果
- 记忆点

**Step 4: Run final verification**

确认：
- 研究平台 Tool 可调用
- briefing 已暴露 analog 研究层
- committee payload 已接入研究信心字段
- Skill 门禁已生效

**Step 5: Commit**

```bash
git add CHANGELOG_TASK.MD docs/plans/2026-04-02-signal-outcome-research-platform-implementation.md
git commit -m "docs: add signal outcome research platform plan"
```

---

## Design Notes

- 方案 C 虽然是平台化全做，但实现顺序仍然必须 TDD，先合同、再快照、再回填、再 analog、最后接 briefing。
- `历史相似信号` 第一版不做复杂机器学习相似度，先做标签重合与结构分组，保持可解释。
- `forward returns` 第一版固定 `1/3/5/10/20` 日，避免一开始把 horizon 做成过度泛化配置。
- `research_limitations` 必须进 committee payload，明确告诉投决会哪些结论是样本不足、哪些是数据缺失。
- 研究层是事实与统计层，不替代最终投决；投决会 Agent 只能基于统一事实投票。

## Suggested Phase Order

1. 先做 snapshot 与 forward returns
2. 再做 analog summary
3. 再接 briefing 与 committee payload
4. 最后补 Skill 门禁和 vote Tool
