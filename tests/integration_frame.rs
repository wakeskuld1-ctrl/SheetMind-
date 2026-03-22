use calamine::{Reader, open_workbook_auto};
use excel_skill::domain::handles::TableHandle;
use excel_skill::excel::header_inference::infer_header_schema;
use excel_skill::frame::loader::{LoadedTable, load_confirmed_table, load_table_from_table_ref};
use excel_skill::frame::region_loader::load_table_region;
use excel_skill::frame::registry::TableRegistry;
use excel_skill::frame::table_ref_store::PersistedTableRef;
use excel_skill::frame::workbook_ref_store::{
    PersistedWorkbookDraft, WorkbookDraftStore, WorkbookSheetInput,
};
use excel_skill::ops::analyze::analyze_table;
use excel_skill::ops::append::append_tables;
use excel_skill::ops::cast::{CastColumnSpec, CastTargetType, cast_column_types};
use excel_skill::ops::cluster_kmeans::cluster_kmeans;
use excel_skill::ops::decision_assistant::decision_assistant;
use excel_skill::ops::deduplicate_by_key::{
    DeduplicateKeep, OrderDirection, OrderSpec, deduplicate_by_key,
};
use excel_skill::ops::derive::{
    CaseWhenRule, DateBucketRule, DerivationSpec, DeriveCondition, DeriveConditionGroup,
    DeriveOperator, DerivePredicate, LogicalMode, derive_columns,
};
use excel_skill::ops::distinct_rows::{DistinctKeep, distinct_rows};
use excel_skill::ops::export::export_excel_workbook;
use excel_skill::ops::fill_lookup::{
    FillLookupRule, fill_missing_from_lookup, fill_missing_from_lookup_by_keys,
};
use excel_skill::ops::fill_missing_values::{
    FillMissingRule, FillMissingStrategy, fill_missing_values,
};
use excel_skill::ops::filter::{FilterCondition, FilterOperator, filter_rows};
use excel_skill::ops::format_table_for_export::{ExportFormatOptions, format_table_for_export};
use excel_skill::ops::group::{AggregationOperator, AggregationSpec, group_and_aggregate};
use excel_skill::ops::join::{JoinKeepMode, join_tables};
use excel_skill::ops::linear_regression::linear_regression;
use excel_skill::ops::logistic_regression::logistic_regression;
use excel_skill::ops::lookup_values::{LookupSelect, lookup_values, lookup_values_by_keys};
use excel_skill::ops::model_prep::{
    MissingStrategy, prepare_binary_classification_dataset, prepare_regression_dataset,
};
use excel_skill::ops::multi_table_plan::suggest_multi_table_plan;
use excel_skill::ops::normalize_text::{NormalizeTextRule, ReplacePair, normalize_text_columns};
use excel_skill::ops::parse_datetime::{
    DateTimeTargetType, ParseDateTimeRule, parse_datetime_columns,
};
use excel_skill::ops::pivot::{PivotAggregation, pivot_table};
use excel_skill::ops::preview::preview_table;
use excel_skill::ops::rename::{RenameColumnMapping, rename_columns};
use excel_skill::ops::select::select_columns;
use excel_skill::ops::sort::{SortSpec, sort_rows};
use excel_skill::ops::stat_summary::stat_summary;
use excel_skill::ops::summary::summarize_table;
use excel_skill::ops::table_links::suggest_table_links;
use excel_skill::ops::table_workflow::suggest_table_workflow;
use excel_skill::ops::top_n::top_n_rows;
use excel_skill::ops::window::{
    WindowCalculation, WindowCalculationKind, WindowOrderSpec, window_calculation,
};
use polars::prelude::{DataFrame, DataType, NamedFrom, Series};

mod common;

use crate::common::{
    create_chinese_path_fixture, create_positioned_workbook, create_test_output_path,
};

#[test]
fn load_confirmed_table_builds_polars_dataframe_from_excel_rows() {
    let inference = infer_header_schema("tests/fixtures/basic-sales.xlsx", "Sales").unwrap();
    let loaded =
        load_confirmed_table("tests/fixtures/basic-sales.xlsx", "Sales", &inference).unwrap();

    assert_eq!(loaded.handle.columns(), &["user_id", "region", "sales"]);
    assert_eq!(loaded.dataframe.height(), 2);
    assert_eq!(loaded.dataframe.width(), 3);
    assert_eq!(loaded.dataframe.column("user_id").unwrap().len(), 2);
}

#[test]
fn load_confirmed_table_accepts_chinese_windows_path() {
    let fixture_path = create_chinese_path_fixture("\u{57fa}\u{7840}\u{9500}\u{552e}-frame.xlsx");
    let inference = infer_header_schema(fixture_path.to_str().unwrap(), "Sales").unwrap();
    let loaded = load_confirmed_table(fixture_path.to_str().unwrap(), "Sales", &inference).unwrap();

    // 2026-03-22: ???? DataFrame ??????????????????? preview/select/stat_summary ?????????
    assert_eq!(loaded.handle.columns(), &["user_id", "region", "sales"]);
    assert_eq!(loaded.dataframe.height(), 2);
}

