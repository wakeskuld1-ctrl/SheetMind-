// 2026-03-31 CST: 这里给 GUI 数据处理状态测试补 feature 门，原因是默认构建不应强依赖桌面层；
// 目的：把 GUI 专属页面状态验证限制在 `gui` feature 下，保持主业务链独立。
#![cfg(feature = "gui")]

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
