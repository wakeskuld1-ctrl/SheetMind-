# Foundation Metadata Migration Contract Phase 3 Stage 4

<!-- 2026-04-08 CST: 新增 foundation metadata migration contract 第一阶段文档。原因：registry、validator 和 schema versioning 已完成，下一步需要把字段演进对象正式建模。目的：为后续 validator 联动、repository 级审计和真正的 migration 执行器提供治理契约。 -->

## 1. 阶段目标

- 为 `MetadataFieldDefinition` 增加正式演进治理字段
- 支持 `deprecated / replaced_by / aliases`
- 在 schema 构建期完成最小合法性校验
- 保持范围只在 contract 层，不做 migration 执行

## 2. 本阶段批准范围

- 做：
  - `deprecated: bool`
  - `replaced_by: Option<String>`
  - `aliases: Vec<String>`
  - replacement target 合法性校验
  - alias 冲突校验
- 不做：
  - alias 解析执行层
  - validator 联动
  - repository 级批量迁移
  - 自动重写 metadata

## 3. 当前合法性边界

- `replaced_by` 不能引用未知字段
- `replaced_by` 不能自指
- alias 不能与正式字段 key 冲突
- alias 在 schema 内必须全局唯一

## 4. TDD 结果

- 红测文件：`tests/metadata_migration_contract_unit.rs`
- 已锁定 4 条行为测试：
  - 注册 migration contract
  - unknown replaced_by target
  - self replaced_by target
  - alias conflict with field key

## 5. 验证记录

```powershell
cargo test --test metadata_migration_contract_unit -- --nocapture
cargo test --test metadata_schema_registry_unit --test metadata_schema_versioning_unit --test metadata_migration_contract_unit --test metadata_validator_unit --test knowledge_repository_unit --test knowledge_ingestion_unit --test knowledge_bundle_unit -- --nocapture
```

## 6. 下一阶段建议

<!-- 2026-04-08 CST: 追加下一阶段候选方案。原因：本阶段 contract 已经落地，但下一步有多条可能推进路径。目的：把推荐顺序和边界前置写清，避免后续接手时误跳到业务化或过早做执行器。 -->

1. 方案 A：Validator 联动
   - 目标：让 `deprecated / aliases / replaced_by` 进入节点级 issue 输出
   - 优点：复用现有 `MetadataValidator` 演进路径，最小闭环最清晰
   - 缺点：先解决单节点治理信号，还不覆盖整库审计
   - 推荐度：最高，建议作为下一阶段默认入口
2. 方案 B：Repository-Level Audit
   - 目标：批量识别仍在使用 deprecated 字段、命中 alias 和可迁移字段的节点
   - 优点：能直接产出整库迁移清单，利于治理运营
   - 缺点：需要先引入仓库扫描与聚合输出，改动面比方案 A 更大
3. 方案 C：Migration Executor
   - 目标：引入 dry-run / rewrite 级别的迁移执行接口
   - 优点：为后续自动迁移预留正式执行层
   - 缺点：当前做会偏早，容易引入暂时无人消费的抽象

## 7. 当前推荐路线

- 默认先走方案 A：`Validator` 联动
- 推荐落地顺序：
  - 新增红测，覆盖 deprecated warning、alias 命中提示、可迁移字段提示
  - 最小实现 `MetadataValidationIssue` 扩展
  - 跑定向回归
  - 再决定是否进入方案 B 的整库审计
