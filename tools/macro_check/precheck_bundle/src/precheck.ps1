# 2026-04-22 CST: Added a customer precheck entry script so support can hand
# out one package that collects environment facts before shipping the real EXE.
#
# 2026-04-22 CST: Keep script literals ASCII-only because Windows PowerShell 5
# can misparse UTF-8 files without BOM when they contain non-ASCII characters.

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
. (Join-Path $scriptRoot "precheck_core.ps1")

$macroCheckScript = Join-Path $scriptRoot "macro_check.ps1"
$reportsRoot = Join-Path $scriptRoot "reports"
$reportDirectory = Get-PrecheckReportDirectory -ReportsRoot $reportsRoot
$macroOutputDirectory = Join-Path $reportDirectory "macro_probe"

if (-not (Test-Path -LiteralPath $reportsRoot)) {
    New-Item -ItemType Directory -Force -Path $reportsRoot | Out-Null
}

if (-not (Test-Path -LiteralPath $macroCheckScript)) {
    throw ("Missing bundled macro_check.ps1: " + $macroCheckScript)
}

$macroRunOutput = @(& $macroCheckScript -OutputDirectory $macroOutputDirectory)
$macroPayloadPath = Join-Path $macroOutputDirectory "macro_check_report.json"
if (-not (Test-Path -LiteralPath $macroPayloadPath)) {
    throw ("Missing macro payload: " + $macroPayloadPath)
}

$macroLogPath = Join-Path $macroOutputDirectory "macro_check_stdout.log"
if ($macroRunOutput.Count -gt 0) {
    [System.IO.File]::WriteAllText($macroLogPath, ($macroRunOutput -join [Environment]::NewLine), [System.Text.UTF8Encoding]::new($true))
}

$macroPayload = Get-Content -LiteralPath $macroPayloadPath -Raw -Encoding UTF8 | ConvertFrom-Json
$systemSummary = Get-SystemSummary
$dotNetSignals = Get-DotNetRuntimeSignals
$vcRuntimeSignals = Get-VcRuntimeSignals
$outcome = Get-PrecheckOutcome -MacroPayload $macroPayload -DotNetSignals $dotNetSignals -VcRuntimeSignals $vcRuntimeSignals
$artifactPaths = Write-PrecheckArtifacts -OutputDirectory $reportDirectory -SystemSummary $systemSummary -DotNetSignals $dotNetSignals -VcRuntimeSignals $vcRuntimeSignals -MacroPayload $macroPayload -Outcome $outcome
$archivePath = New-PrecheckReturnArchive -ReportDirectory $reportDirectory

Write-Output ("precheck_grade=" + $outcome.grade)
Write-Output ("precheck_report_txt=" + $artifactPaths.txt_path)
Write-Output ("precheck_report_json=" + $artifactPaths.json_path)
Write-Output ("precheck_return_zip=" + $archivePath)
