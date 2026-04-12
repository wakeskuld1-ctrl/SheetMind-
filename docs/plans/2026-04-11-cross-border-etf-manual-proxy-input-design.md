# Cross-Border ETF Manual Proxy Input Design

## Context
- Date: 2026-04-11
- Scope: governed manual proxy inputs for `cross_border_etf`
- Goal: let FX, overseas market, and session-gap proxy values enter the formal security decision chain before real external feed ingestion exists

## Problem
- `cross_border_etf` had placeholder proxy contracts only.
- Feature snapshot and approval flows could not freeze manually supplied FX or overseas market context.
- Runtime governance could validate sub-pool identity, but not consume auditable cross-border proxy evidence.

## Design
- Extend `SecurityExternalProxyInputs` with six governed fields:
  - `fx_proxy_status`
  - `fx_return_5d`
  - `overseas_market_proxy_status`
  - `overseas_market_return_5d`
  - `market_session_gap_status`
  - `market_session_gap_days`
- Extend the `cross_border_etf` feature family so runtime scorecard and training share the same contract.
- Reuse the existing proxy-governance pattern:
  - explicit status wins
  - numeric-only input auto-upgrades active-pool status to `manual_bound`
  - missing active-pool fields fall back to `placeholder_unbound`
  - non-active pools stay `not_applicable`
- Freeze the three numeric proxy inputs into `raw_features_json`.
- Surface all six cross-border proxy inputs into `group_features_json["X"]`.
- Include all six fields in the evidence hash so committee, chair, approval, and later review artifacts can audit the same proxy payload.

## Why this boundary
- No external data fetching is introduced in this step.
- No historical proxy backfill is introduced in this step.
- No model-governance thresholds change in this step.
- This keeps the task focused on governed contract wiring only.

## Expected outcome
- `cross_border_etf` can formally consume manual FX and overseas session context.
- Later Nikkei, US, and broader QDII proxy ingestion can reuse the same contract without schema churn.
