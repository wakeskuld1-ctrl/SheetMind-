use eframe::egui;

use crate::gui::state::{AppState, DataProcessingOperationGroup, DataProcessingPreset};

// 2026-03-29 CST: 这里渲染“数据处理”页，原因是 Task 7 要把原本的占位文案升级成真正可浏览的页面骨架；
// 目的：先把模板分组、预览区、参数区和历史区完整铺开，后续只需要把按钮逐步接入真实 Tool。
pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    ui.label("围绕常用数据清洗与整理动作提供可视入口，后续会逐步接入真实处理算法。");
    ui.separator();

    ui.columns(3, |columns| {
        render_operation_groups(&mut columns[0], state);
        render_preview_panel(&mut columns[1], state);
        render_parameter_panel(&mut columns[2], state);
    });

    ui.separator();
    render_history_panel(ui, state);
}

// 2026-03-29 CST: 这里渲染左侧模板分组，原因是方案 B 要先把常用处理能力做成可点选的预设入口；
// 目的：让用户在没有真实执行按钮之前，也能快速理解产品当前覆盖的处理能力版图。
fn render_operation_groups(ui: &mut egui::Ui, state: &mut AppState) {
    ui.heading("预设操作");
    ui.separator();

    let groups = state.data_processing.operation_groups.clone();
    let selected_group_id = state.data_processing.selected_group_id.clone();
    let selected_preset_id = state.data_processing.selected_preset_id.clone();

    for group in groups {
        let is_group_selected = selected_group_id == group.id;
        ui.collapsing(
            format!(
                "{}{}",
                if is_group_selected {
                    "当前分组 · "
                } else {
                    ""
                },
                group.label
            ),
            |ui| {
                render_group_presets(
                    ui,
                    state,
                    &group,
                    selected_group_id.as_str(),
                    selected_preset_id.as_str(),
                );
            },
        );
        ui.add_space(6.0);
    }
}

// 2026-03-29 CST: 这里拆出分组内模板渲染，原因是每个分组都要复用同一套“标题 + 描述 + 选择”逻辑；
// 目的：避免数据处理页在首版就把模板渲染逻辑写成无法维护的大段重复代码。
fn render_group_presets(
    ui: &mut egui::Ui,
    state: &mut AppState,
    group: &DataProcessingOperationGroup,
    selected_group_id: &str,
    selected_preset_id: &str,
) {
    for preset in &group.presets {
        let selected = selected_group_id == group.id && selected_preset_id == preset.id;
        render_single_preset(ui, state, group, preset, selected);
        ui.add_space(4.0);
    }
}

// 2026-03-29 CST: 这里渲染单个模板项，原因是每个预设都需要统一处理选中态和历史回写；
// 目的：把模板点击后的状态联动集中在一个函数里，便于后续接执行按钮时继续扩展。
fn render_single_preset(
    ui: &mut egui::Ui,
    state: &mut AppState,
    group: &DataProcessingOperationGroup,
    preset: &DataProcessingPreset,
    selected: bool,
) {
    if ui.selectable_label(selected, preset.label).clicked() {
        let processing = state.data_processing_mut();
        processing.select_preset(group.id, preset.id);
        processing.push_history(format!("选择预设：{} / {}", group.label, preset.label));
    }

    ui.small(preset.description);
}

// 2026-03-29 CST: 这里渲染中部预览区，原因是数据处理页必须保留结果预览的固定视觉锚点；
// 目的：先为处理前后样本对比、字段变化和统计摘要预留位置，避免后续执行结果无处承接。
fn render_preview_panel(ui: &mut egui::Ui, state: &AppState) {
    ui.heading("处理预览");
    ui.separator();
    ui.label(&state.data_processing.preview_message);
    ui.add_space(8.0);
    ui.group(|ui| {
        ui.label("执行前样本");
        ui.small("这里会显示原始数据样本、列概览和质量提示。");
    });
    ui.add_space(8.0);
    ui.group(|ui| {
        ui.label("执行后样本");
        ui.small("这里会显示处理结果预览、字段变化和影响范围摘要。");
    });
}

// 2026-03-29 CST: 这里渲染右侧参数区，原因是数据处理动作未来都会需要不同参数；
// 目的：先把当前动作说明、参数表单占位和执行入口预留出来，后续接线时不再调整页面框架。
fn render_parameter_panel(ui: &mut egui::Ui, state: &AppState) {
    ui.heading("参数与执行");
    ui.separator();

    if let Some(preset) = state.data_processing.selected_preset() {
        ui.label(format!("当前动作：{}", preset.label));
        ui.small(preset.description);
    } else {
        ui.label("当前动作：尚未选择");
    }

    ui.add_space(8.0);
    ui.label(&state.data_processing.parameter_hint);
    ui.add_space(8.0);
    ui.group(|ui| {
        ui.label("参数表单占位");
        ui.small("后续会按所选预设动态生成列选择、条件、映射和输出策略。");
    });
    ui.add_space(8.0);
    ui.label("执行按钮预留");
    ui.small("后续会接入 ToolRunner，并把执行结果同步回预览区与操作历史。");
}

// 2026-03-29 CST: 这里渲染操作历史区，原因是 Task 7 已明确要求数据处理页具备历史模型；
// 目的：先让预设选择也能沉淀为可见历史，后续真实执行时可以继续沿用同一块区域。
fn render_history_panel(ui: &mut egui::Ui, state: &AppState) {
    ui.heading("操作历史");
    ui.separator();

    if state.data_processing.history.is_empty() {
        ui.label("尚无操作历史。");
        return;
    }

    for (index, item) in state.data_processing.history.iter().enumerate() {
        ui.label(format!("{}. {}", index + 1, item));
    }
}