#[test]
fn load_table_region_builds_dataframe_from_explicit_range() {
    let workbook_path = create_positioned_workbook(
        "load_table_region_basic",
        "region-basic.xlsx",
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

    let loaded = load_table_region(workbook_path.to_str().unwrap(), "Report", "B3:D5", 1).unwrap();
    let preview = preview_table(&loaded.dataframe, loaded.dataframe.height()).unwrap();

    // 2026-03-22: 这里先锁定显式区域加载会只消费 B3:D5，目的是避免把整张 Sheet 的空白边界重新带回 DataFrame。
    assert_eq!(loaded.handle.columns(), &["user_id", "region", "sales"]);
    assert_eq!(loaded.dataframe.height(), 2);
    assert_eq!(preview.rows[0]["user_id"], "1001");
    assert_eq!(preview.rows[1]["sales"], "95");
}

#[test]
fn load_table_region_supports_explicit_multi_row_headers() {
    let workbook_path = create_positioned_workbook(
        "load_table_region_multi_header",
        "region-multi-header.xlsx",
        &[(
            "Report",
            vec![
                (1, 1, "Customer"),
                (1, 2, "Customer"),
                (1, 3, "Metrics"),
                (2, 1, "ID"),
                (2, 2, "Region"),
                (2, 3, "Sales"),
                (3, 1, "1001"),
                (3, 2, "North"),
                (3, 3, "88"),
                (4, 1, "1002"),
                (4, 2, "South"),
                (4, 3, "95"),
            ],
        )],
    );

    let loaded = load_table_region(workbook_path.to_str().unwrap(), "Report", "B2:D5", 2).unwrap();

    // 2026-03-22: 这里先锁定显式多层表头会按 header_row_count 合并路径，目的是避免首版区域加载只能处理单层表头。
    assert_eq!(
        loaded.handle.columns(),
        &["customer_id", "customer_region", "metrics_sales"]
    );
    assert_eq!(loaded.dataframe.height(), 2);
}

#[test]
fn load_table_region_rejects_invalid_range_syntax() {
    let workbook_path = create_positioned_workbook(
        "load_table_region_invalid",
        "region-invalid.xlsx",
        &[("Report", vec![(2, 1, "user_id")])],
    );

    let error = load_table_region(workbook_path.to_str().unwrap(), "Report", "B3", 1)
        .err()
        .unwrap();

    // 2026-03-22: 这里先锁定非法区域语法会直接报错，目的是让上层尽早修正输入而不是静默加载错误区域。
    assert!(error.to_string().contains("区域"));
}

#[test]
fn load_table_from_region_table_ref_reloads_same_explicit_region() {
    let workbook_path = create_positioned_workbook(
        "load_region_table_ref_roundtrip",
        "region-table-ref.xlsx",
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
    let persisted = PersistedTableRef::new_for_test(
        "table_region_reload",
        workbook_path.to_str().unwrap(),
        "Report",
        vec!["user_id".into(), "region".into(), "sales".into()],
        1,
        1,
        Some("B3:D5".to_string()),
    );

    let loaded = load_table_from_table_ref(&persisted).unwrap();
    let preview = preview_table(&loaded.dataframe, loaded.dataframe.height()).unwrap();

    // 2026-03-22: 这里先锁定 region table_ref 复用时仍只加载指定局部区域，目的是避免回放时退化回整张 Sheet。
    assert_eq!(loaded.handle.columns(), &["user_id", "region", "sales"]);
    assert_eq!(loaded.dataframe.height(), 2);
    assert_eq!(preview.rows[0]["user_id"], "1001");
    assert_eq!(preview.rows[1]["sales"], "95");
}

#[test]
fn registry_stores_loaded_dataframe_for_confirmed_table() {
    let inference = infer_header_schema("tests/fixtures/basic-sales.xlsx", "Sales").unwrap();
    let loaded =
        load_confirmed_table("tests/fixtures/basic-sales.xlsx", "Sales", &inference).unwrap();
    let mut registry = TableRegistry::new();

    let table_id = registry.register_loaded(loaded);

    assert_eq!(
        registry.get(&table_id).unwrap().columns(),
        &["user_id", "region", "sales"]
    );
    assert_eq!(registry.get_dataframe(&table_id).unwrap().height(), 2);
}

#[test]
fn preview_table_returns_requested_number_of_rows() {
    let inference = infer_header_schema("tests/fixtures/basic-sales.xlsx", "Sales").unwrap();
    let loaded =
        load_confirmed_table("tests/fixtures/basic-sales.xlsx", "Sales", &inference).unwrap();

    let preview = preview_table(&loaded.dataframe, 1).unwrap();

    assert_eq!(preview.columns, vec!["user_id", "region", "sales"]);
    assert_eq!(preview.rows.len(), 1);
    assert_eq!(preview.rows[0]["user_id"], "1");
}

#[test]
fn select_columns_returns_dataframe_with_only_requested_columns() {
    let inference = infer_header_schema("tests/fixtures/basic-sales.xlsx", "Sales").unwrap();
    let loaded =
        load_confirmed_table("tests/fixtures/basic-sales.xlsx", "Sales", &inference).unwrap();

    let selected = select_columns(&loaded, &["region", "sales"]).unwrap();

    assert_eq!(selected.handle.columns(), &["region", "sales"]);
    assert_eq!(selected.dataframe.width(), 2);
    assert_eq!(selected.dataframe.height(), 2);
}

#[test]
fn normalize_text_columns_applies_trim_case_replace_and_char_rules() {
    let loaded = LoadedTable {
        // 2026-03-23: 这里构造带空格、大小写与分隔符噪音的文本列，目的是先锁定文本标准化 Tool 的真实清洗顺序。
        handle: TableHandle::new_confirmed(
            "memory://normalize-text",
            "Sheet1",
            vec!["customer_code".into(), "region".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new(
                "customer_code".into(),
                vec![Some("  A- 001  "), Some(" b-002 ")],
            )
            .into(),
            Series::new("region".into(), vec![Some("  EAST  "), Some("West Zone")]).into(),
        ])
        .unwrap(),
    };

    let normalized = normalize_text_columns(
        &loaded,
        &[
            NormalizeTextRule {
                column: "customer_code".to_string(),
                trim: true,
                collapse_whitespace: true,
                lowercase: true,
                uppercase: false,
                remove_chars: vec!["-".to_string()],
                replace_pairs: vec![],
            },
            NormalizeTextRule {
                column: "region".to_string(),
                trim: true,
                collapse_whitespace: true,
                lowercase: false,
                uppercase: true,
                remove_chars: vec![],
                replace_pairs: vec![ReplacePair {
                    from: "ZONE".to_string(),
                    to: "AREA".to_string(),
                }],
            },
        ],
    )
    .unwrap();
    let preview = preview_table(&normalized.dataframe, normalized.dataframe.height()).unwrap();

    // 2026-03-23: 这里锁定多规则叠加后的最终文本结果，目的是防止 trim / 替换 / 大小写顺序漂移。
    assert_eq!(preview.rows[0]["customer_code"], "a 001");
    assert_eq!(preview.rows[1]["customer_code"], "b002");
    assert_eq!(preview.rows[0]["region"], "EAST");
    assert_eq!(preview.rows[1]["region"], "WEST AREA");
}

#[test]
fn normalize_text_columns_rejects_duplicate_rules_for_same_column() {
    let loaded = LoadedTable {
        // 2026-03-23: 这里构造最小文本列，目的是锁定同一列重复规则时会保守报错而不是偷偷覆盖。
        handle: TableHandle::new_confirmed(
            "memory://normalize-duplicate-rule",
            "Sheet1",
            vec!["region".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("region".into(), vec![Some("East")]).into(),
        ])
        .unwrap(),
    };

    let error = normalize_text_columns(
        &loaded,
        &[
            NormalizeTextRule {
                column: "region".to_string(),
                trim: true,
                collapse_whitespace: false,
                lowercase: true,
                uppercase: false,
                remove_chars: vec![],
                replace_pairs: vec![],
            },
            NormalizeTextRule {
                column: "region".to_string(),
                trim: false,
                collapse_whitespace: false,
                lowercase: false,
                uppercase: true,
                remove_chars: vec![],
                replace_pairs: vec![],
            },
        ],
    )
    .err()
    .unwrap();

    // 2026-03-23: 这里锁定重复列规则报错，目的是防止参数二义性被静默吞掉。
    assert!(error.to_string().contains("region"));
}

#[test]
fn rename_columns_renames_requested_columns_without_touching_rows() {
    let loaded = LoadedTable {
        // 2026-03-23: 这里构造最小表，目的是锁定 rename 只改 schema 不改数据内容与行数。
        handle: TableHandle::new_confirmed(
            "memory://rename-columns",
            "Sheet1",
            vec!["user_id".into(), "sales".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("user_id".into(), vec![Some("1"), Some("2")]).into(),
            Series::new("sales".into(), vec![Some("120"), Some("95")]).into(),
        ])
        .unwrap(),
    };

    let renamed = rename_columns(
        &loaded,
        &[
            RenameColumnMapping {
                from: "user_id".to_string(),
                to: "customer_id".to_string(),
            },
            RenameColumnMapping {
                from: "sales".to_string(),
                to: "revenue".to_string(),
            },
        ],
    )
    .unwrap();
    let preview = preview_table(&renamed.dataframe, renamed.dataframe.height()).unwrap();

    // 2026-03-23: 这里锁定输出列名与原始行值，目的是确保 rename 不会改坏表内容。
    assert_eq!(renamed.handle.columns(), &["customer_id", "revenue"]);
    assert_eq!(preview.rows[0]["customer_id"], "1");
    assert_eq!(preview.rows[1]["revenue"], "95");
}

#[test]
fn rename_columns_rejects_conflicting_target_names() {
    let loaded = LoadedTable {
        // 2026-03-23: 这里构造已有列名冲突场景，目的是先锁定 rename 的保守报错边界。
        handle: TableHandle::new_confirmed(
            "memory://rename-conflict",
            "Sheet1",
            vec!["user_id".into(), "sales".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("user_id".into(), vec![Some("1")]).into(),
            Series::new("sales".into(), vec![Some("120")]).into(),
        ])
        .unwrap(),
    };

    let error = rename_columns(
        &loaded,
        &[RenameColumnMapping {
            from: "user_id".to_string(),
            to: "sales".to_string(),
        }],
    )
    .err()
    .unwrap();

    // 2026-03-23: 这里锁定冲突列名报错，目的是避免新列名覆盖已有列。
    assert!(error.to_string().contains("sales"));
}

#[test]
fn rename_columns_reports_missing_source_column() {
    let loaded = LoadedTable {
        // 2026-03-23: 这里构造最小表，目的是锁定不存在源列时的明确报错。
        handle: TableHandle::new_confirmed(
            "memory://rename-missing-source",
            "Sheet1",
            vec!["user_id".into()],
        ),
        dataframe: DataFrame::new(vec![Series::new("user_id".into(), vec![Some("1")]).into()])
            .unwrap(),
    };

    let error = rename_columns(
        &loaded,
        &[RenameColumnMapping {
            from: "sales".to_string(),
            to: "revenue".to_string(),
        }],
    )
    .err()
    .unwrap();

    // 2026-03-23: 这里锁定缺失源列报错，目的是让上层尽快修正字段口径。
    assert!(error.to_string().contains("sales"));
}

#[test]
fn fill_missing_values_supports_constant_zero_and_forward_fill_strategies() {
    let loaded = LoadedTable {
        // 2026-03-22: 这里构造同时包含 null、空串和纯空白的最小表，目的是锁定通用补空 Tool 的第一版核心缺失口径。
        handle: TableHandle::new_confirmed(
            "memory://fill-missing-values",
            "Sheet1",
            vec![
                "user_id".into(),
                "city".into(),
                "sales".into(),
                "region".into(),
            ],
        ),
        dataframe: DataFrame::new(vec![
            Series::new(
                "user_id".into(),
                vec![Some("1"), Some("2"), Some("3"), Some("4")],
            )
            .into(),
            Series::new(
                "city".into(),
                vec![Option::<&str>::None, Some("Urumqi"), Some(""), Some("  ")],
            )
            .into(),
            Series::new(
                "sales".into(),
                vec![Some(""), Some("15"), Option::<&str>::None, Some("  ")],
            )
            .into(),
            Series::new(
                "region".into(),
                vec![Some("North"), Some(""), Some("West"), Option::<&str>::None],
            )
            .into(),
        ])
        .unwrap(),
    };

    let filled = fill_missing_values(
        &loaded,
        &[
            FillMissingRule {
                column: "city".to_string(),
                strategy: FillMissingStrategy::Constant,
                value: Some("Unknown".to_string()),
            },
            FillMissingRule {
                column: "sales".to_string(),
                strategy: FillMissingStrategy::Zero,
                value: None,
            },
            FillMissingRule {
                column: "region".to_string(),
                strategy: FillMissingStrategy::ForwardFill,
                value: None,
            },
        ],
    )
    .unwrap();
    let preview = preview_table(&filled.dataframe, filled.dataframe.height()).unwrap();

    // 2026-03-22: 这里锁定 constant / zero / forward_fill 三种策略的最小稳定行为，目的是先补齐表处理层最常用的补空动作。
    assert_eq!(preview.rows[0]["city"], "Unknown");
    assert_eq!(preview.rows[2]["city"], "Unknown");
    assert_eq!(preview.rows[3]["city"], "Unknown");
    assert_eq!(preview.rows[0]["sales"], "0");
    assert_eq!(preview.rows[2]["sales"], "0");
    assert_eq!(preview.rows[3]["sales"], "0");
    assert_eq!(preview.rows[1]["region"], "North");
    assert_eq!(preview.rows[3]["region"], "West");
}

#[test]
fn fill_missing_values_rejects_constant_rule_without_value() {
    let loaded = LoadedTable {
        // 2026-03-22: 这里构造最小单列表，目的是锁定 constant 缺少 value 时不会静默写入空值。
        handle: TableHandle::new_confirmed(
            "memory://fill-missing-values-invalid",
            "Sheet1",
            vec!["city".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("city".into(), vec![Option::<&str>::None, Some("Urumqi")]).into(),
        ])
        .unwrap(),
    };

    let error = fill_missing_values(
        &loaded,
        &[FillMissingRule {
            column: "city".to_string(),
            strategy: FillMissingStrategy::Constant,
            value: None,
        }],
    )
    .err()
    .unwrap();

    // 2026-03-22: 这里锁定 constant 必须显式给 value，目的是保持补空行为可解释而不是默认乱填。
    assert!(error.to_string().contains("city"));
}

#[test]
fn distinct_rows_supports_full_row_and_subset_deduplication() {
    let loaded = LoadedTable {
        // 2026-03-22: 这里构造既有整行重复也有主键重复的最小表，目的是锁定 distinct_rows 的两种核心去重口径。
        handle: TableHandle::new_confirmed(
            "memory://distinct-rows",
            "Sheet1",
            vec!["user_id".into(), "region".into(), "sales".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new(
                "user_id".into(),
                vec![Some("1"), Some("1"), Some("2"), Some("2")],
            )
            .into(),
            Series::new(
                "region".into(),
                vec![Some("East"), Some("East"), Some("West"), Some("West")],
            )
            .into(),
            Series::new(
                "sales".into(),
                vec![Some("100"), Some("100"), Some("90"), Some("95")],
            )
            .into(),
        ])
        .unwrap(),
    };

    let full_row_distinct = distinct_rows(&loaded, &[], DistinctKeep::First).unwrap();
    let full_preview = preview_table(
        &full_row_distinct.dataframe,
        full_row_distinct.dataframe.height(),
    )
    .unwrap();

    let subset_distinct =
        distinct_rows(&loaded, &["user_id", "region"], DistinctKeep::Last).unwrap();
    let subset_preview = preview_table(
        &subset_distinct.dataframe,
        subset_distinct.dataframe.height(),
    )
    .unwrap();

    // 2026-03-22: 这里锁定整行去重会去掉完全重复行，而按子集列去重时能按 keep=last 保留最后一条记录。
    assert_eq!(full_row_distinct.dataframe.height(), 3);
    assert_eq!(full_preview.rows[2]["sales"], "95");
    assert_eq!(subset_distinct.dataframe.height(), 2);
    assert_eq!(subset_preview.rows[0]["sales"], "100");
    assert_eq!(subset_preview.rows[1]["sales"], "95");
}

#[test]
fn distinct_rows_reports_missing_subset_column() {
    let loaded = LoadedTable {
        // 2026-03-22: 这里构造最小表，目的是锁定子集列不存在时不会静默退化成整表去重。
        handle: TableHandle::new_confirmed(
            "memory://distinct-rows-missing",
            "Sheet1",
            vec!["user_id".into(), "region".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("user_id".into(), vec![Some("1"), Some("2")]).into(),
            Series::new("region".into(), vec![Some("East"), Some("West")]).into(),
        ])
        .unwrap(),
    };

    let error = distinct_rows(&loaded, &["user_id", "sales"], DistinctKeep::First)
        .err()
        .unwrap();

    // 2026-03-22: 这里锁定缺失 subset 列会明确报错，目的是保持去重口径完全显式。
    assert!(error.to_string().contains("sales"));
}

#[test]
fn deduplicate_by_key_keeps_first_record_without_order_by() {
    let loaded = LoadedTable {
        // 2026-03-22: 这里构造同一业务键重复出现的最小表，目的是锁定 deduplicate_by_key 在未提供排序规则时默认保留首条记录。
        handle: TableHandle::new_confirmed(
            "memory://deduplicate-by-key-first",
            "Sheet1",
            vec!["user_id".into(), "region".into(), "sales".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new(
                "user_id".into(),
                vec![Some("1"), Some("1"), Some("2"), Some("2")],
            )
            .into(),
            Series::new(
                "region".into(),
                vec![Some("East"), Some("East"), Some("West"), Some("West")],
            )
            .into(),
            Series::new(
                "sales".into(),
                vec![Some("100"), Some("120"), Some("90"), Some("95")],
            )
            .into(),
        ])
        .unwrap(),
    };

    let deduplicated =
        deduplicate_by_key(&loaded, &["user_id", "region"], &[], DeduplicateKeep::First).unwrap();
    let preview = preview_table(&deduplicated.dataframe, deduplicated.dataframe.height()).unwrap();

    // 2026-03-22: 这里锁定未给排序规则时的默认行为，目的是让“按主键去重”先具备最保守、最好解释的第一版语义。
    assert_eq!(deduplicated.dataframe.height(), 2);
    assert_eq!(preview.rows[0]["sales"], "100");
    assert_eq!(preview.rows[1]["sales"], "90");
}

#[test]
fn deduplicate_by_key_keeps_last_record_by_order_rule() {
    let loaded = LoadedTable {
        // 2026-03-22: 这里构造同键多行且更新时间递增的最小表，目的是锁定 deduplicate_by_key 能按排序规则保留最后一条有效记录。
        handle: TableHandle::new_confirmed(
            "memory://deduplicate-by-key-order",
            "Sheet1",
            vec!["user_id".into(), "updated_at".into(), "score".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new(
                "user_id".into(),
                vec![Some("1"), Some("1"), Some("2"), Some("2")],
            )
            .into(),
            Series::new(
                "updated_at".into(),
                vec![
                    Some("2026-03-01"),
                    Some("2026-03-03"),
                    Some("2026-03-02"),
                    Some("2026-03-04"),
                ],
            )
            .into(),
            Series::new(
                "score".into(),
                vec![Some("70"), Some("85"), Some("90"), Some("88")],
            )
            .into(),
        ])
        .unwrap(),
    };

    let deduplicated = deduplicate_by_key(
        &loaded,
        &["user_id"],
        &[OrderSpec {
            column: "updated_at".to_string(),
            direction: OrderDirection::Asc,
        }],
        DeduplicateKeep::Last,
    )
    .unwrap();
    let preview = preview_table(&deduplicated.dataframe, deduplicated.dataframe.height()).unwrap();

    // 2026-03-22: 这里锁定先排序再保留末条的行为，目的是让业务“保留最新记录”的去重诉求可以稳定落在 Tool 层。
    assert_eq!(deduplicated.dataframe.height(), 2);
    assert_eq!(preview.rows[0]["updated_at"], "2026-03-03");
    assert_eq!(preview.rows[0]["score"], "85");
    assert_eq!(preview.rows[1]["updated_at"], "2026-03-04");
    assert_eq!(preview.rows[1]["score"], "88");
}

#[test]
fn deduplicate_by_key_reports_missing_key_column() {
    let loaded = LoadedTable {
        // 2026-03-22: 这里构造最小表，目的是锁定 key 列缺失时会显式报错而不是悄悄退化成其它去重逻辑。
        handle: TableHandle::new_confirmed(
            "memory://deduplicate-by-key-missing-key",
            "Sheet1",
            vec!["user_id".into(), "sales".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("user_id".into(), vec![Some("1"), Some("2")]).into(),
            Series::new("sales".into(), vec![Some("100"), Some("90")]).into(),
        ])
        .unwrap(),
    };

    let error = deduplicate_by_key(&loaded, &["customer_id"], &[], DeduplicateKeep::First)
        .err()
        .unwrap();

    // 2026-03-22: 这里锁定错误会指出缺失的键列，目的是让上层快速修正字段口径。
    assert!(error.to_string().contains("customer_id"));
}

#[test]
fn deduplicate_by_key_reports_missing_order_column() {
    let loaded = LoadedTable {
        // 2026-03-22: 这里构造最小表，目的是锁定排序列缺失时不会继续用不完整规则执行主键去重。
        handle: TableHandle::new_confirmed(
            "memory://deduplicate-by-key-missing-order",
            "Sheet1",
            vec!["user_id".into(), "updated_at".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("user_id".into(), vec![Some("1"), Some("1")]).into(),
            Series::new(
                "updated_at".into(),
                vec![Some("2026-03-01"), Some("2026-03-02")],
            )
            .into(),
        ])
        .unwrap(),
    };

    let error = deduplicate_by_key(
        &loaded,
        &["user_id"],
        &[OrderSpec {
            column: "score".to_string(),
            direction: OrderDirection::Desc,
        }],
        DeduplicateKeep::Last,
    )
    .err()
    .unwrap();

    // 2026-03-22: 这里锁定错误会指向缺失的排序列，目的是避免“保留最新/最大值”这类语义被静默做错。
    assert!(error.to_string().contains("score"));
}

#[test]
fn format_table_for_export_reorders_and_renames_columns() {
    let loaded = LoadedTable {
        // 2026-03-22: 这里构造最小导出表，目的是锁定导出前整理会同时生效列顺序与表头别名。
        handle: TableHandle::new_confirmed(
            "memory://format-table-export",
            "Sheet1",
            vec!["user_id".into(), "region".into(), "sales".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("user_id".into(), vec![Some("1"), Some("2")]).into(),
            Series::new("region".into(), vec![Some("East"), Some("West")]).into(),
            Series::new("sales".into(), vec![Some("120"), Some("95")]).into(),
        ])
        .unwrap(),
    };

    let formatted = format_table_for_export(
        &loaded,
        &ExportFormatOptions {
            column_order: vec![
                "region".to_string(),
                "sales".to_string(),
                "user_id".to_string(),
            ],
            rename_mappings: vec![
                RenameColumnMapping {
                    from: "region".to_string(),
                    to: "区域".to_string(),
                },
                RenameColumnMapping {
                    from: "sales".to_string(),
                    to: "销售额".to_string(),
                },
                RenameColumnMapping {
                    from: "user_id".to_string(),
                    to: "客户ID".to_string(),
                },
            ],
            drop_unspecified_columns: false,
        },
    )
    .unwrap();
    let preview = preview_table(&formatted.dataframe, formatted.dataframe.height()).unwrap();

    // 2026-03-22: 这里锁定导出前整理后的最终列布局，目的是让后续 workbook 组装层消费稳定、面向客户的表头结构。
    assert_eq!(formatted.handle.columns(), &["区域", "销售额", "客户ID"]);
    assert_eq!(preview.rows[0]["区域"], "East");
    assert_eq!(preview.rows[0]["销售额"], "120");
    assert_eq!(preview.rows[1]["客户ID"], "2");
}

#[test]
fn format_table_for_export_drops_unspecified_columns_when_requested() {
    let loaded = LoadedTable {
        // 2026-03-22: 这里构造包含额外技术列的最小表，目的是锁定 drop_unspecified_columns=true 时只保留客户可见列。
        handle: TableHandle::new_confirmed(
            "memory://format-table-export-drop",
            "Sheet1",
            vec![
                "user_id".into(),
                "region".into(),
                "sales".into(),
                "debug_tag".into(),
            ],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("user_id".into(), vec![Some("1")]).into(),
            Series::new("region".into(), vec![Some("East")]).into(),
            Series::new("sales".into(), vec![Some("120")]).into(),
            Series::new("debug_tag".into(), vec![Some("internal")]).into(),
        ])
        .unwrap(),
    };

    let formatted = format_table_for_export(
        &loaded,
        &ExportFormatOptions {
            column_order: vec!["user_id".to_string(), "sales".to_string()],
            rename_mappings: vec![],
            drop_unspecified_columns: true,
        },
    )
    .unwrap();

    // 2026-03-22: 这里锁定导出裁剪语义，目的是避免内部中间列被错误带进客户交付报表。
    assert_eq!(formatted.handle.columns(), &["user_id", "sales"]);
    assert_eq!(formatted.dataframe.width(), 2);
}

#[test]
fn format_table_for_export_reports_missing_column_in_column_order() {
    let loaded = LoadedTable {
        // 2026-03-22: 这里构造最小表，目的是锁定导出列顺序里写错字段时会明确报错。
        handle: TableHandle::new_confirmed(
            "memory://format-table-export-missing",
            "Sheet1",
            vec!["user_id".into(), "sales".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("user_id".into(), vec![Some("1")]).into(),
            Series::new("sales".into(), vec![Some("120")]).into(),
        ])
        .unwrap(),
    };

    let error = format_table_for_export(
        &loaded,
        &ExportFormatOptions {
            column_order: vec!["region".to_string()],
            rename_mappings: vec![],
            drop_unspecified_columns: false,
        },
    )
    .err()
    .unwrap();

    // 2026-03-22: 这里锁定缺失列错误，目的是让上层在导出前就修正口径而不是生成残缺报表。
    assert!(error.to_string().contains("region"));
}

#[test]
fn workbook_draft_roundtrip_preserves_multiple_sheets() {
    let workbook_ref = WorkbookDraftStore::create_workbook_ref();
    let store = WorkbookDraftStore::workspace_default().unwrap();
    let draft = PersistedWorkbookDraft::from_sheet_inputs(
        &workbook_ref,
        vec![
            WorkbookSheetInput {
                sheet_name: "概览".to_string(),
                source_refs: vec!["result_summary".to_string()],
                dataframe: DataFrame::new(vec![
                    Series::new("区域".into(), vec![Some("East"), Some("West")]).into(),
                    Series::new("销售额".into(), vec![Some("120"), Some("95")]).into(),
                ])
                .unwrap(),
            },
            WorkbookSheetInput {
                sheet_name: "明细".to_string(),
                source_refs: vec!["result_detail".to_string()],
                dataframe: DataFrame::new(vec![
                    Series::new("客户ID".into(), vec![Some("1"), Some("2")]).into(),
                    Series::new("产品".into(), vec![Some("A"), Some("B")]).into(),
                ])
                .unwrap(),
            },
        ],
    )
    .unwrap();

    store.save(&draft).unwrap();
    let reloaded = store.load(&workbook_ref).unwrap();
    let first_sheet = reloaded.worksheets[0].to_dataframe().unwrap();
    let second_sheet = reloaded.worksheets[1].to_dataframe().unwrap();

    // 2026-03-22: 这里锁定 workbook 草稿会完整保留多 Sheet 快照，目的是让导出动作脱离原始 Excel 也能稳定执行。
    assert_eq!(reloaded.worksheets.len(), 2);
    assert_eq!(reloaded.worksheets[0].sheet_name, "概览");
    assert_eq!(reloaded.worksheets[1].sheet_name, "明细");
    assert_eq!(first_sheet.height(), 2);
    assert_eq!(first_sheet.get_column_names(), &["区域", "销售额"]);
    assert_eq!(second_sheet.get_column_names(), &["客户ID", "产品"]);
}

#[test]
fn export_excel_workbook_writes_all_sheets_from_draft() {
    let output_path = create_test_output_path("export_excel_workbook_frame", "xlsx");
    let draft = PersistedWorkbookDraft::from_sheet_inputs(
        "workbook_export_frame",
        vec![
            WorkbookSheetInput {
                sheet_name: "Summary".to_string(),
                source_refs: vec!["result_summary".to_string()],
                dataframe: DataFrame::new(vec![
                    Series::new("region".into(), vec![Some("East"), Some("West")]).into(),
                    Series::new("sales".into(), vec![Some("120"), Some("95")]).into(),
                ])
                .unwrap(),
            },
            WorkbookSheetInput {
                sheet_name: "Detail".to_string(),
                source_refs: vec!["result_detail".to_string()],
                dataframe: DataFrame::new(vec![
                    Series::new("user_id".into(), vec![Some("1"), Some("2")]).into(),
                    Series::new("product".into(), vec![Some("A"), Some("B")]).into(),
                ])
                .unwrap(),
            },
        ],
    )
    .unwrap();

    export_excel_workbook(&draft, output_path.to_str().unwrap()).unwrap();

    let mut workbook = open_workbook_auto(&output_path).unwrap();
    let summary = workbook.worksheet_range("Summary").unwrap();
    let detail = workbook.worksheet_range("Detail").unwrap();

    // 2026-03-22: 这里锁定多 Sheet 导出后的工作簿可以再次被标准 Excel 读取，目的是保证 compose -> export 链路具备真实交付能力。
    assert_eq!(summary.get((0, 0)).unwrap().to_string(), "region");
    assert_eq!(summary.get((1, 1)).unwrap().to_string(), "120");
    assert_eq!(detail.get((0, 0)).unwrap().to_string(), "user_id");
    assert_eq!(detail.get((2, 1)).unwrap().to_string(), "B");
}

#[test]
fn fill_missing_from_lookup_only_fills_blank_or_null_values() {
    let base = LoadedTable {
        // 2026-03-23: 这里构造同时含 null、空字符串和非空值的主表，目的是锁定“只补空不覆盖”的核心边界。
        handle: TableHandle::new_confirmed(
            "memory://fill-base",
            "Base",
            vec!["user_id".into(), "city".into(), "tier".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("user_id".into(), vec![Some("1"), Some("2"), Some("3")]).into(),
            Series::new(
                "city".into(),
                vec![Option::<&str>::None, Some("Urumqi"), Some("")],
            )
            .into(),
            Series::new(
                "tier".into(),
                vec![Some(""), Some("A"), Option::<&str>::None],
            )
            .into(),
        ])
        .unwrap(),
    };
    let lookup = LoadedTable {
        // 2026-03-23: 这里构造一张唯一键查值表，目的是验证多字段补值时的稳定查找行为。
        handle: TableHandle::new_confirmed(
            "memory://fill-lookup",
            "Lookup",
            vec!["user_id".into(), "city".into(), "tier".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("user_id".into(), vec![Some("1"), Some("2"), Some("3")]).into(),
            Series::new(
                "city".into(),
                vec![Some("Beijing"), Some("Shanghai"), Some("Shenzhen")],
            )
            .into(),
            Series::new("tier".into(), vec![Some("S"), Some("A"), Some("B")]).into(),
        ])
        .unwrap(),
    };

    let filled = fill_missing_from_lookup(
        &base,
        &lookup,
        "user_id",
        "user_id",
        &[
            FillLookupRule {
                base_column: "city".to_string(),
                lookup_column: "city".to_string(),
            },
            FillLookupRule {
                base_column: "tier".to_string(),
                lookup_column: "tier".to_string(),
            },
        ],
    )
    .unwrap();
    let preview = preview_table(&filled.dataframe, filled.dataframe.height()).unwrap();

    // 2026-03-23: 这里锁定 null / 空字符串会被补齐，而已有值不会被覆盖，目的是避免 lookup 把用户原始数据刷掉。
    assert_eq!(preview.rows[0]["city"], "Beijing");
    assert_eq!(preview.rows[1]["city"], "Urumqi");
    assert_eq!(preview.rows[2]["city"], "Shenzhen");
    assert_eq!(preview.rows[0]["tier"], "S");
    assert_eq!(preview.rows[1]["tier"], "A");
    assert_eq!(preview.rows[2]["tier"], "B");
}

#[test]
fn fill_missing_from_lookup_rejects_duplicate_lookup_keys() {
    let base = LoadedTable {
        // 2026-03-23: 这里构造最小主表，目的是单独锁定 lookup 键不唯一时的保守报错。
        handle: TableHandle::new_confirmed(
            "memory://fill-duplicate-base",
            "Base",
            vec!["user_id".into(), "city".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("user_id".into(), vec![Some("1")]).into(),
            Series::new("city".into(), vec![Option::<&str>::None]).into(),
        ])
        .unwrap(),
    };
    let lookup = LoadedTable {
        // 2026-03-23: 这里故意让 key 重复，目的是验证第一版 fill lookup 不会在多命中时自作主张选一条。
        handle: TableHandle::new_confirmed(
            "memory://fill-duplicate-lookup",
            "Lookup",
            vec!["user_id".into(), "city".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("user_id".into(), vec![Some("1"), Some("1")]).into(),
            Series::new("city".into(), vec![Some("Beijing"), Some("Shanghai")]).into(),
        ])
        .unwrap(),
    };

    let error = fill_missing_from_lookup(
        &base,
        &lookup,
        "user_id",
        "user_id",
        &[FillLookupRule {
            base_column: "city".to_string(),
            lookup_column: "city".to_string(),
        }],
    )
    .err()
    .unwrap();

    // 2026-03-23: 这里锁定重复 key 报错，目的是要求上层先显式去重再补值。
    assert!(error.to_string().contains("user_id"));
}

#[test]
fn fill_missing_from_lookup_keeps_original_value_when_lookup_key_missing() {
    let base = LoadedTable {
        // 2026-03-23: 这里构造一条查不到 lookup 的主表记录，目的是锁定未命中时保持原值不报错。
        handle: TableHandle::new_confirmed(
            "memory://fill-miss-base",
            "Base",
            vec!["user_id".into(), "city".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("user_id".into(), vec![Some("9")]).into(),
            Series::new("city".into(), vec![Some("")]).into(),
        ])
        .unwrap(),
    };
    let lookup = LoadedTable {
        // 2026-03-23: 这里故意不给命中 key，目的是验证回填逻辑不会捏造不存在的值。
        handle: TableHandle::new_confirmed(
            "memory://fill-miss-lookup",
            "Lookup",
            vec!["user_id".into(), "city".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("user_id".into(), vec![Some("1")]).into(),
            Series::new("city".into(), vec![Some("Beijing")]).into(),
        ])
        .unwrap(),
    };

    let filled = fill_missing_from_lookup(
        &base,
        &lookup,
        "user_id",
        "user_id",
        &[FillLookupRule {
            base_column: "city".to_string(),
            lookup_column: "city".to_string(),
        }],
    )
    .unwrap();
    let preview = preview_table(&filled.dataframe, filled.dataframe.height()).unwrap();

    // 2026-03-23: 这里锁定 lookup 未命中时保留原值，目的是保持“只补得到的值”的保守行为。
    assert_eq!(preview.rows[0]["city"], "");
}

#[test]
fn fill_missing_from_lookup_by_composite_keys_fills_matching_rows_only() {
    let base = LoadedTable {
        // 2026-03-23: 这里构造“客户 + 月份”复合键主表，目的是先锁定回填能力不再只支持单键。
        handle: TableHandle::new_confirmed(
            "memory://fill-composite-base",
            "Base",
            vec![
                "customer_id".into(),
                "month".into(),
                "city".into(),
                "tier".into(),
            ],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("customer_id".into(), vec![Some("1"), Some("1"), Some("2")]).into(),
            Series::new(
                "month".into(),
                vec![Some("2026-01"), Some("2026-02"), Some("2026-01")],
            )
            .into(),
            Series::new(
                "city".into(),
                vec![Option::<&str>::None, Some(""), Some("Urumqi")],
            )
            .into(),
            Series::new(
                "tier".into(),
                vec![Some(""), Option::<&str>::None, Some("A")],
            )
            .into(),
        ])
        .unwrap(),
    };
    let lookup = LoadedTable {
        // 2026-03-23: 这里构造复合键 lookup 表，目的是锁定只有键完全一致时才会回填对应月份值。
        handle: TableHandle::new_confirmed(
            "memory://fill-composite-lookup",
            "Lookup",
            vec![
                "customer_id".into(),
                "month".into(),
                "city".into(),
                "tier".into(),
            ],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("customer_id".into(), vec![Some("1"), Some("1"), Some("2")]).into(),
            Series::new(
                "month".into(),
                vec![Some("2026-01"), Some("2026-02"), Some("2026-01")],
            )
            .into(),
            Series::new(
                "city".into(),
                vec![Some("Beijing"), Some("Shanghai"), Some("Shenzhen")],
            )
            .into(),
            Series::new("tier".into(), vec![Some("A"), Some("B"), Some("C")]).into(),
        ])
        .unwrap(),
    };

    let filled = fill_missing_from_lookup_by_keys(
        &base,
        &lookup,
        &["customer_id", "month"],
        &["customer_id", "month"],
        &[
            FillLookupRule {
                base_column: "city".to_string(),
                lookup_column: "city".to_string(),
            },
            FillLookupRule {
                base_column: "tier".to_string(),
                lookup_column: "tier".to_string(),
            },
        ],
    )
    .unwrap();
    let preview = preview_table(&filled.dataframe, filled.dataframe.height()).unwrap();

    // 2026-03-23: 这里锁定复合键回填命中的是“同客户同月份”，目的是避免不同月份数据串值。
    assert_eq!(preview.rows[0]["city"], "Beijing");
    assert_eq!(preview.rows[0]["tier"], "A");
    assert_eq!(preview.rows[1]["city"], "Shanghai");
    assert_eq!(preview.rows[1]["tier"], "B");
    assert_eq!(preview.rows[2]["city"], "Urumqi");
    assert_eq!(preview.rows[2]["tier"], "A");
}

