# P2 Unknown Failure Regression Gates (2026-03-26)

## Required commands

```powershell
cargo test tool_catalog_includes_recover_multi_table_failure -- --exact
cargo test execute_multi_table_plan_failed_step_returns_unknown_failure_diagnostics -- --exact
cargo test execute_multi_table_plan_missing_tool_call_returns_unknown_failure_diagnostics -- --exact
cargo test execute_multi_table_plan_stops_after_target_step_id -- --exact
cargo test recover_multi_table_failure_runs_replay_then_full_chain -- --exact
cargo test recover_multi_table_failure_uses_runtime_continuation_template -- --exact
cargo test recover_multi_table_failure_allows_replay_template_overrides -- --exact
cargo test recover_multi_table_failure_allows_continue_template_overrides -- --exact
cargo test recover_multi_table_failure_rejects_invalid_template_overrides -- --exact
cargo test recover_multi_table_failure_accepts_legacy_template_arg_overrides -- --exact
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
  - `failure_diagnostics.recovery_templates.update_session_state`
  - `failure_diagnostics.recovery_templates.resume_execution`
  - `failure_diagnostics.recovery_templates.resume_full_chain`
  - `failure_diagnostics.state_synced = true`
- `stopped_after_step_id` replay path must include `continuation_templates.resume_full_chain`
- Recovery macro tests must end with `macro_status = completed` and `final_execution_status = completed`
- Template-override macro tests must prove replay/continue args can be patched and invalid override shape is rejected
- Legacy `template_arg_overrides` remains compatible with the same override behavior
- Controlled-stop tests must still expose original stop statuses without behavior drift.

## Evidence

- `docs/acceptance/artifacts/2026-03-26-p2-unknown-failure/manifest.json`
- `docs/acceptance/artifacts/2026-03-26-p2-unknown-failure/*.log`
