use std::env;

use excel_skill::gui::app::SheetMindApp;

// 2026-03-29 CST: 这里先提供 GUI 二进制独立入口，原因是首发桌面版必须新增可启动的 GUI 壳；
// 目的：在不影响现有 `src/main.rs` CLI 主链的前提下，为桌面产品保留独立启动点。
fn main() -> eframe::Result<()> {
    if env::args().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return Ok(());
    }

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "SheetMind",
        options,
        Box::new(|_cc| Ok(Box::new(SheetMindApp::new()))),
    )
}

// 2026-03-29 CST: 这里单独保留帮助输出，原因是测试需要在无图形环境下验证 GUI 二进制存在；
// 目的：通过 `--help` 提供稳定的最小契约，避免测试阶段强依赖窗口启动。
fn print_help() {
    println!("SheetMind Desktop GUI");
    println!("用法: sheetmind_app [--help]");
}
