use eframe::egui;
use rfd::FileDialog;

use crate::gui::bridge::tool_runner::ToolRunner;
use crate::gui::state::{AppState, FilesPageState};

// 2026-03-29 CST: 这里渲染“文件与表”页，原因是文件导入和 Sheet 选择是桌面 GUI 主流程的第一站；
// 目的：先把文件选择、Sheet 列表、表确认占位和右侧说明骨架搭起来，再逐步接入完整识别链路。
pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    let runner = ToolRunner::new();

    ui.horizontal(|ui| {
        if ui.button("选择 Excel 文件").clicked()
            && let Some(path_buf) = FileDialog::new()
                .add_filter("Excel", &["xlsx", "xlsm", "xls"])
                .pick_file()
        {
            let path_text = path_buf.display().to_string();
            let files_state = state.files_page_mut();
            files_state.selected_file_path = Some(path_text.clone());
            files_state.preview_message = format!("已选择文件：{path_text}");

            if let Ok(result) = runner.list_sheets(&path_text) {
                files_state.sheet_names = extract_sheet_names(&result.data);
                if files_state.selected_sheet.is_none() {
                    files_state.selected_sheet = files_state.sheet_names.first().cloned();
                }
            }

            state.current_file_name = path_buf
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or(path_text);
        }
    });

    ui.separator();
    ui.columns(3, |columns| {
        render_file_sidebar(&mut columns[0], &state.files_page);
        render_preview_panel(&mut columns[1], &state.files_page);
        render_schema_panel(&mut columns[2], &state.files_page);
    });
}

fn render_file_sidebar(ui: &mut egui::Ui, state: &FilesPageState) {
    ui.heading("文件与 Sheet");
    ui.separator();
    ui.label(
        state
            .selected_file_path
            .as_deref()
            .unwrap_or("尚未选择 Excel 文件"),
    );
    ui.separator();
    ui.label("Sheet 列表");
    for sheet_name in &state.sheet_names {
        ui.label(format!("• {sheet_name}"));
    }
}

fn render_preview_panel(ui: &mut egui::Ui, state: &FilesPageState) {
    ui.heading("预览区");
    ui.separator();
    ui.label(&state.preview_message);
    ui.separator();
    ui.label("这里将承载表区域识别、预览表格和样本行。");
}

fn render_schema_panel(ui: &mut egui::Ui, state: &FilesPageState) {
    ui.heading("字段与确认");
    ui.separator();
    ui.label(
        state
            .selected_sheet
            .as_deref()
            .unwrap_or("尚未选中工作表"),
    );
    ui.separator();
    ui.label("这里将承载表头确认、字段结构和数据集建立。");
}

fn extract_sheet_names(payload: &serde_json::Value) -> Vec<String> {
    payload
        .get("sheets")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(ToString::to_string))
                .collect::<Vec<_>>()
        })
        .or_else(|| {
            payload
                .get("sheet_names")
                .and_then(|value| value.as_array())
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|item| item.as_str().map(ToString::to_string))
                        .collect::<Vec<_>>()
                })
        })
        .unwrap_or_default()
}
