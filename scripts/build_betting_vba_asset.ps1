param(
    [string]$OutputVbaProjectPath = "D:\Rust\Excel_Skill\assets\excel_templates\betting_optimizer\vbaProject.bin"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# 2026-04-20 CST: Keep this builder ASCII-only because Windows PowerShell 5 can
# misparse UTF-8 script literals without BOM; the form shell stays structural
# here and the localized captions are initialized from VBA source at runtime.

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Split-Path -Parent $scriptRoot
$vbaRoot = Join-Path $repoRoot "assets\excel_templates\betting_optimizer\vba"
$moduleSourcePath = Join-Path $vbaRoot "BettingSolverRunner.bas"
$formCodePath = Join-Path $vbaRoot "SolverProgressFormCode.bas"

$moduleSource = Get-Content -LiteralPath $moduleSourcePath -Raw -Encoding UTF8
$formCode = Get-Content -LiteralPath $formCodePath -Raw -Encoding UTF8

$outputDir = Split-Path -Parent $OutputVbaProjectPath
if (-not (Test-Path -LiteralPath $outputDir)) {
    New-Item -ItemType Directory -Force -Path $outputDir | Out-Null
}

$tempRoot = Join-Path $repoRoot ".codex_tmp\betting_vba_build"
New-Item -ItemType Directory -Force -Path $tempRoot | Out-Null
$tempWorkbookPath = Join-Path $tempRoot "betting_macro_asset.xlsm"
if (Test-Path -LiteralPath $tempWorkbookPath) {
    Remove-Item -LiteralPath $tempWorkbookPath -Force
}

$excel = $null
$workbook = $null

try {
    $excel = New-Object -ComObject Excel.Application
    $excel.Visible = $false
    $excel.DisplayAlerts = $false
    $workbook = $excel.Workbooks.Add()
    $vbProject = $workbook.VBProject

    $runnerModule = $vbProject.VBComponents.Add(1)
    $runnerModule.Name = "BettingSolverRunner"
    $runnerModule.CodeModule.AddFromString($moduleSource)

    $progressForm = $vbProject.VBComponents.Add(3)
    $progressForm.Name = "SolverProgressForm"
    $designer = $progressForm.Designer

    $designer.Caption = "Solver Progress"
    $statusLabel = $designer.Controls.Add("Forms.Label.1", "lblStatus", $true)
    $statusLabel.Left = 18
    $statusLabel.Top = 18
    $statusLabel.Width = 232
    $statusLabel.Height = 48
    $statusLabel.WordWrap = $true
    $statusLabel.Caption = "Status"

    $hintLabel = $designer.Controls.Add("Forms.Label.1", "lblHint", $true)
    $hintLabel.Left = 18
    $hintLabel.Top = 72
    $hintLabel.Width = 232
    $hintLabel.Height = 18
    $hintLabel.Caption = "Hint"

    $cancelButton = $designer.Controls.Add("Forms.CommandButton.1", "btnCancel", $true)
    $cancelButton.Left = 98
    $cancelButton.Top = 98
    $cancelButton.Width = 76
    $cancelButton.Height = 26
    $cancelButton.Caption = "Cancel"

    $progressForm.CodeModule.AddFromString($formCode)

    $workbook.SaveAs($tempWorkbookPath, 52)
    $workbook.Close($false)
    $workbook = $null

    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $zip = [System.IO.Compression.ZipFile]::OpenRead($tempWorkbookPath)
    try {
        $entry = $zip.GetEntry("xl/vbaProject.bin")
        if ($null -eq $entry) {
            throw "missing xl/vbaProject.bin in generated workbook"
        }

        $stream = $entry.Open()
        try {
            $fileStream = [System.IO.File]::Create($OutputVbaProjectPath)
            try {
                $stream.CopyTo($fileStream)
            }
            finally {
                $fileStream.Dispose()
            }
        }
        finally {
            $stream.Dispose()
        }
    }
    finally {
        $zip.Dispose()
    }

    Write-Output "built_vba_asset=$OutputVbaProjectPath"
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
