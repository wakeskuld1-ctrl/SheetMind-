# 2026-04-22 CST: Added a dedicated macro check core so the delivery probe can
# keep host detection, risk grading, and report rendering testable outside the
# workbook-open flow.
#
# 2026-04-22 CST: Keep script literals ASCII-only because Windows PowerShell 5
# can misparse UTF-8 files without BOM when they contain non-ASCII characters.

Set-StrictMode -Version Latest

function New-MacroCheckIssue {
    param(
        [Parameter(Mandatory = $true)]
        [ValidateSet("info", "warning", "high_risk")]
        [string]$Severity,

        [Parameter(Mandatory = $true)]
        [string]$Code,

        [Parameter(Mandatory = $true)]
        [string]$Message
    )

    [pscustomobject]@{
        severity = $Severity
        code = $Code
        message = $Message
    }
}

function Get-MacroCheckTimestamp {
    Get-Date -Format "yyyy-MM-dd HH:mm:ss"
}

function Get-RegistryStringValue {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path,

        [Parameter(Mandatory = $true)]
        [string]$Name
    )

    try {
        $item = Get-ItemProperty -LiteralPath $Path -ErrorAction Stop
        $value = $item.$Name
        if ($null -eq $value) {
            return $null
        }
        return [string]$value
    }
    catch {
        return $null
    }
}

function Get-OptionalPropertyValue {
    param(
        [Parameter(Mandatory = $true)]
        [object]$InputObject,

        [Parameter(Mandatory = $true)]
        [string]$PropertyName
    )

    if ($null -eq $InputObject) {
        return $null
    }

    $property = $InputObject.PSObject.Properties[$PropertyName]
    if ($null -eq $property) {
        return $null
    }

    return $property.Value
}

function Resolve-CommandPathIfExists {
    param(
        [Parameter(Mandatory = $true)]
        [string]$CommandName
    )

    try {
        $command = Get-Command $CommandName -ErrorAction Stop
        return $command.Source
    }
    catch {
        return $null
    }
}

function Test-HostPathAlreadyTracked {
    param(
        [Parameter()]
        [object[]]$Hosts,

        [Parameter(Mandatory = $true)]
        [string]$CandidatePath
    )

    foreach ($trackedHost in $Hosts) {
        if ($null -ne $trackedHost -and $trackedHost.executable_path -eq $CandidatePath) {
            return $true
        }
    }

    return $false
}

function Get-FileAssociationCommand {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Extension
    )

    $userChoicePath = "Registry::HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\$Extension\UserChoice"
    $progId = Get-RegistryStringValue -Path $userChoicePath -Name "ProgId"
    if ([string]::IsNullOrWhiteSpace($progId)) {
        $progId = Get-RegistryStringValue -Path "Registry::HKEY_CLASSES_ROOT\$Extension" -Name "(default)"
    }

    if ([string]::IsNullOrWhiteSpace($progId)) {
        return [pscustomobject]@{
            extension = $Extension
            prog_id = $null
            open_command = $null
        }
    }

    $commandPath = "Registry::HKEY_CLASSES_ROOT\$progId\shell\open\command"
    $openCommand = Get-RegistryStringValue -Path $commandPath -Name "(default)"
    [pscustomobject]@{
        extension = $Extension
        prog_id = $progId
        open_command = $openCommand
    }
}

