use excel_skill::domain::handles::TableHandle;
use excel_skill::domain::schema::{SchemaState, infer_schema_state_label};
use excel_skill::frame::registry::TableRegistry;
use excel_skill::frame::chart_ref_store::{
    ChartDraftStore, PersistedChartDraft, PersistedChartSeriesSpec, PersistedChartType,
};
use excel_skill::frame::result_ref_store::{PersistedResultDataset, ResultRefStore};
use excel_skill::frame::table_ref_store::{PersistedTableRef, TableRefStore};
use excel_skill::ops::report_delivery::chart_ref_to_report_delivery_chart;
use excel_skill::runtime::local_memory::{
    LocalMemoryRuntime, SchemaStatus, SessionStage, SessionStatePatch,
};
use polars::prelude::{DataFrame, DataType, NamedFrom, Series, TimeUnit};

#[test]
fn registry_assigns_incrementing_table_ids_for_confirmed_tables() {
    let mut registry = TableRegistry::new();
    let table = TableHandle::new_confirmed(
        "tests/fixtures/basic-sales.xlsx",
        "Sales",
        vec![
            "user_id".to_string(),
            "region".to_string(),
            "sales".to_string(),
        ],
    );

    let table_id = registry.register(table);
    assert_eq!(table_id, "table_1");
}

#[test]
fn confirmed_table_exposes_confirmed_schema_label() {
    let table = TableHandle::new_confirmed(
        "tests/fixtures/basic-sales.xlsx",
        "Sales",
        vec!["user_id".to_string()],
    );

    assert_eq!(table.schema_state(), &SchemaState::Confirmed);
    assert_eq!(infer_schema_state_label(table.schema_state()), "confirmed");
}

#[test]
fn stored_table_ref_round_trips_through_disk() {
    let store_dir = std::path::PathBuf::from("tests")
        .join("runtime_fixtures")
        .join("table_ref_store");
    std::fs::create_dir_all(&store_dir).unwrap();
    let store = TableRefStore::new(store_dir);

    // 2026-03-23: 修正这里的变量名，原因是上一轮把 `record` 误改成 `_record` 后仍继续按 `record` 使用，导致全量测试编译失败。
    // 2026-03-23: 目的是真正恢复 round-trip 回归测试，而不是只消除 unused warning。
    let record = PersistedTableRef::new_for_test(
        "table_test_roundtrip",
        "tests/fixtures/title-gap-header.xlsx",
        "Sheet1",
        vec!["user_id".to_string(), "sales".to_string()],
        1,
        3,
        None,
    );

    store.save(&record).unwrap();
    let loaded = store.load("table_test_roundtrip").unwrap();

    // 2026-03-22: 杩欓噷閿佸畾 table_ref 浼氱湡姝ｈ惤鐩樺苟鑳借鍥烇紝鐩殑鏄‘淇濇柟妗?C 涓嶆槸鍗曡繘绋嬪唴鍋囧彞鏌勶紝鑰屾槸璺ㄨ姹傚彲澶嶇敤鐨勬寔涔呭寲寮曠敤銆?    assert_eq!(loaded.table_ref, "table_test_roundtrip");
    assert_eq!(loaded.source_path, "tests/fixtures/title-gap-header.xlsx");
    assert_eq!(loaded.sheet_name, "Sheet1");
    assert_eq!(
        loaded.columns,
        vec!["user_id".to_string(), "sales".to_string()]
    );
    assert_eq!(loaded.data_start_row_index, 3);
    assert_eq!(loaded.region, None);
}

#[test]
fn stored_region_table_ref_round_trips_and_reloads_same_region() {
    let store_dir = std::path::PathBuf::from("tests")
        .join("runtime_fixtures")
        .join("table_ref_store_region");
    std::fs::create_dir_all(&store_dir).unwrap();
    let store = TableRefStore::new(store_dir);

    let _record = PersistedTableRef::new_for_test(
        "table_region_roundtrip",
        "tests/runtime_fixtures/generated_workbooks/region_seed_placeholder.xlsx",
        "Report",
        vec![
            "user_id".to_string(),
            "region".to_string(),
            "sales".to_string(),
        ],
        1,
        1,
        Some("B3:D5".to_string()),
    );

    // 2026-03-22: 杩欓噷鍏堥攣瀹?region table_ref 鐨?JSON 缁撴瀯浼氬甫涓婃樉寮忓尯鍩燂紝鐩殑鏄负灞€閮ㄥ尯鍩熺‘璁ゆ€佽法璇锋眰澶嶇敤鎵撳簳銆?    store.save(&record).unwrap();
    let loaded = store.load("table_region_roundtrip").unwrap();
    assert_eq!(loaded.region.as_deref(), Some("B3:D5"));
}

