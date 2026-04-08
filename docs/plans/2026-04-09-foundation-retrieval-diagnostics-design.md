# Foundation Retrieval Diagnostics Design

## 1. 背景

当前 foundation retrieval 已经完成三层排序增强：

`文本分数 -> source priority -> evidence_refs 数量 -> locator 精度 -> node_id`

但当前排序链虽然已经更稳定，仍然存在一个交接缺口：

- 后续 AI 或开发者能看到排序结果
- 但看不到某个命中为什么排在前面
- 一旦继续增强 retrieval，容易误把 tie-break 信号改成主分数，或者把已有层级顺序串台

因此，这一轮不继续扩排序规则，而是给 retrieval 增加“可解释诊断能力”。

## 2. 目标

本轮目标是为 foundation retrieval 提供一套仅限底座内部使用的诊断输出，能够回答：

- 某个命中节点为什么命中
- 某个命中节点为什么排在当前位置
- 当前排序链上每一层信号分别取到了什么值

本轮不追求：

- CLI 输出
- Tool 输出
- GUI 展示
- 新配置层
- 业务线接线

## 3. 方案选择

### 方案 A：只在 `retrieval_engine.rs` 内新增 diagnostics API

做法：

- 保持现有 `retrieve()` 合同不变
- 新增 `retrieve_with_diagnostics()` 或等价的 foundation 内部 API
- 返回 `hits + diagnostics` 的并行结果

优点：

- 变更范围最小
- 不会提前扩张 pipeline / assembler 合同
- 最符合 foundation 内部增强原则

缺点：

- 诊断结果暂时只能由底座调用方主动消费

### 方案 B：把 diagnostics 直接挂到 `NavigationEvidence`

做法：

- 修改 `NavigationEvidence`
- 让 `NavigationPipeline` 直接输出 retrieval 诊断

优点：

- 更接近未来统一输出

缺点：

- 会提前扩张 pipeline 合同
- 当前阶段略重，容易把 foundation 输出面做大

### 结论

采用方案 A。

原因：

- 当前目标是先把 retrieval 自己变得可解释，而不是提前设计应用层承接面
- 继续保持 foundation 小步增强，不把 diagnostics 扩散到 CLI / Tool / GUI

## 4. 设计

### 4.1 新增结构

在 `src/ops/foundation/retrieval_engine.rs` 内新增：

- `RetrievalExecution`
- `RetrievalDiagnostic`

建议合同：

- `RetrievalExecution.hits` 仍然沿用当前 `Vec<RetrievalHit>`
- `RetrievalExecution.diagnostics` 与最终排序后的 hits 一一对齐
- `RetrievalDiagnostic` 只表达可解释信号，不引入行为配置

### 4.2 诊断内容

每条 `RetrievalDiagnostic` 至少记录：

- `node_id`
- `title_overlap`
- `body_overlap`
- `phrase_bonus`
- `seed_bonus`
- `text_score`
- `source_priority`
- `evidence_ref_count`
- `best_locator`
- `locator_priority`

其中：

- `text_score` 负责解释“为什么命中”
- `source_priority`、`evidence_ref_count`、`locator_priority` 负责解释“为什么在同分里排成这样”

### 4.3 排序与合同边界

保持以下合同不变：

- `retrieve()` 继续返回 `Vec<RetrievalHit>`
- `RetrievalHit` 不新增字段
- diagnostics 不参与 hit creation
- diagnostics 不改变排序规则，只解释既有排序规则
- 不引入 `RetrievalConfig`

### 4.4 实现方式

建议把当前散落的排序信号收口到一个内部候选结构，例如：

- 先构建 `ScoredRetrievalCandidate`
- 同时保存：
  - `RetrievalHit`
  - `RetrievalDiagnostic`
  - 排序所需的内部 key
- 排序后再拆成 `hits` 与 `diagnostics`

这样可以避免：

- 排序逻辑和 diagnostics 各算一遍
- 后续改一边忘一边

## 5. 测试策略

严格走 TDD。

先补红灯测试，至少覆盖：

1. `retrieve_with_diagnostics()` 会返回与 hits 对齐的 diagnostics
2. diagnostics 会完整暴露 title/body/phrase/seed/source/evidence/locator 信号
3. `retrieve()` 旧合同保持不变

## 6. 风险

- 如果 diagnostics 字段太多，可能把底座内部调试对象误做成长期对外合同
- 如果让 pipeline 也跟着改，容易把本轮从 retrieval 内部增强扩散成输出层改造
- 如果 diagnostics 再参与排序计算，会破坏“解释”和“行为”分离

## 7. 本轮完成标准

- retrieval engine 新增 diagnostics API
- 现有 `retrieve()` 合同不变
- 新增 diagnostics 测试先红后绿
- foundation 最小回归集继续通过
- 交接手册、执行记录、任务日志同步更新
