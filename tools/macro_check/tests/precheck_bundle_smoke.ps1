# 2026-04-22 CST: Added a smoke test for the precheck bundle so the customer
# environment package stays self-contained and produces stable artifacts.

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$toolRoot = Split-Path -Parent $scriptRoot
. (Join-Path $toolRoot "precheck_bundle\src\precheck_core.ps1")

function Assert-True {
    param(
        [bool]$Condition,
        [string]$Message
    )

    if (-not $Condition) {
        throw $Message
    }
}

$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("precheck_bundle_smoke_" + [guid]::NewGuid().ToString("N"))
$reportsRoot = Join-Path $tempRoot "reports"
New-Item -ItemType Directory -Force -Path $reportsRoot | Out-Null

try {
    $batPath = (Get-ChildItem -LiteralPath (Join-Path $toolRoot "precheck_bundle") -Filter "*.bat" -File | Select-Object -First 1).FullName
    Assert-True (-not [string]::IsNullOrWhiteSpace($batPath)) "Expected one BAT launcher in precheck bundle"
    $batText = Get-Content -LiteralPath $batPath -Raw -Encoding Default
    Assert-True ($batText.Contains("if not ""%EXIT_CODE%""==""0"" (")) "Expected BAT to keep the window only on failure"
    Assert-True ($batText.Contains("Windows PowerShell 5.1 or newer is required.")) "Expected PowerShell version gate message"
    Assert-True ($batText.Contains("powershell_version_error.txt")) "Expected PowerShell version error artifact"
    Assert-True ($batText.Contains("Missing src\precheck.ps1")) "Expected missing-script failure hint"
    Assert-True ($batText.Contains("Precheck failed with exit code")) "Expected runtime failure hint"

    $readmePath = (Get-ChildItem -LiteralPath (Join-Path $toolRoot "precheck_bundle") -Filter "*.txt" -File | Select-Object -First 1).FullName
    Assert-True (-not [string]::IsNullOrWhiteSpace($readmePath)) "Expected one text readme in precheck bundle"
    $readmeText = Get-Content -LiteralPath $readmePath -Raw -Encoding UTF8
    Assert-True ($readmeText.Contains("powershell_version_error.txt")) "Expected readme to mention version error return file"

    $reportDirectory = Get-PrecheckReportDirectory -ReportsRoot $reportsRoot
    Assert-True ($reportDirectory.StartsWith($reportsRoot)) "Expected report directory under reports root"

    $macroPayload = [pscustomobject]@{
        inventory = [pscustomobject]@{
            default_host = "excel"
            has_excel = $true
            has_wps = $false
        }
        probe = [pscustomobject]@{
            host_started = $true
            marker_written = $true
            process_name = "EXCEL"
            actual_host = "excel"
        }
        security = [pscustomobject]@{
            firewall_profiles = @(
                [pscustomobject]@{
                    name = "Public"
                    enabled = $true
                    default_inbound_action = "Block"
                    default_outbound_action = "Allow"
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
                    display_name = "360 Total Security"
                    evidence_source = "uninstall"
                    evidence_value = "360 Total Security"
                }
            )
            office_macro_policies = @()
        }
        outcome = [pscustomobject]@{
            grade = "runnable"
            issues = @()
        }
    }

    $dotNetSignals = [pscustomobject]@{
        dotnet_framework_release = 528040
        dotnet_framework_version_hint = "4.8"
    }

    $vcRuntimeSignals = @(
        [pscustomobject]@{
            display_name = "Microsoft Visual C++ 2015-2022 Redistributable (x64)"
            display_version = "14.38.33135"
        }
    )

    $outcome = Get-PrecheckOutcome -MacroPayload $macroPayload -DotNetSignals $dotNetSignals -VcRuntimeSignals $vcRuntimeSignals
    Assert-True ($outcome.grade -eq "runnable_with_risk") "Expected warning signals to elevate the grade"

    $systemSummary = [pscustomobject]@{
        computer_name = "TEST-PC"
        current_user = "TEST\\User"
        is_administrator = $false
        os_caption = "Windows 11"
        os_version = "10.0"
        os_build = "22631"
        os_architecture = "64-bit"
        manufacturer = "Test"
        model = "Virtual"
        total_memory_gb = 16
        powershell_version = "5.1"
    }

    $artifactPaths = Write-PrecheckArtifacts -OutputDirectory $reportDirectory -SystemSummary $systemSummary -DotNetSignals $dotNetSignals -VcRuntimeSignals $vcRuntimeSignals -MacroPayload $macroPayload -Outcome $outcome
    Assert-True (Test-Path -LiteralPath $artifactPaths.txt_path) "Expected precheck txt report"
    Assert-True (Test-Path -LiteralPath $artifactPaths.json_path) "Expected precheck json report"

    $archivePath = New-PrecheckReturnArchive -ReportDirectory $reportDirectory
    Assert-True (Test-Path -LiteralPath $archivePath) "Expected precheck return zip"
}
finally {
    if (Test-Path -LiteralPath $tempRoot) {
        Remove-Item -LiteralPath $tempRoot -Recurse -Force
    }
}

Write-Output "precheck_bundle_smoke=passed"
