# 2026-04-11 Gold ETF Manual Proxy Input Plan

## Objective
- Make `gold_etf` manual proxy inputs formally consumable inside the existing governed decision pipeline.

## Steps
1. Add red tests that prove manual gold proxy inputs must survive feature freezing and approval-scorecard consumption.
2. Thread `external_proxy_inputs` through the request chain:
   - feature snapshot
   - forward outcome
   - committee
   - scorecard training
   - master scorecard
   - chair resolution
   - submit approval
3. Extend the governed gold ETF feature family to include:
   - proxy status fields
   - numeric proxy fields
4. Ensure runtime hashing and raw feature emission remain deterministic.
5. Run focused and adjacent regressions.
6. Record design and close-out details in the task journal.

## Acceptance Criteria
- Manual gold ETF proxy fields appear in `raw_features_json`.
- Gold proxy fields also appear in the grouped feature output used by downstream consumers.
- Approval/scorecard path can read the manual gold inputs without falling back to `placeholder_unbound`.
- Existing ETF family mismatch protections remain intact.

## Risks
- Structural availability does not guarantee fit quality.
- Manual values can drift from future external sources if backfill governance is not added later.
- Gold ETF modeling is still not production-grade until real proxy history and promotion gates exist.

## Deferred Work
- Treasury ETF real proxy ingestion
- Cross-border ETF FX and overseas-market proxy ingestion
- Equity ETF fund-flow and premium/discount proxy ingestion
- Historical proxy backfill and model refit based on real external time series
