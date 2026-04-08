# Foundation Metadata Schema Versioning Phase 3 Stage 3

<!-- 2026-04-08 CST: 新增 foundation metadata schema versioning 第一阶段文档。原因：registry 和 validator 已完成，下一步必须把元数据治理推进到“正式版本契约”层。目的：为后续 migration contract 和 repository 级批量治理提供版本基础。 -->

## 1. 阶段目标

- 为 `MetadataSchema` 增加正式 `schema_version`
- 提供默认版本与显式版本构造入口
- 提供最小兼容性判断入口
- 保持范围只在 foundation 通用标准能力层

## 2. 本阶段批准范围

- 做：
  - 默认 `schema_version`
  - 显式 `schema_version`
  - 空白版本拒绝
  - 最小兼容性判断
- 不做：
  - migration 执行器
  - deprecated / replaced_by
  - field alias
  - concept inheritance
  - repository 级批量版本迁移

## 3. 设计收口

- 默认版本常量：`DEFAULT_METADATA_SCHEMA_VERSION = "metadata-schema:v1"`
- 兼容入口：
  - `MetadataSchema::new(...)`
  - `MetadataSchema::new_with_version(...)`
  - `MetadataSchema::is_compatible_with(...)`
- 构建错误新增：
  - `MetadataSchemaError::InvalidSchemaVersion`

## 4. 当前兼容语义

- 第一阶段只做精确版本匹配
- `is_compatible_with()` 当前不做跨版本推断
- 这不是 migration 语义，只是 version contract

## 5. TDD 结果

- 红测文件：`tests/metadata_schema_versioning_unit.rs`
- 已锁定 4 条行为测试：
  - 默认版本
  - 显式版本
  - 空白版本拒绝
  - 精确版本兼容性

## 6. 验证记录

```powershell
cargo test --test metadata_schema_versioning_unit -- --nocapture
cargo test --test metadata_schema_registry_unit --test metadata_schema_versioning_unit --test metadata_validator_unit --test knowledge_repository_unit --test knowledge_ingestion_unit --test knowledge_bundle_unit -- --nocapture
```

## 7. 下一阶段建议

1. Migration Contract
   - `deprecated / replaced_by / alias`
2. Repository-Level Validation
   - 批量错误聚合与版本审计
3. Advanced Compatibility
   - 如后续获批，再引入更复杂兼容规则
