# Foundation Metadata Schema Registry Phase 3 Stage 1

> **For Claude/Codex:** 本文档描述 foundation 从“标准能力底座”进入“元数据管理层”的第一步，只包含字段注册和 concept 绑定，不包含 validator、migration 或业务适配。

<!-- 2026-04-08 CST: 新增 foundation metadata schema registry 阶段文档。原因：当前已经不只是做 metadata 存储和过滤，而是开始做 metadata 字段与 concept 的正式治理对象。目的：为后续 validator 和 versioning 提供统一交接入口。 -->

## 1. 目标

- 在 foundation 中补齐 `MetadataSchema` registry。
- 把 metadata 从“字符串 map”提升为“字段定义 + concept 绑定”的正式管理对象。
- 为下一阶段 `validator` 和 `versioning` 打底。

## 2. 本阶段范围

- `MetadataValueType`
- `MetadataFieldDefinition`
  - `key`
  - `value_type`
  - `description`
  - `allowed_values`
- `ConceptMetadataPolicy`
  - `concept_id`
  - `allowed_field_keys`
  - `required_field_keys`
- `MetadataSchema`
  - 字段注册表
  - concept policy 注册表
  - 构建期去重和未知字段引用校验

## 3. 非目标

- 不做 metadata validator。
- 不做 schema versioning / migration。
- 不做 deprecated / replaced_by。
- 不做 concept inheritance。
- 不做业务域适配。

## 4. 已完成测试

- `tests/metadata_schema_registry_unit.rs`
  - `metadata_schema_registers_field_definitions_with_allowed_values`
  - `metadata_schema_binds_allowed_and_required_fields_to_concept`
  - `metadata_schema_rejects_unknown_field_references_in_concept_policy`
- 已通过回归：
  - `cargo test --test metadata_schema_registry_unit -- --nocapture`

## 5. 当前边界

- 现在已经能管理“字段是什么”和“concept 能用什么字段”。
- 现在还不能校验具体节点 metadata 是否满足这些规则。
- 现在还没有 schema 迁移和版本治理。

## 6. 下一阶段建议

1. Metadata Validator
   - 校验 required 字段、allowed values、字段类型和 concept-field 兼容性。
2. Schema Versioning
   - 增加 metadata schema version、deprecated / replaced_by 和 migration 入口。
3. 更强 registry 治理
   - 如再次获批，再考虑 concept inheritance、field alias 和审计记录。
