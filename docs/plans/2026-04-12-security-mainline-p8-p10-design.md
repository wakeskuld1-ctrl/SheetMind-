# Security Mainline P8-P10 Design

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** complete the remaining stock mainline so the system can move from governed approval artifacts into a replayable execution loop, then backfill enough data to run verification-grade validation.

**Architecture:** build the remaining lifecycle in three phases. `P8` adds the missing formal objects for condition review, execution records, and post-trade review, then binds them to the existing approval/package/position-plan chain. `P9` standardizes backfillable verification data so the new lifecycle can replay against stable runtime fixtures. `P10` closes with end-to-end verification, operator-facing docs, and handoff updates.

**Tech Stack:** Rust, serde/serde_json, existing stock ops tool chain, CLI dispatcher/catalog, runtime JSON artifacts, markdown handoff docs.

---

## Current Baseline

- The approval mainline is already formalized around:
  - `security_decision_committee`
  - `security_scorecard`
  - `security_master_scorecard`
  - `security_chair_resolution`
  - `security_decision_submit_approval`
  - `security_decision_package_revision`
- Governance is already formalized through:
  - `security_history_expansion`
  - `security_shadow_evaluation`
  - `security_model_promotion`
- The missing lifecycle gap is no longer pre-trade approval. It is the lack of formal post-approval execution and replay objects in the current branch.

## Phase Summary

### P8: Execution And Review Loop

**Intent**
- Add the missing lifecycle objects that turn an approved decision into something that can be executed, revisited, and reviewed.

**New formal objects**
- `security_condition_review`
- `security_execution_record`
- `security_post_trade_review`

**What each object owns**
- `security_condition_review`
  - formal review trigger
  - review status
  - recommended follow-up action
  - binding to `decision_ref`, `approval_ref`, `position_plan_ref`, and optional package path
- `security_execution_record`
  - formal execution or non-execution event
  - action type such as `build/add/reduce/exit/freeze/unfreeze/observe_only`
  - links back to decision, approval, position plan, and optional condition review
- `security_post_trade_review`
  - final or interim replay review
  - layered attribution:
    - `data_issue`
    - `model_issue`
    - `governance_issue`
    - `execution_issue`
  - prescribed follow-up such as:
    - continue shadow
    - retrain
    - downgrade
    - freeze consumption

**Mainline integration**
- Tools must become first-class catalog/dispatcher entries.
- Approval/package chain must preserve the new refs rather than relying on off-chain notes.
- Holding ledger must gain stable references for:
  - condition review
  - execution record
  - post-trade review

### P9: Backfillable Verification Data

**Intent**
- Backfill enough verification-ready data to exercise the new lifecycle with reproducible runtime evidence.

**Scope**
- Extend runtime evidence for a small but stable validation slice.
- Prioritize data that can drive the newly added P8 chain:
  - approved decision artifacts
  - condition review inputs
  - execution outcome samples
  - post-trade review attribution samples

**Data policy**
- Do not aim for full market coverage in this phase.
- Aim for enough realistic historical/runtime examples to make:
  - tool contracts verifiable
  - lifecycle references replayable
  - end-to-end tests meaningful

### P10: Verification And Operating Closure

**Intent**
- Verify the entire lifecycle with executable tests and leave the workspace in a handoff-ready state.

**Scope**
- Run focused end-to-end suites for:
  - approval
  - condition review
  - execution record
  - post-trade review
  - data backfill driven replay
- Update operator-facing docs so the system is usable without reopening source files.

## Key Design Rules

### 1. Reuse Existing Anchors

- New lifecycle objects must bind to existing stable refs:
  - `decision_ref`
  - `approval_ref`
  - `position_plan_ref`
  - `decision_package_path` where applicable
- Avoid creating parallel identifiers when a stable upstream ref already exists.

### 2. Keep Governance Explicit

- A post-trade object must not silently override governance conclusions.
- If a review suggests retraining or downgrade, that must be a formal field, not prose only.

### 3. Preserve Replayability

- Every new object must be serializable to runtime JSON and discoverable by CLI.
- The runtime chain must allow replay in this order:
  - approval
  - condition review
  - execution record
  - post-trade review

### 4. Keep P9 Narrow

- P9 is not full data engineering.
- It only backfills the minimum realistic evidence set needed to validate the new lifecycle.

## Risks And Controls

### Risk: P8 duplicates previously abandoned object shapes
- **Control:** implement only against the current branch structure and current refs.

### Risk: post-trade review becomes a prose dump
- **Control:** force layered attribution fields and concrete follow-up actions.

### Risk: data backfill expands without bound
- **Control:** restrict P9 to a verification slice that directly supports the new lifecycle tests.

### Risk: approval consumers misread review artifacts as execution facts
- **Control:** keep execution records and post-trade reviews as explicit separate document types with their own statuses.

## Success Criteria

The P8-P10 bundle is successful when:

- The current branch has formal CLI tools for:
  - `security_condition_review`
  - `security_execution_record`
  - `security_post_trade_review`
- The approval/package/holding chain can preserve refs to those objects.
- There is enough runtime data to replay at least one realistic lifecycle through those objects.
- Focused end-to-end tests pass for the new lifecycle.
- Operator-facing docs explain how to use the lifecycle without reading source code.
