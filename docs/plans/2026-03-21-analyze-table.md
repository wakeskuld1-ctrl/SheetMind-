# Analyze Table Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 新增 `analyze_table` Tool，以数据质量诊断为主、少量业务统计为辅，为分析建模层 V1 提供双层输出桥接能力。

**Architecture:** 在 `src/ops/analyze.rs` 中实现规则诊断 + 轻量统计增强引擎，直接复用 `summarize_table` 与 DataFrame 扫描结果产出结构化 finding，再由 Tool 层拼装用户可读的中文总结。保持 Skill 只编排、Rust Tool 负责计算的边界不变，不通过 Tool 套 Tool 的方式复用能力。

**Tech Stack:** Rust、polars、serde、serde_json、thiserror、assert_cmd、calamine；测试夹具已静态落地，任何外部脚本工具都只属于研发辅助，不属于客户运行依赖。

---

### Task 1: 定义 analyze_table 的输出契约

**Files:**
- Modify: `D:/Rust/Excel_Skill/src/tools/contracts.rs`
- Modify: `D:/Rust/Excel_Skill/src/lib.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`

**Step 1: Write the failing test**
- 在 `integration_cli_json.rs` 新增 `analyze_table_returns_dual_layer_payload`。
- 断言返回包含：
  - `row_count`
  - `column_count`
  - `table_health`
  - `structured_findings`
  - `human_summary`

**Step 2: Run test to verify it fails**

Run: `cargo test --test integration_cli_json analyze_table_returns_dual_layer_payload -v`
Expected: FAIL，因为 `analyze_table` Tool 尚不存在。

**Step 3: Write minimal implementation**
- 在 `contracts.rs` 中补齐 Tool catalog。
- 在 `lib.rs` 接线新模块导出。
- 先让 `dispatcher` 能识别 `analyze_table`，即使返回最小占位数据也行。

**Step 4: Run test to verify it passes**

Run: `cargo test --test integration_cli_json analyze_table_returns_dual_layer_payload -v`
Expected: PASS

### Task 2: 落 analyze_table 骨架并复用 summarize_table

**Files:**
- Create: `D:/Rust/Excel_Skill/src/ops/analyze.rs`
- Modify: `D:/Rust/Excel_Skill/src/ops/mod.rs`
- Modify: `D:/Rust/Excel_Skill/src/tools/dispatcher.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_frame.rs`

**Step 1: Write the failing test**
- 在 `integration_frame.rs` 新增 `analyze_table_builds_findings_from_summary_profiles`。
- 断言 analyze 结果至少能根据 `missing_rate` 和 `distinct_count` 生成基础 finding。

**Step 2: Run test to verify it fails**

Run: `cargo test --test integration_frame analyze_table_builds_findings_from_summary_profiles -v`
Expected: FAIL，因为 `analyze.rs` 尚不存在。

**Step 3: Write minimal implementation**
- 在 `analyze.rs` 中定义：
  - `AnalyzeFinding`
  - `TableHealth`
  - `HumanSummary`
  - `AnalyzeResult`
- 先通过复用 `summarize_table` 生成：
  - `all_missing`
  - `high_missing_rate`
  - `single_value_column`

**Step 4: Run test to verify it passes**

Run: `cargo test --test integration_frame analyze_table_builds_findings_from_summary_profiles -v`
Expected: PASS

### Task 3: 补高缺失、全空列、低信息量诊断

**Files:**
- Modify: `D:/Rust/Excel_Skill/src/ops/analyze.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_frame.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`
- Create: `D:/Rust/Excel_Skill/tests/fixtures/analyze-quality.xlsx`

**Step 1: Write the failing test**
- 新增 frame/CLI 测试，覆盖：
  - 全空列
  - 高缺失列
  - 单一取值列
- 断言 `table_health.level` 为 `warning` 或 `risky`。

**Step 2: Run test to verify it fails**

Run: `cargo test --test integration_frame analyze_table_flags_quality_risks -v`
Expected: FAIL，因为当前 finding 规则不完整。

Run: `cargo test --test integration_cli_json analyze_table_reports_quality_risks_in_human_summary -v`
Expected: FAIL，因为 CLI 还不能输出完整人类摘要。

**Step 3: Write minimal implementation**
- 固化阈值：
  - `missing_rate >= 0.30` -> 高缺失
  - `missing_rate == 1.0` -> 全空列
  - `distinct_count <= 1` -> 低信息量
