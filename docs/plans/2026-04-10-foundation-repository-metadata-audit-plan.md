# Foundation Repository Metadata Audit Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 foundation 新增 repository-level metadata audit，聚合节点级 validator issue，并输出最小 evidence hygiene diagnostics。

**Architecture:** 新增 `repository_metadata_audit` 模块，输入 `KnowledgeRepository + MetadataSchema`，内部复用 `MetadataValidator` 完成节点级校验，再按 issue 类型、concept 和 evidence hygiene 维度聚合成统一报告对象。本轮保持只读审计，不实现 rewrite、migration executor 或业务层接线。

**Tech Stack:** Rust, cargo test, foundation knowledge repository, metadata schema, metadata validator

---

### Task 1: 编写 repository audit 红测

**Files:**
- Create: `D:/Rust/Excel_Skill/tests/repository_metadata_audit_unit.rs`

**Step 1: Write the failing test**

- 新增“聚合多个节点 validator issue”的红测
- 新增“按 issue 类型统计计数”的红测
- 新增“按 concept 统计计数”的红测
- 新增“输出 duplicate evidence / weak locator / weak source_ref 诊断”的红测

**Step 2: Run test to verify it fails**

Run: `cargo test --test repository_metadata_audit_unit -- --nocapture`
Expected: FAIL，因为当前尚无 `repository_metadata_audit` 模块与正式报告对象。

### Task 2: 新增 repository audit 模块最小实现

**Files:**
- Create: `D:/Rust/Excel_Skill/src/ops/foundation/repository_metadata_audit.rs`
- Modify: `D:/Rust/Excel_Skill/src/ops/foundation.rs`

**Step 1: Write the minimal implementation**

- 新增 `RepositoryMetadataAuditReport`
- 新增 `RepositoryMetadataAuditIssue`
- 新增 `RepositoryEvidenceHygieneDiagnostic`
- 新增审计入口类型与 `audit(...)` 方法
- 在 `foundation.rs` 导出新模块

**Step 2: Keep implementation minimal**

- 只做只读聚合
- 不做自动 rewrite
- 不做 migration executor
- 不引入业务层依赖

### Task 3: 接入 validator 聚合与 hygiene diagnostics

**Files:**
- Modify: `D:/Rust/Excel_Skill/src/ops/foundation/repository_metadata_audit.rs`

**Step 1: Aggregate validator issues**

- 遍历 repository 全节点
- 复用 `MetadataValidator::validate_node(...)`
- 生成 issue 明细与 issue 类型计数
- 生成 concept 维度计数

**Step 2: Add minimal evidence hygiene diagnostics**

- 基于 `EvidenceRef` 生成：
  - `DuplicateEvidenceRef`
  - `WeakLocator`
  - `WeakSourceRef`

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

**Step 1: Record the new boundary**

- 写清 repository-level audit 已完成
- 写清 hygiene diagnostics 当前只做到最小版
- 明确当前还没有进入 migration executor

**Step 2: Append task journal entry**

- 按 `task-journal` 模板追加本轮任务记录