#[test]
fn fill_missing_from_lookup_rejects_mismatched_composite_key_lengths() {
    let base = LoadedTable {
        // 2026-03-23: 这里构造最小主表，目的是单独锁定复合键列数不一致时的明确报错。
        handle: TableHandle::new_confirmed(
            "memory://fill-key-arity-base",
            "Base",
            vec!["customer_id".into(), "month".into(), "city".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("customer_id".into(), vec![Some("1")]).into(),
            Series::new("month".into(), vec![Some("2026-01")]).into(),
            Series::new("city".into(), vec![Option::<&str>::None]).into(),
        ])
        .unwrap(),
    };
    let lookup = LoadedTable {
        // 2026-03-23: 这里构造最小 lookup 表，目的是让失败原因只落在复合键规格上。
        handle: TableHandle::new_confirmed(
            "memory://fill-key-arity-lookup",
            "Lookup",
            vec!["customer_id".into(), "month".into(), "city".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("customer_id".into(), vec![Some("1")]).into(),
            Series::new("month".into(), vec![Some("2026-01")]).into(),
            Series::new("city".into(), vec![Some("Beijing")]).into(),
        ])
        .unwrap(),
    };

    let error = fill_missing_from_lookup_by_keys(
        &base,
        &lookup,
        &["customer_id", "month"],
        &["customer_id"],
        &[FillLookupRule {
            base_column: "city".to_string(),
            lookup_column: "city".to_string(),
        }],
    )
    .err()
    .unwrap();

    // 2026-03-23: 这里锁定复合键列数不一致会直白报错，目的是避免用户误以为系统会自动猜测剩余键列。
    assert!(error.to_string().contains("键列数量"));
}

#[test]
fn pivot_table_builds_sum_wide_table_by_row_and_column_dimensions() {
    let loaded = LoadedTable {
        // 2026-03-23: 这里构造最小行列透视场景，目的是锁定 pivot 的宽表输出结构与 sum 聚合结果。
        handle: TableHandle::new_confirmed(
            "memory://pivot-sum",
            "Sheet1",
            vec!["region".into(), "month".into(), "sales".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new(
                "region".into(),
                vec![Some("East"), Some("East"), Some("West")],
            )
            .into(),
            Series::new("month".into(), vec![Some("Jan"), Some("Feb"), Some("Jan")]).into(),
            Series::new("sales".into(), vec![100_i64, 80_i64, 90_i64]).into(),
        ])
        .unwrap(),
    };

    let pivoted = pivot_table(
        &loaded,
        &["region"],
        &["month"],
        &["sales"],
        PivotAggregation::Sum,
    )
    .unwrap();
    let preview = preview_table(&pivoted.dataframe, pivoted.dataframe.height()).unwrap();

    // 2026-03-23: 这里锁定输出列与透视结果，目的是确保 V1 宽表结构稳定可导出。
    assert_eq!(pivoted.handle.columns(), &["region", "Feb", "Jan"]);
    assert_eq!(preview.rows[0]["region"], "East");
    assert_eq!(preview.rows[0]["Feb"], "80");
    assert_eq!(preview.rows[0]["Jan"], "100");
    assert_eq!(preview.rows[1]["region"], "West");
    assert_eq!(preview.rows[1]["Jan"], "90");
}

#[test]
fn pivot_table_rejects_non_numeric_sum_value_column() {
    let loaded = LoadedTable {
        // 2026-03-23: 这里构造非数值 value 列，目的是锁定 sum / mean 聚合的类型门禁。
        handle: TableHandle::new_confirmed(
            "memory://pivot-invalid",
            "Sheet1",
            vec!["region".into(), "month".into(), "label".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("region".into(), vec![Some("East")]).into(),
            Series::new("month".into(), vec![Some("Jan")]).into(),
            Series::new("label".into(), vec![Some("vip")]).into(),
        ])
        .unwrap(),
    };

    let error = pivot_table(
        &loaded,
        &["region"],
        &["month"],
        &["label"],
        PivotAggregation::Sum,
    )
    .err()
    .unwrap();

    // 2026-03-23: 这里锁定非数值聚合报错，目的是避免把文本列误当数值去算。
    assert!(error.to_string().contains("label"));
}

#[test]
fn pivot_table_builds_mean_values_for_repeated_cells() {
    let loaded = LoadedTable {
        // 2026-03-23: 这里构造同一透视单元格多行命中场景，目的是锁定 mean 聚合不是最后一条覆盖。
        handle: TableHandle::new_confirmed(
            "memory://pivot-mean",
            "Sheet1",
            vec!["region".into(), "month".into(), "sales".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new(
                "region".into(),
                vec![Some("East"), Some("East"), Some("East"), Some("West")],
            )
            .into(),
            Series::new(
                "month".into(),
                vec![Some("Jan"), Some("Jan"), Some("Feb"), Some("Jan")],
            )
            .into(),
            Series::new("sales".into(), vec![100.0_f64, 80.0, 60.0, 90.0]).into(),
        ])
        .unwrap(),
    };

    let pivoted = pivot_table(
        &loaded,
        &["region"],
        &["month"],
        &["sales"],
        PivotAggregation::Mean,
    )
    .unwrap();
    let preview = preview_table(&pivoted.dataframe, pivoted.dataframe.height()).unwrap();

    // 2026-03-23: 这里锁定 East-Jan 会取均值 90，目的是验证 mean 聚合累加器正确工作。
    assert_eq!(preview.rows[0]["region"], "East");
    assert_eq!(preview.rows[0]["Jan"], "90");
    assert_eq!(preview.rows[0]["Feb"], "60");
}


#[test]
fn parse_datetime_columns_normalizes_date_and_datetime_strings() {
    let loaded = LoadedTable {
        // 2026-03-23: 这里构造常见日期与日期时间文本，目的是锁定标准化后输出为统一 ISO 口径。
        handle: TableHandle::new_confirmed(
            "memory://parse-datetime",
            "Sheet1",
            vec!["biz_date".into(), "created_at".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new(
                "biz_date".into(),
                vec![Some("2026/03/01"), Some("2026-03-02")],
            )
            .into(),
            Series::new(
                "created_at".into(),
                vec![Some("2026-03-01 8:30"), Some("2026-03-02T09:15:20")],
            )
            .into(),
        ])
        .unwrap(),
    };

    let parsed = parse_datetime_columns(
        &loaded,
        &[
            ParseDateTimeRule {
                column: "biz_date".to_string(),
                target_type: DateTimeTargetType::Date,
            },
            ParseDateTimeRule {
                column: "created_at".to_string(),
                target_type: DateTimeTargetType::DateTime,
            },
        ],
    )
    .unwrap();
    let preview = preview_table(&parsed.dataframe, parsed.dataframe.height()).unwrap();

    // 2026-03-23: 这里锁定标准日期与日期时间格式，目的是让时间类分析的上游口径先稳定下来。
    assert_eq!(preview.rows[0]["biz_date"], "2026-03-01");
    assert_eq!(preview.rows[1]["biz_date"], "2026-03-02");
    assert_eq!(preview.rows[0]["created_at"], "2026-03-01 08:30:00");
    assert_eq!(preview.rows[1]["created_at"], "2026-03-02 09:15:20");
}

#[test]
fn parse_datetime_columns_rejects_invalid_non_empty_values() {
    let loaded = LoadedTable {
        // 2026-03-23: 这里构造非法日期文本，目的是先锁定非空但不可解析时必须报错而不是悄悄保留脏值。
        handle: TableHandle::new_confirmed(
            "memory://parse-datetime-invalid",
            "Sheet1",
            vec!["biz_date".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("biz_date".into(), vec![Some("2026-13-99")]).into(),
        ])
        .unwrap(),
    };

    let error = parse_datetime_columns(
        &loaded,
        &[ParseDateTimeRule {
            column: "biz_date".to_string(),
            target_type: DateTimeTargetType::Date,
        }],
    )
    .err()
    .unwrap();

    // 2026-03-23: 这里锁定非法值报错，目的是避免时间字段在进入后续分析前继续带脏数据流转。
    assert!(error.to_string().contains("biz_date"));
}

#[test]
fn parse_datetime_columns_rejects_invalid_calendar_dates() {
    let loaded = LoadedTable {
        // 2026-03-22: 这里构造“月份合法但日历不存在”的日期，目的是锁定 parse_datetime_columns 会拦住 2 月 30 日这类假日期。
        handle: TableHandle::new_confirmed(
            "memory://parse-datetime-calendar-invalid",
            "Sheet1",
            vec!["biz_date".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("biz_date".into(), vec![Some("2026-02-30")]).into(),
        ])
        .unwrap(),
    };

    let error = parse_datetime_columns(
        &loaded,
        &[ParseDateTimeRule {
            column: "biz_date".to_string(),
            target_type: DateTimeTargetType::Date,
        }],
    )
    .err()
    .unwrap();

    assert!(error.to_string().contains("biz_date"));
}

#[test]
fn parse_datetime_columns_accepts_excel_serial_date_numbers() {
    let loaded = LoadedTable {
        // 2026-03-22: 这里构造 Excel 原生日期序列值，目的是锁定 parse_datetime_columns 能直接吃 1900 系统序列值而不是只认文本。
        handle: TableHandle::new_confirmed(
            "memory://parse-datetime-serial",
            "Sheet1",
            vec!["biz_date".into(), "created_at".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("biz_date".into(), vec![61_i64]).into(),
            Series::new("created_at".into(), vec![61.5_f64]).into(),
        ])
        .unwrap(),
    };

    let parsed = parse_datetime_columns(
        &loaded,
        &[
            ParseDateTimeRule {
                column: "biz_date".to_string(),
                target_type: DateTimeTargetType::Date,
            },
            ParseDateTimeRule {
                column: "created_at".to_string(),
                target_type: DateTimeTargetType::DateTime,
            },
        ],
    )
    .unwrap();
    let preview = preview_table(&parsed.dataframe, parsed.dataframe.height()).unwrap();

    assert_eq!(preview.rows[0]["biz_date"], "1900-03-01");
    assert_eq!(preview.rows[0]["created_at"], "1900-03-01 12:00:00");
}

#[test]
fn lookup_values_appends_selected_columns_without_changing_row_count() {
    let base = LoadedTable {
        // 2026-03-23: 这里构造主表，目的是锁定轻量查值只带回列、不改变主表行数与顺序。
        handle: TableHandle::new_confirmed(
            "memory://lookup-base",
            "Base",
            vec!["user_id".into(), "amount".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("user_id".into(), vec![Some("1"), Some("2"), Some("3")]).into(),
            Series::new("amount".into(), vec![120_i64, 95_i64, 88_i64]).into(),
        ])
        .unwrap(),
    };
    let lookup = LoadedTable {
        // 2026-03-23: 这里构造唯一键查值表，目的是验证 VLOOKUP/XLOOKUP 心智下的带列行为。
        handle: TableHandle::new_confirmed(
            "memory://lookup-dict",
            "Lookup",
            vec!["user_id".into(), "city".into(), "tier".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("user_id".into(), vec![Some("1"), Some("2"), Some("3")]).into(),
            Series::new(
                "city".into(),
                vec![Some("Beijing"), Some("Shanghai"), Some("Shenzhen")],
            )
            .into(),
            Series::new("tier".into(), vec![Some("A"), Some("B"), Some("C")]).into(),
        ])
        .unwrap(),
    };

    let looked_up = lookup_values(
        &base,
        &lookup,
        "user_id",
        "user_id",
        &[
            LookupSelect {
                lookup_column: "city".to_string(),
                output_column: "customer_city".to_string(),
            },
            LookupSelect {
                lookup_column: "tier".to_string(),
                output_column: "customer_tier".to_string(),
            },
        ],
    )
    .unwrap();
    let preview = preview_table(&looked_up.dataframe, looked_up.dataframe.height()).unwrap();

    // 2026-03-23: 这里锁定查值后只新增输出列，目的是避免 lookup_values 退化成 join 语义。
    assert_eq!(
        looked_up.handle.columns(),
        &["user_id", "amount", "customer_city", "customer_tier"]
    );
    assert_eq!(looked_up.dataframe.height(), 3);
    assert_eq!(preview.rows[0]["customer_city"], "Beijing");
    assert_eq!(preview.rows[1]["customer_tier"], "B");
    assert_eq!(preview.rows[2]["amount"], "88");
}

#[test]
fn lookup_values_keeps_empty_output_when_lookup_key_missing() {
    let base = LoadedTable {
        // 2026-03-23: 这里构造带未命中 key 的主表，目的是锁定查不到时输出为空而不是报错或捏造值。
        handle: TableHandle::new_confirmed(
            "memory://lookup-miss-base",
            "Base",
            vec!["user_id".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("user_id".into(), vec![Some("1"), Some("9")]).into(),
        ])
        .unwrap(),
    };
    let lookup = LoadedTable {
        // 2026-03-23: 这里故意只给部分 key，目的是验证轻量查值的未命中保守行为。
        handle: TableHandle::new_confirmed(
            "memory://lookup-miss-dict",
            "Lookup",
            vec!["user_id".into(), "city".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("user_id".into(), vec![Some("1")]).into(),
            Series::new("city".into(), vec![Some("Beijing")]).into(),
        ])
        .unwrap(),
    };

    let looked_up = lookup_values(
        &base,
        &lookup,
        "user_id",
        "user_id",
        &[LookupSelect {
            lookup_column: "city".to_string(),
            output_column: "city".to_string(),
        }],
    )
    .unwrap();
    let preview = preview_table(&looked_up.dataframe, looked_up.dataframe.height()).unwrap();

    // 2026-03-23: 这里锁定未命中时输出列为空，目的是让上层能继续决定是否回填默认值。
    assert_eq!(preview.rows[0]["city"], "Beijing");
    assert_eq!(preview.rows[1]["city"], "");
}

#[test]
fn lookup_values_rejects_duplicate_lookup_keys() {
    let base = LoadedTable {
        // 2026-03-23: 这里构造最小主表，目的是单独锁定 lookup key 不唯一时的报错边界。
        handle: TableHandle::new_confirmed(
            "memory://lookup-dup-base",
            "Base",
            vec!["user_id".into()],
        ),
        dataframe: DataFrame::new(vec![Series::new("user_id".into(), vec![Some("1")]).into()])
            .unwrap(),
    };
    let lookup = LoadedTable {
        // 2026-03-23: 这里故意让查值表 key 重复，目的是避免系统在多命中时私自挑一条。
        handle: TableHandle::new_confirmed(
            "memory://lookup-dup-dict",
            "Lookup",
            vec!["user_id".into(), "city".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("user_id".into(), vec![Some("1"), Some("1")]).into(),
            Series::new("city".into(), vec![Some("Beijing"), Some("Shanghai")]).into(),
        ])
        .unwrap(),
    };

    let error = lookup_values(
        &base,
        &lookup,
        "user_id",
        "user_id",
        &[LookupSelect {
            lookup_column: "city".to_string(),
            output_column: "city".to_string(),
        }],
    )
    .err()
    .unwrap();

    // 2026-03-23: 这里锁定重复 key 报错，目的是要求上层先显式去重再查值。
    assert!(error.to_string().contains("user_id"));
}

#[test]
fn lookup_values_rejects_conflicting_output_column() {
    let base = LoadedTable {
        // 2026-03-23: 这里构造主表已有 city 列，目的是锁定输出列与原列冲突时必须显式避让。
        handle: TableHandle::new_confirmed(
            "memory://lookup-conflict-base",
            "Base",
            vec!["user_id".into(), "city".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("user_id".into(), vec![Some("1")]).into(),
            Series::new("city".into(), vec![Some("Urumqi")]).into(),
        ])
        .unwrap(),
    };
    let lookup = LoadedTable {
        // 2026-03-23: 这里构造查值表 city 列，目的是验证 output_column 与主表列名冲突的保护逻辑。
        handle: TableHandle::new_confirmed(
            "memory://lookup-conflict-dict",
            "Lookup",
            vec!["user_id".into(), "city".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("user_id".into(), vec![Some("1")]).into(),
            Series::new("city".into(), vec![Some("Beijing")]).into(),
        ])
        .unwrap(),
    };

    let error = lookup_values(
        &base,
        &lookup,
        "user_id",
        "user_id",
        &[LookupSelect {
            lookup_column: "city".to_string(),
            output_column: "city".to_string(),
        }],
    )
    .err()
    .unwrap();

    // 2026-03-23: 这里锁定输出列冲突报错，目的是避免 lookup_values 在静默覆盖主表字段时引入脏结果。
    assert!(error.to_string().contains("city"));
}

#[test]
fn lookup_values_by_composite_keys_appends_selected_columns() {
    let base = LoadedTable {
        // 2026-03-23: 这里构造“客户 + 月份”复合键主表，目的是先锁定轻量查值支持复合键带列。
        handle: TableHandle::new_confirmed(
            "memory://lookup-composite-base",
            "Base",
            vec!["customer_id".into(), "month".into(), "amount".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("customer_id".into(), vec![Some("1"), Some("1"), Some("2")]).into(),
            Series::new(
                "month".into(),
                vec![Some("2026-01"), Some("2026-02"), Some("2026-01")],
            )
            .into(),
            Series::new("amount".into(), vec![120_i64, 95_i64, 88_i64]).into(),
        ])
        .unwrap(),
    };
    let lookup = LoadedTable {
        // 2026-03-23: 这里构造复合键查值表，目的是验证 lookup_values 不会跨月份串带城市和分层。
        handle: TableHandle::new_confirmed(
            "memory://lookup-composite-dict",
            "Lookup",
            vec![
                "customer_id".into(),
                "month".into(),
                "city".into(),
                "tier".into(),
            ],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("customer_id".into(), vec![Some("1"), Some("1"), Some("2")]).into(),
            Series::new(
                "month".into(),
                vec![Some("2026-01"), Some("2026-02"), Some("2026-01")],
            )
            .into(),
            Series::new(
                "city".into(),
                vec![Some("Beijing"), Some("Shanghai"), Some("Shenzhen")],
            )
            .into(),
            Series::new("tier".into(), vec![Some("A"), Some("B"), Some("C")]).into(),
        ])
        .unwrap(),
    };

    let looked_up = lookup_values_by_keys(
        &base,
        &lookup,
        &["customer_id", "month"],
        &["customer_id", "month"],
        &[
            LookupSelect {
                lookup_column: "city".to_string(),
                output_column: "city".to_string(),
            },
            LookupSelect {
                lookup_column: "tier".to_string(),
                output_column: "tier".to_string(),
            },
        ],
    )
    .unwrap();
    let preview = preview_table(&looked_up.dataframe, looked_up.dataframe.height()).unwrap();

    // 2026-03-23: 这里锁定复合键查值结果按完整键命中，目的是让“客户ID + 月份”这种真实经营分析口径稳定可用。
    assert_eq!(preview.rows[0]["city"], "Beijing");
    assert_eq!(preview.rows[1]["city"], "Shanghai");
    assert_eq!(preview.rows[2]["city"], "Shenzhen");
    assert_eq!(preview.rows[1]["tier"], "B");
}

