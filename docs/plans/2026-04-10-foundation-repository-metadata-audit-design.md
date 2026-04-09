# Foundation Repository Metadata Audit Design

<!-- 2026-04-10 CST：新增 repository-level audit 设计文档。原因：foundation metadata 治理已经完成 validator linkage，下一步需要把节点级信号提升到仓库级可审计报告，但又不能越界到 migration executor。目的：固定本轮只做只读聚合审计、最小 hygiene diagnostics 和正式报告契约。 -->

## 1. 目标

- 在 `foundation` 主线上新增 repository-level metadata audit
- 复用现有 `MetadataValidator`，把节点级 issue 聚合成仓库级审计报告
- 新增最小 evidence hygiene diagnostics：
  - 重复证据
  - 弱 locator
  - 弱 source_ref
- 保持本轮为只读审计，不做自动 rewrite，不做 migration executor，不接证券业务层

## 2. 方案对比

### 方案 A：轻量审计清单

- 做法：只遍历全部节点并返回扁平 issue 列表
- 优点：实现最快、改动最小
- 缺点：缺少按 issue 类型、按 concept 的聚合视角，也无法自然承载 hygiene diagnostics

### 方案 B：聚合审计报告

- 做法：新增正式 audit 模块，输出统一报告对象，包含摘要、明细、聚合计数和 hygiene diagnostics
- 优点：最符合当前阶段，既复用 validator，又把 repository 级治理视角正式补齐
- 缺点：比轻量清单多一层报告模型和测试，工作量中等

### 方案 C：审计加迁移建议

- 做法：在聚合报告之外，额外输出 rewrite 建议和替换动作
- 优点：更接近后续治理闭环
- 缺点：容易越界到 migration executor，与当前已批准范围不符

## 3. 采用方案

- 本轮采用 `方案 B：聚合审计报告`
- 原因：
  - 用户已明确选择 `方案B`
  - 当前阶段需要的是 repository 级观察与治理入口，而不是自动迁移
  - 后续 evidence hygiene diagnostics 也更适合挂在正式报告模型里，而不是扁平 issue 清单

## 4. 范围边界

### 本轮要做

- 新增 repository-level metadata audit 入口
- 审计输入：
  - `KnowledgeRepository`
  - `MetadataSchema`
- 审计输出：
  - 仓库摘要
  - 节点 issue 明细
  - issue 类型聚合计数
  - concept 维度聚合计数
  - hygiene diagnostics 明细
- 最小 hygiene 规则先固定为：
  - 同一 `source_ref + locator` 在多个节点重复出现，记为重复证据
  - `locator` 为空或全空白，记为弱 locator
  - `source_ref` 为空或全空白，记为弱 source_ref

### 本轮不做

- 自动 rewrite
- dry-run migration
- migration executor
- repository 自动修复
- 更复杂的 locator 质量评分
- foundation 与证券业务层联动

## 5. 核心设计

### 5.1 新模块

- 新增 `src/ops/foundation/repository_metadata_audit.rs`
- 在 `src/ops/foundation.rs` 暴露模块出口

### 5.2 核心对象

- `RepositoryMetadataAuditReport`
  - `total_nodes`
  - `audited_nodes`
  - `issue_count`
  - `issues`
  - `issue_type_counts`
  - `concept_issue_counts`
  - `hygiene_diagnostics`
- `RepositoryMetadataAuditIssue`
  - `node_id`
  - `issue`
- `RepositoryEvidenceHygieneDiagnostic`
  - `DuplicateEvidenceRef`
  - `WeakLocator`
  - `WeakSourceRef`

### 5.3 数据流

1. 遍历 `KnowledgeRepository.bundle().nodes`
2. 对每个节点调用 `MetadataValidator::validate_node(...)`
3. 收集节点 issue 明细
4. 根据 issue 明细聚合：
   - issue 类型计数
   - concept 维度计数
5. 单独遍历全部 `evidence_refs`
6. 生成最小 hygiene diagnostics
7. 输出统一报告对象

## 6. 语义约定

### 6.1 issue 类型聚合

- 以 `MetadataValidationIssue` 的变体名作为聚合键
- 本轮不引入额外字符串枚举层，避免和现有 validator 重复建模

### 6.2 concept 计数

- 仅统计带 `concept_id` 的 issue：
  - `MissingConceptPolicy`
  - `MissingRequiredField`
  - `DisallowedField`
- 不强行给 `InvalidValueType / InvalidAllowedValue / AliasFieldUsed / DeprecatedFieldUsed` 虚构 concept 归属

### 6.3 duplicate evidence 语义

- 同一 `source_ref + locator` 出现在两个及以上节点上，即输出重复证据诊断
- 本轮只做“发现并报告”，不推断哪一条应该保留

## 7. TDD 计划

- 新增测试文件：`tests/repository_metadata_audit_unit.rs`
- 第一批红测锁定 4 类行为：
  - 能聚合多个节点的 validator issue
  - 能按 issue 类型做计数
  - 能按 concept 做计数
  - 能输出最小 hygiene diagnostics

## 8. 验证范围

```powershell
cargo test --test repository_metadata_audit_unit -- --nocapture
cargo test --test repository_metadata_audit_unit --test metadata_validator_unit --test metadata_schema_registry_unit --test metadata_schema_versioning_unit --test metadata_migration_contract_unit --test knowledge_repository_unit --test knowledge_ingestion_unit --test knowledge_bundle_unit -- --nocapture
```

## 9. 下一步边界

- 本轮完成后，foundation metadata 治理将具备：
  - registry
  - validator
  - repository audit
- 下一步仍然应优先考虑：
  - 扩细 hygiene diagnostics
  - 或在再次获批后讨论 migration executor
- 不要把本轮 repository audit 误判成自动迁移的开始
