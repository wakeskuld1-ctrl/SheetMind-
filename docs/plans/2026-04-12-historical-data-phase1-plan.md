# Historical Data Phase 1 Plan

## Task 1
- Add RED tests for stock fundamental history tooling.
- Add RED tests for stock disclosure history tooling.
- Add RED tests that `security_analysis_fullstack` consumes governed history first.
- Add RED tests that `security_real_data_validation_backfill` persists slice-local history stores.

## Task 2
- Implement stock fundamental runtime store.
- Implement `security_fundamental_history_backfill`.
- Wire dispatcher/catalog/module exports.

## Task 3
- Implement stock disclosure runtime store.
- Implement `security_disclosure_history_backfill`.
- Wire dispatcher/catalog/module exports.

## Task 4
- Update `security_analysis_fullstack` to:
  - read governed fundamental history first
  - read governed disclosure history first
  - fall back to live fetch only when needed

## Task 5
- Update `security_real_data_validation_backfill` to persist fetched fundamental/disclosure contexts into slice-local governed stores.
- Expose the new persisted paths in result/manifest contracts when helpful.

## Task 6
- Run focused regressions for:
  - new backfill tools
  - fullstack
  - validation slice refresh
- Append `.trae/CHANGELOG_TASK.md`
- Summarize possible issues and recommended extra tests
