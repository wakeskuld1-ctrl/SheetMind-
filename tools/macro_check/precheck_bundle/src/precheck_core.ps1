# 2026-04-22 CST: Added a dedicated precheck helper layer so support can
# collect customer environment facts before shipping the real delivery EXE.
#
# 2026-04-22 CST: Keep script literals ASCII-only because Windows PowerShell 5
# can misparse UTF-8 files without BOM when they contain non-ASCII characters.

Set-StrictMode -Version Latest

function Get-PrecheckTimestamp {
    Get-Date -Format "yyyyMMdd_HHmmss"
}

function Get-PrecheckReportDirectory {
    param(
        [Parameter(Mandatory = $true)]
        [string]$ReportsRoot
    )

    return (Join-Path $ReportsRoot (Get-PrecheckTimestamp))
}

function Test-IsAdministrator {
    try {
        $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
        $principal = New-Object Security.Principal.WindowsPrincipal($identity)
        return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    }
    catch {
        return $false
    }
}

function Get-SystemSummary {
    $os = Get-CimInstance -ClassName Win32_OperatingSystem -ErrorAction SilentlyContinue
    $computer = Get-CimInstance -ClassName Win32_ComputerSystem -ErrorAction SilentlyContinue

    [pscustomobject]@{
        computer_name = $env:COMPUTERNAME
        current_user = [System.Security.Principal.WindowsIdentity]::GetCurrent().Name
        is_administrator = Test-IsAdministrator
        os_caption = if ($null -ne $os) { $os.Caption } else { $null }
        os_version = if ($null -ne $os) { $os.Version } else { $null }
        os_build = if ($null -ne $os) { $os.BuildNumber } else { $null }
        os_architecture = if ($null -ne $os) { $os.OSArchitecture } else { $null }
        manufacturer = if ($null -ne $computer) { $computer.Manufacturer } else { $null }
        model = if ($null -ne $computer) { $computer.Model } else { $null }
        total_memory_gb = if ($null -ne $computer) { [math]::Round(($computer.TotalPhysicalMemory / 1GB), 2) } else { $null }
        powershell_version = $PSVersionTable.PSVersion.ToString()
    }
}

function Get-DotNetRuntimeSignals {
    $release = $null
    try {
        $release = (Get-ItemProperty -LiteralPath "Registry::HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\NET Framework Setup\NDP\v4\Full" -ErrorAction Stop).Release
    }
    catch {
    }

    [pscustomobject]@{
        dotnet_framework_release = $release
        dotnet_framework_version_hint = switch ($release) {
            { $_ -ge 533320 } { "4.8.1-or-newer"; break }
            { $_ -ge 528040 } { "4.8"; break }
            { $_ -ge 461808 } { "4.7.2"; break }
            default { $null }
        }
    }
}

function Get-VcRuntimeSignals {
    $paths = @(
        "Registry::HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*",
        "Registry::HKEY_LOCAL_MACHINE\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*"
    )

    $matches = New-Object System.Collections.Generic.List[object]
    foreach ($path in $paths) {
        foreach ($entry in @(Get-ItemProperty -Path $path -ErrorAction SilentlyContinue)) {
            $displayNameProperty = $entry.PSObject.Properties["DisplayName"]
            if ($null -eq $displayNameProperty) {
                continue
            }

            $displayName = [string]$displayNameProperty.Value
            if ([string]::IsNullOrWhiteSpace($displayName)) {
                continue
            }

            $normalized = $displayName.ToLowerInvariant()
            if (-not $normalized.Contains("visual c++")) {
                continue
            }

            $matches.Add([pscustomobject]@{
                display_name = $displayName
                display_version = if ($null -ne $entry.PSObject.Properties["DisplayVersion"]) { [string]$entry.DisplayVersion } else { $null }
            })
        }
    }

    return @($matches.ToArray())
}

