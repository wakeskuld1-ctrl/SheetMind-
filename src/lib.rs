// 2026-03-21: 这里公开分层模块入口，目的是让后续 CLI、问答界面与 Tool 适配层都复用同一套核心逻辑。
pub mod domain;
// 2026-03-21: 这里暴露 Excel 基础能力模块，目的是先打通工作簿探测与后续 schema 识别的底座。
pub mod excel;
// 2026-03-21: 这里暴露表注册表模块，目的是为确认后的表对象分配 table_id 并承接后续 DataFrame 生命周期。
pub mod frame;
// 2026-03-21: 这里暴露 DataFrame 原子操作模块，目的是让 Tool 层调用稳定、可测试的计算操作。
pub mod ops;
// 2026-03-22: 这里暴露本地运行时记忆模块，目的是把会话状态从 Skill 协议层下沉到独立的本地持久层。
pub mod runtime;
// 2026-03-21: 这里暴露 Tool 模块，目的是让 CLI 只负责收发 JSON，而具体分发规则由独立模块维护。
pub mod tools;

use tools::contracts::ToolResponse;

// 2026-03-21: 这里保留最小目录输出函数，目的是让 CLI 在没有请求体时也能稳定返回 JSON 能力目录。
pub fn tool_catalog_json() -> String {
    // 2026-03-21: 统一复用 ToolResponse 的目录工厂，目的是避免 main 与 Tool 层各自维护一份目录结构。
    serde_json::to_string(&ToolResponse::tool_catalog())
        .expect("tool catalog serialization should succeed")
}
