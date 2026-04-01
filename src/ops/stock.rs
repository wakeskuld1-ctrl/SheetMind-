// 2026-03-31 CST: 这里建立 stock 模块边界，原因是股票导入、同步和技术面咨询已经不再属于通用分析底座。
// 目的：把股票业务域单独收口，后续新增指标、行情同步和技术咨询一律从这里扩展，不再反向挂回 foundation。
#[path = "import_stock_price_history.rs"]
pub mod import_stock_price_history;
#[path = "security_analysis_contextual.rs"]
pub mod security_analysis_contextual;
#[path = "security_analysis_fullstack.rs"]
pub mod security_analysis_fullstack;
#[path = "sync_stock_price_history.rs"]
pub mod sync_stock_price_history;
#[path = "technical_consultation_basic.rs"]
pub mod technical_consultation_basic;
