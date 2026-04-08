# Foundation Navigation Pipeline Design

<!-- 2026-04-08 CST: 新增 foundation navigation pipeline 设计文档。原因：底座已经具备 route、roam、retrieve、assemble 四个段落，但还缺一个正式入口把它们收成闭环。目的：明确 Task 8 与 Task 9 的承接方式，避免继续以测试手工拼装代替正式实现。 -->

## 目标

在 foundation 主线内增加一个纯内存、业务无关、可测试的 `NavigationPipeline`，把以下四段串成最小闭环：

1. `CapabilityRouter`
2. `RoamingEngine`
3. `RetrievalEngine`
4. `EvidenceAssembler`

输出目标是一个稳定的 `NavigationEvidence`，而不是裸露的中间结果集合。

## 本轮范围

本轮只做：

- `NavigationPipeline`
- `NavigationPipelineError`
- 纯内存 happy path 集成测试
- route 失败与 retrieve 失败阶段边界测试
- foundation 交接与执行记录补齐

本轮不做：

- CLI / Tool 接入
- GUI 接入
- 向业务线暴露导航能力
- provider / embedding / vector
- profile、配置分层或复杂调度

## 方案比较

### 方案 A：最小 pipeline 直连

做法：

- 直接在 foundation 内新增 `NavigationPipeline`
- `run(question)` 内顺序执行 route -> roam -> retrieve -> assemble
- 错误按阶段抬升

优点：

- 改动最小
- 最适合当前 Task 8 / 9 的收口节奏
- 有利于先建立正式闭环合同

缺点：

- 当前策略仍是固定值
- 后续配置化要继续往前推一节

### 方案 B：先抽 orchestration trait

做法：

- 先定义统一 orchestration trait
- 再为 pipeline 提供默认实现

优点：

- 理论上更易扩展

缺点：

- 当前明显过度设计
- 会把“先跑通闭环”变成“先造抽象层”

### 方案 C：直接接 CLI

做法：

- 一边做 pipeline，一边补 CLI / Tool 入口

优点：

- 上层可立即调用

缺点：

- 会把底座验证和应用接入混在一起
- 不利于隔离 foundation 与业务线

## 选型

采用方案 A。

原因：

- 当前最需要的是正式闭环，不是接口抽象或上层接入。
- 方案 A 最符合用户明确要求的“沿现有架构继续推进，不要每次接手就重构”。
- 方案 A 最利于后续再以小步方式进入配置化与 retrieval 增强。

## 数据流

基础数据流为：

`question -> route -> candidate scope -> retrieval hits -> evidence assembly`

细化如下：

1. `CapabilityRouter` 根据问题定位种子概念。
2. `RoamingEngine` 以种子概念扩展候选域。
3. `RetrievalEngine` 在候选域内执行检索。
4. `EvidenceAssembler` 把 route、path、hits、citations、summary 收成单个结果对象。

## 错误边界

本轮只保留三类阶段错误：

- `NavigationPipelineError::Route`
- `NavigationPipelineError::Roam`
- `NavigationPipelineError::Retrieve`

这样做的原因是当前阶段最重要的是保留“失败发生在哪一层”的边界感，便于测试与后续交接。

## 测试策略

### Happy Path

验证：

- 问题命中 `revenue`
- 漫游通过 `Supports` 到 `trend`
- retrieval 命中两个节点
- 输出中带有 citations 与 summary

### Route Error

验证：

- 当问题无法命中概念时，pipeline 显式返回 `Route` 错误

### Retrieve Error

验证：

- 当概念命中但候选域里没有匹配证据时，pipeline 显式返回 `Retrieve` 错误

## 风险控制

1. 不在本轮引入配置体系。
2. 不在本轮引入 CLI。
3. 不在本轮接业务线。
4. 不反向修改已完成的 ontology / roaming / retrieval 模块边界。

## 完成标准

满足以下条件即可视为本轮完成：

- `NavigationPipeline` 正式存在
- `run(question)` 返回 `NavigationEvidence`
- 三类阶段边界测试可用
- foundation 最小回归集继续通过
- 交接文档清楚写明默认继续 foundation 主线
