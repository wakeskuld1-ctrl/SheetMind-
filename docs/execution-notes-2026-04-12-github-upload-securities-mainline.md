# 2026-04-12 GitHub Upload Closeout For Securities Mainline

## Scope
- Objective:
  - upload the securities and stock mainline work since `2026-04-10` on a dedicated branch
  - keep the upload limited to securities-related code, tests, execution notes, and handoff context
- Branch:
  - `codex/etf-proxy-import-latest-ready-20260412`

## Included Workstreams
- ETF proxy-import hardening for validation and latest reruns
- governed stock price / disclosure / fundamental history backfill pipeline
- multi-head scorecard, master scorecard, and future prediction mode support
- approval, position, condition-review, execution-record, and post-trade-review mainline wiring
- pooled ETF training and latest rerun diagnostics

## Explicit Exclusions
- `docs/AI_HANDOFF.md`
  - left untouched because the file is still in merge-conflict state
- unrelated `foundation` changes outside the securities upload scope
- bulky runtime fixture directories that are regenerated locally

## Evidence Used For This Upload
- task journal:
  - `.trae/CHANGELOG_TASK.md`
- execution notes:
  - `docs/execution-notes-2026-04-12-p8-lifecycle-closeout.md`
  - `docs/execution-notes-2026-04-12-p9-p10-validation-closeout.md`
  - `docs/execution-notes-2026-04-12-real-data-validation-backfill.md`
  - `docs/execution-notes-2026-04-13-multi-head-live-validation.md`
- latest ETF proxy fix summary:
  - `.excel_skill_runtime\\pool_training_fix_20260412\\latest_chair_pool_summary_after_proxy_import.json`

## Verification Performed
- `cargo fmt --all`
- `cargo test --test security_real_data_validation_backfill_cli -- --nocapture`

## Known Open Items
- This upload closes the ETF pool proxy auto-import bug, but it does not mean the full live-trading acceptance gate is finished.
- `513180.SH` and `515790.SH` now reach `score_status = ready`, yet latest final actions remain conservative.
- `docs/AI_HANDOFF.md` is still unresolved and intentionally excluded from this upload.

## Upload Intent
- Create one focused GitHub branch upload for the securities mainline and its handoff context.
- Avoid mixing in unrelated dirty files from the broader workspace.
