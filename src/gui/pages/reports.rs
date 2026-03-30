use eframe::egui;

use crate::gui::state::{AppState, ReportTemplateCard};

// 2026-03-29 CST: 这里渲染“报告导出”页，原因是 Task 9 要把导出页从占位文案升级成产品化骨架；
// 目的：先把导出模板卡片、输出配置和最近导出记录完整搭出来，后续再接真实导出能力。
pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    ui.label("围绕常见交付场景提供导出模板入口，帮助用户快速整理成可交付的结果包。");
    ui.separator();

    render_template_cards(ui, state);
    ui.separator();

    ui.columns(3, |columns| {
        render_export_options(&mut columns[0], state);
        render_export_preview(&mut columns[1], state);
        render_recent_exports(&mut columns[2], state);
    });
}

// 2026-03-29 CST: 这里渲染导出模板卡片，原因是方案 B 需要导出页优先表达“我要交付成什么”；
// 目的：让导出页具备明确的交付语义，而不是只有路径和格式输入框。
fn render_template_cards(ui: &mut egui::Ui, state: &mut AppState) {
    ui.heading("导出模板");
    ui.add_space(6.0);

    let templates = state.reports.templates.clone();
    let selected_template_id = state.reports.selected_template_id.clone();

    ui.horizontal_wrapped(|ui| {
        for template in templates {
            render_single_template_card(ui, state, &template, selected_template_id == template.id);
            ui.add_space(8.0);
        }
    });
}

// 2026-03-29 CST: 这里渲染单个模板卡片，原因是每张卡片都需要统一处理选中态和模板切换；
// 目的：把模板切换逻辑集中到单处，方便后续挂接不同模板的真实导出流程。
fn render_single_template_card(
    ui: &mut egui::Ui,
    state: &mut AppState,
    template: &ReportTemplateCard,
    selected: bool,
) {
    ui.group(|ui| {
        if ui.selectable_label(selected, template.title).clicked() {
            state.reports_mut().select_template(template.id);
        }
        ui.small(template.summary);
        ui.add_space(4.0);
        ui.small(format!("适用对象：{}", template.target_audience));
    });
}

// 2026-03-29 CST: 这里渲染导出配置区，原因是导出页需要固定承载格式、路径和模板说明；
// 目的：先把核心导出配置摆稳，后续真实导出时不再重复调整页面结构。
fn render_export_options(ui: &mut egui::Ui, state: &AppState) {
    ui.heading("导出配置");
    ui.separator();

    if let Some(template) = state.reports.selected_template() {
        ui.label(format!("当前模板：{}", template.title));
        ui.small(template.summary);
        ui.add_space(6.0);
        ui.small(format!("适用对象：{}", template.target_audience));
    } else {
        ui.label("当前模板：尚未选择");
    }

    ui.add_space(8.0);
    ui.label(format!("输出路径：{}", state.reports.output_path));
    ui.label(format!("导出格式：{}", state.reports.export_format));
    ui.add_space(8.0);
    ui.group(|ui| {
        ui.label("配置占位");
        ui.small("后续会接入文件命名规则、是否包含图表、是否打包交付说明等选项。");
    });
}

// 2026-03-29 CST: 这里渲染导出预览区，原因是导出页需要告诉用户当前模板会产出什么内容；
// 目的：让用户在导出前就能理解交付包结构，减少“导出后才发现不对”的风险。
fn render_export_preview(ui: &mut egui::Ui, state: &AppState) {
    ui.heading("导出预览");
    ui.separator();

    if let Some(template) = state.reports.selected_template() {
        ui.label(format!("将生成：{}", template.title));
        ui.small("这里会显示导出内容预览、包含的图表/表格和交付说明。");
    } else {
        ui.label("尚未选择导出模板。");
    }

    ui.add_space(8.0);
    ui.group(|ui| {
        ui.label("交付内容占位");
        ui.small("后续会展示摘要页、图表页、明细页、说明文档等导出清单。");
    });
}

// 2026-03-29 CST: 这里渲染最近导出记录区，原因是导出页通常需要让用户回看最近生成过什么；
// 目的：先把导出历史作为固定区域留住，后续真实导出时可以直接回填记录。
fn render_recent_exports(ui: &mut egui::Ui, state: &AppState) {
    ui.heading("最近导出");
    ui.separator();

    if state.reports.recent_exports.is_empty() {
        ui.label("暂无导出记录。");
        return;
    }

    for item in &state.reports.recent_exports {
        ui.label(format!("• {}", item));
    }
}