function Get-InstalledSpreadsheetHosts {
    $hosts = @()

    $excelPath = Resolve-CommandPathIfExists -CommandName "EXCEL.EXE"
    if (-not [string]::IsNullOrWhiteSpace($excelPath)) {
        $hosts += [pscustomobject]@{
            host_key = "excel"
            display_name = "Microsoft Excel"
            executable_path = $excelPath
            source = "command"
        }
    }

    $wpsCandidates = @(
        (Resolve-CommandPathIfExists -CommandName "et.exe"),
        (Resolve-CommandPathIfExists -CommandName "wps.exe")
    ) | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }

    foreach ($candidate in $wpsCandidates | Select-Object -Unique) {
        $hosts += [pscustomobject]@{
            host_key = "wps"
            display_name = "WPS Spreadsheets"
            executable_path = $candidate
            source = "command"
        }
    }

    $officeAppPaths = @(
        "Registry::HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\EXCEL.EXE",
        "Registry::HKEY_LOCAL_MACHINE\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\App Paths\EXCEL.EXE"
    )
    foreach ($path in $officeAppPaths) {
        $resolved = Get-RegistryStringValue -Path $path -Name "(default)"
        if (-not [string]::IsNullOrWhiteSpace($resolved) -and -not (Test-HostPathAlreadyTracked -Hosts $hosts -CandidatePath $resolved)) {
            $hosts += [pscustomobject]@{
                host_key = "excel"
                display_name = "Microsoft Excel"
                executable_path = $resolved
                source = "registry"
            }
        }
    }

    $wpsRegistryCandidates = @(
        "Registry::HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\et.exe",
        "Registry::HKEY_LOCAL_MACHINE\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\App Paths\et.exe"
    )
    foreach ($path in $wpsRegistryCandidates) {
        $resolved = Get-RegistryStringValue -Path $path -Name "(default)"
        if (-not [string]::IsNullOrWhiteSpace($resolved) -and -not (Test-HostPathAlreadyTracked -Hosts $hosts -CandidatePath $resolved)) {
            $hosts += [pscustomobject]@{
                host_key = "wps"
                display_name = "WPS Spreadsheets"
                executable_path = $resolved
                source = "registry"
            }
        }
    }

    return $hosts
}

function Get-SpreadsheetHostInventory {
    $hosts = Get-InstalledSpreadsheetHosts
    $xlsmAssociation = Get-FileAssociationCommand -Extension ".xlsm"
    $xlsxAssociation = Get-FileAssociationCommand -Extension ".xlsx"

    $defaultHost = "unknown"
    $openCommand = $xlsmAssociation.open_command
    if (-not [string]::IsNullOrWhiteSpace($openCommand)) {
        $normalized = $openCommand.ToLowerInvariant()
        if ($normalized.Contains("excel")) {
            $defaultHost = "excel"
        }
        elseif ($normalized.Contains("\et.exe") -or $normalized.Contains("\wps.exe")) {
            $defaultHost = "wps"
        }
    }

    [pscustomobject]@{
        discovered_at = Get-MacroCheckTimestamp
        hosts = @($hosts)
        xlsm_association = $xlsmAssociation
        xlsx_association = $xlsxAssociation
        default_host = $defaultHost
        has_excel = [bool]($hosts | Where-Object { $_.host_key -eq "excel" })
        has_wps = [bool]($hosts | Where-Object { $_.host_key -eq "wps" })
    }
}

function Get-PreferredHostRecord {
    param(
        [Parameter(Mandatory = $true)]
        [psobject]$Inventory,

        [Parameter(Mandatory = $true)]
        [ValidateSet("default", "excel", "wps")]
        [string]$PreferredHost
    )

    if ($PreferredHost -eq "default") {
        return $null
    }

    foreach ($candidate in $Inventory.hosts) {
        if ($candidate.host_key -eq $PreferredHost) {
            return $candidate
        }
    }

    return $null
}

# 2026-04-22 CST: Added launcher argument templating so delivery EXEs can pass
# the runtime probe workbook path through a stable placeholder contract.
function Resolve-LauncherArgumentsTemplate {
    param(
        [AllowNull()]
        [string]$Template,

        [Parameter(Mandatory = $true)]
        [string]$ProbeWorkbookPath
    )

    $quotedProbePath = '"' + $ProbeWorkbookPath + '"'
    if ([string]::IsNullOrWhiteSpace($Template)) {
        return $quotedProbePath
    }

    if ($Template.Contains("{probe_workbook}")) {
        $resolved = $Template.Replace('"{probe_workbook}"', $quotedProbePath)
        return $resolved.Replace("{probe_workbook}", $quotedProbePath)
    }

    return ($Template.TrimEnd() + " " + $quotedProbePath).Trim()
}

function Get-FirewallProfileSignals {
    if ($null -eq (Get-Command Get-NetFirewallProfile -ErrorAction SilentlyContinue)) {
        return @()
    }

    try {
        return @(Get-NetFirewallProfile | ForEach-Object {
            [pscustomobject]@{
                name = $_.Name
                enabled = [bool]$_.Enabled
                default_inbound_action = [string]$_.DefaultInboundAction
                default_outbound_action = [string]$_.DefaultOutboundAction
            }
        })
    }
    catch {
        return @()
    }
}

