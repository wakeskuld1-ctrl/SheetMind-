# Equity ETF Manual Proxy Input Design

## Context
- Date: 2026-04-11
- Scope: governed manual proxy inputs for `equity_etf`
- Goal: let fund-flow, premium-discount, and benchmark-relative-strength proxy values enter the formal security decision chain before real ETF market-structure ingestion exists

## Problem
- `equity_etf` only had placeholder proxy contracts.
- Feature snapshot and approval flows could not freeze manually supplied ETF flow or premium-discount context.
- Runtime governance could validate `equity_etf` identity, but could not consume auditable equity ETF proxy evidence.

## Design
- Extend `SecurityExternalProxyInputs` with six governed fields:
  - `etf_fund_flow_proxy_status`
  - `etf_fund_flow_5d`
  - `premium_discount_proxy_status`
  - `premium_discount_pct`
  - `benchmark_relative_strength_status`
  - `benchmark_relative_return_5d`
- Extend the `equity_etf` feature family so runtime scorecard and training share the same contract.
- Reuse the existing ETF proxy-governance pattern:
  - explicit status wins
  - numeric-only input auto-upgrades active-pool status to `manual_bound`
  - missing active-pool fields fall back to `placeholder_unbound`
  - non-active pools stay `not_applicable`
- Freeze the three numeric proxy inputs into `raw_features_json`.
- Surface all six equity ETF proxy inputs into `group_features_json["X"]`.
- Include all six fields in the evidence hash so committee, chair, approval, and later review artifacts can audit the same proxy payload.

## Why this boundary
- No ETF flow API ingestion is introduced in this step.
- No historical premium-discount backfill is introduced in this step.
- No sub-pool reclassification is introduced in this step.
- This keeps the task focused on governed contract wiring only.

## Expected outcome
- `equity_etf` can formally consume manual ETF market-structure context.
- Later sector ETF and broad-index ETF proxy ingestion can reuse the same contract without schema churn.
