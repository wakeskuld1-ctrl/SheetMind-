# 2026-04-11 Gold ETF Manual Proxy Input Design

## Background
- The governed ETF proxy-contract rollout already introduced placeholder contract fields for `gold_etf`.
- Scheme B for the next step requires those placeholders to accept manually supplied proxy values without creating an out-of-band path.
- The first live target is `gold_etf`, because local technical history alone is not enough to explain gold ETF decisions.

## Scope
- Allow manually supplied gold ETF proxy inputs to enter the formal security pipeline.
- Keep the path auditable through the same request/response objects already used by:
  - `security_feature_snapshot`
  - `security_forward_outcome`
  - `security_decision_committee`
  - `security_scorecard_training`
  - `security_master_scorecard`
  - `security_chair_resolution`
  - `security_decision_submit_approval`
- Do not add network fetch, historical backfill, or automatic proxy collection.

## Manual Proxy Fields
- `gold_spot_proxy_status`
- `gold_spot_proxy_return_5d`
- `usd_index_proxy_status`
- `usd_index_proxy_return_5d`
- `real_rate_proxy_status`
- `real_rate_proxy_delta_bp_5d`

## Contract Rules
- Active `gold_etf` requests should emit:
  - explicit provided status when present
  - `manual_bound` when a numeric input is provided without an explicit status
  - `placeholder_unbound` when the gold proxy field is required but still missing
- Non-gold ETF requests should emit `not_applicable` for the gold proxy contract fields.
- Numeric proxy fields should stay in the raw feature schema even before historical backfill exists.

## Implementation Notes
- Add `SecurityExternalProxyInputs` to every request object that needs to preserve a frozen evidence trail.
- Keep hash calculation deterministic by hashing floating-point values via `to_bits()`.
- Reuse a single gold numeric-field registry so later expansion stays append-only.

## Verification Targets
- `security_feature_snapshot` preserves manual gold ETF proxy inputs in raw/group features.
- `security_decision_submit_approval` consumes those inputs through scorecard raw features.
- Existing ETF subscope mismatch guards remain green.

## Out of Scope
- Real-time gold, USD, or real-rate data collection
- Historical proxy backfill
- Production model promotion based on these new inputs alone
