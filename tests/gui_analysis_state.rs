// 2026-03-31 CST: 这里给 GUI 状态测试补 feature 门，原因是当前产品默认主线没有 GUI；
// 目的：避免默认 `cargo test` 被桌面依赖拉进编译链，只有显式开启 `gui` feature 时才验证这组测试。
#![cfg(feature = "gui")]

use excel_skill::gui::state::{AnalysisState, AnalysisTaskKind};

#[test]
fn analysis_task_kinds_include_modeling() {
    assert!(matches!(
        AnalysisTaskKind::Modeling,
        AnalysisTaskKind::Modeling
    ));
}

#[test]
fn analysis_state_exposes_task_cards() {
    let state = AnalysisState::default();

    assert_eq!(state.task_cards.len(), 6);
    assert!(
        state
            .task_cards
            .iter()
            .any(|card| card.kind == AnalysisTaskKind::Modeling)
    );
    assert!(!state.parameter_hint.is_empty());
    assert!(!state.result_summary.is_empty());
}
