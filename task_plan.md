# 任务计划

<!-- 2026-04-08 CST: 重写 task_plan。原因：原文件仍停留在七席委员会旧任务，已经不能反映当前 foundation 通用标准能力工作。目的：把当前任务范围、非目标、完成状态和后续建议统一收口。 -->

## 当前任务目标

- 在 foundation 线上继续推进 Phase 3 第一阶段，补齐 `Metadata Schema Registry`。
- 本轮只做“字段注册 + concept policy 绑定”，不做 validator 和 versioning。
- 保持 foundation 仍为独立通用层，不接证券分析主链。

## 当前任务阶段

| 阶段 | 状态 | 说明 |
| --- | --- | --- |
| P1 范围收口 | 完成 | 已按用户确认选择方案 B，只做字段注册与 concept policy 绑定。 |
| P2 红测建立 | 完成 | 已新增 `metadata_schema_registry_unit`，先钉住字段定义、concept 绑定和未知字段引用错误。 |
| P3 最小实现 | 完成 | 已补 `MetadataValueType`、`MetadataFieldDefinition`、`ConceptMetadataPolicy`、`MetadataSchema`。 |
| P4 定向回归 | 完成 | `metadata_schema_registry_unit` 已全绿。 |
| P5 文档与日志 | 完成 | 已同步 README、AI_HANDOFF、计划文档、任务日志与发现记录。 |

## 已完成范围

- 标准知识包：`KnowledgeBundle`
- 标准仓储：`KnowledgeRepository`
- 通用 metadata 精确过滤：`MetadataFilter`
- 节点 metadata 标准化：`KnowledgeNode.metadata`
- 标准导入能力：`knowledge_ingestion`
- 扩展过滤能力：`MetadataFilter` 多字段 AND + concept scope
- 标准布局能力：`bundle.json + repository.manifest.json`
- 元数据管理能力：`MetadataSchema + ConceptMetadataPolicy`

## 非目标

- 不做证券分析业务适配。
- 不做向量检索。
- 不做高级 metadata DSL。
- 不做业务原始文件直接入库。
- 不做 foundation 与业务主链整合。

## 已知约束

- 不能误改用户已有证券分析链相关脏改动。
- Windows 上大块 `apply_patch` 可能失败，需要分块修改。
- Windows 下 `cargo test` 偶发会被残留 `excel_skill.exe` / `cargo` 进程锁住并触发 `os error 5`。
- 当前 `task-journal` 脚本在本地 PowerShell 编码环境下解析失败，需要手工追加 `CHANGELOG_TASK.MD`。

## 下一阶段建议

## 2026-04-08 最新状态补充

<!-- 2026-04-08 CST: 追加 metadata validator 阶段状态。原因：task_plan 当前正文仍停留在 schema registry 阶段。目的：不覆盖历史，但把本轮已完成内容和下一步优先级补充清楚。 -->

- 已完成：`MetadataValidator + MetadataValidationIssue`
- 已完成范围：
  - required 字段校验
  - concept policy 缺失校验
  - disallowed field 校验
  - allowed values 校验
  - value type 校验
  - multi-concept compatibility 校验
- 已完成验证：
  - `cargo test --test metadata_validator_unit -- --nocapture`
  - `cargo test --test metadata_schema_registry_unit --test metadata_validator_unit --test knowledge_repository_unit --test knowledge_ingestion_unit --test knowledge_bundle_unit -- --nocapture`
- 当前下一步应转入：
  - `Schema Versioning`
  - `Migration Contract`
  - repository 级批量 metadata 校验（仅在后续获批时进入）

## 2026-04-08 Schema Versioning 第一阶段补充

<!-- 2026-04-08 CST: 追加 schema versioning 第一阶段状态。原因：本轮已经完成 metadata schema 正式版本契约。目的：把下一步优先级进一步收口到 migration contract。 -->

- 已完成：
  - `MetadataSchema.schema_version`
  - `DEFAULT_METADATA_SCHEMA_VERSION`
  - `MetadataSchema::new_with_version(...)`
  - `MetadataSchema::is_compatible_with(...)`
  - `MetadataSchemaError::InvalidSchemaVersion`
- 已完成验证：
  - `cargo test --test metadata_schema_versioning_unit -- --nocapture`
  - `cargo test --test metadata_schema_registry_unit --test metadata_schema_versioning_unit --test metadata_validator_unit --test knowledge_repository_unit --test knowledge_ingestion_unit --test knowledge_bundle_unit -- --nocapture`
- 当前下一步应转入：
  - `Migration Contract`
  - repository 级批量版本审计
  - 更强 compatibility 规则（仅在后续获批时进入）

## 2026-04-08 Migration Contract 第一阶段补充

<!-- 2026-04-08 CST: 追加 migration contract 第一阶段状态。原因：本轮已经完成字段演进治理对象与最小构建期校验。目的：把下一步优先级收口到 validator 联动或 repository 级审计。 -->

- 已完成：
  - `MetadataFieldDefinition.deprecated`
  - `MetadataFieldDefinition.replaced_by`
  - `MetadataFieldDefinition.aliases`
  - `MetadataFieldDefinition::deprecated()`
  - `MetadataFieldDefinition::with_replaced_by(...)`
  - `MetadataFieldDefinition::with_alias(...)`
- 已完成错误边界：
  - `UnknownReplacementTarget`
  - `SelfReplacementTarget`
  - `DuplicateFieldAlias`
