// 2026-03-31 CST: 这里给 GUI 文件流转测试补 feature 门，原因是文件选择与桌面页面流程属于可选 GUI 能力；
// 目的：默认测试不再为这组桌面流程承担编译和链接成本。
#![cfg(feature = "gui")]

use excel_skill::gui::state::FilesPageState;

#[test]
fn files_page_state_can_store_selected_sheet() {
    let mut state = FilesPageState::default();
    state.selected_sheet = Some("Sheet1".to_string());
    assert_eq!(state.selected_sheet.as_deref(), Some("Sheet1"));
}
