# P2 Unknown Failure Observability Hardening

## Goal

Upgrade `execute_multi_table_plan` unknown-failure branch from plain error text to a structured recovery contract that Skills can route deterministically.

## Scope

- Runtime dispatcher output (`src/tools/dispatcher.rs`)
- Integration tests (`tests/integration_cli_json.rs`)
- Skill routing templates (`skills/excel-orchestrator-v1/*`, `skills/table-processing-v1/*`)
- Acceptance evidence (`docs/acceptance/*`)
- Recovery macro tool (`recover_multi_table_failure`)

## Problem

Current unknown failures are surfaced mainly as `execution_status=failed` + `stop_reason`. This is human-readable but weak for deterministic routing and replay.

## Design

When unknown failure is hit, runtime now emits:

- `failure_diagnostics.failure_class = "unknown_runtime_failure"`
- `failure_diagnostics.fallback_route = "table_processing_diagnostics"`
- `failure_diagnostics.fallback_message` (deterministic fallback wording)
- `failure_diagnostics.failed_step_id`
- `failure_diagnostics.failed_action`
- `failure_diagnostics.failed_tool`
- `failure_diagnostics.raw_error`
- `failure_diagnostics.recovery_templates.update_session_state`
- `failure_diagnostics.recovery_templates.resume_execution`
- `failure_diagnostics.recovery_templates.resume_full_chain`
- `failure_diagnostics.state_synced`
- `continuation_templates.resume_full_chain` after blocked-step replay succeeds

`stop_reason` remains for backward compatibility.

## Acceptance Targets

1. Step runtime error inside plan execution returns structured `failure_diagnostics`.
2. Missing `suggested_tool_call` branch also returns structured `failure_diagnostics`.
3. Unknown-failure branch writes session stage/user-goal back to table-processing context.
4. Existing controlled-stop branches (`stopped_needs_preflight_confirmation`, `stopped_missing_result_bindings`, `stopped_join_risk_threshold`) are unchanged.
5. Skill-layer templates include this new branch and deterministic recovery-template usage.
6. One-call macro path can execute replay + continuation without manual template stitching.

## Exit Criteria

- New tests pass for both unknown-failure entry points.
- Existing stop-status tests continue passing.
- Skills and acceptance docs describe deterministic fallback route for unknown failures.
