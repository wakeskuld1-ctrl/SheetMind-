use calamine::{Reader, open_workbook_auto};
use excel_skill::domain::handles::TableHandle;
use excel_skill::excel::header_inference::infer_header_schema;
use excel_skill::frame::chart_ref_store::{
    PersistedChartDraft, PersistedChartSeriesSpec, PersistedChartType,
};
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
use excel_skill::ops::chart_svg::render_chart_svg;
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
use excel_skill::ops::report_delivery::{
    ReportDeliveryRequest, ReportDeliverySection, build_report_delivery_draft,
};
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
fn render_line_chart_svg_contains_polyline() {
    let dataframe = DataFrame::new(vec![
        Series::new("month".into(), ["Jan", "Feb", "Mar"]).into(),
        Series::new("revenue".into(), [120_i64, 150_i64, 132_i64]).into(),
    ])
    .unwrap();
    let draft = PersistedChartDraft::from_dataframe_with_layout(
        "chart_line_svg",
        "build_chart",
        vec!["result_line_seed".to_string()],
        &dataframe,
        PersistedChartType::Line,
        Some("Monthly Revenue".to_string()),
        "month",
        Some("月份".to_string()),
        Some("收入".to_string()),
        true,
        840,
        480,
        vec![PersistedChartSeriesSpec {
            value_column: "revenue".to_string(),
            name: Some("Revenue".to_string()),
        }],
    )
    .unwrap();

    // 2026-03-23: 这里先锁定 line 图 SVG 至少包含折线主体，原因是后续 CLI 导出成功也不能只是空白画布；目的是给渲染层一个最小可视回归锚点。
    let svg = render_chart_svg(&draft).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("Monthly Revenue"));
    assert!(svg.contains("<polyline"));
}

#[test]
fn render_scatter_chart_svg_contains_points() {
    let dataframe = DataFrame::new(vec![
        Series::new("x_value".into(), [1_i64, 2_i64, 3_i64, 4_i64]).into(),
        Series::new("y_value".into(), [10_i64, 13_i64, 17_i64, 20_i64]).into(),
    ])
    .unwrap();
    let draft = PersistedChartDraft::from_dataframe_with_layout(
        "chart_scatter_svg",
        "build_chart",
        vec!["result_scatter_seed".to_string()],
        &dataframe,
        PersistedChartType::Scatter,
        Some("Value Scatter".to_string()),
        "x_value",
        Some("X".to_string()),
        Some("Y".to_string()),
        false,
        840,
        480,
        vec![PersistedChartSeriesSpec {
            value_column: "y_value".to_string(),
            name: Some("Y Value".to_string()),
        }],
    )
    .unwrap();

    // 2026-03-23: 这里先锁定 scatter 图 SVG 至少包含点元素，原因是散点图坐标映射和柱线图不同；目的是单独覆盖 X/Y 数值轴的渲染路径。
    let svg = render_chart_svg(&draft).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("Value Scatter"));
    assert!(svg.contains("<circle"));
}

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

    // 2026-03-22: 杩欓噷鍏堥攣瀹氭樉寮忓尯鍩熷姞杞戒細鍙秷璐?B3:D5锛岀洰鐨勬槸閬垮厤鎶婃暣寮?Sheet 鐨勭┖鐧借竟鐣岄噸鏂板甫鍥?DataFrame銆?
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

    // 2026-03-22: 杩欓噷鍏堥攣瀹氭樉寮忓灞傝〃澶翠細鎸?header_row_count 鍚堝苟璺緞锛岀洰鐨勬槸閬垮厤棣栫増鍖哄煙鍔犺浇鍙兘澶勭悊鍗曞眰琛ㄥご銆?
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

    // 2026-03-22: 这里锁定非法区域语法会直接报错，目的是让上层尽早修正输入，而不是静默加载错误区域。
    // 2026-03-23: 这里把历史乱码断言收口成稳定中文子串，原因是测试源码编码曾受损；目的是避免“区域格式无效”被误判为行为回归。
    assert!(error.to_string().contains("区域格式无效"));
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

    // 2026-03-22: 杩欓噷鍏堥攣瀹?region table_ref 澶嶇敤鏃朵粛鍙姞杞芥寚瀹氬眬閮ㄥ尯鍩燂紝鐩殑鏄伩鍏嶅洖鏀炬椂閫€鍖栧洖鏁村紶 Sheet銆?
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
        // 2026-03-23: 杩欓噷鏋勯€犲甫绌烘牸銆佸ぇ灏忓啓涓庡垎闅旂鍣煶鐨勬枃鏈垪锛岀洰鐨勬槸鍏堥攣瀹氭枃鏈爣鍑嗗寲 Tool 鐨勭湡瀹炴竻娲楅『搴忋€?
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

    // 2026-03-23: 杩欓噷閿佸畾澶氳鍒欏彔鍔犲悗鐨勬渶缁堟枃鏈粨鏋滐紝鐩殑鏄槻姝?trim / 鏇挎崲 / 澶у皬鍐欓『搴忔紓绉汇€?
    assert_eq!(preview.rows[0]["customer_code"], "a 001");
    assert_eq!(preview.rows[1]["customer_code"], "b002");
    assert_eq!(preview.rows[0]["region"], "EAST");
    assert_eq!(preview.rows[1]["region"], "WEST AREA");
}

