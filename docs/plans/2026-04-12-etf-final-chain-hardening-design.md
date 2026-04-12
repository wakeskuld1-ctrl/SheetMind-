# ETF Final Chain Hardening Design

## Goal

Make ETF historical proxy inputs survive from governed history hydration all the way to the formal final decision layer, while also making validation slices ETF-native and producing per-subscope ETF scorecard artifacts that the chair flow can actually consume.

## Scope

This design covers three gaps that are currently blocking ETFs at the final conclusion stage:

1. Historical ETF proxy hydration only works reliably on `security_feature_snapshot`, but not through the deeper `committee -> scorecard -> master_scorecard -> chair_resolution` chain.
2. Validation slices are still not ETF-native enough:
   - `511010.SH` currently misses the treasury peer environment symbol `511060.SH`.
   - `512800.SH` still relies on `a_share_bank` instead of an ETF-native slice profile.
3. ETF sub-pools do not yet have runtime-ready scorecard artifacts for the final chair flow.

## Design Decision

### Approach A: Deep-chain hydration only

Pros:
- Smallest code surface.
- Fastest way to prove the chair path can read governed ETF proxy history.

Cons:
- Validation slices still remain structurally weak.
- Final chair output can still degrade because the ETF model binding remains missing or wrong.

### Approach B: Deep-chain hydration + ETF-native validation slices

Pros:
- Fixes both the proxy propagation problem and the missing ETF environment symbol problem.
- Produces better replay inputs for later validation.

Cons:
- Final chair output can still stop at `model_unavailable` or `cross_section_invalid`.

### Approach C: Deep-chain hydration + ETF-native validation slices + ETF sub-scope artifacts

Pros:
- Solves the full last-mile chain instead of only one layer.
- Gives us a meaningful rerun of `security_chair_resolution` for all ETF sub-pools.
- Matches the current project goal: move ETFs from "snapshot-ready" to "formal conclusion-ready".

Cons:
- Largest change set.
- Requires disciplined TDD across three adjacent areas.

## Chosen Design

Use **Approach C**.

## Architecture

### 1. Unified ETF proxy hydration helper

Create one shared helper that loads historical external proxy inputs by `symbol + as_of_date`, merges them with request-level overrides, and returns one effective governed payload.

The helper will be reused by:
- `security_feature_snapshot`
- `security_decision_committee`
- `security_forward_outcome`

This keeps the evidence, scorecard, master scorecard, and chair flows aligned on the same proxy payload.

### 2. ETF-native validation-slice enrichment

Extend `security_real_data_validation_backfill` so it can sync additional ETF environment symbols when the request describes an ETF slice.

Initial rules:
- treasury ETF slices should include the treasury peer environment symbol `511060.SH` when required.
- equity ETF slices should keep ETF-native profile semantics instead of falling back to industry-only terminology.

The goal is not to create a complete asset-knowledge engine yet. The goal is to make the governed ETF validation slices structurally sufficient for final-chain replay.

### 3. ETF sub-scope artifact generation and formal chair consumption

Train and bind ETF sub-scope scorecard artifacts for:
- `treasury_etf`
- `gold_etf`
- `cross_border_etf`
- `equity_etf`

Then rerun the final chair flow with:
- governed ETF history
- governed ETF proxy history
- ETF-native slice environment
- ETF-subscope-correct artifact binding

## Data Flow

1. Validation slice refresh writes:
   - stock history
   - governed ETF proxy history
   - manifest
2. Committee and forward outcome both hydrate the same ETF proxy payload.
3. Scorecard sees the same governed ETF proxy fields that snapshot already saw.
4. ETF scorecard runtime checks the sub-scope feature family against ETF-native artifacts.
5. Chair resolution consumes:
   - committee result
   - scorecard result
   - master scorecard result
   without losing ETF proxy values in the deeper chain.

## Error Handling

- Missing governed ETF proxy history should still degrade cleanly, not panic.
- Missing ETF peer environment symbols in validation slices should fail with a clear governed replay message.
- Wrong ETF artifact family should remain `cross_section_invalid`.
- Missing ETF artifact should remain `model_unavailable`.

## Testing Strategy

1. Add red tests for deep-chain ETF proxy hydration in `security_chair_resolution_cli`.
2. Add red tests for ETF-native validation-slice enrichment in `security_real_data_validation_backfill_cli`.
3. Add red tests for ETF sub-scope artifact training/binding and final chair reruns.
4. Run focused regressions for:
   - feature snapshot
   - scorecard training
   - real-data validation backfill
   - chair resolution

## Expected Outcome

After this hardening round:
- ETF proxy history should survive into the final formal decision layer.
- ETF validation slices should be materially more native and replayable.
- ETF chair conclusions should fail only for real model/data reasons, not because the proxy chain breaks halfway.
