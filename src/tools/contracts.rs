use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::tools::catalog;

#[derive(Debug, Clone, Deserialize)]
pub struct ToolRequest {
    pub tool: String,
    #[serde(default)]
    pub args: Value,
}

#[derive(Debug, Clone, Serialize)]
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
