use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::tools::catalog;

// 2026-04-08 CST: 这里给 ToolRequest 补序列化能力，原因是七席委员会子进程需要把内部 tool 请求重新编码后写入 CLI stdin；
// 目的：让正式 dispatcher 合同既能接收外部请求，也能被投决会内部 seat agent 复用，避免再造第二套进程内协议。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolRequest {
    pub tool: String,
    #[serde(default)]
    pub args: Value,
}

// 2026-04-08 CST: 这里给 ToolResponse 补反序列化能力，原因是七席委员会父进程需要把子进程返回的 JSON 安全回读成正式响应；
// 目的：确保独立执行证明链仍沿用现有 ToolResponse 合同，而不是在 committee 内部额外拼装弱类型 JSON。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolResponse {
    pub status: String,
    pub data: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ToolResponse {
    pub fn ok(data: Value) -> Self {
        Self {
            status: "ok".to_string(),
            data,
            error: None,
        }
    }

    // 2026-04-02 CST: 这里补一个强类型序列化入口，原因是 security_decision_briefing 后续会引入更厚的结构化响应，不适合在每个 dispatcher 分支重复手写 `json!(result)`；
    // 目的：让 Tool 层可以直接复用 serde 序列化结果，统一合同输出路径并减少重复样板。
    pub fn ok_serialized<T: Serialize>(data: &T) -> Self {
        let serialized =
            serde_json::to_value(data).expect("tool response serialization should succeed");
        Self::ok(serialized)
    }

    pub fn needs_confirmation(data: Value) -> Self {
        Self {
            status: "needs_confirmation".to_string(),
            data,
            error: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            status: "error".to_string(),
            data: json!({}),
            error: Some(message.into()),
        }
    }

    pub fn tool_catalog() -> Self {
        Self::ok(json!({
            "tool_catalog": catalog::tool_names(),
            // 2026-03-31 CST: 这里把分模块目录一并暴露给外部，原因是当前项目已经明确分成 foundation / stock 两个能力域。
            // 目的：在不破坏原有平铺 tool_catalog 契约的前提下，为 AI、GUI 和后续路由提供稳定的模块边界元数据。
            "tool_catalog_modules": {
                "foundation": catalog::foundation_tool_names(),
                "stock": catalog::stock_tool_names(),
            }
        }))
    }
}
