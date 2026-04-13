# Foundation Hygiene Export DTO V1 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a stable versioned export DTO for `RepositoryMetadataAudit` so downstream AI and upper-layer callers can consume foundation hygiene output without coupling to internal report structures.

**Architecture:** Keep `RepositoryMetadataAuditReport` as the internal model, add a `v1` export DTO layer plus one-way conversion from report to DTO, and lock the export contract with focused TDD coverage using the existing sample, large-stability, and boundary fixtures.

**Tech Stack:** Rust, cargo test, foundation repository metadata audit unit tests

---

### Task 1: Add the failing DTO export test on the base fixture

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\repository_metadata_audit_unit.rs`

**Step 1: Write the failing test**

- Add one new test that:
  - builds `sample_repository()`
  - runs `RepositoryMetadataAudit::new(&schema).audit(&repository)`
  - converts the report into `RepositoryMetadataAuditExportDtoV1`
  - asserts stable exported fields for:
    - totals
    - summary severity counts
    - grouped views
    - reason views

**Step 2: Run test to verify it fails**

Run: `cargo test --test repository_metadata_audit_unit dto_export_v1_mirrors_foundation_hygiene_contract -- --nocapture`

Expected: FAIL because the export DTO and conversion layer do not exist yet.

### Task 2: Add failing export coverage for stability and boundary semantics

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\repository_metadata_audit_unit.rs`

**Step 1: Write the failing tests**

- Add one test over `large_stability_repository()` to lock exported ordering.
- Add one test over `repeated_reason_boundary_repository()` to lock exported count semantics.

**Step 2: Run tests to verify they fail**

Run: `cargo test --test repository_metadata_audit_unit -- --nocapture`

Expected: FAIL because export conversion does not yet preserve the required contract.

### Task 3: Implement the minimal DTO model and conversion

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\foundation\repository_metadata_audit.rs`

**Step 1: Add versioned DTO structs**

- Add:
  - `RepositoryMetadataAuditExportDtoV1`
  - child DTO structs for summary, grouped views, reason views, issue detail, and hygiene detail as needed

**Step 2: Add stable conversion entry point**

- Implement a one-way conversion such as:
  - `RepositoryMetadataAuditExportDtoV1::from_report(&RepositoryMetadataAuditReport) -> Self`

**Step 3: Keep the mapping minimal**

- Reuse current string keys and ordering
- Do not change the existing internal report contract

### Task 4: Run focused tests to green

**Files:**
- Test: `D:\Rust\Excel_Skill\tests\repository_metadata_audit_unit.rs`

**Step 1: Run focused DTO export tests**

Run:
- `cargo test --test repository_metadata_audit_unit dto_export_v1_mirrors_foundation_hygiene_contract -- --nocapture`
- `cargo test --test repository_metadata_audit_unit dto_export_v1_keeps_large_repository_ordering_stable -- --nocapture`
- `cargo test --test repository_metadata_audit_unit dto_export_v1_preserves_reason_count_semantics_on_repeated_reasons -- --nocapture`

Expected: PASS

### Task 5: Run the foundation regression set

**Files:**
- Test:
  - `D:\Rust\Excel_Skill\tests\metadata_validator_unit.rs`
  - `D:\Rust\Excel_Skill\tests\metadata_registry_unit.rs`
  - `D:\Rust\Excel_Skill\tests\metadata_scope_resolver_unit.rs`
  - `D:\Rust\Excel_Skill\tests\knowledge_record_unit.rs`
  - `D:\Rust\Excel_Skill\tests\knowledge_graph_store_unit.rs`
  - `D:\Rust\Excel_Skill\tests\ontology_schema_unit.rs`
  - `D:\Rust\Excel_Skill\tests\ontology_store_unit.rs`
  - `D:\Rust\Excel_Skill\tests\repository_metadata_audit_unit.rs`

**Step 1: Run regression commands**

Run:
- `cargo test --test repository_metadata_audit_unit -- --nocapture`
- `cargo test --test metadata_validator_unit -- --nocapture`
- `cargo test --test metadata_registry_unit -- --nocapture`
- `cargo test --test metadata_scope_resolver_unit -- --nocapture`
- `cargo test --test knowledge_record_unit -- --nocapture`
- `cargo test --test knowledge_graph_store_unit -- --nocapture`
- `cargo test --test ontology_schema_unit -- --nocapture`
- `cargo test --test ontology_store_unit -- --nocapture`

Expected: PASS

### Task 6: Update handoff and task journal

**Files:**
- Create: `D:\Rust\Excel_Skill\docs\ai-handoff-2026-04-12-foundation-hygiene-export-dto-v1.md`
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Add handoff note**

- Record:
  - the new `v1` export contract scope
  - which semantics were intentionally mirrored
  - which internal model remained unchanged
  - verification commands

**Step 2: Append task journal entry**

- Append this slice with the standard `task-journal` structure.

### Task 7: Final verification before completion

**Files:**
- Review only

**Step 1: Re-check diff**

Run:
- `git diff -- src/ops/foundation/repository_metadata_audit.rs tests/repository_metadata_audit_unit.rs docs/plans/2026-04-12-foundation-hygiene-export-dto-v1-design.md docs/plans/2026-04-12-foundation-hygiene-export-dto-v1-plan.md docs/ai-handoff-2026-04-12-foundation-hygiene-export-dto-v1.md .trae/CHANGELOG_TASK.md`

Expected: only DTO-export slice files are included for this task.
