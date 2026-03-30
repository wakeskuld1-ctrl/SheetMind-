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

// 2026-03-30 CST: 这里定义授权刷新桥接结果，原因是“在线成功 / 在线失败但回退本地 / 真正错误”不能只靠一份摘要文本区分；
// 目的：让应用壳在刷新完成后既能更新摘要，也能根据结果类型给页面落 warning 或 error 提示。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LicenseRefreshResult {
    pub summary: LicenseSummary,
    pub warning_message: Option<String>,
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
    match load_license_summary_result(false) {
        Ok(result) => result.summary,
        Err(error) => LicenseSummary {
            status_text: error,
            ..LicenseSummary::default()
        },
    }
}

// 2026-03-30 CST: 这里补充授权页主动刷新入口，原因是刷新按钮不能继续只读本地缓存，需要走在线校验并返回细分结果。
// 目的：把“启动快照读取”和“主动刷新校验”分成两个稳定桥接入口，避免 GUI 层直接依赖 LicenseService 细节。
pub fn refresh_license_summary() -> Result<LicenseRefreshResult, String> {
    load_license_summary_result(true)
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

impl LicenseRefreshResult {
    // 2026-03-30 CST: 这里补充纯成功构造器，原因是测试需要在不依赖真实授权服务的情况下稳定注入刷新结果；
    // 目的：让 TDD 可以直接构造“刷新成功且无警告”的结果，复用正式落地路径。
    pub fn success(summary: LicenseSummary) -> Self {
        Self {
            summary,
            warning_message: None,
        }
    }

    // 2026-03-30 CST: 这里统一把授权服务结果转换为刷新桥接结果，原因是 GUI 需要知道本次刷新是否带有在线校验警告；
    // 目的：把 warning 判定集中收口在桥接层，避免应用壳靠硬编码字符串零散判断。
    fn from_status_result(result: &LicenseStatusResult, refresh: bool) -> Self {
        let warning_message = if refresh && result.message.starts_with("在线校验失败") {
            Some(result.message.clone())
        } else {
            None
        };

        Self {
            summary: LicenseSummary::from_status_result(result),
            warning_message,
        }
    }
}

// 2026-03-30 CST: 这里抽出通用授权摘要读取逻辑，原因是启动时读取和主动刷新都依赖同一条授权服务桥接链路；
// 目的：让不同读取模式只通过 `refresh` 开关区分，减少桥接层重复代码。
fn load_license_summary_result(refresh: bool) -> Result<LicenseRefreshResult, String> {
    let service = LicenseService::from_env();
    let result = service.status(refresh).map_err(|error| error.to_string())?;
    Ok(LicenseRefreshResult::from_status_result(&result, refresh))
}
