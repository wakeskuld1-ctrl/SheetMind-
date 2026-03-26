param(
    [ValidateSet("p1", "p2", "all")]
    [string]$Suite = "all",
    [switch]$LowMemory
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Invoke-ExactCliJsonTest {
    param(
        [Parameter(Mandatory = $true)]
        [string]$TestName
    )

    Write-Host "[gate] cargo test --test integration_cli_json $TestName -- --exact"
    cargo test --test integration_cli_json $TestName -- --exact
    if ($LASTEXITCODE -ne 0) {
        throw "Regression gate failed: $TestName"
    }
}

$p1Tests = @(
    "open_workbook_missing_path_returns_utf8_error_message",
    "cli_open_workbook_accepts_chinese_windows_path",
    "cli_open_workbook_accepts_gbk_encoded_json_with_chinese_path",
    "execute_multi_table_plan_stops_before_join_without_auto_confirm",
    "execute_multi_table_plan_stops_when_result_bindings_are_missing",
    "execute_multi_table_plan_stops_when_join_risk_threshold_exceeded",
    "execute_multi_table_plan_auto_confirm_applies_default_join_risk_guard"
)

$p2Tests = @(
    "tool_catalog_includes_recover_multi_table_failure",
    "execute_multi_table_plan_failed_step_returns_unknown_failure_diagnostics",
    "execute_multi_table_plan_missing_tool_call_returns_unknown_failure_diagnostics",
    "execute_multi_table_plan_stops_after_target_step_id",
    "recover_multi_table_failure_runs_replay_then_full_chain",
    "recover_multi_table_failure_uses_runtime_continuation_template",
    "recover_multi_table_failure_allows_replay_template_overrides",
    "recover_multi_table_failure_allows_continue_template_overrides",
    "recover_multi_table_failure_rejects_invalid_template_overrides",
    "recover_multi_table_failure_accepts_legacy_template_arg_overrides",
    "recover_multi_table_failure_reports_ignored_template_overrides",
    "recover_multi_table_failure_strict_overrides_reject_unknown_template_keys"
)

if ($LowMemory) {
    # Windows runners with limited page file can fail mmap during heavy builds.
    # For deterministic gate runs, cap cargo parallel build jobs.
    $env:CARGO_BUILD_JOBS = "1"
}

$testsToRun = switch ($Suite) {
    "p1" { $p1Tests }
    "p2" { $p2Tests }
    default { @($p1Tests + $p2Tests) }
}

Write-Host "[gate] suite=$Suite tests=$($testsToRun.Count) low_memory=$($LowMemory.IsPresent)"

foreach ($testName in $testsToRun) {
    Invoke-ExactCliJsonTest -TestName $testName
}

Write-Host "[gate] all selected recovery gates passed."
