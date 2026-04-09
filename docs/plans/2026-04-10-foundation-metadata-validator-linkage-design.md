# Foundation Metadata Validator Linkage Design

<!-- 2026-04-10 CST: 新增 validator 联动设计文档。原因：migration contract 第一阶段已经把 deprecated / replaced_by / aliases 纳入 schema 治理层，但节点级 validator 还没有消费这些治理信号。目的：把本轮只做 foundation 节点级联动、不做 repository audit / migration executor 的边界固定下来，避免后续实现越界。 -->

## 1. 目标

- 只在 `src/ops/foundation/` 内补齐 `MetadataValidator` 对治理信号的消费
- 把 `deprecated / replaced_by / aliases` 接入节点级结构化 issue 输出
- 保持范围停留在单节点校验，不扩到 repository 批量审计，不做自动修复

## 2. 本轮范围

- 做：
  - alias 字段命中 canonical 字段时，输出结构化 alias issue
  - deprecated 字段被使用时，输出结构化 deprecated issue
  - deprecated 字段存在 `replaced_by` 时，在 issue 中显式带出推荐替代字段
  - required / disallowed / allowed values / value type 校验能够消费 canonical 字段解析结果
- 不做：
  - repository 级批量扫描
  - metadata 自动重写
  - migration executor
  - foundation 与证券业务链联动

## 3. 设计选择

### 方案 A

- 在 `MetadataSchema` 中增加 alias 解析索引
- 在 `MetadataValidator` 中做 canonical 字段解析
- 在 validator 输出中新增：
  - `AliasFieldUsed`
  - `DeprecatedFieldUsed`

这是本轮采用的方案，因为它只扩展 foundation validator 的只读能力，不引入重写行为。

### 不采用的方案

- 直接把 alias 自动改写成 canonical
  - 原因：这会越过 validator 边界，滑向 migration executor
- 直接做 repository-level audit
  - 原因：这会扩大到下一阶段，不符合当前批准范围

## 4. 行为约定

- alias 命中 canonical 字段时：
  - validator 不应再把该字段误判成 `DisallowedField`
  - required 字段检查也应把 alias 视为满足 canonical 字段存在
  - 但仍应输出 `AliasFieldUsed`
- deprecated 字段被使用时：
  - 输出 `DeprecatedFieldUsed`
  - 如果 schema 有 `replaced_by`，则在 issue 中返回推荐字段
- alias 若映射到 deprecated 字段：
  - 同时输出 `AliasFieldUsed`
  - 同时输出 `DeprecatedFieldUsed`

## 5. TDD 计划

- 先补 `tests/metadata_validator_unit.rs` 红测
  - alias 字段被 canonical policy 接受，但仍输出 alias issue
  - deprecated 字段输出 deprecated issue 和 replacement 建议
  - alias 命中 required canonical 字段时，不再误报缺失 required
- 红测确认失败后，再补最小实现

## 6. 验证范围

```powershell
cargo test --test metadata_validator_unit -- --nocapture
cargo test --test metadata_schema_registry_unit --test metadata_schema_versioning_unit --test metadata_migration_contract_unit --test metadata_validator_unit --test knowledge_repository_unit --test knowledge_ingestion_unit --test knowledge_bundle_unit -- --nocapture
```

## 7. 下一步边界

- 本轮完成后，下一阶段才适合进入 `Repository-Level Audit`
- 不要把本轮 validator 联动误当成 migration executor 的开始
