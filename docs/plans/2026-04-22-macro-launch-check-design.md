# 2026-04-22 Macro Launch Check Design

## Intent

Upgrade the current delivery-side probe so it answers the real field question:

- Can the customer machine launch the delivery `EXE`, hand off to Excel or WPS, and successfully execute the startup macro while exposing likely security blockers?

## Approved Scope

- Keep the real workbook-open probe instead of static guessing only.
- Support direct workbook launch, forced Excel, forced WPS, or a caller-provided `EXE` launcher.
- Record host inventory:
  - default `.xlsm` association
  - detected spreadsheet hosts
  - observed host process during the probe
- Record launcher inventory when an `EXE` path is provided:
  - launcher path
  - launcher arguments template
  - launcher start success
  - launcher process name
- Record security signals:
  - Windows Defender Firewall profile state
  - active firewall application rules for the launcher path
  - Windows Security Center antivirus registrations
  - common standalone antivirus traces from uninstall entries, services, and processes
  - Office macro policy registry hints
- Generate internal reports for the delivery team in `txt` and `json`.

## Explicit Non-Goals

- No automatic remediation of firewall or antivirus settings
- No full enterprise security audit
- No attempt to prove which product blocked the chain with 100 percent certainty
- No attempt to reproduce the full business workbook workflow

## Architecture

### Folder

`tools/macro_check/`

### Main Files

- `run_macro_check.bat`
- `macro_check.ps1`
- `macro_check_core.ps1`
- `assets/macro_probe.xlsm`
- `build_macro_probe_asset.ps1`
- `assets/vba/*`
- `tests/macro_check_core_smoke.ps1`

## Runtime Flow

1. BAT or PowerShell launches the checker.
2. PowerShell collects host inventory.
3. PowerShell collects security signals before launching the probe.
4. PowerShell copies the shipped probe workbook into a runtime folder.
5. PowerShell launches the probe through one of these paths:
   - default workbook association
   - forced Excel
   - forced WPS
   - caller-provided `EXE`, optionally using a probe placeholder in its arguments
6. The workbook `Workbook_Open` macro writes a marker file and closes itself.
7. PowerShell waits for the marker file, grades the result, and writes reports.

## Probe Contract

### Success Signal

- `macro_probe_signal.txt` appears in the runtime folder.

### Failure Signals

- launcher process does not start
- no host process appears
- host appears but marker file never appears
- optional `macro_probe_error.txt` is written by VBA

## Security Contract

### Facts

- Firewall profile state comes from NetSecurity cmdlets when available.
- Antivirus products come from `root/SecurityCenter2` when available.
- Standalone antivirus traces come from uninstall entries, services, and running processes.
- Office macro policy hints come from registry security values.

### Inference Rules

- Explicit firewall block rules for the launcher path are treated as a blocking risk.
- Third-party antivirus presence is reported as an environment risk signal, not as proof of blocking.
- Restrictive Office macro policy values are reported as high-risk hints when macro execution fails.

## Risk Grades

- `runnable`
- `runnable_with_risk`
- `high_risk`

## Main Risks

- Existing Excel security prompts may allow the host to start but still block macro execution.
- Existing open Excel instances may make host observation noisier.
- Some customer machines may not expose Security Center or firewall cmdlets because of policy or edition differences.
- Antivirus detection can identify likely products without proving that they actively blocked the probe.

## Acceptance

- The folder can still be copied and double-clicked for workbook-only probing.
- PowerShell can also accept a caller-provided `EXE` path and optional launcher arguments.
- Reports include launcher status, host status, macro status, firewall signals, Office macro policy hints, and standalone antivirus traces.
- The report distinguishes observed facts from blocking hints.
- The probe workbook remains reproducible from VBA source.
