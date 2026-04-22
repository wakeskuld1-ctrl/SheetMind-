# 2026-04-22 CST: Added a smoke test for the customer bundle so the root
# launcher contract stays stable when support prepares a package for clients.

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$toolRoot = Split-Path -Parent $scriptRoot
. (Join-Path $toolRoot "customer_bundle\src\delivery_check_core.ps1")

function Assert-True {
    param(
        [bool]$Condition,
        [string]$Message
    )

    if (-not $Condition) {
        throw $Message
    }
}

$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("customer_bundle_smoke_" + [guid]::NewGuid().ToString("N"))
$appDirectory = Join-Path $tempRoot "app"
$reportsRoot = Join-Path $tempRoot "reports"

New-Item -ItemType Directory -Force -Path $appDirectory | Out-Null
New-Item -ItemType Directory -Force -Path $reportsRoot | Out-Null

try {
    $missingCaught = $false
    try {
        Resolve-DeliveryLauncherPath -AppDirectory $appDirectory | Out-Null
    }
    catch {
        $missingCaught = $_.Exception.Message.Contains("No delivery EXE")
    }
    Assert-True $missingCaught "Expected empty app directory to be rejected"

    $singleExePath = Join-Path $appDirectory "DeliveryLauncher.exe"
    New-Item -ItemType File -Path $singleExePath | Out-Null
    $resolvedLauncherPath = Resolve-DeliveryLauncherPath -AppDirectory $appDirectory
    Assert-True ($resolvedLauncherPath -eq $singleExePath) "Expected the only EXE to be selected"

    $secondExePath = Join-Path $appDirectory "AnotherLauncher.exe"
    New-Item -ItemType File -Path $secondExePath | Out-Null
    $multipleCaught = $false
    try {
        Resolve-DeliveryLauncherPath -AppDirectory $appDirectory | Out-Null
    }
    catch {
        $multipleCaught = $_.Exception.Message.Contains("Multiple EXE")
    }
    Assert-True $multipleCaught "Expected multiple EXEs to be rejected"

    Remove-Item -LiteralPath $secondExePath -Force
    $setupArtifact = Write-SetupFailureArtifact -ReportsRoot $reportsRoot -Message "setup failed"
    Assert-True (Test-Path -LiteralPath $setupArtifact.error_path) "Expected setup failure artifact"

    $reportDirectory = Get-DeliveryReportDirectory -ReportsRoot $reportsRoot
    New-Item -ItemType Directory -Force -Path $reportDirectory | Out-Null
    [System.IO.File]::WriteAllText((Join-Path $reportDirectory "sample.txt"), "ok", [System.Text.UTF8Encoding]::new($true))
    $archivePath = New-CustomerReturnArchive -ReportDirectory $reportDirectory
    Assert-True (Test-Path -LiteralPath $archivePath) "Expected customer return zip"
}
finally {
    if (Test-Path -LiteralPath $tempRoot) {
        Remove-Item -LiteralPath $tempRoot -Recurse -Force
    }
}

Write-Output "customer_bundle_smoke=passed"
