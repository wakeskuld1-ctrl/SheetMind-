// 2026-04-07 CST: 这里先创建 ontology store 占位结构，原因是 schema 定义和 schema 查询职责要分离。
// 目的：先固定模块落点，下一步再把概念索引、别名索引和关系读取能力逐步补进来。
#[derive(Debug, Clone, Default)]
pub struct OntologyStore;
