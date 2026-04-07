// 2026-03-31 CST: 这里给 GUI 工具桥接测试补 feature 门，原因是桌面桥接层应该作为可选能力存在；
// 目的：让默认测试专注核心工具协议，GUI 桥接验证改走显式 `gui` feature。
#![cfg(feature = "gui")]

use excel_skill::gui::bridge::tool_runner::ToolRunner;

#[test]
fn tool_runner_can_request_catalog() {
    let runner = ToolRunner::new();
    let response = runner.catalog().unwrap();
    assert!(response.success);
}
