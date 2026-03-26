# P0 Join Risk-Threshold Trial Guide (2026-03-26)

## Goal

Validate the P0 multi-table execution safety chain for Skill-driven runtime:

1. Plan generation works for multi-table join flow.
2. Auto execution (`auto_confirm_join=true`) applies default risk guard.
3. Strict guard can stop execution at `join_preflight` with machine-readable breaches.
4. After explicit confirmation, rerun with relaxed thresholds can complete.

## Runtime scope

- Binary-first execution path only.
- No Python/pandas/Jupyter/Node runtime dependency.
- Tool under test: `execute_multi_table_plan`.

## Fixture

- `tests/fixtures/join-customers.xlsx` (sheet: `Customers`)
- `tests/fixtures/join-orders.xlsx` (sheet: `Orders`)

## Trial steps

### Step 1: Build plan

Use `suggest_multi_table_plan` and confirm there are `join_preflight` + `join_tables` steps.

Expected:
- `status = ok`
- plan includes `step_1` action `join_preflight`
- plan includes `step_2` action `join_tables`

### Step 2: Execute with default safe mode

Call `execute_multi_table_plan` with:
- `auto_confirm_join = true`
- no explicit thresholds

Expected:
- `execution_status = completed`
- response includes auto guard defaults:
  - `max_left_unmatched_rows = 10`
  - `max_right_unmatched_rows = 10`
  - `max_left_duplicate_keys = 5`
  - `max_right_duplicate_keys = 5`

### Step 3: Execute with strict guard

Call `execute_multi_table_plan` with:
- `auto_confirm_join = true`
- all four thresholds set to `0`

Expected:
- `execution_status = stopped_join_risk_threshold`
- `stopped_at_step_id = step_1`
- `executed_steps[0].action = join_preflight`
- `executed_steps[0].join_risk_guard_breaches` exists and is non-empty

### Step 4: Explicitly rerun with relaxed thresholds

Call `execute_multi_table_plan` with:
- `auto_confirm_join = true`
- relaxed thresholds (example: 30/30/10/10)

Expected:
- `execution_status = completed`
- join chain executes to `join_tables`

## Artifacts

This trial's requests/responses are archived in:

- `docs/acceptance/artifacts/2026-03-26-p0-join-risk-threshold/manifest.json`
- `docs/acceptance/artifacts/2026-03-26-p0-join-risk-threshold/step_01_suggest_multi_table_plan_request.json`
- `docs/acceptance/artifacts/2026-03-26-p0-join-risk-threshold/step_01_suggest_multi_table_plan_response.json`
- `docs/acceptance/artifacts/2026-03-26-p0-join-risk-threshold/step_02_execute_default_guard_request.json`
- `docs/acceptance/artifacts/2026-03-26-p0-join-risk-threshold/step_02_execute_default_guard_response.json`
- `docs/acceptance/artifacts/2026-03-26-p0-join-risk-threshold/step_03_execute_strict_guard_request.json`
- `docs/acceptance/artifacts/2026-03-26-p0-join-risk-threshold/step_03_execute_strict_guard_response.json`
- `docs/acceptance/artifacts/2026-03-26-p0-join-risk-threshold/step_04_execute_relaxed_guard_request.json`
- `docs/acceptance/artifacts/2026-03-26-p0-join-risk-threshold/step_04_execute_relaxed_guard_response.json`
