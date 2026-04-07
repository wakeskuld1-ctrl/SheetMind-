// 2026-04-07 CST: 这里先创建 retrieval engine 占位结构，原因是当前检索只应作为候选域内执行器存在。
// 目的：从模块层面避免把 retrieval 再次误用成 foundation 的系统入口。
#[derive(Debug, Clone, Default)]
pub struct RetrievalEngine;
