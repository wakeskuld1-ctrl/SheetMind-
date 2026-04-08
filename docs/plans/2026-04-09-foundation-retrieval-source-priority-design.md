# Foundation Retrieval Source Priority Design

<!-- 2026-04-09 CST: 新增 Task 11 第二层设计文档。原因：retrieval 第一层排序增强已经完成，用户明确选择继续沿 source_ref 方向做第二层增强；目的：先冻结“source_ref 只做次级 tie-break，不反压文本相关性”的边界，避免 retrieval 过早演化成配置系统。 -->

## 目标

在 foundation 的 `RetrievalEngine` 中增加 `source_ref` 次级排序信号，让更像“主数据源”的证据来源在文本分数相同或接近时排得更靠前。

本轮只做：

1. 为 `source_ref` 增加固定优先级规则。
2. 只把它作为 retrieval 的次级排序信号。
3. 保持 `RetrievalHit` 合同不变。
4. 不引入 `RetrievalConfig`。

本轮不做：

- 让 `source_ref` 直接覆盖文本分数
- 为 `source_ref` 暴露配置入口
- 引入 provider / vector / embedding
- 修改 `NavigationPipeline` 合同

## 当前问题

在 Task 11 第一层之后，retrieval 已经有：

- title 权重
- phrase bonus
- seed concept bonus

但仍缺少一个与“证据来源质量”相关的次级信号。

这会带来两个问题：

1. 文本分数相同的节点，目前仍只能靠 `node_id` 稳定排序。
2. 更接近主数据源的节点，无法比说明/计划/派生来源得到更合理的优先级。

## 方案比较

### 方案 A：直接把 source_ref 做成主分数

做法：

- `source_ref` 直接参与主分数计算，权重接近 title/phrase 信号

优点：

- 增强效果明显

缺点：

- 容易反压文本相关性
- 现在过重，不符合当前小步策略

### 方案 B：source_ref 固定 bonus

做法：

- 给特定来源类型加固定 bonus
- bonus 直接加到主分数上

优点：

- 实现简单

缺点：

- 仍会把来源偏好混进主分数
- 长期不容易解释

### 方案 C：source_ref 分层作为 tie-break

做法：

- 文本分数仍然保持第一优先级
- 当文本分数相同或排序已进入 tie-break 阶段时，再比较 `source_ref` 优先级

优点：

- 最稳
- 不会破坏第一层文本增强
- 最符合当前 foundation 渐进增强节奏

缺点：

- 体感提升比主分数方案弱

## 选型

采用方案 C。

原因：

- 当前 foundation 还在做 retrieval 的第二小步增强，必须先守住“文本相关性优先”的主轴。
- 让 `source_ref` 做次级 tie-break，既能体现来源偏好，又不会让来源信号反客为主。
- 这也最符合用户当前想法：“支持来源优先，但不要把底座复杂化”。

## 规则设计

本轮使用固定优先级，不开放配置：

### Primary Source

更像主数据源、原始明细、基础表：

- 不含派生/说明关键词
- 例如：`sheet:sales_raw`、`sheet:invoice_detail`

### Derived / Summary Source

更像说明、摘要、趋势、报告：

- 包含 `summary`
- 包含 `trend`
- 包含 `report`
- 包含 `analysis`
- 包含 `derived`

### Planning Source

更像计划、预测、方案：

- 包含 `plan`
- 包含 `forecast`
- 包含 `scenario`

优先级原则：

1. Primary Source 最优先
2. Derived / Summary 次之
3. Planning 最后

## 作用方式

`source_ref` 只参与排序，不参与“是否命中”的判定。

也就是说：

- 没有文本命中的节点，仍然不能因为来源优先而变成 hit
- 只有已经命中的节点，才允许通过来源优先级调整前后顺序

## 测试策略

### 测试 1：同分时 primary source 优先

验证：

- 两个节点文本分数相同
- `sheet:sales_raw` 应排在 `sheet:trend_summary` 前面

### 测试 2：同分时 derived source 优先于 planning source

验证：

- 两个节点文本分数相同
- `sheet:summary_report` 应排在 `sheet:plan_forecast` 前面

### 测试 3：source priority 不能压过更高文本分数

验证：

- 一个节点文本分数更高但来源较弱
- 另一个节点文本分数更低但来源更强
- 结果仍应先按文本分数排序

## 风险控制

1. 不修改 hit 命中条件。
2. 不修改 `RetrievalHit` 字段。
3. 不新增 `RetrievalConfig`。
4. 不让来源优先级反压文本相关性。

## 完成标准

满足以下条件即可视为 Task 11 第二层完成：

- retrieval 新增 source priority 红灯测试
- 测试先红后绿
- retrieval 第一层排序增强继续保留
- foundation 最小回归集继续通过
