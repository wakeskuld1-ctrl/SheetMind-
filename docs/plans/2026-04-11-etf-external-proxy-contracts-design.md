# ETF External Proxy Contracts Design

Date: 2026-04-11  
Scope: `equity_etf`, `treasury_etf`, `gold_etf`, `cross_border_etf`

## Background

The ETF split completed earlier only separated model identity and minimum technical
factor entrances. That was enough to stop obviously wrong artifact bindings, but it
was not enough to prepare the system for production-grade ETF modeling:

- `treasury_etf` still needs rate-curve and funding semantics.
- `gold_etf` still needs gold-price and USD semantics.
- `cross_border_etf` still needs FX and overseas-market semantics.
- `equity_etf` still needs fund-flow and premium/discount semantics.

We do not want to connect real external feeds in this round, because that would
expand the boundary into ingestion, scheduling, and source governance.

## Decision

Introduce placeholder external proxy field contracts now, while keeping all values
local and schema-stable.

### Treasury ETF

- `yield_curve_proxy_status`
- `funding_liquidity_proxy_status`

### Gold ETF

- `gold_spot_proxy_status`
- `usd_index_proxy_status`

### Cross-Border ETF

- `fx_proxy_status`
- `overseas_market_proxy_status`

### Equity ETF

- `etf_fund_flow_proxy_status`
- `premium_discount_proxy_status`

## Runtime Semantics

The current round only freezes contract names and structural presence.

- Active ETF sub-pool fields are emitted as `placeholder_unbound`.
- Inactive ETF sub-pool fields are emitted as `not_applicable`.
- Runtime scorecard guard treats missing required proxy-contract fields inside an
  ETF artifact as `cross_section_invalid`.

This means the system can already say:

- "this ETF artifact is structurally not ready for this sub-pool"

without pretending that real external macro or overseas data is already connected.

## Why This Design

### Benefits

- Keeps schema stable before external data integration.
- Lets training, scorecard, and approval consume one auditable contract.
- Prevents later external-data rollout from forcing breaking field-name changes.
- Makes missing ETF external semantics visible in governance today.

### Non-Goals

- No real rate, FX, gold, or overseas feed ingestion in this round.
- No fit-quality promotion logic changes in this round.
- No new production ranking claims from these placeholder fields alone.

## Expected Effect

After this round:

- ETF sub-pools are no longer separated only by technical factor entrances.
- Artifacts that omit required ETF external-proxy contracts fail structurally.
- Later integration of real external feeds can bind into existing fields instead of
  changing training/runtime contracts again.
