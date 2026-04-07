// 2026-04-07 CST: 这里先创建 evidence assembler 占位结构，原因是最终输出装配需要与检索和漫游解耦。
// 目的：后续可以统一收敛 route、path、hits 和 citations，而不把输出拼装散落到多个模块里。
#[derive(Debug, Clone, Default)]
pub struct EvidenceAssembler;
