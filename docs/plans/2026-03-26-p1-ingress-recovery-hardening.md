# P1 Ingress Recovery Hardening

## Goal

Standardize how the Skill layer handles entry failures and execution-stop states so users always get:

1. a clear diagnosis category,
2. a deterministic next action,
3. a safe default route.

This closes the P1 gap between "features exist" and "first-contact experience is stable."

## Scope

- `skills/excel-orchestrator-v1/*`
- `skills/table-processing-v1/*`
- acceptance dialogue coverage for recovery and safety-stop branches

This round is Skill-layer and acceptance-flow hardening. No new computation logic is introduced.

## Recovery Classes

### Class A: Path format/syntax errors

Examples:
- invalid Windows separator or quoting
- malformed drive path
- wrong sheet selector format

Default action:
- route to table-processing recovery
- correct path format first
- retry open/list before any analysis action

### Class B: Chinese-path compatibility failures

Examples:
- host can locate file but backend read fails on non-ASCII path

Default action:
- classify as compatibility issue, not "file missing"
- request explicit user confirmation for ASCII temp-copy fallback
- retry with temp path after confirmation

### Class C: Controlled execution safety stops

Examples:
- `stopped_join_risk_threshold`
- `stopped_missing_result_bindings`
- `stopped_needs_preflight_confirmation`

Default action:
- do not classify as runtime crash
- explain exact stop reason and blocking field(s)
- safe-first route:
  - `stopped_join_risk_threshold` -> table-processing cleanup path
  - `stopped_missing_result_bindings` -> resume plan chain with correct bindings
  - `stopped_needs_preflight_confirmation` -> ask explicit confirmation, then rerun

### Class D: Unknown runtime/tool failures

Examples:
- malformed response payload
- unclassified dispatcher error

Default action:
- collect minimal error context
- return deterministic fallback wording
- route to table-processing diagnostics unless user explicitly asks to stop

## Routing Priority

Apply in this strict order:

1. Path format correction (Class A)
2. Chinese-path compatibility fallback (Class B)
3. Controlled stop handling (Class C)
4. Unknown-failure fallback (Class D)

Only after these checks should the orchestrator conclude "file unreadable" or "cannot continue."

## User-Facing Response Contract

Every recovery response should keep the same structure:

1. Current understanding
2. Current status
3. Next action

And every Class C stop must include:

- stop type (`execution_status`)
- stop location (`stopped_at_step_id` when available)
- plain-language explanation of breach/missing requirement
- whether user confirmation is required for retry

## Acceptance Targets (P1-Step1)

Minimum new acceptance coverage:

1. path syntax fail -> corrected and resumed
2. Chinese-path compatibility fail -> confirmed temp-copy fallback -> resumed
3. `stopped_join_risk_threshold` -> safe-first cleanup route
4. `stopped_missing_result_bindings` -> binding completion route

## Exit Criteria

- both Skills share the same classification/routing language,
- cases and request templates include all four classes,
- acceptance dialogues include at least one scene for each controlled stop class in scope.
