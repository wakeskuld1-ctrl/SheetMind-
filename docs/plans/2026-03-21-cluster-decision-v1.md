# 聚类与决策助手 V1 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 补齐聚类 Tool、统一分析建模层输出、实现决策助手层 V1，并完成 V1 验收。

**Architecture:** 在 `ops` 层新增聚类与决策助手模块；在公共建模输出模块中统一线性回归、逻辑回归、聚类的总览字段；在 `dispatcher` 与 `contracts` 中接入新 Tool；通过集成测试锁定内存层与 CLI JSON 契约。

**Tech Stack:** Rust 2024、Polars 0.51、Serde、现有 CLI JSON 调度架构

---

### Task 1: 落聚类失败测试

**Files:**
- Modify: `tests/integration_frame.rs`
- Modify: `tests/integration_cli_json.rs`

**Step 1: Write the failing test**
- 为 `cluster_kmeans` 增加内存层测试，覆盖：基础聚类、缺失删行、非法 K、非数值特征。
- 为 CLI 增加目录暴露测试与最小成功/失败测试。

**Step 2: Run test to verify it fails**
- Run: `cargo test cluster_kmeans -v`
- Expected: FAIL，提示函数或 Tool 不存在。

**Step 3: Write minimal implementation**
- 不在此任务实现生产代码。

**Step 4: Run test to verify it still fails for the expected reason**
- Run: `cargo test cluster_kmeans -v`
- Expected: FAIL，且失败点是缺少实现，不是测试拼写错误。

### Task 2: 实现聚类公共准备与算法

**Files:**
- Modify: `src/ops/model_prep.rs`
- Create: `src/ops/model_output.rs`
- Create: `src/ops/cluster_kmeans.rs`
- Modify: `src/ops/mod.rs`

**Step 1: Write the failing test**
- 沿用 Task 1 的失败测试。

**Step 2: Run test to verify it fails**
- Run: `cargo test cluster_kmeans -v`
- Expected: FAIL

**Step 3: Write minimal implementation**
- 在 `model_prep` 增加聚类样本准备函数。
- 在 `cluster_kmeans` 实现确定性 KMeans。
- 在 `model_output` 提供共享输出结构。
- 接到 `ops::mod`。

**Step 4: Run test to verify it passes**
- Run: `cargo test cluster_kmeans -v`
- Expected: PASS

### Task 3: 统一分析建模层输出

**Files:**
- Modify: `src/ops/linear_regression.rs`
- Modify: `src/ops/logistic_regression.rs`
- Modify: `tests/integration_frame.rs`
- Modify: `tests/integration_cli_json.rs`

**Step 1: Write the failing test**
- 为线性回归、逻辑回归、聚类补统一字段断言：`model_kind`、`problem_type`、`data_summary`、`quality_summary`。

**Step 2: Run test to verify it fails**
- Run: `cargo test linear_regression logistic_regression cluster_kmeans -v`
- Expected: FAIL

**Step 3: Write minimal implementation**
- 改三类模型结果结构，补共享字段并保持旧字段兼容。

**Step 4: Run test to verify it passes**
- Run: `cargo test linear_regression logistic_regression cluster_kmeans -v`
- Expected: PASS

### Task 4: 接入调度层与目录

**Files:**
- Modify: `src/tools/contracts.rs`
- Modify: `src/tools/dispatcher.rs`
- Modify: `tests/integration_cli_json.rs`

**Step 1: Write the failing test**
- 为 `tool_catalog` 增加 `cluster_kmeans`、`decision_assistant` 断言。
- 为 CLI 调度增加新 Tool 请求测试。

**Step 2: Run test to verify it fails**
- Run: `cargo test tool_catalog_includes_cluster_kmeans tool_catalog_includes_decision_assistant -v`
- Expected: FAIL

**Step 3: Write minimal implementation**
- 更新目录。
- 在 `dispatcher` 增加聚类与决策助手参数解析、预处理、错误透传。

**Step 4: Run test to verify it passes**
- Run: `cargo test tool_catalog_includes_cluster_kmeans tool_catalog_includes_decision_assistant -v`
- Expected: PASS

### Task 5: 落决策助手失败测试

**Files:**
- Modify: `tests/integration_frame.rs`
- Modify: `tests/integration_cli_json.rs`

**Step 1: Write the failing test**
- 覆盖：阻塞风险聚合、优先动作排序、业务亮点、下一步 Tool 建议。

**Step 2: Run test to verify it fails**
- Run: `cargo test decision_assistant -v`
- Expected: FAIL

**Step 3: Write minimal implementation**
- 不在此任务实现生产代码。

**Step 4: Run test to verify it still fails for the expected reason**
- Run: `cargo test decision_assistant -v`
- Expected: FAIL，且失败原因为缺少实现。

### Task 6: 实现决策助手层 V1

**Files:**
- Create: `src/ops/decision_assistant.rs`
- Modify: `src/ops/mod.rs`
- Modify: `src/tools/dispatcher.rs`
- Modify: `tests/integration_frame.rs`
- Modify: `tests/integration_cli_json.rs`

**Step 1: Write the failing test**
- 沿用 Task 5 的失败测试。

**Step 2: Run test to verify it fails**
- Run: `cargo test decision_assistant -v`
- Expected: FAIL

**Step 3: Write minimal implementation**
- 复用 `analyze_table` 与 `stat_summary`。
- 产出阻塞风险、优先动作、业务亮点、下一步 Tool 建议和中文摘要。

**Step 4: Run test to verify it passes**
- Run: `cargo test decision_assistant -v`
- Expected: PASS

### Task 7: UTF-8 定点收口与验收

**Files:**
- Modify: `src/tools/dispatcher.rs`
- Modify: 本轮实际触达且存在乱码的相关文件
- Modify: `.trae/CHANGELOG_TASK.md`

**Step 1: Write the failing test**
- 用现有 CLI 错误文案测试锁定中文可读提示。

**Step 2: Run test to verify it fails if needed**
- Run: `cargo test integration_cli_json -- --nocapture`
- Expected: 若有乱码导致断言失败则先红。

**Step 3: Write minimal implementation**
- 仅对本轮实际修改的文件做 UTF-8 收口，不做无关大扫除。
- 追加任务日志。

**Step 4: Run full verification**
- Run: `cargo test -v`
- Run: `cargo build --release -v`
- Expected: 全绿
