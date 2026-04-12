# ETF External Proxy Contracts Plan

Date: 2026-04-11

## Goal

Land placeholder external proxy contracts for all governed ETF sub-pools without
introducing real external data ingestion.

## Steps

1. Add red tests for ETF sub-pool proxy-contract presence.
2. Add red tests for ETF training config classification of `_proxy_status` fields.
3. Add red approval regression for `gold_etf` artifact missing gold proxy contracts.
4. Extend ETF feature-family constants and raw feature seed contract emission.
5. Update ETF training config builder so `_proxy_status` fields are categorical.
6. Re-run focused regressions plus adjacent ETF runtime and approval tests.
7. Record the result in `.trae/CHANGELOG_TASK.md`.

## Touched Files

- `src/ops/security_decision_evidence_bundle.rs`
- `src/ops/security_scorecard_training.rs`
- `tests/security_decision_submit_approval_cli.rs`
- `.trae/CHANGELOG_TASK.md`

## Verification

- `cargo test --lib required_etf_feature_family_includes_external_proxy_contracts -- --nocapture`
- `cargo test --lib etf_training_feature_config_separates_treasury_and_gold_subscopes -- --nocapture`
- `cargo test --test security_decision_submit_approval_cli security_decision_submit_approval_rejects_gold_etf_binding_without_gold_proxy_contract -- --nocapture`
- `cargo test --lib etf_runtime_guard_rejects_treasury_binding_without_treasury_feature_family -- --nocapture`
- `cargo test --test security_scorecard_training_cli -- --nocapture`
- `cargo test --test security_decision_submit_approval_cli security_decision_submit_approval_rejects_wrong_etf_subscope_binding_for_bond_etf -- --nocapture`

## Expected Output

- ETF evidence seeds expose stable proxy placeholder fields for all governed ETF
  sub-pools.
- ETF training configs classify proxy fields correctly as categorical inputs.
- Approval flow rejects a `gold_etf` artifact that lacks the minimum gold proxy
  contract.
