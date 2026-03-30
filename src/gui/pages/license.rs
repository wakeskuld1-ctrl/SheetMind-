use eframe::egui;

use crate::gui::bridge::license_bridge::LicenseSummary;
use crate::gui::state::{
    AppState, LicenseActionItem, LicensePageState, LicenseRefreshFeedbackKind,
};

// 2026-03-29 CST: 这里定义授权页可发出的动作事件，原因是授权页不能直接依赖应用壳细节。
// 目的：把页面点击行为收口成稳定事件，再由应用壳统一决定如何执行刷新或后续动作。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LicensePageAction {
    RefreshStatus,
}

// 2026-03-29 CST: 这里渲染“授权与设置”页，原因是 Task 10 需要把授权页从占位文案升级成产品化授权中心。
// 目的：先把授权摘要、设备状态、动作区、本地设置区和排障说明区完整搭起来，并把页面动作以事件形式返回给应用壳。
pub fn render(
    ui: &mut egui::Ui,
    state: &mut AppState,
    summary: &LicenseSummary,
) -> Option<LicensePageAction> {
    ui.label("统一查看当前授权状态、设备绑定情况、本地设置和后续授权动作。");
    ui.separator();

    let mut action = None;
    ui.columns(3, |columns| {
        render_summary_panel(&mut columns[0], state, summary);
        action = render_action_panel(&mut columns[1], state);
        render_support_panel(&mut columns[2], state);
    });

    action
}

// 2026-03-29 CST: 这里渲染授权摘要与设备状态，原因是授权页首先要回答“当前授权到什么状态了”。
// 目的：把授权摘要和设备摘要固定为首页块，和顶部栏形成稳定一致的状态展示。
fn render_summary_panel(ui: &mut egui::Ui, state: &AppState, summary: &LicenseSummary) {
    ui.heading("授权摘要");
    ui.separator();
    ui.label(format!("状态：{}", state.license_status_text()));
    ui.label(format!("邮箱：{}", summary.license_email));
    ui.label(format!("最近校验：{}", summary.last_validated_at));
    ui.label(format!("设备状态：{}", summary.device_status));
    ui.add_space(8.0);

    ui.group(|ui| {
        ui.label("设备绑定概览");
        ui.small("后续会在这里显示设备绑定详情、绑定次数和许可证绑定关系。");
    });
}

// 2026-03-29 CST: 这里渲染动作区和本地设置区，原因是方案 B 要求授权页不仅展示状态，还要明确可执行动作。
// 目的：先把激活、刷新、解绑动作固化成产品化动作区，后续接真实流程时不再改布局。
fn render_action_panel(ui: &mut egui::Ui, state: &AppState) -> Option<LicensePageAction> {
    ui.heading("动作与设置");
    ui.separator();

    let mut action = None;
    for item in &state.license_page.available_actions {
        if let Some(next_action) = render_action_item(ui, item, &state.license_page) {
            action = Some(next_action);
        }
        ui.add_space(6.0);
    }

    render_refresh_feedback(ui, &state.license_page);

    ui.add_space(8.0);
    ui.group(|ui| {
        ui.label("本地设置");
        ui.small(&state.license_page.local_settings_hint);
    });

    action
}

// 2026-03-29 CST: 这里统一渲染授权动作项，原因是每个授权动作都需要一致的标题、说明和触发入口。
// 目的：让“刷新状态”先具备真实可触发入口，其余动作保留同样结构，后续逐步接线。
fn render_action_item(
    ui: &mut egui::Ui,
    action: &LicenseActionItem,
    license_page: &LicensePageState,
) -> Option<LicensePageAction> {
    let mut next_action = None;

    ui.group(|ui| {
        ui.label(action.label);
        ui.small(action.summary);
        ui.add_space(6.0);

        if action.id == "refresh" {
            // 2026-03-29 CST: 这里先把刷新动作接成真实按钮，原因是授权状态同步闭环要从页面动作进入应用壳。
            // 目的：让授权页可以显式请求刷新状态，并由上层统一执行状态回写。
            let button_label = if license_page.refresh_in_progress {
                "刷新中..."
            } else {
                "刷新状态"
            };
            if ui
                .add_enabled(
                    !license_page.refresh_in_progress,
                    egui::Button::new(button_label),
                )
                .clicked()
            {
                next_action = Some(LicensePageAction::RefreshStatus);
            }
        } else {
            ui.add_enabled(false, egui::Button::new("即将接入"));
        }
    });

    next_action
}

// 2026-03-30 CST: 这里统一渲染授权刷新反馈，原因是刷新中的提示、在线失败警告和真正错误都需要稳定落点。
// 目的：让授权页动作区在不引入完整消息中心的前提下，先具备可见的反馈闭环。
fn render_refresh_feedback(ui: &mut egui::Ui, license_page: &LicensePageState) {
    if license_page.refresh_in_progress {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.add(egui::Spinner::new());
                ui.small("正在刷新授权状态，请稍候。");
            });
        });
        return;
    }

    let Some(message) = &license_page.refresh_feedback_message else {
        return;
    };

    let color = match license_page.refresh_feedback_kind {
        LicenseRefreshFeedbackKind::Info => egui::Color32::from_rgb(46, 125, 50),
        LicenseRefreshFeedbackKind::Warning => egui::Color32::from_rgb(245, 124, 0),
        LicenseRefreshFeedbackKind::Error => egui::Color32::from_rgb(198, 40, 40),
    };

    ui.group(|ui| {
        ui.colored_label(color, message);
    });
}

// 2026-03-29 CST: 这里渲染排障与说明区，原因是授权问题通常需要用户知道该怎么判断和下一步怎么做。
// 目的：把常见说明和排障建议提前做成固定模块，降低首发销售版的支持成本。
fn render_support_panel(ui: &mut egui::Ui, state: &AppState) {
    ui.heading("说明与排障");
    ui.separator();

    for note in &state.license_page.troubleshooting_notes {
        ui.label(format!("• {}", note));
    }

    ui.add_space(8.0);
    ui.group(|ui| {
        ui.label("后续动作占位");
        ui.small("后续会接入许可证输入框、解绑确认和本地环境设置。");
    });
}
