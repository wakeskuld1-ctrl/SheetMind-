use eframe::egui;

use crate::gui::state::AppState;

// 2026-03-29 CST: 这里渲染工作台页，原因是首版 GUI 首页需要从“纯占位”升级为可扫描的任务入口页；
// 目的：先把快速开始和推荐任务呈现出来，后续再补最近项目与最近导出。
pub fn render(ui: &mut egui::Ui, state: &AppState) {
    ui.label("把 Excel 处理、分析和交付整理成一条稳定流程。");
    ui.separator();

    ui.heading("快速开始");
    for action in state.quick_actions() {
        ui.label(format!("• {}", action.label));
    }

    ui.separator();
    ui.heading("推荐任务");
    for task in &state.dashboard.recommended_tasks {
        ui.label(format!("• {}", task));
    }
}
