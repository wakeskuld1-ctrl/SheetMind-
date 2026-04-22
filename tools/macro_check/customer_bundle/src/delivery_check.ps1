# 2026-04-22 CST: Added a customer-bundle entry script so support can hand out
# one root BAT while the script auto-discovers the packaged delivery EXE.
#
# 2026-04-22 CST: Keep script literals ASCII-only because Windows PowerShell 5
# can misparse UTF-8 files without BOM when they contain non-ASCII characters.

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
. (Join-Path $scriptRoot "delivery_check_core.ps1")

$macroCheckScript = Join-Path $scriptRoot "macro_check.ps1"
$appDirectory = Join-Path $scriptRoot "app"
$reportsRoot = Join-Path $scriptRoot "reports"
$launcherArgsPath = Join-Path $scriptRoot "launcher.args.txt"
$launcherWorkdirPath = Join-Path $scriptRoot "launcher.workdir.txt"

if (-not (Test-Path -LiteralPath $macroCheckScript)) {
    throw ("Missing bundled macro_check.ps1: " + $macroCheckScript)
}

try {
    $launcherPath = Resolve-DeliveryLauncherPath -AppDirectory $appDirectory
}
catch {
    $setupArtifact = Write-SetupFailureArtifact -ReportsRoot $reportsRoot -Message $_.Exception.Message
    Write-Output "delivery_check_status=setup_error"
    Write-Output ("delivery_check_error_path=" + $setupArtifact.error_path)
    Write-Output ("delivery_check_report_directory=" + $setupArtifact.report_directory)
    exit 1
}

$launcherArguments = Get-OptionalTextFileValue -Path $launcherArgsPath
$launcherWorkingDirectory = Get-OptionalTextFileValue -Path $launcherWorkdirPath
if ([string]::IsNullOrWhiteSpace($launcherWorkingDirectory)) {
    $launcherWorkingDirectory = Split-Path -Parent $launcherPath
}

$reportDirectory = Get-DeliveryReportDirectory -ReportsRoot $reportsRoot
if (-not (Test-Path -LiteralPath $reportsRoot)) {
    New-Item -ItemType Directory -Force -Path $reportsRoot | Out-Null
}

& $macroCheckScript -LauncherPath $launcherPath -LauncherArguments $launcherArguments -LauncherWorkingDirectory $launcherWorkingDirectory -OutputDirectory $reportDirectory
$archivePath = New-CustomerReturnArchive -ReportDirectory $reportDirectory

Write-Output "delivery_check_status=completed"
Write-Output ("delivery_check_launcher_path=" + $launcherPath)
Write-Output ("delivery_check_report_directory=" + $reportDirectory)
Write-Output ("delivery_check_return_zip=" + $archivePath)