function Get-FirewallRulesForProgram {
    param(
        [AllowNull()]
        [string]$ProgramPath
    )

    if ([string]::IsNullOrWhiteSpace($ProgramPath)) {
        return @()
    }

    if ($null -eq (Get-Command Get-NetFirewallApplicationFilter -ErrorAction SilentlyContinue)) {
        return @()
    }

    if ($null -eq (Get-Command Get-NetFirewallRule -ErrorAction SilentlyContinue)) {
        return @()
    }

    $resolvedPath = $ProgramPath
    try {
        $resolvedPath = (Resolve-Path -LiteralPath $ProgramPath -ErrorAction Stop).Path
    }
    catch {
    }

    try {
        $filters = @(Get-NetFirewallApplicationFilter -PolicyStore ActiveStore | Where-Object {
            -not [string]::IsNullOrWhiteSpace($_.Program) -and $_.Program.Equals($resolvedPath, [System.StringComparison]::OrdinalIgnoreCase)
        })
        if ($filters.Count -eq 0) {
            return @()
        }

        $rules = @(Get-NetFirewallRule -PolicyStore ActiveStore)
        $matches = New-Object System.Collections.Generic.List[object]
        foreach ($filter in $filters) {
            foreach ($rule in $rules) {
                if ($rule.InstanceID -ne $filter.InstanceID) {
                    continue
                }

                $matches.Add([pscustomobject]@{
                    display_name = $rule.DisplayName
                    action = [string]$rule.Action
                    enabled = [bool]$rule.Enabled
                    direction = [string]$rule.Direction
                    program = $filter.Program
                })
            }
        }

        return $matches.ToArray()
    }
    catch {
        return @()
    }
}

function Get-SecurityCenterAntivirusProducts {
    try {
        return @(Get-CimInstance -Namespace "root/SecurityCenter2" -ClassName "AntiVirusProduct" -ErrorAction Stop | ForEach-Object {
            [pscustomobject]@{
                display_name = $_.displayName
                product_exe = $_.pathToSignedProductExe
                reporting_exe = $_.pathToSignedReportingExe
                source = "security_center"
            }
        })
    }
    catch {
        return @()
    }
}

function Get-UninstallDisplayEntries {
    $paths = @(
        "Registry::HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*",
        "Registry::HKEY_LOCAL_MACHINE\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*",
        "Registry::HKEY_CURRENT_USER\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*"
    )

    $entries = New-Object System.Collections.Generic.List[object]
    foreach ($path in $paths) {
        foreach ($entry in @(Get-ItemProperty -Path $path -ErrorAction SilentlyContinue)) {
            $displayName = [string](Get-OptionalPropertyValue -InputObject $entry -PropertyName "DisplayName")
            if ([string]::IsNullOrWhiteSpace($displayName)) {
                continue
            }

            $entries.Add([pscustomobject]@{
                name = $displayName
                publisher = [string](Get-OptionalPropertyValue -InputObject $entry -PropertyName "Publisher")
                source = "uninstall"
            })
        }
    }

    return $entries.ToArray()
}

function Get-ServiceDisplayEntries {
    $entries = New-Object System.Collections.Generic.List[object]
    foreach ($service in @(Get-Service -ErrorAction SilentlyContinue)) {
        $combined = @($service.Name, $service.DisplayName) -join " "
        if ([string]::IsNullOrWhiteSpace($combined.Trim())) {
            continue
        }

        $entries.Add([pscustomobject]@{
            name = $combined
            publisher = $null
            source = "service"
        })
    }

    return $entries.ToArray()
}

function Get-ProcessDisplayEntries {
    $entries = New-Object System.Collections.Generic.List[object]
    foreach ($process in @(Get-Process -ErrorAction SilentlyContinue)) {
        $path = $null
        try {
            $path = $process.Path
        }
        catch {
        }

        $combined = @($process.ProcessName, $path) -join " "
        if ([string]::IsNullOrWhiteSpace($combined.Trim())) {
            continue
        }

        $entries.Add([pscustomobject]@{
            name = $combined
            publisher = $null
            source = "process"
        })
    }

    return $entries.ToArray()
}