- 根据 finding 聚合 `table_health`。

**Step 4: Run test to verify it passes**

Run: `cargo test --test integration_frame analyze_table_flags_quality_risks -v`
Expected: PASS

Run: `cargo test --test integration_cli_json analyze_table_reports_quality_risks_in_human_summary -v`
Expected: PASS

### Task 4: 补重复行和候选键风险诊断

**Files:**
- Modify: `D:/Rust/Excel_Skill/src/ops/analyze.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_frame.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`
- Create: `D:/Rust/Excel_Skill/tests/fixtures/analyze-keys.xlsx`

**Step 1: Write the failing test**
- 覆盖：
  - 完全重复行
  - `user_id` 重复
  - `user_id` 空值
- 断言会输出：
  - `duplicate_rows`
  - `duplicate_candidate_key`
  - `blank_candidate_key`

**Step 2: Run test to verify it fails**

Run: `cargo test --test integration_frame analyze_table_detects_duplicate_rows_and_key_risks -v`
Expected: FAIL，因为当前未做重复与候选键扫描。

**Step 3: Write minimal implementation**
- 在 `analyze.rs` 中补：
  - 行级去重计数
  - 候选键识别
  - 候选键重复/空值检测

**Step 4: Run test to verify it passes**

Run: `cargo test --test integration_frame analyze_table_detects_duplicate_rows_and_key_risks -v`
Expected: PASS

### Task 5: 补类别失衡、零值占比、异常值提示

**Files:**
- Modify: `D:/Rust/Excel_Skill/src/ops/analyze.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_frame.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`
- Create: `D:/Rust/Excel_Skill/tests/fixtures/analyze-distribution.xlsx`

**Step 1: Write the failing test**
- 覆盖：
  - 类别 top1 占比过高
  - 数值列零值占比过高
  - 数值列存在明显异常值
- 断言会输出：
  - `high_category_imbalance`
  - `high_zero_ratio`
  - `outlier_suspected`

**Step 2: Run test to verify it fails**

Run: `cargo test --test integration_frame analyze_table_detects_distribution_risks -v`
Expected: FAIL，因为当前没有分布和异常诊断逻辑。

**Step 3: Write minimal implementation**
- 文本列计算 `top1_share`。
- 数值列扫描 0 值占比。
- 样本量足够时用 IQR 生成异常值提示。

**Step 4: Run test to verify it passes**

Run: `cargo test --test integration_frame analyze_table_detects_distribution_risks -v`
Expected: PASS

### Task 6: 生成 human_summary 与 next_actions

**Files:**
- Modify: `D:/Rust/Excel_Skill/src/ops/analyze.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`

**Step 1: Write the failing test**
- 新增 `analyze_table_generates_readable_human_summary`。
- 断言 `human_summary` 包含：
  - `overall`
  - `major_issues`
  - `quick_insights`
  - `recommended_next_step`

**Step 2: Run test to verify it fails**

Run: `cargo test --test integration_cli_json analyze_table_generates_readable_human_summary -v`
Expected: FAIL，因为目前只有 finding，没有稳定中文总结。

**Step 3: Write minimal implementation**
- 按固定模板生成用户可读摘要：
  - 一句总体结论
  - 三条主要问题
  - 两条快速观察
  - 一条下一步建议

**Step 4: Run test to verify it passes**

Run: `cargo test --test integration_cli_json analyze_table_generates_readable_human_summary -v`
Expected: PASS

### Task 7: 全量回归与 release 验证

**Files:**
- Modify: `D:/Rust/Excel_Skill/.trae/CHANGELOG_TASK.md`

**Step 1: Run targeted verification**

Run: `cargo test --test integration_frame --test integration_cli_json -v`
Expected: PASS

**Step 2: Run full verification**

Run: `cargo test -v`
Expected: PASS

**Step 3: Run release build**

Run: `cargo build --release -v`
Expected: PASS

**Step 4: Run binary smoke test**

Run: `D:/Rust/Excel_Skill/target/release/excel_skill.exe`
Expected: PASS，且 `tool_catalog` 中出现 `analyze_table`

**Step 5: Journal**
- 追加 `.trae/CHANGELOG_TASK.md`，记录 `analyze_table` 的契约、诊断项、夹具与验证结果。