#[test]
fn normalize_text_columns_rejects_duplicate_rules_for_same_column() {
    let loaded = LoadedTable {
        // 2026-03-23: 杩欓噷鏋勯€犳渶灏忔枃鏈垪锛岀洰鐨勬槸閿佸畾鍚屼竴鍒楅噸澶嶈鍒欐椂浼氫繚瀹堟姤閿欒€屼笉鏄伔鍋疯鐩栥€?
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

    // 2026-03-23: 杩欓噷閿佸畾閲嶅鍒楄鍒欐姤閿欙紝鐩殑鏄槻姝㈠弬鏁颁簩涔夋€ц闈欓粯鍚炴帀銆?
    assert!(error.to_string().contains("region"));
}

#[test]
fn rename_columns_renames_requested_columns_without_touching_rows() {
    let loaded = LoadedTable {
        // 2026-03-23: 杩欓噷鏋勯€犳渶灏忚〃锛岀洰鐨勬槸閿佸畾 rename 鍙敼 schema 涓嶆敼鏁版嵁鍐呭涓庤鏁般€?
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

    // 2026-03-23: 杩欓噷閿佸畾杈撳嚭鍒楀悕涓庡師濮嬭鍊硷紝鐩殑鏄‘淇?rename 涓嶄細鏀瑰潖琛ㄥ唴瀹广€?
    assert_eq!(renamed.handle.columns(), &["customer_id", "revenue"]);
    assert_eq!(preview.rows[0]["customer_id"], "1");
    assert_eq!(preview.rows[1]["revenue"], "95");
}

#[test]
fn rename_columns_rejects_conflicting_target_names() {
    let loaded = LoadedTable {
        // 2026-03-23: 杩欓噷鏋勯€犲凡鏈夊垪鍚嶅啿绐佸満鏅紝鐩殑鏄厛閿佸畾 rename 鐨勪繚瀹堟姤閿欒竟鐣屻€?
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

    // 2026-03-23: 杩欓噷閿佸畾鍐茬獊鍒楀悕鎶ラ敊锛岀洰鐨勬槸閬垮厤鏂板垪鍚嶈鐩栧凡鏈夊垪銆?
    assert!(error.to_string().contains("sales"));
}

#[test]
fn rename_columns_reports_missing_source_column() {
    let loaded = LoadedTable {
        // 2026-03-23: 杩欓噷鏋勯€犳渶灏忚〃锛岀洰鐨勬槸閿佸畾涓嶅瓨鍦ㄦ簮鍒楁椂鐨勬槑纭姤閿欍€?
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

    // 2026-03-23: 杩欓噷閿佸畾缂哄け婧愬垪鎶ラ敊锛岀洰鐨勬槸璁╀笂灞傚敖蹇慨姝ｅ瓧娈靛彛寰勩€?
    assert!(error.to_string().contains("sales"));
}

#[test]
fn fill_missing_values_supports_constant_zero_and_forward_fill_strategies() {
    let loaded = LoadedTable {
        // 2026-03-22: 杩欓噷鏋勯€犲悓鏃跺寘鍚?null銆佺┖涓插拰绾┖鐧界殑鏈€灏忚〃锛岀洰鐨勬槸閿佸畾閫氱敤琛ョ┖ Tool 鐨勭涓€鐗堟牳蹇冪己澶卞彛寰勩€?
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

    // 2026-03-22: 杩欓噷閿佸畾 constant / zero / forward_fill 涓夌绛栫暐鐨勬渶灏忕ǔ瀹氳涓猴紝鐩殑鏄厛琛ラ綈琛ㄥ鐞嗗眰鏈€甯哥敤鐨勮ˉ绌哄姩浣溿€?
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
        // 2026-03-22: 杩欓噷鏋勯€犳渶灏忓崟鍒楄〃锛岀洰鐨勬槸閿佸畾 constant 缂哄皯 value 鏃朵笉浼氶潤榛樺啓鍏ョ┖鍊笺€?
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

    // 2026-03-22: 杩欓噷閿佸畾 constant 蹇呴』鏄惧紡缁?value锛岀洰鐨勬槸淇濇寔琛ョ┖琛屼负鍙В閲婅€屼笉鏄粯璁や贡濉€?
    assert!(error.to_string().contains("city"));
}

#[test]
fn distinct_rows_supports_full_row_and_subset_deduplication() {
    let loaded = LoadedTable {
        // 2026-03-22: 杩欓噷鏋勯€犳棦鏈夋暣琛岄噸澶嶄篃鏈変富閿噸澶嶇殑鏈€灏忚〃锛岀洰鐨勬槸閿佸畾 distinct_rows 鐨勪袱绉嶆牳蹇冨幓閲嶅彛寰勩€?
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

    // 2026-03-22: 杩欓噷閿佸畾鏁磋鍘婚噸浼氬幓鎺夊畬鍏ㄩ噸澶嶈锛岃€屾寜瀛愰泦鍒楀幓閲嶆椂鑳芥寜 keep=last 淇濈暀鏈€鍚庝竴鏉¤褰曘€?
    assert_eq!(full_row_distinct.dataframe.height(), 3);
    assert_eq!(full_preview.rows[2]["sales"], "95");
    assert_eq!(subset_distinct.dataframe.height(), 2);
    assert_eq!(subset_preview.rows[0]["sales"], "100");
    assert_eq!(subset_preview.rows[1]["sales"], "95");
}

#[test]
fn distinct_rows_reports_missing_subset_column() {
    let loaded = LoadedTable {
        // 2026-03-22: 杩欓噷鏋勯€犳渶灏忚〃锛岀洰鐨勬槸閿佸畾瀛愰泦鍒椾笉瀛樺湪鏃朵笉浼氶潤榛橀€€鍖栨垚鏁磋〃鍘婚噸銆?
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

    // 2026-03-22: 杩欓噷閿佸畾缂哄け subset 鍒椾細鏄庣‘鎶ラ敊锛岀洰鐨勬槸淇濇寔鍘婚噸鍙ｅ緞瀹屽叏鏄惧紡銆?
    assert!(error.to_string().contains("sales"));
}

#[test]
fn deduplicate_by_key_keeps_first_record_without_order_by() {
    let loaded = LoadedTable {
        // 2026-03-22: 杩欓噷鏋勯€犲悓涓€涓氬姟閿噸澶嶅嚭鐜扮殑鏈€灏忚〃锛岀洰鐨勬槸閿佸畾 deduplicate_by_key 鍦ㄦ湭鎻愪緵鎺掑簭瑙勫垯鏃堕粯璁や繚鐣欓鏉¤褰曘€?
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

    // 2026-03-22: 杩欓噷閿佸畾鏈粰鎺掑簭瑙勫垯鏃剁殑榛樿琛屼负锛岀洰鐨勬槸璁┾€滄寜涓婚敭鍘婚噸鈥濆厛鍏峰鏈€淇濆畧銆佹渶濂借В閲婄殑绗竴鐗堣涔夈€?
    assert_eq!(deduplicated.dataframe.height(), 2);
    assert_eq!(preview.rows[0]["sales"], "100");
    assert_eq!(preview.rows[1]["sales"], "90");
}

#[test]
fn deduplicate_by_key_keeps_last_record_by_order_rule() {
    let loaded = LoadedTable {
        // 2026-03-22: 杩欓噷鏋勯€犲悓閿琛屼笖鏇存柊鏃堕棿閫掑鐨勬渶灏忚〃锛岀洰鐨勬槸閿佸畾 deduplicate_by_key 鑳芥寜鎺掑簭瑙勫垯淇濈暀鏈€鍚庝竴鏉℃湁鏁堣褰曘€?
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

    // 2026-03-22: 杩欓噷閿佸畾鍏堟帓搴忓啀淇濈暀鏈潯鐨勮涓猴紝鐩殑鏄涓氬姟鈥滀繚鐣欐渶鏂拌褰曗€濈殑鍘婚噸璇夋眰鍙互绋冲畾钀藉湪 Tool 灞傘€?
    assert_eq!(deduplicated.dataframe.height(), 2);
    assert_eq!(preview.rows[0]["updated_at"], "2026-03-03");
    assert_eq!(preview.rows[0]["score"], "85");
    assert_eq!(preview.rows[1]["updated_at"], "2026-03-04");
    assert_eq!(preview.rows[1]["score"], "88");
}

#[test]
fn deduplicate_by_key_reports_missing_key_column() {
    let loaded = LoadedTable {
        // 2026-03-22: 杩欓噷鏋勯€犳渶灏忚〃锛岀洰鐨勬槸閿佸畾 key 鍒楃己澶辨椂浼氭樉寮忔姤閿欒€屼笉鏄倓鎮勯€€鍖栨垚鍏跺畠鍘婚噸閫昏緫銆?
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

    // 2026-03-22: 杩欓噷閿佸畾閿欒浼氭寚鍑虹己澶辩殑閿垪锛岀洰鐨勬槸璁╀笂灞傚揩閫熶慨姝ｅ瓧娈靛彛寰勩€?
    assert!(error.to_string().contains("customer_id"));
}

#[test]
fn deduplicate_by_key_reports_missing_order_column() {
    let loaded = LoadedTable {
        // 2026-03-22: 杩欓噷鏋勯€犳渶灏忚〃锛岀洰鐨勬槸閿佸畾鎺掑簭鍒楃己澶辨椂涓嶄細缁х画鐢ㄤ笉瀹屾暣瑙勫垯鎵ц涓婚敭鍘婚噸銆?
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

    // 2026-03-22: 杩欓噷閿佸畾閿欒浼氭寚鍚戠己澶辩殑鎺掑簭鍒楋紝鐩殑鏄伩鍏嶁€滀繚鐣欐渶鏂?鏈€澶у€尖€濊繖绫昏涔夎闈欓粯鍋氶敊銆?
    assert!(error.to_string().contains("score"));
}

#[test]
fn format_table_for_export_reorders_and_renames_columns() {
    let loaded = LoadedTable {
        // 2026-03-22: 杩欓噷鏋勯€犳渶灏忓鍑鸿〃锛岀洰鐨勬槸閿佸畾瀵煎嚭鍓嶆暣鐞嗕細鍚屾椂鐢熸晥鍒楅『搴忎笌琛ㄥご鍒悕銆?
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
                    to: "鍖哄煙".to_string(),
                },
                RenameColumnMapping {
                    from: "sales".to_string(),
                    to: "閿€鍞".to_string(),
                },
                RenameColumnMapping {
                    from: "user_id".to_string(),
                    to: "瀹㈡埛ID".to_string(),
                },
            ],
            drop_unspecified_columns: false,
        },
    )
    .unwrap();
    let preview = preview_table(&formatted.dataframe, formatted.dataframe.height()).unwrap();

    // 2026-03-22: 杩欓噷閿佸畾瀵煎嚭鍓嶆暣鐞嗗悗鐨勬渶缁堝垪甯冨眬锛岀洰鐨勬槸璁╁悗缁?workbook 缁勮灞傛秷璐圭ǔ瀹氥€侀潰鍚戝鎴风殑琛ㄥご缁撴瀯銆?
    assert_eq!(
        formatted.handle.columns(),
        &["鍖哄煙", "閿€鍞", "瀹㈡埛ID"]
    );
    assert_eq!(preview.rows[0]["鍖哄煙"], "East");
    assert_eq!(preview.rows[0]["閿€鍞"], "120");
    assert_eq!(preview.rows[1]["瀹㈡埛ID"], "2");
}

#[test]
fn format_table_for_export_drops_unspecified_columns_when_requested() {
    let loaded = LoadedTable {
        // 2026-03-22: 杩欓噷鏋勯€犲寘鍚澶栨妧鏈垪鐨勬渶灏忚〃锛岀洰鐨勬槸閿佸畾 drop_unspecified_columns=true 鏃跺彧淇濈暀瀹㈡埛鍙鍒椼€?
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

    // 2026-03-22: 杩欓噷閿佸畾瀵煎嚭瑁佸壀璇箟锛岀洰鐨勬槸閬垮厤鍐呴儴涓棿鍒楄閿欒甯﹁繘瀹㈡埛浜や粯鎶ヨ〃銆?
    assert_eq!(formatted.handle.columns(), &["user_id", "sales"]);
    assert_eq!(formatted.dataframe.width(), 2);
}

#[test]
fn format_table_for_export_reports_missing_column_in_column_order() {
    let loaded = LoadedTable {
        // 2026-03-22: 杩欓噷鏋勯€犳渶灏忚〃锛岀洰鐨勬槸閿佸畾瀵煎嚭鍒楅『搴忛噷鍐欓敊瀛楁鏃朵細鏄庣‘鎶ラ敊銆?
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

    // 2026-03-22: 杩欓噷閿佸畾缂哄け鍒楅敊璇紝鐩殑鏄涓婂眰鍦ㄥ鍑哄墠灏变慨姝ｅ彛寰勮€屼笉鏄敓鎴愭畫缂烘姤琛ㄣ€?
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
                sheet_name: "姒傝".to_string(),
                source_refs: vec!["result_summary".to_string()],
                dataframe: DataFrame::new(vec![
                    Series::new("鍖哄煙".into(), vec![Some("East"), Some("West")]).into(),
                    Series::new("閿€鍞".into(), vec![Some("120"), Some("95")]).into(),
                ])
                .unwrap(),
                title: None,
                subtitle: None,
                data_start_row: 0,
            },
            WorkbookSheetInput {
                sheet_name: "鏄庣粏".to_string(),
                source_refs: vec!["result_detail".to_string()],
                dataframe: DataFrame::new(vec![
                    Series::new("瀹㈡埛ID".into(), vec![Some("1"), Some("2")]).into(),
                    Series::new("浜у搧".into(), vec![Some("A"), Some("B")]).into(),
                ])
                .unwrap(),
                title: None,
                subtitle: None,
                data_start_row: 0,
            },
        ],
    )
    .unwrap();

    store.save(&draft).unwrap();
    let reloaded = store.load(&workbook_ref).unwrap();
    let first_sheet = reloaded.worksheets[0].to_dataframe().unwrap();
    let second_sheet = reloaded.worksheets[1].to_dataframe().unwrap();

    // 2026-03-22: 杩欓噷閿佸畾 workbook 鑽夌浼氬畬鏁翠繚鐣欏 Sheet 蹇収锛岀洰鐨勬槸璁╁鍑哄姩浣滆劚绂诲師濮?Excel 涔熻兘绋冲畾鎵ц銆?
    assert_eq!(reloaded.worksheets.len(), 2);
    assert_eq!(reloaded.worksheets[0].sheet_name, "姒傝");
    assert_eq!(reloaded.worksheets[1].sheet_name, "鏄庣粏");
    assert_eq!(first_sheet.height(), 2);
    assert_eq!(first_sheet.get_column_names(), &["鍖哄煙", "閿€鍞"]);
    assert_eq!(second_sheet.get_column_names(), &["瀹㈡埛ID", "浜у搧"]);
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
                title: None,
                subtitle: None,
                data_start_row: 0,
            },
            WorkbookSheetInput {
                sheet_name: "Detail".to_string(),
                source_refs: vec!["result_detail".to_string()],
                dataframe: DataFrame::new(vec![
                    Series::new("user_id".into(), vec![Some("1"), Some("2")]).into(),
                    Series::new("product".into(), vec![Some("A"), Some("B")]).into(),
                ])
                .unwrap(),
                title: None,
                subtitle: None,
                data_start_row: 0,
            },
        ],
    )
    .unwrap();

    export_excel_workbook(&draft, output_path.to_str().unwrap()).unwrap();

    let mut workbook = open_workbook_auto(&output_path).unwrap();
    let summary = workbook.worksheet_range("Summary").unwrap();
    let detail = workbook.worksheet_range("Detail").unwrap();

    // 2026-03-22: 杩欓噷閿佸畾澶?Sheet 瀵煎嚭鍚庣殑宸ヤ綔绨垮彲浠ュ啀娆¤鏍囧噯 Excel 璇诲彇锛岀洰鐨勬槸淇濊瘉 compose -> export 閾捐矾鍏峰鐪熷疄浜や粯鑳藉姏銆?
    assert_eq!(summary.get((0, 0)).unwrap().to_string(), "region");
    assert_eq!(summary.get((1, 1)).unwrap().to_string(), "120");
    assert_eq!(detail.get((0, 0)).unwrap().to_string(), "user_id");
    assert_eq!(detail.get((2, 1)).unwrap().to_string(), "B");
}

#[test]
fn report_delivery_builds_standard_template_draft() {
    let workbook_ref = WorkbookDraftStore::create_workbook_ref();
    let draft = build_report_delivery_draft(
        &workbook_ref,
        ReportDeliveryRequest {
            report_name: "\u{7ecf}\u{8425}\u{5206}\u{6790}\u{6c47}\u{62a5}".to_string(),
            report_subtitle: None,
            summary: ReportDeliverySection {
                sheet_name: "\u{6458}\u{8981}\u{9875}".to_string(),
                source_refs: vec!["result_summary".to_string()],
                dataframe: DataFrame::new(vec![
                    Series::new(
                        "\u{6307}\u{6807}".into(),
                        vec![
                            Some("\u{603b}\u{5ba2}\u{6237}\u{6570}"),
                            Some("\u{603b}\u{6536}\u{5165}"),
                        ],
                    )
                    .into(),
                    Series::new("\u{503c}".into(), vec![Some("2"), Some("215")]).into(),
                ])
                .unwrap(),
            },
            analysis: ReportDeliverySection {
                sheet_name: "\u{5206}\u{6790}\u{7ed3}\u{679c}\u{9875}".to_string(),
                source_refs: vec!["result_analysis".to_string()],
                dataframe: DataFrame::new(vec![
                    Series::new("user_id".into(), vec![Some("1"), Some("2")]).into(),
                    Series::new("sales".into(), vec![Some("120"), Some("95")]).into(),
                ])
                .unwrap(),
            },
            include_chart_sheet: true,
            chart_sheet_name: "\u{56fe}\u{8868}\u{9875}".to_string(),
            charts: vec![],
        },
    )
    .unwrap();

    let summary_sheet = draft.worksheets[0].to_dataframe().unwrap();
    let analysis_sheet = draft.worksheets[1].to_dataframe().unwrap();
    let chart_sheet = draft.worksheets[2].to_dataframe().unwrap();

    assert_eq!(draft.worksheets.len(), 3);
    assert_eq!(draft.worksheets[0].sheet_name, "\u{6458}\u{8981}\u{9875}");
    assert_eq!(
        draft.worksheets[1].sheet_name,
        "\u{5206}\u{6790}\u{7ed3}\u{679c}\u{9875}"
    );
    assert_eq!(draft.worksheets[2].sheet_name, "\u{56fe}\u{8868}\u{9875}");
    assert_eq!(
        summary_sheet.get_column_names(),
        &["\u{6307}\u{6807}", "\u{503c}"]
    );
    assert_eq!(analysis_sheet.get_column_names(), &["user_id", "sales"]);
    assert_eq!(
        chart_sheet.get_column_names(),
        &["\u{6a21}\u{5757}", "\u{72b6}\u{6001}", "\u{8bf4}\u{660e}"]
    );
    assert_eq!(draft.charts.len(), 0);
}

#[test]
fn report_delivery_can_build_template_without_chart_sheet() {
    let workbook_ref = WorkbookDraftStore::create_workbook_ref();
    let draft = build_report_delivery_draft(
        &workbook_ref,
        ReportDeliveryRequest {
            report_name: "\u{7ecf}\u{8425}\u{5206}\u{6790}\u{6c47}\u{62a5}".to_string(),
            report_subtitle: None,
            summary: ReportDeliverySection {
                sheet_name: "\u{6458}\u{8981}\u{9875}".to_string(),
                source_refs: vec!["result_summary".to_string()],
                dataframe: DataFrame::new(vec![
                    Series::new(
                        "\u{6307}\u{6807}".into(),
                        vec![Some("\u{603b}\u{5ba2}\u{6237}\u{6570}")],
                    )
                    .into(),
                    Series::new("\u{503c}".into(), vec![Some("2")]).into(),
                ])
                .unwrap(),
            },
            analysis: ReportDeliverySection {
                sheet_name: "\u{5206}\u{6790}\u{7ed3}\u{679c}\u{9875}".to_string(),
                source_refs: vec!["result_analysis".to_string()],
                dataframe: DataFrame::new(vec![
                    Series::new("user_id".into(), vec![Some("1"), Some("2")]).into(),
                    Series::new("sales".into(), vec![Some("120"), Some("95")]).into(),
                ])
                .unwrap(),
            },
            include_chart_sheet: false,
            chart_sheet_name: "\u{56fe}\u{8868}\u{9875}".to_string(),
            charts: vec![],
        },
    )
    .unwrap();

    assert_eq!(draft.worksheets.len(), 2);
    assert_eq!(draft.worksheets[0].sheet_name, "\u{6458}\u{8981}\u{9875}");
    assert_eq!(
        draft.worksheets[1].sheet_name,
        "\u{5206}\u{6790}\u{7ed3}\u{679c}\u{9875}"
    );
    assert_eq!(draft.charts.len(), 0);
}

#[test]
fn report_delivery_builds_chart_specs_for_analysis_sheet() {
    let workbook_ref = WorkbookDraftStore::create_workbook_ref();
    let draft = build_report_delivery_draft(
        &workbook_ref,
        ReportDeliveryRequest {
            report_name: "\u{533a}\u{57df}\u{6536}\u{5165}\u{6c47}\u{62a5}".to_string(),
            report_subtitle: None,
            summary: ReportDeliverySection {
                sheet_name: "\u{6458}\u{8981}\u{9875}".to_string(),
                source_refs: vec!["result_summary".to_string()],
                dataframe: DataFrame::new(vec![
                    Series::new(
                        "\u{6307}\u{6807}".into(),
                        vec![Some("\u{603b}\u{5ba2}\u{6237}\u{6570}")],
                    )
                    .into(),
                    Series::new("\u{503c}".into(), vec![Some("3")]).into(),
                ])
                .unwrap(),
            },
            analysis: ReportDeliverySection {
                sheet_name: "\u{5206}\u{6790}\u{7ed3}\u{679c}\u{9875}".to_string(),
                source_refs: vec!["result_analysis".to_string()],
                dataframe: DataFrame::new(vec![
                    Series::new("region".into(), vec![Some("North"), Some("South")]).into(),
                    Series::new("sales".into(), vec![Some("120"), Some("95")]).into(),
                ])
                .unwrap(),
            },
            include_chart_sheet: true,
            chart_sheet_name: "\u{56fe}\u{8868}\u{9875}".to_string(),
            charts: vec![excel_skill::ops::report_delivery::ReportDeliveryChart {
                chart_ref: None,
                source_refs: vec![],
                chart_type: excel_skill::ops::report_delivery::ReportDeliveryChartType::Column,
                title: Some("\u{533a}\u{57df}\u{6536}\u{5165}\u{67f1}\u{72b6}\u{56fe}".to_string()),
                category_column: "region".to_string(),
                value_column: "sales".to_string(),
                series: vec![],
                show_legend: false,
                legend_position: None,
                chart_style: None,
                x_axis_name: None,
                y_axis_name: None,
                anchor_row: Some(1),
                anchor_col: Some(0),
            }],
        },
    )
    .unwrap();

    assert_eq!(draft.charts.len(), 1);
    assert_eq!(
        draft.charts[0].target_sheet_name,
        "\u{56fe}\u{8868}\u{9875}"
    );
    assert_eq!(
        draft.charts[0].data_sheet_name,
        "\u{5206}\u{6790}\u{7ed3}\u{679c}\u{9875}"
    );
    assert_eq!(draft.charts[0].category_column, "region");
    assert_eq!(draft.charts[0].value_column, "sales");
    assert_eq!(
        draft.charts[0].title.as_deref(),
        Some("\u{533a}\u{57df}\u{6536}\u{5165}\u{67f1}\u{72b6}\u{56fe}")
    );
}

#[test]
fn report_delivery_builds_multi_series_chart_specs() {
    let workbook_ref = WorkbookDraftStore::create_workbook_ref();
    let draft = build_report_delivery_draft(
        &workbook_ref,
        ReportDeliveryRequest {
            report_name: "\u{6708}\u{5ea6}\u{7ecf}\u{8425}\u{6c47}\u{62a5}".to_string(),
            report_subtitle: None,
            summary: ReportDeliverySection {
                sheet_name: "\u{6458}\u{8981}\u{9875}".to_string(),
                source_refs: vec!["result_summary".to_string()],
                dataframe: DataFrame::new(vec![
                    Series::new(
                        "\u{6307}\u{6807}".into(),
                        vec![Some("\u{603b}\u{5ba2}\u{6237}\u{6570}")],
                    )
                    .into(),
                    Series::new("\u{503c}".into(), vec![Some("3")]).into(),
                ])
                .unwrap(),
            },
            analysis: ReportDeliverySection {
                sheet_name: "\u{5206}\u{6790}\u{7ed3}\u{679c}\u{9875}".to_string(),
                source_refs: vec!["result_analysis".to_string()],
                dataframe: DataFrame::new(vec![
                    Series::new("month".into(), vec![Some("1\u{6708}"), Some("2\u{6708}")]).into(),
                    Series::new("revenue".into(), vec![Some("120"), Some("95")]).into(),
                    Series::new("profit".into(), vec![Some("35"), Some("28")]).into(),
                ])
                .unwrap(),
            },
            include_chart_sheet: true,
            chart_sheet_name: "\u{56fe}\u{8868}\u{9875}".to_string(),
            charts: vec![excel_skill::ops::report_delivery::ReportDeliveryChart {
                chart_ref: None,
                source_refs: vec![],
                chart_type: excel_skill::ops::report_delivery::ReportDeliveryChartType::Column,
                title: Some(
                    "\u{8425}\u{6536}\u{4e0e}\u{5229}\u{6da6}\u{67f1}\u{72b6}\u{56fe}".to_string(),
                ),
                category_column: "month".to_string(),
                value_column: String::new(),
                series: vec![
                    excel_skill::ops::report_delivery::ReportDeliveryChartSeries {
                        value_column: "revenue".to_string(),
                        name: Some("\u{8425}\u{6536}".to_string()),
                    },
                    excel_skill::ops::report_delivery::ReportDeliveryChartSeries {
                        value_column: "profit".to_string(),
                        name: Some("\u{5229}\u{6da6}".to_string()),
                    },
                ],
                show_legend: false,
                legend_position: None,
                chart_style: None,
                x_axis_name: None,
                y_axis_name: None,
                anchor_row: None,
                anchor_col: None,
            }],
        },
    )
    .unwrap();

    // 2026-03-23: 杩欓噷鍏堥攣瀹?report_delivery 浼氭妸鍗曞浘澶氱郴鍒楀浐鍖栬繘鑽夌锛屽師鍥犳槸瀵煎嚭闃舵鍙簲娑堣垂绋冲畾鍏冩暟鎹紱鐩殑鏄伩鍏嶅绯诲垪閫昏緫鏁ｈ惤鍦ㄥ鍑哄眰涓存椂鎷艰銆?    assert_eq!(draft.charts.len(), 1);
    assert_eq!(draft.charts[0].series.len(), 2);
    assert_eq!(draft.charts[0].series[0].value_column, "revenue");
    assert_eq!(draft.charts[0].series[1].value_column, "profit");
    assert_eq!(
        draft.charts[0].series[1].name.as_deref(),
        Some("\u{5229}\u{6da6}")
    );
}

#[test]
fn report_delivery_auto_layouts_multiple_charts_into_grid() {
    let workbook_ref = WorkbookDraftStore::create_workbook_ref();
    let draft = build_report_delivery_draft(
        &workbook_ref,
        ReportDeliveryRequest {
            report_name: "\u{6708}\u{5ea6}\u{7ecf}\u{8425}\u{6c47}\u{62a5}".to_string(),
            report_subtitle: None,
            summary: ReportDeliverySection {
                sheet_name: "\u{6458}\u{8981}\u{9875}".to_string(),
                source_refs: vec!["result_summary".to_string()],
                dataframe: DataFrame::new(vec![
                    Series::new(
                        "\u{6307}\u{6807}".into(),
                        vec![Some("\u{603b}\u{5ba2}\u{6237}\u{6570}")],
                    )
                    .into(),
                    Series::new("\u{503c}".into(), vec![Some("3")]).into(),
                ])
                .unwrap(),
            },
            analysis: ReportDeliverySection {
                sheet_name: "\u{5206}\u{6790}\u{7ed3}\u{679c}\u{9875}".to_string(),
                source_refs: vec!["result_analysis".to_string()],
                dataframe: DataFrame::new(vec![
                    Series::new("month".into(), vec![Some("1\u{6708}"), Some("2\u{6708}")]).into(),
                    Series::new("revenue".into(), vec![Some("120"), Some("95")]).into(),
                    Series::new("profit".into(), vec![Some("35"), Some("28")]).into(),
                ])
                .unwrap(),
            },
            include_chart_sheet: true,
            chart_sheet_name: "\u{56fe}\u{8868}\u{9875}".to_string(),
            charts: vec![
                excel_skill::ops::report_delivery::ReportDeliveryChart {
                    chart_ref: None,
                    source_refs: vec![],
                    chart_type: excel_skill::ops::report_delivery::ReportDeliveryChartType::Column,
                    title: Some("\u{8425}\u{6536}\u{67f1}\u{72b6}\u{56fe}".to_string()),
                    category_column: "month".to_string(),
                    value_column: "revenue".to_string(),
                    series: vec![],
                    show_legend: false,
                    legend_position: None,
                    chart_style: None,
                    x_axis_name: None,
                    y_axis_name: None,
                    anchor_row: None,
                    anchor_col: None,
                },
                excel_skill::ops::report_delivery::ReportDeliveryChart {
                    chart_ref: None,
                    source_refs: vec![],
                    chart_type: excel_skill::ops::report_delivery::ReportDeliveryChartType::Line,
                    title: Some("\u{5229}\u{6da6}\u{6298}\u{7ebf}\u{56fe}".to_string()),
                    category_column: "month".to_string(),
                    value_column: "profit".to_string(),
                    series: vec![],
                    show_legend: false,
                    legend_position: None,
                    chart_style: None,
                    x_axis_name: None,
                    y_axis_name: None,
                    anchor_row: None,
                    anchor_col: None,
                },
            ],
        },
    )
    .unwrap();

    // 2026-03-23: 杩欓噷鍏堥攣瀹氬鍥句細鑷姩閾哄紑鍒颁笉鍚岄敋鐐癸紝鍘熷洜鏄粨鏋滀氦浠樺眰涓嶈兘缁х画鎶婂寮犲浘鍏ㄥ爢鍦ㄥ乏涓婅锛涚洰鐨勬槸缁欏悗缁鍑哄眰鎻愪緵绋冲畾甯冨眬杈撳叆銆?    assert_eq!(draft.charts.len(), 2);
    assert_eq!(draft.charts[0].anchor_row, 1);
    assert_eq!(draft.charts[0].anchor_col, 0);
    assert_eq!(draft.charts[1].anchor_row, 1);
    assert_eq!(draft.charts[1].anchor_col, 8);
}

#[test]
fn fill_missing_from_lookup_only_fills_blank_or_null_values() {
    let base = LoadedTable {
        // 2026-03-23: 杩欓噷鏋勯€犲悓鏃跺惈 null銆佺┖瀛楃涓插拰闈炵┖鍊肩殑涓昏〃锛岀洰鐨勬槸閿佸畾鈥滃彧琛ョ┖涓嶈鐩栤€濈殑鏍稿績杈圭晫銆?
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
        // 2026-03-23: 杩欓噷鏋勯€犱竴寮犲敮涓€閿煡鍊艰〃锛岀洰鐨勬槸楠岃瘉澶氬瓧娈佃ˉ鍊兼椂鐨勭ǔ瀹氭煡鎵捐涓恒€?
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

    // 2026-03-23: 杩欓噷閿佸畾 null / 绌哄瓧绗︿覆浼氳琛ラ綈锛岃€屽凡鏈夊€间笉浼氳瑕嗙洊锛岀洰鐨勬槸閬垮厤 lookup 鎶婄敤鎴峰師濮嬫暟鎹埛鎺夈€?
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
        // 2026-03-23: 杩欓噷鏋勯€犳渶灏忎富琛紝鐩殑鏄崟鐙攣瀹?lookup 閿笉鍞竴鏃剁殑淇濆畧鎶ラ敊銆?
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
        // 2026-03-23: 杩欓噷鏁呮剰璁?key 閲嶅锛岀洰鐨勬槸楠岃瘉绗竴鐗?fill lookup 涓嶄細鍦ㄥ鍛戒腑鏃惰嚜浣滀富寮犻€変竴鏉°€?
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

    // 2026-03-23: 杩欓噷閿佸畾閲嶅 key 鎶ラ敊锛岀洰鐨勬槸瑕佹眰涓婂眰鍏堟樉寮忓幓閲嶅啀琛ュ€笺€?
    assert!(error.to_string().contains("user_id"));
}

#[test]
fn fill_missing_from_lookup_keeps_original_value_when_lookup_key_missing() {
    let base = LoadedTable {
        // 2026-03-23: 杩欓噷鏋勯€犱竴鏉℃煡涓嶅埌 lookup 鐨勪富琛ㄨ褰曪紝鐩殑鏄攣瀹氭湭鍛戒腑鏃朵繚鎸佸師鍊间笉鎶ラ敊銆?
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
        // 2026-03-23: 杩欓噷鏁呮剰涓嶇粰鍛戒腑 key锛岀洰鐨勬槸楠岃瘉鍥炲～閫昏緫涓嶄細鎹忛€犱笉瀛樺湪鐨勫€笺€?
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

    // 2026-03-23: 杩欓噷閿佸畾 lookup 鏈懡涓椂淇濈暀鍘熷€硷紝鐩殑鏄繚鎸佲€滃彧琛ュ緱鍒扮殑鍊尖€濈殑淇濆畧琛屼负銆?
    assert_eq!(preview.rows[0]["city"], "");
}

#[test]
fn fill_missing_from_lookup_by_composite_keys_fills_matching_rows_only() {
    let base = LoadedTable {
        // 2026-03-23: 杩欓噷鏋勯€犫€滃鎴?+ 鏈堜唤鈥濆鍚堥敭涓昏〃锛岀洰鐨勬槸鍏堥攣瀹氬洖濉兘鍔涗笉鍐嶅彧鏀寔鍗曢敭銆?
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
        // 2026-03-23: 杩欓噷鏋勯€犲鍚堥敭 lookup 琛紝鐩殑鏄攣瀹氬彧鏈夐敭瀹屽叏涓€鑷存椂鎵嶄細鍥炲～瀵瑰簲鏈堜唤鍊笺€?
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

    // 2026-03-23: 杩欓噷閿佸畾澶嶅悎閿洖濉懡涓殑鏄€滃悓瀹㈡埛鍚屾湀浠解€濓紝鐩殑鏄伩鍏嶄笉鍚屾湀浠芥暟鎹覆鍊笺€?
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
        // 2026-03-23: 杩欓噷鏋勯€犳渶灏忎富琛紝鐩殑鏄崟鐙攣瀹氬鍚堥敭鍒楁暟涓嶄竴鑷存椂鐨勬槑纭姤閿欍€?
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
        // 2026-03-23: 杩欓噷鏋勯€犳渶灏?lookup 琛紝鐩殑鏄澶辫触鍘熷洜鍙惤鍦ㄥ鍚堥敭瑙勬牸涓娿€?
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

    // 2026-03-23: 这里锁定复合键列数不一致会直接报错，目的是避免用户误以为系统会自动猜测缺失的键列。
    // 2026-03-23: 这里改成稳定中文断言，原因是原断言文本已乱码；目的是继续校验真正的错误语义而不是编码噪声。
    assert!(error.to_string().contains("键列数量不一致"));
}

#[test]
fn pivot_table_builds_sum_wide_table_by_row_and_column_dimensions() {
    let loaded = LoadedTable {
        // 2026-03-23: 杩欓噷鏋勯€犳渶灏忚鍒楅€忚鍦烘櫙锛岀洰鐨勬槸閿佸畾 pivot 鐨勫琛ㄨ緭鍑虹粨鏋勪笌 sum 鑱氬悎缁撴灉銆?
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

    // 2026-03-23: 杩欓噷閿佸畾杈撳嚭鍒椾笌閫忚缁撴灉锛岀洰鐨勬槸纭繚 V1 瀹借〃缁撴瀯绋冲畾鍙鍑恒€?
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
        // 2026-03-23: 杩欓噷鏋勯€犻潪鏁板€?value 鍒楋紝鐩殑鏄攣瀹?sum / mean 鑱氬悎鐨勭被鍨嬮棬绂併€?
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

    // 2026-03-23: 杩欓噷閿佸畾闈炴暟鍊艰仛鍚堟姤閿欙紝鐩殑鏄伩鍏嶆妸鏂囨湰鍒楄褰撴暟鍊煎幓绠椼€?
    assert!(error.to_string().contains("label"));
}

#[test]
fn pivot_table_builds_mean_values_for_repeated_cells() {
    let loaded = LoadedTable {
        // 2026-03-23: 杩欓噷鏋勯€犲悓涓€閫忚鍗曞厓鏍煎琛屽懡涓満鏅紝鐩殑鏄攣瀹?mean 鑱氬悎涓嶆槸鏈€鍚庝竴鏉¤鐩栥€?
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

    // 2026-03-23: 杩欓噷閿佸畾 East-Jan 浼氬彇鍧囧€?90锛岀洰鐨勬槸楠岃瘉 mean 鑱氬悎绱姞鍣ㄦ纭伐浣溿€?
    assert_eq!(preview.rows[0]["region"], "East");
    assert_eq!(preview.rows[0]["Jan"], "90");
    assert_eq!(preview.rows[0]["Feb"], "60");
}

#[test]
fn parse_datetime_columns_normalizes_date_and_datetime_strings() {
    let loaded = LoadedTable {
        // 2026-03-23: 杩欓噷鏋勯€犲父瑙佹棩鏈熶笌鏃ユ湡鏃堕棿鏂囨湰锛岀洰鐨勬槸閿佸畾鏍囧噯鍖栧悗杈撳嚭涓虹粺涓€ ISO 鍙ｅ緞銆?
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

    // 2026-03-23: 杩欓噷閿佸畾鏍囧噯鏃ユ湡涓庢棩鏈熸椂闂存牸寮忥紝鐩殑鏄鏃堕棿绫诲垎鏋愮殑涓婃父鍙ｅ緞鍏堢ǔ瀹氫笅鏉ャ€?
    assert_eq!(preview.rows[0]["biz_date"], "2026-03-01");
    assert_eq!(preview.rows[1]["biz_date"], "2026-03-02");
    assert_eq!(preview.rows[0]["created_at"], "2026-03-01 08:30:00");
    assert_eq!(preview.rows[1]["created_at"], "2026-03-02 09:15:20");
}

#[test]
fn parse_datetime_columns_rejects_invalid_non_empty_values() {
    let loaded = LoadedTable {
        // 2026-03-23: 杩欓噷鏋勯€犻潪娉曟棩鏈熸枃鏈紝鐩殑鏄厛閿佸畾闈炵┖浣嗕笉鍙В鏋愭椂蹇呴』鎶ラ敊鑰屼笉鏄倓鎮勪繚鐣欒剰鍊笺€?
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

    // 2026-03-23: 杩欓噷閿佸畾闈炴硶鍊兼姤閿欙紝鐩殑鏄伩鍏嶆椂闂村瓧娈靛湪杩涘叆鍚庣画鍒嗘瀽鍓嶇户缁甫鑴忔暟鎹祦杞€?
    assert!(error.to_string().contains("biz_date"));
}

#[test]
fn parse_datetime_columns_rejects_invalid_calendar_dates() {
    let loaded = LoadedTable {
        // 2026-03-22: 杩欓噷鏋勯€犫€滄湀浠藉悎娉曚絾鏃ュ巻涓嶅瓨鍦ㄢ€濈殑鏃ユ湡锛岀洰鐨勬槸閿佸畾 parse_datetime_columns 浼氭嫤浣?2 鏈?30 鏃ヨ繖绫诲亣鏃ユ湡銆?
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
        // 2026-03-22: 杩欓噷鏋勯€?Excel 鍘熺敓鏃ユ湡搴忓垪鍊硷紝鐩殑鏄攣瀹?parse_datetime_columns 鑳界洿鎺ュ悆 1900 绯荤粺搴忓垪鍊艰€屼笉鏄彧璁ゆ枃鏈€?
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
        // 2026-03-23: 杩欓噷鏋勯€犱富琛紝鐩殑鏄攣瀹氳交閲忔煡鍊煎彧甯﹀洖鍒椼€佷笉鏀瑰彉涓昏〃琛屾暟涓庨『搴忋€?
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
        // 2026-03-23: 杩欓噷鏋勯€犲敮涓€閿煡鍊艰〃锛岀洰鐨勬槸楠岃瘉 VLOOKUP/XLOOKUP 蹇冩櫤涓嬬殑甯﹀垪琛屼负銆?
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

    // 2026-03-23: 杩欓噷閿佸畾鏌ュ€煎悗鍙柊澧炶緭鍑哄垪锛岀洰鐨勬槸閬垮厤 lookup_values 閫€鍖栨垚 join 璇箟銆?
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
        // 2026-03-23: 杩欓噷鏋勯€犲甫鏈懡涓?key 鐨勪富琛紝鐩殑鏄攣瀹氭煡涓嶅埌鏃惰緭鍑轰负绌鸿€屼笉鏄姤閿欐垨鎹忛€犲€笺€?
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
        // 2026-03-23: 杩欓噷鏁呮剰鍙粰閮ㄥ垎 key锛岀洰鐨勬槸楠岃瘉杞婚噺鏌ュ€肩殑鏈懡涓繚瀹堣涓恒€?
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

    // 2026-03-23: 杩欓噷閿佸畾鏈懡涓椂杈撳嚭鍒椾负绌猴紝鐩殑鏄涓婂眰鑳界户缁喅瀹氭槸鍚﹀洖濉粯璁ゅ€笺€?
    assert_eq!(preview.rows[0]["city"], "Beijing");
    assert_eq!(preview.rows[1]["city"], "");
}

#[test]
fn lookup_values_rejects_duplicate_lookup_keys() {
    let base = LoadedTable {
        // 2026-03-23: 杩欓噷鏋勯€犳渶灏忎富琛紝鐩殑鏄崟鐙攣瀹?lookup key 涓嶅敮涓€鏃剁殑鎶ラ敊杈圭晫銆?
        handle: TableHandle::new_confirmed(
            "memory://lookup-dup-base",
            "Base",
            vec!["user_id".into()],
        ),
        dataframe: DataFrame::new(vec![Series::new("user_id".into(), vec![Some("1")]).into()])
            .unwrap(),
    };
    let lookup = LoadedTable {
        // 2026-03-23: 杩欓噷鏁呮剰璁╂煡鍊艰〃 key 閲嶅锛岀洰鐨勬槸閬垮厤绯荤粺鍦ㄥ鍛戒腑鏃剁鑷寫涓€鏉°€?
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

    // 2026-03-23: 杩欓噷閿佸畾閲嶅 key 鎶ラ敊锛岀洰鐨勬槸瑕佹眰涓婂眰鍏堟樉寮忓幓閲嶅啀鏌ュ€笺€?
    assert!(error.to_string().contains("user_id"));
}

#[test]
fn lookup_values_rejects_conflicting_output_column() {
    let base = LoadedTable {
        // 2026-03-23: 杩欓噷鏋勯€犱富琛ㄥ凡鏈?city 鍒楋紝鐩殑鏄攣瀹氳緭鍑哄垪涓庡師鍒楀啿绐佹椂蹇呴』鏄惧紡閬胯銆?
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
        // 2026-03-23: 杩欓噷鏋勯€犳煡鍊艰〃 city 鍒楋紝鐩殑鏄獙璇?output_column 涓庝富琛ㄥ垪鍚嶅啿绐佺殑淇濇姢閫昏緫銆?
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

    // 2026-03-23: 杩欓噷閿佸畾杈撳嚭鍒楀啿绐佹姤閿欙紝鐩殑鏄伩鍏?lookup_values 鍦ㄩ潤榛樿鐩栦富琛ㄥ瓧娈垫椂寮曞叆鑴忕粨鏋溿€?
    assert!(error.to_string().contains("city"));
}

#[test]
fn lookup_values_by_composite_keys_appends_selected_columns() {
    let base = LoadedTable {
        // 2026-03-23: 杩欓噷鏋勯€犫€滃鎴?+ 鏈堜唤鈥濆鍚堥敭涓昏〃锛岀洰鐨勬槸鍏堥攣瀹氳交閲忔煡鍊兼敮鎸佸鍚堥敭甯﹀垪銆?
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
        // 2026-03-23: 杩欓噷鏋勯€犲鍚堥敭鏌ュ€艰〃锛岀洰鐨勬槸楠岃瘉 lookup_values 涓嶄細璺ㄦ湀浠戒覆甯﹀煄甯傚拰鍒嗗眰銆?
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

    // 2026-03-23: 杩欓噷閿佸畾澶嶅悎閿煡鍊肩粨鏋滄寜瀹屾暣閿懡涓紝鐩殑鏄鈥滃鎴稩D + 鏈堜唤鈥濊繖绉嶇湡瀹炵粡钀ュ垎鏋愬彛寰勭ǔ瀹氬彲鐢ㄣ€?
    assert_eq!(preview.rows[0]["city"], "Beijing");
    assert_eq!(preview.rows[1]["city"], "Shanghai");
    assert_eq!(preview.rows[2]["city"], "Shenzhen");
    assert_eq!(preview.rows[1]["tier"], "B");
}

#[test]
fn lookup_values_rejects_mismatched_composite_key_lengths() {
    let base = LoadedTable {
        // 2026-03-23: 杩欓噷鏋勯€犳渶灏忎富琛紝鐩殑鏄崟鐙攣瀹氬鍚堥敭鍒楁暟涓嶄竴鑷存椂鐨勬槑纭姤閿欍€?
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
        // 2026-03-23: 杩欓噷鏋勯€犳渶灏?lookup 琛紝鐩殑鏄澶辫触鍘熷洜鍙惤鍦ㄥ鍚堥敭瑙勬牸杈圭晫銆?
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

    // 2026-03-23: 这里锁定复合键列数不一致会直接报错，目的是避免用户把单键和复合键配置混用。
    // 2026-03-23: 这里改成稳定中文断言，原因是原断言文本已乱码；目的是继续校验真正的错误语义而不是编码噪声。
    assert!(error.to_string().contains("键列数量不一致"));
}

#[test]
fn window_calculation_builds_row_number_and_dense_rank_by_partition() {
    let loaded = LoadedTable {
        // 2026-03-23: 杩欓噷鏋勯€犲垎缁勫唴鎺掑簭鍦烘櫙锛岀洰鐨勬槸閿佸畾 row_number 鍜?dense rank 鐨勭涓€鐗堢獥鍙ｈ涓恒€?
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

    // 2026-03-23: 杩欓噷閿佸畾绐楀彛鍒椾細鎸夊師琛ㄨ鍥炲～锛岀洰鐨勬槸璁╃敤鎴锋棤闇€閲嶆柊鐞嗚В缁撴灉琛岄『搴忋€?
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
        // 2026-03-23: 杩欓噷鏁呮剰鎵撲贡鍘熻〃椤哄簭锛岀洰鐨勬槸閿佸畾绱鍜屾寜鎸囧畾鎺掑簭璁＄畻浣嗕粛鍥炲～鍒板師琛屻€?
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

    // 2026-03-23: 杩欓噷閿佸畾绱鍜屾寜鎺掑簭搴忓垪鎺ㄨ繘锛岀洰鐨勬槸涓哄悗缁秼鍔垮垎鏋愬拰缁忚惀绱鎸囨爣鎻愪緵绋冲畾搴曞骇銆?
    assert_eq!(preview.rows[0]["running_amount"], "180");
    assert_eq!(preview.rows[1]["running_amount"], "100");
    assert_eq!(preview.rows[2]["running_amount"], "240");
    assert_eq!(preview.rows[3]["running_amount"], "90");
}

#[test]
fn window_calculation_rejects_non_numeric_cumulative_sum_source() {
    let loaded = LoadedTable {
        // 2026-03-23: 杩欓噷鏋勯€犳枃鏈瀷 source 鍒楋紝鐩殑鏄攣瀹?cumulative_sum 鐨勬暟鍊肩被鍨嬮棬绂併€?
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

    // 2026-03-23: 杩欓噷閿佸畾闈炴暟鍊肩疮璁℃姤閿欙紝鐩殑鏄伩鍏嶆妸鏂囨湰鍒楄绠楁垚缁忚惀鎸囨爣銆?
    assert!(error.to_string().contains("label"));
}

#[test]
fn window_calculation_supports_shift_percent_rank_and_rolling_metrics() {
    let loaded = LoadedTable {
        // 2026-03-23: 杩欓噷鏋勯€犱贡搴忔棩鏈熷垎鍖烘暟鎹紝鐩殑鏄厛閿佸畾 lag/lead銆乸ercent_rank銆乺olling_sum銆乺olling_mean 鐨勭粍鍚堣涓恒€?
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

    // 2026-03-23: 杩欓噷閿佸畾绐楀彛缁撴灉浼氭寜鍘熷琛屽洖濉紝鐩殑鏄闂瓟寮忚皟鐢ㄤ笉闇€瑕侀噸鏂拌В閲婅浣嶇疆銆?
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
    // 2026-03-21: 杩欓噷鍏堝彇鍑哄簳灞?Series锛岀洰鐨勬槸鍏煎褰撳墠 Polars 鏂扮増 Column API 鐨勮鍙栨柟寮忋€?
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

    // 2026-03-21: 杩欓噷鏍￠獙 sales 鍒楀凡缁忓畬鎴愭暟鍊煎寲杞崲锛岀洰鐨勬槸纭繚鍚庣画鑱氬悎鍜屾帓搴忛兘鍩轰簬鐪熷疄鏁板€笺€?
    let sales_series = casted
        .dataframe
        .column("sales")
        .unwrap()
        .as_materialized_series();
    assert_eq!(sales_series.dtype(), &DataType::Int64);
    // 2026-03-21: 杩欓噷缁х画鏍￠獙杞崲鍚庣殑灞曠ず鍊硷紝鐩殑鏄‘璁?cast 鍚庣殑缁撴灉浠嶈兘绋冲畾杈撳嚭缁欎笂灞傞瑙堛€?
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
    // 2026-03-21: 杩欓噷鏍￠獙鍒嗙粍鑱氬悎鍚庣殑棣栬缁撴灉锛岀洰鐨勬槸纭繚鍒嗙粍鍒楁帓搴忓拰鑱氬悎鍊艰緭鍑洪兘绋冲畾鍙娴嬨€?
    assert_eq!(preview.rows[0]["region"], "East");
    assert_eq!(preview.rows[0]["sales_sum"], "200");
    assert_eq!(preview.rows[0]["sales_count"], "2");
    // 2026-03-21: 杩欓噷鏍￠獙绗簩涓垎缁勭粨鏋滐紝鐩殑鏄‘淇濆缁勫満鏅笅涓嶄細鍑虹幇鑱氬悎涓蹭綅銆?
    assert_eq!(preview.rows[1]["region"], "West");
    assert_eq!(preview.rows[1]["sales_sum"], "150");
    assert_eq!(preview.rows[1]["sales_count"], "2");
}

#[test]
fn derive_columns_supports_condition_groups_date_bucket_and_template() {
    let loaded = LoadedTable {
        // 2026-03-23: 杩欓噷鏋勯€犵粡钀ュ垎鏋愬父瑙佷腑闂磋〃锛岀洰鐨勬槸閿佸畾 derive_columns 澧炲己鍚庡彲鐩存帴浜у嚭鏍囩銆佹椂娈靛拰璇存槑鍒椼€?
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
            Series::new(
                "customer_id".into(),
                vec![Some("C001"), Some("C002"), Some("C003")],
            )
            .into(),
            Series::new("sales".into(), vec![120_i64, 95_i64, 60_i64]).into(),
            Series::new("visits".into(), vec![3_i64, 5_i64, 1_i64]).into(),
            Series::new(
                "biz_date".into(),
                vec![Some("2026-01-15"), Some("2026-04-10"), Some("2026-08-01")],
            )
            .into(),
            Series::new(
                "region".into(),
                vec![Some("East"), Some("West"), Some("North")],
            )
            .into(),
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

    // 2026-03-23: 杩欓噷閿佸畾 all/any銆佹棩鏈熷垎娈典笌妯℃澘鍒椾細涓€璧风敓鏁堬紝鐩殑鏄‘淇濈粡钀ュ垎鏋愭ˉ鎺ュ眰鑳界洿鎺ユ嫾鍑哄彲瑙ｉ噴璇存槑銆?
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
    // 2026-03-21: 杩欓噷鏍￠獙澶氬垪鎺掑簭鍚庣殑棣栬锛岀洰鐨勬槸纭繚涓绘帓搴忛敭鍜屾鎺掑簭閿兘鎸夐鏈熺敓鏁堛€?
    assert_eq!(preview.rows[0]["region"], "East");
    assert_eq!(preview.rows[0]["sales"], "120");
    // 2026-03-21: 杩欓噷鏍￠獙绗簩琛岋紝鐩殑鏄‘璁ゅ悓缁勫唴鎸?sales 闄嶅簭绋冲畾鎺掑垪銆?
    assert_eq!(preview.rows[1]["region"], "East");
    assert_eq!(preview.rows[1]["sales"], "80");
    // 2026-03-21: 杩欓噷鏍￠獙璺ㄧ粍鍒囨崲鍚庣殑椤哄簭锛岀洰鐨勬槸纭绗簩鎺掑簭缁勪笉浼氫覆浣嶃€?
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
    // 2026-03-21: 杩欓噷鏍￠獙 top_n 鐨勯琛岋紝鐩殑鏄‘淇濆厛鎺掑簭鍚庢埅鍙栨椂浼氫繚鐣欐渶澶?sales 璁板綍銆?
    assert_eq!(preview.rows[0]["sales"], "120");
    // 2026-03-21: 杩欓噷鏍￠獙绗簩琛岋紝鐩殑鏄‘淇濆彧鎴彇鎺掑簭鍚庣殑鍓嶄袱鏉★紝鑰屼笉鏄師濮嬭緭鍏ュ墠涓ゆ潯銆?
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
    // 2026-03-21: 杩欓噷鏍￠獙鍖归厤淇濈暀妯″紡鐨勯琛岋紝鐩殑鏄‘淇濆悓涓€ user_id 鑳藉睍寮€鎴愬鏉¤鍗曡褰曘€?
    assert_eq!(preview.rows[0]["user_id"], "1");
    assert_eq!(preview.rows[0]["name"], "Alice");
    assert_eq!(preview.rows[0]["order_id"], "101");
    // 2026-03-21: 杩欓噷鏍￠獙鏈€鍚庝竴琛岋紝鐩殑鏄‘淇濆彧淇濈暀涓よ竟閮借兘鍏宠仈鎴愬姛鐨勮褰曘€?
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
    // 2026-03-21: 杩欓噷鏍￠獙宸︿繚鐣欐ā寮忎笅鐨勭┖鍖归厤琛岋紝鐩殑鏄‘淇濆乏琛ㄧ敤鎴?3 涓嶄細鍥犱负鍙宠〃缂哄崟鑰屼涪澶便€?
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
    // 2026-03-21: 杩欓噷鏍￠獙 keep_right 浼氫繚鐣欏彸琛ㄧ嫭鏈夎褰曪紝鐩殑鏄‘璁ゆ湭鍖归厤璁㈠崟涓嶄細琚涓㈠純銆?
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

    // 2026-03-21: 杩欓噷閿佸畾绌洪敭涓嶅弬涓庢樉鎬у叧鑱旓紝鐩殑鏄伩鍏嶆湭濉?ID 鐨勮剰鏁版嵁鍦?matched_only 涓嬭鎷煎埌涓€璧枫€?
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

    // 2026-03-21: 杩欓噷閿佸畾澶氬澶氬睍寮€琛屾暟锛岀洰鐨勬槸纭鍚屼竴 key 鐨勫乏鍙冲鏉¤褰曚細鍋氱瑳鍗″皵灞曞紑鑰屼笉鏄瑕嗙洊銆?
    assert_eq!(joined.dataframe.height(), 5);
    // 2026-03-21: 杩欓噷閿佸畾杩炵画鍐茬獊鍒楅噸鍛藉悕锛岀洰鐨勬槸纭繚鍙宠〃鍚屽悕鍒椾笉浼氳鐩栧乏琛ㄥ凡鏈夊垪锛屼篃涓嶄細鍑虹幇閲嶅鍒楀悕銆?
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
        // 2026-03-23: 杩欓噷鏋勯€犳暣鏁颁富閿乏琛紝鐩殑鏄厛閿佸畾 join_tables 鍦ㄤ笉鎵嬪伐 casts 鏃朵篃鑳藉悆鏁板€煎瀷閿€?
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
        // 2026-03-23: 杩欓噷鏋勯€犳诞鐐逛富閿彸琛紝鐩殑鏄鐜扳€?鈥濆拰鈥?.0鈥濇暟鍊肩瓑浠蜂絾瀛楃涓插睍绀轰笉鍚屽鑷寸殑鍏宠仈涓嶇ǔ闂銆?
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

    let joined = join_tables(
        &left,
        &right,
        "user_id",
        "user_id",
        JoinKeepMode::MatchedOnly,
    )
    .unwrap();
    let preview = preview_table(&joined.dataframe, joined.dataframe.height()).unwrap();

    // 2026-03-23: 杩欓噷閿佸畾鏁存暟閿笌娴偣閿細鎸夊悓涓€鏁板€艰涔夊尮閰嶏紝鐩殑鏄噺灏戞樉鎬у叧鑱斿墠杩樿棰濆 casts 鐨勮礋鎷呫€?
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
    // 2026-03-21: 杩欓噷鏍￠獙绾靛悜杩藉姞鍚庣殑鍓嶄袱琛岋紝鐩殑鏄‘淇濆師濮嬩笂鍗婇儴鍒嗘暟鎹『搴忎笉浼氳鎵撲贡銆?
    assert_eq!(preview.rows[0]["user_id"], "1");
    assert_eq!(preview.rows[1]["user_id"], "2");
    // 2026-03-21: 杩欓噷鏍￠獙杩藉姞鍚庣殑鍚庝袱琛岋紝鐩殑鏄‘淇濅笅鍗婇儴鍒嗘暟鎹湡姝ｆ嫾鎺ュ埌浜嗙粨鏋滃熬閮ㄣ€?
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
    // 2026-03-21: 杩欓噷鏍￠獙缁撴灉鍒楅『搴忎粛浠ヤ笂琛ㄤ负鍑嗭紝鐩殑鏄伩鍏嶄笅琛ㄥ垪椤哄簭涓嶅悓瀵艰嚧缁撴灉 schema 婕傜Щ銆?
    assert_eq!(preview.columns, vec!["user_id", "region", "sales"]);
    // 2026-03-21: 杩欓噷鏍￠獙閲嶆帓鍒楅『搴忓悗鐨勭涓夎锛岀洰鐨勬槸纭繚 user_id/region/sales 鏄寜鍒楀悕鑰屼笉鏄寜浣嶇疆鎷兼帴銆?
    assert_eq!(preview.rows[2]["user_id"], "3");
    assert_eq!(preview.rows[2]["region"], "North");
    assert_eq!(preview.rows[2]["sales"], "90");
    // 2026-03-21: 杩欓噷鏍￠獙绗洓琛岋紝鐩殑鏄‘淇濇暣寮犱笅琛ㄩ兘鎸夊垪鍚嶅榻愬悗杩藉姞鎴愬姛銆?
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
        Ok(_) => panic!("append_tables 在列结构不一致时不应成功"),
        Err(error) => error,
    };

    // 2026-03-21: 杩欓噷鏍￠獙鎸夊垪鍚嶅榻愭ā寮忎粛浼氭嫆缁濈己鍒?寮傛瀯琛紝鐩殑鏄伩鍏嶆妸涓嶅悓缁撴瀯鏁版嵁璇嫾鎺ヨ繘鍚屼竴寮犺〃銆?
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

    // 2026-03-21: 杩欓噷鏍￠獙鏂囨湰鍒楁憳瑕佺粨鏋勶紝鐩殑鏄‘淇濈鏁ｅ瓧娈佃兘绋冲畾杈撳嚭绫诲埆鍒嗗竷鍜岃川閲忔寚鏍囥€?
    assert_eq!(region_summary.summary_kind, "string");
    assert_eq!(region_summary.distinct_count, Some(2));
    // 2026-03-21: 杩欓噷琛ョ己澶辩巼鏂█锛岀洰鐨勬槸璁╃粺璁℃憳瑕佺洿鎺ョ粰涓婂眰闂瓟鎻愪緵鏇寸洿瑙傜殑鏁版嵁璐ㄩ噺鎸囨爣銆?
    assert_eq!(region_summary.missing_rate, Some(0.0));
    assert_eq!(region_summary.top_values[0].value, "East");
    assert_eq!(region_summary.top_values[0].count, 1);
    // 2026-03-21: 杩欓噷鏍￠獙鏁板€煎垪鎽樿缁撴瀯锛岀洰鐨勬槸纭繚寤烘ā鍓嶆渶甯哥敤缁熻閲忎繚鎸佺ǔ瀹氳緭鍑恒€?
    assert_eq!(sales_summary.summary_kind, "numeric");
    assert_eq!(sales_summary.count, 2);
    // 2026-03-21: 杩欓噷琛ユ暟鍊煎垪缂哄け鐜囨柇瑷€锛岀洰鐨勬槸纭鏁板€兼憳瑕佸拰鏂囨湰鎽樿娌跨敤鍚屼竴濂楃己澶辩巼璇箟銆?
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
        // 2026-03-21: 杩欓噷鏋勯€犲叏绌哄垪鍦烘櫙锛岀洰鐨勬槸妯℃嫙 Excel 鐪熷疄涓氬姟閲屾暣鍒楁湭濉啓鐨勮緭鍏ャ€?
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

    // 2026-03-21: 杩欓噷鏍￠獙鍏ㄧ┖鍒椾笉浼氬穿婧冿紝鐩殑鏄繚璇?count/null_count 涓庣┖鍒嗗竷閮藉彲绋冲畾杩斿洖銆?
    assert_eq!(notes_summary.count, 0);
    assert_eq!(notes_summary.null_count, 2);
    // 2026-03-21: 杩欓噷琛ュ叏绌哄垪缂哄け鐜囨柇瑷€锛岀洰鐨勬槸纭繚鈥滃叏缂哄け鈥濆満鏅兘杩斿洖 100% 缂哄け鐜囪€屼笉鏄穿婧冦€?
    assert_eq!(notes_summary.missing_rate, Some(1.0));
    assert_eq!(notes_summary.top_values.len(), 0);
    // 2026-03-21: 杩欓噷椤哄甫鏍￠獙甯冨皵鍒楃敾鍍忥紝鐩殑鏄‘璁?summary Tool 鑳藉悓鏃惰鐩栧绉嶅瓧娈电被鍨嬨€?
    assert_eq!(active_summary.summary_kind, "boolean");
    // 2026-03-21: 杩欓噷琛ュ竷灏斿垪缂哄け鐜囨柇瑷€锛岀洰鐨勬槸纭甯冨皵鐢诲儚涔熻兘鍚戜笂灞傜ǔ瀹氭毚闇茶川閲忔寚鏍囥€?
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
        // 2026-03-21: 杩欓噷鏋勯€犵┖鐧藉瓧绗︿覆涓庣函绌烘牸锛岀洰鐨勬槸妯℃嫙 Excel 閲屽父瑙佺殑鈥滅湅浼兼湁鍊笺€佸疄闄呮病濉€濄€?
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

    // 2026-03-21: 杩欓噷鏍￠獙绌虹櫧浼氬苟鍏ョ己澶辩粺璁★紝鐩殑鏄 Excel 鐢ㄦ埛鐪嬪埌鐨勬湁鏁堝€兼暟閲忔洿绗﹀悎鐩磋銆?
    assert_eq!(notes_summary.count, 1);
    assert_eq!(notes_summary.null_count, 3);
    // 2026-03-21: 杩欓噷琛ョ┖鐧藉崰缂哄け鐜囨柇瑷€锛岀洰鐨勬槸闃叉鍚庣画鏀瑰姩鍙洿鏂拌鏁拌€岄仐婕忔瘮渚嬨€?
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
        // 2026-03-21: 杩欓噷鏋勯€犱笟鍔″崰浣嶇己澶卞€硷紝鐩殑鏄鐩?Excel 涓?N/A銆丯A銆乶ull 涓€绫诲父瑙佸啓娉曘€?
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

    // 2026-03-21: 杩欓噷鏍￠獙鍗犱綅鍊间笉浼氳璇涓烘湁鏁堟枃鏈紝鐩殑鏄伩鍏嶆憳瑕侀珮浼板彲鐢ㄦ暟鎹噺銆?
    assert_eq!(notes_summary.count, 1);
    assert_eq!(notes_summary.null_count, 4);
    // 2026-03-21: 杩欓噷琛ュ崰浣嶇己澶辩巼鏂█锛岀洰鐨勬槸璁?N/A/NA/null 涓€绫诲€肩殑缂哄け缁熻鏇村畬鏁淬€?
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
        // 2026-03-21: 杩欓噷鐩存帴鏋勯€犳棩鏈熷垪涓庤剰鏁版嵁鍒楋紝鐩殑鏄湪涓嶄緷璧?Excel 瑙ｆ瀽缁嗚妭鏃跺厛閿佸畾鎽樿鏍稿績璇箟銆?
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

    // 2026-03-21: 杩欓噷閿佸畾鏃ユ湡鏂囨湰鍒楃敾鍍忥紝鐩殑鏄‘璁?V1 鍦ㄤ笉寮曞叆棰濆鏃ユ湡鐗规€х殑鍓嶆彁涓嬩篃鑳界ǔ瀹氭憳瑕併€?
    assert_eq!(date_summary.dtype, "string");
    assert_eq!(date_summary.summary_kind, "string");
    assert_eq!(date_summary.count, 3);
    assert_eq!(date_summary.null_count, 2);
    assert_eq!(date_summary.missing_rate, Some(0.4));
    assert_eq!(date_summary.distinct_count, Some(3));
    // 2026-03-21: 杩欓噷閿佸畾娣峰悎鑴忔暟鎹垪鐨勬憳瑕侊紝鐩殑鏄‘璁ゅ崰浣嶅€煎拰绌虹櫧鍊间粛浼氬苟鍏ュ彛寰勪竴鑷寸殑缂哄け缁熻銆?
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
        // 2026-03-21: 杩欓噷鐩存帴鏋勯€犱綆淇℃伅閲忓垪鍜屽叏绌哄垪锛岀洰鐨勬槸鍏堥攣瀹?analyze_table 鐨勭涓€鎵硅鍒欒緭鍑恒€?
        handle: TableHandle::new_confirmed(
            "memory://analyze-basic",
            "Sheet1",
            vec!["region".into(), "notes".into()],
        ),
        dataframe,
    };

    let result = analyze_table(&loaded, &[], 3);

    // 2026-03-21: 杩欓噷瑕佹眰 analyze_table 鑷冲皯鑳戒粠鎽樿鐢诲儚鐢熸垚鍩虹 finding锛岀洰鐨勬槸鎶?bridge Tool 浠庡崰浣嶇姸鎬佹帹杩涘埌鐪熸璇婃柇銆?
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
        // 2026-03-21: 杩欓噷鏋勯€犻珮缂哄け鍒椼€佸叏绌哄垪鍜屽崟涓€鍙栧€煎垪锛岀洰鐨勬槸閿佸畾 analyze_table 鐨勭涓€鎵硅川閲忚瘖鏂鍒欍€?
        handle: TableHandle::new_confirmed(
            "memory://analyze-quality",
            "Sheet1",
            vec!["status".into(), "phone".into(), "notes".into()],
        ),
        dataframe,
    };

    let result = analyze_table(&loaded, &[], 3);

    // 2026-03-21: 杩欓噷瑕佹眰楂樼己澶卞垪琚瘑鍒嚭鏉ワ紝鐩殑鏄鍚庣画闂瓟鍜屽缓妯￠兘鑳戒紭鍏堣仛鐒﹂珮椋庨櫓瀛楁銆?
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
        // 2026-03-21: 杩欓噷鏋勯€犻噸澶嶈銆侀噸澶?ID 鍜岀┖ ID锛岀洰鐨勬槸閿佸畾鍒嗘瀽寤烘ā鍓嶆渶鍏抽敭鐨勯敭璐ㄩ噺椋庨櫓璇婃柇銆?
        handle: TableHandle::new_confirmed(
            "memory://analyze-keys",
            "Sheet1",
            vec!["user_id".into(), "region".into(), "amount".into()],
        ),
        dataframe,
    };

    let result = analyze_table(&loaded, &[], 3);

    // 2026-03-21: 杩欓噷瑕佹眰鏁磋閲嶅鑳借璇嗗埆鍑烘潵锛岀洰鐨勬槸閬垮厤鍚庣画鑱氬悎銆佸缓妯″拰鍏宠仈鏃堕噸澶嶆斁澶ф牱鏈€?
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
        // 2026-03-21: 杩欓噷鏋勯€犵被鍒け琛°€侀浂鍊煎崰姣旈珮鍜屽紓甯稿€煎垪锛岀洰鐨勬槸閿佸畾 bridge Tool 鐨勮交閲忕粺璁″寮鸿兘鍔涖€?
        handle: TableHandle::new_confirmed(
            "memory://analyze-distribution",
            "Sheet1",
            vec!["region".into(), "zero_metric".into(), "amount".into()],
        ),
        dataframe,
    };

    let result = analyze_table(&loaded, &[], 3);

    // 2026-03-21: 杩欓噷瑕佹眰绫诲埆楂樺害闆嗕腑鑳借璇嗗埆鍑烘潵锛岀洰鐨勬槸鎻愰啋鐢ㄦ埛鍏堝叧娉ㄥけ琛″垎甯冦€?
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
        // 2026-03-21: 杩欓噷鏋勯€犱富绫诲埆鍜屾暟鍊艰寖鍥撮兘寰堢洿瑙傜殑灏忚〃锛岀洰鐨勬槸鍏堥攣瀹氱嫭绔?business_observations 濂戠害銆?
        handle: TableHandle::new_confirmed(
            "memory://analyze-business-observations",
            "Sheet1",
            vec!["region".into(), "amount".into()],
        ),
        dataframe,
    };

    let result = analyze_table(&loaded, &[], 3);

    // 2026-03-21: 杩欓噷瑕佹眰绫诲埆涓诲垎甯冭兘杩涘叆涓氬姟瑙傚療锛岀洰鐨勬槸璁╀笂灞傚湪璐ㄩ噺璇婃柇涔嬪杩樿兘鎷垮埌灏戦噺鍙涓氬姟鎻愮ず銆?
    assert!(
        result
            .business_observations
            .iter()
            .any(|observation| observation.observation_type == "top_category"
                && observation.column.as_deref() == Some("region"))
    );
    // 2026-03-21: 杩欓噷瑕佹眰鏁板€艰寖鍥翠篃杩涘叆涓氬姟瑙傚療锛岀洰鐨勬槸缁欏垎鏋愬缓妯″眰涓€涓交閲忕粺璁℃ˉ鎺ヨ緭鍑恒€?
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
        // 2026-03-21: 杩欓噷鏋勯€犱細鍚屾椂瑙﹀彂澶氭潯鍚屽垪 finding 鐨勫満鏅紝鐩殑鏄攣瀹氭帓搴忕ǔ瀹氭€у拰鎽樿鍘嬬缉閫昏緫銆?
        handle: TableHandle::new_confirmed(
            "memory://analyze-priority",
            "Sheet1",
            vec!["phone".into(), "notes".into()],
        ),
        dataframe,
    };

    let result = analyze_table(&loaded, &[], 5);

    // 2026-03-21: 杩欓噷閿佸畾楂樹紭鍏堢骇鐨勫叏绌哄垪 finding 浼氭帓鍦ㄥ墠闈紝鐩殑鏄涓婂眰浼樺厛鐪嬪埌鐪熸闃诲鍒嗘瀽鐨勯棶棰樸€?
    assert_eq!(result.structured_findings[0].code, "all_missing");
    assert_eq!(
        result.structured_findings[0].column.as_deref(),
        Some("notes")
    );
    // 2026-03-21: 杩欓噷閿佸畾鍚屼竴鍒楃殑缂哄け椋庨櫓浼氬厛浜庝綆淇℃伅閲忛闄╋紝鐩殑鏄噺灏戠粨鏋滈槄璇绘椂鐨勫櫔闊炽€?
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
    // 2026-03-21: 杩欓噷閿佸畾鎽樿涓婚棶棰樹細鍘嬬缉鍚屽垪閲嶅鎻愮ず锛岀洰鐨勬槸閬垮厤鐢ㄦ埛鐪嬪埌 phone 杩炵画閲嶅鎶ラ敊銆?
    assert_eq!(
        result
            .human_summary
            .major_issues
            .iter()
            .filter(|message| message.contains("phone"))
            .count(),
        1
    );
    // 2026-03-21: 杩欓噷閿佸畾 notes 涓嶄細鍐嶈璇瘑鍒垚鍊欓€夐敭锛岀洰鐨勬槸淇帀 contains(\"no\") 甯︽潵鐨勫亣闃虫€с€?
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
        // 2026-03-21: 杩欓噷鏋勯€犱富缁村害楂樺害闆嗕腑鐨勫皬琛紝鐩殑鏄攣瀹氭墿灞曞悗鐨勪笟鍔¤瀵熺被鍨嬨€?
        handle: TableHandle::new_confirmed(
            "memory://analyze-extended-business-observations",
            "Sheet1",
            vec!["region".into(), "amount".into()],
        ),
        dataframe,
    };

    let result = analyze_table(&loaded, &[], 5);

    // 2026-03-21: 杩欓噷瑕佹眰 dominant_dimension 鐙珛杈撳嚭锛岀洰鐨勬槸璁╂ˉ鎺ュ眰鑳界粰鍑衡€滀富鍒嗗竷缁村害鈥濇彁绀鸿€屼笉娣峰叆璐ㄩ噺 finding銆?
    assert!(
        result
            .business_observations
            .iter()
            .any(
                |observation| observation.observation_type == "dominant_dimension"
                    && observation.column.as_deref() == Some("region")
            )
    );
    // 2026-03-21: 杩欓噷瑕佹眰 numeric_center 鐙珛杈撳嚭锛岀洰鐨勬槸璁╁垎鏋愬缓妯″墠鍏堟嬁鍒颁竴灞傝交閲忎腑蹇冪粺璁¤瀵熴€?
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
        // 2026-03-21: 杩欓噷鏋勯€犱腑鏂囧拰绱у噾鍛藉悕鐨勯敭鍒楋紝鐩殑鏄攣瀹氬€欓€夐敭璇嗗埆澧炲己涓嶄細鍐嶅彧渚濊禆鑻辨枃 token銆?
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
    // 2026-03-23: 这里把历史乱码列名恢复为真实 UTF-8 中文，原因是候选键识别依赖列名语义；目的是避免测试输入本身把能力误伤成回归。
    assert!(
        result
            .structured_findings
            .iter()
            .any(|finding| finding.code == "duplicate_candidate_key"
                && finding.column.as_deref() == Some("客户编号"))
    );
    // 2026-03-21: 杩欓噷閿佸畾 uid 浼氳璇嗗埆鎴愬€欓€夐敭锛岀洰鐨勬槸瑕嗙洊甯歌缂╁啓鍛藉悕銆?
    assert!(
        result
            .structured_findings
            .iter()
            .any(|finding| finding.code == "blank_candidate_key"
                && finding.column.as_deref() == Some("uid"))
    );
    // 2026-03-21: 杩欓噷閿佸畾 userid 涔熶細琚瘑鍒紝鐩殑鏄鐩栨棤鍒嗛殧绗︾殑绱у噾鑻辨枃鍛藉悕銆?
    assert!(
        result
            .structured_findings
            .iter()
            .any(|finding| finding.code == "duplicate_candidate_key"
                && finding.column.as_deref() == Some("userid"))
    );
    // 2026-03-21: 杩欓噷缁х画閿佸畾 notice 涓嶄細璇姤锛岀洰鐨勬槸闃叉鏀惧瑙勫垯鍚庨噸鏂板紩鍏ュ亣闃虫€с€?
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
        // 2026-03-21: 杩欓噷鏋勯€犳槑鏄惧亸鎬佷笖鍚瀬绔€肩殑鏁板€煎垪锛岀洰鐨勬槸閿佸畾涓績缁熻浼氫紭鍏堢敤涓綅鏁拌€屼笉鏄潎鍊笺€?
        handle: TableHandle::new_confirmed(
            "memory://analyze-skewed-center",
            "Sheet1",
            vec!["amount".into()],
        ),
        dataframe,
    };

    let result = analyze_table(&loaded, &[], 5);

    // 2026-03-21: 杩欓噷閿佸畾鍋忔€佸垪鐨勪腑蹇冭瀵熶細鍒囧埌 median_center锛岀洰鐨勬槸璁╀笟鍔℃彁绀烘洿绋冲仴銆?
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
    // 2026-03-21: 杩欓噷閿佸畾鍚屼竴鍒椾笉鍐嶈緭鍑?numeric_center锛岀洰鐨勬槸閬垮厤鍧囧€煎拰涓綅鏁板悓鏃跺嚭鐜伴€犳垚鐞嗚В娣蜂贡銆?
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
        // 2026-03-21: 杩欓噷鍚屾椂鏋勯€犳暟鍊笺€佺被鍒拰甯冨皵鍒楋紝鐩殑鏄厛閿佸畾缁熻妗ユ帴 Tool 鐨勪笁绫诲熀纭€杈撳嚭缁撴瀯銆?
        handle: TableHandle::new_confirmed(
            "memory://stat-summary-basic",
            "Sheet1",
            vec!["sales".into(), "region".into(), "is_active".into()],
        ),
        dataframe,
    };

    let result = stat_summary(&loaded, &[], 3).unwrap();

    // 2026-03-21: 杩欓噷閿佸畾琛ㄧ骇姒傝锛岀洰鐨勬槸璁╁悗缁缓妯?Tool 鑳藉厛鍒ゆ柇褰撳墠琛ㄧ殑瀛楁绫诲瀷鍒嗗竷銆?
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
    // 2026-03-21: 杩欓噷閿佸畾鏁板€煎垪鐨勫垎浣嶆暟鍜岄浂鍊煎崰姣旓紝鐩殑鏄粰鍚庣画鍥炲綊鍜屽紓甯稿€煎垽鏂彁渚涚ǔ瀹氭ˉ鎺ョ粺璁￠噺銆?
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
    // 2026-03-21: 杩欓噷閿佸畾绫诲埆鍒楃殑涓诲€煎崰姣旓紝鐩殑鏄鑱氱被鍓嶅拰涓氬姟闂瓟閮借兘蹇€熺悊瑙ｅ垎甯冮泦涓害銆?
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
    // 2026-03-21: 杩欓噷閿佸畾甯冨皵鍒?true_ratio锛岀洰鐨勬槸璁╁垎绫诲缓妯″墠鑳藉揩閫熺湅鍒版爣绛惧垎甯冦€?
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

    // 2026-03-21: 杩欓噷閿佸畾楂橀浂鍊煎垪鐨?zero_ratio锛岀洰鐨勬槸璁╁缓妯″墠妫€鏌ヨ兘鐩存帴璇嗗埆鈥滈粯璁ゅ€艰繃澶氣€濈殑鏁板€煎瓧娈点€?
    assert_eq!(zero_metric_summary.zero_ratio, Some(0.8));
    assert_eq!(zero_metric_summary.median, Some(0.0));
    // 2026-03-21: 杩欓噷閿佸畾鍋忔€佸垪浼氱ǔ瀹氳繑鍥炰腑浣嶆暟涓庡洓鍒嗕綅鏁帮紝鐩殑鏄槻姝㈡瀬绔€兼妸涓績缁熻瑙ｉ噴甯﹀亸銆?
    assert_eq!(amount_summary.median, Some(2.0));
    assert_eq!(amount_summary.q1, Some(2.0));
    assert_eq!(amount_summary.q3, Some(3.0));
    // 2026-03-21: 杩欓噷閿佸畾绫诲埆涓诲€煎崰姣旓紝鐩殑鏄鍚庣画鑱氱被鍓嶅拰涓氬姟闂瓟閮借兘鐪嬫噦鍒嗗竷闆嗕腑搴︺€?
    assert_eq!(region_summary.top_values[0].value, "East");
    assert_eq!(region_summary.top_share, Some(0.8));
    // 2026-03-21: 这里锁定人类摘要会带出长尾和主区域提示，目的是让终端直接展示也有业务可读性。
    // 2026-03-23: 这里把历史乱码断言恢复为稳定 UTF-8 文本，原因是原断言只是在比对错误字节；目的是继续校验业务摘要语义。
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
        // 2026-03-23: 这里把历史乱码列名恢复为真实 UTF-8 中文，原因是日期/时间/金额识别依赖列名启发式；目的是避免测试输入本身改变行为。
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
    // 2026-03-21: 杩欓噷閿佸畾鏃ユ湡闆嗕腑瑙傚療锛岀洰鐨勬槸璁╃敤鎴风洿鎺ョ湅鍒拌褰曟槸鍚︿富瑕侀泦涓湪鏌愪釜鏈堜唤銆?
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
    // 2026-03-21: 杩欓噷閿佸畾鏃堕棿楂樺嘲瑙傚療锛岀洰鐨勬槸璁╅棶绛旂晫闈㈠彲浠ョ洿鐧芥彁绀轰富瑕佸彂鐢熸椂娈点€?
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
    // 2026-03-21: 杩欓噷閿佸畾閲戦鍏稿瀷鍖洪棿瑙傚療锛岀洰鐨勬槸璁╃敤鎴风湅鍒扳€滃父瑙侀噾棰濆甫鈥濊€屼笉鏄彧鐪嬫渶灏忔渶澶у€笺€?
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
    // 2026-03-21: 杩欓噷閿佸畾璐熼噾棰濊瀵燂紝鐩殑鏄彁閱掗€€娆俱€佸啿閿€鎴栧噣棰濈被鍦烘櫙鐨勭壒娈婂惈涔夈€?
    assert!(
        result
            .business_observations
            .iter()
            .any(
                |observation| observation.observation_type == "amount_negative_presence"
                    && observation.column.as_deref() == Some("实付金额")
            )
    );
    // 2026-03-21: 杩欓噷閿佸畾閲戦闀垮熬瑙傚療锛岀洰鐨勬槸鎶娾€滃皯閲忛珮鍊兼媺楂樺潎鍊尖€濈殑椋庨櫓鐢ㄤ笟鍔¤瑷€琛ㄨ揪鍑烘潵銆?
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
        // 2026-03-21: 杩欓噷鏋勯€犱竴涓簿纭嚎鎬у叧绯绘牱鏈紝鐩殑鏄厛閿佸畾 V1 绾挎€у洖褰掍細杈撳嚭绋冲畾绯绘暟銆佹埅璺濆拰 R2銆?
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

    // 2026-03-21: 杩欓噷閿佸畾鏈夋晥鏍锋湰琛屾暟锛岀洰鐨勬槸淇濊瘉鍚庣画缂哄け鍒犺閫昏緫涓嶄細璇垹姝ｅ父鏍锋湰銆?
    assert_eq!(result.row_count_used, 5);
    assert_eq!(result.dropped_rows, 0);
    assert_eq!(result.model_kind, "linear_regression");
    assert_eq!(result.problem_type, "regression");
    assert_eq!(result.data_summary.feature_count, 2);
    assert_eq!(result.quality_summary.primary_metric.name, "r2");
    // 2026-03-21: 杩欓噷閿佸畾绯绘暟椤哄簭涓庣壒寰佸垪涓€涓€瀵瑰簲锛岀洰鐨勬槸璁?Skill 鍜屼笂灞傞棶绛斿彲浠ョǔ瀹氬紩鐢ㄥ缓妯＄粨鏋溿€?
    assert_eq!(result.coefficients.len(), 2);
    assert_eq!(result.coefficients[0].feature, "feature_a");
    assert!((result.coefficients[0].value - 2.0).abs() < 1e-9);
    assert_eq!(result.coefficients[1].feature, "feature_b");
    assert!((result.coefficients[1].value - 3.0).abs() < 1e-9);
    // 2026-03-21: 杩欓噷閿佸畾鎴窛鍜屾嫙鍚堜紭搴︼紝鐩殑鏄‘淇濆洖褰掓牳蹇冭绠椾笉鏄彧杩斿洖浜嗚繎浼艰疆寤撱€?
    assert!((result.intercept - 10.0).abs() < 1e-9);
    assert!((result.r2 - 1.0).abs() < 1e-9);
    // 2026-03-21: 这里锁定中文摘要非空，目的是让非技术用户也能直接读结果。
    // 2026-03-23: 这里把历史乱码断言恢复为稳定 UTF-8 文本，原因是原断言已退化成编码噪声；目的是继续校验建模摘要可读性。
    assert!(result.human_summary.overall.contains("有效样本"));
}

#[test]
fn linear_regression_rejects_non_numeric_target() {
    let dataframe = DataFrame::new(vec![
        Series::new("feature_a".into(), vec![1_i64, 2, 3]).into(),
        Series::new("label".into(), vec!["high", "mid", "low"]).into(),
    ])
    .unwrap();
    let loaded = LoadedTable {
        // 2026-03-21: 杩欓噷鏋勯€犳枃鏈洰鏍囧垪锛岀洰鐨勬槸鍏堥攣瀹?V1 涓嶄細鎶婇潪鏁板€肩洰鏍囪閫佽繘绾挎€у洖褰掋€?
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

    // 2026-03-21: 这里锁定人话错误信息，目的是让低 IT 用户也能直接知道问题出在目标列类型上。
    // 2026-03-23: 这里把乱码断言恢复为稳定 UTF-8 文本，原因是历史编码损坏；目的是继续校验回归门禁而不是错误字节序。
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
        // 2026-03-21: 杩欓噷鏋勯€犳枃鏈壒寰佸垪锛岀洰鐨勬槸鍏堥攣瀹?V1 闇€瑕佹樉寮忔暟鍊肩壒寰佽€屼笉鏄伔鍋风寽娴嬬紪鐮併€?
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
    // 2026-03-23: 这里把乱码断言恢复为稳定 UTF-8 文本，原因是历史编码损坏；目的是继续校验建模前门禁是否还在。
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
        // 2026-03-21: 杩欓噷鏁呮剰鏀惧叆涓€涓己澶辩壒寰佸€硷紝鐩殑鏄攣瀹?V1 浼氬厛鍒犳帀鍧忚鍐嶅缓妯★紝鑰屼笉鏄洿鎺ユ姤閿欎腑鏂€?
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

    // 2026-03-21: 杩欓噷閿佸畾鍒犺鍚庣殑鏍锋湰鏁板拰涓㈠純鏁帮紝鐩殑鏄涓婂眰鑳介€忔槑鎻愮ず鈥滃摢浜涙暟鎹璺宠繃浜嗏€濄€?
    assert_eq!(result.row_count_used, 4);
    assert_eq!(result.dropped_rows, 1);
    // 2026-03-21: 杩欓噷閿佸畾缂哄け鍒犺鍚庝粛鑳芥仮澶嶆纭叧绯伙紝鐩殑鏄繚璇佸垹琛岄€昏緫涓嶄細姹℃煋鎷熷悎缁撴灉銆?
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
        // 2026-03-21: 杩欓噷鏋勯€犲畬鍏ㄩ噸澶嶄俊鎭殑涓ゅ垪鐗瑰緛锛岀洰鐨勬槸閿佸畾濂囧紓鐭╅樀浼氳璇嗗埆骞剁粰鍑虹洿鐧芥姤閿欍€?
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

    // 2026-03-21: 杩欓噷閿佸畾鍏辩嚎鎬ч敊璇枃妗堬紝鐩殑鏄伩鍏嶇敤鎴峰彧鐪嬪埌鈥滅煩闃典笉鍙€嗏€濊繖绉嶉毦鎳傛湳璇€?
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
        // 2026-03-21: 杩欓噷鏋勯€犲惈缂哄け鍊肩殑鍥炲綊鏍锋湰锛岀洰鐨勬槸鍏堥攣瀹氬叕鍏卞噯澶囧眰浼氭寜缁熶竴鍙ｅ緞鍒犺骞朵繚鐣欏畬鏁寸煩闃点€?
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

    // 2026-03-21: 杩欓噷閿佸畾鍒犺缁熻锛岀洰鐨勬槸淇濊瘉鍚庣画鍥炲綊鍜岃仛绫诲叡浜悓涓€濂楁湁鏁堟牱鏈彛寰勩€?
    assert_eq!(prepared.row_count_used, 3);
    assert_eq!(prepared.dropped_rows, 1);
    // 2026-03-21: 杩欓噷閿佸畾鎴窛鍒椾細琚啓鍏ヨ璁＄煩闃碉紝鐩殑鏄涓婂眰绠楁硶涓嶅繀閲嶅澶勭悊 intercept 閫昏緫銆?
    assert_eq!(prepared.feature_matrix[0], vec![1.0, 1.0, 10.0]);
    assert_eq!(prepared.targets, vec![5.0, 7.0, 11.0]);
}

#[test]
fn model_prep_builds_binary_classification_dataset_from_text_labels() {
    let dataframe = DataFrame::new(vec![
        Series::new("age".into(), vec![18.0_f64, 22.0, 38.0, 45.0]).into(),
        Series::new("score".into(), vec![10.0_f64, 15.0, 50.0, 60.0]).into(),
        Series::new(
            "result".into(),
            vec!["澶辫触", "澶辫触", "鎴愬姛", "鎴愬姛"],
        )
        .into(),
    ])
    .unwrap();
    let loaded = LoadedTable {
        // 2026-03-21: 杩欓噷鏋勯€犱腑鏂囦簩鍒嗙被鐩爣鍒楋紝鐩殑鏄攣瀹氬叕鍏卞噯澶囧眰浼氱ǔ瀹氭槧灏勬璐熺被鑰屼笉鏄妸鏂囨湰鐩存帴涓㈢粰绠楁硶灞傘€?
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
        Some("鎴愬姛"),
    )
    .unwrap();

    // 2026-03-21: 杩欓噷閿佸畾姝ｇ被鏍囩鍜屾暟鍊兼槧灏勶紝鐩殑鏄閫昏緫鍥炲綊缁撴灉鑳藉洿缁曠敤鎴风悊瑙ｇ殑鈥滄绫烩€濊緭鍑恒€?
    assert_eq!(prepared.row_count_used, 4);
    assert_eq!(prepared.positive_label, "鎴愬姛");
    assert_eq!(prepared.negative_label, "澶辫触");
    assert_eq!(prepared.targets, vec![0.0, 0.0, 1.0, 1.0]);
}

#[test]
fn model_prep_rejects_non_binary_target_values() {
    let dataframe = DataFrame::new(vec![
        Series::new("feature_a".into(), vec![1.0_f64, 2.0, 3.0]).into(),
        Series::new("stage".into(), vec!["new", "processing", "done"]).into(),
    ])
    .unwrap();
    let loaded = LoadedTable {
        // 2026-03-21: 杩欓噷鏋勯€犱笁鍒嗙被鐩爣鍒楋紝鐩殑鏄厛閿佸畾 V1 浜屽垎绫诲噯澶囧眰涓嶄細鎶婂鍒嗙被璇斁琛屻€?
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

    // 2026-03-21: 这里锁定直接可读的报错，目的是让用户知道当前不是二分类，而不是只看到抽象映射失败。
    // 2026-03-23: 这里把乱码断言恢复为稳定 UTF-8 文本，原因是历史编码损坏；目的是继续校验逻辑回归 V1 的边界提示。
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
        // 2026-03-21: 杩欓噷鏋勯€犲彲鍒嗙鐨勪簩鍒嗙被鏁版嵁锛岀洰鐨勬槸鍏堥攣瀹氶€昏緫鍥炲綊 V1 鑳藉畬鎴愭渶灏忚缁冮棴鐜€?
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

    // 2026-03-21: 杩欓噷閿佸畾閫昏緫鍥炲綊杈撳嚭鍩虹瀛楁锛岀洰鐨勬槸纭繚鍒嗘瀽寤烘ā灞傝兘绋冲畾娑堣垂鍒嗙被妯″瀷缁撴灉銆?
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
    // 2026-03-23: 这里把历史乱码断言恢复为稳定 UTF-8 文本，原因是原断言已退化成编码噪声；目的是继续校验分类建模摘要可读性。
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
        // 2026-03-21: 杩欓噷鏋勯€犳枃鏈爣绛惧苟鎸囧畾姝ｇ被锛岀洰鐨勬槸閿佸畾鈥滃摢涓€绫荤畻姝ｇ被鈥濅笉浼氳绯荤粺鎿呰嚜鏀瑰啓銆?
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

    // 2026-03-21: 杩欓噷閿佸畾姝ｇ被鏍囩鍜岀被鍒垎甯冿紝鐩殑鏄涓婂眰鎽樿鍥寸粫鐢ㄦ埛鍏冲績鐨勨€滄垚浜も€濇潵瑙ｉ噴妯″瀷銆?
    // 2026-03-23: 这里把测试标签值恢复为真实 UTF-8 中文，原因是历史乱码会让正类标签找不到；目的是校验“显式指定正类”这条能力本身。
    assert_eq!(result.positive_label, "成交");
    assert_eq!(result.class_balance.positive_count, 2);
    assert_eq!(result.class_balance.negative_count, 2);
}

#[test]
fn logistic_regression_rejects_non_binary_target() {
    let dataframe = DataFrame::new(vec![
        Series::new("feature_a".into(), vec![1.0_f64, 2.0, 3.0, 4.0]).into(),
        Series::new("status".into(), vec!["new", "processing", "done", "done"]).into(),
    ])
    .unwrap();
    let loaded = LoadedTable {
        // 2026-03-21: 杩欓噷鏋勯€犱笁绉嶇姸鎬佸€硷紝鐩殑鏄厛閿佸畾閫昏緫鍥炲綊 V1 鍙帴鍙椾簩鍒嗙被鐩爣鍒椼€?
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
    // 2026-03-23: 这里把乱码断言恢复为稳定 UTF-8 文本，原因是原断言已退化成编码噪声；目的是继续校验逻辑回归 V1 的边界提示。
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
        // 2026-03-21: 杩欓噷鏋勯€犱袱鍥㈡槑鏄惧垎寮€鐨勭偣锛岀洰鐨勬槸鍏堥攣瀹氳仛绫?Tool 鐨勬渶灏忚缁冮棴鐜拰缁熶竴杈撳嚭缁撴瀯銆?
        handle: TableHandle::new_confirmed(
            "memory://cluster-kmeans-basic",
            "Sheet1",
            vec!["x".into(), "y".into()],
        ),
        dataframe,
    };

    let result = cluster_kmeans(&loaded, &["x", "y"], 2, 50, MissingStrategy::DropRows).unwrap();

    // 2026-03-21: 杩欓噷閿佸畾鍏变韩寤烘ā鎬昏瀛楁锛岀洰鐨勬槸璁╁垎鏋愬缓妯″眰涓夌被 Tool 浠ュ悗閮借兘鎸夌粺涓€鍏ュ彛琚?Skill 娑堣垂銆?
    assert_eq!(result.model_kind, "cluster_kmeans");
    assert_eq!(result.problem_type, "clustering");
    assert_eq!(result.data_summary.feature_count, 2);
    assert_eq!(result.data_summary.row_count_used, 6);
    assert_eq!(result.data_summary.dropped_rows, 0);
    assert_eq!(result.quality_summary.primary_metric.name, "inertia");
    // 2026-03-21: 杩欓噷閿佸畾鑱氱被涓荤粨鏋滃瓧娈碉紝鐩殑鏄‘淇?V1 鑷冲皯鑳界ǔ瀹氳繑鍥炲垎缁勬暟閲忋€佷腑蹇冪偣鍜屾垚鍛樺綊灞炪€?
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
    // 2026-03-23: 这里把历史乱码断言恢复为稳定 UTF-8 文本，原因是原断言已退化成编码噪声；目的是继续校验聚类摘要的主题表达。
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
        // 2026-03-21: 杩欓噷鏁呮剰鏀惧叆缂哄け鐗瑰緛鍊硷紝鐩殑鏄攣瀹氳仛绫讳篃娌跨敤缁熶竴鍒犺鍙ｅ緞锛岃€屼笉鏄倓鎮勬贩鍏ヨ剰鏍锋湰銆?
        handle: TableHandle::new_confirmed(
            "memory://cluster-kmeans-drop-missing",
            "Sheet1",
            vec!["x".into(), "y".into()],
        ),
        dataframe,
    };

    let result = cluster_kmeans(&loaded, &["x", "y"], 2, 50, MissingStrategy::DropRows).unwrap();

    // 2026-03-21: 杩欓噷閿佸畾鍒犺缁熻锛岀洰鐨勬槸璁╀笂灞傝兘閫忔槑鎻愮ず鈥滃摢浜涙暟鎹病鏈夎繘鍏ヨ仛绫烩€濄€?
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
        // 2026-03-21: 杩欓噷鏋勯€犳渶灏忔牱鏈泦锛岀洰鐨勬槸閿佸畾 K 鍊奸潪娉曟椂浼氱洿鎺ョ粰鍑虹洿鐧介敊璇紝鑰屼笉鏄缁冮樁娈垫墠宕┿€?
        handle: TableHandle::new_confirmed(
            "memory://cluster-kmeans-invalid-k",
            "Sheet1",
            vec!["x".into(), "y".into()],
        ),
        dataframe,
    };

    let error = cluster_kmeans(&loaded, &["x", "y"], 4, 50, MissingStrategy::DropRows).unwrap_err();

    // 2026-03-21: 杩欓噷閿佸畾 K 鍊兼姤閿欐枃妗堬紝鐩殑鏄浣?IT 鐢ㄦ埛涔熺煡閬撴槸鈥滃垎缁勬暟澶ぇ鈥濊€屼笉鏄畻娉曢粦鐩掑紓甯搞€?
    assert!(error.to_string().contains("当前分组数"));
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
            vec![Some("yes"), Some("yes"), Some("no"), Some("no")],
        )
        .into(),
    ])
    .unwrap();
    let loaded = LoadedTable {
        // 2026-03-21: 杩欓噷鏋勯€犻噸澶嶉敭銆侀噸澶嶈鍜屽彲寤烘ā瀛楁骞跺瓨鐨勫皬琛紝鐩殑鏄攣瀹氬喅绛栧姪鎵嬩細鍏堟姄璐ㄩ噺闃诲锛屽啀缁欎笅涓€姝ュ缓璁€?
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

    // 2026-03-21: 杩欓噷閿佸畾鍐崇瓥鍔╂墜鏁翠綋褰㈡€侊紝鐩殑鏄涓婂眰闂瓟鐣岄潰鐩存帴鎷垮埌鈥滈樆濉為」 + 鍔ㄤ綔 + 涓嬩竴姝モ€濈殑绋冲畾缁撴瀯銆?
    assert_eq!(result.assistant_kind, "quality_diagnostic");
    assert_eq!(result.table_health.level, "risky");
    assert!(!result.blocking_risks.is_empty());
    assert!(!result.priority_actions.is_empty());
    assert!(!result.next_tool_suggestions.is_empty());
    // 2026-03-21: 杩欓噷閿佸畾楂樹紭鍏堢骇鍔ㄤ綔浼氬厛鎸囧悜璐ㄩ噺闂锛岀洰鐨勬槸閬垮厤鐢ㄦ埛杩樻病娓呮礂鏁版嵁灏辫寮曞鍘诲缓妯°€?
    assert_eq!(result.priority_actions[0].priority, "high");
    assert!(result.priority_actions[0].title.contains("先处理"));
    // 2026-03-21: 杩欓噷閿佸畾鍦ㄦ湁鏁板€煎垪鍜屽垎绫诲垪鏃朵細缁欏嚭寤烘ā鍊欓€夛紝鐩殑鏄妸鍐崇瓥鍔╂墜鐪熸妗ユ帴鍒板垎鏋愬缓妯″眰銆?
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
    // 2026-03-23: 这里把历史乱码断言恢复为稳定 UTF-8 文本，原因是原断言已退化成编码噪声；目的是继续校验决策助手会强调“优先处理”的顺序。
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

    // 2026-03-21: 杩欓噷閿佸畾鏄庢樉鐨?user_id 鍏宠仈鍊欓€変細琚洿鎺ヨ瘑鍒紝鐩殑鏄妸澶氳〃宸ヤ綔娴佹帹杩涘埌鈥滃厛寤鸿銆佸啀鎵ц鈥濈殑闃舵銆?
    assert!(!result.candidates.is_empty());
    assert_eq!(result.candidates[0].left_column, "user_id");
    assert_eq!(result.candidates[0].right_column, "user_id");
    assert_eq!(result.candidates[0].confidence, "high");
    assert!(result.candidates[0].match_row_count >= 2);
    // 2026-03-21: 杩欓噷閿佸畾杩斿洖鐨勬槸涓氬姟璇█闂鑰屼笉鏄妧鏈湳璇紝鐩殑鏄 Skill 鍙互鐩存帴鎷垮幓闂敤鎴枫€?
    assert!(result.candidates[0].question.contains("是否用"));
    assert_eq!(
        result.candidates[0].keep_mode_options[0].keep_mode,
        "matched_only"
    );
}

#[test]
fn suggest_table_links_returns_empty_candidates_when_no_obvious_relation_exists() {
    let left = LoadedTable {
        // 2026-03-21: 杩欓噷鏋勯€犱袱寮犳病鏈夋槑鏄句氦闆嗙殑琛紝鐩殑鏄攣瀹?V2.1 鍙仛淇濆畧寤鸿锛屼笉浼氫贡鐚滃叧鑱斻€?
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
        // 2026-03-21: 杩欓噷璁╁彸琛ㄥ彧鍖呭惈璁㈠崟閲戦绫诲瓧娈碉紝鐩殑鏄‘璁ゆ病鏈夋樉鎬ч敭鐗瑰緛鏃剁郴缁熶細鏄庣‘杩斿洖鈥滄病璇嗗埆鍒扳€濄€?
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

    // 2026-03-21: 杩欓噷閿佸畾绯荤粺鍦ㄦ病鏈夋槑鏄惧叧绯绘椂涓嶄細浼€犲€欓€夛紝鐩殑鏄伩鍏嶈瀵肩敤鎴峰仛閿欒鍏宠仈銆?
    assert_eq!(result.candidates.len(), 0);
    // 2026-03-23: 这里把历史乱码断言恢复为稳定 UTF-8 文本，原因是原断言已退化成编码噪声；目的是继续校验无明显关联候选时的人话提示。
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

    // 2026-03-22: 杩欓噷閿佸畾缁撴瀯涓€鑷磋〃浼氫紭鍏堟帹鑽愮旱鍚戣拷鍔狅紝鐩殑鏄妸鈥滃悓缁撴瀯鍒嗘壒鍒拌揪鈥濈殑鍦烘櫙浠?Skill 鐚滄祴杞垚绋冲畾 Tool 鍒ゆ柇銆?
    assert_eq!(result.recommended_action, "append_tables");
    assert!(result.append_candidate.is_some());
    assert_eq!(result.append_candidate.as_ref().unwrap().confidence, "high");
    // 2026-03-22: 杩欓噷閿佸畾杩藉姞寤鸿鐩存帴杈撳嚭涓氬姟纭闂锛岀洰鐨勬槸璁╅潪鎶€鏈敤鎴蜂篃鑳藉惉鎳傗€滄槸鍚︿笂涓嬫嫾鎺モ€濄€?
    assert!(
        result
            .append_candidate
            .as_ref()
            .unwrap()
            .question
            .contains("追加")
    );
    // 2026-03-22: 杩欓噷閿佸畾宸ヤ綔娴佸缓璁細鐩存帴缁欏嚭杩藉姞璋冪敤楠ㄦ灦锛岀洰鐨勬槸璁?Skill 涓嶅啀鑷繁鎷兼帴 top/bottom 鍙傛暟銆?
    assert_eq!(
        result.suggested_tool_call.as_ref().unwrap().tool,
        "append_tables"
    );
    assert_eq!(
        result.suggested_tool_call.as_ref().unwrap().args["top"]["sheet"],
        "Sales"
    );
    // 2026-03-23: 这里把历史乱码断言恢复为稳定 UTF-8 文本，原因是原断言已退化成编码噪声；目的是继续校验追加场景的人话总结。
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

    // 2026-03-22: 杩欓噷閿佸畾缁撴瀯涓嶄竴鑷翠絾鏄炬€ч敭娓呮鏃朵細鎺ㄨ崘鍏宠仈锛岀洰鐨勬槸鎶娾€滃厛鍒ゆ柇鍔ㄤ綔绫诲瀷鈥濈ǔ瀹氭矇娣€鍒板伐浣滄祦灞傘€?
    assert_eq!(result.recommended_action, "join_tables");
    assert!(result.append_candidate.is_none());
    assert!(!result.link_candidates.is_empty());
    assert_eq!(result.link_candidates[0].left_column, "user_id");
    assert_eq!(result.link_candidates[0].right_column, "user_id");
    // 2026-03-22: 杩欓噷閿佸畾宸ヤ綔娴佸缓璁細鐩存帴缁欏嚭鍏宠仈璋冪敤楠ㄦ灦锛岀洰鐨勬槸璁?Skill 鑳界洿鎺ユ壙鎺ラ涓樉鎬у€欓€夈€?
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
        // 2026-03-22: 杩欓噷鏋勯€犳棦涓嶈兘鐩存帴杩藉姞銆佷篃娌℃湁鏄炬€ч敭鍏崇郴鐨勮〃锛岀洰鐨勬槸閿佸畾宸ヤ綔娴佸眰浼氫繚瀹堝洖閫€鑰屼笉鏄己琛屾帹鑽愬姩浣溿€?
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
        // 2026-03-22: 杩欓噷璁╁彸琛ㄥ彧淇濈暀閲戦鍜屾笭閬撳垪锛岀洰鐨勬槸纭宸ヤ綔娴佸眰鍦ㄦ棤鏄庢樉鍔ㄤ綔鏃朵細鏄庣‘瑕佹眰缁х画纭銆?
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

    // 2026-03-22: 杩欓噷閿佸畾淇濆畧鍥為€€璺緞锛岀洰鐨勬槸閬垮厤绯荤粺鍦ㄥ琛ㄥ満鏅噷缁欏嚭閿欒杩藉姞鎴栭敊璇叧鑱斿缓璁€?
    assert_eq!(result.recommended_action, "manual_confirmation");
    assert!(result.append_candidate.is_none());
    assert_eq!(result.link_candidates.len(), 0);
    assert!(result.suggested_tool_call.is_none());
    // 2026-03-23: 这里把历史乱码断言恢复为稳定 UTF-8 文本，原因是原断言已退化成编码噪声；目的是继续校验保守回退的人话提示。
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

    // 2026-03-22: 杩欓噷閿佸畾澶氬紶鍚岀粨鏋勮〃浼氬厛鐢熸垚杩藉姞閾撅紝鐩殑鏄妸鈥滃厛鍚堝苟鍚岀粨鏋勬壒娆℃暟鎹€濇矇娣€鎴愬琛ㄨ鍒掔殑榛樿椤哄簭銆?
    assert_eq!(result.steps.len(), 2);
    assert_eq!(result.steps[0].action, "append_tables");
    assert_eq!(result.steps[0].input_refs, vec!["sales_a", "sales_b"]);
    // 2026-03-23: 这里把历史乱码断言恢复为稳定 UTF-8 文本，原因是原断言已退化成编码噪声；目的是继续校验多表计划第一步仍然明确指向追加。
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

    // 2026-03-22: 杩欓噷閿佸畾鏄炬€у彲鍏宠仈鐨勫弻琛ㄤ細鐢熸垚 join 璁″垝锛岀洰鐨勬槸璁╁琛ㄨ鍒掑櫒鑳界洿鎺ユ壙鎺ュ凡瀹屾垚鐨勫叧绯诲缓璁兘鍔涖€?
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
        // 2026-03-22: 杩欓噷鏋勯€犱竴缁勬棦涓嶈兘杩藉姞鍙堜笉鑳芥樉鎬у叧鑱旂殑琛紝鐩殑鏄攣瀹氬琛ㄨ鍒掑櫒浼氫繚瀹堜繚鐣欐湭鍐宠〃锛岃€屼笉鏄贡鎺掓楠ゃ€?
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
        // 2026-03-22: 杩欓噷璁╁彸琛ㄥ彧淇濈暀璁㈠崟閲戦鍜屾笭閬擄紝鐩殑鏄‘璁ゆ棤鏄庢樉鍔ㄤ綔鏃朵細鍘熸牱鐣欏湪 unresolved_refs銆?
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

    // 2026-03-22: 杩欓噷閿佸畾鏃犳槑鏄捐鍒掓椂涓嶄細浼€犳楠わ紝鐩殑鏄伩鍏嶇郴缁熼敊璇紩瀵煎琛ㄥ鐞嗛『搴忋€?
    assert_eq!(result.steps.len(), 0);
    assert_eq!(result.unresolved_refs, vec!["left", "right"]);
    // 2026-03-23: 这里把历史乱码断言恢复为稳定 UTF-8 文本，原因是原断言已退化成编码噪声；目的是继续校验无计划时的人话总结。
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
            // 2026-03-22: 杩欓噷鏁呮剰鎶婁富琛ㄦ斁鍦ㄥ墠闈€佸悓缁撴瀯鏄庣粏琛ㄦ斁鍦ㄥ悗闈紝鐩殑鏄攣瀹氳鍒掑櫒浼氬厛鍚堝苟鍚岀粨鏋勬壒娆℃暟鎹紝鍐嶆妸涓棿缁撴灉涓庝富琛ㄦ樉鎬у叧鑱斻€?
            ("customers".to_string(), customers),
            ("sales_a".to_string(), sales_a),
            ("sales_b".to_string(), sales_b),
        ],
        3,
    )
    .unwrap();

    // 2026-03-22: 杩欓噷閿佸畾娣峰悎鍦烘櫙浼氬厛杩藉姞锛岀洰鐨勬槸闃叉璁″垝鍣ㄥ湪鍙拷鍔犳椂鐩存帴璺冲幓 join锛屽鑷存壒娆℃暟鎹病鏈夊厛鏀跺彛銆?
    assert_eq!(result.steps.len(), 2);
    assert_eq!(result.steps[0].action, "append_tables");
    assert_eq!(result.steps[0].input_refs, vec!["sales_a", "sales_b"]);
    assert_eq!(result.steps[0].result_ref, "step_1_result");
    // 2026-03-23: 这里把历史乱码断言恢复为稳定 UTF-8 文本，原因是原断言已退化成编码噪声；目的是继续校验混合计划第一步仍然是先追加。
    assert!(result.steps[0].question.contains("追加"));
    // 2026-03-22: 杩欓噷閿佸畾绗簩姝ヤ細寮曠敤绗竴姝ヤ腑闂寸粨鏋滃幓鍋氭樉鎬у叧鑱旓紝鐩殑鏄‘淇濆琛ㄨ鍒掔湡姝ｅ叿澶囬摼寮忕紪鎺掕兘鍔涳紝鑰屼笉鏄彧浼氱嫭绔嬬粰鍗曟寤鸿銆?
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
