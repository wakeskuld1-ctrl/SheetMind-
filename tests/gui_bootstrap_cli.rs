// 2026-03-31 CST: 这里给 GUI 启动测试补 feature 门，原因是桌面入口不应污染默认 CLI 主线；
// 目的：把 `sheetmind_app` 的验证收口到显式 `--features gui` 场景，避免默认测试误编译 GUI。
#![cfg(feature = "gui")]

use assert_cmd::Command;

#[test]
fn sheetmind_app_help_or_bootstrap_runs() {
    let mut cmd = Command::cargo_bin("sheetmind_app").unwrap();
    cmd.arg("--help");
    cmd.assert().success();
}
