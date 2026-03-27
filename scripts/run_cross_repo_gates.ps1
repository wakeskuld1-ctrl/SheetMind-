param(
    # 2026-03-27 20:30:00 +08:00: Provide explicit suite selection so operators can run focused or full cross-repo gates.
    [ValidateSet("recovery_only", "contract_only", "all")]
    [string]$Suite = "all",
    # 2026-03-27 20:30:00 +08:00: Forward low-memory mode across repos to reduce build instability on constrained hosts.
    [switch]$LowMemory
)

# 2026-03-27 20:30:00 +08:00: Enable strict mode to fail fast on script mistakes and avoid false-green gate outcomes.
Set-StrictMode -Version Latest
# 2026-03-27 20:30:00 +08:00: Promote non-terminating errors to terminating errors for deterministic gate behavior.
$ErrorActionPreference = "Stop"

function Invoke-Step {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Name,
        [Parameter(Mandatory = $true)]
        [scriptblock]$Action
    )

    # 2026-03-27 20:30:00 +08:00: Emit step boundaries to improve cross-repo troubleshooting and CI observability.
    Write-Host "[cross-gate] start: $Name"
    & $Action
    if ($LASTEXITCODE -ne 0) {
        throw "cross repo gate step failed: $Name"
    }
    Write-Host "[cross-gate] done : $Name"
}

$root = (Resolve-Path "$PSScriptRoot\..").Path
$scenesRoot = (Resolve-Path "$PSScriptRoot\..\..\SheetMind-Scenes").Path

if ($LowMemory) {
    # 2026-03-27 20:30:00 +08:00: Cap cargo job parallelism to reduce memory spikes during multi-repo gate execution.
    $env:CARGO_BUILD_JOBS = "1"
}

switch ($Suite) {
    "recovery_only" {
        Invoke-Step -Name "sheetmind recovery gates" -Action {
            Push-Location $root
            try {
                & "$root\scripts\run_recovery_regression_gates.ps1" -Suite all -LowMemory:$LowMemory
            }
            finally {
                Pop-Location
            }
        }
    }
    "contract_only" {
        Invoke-Step -Name "sheetmind_scenes contract gates" -Action {
            Push-Location $scenesRoot
            try {
                & "$scenesRoot\scripts\run_contract_gates.ps1" -Suite all -LowMemory:$LowMemory
            }
            finally {
                Pop-Location
            }
        }
    }
    default {
        Invoke-Step -Name "sheetmind recovery gates" -Action {
            Push-Location $root
            try {
                & "$root\scripts\run_recovery_regression_gates.ps1" -Suite all -LowMemory:$LowMemory
            }
            finally {
                Pop-Location
            }
        }
        Invoke-Step -Name "sheetmind_scenes contract gates" -Action {
            Push-Location $scenesRoot
            try {
                & "$scenesRoot\scripts\run_contract_gates.ps1" -Suite all -LowMemory:$LowMemory
            }
            finally {
                Pop-Location
            }
        }
    }
}

# 2026-03-27 20:30:00 +08:00: Emit unified completion marker for CI and manual operation handoff.
Write-Host "[cross-gate] all selected gates passed."
