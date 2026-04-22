# 2026-04-22 Macro Launch Check Handoff

## Current Objective
- Current goal: package two support-side Windows bundles around the macro readiness checker.
- Stage: implementation and local verification are complete for the macro/precheck work. The next stage is company-side and customer-side trial runs.
- Important boundary: this branch also contains pre-existing betting-related work from the current worktree. Do not assume every change in the branch belongs to the macro checker.

## Contract And Decision State
- The checker is split into two support bundles:
  - `tools/macro_check/precheck_bundle/`
    - Purpose: collect customer environment facts before the real delivery EXE is shipped.
    - Root entry: `开始预检.bat`
    - No EXE is required.
  - `tools/macro_check/customer_bundle/`
    - Purpose: verify the real delivery EXE path later.
    - Root entry: `开始检测.bat`
    - Requires exactly one EXE in `src/app/`.
- The root customer experience contract is:
  - user double-clicks one BAT file
  - success path auto-exits
  - failure path keeps the window open with a short message
  - reports and a return zip are written under `src/reports/`
- The macro checker contract is:
  - real workbook-open probe using `assets/macro_probe.xlsm`
  - success signal is `macro_probe_signal.txt`
  - report outputs are `txt` and `json`
- Security signal collection currently includes:
  - firewall profile state
  - launcher-specific firewall rules
  - Windows Security Center antivirus products
  - standalone antivirus traces from uninstall entries, services, and processes
  - Excel macro policy registry hints
  - .NET Framework and VC++ runtime hints in the precheck bundle
- Customer readability contract:
  - `precheck_bundle/阅读说明.txt` tells the user to run the BAT and send back the newest zip

## Evidence And Verification
- Verified files:
  - `tools/macro_check/macro_check.ps1`
  - `tools/macro_check/macro_check_core.ps1`
  - `tools/macro_check/precheck_bundle/开始预检.bat`
  - `tools/macro_check/precheck_bundle/阅读说明.txt`
  - `tools/macro_check/precheck_bundle/src/precheck.ps1`
  - `tools/macro_check/precheck_bundle/src/precheck_core.ps1`
  - `tools/macro_check/customer_bundle/src/delivery_check.ps1`
  - `tools/macro_check/customer_bundle/src/delivery_check_core.ps1`
- Verified commands:
  - `powershell -ExecutionPolicy Bypass -File tools/macro_check/tests/macro_check_core_smoke.ps1`
  - `powershell -ExecutionPolicy Bypass -File tools/macro_check/tests/customer_bundle_smoke.ps1`
  - `powershell -ExecutionPolicy Bypass -File tools/macro_check/tests/precheck_bundle_smoke.ps1`
  - `powershell -ExecutionPolicy Bypass -File tools/macro_check/build_macro_probe_asset.ps1`
  - `powershell -ExecutionPolicy Bypass -File tools/macro_check/precheck_bundle/src/precheck.ps1`
- Last local precheck artifacts:
  - `tools/macro_check/precheck_bundle/src/reports/20260422_213750/`
  - `tools/macro_check/precheck_bundle/src/reports/customer_return_20260422_213750.zip`
- Verified behavior:
  - precheck bundle runs without a delivery EXE
  - success path exits without `pause`
  - failure path still pauses in the BAT
  - the checker tries to close only host processes started during the current probe run
- Unverified behavior:
  - real customer-side delivery EXE behavior inside `customer_bundle`
  - machines with WPS only
  - machines with aggressive third-party antivirus

## Open Risks And Blockers
- The precheck bundle estimates readiness only. It does not prove the future delivery EXE will run.
- The host auto-close logic is intentionally conservative:
  - it should close Excel/WPS started by the probe
  - it should not force-close a host session that was already open before the run
- `customer_bundle` still needs real EXE trials before it can be treated as stable.
- This branch contains unrelated betting changes. If someone wants to review only the macro work, filter by `tools/macro_check/` and the three docs under `docs/plans/2026-04-22-*`.

## Resume Guide
- Read first:
  - `tools/macro_check/README.md`
  - `docs/plans/2026-04-22-macro-launch-check-design.md`
  - `docs/plans/2026-04-22-macro-launch-check-plan.md`
  - `docs/plans/2026-04-22-macro-launch-check-handoff.md`
- Run first:
  - `powershell -ExecutionPolicy Bypass -File tools/macro_check/tests/precheck_bundle_smoke.ps1`
  - `powershell -ExecutionPolicy Bypass -File tools/macro_check/precheck_bundle/src/precheck.ps1`
- Then do this next:
  - Copy `tools/macro_check/precheck_bundle/` to the target company machine and run `开始预检.bat`
  - Inspect the newest `customer_return_*.zip` under `src/reports/`
  - If the company machine passes precheck, prepare the later `customer_bundle/` package with the real EXE
- Safe to change:
  - customer-facing wording in `阅读说明.txt`
  - bundle-local scripts in `precheck_bundle/src/` and `customer_bundle/src/`
- Dangerous to change:
  - `assets/macro_probe.xlsm` without rebuilding and re-verifying
  - host auto-close behavior without testing against already-open Excel sessions
