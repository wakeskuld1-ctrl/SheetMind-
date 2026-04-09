// 2026-03-22: 这里导出本地记忆运行时模块，目的是让 dispatcher 和测试都能复用统一的 SQLite 状态层。
pub mod local_memory;
// 2026-03-29 CST: 这里导出本地授权状态存储模块，原因是 Lemon 直连授权方案要求把授权缓存落到现有 SQLite runtime；
// 目的：复用当前单机 EXE 的 runtime 目录，不新开第二套授权落盘体系。
pub mod license_store;
// 2026-03-28 CST: 这里导出股票历史存储模块，原因是后续股票能力要沿 Rust + SQLite 主线扩展；
// 目的：把行情历史持久层独立出来，避免和 session 记忆表混在同一个实现里。
pub mod stock_history_store;
