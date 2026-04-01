use thiserror::Error;

use crate::license::types::LemonLicenseApiResponse;

#[derive(Debug, Error)]
pub enum LemonLicenseClientError {
    #[error("请求 Lemon Squeezy 失败: {0}")]
    Transport(String),
    #[error("Lemon Squeezy 返回错误: {0}")]
    Api(String),
    #[error("解析 Lemon Squeezy 响应失败: {0}")]
    Parse(String),
}

#[derive(Debug, Clone)]
pub struct LemonLicenseClient {
    base_url: String,
}

impl LemonLicenseClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }

    pub fn activate(
        &self,
        license_key: &str,
        instance_name: &str,
    ) -> Result<LemonLicenseApiResponse, LemonLicenseClientError> {
        self.post_form(
            "/v1/licenses/activate",
            &[
                ("license_key", license_key),
                ("instance_name", instance_name),
            ],
        )
    }

    pub fn validate(
        &self,
        license_key: &str,
        instance_id: &str,
    ) -> Result<LemonLicenseApiResponse, LemonLicenseClientError> {
        self.post_form(
            "/v1/licenses/validate",
            &[("license_key", license_key), ("instance_id", instance_id)],
        )
    }

    pub fn deactivate(
        &self,
        license_key: &str,
        instance_id: &str,
    ) -> Result<LemonLicenseApiResponse, LemonLicenseClientError> {
        self.post_form(
            "/v1/licenses/deactivate",
            &[("license_key", license_key), ("instance_id", instance_id)],
        )
    }

    fn post_form(
        &self,
        path: &str,
        fields: &[(&str, &str)],
    ) -> Result<LemonLicenseApiResponse, LemonLicenseClientError> {
        // 2026-03-29 CST: 这里统一走 form POST，原因是 Lemon License API 当前就是表单编码接口；
        // 目的：把激活 / 校验 / 反激活三条 HTTP 合同收口成一个稳定实现，减少重复代码。
        let url = format!("{}{}", self.base_url.trim_end_matches('/'), path);
        let response = ureq::post(&url)
            .set("Accept", "application/json")
            .send_form(fields);

        match response {
            Ok(response) => parse_response_body(
                response
                    .into_string()
                    .map_err(|error| LemonLicenseClientError::Transport(error.to_string()))?,
            ),
            Err(ureq::Error::Status(_, response)) => {
                let body = response
                    .into_string()
                    .map_err(|error| LemonLicenseClientError::Transport(error.to_string()))?;
                match serde_json::from_str::<LemonLicenseApiResponse>(&body) {
                    Ok(payload) => {
                        if let Some(error) = payload.error.clone() {
                            Err(LemonLicenseClientError::Api(error))
                        } else {
                            Err(LemonLicenseClientError::Api(body))
                        }
                    }
                    Err(_) => Err(LemonLicenseClientError::Api(body)),
                }
            }
            Err(ureq::Error::Transport(error)) => {
                Err(LemonLicenseClientError::Transport(error.to_string()))
            }
        }
    }
}

fn parse_response_body(body: String) -> Result<LemonLicenseApiResponse, LemonLicenseClientError> {
    serde_json::from_str::<LemonLicenseApiResponse>(&body)
        .map_err(|error| LemonLicenseClientError::Parse(error.to_string()))
}
