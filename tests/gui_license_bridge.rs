use excel_skill::gui::bridge::license_bridge::LicenseSummary;

#[test]
fn license_summary_defaults_to_unlicensed() {
    let summary = LicenseSummary::default();
    assert!(!summary.licensed);
}
