# Foundation Metadata Validator Linkage Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 让 foundation 节点级 `MetadataValidator` 正式消费 `deprecated / replaced_by / aliases` 治理信号，并输出结构化 issue。

**Architecture:** 在 `MetadataSchema` 内保留 alias 到 canonical 字段的解析索引，再由 `MetadataValidator` 统一解析字段 key，复用 canonical 结果完成 required / disallowed / value/type 校验，同时额外输出 alias 与 deprecated 诊断信号。

**Tech Stack:** Rust, cargo test, foundation metadata schema/validator modules

---

### Task 1: 补 alias / deprecated 红测

**Files:**
- Modify: `D:/Rust/Excel_Skill/tests/metadata_validator_unit.rs`

**Step 1: Write the failing test**

- 新增 alias 命中 canonical 字段的红测
- 新增 deprecated 字段 issue 的红测
- 新增 alias 满足 required canonical 字段的红测

**Step 2: Run test to verify it fails**

Run: `cargo test --test metadata_validator_unit -- --nocapture`
Expected: FAIL，因为当前 validator 还不会解析 alias，也不会输出 deprecated / alias issue。

### Task 2: 实现 schema 解析最小能力

**Files:**
- Modify: `D:/Rust/Excel_Skill/src/ops/foundation/metadata_schema.rs`

**Step 1: Write the minimal implementation**

- 给 `MetadataSchema` 保存 alias 索引
- 增加 canonical 解析辅助方法

**Step 2: Keep implementation minimal**

- 不做 rewrite
- 不做 repository audit
- 只提供 validator 所需的只读解析能力

### Task 3: 实现 validator 联动

**Files:**
- Modify: `D:/Rust/Excel_Skill/src/ops/foundation/metadata_validator.rs`

**Step 1: Add new issue types**

- `AliasFieldUsed`
- `DeprecatedFieldUsed`

**Step 2: Use canonical resolution**

- alias 不再误伤 required / disallowed
- deprecated issue 返回 `replaced_by`

### Task 4: 验证

**Files:**
- Test: `D:/Rust/Excel_Skill/tests/metadata_validator_unit.rs`

**Step 1: Run focused test**

Run: `cargo test --test metadata_validator_unit -- --nocapture`
Expected: PASS

**Step 2: Run metadata governance regression**

Run: `cargo test --test metadata_schema_registry_unit --test metadata_schema_versioning_unit --test metadata_migration_contract_unit --test metadata_validator_unit --test knowledge_repository_unit --test knowledge_ingestion_unit --test knowledge_bundle_unit -- --nocapture`
Expected: PASS

### Task 5: 文档与日志

**Files:**
- Modify: `D:/Rust/Excel_Skill/docs/AI_HANDOFF.md`
- Modify: `D:/Rust/Excel_Skill/task_plan.md`
- Modify: `D:/Rust/Excel_Skill/progress.md`
- Modify: `D:/Rust/Excel_Skill/findings.md`
- Modify: `D:/Rust/Excel_Skill/.trae/CHANGELOG_TASK.md`

**Step 1: Record the new boundary**

- 写清楚 validator 现在已经消费 alias / deprecated 信号
- 明确下一步仍然是 repository-level audit，不是 migration executor
