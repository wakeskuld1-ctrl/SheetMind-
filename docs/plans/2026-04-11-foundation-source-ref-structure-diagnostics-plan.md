# 2026-04-11 Foundation Source Ref Structure Diagnostics Plan

## 执行范围

只在 foundation `repository_metadata_audit` 层扩展 `WeakSourceRef` 结构诊断：

- 先写设计和计划
- 再写红测
- 最后做最小实现和回归

## 步骤

1. 更新 `tests/repository_metadata_audit_unit.rs`
   - 新增 `EntityMissing / ContainsWhitespace / InvalidCharacter / UnknownNamespace` 断言
   - 扩展 sample repository fixture

2. 运行红测
   - `cargo test --test repository_metadata_audit_unit -- --nocapture`

3. 更新 `src/ops/foundation/repository_metadata_audit.rs`
   - 扩展 `RepositoryWeakSourceRefReason`
   - 补充 `weak_source_ref_reason(...)`
   - 保持诊断顺序稳定

4. 运行绿测
   - `cargo test --test repository_metadata_audit_unit -- --nocapture`

5. 运行 foundation 回归
   - `cargo test --test repository_metadata_audit_unit --test metadata_validator_unit --test metadata_schema_registry_unit --test metadata_schema_versioning_unit --test metadata_migration_contract_unit --test knowledge_repository_unit --test knowledge_ingestion_unit --test knowledge_bundle_unit -- --nocapture`

6. 更新交接与任务日志
   - `docs/AI_HANDOFF.md`
   - `task_plan.md`
   - `progress.md`
   - `findings.md`
   - `.trae/CHANGELOG_TASK.md`

## 风险

- `UnknownNamespace` 当前只基于最小允许集合，后续如果规范扩大，需要同步测试。
- `ContainsWhitespace` 在极少数场景可能只是历史兼容写法，因此当前仍定位为弱提示。
- 仓库现有 `dead_code` warnings 不属于本轮处理范围。
