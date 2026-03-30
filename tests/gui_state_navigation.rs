use excel_skill::gui::state::{AppPage, AppState};

#[test]
fn app_state_can_switch_pages() {
    let mut state = AppState::default();
    state.set_page(AppPage::AnalysisModeling);
    assert_eq!(state.current_page(), AppPage::AnalysisModeling);
}
