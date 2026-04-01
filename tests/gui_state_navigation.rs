// 2026-03-31 CST: 这里给 GUI 导航状态测试补 feature 门，原因是导航契约属于桌面页面层；
// 目的：默认主线不再编译 GUI 导航测试，维持 CLI / Tool 侧的构建纯度。
#![cfg(feature = "gui")]

use excel_skill::gui::state::{AppPage, AppState};

#[test]
fn app_state_can_switch_pages() {
    let mut state = AppState::default();
    state.set_page(AppPage::AnalysisModeling);
    assert_eq!(state.current_page(), AppPage::AnalysisModeling);
}
