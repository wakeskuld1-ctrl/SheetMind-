use assert_cmd::Command;

#[test]
fn sheetmind_app_help_or_bootstrap_runs() {
    let mut cmd = Command::cargo_bin("sheetmind_app").unwrap();
    cmd.arg("--help");
    cmd.assert().success();
}
