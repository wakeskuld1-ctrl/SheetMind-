# P1 Ingress Recovery Regression Gates (2026-03-26)

Use this runbook to quickly re-validate P1 recovery behavior after any routing/runtime change.

## Workspace

- Repo root: `E:\Excel\SheetMind-`
- Branch baseline: `codex/p0-preflight-chain`

## Required commands

Run each command from repo root.

```powershell
cargo test open_workbook_missing_path_returns_utf8_error_message -- --exact
cargo test cli_open_workbook_accepts_chinese_windows_path -- --exact
cargo test cli_open_workbook_accepts_gbk_encoded_json_with_chinese_path -- --exact
cargo test execute_multi_table_plan_stops_before_join_without_auto_confirm -- --exact
cargo test execute_multi_table_plan_stops_when_result_bindings_are_missing -- --exact
cargo test execute_multi_table_plan_stops_when_join_risk_threshold_exceeded -- --exact
cargo test execute_multi_table_plan_auto_confirm_applies_default_join_risk_guard -- --exact
```

## Gate mapping

- Class A (path format): `open_workbook_missing_path_returns_utf8_error_message`
- Class B (Chinese path compatibility):
  - `cli_open_workbook_accepts_chinese_windows_path`
  - `cli_open_workbook_accepts_gbk_encoded_json_with_chinese_path`
- Class C (controlled stops):
  - `execute_multi_table_plan_stops_before_join_without_auto_confirm`
  - `execute_multi_table_plan_stops_when_result_bindings_are_missing`
  - `execute_multi_table_plan_stops_when_join_risk_threshold_exceeded`
  - `execute_multi_table_plan_auto_confirm_applies_default_join_risk_guard`

## Expected result

- Every command exits `0`.
- No test should regress controlled stop status names:
  - `stopped_needs_preflight_confirmation`
  - `stopped_missing_result_bindings`
  - `stopped_join_risk_threshold`
- Join auto-confirm path keeps default guard values when thresholds are omitted.

## Stored evidence from 2026-03-26 run

- `docs/acceptance/artifacts/2026-03-26-p1-ingress-recovery/manifest.json`
- `docs/acceptance/artifacts/2026-03-26-p1-ingress-recovery/*.log`
