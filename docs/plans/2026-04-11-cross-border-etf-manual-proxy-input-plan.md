# Cross-Border ETF Manual Proxy Input Plan

## Steps
1. Add failing tests for:
   - cross-border ETF feature family contract
   - cross-border ETF training config
   - feature snapshot raw/group freeze
   - approval flow scorecard consumption
2. Extend the governed external proxy request schema.
3. Extend cross-border ETF proxy field registries and feature family.
4. Freeze cross-border status and numeric proxies into evidence raw features.
5. Surface the same fields into feature snapshot group `X`.
6. Include cross-border proxy payload in the evidence hash.
7. Re-run focused tests and adjacent regressions.
8. Append the task journal entry.

## Validation targets
- `required_etf_feature_family_includes_external_proxy_contracts`
- `etf_training_feature_config_separates_treasury_and_gold_subscopes`
- `security_feature_snapshot_preserves_cross_border_etf_manual_proxy_inputs`
- `security_decision_submit_approval_scorecard_consumes_cross_border_etf_manual_proxy_inputs`

## Non-goals
- real FX feed ingestion
- overseas market backfill
- session calendar service
- model retraining quality upgrades
