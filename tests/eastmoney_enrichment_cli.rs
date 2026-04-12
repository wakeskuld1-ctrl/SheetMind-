use excel_skill::runtime::eastmoney_budget_store::{
    EastMoneyBudgetScope, EastMoneyBudgetStore, EastMoneyBudgetStoreConfig,
};
use excel_skill::runtime::eastmoney_cache_store::EastMoneyCacheStore;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct CacheFixture {
    value: String,
}

fn create_runtime_root(prefix: &str) -> PathBuf {
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let runtime_root = PathBuf::from("tests")
        .join("runtime_fixtures")
        .join("eastmoney_enrichment")
        .join(format!("{prefix}_{unique_suffix}"));
    fs::create_dir_all(&runtime_root).expect("eastmoney runtime root should exist");
    runtime_root
}

#[test]
fn eastmoney_budget_store_tracks_scope_usage_and_exhaustion() {
    let runtime_root = create_runtime_root("budget_store");
    let store = EastMoneyBudgetStore::new(
        runtime_root.join("eastmoney_budget.json"),
        EastMoneyBudgetStoreConfig {
            total_daily_limit: 3,
            capital_flow_daily_limit: 2,
            event_daily_limit: 1,
        },
    )
    .expect("budget store should be created");

    let first = store
        .consume(EastMoneyBudgetScope::CapitalFlow)
        .expect("first consume should succeed");
    assert_eq!(first.status, "available");
    assert_eq!(first.remaining_for_scope, 1);

    let second = store
        .consume(EastMoneyBudgetScope::CapitalFlow)
        .expect("second consume should succeed");
    assert_eq!(second.status, "available");
    assert_eq!(second.remaining_for_scope, 0);

    let exhausted = store
        .consume(EastMoneyBudgetScope::CapitalFlow)
        .expect("third consume should return exhausted status");
    assert_eq!(exhausted.status, "budget_exhausted");
    assert_eq!(exhausted.remaining_for_scope, 0);
}

#[test]
fn eastmoney_cache_store_returns_cached_payload_before_expiry() {
    let runtime_root = create_runtime_root("cache_store");
    let store = EastMoneyCacheStore::new(runtime_root.join("eastmoney_cache"))
        .expect("cache store should be created");

    store
        .put(
            "capital_flow",
            "002352.SZ",
            3600,
            &CacheFixture {
                value: "cached".to_string(),
            },
        )
        .expect("cache write should succeed");

    let cached = store
        .get::<CacheFixture>("capital_flow", "002352.SZ")
        .expect("cache read should succeed")
        .expect("cache entry should exist");
    assert_eq!(
        cached.payload,
        CacheFixture {
            value: "cached".to_string()
        }
    );
    assert!(cached.cache_hit);
}
