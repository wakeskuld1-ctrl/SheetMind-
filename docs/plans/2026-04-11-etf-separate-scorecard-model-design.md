# ETF Separate Scorecard Model Design

> Scope: split ETF and equity scorecard modeling semantics, add ETF-specific feature seeds, and guard against cross-sectional invalid outputs when different ETF symbols collapse into the same feature bucket.

## Goal

Fix the current defect where different ETF symbols can receive identical probabilities only because the scorecard stack consumes a too-small coarse feature set. The first implementation stage must stay inside the existing local history + technical snapshot pipeline and must not depend on new external macro, futures, gold, or overseas-market feeds.

## Why This Change Exists

- The current scorecard runtime and training chain mostly consume only four coarse features: `integrated_stance`, `technical_alignment`, `data_gap_count`, and `risk_note_count`.
- When two ETF symbols land in the same buckets for those coarse features, the model necessarily produces the same score and probability.
- This makes the result mathematically explainable but operationally invalid for relative ETF comparison.
- ETFs and equities should share the governance chain, but they should not be forced into the same feature semantics and model family.

## Chosen Approach

### Option A: Only add a runtime invalid-output guard

Pros:
- Smallest change.
- Stops misleading outputs quickly.

Cons:
- Does not improve ETF modeling ability.
- Same-feature collapse keeps happening underneath.

### Option B: Only add ETF-specific features

Pros:
- Improves ETF signal resolution.
- Preserves the current scorecard architecture.

Cons:
- Still allows silent invalid comparisons if features collapse again later.
- No explicit governance signal for low-confidence cross-sectional output.

### Option C: Split model family + add ETF features + add invalid-output guard

Pros:
- Fixes the current root cause and adds a safety rail.
- Keeps the existing governance chain intact while separating modeling semantics.
- Matches the user requirement that ETF and equity should become different model tracks.

Cons:
- Broader than a hotfix.
- Requires updates across evidence seed, training config, runtime scorecard, and tests.

### Recommendation

Choose Option C.

The implementation will still stay bounded:
- no new external data sources in this round;
- ETF and equity share the same committee / chair / approval pipeline;
- only the feature family and model validity logic diverge.

## Architecture

### 1. Shared governance, separate modeling family

The following chain remains shared:
- `security_decision_committee`
- `security_scorecard`
- `security_master_scorecard`
- `security_chair_resolution`
- approval / package / holding ledger

The following layer diverges by instrument type:
- feature seed enrichment
- training feature config
- model registry identity
- runtime scorecard validity guard

### 2. ETF-specific feature family (stage 1)

The first ETF feature family must reuse already-available technical snapshot fields from `technical_consultation_basic` and should focus on characteristics that help distinguish bond ETFs and other low-volatility ETF products:

- `close_vs_sma50`
- `close_vs_sma200`
- `volume_ratio_20`
- `mfi_14`
- `cci_20`
- `williams_r_14`
- `boll_width_ratio_20`
- `atr_14`
- `rsrs_zscore_18_60`
- `support_gap_pct_20`
- `resistance_gap_pct_20`

These features are chosen because they are already derivable inside the existing local history pipeline and can improve ETF cross-sectional resolution without expanding system boundaries.

### 3. Feature seed source of truth

`build_evidence_bundle_feature_seed(...)` becomes the canonical place to expose ETF-specific raw features from the existing technical snapshot. This keeps:
- feature snapshot,
- scorecard runtime,
- training collection,
- future model governance

on the same source-of-truth seed structure.

### 4. Separate training config by instrument scope

The training stack must choose feature configs based on `instrument_scope`:
- `EQUITY` keeps the existing minimal set for now.
- `ETF` uses the ETF feature family plus the existing governance/coarse features.

This means ETF and equity will still write through the same artifact contract, but the model identity should reflect different instrument scopes and feature content.

### 5. Cross-sectional invalid guard

Runtime scorecard should explicitly detect this failure mode:
- different ETF symbols,
- same analysis date,
- same effective ETF feature signature,
- same probability/score because the signal space collapsed.

When the runtime detects such a collapse, the scorecard output should mark itself as invalid for cross-sectional comparison instead of pretending the probability is actionable.

Stage 1 guard behavior:
- introduce a new score status such as `cross_section_invalid`;
- downgrade quant signal / quant stance accordingly;
- add a limitation explaining that the ETF feature set did not produce enough symbol separation for valid relative comparison.

## Data Boundaries

This round explicitly does **not** add:
- gold-linked features,
- overseas market features,
- futures basis features,
- real-time data dependencies.

Those will belong to a later ETF expansion phase after the local-history ETF model family is stable and auditable.

## Testing Strategy

### Red tests first

1. ETF feature seed exposes differentiating numeric fields from the technical snapshot.
2. ETF training config includes ETF-specific numeric features when `instrument_scope = ETF`.
3. Runtime scorecard downgrades to invalid when ETF symbols collapse into the same effective signal space.

### Regression expectations

- Equity training tests continue to pass unchanged.
- Existing scorecard training tests continue to pass for equity fixtures.
- ETF runtime output no longer silently treats identical coarse snapshots as a valid relative comparison.

## Risks

- If ETF-specific features are added only to runtime but not training, model bins will stay incomplete.
- If the invalid guard is too aggressive, it may block legitimate ETF pairs that really are very similar.
- If we add too many features at once, the small ETF training sample may become sparse.

## Risk Controls

- Keep the first ETF feature set limited to snapshot fields already computed.
- Preserve the existing equity path unchanged.
- Use the invalid guard only for ETF modeling scope.
- Add precise tests around the collapse case before implementation.
