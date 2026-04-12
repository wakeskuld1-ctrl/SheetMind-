# Real Data Validation Backfill Design

## Goal
- 2026-04-12 CST: Add one governed real-data backfill entry for validation slices, because the securities mainline now has a formal lifecycle replay path but still lacks a repeatable way to refresh validation data from live-compatible providers.
- Purpose: make `price history + public disclosure context + validation manifest` reproducible instead of relying on manual shell sequencing.

## Problem
- The current validation slice under `.excel_skill_runtime/validation_slices/` is replayable, but it is still seeded from deterministic fixtures.
- Existing real-data capabilities are fragmented:
  - `sync_stock_price_history` can import real historical prices.
  - `security_analysis_fullstack` can fetch public financial and announcement context.
- There is no single governed tool that refreshes one validation slice with both kinds of data and records where the data landed.

## Recommended Approach
- Create a new stock tool: `security_real_data_validation_backfill`.
- Scope it narrowly to validation usage, not general production ingestion.
- The tool will:
  1. sync real price history for `symbol`, `market_symbol`, and `sector_symbol` into a dedicated validation-slice runtime DB,
  2. fetch one fullstack public-disclosure context for the primary symbol,
  3. write a stable manifest under the validation-slice root.

## Why This Approach
- It reuses the existing provider and analysis chains instead of inventing a parallel ingestion pipeline.
- It keeps “real data for verification” separate from “live production decision inputs”, which is safer while the project still mixes synthetic 2026 test dates with live-compatible provider logic.
- It gives operators and later AIs one repeatable entry point instead of a shell recipe.

## Tool Contract
- Request fields:
  - `slice_id`
  - `symbol`
  - optional `market_symbol`
  - optional `sector_symbol`
  - `start_date`
  - `end_date`
  - optional `providers`
  - optional `validation_runtime_root`
  - `created_at`
- Result fields:
  - `slice_id`
  - `validation_runtime_root`
  - `runtime_db_path`
  - `price_sync_summaries`
  - `fullstack_context_path`
  - `manifest_path`

## Architecture
- Refactor the real price-sync logic so provider fetch can be reused without forcing everything through the workspace-default runtime DB.
- The new tool will persist into:
  - `<validation_runtime_root>/stock_history.db`
  - `<validation_runtime_root>/fullstack_context.json`
  - `<validation_runtime_root>/real_data_validation_manifest.json`
- The validation manifest becomes the operator-facing entry point for this refreshed slice.

## Testing Strategy
- Add one focused CLI integration test with mocked providers:
  - mocked Tencent/Sina URLs for price history
  - mocked EastMoney financial and announcement URLs
- Assert:
  - price history really lands in the dedicated validation runtime DB
  - fullstack context file is written
  - manifest points to the written files and records the imported symbols

## Boundaries
- This tool will not yet backfill treasury/gold/cross-border proxy histories from live macro sources.
- It is for “verification-ready real data slices”, not for directly approving trades.
