// 2026-04-07 CST: 这里先创建 knowledge graph store 占位结构，原因是图谱存储查询需要独立于 record 定义存在。
// 目的：后续可以在这里先落纯内存实现，再根据需要演进持久化层，而不影响上层模块边界。
#[derive(Debug, Clone, Default)]
pub struct KnowledgeGraphStore;
