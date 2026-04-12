# 2026-04-12 Real-Data Validation Backfill

## What Was Added
- Added a governed stock tool: `security_real_data_validation_backfill`
- Reused the formal stock-history sync provider chain without forcing workspace-default persistence
- Reused the formal `security_analysis_fullstack` chain against a slice-local `stock_history.db`
- Persisted a stable real-data validation manifest beside the refreshed slice

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
  - optional `market_profile`
  - optional `sector_profile`
  - `created_at`
- Result fields:
  - `slice_id`
  - `validation_runtime_root`
  - `runtime_db_path`
  - `price_sync_summaries`
  - `fullstack_context_path`
  - `manifest_path`

## First Real Slice
- Slice id: `601916_SH_real_data_20260412`
- Root:
  - `.excel_skill_runtime/validation_slices/601916_SH_real_data_20260412/`
- Artifacts:
  - `.excel_skill_runtime/validation_slices/601916_SH_real_data_20260412/stock_history.db`
  - `.excel_skill_runtime/validation_slices/601916_SH_real_data_20260412/fullstack_context.json`
  - `.excel_skill_runtime/validation_slices/601916_SH_real_data_20260412/real_data_validation_manifest.json`

## First Run Notes
- First attempt with `2025-01-01 -> 2025-08-08` failed the governed technical gate:
  - `历史数据不足，至少需要 200 条，当前只有 89 条`
- Second attempt with `2024-01-02 -> 2025-08-08` succeeded.
- Imported coverage:
  - `601916.SH`: `388`
  - `510300.SH`: `388`
  - `512800.SH`: `388`

## Second Real Slice Batch
- Added four ETF-oriented live-compatible validation slices:
  - `.excel_skill_runtime/validation_slices/511010_SH_real_data_20260412/`
  - `.excel_skill_runtime/validation_slices/518880_SH_real_data_20260412/`
  - `.excel_skill_runtime/validation_slices/513500_SH_real_data_20260412/`
  - `.excel_skill_runtime/validation_slices/512800_SH_real_data_20260412/`
- Imported coverage per synced symbol:
  - `388` rows from `2024-01-02 -> 2025-08-08`
- Slice wiring:
  - `511010.SH` -> `510300.SH` + `512800.SH`
  - `518880.SH` -> `510300.SH` + `512800.SH`
  - `513500.SH` -> `510300.SH` + `512800.SH`
  - `512800.SH` -> `510300.SH` + `sector_profile = a_share_bank`

## Second Batch Notes
- Direct ETF attempts without contextual proxies failed the governed fullstack gate.
- Stable validation inputs for the current stack still require:
  - explicit `market_symbol = 510300.SH`, or
  - `market_profile = a_share_core`
- The current fullstack chain also needs:
  - explicit `sector_symbol`, or
  - `sector_profile = a_share_bank`
- For validation usage, binding the ETF slices to the reusable `510300.SH` / `512800.SH` environment proxies is sufficient to keep the slices auditable and replayable.

## Focused Verification
- `cargo test --test security_real_data_validation_backfill_cli -- --nocapture`
- `cargo test --test stock_price_history_import_cli -- --nocapture`
- `cargo test --test security_lifecycle_validation_cli -- --nocapture`

## Known Boundaries
- The tool currently refreshes validation slices only; it is not a production ingestion pipeline.
- The first live-compatible slice still depends on public endpoints and may drift if providers change contracts.
- The ETF-oriented validation slices still reuse generic A-share environmental proxies, so they are suitable for governed validation but not yet category-native real-proxy evaluation packs.
- `docs/security-holding-ledger.md` still contains historical encoding noise, so this note keeps the first real-data slice discoverable without broad ledger rewrites.