- 已完成验证：
  - `cargo test --test metadata_migration_contract_unit -- --nocapture`
  - `cargo test --test metadata_schema_registry_unit --test metadata_schema_versioning_unit --test metadata_migration_contract_unit --test metadata_validator_unit --test knowledge_repository_unit --test knowledge_ingestion_unit --test knowledge_bundle_unit -- --nocapture`
- 当前下一步应转入：
  - validator 联动
  - repository 级批量演进审计
  - migration executor（仅在后续获批时进入）

## 2026-04-08 下一阶段方案收口

<!-- 2026-04-08 CST: 追加下一阶段候选方案收口。原因：用户已要求把下一步方案一并写入仓库文档并推送。目的：把默认推荐路线固定到计划文件，避免交接时重复讨论范围。 -->

- 候选方案 A：`Validator` 联动
  - 把 `deprecated / aliases / replaced_by` 接入节点级 issue 输出
  - 优先级最高，默认先做
- 候选方案 B：Repository-Level Audit
  - 基于 schema 和 validator 信号做整库批量扫描与清单聚合
- 候选方案 C：Migration Executor
  - 只在后续继续获批时再进入 dry-run / rewrite 执行层
- 当前默认顺序：
  - 先 A
  - 后 B
  - 最后 C

1. Metadata Validator
   - 校验 required 字段、allowed values、字段类型和 concept-field 兼容性。
2. Schema Versioning
   - 增加 metadata schema version、deprecated / replaced_by 和 migration 入口。
3. 更强 registry 治理
   - 如再次获批，再考虑 concept inheritance、field alias 和审计记录。
## 2026-04-10 Validator Linkage 状态补充

<!-- 2026-04-10 CST：补充 validator linkage 阶段状态。原因：task_plan 当前正文停留在 migration contract 之后的候选方案，和已完成代码不一致。目的：把方案 A 已完成、方案 B 成为下一步的状态正式写实。 -->

- 已完成：`Validator Linkage`
- 已完成范围：
  - `MetadataSchema` 正式保留 alias 解析索引
  - `MetadataValidator` 正式消费 `deprecated / replaced_by / aliases`
  - 新增结构化 issue：
    - `AliasFieldUsed`
    - `DeprecatedFieldUsed`
  - alias 不再误伤 required / disallowed 校验
  - deprecated 字段可通过 `replaced_by` 满足 required 字段兼容语义
- 已完成验证：
  - `cargo test --test metadata_validator_unit -- --nocapture`
  - `cargo test --test metadata_schema_registry_unit --test metadata_schema_versioning_unit --test metadata_migration_contract_unit --test metadata_validator_unit --test knowledge_repository_unit --test knowledge_ingestion_unit --test knowledge_bundle_unit -- --nocapture`
- 当前下一步顺序：
  1. `Repository-Level Audit`
  2. `Migration Executor` 仅在再次获批后进入
- 当前非目标再次重申：
  - 不做自动 rewrite
  - 不做 repository 以外的业务层接线
  - 不因为已完成 validator linkage 就回头重开 foundation 架构

## 2026-04-10 Repository-Level Audit 状态补充

<!-- 2026-04-10 CST：补充 repository-level audit 阶段状态。原因：方案B已经完成实现与验证，task_plan 需要从“下一步候选”推进到“当前已完成”。目的：明确当前做到仓库级审计，但仍未进入 migration executor。 -->

- 已完成：`Repository-Level Audit`
- 已完成范围：
  - 新增 `RepositoryMetadataAuditReport`
  - 新增 `RepositoryMetadataAuditIssue`
  - 新增 `RepositoryEvidenceHygieneDiagnostic`
  - 正式复用 `MetadataValidator` 聚合 repository 全节点 issue
  - 正式输出 `issue_type_counts`
  - 正式输出 `concept_issue_counts`
  - 正式输出最小 hygiene diagnostics：
    - `DuplicateEvidenceRef`
    - `WeakLocator`
    - `WeakSourceRef`
- 已完成验证：
  - `cargo test --test repository_metadata_audit_unit -- --nocapture`
  - `cargo test --test repository_metadata_audit_unit --test metadata_validator_unit --test metadata_schema_registry_unit --test metadata_schema_versioning_unit --test metadata_migration_contract_unit --test knowledge_repository_unit --test knowledge_ingestion_unit --test knowledge_bundle_unit -- --nocapture`
- 当前下一步顺序：
  1. 扩细 evidence hygiene diagnostics
  2. `Migration Executor` 仅在再次获批后进入
- 当前非目标再次重申：
  - 不做自动 rewrite
  - 不做自动迁移执行
  - 不把 foundation 审计层接到证券业务层

## 2026-04-10 Task Update: Foundation Evidence Hygiene Diagnostics Expansion
- 已完成：扩展 repository metadata audit 的 hygiene diagnostics，新增 `MissingEvidenceRef`、`DuplicateEvidenceRefWithinNode`、`RepositoryWeakLocatorReason`、`RepositoryWeakSourceRefReason`。
- 已验证：`cargo test --test repository_metadata_audit_unit -- --nocapture`；`cargo test --test repository_metadata_audit_unit --test metadata_validator_unit --test metadata_schema_registry_unit --test metadata_schema_versioning_unit --test metadata_migration_contract_unit --test knowledge_repository_unit --test knowledge_ingestion_unit --test knowledge_bundle_unit -- --nocapture`。
- 下一步候选：1) 细化 locator/source_ref 结构规则；2) 给 audit report 增加 severity/summary；3) 仍然不进入 migration executor。
