// 2026-03-29 CST: 这里建立 GUI 桥接层模块，原因是桌面界面不能直接散落调用底层业务服务；
// 目的：把 GUI 需要的授权摘要、Tool 调用和视图模型集中收口在单独桥接层。
pub mod license_bridge;
pub mod tool_runner;
pub mod view_models;
