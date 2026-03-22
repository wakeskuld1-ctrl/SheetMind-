// 2026-03-22: 这里导出本地记忆运行时模块，目的是让 dispatcher 和测试都能复用统一的 SQLite 状态层。
pub mod local_memory;