function Get-PrecheckOutcome {
    param(
        [Parameter(Mandatory = $true)]
        [psobject]$MacroPayload,

        [Parameter(Mandatory = $true)]
        [psobject]$DotNetSignals,

        [Parameter(Mandatory = $true)]
        [object[]]$VcRuntimeSignals
    )

    $issues = New-Object System.Collections.Generic.List[object]
    foreach ($issue in @($MacroPayload.outcome.issues)) {
        $issues.Add($issue)
    }

    if (@($issues | Where-Object { $_.code -eq "standalone_antivirus_present" }).Count -eq 0 -and @($MacroPayload.security.standalone_antivirus_hits).Count -gt 0) {
        $productNames = (@($MacroPayload.security.standalone_antivirus_hits) | Select-Object -ExpandProperty display_name -Unique) -join ", "
        $issues.Add([pscustomobject]@{
            severity = "warning"
            code = "standalone_antivirus_present"
            message = ("Standalone antivirus traces were detected during precheck: " + $productNames)
        })
    }

    if ($null -eq $DotNetSignals.dotnet_framework_release) {
        $issues.Add([pscustomobject]@{
            severity = "warning"
            code = "dotnet_unknown"
            message = ".NET Framework 4.x release information could not be detected."
        })
    }

    if (@($VcRuntimeSignals).Count -eq 0) {
        $issues.Add([pscustomobject]@{
            severity = "warning"
            code = "vc_runtime_unknown"
            message = "No Visual C++ runtime entry was detected from uninstall records."
        })
    }

    $grade = [string]$MacroPayload.outcome.grade
    $recommendation = "Environment looks broadly ready for a later delivery EXE test."
    if ($grade -eq "high_risk") {
        $recommendation = "Environment has blocking or high-risk signals. Review before shipping the delivery EXE."
    }
    elseif ($grade -eq "runnable_with_risk" -or @($issues | Where-Object { $_.severity -eq "warning" }).Count -gt 0) {
        $grade = "runnable_with_risk"
        $recommendation = "Environment is usable but has risk signals. Review the warnings before shipping the delivery EXE."
    }

    [pscustomobject]@{
        generated_at = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        grade = $grade
        recommendation = $recommendation
        issues = @($issues.ToArray())
    }
}

function ConvertTo-PrecheckReportText {
    param(
        [Parameter(Mandatory = $true)]
        [psobject]$SystemSummary,

        [Parameter(Mandatory = $true)]
        [psobject]$DotNetSignals,

        [Parameter(Mandatory = $true)]
        [object[]]$VcRuntimeSignals,

        [Parameter(Mandatory = $true)]
        [psobject]$MacroPayload,

        [Parameter(Mandatory = $true)]
        [psobject]$Outcome
    )

    $lines = New-Object System.Collections.Generic.List[string]
    $lines.Add("Customer Environment Precheck Report")
    $lines.Add("Generated At: $($Outcome.generated_at)")
    $lines.Add("")
    $lines.Add("Outcome")
    $lines.Add("Grade: $($Outcome.grade)")
    $lines.Add("Recommendation: $($Outcome.recommendation)")
    $lines.Add("")
    $lines.Add("System Summary")
    $lines.Add("Computer Name: $($SystemSummary.computer_name)")
    $lines.Add("Current User: $($SystemSummary.current_user)")
    $lines.Add("Is Administrator: $($SystemSummary.is_administrator)")
    $lines.Add("OS: $($SystemSummary.os_caption)")
    $lines.Add("OS Version: $($SystemSummary.os_version)")
    $lines.Add("OS Build: $($SystemSummary.os_build)")
    $lines.Add("OS Architecture: $($SystemSummary.os_architecture)")
    $lines.Add("Manufacturer: $($SystemSummary.manufacturer)")
    $lines.Add("Model: $($SystemSummary.model)")
    $lines.Add("Memory GB: $($SystemSummary.total_memory_gb)")
    $lines.Add("PowerShell Version: $($SystemSummary.powershell_version)")
    $lines.Add("")
    $lines.Add("Runtime Clues")
    $lines.Add("DotNet Release: $($DotNetSignals.dotnet_framework_release)")
    $lines.Add("DotNet Version Hint: $($DotNetSignals.dotnet_framework_version_hint)")
    if (@($VcRuntimeSignals).Count -eq 0) {
        $lines.Add("VC Runtime Entries: none detected")
    }
    else {
        foreach ($entry in $VcRuntimeSignals) {
            $lines.Add(("VC Runtime: {0} Version={1}" -f $entry.display_name, $entry.display_version))
        }
    }
    $lines.Add("")
    $lines.Add("Macro Environment")
    $lines.Add("Macro Probe Grade: $($MacroPayload.outcome.grade)")
    $lines.Add("Default Host: $($MacroPayload.inventory.default_host)")
    $lines.Add("Excel Installed: $($MacroPayload.inventory.has_excel)")
    $lines.Add("WPS Installed: $($MacroPayload.inventory.has_wps)")
    $lines.Add("Host Started: $($MacroPayload.probe.host_started)")
    $lines.Add("Marker Written: $($MacroPayload.probe.marker_written)")
    $lines.Add("Observed Process: $($MacroPayload.probe.process_name)")
    $lines.Add("Actual Host: $($MacroPayload.probe.actual_host)")
    $lines.Add("")
    $lines.Add("Security Signals")
    foreach ($profile in @($MacroPayload.security.firewall_profiles)) {
        $lines.Add(("Firewall Profile: {0} Enabled={1} Inbound={2} Outbound={3}" -f $profile.name, $profile.enabled, $profile.default_inbound_action, $profile.default_outbound_action))
    }
    if (@($MacroPayload.security.antivirus_products).Count -eq 0) {
        $lines.Add("Security Center Antivirus: none")
    }
    else {
        foreach ($product in @($MacroPayload.security.antivirus_products)) {
            $lines.Add(("Security Center Antivirus: {0} Source={1}" -f $product.display_name, $product.source))
        }
    }
    if (@($MacroPayload.security.standalone_antivirus_hits).Count -eq 0) {
        $lines.Add("Standalone Antivirus Hits: none")
    }
    else {
        foreach ($hit in @($MacroPayload.security.standalone_antivirus_hits)) {
            $lines.Add(("Standalone Antivirus Hit: {0} Source={1} Evidence={2}" -f $hit.display_name, $hit.evidence_source, $hit.evidence_value))
        }
    }
    if (@($MacroPayload.security.office_macro_policies).Count -eq 0) {
        $lines.Add("Office Macro Policies: none")
    }
    else {
        foreach ($policy in @($MacroPayload.security.office_macro_policies)) {
            $lines.Add(("Office Macro Policy: {0} {1} {2}={3} Risk={4} Detail={5}" -f $policy.scope, $policy.office_version, $policy.value_name, $policy.value, $policy.risk_level, $policy.description))
        }
    }
    $lines.Add("")
    $lines.Add("Limits")
    $lines.Add("- This precheck does not prove that the future delivery EXE will run.")
    $lines.Add("- This precheck only estimates readiness from host, macro, and security signals.")
    $lines.Add("")
    $lines.Add("Issues")
    if (@($Outcome.issues).Count -eq 0) {
        $lines.Add("- No obvious issues")
    }
    else {
        foreach ($issue in @($Outcome.issues)) {
            $lines.Add("- [$($issue.severity)] $($issue.code): $($issue.message)")
        }
    }

    return ($lines -join [Environment]::NewLine)
}