function Get-StandaloneAntivirusDefinitions {
    return @(
        [pscustomobject]@{ vendor_key = "360"; display_name = "360 Total Security"; patterns = @("360 total security", "360safe", "360tray", "360sd", "zhudongfangyu") },
        [pscustomobject]@{ vendor_key = "kingsoft"; display_name = "Kingsoft Antivirus"; patterns = @("kingsoft", "ksafe", "kxescore") },
        [pscustomobject]@{ vendor_key = "huorong"; display_name = "Huorong Security"; patterns = @("huorong", "hipstray", "hipsdaemon", "hipsmain") },
        [pscustomobject]@{ vendor_key = "qqpc"; display_name = "Tencent PC Manager"; patterns = @("qqpc", "qqpctray", "tencent pc manager") },
        [pscustomobject]@{ vendor_key = "mcafee"; display_name = "McAfee"; patterns = @("mcafee") },
        [pscustomobject]@{ vendor_key = "symantec"; display_name = "Symantec"; patterns = @("symantec") },
        [pscustomobject]@{ vendor_key = "norton"; display_name = "Norton"; patterns = @("norton") },
        [pscustomobject]@{ vendor_key = "kaspersky"; display_name = "Kaspersky"; patterns = @("kaspersky", "avp.exe") }
    )
}

function Get-StandaloneAntivirusHits {
    $definitions = Get-StandaloneAntivirusDefinitions
    $candidateEntries = @()
    $candidateEntries += Get-UninstallDisplayEntries
    $candidateEntries += Get-ServiceDisplayEntries
    $candidateEntries += Get-ProcessDisplayEntries

    $hits = New-Object System.Collections.Generic.List[object]
    $seenKeys = New-Object System.Collections.Generic.HashSet[string]

    foreach ($entry in $candidateEntries) {
        $haystack = @($entry.name, $entry.publisher) -join " "
        $normalized = $haystack.ToLowerInvariant()

        foreach ($definition in $definitions) {
            $matched = $false
            foreach ($pattern in $definition.patterns) {
                if ($normalized.Contains($pattern.ToLowerInvariant())) {
                    $matched = $true
                    break
                }
            }

            if (-not $matched) {
                continue
            }

            $key = $definition.vendor_key + "|" + $entry.source + "|" + $entry.name
            if ($seenKeys.Add($key)) {
                $hits.Add([pscustomobject]@{
                    vendor_key = $definition.vendor_key
                    display_name = $definition.display_name
                    evidence_source = $entry.source
                    evidence_value = $entry.name
                })
            }
        }
    }

    return $hits.ToArray()
}

function Get-OfficeMacroPolicySignals {
    $policyRoots = @(
        @{ scope = "HKCU"; path = "Registry::HKEY_CURRENT_USER\Software\Microsoft\Office" },
        @{ scope = "HKLM"; path = "Registry::HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Office" },
        @{ scope = "HKLM"; path = "Registry::HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Microsoft\Office" },
        @{ scope = "HKLM"; path = "Registry::HKEY_LOCAL_MACHINE\SOFTWARE\WOW6432Node\Microsoft\Office" },
        @{ scope = "HKLM"; path = "Registry::HKEY_LOCAL_MACHINE\SOFTWARE\WOW6432Node\Policies\Microsoft\Office" }
    )

    $signals = New-Object System.Collections.Generic.List[object]
    foreach ($root in $policyRoots) {
        if (-not (Test-Path -LiteralPath $root.path)) {
            continue
        }

        foreach ($versionKey in @(Get-ChildItem -LiteralPath $root.path -ErrorAction SilentlyContinue)) {
            $securityPath = Join-Path $versionKey.PSPath "Excel\Security"
            if (-not (Test-Path -LiteralPath $securityPath)) {
                continue
            }

            try {
                $securityItem = Get-ItemProperty -LiteralPath $securityPath -ErrorAction Stop
            }
            catch {
                continue
            }

            foreach ($valueName in @("VBAWarnings", "BlockMacrosFromInternet", "AccessVBOM")) {
                $value = Get-OptionalPropertyValue -InputObject $securityItem -PropertyName $valueName
                if ($null -eq $value) {
                    continue
                }

                $riskLevel = "info"
                $description = [string]$value
                if ($valueName -eq "VBAWarnings") {
                    switch ([int]$value) {
                        4 {
                            $riskLevel = "high_risk"
                            $description = "Disable all macros without notification."
                        }
                        3 {
                            $riskLevel = "warning"
                            $description = "Only signed macros are allowed."
                        }
                        2 {
                            $riskLevel = "warning"
                            $description = "Disable macros with notification."
                        }
                    }
                }
                elseif ($valueName -eq "BlockMacrosFromInternet" -and [int]$value -eq 1) {
                    $riskLevel = "warning"
                    $description = "Block macros in files from the Internet."
                }

                $signals.Add([pscustomobject]@{
                    scope = $root.scope
                    office_version = $versionKey.PSChildName
                    application = "Excel"
                    value_name = $valueName
                    value = $value
                    risk_level = $riskLevel
                    description = $description
                })
            }
        }
    }

    return $signals.ToArray()
}

