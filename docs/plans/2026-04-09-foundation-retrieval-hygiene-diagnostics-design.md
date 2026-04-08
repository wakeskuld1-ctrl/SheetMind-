# Foundation Retrieval Hygiene Diagnostics Design

## 1. 背景

当前 foundation retrieval 已经具备：

- 稳定排序链
- `retrieve_with_diagnostics()`
- 命中原因与 tie-break 原因的可解释输出

但当前 diagnostics 仍然偏“为什么排成这样”，还没有覆盖“证据本身是否健康”。

这会带来一个交接缺口：

- 节点可能靠较多 `evidence_refs` 排得靠前
- 但其中可能包含重复证据
- 或者 locator 很弱
- 或者 source_ref 几乎没有语义

因此，这一轮继续沿 foundation retrieval 主线推进，补一层 evidence hygiene diagnostics。

## 2. 目标

本轮目标是在 `RetrievalDiagnostic` 中新增最小 hygiene 诊断能力，覆盖：

- 重复证据
- 弱 locator
- 弱 source_ref

本轮不做：

- 排序改动
- 命中判定改动
- CLI / Tool / GUI 承接
- 配置层扩张
- 业务线接线

## 3. 方案比较

### 方案 A：在 `RetrievalDiagnostic` 上直接增加 hygiene 字段

做法：

- 在当前 `RetrievalDiagnostic` 上新增：
  - `duplicate_evidence_ref_count`
  - `weak_locator_count`
  - `weak_source_ref_count`
  - `hygiene_flags`

优点：

- 变更最小
- 继续保持 diagnostics 只在 retrieval 内部演进
- 后续 AI 最容易直接消费

缺点：

- `RetrievalDiagnostic` 会继续变胖一些

### 方案 B：新增独立 `EvidenceHygieneDiagnostic`

做法：

- 从 `RetrievalDiagnostic` 中拆出 hygiene 子对象

优点：

- 结构更整齐

缺点：

- 对当前阶段偏重
- 有提前做小型子系统的倾向

### 方案 C：只增加告警字符串，不增加计数字段

做法：

- 只新增 `hygiene_flags: Vec<String>`

优点：

- 字段更少

缺点：

- 不够利于后续 AI 判断严重性
- 很难区分“一次问题”还是“多次问题”

### 结论

采用方案 A。

原因：

- 当前最需要的是最小、可解释、可测试的诊断增强
- 直接挂到 `RetrievalDiagnostic` 上最符合现在的 YAGNI 节奏
- 保留计数字段也更利于后续 AI 做进一步判断

## 4. 设计

### 4.1 新增字段

在 `RetrievalDiagnostic` 上新增：

- `duplicate_evidence_ref_count: usize`
- `weak_locator_count: usize`
- `weak_source_ref_count: usize`
- `hygiene_flags: Vec<RetrievalHygieneFlag>`

新增枚举：

- `RetrievalHygieneFlag::DuplicateEvidenceRefs`
- `RetrievalHygieneFlag::WeakLocator`
- `RetrievalHygieneFlag::WeakSourceRef`

### 4.2 诊断规则

#### 重复证据

判定口径：

- 同一节点内 `source_ref + locator` 完全相同，视为重复证据

输出：

- `duplicate_evidence_ref_count` 记录重复条目数量
- 有重复时追加 `DuplicateEvidenceRefs`

#### 弱 locator

最小启发式规则：

- 空 locator
- 无法解析的 locator
- 过宽范围 locator

本轮“过宽范围”只做最小规则，不引入配置：

- Excel/WPS 风格范围面积超过固定阈值时，视为 weak locator

#### 弱 source_ref

最小启发式规则：

- 归一化后为空
- 仅包含过泛占位词
- 有效语义不足，无法体现来源区分度

本轮仍保持最小启发式，不引入外部词典或复杂分类器。

### 4.3 合同边界

保持以下边界不变：

- hygiene diagnostics 只解释质量问题，不改变排序
- hygiene diagnostics 不参与 hit creation
- `retrieve()` 合同不变
- `RetrievalHit` 不新增字段
- 不引入 `RetrievalConfig`
- 不扩散到 CLI / Tool / GUI

## 5. 测试策略

严格走 TDD，先补红灯测试：

1. 识别同节点重复证据
2. 识别弱 locator
3. 识别弱 source_ref
4. 既有 diagnostics 合同继续保持兼容

## 6. 风险

- 如果弱 locator 阈值过激进，容易把正常大范围引用误标为风险
- 如果弱 source_ref 规则过宽，会把真实但简短的来源误标
- 如果 hygiene 被误接入排序，会破坏当前解释/行为分离

## 7. 完成标准

- `RetrievalDiagnostic` 能输出最小 hygiene 信息
- 红灯测试先失败再转绿
- 排序和命中行为不变
- foundation 最小回归集继续通过
- 交接手册、执行记录、任务日志同步更新
