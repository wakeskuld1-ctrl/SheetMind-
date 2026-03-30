use crate::license::service::LicenseService;
use crate::license::types::LicenseStatusResult;

// 2026-03-29 CST: 这里定义授权摘要视图模型，原因是 GUI 不应该直接耦合远端授权或本地存储明细；
// 目的：把授权状态压缩成界面可直接消费的最小字段集合。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LicenseSummary {
    pub licensed: bool,
    pub status_text: String,
    pub license_email: String,
    pub last_validated_at: String,
    pub device_status: String,
}

impl Default for LicenseSummary {
    fn default() -> Self {
        Self {
            licensed: false,
            status_text: "未授权".to_string(),
            license_email: "未绑定邮箱".to_string(),
            last_validated_at: "尚未校验".to_string(),
            device_status: "未绑定设备".to_string(),
        }
    }
}

// 2026-03-29 CST: 这里提供 GUI 读取授权摘要入口，原因是启动时顶栏和设置页都需要同一份授权状态；
// 目的：复用现有 `LicenseService`，只做摘要转换，不在 GUI 层复制授权业务规则。
pub fn load_license_summary() -> LicenseSummary {
    let service = LicenseService::from_env();
    match service.status(false) {
        Ok(result) => LicenseSummary::from_status_result(&result),
        Err(error) => LicenseSummary {
            status_text: error.to_string(),
            ..LicenseSummary::default()
        },
    }
}

impl LicenseSummary {
    // 2026-03-29 CST: 这里把授权服务结果转换为 GUI 摘要，原因是界面只关心能否显示、何时校验、当前设备状态；
    // 目的：把 `LicenseStatusResult` 的技术细节收敛成稳定的展示字段。
    pub fn from_status_result(result: &LicenseStatusResult) -> Self {
        Self {
            licensed: result.licensed,
            status_text: result.message.clone(),
            license_email: result
                .customer_email
                .clone()
                .unwrap_or_else(|| "未绑定邮箱".to_string()),
            last_validated_at: result
                .validated_at
                .clone()
                .unwrap_or_else(|| "尚未校验".to_string()),
            device_status: if result.instance_id.is_some() {
                "已绑定设备".to_string()
            } else {
                "未绑定设备".to_string()
            },
        }
    }
}
