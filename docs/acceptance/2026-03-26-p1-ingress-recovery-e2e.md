# P1 Ingress Recovery E2E Record (2026-03-26)

## Test summary

- Date: 2026-03-26
- Workspace: `E:\Excel\SheetMind-`
- Branch: `codex/p0-preflight-chain`
- Objective: validate that ingress and controlled-stop recovery are routed in deterministic A/B/C/D order with a unified user-facing response contract.

## Recovery matrix and evidence

### Class A: path format / syntax correction first

Expected behavior:
- classify as entry-format issue before any data-quality diagnosis
- route to table-processing recovery instead of analysis/modeling

Evidence:
- Runtime guard test log: `docs/acceptance/artifacts/2026-03-26-p1-ingress-recovery/step_00_path_input_validation.log`
- Skill routing contract:
  - `skills/excel-orchestrator-v1/SKILL.md`
  - `skills/table-processing-v1/SKILL.md`

Observed:
- Input validation errors are deterministic and user-readable.
- Recovery policy is documented as "format first, then retry".

### Class B: Chinese-path compatibility fallback

Expected behavior:
- classify as compatibility issue (not missing-file issue)
- require explicit user confirmation before ASCII temp-copy fallback

Evidence:
- Runtime compatibility logs:
  - `docs/acceptance/artifacts/2026-03-26-p1-ingress-recovery/step_05_chinese_path_windows.log`
  - `docs/acceptance/artifacts/2026-03-26-p1-ingress-recovery/step_06_chinese_path_gbk_payload.log`
- Skill routing and confirmation rules:
  - `skills/excel-orchestrator-v1/SKILL.md`
  - `skills/table-processing-v1/SKILL.md`

Observed:
- CLI ingestion accepts Chinese Windows paths and GBK payloads.
- Skill contract keeps fallback behind explicit user confirmation.

### Class C: controlled execution stops

Expected behavior:
- treat stop statuses as controlled gates, not runtime crashes
- always return stop type, stop location, and next safe action

Evidence:
- `stopped_needs_preflight_confirmation`:
  - `docs/acceptance/artifacts/2026-03-26-p1-ingress-recovery/step_01_stopped_needs_preflight_confirmation.log`
- `stopped_missing_result_bindings`:
  - `docs/acceptance/artifacts/2026-03-26-p1-ingress-recovery/step_02_stopped_missing_result_bindings.log`
- `stopped_join_risk_threshold`:
  - `docs/acceptance/artifacts/2026-03-26-p1-ingress-recovery/step_03_stopped_join_risk_threshold.log`
- Guard-default continuity for auto-confirm path:
  - `docs/acceptance/artifacts/2026-03-26-p1-ingress-recovery/step_04_default_guard_auto_confirm.log`

Observed:
- all three controlled statuses are surfaced with deterministic stop metadata.
- rerun path remains explicit and safety-first.

### Class D: unknown runtime/tool failures

Expected behavior:
- fallback wording should remain deterministic
- default route remains table-processing diagnostics unless user stops

Evidence:
- fallback contract language in:
  - `docs/plans/2026-03-26-p1-ingress-recovery-hardening.md`
  - `skills/excel-orchestrator-v1/SKILL.md`
  - `skills/table-processing-v1/SKILL.md`

Observed:
- Class D behavior is documented and aligned across orchestrator/table-processing skills.

## Unified UX contract check

P1 requires every recovery response to keep the same 3-part structure:
1. current understanding
2. current status
3. next action

Coverage evidence:
- `skills/excel-orchestrator-v1/acceptance-dialogues.md`
- `skills/table-processing-v1/acceptance-dialogues.md`
- `skills/excel-orchestrator-v1/cases.md`
- `skills/table-processing-v1/cases.md`

## Acceptance conclusion

P1 ingress-recovery hardening is complete for the Skill/runtime boundary:
- routing order A -> B -> C -> D is explicit and consistent;
- Class C controlled-stop statuses are executable and covered;
- orchestrator and table-processing now share one UX recovery contract.

Artifact index:
- `docs/acceptance/artifacts/2026-03-26-p1-ingress-recovery/manifest.json`
