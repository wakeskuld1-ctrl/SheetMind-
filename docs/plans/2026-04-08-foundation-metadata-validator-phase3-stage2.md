# Foundation Metadata Validator Phase 3 Stage 2

<!-- 2026-04-08 CST: 新增 foundation metadata validator 阶段文档。原因：metadata schema registry 已完成“字段注册 + concept policy 绑定”，需要把治理定义真正落到节点级执行校验。目的：为后续 schema versioning 接手提供明确边界与完成记录。 -->

## 1. 阶段目标

- 在 `src/ops/foundation/` 内新增节点级 `MetadataValidator`
- 基于 `MetadataSchema` 与 `ConceptMetadataPolicy` 执行结构化 metadata 校验
- 输出可枚举、可测试、可复用的 `MetadataValidationIssue`
- 保持范围只在 foundation 通用标准能力层，不接证券分析主链

## 2. 本阶段批准范围

- 做：
  - required 字段校验
  - concept 缺失 policy 校验
  - concept 不允许字段校验
  - allowed values 校验
  - metadata value type 校验
  - 多 concept 节点兼容性校验
- 不做：
  - repository 全量扫描校验
  - schema migration
  - edge metadata 校验
  - 自动修复
  - 业务化入库规则

## 3. 设计收口

- 模块：`src/ops/foundation/metadata_validator.rs`
- 导出：`src/ops/foundation.rs`
- 核心入口：`MetadataValidator::validate_node(&KnowledgeNode) -> Vec<MetadataValidationIssue>`
- 错误模型：
  - `MissingConceptPolicy`
  - `MissingRequiredField`
  - `DisallowedField`
  - `InvalidAllowedValue`
  - `InvalidValueType`

## 4. 多 Concept 语义

- 字段兼容性按 concept 逐个校验
- 某字段只要被任一 concept 不允许，就要对该 concept 报 `DisallowedField`
- required 字段按并集处理
- concept 没有 policy 时，报 `MissingConceptPolicy`
- `KnowledgeNode.metadata` 仍然保持 `BTreeMap<String, String>`，类型校验基于字符串解析

## 5. TDD 结果

- 红测文件：`tests/metadata_validator_unit.rs`
- 已锁定 5 条行为测试：
  - required 字段缺失
  - concept policy 缺失
  - disallowed field
  - allowed values + type
  - 多 concept 兼容性
- 已按 TDD 执行：
  - 先跑红：缺少 `foundation::metadata_validator`
  - 再补最小实现
  - 最后跑绿并做小范围回归

## 6. 已完成项

- 已新增 `MetadataValidator`
- 已新增 `MetadataValidationIssue`
- 已完成 `foundation` 模块导出
- 已完成节点级结构化校验闭环
- 已完成 foundation 相关小回归

## 7. 验证记录

```powershell
cargo test --test metadata_validator_unit -- --nocapture
cargo test --test metadata_schema_registry_unit --test metadata_validator_unit --test knowledge_repository_unit --test knowledge_ingestion_unit --test knowledge_bundle_unit -- --nocapture
```

## 8. 当前边界

- 这仍然是 foundation 通用标准能力
- 这不是完整知识库治理系统
- 这不是 repository 级批量审计工具
- 这还没有进入 schema versioning / migration

## 9. 下一阶段建议

1. Schema Versioning
   - 为 `MetadataSchema` 增加 version 标识、兼容入口和演进边界
2. Migration Contract
   - 明确 deprecated / replaced_by / alias 的最小治理对象
3. Repository-Level Validation
   - 如后续再获批准，再把节点级 validator 扩展到批量审计
