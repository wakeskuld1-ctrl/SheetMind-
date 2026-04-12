# EastMoney Standard Enhancement Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为证券分析主链新增东财资金面与事件面补充能力，并加入预算池与缓存池，同时保持现有 fullstack 与 evidence bundle 的兼容契约。

**Architecture:** 新增独立 `providers/eastmoney` 与 runtime store，证券域通过 `eastmoney_enrichment` 聚合资金面/事件面；`contextual/fullstack/evidence_bundle` 只消费结构化补充对象。预算耗尽与远端异常统一降级，不打断主链。

**Tech Stack:** Rust, serde, serde_json, rusqlite, ureq, cargo test

---

### Task 1: 建立设计记录与测试入口

**Files:**
- Create: `tests/eastmoney_enrichment_cli.rs`
- Modify: `docs/plans/2026-04-11-eastmoney-standard-enhancement-design.md`
- Modify: `docs/plans/2026-04-11-eastmoney-standard-enhancement.md`

**Step 1: Write the failing test**

- 为预算池、缓存池、资金面接入写最小失败测试。

**Step 2: Run test to verify it fails**

Run: `cargo test --test eastmoney_enrichment_cli -- --nocapture`
Expected: FAIL，因为东财补充层尚不存在。

**Step 3: Write minimal implementation**

- 新增测试夹具和最小编译入口。

**Step 4: Run test to verify it passes**

Run: `cargo test --test eastmoney_enrichment_cli -- --nocapture`
Expected: PASS

### Task 2: 建立 provider 与运行时存储骨架

**Files:**
- Create: `src/providers/mod.rs`
- Create: `src/providers/eastmoney/mod.rs`
- Create: `src/providers/eastmoney/client.rs`
- Create: `src/providers/eastmoney/types.rs`
- Create: `src/providers/eastmoney/cache.rs`
- Create: `src/runtime/eastmoney_budget_store.rs`
- Create: `src/runtime/eastmoney_cache_store.rs`
- Modify: `src/lib.rs`
- Modify: `src/runtime/mod.rs`

**Step 1: Write the failing test**

- 写预算消耗、缓存命中两个测试。

**Step 2: Run test to verify it fails**

Run: `cargo test --test eastmoney_enrichment_cli budget -- --nocapture`
Expected: FAIL，因为 store/provider 尚不存在。

**Step 3: Write minimal implementation**

- 实现预算统计、缓存读取/写入最小能力。

**Step 4: Run test to verify it passes**

Run: `cargo test --test eastmoney_enrichment_cli budget -- --nocapture`
Expected: PASS

### Task 3: 建立证券域补充对象

**Files:**
- Create: `src/ops/eastmoney_enrichment.rs`
- Modify: `src/ops/stock.rs`

**Step 1: Write the failing test**

- 写资金面与事件面聚合测试。

**Step 2: Run test to verify it fails**

Run: `cargo test --test eastmoney_enrichment_cli enrichment -- --nocapture`
Expected: FAIL

**Step 3: Write minimal implementation**

- 定义 `CapitalFlowContext`、`EventContext`、`EastMoneyEnrichment`
- 聚合 provider + budget + cache

**Step 4: Run test to verify it passes**

Run: `cargo test --test eastmoney_enrichment_cli enrichment -- --nocapture`
Expected: PASS

### Task 4: 接入 contextual/fullstack/evidence bundle

**Files:**
- Modify: `src/ops/security_analysis_contextual.rs`
- Modify: `src/ops/security_analysis_fullstack.rs`
- Modify: `src/ops/security_decision_evidence_bundle.rs`

**Step 1: Write the failing test**

- 在 CLI 测试中新增预算耗尽降级与资金面接入断言。

**Step 2: Run test to verify it fails**

Run: `cargo test --test security_analysis_fullstack_cli -- --nocapture`
Expected: FAIL，因为新字段与降级逻辑尚未实现。

**Step 3: Write minimal implementation**

- `contextual` 增加资金面字段
- `fullstack` 改为消费 enrichment
- `evidence bundle` 纳入补充数据与缺口说明

**Step 4: Run test to verify it passes**

Run: `cargo test --test security_analysis_fullstack_cli -- --nocapture`
Run: `cargo test --test security_decision_evidence_bundle_cli -- --nocapture`
Expected: PASS

### Task 5: 文档、配置与回归

**Files:**
- Modify: `Cargo.toml`
- Modify: `.env.example`
- Modify: `README.md`
- Modify: `task_plan.md`
- Modify: `findings.md`
- Modify: `progress.md`
- Modify: `CHANGELOG_TASK.MD`

**Step 1: Write the failing test**

- 无独立新测试，使用回归验证。

**Step 2: Run test to verify it fails**

- 若前序未完成，此阶段不开始。

**Step 3: Write minimal implementation**

- 补充环境变量说明与使用文档
- 更新任务记录

**Step 4: Run test to verify it passes**

Run: `cargo test --test eastmoney_enrichment_cli --test security_analysis_fullstack_cli --test security_decision_evidence_bundle_cli -- --nocapture`
Expected: PASS
