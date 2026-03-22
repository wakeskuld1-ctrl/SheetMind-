mod common;

use excel_skill::domain::handles::TableHandle;
use excel_skill::domain::schema::SchemaState;
use excel_skill::excel::reader::{list_sheets, open_workbook};
use excel_skill::excel::sheet_range::inspect_sheet_range;

use crate::common::{create_chinese_path_fixture, create_positioned_workbook};

#[test]
fn schema_state_blocks_table_operations_until_confirmed() {
    let table = TableHandle::new_pending("sales.xlsx", "Sheet1");
    assert_eq!(table.schema_state(), &SchemaState::Pending);
    assert!(!table.schema_state().is_confirmed());
}

#[test]
fn open_workbook_lists_visible_sheets() {
    let response = open_workbook("tests/fixtures/basic-sales.xlsx").unwrap();
    assert_eq!(response.sheet_names, vec!["Sales", "Lookup"]);
}

#[test]
fn list_sheets_lists_visible_sheets() {
    let response = list_sheets("tests/fixtures/basic-sales.xlsx").unwrap();

    // 2026-03-22: 这里先锁定 list_sheets 与 open_workbook 保持一致的基础探查结果，目的是把 sheet 探查正式沉淀成独立 I/O 能力。
    assert_eq!(response.sheet_names, vec!["Sales", "Lookup"]);
}

#[test]
fn open_workbook_accepts_chinese_windows_path() {
    let fixture_path = create_chinese_path_fixture("\u{57fa}\u{7840}\u{9500}\u{552e}.xlsx");
    let response = open_workbook(fixture_path.to_str().unwrap()).unwrap();

    // 2026-03-22: ???????????????????????????????????????????
    assert_eq!(response.sheet_names, vec!["Sales", "Lookup"]);
    assert!(response.path.contains("\u{4e2d}\u{6587}\u{8def}\u{5f84}"));
}

#[test]
fn inspect_sheet_range_reports_used_range_and_sample_for_offset_table() {
    let workbook_path = create_positioned_workbook(
        "inspect_sheet_range_offset",
        "offset-inspect.xlsx",
        &[(
            "Report",
            vec![
                (2, 1, "user_id"),
                (2, 2, "region"),
                (2, 3, "sales"),
                (3, 1, "1001"),
                (3, 2, "North"),
                (3, 3, "88"),
                (4, 1, "1002"),
                (4, 2, "South"),
                (4, 3, "95"),
            ],
        )],
    );

    let inspection = inspect_sheet_range(workbook_path.to_str().unwrap(), "Report", 3).unwrap();

    // 2026-03-22: 这里先锁定 used range 会按真实非空边界返回 B3:D5，目的是避免区域探查误把空白前缀算进结果。
    assert_eq!(inspection.used_range, "B3:D5");
    assert_eq!(inspection.start_row, 3);
    assert_eq!(inspection.start_col, 2);
    assert_eq!(inspection.end_row, 5);
    assert_eq!(inspection.end_col, 4);
    assert_eq!(inspection.row_count_estimate, 3);
    assert_eq!(inspection.column_count_estimate, 3);
    // 2026-03-22: 这里先锁定样本行会保留原始工作表行号和值，目的是让上层能据此继续确认 header 与数据区。
    assert_eq!(inspection.sample_rows.len(), 3);
    assert_eq!(inspection.sample_rows[0].row_number, 3);
    assert_eq!(
        inspection.sample_rows[0].values,
        vec!["user_id", "region", "sales"]
    );
    assert_eq!(inspection.sample_rows[1].row_number, 4);
    assert_eq!(
        inspection.sample_rows[1].values,
        vec!["1001", "North", "88"]
    );
}
