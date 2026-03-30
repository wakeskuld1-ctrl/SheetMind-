use excel_skill::gui::bridge::tool_runner::ToolRunner;

#[test]
fn tool_runner_can_request_catalog() {
    let runner = ToolRunner::new();
    let response = runner.catalog().unwrap();
    assert!(response.success);
}
