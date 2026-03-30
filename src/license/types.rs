use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct LicenseConfig {
    pub enforced: bool,
    pub api_base_url: String,
    pub store_id: Option<u64>,
    pub product_id: Option<u64>,
    pub variant_id: Option<u64>,
    pub validate_max_age_hours: i64,
    pub offline_grace_hours: i64,
}

impl LicenseConfig {
    pub fn from_env() -> Self {
        // 2026-03-29 CST: 这里统一从环境变量加载 Lemon 配置，原因是商店标识不能直接硬编码进仓库；
        // 目的：让开发环境和售卖版 EXE 可以共用同一套代码，只通过部署配置切换门禁。
        Self {
            enforced: parse_env_bool("EXCEL_SKILL_LICENSE_ENFORCED").unwrap_or(false),
            api_base_url: std::env::var("EXCEL_SKILL_LEMON_BASE_URL")
                .unwrap_or_else(|_| "https://api.lemonsqueezy.com".to_string()),
            store_id: parse_env_u64("EXCEL_SKILL_LEMON_STORE_ID"),
            product_id: parse_env_u64("EXCEL_SKILL_LEMON_PRODUCT_ID"),
            variant_id: parse_env_u64("EXCEL_SKILL_LEMON_VARIANT_ID"),
            validate_max_age_hours: parse_env_i64("EXCEL_SKILL_LICENSE_VALIDATE_MAX_AGE_HOURS")
                .unwrap_or(72),
            offline_grace_hours: parse_env_i64("EXCEL_SKILL_LICENSE_OFFLINE_GRACE_HOURS")
                .unwrap_or(168),
        }
    }

    pub fn is_ready(&self) -> bool {
        self.store_id.is_some() && self.product_id.is_some() && self.variant_id.is_some()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct LicenseActivateRequest {
    pub license_key: String,
    #[serde(default)]
    pub instance_name: Option<String>,
    #[serde(default)]
    pub customer_email: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct LicenseStatusRequest {
    #[serde(default)]
    pub refresh: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LicenseStatusResult {
    pub mode: String,
    pub configured: bool,
    pub enforced: bool,
    pub licensed: bool,
    pub status: String,
    pub message: String,
    pub license_key_masked: Option<String>,
    pub customer_email: Option<String>,
    pub instance_id: Option<String>,
    pub instance_name: Option<String>,
    pub store_id: Option<u64>,
    pub product_id: Option<u64>,
    pub variant_id: Option<u64>,
    pub validated_at: Option<String>,
    pub next_validation_due_at: Option<String>,
    pub offline_grace_expires_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LicenseDeactivateResult {
    pub licensed: bool,
    pub deactivated: bool,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LemonLicenseApiResponse {
    #[serde(default)]
    pub activated: Option<bool>,
    #[serde(default)]
    pub valid: Option<bool>,
    #[serde(default)]
    pub deactivated: Option<bool>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub license_key: Option<LemonLicenseKeyPayload>,
    #[serde(default)]
    pub instance: Option<LemonLicenseInstancePayload>,
    #[serde(default)]
    pub meta: Option<LemonLicenseMetaPayload>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LemonLicenseKeyPayload {
    pub key: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LemonLicenseInstancePayload {
    #[serde(deserialize_with = "deserialize_stringish")]
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LemonLicenseMetaPayload {
    #[serde(default)]
    pub store_id: Option<u64>,
    #[serde(default)]
    pub product_id: Option<u64>,
    #[serde(default)]
    pub variant_id: Option<u64>,
    #[serde(default)]
    pub customer_email: Option<String>,
}

fn parse_env_bool(name: &str) -> Option<bool> {
    let value = std::env::var(name).ok()?;
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn parse_env_u64(name: &str) -> Option<u64> {
    std::env::var(name).ok()?.trim().parse::<u64>().ok()
}

fn parse_env_i64(name: &str) -> Option<i64> {
    std::env::var(name).ok()?.trim().parse::<i64>().ok()
}

fn deserialize_stringish<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    // 2026-03-29 CST: 这里兼容数字或字符串形式的 instance_id，原因是第三方 API 字段在不同环境里可能出现不同 JSON 形态；
    // 目的：避免因为 ID 类型细节漂移把整条授权链路打断。
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::String(text) => Ok(text),
        Value::Number(number) => Ok(number.to_string()),
        other => Err(serde::de::Error::custom(format!(
            "无法解析实例 ID: {other}"
        ))),
    }
}