#[test]
fn runtime_persists_session_state_round_trip() {
    let runtime_db = std::path::PathBuf::from("tests")
        .join("runtime_fixtures")
        .join("local_memory_registry")
        .join("runtime-round-trip.db");
    if let Some(parent) = runtime_db.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    if runtime_db.exists() {
        std::fs::remove_file(&runtime_db).unwrap();
    }

    let runtime = LocalMemoryRuntime::new(runtime_db);
    runtime
        .update_session_state(
            "session_runtime_round_trip",
            &SessionStatePatch {
                current_workbook: Some("tests/fixtures/title-gap-header.xlsx".to_string()),
                current_sheet: Some("Sheet1".to_string()),
                // 2026-03-23: 这里补 file_ref 失败测试占位，原因是本轮要继续保持旧 round-trip 行为稳定；目的是在扩展激活句柄语义时不破坏已有会话字段。
                current_file_ref: None,
                current_sheet_index: None,
                current_stage: Some(SessionStage::AnalysisModeling),
                schema_status: Some(SchemaStatus::Confirmed),
                active_table_ref: Some("table_runtime_round_trip".to_string()),
                // 2026-03-23: 这里补 active_handle_ref 失败测试输入，原因是方案B要把“确认态表句柄”和“当前最新激活句柄”拆开；目的是锁定 session_state 能同时记住稳定回源 table_ref 与最新结果句柄。
                active_handle_ref: Some("result_runtime_round_trip".to_string()),
                // 2026-03-23: 这里补 active_handle_kind 失败测试输入，原因是上层 Skill 需要直接区分 table_ref/result_ref/workbook_ref；目的是避免仅靠前缀猜测导致状态语义漂移。
                active_handle_kind: Some("result_ref".to_string()),
                last_user_goal: Some("先看统计摘要".to_string()),
                selected_columns: Some(vec!["sales".to_string()]),
            },
        )
        .unwrap();

    let state = runtime
        .get_session_state("session_runtime_round_trip")
        .unwrap();

    // 2026-03-22: 这里先锁定 SQLite runtime 的最小 round-trip 行为，目的是确保 orchestrator 读到的是落盘后的真实会话状态。
    assert_eq!(
        state.current_workbook.as_deref(),
        Some("tests/fixtures/title-gap-header.xlsx")
    );
    assert_eq!(state.current_sheet.as_deref(), Some("Sheet1"));
    assert_eq!(state.current_stage, SessionStage::AnalysisModeling);
    assert_eq!(state.schema_status, SchemaStatus::Confirmed);
    assert_eq!(
        state.active_table_ref.as_deref(),
        Some("table_runtime_round_trip")
    );
    assert_eq!(
        state.active_handle_ref.as_deref(),
        Some("result_runtime_round_trip")
    );
    assert_eq!(state.active_handle_kind.as_deref(), Some("result_ref"));
    assert_eq!(state.last_user_goal.as_deref(), Some("先看统计摘要"));
    assert_eq!(state.selected_columns, vec!["sales".to_string()]);
}

#[test]
fn stored_result_dataset_round_trips_through_disk() {
    let store_dir = std::path::PathBuf::from("tests")
        .join("runtime_fixtures")
        .join("result_ref_store");
    std::fs::create_dir_all(&store_dir).unwrap();
    let store = ResultRefStore::new(store_dir);
    // 2026-03-22: 这里构造混合类型结果集，目的是先锁定后续链式执行需要的最小持久化能力，而不是只保存字符串表。
    let dataframe = DataFrame::new(vec![
        Series::new("customer_id".into(), ["c001", "c002"]).into(),
        Series::new("score".into(), [98_i64, 76_i64]).into(),
        Series::new("priority".into(), ["A", "B"]).into(),
        Series::new("matched".into(), [true, false]).into(),
    ])
    .unwrap();
    let record = PersistedResultDataset::from_dataframe(
        "result_round_trip",
        "group_and_aggregate",
        vec!["table_sales".to_string()],
        &dataframe,
    )
    .unwrap();

    store.save(&record).unwrap();
    let loaded = store.load("result_round_trip").unwrap();
    let restored = loaded.to_dataframe().unwrap();

    // 2026-03-22: 杩欓噷鍏堥攣瀹?result_ref 涓嶅彧鏄厓鏁版嵁鍙ユ焺锛岃繕鑳芥妸 DataFrame 缁撴瀯鍜屾暟鎹師鏍锋仮澶嶅洖鏉ワ紝鐩殑鏄粰鍚庣画璺ㄦ楠ら棴鐜墦搴曘€?    assert_eq!(loaded.result_ref, "result_round_trip");
    assert_eq!(loaded.produced_by, "group_and_aggregate");
    assert_eq!(loaded.source_refs, vec!["table_sales".to_string()]);
    assert_eq!(restored.height(), 2);
    assert_eq!(restored.width(), 4);
    assert_eq!(
        restored
            .column("customer_id")
            .unwrap()
            .str()
            .unwrap()
            .get(0),
        Some("c001")
    );
    assert_eq!(
        restored.column("score").unwrap().i64().unwrap().get(1),
        Some(76)
    );
    assert_eq!(
        restored.column("matched").unwrap().bool().unwrap().get(0),
        Some(true)
    );
}

