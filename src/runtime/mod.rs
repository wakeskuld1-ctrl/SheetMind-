// 2026-03-22: 这里导出本地记忆运行时模块，目的是让 dispatcher 和测试都能复用统一的 SQLite 状态层。
pub mod local_memory;
// 2026-03-29 CST: 这里导出本地授权状态存储模块，原因是 Lemon 直连授权方案要求把授权缓存落到现有 SQLite runtime；
// 目的：复用当前单机 EXE 的 runtime 目录，不新开第二套授权落盘体系。
pub mod eastmoney_budget_store;
pub mod eastmoney_cache_store;
pub mod license_store;
// 2026-03-28 CST: 这里导出股票历史存储模块，原因是后续股票能力要沿 Rust + SQLite 主线扩展；
// 目的：把行情历史持久层独立出来，避免和 session 记忆表混在同一个实现里。
pub mod stock_history_store;
// 2026-04-11 CST: Export the dated external-proxy runtime store, because P4
// needs governed proxy backfill to live in the same runtime family as stock
// history and approval artifacts.
// Purpose: give stock tools one stable runtime module for dated proxy reads/writes.
pub mod security_external_proxy_store;
// 2026-04-12 CST: Export the governed stock fundamental-history store, because
// Historical Data Phase 1 needs replayable financial snapshots to live beside
// price and proxy history in the runtime family.
// Purpose: give fullstack and validation one stable SQLite home for stock fundamentals.
pub mod security_fundamental_history_store;
// 2026-04-12 CST: Export the governed stock disclosure-history store, because
// Historical Data Phase 1 needs replayable announcement history instead of
// rebuilding event context from one-off live fetches every time.
// Purpose: give fullstack and validation one stable SQLite home for stock disclosures.
pub mod security_disclosure_history_store;
