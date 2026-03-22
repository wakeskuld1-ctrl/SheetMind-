// 2026-03-21: 这里导出 Tool 协议与调度模块，目的是把 CLI 输入输出适配与核心能力解耦，便于未来问答界面复用。
pub mod contracts;
// 2026-03-21: 这里拆出调度模块，目的是让每个 Tool 的分发逻辑保持单一职责，避免 main 膨胀。
pub mod dispatcher;
