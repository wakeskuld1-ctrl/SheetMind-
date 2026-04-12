# 2026-04-11 Treasury ETF Manual Proxy Input Design

## Background
- The ETF governance layer already recognizes `treasury_etf` as a separate model subscope.
- Before this task, treasury ETF only had placeholder proxy contracts:
  - `yield_curve_proxy_status`
  - `funding_liquidity_proxy_status`
- Scheme B for the next step requires treasury ETF to accept manually supplied proxy inputs through the same governed path already used by `gold_etf`.

## Scope
- Allow manually supplied treasury ETF proxy inputs to enter the governed evidence, snapshot, scorecard, and approval chain.
- Keep the path auditable through existing request/response contracts.
- Do not add network fetch, historical backfill, or automated macro ingestion.

## Manual Proxy Fields
- `yield_curve_proxy_status`
- `yield_curve_slope_delta_bp_5d`
- `funding_liquidity_proxy_status`
- `funding_liquidity_spread_delta_bp_5d`

## Contract Rules
- Active `treasury_etf` requests should emit:
  - explicit provided status when present
  - `manual_bound` when a numeric treasury proxy is supplied without an explicit status
  - `placeholder_unbound` when the treasury proxy contract is required but still missing
- Non-treasury ETF requests should emit `not_applicable` for the treasury proxy fields.
- Treasury numeric proxy fields must remain part of the raw feature schema so later scorecard and training consumers can rely on a stable contract.

## Implementation Notes
- Extend `SecurityExternalProxyInputs` instead of introducing a separate treasury-only side payload.
- Freeze treasury numeric proxy values into raw features and grouped `X` features.
- Reuse helper-based numeric extraction so future treasury proxy additions remain append-only.
- Extend evidence hashing so manual treasury proxy values participate in deterministic audit hashing.

## Verification Targets
- `security_feature_snapshot` preserves manual treasury ETF proxy inputs in raw/group features.
- `security_decision_submit_approval` consumes those inputs through scorecard raw features.
- Existing treasury ETF structural guards remain green when the artifact is missing required treasury fields.

## Out of Scope
- Yield curve time-series ingestion
- DR007 / Shibor / repo data collection
- Historical treasury proxy backfill
- Production promotion of treasury ETF scorecards
