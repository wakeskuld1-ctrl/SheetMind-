// 2026-03-22 CST：这里导出本地记忆运行时模块，原因是 dispatcher、测试和后续 GUI 都需要复用同一套 SQLite 状态层；
// 目的：保持会话状态落盘入口统一，避免不同上层各自维护一份运行时协议。
pub mod local_memory;
// 2026-03-29 CST：这里导出授权状态存储模块，原因是现有授权主线已经确认沿 runtime SQLite 落盘；
// 目的：让 CLI、测试和桌面壳都共用同一份授权缓存逻辑。
pub mod license_store;
// 2026-03-28 CST：这里导出股票历史存储模块，原因是股票能力已明确沿 Rust + SQLite 主线扩展；
// 目的：把行情历史持久层和 session 状态层拆开，保证各自职责清晰。
pub mod stock_history_store;
// 2026-04-02 CST：这里导出共振平台存储模块，原因是方案 3 已确认要先把因子、事件和快照沉淀到正式 SQLite；
// 目的：让后续 Tool 和研究入口都能复用同一份共振数据底座，而不是各写各的临时落盘逻辑。
pub mod security_resonance_store;
// 2026-04-02 CST：这里导出信号结果研究平台存储模块，原因是方案C要求把快照、未来收益和 analog study 独立沉到 runtime；
// 目的：让历史验证层和行情层、共振层解耦，后续 briefing 与投决会都能稳定复用。
pub mod signal_outcome_store;
