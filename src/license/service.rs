use chrono::{Duration, Utc};
use thiserror::Error;

use crate::license::client::{LemonLicenseClient, LemonLicenseClientError};
use crate::license::types::{
    LemonLicenseApiResponse, LemonLicenseMetaPayload, LicenseActivateRequest, LicenseConfig,
    LicenseDeactivateResult, LicenseStatusResult,
};
use crate::runtime::license_store::{LicenseStore, LicenseStoreError, StoredLicenseState};

const PUBLIC_LICENSE_TOOLS: &[&str] = &["license_activate", "license_status", "license_deactivate"];

#[derive(Debug, Error)]
pub enum LicenseServiceError {
    #[error("{0}")]
    Store(#[from] LicenseStoreError),
    #[error("{0}")]
    Client(#[from] LemonLicenseClientError),
    #[error("授权门禁已开启，但 Lemon Squeezy 配置不完整，请先设置商店与产品参数")]
    IncompleteConfiguration,
    #[error("当前 EXE 未激活，请先调用 license_activate")]
    MissingLicense,
    #[error("授权校验失败，请先重新激活或检查 Lemon Squeezy 授权状态")]
    LicenseInvalid,
    #[error("{0}")]
    Message(String),
}

#[derive(Debug, Clone)]
pub struct LicenseService {
    config: LicenseConfig,
    client: LemonLicenseClient,
}

impl LicenseService {
    pub fn from_env() -> Self {
        let config = LicenseConfig::from_env();
        let client = LemonLicenseClient::new(config.api_base_url.clone());
        Self { config, client }
    }

    pub fn enforce_tool_access(&self, tool_name: &str) -> Result<(), LicenseServiceError> {
        if PUBLIC_LICENSE_TOOLS.contains(&tool_name) {
            return Ok(());
        }

        if !self.config.enforced {
            return Ok(());
        }

        if !self.config.is_ready() {
            return Err(LicenseServiceError::IncompleteConfiguration);
        }

        let store = LicenseStore::workspace_default()?;
        let Some(state) = store.load()? else {
            return Err(LicenseServiceError::MissingLicense);
        };

        if self.is_state_fresh(&state) {
            return Ok(());
        }

        match self.refresh_state(&state) {
            Ok(refreshed_state) => {
                if is_remote_state_valid(&refreshed_state) {
                    Ok(())
                } else {
                    Err(LicenseServiceError::LicenseInvalid)
                }
            }
            Err(error) => {
                if self.is_within_offline_grace(&state) {
                    Ok(())
                } else {
                    Err(error)
                }
            }
        }
    }

    pub fn activate(
        &self,
        request: &LicenseActivateRequest,
    ) -> Result<LicenseStatusResult, LicenseServiceError> {
        self.ensure_ready()?;

        let instance_name = request
            .instance_name
            .clone()
            .unwrap_or_else(default_instance_name);
        let response = self.client.activate(&request.license_key, &instance_name)?;
        if response.activated != Some(true) {
            return Err(LicenseServiceError::Message(
                "Lemon Squeezy 未确认激活成功".to_string(),
            ));
        }

        let state = self.state_from_remote_response(
            &response,
            &request.license_key,
            &instance_name,
            request.customer_email.as_deref(),
        )?;

        let store = LicenseStore::workspace_default()?;
        store.save(&state)?;
        self.build_status_result(Some(state), "激活成功".to_string())
    }

    pub fn status(&self, refresh: bool) -> Result<LicenseStatusResult, LicenseServiceError> {
        let store = LicenseStore::workspace_default()?;
        let state = store.load()?;

        if !self.config.enforced {
            return self.build_status_result(
                state,
                "授权门禁当前未开启，开发环境仍可直接使用".to_string(),
            );
        }

        if !self.config.is_ready() {
            return self.build_status_result(
                state,
                "授权门禁已开启，但 Lemon Squeezy 配置还不完整".to_string(),
            );
        }

        if !refresh {
            return self.build_status_result(state, "已读取本地授权状态".to_string());
        }

        let Some(current_state) = state else {
            return self.build_status_result(None, "当前还没有本地授权记录".to_string());
        };

        match self.refresh_state(&current_state) {
            Ok(refreshed_state) => {
                self.build_status_result(Some(refreshed_state), "已完成在线授权校验".to_string())
            }
            Err(error) => self.build_status_result(
                Some(current_state),
                format!("在线校验失败，已返回本地状态: {error}"),
            ),
        }
    }

    pub fn deactivate(&self) -> Result<LicenseDeactivateResult, LicenseServiceError> {
        let store = LicenseStore::workspace_default()?;
        let Some(state) = store.load()? else {
            return Ok(LicenseDeactivateResult {
                licensed: false,
                deactivated: false,
                message: "当前没有本地授权记录".to_string(),
            });
        };

        let response = self
            .client
            .deactivate(&state.license_key, &state.instance_id)?;
        if response.deactivated != Some(true) {
            return Err(LicenseServiceError::Message(
                "Lemon Squeezy 未确认反激活成功".to_string(),
            ));
        }

        // 2026-03-29 CST: 这里在远端确认成功后立即清空本地缓存，原因是反激活如果只停远端不清本地，
        // 目的：会导致单机缓存继续放行，违背这轮“限制普通传播”的授权目标。
        store.clear()?;
        Ok(LicenseDeactivateResult {
            licensed: false,
            deactivated: true,
            message: "授权已停用，本地缓存已清空".to_string(),
        })
    }

    fn refresh_state(
        &self,
        state: &StoredLicenseState,
    ) -> Result<StoredLicenseState, LicenseServiceError> {
        let response = self
            .client
            .validate(&state.license_key, &state.instance_id)?;
        if response.valid != Some(true) {
            return Err(LicenseServiceError::LicenseInvalid);
        }

        let refreshed_state = self.state_from_remote_response(
            &response,
            &state.license_key,
            &state.instance_name,
            state.customer_email.as_deref(),
        )?;
        let store = LicenseStore::workspace_default()?;
        store.save(&refreshed_state)?;
        Ok(refreshed_state)
    }

    fn state_from_remote_response(
        &self,
        response: &LemonLicenseApiResponse,
        fallback_license_key: &str,
        fallback_instance_name: &str,
        fallback_customer_email: Option<&str>,
    ) -> Result<StoredLicenseState, LicenseServiceError> {
        if let Some(error) = response.error.clone() {
            return Err(LicenseServiceError::Message(error));
        }

        let Some(meta) = response.meta.clone() else {
            return Err(LicenseServiceError::Message(
                "Lemon Squeezy 响应缺少 meta 信息".to_string(),
            ));
        };
        self.ensure_meta_matches(&meta)?;

        let Some(license_key) = response.license_key.clone() else {
            return Err(LicenseServiceError::Message(
                "Lemon Squeezy 响应缺少 license_key 信息".to_string(),
            ));
        };
        let Some(instance) = response.instance.clone() else {
            return Err(LicenseServiceError::Message(
                "Lemon Squeezy 响应缺少 instance 信息".to_string(),
            ));
        };

        let now = Utc::now().to_rfc3339();
        Ok(StoredLicenseState {
            license_key: if license_key.key.is_empty() {
                fallback_license_key.to_string()
            } else {
                license_key.key.clone()
            },
            license_key_masked: mask_license_key(fallback_license_key),
            instance_id: instance.id,
            instance_name: instance
                .name
                .unwrap_or_else(|| fallback_instance_name.to_string()),
            customer_email: meta
                .customer_email
                .or_else(|| fallback_customer_email.map(str::to_string)),
            store_id: meta.store_id,
            product_id: meta.product_id,
            variant_id: meta.variant_id,
            license_status: license_key.status.unwrap_or_else(|| "active".to_string()),
            last_validation_status: "validated".to_string(),
            activated_at: now.clone(),
            validated_at: now,
            last_error: None,
        })
    }

    fn ensure_ready(&self) -> Result<(), LicenseServiceError> {
        if self.config.is_ready() {
            Ok(())
        } else {
            Err(LicenseServiceError::IncompleteConfiguration)
        }
    }

    fn ensure_meta_matches(
        &self,
        meta: &LemonLicenseMetaPayload,
    ) -> Result<(), LicenseServiceError> {
        if self.config.store_id != meta.store_id {
            return Err(LicenseServiceError::Message(
                "Lemon Squeezy store_id 与本地配置不一致".to_string(),
            ));
        }
        if self.config.product_id != meta.product_id {
            return Err(LicenseServiceError::Message(
                "Lemon Squeezy product_id 与本地配置不一致".to_string(),
            ));
        }
        if self.config.variant_id != meta.variant_id {
            return Err(LicenseServiceError::Message(
                "Lemon Squeezy variant_id 与本地配置不一致".to_string(),
            ));
        }
        Ok(())
    }

    fn build_status_result(
        &self,
        state: Option<StoredLicenseState>,
        message: String,
    ) -> Result<LicenseStatusResult, LicenseServiceError> {
        let (
            licensed,
            status,
            license_key_masked,
            customer_email,
            instance_id,
            instance_name,
            validated_at,
        ) = match state.clone() {
            Some(state) => (
                is_remote_state_valid(&state),
                local_status_label(self, &state),
                Some(state.license_key_masked),
                state.customer_email,
                Some(state.instance_id),
                Some(state.instance_name),
                Some(state.validated_at),
            ),
            None => (
                false,
                if self.config.enforced {
                    "unlicensed".to_string()
                } else {
                    "disabled".to_string()
                },
                None,
                None,
                None,
                None,
                None,
            ),
        };

        let next_validation_due_at =
            validated_at
                .as_deref()
                .and_then(parse_rfc3339)
                .map(|timestamp| {
                    (timestamp + Duration::hours(self.config.validate_max_age_hours)).to_rfc3339()
                });
        let offline_grace_expires_at =
            validated_at
                .as_deref()
                .and_then(parse_rfc3339)
                .map(|timestamp| {
                    (timestamp + Duration::hours(self.config.offline_grace_hours)).to_rfc3339()
                });

        Ok(LicenseStatusResult {
            mode: if self.config.enforced {
                "enforced".to_string()
            } else {
                "disabled".to_string()
            },
            configured: self.config.is_ready(),
            enforced: self.config.enforced,
            licensed,
            status,
            message,
            license_key_masked,
            customer_email,
            instance_id,
            instance_name,
            store_id: self.config.store_id,
            product_id: self.config.product_id,
            variant_id: self.config.variant_id,
            validated_at,
            next_validation_due_at,
            offline_grace_expires_at,
        })
    }

    fn is_state_fresh(&self, state: &StoredLicenseState) -> bool {
        let Some(validated_at) = parse_rfc3339(&state.validated_at) else {
            return false;
        };
        Utc::now() < validated_at + Duration::hours(self.config.validate_max_age_hours)
    }

    fn is_within_offline_grace(&self, state: &StoredLicenseState) -> bool {
        let Some(validated_at) = parse_rfc3339(&state.validated_at) else {
            return false;
        };
        Utc::now() <= validated_at + Duration::hours(self.config.offline_grace_hours)
    }
}

fn default_instance_name() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "excel-skill-machine".to_string())
}

fn mask_license_key(license_key: &str) -> String {
    let chars = license_key.chars().collect::<Vec<_>>();
    if chars.len() <= 8 {
        return "****".to_string();
    }
    let prefix = chars.iter().take(4).collect::<String>();
    let suffix = chars.iter().rev().take(4).collect::<String>();
    let suffix = suffix.chars().rev().collect::<String>();
    format!("{prefix}****{suffix}")
}

fn is_remote_state_valid(state: &StoredLicenseState) -> bool {
    state.license_status.eq_ignore_ascii_case("active")
}

fn parse_rfc3339(value: &str) -> Option<chrono::DateTime<Utc>> {
    chrono::DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|timestamp| timestamp.with_timezone(&Utc))
}

fn local_status_label(service: &LicenseService, state: &StoredLicenseState) -> String {
    if service.is_state_fresh(state) {
        "validated".to_string()
    } else if service.is_within_offline_grace(state) {
        "stale".to_string()
    } else {
        "expired".to_string()
    }
}