function Get-MacroSecuritySignals {
    param(
        [AllowNull()]
        [string]$LauncherPath
    )

    [pscustomobject]@{
        firewall_profiles = @(Get-FirewallProfileSignals)
        launcher_firewall_rules = @(Get-FirewallRulesForProgram -ProgramPath $LauncherPath)
        antivirus_products = @(Get-SecurityCenterAntivirusProducts)
        standalone_antivirus_hits = @(Get-StandaloneAntivirusHits)
        office_macro_policies = @(Get-OfficeMacroPolicySignals)
    }
}

function Get-MacroCheckOutcome {
    param(
        [Parameter(Mandatory = $true)]
        [psobject]$Inventory,

        [Parameter(Mandatory = $true)]
        [psobject]$ProbeResult,

        [Parameter(Mandatory = $true)]
        [psobject]$LaunchResult,

        [Parameter(Mandatory = $true)]
        [psobject]$SecuritySignals
    )

    $issues = New-Object System.Collections.Generic.List[object]

    if (-not $Inventory.has_excel -and -not $Inventory.has_wps) {
        $issues.Add((New-MacroCheckIssue -Severity "high_risk" -Code "host_missing" -Message "No Excel or WPS host was detected."))
    }

    if ($Inventory.default_host -eq "unknown") {
        $issues.Add((New-MacroCheckIssue -Severity "warning" -Code "association_unknown" -Message "The default .xlsm association could not be identified."))
    }

    if ($Inventory.has_excel -and $Inventory.has_wps) {
        $issues.Add((New-MacroCheckIssue -Severity "warning" -Code "dual_host" -Message "Excel and WPS are both installed, so the default host may affect the probe result."))
    }

    if ($ProbeResult.requested_host -ne "default" -and -not $ProbeResult.requested_host_available) {
        $issues.Add((New-MacroCheckIssue -Severity "high_risk" -Code "requested_host_missing" -Message ("The requested host was not available: " + $ProbeResult.requested_host)))
    }

    if ($ProbeResult.requested_host -ne "default" -and $ProbeResult.actual_host -and $ProbeResult.actual_host -ne $ProbeResult.requested_host) {
        $issues.Add((New-MacroCheckIssue -Severity "warning" -Code "requested_host_mismatch" -Message ("The requested host was " + $ProbeResult.requested_host + ", but the observed host was " + $ProbeResult.actual_host)))
    }

    if ($LaunchResult.mode -eq "launcher" -and -not $LaunchResult.launcher_started) {
        $issues.Add((New-MacroCheckIssue -Severity "high_risk" -Code "launcher_not_started" -Message ("The delivery launcher did not start: " + $LaunchResult.launcher_path)))
    }

    if (-not $ProbeResult.host_started -and $ProbeResult.requested_host_available) {
        $issues.Add((New-MacroCheckIssue -Severity "high_risk" -Code "host_not_started" -Message "The probe workbook did not start the default spreadsheet host."))
    }

    if ($ProbeResult.host_started -and -not $ProbeResult.marker_written) {
        $issues.Add((New-MacroCheckIssue -Severity "high_risk" -Code "marker_missing" -Message "The host started, but the macro probe did not write its marker file."))
    }

    if ($ProbeResult.blocking_hint) {
        $issues.Add((New-MacroCheckIssue -Severity "warning" -Code "blocking_hint" -Message $ProbeResult.blocking_hint))
    }

    $blockRules = @($SecuritySignals.launcher_firewall_rules | Where-Object { $_.enabled -and $_.action -eq "Block" })
    if ($blockRules.Count -gt 0) {
        $ruleNames = ($blockRules | Select-Object -ExpandProperty display_name -Unique) -join ", "
        $issues.Add((New-MacroCheckIssue -Severity "high_risk" -Code "launcher_firewall_block" -Message ("Active firewall block rules were found for the launcher: " + $ruleNames)))
    }

    $standaloneHits = @($SecuritySignals.standalone_antivirus_hits)
    if ($standaloneHits.Count -gt 0) {
        $productNames = ($standaloneHits | Select-Object -ExpandProperty display_name -Unique) -join ", "
        $issues.Add((New-MacroCheckIssue -Severity "warning" -Code "standalone_antivirus_present" -Message ("Standalone antivirus traces were detected: " + $productNames)))
    }

    $highRiskPolicies = @($SecuritySignals.office_macro_policies | Where-Object { $_.risk_level -eq "high_risk" })
    if ($highRiskPolicies.Count -gt 0) {
        $policySummary = ($highRiskPolicies | ForEach-Object { $_.office_version + ":" + $_.value_name + "=" + $_.value }) -join ", "
        $severity = "warning"
        if (-not $ProbeResult.marker_written) {
            $severity = "high_risk"
        }

        $issues.Add((New-MacroCheckIssue -Severity $severity -Code "office_macro_policy" -Message ("Restrictive Excel macro policy hints were detected: " + $policySummary)))
    }

    $severityOrder = @{
        info = 0
        warning = 1
        high_risk = 2
    }
    $maxSeverity = 0
    foreach ($issue in $issues) {
        if ($severityOrder[$issue.severity] -gt $maxSeverity) {
            $maxSeverity = $severityOrder[$issue.severity]
        }
    }

    $grade = "runnable"
    if ($maxSeverity -ge 2) {
        $grade = "high_risk"
    }
    elseif ($maxSeverity -eq 1) {
        $grade = "runnable_with_risk"
    }

    [pscustomobject]@{
        generated_at = Get-MacroCheckTimestamp
        grade = $grade
        issues = $issues.ToArray()
    }
}

