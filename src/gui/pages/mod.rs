// 2026-03-29 CST: 这里维护 GUI 页面模块目录，原因是七个一级页面会逐步从应用壳中拆分出来；
// 目的：保持页面职责清晰，避免 `app.rs` 再次膨胀回单文件大杂烩。
pub mod ai;
pub mod analysis;
pub mod dashboard;
pub mod data_processing;
pub mod files;
pub mod license;
pub mod reports;
