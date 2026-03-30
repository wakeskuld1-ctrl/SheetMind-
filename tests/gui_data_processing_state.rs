use excel_skill::gui::state::DataProcessingState;

#[test]
fn data_processing_state_tracks_history() {
    let mut state = DataProcessingState::default();
    state.push_history("筛选：地区=华东");

    assert_eq!(state.history.len(), 1);
    assert_eq!(state.history[0], "筛选：地区=华东");
}

#[test]
fn data_processing_state_exposes_operation_presets() {
    let state = DataProcessingState::default();

    assert!(!state.operation_groups.is_empty());
    assert!(
        state
            .operation_groups
            .iter()
            .any(|group| !group.presets.is_empty())
    );
}
