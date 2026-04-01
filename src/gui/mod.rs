// 2026-03-29 CST: 这里建立 GUI 模块根入口，原因是首发桌面版需要独立于 CLI 的产品层；
// 目的：先提供最小模块边界，后续再逐步补充 app、state、bridge 和 pages。
pub mod app;
pub mod bridge;
pub mod pages;
pub mod state;
pub mod theme;