#[test]
fn lookup_values_rejects_mismatched_composite_key_lengths() {
    let base = LoadedTable {
        // 2026-03-23: 这里构造最小主表，目的是单独锁定复合键列数不一致时的明确报错。
        handle: TableHandle::new_confirmed(
            "memory://lookup-key-arity-base",
            "Base",
            vec!["customer_id".into(), "month".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("customer_id".into(), vec![Some("1")]).into(),
            Series::new("month".into(), vec![Some("2026-01")]).into(),
        ])
        .unwrap(),
    };
    let lookup = LoadedTable {
        // 2026-03-23: 这里构造最小 lookup 表，目的是让失败原因只落在复合键规格边界。
        handle: TableHandle::new_confirmed(
            "memory://lookup-key-arity-dict",
            "Lookup",
            vec!["customer_id".into(), "month".into(), "city".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("customer_id".into(), vec![Some("1")]).into(),
            Series::new("month".into(), vec![Some("2026-01")]).into(),
            Series::new("city".into(), vec![Some("Beijing")]).into(),
        ])
        .unwrap(),
    };

    let error = lookup_values_by_keys(
        &base,
        &lookup,
        &["customer_id", "month"],
        &["customer_id"],
        &[LookupSelect {
            lookup_column: "city".to_string(),
            output_column: "city_out".to_string(),
        }],
    )
    .err()
    .unwrap();

    // 2026-03-23: 这里锁定复合键列数不一致会直白报错，目的是避免用户误把单键和双键配置混用。
    assert!(error.to_string().contains("键列数量"));
}

#[test]
fn window_calculation_builds_row_number_and_dense_rank_by_partition() {
    let loaded = LoadedTable {
        // 2026-03-23: 这里构造分组内排序场景，目的是锁定 row_number 和 dense rank 的第一版窗口行为。
        handle: TableHandle::new_confirmed(
            "memory://window-rank",
            "Sheet1",
            vec!["region".into(), "amount".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new(
                "region".into(),
                vec![Some("East"), Some("East"), Some("East"), Some("West")],
            )
            .into(),
            Series::new("amount".into(), vec![100_i64, 80_i64, 80_i64, 90_i64]).into(),
        ])
        .unwrap(),
    };

    let calculated = window_calculation(
        &loaded,
        &["region"],
        &[WindowOrderSpec {
            column: "amount".to_string(),
            descending: true,
        }],
        &[
            WindowCalculation {
                kind: WindowCalculationKind::RowNumber,
                source_column: None,
                output_column: "row_number".to_string(),
                offset: None,
                window_size: None,
            },
            WindowCalculation {
                kind: WindowCalculationKind::Rank,
                source_column: None,
                output_column: "dense_rank".to_string(),
                offset: None,
                window_size: None,
            },
        ],
    )
    .unwrap();
    let preview = preview_table(&calculated.dataframe, calculated.dataframe.height()).unwrap();

    // 2026-03-23: 这里锁定窗口列会按原表行回填，目的是让用户无需重新理解结果行顺序。
    assert_eq!(preview.rows[0]["row_number"], "1");
    assert_eq!(preview.rows[0]["dense_rank"], "1");
    assert_eq!(preview.rows[1]["row_number"], "2");
    assert_eq!(preview.rows[1]["dense_rank"], "2");
    assert_eq!(preview.rows[2]["row_number"], "3");
    assert_eq!(preview.rows[2]["dense_rank"], "2");
    assert_eq!(preview.rows[3]["row_number"], "1");
    assert_eq!(preview.rows[3]["dense_rank"], "1");
}

#[test]
fn window_calculation_builds_partitioned_cumulative_sum_in_sort_order() {
    let loaded = LoadedTable {
        // 2026-03-23: 这里故意打乱原表顺序，目的是锁定累计和按指定排序计算但仍回填到原行。
        handle: TableHandle::new_confirmed(
            "memory://window-cumsum",
            "Sheet1",
            vec!["region".into(), "biz_date".into(), "amount".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new(
                "region".into(),
                vec![Some("East"), Some("East"), Some("East"), Some("West")],
            )
            .into(),
            Series::new(
                "biz_date".into(),
                vec![
                    Some("2026-03-02"),
                    Some("2026-03-01"),
                    Some("2026-03-03"),
                    Some("2026-03-01"),
                ],
            )
            .into(),
            Series::new("amount".into(), vec![80_i64, 100_i64, 60_i64, 90_i64]).into(),
        ])
        .unwrap(),
    };

    let calculated = window_calculation(
        &loaded,
        &["region"],
        &[WindowOrderSpec {
            column: "biz_date".to_string(),
            descending: false,
        }],
        &[WindowCalculation {
            kind: WindowCalculationKind::CumulativeSum,
            source_column: Some("amount".to_string()),
            output_column: "running_amount".to_string(),
            offset: None,
            window_size: None,
        }],
    )
    .unwrap();
    let preview = preview_table(&calculated.dataframe, calculated.dataframe.height()).unwrap();

    // 2026-03-23: 这里锁定累计和按排序序列推进，目的是为后续趋势分析和经营累计指标提供稳定底座。
    assert_eq!(preview.rows[0]["running_amount"], "180");
    assert_eq!(preview.rows[1]["running_amount"], "100");
    assert_eq!(preview.rows[2]["running_amount"], "240");
    assert_eq!(preview.rows[3]["running_amount"], "90");
}

#[test]
fn window_calculation_rejects_non_numeric_cumulative_sum_source() {
    let loaded = LoadedTable {
        // 2026-03-23: 这里构造文本型 source 列，目的是锁定 cumulative_sum 的数值类型门禁。
        handle: TableHandle::new_confirmed(
            "memory://window-invalid-cumsum",
            "Sheet1",
            vec!["region".into(), "label".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("region".into(), vec![Some("East")]).into(),
            Series::new("label".into(), vec![Some("vip")]).into(),
        ])
        .unwrap(),
    };

    let error = window_calculation(
        &loaded,
        &["region"],
        &[WindowOrderSpec {
            column: "region".to_string(),
            descending: false,
        }],
        &[WindowCalculation {
            kind: WindowCalculationKind::CumulativeSum,
            source_column: Some("label".to_string()),
            output_column: "running_label".to_string(),
            offset: None,
            window_size: None,
        }],
    )
    .err()
    .unwrap();

    // 2026-03-23: 这里锁定非数值累计报错，目的是避免把文本列误算成经营指标。
    assert!(error.to_string().contains("label"));
}

#[test]
fn window_calculation_supports_shift_percent_rank_and_rolling_metrics() {
    let loaded = LoadedTable {
        // 2026-03-23: 这里构造乱序日期分区数据，目的是先锁定 lag/lead、percent_rank、rolling_sum、rolling_mean 的组合行为。
        handle: TableHandle::new_confirmed(
            "memory://window-advanced",
            "Sheet1",
            vec![
                "region".into(),
                "biz_date".into(),
                "amount".into(),
                "customer".into(),
            ],
        ),
        dataframe: DataFrame::new(vec![
            Series::new(
                "region".into(),
                vec![Some("East"), Some("East"), Some("East"), Some("West")],
            )
            .into(),
            Series::new(
                "biz_date".into(),
                vec![
                    Some("2026-01-03"),
                    Some("2026-01-01"),
                    Some("2026-01-02"),
                    Some("2026-01-01"),
                ],
            )
            .into(),
            Series::new("amount".into(), vec![80_i64, 100, 100, 60]).into(),
            Series::new(
                "customer".into(),
                vec![Some("C"), Some("A"), Some("B"), Some("W")],
            )
            .into(),
        ])
        .unwrap(),
    };

    let calculated = window_calculation(
        &loaded,
        &["region"],
        &[WindowOrderSpec {
            column: "biz_date".to_string(),
            descending: false,
        }],
        &[
            WindowCalculation {
                kind: WindowCalculationKind::Lag,
                source_column: Some("customer".to_string()),
                output_column: "prev_customer".to_string(),
                offset: Some(1),
                window_size: None,
            },
            WindowCalculation {
                kind: WindowCalculationKind::Lead,
                source_column: Some("customer".to_string()),
                output_column: "next_customer".to_string(),
                offset: Some(1),
                window_size: None,
            },
            WindowCalculation {
                kind: WindowCalculationKind::PercentRank,
                source_column: None,
                output_column: "percent_rank".to_string(),
                offset: None,
                window_size: None,
            },
            WindowCalculation {
                kind: WindowCalculationKind::RollingSum,
                source_column: Some("amount".to_string()),
                output_column: "rolling_sum_2".to_string(),
                offset: None,
                window_size: Some(2),
            },
            WindowCalculation {
                kind: WindowCalculationKind::RollingMean,
                source_column: Some("amount".to_string()),
                output_column: "rolling_mean_2".to_string(),
                offset: None,
                window_size: Some(2),
            },
        ],
    )
    .unwrap();
    let preview = preview_table(&calculated.dataframe, calculated.dataframe.height()).unwrap();

    // 2026-03-23: 这里锁定窗口结果会按原始行回填，目的是让问答式调用不需要重新解释行位置。
    assert_eq!(preview.rows[0]["prev_customer"], "B");
    assert_eq!(preview.rows[0]["next_customer"], "");
    assert_eq!(preview.rows[0]["percent_rank"], "1");
    assert_eq!(preview.rows[0]["rolling_sum_2"], "180");
    assert_eq!(preview.rows[0]["rolling_mean_2"], "90");

    assert_eq!(preview.rows[1]["prev_customer"], "");
    assert_eq!(preview.rows[1]["next_customer"], "B");
    assert_eq!(preview.rows[1]["percent_rank"], "0");
    assert_eq!(preview.rows[1]["rolling_sum_2"], "100");
    assert_eq!(preview.rows[1]["rolling_mean_2"], "100");

    assert_eq!(preview.rows[2]["prev_customer"], "A");
    assert_eq!(preview.rows[2]["next_customer"], "C");
    assert_eq!(preview.rows[2]["percent_rank"], "0.5");
    assert_eq!(preview.rows[2]["rolling_sum_2"], "200");
    assert_eq!(preview.rows[2]["rolling_mean_2"], "100");

    assert_eq!(preview.rows[3]["prev_customer"], "");
    assert_eq!(preview.rows[3]["next_customer"], "");
    assert_eq!(preview.rows[3]["percent_rank"], "0");
    assert_eq!(preview.rows[3]["rolling_sum_2"], "60");
    assert_eq!(preview.rows[3]["rolling_mean_2"], "60");
}

#[test]
fn filter_rows_returns_only_matching_records() {
    let inference = infer_header_schema("tests/fixtures/basic-sales.xlsx", "Sales").unwrap();
    let loaded =
        load_confirmed_table("tests/fixtures/basic-sales.xlsx", "Sales", &inference).unwrap();

    let filtered = filter_rows(
        &loaded,
        &[FilterCondition {
            column: "region".to_string(),
            operator: FilterOperator::Equals,
            value: "East".to_string(),
        }],
    )
    .unwrap();

    assert_eq!(filtered.dataframe.height(), 1);
    assert_eq!(filtered.handle.columns(), &["user_id", "region", "sales"]);
    // 2026-03-21: 这里先取出底层 Series，目的是兼容当前 Polars 新版 Column API 的读取方式。
    let region_series = filtered
        .dataframe
        .column("region")
        .unwrap()
        .as_materialized_series();
    assert_eq!(region_series.str_value(0).unwrap(), "East");
}

#[test]
fn cast_column_types_converts_string_column_to_int64() {
    let inference = infer_header_schema("tests/fixtures/basic-sales.xlsx", "Sales").unwrap();
    let loaded =
        load_confirmed_table("tests/fixtures/basic-sales.xlsx", "Sales", &inference).unwrap();

    let casted = cast_column_types(
        &loaded,
        &[CastColumnSpec {
            column: "sales".to_string(),
            target_type: CastTargetType::Int64,
        }],
    )
    .unwrap();

    // 2026-03-21: 这里校验 sales 列已经完成数值化转换，目的是确保后续聚合和排序都基于真实数值。
    let sales_series = casted
        .dataframe
        .column("sales")
        .unwrap()
        .as_materialized_series();
    assert_eq!(sales_series.dtype(), &DataType::Int64);
    // 2026-03-21: 这里继续校验转换后的展示值，目的是确认 cast 后的结果仍能稳定输出给上层预览。
    assert_eq!(sales_series.str_value(0).unwrap(), "120");
}

#[test]
fn group_and_aggregate_sums_and_counts_by_region() {
    let inference = infer_header_schema("tests/fixtures/group-sales.xlsx", "Sales").unwrap();
    let loaded =
        load_confirmed_table("tests/fixtures/group-sales.xlsx", "Sales", &inference).unwrap();
    let casted = cast_column_types(
        &loaded,
        &[CastColumnSpec {
            column: "sales".to_string(),
            target_type: CastTargetType::Int64,
        }],
    )
    .unwrap();

    let grouped = group_and_aggregate(
        &casted,
        &["region"],
        &[
            AggregationSpec {
                column: "sales".to_string(),
                operator: AggregationOperator::Sum,
            },
            AggregationSpec {
                column: "sales".to_string(),
                operator: AggregationOperator::Count,
            },
        ],
    )
    .unwrap();
    let preview = preview_table(&grouped.dataframe, grouped.dataframe.height()).unwrap();

    assert_eq!(
        grouped.handle.columns(),
        &["region", "sales_sum", "sales_count"]
    );
    assert_eq!(grouped.dataframe.height(), 2);
    // 2026-03-21: 这里校验分组聚合后的首行结果，目的是确保分组列排序和聚合值输出都稳定可预测。
    assert_eq!(preview.rows[0]["region"], "East");
    assert_eq!(preview.rows[0]["sales_sum"], "200");
    assert_eq!(preview.rows[0]["sales_count"], "2");
    // 2026-03-21: 这里校验第二个分组结果，目的是确保多组场景下不会出现聚合串位。
    assert_eq!(preview.rows[1]["region"], "West");
    assert_eq!(preview.rows[1]["sales_sum"], "150");
    assert_eq!(preview.rows[1]["sales_count"], "2");
}

#[test]
fn derive_columns_supports_condition_groups_date_bucket_and_template() {
    let loaded = LoadedTable {
        // 2026-03-23: 这里构造经营分析常见中间表，目的是锁定 derive_columns 增强后可直接产出标签、时段和说明列。
        handle: TableHandle::new_confirmed(
            "memory://derive-advanced",
            "Sheet1",
            vec![
                "customer_id".into(),
                "sales".into(),
                "visits".into(),
                "biz_date".into(),
                "region".into(),
            ],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("customer_id".into(), vec![Some("C001"), Some("C002"), Some("C003")]).into(),
            Series::new("sales".into(), vec![120_i64, 95_i64, 60_i64]).into(),
            Series::new("visits".into(), vec![3_i64, 5_i64, 1_i64]).into(),
            Series::new(
                "biz_date".into(),
                vec![Some("2026-01-15"), Some("2026-04-10"), Some("2026-08-01")],
            )
            .into(),
            Series::new("region".into(), vec![Some("East"), Some("West"), Some("North")]).into(),
        ])
        .unwrap(),
    };

    let derived = derive_columns(
        &loaded,
        &[
            DerivationSpec::CaseWhen {
                output_column: "priority".to_string(),
                rules: vec![
                    CaseWhenRule {
                        when: DerivePredicate::Group(DeriveConditionGroup {
                            mode: LogicalMode::All,
                            conditions: vec![
                                DeriveCondition {
                                    column: "sales".to_string(),
                                    operator: DeriveOperator::Gte,
                                    value: "100".to_string(),
                                },
                                DeriveCondition {
                                    column: "visits".to_string(),
                                    operator: DeriveOperator::Gte,
                                    value: "3".to_string(),
                                },
                            ],
                        }),
                        value: "A".to_string(),
                    },
                    CaseWhenRule {
                        when: DerivePredicate::Group(DeriveConditionGroup {
                            mode: LogicalMode::Any,
                            conditions: vec![
                                DeriveCondition {
                                    column: "sales".to_string(),
                                    operator: DeriveOperator::Gte,
                                    value: "90".to_string(),
                                },
                                DeriveCondition {
                                    column: "visits".to_string(),
                                    operator: DeriveOperator::Gte,
                                    value: "5".to_string(),
                                },
                            ],
                        }),
                        value: "B".to_string(),
                    },
                ],
                else_value: "C".to_string(),
            },
            DerivationSpec::DateBucketize {
                source_column: "biz_date".to_string(),
                output_column: "season".to_string(),
                buckets: vec![
                    DateBucketRule {
                        label: "Q1".to_string(),
                        start_inclusive: Some("2026-01-01".to_string()),
                        end_exclusive: Some("2026-04-01".to_string()),
                    },
                    DateBucketRule {
                        label: "Q2".to_string(),
                        start_inclusive: Some("2026-04-01".to_string()),
                        end_exclusive: Some("2026-07-01".to_string()),
                    },
                ],
                else_value: "H2".to_string(),
            },
            DerivationSpec::Template {
                output_column: "reason".to_string(),
                template: "{customer_id}-{region}-{priority}-{season}".to_string(),
            },
        ],
    )
    .unwrap();
    let preview = preview_table(&derived.dataframe, derived.dataframe.height()).unwrap();

    // 2026-03-23: 这里锁定 all/any、日期分段与模板列会一起生效，目的是确保经营分析桥接层能直接拼出可解释说明。
    assert_eq!(preview.rows[0]["priority"], "A");
    assert_eq!(preview.rows[1]["priority"], "B");
    assert_eq!(preview.rows[2]["priority"], "C");
    assert_eq!(preview.rows[0]["season"], "Q1");
    assert_eq!(preview.rows[1]["season"], "Q2");
    assert_eq!(preview.rows[2]["season"], "H2");
    assert_eq!(preview.rows[0]["reason"], "C001-East-A-Q1");
    assert_eq!(preview.rows[1]["reason"], "C002-West-B-Q2");
}

#[test]
fn sort_rows_orders_by_region_then_sales_descending() {
    let inference = infer_header_schema("tests/fixtures/group-sales.xlsx", "Sales").unwrap();
    let loaded =
        load_confirmed_table("tests/fixtures/group-sales.xlsx", "Sales", &inference).unwrap();
    let casted = cast_column_types(
        &loaded,
        &[CastColumnSpec {
            column: "sales".to_string(),
            target_type: CastTargetType::Int64,
        }],
    )
    .unwrap();

    let sorted = sort_rows(
        &casted,
        &[
            SortSpec {
                column: "region".to_string(),
                descending: false,
            },
            SortSpec {
                column: "sales".to_string(),
                descending: true,
            },
        ],
    )
    .unwrap();
    let preview = preview_table(&sorted.dataframe, sorted.dataframe.height()).unwrap();

    assert_eq!(sorted.handle.columns(), &["user_id", "region", "sales"]);
    assert_eq!(sorted.dataframe.height(), 4);
    // 2026-03-21: 这里校验多列排序后的首行，目的是确保主排序键和次排序键都按预期生效。
    assert_eq!(preview.rows[0]["region"], "East");
    assert_eq!(preview.rows[0]["sales"], "120");
    // 2026-03-21: 这里校验第二行，目的是确认同组内按 sales 降序稳定排列。
    assert_eq!(preview.rows[1]["region"], "East");
    assert_eq!(preview.rows[1]["sales"], "80");
    // 2026-03-21: 这里校验跨组切换后的顺序，目的是确认第二排序组不会串位。
    assert_eq!(preview.rows[2]["region"], "West");
    assert_eq!(preview.rows[2]["sales"], "90");
}

#[test]
fn top_n_rows_keeps_highest_sales_records_after_sorting() {
    let inference = infer_header_schema("tests/fixtures/group-sales.xlsx", "Sales").unwrap();
    let loaded =
        load_confirmed_table("tests/fixtures/group-sales.xlsx", "Sales", &inference).unwrap();
    let casted = cast_column_types(
        &loaded,
        &[CastColumnSpec {
            column: "sales".to_string(),
            target_type: CastTargetType::Int64,
        }],
    )
    .unwrap();

    let top_rows = top_n_rows(
        &casted,
        &[SortSpec {
            column: "sales".to_string(),
            descending: true,
        }],
        2,
    )
    .unwrap();
    let preview = preview_table(&top_rows.dataframe, top_rows.dataframe.height()).unwrap();

    assert_eq!(top_rows.handle.columns(), &["user_id", "region", "sales"]);
    assert_eq!(top_rows.dataframe.height(), 2);
    // 2026-03-21: 这里校验 top_n 的首行，目的是确保先排序后截取时会保留最大 sales 记录。
    assert_eq!(preview.rows[0]["sales"], "120");
    // 2026-03-21: 这里校验第二行，目的是确保只截取排序后的前两条，而不是原始输入前两条。
    assert_eq!(preview.rows[1]["sales"], "90");
}

