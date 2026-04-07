// 2026-04-07 CST: 这里先创建 knowledge record 占位结构，原因是知识节点与知识边是 foundation
// 导航内核的独立数据模型，不应继续散落在其他业务模块中。
// 目的：先把图谱数据模型的命名空间固定下来，后续再补节点、边和证据引用结构。
#[derive(Debug, Clone, Default)]
pub struct KnowledgeRecord;
