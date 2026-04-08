# Foundation Retrieval Enhancement Design

<!-- 2026-04-08 CST: 新增 Task 11 设计文档。原因：Task 10 已完成 pipeline 配置化收口，foundation 默认下一步转入 retrieval 增强第一层；目的：先冻结“只增强排序，不拆配置、不接 CLI”的边界，避免 retrieval 提前膨胀成第二套系统。 -->

## 目标

把当前 foundation 中的 `RetrievalEngine` 从“候选域内 token 交集计数器”推进到“最小可演进证据排序器”。

本轮只做：

1. 增强候选域内命中的排序表达力。
2. 保持 `RetrievalHit` 输出结构不变。
3. 不引入 `RetrievalConfig`。
4. 不接入 CLI / Tool / GUI。

本轮不做：

- 拆分 retrieval 配置对象
- 引入向量检索
- 引入 embedding/provider
- 修改 pipeline 对外合同
- 增加复杂 explainability 输出对象

## 当前问题

当前 [retrieval_engine.rs](/C:/Users/wakes/.codex/worktrees/Excel_Skill/codex-security-post-meeting-package-binding/src/ops/foundation/retrieval_engine.rs) 的命中打分过于单薄：

- `title` 与 `body` 权重相同
- 不识别完整短语命中
- 不利用 seed concept 与漫游 concept 的层次差异
- 最终排序几乎完全依赖 token 交集数量

这会带来三个问题：

1. 标题高度相关的节点，可能只因为正文 token 多寡而被压后。
2. 完整问题短语命中，无法比零散 token 命中得到更稳定的排序优势。
3. 种子概念节点与漫游扩展节点缺少优先级差异，不利于保持“先核心、后补全”的检索语义。

## 方案比较

### 方案 A：最小排序增强

做法：

- 保持 `RetrievalEngine` 无状态
- 仅增强内部评分
- 引入三类轻量信号：
  - `title` 命中权重大于 `body`
  - 完整短语命中额外加分
  - 命中 seed concept 的节点额外加分

优点：

- 改动最小
- 不改变现有输出结构
- 最符合当前主线节奏

缺点：

- 还不是可配置检索器

### 方案 B：排序增强 + 最小解释字段

做法：

- 在方案 A 基础上，为 `RetrievalHit` 增加 `matched_terms` 或 `score_breakdown`

优点：

- 更利于后续 evidence 与 UI 展示

缺点：

- 会扩大输出合同
- 影响面明显大于当前任务需要

### 方案 C：直接引入 RetrievalConfig

做法：

- 把标题权重、正文权重、短语 bonus、seed bonus 做成可配置对象

优点：

- 长期可调性更强

缺点：

- 明显过早
- 与当前“不要过早拆 RetrievalConfig”的项目规则冲突

## 选型

采用方案 A。

原因：

- 当前最需要的是把排序信号补到“足够稳定”，不是把 retrieval 做成另一套配置系统。
- 方案 A 可以在不改变 pipeline 合同的前提下，先把排序质量向前推进一层。
- 方案 A 最符合当前 foundation 的 `KISS / YAGNI / 渐进增强` 原则。

## 结构设计

保持以下对象不变：

- `RetrievalEngine`
- `RetrievalHit`
- `RetrievalEngineError`

只在 `RetrievalEngine` 内部新增最小辅助逻辑：

- `normalized_text()`：把标题/正文与问题放入一致的归一化文本空间
- `seed_concept_ids()`：从 `CandidateScope` 推导 seed concepts
- `score_node()`：组合多种轻量信号得到最终分数

## 评分规则

本轮建议采用固定权重策略：

1. `title` token 命中权重大于 `body`
2. 问题完整短语命中 `title/body` 时额外加分
3. 节点 concept 命中 seed concept 时额外加分

说明：

- 这些权重先固定在实现里
- 本轮目标是验证“信号有效”，不是暴露调参入口

## Seed Concept 推导

本轮不修改 `retrieve()` 的签名，不额外引入 route 参数。

seed concept 的最小推导规则：

1. 从 `scope.path` 提取所有 `to_concept_id`
2. `scope.concept_ids` 中未出现在 `to_concept_id` 集合里的概念，视为 seed concepts
3. 如果 `scope.path` 为空，则把当前 `concept_ids` 全部视为 seed concepts

这样做的原因是：

- 不改变当前 pipeline 数据流
- 足够支撑本轮“种子优先于漫游扩展”的排序合同

## 测试策略

### 测试 1：标题优先于正文

验证：

- 当一个节点只在 `title` 强命中，另一个节点只在 `body` 命中时
- 标题命中的节点应排在前面

### 测试 2：短语优先于分散 token

验证：

- 当问题是一个完整短语时
- 完整短语命中的节点应优先于只命中相同 token 但不成短语的节点

### 测试 3：seed concept 优先于漫游概念

验证：

- 当 seed concept 节点与漫游扩展 concept 节点在文本命中接近时
- seed concept 节点应排在前面

## 风险控制

1. 不修改 `RetrievalHit` 合同。
2. 不修改 `NavigationPipelineConfig`。
3. 不提前引入 `RetrievalConfig`。
4. 不让 retrieval 反过来侵入 route / roam 的边界。

## 完成标准

满足以下条件即可视为 Task 11 第一层完成：

- retrieval 新增三条排序增强测试
- 测试先红后绿
- 现有 `navigation_pipeline_integration` 继续通过
- foundation 最小回归集继续通过
- AI handoff 明确写明 retrieval 已进入“第一层排序增强”
