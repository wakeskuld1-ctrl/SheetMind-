use assert_cmd::Command;
use excel_skill::gui::app::SheetMindApp;
use excel_skill::gui::bridge::tool_runner::ToolRunner;
use excel_skill::gui::state::AppPage;

#[test]
fn gui_smoke_bootstrap_and_navigation_contract_are_available() {
    let mut cmd = Command::cargo_bin("sheetmind_app").unwrap();
    cmd.arg("--help");
    cmd.assert().success();

    let app = SheetMindApp::new();

    assert_eq!(app.state.current_page(), AppPage::Dashboard);
    assert!(!app.state.license_status_text().is_empty());
    assert_eq!(SheetMindApp::navigation_items().len(), 7);
    assert_eq!(SheetMindApp::page_title(AppPage::Dashboard), "工作台");
    assert_eq!(SheetMindApp::page_title(AppPage::LicenseSettings), "授权与设置");
}

#[test]
fn gui_smoke_tool_runner_catalog_is_available() {
    let runner = ToolRunner::new();
    let response = runner.catalog().unwrap();

    assert!(response.success);
}
