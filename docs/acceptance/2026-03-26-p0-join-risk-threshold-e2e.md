# P0 Join Risk-Threshold E2E Record (2026-03-26)

## Test summary

- Date: 2026-03-26
- Workspace: `E:\Excel\SheetMind-`
- Objective: confirm `execute_multi_table_plan` provides a controllable safety stop for risky joins while preserving automatic execution for acceptable risk.

## Scenario A: default guard + auto confirm

Request artifact:
- `docs/acceptance/artifacts/2026-03-26-p0-join-risk-threshold/step_02_execute_default_guard_request.json`

Response artifact:
- `docs/acceptance/artifacts/2026-03-26-p0-join-risk-threshold/step_02_execute_default_guard_response.json`

Observed result:
- `status = ok`
- `execution_status = completed`
- runtime reported default guard values in `join_risk_guard`:
  - left unmatched: `10`
  - right unmatched: `10`
  - left duplicate keys: `5`
  - right duplicate keys: `5`
- executed steps include `join_preflight` followed by `join_tables`.

Interpretation:
- Auto path is productive and safe by default.

## Scenario B: strict guard (all thresholds = 0)

Request artifact:
- `docs/acceptance/artifacts/2026-03-26-p0-join-risk-threshold/step_03_execute_strict_guard_request.json`

Response artifact:
- `docs/acceptance/artifacts/2026-03-26-p0-join-risk-threshold/step_03_execute_strict_guard_response.json`

Observed result:
- `status = ok`
- `execution_status = stopped_join_risk_threshold`
- `stopped_at_step_id = step_1`
- stopped step action is `join_preflight`
- `join_risk_guard_breaches` includes concrete exceeded metrics (unmatched rows and duplicate keys)

Interpretation:
- Runtime can stop before formal join execution when risk is above user/tenant policy.

## Scenario C: explicit relaxed guard rerun

Request artifact:
- `docs/acceptance/artifacts/2026-03-26-p0-join-risk-threshold/step_04_execute_relaxed_guard_request.json`

Response artifact:
- `docs/acceptance/artifacts/2026-03-26-p0-join-risk-threshold/step_04_execute_relaxed_guard_response.json`

Observed result:
- `status = ok`
- `execution_status = completed`
- execution proceeds to `join_tables`

Interpretation:
- After explicit human decision, workflow can continue with custom tolerance.

## Acceptance conclusion

P0 objective for multi-table safety chain is met:

1. Planning and execution are integrated.
2. Auto-confirm joins now have default guard rails.
3. Guard breach produces deterministic stop status and structured breach reasons.
4. Orchestrator documentation now defines safe-first routing on threshold stops.

## Remaining boundary

- This record validates runtime and orchestration policy.
- It does not replace P1 unified UX hardening or P2 report/chart productization work.
