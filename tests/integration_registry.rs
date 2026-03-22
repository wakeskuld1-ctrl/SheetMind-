use excel_skill::domain::handles::TableHandle;
use excel_skill::domain::schema::{SchemaState, infer_schema_state_label};
use excel_skill::frame::registry::TableRegistry;
use excel_skill::frame::result_ref_store::{PersistedResultDataset, ResultRefStore};
use excel_skill::frame::table_ref_store::{PersistedTableRef, TableRefStore};
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

    // 2026-03-22: 这里锁定 table_ref 会真正落盘并能读回，目的是确保方案 C 不是单进程内假句柄，而是跨请求可复用的持久化引用。
    assert_eq!(loaded.table_ref, "table_test_roundtrip");
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

    let record = PersistedTableRef::new_for_test(
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

    // 2026-03-22: 这里先锁定 region table_ref 的 JSON 结构会带上显式区域，目的是为局部区域确认态跨请求复用打底。
    store.save(&record).unwrap();
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
                // 2026-03-23: 这里补齐 file_ref/sheet_index 缺省值，原因是会话状态 patch 已扩展文件句柄维度；目的是保持旧测试继续锁定原有状态往返行为。
                current_file_ref: None,
                current_sheet_index: None,
                current_stage: Some(SessionStage::AnalysisModeling),
                schema_status: Some(SchemaStatus::Confirmed),
                active_table_ref: Some("table_runtime_round_trip".to_string()),
                last_user_goal: Some("先看统计摘要".to_string()),
                selected_columns: Some(vec!["sales".to_string()]),
            },
        )
        .unwrap();

    let state = runtime
        .get_session_state("session_runtime_round_trip")
        .unwrap();

    // 2026-03-22: 这里先锁定 SQLite runtime 的最小 round-trip 行为，目的是确保 orchestrator 后续读取到的不是临时内存态，而是真实本地持久状态。
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

    // 2026-03-22: 这里先锁定 result_ref 不只是元数据句柄，还能把 DataFrame 结构和数据原样恢复回来，目的是给后续跨步骤闭环打底。
    assert_eq!(loaded.result_ref, "result_round_trip");
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
