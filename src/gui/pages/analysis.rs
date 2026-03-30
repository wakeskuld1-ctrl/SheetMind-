use eframe::egui;

use crate::gui::state::{AnalysisTaskCard, AnalysisTaskKind, AppState};

// 2026-03-29 CST: 这里渲染“分析建模”页，原因是 Task 8 要把原来的占位文案升级成真正的任务导向页面；
// 目的：先把任务卡片入口、参数区、结果区、图表占位和风险解释区完整搭起，后续再逐步接入真实分析算法。
pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    ui.label("围绕常见分析任务提供卡片化入口，帮助用户从概览、诊断到建模逐步推进。");
    ui.separator();

    render_task_cards(ui, state);
    ui.separator();

    ui.columns(3, |columns| {
        render_parameter_panel(&mut columns[0], state);
        render_result_panel(&mut columns[1], state);
        render_risk_panel(&mut columns[2], state);
    });
}

// 2026-03-29 CST: 这里渲染顶部任务卡片，原因是方案 B 需要分析页优先呈现“我想做哪类分析”；
// 目的：让分析页入口更像产品能力面板，而不是传统的生硬 tab 切换条。
fn render_task_cards(ui: &mut egui::Ui, state: &mut AppState) {
    ui.heading("分析任务");
    ui.add_space(6.0);

    let task_cards = state.analysis.task_cards.clone();
    let selected_task = state.analysis.selected_task;

    ui.horizontal_wrapped(|ui| {
        for card in task_cards {
            render_single_task_card(ui, state, &card, selected_task == card.kind);
            ui.add_space(8.0);
        }
    });
}

// 2026-03-29 CST: 这里渲染单个任务卡片，原因是每张卡片都要统一处理选中态和任务切换；
// 目的：把卡片点击后的状态联动集中起来，后续接执行逻辑时不需要重新整理页面结构。
fn render_single_task_card(
    ui: &mut egui::Ui,
    state: &mut AppState,
    card: &AnalysisTaskCard,
    selected: bool,
) {
    ui.group(|ui| {
        if ui.selectable_label(selected, card.title).clicked() {
            state.analysis_mut().select_task(card.kind);
        }
        ui.small(card.subtitle);
        ui.add_space(4.0);
        ui.small(format!("推荐场景：{}", card.recommended_input));
    });
}

// 2026-03-29 CST: 这里渲染参数区，原因是不同分析任务后续会有不同的字段选择和参数输入；
// 目的：先把任务说明、参数表单占位和执行入口固定下来，后续接真实 Tool 时不再反复调布局。
fn render_parameter_panel(ui: &mut egui::Ui, state: &AppState) {
    ui.heading("参数区");
    ui.separator();

    if let Some(card) = state.analysis.selected_card() {
        ui.label(format!("当前任务：{}", card.title));
        ui.small(card.subtitle);
        ui.add_space(6.0);
        ui.small(format!("推荐输入：{}", card.recommended_input));
    } else {
        ui.label("当前任务：尚未选择");
    }

    ui.add_space(8.0);
    ui.label(&state.analysis.parameter_hint);
    ui.add_space(8.0);
    ui.group(|ui| {
        ui.label("参数表单占位");
        ui.small("后续会接入字段选择、时间粒度、目标列、对比组和模型参数配置。");
    });
    ui.add_space(8.0);
    ui.label("执行入口预留");
    ui.small("后续会接到分析 Tool 桥接层，并把结果同步回结果区和图表区。");
}

// 2026-03-29 CST: 这里渲染结果区，原因是分析页必须有固定位置承接统计摘要、图表和结果说明；
// 目的：让未来的分析结果不需要重新寻找落点，直接按照这套骨架逐步填实。
fn render_result_panel(ui: &mut egui::Ui, state: &AppState) {
    ui.heading("结果区");
    ui.separator();
    ui.label(&state.analysis.result_summary);
    ui.add_space(8.0);
    ui.group(|ui| {
        ui.label("核心发现");
        ui.small("这里会显示统计摘要、显著结论、模型指标或推荐动作。");
    });
    ui.add_space(8.0);
    ui.group(|ui| {
        ui.label("图表占位");
        ui.small(&state.analysis.chart_hint);
    });
}

// 2026-03-29 CST: 这里渲染风险与解释区，原因是分析结果不能只给图和数，还要保留解释与风险提示；
// 目的：把“结论如何理解、哪里可能有风险”提前做成固定模块，后续便于交付给业务使用。
fn render_risk_panel(ui: &mut egui::Ui, state: &AppState) {
    ui.heading("风险与解释");
    ui.separator();

    for note in &state.analysis.risk_notes {
        ui.label(format!("• {}", note));
    }

    ui.add_space(8.0);
    render_next_steps(ui, state.analysis.selected_task);
}

// 2026-03-29 CST: 这里补充下一步建议，原因是方案 B 强调任务导向，不希望页面停留在静态说明；
// 目的：根据当前任务给出下一步动作提示，让用户更容易理解后续将如何继续推进。
fn render_next_steps(ui: &mut egui::Ui, task: AnalysisTaskKind) {
    ui.group(|ui| {
        ui.label("下一步建议");
        ui.small(match task {
            AnalysisTaskKind::Overview => {
                "先确认样本量、字段类型和分布，再进入质量诊断或关系分析。"
            }
            AnalysisTaskKind::QualityDiagnosis => {
                "先修复缺失值和异常值，再决定是否进入相关性或建模阶段。"
            }
            AnalysisTaskKind::RelationshipAnalysis => {
                "先看关键变量关系，再收敛到趋势分析或模型分析。"
            }
            AnalysisTaskKind::TrendAnalysis => "先确认时间粒度和波动周期，再补充预测或策略判断。",
            AnalysisTaskKind::Modeling => "先明确目标列和训练字段，再进入回归、分类或聚类。",
            AnalysisTaskKind::DecisionSupport => {
                "把已得到的发现转成结论、风险与建议动作，准备导出交付。"
            }
        });
    });
}
