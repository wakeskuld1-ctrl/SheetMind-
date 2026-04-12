# Security Mainline P8-P10 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** implement the remaining execution/review loop, then backfill enough data to validate it, and finally close with end-to-end verification.

**Architecture:** build three new stock lifecycle tools on top of the current approval chain, then attach a narrow verification-data backfill slice, and finally verify the whole replay path. The plan assumes the existing branch state is authoritative and avoids resurrecting outdated module layouts.

**Tech Stack:** Rust, serde/serde_json, stock tool dispatcher/catalog, runtime JSON artifact persistence, CLI integration tests, markdown docs.

---

### Phase P8: Execution And Review Loop

#### Task P8-1: Add `security_condition_review` core object

**Files:**
- Create: `src/ops/security_condition_review.rs`
- Modify: `src/ops/mod.rs`
- Modify: `src/ops/stock.rs`
- Test: `tests/security_condition_review_cli.rs`

**Steps:**
1. Write the failing CLI contract test for:
   - tool discovery
   - manual review request parsing
   - structured result with `recommended_follow_up_action`
2. Run the focused test and verify it fails for missing tool/module.
3. Implement the minimal review document, request, result, and routing logic.
4. Register the tool in catalog and dispatcher.
5. Re-run the focused test until green.

#### Task P8-2: Add `security_execution_record` formal object

**Files:**
- Create: `src/ops/security_execution_record.rs`
- Modify: `src/ops/mod.rs`
- Modify: `src/ops/stock.rs`
- Modify: `src/tools/catalog.rs`
- Modify: `src/tools/dispatcher/stock_ops.rs`
- Modify: `src/tools/dispatcher.rs`
- Test: `tests/security_execution_record_cli.rs`

**Steps:**
1. Write failing tests for:
   - tool discovery
   - execution event serialization
   - binding to `decision_ref / approval_ref / position_plan_ref`
2. Run tests and verify red.
3. Implement the minimal execution record contract and tool.
4. Wire catalog and dispatcher.
5. Re-run focused tests until green.

#### Task P8-3: Add `security_post_trade_review` formal object

**Files:**
- Create: `src/ops/security_post_trade_review.rs`
- Modify: `src/ops/mod.rs`
- Modify: `src/ops/stock.rs`
- Modify: `src/tools/catalog.rs`
- Modify: `src/tools/dispatcher/stock_ops.rs`
- Modify: `src/tools/dispatcher.rs`
- Test: `tests/security_post_trade_review_cli.rs`

**Steps:**
1. Write failing tests for:
   - tool discovery
   - layered attribution fields
   - follow-up governance action fields
2. Run tests and verify red.
3. Implement the minimal post-trade review contract and tool.
4. Wire catalog and dispatcher.
5. Re-run focused tests until green.

#### Task P8-4: Bind P8 objects into approval/package/holding chain

**Files:**
- Modify: `src/ops/security_decision_package.rs`
- Modify: `src/ops/security_decision_package_revision.rs`
- Modify: `src/ops/security_decision_submit_approval.rs`
- Modify: `docs/security-holding-ledger.md`
- Test: `tests/security_decision_package_revision_cli.rs`

**Steps:**
1. Write failing tests for package/object-graph support of:
   - `condition_review_ref`
   - `execution_record_ref`
   - `post_trade_review_ref`
2. Run focused package tests and verify red.
3. Add the smallest object-graph/governance fields needed to carry the new refs.
4. Update holding ledger format guidance to preserve the same refs.
5. Re-run focused tests until green.

#### Task P8-5: Add layered attribution and governance feedback

**Files:**
- Modify: `src/ops/security_post_trade_review.rs`
- Modify: `src/ops/security_shadow_evaluation.rs`
- Modify: `src/ops/security_model_promotion.rs`
- Test: `tests/security_post_trade_review_cli.rs`

**Steps:**
1. Write failing tests for post-trade review follow-up semantics:
   - continue shadow
   - retrain
   - downgrade
   - freeze consumption
2. Run focused tests and verify red.
3. Implement minimal structured feedback fields.
4. Re-run focused tests until green.

#### Task P8-6: Operator view closure

**Files:**
- Modify: `docs/security-holding-ledger.md`
- Modify: `docs/AI_HANDOFF.md`
- Modify: `docs/交接摘要_证券分析_给后续AI.md`

**Steps:**
1. Update the holding ledger template so every holding can preserve:
   - condition review refs
   - execution record refs
   - post-trade review refs
2. Update handoff docs with the new lifecycle order.

### Phase P9: Verification Data Backfill

#### Task P9-1: Define the verification data slice

**Files:**
- Create: `docs/plans/2026-04-12-security-verification-data-slice.md`

**Steps:**
1. Document the minimal symbols, dates, and artifact types needed for validation.
2. Keep the slice narrow and directly tied to P8 lifecycle tests.

#### Task P9-2: Backfill runtime data needed by the new lifecycle

**Files:**
- Modify: runtime outputs under `.excel_skill_runtime/`
- Modify: `docs/security-holding-ledger.md`
- Test: existing/new lifecycle CLI tests as needed

**Steps:**
1. Backfill the minimal stock history and external proxy history needed for replay.
2. Materialize realistic sample artifacts for:
   - condition review
   - execution record
   - post-trade review
3. Record the resulting refs in the holding ledger where appropriate.

### Phase P10: Verification Closure

#### Task P10-1: End-to-end focused verification

**Files:**
- Test only: relevant stock lifecycle suites

**Steps:**
1. Run focused tests for:
   - `security_condition_review_cli`
   - `security_execution_record_cli`
   - `security_post_trade_review_cli`
   - `security_decision_submit_approval_cli`
   - `security_decision_package_revision_cli`
2. Fix regressions until green.

#### Task P10-2: Final docs and task journal closure

**Files:**
- Modify: `.trae/CHANGELOG_TASK.md`
- Modify: operator-facing docs updated during implementation

**Steps:**
1. Append the final task journal entry summarizing P8-P10.
2. Make sure docs explain how to use the new lifecycle and where the validation data lives.
