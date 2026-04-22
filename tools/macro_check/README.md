# Macro Check

This folder packages a focused delivery check for one question only:

- Can the customer machine open an `.xlsm` workbook and actually run its startup macro?
- Can the delivery `EXE` hand off to Excel or WPS and still let the startup macro run?

## Files

- `run_macro_check.bat`: double-click entrypoint using the default `.xlsm` association
- `run_macro_check_excel.bat`: force Excel when Excel is installed
- `run_macro_check_wps.bat`: force WPS when WPS is installed
- `precheck_bundle/`: support-ready package for customer environment collection before the real EXE is shipped
- `customer_bundle/`: support-ready package with one root launcher for customer handoff
- `macro_check.ps1`: main runner
- `macro_check_core.ps1`: reusable host detection, grading, and report helpers
- `assets/macro_probe.xlsm`: shipped probe workbook
- `build_macro_probe_asset.ps1`: reproducible builder for the probe workbook
- `tests/macro_check_core_smoke.ps1`: no-dependency smoke test for the core helpers

## Reports

Running the checker writes:

- `macro_check_report.txt`
- `macro_check_report.json`

Both files are created under a timestamped folder in `tools/macro_check/reports/` unless an explicit output path is provided.

## Host Selection

- Use `run_macro_check.bat` to test the machine's default `.xlsm` host.
- Use `run_macro_check_excel.bat` to force Excel.
- Use `run_macro_check_wps.bat` to force WPS.

Reports now include:

- requested host
- requested host availability
- observed process
- actual host classification

## Launcher Selection

If the real delivery path starts from an `EXE`, run:

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\macro_check\macro_check.ps1 `
  -LauncherPath "C:\Path\To\DeliveryLauncher.exe"
```

By default the checker appends the runtime probe workbook path as the final argument.

If the launcher needs a specific argument template, use `{probe_workbook}` as the placeholder:

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\macro_check\macro_check.ps1 `
  -LauncherPath "C:\Path\To\DeliveryLauncher.exe" `
  -LauncherArguments "--open {probe_workbook}" `
  -LauncherWorkingDirectory "C:\Path\To"
```

The report will capture:

- launcher mode
- launcher start success
- launcher process name
- observed spreadsheet host
- macro marker result

## Security Signals

The report also captures environment clues that often affect delivery:

- Windows Defender Firewall profile state
- active firewall application rules for the launcher path
- Windows Security Center antivirus registrations
- common standalone antivirus traces such as `360`, `Kingsoft`, `Huorong`, and `Tencent PC Manager`
- Excel macro policy registry hints

## Support Bundle Stages

- Use `precheck_bundle/` before delivery when you only need customer machine facts and readiness signals.
- Use `customer_bundle/` during delivery when you want to wrap a real EXE and verify the full launcher path.

## Rebuild The Probe Workbook

If the shipped probe workbook must be rebuilt on a development machine with Excel installed:

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\macro_check\build_macro_probe_asset.ps1
```

## Scope

This tool intentionally does **not**:

- change firewall settings
- disable antivirus products
- promise a single root cause with complete certainty
- reproduce the full business workbook workflow

It answers whether Excel or WPS appears able to launch workbook macros through a real workbook-open probe, and it collects likely environment blockers around that chain.
