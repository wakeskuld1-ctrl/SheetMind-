// 2026-04-07 CST: 这里先创建 ontology schema 占位结构，原因是当前任务只要求先把 foundation
// 导航内核的模块边界挂起来，并通过导出测试验证入口已经可见。
// 目的：先建立最小可编译骨架，后续再按 TDD 逐步补概念、关系和校验逻辑。
#[derive(Debug, Clone, Default)]
pub struct OntologySchema;
