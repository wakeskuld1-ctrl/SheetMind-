use excel_skill::gui::state::{AiState, ReportsState};

#[test]
fn reports_and_ai_states_have_defaults() {
    let reports = ReportsState::default();
    let ai = AiState::default();

    assert!(!reports.templates.is_empty());
    assert!(!reports.export_format.is_empty());
    assert!(!ai.context_summary.is_empty());
    assert!(ai.suggestions.is_empty());
}