function Write-PrecheckArtifacts {
    param(
        [Parameter(Mandatory = $true)]
        [string]$OutputDirectory,

        [Parameter(Mandatory = $true)]
        [psobject]$SystemSummary,

        [Parameter(Mandatory = $true)]
        [psobject]$DotNetSignals,

        [Parameter(Mandatory = $true)]
        [object[]]$VcRuntimeSignals,

        [Parameter(Mandatory = $true)]
        [psobject]$MacroPayload,

        [Parameter(Mandatory = $true)]
        [psobject]$Outcome
    )

    if (-not (Test-Path -LiteralPath $OutputDirectory)) {
        New-Item -ItemType Directory -Force -Path $OutputDirectory | Out-Null
    }

    $txtPath = Join-Path $OutputDirectory "precheck_report.txt"
    $jsonPath = Join-Path $OutputDirectory "precheck_report.json"
    $reportText = ConvertTo-PrecheckReportText -SystemSummary $SystemSummary -DotNetSignals $DotNetSignals -VcRuntimeSignals $VcRuntimeSignals -MacroPayload $MacroPayload -Outcome $Outcome
    [System.IO.File]::WriteAllText($txtPath, $reportText, [System.Text.UTF8Encoding]::new($true))

    $payload = [pscustomobject]@{
        system = $SystemSummary
        dotnet = $DotNetSignals
        vc_runtime = @($VcRuntimeSignals)
        macro_payload = $MacroPayload
        outcome = $Outcome
    }
    $json = $payload | ConvertTo-Json -Depth 10
    [System.IO.File]::WriteAllText($jsonPath, $json, [System.Text.UTF8Encoding]::new($true))

    [pscustomobject]@{
        txt_path = $txtPath
        json_path = $jsonPath
    }
}

function New-PrecheckReturnArchive {
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
