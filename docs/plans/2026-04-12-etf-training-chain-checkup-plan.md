# ETF Training Chain Checkup Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Diagnose whether the ETF training chain is ready to move from pipeline completion into precision-improvement work, and isolate the highest-value bottlenecks before any code change.

**Architecture:** This round stays read-only on product code and focuses on governed evidence. We validate the ETF mainline, inspect the training-governance chain, then map the observed gaps into a small number of precision-focused follow-up changes.

**Tech Stack:** Rust, Cargo CLI tests, JSON tool dispatcher, SQLite runtime fixtures

---

### Task 1: Verify ETF mainline and training-governance slices

**Files:**
- Inspect: `src/ops/security_external_proxy_history_import.rs`
- Inspect: `src/ops/security_feature_snapshot.rs`
- Inspect: `src/ops/security_master_scorecard.rs`
- Inspect: `src/ops/security_chair_resolution.rs`
- Inspect: `src/ops/security_scorecard_training.rs`
- Inspect: `src/ops/security_scorecard_refit_run.rs`
- Inspect: `src/ops/security_shadow_evaluation.rs`
- Inspect: `src/ops/security_model_promotion.rs`
- Test: `tests/security_external_proxy_history_import_cli.rs`
- Test: `tests/security_feature_snapshot_cli.rs`
- Test: `tests/security_master_scorecard_cli.rs`
- Test: `tests/security_chair_resolution_cli.rs`
- Test: `tests/security_scorecard_training_cli.rs`
- Test: `tests/security_scorecard_refit_cli.rs`
- Test: `tests/security_shadow_evaluation_cli.rs`
- Test: `tests/security_model_promotion_cli.rs`

**Step 1: Re-run the critical CLI slices**

Run:
```powershell
cargo test --test security_scorecard_training_cli -- --nocapture --test-threads=1
cargo test --test security_scorecard_refit_cli -- --nocapture --test-threads=1
cargo test --test security_shadow_evaluation_cli -- --nocapture --test-threads=1
cargo test --test security_model_promotion_cli -- --nocapture --test-threads=1
```

Expected:
- PASS for all targeted suites

**Step 2: Record closure evidence**

Expected evidence:
- ETF mainline is operational
- training -> refit -> shadow evaluation -> promotion chain exists and passes regression tests

### Task 2: Diagnose precision bottlenecks in the training pipeline

**Files:**
- Inspect: `src/ops/security_scorecard_training.rs`
- Inspect: `src/ops/security_decision_evidence_bundle.rs`
- Inspect: `src/ops/security_master_scorecard.rs`
- Test: `tests/security_scorecard_training_cli.rs`

**Step 1: Confirm sample sizing and sampling strategy**

Inspect:
- default sample targets
- candidate date cap
- evenly spaced date selection

Expected findings:
- defaults are still thin for ETF precision work
- sampling is governed but not yet rich enough for stronger generalization

**Step 2: Confirm readiness thresholds by head type**

Inspect:
- classification readiness gate
- regression readiness gate

Expected findings:
- classification has stronger quality gating
- regression readiness is still permissive

**Step 3: Confirm ETF training coverage depth**

Inspect:
- ETF feature-family unit tests
- end-to-end CLI coverage

Expected findings:
- ETF feature contracts are present
- ETF end-to-end training regression coverage is still weaker than desired

### Task 3: Diagnose runtime landing of trained models

**Files:**
- Inspect: `src/ops/security_master_scorecard.rs`
- Inspect: `src/ops/security_chair_resolution.rs`
- Test: `tests/security_master_scorecard_cli.rs`
- Test: `tests/security_chair_resolution_cli.rs`

**Step 1: Confirm how runtime consumes trained heads**

Inspect:
- direct model-path inputs
- multi-head availability rules

Expected findings:
- runtime currently consumes explicit model file paths
- promotion / registry outputs are not yet the default runtime selection path

**Step 2: Confirm multi-head admission rule**

Inspect:
- `head_count`
- aggregation status transitions

Expected findings:
- runtime promotes to multi-head context based mainly on availability, not full quality governance

### Task 4: Convert diagnosis into implementation options

**Files:**
- Update: `findings.md`
- Update: `progress.md`
- Update: `task_plan.md`

**Step 1: Rank the next precision improvements**

Expected priority:
1. Add ETF end-to-end training regression coverage
2. Thicken ETF sample plan and sampling strategy
3. Tighten regression-head readiness thresholds
4. Wire promotion / registry outputs into runtime model selection

**Step 2: Present multiple implementation options to the user**

Expected output:
- Option A: patch ETF training regression coverage first
- Option B: patch sample thickness and readiness gates together
- Option C: land runtime registry selection after training quality hardening
