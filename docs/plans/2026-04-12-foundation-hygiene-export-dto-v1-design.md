# 2026-04-12 Foundation Hygiene Export DTO V1 Design

## 背景

foundation `RepositoryMetadataAudit` 当前已经具备较完整的内部聚合能力，输出包含：
- `issues`
- `hygiene_diagnostics`
- `hygiene_summary`
- `hygiene_views`
- `hygiene_reason_views`

这些能力已经足够支撑 foundation 底座内部治理，但对于后续 AI、上层编排、CLI 封装或未来 GUI 来说，当前仍然存在一个明显风险：

上层如果直接依赖 `RepositoryMetadataAuditReport` 的内部结构，就会把底座内部实现细节泄漏成外部契约。这样后面一旦我们继续补聚合层、补字段、调整内部组织方式，就容易出现：

- 上层调用与底座内部结构强耦合
- AI 需要反复猜测哪些字段是稳定契约，哪些只是内部实现
- 后续扩展时不敢改内部结构，因为担心误伤外部调用

## 目标

在不重构当前 foundation 审计主流程的前提下，增加一层稳定的 `v1` 导出 DTO，让 `RepositoryMetadataAudit` 能对外提供一个明确、可序列化、可长期维护的契约面。

这层 DTO 需要满足：
- 对外字段语义稳定
- 排序语义稳定
- reason / severity / count 口径稳定
- 不要求上层理解内部 Rust 结构细节
- 不改变当前 audit 内部主线行为

## 非目标

- 不重写 `RepositoryMetadataAudit` 聚合逻辑
- 不删除现有 `RepositoryMetadataAuditReport`
- 不在这轮引入新的业务治理规则
- 不做 GUI 或业务层格式设计
- 不做跨模块大重构

## 方案选择

本轮采用 `方案 C`：
- 新增 `v1` 版本化 DTO
- 以镜像导出为主，不在本轮过度瘦身
- 通过单独转换层把内部 report 映射为对外稳定结构

原因是：
- 能保证对外契约稳定
- 又不会把本轮工作做成一次架构重写
- 与用户“不要一来就重构”的项目纪律一致

## 设计

### 1. 新增对外 DTO 类型

在 foundation 范围内新增一组导出结构，命名明确带版本后缀或版本前缀，例如：
- `RepositoryMetadataAuditExportDtoV1`
- `RepositoryMetadataAuditIssueDtoV1`
- `RepositoryEvidenceHygieneSummaryDtoV1`
- `RepositoryEvidenceHygieneViewsDtoV1`
- `RepositoryEvidenceHygieneReasonViewsDtoV1`

设计原则：
- DTO 只暴露稳定字段
- 枚举类语义对外统一导出为稳定字符串键，避免上层依赖 Rust enum 本体
- 数组顺序沿用当前已被测试锁定的稳定排序语义

### 2. 保持“内部模型”和“外部契约”分层

当前 `RepositoryMetadataAuditReport` 继续作为内部模型保留，不直接被删除或替换。

新增一层导出转换：
- `RepositoryMetadataAuditReport::to_export_dto_v1()`
或
- `RepositoryMetadataAuditExportDtoV1::from_report(&report)`

本轮更推荐后者，因为：
- DTO 对内部模型是单向依赖
- 更符合“对外契约层不反向污染内部模型”的边界
- 更容易在未来继续新增 `v2`

### 3. DTO 字段口径

DTO 应镜像当前 foundation 已锁定的语义，但会把语义更明确地表达出来：

- summary 中的 reason counts
  - 继续表示“原始 diagnostic 条数”
- reason views 中的 `diagnostic_count`
  - 当前继续表示“去重后的节点数”
- `affected_node_count`
  - 当前继续与上者一致
- `node_ids`
  - 继续按字典序稳定输出

这意味着本轮不是重新定义口径，而是把当前口径明确导出并锁定。

### 4. 字段组织原则

DTO 结构建议保留四层：
- `metadata_summary`
- `hygiene_summary`
- `hygiene_views`
- `hygiene_reason_views`

如果当前 `issues` 与 `hygiene_diagnostics` 也需要导出，则应继续保留，但使用 DTO 子结构表达，不直接泄漏内部枚举。

这样上层调用时可以根据需要选择：
- 只看 summary
- 看 grouped views
- 看 reason views
- 必要时看 detail

## 错误处理

本轮 DTO 转换不引入 fallible API。

原因：
- 当前 report 已经是内存内合法聚合结果
- DTO 转换只是结构映射，不涉及 IO、不涉及外部依赖
- 如果将来需要序列化为 JSON，再在更上层处理序列化错误即可

## 测试策略

本轮严格走 TDD。

### 第一层：基础导出测试
- 基于 `sample_repository()`
- 断言 `report -> export dto v1` 的字段映射正确
- 断言字符串键、计数、排序符合当前契约

### 第二层：大仓库稳定性导出测试
- 复用 `large_stability_repository()`
- 断言导出 DTO 中：
  - severity views 顺序稳定
  - node views 顺序稳定
  - reason views 的 `node_ids` 顺序稳定

### 第三层：边界语义导出测试
- 复用 `repeated_reason_boundary_repository()`
- 断言 summary 与 reason views 的计数口径按当前已锁定语义导出

## 预期结果

本轮完成后，foundation 底座将获得一层明确的对外稳定契约：
- 上层 AI 不再需要直接理解内部 report
- 后续如果内部聚合结构调整，只要 `v1` 适配层不变，对外调用就不受影响
- 后续可以在同一路径继续扩展 `v2`，而不是反复重写上层接入方式
