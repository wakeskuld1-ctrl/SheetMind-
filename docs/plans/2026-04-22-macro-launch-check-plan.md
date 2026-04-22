# 2026-04-22 Macro Launch Check Plan

## Phase 1

- Refresh the approved design so the contract includes:
  - optional `EXE` launcher entry
  - firewall signals
  - Office macro policy hints
  - standalone antivirus traces, especially `360` and `Kingsoft`

## Phase 2

- Extend the smoke test first so it covers:
  - launcher-aware report rendering
  - firewall summary rendering
  - standalone antivirus hit rendering
  - risk grading for explicit launcher block signals

## Phase 3

- Extend `macro_check_core.ps1` with:
  - launcher argument resolution
  - firewall profile and application-rule collection
  - antivirus inventory collection from Security Center, uninstall entries, services, and processes
  - Office macro policy collection
  - updated grading and report rendering

## Phase 4

- Extend `macro_check.ps1` with:
  - optional launcher parameters
  - launcher-aware probe execution
  - security signal collection
  - unified JSON and text report payload

## Phase 5

- Update README and usage examples.
- Run smoke verification.
- Run at least one local runtime verification against the shipped probe path.

## Verification Targets

- `powershell -ExecutionPolicy Bypass -File tools/macro_check/tests/macro_check_core_smoke.ps1`
- `powershell -ExecutionPolicy Bypass -File tools/macro_check/build_macro_probe_asset.ps1`
- `powershell -ExecutionPolicy Bypass -File tools/macro_check/macro_check.ps1 -OutputDirectory ...`
- `powershell -ExecutionPolicy Bypass -File tools/macro_check/macro_check.ps1 -LauncherPath <path-to-exe> -OutputDirectory ...`

## Current Expected Residual Risk

- A successful macro probe cannot prove that third-party antivirus will never block the business `EXE` in a future run.
- Antivirus and firewall signals provide strong hints, but some customer machines may hide or virtualize these details.
- Runtime verification of the real delivery `EXE` remains pending until a concrete launcher path is supplied.