#[test]
fn stored_result_dataset_round_trips_dense_nulls_and_all_null_columns() {
    let store_dir = std::path::PathBuf::from("tests")
        .join("runtime_fixtures")
        .join("result_ref_store_dense_nulls");
    std::fs::create_dir_all(&store_dir).unwrap();
    let store = ResultRefStore::new(store_dir);
    // 2026-03-22: 这里补密集空值与全空列 round-trip 测试，目的是锁定 result_ref_store 不会因为空值密集而丢行或把全空列写坏。
    let dataframe = DataFrame::new(vec![
        Series::new("customer_id".into(), ["c001", "c002", "c003"]).into(),
        Series::new("score".into(), [Some(95_i64), None, Some(88_i64)]).into(),
        Series::new(
            "note".into(),
            [None::<String>, None::<String>, None::<String>],
        )
        .into(),
        Series::new("tag".into(), [Some("A"), None, Some("C")]).into(),
    ])
    .unwrap();
    let record = PersistedResultDataset::from_dataframe(
        "result_dense_nulls",
        "stat_summary",
        vec!["result_upstream".to_string()],
        &dataframe,
    )
    .unwrap();

    store.save(&record).unwrap();
    let restored = store.load("result_dense_nulls").unwrap().to_dataframe().unwrap();

    assert_eq!(restored.height(), 3);
    assert_eq!(restored.width(), 4);
    assert_eq!(restored.column("score").unwrap().i64().unwrap().get(1), None);
    assert_eq!(restored.column("note").unwrap().null_count(), 3);
    assert_eq!(restored.column("tag").unwrap().str().unwrap().get(1), None);
}

#[test]
fn stored_result_dataset_preserves_non_finite_float_values() {
    let store_dir = std::path::PathBuf::from("tests")
        .join("runtime_fixtures")
        .join("result_ref_store_non_finite");
    std::fs::create_dir_all(&store_dir).unwrap();
    let store = ResultRefStore::new(store_dir);
    // 2026-03-22: 这里补非有限浮点值 round-trip 测试，目的是锁定 NaN/Infinity 不会在 JSON 落盘后静默变成空值。
    let dataframe = DataFrame::new(vec![
        Series::new(
            "score".into(),
            [Some(f64::NAN), Some(f64::INFINITY), Some(f64::NEG_INFINITY)],
        )
        .into(),
    ])
    .unwrap();
    let record = PersistedResultDataset::from_dataframe(
        "result_non_finite",
        "linear_regression",
        vec!["result_metrics".to_string()],
        &dataframe,
    )
    .unwrap();

    store.save(&record).unwrap();
    let restored = store.load("result_non_finite").unwrap().to_dataframe().unwrap();
    let values = restored.column("score").unwrap().f64().unwrap();

    assert!(values.get(0).unwrap().is_nan());
    assert!(values.get(1).unwrap().is_infinite());
    assert!(values.get(1).unwrap().is_sign_positive());
    assert!(values.get(2).unwrap().is_infinite());
    assert!(values.get(2).unwrap().is_sign_negative());
}

#[test]
fn stored_result_dataset_round_trips_date_and_datetime_columns() {
    let store_dir = std::path::PathBuf::from("tests")
        .join("runtime_fixtures")
        .join("result_ref_store_datetime");
    std::fs::create_dir_all(&store_dir).unwrap();
    let store = ResultRefStore::new(store_dir);
    // 2026-03-22: 这里补日期与日期时间列 round-trip 测试，目的是锁定 result_ref_store 不会把时间语义退化成普通字符串列。
    let date_series = Series::new("biz_date".into(), [20_000_i32, 20_001_i32])
        .cast(&DataType::Date)
        .unwrap();
    let datetime_series = Series::new(
        "created_at".into(),
        [1_700_000_000_000_i64, 1_700_003_600_000_i64],
    )
    .cast(&DataType::Datetime(TimeUnit::Milliseconds, None))
    .unwrap();
    let dataframe = DataFrame::new(vec![date_series.into(), datetime_series.into()]).unwrap();
    let record = PersistedResultDataset::from_dataframe(
        "result_datetime_round_trip",
        "parse_datetime_columns",
        vec!["table_dates".to_string()],
        &dataframe,
    )
    .unwrap();

    store.save(&record).unwrap();
    let restored = store
        .load("result_datetime_round_trip")
        .unwrap()
        .to_dataframe()
        .unwrap();

    assert_eq!(restored.column("biz_date").unwrap().dtype(), &DataType::Date);
    assert_eq!(
        restored.column("created_at").unwrap().dtype(),
        &DataType::Datetime(TimeUnit::Milliseconds, None)
    );
}