#[test]
fn join_tables_keeps_only_matched_rows_for_explicit_key() {
    let left_inference =
        infer_header_schema("tests/fixtures/join-customers.xlsx", "Customers").unwrap();
    let right_inference = infer_header_schema("tests/fixtures/join-orders.xlsx", "Orders").unwrap();
    let left = load_confirmed_table(
        "tests/fixtures/join-customers.xlsx",
        "Customers",
        &left_inference,
    )
    .unwrap();
    let right = load_confirmed_table(
        "tests/fixtures/join-orders.xlsx",
        "Orders",
        &right_inference,
    )
    .unwrap();

    let joined = join_tables(
        &left,
        &right,
        "user_id",
        "user_id",
        JoinKeepMode::MatchedOnly,
    )
    .unwrap();
    let preview = preview_table(&joined.dataframe, joined.dataframe.height()).unwrap();

    assert_eq!(joined.dataframe.height(), 3);
    assert_eq!(
        joined.handle.columns(),
        &["user_id", "name", "region", "order_id", "amount"]
    );
    // 2026-03-21: 这里校验匹配保留模式的首行，目的是确保同一 user_id 能展开成多条订单记录。
    assert_eq!(preview.rows[0]["user_id"], "1");
    assert_eq!(preview.rows[0]["name"], "Alice");
    assert_eq!(preview.rows[0]["order_id"], "101");
    // 2026-03-21: 这里校验最后一行，目的是确保只保留两边都能关联成功的记录。
    assert_eq!(preview.rows[2]["user_id"], "2");
    assert_eq!(preview.rows[2]["amount"], "90");
}

#[test]
fn join_tables_keep_left_preserves_unmatched_left_rows() {
    let left_inference =
        infer_header_schema("tests/fixtures/join-customers.xlsx", "Customers").unwrap();
    let right_inference = infer_header_schema("tests/fixtures/join-orders.xlsx", "Orders").unwrap();
    let left = load_confirmed_table(
        "tests/fixtures/join-customers.xlsx",
        "Customers",
        &left_inference,
    )
    .unwrap();
    let right = load_confirmed_table(
        "tests/fixtures/join-orders.xlsx",
        "Orders",
        &right_inference,
    )
    .unwrap();

    let joined = join_tables(&left, &right, "user_id", "user_id", JoinKeepMode::KeepLeft).unwrap();

    assert_eq!(joined.dataframe.height(), 4);
    // 2026-03-21: 这里校验左保留模式下的空匹配行，目的是确保左表用户 3 不会因为右表缺单而丢失。
    assert_eq!(joined.dataframe.column("order_id").unwrap().null_count(), 1);
}

#[test]
fn join_tables_keep_right_preserves_unmatched_right_rows() {
    let left_inference =
        infer_header_schema("tests/fixtures/join-customers.xlsx", "Customers").unwrap();
    let right_inference = infer_header_schema("tests/fixtures/join-orders.xlsx", "Orders").unwrap();
    let left = load_confirmed_table(
        "tests/fixtures/join-customers.xlsx",
        "Customers",
        &left_inference,
    )
    .unwrap();
    let right = load_confirmed_table(
        "tests/fixtures/join-orders.xlsx",
        "Orders",
        &right_inference,
    )
    .unwrap();

    let joined = join_tables(&left, &right, "user_id", "user_id", JoinKeepMode::KeepRight).unwrap();
    let preview = preview_table(&joined.dataframe, joined.dataframe.height()).unwrap();

    assert_eq!(joined.dataframe.height(), 4);
    // 2026-03-21: 这里校验 keep_right 会保留右表独有记录，目的是确认未匹配订单不会被误丢弃。
    assert_eq!(preview.rows[3]["order_id"], "104");
    assert_eq!(preview.rows[3]["amount"], "200");
    assert_eq!(joined.dataframe.column("name").unwrap().null_count(), 1);
}

#[test]
fn join_tables_ignores_blank_keys_in_matched_only_mode() {
    let left_inference =
        infer_header_schema("tests/fixtures/join-empty-keys.xlsx", "Customers").unwrap();
    let right_inference =
        infer_header_schema("tests/fixtures/join-empty-keys.xlsx", "Orders").unwrap();
    let left = load_confirmed_table(
        "tests/fixtures/join-empty-keys.xlsx",
        "Customers",
        &left_inference,
    )
    .unwrap();
    let right = load_confirmed_table(
        "tests/fixtures/join-empty-keys.xlsx",
        "Orders",
        &right_inference,
    )
    .unwrap();

    let joined = join_tables(
        &left,
        &right,
        "user_id",
        "user_id",
        JoinKeepMode::MatchedOnly,
    )
    .unwrap();
    let preview = preview_table(&joined.dataframe, joined.dataframe.height()).unwrap();

    // 2026-03-21: 这里锁定空键不参与显性关联，目的是避免未填 ID 的脏数据在 matched_only 下误拼到一起。
    assert_eq!(joined.dataframe.height(), 1);
    assert_eq!(preview.rows[0]["user_id"], "1");
    assert_eq!(preview.rows[0]["order_id"], "101");
}

#[test]
fn join_tables_expands_many_to_many_matches_and_renames_repeated_conflicts() {
    let left_inference =
        infer_header_schema("tests/fixtures/join-conflict-columns.xlsx", "Left").unwrap();
    let right_inference =
        infer_header_schema("tests/fixtures/join-conflict-columns.xlsx", "Right").unwrap();
    let left = load_confirmed_table(
        "tests/fixtures/join-conflict-columns.xlsx",
        "Left",
        &left_inference,
    )
    .unwrap();
    let right = load_confirmed_table(
        "tests/fixtures/join-conflict-columns.xlsx",
        "Right",
        &right_inference,
    )
    .unwrap();

    let joined = join_tables(
        &left,
        &right,
        "user_id",
        "user_id",
        JoinKeepMode::MatchedOnly,
    )
    .unwrap();
    let preview = preview_table(&joined.dataframe, joined.dataframe.height()).unwrap();

    // 2026-03-21: 这里锁定多对多展开行数，目的是确认同一 key 的左右多条记录会做笛卡尔展开而不是被覆盖。
    assert_eq!(joined.dataframe.height(), 5);
    // 2026-03-21: 这里锁定连续冲突列重命名，目的是确保右表同名列不会覆盖左表已有列，也不会出现重复列名。
    assert_eq!(
        joined.handle.columns(),
        &[
            "user_id",
            "region",
            "region_right",
            "tag",
            "region_right_right",
            "region_right_right_right",
            "amount"
        ]
    );
    assert_eq!(preview.rows[0]["user_id"], "1");
    assert_eq!(preview.rows[0]["amount"], "120");
}

#[test]
fn join_tables_aligns_integer_and_float_keys_without_manual_casts() {
    let left = LoadedTable {
        // 2026-03-23: 这里构造整数主键左表，目的是先锁定 join_tables 在不手工 casts 时也能吃数值型键。
        handle: TableHandle::new_confirmed(
            "memory://join-left-int",
            "Left",
            vec!["user_id".into(), "name".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("user_id".into(), [1_i64, 2_i64, 3_i64]).into(),
            Series::new("name".into(), ["Alice", "Bob", "Cara"]).into(),
        ])
        .unwrap(),
    };
    let right = LoadedTable {
        // 2026-03-23: 这里构造浮点主键右表，目的是复现“1”和“1.0”数值等价但字符串展示不同导致的关联不稳问题。
        handle: TableHandle::new_confirmed(
            "memory://join-right-float",
            "Right",
            vec!["user_id".into(), "order_id".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("user_id".into(), [1.0_f64, 2.0_f64, 4.0_f64]).into(),
            Series::new("order_id".into(), ["A-101", "A-102", "A-104"]).into(),
        ])
        .unwrap(),
    };

    let joined = join_tables(&left, &right, "user_id", "user_id", JoinKeepMode::MatchedOnly)
        .unwrap();
    let preview = preview_table(&joined.dataframe, joined.dataframe.height()).unwrap();

    // 2026-03-23: 这里锁定整数键与浮点键会按同一数值语义匹配，目的是减少显性关联前还要额外 casts 的负担。
    assert_eq!(joined.dataframe.height(), 2);
    assert_eq!(preview.rows[0]["user_id"], "1");
    assert_eq!(preview.rows[0]["name"], "Alice");
    assert_eq!(preview.rows[0]["order_id"], "A-101");
    assert_eq!(preview.rows[1]["user_id"], "2");
    assert_eq!(preview.rows[1]["order_id"], "A-102");
}

#[test]
fn append_tables_stacks_rows_when_columns_match_exactly() {
    let top_inference = infer_header_schema("tests/fixtures/append-sales-a.xlsx", "Sales").unwrap();
    let bottom_inference =
        infer_header_schema("tests/fixtures/append-sales-b.xlsx", "Sales").unwrap();
    let top = load_confirmed_table(
        "tests/fixtures/append-sales-a.xlsx",
        "Sales",
        &top_inference,
    )
    .unwrap();
    let bottom = load_confirmed_table(
        "tests/fixtures/append-sales-b.xlsx",
        "Sales",
        &bottom_inference,
    )
    .unwrap();

    let appended = append_tables(&top, &bottom).unwrap();
    let preview = preview_table(&appended.dataframe, appended.dataframe.height()).unwrap();

    assert_eq!(appended.handle.columns(), &["user_id", "region", "sales"]);
    assert_eq!(appended.dataframe.height(), 4);
    // 2026-03-21: 这里校验纵向追加后的前两行，目的是确保原始上半部分数据顺序不会被打乱。
    assert_eq!(preview.rows[0]["user_id"], "1");
    assert_eq!(preview.rows[1]["user_id"], "2");
    // 2026-03-21: 这里校验追加后的后两行，目的是确保下半部分数据真正拼接到了结果尾部。
    assert_eq!(preview.rows[2]["user_id"], "3");
    assert_eq!(preview.rows[3]["sales"], "60");
}

#[test]
fn append_tables_aligns_bottom_columns_by_name_before_stacking() {
    let top_inference = infer_header_schema("tests/fixtures/append-sales-a.xlsx", "Sales").unwrap();
    let bottom_inference =
        infer_header_schema("tests/fixtures/append-sales-reordered.xlsx", "Sales").unwrap();
    let top = load_confirmed_table(
        "tests/fixtures/append-sales-a.xlsx",
        "Sales",
        &top_inference,
    )
    .unwrap();
    let bottom = load_confirmed_table(
        "tests/fixtures/append-sales-reordered.xlsx",
        "Sales",
        &bottom_inference,
    )
    .unwrap();

    let appended = append_tables(&top, &bottom).unwrap();
    let preview = preview_table(&appended.dataframe, appended.dataframe.height()).unwrap();

    assert_eq!(appended.handle.columns(), &["user_id", "region", "sales"]);
    assert_eq!(appended.dataframe.height(), 4);
    // 2026-03-21: 这里校验结果列顺序仍以上表为准，目的是避免下表列顺序不同导致结果 schema 漂移。
    assert_eq!(preview.columns, vec!["user_id", "region", "sales"]);
    // 2026-03-21: 这里校验重排列顺序后的第三行，目的是确保 user_id/region/sales 是按列名而不是按位置拼接。
    assert_eq!(preview.rows[2]["user_id"], "3");
    assert_eq!(preview.rows[2]["region"], "North");
    assert_eq!(preview.rows[2]["sales"], "90");
    // 2026-03-21: 这里校验第四行，目的是确保整张下表都按列名对齐后追加成功。
    assert_eq!(preview.rows[3]["user_id"], "4");
    assert_eq!(preview.rows[3]["region"], "East");
    assert_eq!(preview.rows[3]["sales"], "60");
}

#[test]
fn append_tables_rejects_tables_with_mismatched_columns() {
    let top_inference = infer_header_schema("tests/fixtures/append-sales-a.xlsx", "Sales").unwrap();
    let bottom_inference =
        infer_header_schema("tests/fixtures/append-sales-mismatch.xlsx", "Sales").unwrap();
    let top = load_confirmed_table(
        "tests/fixtures/append-sales-a.xlsx",
        "Sales",
        &top_inference,
    )
    .unwrap();
    let bottom = load_confirmed_table(
        "tests/fixtures/append-sales-mismatch.xlsx",
        "Sales",
        &bottom_inference,
    )
    .unwrap();

    let error = match append_tables(&top, &bottom) {
        Ok(_) => panic!("append_tables 在列结构不一致时不应该成功"),
        Err(error) => error,
    };

    // 2026-03-21: 这里校验按列名对齐模式仍会拒绝缺列/异构表，目的是避免把不同结构数据误拼接进同一张表。
    assert_eq!(
        error.to_string(),
        "append_tables 要求两张表包含完全相同的列名"
    );
}

#[test]
fn summarize_table_reports_numeric_and_text_metrics() {
    let inference = infer_header_schema("tests/fixtures/basic-sales.xlsx", "Sales").unwrap();
    let loaded =
        load_confirmed_table("tests/fixtures/basic-sales.xlsx", "Sales", &inference).unwrap();
    let casted = cast_column_types(
        &loaded,
        &[CastColumnSpec {
            column: "sales".to_string(),
            target_type: CastTargetType::Int64,
        }],
    )
    .unwrap();

    let summaries = summarize_table(&casted, &["region", "sales"], 2).unwrap();
    let region_summary = summaries
        .iter()
        .find(|summary| summary.column == "region")
        .unwrap();
    let sales_summary = summaries
        .iter()
        .find(|summary| summary.column == "sales")
        .unwrap();

    // 2026-03-21: 这里校验文本列摘要结构，目的是确保离散字段能稳定输出类别分布和质量指标。
    assert_eq!(region_summary.summary_kind, "string");
    assert_eq!(region_summary.distinct_count, Some(2));
    // 2026-03-21: 这里补缺失率断言，目的是让统计摘要直接给上层问答提供更直观的数据质量指标。
    assert_eq!(region_summary.missing_rate, Some(0.0));
    assert_eq!(region_summary.top_values[0].value, "East");
    assert_eq!(region_summary.top_values[0].count, 1);
    // 2026-03-21: 这里校验数值列摘要结构，目的是确保建模前最常用统计量保持稳定输出。
    assert_eq!(sales_summary.summary_kind, "numeric");
    assert_eq!(sales_summary.count, 2);
    // 2026-03-21: 这里补数值列缺失率断言，目的是确认数值摘要和文本摘要沿用同一套缺失率语义。
    assert_eq!(sales_summary.missing_rate, Some(0.0));
    assert_eq!(sales_summary.min_number, Some(95.0));
    assert_eq!(sales_summary.max_number, Some(120.0));
    assert_eq!(sales_summary.mean, Some(107.5));
    assert_eq!(sales_summary.sum, Some(215.0));
}

#[test]
fn summarize_table_handles_all_null_column_without_panicking() {
    let dataframe = DataFrame::new(vec![
        Series::new("notes".into(), vec![Option::<&str>::None, None]).into(),
        Series::new("active".into(), vec![Some(true), Some(false)]).into(),
    ])
    .unwrap();
    let loaded = LoadedTable {
        // 2026-03-21: 这里构造全空列场景，目的是模拟 Excel 真实业务里整列未填写的输入。
        handle: TableHandle::new_confirmed(
            "memory://summary-null",
            "Sheet1",
            vec!["notes".into(), "active".into()],
        ),
        dataframe,
    };

    let summaries = summarize_table(&loaded, &["notes", "active"], 2).unwrap();
    let notes_summary = summaries
        .iter()
        .find(|summary| summary.column == "notes")
        .unwrap();
    let active_summary = summaries
        .iter()
        .find(|summary| summary.column == "active")
        .unwrap();

    // 2026-03-21: 这里校验全空列不会崩溃，目的是保证 count/null_count 与空分布都可稳定返回。
    assert_eq!(notes_summary.count, 0);
    assert_eq!(notes_summary.null_count, 2);
    // 2026-03-21: 这里补全空列缺失率断言，目的是确保“全缺失”场景能返回 100% 缺失率而不是崩溃。
    assert_eq!(notes_summary.missing_rate, Some(1.0));
    assert_eq!(notes_summary.top_values.len(), 0);
    // 2026-03-21: 这里顺带校验布尔列画像，目的是确认 summary Tool 能同时覆盖多种字段类型。
    assert_eq!(active_summary.summary_kind, "boolean");
    // 2026-03-21: 这里补布尔列缺失率断言，目的是确认布尔画像也能向上层稳定暴露质量指标。
    assert_eq!(active_summary.missing_rate, Some(0.0));
    assert_eq!(active_summary.true_count, Some(1));
    assert_eq!(active_summary.false_count, Some(1));
}

#[test]
fn summarize_table_treats_blank_and_whitespace_strings_as_missing() {
    let dataframe = DataFrame::new(vec![
        Series::new(
            "notes".into(),
            vec![Some(""), Some("   "), Some("done"), Option::<&str>::None],
        )
        .into(),
    ])
    .unwrap();
    let loaded = LoadedTable {
        // 2026-03-21: 这里构造空白字符串与纯空格，目的是模拟 Excel 里常见的“看似有值、实际没填”。
        handle: TableHandle::new_confirmed(
            "memory://summary-blanks",
            "Sheet1",
            vec!["notes".into()],
        ),
        dataframe,
    };

    let summaries = summarize_table(&loaded, &["notes"], 3).unwrap();
    let notes_summary = summaries
        .iter()
        .find(|summary| summary.column == "notes")
        .unwrap();

    // 2026-03-21: 这里校验空白会并入缺失统计，目的是让 Excel 用户看到的有效值数量更符合直觉。
    assert_eq!(notes_summary.count, 1);
    assert_eq!(notes_summary.null_count, 3);
    // 2026-03-21: 这里补空白占缺失率断言，目的是防止后续改动只更新计数而遗漏比例。
    assert_eq!(notes_summary.missing_rate, Some(0.75));
    assert_eq!(notes_summary.distinct_count, Some(1));
    assert_eq!(notes_summary.top_values.len(), 1);
    assert_eq!(notes_summary.top_values[0].value, "done");
    assert_eq!(notes_summary.top_values[0].count, 1);
}

#[test]
fn summarize_table_treats_placeholder_text_as_missing() {
    let dataframe = DataFrame::new(vec![
        Series::new(
            "notes".into(),
            vec![
                Some("N/A"),
                Some("NA"),
                Some("null"),
                Some("NULL"),
                Some("done"),
            ],
        )
        .into(),
    ])
    .unwrap();
    let loaded = LoadedTable {
        // 2026-03-21: 这里构造业务占位缺失值，目的是覆盖 Excel 中 N/A、NA、null 一类常见写法。
        handle: TableHandle::new_confirmed(
            "memory://summary-placeholders",
            "Sheet1",
            vec!["notes".into()],
        ),
        dataframe,
    };

    let summaries = summarize_table(&loaded, &["notes"], 5).unwrap();
    let notes_summary = summaries
        .iter()
        .find(|summary| summary.column == "notes")
        .unwrap();

    // 2026-03-21: 这里校验占位值不会被误记为有效文本，目的是避免摘要高估可用数据量。
    assert_eq!(notes_summary.count, 1);
    assert_eq!(notes_summary.null_count, 4);
    // 2026-03-21: 这里补占位缺失率断言，目的是让 N/A/NA/null 一类值的缺失统计更完整。
    assert_eq!(notes_summary.missing_rate, Some(0.8));
    assert_eq!(notes_summary.distinct_count, Some(1));
    assert_eq!(notes_summary.top_values.len(), 1);
    assert_eq!(notes_summary.top_values[0].value, "done");
}

