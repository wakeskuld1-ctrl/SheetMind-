# Foundation Evidence Hygiene Diagnostics Expansion Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 扩展 foundation repository audit 的 evidence hygiene diagnostics，补上缺失证据、节点内重复证据以及弱 locator/source_ref 的最小原因分类。

**Architecture:** 在 `repository_metadata_audit` 模块内扩展 diagnostics 枚举和收集逻辑，继续以 `KnowledgeRepository` 为输入做只读分析，不改动 validator 语义，也不引入自动修复。测试继续集中在 `repository_metadata_audit_unit`，用单个样本仓库覆盖新增规则。

**Tech Stack:** Rust, cargo test, foundation repository metadata audit

---

### Task 1: 扩展 hygiene diagnostics 红测

**Files:**
- Modify: `D:/Rust/Excel_Skill/tests/repository_metadata_audit_unit.rs`

**Step 1: Write the failing test**

- 新增缺失 evidence_ref 红测
- 新增节点内重复 evidence_ref 红测
- 新增 `WeakLocator::TooShort` 红测
- 新增 `WeakSourceRef::TooShort` 红测
- 新增 `WeakSourceRef::MissingNamespace` 红测

**Step 2: Run test to verify it fails**

Run: `cargo test --test repository_metadata_audit_unit -- --nocapture`
Expected: FAIL，因为当前 diagnostics 还没有这些新增变体与原因分类。

### Task 2: 扩展 diagnostics 模型

**Files:**
- Modify: `D:/Rust/Excel_Skill/src/ops/foundation/repository_metadata_audit.rs`

**Step 1: Add new enums and variants**

- 新增 `RepositoryWeakLocatorReason`
- 新增 `RepositoryWeakSourceRefReason`
- 新增 `MissingEvidenceRef`
- 新增 `DuplicateEvidenceRefWithinNode`
- 扩展 `WeakLocator / WeakSourceRef`

**Step 2: Keep implementation minimal**

- 不做 severity
- 不做自动修复
- 不做复杂正则校验

### Task 3: 扩展收集逻辑

**Files:**
- Modify: `D:/Rust/Excel_Skill/src/ops/foundation/repository_metadata_audit.rs`

**Step 1: Implement missing and in-node duplicate detection**

- 对空 `evidence_refs` 输出 `MissingEvidenceRef`
- 对节点内重复 `source_ref + locator` 输出 `DuplicateEvidenceRefWithinNode`

**Step 2: Implement weak-quality reasons**

- `locator.trim().is_empty()` -> `Blank`
- `locator.trim().len() < 3` -> `TooShort`
- `source_ref.trim().is_empty()` -> `Blank`
- `source_ref.trim().len() < 4` -> `TooShort`
- `source_ref` 不含 `:` -> `MissingNamespace`

### Task 4: 验证红转绿

**Files:**
- Test: `D:/Rust/Excel_Skill/tests/repository_metadata_audit_unit.rs`

**Step 1: Run focused test**

Run: `cargo test --test repository_metadata_audit_unit -- --nocapture`
Expected: PASS

**Step 2: Run metadata governance regression**

Run: `cargo test --test repository_metadata_audit_unit --test metadata_validator_unit --test metadata_schema_registry_unit --test metadata_schema_versioning_unit --test metadata_migration_contract_unit --test knowledge_repository_unit --test knowledge_ingestion_unit --test knowledge_bundle_unit -- --nocapture`
Expected: PASS

### Task 5: 文档与任务日志收口

**Files:**
- Modify: `D:/Rust/Excel_Skill/docs/AI_HANDOFF.md`
- Modify: `D:/Rust/Excel_Skill/task_plan.md`
- Modify: `D:/Rust/Excel_Skill/progress.md`
- Modify: `D:/Rust/Excel_Skill/findings.md`
- Modify: `D:/Rust/Excel_Skill/.trae/CHANGELOG_TASK.md`

**Step 1: Record the expanded hygiene boundary**

- 写清 evidence hygiene diagnostics 已从 3 项扩到更细粒度规则
- 写清当前仍然不做自动修复与 migration executor

**Step 2: Append task journal entry**

- 按 `task-journal` 模板追加本轮任务记录
