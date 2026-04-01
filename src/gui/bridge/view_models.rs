use serde_json::Value;

// 2026-03-29 CST: 这里定义 GUI 通用 Tool 执行结果视图模型，原因是界面层不适合直接到处处理底层 `ToolResponse`；
// 目的：把状态、数据和错误统一收口成 GUI 可复用的轻量结果对象。
#[derive(Debug, Clone)]
pub struct ToolRunResult {
    pub success: bool,
    pub status: String,
    pub data: Value,
    pub error: Option<String>,
}

impl ToolRunResult {
    // 2026-03-29 CST: 这里提供成功构造器，原因是 GUI 桥接层需要把不同来源的成功结果统一成同一结构；
    // 目的：减少每个桥接方法重复手写结果包装逻辑。
    pub fn ok(status: impl Into<String>, data: Value) -> Self {
        Self {
            success: true,
            status: status.into(),
            data,
            error: None,
        }
    }

    // 2026-03-29 CST: 这里提供失败构造器，原因是 GUI 页面需要统一判断并展示错误信息；
    // 目的：让桥接层在返回错误时仍能保持固定结果形状。
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            status: "error".to_string(),
            data: Value::Object(Default::default()),
            error: Some(message.into()),
        }
    }
}
