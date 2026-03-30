use excel_skill::gui::app::SheetMindApp;
use excel_skill::gui::bridge::license_bridge::LicenseSummary;
use excel_skill::gui::pages::license::LicensePageAction;
use excel_skill::gui::state::{AppState, LicensePageState};

#[test]
fn app_state_exposes_license_status_text() {
    let state = AppState::default();

    assert!(!state.license_status_text().is_empty());
}

#[test]
fn license_page_state_exposes_default_actions() {
    let state = LicensePageState::default();

    assert!(!state.available_actions.is_empty());
    assert!(!state.local_settings_hint.is_empty());
}

#[test]
fn sheetmind_app_keeps_license_summary_and_status_text_in_sync() {
    let app = SheetMindApp::new();

    assert_eq!(
        app.license_summary().status_text,
        app.state.license_status_text()
    );
}

#[test]
fn sheetmind_app_refresh_updates_license_summary_and_status_text() {
    let mut app = SheetMindApp::default();
    let refreshed = LicenseSummary {
        licensed: true,
        status_text: "已授权".to_string(),
        license_email: "owner@example.com".to_string(),
        last_validated_at: "2026-03-29T13:00:00+08:00".to_string(),
        device_status: "已绑定设备".to_string(),
    };

    app.refresh_license_summary_with(|| refreshed.clone());

    assert_eq!(app.license_summary(), &refreshed);
    assert_eq!(app.state.license_status_text(), "已授权");
}

#[test]
fn sheetmind_app_handles_refresh_license_page_action() {
    let mut app = SheetMindApp::default();
    let refreshed = LicenseSummary {
        licensed: true,
        status_text: "刷新后已授权".to_string(),
        license_email: "owner@example.com".to_string(),
        last_validated_at: "2026-03-29T13:10:00+08:00".to_string(),
        device_status: "已绑定设备".to_string(),
    };

    app.handle_license_page_action_with(LicensePageAction::RefreshStatus, || refreshed.clone());

    assert_eq!(app.license_summary(), &refreshed);
    assert_eq!(app.state.license_status_text(), "刷新后已授权");
}
