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
