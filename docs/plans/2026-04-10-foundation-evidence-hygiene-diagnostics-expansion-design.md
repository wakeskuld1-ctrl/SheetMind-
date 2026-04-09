# Foundation Evidence Hygiene Diagnostics Expansion Design

<!-- 2026-04-10 CST：新增 evidence hygiene diagnostics 扩展设计文档。原因：repository-level audit 已经落地，但当前 hygiene diagnostics 还停留在最小版，无法覆盖“缺失证据、节点内重复、弱 locator、弱 source_ref”这类更实际的仓库级问题。目的：在不越界到 migration executor 的前提下，把 foundation 审计层的证据质量诊断补厚一层。 -->

## 1. 目标

- 扩展 `RepositoryMetadataAudit` 的 `evidence hygiene diagnostics`
- 保持扩展发生在 foundation 仓库级审计层，不动业务层
- 继续保持只读诊断，不做自动 rewrite，不做 migration executor

## 2. 可选方案

### 方案 A：弱证据质量扩细

- 做法：在现有 `DuplicateEvidenceRef / WeakLocator / WeakSourceRef` 基础上，补更细规则与缺失类诊断
- 优点：最贴合当前路径，完全承接 repository audit
- 缺点：仍然只是诊断层，不产出迁移动作

### 方案 B：审计报告分级

- 做法：新增 severity、health score、summary 等报告层增强
- 优点：更像正式治理看板
- 缺点：当前更偏“展示增强”，不如先把基础诊断做实

### 方案 C：重复证据专项治理

- 做法：只深挖重复证据，不扩其它 hygiene 规则
- 优点：聚焦清楚
- 缺点：范围偏窄，不能一次把 evidence hygiene 底座补全

## 3. 采用方案

- 本轮采用 `方案 A：弱证据质量扩细`
- 原因：
  - 用户已明确选择 `方案A`
  - 它和当前 `Repository-Level Audit` 最连续
  - 先补底座规则，再谈报告分级或迁移动作，更符合当前 foundation 路线

## 4. 本轮范围

### 本轮要做

- 保留现有：
  - `DuplicateEvidenceRef`
  - `WeakLocator`
  - `WeakSourceRef`
- 新增：
  - `MissingEvidenceRef`
  - `DuplicateEvidenceRefWithinNode`
- 扩展弱质量诊断的原因分类：
  - `WeakLocator`：
    - `Blank`
    - `TooShort`
  - `WeakSourceRef`：
    - `Blank`
    - `TooShort`
    - `MissingNamespace`

### 本轮不做

- 自动修复 evidence_ref
- 自动补 source_ref / locator
- 复杂的正则级 locator 结构校验
- severity 打分系统
- foundation 与证券业务层联动
- migration executor

## 5. 核心设计

### 5.1 模型扩展

在 `src/ops/foundation/repository_metadata_audit.rs` 中新增：

- `RepositoryWeakLocatorReason`
- `RepositoryWeakSourceRefReason`

并扩展 `RepositoryEvidenceHygieneDiagnostic`：

- `MissingEvidenceRef { node_id }`
- `DuplicateEvidenceRefWithinNode { node_id, source_ref, locator, occurrence_count }`
- `WeakLocator { node_id, source_ref, locator, reason }`
- `WeakSourceRef { node_id, source_ref, locator, reason }`

### 5.2 规则定义

#### MissingEvidenceRef

- 当节点 `evidence_refs` 为空时输出

#### DuplicateEvidenceRefWithinNode

- 同一节点内，同一 `source_ref + locator` 重复出现两次及以上时输出

#### WeakLocator

- `Blank`：
  - `locator.trim().is_empty()`
- `TooShort`：
  - 非空，但 `trim()` 后长度小于 3

#### WeakSourceRef

- `Blank`：
  - `source_ref.trim().is_empty()`
- `TooShort`：
  - 非空，但 `trim()` 后长度小于 4
- `MissingNamespace`：
  - 非空且长度足够，但不包含 `:`

## 6. 数据流

1. 遍历 repository 全节点
2. 若节点无 `evidence_refs`，输出 `MissingEvidenceRef`
3. 遍历节点内全部 `evidence_refs`
4. 先做节点内重复登记
5. 再做全仓库跨节点重复登记
6. 同时做 `locator` / `source_ref` 弱质量检查
7. 统一输出 diagnostics 列表

## 7. TDD 计划

- 修改 `tests/repository_metadata_audit_unit.rs`
- 第一批红测覆盖：
  - 缺失 evidence_ref
  - 同节点内重复 evidence_ref
  - `WeakLocator::TooShort`
  - `WeakSourceRef::TooShort`
  - `WeakSourceRef::MissingNamespace`

## 8. 验证范围

```powershell
cargo test --test repository_metadata_audit_unit -- --nocapture
cargo test --test repository_metadata_audit_unit --test metadata_validator_unit --test metadata_schema_registry_unit --test metadata_schema_versioning_unit --test metadata_migration_contract_unit --test knowledge_repository_unit --test knowledge_ingestion_unit --test knowledge_bundle_unit -- --nocapture
```

## 9. 这轮完成后的能力计数口径

按 `evidence hygiene diagnostics` 子线统计：

- 原有能力：3 项
  - `DuplicateEvidenceRef`
  - `WeakLocator`
  - `WeakSourceRef`
- 本轮计划新增能力：4 项
  - `MissingEvidenceRef`
  - `DuplicateEvidenceRefWithinNode`
  - `WeakLocator` 原因分型
  - `WeakSourceRef` 原因分型

## 10. 下一步边界

- 本轮完成后，优先还可继续做：
  - locator 更细结构规则
  - source_ref 更细命名规则
  - diagnostics severity / summary
- 不要把这轮 evidence hygiene 扩细误判成开始做自动迁移
