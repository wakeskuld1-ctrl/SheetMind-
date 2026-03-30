use serde_json::{Value, json};

use crate::license::service::LicenseService;
use crate::tools::contracts::{ToolRequest, ToolResponse};
use crate::tools::dispatcher::dispatch;

use super::view_models::ToolRunResult;

// 2026-03-29 CST: 这里定义 GUI Tool 执行桥接器，原因是桌面界面不能直接散落构造 `ToolRequest` 和解析 `ToolResponse`；
// 目的：把 GUI 对现有 Tool Contract 的调用统一收口到一个本地桥接对象上。
#[derive(Debug, Default, Clone)]
pub struct ToolRunner;

impl ToolRunner {
    // 2026-03-29 CST: 这里保留无状态构造入口，原因是当前 GUI 桥接层还不需要额外依赖注入；
    // 目的：先给页面层一个稳定的最小调用点，后续再视需要扩展。
    pub fn new() -> Self {
        Self
    }

    // 2026-03-29 CST: 这里提供目录查询桥接，原因是 GUI 首批需要先验证能否连接到底层 Tool 主链；
    // 目的：用最小无副作用请求证明 GUI 与 Engine 之间的桥已经通了。
    pub fn catalog(&self) -> Result<ToolRunResult, String> {
        self.run("tool_catalog", Value::Null)
    }

    // 2026-03-29 CST: 这里提供 workbook 打开桥接，原因是“打开 Excel 文件”是 GUI 首发主流程起点；
    // 目的：为文件页后续接入真实导入动作预留统一调用口。
    pub fn open_workbook(&self, path: &str) -> Result<ToolRunResult, String> {
        self.run("open_workbook", json!({ "path": path }))
    }

    // 2026-03-29 CST: 这里提供 sheet 列表桥接，原因是文件页下一步需要把 workbook 转成可选工作表列表；
    // 目的：让 GUI 能复用现有 workbook 能力而不是自己解析 Excel。
    pub fn list_sheets(&self, path: &str) -> Result<ToolRunResult, String> {
        self.run("list_sheets", json!({ "path": path }))
    }

    // 2026-03-29 CST: 这里提供表预览桥接，原因是文件页和处理页都需要统一的预览入口；
    // 目的：先把 GUI 的“预览”动作绑定到现有 Tool 合同上。
    pub fn preview_table(&self, source: Value) -> Result<ToolRunResult, String> {
        self.run("preview_table", source)
    }

    // 2026-03-29 CST: 这里提供授权状态桥接，原因是 GUI 顶栏和设置页都需要走统一的授权读取入口；
    // 目的：保持 GUI 仍然复用现有授权服务，而不是复制授权业务逻辑。
    pub fn license_status(&self, refresh: bool) -> Result<ToolRunResult, String> {
        let service = LicenseService::from_env();
        match service.status(refresh) {
            Ok(result) => Ok(ToolRunResult::ok(
                "ok",
                serde_json::to_value(result).map_err(|error| error.to_string())?,
            )),
            Err(error) => Ok(ToolRunResult::error(error.to_string())),
        }
    }

    fn run(&self, tool: &str, args: Value) -> Result<ToolRunResult, String> {
        if tool == "tool_catalog" {
            let response = ToolResponse::tool_catalog();
            return Ok(Self::map_tool_response(response));
        }

        let request = ToolRequest {
            tool: tool.to_string(),
            args,
        };
        let response = dispatch(request);
        Ok(Self::map_tool_response(response))
    }

    fn map_tool_response(response: ToolResponse) -> ToolRunResult {
        let success = response.status == "ok" || response.status == "needs_confirmation";
        ToolRunResult {
            success,
            status: response.status,
            data: response.data,
            error: response.error,
        }
    }
}
