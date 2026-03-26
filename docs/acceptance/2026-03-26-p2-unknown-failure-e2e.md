# P2 Unknown Failure Observability E2E Record (2026-03-26)

## Test summary

- Date: 2026-03-26
- Workspace: `E:\Excel\SheetMind-`
- Branch: `codex/p0-preflight-chain`
- Objective: validate unknown runtime/tool failures now expose deterministic `failure_diagnostics` for Skill-level recovery routing.

## Scenario A: failed step with runtime/tool error

Evidence:
- `docs/acceptance/artifacts/2026-03-26-p2-unknown-failure/step_01_unknown_failure_diagnostics.log`

Observed:
- `execution_status = failed`
- `failure_diagnostics.failure_class = unknown_runtime_failure`
- `failure_diagnostics.fallback_route = table_processing_diagnostics`
- `failure_diagnostics.failed_step_id` and `failed_action` are returned
- `failure_diagnostics.raw_error` mirrors `stop_reason`
- `failure_diagnostics.recovery_templates` provides deterministic `update_session_state` and resume calls
- resume calls include both blocked-step replay and full-chain continuation templates
- session state is written back to `current_stage=table_processing` with a recovery goal
- blocked-step replay path now emits `continuation_templates.resume_full_chain` for immediate continuation

Interpretation:
- Orchestrator can now route unknown failures deterministically instead of relying on free-text parsing.

## Scenario B: missing suggested tool call inside plan step

Evidence:
- `docs/acceptance/artifacts/2026-03-26-p2-unknown-failure/step_02_missing_tool_call_diagnostics.log`

Observed:
- `execution_status = failed`
- same diagnostics contract is returned even when failure is due to malformed step payload

Interpretation:
- Unknown failure contract is stable across multiple failure entry points.

## Regression checks: controlled stop branches remain stable

Evidence:
- `docs/acceptance/artifacts/2026-03-26-p2-unknown-failure/step_03_controlled_stop_missing_bindings.log`
- `docs/acceptance/artifacts/2026-03-26-p2-unknown-failure/step_04_controlled_stop_join_risk.log`
- `docs/acceptance/artifacts/2026-03-26-p2-unknown-failure/step_05_controlled_stop_preflight_confirmation.log`
- `docs/acceptance/artifacts/2026-03-26-p2-unknown-failure/step_06_continuation_template_after_replay.log`

Observed:
- `stopped_missing_result_bindings` still works and is unchanged
- `stopped_join_risk_threshold` still works and is unchanged
- `stopped_needs_preflight_confirmation` still works and is unchanged
- replay-stop branch (`stopped_after_step_id`) now returns continuation template for full-chain continuation

## Scenario C: one-call recovery macro flow

Evidence:
- `docs/acceptance/artifacts/2026-03-26-p2-unknown-failure/step_00_tool_catalog_recover_macro.log`
- `docs/acceptance/artifacts/2026-03-26-p2-unknown-failure/step_07_recover_macro_full_chain.log`
- `docs/acceptance/artifacts/2026-03-26-p2-unknown-failure/step_08_recover_macro_runtime_template.log`

Observed:
- `recover_multi_table_failure` is visible in tool catalog.
- Macro can run blocked-step replay and then continue full chain in one call.
- Macro can consume runtime-provided continuation template directly.
- Macro now accepts `template_overrides` (and legacy `template_arg_overrides`) to patch replay/continue args without rebuilding full templates.

## Acceptance conclusion

P2 unknown-failure observability slice is complete:

1. runtime now emits structured diagnostics for unknown failures;
2. fallback route is explicit (`table_processing_diagnostics`);
3. existing controlled-stop statuses are not regressed.

Artifact index:
- `docs/acceptance/artifacts/2026-03-26-p2-unknown-failure/manifest.json`
