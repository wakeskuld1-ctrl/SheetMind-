use excel_skill::domain::schema::{ConfidenceLevel, SchemaState};
use excel_skill::excel::header_inference::infer_header_schema;

mod common;

use crate::common::create_chinese_path_fixture;

#[test]
fn infer_multi_row_header_builds_canonical_columns() {
    let result = infer_header_schema("tests/fixtures/multi-header-sales.xlsx", "Report").unwrap();
    assert_eq!(result.columns[0].canonical_name, "region_east_sales");
    assert_eq!(result.confidence, ConfidenceLevel::High);
    assert_eq!(result.schema_state, SchemaState::Confirmed);
}

#[test]
fn low_confidence_header_stays_unconfirmed() {
    let result = infer_header_schema("tests/fixtures/title-gap-header.xlsx", "Sheet1").unwrap();
    assert_eq!(result.confidence, ConfidenceLevel::Medium);
    assert!(!result.schema_state.is_confirmed());
}

#[test]
fn non_ascii_headers_do_not_stay_high_confidence_with_empty_canonical_names() {
    let result = infer_header_schema("tests/fixtures/header-non-ascii.xlsx", "Sheet1").unwrap();

    // 2026-03-22: 这里锁定纯中文表头不会再产出空 canonical_name，目的是避免后续 DataFrame 因重复空列名直接崩溃。
    assert_eq!(result.columns[0].canonical_name, "column_1");
    assert_eq!(result.columns[1].canonical_name, "column_2");
    assert_eq!(result.columns[2].canonical_name, "column_3");
    assert!(
        result
            .columns
            .iter()
            .all(|column| !column.canonical_name.is_empty())
    );
    // 2026-03-22: 这里锁定回退列名场景必须降级为待确认，目的是阻止“识别不稳却自动放行”的误判。
    assert_eq!(result.confidence, ConfidenceLevel::Medium);
    assert_eq!(result.schema_state, SchemaState::Pending);
}

#[test]
fn infer_header_schema_accepts_chinese_windows_path() {
    let fixture_path = create_chinese_path_fixture("\u{57fa}\u{7840}\u{9500}\u{552e}-header.xlsx");
    let result = infer_header_schema(fixture_path.to_str().unwrap(), "Sales").unwrap();

    // 2026-03-22: ???????????????????????? normalize_table ???????????
    assert_eq!(result.columns[0].canonical_name, "user_id");
    assert_eq!(result.schema_state, SchemaState::Confirmed);
}
