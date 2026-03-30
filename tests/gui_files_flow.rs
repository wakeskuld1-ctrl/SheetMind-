use excel_skill::gui::state::FilesPageState;

#[test]
fn files_page_state_can_store_selected_sheet() {
    let mut state = FilesPageState::default();
    state.selected_sheet = Some("Sheet1".to_string());
    assert_eq!(state.selected_sheet.as_deref(), Some("Sheet1"));
}
