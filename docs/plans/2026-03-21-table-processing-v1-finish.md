# Table Processing V1 Finish Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 完成表处理层 V1 的最后收尾，包括统计摘要质量指标、复杂摘要场景、显性关联边界覆盖，以及单二进制交付验证。

**Architecture:** 继续保持 Skill 只编排、Rust Tool 负责真实计算的边界不变。本轮先在 `summarize_table` 内补 `missing_rate` 和复杂数据画像，再通过集成测试加固 `join_tables` 的显性关联边界，最后通过 `release` 构建和 CLI 冒烟验证确认“弱 IT 用户可直接拿二进制使用”的交付形态。

**Tech Stack:** Rust、calamine、polars、serde、serde_json、thiserror、assert_cmd；测试夹具已静态落地，外部脚本只允许作为研发辅助，不属于客户运行依赖。

---

### Task 1: 为 summarize_table 增加 missing_rate

**Files:**
- Modify: `D:/Rust/Excel_Skill/tests/integration_frame.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`
- Modify: `D:/Rust/Excel_Skill/src/ops/summary.rs`

**Step 1: Write the failing test**
- 在 `integration_frame.rs` 为数值列、文本列、布尔列补 `missing_rate` 断言。
- 在 `integration_cli_json.rs` 为 `summarize_table` Tool 返回值补 `missing_rate` 断言。

**Step 2: Run test to verify it fails**
Run: `cargo test --test integration_frame summarize_table_reports_numeric_and_text_metrics -v`
Expected: FAIL，提示缺少 `missing_rate` 字段或断言不成立。

Run: `cargo test --test integration_cli_json summarize_table_returns_column_profiles -v`
Expected: FAIL，提示 JSON 中没有 `missing_rate`。

**Step 3: Write minimal implementation**
- 在 `ColumnSummary` 增加 `missing_rate: Option<f64>`。
- 统一按 `null_count / 总行数` 计算缺失率；总行数为 0 时返回 `None`。
- 保持原有字段和 JSON 结构兼容。

**Step 4: Run test to verify it passes**
Run: `cargo test --test integration_frame summarize_table_reports_numeric_and_text_metrics -v`
Expected: PASS

Run: `cargo test --test integration_cli_json summarize_table_returns_column_profiles -v`
Expected: PASS

### Task 2: 补统计摘要复杂场景测试与最小实现

**Files:**
- Modify: `D:/Rust/Excel_Skill/tests/integration_frame.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`
- Modify: `D:/Rust/Excel_Skill/src/ops/summary.rs`
- Create: `D:/Rust/Excel_Skill/tests/fixtures/summary-dates.xlsx`
- Create: `D:/Rust/Excel_Skill/tests/fixtures/summary-mixed-dirty.xlsx`
- Create: `D:/Rust/Excel_Skill/tests/fixtures/summary-wide.xlsx`

**Step 1: Write the failing test**
- 日期列：验证不会崩溃，并输出稳定的 `count/null_count/distinct_count/top_values/missing_rate`。
- 混合脏数据列：验证 `1`、`1.2`、`N/A`、空格、`abc` 等混合输入时摘要稳定。
- 超宽表：验证几十列以上时能返回完整列摘要数量且不遗漏尾部列。

**Step 2: Run test to verify it fails**
Run: `cargo test --test integration_frame summarize_table_handles_date_and_dirty_columns_stably -v`
Expected: FAIL，当前缺少测试夹具或摘要字段/类型识别不满足断言。

Run: `cargo test --test integration_cli_json summarize_table_summarizes_wide_sheet_without_losing_columns -v`
Expected: FAIL，当前没有对应夹具或输出不满足断言。

**Step 3: Write minimal implementation**
- 仅在必要时扩展 `dtype_label`，让日期/日期时间列不要落到含糊的 `unknown`。
- 保持日期列先按离散值摘要处理，不引入额外建模级语义。
- 如无实现缺陷，则只保留测试和夹具，不扩大代码面。

**Step 4: Run test to verify it passes**
Run: `cargo test --test integration_frame summarize_table_handles_date_and_dirty_columns_stably -v`
Expected: PASS

Run: `cargo test --test integration_cli_json summarize_table_summarizes_wide_sheet_without_losing_columns -v`
Expected: PASS

### Task 3: 补 join_tables 边界测试与最小修正

**Files:**
- Modify: `D:/Rust/Excel_Skill/tests/integration_frame.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`
- Modify: `D:/Rust/Excel_Skill/src/ops/join.rs`
- Create: `D:/Rust/Excel_Skill/tests/fixtures/join-empty-keys.xlsx`
- Create: `D:/Rust/Excel_Skill/tests/fixtures/join-duplicate-orders.xlsx`
- Create: `D:/Rust/Excel_Skill/tests/fixtures/join-conflict-columns.xlsx`

**Step 1: Write the failing test**
- 空 key：确认空字符串键在 `matched_only` 下不会误匹配。
- 重复 key：确认左 1 x 右 N 会展开成多条结果。
- 同名非键列：确认结果列名连续冲突时稳定追加 `_right`。

**Step 2: Run test to verify it fails**
Run: `cargo test --test integration_frame join_tables_ignores_blank_keys_and_renames_conflicting_columns -v`
Expected: FAIL，暴露当前空 key 误匹配或列名冲突问题。

Run: `cargo test --test integration_cli_json join_tables_expands_duplicate_matches_and_keeps_stable_columns -v`
Expected: FAIL，暴露当前 CLI 返回结果与预期不符。

**Step 3: Write minimal implementation**
- 如果空 key 目前会匹配，则在索引构建和左侧查找时把空白键视为“不可关联”。
- 保持多对多展开语义不变，只修正与业务直觉冲突的边界。
- 如同名冲突命名已正确，则不改生产代码，只保留测试。

**Step 4: Run test to verify it passes**
Run: `cargo test --test integration_frame join_tables_ignores_blank_keys_and_renames_conflicting_columns -v`
Expected: PASS

Run: `cargo test --test integration_cli_json join_tables_expands_duplicate_matches_and_keeps_stable_columns -v`
Expected: PASS

### Task 4: 做交付验证并记录任务日志

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
Expected: PASS，产出 `D:/Rust/Excel_Skill/target/release/excel_skill.exe`

**Step 4: Run binary smoke test**
Run: `./target/release/excel_skill.exe`
Expected: PASS，输出 JSON `tool_catalog`

**Step 5: Journal**
- 按 `task-journal` 规范追加 `.trae/CHANGELOG_TASK.md`，记录 `missing_rate`、复杂摘要测试、join 边界测试和 release 交付验证。
