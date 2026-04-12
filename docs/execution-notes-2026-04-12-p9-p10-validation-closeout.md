# 2026-04-12 P9-P10 Validation Closeout

## What Was Added
- Defined a governed verification data slice for `601916.SH` on `2026-04-12`.
- Added a full lifecycle integration test that generates:
  - `condition_review.json`
  - `execution_record.json`
  - `post_trade_review.json`
  - `validation_slice_manifest.json`
- Preserved the sample in test-owned runtime fixtures for repeatable replay.

## Validation Entry Points
- Test fixture root:
  - `tests/runtime_fixtures/security_lifecycle_validation/`
- Stable operator copy target:
  - `.excel_skill_runtime/validation_slices/601916_SH_2026-04-12_lifecycle/`

## Replay Sequence
1. `security_decision_submit_approval`
2. `security_condition_review`
3. `security_execution_record`
4. `security_post_trade_review`
5. `security_decision_package_revision`

## Focused Verification
- `cargo test --test security_lifecycle_validation_cli -- --nocapture`
- `cargo test --test security_condition_review_cli -- --nocapture`
- `cargo test --test security_execution_record_cli -- --nocapture`
- `cargo test --test security_post_trade_review_cli -- --nocapture`
- `cargo test --test security_decision_submit_approval_cli -- --nocapture`
- `cargo test --test security_decision_package_revision_cli -- --nocapture`

## Known Boundaries
- The validation slice is deterministic and operator-friendly, but it still uses mocked disclosure endpoints for approval-time evidence.
- The copied validation slice is for replay and regression only; it is not a production investment evidence pack.

## Real-Data Validation Slice
- Added one governed real-data validation slice refresh entry:
  - tool: `security_real_data_validation_backfill`
  - slice: `.excel_skill_runtime/validation_slices/601916_SH_real_data_20260412/`
- Refreshed with live-compatible public sources on `2026-04-12 CST`:
  - primary symbol: `601916.SH`
  - market symbol: `510300.SH`
  - sector symbol: `512800.SH`
  - price window: `2024-01-02 -> 2025-08-08`
  - provider used: `sina`
- Generated operator-facing artifacts:
  - `.excel_skill_runtime/validation_slices/601916_SH_real_data_20260412/stock_history.db`
  - `.excel_skill_runtime/validation_slices/601916_SH_real_data_20260412/fullstack_context.json`
  - `.excel_skill_runtime/validation_slices/601916_SH_real_data_20260412/real_data_validation_manifest.json`
- Imported row coverage:
  - `601916.SH`: `388`
  - `510300.SH`: `388`
  - `512800.SH`: `388`

## Real-Data Verification Notes
- The first live-compatible attempt with `2025-01-01 -> 2025-08-08` failed at the formal fullstack gate because only `89` rows were available, which is below the `200`-row technical minimum.
- Extending the window to `2024-01-02 -> 2025-08-08` resolved that governance blocker and produced a reusable validation slice.
- This slice is still for validation and replay, not for direct trade approval.

## Additional ETF-Oriented Real-Data Slices
- Added four more live-compatible validation slices for cross-asset verification:
  - `.excel_skill_runtime/validation_slices/511010_SH_real_data_20260412/`
  - `.excel_skill_runtime/validation_slices/518880_SH_real_data_20260412/`
  - `.excel_skill_runtime/validation_slices/513500_SH_real_data_20260412/`
  - `.excel_skill_runtime/validation_slices/512800_SH_real_data_20260412/`
- Stable contextual proxy wiring for the current stack:
  - `511010.SH`, `518880.SH`, `513500.SH` -> `510300.SH` + `512800.SH`
  - `512800.SH` -> `510300.SH` + `sector_profile = a_share_bank`
- Imported row coverage per synced symbol:
  - `388` rows over `2024-01-02 -> 2025-08-08`
- These slices are now available beside the original `601916.SH` live-compatible sample, so later shadow/promotion checks can start from a broader validation set.