function ConvertTo-MacroCheckReportText {
    param(
        [Parameter(Mandatory = $true)]
        [psobject]$Inventory,

        [Parameter(Mandatory = $true)]
        [psobject]$ProbeResult,

        [Parameter(Mandatory = $true)]
        [psobject]$LaunchResult,

        [Parameter(Mandatory = $true)]
        [psobject]$SecuritySignals,

        [Parameter(Mandatory = $true)]
        [psobject]$Outcome
    )

    $lines = New-Object System.Collections.Generic.List[string]
    $lines.Add("Macro Launch Readiness Report")
    $lines.Add("Generated At: $($Outcome.generated_at)")
    $lines.Add("")
    $lines.Add("Outcome")
    $lines.Add("Grade: $($Outcome.grade)")
    $lines.Add("")
    $lines.Add("Host Inventory")
    $lines.Add("Default Host: $($Inventory.default_host)")
    $lines.Add("Excel Installed: $($Inventory.has_excel)")
    $lines.Add("WPS Installed: $($Inventory.has_wps)")
    $lines.Add("XLSM Association: $($Inventory.xlsm_association.open_command)")
    $lines.Add("")
    $lines.Add("Launcher")
    $lines.Add("Launch Mode: $($LaunchResult.mode)")
    $lines.Add("Launcher Path: $($LaunchResult.launcher_path)")
    $lines.Add("Launcher Arguments Template: $($LaunchResult.launcher_arguments)")
    $lines.Add("Launcher Resolved Arguments: $($LaunchResult.launcher_resolved_arguments)")
    $lines.Add("Launcher Working Directory: $($LaunchResult.launcher_working_directory)")
    $lines.Add("Launcher Started: $($LaunchResult.launcher_started)")
    $lines.Add("Launcher Process: $($LaunchResult.launcher_process_name)")
    $lines.Add("")
    $lines.Add("Dynamic Probe")
    $lines.Add("Requested Host: $($ProbeResult.requested_host)")
    $lines.Add("Requested Host Available: $($ProbeResult.requested_host_available)")
    $lines.Add("Host Started: $($ProbeResult.host_started)")
    $lines.Add("Marker Written: $($ProbeResult.marker_written)")
    $lines.Add("Marker Path: $($ProbeResult.marker_path)")
    $lines.Add("Observed Process: $($ProbeResult.process_name)")
    $lines.Add("Actual Host: $($ProbeResult.actual_host)")
    $lines.Add("Blocking Hint: $($ProbeResult.blocking_hint)")
    $lines.Add("")
    $lines.Add("Security Signals")

    if (@($SecuritySignals.firewall_profiles).Count -eq 0) {
        $lines.Add("Firewall Profiles: none collected")
    }
    else {
        foreach ($profile in $SecuritySignals.firewall_profiles) {
            $lines.Add(("Firewall Profile: {0} Enabled={1} Inbound={2} Outbound={3}" -f $profile.name, $profile.enabled, $profile.default_inbound_action, $profile.default_outbound_action))
        }
    }

    if (@($SecuritySignals.launcher_firewall_rules).Count -eq 0) {
        $lines.Add("Launcher Firewall Rules: none")
    }
    else {
        foreach ($rule in $SecuritySignals.launcher_firewall_rules) {
            $lines.Add(("Launcher Firewall Rule: {0} Action={1} Enabled={2} Direction={3} Program={4}" -f $rule.display_name, $rule.action, $rule.enabled, $rule.direction, $rule.program))
        }
    }

    if (@($SecuritySignals.antivirus_products).Count -eq 0) {
        $lines.Add("Security Center Antivirus: none")
    }
    else {
        foreach ($product in $SecuritySignals.antivirus_products) {
            $lines.Add(("Security Center Antivirus: {0} Source={1}" -f $product.display_name, $product.source))
        }
    }

    if (@($SecuritySignals.standalone_antivirus_hits).Count -eq 0) {
        $lines.Add("Standalone Antivirus Hits: none")
    }
    else {
        foreach ($hit in $SecuritySignals.standalone_antivirus_hits) {
            $lines.Add(("Standalone Antivirus Hit: {0} Source={1} Evidence={2}" -f $hit.display_name, $hit.evidence_source, $hit.evidence_value))
        }
    }

    if (@($SecuritySignals.office_macro_policies).Count -eq 0) {
        $lines.Add("Office Macro Policies: none")
    }
    else {
        foreach ($policy in $SecuritySignals.office_macro_policies) {
            $lines.Add(("Office Macro Policy: {0} {1} {2}={3} Risk={4} Detail={5}" -f $policy.scope, $policy.office_version, $policy.value_name, $policy.value, $policy.risk_level, $policy.description))
        }
    }

    $lines.Add("")
    $lines.Add("Issues")

    if ($Outcome.issues.Count -eq 0) {
        $lines.Add("- No obvious issues")
    }
    else {
        foreach ($issue in $Outcome.issues) {
            $lines.Add("- [$($issue.severity)] $($issue.code): $($issue.message)")
        }
    }

    return ($lines -join [Environment]::NewLine)
}