#[test]
fn chart_draft_roundtrips_through_disk() {
    let store_dir = std::path::PathBuf::from("tests")
        .join("runtime_fixtures")
        .join("chart_ref_store");
    std::fs::create_dir_all(&store_dir).unwrap();
    let store = ChartDraftStore::new(store_dir);

    let dataframe = DataFrame::new(vec![
        Series::new("month".into(), ["Jan", "Feb", "Mar"]).into(),
        Series::new("revenue".into(), [120_i64, 150_i64, 132_i64]).into(),
        Series::new("profit".into(), [35_i64, 40_i64, 38_i64]).into(),
    ])
    .unwrap();

    let draft = PersistedChartDraft::from_dataframe(
        "chart_round_trip",
        "build_chart",
        vec!["result_monthly_metrics".to_string()],
        &dataframe,
        PersistedChartType::Column,
        Some("Revenue vs Profit".to_string()),
        "month",
        vec![
            PersistedChartSeriesSpec {
                value_column: "revenue".to_string(),
                name: Some("Revenue".to_string()),
            },
            PersistedChartSeriesSpec {
                value_column: "profit".to_string(),
                name: Some("Profit".to_string()),
            },
        ],
    )
    .unwrap();

    store.save(&draft).unwrap();
    let loaded = store.load("chart_round_trip").unwrap();
    let restored = loaded.to_dataframe().unwrap();

    assert_eq!(loaded.chart_ref, "chart_round_trip");
    assert_eq!(loaded.produced_by, "build_chart");
    assert_eq!(loaded.source_refs, vec!["result_monthly_metrics".to_string()]);
    assert_eq!(loaded.chart_type, PersistedChartType::Column);
    assert_eq!(loaded.title.as_deref(), Some("Revenue vs Profit"));
    assert_eq!(loaded.category_column, "month");
    assert_eq!(loaded.series.len(), 2);
    assert_eq!(restored.height(), 3);
    assert_eq!(restored.column("revenue").unwrap().i64().unwrap().get(1), Some(150));
}

#[test]
fn chart_draft_can_be_mapped_to_report_delivery_chart() {
    let dataframe = DataFrame::new(vec![
        Series::new("month".into(), ["Jan", "Feb", "Mar"]).into(),
        Series::new("revenue".into(), [120_i64, 150_i64, 132_i64]).into(),
        Series::new("profit".into(), [35_i64, 40_i64, 38_i64]).into(),
    ])
    .unwrap();
    let draft = PersistedChartDraft::from_dataframe_with_layout(
        "chart_bridge_round_trip",
        "build_chart",
        vec!["result_monthly_metrics".to_string()],
        &dataframe,
        PersistedChartType::Column,
        Some("Revenue vs Profit".to_string()),
        "month",
        Some("Month".to_string()),
        Some("Amount".to_string()),
        true,
        720,
        420,
        vec![
            PersistedChartSeriesSpec {
                value_column: "revenue".to_string(),
                name: Some("Revenue".to_string()),
            },
            PersistedChartSeriesSpec {
                value_column: "profit".to_string(),
                name: Some("Profit".to_string()),
            },
        ],
    )
    .unwrap();

    // 2026-03-24: 这里先锁定 chart_ref 草稿能稳定桥接成 report_delivery 图表规格，原因是方案 A 的核心是统一两条图表通路；目的是避免桥接逻辑继续散落在 dispatcher 里。
    let chart = chart_ref_to_report_delivery_chart(&draft);

    assert_eq!(chart.chart_ref.as_deref(), Some("chart_bridge_round_trip"));
    assert_eq!(chart.source_refs, vec!["result_monthly_metrics".to_string()]);
    assert_eq!(chart.title.as_deref(), Some("Revenue vs Profit"));
    assert_eq!(chart.category_column, "month");
    assert_eq!(chart.x_axis_name.as_deref(), Some("Month"));
    assert_eq!(chart.y_axis_name.as_deref(), Some("Amount"));
    assert_eq!(chart.show_legend, true);
    assert_eq!(chart.series.len(), 2);
}
