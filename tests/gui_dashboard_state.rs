use excel_skill::gui::state::AppState;

#[test]
fn dashboard_state_exposes_quick_actions() {
    let state = AppState::default();
    assert!(!state.quick_actions().is_empty());
}
