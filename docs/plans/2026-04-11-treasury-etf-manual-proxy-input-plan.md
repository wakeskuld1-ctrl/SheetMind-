# 2026-04-11 Treasury ETF Manual Proxy Input Plan

## Objective
- Make `treasury_etf` manual proxy inputs formally consumable inside the governed security decision chain.

## Steps
1. Add red tests proving treasury ETF proxy inputs must survive feature freezing and approval-scorecard consumption.
2. Extend the treasury ETF feature family to include numeric proxy fields.
3. Thread treasury manual proxy values through the evidence seed and grouped feature output.
4. Ensure active treasury ETF requests auto-resolve proxy status to `manual_bound` when only numeric values are provided.
5. Extend evidence hashing for treasury proxy values.
6. Run focused and adjacent regressions.
7. Record design and close-out details in the task journal.

## Acceptance Criteria
- Manual treasury ETF proxy values appear in `raw_features_json`.
- Treasury proxy values also appear in grouped `X` features.
- Approval/scorecard path can read treasury manual proxy inputs without falling back to `placeholder_unbound`.
- Treasury ETF structural mismatch guards remain intact.

## Risks
- Structural completeness still does not guarantee macro relevance or fit quality.
- Manual proxy values can drift from future governed external feeds if backfill is not added later.
- Treasury ETF modeling remains research-grade until real historical treasury proxy series are available.

## Deferred Work
- Real treasury proxy ingestion from rate and funding sources
- Historical backfill for treasury proxy time series
- Treasury ETF candidate/shadow/champion promotion gates
- More detailed treasury macro proxy grouping beyond the current grouped `X` shell
