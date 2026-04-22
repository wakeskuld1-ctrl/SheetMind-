# 2026-04-22 CST: Added a dedicated asset builder so the repository can keep
# the shipped probe workbook reproducible from VBA source files.
#
# 2026-04-22 CST: Keep script literals ASCII-only because Windows PowerShell 5
# can misparse UTF-8 files without BOM when they contain non-ASCII characters.

param(
    [string]$OutputWorkbookPath = "D:\Rust\Excel_Skill\tools\macro_check\assets\macro_probe.xlsm"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$toolRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$vbaRoot = Join-Path $toolRoot "assets\vba"
$thisWorkbookPath = Join-Path $vbaRoot "ThisWorkbookCode.txt"

function Normalize-VbaSource {
    param(
        [Parameter(Mandatory = $true)]
        [string]$SourceText
    )

    $normalized = $SourceText -replace "`r`n", "`n"
    $normalized = $normalized -replace "^Attribute VB_Name = .*$`n?", ""
    $normalized = $normalized.Replace([char]0x201C, '"')
    $normalized = $normalized.Replace([char]0x201D, '"')
    $normalized = $normalized.Replace([char]0x2018, "'")
    $normalized = $normalized.Replace([char]0x2019, "'")
    $normalized = $normalized.TrimStart([char]0xFEFF)
    return $normalized
}

function Write-CodeModuleText {
    param(
        [Parameter(Mandatory = $true)]
        [object]$CodeModule,

        [Parameter(Mandatory = $true)]
        [string]$SourceText
    )

    $lineCount = $CodeModule.CountOfLines
    if ($lineCount -gt 0) {
        $CodeModule.DeleteLines(1, $lineCount)
    }

    $lines = $SourceText -split "`n", 0, "SimpleMatch"
    for ($index = 0; $index -lt $lines.Count; $index++) {
        $line = $lines[$index].TrimEnd("`r")
        $CodeModule.InsertLines($index + 1, $line)
    }
}

$thisWorkbookSource = Normalize-VbaSource -SourceText (Get-Content -LiteralPath $thisWorkbookPath -Raw -Encoding UTF8)

$outputDir = Split-Path -Parent $OutputWorkbookPath
if (-not (Test-Path -LiteralPath $outputDir)) {
    New-Item -ItemType Directory -Force -Path $outputDir | Out-Null
}

if (Test-Path -LiteralPath $OutputWorkbookPath) {
    Remove-Item -LiteralPath $OutputWorkbookPath -Force
}

$excel = $null
$workbook = $null

try {
    $excel = New-Object -ComObject Excel.Application
    $excel.Visible = $false
    $excel.DisplayAlerts = $false
    $workbook = $excel.Workbooks.Add()

    $sheet = $workbook.Worksheets.Item(1)
    $sheet.Name = "MacroProbe"
    $sheet.Range("A1").Value2 = "Macro probe workbook"
    $sheet.Range("A2").Value2 = "Open this file and wait for it to close itself."

    $vbProject = $workbook.VBProject
    $thisWorkbookComponent = $vbProject.VBComponents.Item("ThisWorkbook")
    Write-CodeModuleText -CodeModule $thisWorkbookComponent.CodeModule -SourceText $thisWorkbookSource

    $workbook.SaveAs($OutputWorkbookPath, 52)
    $workbook.Close($false)
    $workbook = $null

    Write-Output "macro_probe_asset=$OutputWorkbookPath"
}
finally {
    if ($workbook -ne $null) {
        $workbook.Close($false)
        [System.Runtime.InteropServices.Marshal]::ReleaseComObject($workbook) | Out-Null
    }
    if ($excel -ne $null) {
        $excel.Quit()
        [System.Runtime.InteropServices.Marshal]::ReleaseComObject($excel) | Out-Null
    }
    [gc]::Collect()
    [gc]::WaitForPendingFinalizers()
}
