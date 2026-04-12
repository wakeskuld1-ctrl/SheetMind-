use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::runtime_paths::workspace_runtime_dir;

// 2026-04-11 UTC+08: Added typed cache envelope so upper layers can know a response was served
// from local cache; purpose: avoid burning limited EastMoney free calls on repeated analysis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EastMoneyCached<T> {
    pub payload: T,
    pub cache_hit: bool,
}

#[derive(Debug, Clone)]
pub struct EastMoneyCacheStore {
    root: PathBuf,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct EastMoneyCacheEntry {
    expires_at_epoch_seconds: u64,
    payload: serde_json::Value,
}

impl EastMoneyCacheStore {
    pub fn new(root: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        Ok(Self { root })
    }

    pub fn workspace_default() -> Result<Self, String> {
        let runtime_dir = workspace_runtime_dir()?;
        Self::new(runtime_dir.join("eastmoney_cache"))
    }

    pub fn put<T: Serialize>(
        &self,
        namespace: &str,
        key: &str,
        ttl_seconds: u64,
        payload: &T,
    ) -> Result<(), String> {
        let payload = serde_json::to_value(payload).map_err(|error| error.to_string())?;
        let entry = EastMoneyCacheEntry {
            expires_at_epoch_seconds: current_epoch_seconds().saturating_add(ttl_seconds),
            payload,
        };
        let path = self.entry_path(namespace, key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let body = serde_json::to_string_pretty(&entry).map_err(|error| error.to_string())?;
        fs::write(path, body).map_err(|error| error.to_string())
    }

    pub fn get<T: DeserializeOwned>(
        &self,
        namespace: &str,
        key: &str,
    ) -> Result<Option<EastMoneyCached<T>>, String> {
        let path = self.entry_path(namespace, key);
        if !path.exists() {
            return Ok(None);
        }
        let body = fs::read_to_string(&path).map_err(|error| error.to_string())?;
        let entry: EastMoneyCacheEntry =
            serde_json::from_str(&body).map_err(|error| error.to_string())?;
        if entry.expires_at_epoch_seconds <= current_epoch_seconds() {
            let _ = fs::remove_file(&path);
            return Ok(None);
        }
        let payload = serde_json::from_value(entry.payload).map_err(|error| error.to_string())?;
        Ok(Some(EastMoneyCached {
            payload,
            cache_hit: true,
        }))
    }

    fn entry_path(&self, namespace: &str, key: &str) -> PathBuf {
        self.root
            .join(namespace)
            .join(format!("{}.json", sanitize_key(key)))
    }
}

fn sanitize_key(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn current_epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
