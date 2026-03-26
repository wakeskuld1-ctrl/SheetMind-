# P2 Unknown Failure Regression Gates (2026-03-26)

## Required commands

```powershell
cargo test execute_multi_table_plan_failed_step_returns_unknown_failure_diagnostics -- --exact
cargo test execute_multi_table_plan_missing_tool_call_returns_unknown_failure_diagnostics -- --exact
cargo test execute_multi_table_plan_stops_when_result_bindings_are_missing -- --exact
cargo test execute_multi_table_plan_stops_when_join_risk_threshold_exceeded -- --exact
cargo test execute_multi_table_plan_stops_before_join_without_auto_confirm -- --exact
```

## Expected result

- All commands exit `0`.
- Unknown failure tests must include:
  - `execution_status = failed`
  - `failure_diagnostics.failure_class = unknown_runtime_failure`
  - `failure_diagnostics.fallback_route = table_processing_diagnostics`
- Controlled-stop tests must still expose original stop statuses without behavior drift.

## Evidence

- `docs/acceptance/artifacts/2026-03-26-p2-unknown-failure/manifest.json`
- `docs/acceptance/artifacts/2026-03-26-p2-unknown-failure/*.log`
