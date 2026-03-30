// 2026-03-31 CST: 这里给 GUI 授权桥接测试补 feature 门，原因是 GUI 授权展示层不该混入默认主线；
// 目的：保留授权核心逻辑可复用，同时把桌面桥接验证隔离到显式 GUI 构建。
#![cfg(feature = "gui")]

use excel_skill::gui::bridge::license_bridge::LicenseSummary;

#[test]
fn license_summary_defaults_to_unlicensed() {
    let summary = LicenseSummary::default();
    assert!(!summary.licensed);
}