#[test]
fn summarize_table_handles_date_and_dirty_columns_stably() {
    let event_date = Series::new(
        "event_date".into(),
        vec![
            Some("2026-03-01"),
            Option::<&str>::None,
            Some("2026-03-03"),
            None,
            Some("2026-03-05"),
        ],
    );
    let dirty_score = Series::new(
        "dirty_score".into(),
        vec![Some("1"), Some("1.2"), Some("N/A"), Some(" "), Some("abc")],
    );
    let dataframe = DataFrame::new(vec![event_date.into(), dirty_score.into()]).unwrap();
    let loaded = LoadedTable {
        // 2026-03-21: 这里直接构造日期列与脏数据列，目的是在不依赖 Excel 解析细节时先锁定摘要核心语义。
        handle: TableHandle::new_confirmed(
            "memory://summary-date-dirty",
            "Sheet1",
            vec!["event_date".into(), "dirty_score".into()],
        ),
        dataframe,
    };

    let summaries = summarize_table(&loaded, &["event_date", "dirty_score"], 5).unwrap();
    let date_summary = summaries
        .iter()
        .find(|summary| summary.column == "event_date")
        .unwrap();
    let dirty_summary = summaries
        .iter()
        .find(|summary| summary.column == "dirty_score")
        .unwrap();

    // 2026-03-21: 这里锁定日期文本列画像，目的是确认 V1 在不引入额外日期特性的前提下也能稳定摘要。
    assert_eq!(date_summary.dtype, "string");
    assert_eq!(date_summary.summary_kind, "string");
    assert_eq!(date_summary.count, 3);
    assert_eq!(date_summary.null_count, 2);
    assert_eq!(date_summary.missing_rate, Some(0.4));
    assert_eq!(date_summary.distinct_count, Some(3));
    // 2026-03-21: 这里锁定混合脏数据列的摘要，目的是确认占位值和空白值仍会并入口径一致的缺失统计。
    assert_eq!(dirty_summary.summary_kind, "string");
    assert_eq!(dirty_summary.count, 3);
    assert_eq!(dirty_summary.null_count, 2);
    assert_eq!(dirty_summary.missing_rate, Some(0.4));
    assert_eq!(dirty_summary.distinct_count, Some(3));
    assert_eq!(dirty_summary.top_values.len(), 3);
}

#[test]
fn analyze_table_builds_findings_from_summary_profiles() {
    let dataframe = DataFrame::new(vec![
        Series::new(
            "region".into(),
            vec![Some("East"), Some("East"), Some("East")],
        )
        .into(),
        Series::new("notes".into(), vec![Option::<&str>::None, None, None]).into(),
    ])
    .unwrap();
    let loaded = LoadedTable {
        // 2026-03-21: 这里直接构造低信息量列和全空列，目的是先锁定 analyze_table 的第一批规则输出。
        handle: TableHandle::new_confirmed(
            "memory://analyze-basic",
            "Sheet1",
            vec!["region".into(), "notes".into()],
        ),
        dataframe,
    };

    let result = analyze_table(&loaded, &[], 3);

    // 2026-03-21: 这里要求 analyze_table 至少能从摘要画像生成基础 finding，目的是把 bridge Tool 从占位状态推进到真正诊断。
    assert!(
        result
            .structured_findings
            .iter()
            .any(|finding| finding.code == "single_value_column"
                && finding.column.as_deref() == Some("region"))
    );
    assert!(
        result
            .structured_findings
            .iter()
            .any(|finding| finding.code == "all_missing"
                && finding.column.as_deref() == Some("notes"))
    );
    assert_eq!(result.table_health.level, "risky");
}

#[test]
fn analyze_table_flags_quality_risks() {
    let dataframe = DataFrame::new(vec![
        Series::new(
            "status".into(),
            vec![
                Some("active"),
                Some("active"),
                Some("active"),
                Some("active"),
            ],
        )
        .into(),
        Series::new(
            "phone".into(),
            vec![Some("1380000"), Option::<&str>::None, None, None],
        )
        .into(),
        Series::new("notes".into(), vec![Option::<&str>::None, None, None, None]).into(),
    ])
    .unwrap();
    let loaded = LoadedTable {
        // 2026-03-21: 这里构造高缺失列、全空列和单一取值列，目的是锁定 analyze_table 的第一批质量诊断规则。
        handle: TableHandle::new_confirmed(
            "memory://analyze-quality",
            "Sheet1",
            vec!["status".into(), "phone".into(), "notes".into()],
        ),
        dataframe,
    };

    let result = analyze_table(&loaded, &[], 3);

    // 2026-03-21: 这里要求高缺失列被识别出来，目的是让后续问答和建模都能优先聚焦高风险字段。
    assert!(
        result
            .structured_findings
            .iter()
            .any(|finding| finding.code == "high_missing_rate"
                && finding.column.as_deref() == Some("phone"))
    );
    assert!(
        result
            .structured_findings
            .iter()
            .any(|finding| finding.code == "all_missing"
                && finding.column.as_deref() == Some("notes"))
    );
    assert!(
        result
            .structured_findings
            .iter()
            .any(|finding| finding.code == "single_value_column"
                && finding.column.as_deref() == Some("status"))
    );
    assert_eq!(result.table_health.level, "risky");
}

#[test]
fn analyze_table_detects_duplicate_rows_and_key_risks() {
    let dataframe = DataFrame::new(vec![
        Series::new(
            "user_id".into(),
            vec![Some("1"), Some("1"), Some("2"), Option::<&str>::None],
        )
        .into(),
        Series::new(
            "region".into(),
            vec![Some("East"), Some("East"), Some("West"), Some("North")],
        )
        .into(),
        Series::new(
            "amount".into(),
            vec![Some("100"), Some("100"), Some("90"), Some("80")],
        )
        .into(),
    ])
    .unwrap();
    let loaded = LoadedTable {
        // 2026-03-21: 这里构造重复行、重复 ID 和空 ID，目的是锁定分析建模前最关键的键质量风险诊断。
        handle: TableHandle::new_confirmed(
            "memory://analyze-keys",
            "Sheet1",
            vec!["user_id".into(), "region".into(), "amount".into()],
        ),
        dataframe,
    };

    let result = analyze_table(&loaded, &[], 3);

    // 2026-03-21: 这里要求整行重复能被识别出来，目的是避免后续聚合、建模和关联时重复放大样本。
    assert!(
        result
            .structured_findings
            .iter()
            .any(|finding| finding.code == "duplicate_rows")
    );
    assert!(
        result
            .structured_findings
            .iter()
            .any(|finding| finding.code == "duplicate_candidate_key"
                && finding.column.as_deref() == Some("user_id"))
    );
    assert!(
        result
            .structured_findings
            .iter()
            .any(|finding| finding.code == "blank_candidate_key"
                && finding.column.as_deref() == Some("user_id"))
    );
}

#[test]
fn analyze_table_detects_distribution_risks() {
    let dataframe = DataFrame::new(vec![
        Series::new(
            "region".into(),
            vec![
                Some("East"),
                Some("East"),
                Some("East"),
                Some("East"),
                Some("West"),
            ],
        )
        .into(),
        Series::new("zero_metric".into(), vec![0_i64, 0, 0, 0, 10]).into(),
        Series::new("amount".into(), vec![1_i64, 2, 2, 3, 100]).into(),
    ])
    .unwrap();
    let loaded = LoadedTable {
        // 2026-03-21: 这里构造类别失衡、零值占比高和异常值列，目的是锁定 bridge Tool 的轻量统计增强能力。
        handle: TableHandle::new_confirmed(
            "memory://analyze-distribution",
            "Sheet1",
            vec!["region".into(), "zero_metric".into(), "amount".into()],
        ),
        dataframe,
    };

    let result = analyze_table(&loaded, &[], 3);

    // 2026-03-21: 这里要求类别高度集中能被识别出来，目的是提醒用户先关注失衡分布。
    assert!(
        result
            .structured_findings
            .iter()
            .any(|finding| finding.code == "high_category_imbalance"
                && finding.column.as_deref() == Some("region"))
    );
    assert!(
        result
            .structured_findings
            .iter()
            .any(|finding| finding.code == "high_zero_ratio"
                && finding.column.as_deref() == Some("zero_metric"))
    );
    assert!(
        result
            .structured_findings
            .iter()
            .any(|finding| finding.code == "outlier_suspected"
                && finding.column.as_deref() == Some("amount"))
    );
}

#[test]
fn analyze_table_builds_business_observations() {
    let dataframe = DataFrame::new(vec![
        Series::new(
            "region".into(),
            vec![Some("East"), Some("East"), Some("West"), Some("East")],
        )
        .into(),
        Series::new("amount".into(), vec![10_i64, 20, 30, 40]).into(),
    ])
    .unwrap();
    let loaded = LoadedTable {
        // 2026-03-21: 这里构造主类别和数值范围都很直观的小表，目的是先锁定独立 business_observations 契约。
        handle: TableHandle::new_confirmed(
            "memory://analyze-business-observations",
            "Sheet1",
            vec!["region".into(), "amount".into()],
        ),
        dataframe,
    };

    let result = analyze_table(&loaded, &[], 3);

    // 2026-03-21: 这里要求类别主分布能进入业务观察，目的是让上层在质量诊断之外还能拿到少量可读业务提示。
    assert!(
        result
            .business_observations
            .iter()
            .any(|observation| observation.observation_type == "top_category"
                && observation.column.as_deref() == Some("region"))
    );
    // 2026-03-21: 这里要求数值范围也进入业务观察，目的是给分析建模层一个轻量统计桥接输出。
    assert!(
        result
            .business_observations
            .iter()
            .any(
                |observation| observation.observation_type == "numeric_range"
                    && observation.column.as_deref() == Some("amount")
            )
    );
}

#[test]
fn analyze_table_prioritizes_findings_and_compresses_major_issues() {
    let dataframe = DataFrame::new(vec![
        Series::new(
            "phone".into(),
            vec![Some("1380000"), Option::<&str>::None, None, None],
        )
        .into(),
        Series::new("notes".into(), vec![Option::<&str>::None, None, None, None]).into(),
    ])
    .unwrap();
    let loaded = LoadedTable {
        // 2026-03-21: 这里构造会同时触发多条同列 finding 的场景，目的是锁定排序稳定性和摘要压缩逻辑。
        handle: TableHandle::new_confirmed(
            "memory://analyze-priority",
            "Sheet1",
            vec!["phone".into(), "notes".into()],
        ),
        dataframe,
    };

    let result = analyze_table(&loaded, &[], 5);

    // 2026-03-21: 这里锁定高优先级的全空列 finding 会排在前面，目的是让上层优先看到真正阻塞分析的问题。
    assert_eq!(result.structured_findings[0].code, "all_missing");
    assert_eq!(
        result.structured_findings[0].column.as_deref(),
        Some("notes")
    );
    // 2026-03-21: 这里锁定同一列的缺失风险会先于低信息量风险，目的是减少结果阅读时的噪音。
    let phone_finding_codes = result
        .structured_findings
        .iter()
        .filter(|finding| finding.column.as_deref() == Some("phone"))
        .map(|finding| finding.code.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        phone_finding_codes,
        vec![
            "high_missing_rate",
            "high_category_imbalance",
            "single_value_column"
        ]
    );
    // 2026-03-21: 这里锁定摘要主问题会压缩同列重复提示，目的是避免用户看到 phone 连续重复报错。
    assert_eq!(
        result
            .human_summary
            .major_issues
            .iter()
            .filter(|message| message.contains("phone"))
            .count(),
        1
    );
    // 2026-03-21: 这里锁定 notes 不会再被误识别成候选键，目的是修掉 contains(\"no\") 带来的假阳性。
    assert!(
        !result
            .structured_findings
            .iter()
            .any(|finding| finding.code == "blank_candidate_key"
                && finding.column.as_deref() == Some("notes"))
    );
}

#[test]
fn analyze_table_builds_extended_business_observations() {
    let dataframe = DataFrame::new(vec![
        Series::new(
            "region".into(),
            vec![
                Some("East"),
                Some("East"),
                Some("East"),
                Some("East"),
                Some("West"),
            ],
        )
        .into(),
        Series::new("amount".into(), vec![10_i64, 20, 30, 40, 50]).into(),
    ])
    .unwrap();
    let loaded = LoadedTable {
        // 2026-03-21: 这里构造主维度高度集中的小表，目的是锁定扩展后的业务观察类型。
        handle: TableHandle::new_confirmed(
            "memory://analyze-extended-business-observations",
            "Sheet1",
            vec!["region".into(), "amount".into()],
        ),
        dataframe,
    };

    let result = analyze_table(&loaded, &[], 5);

    // 2026-03-21: 这里要求 dominant_dimension 独立输出，目的是让桥接层能给出“主分布维度”提示而不混入质量 finding。
    assert!(
        result
            .business_observations
            .iter()
            .any(
                |observation| observation.observation_type == "dominant_dimension"
                    && observation.column.as_deref() == Some("region")
            )
    );
    // 2026-03-21: 这里要求 numeric_center 独立输出，目的是让分析建模前先拿到一层轻量中心统计观察。
    assert!(
        result
            .business_observations
            .iter()
            .any(
                |observation| observation.observation_type == "numeric_center"
                    && observation.column.as_deref() == Some("amount")
            )
    );
}

#[test]
fn analyze_table_detects_candidate_keys_from_chinese_and_compact_names() {
    let dataframe = DataFrame::new(vec![
        Series::new(
            "客户编号".into(),
            vec![Some("C001"), Some("C001"), Some("C002")],
        )
        .into(),
        Series::new("uid".into(), vec![Some("U1"), Some(""), Some("U2")]).into(),
        Series::new("userid".into(), vec![Some("A1"), Some("A1"), Some("A3")]).into(),
        Series::new("notice".into(), vec![Some("x"), Some("y"), Some("z")]).into(),
    ])
    .unwrap();
    let loaded = LoadedTable {
        // 2026-03-21: 这里构造中文和紧凑命名的键列，目的是锁定候选键识别增强不会再只依赖英文 token。
        handle: TableHandle::new_confirmed(
            "memory://analyze-candidate-key-names",
            "Sheet1",
            vec![
                "客户编号".into(),
                "uid".into(),
                "userid".into(),
                "notice".into(),
            ],
        ),
        dataframe,
    };

    let result = analyze_table(&loaded, &[], 5);

    // 2026-03-21: 这里锁定中文键名会被识别，目的是覆盖 Excel 中文业务表头场景。
    assert!(
        result
            .structured_findings
            .iter()
            .any(|finding| finding.code == "duplicate_candidate_key"
                && finding.column.as_deref() == Some("客户编号"))
    );
    // 2026-03-21: 这里锁定 uid 会被识别成候选键，目的是覆盖常见缩写命名。
    assert!(
        result
            .structured_findings
            .iter()
            .any(|finding| finding.code == "blank_candidate_key"
                && finding.column.as_deref() == Some("uid"))
    );
    // 2026-03-21: 这里锁定 userid 也会被识别，目的是覆盖无分隔符的紧凑英文命名。
    assert!(
        result
            .structured_findings
            .iter()
            .any(|finding| finding.code == "duplicate_candidate_key"
                && finding.column.as_deref() == Some("userid"))
    );
    // 2026-03-21: 这里继续锁定 notice 不会误报，目的是防止放宽规则后重新引入假阳性。
    assert!(
        !result
            .structured_findings
            .iter()
            .any(|finding| finding.column.as_deref() == Some("notice")
                && matches!(
                    finding.code.as_str(),
                    "duplicate_candidate_key" | "blank_candidate_key"
                ))
    );
}

#[test]
fn analyze_table_uses_median_center_for_skewed_numeric_column() {
    let dataframe = DataFrame::new(vec![
        Series::new("amount".into(), vec![1_i64, 2, 2, 3, 100]).into(),
    ])
    .unwrap();
    let loaded = LoadedTable {
        // 2026-03-21: 这里构造明显偏态且含极端值的数值列，目的是锁定中心统计会优先用中位数而不是均值。
        handle: TableHandle::new_confirmed(
            "memory://analyze-skewed-center",
            "Sheet1",
            vec!["amount".into()],
        ),
        dataframe,
    };

    let result = analyze_table(&loaded, &[], 5);

    // 2026-03-21: 这里锁定偏态列的中心观察会切到 median_center，目的是让业务提示更稳健。
    assert!(
        result
            .business_observations
            .iter()
            .any(
                |observation| observation.observation_type == "median_center"
                    && observation.column.as_deref() == Some("amount")
                    && observation.message.contains("2")
            )
    );
    // 2026-03-21: 这里锁定同一列不再输出 numeric_center，目的是避免均值和中位数同时出现造成理解混乱。
    assert!(
        !result
            .business_observations
            .iter()
            .any(
                |observation| observation.observation_type == "numeric_center"
                    && observation.column.as_deref() == Some("amount")
            )
    );
}

#[test]
fn stat_summary_builds_typed_statistical_profiles() {
    let dataframe = DataFrame::new(vec![
        Series::new("sales".into(), vec![0_i64, 10, 20, 30, 100]).into(),
        Series::new(
            "region".into(),
            vec![
                Some("East"),
                Some("East"),
                Some("West"),
                Some("East"),
                Some("North"),
            ],
        )
        .into(),
        Series::new(
            "is_active".into(),
            vec![
                Some(true),
                Some(false),
                Some(true),
                Some(true),
                Option::<bool>::None,
            ],
        )
        .into(),
    ])
    .unwrap();
    let loaded = LoadedTable {
        // 2026-03-21: 这里同时构造数值、类别和布尔列，目的是先锁定统计桥接 Tool 的三类基础输出结构。
        handle: TableHandle::new_confirmed(
            "memory://stat-summary-basic",
            "Sheet1",
            vec!["sales".into(), "region".into(), "is_active".into()],
        ),
        dataframe,
    };

    let result = stat_summary(&loaded, &[], 3).unwrap();

    // 2026-03-21: 这里锁定表级概览，目的是让后续建模 Tool 能先判断当前表的字段类型分布。
    assert_eq!(result.row_count, 5);
    assert_eq!(result.column_count, 3);
    assert_eq!(result.table_overview.numeric_columns, 1);
    assert_eq!(result.table_overview.categorical_columns, 1);
    assert_eq!(result.table_overview.boolean_columns, 1);

    let sales_summary = result
        .numeric_summaries
        .iter()
        .find(|summary| summary.column == "sales")
        .unwrap();
    // 2026-03-21: 这里锁定数值列的分位数和零值占比，目的是给后续回归和异常值判断提供稳定桥接统计量。
    assert_eq!(sales_summary.count, 5);
    assert_eq!(sales_summary.null_count, 0);
    assert_eq!(sales_summary.missing_rate, Some(0.0));
    assert_eq!(sales_summary.q1, Some(10.0));
    assert_eq!(sales_summary.median, Some(20.0));
    assert_eq!(sales_summary.q3, Some(30.0));
    assert_eq!(sales_summary.zero_ratio, Some(0.2));

    let region_summary = result
        .categorical_summaries
        .iter()
        .find(|summary| summary.column == "region")
        .unwrap();
    // 2026-03-21: 这里锁定类别列的主值占比，目的是让聚类前和业务问答都能快速理解分布集中度。
    assert_eq!(region_summary.count, 5);
    assert_eq!(region_summary.distinct_count, Some(3));
    assert_eq!(region_summary.top_values[0].value, "East");
    assert_eq!(region_summary.top_values[0].count, 3);
    assert_eq!(region_summary.top_share, Some(0.6));

    let active_summary = result
        .boolean_summaries
        .iter()
        .find(|summary| summary.column == "is_active")
        .unwrap();
    // 2026-03-21: 这里锁定布尔列 true_ratio，目的是让分类建模前能快速看到标签分布。
    assert_eq!(active_summary.count, 4);
    assert_eq!(active_summary.null_count, 1);
    assert_eq!(active_summary.true_count, Some(3));
    assert_eq!(active_summary.false_count, Some(1));
    assert_eq!(active_summary.true_ratio, Some(0.75));
}

#[test]
fn stat_summary_handles_skew_and_category_distribution() {
    let inference =
        infer_header_schema("tests/fixtures/analyze-distribution.xlsx", "Metrics").unwrap();
    let loaded = load_confirmed_table(
        "tests/fixtures/analyze-distribution.xlsx",
        "Metrics",
        &inference,
    )
    .unwrap();
    let casted = cast_column_types(
        &loaded,
        &[
            CastColumnSpec {
                column: "zero_metric".to_string(),
                target_type: CastTargetType::Int64,
            },
            CastColumnSpec {
                column: "amount".to_string(),
                target_type: CastTargetType::Int64,
            },
        ],
    )
    .unwrap();

    let result = stat_summary(&casted, &["region", "zero_metric", "amount"], 3).unwrap();
    let zero_metric_summary = result
        .numeric_summaries
        .iter()
        .find(|summary| summary.column == "zero_metric")
        .unwrap();
    let amount_summary = result
        .numeric_summaries
        .iter()
        .find(|summary| summary.column == "amount")
        .unwrap();
    let region_summary = result
        .categorical_summaries
        .iter()
        .find(|summary| summary.column == "region")
        .unwrap();

    // 2026-03-21: 这里锁定高零值列的 zero_ratio，目的是让建模前检查能直接识别“默认值过多”的数值字段。
    assert_eq!(zero_metric_summary.zero_ratio, Some(0.8));
    assert_eq!(zero_metric_summary.median, Some(0.0));
    // 2026-03-21: 这里锁定偏态列会稳定返回中位数与四分位数，目的是防止极端值把中心统计解释带偏。
    assert_eq!(amount_summary.median, Some(2.0));
    assert_eq!(amount_summary.q1, Some(2.0));
    assert_eq!(amount_summary.q3, Some(3.0));
    // 2026-03-21: 这里锁定类别主值占比，目的是让后续聚类前和业务问答都能看懂分布集中度。
    assert_eq!(region_summary.top_values[0].value, "East");
    assert_eq!(region_summary.top_share, Some(0.8));
    // 2026-03-21: 这里锁定人类摘要会带出长尾和主区域提示，目的是让终端直接展示也有业务可读性。
    assert!(
        result
            .human_summary
            .key_points
            .iter()
            .any(|message| message.contains("region") || message.contains("East"))
    );
    assert!(
        result
            .human_summary
            .key_points
            .iter()
            .any(|message| message.contains("amount") && message.contains("长尾"))
    );
}

