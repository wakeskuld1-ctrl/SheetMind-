// 2026-03-31 CST: 这里给 GUI 报告与 AI 页面状态测试补 feature 门，原因是这些页面状态不属于默认 CLI 主线；
// 目的：把桌面页签验证与核心能力链解耦，减少默认测试面的非必要耦合。
#![cfg(feature = "gui")]

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
