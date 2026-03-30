use excel_skill::gui::app::SheetMindApp;
use excel_skill::gui::bridge::license_bridge::{LicenseRefreshResult, LicenseSummary};
use excel_skill::gui::pages::license::LicensePageAction;
use excel_skill::gui::state::{AppState, LicensePageState, LicenseRefreshFeedbackKind};

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
fn license_page_state_defaults_to_idle_refresh_feedback() {
    let state = LicensePageState::default();

    assert!(!state.refresh_in_progress);
    assert!(state.refresh_feedback_message.is_none());
    assert_eq!(
        state.refresh_feedback_kind,
        LicenseRefreshFeedbackKind::Info
    );
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

#[test]
fn sheetmind_app_marks_license_refresh_as_in_progress_when_started() {
    let mut app = SheetMindApp::default();

    app.start_license_refresh_with(|| Ok(LicenseRefreshResult::success(LicenseSummary::default())));

    assert!(app.state.license_page.refresh_in_progress);
    assert_eq!(
        app.state.license_page.refresh_feedback_message.as_deref(),
        Some("正在刷新授权状态...")
    );
    assert_eq!(
        app.state.license_page.refresh_feedback_kind,
        LicenseRefreshFeedbackKind::Info
    );
}

#[test]
fn sheetmind_app_applies_license_refresh_warning_and_updates_summary() {
    let mut app = SheetMindApp::default();
    let refreshed = LicenseSummary {
        licensed: true,
        status_text: "在线校验失败，已返回本地状态: 网络波动".to_string(),
        license_email: "owner@example.com".to_string(),
        last_validated_at: "2026-03-30T09:20:00+08:00".to_string(),
        device_status: "已绑定设备".to_string(),
    };

    app.state.license_page.begin_refresh();
    app.apply_license_refresh_result(Ok(LicenseRefreshResult {
        summary: refreshed.clone(),
        warning_message: Some("在线校验失败，已返回本地状态: 网络波动".to_string()),
    }));

    assert_eq!(app.license_summary(), &refreshed);
    assert_eq!(
        app.state.license_page.refresh_feedback_message.as_deref(),
        Some("在线校验失败，已返回本地状态: 网络波动")
    );
    assert_eq!(
        app.state.license_page.refresh_feedback_kind,
        LicenseRefreshFeedbackKind::Warning
    );
    assert!(!app.state.license_page.refresh_in_progress);
}

#[test]
fn sheetmind_app_applies_license_refresh_failure_without_overwriting_summary() {
    let mut app = SheetMindApp::default();
    let existing = LicenseSummary {
        licensed: true,
        status_text: "已授权".to_string(),
        license_email: "owner@example.com".to_string(),
        last_validated_at: "2026-03-30T09:00:00+08:00".to_string(),
        device_status: "已绑定设备".to_string(),
    };

    app.refresh_license_summary_with(|| existing.clone());
    app.state.license_page.begin_refresh();
    app.apply_license_refresh_result(Err("授权服务暂时不可用".to_string()));

    assert_eq!(app.license_summary(), &existing);
    assert_eq!(app.state.license_status_text(), "已授权");
    assert_eq!(
        app.state.license_page.refresh_feedback_message.as_deref(),
        Some("授权服务暂时不可用")
    );
    assert_eq!(
        app.state.license_page.refresh_feedback_kind,
        LicenseRefreshFeedbackKind::Error
    );
    assert!(!app.state.license_page.refresh_in_progress);
}