#[test]
fn analyze_table_builds_date_time_and_amount_observations() {
    let dataframe = DataFrame::new(vec![
        Series::new(
            "订单日期".into(),
            vec![
                Some("2026-03-01"),
                Some("2026-03-02"),
                Some("2026-03-15"),
                Some("2026-03-20"),
                Some("2026-04-01"),
            ],
        )
        .into(),
        Series::new(
            "下单时间".into(),
            vec![
                Some("13:00"),
                Some("14:30"),
                Some("15:00"),
                Some("09:00"),
                Some("13:30"),
            ],
        )
        .into(),
        Series::new("实付金额".into(), vec![-20.0_f64, 20.0, 30.0, 40.0, 500.0]).into(),
    ])
    .unwrap();
    let loaded = LoadedTable {
        // 2026-03-21: 这里同时构造日期、时间和金额列，目的是先锁定观察增强的最小业务桥接输出。
        handle: TableHandle::new_confirmed(
            "memory://analyze-observation-enhancement",
            "Sheet1",
            vec!["订单日期".into(), "下单时间".into(), "实付金额".into()],
        ),
        dataframe,
    };

    let result = analyze_table(&loaded, &[], 12);

    // 2026-03-21: 这里锁定日期范围观察，目的是让用户先知道数据覆盖周期够不够做分析。
    assert!(
        result
            .business_observations
            .iter()
            .any(|observation| observation.observation_type == "date_range"
                && observation.column.as_deref() == Some("订单日期")
                && observation.message.contains("2026-03-01")
                && observation.message.contains("2026-04-01"))
    );
    // 2026-03-21: 这里锁定日期集中观察，目的是让用户直接看到记录是否主要集中在某个月份。
    assert!(
        result
            .business_observations
            .iter()
            .any(
                |observation| observation.observation_type == "date_concentration"
                    && observation.column.as_deref() == Some("订单日期")
                    && observation.message.contains("2026-03")
            )
    );
    // 2026-03-21: 这里锁定时间高峰观察，目的是让问答界面可以直白提示主要发生时段。
    assert!(
        result
            .business_observations
            .iter()
            .any(
                |observation| observation.observation_type == "time_peak_period"
                    && observation.column.as_deref() == Some("下单时间")
                    && observation.message.contains("下午")
            )
    );
    // 2026-03-21: 这里锁定金额典型区间观察，目的是让用户看到“常见金额带”而不是只看最小最大值。
    assert!(
        result
            .business_observations
            .iter()
            .any(
                |observation| observation.observation_type == "amount_typical_band"
                    && observation.column.as_deref() == Some("实付金额")
                    && observation.message.contains("20")
                    && observation.message.contains("40")
            )
    );
    // 2026-03-21: 这里锁定负金额观察，目的是提醒退款、冲销或净额类场景的特殊含义。
    assert!(
        result
            .business_observations
            .iter()
            .any(
                |observation| observation.observation_type == "amount_negative_presence"
                    && observation.column.as_deref() == Some("实付金额")
            )
    );
    // 2026-03-21: 这里锁定金额长尾观察，目的是把“少量高值拉高均值”的风险用业务语言表达出来。
    assert!(
        result
            .business_observations
            .iter()
            .any(
                |observation| observation.observation_type == "amount_skew_hint"
                    && observation.column.as_deref() == Some("实付金额")
                    && observation.message.contains("平均值")
            )
    );
}

#[test]
fn linear_regression_fits_numeric_features_and_target() {
    let dataframe = DataFrame::new(vec![
        Series::new("feature_a".into(), vec![1.0_f64, 2.0, 3.0, 4.0, 5.0]).into(),
        Series::new("feature_b".into(), vec![2.0_f64, 1.0, 3.0, 0.0, 4.0]).into(),
        Series::new("target".into(), vec![18.0_f64, 17.0, 25.0, 18.0, 32.0]).into(),
    ])
    .unwrap();
    let loaded = LoadedTable {
        // 2026-03-21: 这里构造一个精确线性关系样本，目的是先锁定 V1 线性回归会输出稳定系数、截距和 R2。
        handle: TableHandle::new_confirmed(
            "memory://linear-regression-basic",
            "Sheet1",
            vec!["feature_a".into(), "feature_b".into(), "target".into()],
        ),
        dataframe,
    };

    let result = linear_regression(
        &loaded,
        &["feature_a", "feature_b"],
        "target",
        true,
        MissingStrategy::DropRows,
    )
    .unwrap();

    // 2026-03-21: 这里锁定有效样本行数，目的是保证后续缺失删行逻辑不会误删正常样本。
    assert_eq!(result.row_count_used, 5);
    assert_eq!(result.dropped_rows, 0);
    assert_eq!(result.model_kind, "linear_regression");
    assert_eq!(result.problem_type, "regression");
    assert_eq!(result.data_summary.feature_count, 2);
    assert_eq!(result.quality_summary.primary_metric.name, "r2");
    // 2026-03-21: 这里锁定系数顺序与特征列一一对应，目的是让 Skill 和上层问答可以稳定引用建模结果。
    assert_eq!(result.coefficients.len(), 2);
    assert_eq!(result.coefficients[0].feature, "feature_a");
    assert!((result.coefficients[0].value - 2.0).abs() < 1e-9);
    assert_eq!(result.coefficients[1].feature, "feature_b");
    assert!((result.coefficients[1].value - 3.0).abs() < 1e-9);
    // 2026-03-21: 这里锁定截距和拟合优度，目的是确保回归核心计算不是只返回了近似轮廓。
    assert!((result.intercept - 10.0).abs() < 1e-9);
    assert!((result.r2 - 1.0).abs() < 1e-9);
    // 2026-03-21: 这里锁定中文摘要非空，目的是让非技术用户也能直接读取结果。
    assert!(result.human_summary.overall.contains("有效样本"));
}

#[test]
fn linear_regression_rejects_non_numeric_target() {
    let dataframe = DataFrame::new(vec![
        Series::new("feature_a".into(), vec![1_i64, 2, 3]).into(),
        Series::new("label".into(), vec!["高", "中", "低"]).into(),
    ])
    .unwrap();
    let loaded = LoadedTable {
        // 2026-03-21: 这里构造文本目标列，目的是先锁定 V1 不会把非数值目标误送进线性回归。
        handle: TableHandle::new_confirmed(
            "memory://linear-regression-non-numeric-target",
            "Sheet1",
            vec!["feature_a".into(), "label".into()],
        ),
        dataframe,
    };

    let error = linear_regression(
        &loaded,
        &["feature_a"],
        "label",
        true,
        MissingStrategy::DropRows,
    )
    .unwrap_err();

    // 2026-03-21: 这里锁定直白错误文案，目的是让低 IT 用户也能知道问题出在目标列类型上。
    assert!(error.to_string().contains("目标列 `label` 不是数值列"));
}

#[test]
fn linear_regression_rejects_non_numeric_feature() {
    let dataframe = DataFrame::new(vec![
        Series::new("region".into(), vec!["East", "West", "North"]).into(),
        Series::new("target".into(), vec![10_i64, 20, 30]).into(),
    ])
    .unwrap();
    let loaded = LoadedTable {
        // 2026-03-21: 这里构造文本特征列，目的是先锁定 V1 需要显式数值特征而不是偷偷猜测编码。
        handle: TableHandle::new_confirmed(
            "memory://linear-regression-non-numeric-feature",
            "Sheet1",
            vec!["region".into(), "target".into()],
        ),
        dataframe,
    };

    let error = linear_regression(
        &loaded,
        &["region"],
        "target",
        true,
        MissingStrategy::DropRows,
    )
    .unwrap_err();

    // 2026-03-21: 这里锁定特征列类型错误，目的是让用户知道要先 cast 或换列，而不是拿到模糊失败。
    assert!(error.to_string().contains("特征列 `region` 不是数值列"));
}

#[test]
fn linear_regression_drops_missing_rows_before_fit() {
    let dataframe = DataFrame::new(vec![
        Series::new(
            "feature_a".into(),
            vec![Some(1.0_f64), Some(2.0), None, Some(4.0), Some(5.0)],
        )
        .into(),
        Series::new(
            "target".into(),
            vec![
                Some(12.0_f64),
                Some(14.0),
                Some(16.0),
                Some(18.0),
                Some(20.0),
            ],
        )
        .into(),
    ])
    .unwrap();
    let loaded = LoadedTable {
        // 2026-03-21: 这里故意放入一个缺失特征值，目的是锁定 V1 会先删掉坏行再建模，而不是直接报错中断。
        handle: TableHandle::new_confirmed(
            "memory://linear-regression-drop-missing",
            "Sheet1",
            vec!["feature_a".into(), "target".into()],
        ),
        dataframe,
    };

    let result = linear_regression(
        &loaded,
        &["feature_a"],
        "target",
        true,
        MissingStrategy::DropRows,
    )
    .unwrap();

    // 2026-03-21: 这里锁定删行后的样本数和丢弃数，目的是让上层能透明提示“哪些数据被跳过了”。
    assert_eq!(result.row_count_used, 4);
    assert_eq!(result.dropped_rows, 1);
    // 2026-03-21: 这里锁定缺失删行后仍能恢复正确关系，目的是保证删行逻辑不会污染拟合结果。
    assert!((result.coefficients[0].value - 2.0).abs() < 1e-9);
    assert!((result.intercept - 10.0).abs() < 1e-9);
}

#[test]
fn linear_regression_rejects_singular_feature_matrix() {
    let dataframe = DataFrame::new(vec![
        Series::new("feature_a".into(), vec![1.0_f64, 2.0, 3.0, 4.0]).into(),
        Series::new("feature_b".into(), vec![2.0_f64, 4.0, 6.0, 8.0]).into(),
        Series::new("target".into(), vec![5.0_f64, 9.0, 13.0, 17.0]).into(),
    ])
    .unwrap();
    let loaded = LoadedTable {
        // 2026-03-21: 这里构造完全重复信息的两列特征，目的是锁定奇异矩阵会被识别并给出直白报错。
        handle: TableHandle::new_confirmed(
            "memory://linear-regression-singular",
            "Sheet1",
            vec!["feature_a".into(), "feature_b".into(), "target".into()],
        ),
        dataframe,
    };

    let error = linear_regression(
        &loaded,
        &["feature_a", "feature_b"],
        "target",
        true,
        MissingStrategy::DropRows,
    )
    .unwrap_err();

    // 2026-03-21: 这里锁定共线性错误文案，目的是避免用户只看到“矩阵不可逆”这种难懂术语。
    assert!(error.to_string().contains("特征列之间过于重复"));
}

#[test]
fn model_prep_builds_regression_dataset_with_drop_rows() {
    let dataframe = DataFrame::new(vec![
        Series::new(
            "feature_a".into(),
            vec![Some(1.0_f64), Some(2.0), None, Some(4.0)],
        )
        .into(),
        Series::new(
            "feature_b".into(),
            vec![Some(10.0_f64), Some(20.0), Some(30.0), Some(40.0)],
        )
        .into(),
        Series::new(
            "target".into(),
            vec![Some(5.0_f64), Some(7.0), Some(9.0), Some(11.0)],
        )
        .into(),
    ])
    .unwrap();
    let loaded = LoadedTable {
        // 2026-03-21: 这里构造含缺失值的回归样本，目的是先锁定公共准备层会按统一口径删行并保留完整矩阵。
        handle: TableHandle::new_confirmed(
            "memory://model-prep-regression",
            "Sheet1",
            vec!["feature_a".into(), "feature_b".into(), "target".into()],
        ),
        dataframe,
    };

    let prepared = prepare_regression_dataset(
        &loaded,
        &["feature_a", "feature_b"],
        "target",
        true,
        MissingStrategy::DropRows,
    )
    .unwrap();

    // 2026-03-21: 这里锁定删行统计，目的是保证后续回归和聚类共享同一套有效样本口径。
    assert_eq!(prepared.row_count_used, 3);
    assert_eq!(prepared.dropped_rows, 1);
    // 2026-03-21: 这里锁定截距列会被写入设计矩阵，目的是让上层算法不必重复处理 intercept 逻辑。
    assert_eq!(prepared.feature_matrix[0], vec![1.0, 1.0, 10.0]);
    assert_eq!(prepared.targets, vec![5.0, 7.0, 11.0]);
}

#[test]
fn model_prep_builds_binary_classification_dataset_from_text_labels() {
    let dataframe = DataFrame::new(vec![
        Series::new("age".into(), vec![18.0_f64, 22.0, 38.0, 45.0]).into(),
        Series::new("score".into(), vec![10.0_f64, 15.0, 50.0, 60.0]).into(),
        Series::new("result".into(), vec!["失败", "失败", "成功", "成功"]).into(),
    ])
    .unwrap();
    let loaded = LoadedTable {
        // 2026-03-21: 这里构造中文二分类目标列，目的是锁定公共准备层会稳定映射正负类而不是把文本直接丢给算法层。
        handle: TableHandle::new_confirmed(
            "memory://model-prep-binary",
            "Sheet1",
            vec!["age".into(), "score".into(), "result".into()],
        ),
        dataframe,
    };

    let prepared = prepare_binary_classification_dataset(
        &loaded,
        &["age", "score"],
        "result",
        true,
        MissingStrategy::DropRows,
        Some("成功"),
    )
    .unwrap();

    // 2026-03-21: 这里锁定正类标签和数值映射，目的是让逻辑回归结果能围绕用户理解的“正类”输出。
    assert_eq!(prepared.row_count_used, 4);
    assert_eq!(prepared.positive_label, "成功");
    assert_eq!(prepared.negative_label, "失败");
    assert_eq!(prepared.targets, vec![0.0, 0.0, 1.0, 1.0]);
}

#[test]
fn model_prep_rejects_non_binary_target_values() {
    let dataframe = DataFrame::new(vec![
        Series::new("feature_a".into(), vec![1.0_f64, 2.0, 3.0]).into(),
        Series::new("stage".into(), vec!["新建", "处理中", "完成"]).into(),
    ])
    .unwrap();
    let loaded = LoadedTable {
        // 2026-03-21: 这里构造三分类目标列，目的是先锁定 V1 二分类准备层不会把多分类误放行。
        handle: TableHandle::new_confirmed(
            "memory://model-prep-non-binary",
            "Sheet1",
            vec!["feature_a".into(), "stage".into()],
        ),
        dataframe,
    };

    let error = prepare_binary_classification_dataset(
        &loaded,
        &["feature_a"],
        "stage",
        true,
        MissingStrategy::DropRows,
        None,
    )
    .unwrap_err();

    // 2026-03-21: 这里锁定直白报错，目的是让用户知道当前不是二分类而不是看到抽象映射失败。
    assert!(error.to_string().contains("只支持二分类"));
}

#[test]
fn logistic_regression_fits_binary_target_and_reports_accuracy() {
    let dataframe = DataFrame::new(vec![
        Series::new("score".into(), vec![1.0_f64, 2.0, 3.0, 7.0, 8.0, 9.0]).into(),
        Series::new("result".into(), vec![0_i64, 0, 0, 1, 1, 1]).into(),
    ])
    .unwrap();
    let loaded = LoadedTable {
        // 2026-03-21: 这里构造可分离的二分类数据，目的是先锁定逻辑回归 V1 能完成最小训练闭环。
        handle: TableHandle::new_confirmed(
            "memory://logistic-regression-basic",
            "Sheet1",
            vec!["score".into(), "result".into()],
        ),
        dataframe,
    };

    let result = logistic_regression(
        &loaded,
        &["score"],
        "result",
        true,
        MissingStrategy::DropRows,
        None,
    )
    .unwrap();

    // 2026-03-21: 这里锁定逻辑回归输出基础字段，目的是确保分析建模层能稳定消费分类模型结果。
    assert_eq!(result.row_count_used, 6);
    assert_eq!(result.dropped_rows, 0);
    assert_eq!(result.model_kind, "logistic_regression");
    assert_eq!(result.problem_type, "classification");
    assert_eq!(result.data_summary.feature_count, 1);
    assert_eq!(
        result.quality_summary.primary_metric.name,
        "training_accuracy"
    );
    assert_eq!(result.positive_label, "1");
    assert_eq!(result.coefficients.len(), 1);
    assert!(result.training_accuracy >= 0.99);
    assert!(result.human_summary.overall.contains("有效样本"));
}

#[test]
fn logistic_regression_honors_positive_label_for_text_target() {
    let dataframe = DataFrame::new(vec![
        Series::new("amount".into(), vec![10.0_f64, 20.0, 80.0, 90.0]).into(),
        Series::new("status".into(), vec!["未成交", "未成交", "成交", "成交"]).into(),
    ])
    .unwrap();
    let loaded = LoadedTable {
        // 2026-03-21: 这里构造文本标签并指定正类，目的是锁定“哪一类算正类”不会被系统擅自改写。
        handle: TableHandle::new_confirmed(
            "memory://logistic-regression-positive-label",
            "Sheet1",
            vec!["amount".into(), "status".into()],
        ),
        dataframe,
    };

    let result = logistic_regression(
        &loaded,
        &["amount"],
        "status",
        true,
        MissingStrategy::DropRows,
        Some("成交"),
    )
    .unwrap();

    // 2026-03-21: 这里锁定正类标签和类别分布，目的是让上层摘要围绕用户关心的“成交”来解释模型。
    assert_eq!(result.positive_label, "成交");
    assert_eq!(result.class_balance.positive_count, 2);
    assert_eq!(result.class_balance.negative_count, 2);
}

#[test]
fn logistic_regression_rejects_non_binary_target() {
    let dataframe = DataFrame::new(vec![
        Series::new("feature_a".into(), vec![1.0_f64, 2.0, 3.0, 4.0]).into(),
        Series::new("status".into(), vec!["新建", "处理中", "完成", "完成"]).into(),
    ])
    .unwrap();
    let loaded = LoadedTable {
        // 2026-03-21: 这里构造三种状态值，目的是先锁定逻辑回归 V1 只接受二分类目标列。
        handle: TableHandle::new_confirmed(
            "memory://logistic-regression-invalid-target",
            "Sheet1",
            vec!["feature_a".into(), "status".into()],
        ),
        dataframe,
    };

    let error = logistic_regression(
        &loaded,
        &["feature_a"],
        "status",
        true,
        MissingStrategy::DropRows,
        None,
    )
    .unwrap_err();

    // 2026-03-21: 这里锁定二分类限制提示，目的是让用户明白不是所有标签列都能直接做逻辑回归。
    assert!(error.to_string().contains("只支持二分类"));
}

#[test]
fn cluster_kmeans_groups_numeric_rows_and_returns_unified_model_fields() {
    let dataframe = DataFrame::new(vec![
        Series::new("x".into(), vec![1.0_f64, 1.2, 0.8, 10.0, 10.2, 9.8]).into(),
        Series::new("y".into(), vec![1.0_f64, 0.9, 1.1, 10.0, 10.1, 9.9]).into(),
    ])
    .unwrap();
    let loaded = LoadedTable {
        // 2026-03-21: 这里构造两团明显分开的点，目的是先锁定聚类 Tool 的最小训练闭环和统一输出结构。
        handle: TableHandle::new_confirmed(
            "memory://cluster-kmeans-basic",
            "Sheet1",
            vec!["x".into(), "y".into()],
        ),
        dataframe,
    };

    let result = cluster_kmeans(&loaded, &["x", "y"], 2, 50, MissingStrategy::DropRows).unwrap();

    // 2026-03-21: 这里锁定共享建模总览字段，目的是让分析建模层三类 Tool 以后都能按统一入口被 Skill 消费。
    assert_eq!(result.model_kind, "cluster_kmeans");
    assert_eq!(result.problem_type, "clustering");
    assert_eq!(result.data_summary.feature_count, 2);
    assert_eq!(result.data_summary.row_count_used, 6);
    assert_eq!(result.data_summary.dropped_rows, 0);
    assert_eq!(result.quality_summary.primary_metric.name, "inertia");
    // 2026-03-21: 这里锁定聚类主结果字段，目的是确保 V1 至少能稳定返回分组数量、中心点和成员归属。
    assert_eq!(result.cluster_count, 2);
    assert_eq!(result.assignments.len(), 6);
    assert_eq!(result.cluster_sizes.len(), 2);
    assert_eq!(result.cluster_centers.len(), 2);
    assert_eq!(
        result
            .cluster_sizes
            .iter()
            .map(|item| item.row_count)
            .sum::<usize>(),
        6
    );
    assert!(result.human_summary.overall.contains("聚类"));
}

