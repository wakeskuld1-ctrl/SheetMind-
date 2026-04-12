# P6 Shadow Champion Hardening Design

## Background

`P5` made model grade semantics first-class across:
- governed history expansion
- `candidate / shadow / champion`
- approval brief and decision package consumption

The current chain is now governable, but it is still not strong enough to support durable promotion confidence. Two gaps remain:
- historical proxy coverage is recorded, but not standardized into reusable readiness signals
- `shadow` is still mostly a point-in-time judgment, not a continuous observation layer that can justify `champion`

## Goal

Upgrade the current securities governance chain from “grade-aware” to “grade-hardened” by adding:
- standardized historical coverage signals
- repeated shadow-observation semantics
- stricter champion promotion gates
- richer approval/package governance summaries

## Recommended Approach

### Approach A: Only harden promotion gates

Pros:
- smallest code change
- fastest way to make `champion` rarer

Cons:
- gates would still be weakly grounded if history coverage remains loosely described
- approval consumers would still lack enough context to explain why a model stayed blocked

### Approach B: Only standardize historical coverage

Pros:
- stronger data governance foundation
- easier to reason about which pools are still thin

Cons:
- `shadow` and `champion` would still behave like mostly static labels
- approval semantics would improve slowly

### Approach C: Standardize coverage + strengthen shadow continuity + harden champion + expose governance summary

Pros:
- best balance of data truth and governance truth
- keeps all changes inside the current securities mainline
- avoids reworking approval/package contracts twice

Cons:
- larger change set than A or B alone
- requires careful TDD and compatibility defaults

## Decision

Use **Approach C**.

## Scope

### In Scope

- extend `security_history_expansion` with standardized readiness coverage fields
- extend `security_shadow_evaluation` with prior-evaluation replay and consistency status
- extend `security_model_promotion` with harder `champion` requirements and explicit blockers
- extend approval/package consumers with model-governance summaries
- add focused CLI regressions for the hardened chain

### Out Of Scope

- real-time market ingestion
- new UI surfaces
- replacing current training algorithms
- new post-trade system outside the current approval/package/audit chain

## Core Design

### 1. Standardized History Coverage

Each history expansion document should stop being a loose note and become a reusable readiness input.

Add standardized coverage signals:
- `coverage_tier`
- `shadow_readiness_hint`
- `champion_readiness_hint`
- `proxy_field_coverage[]`

Each `proxy_field_coverage` entry describes:
- proxy field name
- coverage status
- covered horizons

### 2. Continuous Shadow Observation

Shadow evaluation should accept prior governed shadow evaluations for the same scope and produce:
- `shadow_observation_count`
- `shadow_consistency_status`
- `promotion_blockers[]`

This lets the chain distinguish:
- one good shadow snapshot
- repeated stable shadow behavior

### 3. Harder Champion Gate

`champion` should require more than “shadow recommended champion once”.

Champion promotion should require:
- champion-capable production readiness
- ready historical proxy coverage
- repeated shadow observations
- stable shadow consistency
- no active promotion blockers

### 4. Approval And Package Governance Summary

Approval consumers should read more than just grade and release mode.

Add a governance summary that includes:
- current model grade
- release mode
- observation count
- consistency status
- promotion blockers

This keeps approval, audit, and later replay aligned on the same explanation layer.

## Testing Strategy

### Focused CLI Tests

- `security_history_expansion_cli`
- `security_shadow_evaluation_cli`
- `security_model_promotion_cli`
- `security_decision_submit_approval_cli`

### Adjacent Regressions

- `security_scorecard_refit_cli`
- `security_master_scorecard_cli`
- `security_chair_resolution_cli`

## Success Criteria

P6 is complete when:
- history expansion emits standardized readiness coverage
- shadow evaluation can accumulate prior governed observations
- champion promotion requires stronger governed evidence
- approval/package outputs expose governance summary fields
- the focused hardened-governance chain is green
