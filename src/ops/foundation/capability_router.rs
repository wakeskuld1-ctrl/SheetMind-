// 2026-04-07 CST: 这里先创建 capability router 占位结构，原因是问题解析和种子概念定位是主链路起点。
// 目的：先把“路由”从“检索”中拆出来，后续实现时可以保持 ontology first 的架构约束。
#[derive(Debug, Clone, Default)]
pub struct CapabilityRouter;