function Write-MacroCheckArtifacts {
    param(
        [Parameter(Mandatory = $true)]
        [string]$OutputDirectory,

        [Parameter(Mandatory = $true)]
        [psobject]$Inventory,

        [Parameter(Mandatory = $true)]
        [psobject]$ProbeResult,

        [Parameter(Mandatory = $true)]
        [psobject]$LaunchResult,

        [Parameter(Mandatory = $true)]
        [psobject]$SecuritySignals,

        [Parameter(Mandatory = $true)]
        [psobject]$Outcome
    )

    if (-not (Test-Path -LiteralPath $OutputDirectory)) {
        New-Item -ItemType Directory -Force -Path $OutputDirectory | Out-Null
    }

    $txtPath = Join-Path $OutputDirectory "macro_check_report.txt"
    $jsonPath = Join-Path $OutputDirectory "macro_check_report.json"

    $reportText = ConvertTo-MacroCheckReportText -Inventory $Inventory -ProbeResult $ProbeResult -LaunchResult $LaunchResult -SecuritySignals $SecuritySignals -Outcome $Outcome
    [System.IO.File]::WriteAllText($txtPath, $reportText, [System.Text.UTF8Encoding]::new($true))

    $payload = [pscustomobject]@{
        inventory = $Inventory
        probe = $ProbeResult
        launch = $LaunchResult
        security = $SecuritySignals
        outcome = $Outcome
    }
    $json = $payload | ConvertTo-Json -Depth 8
    [System.IO.File]::WriteAllText($jsonPath, $json, [System.Text.UTF8Encoding]::new($true))

    [pscustomobject]@{
        txt_path = $txtPath
        json_path = $jsonPath
    }
}
