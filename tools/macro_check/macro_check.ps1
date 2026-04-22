# 2026-04-22 CST: Added the delivery-facing macro launch checker so the team
# can verify workbook macro startup with one BAT entrypoint and one report set.
#
# 2026-04-22 CST: Keep script literals ASCII-only because Windows PowerShell 5
# can misparse UTF-8 files without BOM when they contain non-ASCII characters.

param(
    [string]$OutputDirectory,
    [int]$TimeoutSeconds = 20,
    [string]$LauncherPath,
    [string]$LauncherArguments,
    [string]$LauncherWorkingDirectory,
    [ValidateSet("default", "excel", "wps")]
    [string]$PreferredHost = "default"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$toolRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
. (Join-Path $toolRoot "macro_check_core.ps1")

function Get-MacroCheckOutputDirectory {
    param(
        [string]$ExplicitOutputDirectory,
        [string]$ToolRoot
    )

    if (-not [string]::IsNullOrWhiteSpace($ExplicitOutputDirectory)) {
        return $ExplicitOutputDirectory
    }

    $stamp = Get-Date -Format "yyyyMMdd_HHmmss"
    return (Join-Path $ToolRoot ("reports\" + $stamp))
}

function New-ProbeWorkspace {
    param(
        [Parameter(Mandatory = $true)]
        [string]$OutputDirectory
    )

    $workspace = Join-Path $OutputDirectory "probe_runtime"
    if (Test-Path -LiteralPath $workspace) {
        Remove-Item -LiteralPath $workspace -Recurse -Force
    }
    New-Item -ItemType Directory -Force -Path $workspace | Out-Null
    return $workspace
}

function Get-ProcessSnapshot {
    $snapshot = @{}
    foreach ($name in @("EXCEL", "et", "wps")) {
        $processes = @(Get-Process -Name $name -ErrorAction SilentlyContinue)
        $snapshot[$name] = @($processes | Select-Object -ExpandProperty Id)
    }
    return $snapshot
}

function Add-NewProcessIds {
    param(
        [Parameter(Mandatory = $true)]
        [System.Collections.Generic.HashSet[int]]$TargetSet,

        [AllowEmptyCollection()]
        [int[]]$BeforeIds,

        [AllowEmptyCollection()]
        [int[]]$CurrentIds
    )

    $beforeLookup = New-Object System.Collections.Generic.HashSet[int]
    foreach ($id in @($BeforeIds)) {
        [void]$beforeLookup.Add([int]$id)
    }

    foreach ($id in @($CurrentIds)) {
        if (-not $beforeLookup.Contains([int]$id)) {
            [void]$TargetSet.Add([int]$id)
        }
    }
}

function Stop-StartedHostProcesses {
    param(
        [Parameter(Mandatory = $true)]
        [System.Collections.Generic.HashSet[int]]$ProcessIds
    )

    foreach ($processId in $ProcessIds) {
        try {
            $process = Get-Process -Id $processId -ErrorAction Stop
        }
        catch {
            continue
        }

        try {
            if (-not $process.HasExited) {
                [void]$process.CloseMainWindow()
                Start-Sleep -Milliseconds 800
                $process.Refresh()
            }
        }
        catch {
        }

        try {
            if (-not $process.HasExited) {
                Stop-Process -Id $processId -Force -ErrorAction SilentlyContinue
            }
        }
        catch {
        }
    }
}

function Invoke-MacroProbe {
    param(
        [Parameter(Mandatory = $true)]
        [string]$ProbeWorkbookPath,

        [Parameter(Mandatory = $true)]
        [int]$TimeoutSeconds,

        [Parameter(Mandatory = $true)]
        [psobject]$Inventory,

        [Parameter(Mandatory = $true)]
        [ValidateSet("default", "excel", "wps")]
        [string]$PreferredHost,

        [AllowNull()]
        [string]$LauncherPath,

        [AllowNull()]
        [string]$LauncherArguments,

        [AllowNull()]
        [string]$LauncherWorkingDirectory
    )

    $workspace = Split-Path -Parent $ProbeWorkbookPath
    $markerPath = Join-Path $workspace "macro_probe_signal.txt"
    $errorPath = Join-Path $workspace "macro_probe_error.txt"
    $before = Get-ProcessSnapshot
    $startedProcess = $null
    $hostStarted = $false
    $observedProcess = $null
    $newHostProcessIds = New-Object System.Collections.Generic.HashSet[int]
    $requestedHostRecord = Get-PreferredHostRecord -Inventory $Inventory -PreferredHost $PreferredHost
    $requestedHostAvailable = ($PreferredHost -eq "default" -or $null -ne $requestedHostRecord)
    $launchTarget = $ProbeWorkbookPath
    $launchArguments = $null
    $launchMode = "association"
    $launcherStarted = $false
    $launcherProcessName = $null
    $resolvedLauncherArguments = $null

    if (-not [string]::IsNullOrWhiteSpace($LauncherPath)) {
        $launchMode = "launcher"
        if (-not (Test-Path -LiteralPath $LauncherPath)) {
            return [pscustomobject]@{
                launch_result = [pscustomobject]@{
                    mode = $launchMode
                    launcher_path = $LauncherPath
                    launcher_arguments = $LauncherArguments
                    launcher_resolved_arguments = $null
                    launcher_working_directory = $LauncherWorkingDirectory
                    launcher_started = $false
                    launcher_process_name = $null
                }
                probe_result = [pscustomobject]@{
                    requested_host = $PreferredHost
                    requested_host_available = $requestedHostAvailable
                    host_started = $false
                    marker_written = $false
                    marker_path = $markerPath
                    process_name = $null
                    actual_host = $null
                    blocking_hint = ("Launcher path does not exist: " + $LauncherPath)
                }
            }
        }

        $resolvedLauncherArguments = Resolve-LauncherArgumentsTemplate -Template $LauncherArguments -ProbeWorkbookPath $ProbeWorkbookPath
        $launchTarget = $LauncherPath
        $launchArguments = $resolvedLauncherArguments
    }
    elseif ($PreferredHost -ne "default") {
        $launchMode = "host"
        if (-not $requestedHostAvailable) {
            return [pscustomobject]@{
                launch_result = [pscustomobject]@{
                    mode = $launchMode
                    launcher_path = $null
                    launcher_arguments = $null
                    launcher_resolved_arguments = $null
                    launcher_working_directory = $null
                    launcher_started = $false
                    launcher_process_name = $null
                }
                probe_result = [pscustomobject]@{
                    requested_host = $PreferredHost
                    requested_host_available = $false
                    host_started = $false
                    marker_written = $false
                    marker_path = $markerPath
                    process_name = $null
                    actual_host = $null
                    blocking_hint = ("Requested host is not installed: " + $PreferredHost)
                }
            }
        }

        $launchTarget = $requestedHostRecord.executable_path
        $launchArguments = '"' + $ProbeWorkbookPath + '"'
    }

    try {
        $startProcessArguments = @{
            FilePath = $launchTarget
            PassThru = $true
            ErrorAction = "Stop"
        }
        if ($null -ne $launchArguments) {
            $startProcessArguments.ArgumentList = $launchArguments
        }
        if ($launchMode -eq "launcher" -and -not [string]::IsNullOrWhiteSpace($LauncherWorkingDirectory)) {
            $startProcessArguments.WorkingDirectory = $LauncherWorkingDirectory
        }

        $startedProcess = Start-Process @startProcessArguments
        if ($null -ne $startedProcess) {
            if ($launchMode -eq "launcher") {
                $launcherStarted = $true
                $launcherProcessName = $startedProcess.ProcessName
            }
            else {
                $hostStarted = $true
                $observedProcess = $startedProcess.ProcessName
                [void]$newHostProcessIds.Add([int]$startedProcess.Id)
            }
        }
    }
    catch {
        if ($launchMode -eq "launcher") {
            $launcherStarted = $false
        }
        else {
            $hostStarted = $false
        }
    }

    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    while ((Get-Date) -lt $deadline) {
        if (Test-Path -LiteralPath $markerPath) {
            $blockingHint = $null
            if (Test-Path -LiteralPath $errorPath) {
                $blockingHint = "Probe marker was written, but an additional error file was also captured."
            }
            $result = [pscustomobject]@{
                launch_result = [pscustomobject]@{
                    mode = $launchMode
                    launcher_path = $LauncherPath
                    launcher_arguments = $LauncherArguments
                    launcher_resolved_arguments = $resolvedLauncherArguments
                    launcher_working_directory = $LauncherWorkingDirectory
                    launcher_started = $launcherStarted
                    launcher_process_name = $launcherProcessName
                }
                probe_result = [pscustomobject]@{
                    requested_host = $PreferredHost
                    requested_host_available = $requestedHostAvailable
                    host_started = $true
                    marker_written = $true
                    marker_path = $markerPath
                    process_name = $observedProcess
                    actual_host = (Resolve-ObservedHostKey -ProcessName $observedProcess)
                    blocking_hint = $blockingHint
                }
            }

            if ($newHostProcessIds.Count -gt 0) {
                Stop-StartedHostProcesses -ProcessIds $newHostProcessIds
            }

            return $result
        }

        foreach ($name in @("EXCEL", "et", "wps")) {
            $currentIds = @(Get-Process -Name $name -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Id)
            Add-NewProcessIds -TargetSet $newHostProcessIds -BeforeIds $before[$name] -CurrentIds $currentIds
            if ($currentIds.Count -gt @($before[$name]).Count) {
                $hostStarted = $true
                if ([string]::IsNullOrWhiteSpace($observedProcess)) {
                    $observedProcess = $name
                }
            }
        }

        Start-Sleep -Milliseconds 800
    }

    $blockingHint = $null
    if ($hostStarted) {
        $blockingHint = "Workbook open timed out before the marker file appeared. Macro execution may be blocked by host security prompts or macro settings."
    }
    elseif ($launchMode -eq "launcher" -and $launcherStarted) {
        $blockingHint = "The launcher started, but the spreadsheet host did not appear before timeout."
    }
    elseif (Test-Path -LiteralPath $errorPath) {
        $blockingHint = Get-Content -LiteralPath $errorPath -Raw -Encoding UTF8
    }

    $result = [pscustomobject]@{
        launch_result = [pscustomobject]@{
            mode = $launchMode
            launcher_path = $LauncherPath
            launcher_arguments = $LauncherArguments
            launcher_resolved_arguments = $resolvedLauncherArguments
            launcher_working_directory = $LauncherWorkingDirectory
            launcher_started = $launcherStarted
            launcher_process_name = $launcherProcessName
        }
        probe_result = [pscustomobject]@{
            requested_host = $PreferredHost
            requested_host_available = $requestedHostAvailable
            host_started = $hostStarted
            marker_written = $false
            marker_path = $markerPath
            process_name = $observedProcess
            actual_host = (Resolve-ObservedHostKey -ProcessName $observedProcess)
            blocking_hint = $blockingHint
        }
    }

    if ($result.probe_result.marker_written -and $newHostProcessIds.Count -gt 0) {
        Stop-StartedHostProcesses -ProcessIds $newHostProcessIds
    }

    return $result
}

function Resolve-ObservedHostKey {
    param(
        [string]$ProcessName
    )

    if ([string]::IsNullOrWhiteSpace($ProcessName)) {
        return $null
    }

    $normalized = $ProcessName.ToLowerInvariant()
    if ($normalized -eq "excel") {
        return "excel"
    }
    if ($normalized -eq "et" -or $normalized -eq "wps") {
        return "wps"
    }
    return $normalized
}

$outputDir = Get-MacroCheckOutputDirectory -ExplicitOutputDirectory $OutputDirectory -ToolRoot $toolRoot
if (-not (Test-Path -LiteralPath $outputDir)) {
    New-Item -ItemType Directory -Force -Path $outputDir | Out-Null
}

$inventory = Get-SpreadsheetHostInventory
$probeAssetPath = Join-Path $toolRoot "assets\macro_probe.xlsm"
if (-not (Test-Path -LiteralPath $probeAssetPath)) {
    throw "Missing probe workbook asset: $probeAssetPath"
}

$workspace = New-ProbeWorkspace -OutputDirectory $outputDir
$runtimeProbePath = Join-Path $workspace "macro_probe.xlsm"
Copy-Item -LiteralPath $probeAssetPath -Destination $runtimeProbePath -Force

$securitySignals = Get-MacroSecuritySignals -LauncherPath $LauncherPath
$runtimeResult = Invoke-MacroProbe -ProbeWorkbookPath $runtimeProbePath -TimeoutSeconds $TimeoutSeconds -Inventory $inventory -PreferredHost $PreferredHost -LauncherPath $LauncherPath -LauncherArguments $LauncherArguments -LauncherWorkingDirectory $LauncherWorkingDirectory
$launchResult = $runtimeResult.launch_result
$probeResult = $runtimeResult.probe_result
$outcome = Get-MacroCheckOutcome -Inventory $inventory -ProbeResult $probeResult -LaunchResult $launchResult -SecuritySignals $securitySignals
$artifactPaths = Write-MacroCheckArtifacts -OutputDirectory $outputDir -Inventory $inventory -ProbeResult $probeResult -LaunchResult $launchResult -SecuritySignals $securitySignals -Outcome $outcome

Write-Output ("macro_check_grade=" + $outcome.grade)
Write-Output ("macro_check_launch_mode=" + $launchResult.mode)
Write-Output ("macro_check_launcher_started=" + $launchResult.launcher_started)
Write-Output ("macro_check_requested_host=" + $probeResult.requested_host)
Write-Output ("macro_check_actual_host=" + $probeResult.actual_host)
Write-Output ("macro_check_report_txt=" + $artifactPaths.txt_path)
Write-Output ("macro_check_report_json=" + $artifactPaths.json_path)
