# P7 Automatic Proxy Backfill And Promotion Hardening Design

## Background

`P6` hardened the governed `candidate -> shadow -> champion` lifecycle by adding:
- standardized history-expansion readiness hints
- repeated shadow observations
- stricter champion promotion gates
- approval/package governance summaries

The securities mainline is now grade-aware and grade-hardened, but two structural gaps remain:
- historical external-proxy backfill is already stored and auto-hydrated into snapshots, yet promotion governance still does not consume accumulated backfill evidence directly
- `champion` still relies mostly on one governed shadow layer, not on repeated multi-window stability and explicit OOT-style promotion evidence

## Goal

Upgrade the current securities governance chain from “grade-hardened” to “promotion-evidence-driven” by:
- turning historical external-proxy backfill into a reusable governed coverage source
- adding multi-window shadow stability evidence
- requiring stronger promotion proof before a model becomes `champion`
- exposing the stronger promotion evidence in approval/package consumers

## Recommended Approach

### Approach A: Only strengthen automatic proxy backfill governance

Pros:
- smallest data-side change
- makes historical proxy coverage more truthful

Cons:
- `champion` still would not depend on enough windowed performance evidence
- approval would continue to consume a thin promotion explanation

### Approach B: Only strengthen champion promotion evidence

Pros:
- fastest governance tightening
- easy to explain stricter promotion outcomes

Cons:
- data coverage truth would lag behind
- promotion would still lean on manually summarized history-expansion records

### Approach C: Govern backfill coverage + add multi-window promotion evidence + expose both downstream

Pros:
- best balance of data truth and promotion truth
- reuses the current securities mainline instead of introducing a parallel promotion system
- lets approval/package preserve why a model was still blocked even after repeated shadow runs

Cons:
- larger change set than A or B
- requires careful TDD because several tools now share the same promotion evidence vocabulary

## Decision

Use **Approach C**.

## Scope

### In Scope

- extend `security_external_proxy_backfill` with reusable governed coverage signals
- let `security_history_expansion` consume governed backfill outputs instead of only free-form manual coverage summaries
- extend `security_shadow_evaluation` with multi-window / OOT-style stability evidence
- strengthen `security_model_promotion` so `champion` depends on both shadow continuity and promotion evidence stability
- expose the richer promotion evidence through `security_decision_submit_approval` and `decision_package`
- add focused CLI regressions for the strengthened chain

### Out Of Scope

- real-time third-party data ingestion
- replacing current training algorithms
- UI work
- new execution automation outside the current approval/package/holding-review chain

## Core Design

### 1. Governed Historical Backfill Coverage

`security_external_proxy_backfill` should stop being only a storage write result and become a reusable governed coverage source.

Each backfill result should expose:
- covered proxy fields
- covered symbols
- covered dates
- covered record count
- inferred backfill coverage tier

`security_history_expansion` should accept governed backfill references and fold them into one standardized coverage document. That keeps history expansion grounded in real imported proxy rows instead of only free-form operator notes.

### 2. Multi-window Shadow Stability

`security_shadow_evaluation` should evolve from “single governed shadow snapshot” into “windowed stability judgment”.

Add explicit multi-window evidence such as:
- `shadow_window_count`
- `oot_stability_status`
- `window_consistency_status`
- `promotion_evidence_notes[]`

The minimum design intent is:
- one good window is not enough for `champion`
- repeated governed windows with acceptable OOT/test behavior should be required

### 3. Harder Champion Promotion

`security_model_promotion` should require:
- enough repeated shadow observations
- consistent shadow state
- ready historical proxy coverage
- adequate window count
- acceptable OOT stability
- no active promotion blockers

This keeps promotion rule-based and transparent. We are not adding a hidden promotion scorer.

### 4. Approval And Package Consumption

Approval/package readers should see not only grade and blockers, but also the stronger promotion evidence:
- shadow observation count
- shadow window count
- OOT stability status
- promotion blockers
- approval consumption mode

That ensures downstream audit and replay can distinguish:
- not enough data coverage
- not enough shadow continuity
- not enough OOT stability

## Testing Strategy

### Focused CLI Tests

- `security_external_proxy_backfill_cli`
- `security_history_expansion_cli`
- `security_shadow_evaluation_cli`
- `security_model_promotion_cli`
- `security_decision_submit_approval_cli`

### Adjacent Regressions

- `security_feature_snapshot_cli`
- `security_scorecard_refit_cli`
- `security_master_scorecard_cli`

## Success Criteria

P7 is complete when:
- backfill results expose reusable governed coverage signals
- history expansion can consume governed backfill evidence
- shadow evaluation emits multi-window promotion evidence
- champion promotion requires the new window/OOT evidence
- approval/package outputs preserve the stronger promotion evidence
- the focused promotion-evidence chain is green
