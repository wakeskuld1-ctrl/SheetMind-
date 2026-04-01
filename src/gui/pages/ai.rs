use eframe::egui;

use crate::gui::state::{AiSuggestionCard, AppState};

// 2026-03-29 CST: 这里渲染“AI 助手”页，原因是 Task 9 要把 AI 页从占位文案升级成可扩展容器；
// 目的：先把上下文摘要、推荐动作和拟执行动作区搭出来，后续再接本地规则或大模型。
pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    ui.label("围绕当前文件、数据集和分析阶段提供 AI 建议容器，后续可逐步接入规则引擎或模型。");
    ui.separator();

    ui.columns(3, |columns| {
        render_context_panel(&mut columns[0], state);
        render_suggestions_panel(&mut columns[1], state);
        render_action_panel(&mut columns[2], state);
    });
}

// 2026-03-29 CST: 这里渲染上下文摘要区，原因是 AI 页首先要让用户知道当前 AI 看到了哪些上下文；
// 目的：把文件、数据集和当前阶段的摘要落到固定区域，便于后续 AI 建议可解释。
fn render_context_panel(ui: &mut egui::Ui, state: &AppState) {
    ui.heading("当前上下文");
    ui.separator();
    ui.label(&state.ai.context_summary);
    ui.add_space(8.0);
    ui.group(|ui| {
        ui.label("上下文来源");
        ui.small(format!("项目：{}", state.project_name));
        ui.small(format!("文件：{}", state.current_file_name));
        ui.small(format!("数据集：{}", state.current_dataset_name));
    });
}

// 2026-03-29 CST: 这里渲染推荐动作区，原因是 AI 页的核心价值是建议“下一步可以做什么”；
// 目的：先把建议卡片容器做出来，后续接规则和模型时直接回填即可。
fn render_suggestions_panel(ui: &mut egui::Ui, state: &AppState) {
    ui.heading("推荐动作");
    ui.separator();

    if state.ai.suggestions.is_empty() {
        ui.label("当前还没有生成 AI 建议。");
        ui.add_space(8.0);
        render_placeholder_suggestion(
            ui,
            &AiSuggestionCard {
                id: "placeholder",
                title: "建议容器已就绪",
                reason: "后续接入 AI 或规则后，这里会显示基于当前上下文的推荐动作卡片。",
            },
        );
        return;
    }

    for suggestion in &state.ai.suggestions {
        render_placeholder_suggestion(ui, suggestion);
        ui.add_space(6.0);
    }
}

// 2026-03-29 CST: 这里统一渲染建议卡片，原因是后续建议项需要稳定的标题和原因展示格式；
// 目的：先把建议卡片视图固定下来，避免未来 AI 能力接入后还要重做容器样式。
fn render_placeholder_suggestion(ui: &mut egui::Ui, suggestion: &AiSuggestionCard) {
    ui.group(|ui| {
        ui.label(suggestion.title);
        ui.small(suggestion.reason);
    });
}

// 2026-03-29 CST: 这里渲染拟执行动作区，原因是 AI 页不能只给建议，还要说明“如果执行会发生什么”；
// 目的：为未来的确认执行、影响范围提示和用户确认入口预留稳定位置。
fn render_action_panel(ui: &mut egui::Ui, state: &AppState) {
    ui.heading("拟执行动作");
    ui.separator();
    ui.label(&state.ai.proposed_action);
    ui.add_space(8.0);
    ui.group(|ui| {
        ui.label("执行占位");
        ui.small("后续会在这里展示 AI 准备执行的工具调用、影响范围和确认按钮。");
    });
}
