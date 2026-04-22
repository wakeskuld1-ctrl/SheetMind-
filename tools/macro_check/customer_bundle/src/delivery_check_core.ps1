# 2026-04-22 CST: Added a customer-bundle helper layer so the delivered
# package can auto-discover one EXE in src\app and wrap the generated report.
#
# 2026-04-22 CST: Keep script literals ASCII-only because Windows PowerShell 5
# can misparse UTF-8 files without BOM when they contain non-ASCII characters.

Set-StrictMode -Version Latest

function Get-DeliveryTimestamp {
    Get-Date -Format "yyyyMMdd_HHmmss"
}

function Get-DeliveryReportDirectory {
    param(
        [Parameter(Mandatory = $true)]
        [string]$ReportsRoot
    )

    return (Join-Path $ReportsRoot (Get-DeliveryTimestamp))
}

function Resolve-DeliveryLauncherPath {
    param(
        [Parameter(Mandatory = $true)]
        [string]$AppDirectory
    )

    if (-not (Test-Path -LiteralPath $AppDirectory)) {
        throw ("Missing app directory: " + $AppDirectory)
    }

    $exeFiles = @(Get-ChildItem -LiteralPath $AppDirectory -Filter "*.exe" -File -ErrorAction Stop | Sort-Object Name)
    if ($exeFiles.Count -eq 0) {
        throw ("No delivery EXE was found in: " + $AppDirectory)
    }

    if ($exeFiles.Count -gt 1) {
        $names = ($exeFiles | Select-Object -ExpandProperty Name) -join ", "
        throw ("Multiple EXE files were found in src\\app. Keep exactly one EXE there: " + $names)
    }

    return $exeFiles[0].FullName
}

function Get-OptionalTextFileValue {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    if (-not (Test-Path -LiteralPath $Path)) {
        return $null
    }

    $content = Get-Content -LiteralPath $Path -Raw -Encoding UTF8
    if ([string]::IsNullOrWhiteSpace($content)) {
        return $null
    }

    return $content.Trim()
}

function Write-SetupFailureArtifact {
    param(
        [Parameter(Mandatory = $true)]
        [string]$ReportsRoot,

        [Parameter(Mandatory = $true)]
        [string]$Message
    )

    if (-not (Test-Path -LiteralPath $ReportsRoot)) {
        New-Item -ItemType Directory -Force -Path $ReportsRoot | Out-Null
    }

    $reportDirectory = Get-DeliveryReportDirectory -ReportsRoot $ReportsRoot
    New-Item -ItemType Directory -Force -Path $reportDirectory | Out-Null

    $errorPath = Join-Path $reportDirectory "setup_error.txt"
    [System.IO.File]::WriteAllText($errorPath, $Message, [System.Text.UTF8Encoding]::new($true))

    [pscustomobject]@{
        report_directory = $reportDirectory
        error_path = $errorPath
    }
}

function New-CustomerReturnArchive {
    param(
        [Parameter(Mandatory = $true)]
        [string]$ReportDirectory
    )

    $reportsRoot = Split-Path -Parent $ReportDirectory
    $archivePath = Join-Path $reportsRoot ("customer_return_" + (Split-Path -Leaf $ReportDirectory) + ".zip")

    if (Test-Path -LiteralPath $archivePath) {
        Remove-Item -LiteralPath $archivePath -Force
    }

    Compress-Archive -LiteralPath $ReportDirectory -DestinationPath $archivePath -Force
    return $archivePath
}
