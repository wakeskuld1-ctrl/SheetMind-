# 2026-04-22 CST: Added a no-dependency smoke test so the macro check delivery
# folder can verify grading and report output before wiring the live probe.

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$toolRoot = Split-Path -Parent $scriptRoot
. (Join-Path $toolRoot "macro_check_core.ps1")

function Assert-True {
    param(
        [bool]$Condition,
        [string]$Message
    )

    if (-not $Condition) {
        throw $Message
    }
}

$inventory = [pscustomobject]@{
    default_host = "excel"
    has_excel = $true
    has_wps = $false
    xlsm_association = [pscustomobject]@{
        open_command = '"C:\Program Files\Microsoft Office\root\Office16\EXCEL.EXE" "%1"'
    }
}

$launcherSuccess = [pscustomobject]@{
    mode = "launcher"
    launcher_path = "C:\Delivery\launcher.exe"
    launcher_arguments = '--probe "{probe_workbook}"'
    launcher_resolved_arguments = '--probe "C:\Temp\macro_probe.xlsm"'
    launcher_working_directory = "C:\Delivery"
    launcher_started = $true
    launcher_process_name = "launcher"
}

$cleanSecuritySignals = [pscustomobject]@{
    firewall_profiles = @(
        [pscustomobject]@{
            name = "Domain"
            enabled = $true
            default_inbound_action = "Block"
            default_outbound_action = "Allow"
        }
    )
    launcher_firewall_rules = @()
    antivirus_products = @(
        [pscustomobject]@{
            display_name = "Windows Defender"
            source = "security_center"
        }
    )
    standalone_antivirus_hits = @()
    office_macro_policies = @()
}

$probeSuccess = [pscustomobject]@{
    requested_host = "default"
    requested_host_available = $true
    host_started = $true
    marker_written = $true
    marker_path = "C:\Temp\macro_probe_signal.txt"
    process_name = "EXCEL"
    actual_host = "excel"
    blocking_hint = $null
}

$resolvedArguments = Resolve-LauncherArgumentsTemplate -Template $launcherSuccess.launcher_arguments -ProbeWorkbookPath "C:\Temp\macro_probe.xlsm"
Assert-True ($resolvedArguments -eq '--probe "C:\Temp\macro_probe.xlsm"') "Expected launcher placeholder replacement"

$defaultResolvedArguments = Resolve-LauncherArgumentsTemplate -Template $null -ProbeWorkbookPath "C:\Temp\macro_probe.xlsm"
Assert-True ($defaultResolvedArguments -eq '"C:\Temp\macro_probe.xlsm"') "Expected default launcher arguments"

$successOutcome = Get-MacroCheckOutcome -Inventory $inventory -ProbeResult $probeSuccess -LaunchResult $launcherSuccess -SecuritySignals $cleanSecuritySignals
Assert-True ($successOutcome.grade -eq "runnable") "Expected success grade"

$reportText = ConvertTo-MacroCheckReportText -Inventory $inventory -ProbeResult $probeSuccess -LaunchResult $launcherSuccess -SecuritySignals $cleanSecuritySignals -Outcome $successOutcome
Assert-True ($reportText.Contains("Launcher")) "Expected launcher section"
Assert-True ($reportText.Contains("Security Signals")) "Expected security section"

$probeFailure = [pscustomobject]@{
    requested_host = "default"
    requested_host_available = $true
    host_started = $true
    marker_written = $false
    marker_path = "C:\Temp\macro_probe_signal.txt"
    process_name = "EXCEL"
    actual_host = "excel"
    blocking_hint = "Workbook opened but marker file was not written."
}

$launcherFailure = [pscustomobject]@{
    mode = "launcher"
    launcher_path = "C:\Delivery\launcher.exe"
    launcher_arguments = "--probe"
    launcher_resolved_arguments = '--probe "C:\Temp\macro_probe.xlsm"'
    launcher_working_directory = "C:\Delivery"
    launcher_started = $true
    launcher_process_name = "launcher"
}

$riskySecuritySignals = [pscustomobject]@{
    firewall_profiles = @(
        [pscustomobject]@{
            name = "Public"
            enabled = $true
            default_inbound_action = "Block"
            default_outbound_action = "Allow"
        }
    )
    launcher_firewall_rules = @(
        [pscustomobject]@{
            display_name = "Block Delivery Launcher"
            action = "Block"
            enabled = $true
            direction = "Outbound"
            program = "C:\Delivery\launcher.exe"
        }
    )
    antivirus_products = @(
        [pscustomobject]@{
            display_name = "Windows Defender"
            source = "security_center"
        }
    )
    standalone_antivirus_hits = @(
        [pscustomobject]@{
            vendor_key = "360"
            display_name = "360 Total Security"
            evidence_source = "uninstall"
            evidence_value = "360 Total Security"
        },
        [pscustomobject]@{
            vendor_key = "kingsoft"
            display_name = "Kingsoft Antivirus"
            evidence_source = "service"
            evidence_value = "KSafeSvc"
        }
    )
    office_macro_policies = @(
        [pscustomobject]@{
            scope = "HKCU"
            office_version = "16.0"
            application = "Excel"
            value_name = "VBAWarnings"
            value = 4
            risk_level = "high_risk"
            description = "Disable all macros without notification."
        }
    )
}

$failureOutcome = Get-MacroCheckOutcome -Inventory $inventory -ProbeResult $probeFailure -LaunchResult $launcherFailure -SecuritySignals $riskySecuritySignals
Assert-True ($failureOutcome.grade -eq "high_risk") "Expected high risk grade"
Assert-True ($failureOutcome.issues.Count -ge 4) "Expected failure issues"

$failureText = ConvertTo-MacroCheckReportText -Inventory $inventory -ProbeResult $probeFailure -LaunchResult $launcherFailure -SecuritySignals $riskySecuritySignals -Outcome $failureOutcome
Assert-True ($failureText.Contains("360 Total Security")) "Expected standalone antivirus hit in report"
Assert-True ($failureText.Contains("Kingsoft Antivirus")) "Expected second standalone antivirus hit in report"
Assert-True ($failureText.Contains("Block Delivery Launcher")) "Expected firewall rule in report"

$tempDir = Join-Path ([System.IO.Path]::GetTempPath()) ("macro_check_smoke_" + [guid]::NewGuid().ToString("N"))
$paths = Write-MacroCheckArtifacts -OutputDirectory $tempDir -Inventory $inventory -ProbeResult $probeSuccess -LaunchResult $launcherSuccess -SecuritySignals $cleanSecuritySignals -Outcome $successOutcome
Assert-True (Test-Path -LiteralPath $paths.txt_path) "Expected txt report"
Assert-True (Test-Path -LiteralPath $paths.json_path) "Expected json report"

Remove-Item -LiteralPath $tempDir -Recurse -Force
Write-Output "macro_check_core_smoke=passed"