#[test]
fn cluster_kmeans_drops_missing_rows_before_training() {
    let dataframe = DataFrame::new(vec![
        Series::new(
            "x".into(),
            vec![Some(1.0_f64), Some(1.1), None, Some(9.9), Some(10.1)],
        )
        .into(),
        Series::new(
            "y".into(),
            vec![Some(1.0_f64), Some(1.2), Some(5.0), Some(10.0), Some(9.8)],
        )
        .into(),
    ])
    .unwrap();
    let loaded = LoadedTable {
        // 2026-03-21: 这里故意放入缺失特征值，目的是锁定聚类也沿用统一删行口径，而不是悄悄混入脏样本。
        handle: TableHandle::new_confirmed(
            "memory://cluster-kmeans-drop-missing",
            "Sheet1",
            vec!["x".into(), "y".into()],
        ),
        dataframe,
    };

    let result = cluster_kmeans(&loaded, &["x", "y"], 2, 50, MissingStrategy::DropRows).unwrap();

    // 2026-03-21: 这里锁定删行统计，目的是让上层能透明提示“哪些数据没有进入聚类”。
    assert_eq!(result.row_count_used, 4);
    assert_eq!(result.dropped_rows, 1);
    assert_eq!(result.assignments.len(), 4);
}

#[test]
fn cluster_kmeans_rejects_invalid_cluster_count() {
    let dataframe = DataFrame::new(vec![
        Series::new("x".into(), vec![1.0_f64, 2.0, 3.0]).into(),
        Series::new("y".into(), vec![1.0_f64, 2.0, 3.0]).into(),
    ])
    .unwrap();
    let loaded = LoadedTable {
        // 2026-03-21: 这里构造最小样本集，目的是锁定 K 值非法时会直接给出直白错误，而不是训练阶段才崩。
        handle: TableHandle::new_confirmed(
            "memory://cluster-kmeans-invalid-k",
            "Sheet1",
            vec!["x".into(), "y".into()],
        ),
        dataframe,
    };

    let error = cluster_kmeans(&loaded, &["x", "y"], 4, 50, MissingStrategy::DropRows).unwrap_err();

    // 2026-03-21: 这里锁定 K 值报错文案，目的是让低 IT 用户也知道是“分组数太大”而不是算法黑盒异常。
    assert!(error.to_string().contains("分组数"));
}

#[test]
fn decision_assistant_prioritizes_quality_actions_and_tool_suggestions() {
    let dataframe = DataFrame::new(vec![
        Series::new(
            "user_id".into(),
            vec![Some(1_i64), Some(1_i64), Some(2_i64), Option::<i64>::None],
        )
        .into(),
        Series::new(
            "region".into(),
            vec![Some("East"), Some("East"), Some("West"), Some("East")],
        )
        .into(),
        Series::new(
            "amount".into(),
            vec![Some(100.0_f64), Some(100.0), Some(90.0), Some(0.0)],
        )
        .into(),
        Series::new(
            "success".into(),
            vec![Some("是"), Some("是"), Some("否"), Some("否")],
        )
        .into(),
    ])
    .unwrap();
    let loaded = LoadedTable {
        // 2026-03-21: 这里构造重复键、重复行和可建模字段并存的小表，目的是锁定决策助手会先抓质量阻塞，再给下一步建议。
        handle: TableHandle::new_confirmed(
            "memory://decision-assistant-priority",
            "Sheet1",
            vec![
                "user_id".into(),
                "region".into(),
                "amount".into(),
                "success".into(),
            ],
        ),
        dataframe,
    };

    let result = decision_assistant(&loaded, &[], 5).unwrap();

    // 2026-03-21: 这里锁定决策助手整体形态，目的是让上层问答界面直接拿到“阻塞项 + 动作 + 下一步”的稳定结构。
    assert_eq!(result.assistant_kind, "quality_diagnostic");
    assert_eq!(result.table_health.level, "risky");
    assert!(!result.blocking_risks.is_empty());
    assert!(!result.priority_actions.is_empty());
    assert!(!result.next_tool_suggestions.is_empty());
    // 2026-03-21: 这里锁定高优先级动作会先指向质量问题，目的是避免用户还没清洗数据就被引导去建模。
    assert_eq!(result.priority_actions[0].priority, "high");
    assert!(result.priority_actions[0].title.contains("先处理"));
    // 2026-03-21: 这里锁定在有数值列和分类列时会给出建模候选，目的是把决策助手真正桥接到分析建模层。
    assert!(
        result
            .next_tool_suggestions
            .iter()
            .any(|suggestion| suggestion.tool == "cluster_kmeans")
    );
    assert!(
        result
            .next_tool_suggestions
            .iter()
            .any(|suggestion| suggestion.tool == "logistic_regression")
    );
    assert!(result.human_summary.overall.contains("优先"));
}

#[test]
fn suggest_table_links_returns_obvious_join_candidate_for_matching_key_columns() {
    let left_inference =
        infer_header_schema("tests/fixtures/join-customers.xlsx", "Customers").unwrap();
    let right_inference = infer_header_schema("tests/fixtures/join-orders.xlsx", "Orders").unwrap();
    let left = load_confirmed_table(
        "tests/fixtures/join-customers.xlsx",
        "Customers",
        &left_inference,
    )
    .unwrap();
    let right = load_confirmed_table(
        "tests/fixtures/join-orders.xlsx",
        "Orders",
        &right_inference,
    )
    .unwrap();

    let result = suggest_table_links(&left, &right, 3).unwrap();

    // 2026-03-21: 这里锁定明显的 user_id 关联候选会被直接识别，目的是把多表工作流推进到“先建议、再执行”的阶段。
    assert!(!result.candidates.is_empty());
    assert_eq!(result.candidates[0].left_column, "user_id");
    assert_eq!(result.candidates[0].right_column, "user_id");
    assert_eq!(result.candidates[0].confidence, "high");
    assert!(result.candidates[0].match_row_count >= 2);
    // 2026-03-21: 这里锁定返回的是业务语言问题而不是技术术语，目的是让 Skill 可以直接拿去问用户。
    assert!(result.candidates[0].question.contains("是否用"));
    assert_eq!(
        result.candidates[0].keep_mode_options[0].keep_mode,
        "matched_only"
    );
}

#[test]
fn suggest_table_links_returns_empty_candidates_when_no_obvious_relation_exists() {
    let left = LoadedTable {
        // 2026-03-21: 这里构造两张没有明显交集的表，目的是锁定 V2.1 只做保守建议，不会乱猜关联。
        handle: TableHandle::new_confirmed(
            "memory://table-links-left",
            "Sheet1",
            vec!["customer_name".into(), "region".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("customer_name".into(), vec![Some("Alice"), Some("Bob")]).into(),
            Series::new("region".into(), vec![Some("East"), Some("West")]).into(),
        ])
        .unwrap(),
    };
    let right = LoadedTable {
        // 2026-03-21: 这里让右表只包含订单金额类字段，目的是确认没有显性键特征时系统会明确返回“没识别到”。
        handle: TableHandle::new_confirmed(
            "memory://table-links-right",
            "Sheet1",
            vec!["order_amount".into(), "channel".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("order_amount".into(), vec![10_i64, 20]).into(),
            Series::new("channel".into(), vec![Some("Online"), Some("Store")]).into(),
        ])
        .unwrap(),
    };

    let result = suggest_table_links(&left, &right, 3).unwrap();

    // 2026-03-21: 这里锁定系统在没有明显关系时不会伪造候选，目的是避免误导用户做错误关联。
    assert_eq!(result.candidates.len(), 0);
    assert!(result.human_summary.contains("没有识别"));
}

#[test]
fn suggest_table_workflow_recommends_append_for_same_schema_tables() {
    let left_inference =
        infer_header_schema("tests/fixtures/append-sales-a.xlsx", "Sales").unwrap();
    let right_inference =
        infer_header_schema("tests/fixtures/append-sales-reordered.xlsx", "Sales").unwrap();
    let left = load_confirmed_table(
        "tests/fixtures/append-sales-a.xlsx",
        "Sales",
        &left_inference,
    )
    .unwrap();
    let right = load_confirmed_table(
        "tests/fixtures/append-sales-reordered.xlsx",
        "Sales",
        &right_inference,
    )
    .unwrap();

    let result = suggest_table_workflow(&left, &right, 3).unwrap();

    // 2026-03-22: 这里锁定结构一致表会优先推荐纵向追加，目的是把“同结构分批到达”的场景从 Skill 猜测转成稳定 Tool 判断。
    assert_eq!(result.recommended_action, "append_tables");
    assert!(result.append_candidate.is_some());
    assert_eq!(result.append_candidate.as_ref().unwrap().confidence, "high");
    // 2026-03-22: 这里锁定追加建议直接输出业务确认问题，目的是让非技术用户也能听懂“是否上下拼接”。
    assert!(
        result
            .append_candidate
            .as_ref()
            .unwrap()
            .question
            .contains("追加")
    );
    // 2026-03-22: 这里锁定工作流建议会直接给出追加调用骨架，目的是让 Skill 不再自己拼接 top/bottom 参数。
    assert_eq!(
        result.suggested_tool_call.as_ref().unwrap().tool,
        "append_tables"
    );
    assert_eq!(
        result.suggested_tool_call.as_ref().unwrap().args["top"]["sheet"],
        "Sales"
    );
    assert!(result.human_summary.contains("结构相同"));
}

#[test]
fn suggest_table_workflow_recommends_join_for_linkable_tables() {
    let left_inference =
        infer_header_schema("tests/fixtures/join-customers.xlsx", "Customers").unwrap();
    let right_inference = infer_header_schema("tests/fixtures/join-orders.xlsx", "Orders").unwrap();
    let left = load_confirmed_table(
        "tests/fixtures/join-customers.xlsx",
        "Customers",
        &left_inference,
    )
    .unwrap();
    let right = load_confirmed_table(
        "tests/fixtures/join-orders.xlsx",
        "Orders",
        &right_inference,
    )
    .unwrap();

    let result = suggest_table_workflow(&left, &right, 3).unwrap();

    // 2026-03-22: 这里锁定结构不一致但显性键清楚时会推荐关联，目的是把“先判断动作类型”稳定沉淀到工作流层。
    assert_eq!(result.recommended_action, "join_tables");
    assert!(result.append_candidate.is_none());
    assert!(!result.link_candidates.is_empty());
    assert_eq!(result.link_candidates[0].left_column, "user_id");
    assert_eq!(result.link_candidates[0].right_column, "user_id");
    // 2026-03-22: 这里锁定工作流建议会直接给出关联调用骨架，目的是让 Skill 能直接承接首个显性候选。
    assert_eq!(
        result.suggested_tool_call.as_ref().unwrap().tool,
        "join_tables"
    );
    assert_eq!(
        result.suggested_tool_call.as_ref().unwrap().args["left_on"],
        "user_id"
    );
}

#[test]
fn suggest_table_workflow_falls_back_to_manual_confirmation_when_no_obvious_action_exists() {
    let left = LoadedTable {
        // 2026-03-22: 这里构造既不能直接追加、也没有显性键关系的表，目的是锁定工作流层会保守回退而不是强行推荐动作。
        handle: TableHandle::new_confirmed(
            "memory://table-workflow-left",
            "Sheet1",
            vec!["customer_name".into(), "region".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("customer_name".into(), vec![Some("Alice"), Some("Bob")]).into(),
            Series::new("region".into(), vec![Some("East"), Some("West")]).into(),
        ])
        .unwrap(),
    };
    let right = LoadedTable {
        // 2026-03-22: 这里让右表只保留金额和渠道列，目的是确认工作流层在无明显动作时会明确要求继续确认。
        handle: TableHandle::new_confirmed(
            "memory://table-workflow-right",
            "Sheet1",
            vec!["order_amount".into(), "channel".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("order_amount".into(), vec![10_i64, 20]).into(),
            Series::new("channel".into(), vec![Some("Online"), Some("Store")]).into(),
        ])
        .unwrap(),
    };

    let result = suggest_table_workflow(&left, &right, 3).unwrap();

    // 2026-03-22: 这里锁定保守回退路径，目的是避免系统在多表场景里给出错误追加或错误关联建议。
    assert_eq!(result.recommended_action, "manual_confirmation");
    assert!(result.append_candidate.is_none());
    assert_eq!(result.link_candidates.len(), 0);
    assert!(result.suggested_tool_call.is_none());
    assert!(result.human_summary.contains("没有识别"));
}

#[test]
fn suggest_multi_table_plan_builds_append_chain_for_same_schema_tables() {
    let first_inference =
        infer_header_schema("tests/fixtures/append-sales-a.xlsx", "Sales").unwrap();
    let second_inference =
        infer_header_schema("tests/fixtures/append-sales-b.xlsx", "Sales").unwrap();
    let third_inference =
        infer_header_schema("tests/fixtures/append-sales-reordered.xlsx", "Sales").unwrap();
    let first = load_confirmed_table(
        "tests/fixtures/append-sales-a.xlsx",
        "Sales",
        &first_inference,
    )
    .unwrap();
    let second = load_confirmed_table(
        "tests/fixtures/append-sales-b.xlsx",
        "Sales",
        &second_inference,
    )
    .unwrap();
    let third = load_confirmed_table(
        "tests/fixtures/append-sales-reordered.xlsx",
        "Sales",
        &third_inference,
    )
    .unwrap();

    let result = suggest_multi_table_plan(
        vec![
            ("sales_a".to_string(), first),
            ("sales_b".to_string(), second),
            ("sales_c".to_string(), third),
        ],
        3,
    )
    .unwrap();

    // 2026-03-22: 这里锁定多张同结构表会先生成追加链，目的是把“先合并同结构批次数据”沉淀成多表计划的默认顺序。
    assert_eq!(result.steps.len(), 2);
    assert_eq!(result.steps[0].action, "append_tables");
    assert_eq!(result.steps[0].input_refs, vec!["sales_a", "sales_b"]);
    assert!(result.steps[0].question.contains("追加"));
    assert_eq!(result.steps[1].action, "append_tables");
    assert_eq!(result.steps[1].input_refs, vec!["step_1_result", "sales_c"]);
    assert_eq!(result.unresolved_refs.len(), 1);
    assert_eq!(result.unresolved_refs[0], "step_2_result");
}

#[test]
fn suggest_multi_table_plan_builds_join_step_for_linkable_tables() {
    let left_inference =
        infer_header_schema("tests/fixtures/join-customers.xlsx", "Customers").unwrap();
    let right_inference = infer_header_schema("tests/fixtures/join-orders.xlsx", "Orders").unwrap();
    let left = load_confirmed_table(
        "tests/fixtures/join-customers.xlsx",
        "Customers",
        &left_inference,
    )
    .unwrap();
    let right = load_confirmed_table(
        "tests/fixtures/join-orders.xlsx",
        "Orders",
        &right_inference,
    )
    .unwrap();

    let result = suggest_multi_table_plan(
        vec![
            ("customers".to_string(), left),
            ("orders".to_string(), right),
        ],
        3,
    )
    .unwrap();

    // 2026-03-22: 这里锁定显性可关联的双表会生成 join 计划，目的是让多表计划器能直接承接已完成的关系建议能力。
    assert_eq!(result.steps.len(), 1);
    assert_eq!(result.steps[0].action, "join_tables");
    assert_eq!(result.steps[0].input_refs, vec!["customers", "orders"]);
    assert!(result.steps[0].question.contains("是否用"));
    assert_eq!(result.steps[0].suggested_tool_call["tool"], "join_tables");
    assert_eq!(
        result.steps[0].suggested_tool_call["args"]["left_on"],
        "user_id"
    );
    assert_eq!(
        result.steps[0].suggested_tool_call["args"]["right_on"],
        "user_id"
    );
}

#[test]
fn suggest_multi_table_plan_keeps_unresolved_tables_when_no_obvious_plan_exists() {
    let left = LoadedTable {
        // 2026-03-22: 这里构造一组既不能追加又不能显性关联的表，目的是锁定多表计划器会保守保留未决表，而不是乱排步骤。
        handle: TableHandle::new_confirmed(
            "memory://multi-plan-left",
            "Sheet1",
            vec!["customer_name".into(), "region".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("customer_name".into(), vec![Some("Alice"), Some("Bob")]).into(),
            Series::new("region".into(), vec![Some("East"), Some("West")]).into(),
        ])
        .unwrap(),
    };
    let right = LoadedTable {
        // 2026-03-22: 这里让右表只保留订单金额和渠道，目的是确认无明显动作时会原样留在 unresolved_refs。
        handle: TableHandle::new_confirmed(
            "memory://multi-plan-right",
            "Sheet1",
            vec!["order_amount".into(), "channel".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("order_amount".into(), vec![10_i64, 20]).into(),
            Series::new("channel".into(), vec![Some("Online"), Some("Store")]).into(),
        ])
        .unwrap(),
    };

    let result = suggest_multi_table_plan(
        vec![("left".to_string(), left), ("right".to_string(), right)],
        3,
    )
    .unwrap();

    // 2026-03-22: 这里锁定无明显计划时不会伪造步骤，目的是避免系统错误引导多表处理顺序。
    assert_eq!(result.steps.len(), 0);
    assert_eq!(result.unresolved_refs, vec!["left", "right"]);
    assert!(result.human_summary.contains("没有形成"));
}

#[test]
fn suggest_multi_table_plan_builds_append_then_join_chain_for_mixed_tables() {
    let customer_inference =
        infer_header_schema("tests/fixtures/join-customers.xlsx", "Customers").unwrap();
    let sales_a_inference =
        infer_header_schema("tests/fixtures/append-sales-a.xlsx", "Sales").unwrap();
    let sales_b_inference =
        infer_header_schema("tests/fixtures/append-sales-b.xlsx", "Sales").unwrap();
    let customers = load_confirmed_table(
        "tests/fixtures/join-customers.xlsx",
        "Customers",
        &customer_inference,
    )
    .unwrap();
    let sales_a = load_confirmed_table(
        "tests/fixtures/append-sales-a.xlsx",
        "Sales",
        &sales_a_inference,
    )
    .unwrap();
    let sales_b = load_confirmed_table(
        "tests/fixtures/append-sales-b.xlsx",
        "Sales",
        &sales_b_inference,
    )
    .unwrap();

    let result = suggest_multi_table_plan(
        vec![
            // 2026-03-22: 这里故意把主表放在前面、同结构明细表放在后面，目的是锁定计划器会先合并同结构批次数据，再把中间结果与主表显性关联。
            ("customers".to_string(), customers),
            ("sales_a".to_string(), sales_a),
            ("sales_b".to_string(), sales_b),
        ],
        3,
    )
    .unwrap();

    // 2026-03-22: 这里锁定混合场景会先追加，目的是防止计划器在可追加时直接跳去 join，导致批次数据没有先收口。
    assert_eq!(result.steps.len(), 2);
    assert_eq!(result.steps[0].action, "append_tables");
    assert_eq!(result.steps[0].input_refs, vec!["sales_a", "sales_b"]);
    assert_eq!(result.steps[0].result_ref, "step_1_result");
    assert!(result.steps[0].question.contains("追加"));
    // 2026-03-22: 这里锁定第二步会引用第一步中间结果去做显性关联，目的是确保多表计划真正具备链式编排能力，而不是只会独立给单步建议。
    assert_eq!(result.steps[1].action, "join_tables");
    assert_eq!(
        result.steps[1].input_refs,
        vec!["customers", "step_1_result"]
    );
    assert_eq!(result.steps[1].result_ref, "step_2_result");
    assert!(result.steps[1].question.contains("是否用"));
    assert_eq!(result.steps[1].suggested_tool_call["tool"], "join_tables");
    assert_eq!(
        result.steps[1].suggested_tool_call["args"]["left"]["path"],
        "tests/fixtures/join-customers.xlsx"
    );
    assert_eq!(
        result.steps[1].suggested_tool_call["args"]["right"]["result_ref"],
        "step_1_result"
    );
    assert_eq!(
        result.steps[1].suggested_tool_call["args"]["left_on"],
        "user_id"
    );
    assert_eq!(
        result.steps[1].suggested_tool_call["args"]["right_on"],
        "user_id"
    );
    assert_eq!(result.unresolved_refs, vec!["step_2_result"]);
}
