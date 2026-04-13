# Foundation Hygiene Grouped Views Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add grouped hygiene governance views to foundation repository metadata audit so downstream AI can route by severity and prioritize nodes for cleanup.

**Architecture:** Keep the existing detail list and summary intact, then add a separate `hygiene_views` layer with `by_severity` and `by_node` groupings. Build both views locally from the same diagnostics slice so the grouped layer stays read-only and deterministic.

**Tech Stack:** Rust, cargo test, foundation repository metadata audit unit tests

---

### Task 1: Add the failing grouped-view assertions

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\repository_metadata_audit_unit.rs`

**Step 1: Write the failing test**

- Add assertions for:
  - `report.hygiene_views.by_severity`
  - `report.hygiene_views.by_node`
  - ordering and count expectations for representative nodes

**Step 2: Run test to verify it fails**

Run: `cargo test --test repository_metadata_audit_unit -- --nocapture`

Expected: compile or assertion failure because `hygiene_views` does not exist yet.

### Task 2: Implement grouped-view models

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\foundation\repository_metadata_audit.rs`

**Step 1: Add model structs**

- Add:
  - `RepositoryEvidenceHygieneViews`
  - `RepositoryEvidenceHygieneSeverityGroup`
  - `RepositoryEvidenceHygieneNodeGroup`
  - `hygiene_views` field on `RepositoryMetadataAuditReport`

**Step 2: Add grouping helpers**

- Add helpers for:
  - severity ordering
  - node highest severity
  - node diagnostic type counting
  - deterministic sorting

**Step 3: Wire grouped-view generation**

- Build `hygiene_views` from `hygiene_diagnostics` inside `audit(...)`

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
- Create: `D:\Rust\Excel_Skill\docs\ai-handoff-2026-04-12-foundation-hygiene-grouped-views.md`
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Add handoff note**

- Record:
  - grouped-view model
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
- `git diff -- src/ops/foundation/repository_metadata_audit.rs tests/repository_metadata_audit_unit.rs docs/plans/2026-04-12-foundation-hygiene-grouped-views-design.md docs/plans/2026-04-12-foundation-hygiene-grouped-views-plan.md docs/ai-handoff-2026-04-12-foundation-hygiene-grouped-views.md .trae/CHANGELOG_TASK.md`

Expected: only grouped-view task files are included for this feature slice.
