// 2026-03-21: 这里公开分层模块入口，目的是让后续 CLI、问答界面与 Tool 适配层都复用同一套核心逻辑。
pub mod domain;
// 2026-03-21: 这里暴露 Excel 基础能力模块，目的是先打通工作簿探测与后续 schema 识别的底座。
pub mod excel;
// 2026-03-21: 这里暴露表注册表模块，目的是为确认后的表对象分配 table_id 并承接后续 DataFrame 生命周期。
pub mod frame;
// 2026-03-29 CST: 这里暴露本地授权模块，原因是 Lemon Squeezy 授权要沿现有 Rust / exe 主链落地；
// 目的：让 CLI 主入口和后续测试都能复用同一套授权服务，而不是各自拼装校验逻辑。
pub mod license;
// 2026-03-21: 这里暴露 DataFrame 原子操作模块，目的是让 Tool 层调用稳定、可测试的计算操作。
pub mod ops;
pub mod runtime_paths;
// 2026-03-29 CST: 这里暴露 GUI 模块入口，原因是首发桌面版需要在同一 crate 内复用授权、运行时和 Tool 协议；
// 目的：让 GUI 二进制在不复制工程的前提下，逐步接入状态层、桥接层和页面层。
// 2026-03-31 CST: 这里给 GUI 模块补 feature 门，原因是当前产品默认主线不应把 GUI 与 CLI/股票链路绑死在一起；
// 目的：保留桌面壳为可选能力，同时恢复默认库与命令行构建的纯主线形态。
#[cfg(feature = "gui")]
pub mod gui;
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
