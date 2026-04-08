# Foundation Navigation Pipeline Config Design

<!-- 2026-04-08 CST: 新增 pipeline 配置化设计文档。原因：当前 navigation_pipeline 已打通最小闭环，但漫游关系、深度和候选预算仍硬编码在实现里。目的：先把“样板默认值”提升为“显式配置合同”，继续沿底座主线渐进增强。 -->

## 目标

把 `navigation_pipeline` 从“固定策略样板”推进到“带最小显式配置的可调底座入口”。

本轮只做：

1. 为 pipeline 增加一个最小配置对象。
2. 保持 `NavigationPipeline::new()` 的默认行为不变。
3. 允许调用方覆盖以下三个参数：
   - `allowed_relation_types`
   - `max_depth`
   - `max_concepts`
4. 补齐对应测试与交接记录。

本轮不做：

- 拆分 retrieval 配置
- 引入 profile 预设
- 接入 CLI / Tool
- 把业务层策略向 foundation 反向渗透

## 当前问题

当前 `navigation_pipeline` 已能运行，但以下三项策略仍写死在 `run()` 内：

- allowed relation whitelist
- `max_depth`
- `max_concepts`

这会带来三个问题：

1. 调用方无法针对不同场景调节漫游范围。
2. 测试只能验证一套固定策略。
3. 后续继续增强 retrieval 时，容易被迫回头做重构。

## 方案比较

### 方案 A1：最小配置对象

做法：

- 新增 `NavigationPipelineConfig`
- 保持 `NavigationPipeline::new()` 继续使用默认配置
- 新增 `NavigationPipeline::new_with_config()`
- 配置对象只收 `allowed_relation_types / max_depth / max_concepts`

优点：

- 改动最小
- 与当前代码最贴合
- 不会把配置化做成“小框架”

缺点：

- 当前只覆盖漫游侧
- retrieval 侧配置留到后续任务再接

### 方案 A2：双配置对象

做法：

- 同时引入 `NavigationPipelineConfig`
- 进一步拆出 `RoamingConfig`
- 进一步拆出 `RetrievalConfig`

优点：

- 结构更分层

缺点：

- 以当前阶段来说偏重
- 会让 Task 10 过早引入更多抽象

### 方案 A3：Profile 包装

做法：

- 只暴露 `conservative / balanced / broad` 这类 profile

优点：

- 对上层更友好

缺点：

- 现在过早
- 容易把真实参数合同藏起来

## 选型

采用方案 A1。

原因：

- 当前最需要的是把硬编码策略变成可调合同，而不是引入新的配置体系。
- A1 足够支撑后续 Task 11 的 retrieval 增强。
- A1 最符合当前阶段的 `DRY / KISS / YAGNI`。

## 结构设计

新增：

- `NavigationPipelineConfig`

最小字段：

- `allowed_relation_types: Vec<OntologyRelationType>`
- `max_depth: usize`
- `max_concepts: usize`

最小行为：

- `Default` 提供当前默认策略
- builder 风格方法允许局部覆盖

## 构造方式

保留：

- `NavigationPipeline::new(ontology_store, graph_store)`

新增：

- `NavigationPipeline::new_with_config(ontology_store, graph_store, config)`

设计原则：

- 不破坏当前调用面
- 默认调用仍然零学习成本
- 自定义调用显式表达“我知道我要改策略”

## 测试策略

### 默认配置保持不变

继续复用当前 happy path：

- 问题命中 `revenue`
- 通过 `Supports` 漫游到 `trend`
- 命中 2 个节点

### 自定义配置生效

新增一条集成测试，使用：

- `allowed_relation_types = [DependsOn]`
- `max_depth = 1`
- `max_concepts = 8`

预期：

- 不会沿 `Supports` 扩展
- `roaming_path` 为空
- 只命中 `revenue` 节点

## 风险控制

1. 不拆 retrieval config。
2. 不提前引入 profile 抽象。
3. 不修改 foundation 与业务层边界。
4. 不因为配置化而改变当前默认闭环行为。

## 完成标准

满足以下条件即可视为 Task 10 第一节完成：

- `NavigationPipelineConfig` 正式存在
- `new()` 默认行为不变
- `new_with_config()` 可用
- 自定义配置测试通过
- foundation 最小回归集继续通过
