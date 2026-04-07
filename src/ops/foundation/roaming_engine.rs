// 2026-04-07 CST: 这里先创建 roaming engine 占位结构，原因是知识漫游是 foundation 主链中的独立阶段。
// 目的：先把候选域扩展能力独立出来，后续深度限制和关系限制都在这里收口。
#[derive(Debug, Clone, Default)]
pub struct RoamingEngine;
