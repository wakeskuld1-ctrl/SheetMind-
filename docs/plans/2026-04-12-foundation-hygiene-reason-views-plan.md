# Foundation Hygiene Reason Views Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add weak-reason grouped hygiene views to foundation repository metadata audit so downstream AI can route remediation by reason as well as by severity and node.

**Architecture:** Keep the existing diagnostic, summary, and grouped-view layers intact, then add a separate `hygiene_reason_views` layer built from the same diagnostics slice. The new layer stays read-only, deterministic, and limited to weak-locator and weak-source-ref reasons.

**Tech Stack:** Rust, cargo test, foundation repository metadata audit unit tests

---

### Task 1: Add the failing reason-view assertions

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\repository_metadata_audit_unit.rs`

**Step 1: Write the failing test**

- Add assertions for:
  - `report.hygiene_reason_views.weak_locator_by_reason`
  - `report.hygiene_reason_views.weak_source_ref_by_reason`
  - ordering, severity, count, and representative node list expectations

**Step 2: Run test to verify it fails**

Run: `cargo test --test repository_metadata_audit_unit -- --nocapture`

Expected: compile or assertion failure because `hygiene_reason_views` does not exist yet.

### Task 2: Implement reason-view models

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\foundation\repository_metadata_audit.rs`

**Step 1: Add model structs**

- Add:
  - `RepositoryEvidenceHygieneReasonViews`
  - `RepositoryWeakLocatorReasonGroup`
  - `RepositoryWeakSourceRefReasonGroup`
  - `hygiene_reason_views` field on `RepositoryMetadataAuditReport`

**Step 2: Add grouping helpers**

- Add helpers for:
  - weak locator grouping by reason
  - weak source ref grouping by reason
  - deterministic node-id aggregation
  - stable reason sorting

**Step 3: Wire reason-view generation**

- Build `hygiene_reason_views` from `hygiene_diagnostics` inside `audit(...)`

### Task 3: Verify green

**Files:**
- Test: `D:\Rust\Excel_Skill\tests\repository_metadata_audit_unit.rs`

**Step 1: Run focused test**

Run: `cargo test --test repository_metadata_audit_unit -- --nocapture`

Expected: PASS

### Task 4: Run foundation regression

**Files:**
- Test:
  - `D:\Rust\Excel_Skill\tests\metadata_validator_unit.rs`
  - `D:\Rust\Excel_Skill\tests\metadata_registry_unit.rs`
  - `D:\Rust\Excel_Skill\tests\metadata_scope_resolver_unit.rs`
  - `D:\Rust\Excel_Skill\tests\knowledge_record_unit.rs`
  - `D:\Rust\Excel_Skill\tests\knowledge_graph_store_unit.rs`
  - `D:\Rust\Excel_Skill\tests\ontology_schema_unit.rs`
  - `D:\Rust\Excel_Skill\tests\ontology_store_unit.rs`

**Step 1: Run regression commands**

Run:
- `cargo test --test metadata_validator_unit -- --nocapture`
- `cargo test --test metadata_registry_unit -- --nocapture`
- `cargo test --test metadata_scope_resolver_unit -- --nocapture`
- `cargo test --test knowledge_record_unit -- --nocapture`
- `cargo test --test knowledge_graph_store_unit -- --nocapture`
- `cargo test --test ontology_schema_unit -- --nocapture`
- `cargo test --test ontology_store_unit -- --nocapture`

Expected: PASS

### Task 5: Update handoff and task journal

**Files:**
- Create: `D:\Rust\Excel_Skill\docs\ai-handoff-2026-04-12-foundation-hygiene-reason-views.md`
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Add handoff note**

- Record:
  - reason-view model
  - sorting rules
  - intended downstream usage
  - verification commands

**Step 2: Append task journal entry**

- Append this round with the fixed task-journal structure.

### Task 6: Final verification before completion

**Files:**
- Review only

**Step 1: Re-check diff**

Run:
- `git diff -- src/ops/foundation/repository_metadata_audit.rs tests/repository_metadata_audit_unit.rs docs/plans/2026-04-12-foundation-hygiene-reason-views-design.md docs/plans/2026-04-12-foundation-hygiene-reason-views-plan.md docs/ai-handoff-2026-04-12-foundation-hygiene-reason-views.md .trae/CHANGELOG_TASK.md`

Expected: only reason-view task files are included for this feature slice.
