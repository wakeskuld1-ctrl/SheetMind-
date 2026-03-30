// 2026-03-31 CST: 这里给 GUI 仪表盘状态测试补 feature 门，原因是默认产品主线不含桌面壳；
// 目的：让默认测试只覆盖 CLI / Tool / SQLite 主链，GUI 状态验证改为按需开启。
#![cfg(feature = "gui")]

use excel_skill::gui::state::AppState;

#[test]
fn dashboard_state_exposes_quick_actions() {
    let state = AppState::default();
    assert!(!state.quick_actions().is_empty());
}
